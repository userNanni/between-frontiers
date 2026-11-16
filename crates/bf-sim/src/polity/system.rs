//! The politics system: enacts the laws asked for, measures the clout of the interest groups,
//! moves the movements and lets the countries without a player change their laws under
//! pressure. Rules and constants in `docs/politics.pt-BR.md` ("Dinâmica").

use std::sync::Arc;

use laplace_schelling::schedule::{Access, Cadence, System, TickCtx};
use laplace_schelling::{Date, Fixed};

use super::measure::{self, Facts};
use super::{
    Enactor, LawChangeRecord, MILLE, MovementState, POLITY, PoliticsState, TaxContext, stances,
};
use crate::econ::catalogue::Catalogue;
use crate::econ::prof::P;
use crate::econ::state::LawEffects;
use crate::econ::week::ECON;
use crate::econ::{milli, public};
use crate::game::GameWorld;
use crate::politics::{Among, GoalRef, LawChange, Measure, Politics, Scope, effect_index, margin};

/// A movement must have mobilized this share of the population it is measured against (per
/// mille: support × recruits ÷ people) before a government yields to it.
pub const MIN_MOBILIZED: i32 = 40;
/// Weight of the mobilized share against the clout of the interest groups: a movement that has
/// mobilized 10 % weighs as much as groups with 40 % of the clout.
pub const PUSH_WEIGHT: i32 = 4;
/// Net backing (per mille of clout, after the push) a change needs at full legitimacy; the bar
/// falls in proportion as legitimacy does.
pub const THRESHOLD: i32 = 300;
/// Net backing (per mille of clout) at which the interest groups change a law with no
/// movement asking for it.
pub const GROUPS_THRESHOLD: i32 = 700;
/// A stance of this many weight × per mille counts in full for or against a law.
pub const STANCE_UNIT: i64 = 500;
/// Legitimacy lost per mille of the population mobilized by movements still unanswered.
pub const LEGITIMACY_LOSS: i32 = 3;
/// Years after a change of government during which the groups that count rewrite laws with no
/// movement asking.
pub const REGIME_YEARS: i32 = 5;
/// Years a law group rests after a change before the country changes it again by itself.
pub const COOLDOWN_YEARS: i32 = 5;

/// Effect indices the economy reads, resolved at load.
struct Wired {
    within: usize,
    between: usize,
    caps: [usize; 3],
    collection: usize,
    health: usize,
    education: usize,
    enrolment: usize,
    rent: usize,
    ownership: usize,
    profit_to_treasury: usize,
    tariff: usize,
    frontier: usize,
    foreign_investment: usize,
    franchise: usize,
    repression: usize,
}

impl Wired {
    fn new() -> Self {
        let e = |k: &str| effect_index(k).unwrap_or_else(|| panic!("effect {k}"));
        Self {
            within: e("migration_within_state"),
            between: e("migration_between_states"),
            caps: [e("tax_cap_lower"), e("tax_cap_middle"), e("tax_cap_upper")],
            collection: e("tax_collection"),
            health: e("health_state_share"),
            education: e("education_state_share"),
            enrolment: e("school_enrolment_target"),
            rent: e("landowner_rent"),
            ownership: e("state_ownership"),
            profit_to_treasury: e("state_profit_to_treasury"),
            tariff: e("tariff"),
            frontier: e("frontier_access"),
            foreign_investment: e("foreign_investment"),
            franchise: e("franchise"),
            repression: e("repression"),
        }
    }
}

pub struct PoliticsSystem {
    pub politics: Arc<Politics>,
    cat: Arc<Catalogue>,
    /// Per mille of each profession in each interest group.
    shares: Vec<[u16; P]>,
    /// Per mille of each profession each movement kind can recruit.
    recruits: Vec<[u16; P]>,
    /// School and hospital building kinds.
    kinds: (Option<usize>, Option<usize>),
    /// Goods of the import-share indicator (consumer and industrial goods that trade).
    manufactures: Vec<usize>,
    /// The `schooling` and `hospital_care` goods.
    public_goods: [Option<u16>; 2],
    wired: Wired,
    /// The law group of the forms of government.
    government: Option<usize>,
}

impl PoliticsSystem {
    pub fn new(politics: Arc<Politics>, cat: Arc<Catalogue>) -> Self {
        let row = |list: &[(u8, u16)], all: bool| {
            let mut r = [if all { MILLE } else { 0 }; P];
            for &(pi, s) in list {
                r[usize::from(pi)] = s;
            }
            r
        };
        let shares = politics
            .interest_groups
            .iter()
            // Groups of one community recruit nobody until communities live in the pops.
            .map(|g| {
                row(
                    if g.community.is_none() {
                        &g.professions
                    } else {
                        &[]
                    },
                    false,
                )
            })
            .collect();
        let recruits = politics
            .movements
            .iter()
            .map(|m| row(&m.professions, m.professions.is_empty()))
            .collect();
        let manufactures = cat
            .goods
            .iter()
            .enumerate()
            .filter(|(_, g)| g.tradable && matches!(g.category.as_str(), "consumer" | "industrial"))
            .map(|(i, _)| i)
            .collect();
        let good = |k: &str| cat.good(k);
        Self {
            kinds: (cat.building("school"), cat.building("hospital")),
            public_goods: [good("schooling"), good("hospital_care")],
            manufactures,
            shares,
            recruits,
            wired: Wired::new(),
            government: politics.group(crate::politics::GOVERNMENT_GROUP),
            politics,
            cat,
        }
    }

    /// Clout of each interest group, per mille. Of the political weight, the franchise share
    /// (per mille of adults with a vote) goes by headcount and the rest by income: with no
    /// vote only money speaks; with universal suffrage, heads and money weigh half and half.
    fn clout(&self, f: &Facts, franchise: i32) -> Vec<u16> {
        let franchise = i128::from(franchise.clamp(0, 1000));
        let ts: i128 = f.size.iter().map(|&s| i128::from(s)).sum();
        let ti: i128 = f.income.iter().sum();
        self.shares
            .iter()
            .map(|row| {
                let part = |v: &dyn Fn(usize) -> i128, total: i128| -> i128 {
                    if total <= 0 {
                        return 0;
                    }
                    (0..P).map(|p| i128::from(row[p]) * v(p)).sum::<i128>() / total
                };
                let heads = part(&|p| i128::from(f.size[p]), ts);
                let money = part(&|p| f.income[p], ti);
                ((franchise * heads + (2000 - franchise) * money) / 2000).clamp(0, 1000) as u16
            })
            .collect()
    }

    /// The economy's reading of the laws in force, metropole and colonies.
    fn law_effects(&self, laws: &[Option<u16>]) -> [LawEffects; 2] {
        let w = &self.wired;
        [Scope::Metropole, Scope::Colonies].map(|scope| {
            let d = self.politics.declared(laws, scope);
            let mut e = LawEffects::default();
            let fx = |i: usize| d[i].map(|v| milli(i64::from(v)));
            let share = |i: usize| d[i].map(|v| milli(i64::from(v.clamp(0, 1000))));
            e.migration_reach = [
                fx(w.within).unwrap_or(Fixed::ONE),
                fx(w.between).unwrap_or(Fixed::ONE),
            ];
            for (c, &i) in w.caps.iter().enumerate() {
                if let Some(v) = fx(i) {
                    e.tax_caps[c] = v.min(public::MAX_TAX);
                }
            }
            if let Some(v) = fx(w.collection) {
                e.tax_collection = v.min(Fixed::ONE);
            }
            for (good, i) in self.public_goods.iter().zip([w.education, w.health]) {
                if let (Some(g), Some(v)) = (good, share(i)) {
                    e.public_share.push((*g, v));
                }
            }
            if let Some(v) = share(w.enrolment) {
                e.enrolment_target = v;
            }
            if let Some(v) = fx(w.rent) {
                e.landowner_rent = v;
            }
            if let Some(v) = share(w.ownership) {
                e.state_ownership = v;
            }
            if let Some(v) = share(w.profit_to_treasury) {
                e.state_profit_to_treasury = v;
            }
            if let Some(v) = share(w.tariff) {
                e.tariff = v;
            }
            if let Some(v) = share(w.frontier) {
                e.frontier_access = v;
            }
            if let Some(v) = fx(w.foreign_investment) {
                e.foreign_investment = v.max(Fixed::ZERO);
            }
            e
        })
    }

    /// One year of movements in country `c`, and the laws its government yields.
    fn year(
        &self,
        date: Date,
        world: &GameWorld,
        c: usize,
        facts: &[Facts; 2],
        decide: bool,
        out: &mut Changes,
    ) {
        let p = &*self.politics;
        let s = &world.politics;
        let laws = &s.laws[c];
        let effects = [
            p.effects(laws, Scope::Metropole),
            p.effects(laws, Scope::Colonies),
        ];
        let mut movements = s.movements[c].clone();
        // (movement, law, mobilized per mille) of the movements whose goal is not in force.
        let mut unanswered: Vec<(usize, u16, i32)> = Vec::new();
        let mut mobilized_total = 0i32;
        for (k, kind) in p.movements.iter().enumerate() {
            let scope = match kind.among {
                Among::Everyone => 0,
                Among::Colonies => 1,
                // Communities are not in the pops yet.
                Among::Targeted => {
                    movements[k] = MovementState::default();
                    continue;
                }
            };
            let f = &facts[scope];
            let people = f.population();
            let row = &self.recruits[k];
            let recruits: i64 = (0..P)
                .map(|q| (i128::from(f.size[q]) * i128::from(row[q]) / 1000) as i64)
                .sum();
            let satisfied = match kind.goal {
                GoalRef::Law(l) => laws.contains(&Some(l)),
                // Independence and revision need borders, diplomacy and war (phases 4 and 5):
                // measured, never won.
                GoalRef::Independence | GoalRef::Revision => false,
            };
            let mut weakest = 0u8;
            let mut pressure = i32::MAX;
            for (i, &(m, b)) in kind.cause.iter().enumerate() {
                let value = match m {
                    Measure::Indicator(x) if usize::from(x) == measure::LEGITIMACY => {
                        Some(i32::from(s.legitimacy(c)))
                    }
                    Measure::Indicator(x) => f.indicator(usize::from(x), row),
                    Measure::Effect(e) => Some(effects[scope][usize::from(e)]),
                };
                let mg = value.map_or(0, |v| margin(b, v, m.range()));
                if mg < pressure {
                    pressure = mg;
                    weakest = i as u8;
                }
            }
            if people == 0 || recruits == 0 || satisfied {
                pressure = 0;
            }
            let st = &mut movements[k];
            if pressure > 0 {
                pressure += leader_push(st);
            }
            let pressure = pressure.clamp(0, 1000);
            let mut support = i32::from(st.support);
            if pressure > 0 {
                support += (i32::from(kind.growth) * pressure / 1000).max(1);
            } else if support > 0 {
                // Halves every `persistence` years (ln 2 ≈ 0.693).
                support -= (support * 693 / (1000 * i32::from(kind.persistence))).max(1);
            }
            st.support = support.clamp(0, 1000) as u16;
            st.pressure = pressure as u16;
            st.weakest = weakest;
            if people > 0 && !satisfied {
                let mob = (i64::from(st.support) * recruits / people).clamp(0, 1000) as i32;
                mobilized_total += mob;
                if let GoalRef::Law(l) = kind.goal {
                    unanswered.push((k, l, mob));
                }
            }
        }
        let legitimacy = (1000 - mobilized_total * LEGITIMACY_LOSS).clamp(0, 1000);
        out.movements.push((c, movements, legitimacy as u16));
        if !decide {
            return;
        }

        // The government yields to the strongest push in each law group, if the interest
        // groups that count do not block it; and the groups that count change a law by
        // themselves when they back it by a wide margin.
        let repression = effects[0][self.wired.repression].clamp(0, 1000);
        let threshold = THRESHOLD * legitimacy / 1000;
        let clout = &s.clout[c];
        let backing = |l: usize| -> i32 {
            stances(p, s, c, l)
                .iter()
                .enumerate()
                .map(|(ig, st)| {
                    let unit = (st.total * 1000 / STANCE_UNIT).clamp(-1000, 1000);
                    i64::from(clout.get(ig).copied().unwrap_or(0)) * unit / 1000
                })
                .sum::<i64>() as i32
        };
        let mut push = vec![0i32; p.laws.len()];
        let mut pusher = vec![None; p.laws.len()];
        for &(k, goal, mob) in &unanswered {
            // Reform goes by steps: a movement pushes the next law of the group on the way from
            // the law in force to its goal.
            let group = &p.groups[usize::from(p.laws[usize::from(goal)].group)];
            let g = usize::from(p.laws[usize::from(goal)].group);
            let (Some(at), Some(to)) = (
                group.laws.iter().position(|&x| laws[g] == Some(x)),
                group.laws.iter().position(|&x| x == goal),
            ) else {
                continue;
            };
            // Forms of government have no order: a regime falls at once.
            let l = if Some(g) == self.government {
                goal
            } else {
                // The next law on the way, skipping laws aimed at communities unless they are
                // the goal itself.
                let step = |i: usize| if to > at { i + 1 } else { i - 1 };
                let mut i = step(at);
                while i != to && p.laws[usize::from(group.laws[i])].targeted {
                    i = step(i);
                }
                group.laws[i]
            };
            // Mobilization before repression; repression is applied per law below.
            let v = mob;
            if v > push[usize::from(l)] {
                push[usize::from(l)] = v;
                pusher[usize::from(l)] = Some(k as u16);
            }
        }
        let new_regime = self.government.is_some_and(|gg| {
            s.changes.iter().rev().any(|r| {
                usize::from(r.country) == c
                    && usize::from(p.laws[usize::from(r.to)].group) == gg
                    && date.year - r.date.year <= REGIME_YEARS
            })
        });
        for (g, group) in p.groups.iter().enumerate() {
            if laws[g].is_none() || self.resting(s, c, g, date) {
                continue;
            }
            let mut best: Option<(u16, Enactor, i32)> = None;
            let at = group.laws.iter().position(|&x| laws[g] == Some(x));
            for (i, &l) in group.laws.iter().enumerate() {
                let li = usize::from(l);
                if laws[g] == Some(l) {
                    continue;
                }
                // Without a movement, only a new regime rewrites laws, and one step along the
                // group's order at a time.
                let next = new_regime && at.is_some_and(|a| a.abs_diff(i) == 1);
                if !(push[li] > 0 && pusher[li].is_some()) && !next {
                    continue;
                }
                let b = backing(li);
                // The state represses what the groups that count oppose, not what they back:
                // repression bites in proportion to the clout not backing the law.
                let bite = repression * (1000 - b.clamp(0, 1000)) / 1000;
                let pushed = push[li] * (1000 - bite) / 1000;
                let moved = pushed >= MIN_MOBILIZED;
                if !moved && !next {
                    continue;
                }
                let (score, by) = match pusher[li] {
                    Some(k) if moved => (pushed * PUSH_WEIGHT + b, Enactor::Movement(k)),
                    _ => (b, Enactor::Groups),
                };
                let bar = if by == Enactor::Groups {
                    GROUPS_THRESHOLD
                } else {
                    threshold
                };
                if score >= bar && best.is_none_or(|(_, _, x)| score > x) {
                    best = Some((l, by, score));
                }
            }
            if let Some((l, by, _)) = best {
                out.enact.push((c as u16, l, by));
            }
        }
    }

    /// Whether law group `g` of country `c` changed less than [`COOLDOWN_YEARS`] ago.
    fn resting(&self, s: &PoliticsState, c: usize, g: usize, date: Date) -> bool {
        s.changes.iter().rev().any(|r| {
            usize::from(r.country) == c
                && usize::from(self.politics.laws[usize::from(r.to)].group) == g
                && date.year - r.date.year < COOLDOWN_YEARS
        })
    }
}

/// What a movement's leader adds to the margin of a cause that holds (phase 7: characters).
/// Zero while there are no characters; a leader never creates a cause.
fn leader_push(_movement: &MovementState) -> i32 {
    0
}

#[derive(Default)]
pub struct Changes {
    /// (country, law, who) to enact, in order.
    enact: Vec<(u16, u16, Enactor)>,
    clout: Option<Vec<Vec<u16>>>,
    taxes: Option<Vec<TaxContext>>,
    /// (country, movements, legitimacy).
    movements: Vec<(usize, Vec<MovementState>, u16)>,
    /// Recompute the economy's reading of every country's laws.
    refresh: bool,
}

impl System<GameWorld> for PoliticsSystem {
    type Changes = Changes;

    fn name(&self) -> &'static str {
        "politics"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }

    fn access(&self) -> Access {
        Access::new()
            .read(ECON)
            .write(ECON)
            .read(POLITY)
            .write(POLITY)
            // The immigrant share (`measure::immigrant_share`).
            .read(crate::demography::DEMOGRAPHY)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> Changes {
        let s = &world.politics;
        let p = &*self.politics;
        let mut out = Changes::default();
        // Laws asked for by command, the newest request of each group.
        let group = |l: u16| p.laws.get(usize::from(l)).map(|law| usize::from(law.group));
        for (i, &(c, l)) in s.pending.iter().enumerate() {
            let Some(g) = group(l) else {
                continue;
            };
            let superseded = s.pending[i + 1..]
                .iter()
                .any(|&(c2, l2)| c2 == c && group(l2) == Some(g));
            if !superseded
                && s.law_in_force(usize::from(c), g)
                    .is_some_and(|cur| cur != l)
            {
                out.enact.push((c, l, Enactor::Command));
            }
        }
        if ctx.date.day != 1 || world.econ.regions.is_empty() {
            return out;
        }
        out.refresh = true;
        let econ = &world.econ;
        let facts = measure::facts(&self.cat, self.kinds, &self.manufactures, world);
        let n = s.laws.len().min(facts.len());
        out.taxes = Some(
            (0..n)
                .map(|c| {
                    let m = &econ.markets[c];
                    let b = &m.budget;
                    let rates = if b.tax_auto {
                        public::historical_rates(ctx.date.year)
                            .map(|r| (r * b.adjust).min(public::MAX_TAX))
                    } else {
                        b.tax_rates
                    };
                    let mille = |x: Fixed| {
                        (i128::from(x.raw()) * 1000 / i128::from(Fixed::ONE.raw())).clamp(0, 1000)
                            as u16
                    };
                    let f = &facts[c][0];
                    let mut by_class = [0i128; 3];
                    for (q, inc) in f.income.iter().enumerate() {
                        by_class[public::CLASS_OF[q]] += inc;
                    }
                    let total: i128 = by_class.iter().sum();
                    TaxContext {
                        stress: f.fiscal_stress.clamp(0, 1000) as u16,
                        wanted: rates.map(mille),
                        income: by_class.map(|v| {
                            if total > 0 {
                                (v * 1000 / total) as u16
                            } else {
                                0
                            }
                        }),
                    }
                })
                .collect(),
        );
        out.clout = Some(
            (0..n)
                .map(|c| {
                    let f = p.effects(&s.laws[c], Scope::Metropole)[self.wired.franchise];
                    self.clout(&facts[c][0], f)
                })
                .collect(),
        );
        if ctx.date.month == 1 {
            let player = world
                .player
                .and_then(|id| world.countries.iter().position(|(i, _)| i == id));
            for (c, f) in facts.iter().enumerate().take(n) {
                self.year(ctx.date, world, c, f, player != Some(c), &mut out);
            }
        }
        out
    }

    fn apply(&self, ctx: &TickCtx, changes: Changes, world: &mut GameWorld) {
        let p = &*self.politics;
        let s = &mut world.politics;
        let mut changed = Vec::new();
        for (c, l, by) in changes.enact {
            let g = usize::from(p.laws[usize::from(l)].group);
            let Some(slot) = s.laws.get_mut(usize::from(c)).and_then(|v| v.get_mut(g)) else {
                continue;
            };
            let Some(from) = slot.replace(l) else {
                continue;
            };
            if from != l {
                s.changes.push(LawChangeRecord {
                    date: ctx.date,
                    country: c,
                    from,
                    to: l,
                    by,
                });
                changed.push(usize::from(c));
            }
        }
        s.pending.clear();
        if let Some(clout) = changes.clout {
            s.clout = clout;
        }
        if let Some(taxes) = changes.taxes {
            s.taxes = taxes;
        }
        for (c, movements, legitimacy) in changes.movements {
            s.movements[c] = movements;
            s.legitimacy[c] = legitimacy;
        }
        let countries: Vec<usize> = if changes.refresh {
            (0..s.laws.len()).collect()
        } else {
            changed
        };
        for c in countries {
            let effects = self.law_effects(&world.politics.laws[c]);
            if let Some(m) = world.econ.markets.get_mut(c)
                && m.laws.effects != effects
            {
                m.laws.effects = effects;
                public::cap_taxes(m);
            }
        }
    }
}

/// One dated law change of the reference history against the campaign.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreRow {
    pub tag: String,
    pub law: String,
    /// Reference date, `YYYY-MM-DD`.
    pub reference: String,
    /// First date the country enacted that law in the campaign, if it did.
    pub simulated: Option<Date>,
}

impl ScoreRow {
    /// Simulated year minus reference year; `None` if it never happened.
    pub fn delay(&self) -> Option<i32> {
        let year: i32 = self.reference[..4].parse().ok()?;
        self.simulated.map(|d| d.year - year)
    }
}

/// The campaign's law changes against the dated reference changes, plus the changes the
/// campaign made that the reference does not list.
#[derive(Clone, Debug, Default)]
pub struct Scoreboard {
    pub rows: Vec<ScoreRow>,
    /// (tag, law, date) of simulated changes with no reference counterpart.
    pub extra: Vec<(String, String, Date)>,
}

pub fn score_against_history(
    p: &Politics,
    history: &[LawChange],
    tags: &[&str],
    state: &PoliticsState,
) -> Scoreboard {
    let first = |c: usize, law: usize| {
        state
            .changes
            .iter()
            .find(|r| usize::from(r.country) == c && usize::from(r.to) == law)
            .map(|r| r.date)
    };
    let mut rows = Vec::new();
    for h in history {
        let (Some(c), Some(l)) = (tags.iter().position(|t| *t == h.tag), p.law(&h.law)) else {
            continue;
        };
        rows.push(ScoreRow {
            tag: h.tag.clone(),
            law: h.law.clone(),
            reference: h.date.clone(),
            simulated: first(c, l),
        });
    }
    let extra = state
        .changes
        .iter()
        .filter(|r| {
            let law = &p.laws[usize::from(r.to)].key;
            let tag = tags.get(usize::from(r.country)).copied().unwrap_or("");
            !history.iter().any(|h| h.tag == tag && h.law == *law)
        })
        .map(|r| {
            (
                tags.get(usize::from(r.country))
                    .copied()
                    .unwrap_or("")
                    .to_owned(),
                p.laws[usize::from(r.to)].key.clone(),
                r.date,
            )
        })
        .collect();
    Scoreboard { rows, extra }
}
