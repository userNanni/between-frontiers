//! The strategic AI's crisis and war, in shadow (`docs/ai.pt-BR.md`, slice 4; switch
//! `shadow_ai`, off). Every month, each country without a player weighs whether it would demand
//! land of another, the target whether it would yield, and the demander whether it would declare;
//! nothing is issued. The world stays as it is (no command, no state in the save, no hash); a
//! record grows outside it, read by the tests and the bench. The thresholds are DESIGN VALUES in
//! `data/diplomacy/shadow.ron`.
//!
//! What the shadow does not cover, by design: demands that are not of land (a sphere of
//! influence, a regime, a colony, a payment); and the chain where one war changes the next, as
//! no war happens. The claims (`claims.ron`) cover 1925-1939; after that a motive comes only
//! from land lost in the game, which the shadow never sees lost, so the numbers from 1950 on say
//! little.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use laplace_schelling::schedule::{Access, Cadence, System, TickCtx};
use serde::{Deserialize, Serialize};

use crate::econ::catalogue::Catalogue;
use crate::game::GameWorld;
use crate::relations::{Relations, RelationsSetup};

/// `data/diplomacy/shadow.ron`. Every value is a DESIGN VALUE.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowRules {
    /// A stake (people of 1925 held by the other, or the own people in its metropole) worth a
    /// demand, per mille of the own people.
    pub stake_min: u32,
    /// The share of the target's people an open claim is taken to ask, per mille.
    pub claim_share: u32,
    /// The hostility from which a country would demand.
    pub hostility_min: i32,
    /// The demander's reachable power over the target side's (the target and who would join it),
    /// per mille, to demand and to declare.
    pub power_ratio: u32,
    pub war_ratio: u32,
    /// The most threat from third countries (per mille of the own power) with which a country
    /// would still open a crisis.
    pub threat_max: u32,
    /// Years a war is expected to last at an even power ratio; shorter as the ratio grows.
    pub war_years: u32,
    /// The value of the stake, per mille of the own people, against the cost of the forces in a
    /// year over GDP, per mille, times the years: demand while value × `value_weight` / 1000 is
    /// at least the cost.
    pub value_weight: u32,
    /// Months before the same pair can open another crisis.
    pub cooldown_months: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Motive {
    /// An open claim of `claims.ron`.
    Claim,
    /// Its people in the other's metropole.
    Minority,
    /// Its land of 1925 held by the other.
    LostLand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Demand,
    Yield,
    Refuse,
    Declare,
    BackDown,
}

/// One line of the record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowEvent {
    /// The variant of the thresholds (0: the data's; then the sensitivity variants).
    pub variant: u8,
    pub year: i32,
    pub month: u8,
    pub step: Step,
    pub by: String,
    pub target: String,
    pub motive: Motive,
    /// The stake (per mille of the demander's people), the hostility, the power ratio (per
    /// mille), the expected cost (per mille of GDP over the war).
    pub stake: u32,
    pub hostility: i32,
    pub ratio: u32,
    pub cost: u32,
    /// Who the agreements would call in, on each side.
    pub join_by: Vec<String>,
    pub join_target: Vec<String>,
}

/// The record and the shadow's own state, outside the world.
#[derive(Debug, Default)]
pub struct ShadowLog {
    pub events: Vec<ShadowEvent>,
    /// (variant, demander, target) -> the month (year × 12 + month) the pair can demand again.
    cooldown: BTreeMap<(u8, u16, u16), i32>,
}

/// What the shadow needs, made at load (`GameData::rules_with`).
#[derive(Debug)]
pub struct ShadowSetup {
    pub rules: ShadowRules,
    /// The variants measured in the same run: (label, rules). The first is the data's.
    pub variants: Vec<(String, ShadowRules)>,
    pub relations: RelationsSetup,
    /// The crisis rules and the map's neighbours, for W3's force ratio.
    pub crisis: crate::war::CrisisSetup,
    pub military: Arc<crate::war::Military>,
    pub cat: Arc<Catalogue>,
    pub table: Option<Arc<crate::demography::communities::CommunityTable>>,
    pub log: Arc<Mutex<ShadowLog>>,
}

impl ShadowSetup {
    /// The data's thresholds and four variants: `hostility_min` and `power_ratio` 20 % above and
    /// below.
    pub fn variants_of(rules: &ShadowRules) -> Vec<(String, ShadowRules)> {
        let scale = |v: i64, pm: i64| v * pm / 1000;
        let mut out = vec![("data".to_owned(), rules.clone())];
        for (label, h, p) in [
            ("hostility +20%", 1200, 1000),
            ("hostility -20%", 800, 1000),
            ("power +20%", 1000, 1200),
            ("power -20%", 1000, 800),
        ] {
            let mut r = rules.clone();
            r.hostility_min = scale(i64::from(r.hostility_min), h) as i32;
            r.power_ratio = scale(i64::from(r.power_ratio), p) as u32;
            r.war_ratio = scale(i64::from(r.war_ratio), p) as u32;
            out.push((label.to_owned(), r));
        }
        out
    }
}

/// The monthly system: it reads the world and writes only the record.
pub struct ShadowMonth {
    pub setup: Arc<ShadowSetup>,
}

fn tag(world: &GameWorld, c: u16) -> String {
    world
        .countries
        .iter()
        .nth(usize::from(c))
        .map(|(_, x)| x.tag.clone())
        .unwrap_or_default()
}

/// What the variant `v` would do this month.
fn month(
    s: &ShadowSetup,
    v: u8,
    r: &ShadowRules,
    rel: &Relations,
    world: &GameWorld,
    cooldown: &BTreeMap<(u8, u16, u16), i32>,
    (year, month): (i32, u8),
) -> Vec<ShadowEvent> {
    let now = year * 12 + i32::from(month);
    let n = world.countries.len() as u16;
    let player = world
        .countries
        .iter()
        .position(|(id, _)| Some(id) == world.player)
        .map(|i| i as u16);
    let cinc: Vec<u32> = (0..n)
        .map(|c| {
            rel.countries
                .iter()
                .find(|x| x.country == c)
                .map_or(0, |x| x.cinc)
        })
        .collect();
    let threat = |c: u16| -> u32 {
        rel.countries
            .iter()
            .find(|x| x.country == c)
            .map_or(0, |x| x.threat)
    };
    let great = |c: u16| cinc[usize::from(c)] >= s.relations.rules.great_power;
    let reach = |a: u16, b: u16| -> u32 {
        if rel.near.contains(&(a.min(b), a.max(b))) || (great(a) && great(b)) {
            1000
        } else {
            s.relations.rules.reach_far
        }
    };
    let mut out = Vec::new();
    for a in 0..n {
        if Some(a) == player {
            continue;
        }
        // The best demand of `a` this month, if any: (value over cost, the event).
        let mut best: Option<(i64, ShadowEvent, u16)> = None;
        for b in 0..n {
            if b == a || cooldown.get(&(v, a, b)).is_some_and(|&m| now < m) {
                continue;
            }
            if world.war.conflicts.at_war(a, b) {
                continue;
            }
            let h = rel.hostility(a, b);
            if h < r.hostility_min {
                continue;
            }
            let (taken, abroad) = rel.stakes.get(&(a, b)).copied().unwrap_or((0, 0));
            let claim = s.relations.claims.iter().any(|&(x, y, from, until)| {
                x == a && y == b && from <= year && until.is_none_or(|u| year <= u)
            });
            let (motive, stake) = if claim {
                (Motive::Claim, r.claim_share.max(taken).max(abroad))
            } else if taken >= abroad && taken >= r.stake_min {
                (Motive::LostLand, taken)
            } else if abroad >= r.stake_min {
                (Motive::Minority, abroad.min(1000))
            } else {
                continue;
            };
            let (ta, tb) = (tag(world, a), tag(world, b));
            let joins = crate::war::third::who_would_join(world, &ta, &tb);
            let side = |want: crate::war::third::CallSide| -> Vec<String> {
                joins
                    .iter()
                    .filter(|e| !e.barred && !e.suspended && e.side == Some(want))
                    .map(|e| e.party.clone())
                    .collect()
            };
            let join_by = side(crate::war::third::CallSide::Attacker);
            let join_target = side(crate::war::third::CallSide::Defender);
            // The military power that reaches: the combat power of each side (W3's, as the
            // front reads it) with those who would join, the demander's by the reach the
            // threat uses (whole to a neighbour or between two powers, `reach_far` otherwise).
            let power = |list: &[String], first: u16| -> i128 {
                let one = |c: usize| {
                    i128::from(
                        crate::war::crisis::combat_power(&s.military, &world.war, c)
                            .raw()
                            .max(0),
                    )
                };
                one(usize::from(first))
                    + list
                        .iter()
                        .filter_map(|t| world.countries.iter().position(|(_, c)| c.tag == *t))
                        .map(one)
                        .sum::<i128>()
            };
            let mine = power(&join_by, a) * i128::from(reach(a, b)) / 1000;
            let theirs = power(&join_target, b).max(1);
            let ratio = (mine * 1000 / theirs).min(1_000_000) as u32;
            if ratio < r.power_ratio {
                continue;
            }
            // The threat from the others: the whole threat less what `b` adds.
            let from_b = u64::from(h.max(0) as u32) * u64::from(reach(a, b)) / 1000
                * u64::from(cinc[usize::from(b)])
                / u64::from(cinc[usize::from(a)].max(1));
            if u64::from(threat(a)).saturating_sub(from_b) > u64::from(r.threat_max) {
                continue;
            }
            // The expected cost: the forces' yearly cost over GDP, over the years a war takes
            // at this ratio.
            let gdp = world.econ.markets[usize::from(a)].gdp_last_year;
            let forces = crate::war::third::forces_cost_year(world, a);
            let per_year = if gdp.is_positive() {
                (forces.mul_ratio(1000, 1) / gdp).to_int_floor().max(0) as u64
            } else {
                0
            };
            let cost = (per_year * u64::from(r.war_years) * 1000 / u64::from(ratio.max(1)))
                .min(1_000_000) as u32;
            let value = u64::from(stake) * u64::from(r.value_weight) / 1000;
            if value < u64::from(cost) {
                continue;
            }
            let score = value as i64 - i64::from(cost);
            if best.as_ref().is_none_or(|(x, _, _)| score > *x) {
                best = Some((
                    score,
                    ShadowEvent {
                        variant: v,
                        year,
                        month,
                        step: Step::Demand,
                        by: ta,
                        target: tb,
                        motive,
                        stake,
                        hostility: h,
                        ratio,
                        cost,
                        join_by,
                        join_target,
                    },
                    b,
                ));
            }
        }
        let Some((_, demand, b)) = best else { continue };
        // The target weighs it as the AI does today (W3): the land forces against the share asked.
        let land_ratio = crate::war::crisis::force_ratio(&s.crisis, &s.military, world, a, b);
        let share = i64::from(match demand.motive {
            Motive::Claim => r.claim_share,
            _ => demand.stake.min(1000) * u32::try_from(own_over(world, a, b)).unwrap_or(0) / 1000,
        });
        let w = crate::war::crisis::weigh(&s.crisis.rules, land_ratio, share, false);
        let mut next = demand.clone();
        out.push(demand.clone());
        if w.yields {
            next.step = Step::Yield;
            out.push(next);
        } else {
            next.step = Step::Refuse;
            out.push(next.clone());
            let declares = demand.ratio >= r.war_ratio && !world.war.nuclear.blocked(a, b);
            next.step = if declares {
                Step::Declare
            } else {
                Step::BackDown
            };
            out.push(next);
        }
    }
    out
}

/// The demander's people over the target's, per mille: a stake in the demander's people asks
/// that share of the target's.
fn own_over(world: &GameWorld, a: u16, b: u16) -> i64 {
    let people = |c: u16| -> i64 {
        world
            .econ
            .regions
            .iter()
            .filter(|r| r.market == c && !r.colony)
            .map(|r| r.population())
            .sum()
    };
    (people(a) * 1000 / people(b).max(1)).min(1000)
}

impl System<GameWorld> for ShadowMonth {
    type Changes = Vec<ShadowEvent>;

    fn name(&self) -> &'static str {
        "shadow-ai"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 3 }
    }

    fn access(&self) -> Access {
        Access::new()
            .read(crate::econ::week::ECON)
            .read(crate::war::WAR)
            .read(crate::treaty_system::TREATIES)
            .read(crate::polity::POLITY)
            .read(crate::demography::DEMOGRAPHY)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> Vec<ShadowEvent> {
        let s = &*self.setup;
        let rel = crate::relations::measure(
            &s.relations,
            &s.cat,
            world,
            s.table.as_deref(),
            ctx.date.year,
        );
        let cooldown = s.log.lock().map(|l| l.cooldown.clone()).unwrap_or_default();
        s.variants
            .iter()
            .enumerate()
            .flat_map(|(v, (_, r))| {
                month(
                    s,
                    v as u8,
                    r,
                    &rel,
                    world,
                    &cooldown,
                    (ctx.date.year, ctx.date.month),
                )
            })
            .collect()
    }

    fn apply(&self, ctx: &TickCtx, changes: Vec<ShadowEvent>, world: &mut GameWorld) {
        let s = &*self.setup;
        let now = ctx.date.year * 12 + i32::from(ctx.date.month);
        let index = |t: &str| world.countries.iter().position(|(_, c)| c.tag == t);
        if let Ok(mut log) = s.log.lock() {
            for e in &changes {
                if e.step == Step::Demand
                    && let (Some(a), Some(b)) = (index(&e.by), index(&e.target))
                {
                    let months = s.variants[usize::from(e.variant)].1.cooldown_months as i32;
                    log.cooldown
                        .insert((e.variant, a as u16, b as u16), now + months);
                }
            }
            log.events.extend(changes);
        }
    }
}
