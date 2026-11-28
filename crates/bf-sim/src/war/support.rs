//! War support (owner's decision G4) and how it ends a war (G5, and the political surrender of
//! the orchestrator's note of 2026-11-18). Behind `Switches::war`; the numbers are in
//! `data/military/war.ron`. Design in `docs/war.pt-BR.md`, "Apoio à guerra".
//!
//! Each country at war has a support of 0 to 1,000 per mille, the clout-weighted mean of its
//! interest groups. Every week a target is worked out term by term (the start, the casualties,
//! the duration, the civilian deprivation, the occupied territory and the front's course), and
//! each group moves toward the target plus its bias, at a speed the regime sets: free elections
//! hasten it, censorship slows it. Below the peace threshold the AI asks for peace; below the
//! collapse threshold a countdown runs, and at its end the government falls: a monarchy becomes
//! a republic, a dictatorship a military one (a new form of government, which the treaties read
//! as a change of regime, owner's decision P3), and in a republic the cabinet falls with the form
//! kept. The government that follows asks for peace.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::fight::{Fighting, SegmentWeek};
use crate::game::GameWorld;
use crate::treaty::{ClauseKind, Proposal};

/// `data/military/war.ron`: the conduct of the war.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConductFile {
    pub support: SupportRules,
    pub occupation: OccupationRules,
    /// The colonial war (G6); none in older data.
    #[serde(default)]
    pub colonial: Option<super::colonial::ColonialRules>,
}

/// The rules of war support. Per mille unless named otherwise.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SupportRules {
    /// Support at the declaration: the country declared on, the one that declared with a cause
    /// on record or without one, and the metropole in a colonial war.
    pub attacked: i32,
    pub attacker_cause: i32,
    pub attacker_no_cause: i32,
    pub colonial: i32,
    /// (interest group, bias added to its target).
    pub groups: Vec<(String, i32)>,
    /// Support lost per tenfold of the killed per million people (Mueller, 1973).
    pub casualties_per_decade: i32,
    /// Support lost per year of war.
    pub duration_per_year: i32,
    /// G8 (to the owner): the duration weighs by the share of the land army committed to the war,
    /// times `commitment_factor` (per mille), at most whole. A total war weighs whole; a distant
    /// expedition of regulars little. Off: every war weighs whole.
    #[serde(default)]
    pub duration_by_commitment: bool,
    #[serde(default = "default_commitment_factor")]
    pub commitment_factor: u32,
    /// G9 (the owner, 2026-11-19: on): the people of a state the enemy advances into count as
    /// under the enemy in proportion to the advance over the state's depth, before the state
    /// changes hands; off, the whole state counts at once when it changes hands.
    #[serde(default)]
    pub occupation_by_advance: bool,
    /// Support lost at a full loss of the basic needs met before the war.
    pub deprivation: i32,
    /// Support lost with the whole population under occupation.
    pub territory: i32,
    /// Support gained per km a week the front moves forward (lost when back), and its cap.
    pub result_per_km: i32,
    pub result_cap: i32,
    /// Weeks over which the front's course is smoothed.
    pub result_weeks: i32,
    /// Share of the gap to the target closed in a week, and how full franchise adds to it and
    /// full censorship takes from it.
    pub speed: i32,
    pub free_vote: i32,
    pub censorship: i32,
    /// Below `peace` the AI asks for peace; below `collapse` the countdown runs.
    pub peace: i32,
    pub collapse: i32,
    /// Weeks between two proposals of peace by the same country.
    pub ask_weeks: u16,
    /// Weeks from the collapse to the fall of the government.
    pub revolution_weeks: u16,
    /// A country overrun, its government falls at once: this share of the metropolitan people
    /// under the enemy, or `capital_overrun` of them with the capital taken. Not while an ally
    /// fights on, nor while it holds colonies the enemy has not taken.
    pub overrun: i32,
    pub capital_overrun: i32,
    /// (form of government, form after a war revolution); a form not listed keeps itself and
    /// only the cabinet falls.
    pub revolution: Vec<(String, String)>,
    /// Whether the countdown runs for the player's country too (A3: recommended yes).
    pub player_forced: bool,
}

/// The rules of occupation (G3). Per mille unless named otherwise.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OccupationRules {
    /// Share of the occupied GDP the occupier takes in the first week, and the ceiling it
    /// reaches after `extraction_weeks` (occupied France, 1940–1944: about a third in 1943).
    pub extraction_start: i32,
    pub extraction_cap: i32,
    pub extraction_weeks: u32,
    /// Garrison needed per mille of the occupied population, calm and wholly hostile
    /// (Quinlivan, 1995).
    pub garrison_calm: i32,
    pub garrison_hostile: i32,
    /// Most of the occupier's land men that may stand as garrison.
    pub garrison_cap: i32,
    /// Item 16: the unit type that holds posts (`garrison`). The occupier's own formations of it,
    /// colonial ones apart, stand as garrison first; only what they miss comes out of its field
    /// army, up to `garrison_cap`. `None`: only the field army, as before.
    #[serde(default)]
    pub garrison_unit: Option<String>,
    /// Weeks an occupier without a player bears a garrison short before it orders formations
    /// of `garrison_unit` to cover it (one element for each element's men missing). DESIGN
    /// VALUE.
    #[serde(default = "default_garrison_order_weeks")]
    pub garrison_order_weeks: u32,
    /// Resistance at the occupation, its weekly growth per mille extracted and per mille of
    /// garrison missing, and its weekly fall under a full garrison.
    pub resistance_start: i32,
    pub resistance_per_extraction: i32,
    pub resistance_per_shortfall: i32,
    pub resistance_calm: i32,
    /// Plant levels destroyed per week of full combat in each state of a segment, the
    /// casualties of a week of full combat, and the most of a state's levels destroyed.
    pub devastation_per_week: i32,
    pub devastation_full_losses: u64,
    pub devastation_cap: i32,
    /// Civilians killed per week of full combat, per million people of each state.
    pub civilians_per_million: i64,
    /// Scorched earth (`war::scorch`); `None` with the switch `scorched_earth` off.
    #[serde(default)]
    pub scorched: Option<super::scorch::ScorchRules>,
}

/// What a country's support was made of in a week, per mille: the screen shows it term by term.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportTerms {
    pub start: i32,
    pub casualties: i32,
    pub duration: i32,
    pub deprivation: i32,
    pub territory: i32,
    pub result: i32,
    /// The sum, within 0 and 1,000.
    pub target: i32,
}

/// How a government fell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fall {
    /// A new form of government (law indices): a change of regime.
    Regime { from: u16, to: u16 },
    /// A republic: the cabinet fell, the form stays.
    Cabinet,
}

/// The war support of one country at war.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Support {
    pub country: u16,
    /// Per mille: the clout-weighted mean of `groups`.
    pub value: i32,
    /// Per mille, by interest group in catalogue order (one value when there are none).
    pub groups: Vec<i32>,
    /// Weeks at war.
    pub weeks: u32,
    /// The killed before this war, and the basic needs met before it (the yardstick of the
    /// deprivation).
    pub killed_before: u64,
    pub basic_before: Fixed,
    /// Km a week the front moved forward (negative: back), smoothed.
    pub momentum: Fixed,
    pub terms: SupportTerms,
    /// Weeks left before the government falls, while below the collapse.
    pub countdown: Option<u16>,
    /// The government fell; the one that followed asks for peace.
    pub fallen: Option<Fall>,
    /// Weeks since this country last proposed peace.
    pub asked: u16,
}

impl HashState for Support {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.country);
        h.write_i64(i64::from(self.value));
        h.write_len(self.groups.len());
        for g in &self.groups {
            h.write_i64(i64::from(*g));
        }
        h.write_u32(self.weeks);
        h.write_u64(self.killed_before);
        self.basic_before.hash_state(h);
        self.momentum.hash_state(h);
        for t in [
            self.terms.start,
            self.terms.casualties,
            self.terms.duration,
            self.terms.deprivation,
            self.terms.territory,
            self.terms.result,
            self.terms.target,
        ] {
            h.write_i64(i64::from(t));
        }
        match self.countdown {
            Some(w) => {
                h.write_u8(1);
                h.write_u16(w);
            }
            None => h.write_u8(0),
        }
        match self.fallen {
            None => h.write_u8(0),
            Some(Fall::Cabinet) => h.write_u8(1),
            Some(Fall::Regime { from, to }) => {
                h.write_u8(2);
                h.write_u16(from);
                h.write_u16(to);
            }
        }
        h.write_u16(self.asked);
    }
}

/// The conduct of the war resolved against the politics catalogue, built at load.
#[derive(Clone, Debug)]
pub struct Conduct {
    pub rules: ConductFile,
    /// Bias of each interest group, in catalogue order; empty without politics.
    pub groups: Vec<i32>,
    /// (form of government, form after a war revolution), law indices.
    pub revolution: Vec<(u16, u16)>,
    /// The law group of the forms of government, and the movement that brings a new one.
    pub government: Option<usize>,
    pub regime_change: Option<u16>,
    pub politics: Option<std::sync::Arc<crate::politics::Politics>>,
    franchise: Option<usize>,
    censorship: Option<usize>,
    state_media: Option<usize>,
    /// The mass media (`Switches::media`, slice 2); `None` off.
    pub media: Option<std::sync::Arc<crate::media::MediaRules>>,
}

impl Conduct {
    /// The same conduct with the mass media's rules.
    pub fn with_media(mut self, media: Option<std::sync::Arc<crate::media::MediaRules>>) -> Self {
        self.media = media;
        self
    }

    pub fn resolve(
        rules: ConductFile,
        politics: Option<std::sync::Arc<crate::politics::Politics>>,
    ) -> Result<Self, String> {
        let mut groups = Vec::new();
        let mut revolution = Vec::new();
        let (mut government, mut regime_change) = (None, None);
        if let Some(p) = &politics {
            groups = vec![0; p.interest_groups.len()];
            for (key, bias) in &rules.support.groups {
                let g = p
                    .interest_groups
                    .iter()
                    .position(|g| &g.key == key)
                    .ok_or(format!("war.ron: unknown interest group {key}"))?;
                groups[g] = *bias;
            }
            for (from, to) in &rules.support.revolution {
                let law = |k: &str| p.law(k).ok_or(format!("war.ron: unknown law {k}"));
                revolution.push((law(from)? as u16, law(to)? as u16));
            }
            government = p.group(crate::politics::GOVERNMENT_GROUP);
            regime_change = p
                .movements
                .iter()
                .position(|m| m.key == "regime_change")
                .map(|m| m as u16);
        }
        Ok(Self {
            rules,
            groups,
            revolution,
            government,
            regime_change,
            politics,
            franchise: crate::politics::effect_index("franchise"),
            censorship: crate::politics::effect_index("censorship"),
            state_media: crate::politics::effect_index("state_media"),
            media: None,
        })
    }

    /// (franchise, censorship) in force in `c`, per mille; none without politics.
    fn regime(&self, world: &GameWorld, c: usize) -> (i32, i32) {
        let Some(p) = &self.politics else {
            return (0, 0);
        };
        if world.politics.laws.get(c).is_none() {
            return (0, 0);
        }
        let e = crate::polity::effects_in_force(
            p,
            &world.politics,
            c,
            crate::politics::Scope::Metropole,
        );
        let at = |i: Option<usize>| i.and_then(|i| e.get(i)).copied().unwrap_or(0);
        (at(self.franchise), at(self.censorship))
    }

    /// The media of `c` (slice 2): the channels under its laws; `None` without the media.
    fn media_of(
        &self,
        world: &GameWorld,
        c: usize,
        censorship: i32,
    ) -> Option<crate::media::Channels> {
        let rules = self.media.as_deref()?;
        let p = self.politics.as_ref()?;
        world.politics.laws.get(c)?;
        let e = crate::polity::effects_in_force(
            p,
            &world.politics,
            c,
            crate::politics::Scope::Metropole,
        );
        let owned = self
            .state_media
            .and_then(|i| e.get(i))
            .copied()
            .unwrap_or(0);
        Some(crate::media::channels(
            rules,
            &world.media,
            c,
            owned,
            censorship,
        ))
    }

    /// How the government of `c` falls: its form, if a war revolution changes it.
    fn fall_of(&self, world: &GameWorld, c: usize) -> Fall {
        let form = self
            .government
            .and_then(|g| world.politics.law_in_force(c, g));
        match form.and_then(|f| self.revolution.iter().find(|(from, _)| *from == f)) {
            Some(&(from, to)) if from != to => Fall::Regime { from, to },
            _ => Fall::Cabinet,
        }
    }
}

/// What the support step decides in a week.
#[derive(Clone, Debug, Default)]
pub struct SupportOut {
    /// Governments that fell this week.
    pub falls: Vec<(u16, Fall)>,
    /// (from, to, proposal): peace asked for at the treaties' table.
    pub proposals: Vec<(u16, u16, Proposal)>,
    /// Wars the side that declared gave up: they end as they began.
    pub withdrawn: Vec<u32>,
    /// Colonial wars the new country lost (G6): the metropole's demand, the union, is carried
    /// out without the table; the metropole fought for it.
    pub surrendered: Vec<u32>,
}

fn default_commitment_factor() -> u32 {
    3000
}

/// G8: the weight of duration, per mille, for a war that commits `share` per mille of the land
/// army: `commitment_factor` times the share, at most whole.
pub fn duration_weight(r: &SupportRules, share: u32) -> u32 {
    (u64::from(share) * u64::from(r.commitment_factor) / 1000).min(1000) as u32
}

/// Part of a demand: the first half of its clauses (at least one), the amounts halved.
pub fn part_of(demand: &Proposal) -> Proposal {
    let mut p = demand.clone();
    let keep = p.clauses.len().div_ceil(2).max(1);
    p.clauses.truncate(keep);
    for c in &mut p.clauses {
        match &mut c.kind {
            ClauseKind::Payment { amount } => *amount = *amount / 2,
            ClauseKind::Tribute { yearly } => *yearly = *yearly / 2,
            _ => {}
        }
    }
    p
}

/// A week of war support: every country at war moves toward its target, its government falls
/// at the end of a collapse, and the leaders below the peace threshold (or after a fall) ask for
/// peace. `segments` is this week's combat, `ended` the wars a peace ended this week.
pub fn support_week(
    c: &Conduct,
    map: &super::fight::FightSetup,
    world: &GameWorld,
    f: &mut Fighting,
    segments: &[SegmentWeek],
    ended: &[u32],
    player: Option<u16>,
) -> SupportOut {
    let r = &c.rules.support;
    let mut out = SupportOut::default();
    let wars: Vec<_> = world
        .war
        .conflicts
        .wars
        .iter()
        .filter(|w| !ended.contains(&w.id))
        .collect();
    // (country, start): declared on beats declaring; a cause on record helps the declarer.
    let mut at_war: std::collections::BTreeMap<u16, i32> = std::collections::BTreeMap::new();
    for w in &wars {
        let cause = world
            .war
            .conflicts
            .crises
            .iter()
            .find(|k| k.id == w.crisis)
            .is_some_and(|k| k.casus_belli);
        // A colonial war (G6): the metropole starts at `colonial`, the proclaimed country at
        // the colonial rules' `proclaimed` (GC2).
        let colonial = world.war.colonial.iter().any(|k| k.war == w.id);
        let proclaimed = c
            .rules
            .colonial
            .as_ref()
            .map_or(r.attacked, |k| k.proclaimed);
        for &m in &w.side_a {
            let s = if colonial {
                r.colonial
            } else if cause {
                r.attacker_cause
            } else {
                r.attacker_no_cause
            };
            at_war
                .entry(m)
                .and_modify(|v| *v = (*v).max(s))
                .or_insert(s);
        }
        for &m in &w.side_b {
            at_war.insert(m, if colonial { proclaimed } else { r.attacked });
        }
    }
    f.support.retain(|s| at_war.contains_key(&s.country));
    if at_war.is_empty() {
        return out;
    }
    // People, basic needs met and people under the enemy, by country at war.
    let mut people: std::collections::BTreeMap<u16, (i64, i128, i64)> = Default::default();
    // (metropolitan people, of them under the enemy, capital taken, a colony still held).
    let mut home: std::collections::BTreeMap<u16, (i64, i64, bool, bool)> = Default::default();
    let tag_of = |m: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(m))
            .map(|(_, c)| c.tag.as_str())
    };
    for (ri, reg) in world.econ.regions.iter().enumerate() {
        let m = reg.market;
        if !at_war.contains_key(&m) {
            continue;
        }
        let e = people.entry(m).or_default();
        for p in &reg.pops {
            e.0 += p.size;
            e.1 += i128::from(p.size) * i128::from(p.satisfaction[0].raw());
        }
        let taken = f.controller(ri as u32, m) != m;
        let under = under_enemy(r, f, &map.depth_km, reg, ri as u32, taken);
        e.2 += under;
        let h = home.entry(m).or_default();
        if reg.colony {
            h.3 |= !taken;
        } else {
            h.0 += reg.population();
            h.1 += under;
            if taken {
                h.2 |= tag_of(m).and_then(|t| map.capitals.get(t)) == Some(&reg.state);
            }
        }
    }
    // A country born in revolt fights on as a guerrilla while it has men under arms: it is not
    // overrun, and the land it lost does not weigh on its support (G6).
    let guerrilla = |m: u16| {
        world
            .war
            .colonial
            .iter()
            .any(|k| !k.peace && k.colony == m && k.armed > 0)
    };
    let overrun = |m: u16| {
        !guerrilla(m)
            && home.get(&m).is_some_and(|&(size, taken, capital, colony)| {
                let share = if size > 0 { taken * 1000 / size } else { 0 };
                !colony
                    && (share >= i64::from(r.overrun)
                        || (capital && share >= i64::from(r.capital_overrun)))
            })
    };
    // An ally still fighting: someone on the same side of a war who is not overrun.
    let ally_fights = |m: u16| {
        wars.iter().any(|w| {
            let side = if w.side_a.contains(&m) {
                &w.side_a
            } else if w.side_b.contains(&m) {
                &w.side_b
            } else {
                return false;
            };
            side.iter().any(|&x| x != m && !overrun(x))
        })
    };
    // The front's course: km a week each country moved forward, averaged over its segments.
    let mut course: std::collections::BTreeMap<u16, (Fixed, i64)> = Default::default();
    for g in segments {
        let km = g.outcome.advance_km;
        let a = course.entry(g.a).or_default();
        a.0 += km;
        a.1 += 1;
        let b = course.entry(g.b).or_default();
        b.0 -= km;
        b.1 += 1;
    }
    // The dead of the front, and those of the nuclear strikes received (N6: the user's own
    // support reads nothing of its use).
    let killed = |m: u16| -> u64 {
        f.losses.iter().find(|(k, _)| *k == m).map_or(0, |(_, l)| {
            *l * super::fight::KILLED_PER_MILLE as u64 / 1000
        }) + world.war.nuclear.dead_of(m)
    };
    for (&m, &start) in &at_war {
        let (size, basic, occupied) = people.get(&m).copied().unwrap_or_default();
        let basic = if size > 0 {
            Fixed::from_raw((basic / i128::from(size)) as i64)
        } else {
            Fixed::ONE
        };
        let n = c.groups.len().max(1);
        let dead = killed(m);
        let i = match f.support.binary_search_by_key(&m, |s| s.country) {
            Ok(i) => i,
            Err(i) => {
                let groups = (0..n)
                    .map(|g| (start + c.groups.get(g).copied().unwrap_or(0)).clamp(0, 1000))
                    .collect();
                f.support.insert(
                    i,
                    Support {
                        country: m,
                        value: start,
                        groups,
                        weeks: 0,
                        killed_before: dead,
                        basic_before: basic,
                        momentum: Fixed::ZERO,
                        terms: SupportTerms::default(),
                        countdown: None,
                        fallen: None,
                        asked: r.ask_weeks,
                    },
                );
                i
            }
        };
        let (franchise, censorship) = c.regime(world, usize::from(m));
        // The mass media (slice 2): the internet leaks the censorship (MD6), the state's media
        // add to the target (MD3), the free media hasten the move (MD2).
        let media = c.media_of(world, usize::from(m), censorship);
        let censorship = media.map_or(censorship, |x| x.censorship);
        let s = &mut f.support[i];
        s.weeks += 1;
        s.asked = s.asked.saturating_add(1);
        let avg = course
            .get(&m)
            .filter(|(_, k)| *k > 0)
            .map_or(Fixed::ZERO, |(km, k)| *km / Fixed::from_int(*k));
        s.momentum += (avg - s.momentum) / Fixed::from_int(i64::from(r.result_weeks.max(1)));
        let lost_basic = if s.basic_before.is_positive() {
            ((s.basic_before - basic).max(Fixed::ZERO) * Fixed::from_int(1000) / s.basic_before)
                .to_int_trunc() as i32
        } else {
            0
        };
        let per_million = |x: i64| -> u64 {
            if size > 0 {
                (i128::from(x.max(0)) * 1_000_000 / i128::from(size)) as u64
            } else {
                0
            }
        };
        // G8: the weeks of war weigh by the commitment. Only a metropole whose wars are all
        // colonial commits less than its whole army so far; any other war weighs whole.
        let weeks = if r.duration_by_commitment {
            let colonial: Vec<_> = world
                .war
                .colonial
                .iter()
                .filter(|k| !k.peace && k.owner == m)
                .collect();
            let other = wars.iter().any(|w| {
                (w.side_a.contains(&m) || w.side_b.contains(&m))
                    && !colonial.iter().any(|k| k.war == w.id)
            });
            if colonial.is_empty() || other {
                s.weeks
            } else {
                let share = colonial
                    .iter()
                    .map(|k| u32::from(k.commitment))
                    .sum::<u32>()
                    .min(1000);
                s.weeks * duration_weight(r, share) / 1000
            }
        } else {
            s.weeks
        };
        let t = terms(
            r,
            &Inputs {
                start,
                killed_per_million: per_million(dead.saturating_sub(s.killed_before) as i64),
                weeks,
                deprivation: lost_basic,
                occupied: if size > 0 && !guerrilla(m) {
                    (occupied * 1000 / size) as i32
                } else {
                    0
                },
                momentum: s.momentum,
            },
        );
        s.terms = t;
        let mut v = speed(r, franchise, censorship);
        let mut propaganda = 0;
        if let (Some(ch), Some(rules)) = (media, c.media.as_deref()) {
            v = (v * (1000 + i64::from(rules.effects.free_speed) * i64::from(ch.free) / 1000)
                / 1000)
                .clamp(1, 1000);
            propaganda = rules.effects.propaganda * ch.official / 1000;
        }
        for (g, val) in s.groups.iter_mut().enumerate() {
            let target =
                (t.target + propaganda + c.groups.get(g).copied().unwrap_or(0)).clamp(0, 1000);
            *val = toward(*val, target, v);
        }
        let clout = world
            .politics
            .clout
            .get(usize::from(m))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        s.value = national(&s.groups, clout);
        // The collapse and the fall.
        let forced = r.player_forced || player != Some(m);
        if s.fallen.is_none() && forced && overrun(m) && !ally_fights(m) {
            // Overrun, with no one left to fight on: the government falls at once.
            s.value = 0;
            s.groups.fill(0);
            let fall = c.fall_of(world, usize::from(m));
            s.fallen = Some(fall);
            s.countdown = None;
            s.asked = r.ask_weeks;
            out.falls.push((m, fall));
        }
        if s.fallen.is_none() {
            if s.value < r.collapse && forced {
                let left = s
                    .countdown
                    .map_or(r.revolution_weeks, |w| w.saturating_sub(1));
                if left == 0 {
                    let fall = c.fall_of(world, usize::from(m));
                    s.fallen = Some(fall);
                    s.countdown = None;
                    s.asked = r.ask_weeks;
                    out.falls.push((m, fall));
                } else {
                    s.countdown = Some(left);
                }
            } else {
                s.countdown = None;
            }
        }
    }
    // The leaders decide: the one that declared gives up, the one declared on asks for peace.
    let tag = |m: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(m))
            .map(|(_, c)| c.tag.clone())
    };
    for w in &wars {
        let (Some(&a), Some(&b)) = (w.side_a.first(), w.side_b.first()) else {
            continue;
        };
        let get = |f: &Fighting, m: u16| {
            f.support
                .binary_search_by_key(&m, |s| s.country)
                .ok()
                .map(|i| f.support[i].clone())
        };
        let (Some(sa), Some(sb)) = (get(f, a), get(f, b)) else {
            continue;
        };
        // A country born in revolt also loses when it has no one left under arms (G6).
        let disarmed = world
            .war
            .colonial
            .iter()
            .any(|k| k.war == w.id && !k.peace && k.armed == 0);
        let wants_out = |s: &Support, m: u16| {
            s.fallen.is_some() || (player != Some(m) && s.value < r.peace) || (m == b && disarmed)
        };
        if wants_out(&sa, a) {
            out.withdrawn.push(w.id);
            continue;
        }
        if wants_out(&sb, b) && sb.asked >= r.ask_weeks {
            if player != Some(a) && world.war.colonial.iter().any(|k| k.war == w.id) {
                out.surrendered.push(w.id);
                continue;
            }
            let (Some(from), Some(to)) = (tag(b), tag(a)) else {
                continue;
            };
            let open = world
                .treaties
                .table
                .open
                .iter()
                .any(|p| p.from == from && p.to == to);
            if open {
                continue;
            }
            let proposal = if sb.fallen.is_some() {
                w.demand.clone()
            } else {
                part_of(&w.demand)
            };
            out.proposals.push((b, a, proposal));
            if let Ok(i) = f.support.binary_search_by_key(&b, |s| s.country) {
                f.support[i].asked = 0;
            }
        }
    }
    out
}

/// G9 (the owner, 2026-11-19: (b)): the people of `reg` (region `ri`) under the enemy. All of
/// them when the region changed hands (`taken`); before that, with `occupation_by_advance`, the
/// share the enemy's advance into it has gone over its state's depth.
pub fn under_enemy(
    r: &SupportRules,
    f: &Fighting,
    depth: &std::collections::BTreeMap<u16, Fixed>,
    reg: &crate::econ::Region,
    ri: u32,
    taken: bool,
) -> i64 {
    if taken {
        return reg.population();
    }
    if !r.occupation_by_advance {
        return 0;
    }
    let km = f
        .advance
        .iter()
        .filter(|(_, into, _)| *into == ri)
        .map(|(_, _, km)| *km)
        .max()
        .unwrap_or(Fixed::ZERO);
    let d = depth
        .get(&reg.state)
        .copied()
        .unwrap_or(Fixed::from_int(100));
    if km.is_positive() && d.is_positive() {
        (Fixed::from_int(reg.population()) * (km / d).min(Fixed::ONE)).to_int_floor()
    } else {
        0
    }
}

/// log10(n) × 1,000, from the digits and a table of the mantissa: deterministic, within a few
/// thousandths.
pub fn log10_mille(n: u64) -> i64 {
    const TABLE: [i64; 11] = [0, 0, 301, 477, 602, 699, 778, 845, 903, 954, 1000];
    if n == 0 {
        return 0;
    }
    let mut digits = 0i64;
    let mut p = 1u64;
    while n / p >= 10 {
        p *= 10;
        digits += 1;
    }
    // Mantissa in [1, 10), in thousandths.
    let m = (u128::from(n) * 1000 / u128::from(p)) as i64;
    let (lo, frac) = ((m / 1000) as usize, m % 1000);
    let (a, b) = (TABLE[lo], TABLE[(lo + 1).min(10)]);
    digits * 1000 + a + (b - a) * frac / 1000
}

/// What the target of a week reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct Inputs {
    pub start: i32,
    pub killed_per_million: u64,
    pub weeks: u32,
    /// Per mille of the basic needs met before the war that are no longer met.
    pub deprivation: i32,
    /// Per mille of the people under the enemy.
    pub occupied: i32,
    pub momentum: Fixed,
}

/// The target of a week, term by term.
pub fn terms(r: &SupportRules, i: &Inputs) -> SupportTerms {
    let casualties =
        -(i64::from(r.casualties_per_decade) * log10_mille(1 + i.killed_per_million) / 1000) as i32;
    let duration = -(i64::from(r.duration_per_year) * i64::from(i.weeks) / 52) as i32;
    let deprivation =
        -(i64::from(r.deprivation) * i64::from(i.deprivation.clamp(0, 1000)) / 1000) as i32;
    let territory = -(i64::from(r.territory) * i64::from(i.occupied.clamp(0, 1000)) / 1000) as i32;
    let result = (Fixed::from_int(i64::from(r.result_per_km)) * i.momentum)
        .to_int_trunc()
        .clamp(-i64::from(r.result_cap), i64::from(r.result_cap)) as i32;
    let target =
        (i.start + casualties + duration + deprivation + territory + result).clamp(0, 1000);
    SupportTerms {
        start: i.start,
        casualties,
        duration,
        deprivation,
        territory,
        result,
        target,
    }
}

/// Share of the gap closed in a week, per mille, under `franchise` and `censorship` (per mille).
pub fn speed(r: &SupportRules, franchise: i32, censorship: i32) -> i64 {
    let f = i64::from(franchise.clamp(0, 1000));
    let c = i64::from(censorship.clamp(0, 1000));
    let s = i64::from(r.speed) * (1000 + i64::from(r.free_vote) * f / 1000) / 1000;
    (s * (1000 - i64::from(r.censorship) * c / 1000) / 1000).clamp(1, 1000)
}

/// One step of `value` toward `target` at `speed` per mille, at least one per mille when apart.
pub fn toward(value: i32, target: i32, speed: i64) -> i32 {
    let gap = i64::from(target - value);
    let step = gap * speed / 1000;
    let step = if step == 0 { gap.signum() } else { step };
    value + step as i32
}

/// The clout-weighted mean of the groups; the plain mean when no clout is known.
pub fn national(groups: &[i32], clout: &[u16]) -> i32 {
    let total: i64 = clout.iter().take(groups.len()).map(|&c| i64::from(c)).sum();
    if total > 0 && clout.len() >= groups.len() {
        (groups
            .iter()
            .zip(clout)
            .map(|(&g, &c)| i64::from(g) * i64::from(c))
            .sum::<i64>()
            / total) as i32
    } else if groups.is_empty() {
        0
    } else {
        (groups.iter().map(|&g| i64::from(g)).sum::<i64>() / groups.len() as i64) as i32
    }
}

fn default_garrison_order_weeks() -> u32 {
    4
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> SupportRules {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/military/war.ron"
        ))
        .unwrap();
        ron::from_str::<ConductFile>(&text).unwrap().support
    }

    #[test]
    fn log10_is_close() {
        assert_eq!(log10_mille(1), 0);
        assert_eq!(log10_mille(10), 1000);
        assert_eq!(log10_mille(1000), 3000);
        assert!(
            (log10_mille(291) - 2464).abs() <= 10,
            "{}",
            log10_mille(291)
        );
        assert!((log10_mille(30_000) - 4477).abs() <= 10);
    }

    /// Germany, 1914–1918: some 2 million killed of 67 million, four years, the blockade's
    /// hunger, the front falling back after Amiens, and no German land occupied. The support
    /// passes the collapse in the fourth year: the country gives way from within.
    #[test]
    fn germany_1918_collapses_with_its_land_whole() {
        let r = rules();
        let t = terms(
            &r,
            &Inputs {
                start: r.attacked,
                killed_per_million: 2_000_000 * 1_000_000 / 67_000_000,
                weeks: 4 * 52,
                deprivation: 350,
                occupied: 0,
                momentum: Fixed::from_int(-5),
            },
        );
        assert_eq!(t.territory, 0);
        assert!(t.target < r.collapse, "{t:?}");
        // Without the hunger and the retreat, the same dead and years would not break it.
        let whole = terms(
            &r,
            &Inputs {
                start: r.attacked,
                killed_per_million: 2_000_000 * 1_000_000 / 67_000_000,
                weeks: 4 * 52,
                ..Default::default()
            },
        );
        assert!(whole.target > r.peace, "{whole:?}");
    }

    /// The United States in Vietnam: no defeat, no land lost, 58,000 killed of 200 million. The
    /// support stays above the peace threshold at three years and passes it between the fifth
    /// and the sixth: the country asks for peace without a military defeat.
    #[test]
    fn vietnam_asks_for_peace_without_a_defeat() {
        let r = rules();
        let at = |years: u32, killed: u64| {
            terms(
                &r,
                &Inputs {
                    start: r.attacker_cause,
                    killed_per_million: killed * 1_000_000 / 200_000_000,
                    weeks: years * 52,
                    ..Default::default()
                },
            )
        };
        let t3 = at(3, 30_000);
        assert!(t3.target > r.peace, "{t3:?}");
        let t6 = at(6, 58_000);
        assert!(t6.target < r.peace, "{t6:?}");
        assert!(t6.target > r.collapse, "{t6:?}");
    }

    /// G8, duration by commitment, with the factor of the data: Germany 1914–18 fought with its
    /// whole army (weight 1, the same case as above). The United States had some 540,000 men in
    /// Vietnam of an army of about 1.5 million (a share of 0.36): with the factor 3 the weight is
    /// whole, and the owner's case stands (peace between the fifth and the sixth year). With the
    /// factor 2 it was 0.72, and the peace moved to the eighth year.
    #[test]
    fn duration_by_commitment_keeps_the_owners_cases() {
        let r = rules();
        let weighed = |years: u32, share: u32| years * 52 * duration_weight(&r, share) / 1000;
        let germany = terms(
            &r,
            &Inputs {
                start: r.attacked,
                killed_per_million: 2_000_000 * 1_000_000 / 67_000_000,
                weeks: weighed(4, 1000),
                deprivation: 350,
                occupied: 0,
                momentum: Fixed::from_int(-5),
            },
        );
        assert!(germany.target < r.collapse, "{germany:?}");
        let share = (540_000u64 * 1000 / 1_500_000) as u32;
        let at = |years: u32, killed: u64, factor: u32| {
            let r = SupportRules {
                commitment_factor: factor,
                ..r.clone()
            };
            terms(
                &r,
                &Inputs {
                    start: r.attacker_cause,
                    killed_per_million: killed * 1_000_000 / 200_000_000,
                    weeks: years * 52 * duration_weight(&r, share) / 1000,
                    ..Default::default()
                },
            )
        };
        let f = r.commitment_factor;
        for (factor, years) in [(f, [5, 6, 7, 8]), (2000, [5, 6, 7, 8])] {
            let line: Vec<i32> = years
                .iter()
                .map(|&y| at(y, 58_000 * u64::from(y.min(6)) / 6, factor).target)
                .collect();
            eprintln!("Vietnam, factor {factor}: years 5 to 8 {line:?}");
        }
        assert!(at(3, 30_000, f).target > r.peace);
        let t6 = at(6, 58_000, f);
        assert!(t6.target < r.peace, "{t6:?}");
        assert!(t6.target > r.collapse, "{t6:?}");
    }

    #[test]
    fn censorship_slows_and_the_vote_hastens() {
        let r = rules();
        assert!(speed(&r, 1000, 0) > speed(&r, 0, 0));
        assert!(speed(&r, 0, 1000) < speed(&r, 0, 0));
        assert_eq!(toward(500, 400, 100), 490);
        assert_eq!(toward(500, 499, 100), 499);
    }
}
