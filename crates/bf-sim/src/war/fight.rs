//! The war on land in the tick (phase 5, slice C): who controls each state, the segments of
//! every front, and a week of combat on each by `calc::land_step`. Behind `Switches::war`
//! (off). Design in `docs/war.pt-BR.md`, "A guerra no tick".
//!
//! A segment is a pair of neighbouring states held by the two sides of a war: the one held by
//! side A first. The fronts are the segments between each pair of enemy countries; they are
//! derived every week from the control and the map, never saved. What is saved is the control
//! of the states taken, the advance of each segment, the postures and the losses.

use std::collections::BTreeMap;
use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::calc::{LandField, LandSide, Posture, TerrainMix, apportion, land_step};
use super::catalogue::{Domain, Military, Mobility, Terrain};
use super::state::{CountryWar, Formation, men};
use crate::game::GameWorld;

/// Ratio of land fire at which the AI attacks a front (owner's decision G5: 1.5).
pub const AI_ATTACK_RATIO: (i64, i64) = (3, 2);

/// What the war step needs from the map, built at load.
#[derive(Clone, Debug)]
pub struct FightSetup {
    /// Land neighbours of each map state, by state id.
    pub neighbours: Arc<BTreeMap<u16, Vec<u16>>>,
    /// Depth of each state, km: half the side of a square of its area. The front takes the
    /// state when its advance passes it.
    pub depth_km: BTreeMap<u16, Fixed>,
    /// Terrain of each state (`calc::state_terrain`: climate, mountain, marsh, forest, town),
    /// by state id; open where the data lack the state.
    pub terrain: BTreeMap<u16, TerrainMix>,
    /// Share of the border between two states that runs along a big river, by (lower, higher)
    /// state id; none where absent.
    pub river: BTreeMap<(u16, u16), Fixed>,
    /// States whose winters stop the front (`Terrain` of their climate is cold): the months of
    /// December to February slow the front there.
    pub cold: std::collections::BTreeSet<u16>,
    /// Width of the border between two states, km, by (lower, higher) state id
    /// (`assets/map/borders.ron`): the width of the segment, for the density of its defence.
    pub width_km: BTreeMap<(u16, u16), Fixed>,
    /// The state of each country's capital, by tag.
    pub capitals: BTreeMap<String, u16>,
    /// War support and occupation (G3, G4), when `data/military/war.ron` exists.
    pub conduct: Option<Arc<super::support::Conduct>>,
}

/// The war on land, saved and hashed only when not empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fighting {
    /// (region, market of the occupier) for every region held by another than its owner, sorted
    /// by region. Occupation changes control, not the owner: the owner changes only by peace.
    pub control: Vec<(u32, u16)>,
    /// (from region, into region, km): how far the side holding `from` advanced into `into`.
    pub advance: Vec<(u32, u32, Fixed)>,
    /// (country, enemy, attack): the posture the player set on the front against `enemy`. The AI's
    /// posture is worked out each week (G5) and not saved.
    pub postures: Vec<(u16, u16, bool)>,
    /// (country, men): land casualties since its first war, killed, wounded and captured.
    pub losses: Vec<(u16, u64)>,
    /// The segments of the last week and what happened on each: what moved the front, for the
    /// war screen to read without working the week out again.
    pub last_week: Vec<SegmentWeek>,
    /// (from region, into region, weeks): how long each segment has held still, sorted. Its
    /// defenders dig in up to `front.dig_weeks`.
    pub still: Vec<(u32, u32, u16)>,
    /// The war support of every country at war, by country (G4).
    pub support: Vec<super::support::Support>,
    /// The regions under occupation, by region (G3).
    pub occupied: Vec<super::occupation::Occupied>,
    /// The devastation the combat left, by region (G3).
    pub wrecks: Vec<super::occupation::Wreck>,
}

impl Fighting {
    pub fn is_empty(&self) -> bool {
        self.control.is_empty()
            && self.advance.is_empty()
            && self.postures.is_empty()
            && self.losses.is_empty()
            && self.last_week.is_empty()
            && self.still.is_empty()
            && self.support.is_empty()
            && self.occupied.is_empty()
            && self.wrecks.is_empty()
    }

    /// Weeks segment (`from`, `into`) has held still.
    pub fn still_weeks(&self, from: u32, into: u32) -> u16 {
        self.still
            .binary_search_by_key(&(from, into), |(a, b, _)| (*a, *b))
            .map_or(0, |i| self.still[i].2)
    }

    /// The war support of `country`, if at war.
    pub fn support_of(&self, country: u16) -> Option<&super::support::Support> {
        self.support
            .binary_search_by_key(&country, |s| s.country)
            .ok()
            .map(|i| &self.support[i])
    }

    /// The market controlling region `r`, whose owner is `owner`.
    pub fn controller(&self, r: u32, owner: u16) -> u16 {
        self.control
            .binary_search_by_key(&r, |(k, _)| *k)
            .map_or(owner, |i| self.control[i].1)
    }

    fn set_control(&mut self, r: u32, by: u16, owner: u16) {
        match self.control.binary_search_by_key(&r, |(k, _)| *k) {
            Ok(i) if by == owner => {
                self.control.remove(i);
            }
            Ok(i) => self.control[i].1 = by,
            Err(_) if by == owner => {}
            Err(i) => self.control.insert(i, (r, by)),
        }
    }

    /// The posture `country` set against `enemy`, if any.
    pub fn posture(&self, country: u16, enemy: u16) -> Option<bool> {
        self.postures
            .iter()
            .find(|(c, e, _)| *c == country && *e == enemy)
            .map(|(_, _, a)| *a)
    }

    pub fn set_posture(&mut self, country: u16, enemy: u16, attack: bool) {
        match self
            .postures
            .iter_mut()
            .find(|(c, e, _)| *c == country && *e == enemy)
        {
            Some(p) => p.2 = attack,
            None => {
                self.postures.push((country, enemy, attack));
                self.postures.sort();
            }
        }
    }
}

/// One segment of a front this week, for the screen and the tests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentWeek {
    /// The region held by side A, and the one held by side B.
    pub from: u32,
    pub into: u32,
    /// The countries holding them.
    pub a: u16,
    pub b: u16,
    pub outcome: super::calc::LandOutcome,
}

/// What a week of fighting does, worked out in the forces system's compute.
#[derive(Clone, Debug, Default)]
pub struct FightWeek {
    pub fighting: Fighting,
    /// Wars that ended this week by a peace signed at the treaties' table.
    pub ended: Vec<u32>,
    /// (country, share of its land elements lost this week).
    pub lost_share: Vec<(u16, Fixed)>,
    /// (country, men lost this week): killed, wounded and captured.
    pub lost_men: Vec<(u16, u64)>,
    pub segments: Vec<SegmentWeek>,
    /// What the support decided: governments fallen, peace asked for, wars given up.
    pub support: super::support::SupportOut,
    /// Occupation money: (owner, occupier, amount).
    pub transfers: Vec<super::occupation::Transfer>,
    pub devastation: super::occupation::Devastation,
}

/// The land strength of a country: men, the fire and protection of what it holds, and its
/// average training, over every land formation.
struct Strength {
    men: u64,
    fire: Fixed,
    protection: Fixed,
    training: Fixed,
    mobility: Mobility,
}

/// How a country's army moves: the mobility of the types that hold most of its land men. An
/// army marches at the pace of the bulk of its men (in 1940 most of the German army walked,
/// whatever its armour did). Garrisons (`Static`) hold their posts and do not count; a tie goes
/// to the slower mobility, which holds the rest back. Foot without mobile men.
pub fn majority_mobility(mil: &Military, formations: &[Formation]) -> Mobility {
    let mut by: BTreeMap<Mobility, i64> = BTreeMap::new();
    for f in formations {
        if mil.types[f.unit as usize].domain != Domain::Land || !f.elements.is_positive() {
            continue;
        }
        let m = mil.spec(f.unit, f.generation as usize).mobility;
        if m == Mobility::Static {
            continue;
        }
        *by.entry(m).or_default() += men(mil, f);
    }
    // Ascending order: on a tie the first (slower) one stays.
    let mut best = (Mobility::Foot, 0i64);
    for (m, n) in by {
        if n > best.1 {
            best = (m, n);
        }
    }
    best.0
}

fn strength(mil: &Military, c: &CountryWar) -> Strength {
    let land: Vec<_> = c
        .formations
        .iter()
        .filter(|f| mil.types[f.unit as usize].domain == Domain::Land && f.elements.is_positive())
        .collect();
    let men_total: i64 = land.iter().map(|f| men(mil, f)).sum();
    let equipment: Vec<(u16, Fixed)> = land.iter().flat_map(|f| f.equipment.clone()).collect();
    let elements: Fixed = land.iter().map(|f| f.elements).sum();
    let training = if elements.is_positive() {
        land.iter().map(|f| f.training * f.elements).sum::<Fixed>() / elements
    } else {
        Fixed::ZERO
    };
    Strength {
        men: men_total.max(0) as u64,
        fire: mil.land_fire(&equipment),
        protection: mil.protection(&equipment),
        training,
        mobility: majority_mobility(mil, &c.formations),
    }
}

/// A week of war on land: segments, combat, advance and control. Pure: reads the world.
/// The wars a peace signed at the treaties' table this week ends: an agreement signed in the
/// seven days to `date` between the leaders of the two sides (the one that declared and the one
/// declared on). An ally's separate peace is for a later slice.
pub fn peace_signed(world: &GameWorld, date: laplace_schelling::Date) -> Vec<u32> {
    let tag = |m: u16| {
        world
            .countries
            .iter()
            .nth(m as usize)
            .map(|(_, c)| c.tag.as_str())
    };
    let since = date.to_days() - 7;
    world
        .war
        .conflicts
        .wars
        .iter()
        .filter(|w| {
            world.treaties.table.closed.iter().any(|c| {
                matches!(c.outcome, crate::treaty_table::Outcome::Signed(_))
                    && c.date.to_days() >= since
                    && {
                        // The leaders of the two sides: a separate peace of an ally comes later.
                        let on = |side: &[u16], t: &str| {
                            side.first().is_some_and(|&m| tag(m) == Some(t))
                        };
                        (on(&w.side_a, &c.from) && on(&w.side_b, &c.to))
                            || (on(&w.side_b, &c.from) && on(&w.side_a, &c.to))
                    }
            })
        })
        .map(|w| w.id)
        .collect()
}

/// The pairs of countries at war, (a, b) with a < b, sorted and without repeats: every member
/// of one side of a war against every member of the other, the allies that joined by a call
/// included. Empty in peace. The trade reads it once a week to close these pairs (owner's
/// decision CT7; `econ::trade::Geography::closures`).
pub fn war_pairs(world: &GameWorld) -> Vec<(u16, u16)> {
    let mut out: Vec<(u16, u16)> = world
        .war
        .conflicts
        .wars
        .iter()
        .flat_map(|w| {
            w.side_a
                .iter()
                .flat_map(move |&a| w.side_b.iter().map(move |&b| (a.min(b), a.max(b))))
        })
        .filter(|(a, b)| a != b)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// After a peace, the occupation between the sides ends: the regions one side held of the
/// other go back to their owner (what the peace ceded changed owner by the treaty's bridge),
/// and the advances and postures between them are forgotten.
pub fn end_occupation(f: &mut Fighting, world: &GameWorld, a: &[u16], b: &[u16]) {
    let regions = &world.econ.regions;
    let between =
        |x: u16, y: u16| (a.contains(&x) && b.contains(&y)) || (b.contains(&x) && a.contains(&y));
    f.control
        .retain(|(r, by)| !between(*by, regions[*r as usize].market));
    f.advance.retain(|(x, y, _)| {
        !between(
            f.control
                .binary_search_by_key(x, |(k, _)| *k)
                .map_or(regions[*x as usize].market, |i| f.control[i].1),
            regions[*y as usize].market,
        )
    });
    f.postures.retain(|(c, e, _)| !between(*c, *e));
    let holder = |r: u32| {
        f.control
            .binary_search_by_key(&r, |(k, _)| *k)
            .map_or(regions[r as usize].market, |i| f.control[i].1)
    };
    let still: Vec<_> = f
        .still
        .iter()
        .copied()
        .filter(|(x, y, _)| !between(holder(*x), holder(*y)))
        .collect();
    f.still = still;
}

/// `date` is the day of the week (winter in cold states runs from December to February; a
/// peace signed in the last seven days ends its war).
pub fn fight_week(
    setup: &FightSetup,
    mil: &Military,
    world: &GameWorld,
    date: laplace_schelling::Date,
) -> FightWeek {
    let month = date.month;
    let mut out = FightWeek {
        fighting: world.war.fighting.clone(),
        ..Default::default()
    };
    if world.war.conflicts.wars.is_empty() {
        return out;
    }
    out.ended = peace_signed(world, date);
    for w in world
        .war
        .conflicts
        .wars
        .iter()
        .filter(|w| out.ended.contains(&w.id))
    {
        end_occupation(&mut out.fighting, world, &w.side_a, &w.side_b);
    }
    let ended = out.ended.clone();
    let at_war = |a: u16, b: u16| {
        world.war.conflicts.wars.iter().any(|w| {
            !ended.contains(&w.id)
                && ((w.side_a.contains(&a) && w.side_b.contains(&b))
                    || (w.side_a.contains(&b) && w.side_b.contains(&a)))
        })
    };
    let regions = &world.econ.regions;
    // Map state → region.
    let by_state: BTreeMap<u16, u32> = regions
        .iter()
        .enumerate()
        .map(|(i, r)| (r.state, i as u32))
        .collect();
    let player = world
        .countries
        .iter()
        .position(|(id, _)| Some(id) == world.player)
        .map(|i| i as u16);
    let controller = |r: u32| out.fighting.controller(r, regions[r as usize].market);
    let enemies = at_war;
    // The segments: (from, into) where `from`'s holder is at war with `into`'s, each pair once,
    // the lower-indexed holder's region first.
    let mut segments: Vec<(u32, u32, u16, u16)> = Vec::new();
    for (ri, r) in regions.iter().enumerate() {
        let a = controller(ri as u32);
        let Some(near) = setup.neighbours.get(&r.state) else {
            continue;
        };
        for s in near {
            let Some(&si) = by_state.get(s) else {
                continue;
            };
            let b = controller(si);
            if a < b && enemies(a, b) {
                segments.push((ri as u32, si, a, b));
            }
        }
    }
    segments.sort();
    segments.dedup();
    if segments.is_empty() {
        out.fighting.last_week.clear();
        out.fighting.still.clear();
        let held = setup.conduct.as_ref().map(|k| {
            super::occupation::garrisons(&k.rules.occupation, world, &out.fighting, |c| {
                world
                    .war
                    .countries
                    .get(c as usize)
                    .map_or(0, |cw| strength(mil, cw).men)
            })
        });
        conduct_week(setup, world, &mut out, held.as_ref(), player);
        return out;
    }
    // Each country's land strength, spread evenly over its segments.
    let mut count: BTreeMap<u16, u64> = BTreeMap::new();
    for &(_, _, a, b) in &segments {
        *count.entry(a).or_default() += 1;
        *count.entry(b).or_default() += 1;
    }
    // The garrisons of the occupied land (G3) stand apart from the fronts.
    let held = setup.conduct.as_ref().map(|k| {
        super::occupation::garrisons(&k.rules.occupation, world, &out.fighting, |c| {
            world
                .war
                .countries
                .get(c as usize)
                .map_or(0, |cw| strength(mil, cw).men)
        })
    });
    let strengths: BTreeMap<u16, Strength> = count
        .keys()
        .filter_map(|&c| {
            world.war.countries.get(c as usize).map(|cw| {
                let mut s = strength(mil, cw);
                if let Some((_, g)) = held.as_ref().and_then(|h| h.get(&c))
                    && s.men > 0
                {
                    let left = s.men.saturating_sub(*g);
                    s.fire *= Fixed::from_ratio(left as i64, s.men as i64);
                    s.men = left;
                }
                (c, s)
            })
        })
        .collect();
    // Posture: the player's as set (defend by default); the AI attacks a segment where its
    // fire there is at least 1.5 times the enemy's there (G5): local superiority, so that two
    // even countries still attack where one is thinner.
    let attacks = |c: u16, e: u16, mine: Fixed, theirs: Fixed| -> bool {
        if Some(c) == player {
            return out.fighting.posture(c, e).unwrap_or(false);
        }
        mine * AI_ATTACK_RATIO.1 >= theirs * AI_ATTACK_RATIO.0 && mine.is_positive()
    };
    let share_of = |c: u16, k: u64| -> Vec<u64> {
        let s = &strengths[&c];
        apportion(s.men, &vec![Fixed::ONE; count[&c] as usize])
            .into_iter()
            .take(k as usize)
            .collect()
    };
    let men_on: BTreeMap<u16, Vec<u64>> =
        count.iter().map(|(&c, &n)| (c, share_of(c, n))).collect();
    let mut next: BTreeMap<u16, usize> = BTreeMap::new();
    let mut lost: BTreeMap<u16, u64> = BTreeMap::new();
    let winter_month = matches!(month, 12 | 1 | 2);
    let field_of = |from: u32, into: u32, attack_from_a: bool| -> LandField {
        let (sa, sb) = (regions[from as usize].state, regions[into as usize].state);
        let width_km = setup
            .width_km
            .get(&(sa.min(sb), sa.max(sb)))
            .copied()
            .unwrap_or(Fixed::ZERO);
        // The terrain is the defender's state: where the attack goes.
        let at = if attack_from_a { sb } else { sa };
        LandField {
            terrain: setup
                .terrain
                .get(&at)
                .cloned()
                .unwrap_or_else(|| TerrainMix::only(Terrain::Open)),
            winter: winter_month && setup.cold.contains(&at),
            river: setup
                .river
                .get(&(sa.min(sb), sa.max(sb)))
                .copied()
                .unwrap_or(Fixed::ZERO),
            width_km,
        }
    };
    let supply: BTreeMap<u16, Fixed> = count
        .keys()
        .map(|&c| {
            let fill = world
                .econ
                .markets
                .get(c as usize)
                .and_then(|m| m.buy_ratio.get(mil.rules.ammunition as usize))
                .copied()
                .unwrap_or(Fixed::ONE)
                .clamp(Fixed::ZERO, Fixed::ONE);
            (c, fill)
        })
        .collect();
    let side = |c: u16, men_here: u64, attack: bool, dug: Fixed| -> LandSide {
        let s = &strengths[&c];
        let part = if s.men > 0 {
            Fixed::from_ratio(men_here as i64, s.men as i64)
        } else {
            Fixed::ZERO
        };
        LandSide {
            men: men_here,
            fire: s.fire * part,
            protection: s.protection,
            training: s.training,
            // The share of the ammunition of combat the market delivered last week.
            supply: supply.get(&c).copied().unwrap_or(Fixed::ONE),
            // Neutral until the air enters (slice C2).
            air: Fixed::from_ratio(1, 2),
            posture: if attack {
                Posture::Attack
            } else {
                Posture::Defend
            },
            mobility: s.mobility,
            fortification: dug,
            landing: false,
        }
    };
    for &(from, into, a, b) in &segments {
        let ia = *next.entry(a).or_default();
        let ib = *next.entry(b).or_default();
        *next.get_mut(&a).unwrap() += 1;
        *next.get_mut(&b).unwrap() += 1;
        let ma = men_on[&a].get(ia).copied().unwrap_or(0);
        let mb = men_on[&b].get(ib).copied().unwrap_or(0);
        // The fire each side has on this segment, and how dug in the segment is.
        let local = |c: u16, m: u64| {
            let s = &strengths[&c];
            if s.men > 0 {
                s.fire * Fixed::from_ratio(m as i64, s.men as i64)
            } else {
                Fixed::ZERO
            }
        };
        let (fa, fb) = (local(a, ma), local(b, mb));
        let (att_a, att_b) = (attacks(a, b, fa, fb), attacks(b, a, fb, fa));
        let dug = Fixed::from_ratio(
            i64::from(world.war.fighting.still_weeks(from, into)),
            i64::from(mil.rules.front.dig_weeks.max(1)),
        )
        .min(Fixed::ONE);
        let (sa, sb) = (side(a, ma, att_a, dug), side(b, mb, att_b, dug));
        let field = field_of(from, into, att_a || !att_b);
        let o = land_step(&mil.rules, &field, [&sa, &sb]);
        *lost.entry(a).or_default() += o.losses[0];
        *lost.entry(b).or_default() += o.losses[1];
        out.segments.push(SegmentWeek {
            from,
            into,
            a,
            b,
            outcome: o,
        });
    }
    // Advance and control.
    for seg in &out.segments {
        let km = seg.outcome.advance_km;
        if km == Fixed::ZERO {
            continue;
        }
        let (from, into, taker) = if km.is_positive() {
            (seg.from, seg.into, seg.a)
        } else {
            (seg.into, seg.from, seg.b)
        };
        let km = km.abs();
        let f = &mut out.fighting;
        let i = match f
            .advance
            .binary_search_by_key(&(from, into), |(x, y, _)| (*x, *y))
        {
            Ok(i) => i,
            Err(i) => {
                f.advance.insert(i, (from, into, Fixed::ZERO));
                i
            }
        };
        f.advance[i].2 += km;
        let depth = setup
            .depth_km
            .get(&regions[into as usize].state)
            .copied()
            .unwrap_or(Fixed::from_int(100));
        if f.advance[i].2 >= depth {
            // The state is taken: every advance into it or out of it starts again.
            f.set_control(into, taker, regions[into as usize].market);
            f.advance.retain(|(x, y, _)| *x != into && *y != into);
        }
    }
    // Still segments dig in; a segment that moved starts again, and one that is gone is
    // forgotten.
    let still_km = mil.rules.front.still_km;
    let cap = mil.rules.front.dig_weeks;
    out.fighting.still = out
        .segments
        .iter()
        .filter(|g| g.outcome.advance_km.abs() < still_km)
        .map(|g| {
            let w = world.war.fighting.still_weeks(g.from, g.into);
            (g.from, g.into, w.saturating_add(1).min(cap))
        })
        .collect();
    out.fighting.still.sort();
    out.fighting.still.dedup_by_key(|x| (x.0, x.1));
    out.fighting.last_week = out.segments.clone();
    conduct_week(setup, world, &mut out, held.as_ref(), player);
    // Losses: a share of each country's land elements.
    for (c, l) in lost {
        if l == 0 {
            continue;
        }
        let total = strengths[&c].men.max(1);
        out.lost_share
            .push((c, Fixed::from_ratio(l.min(total) as i64, total as i64)));
        out.lost_men.push((c, l.min(total)));
        match out.fighting.losses.binary_search_by_key(&c, |(k, _)| *k) {
            Ok(i) => out.fighting.losses[i].1 += l,
            Err(i) => out.fighting.losses.insert(i, (c, l)),
        }
    }
    out
}

/// [`Fighting`] as version 26 of the world wrote it, before the entrenchment.
#[derive(Default, Deserialize)]
pub(crate) struct FightingBeforeStill {
    control: Vec<(u32, u16)>,
    advance: Vec<(u32, u32, Fixed)>,
    postures: Vec<(u16, u16, bool)>,
    losses: Vec<(u16, u64)>,
    last_week: Vec<SegmentWeek>,
}

impl From<FightingBeforeStill> for Fighting {
    fn from(f: FightingBeforeStill) -> Self {
        Self {
            control: f.control,
            advance: f.advance,
            postures: f.postures,
            losses: f.losses,
            last_week: f.last_week,
            ..Default::default()
        }
    }
}

/// [`Fighting`] as version 27 of the world wrote it, before the war support and the occupation.
#[derive(Default, Deserialize)]
pub(crate) struct FightingBeforeSupport {
    control: Vec<(u32, u16)>,
    advance: Vec<(u32, u32, Fixed)>,
    postures: Vec<(u16, u16, bool)>,
    losses: Vec<(u16, u64)>,
    last_week: Vec<SegmentWeek>,
    still: Vec<(u32, u32, u16)>,
}

impl From<FightingBeforeSupport> for Fighting {
    fn from(f: FightingBeforeSupport) -> Self {
        Self {
            control: f.control,
            advance: f.advance,
            postures: f.postures,
            losses: f.losses,
            last_week: f.last_week,
            still: f.still,
            ..Default::default()
        }
    }
}

/// The conduct of the war after the week's combat: occupation, devastation and support; the
/// wars the side that declared gave up end here, as a peace does.
fn conduct_week(
    setup: &FightSetup,
    world: &GameWorld,
    out: &mut FightWeek,
    held: Option<&BTreeMap<u16, (u64, u64)>>,
    player: Option<u16>,
) {
    let Some(k) = setup.conduct.as_ref() else {
        return;
    };
    let empty = BTreeMap::new();
    out.transfers = super::occupation::occupation_week(
        &k.rules.occupation,
        world,
        &mut out.fighting,
        held.unwrap_or(&empty),
    );
    out.devastation = super::occupation::devastation_week(
        &k.rules.occupation,
        world,
        &mut out.fighting,
        &out.segments,
    );
    out.support = super::support::support_week(
        k,
        &setup.capitals,
        world,
        &mut out.fighting,
        &out.segments,
        &out.ended,
        player,
    );
    for w in world
        .war
        .conflicts
        .wars
        .iter()
        .filter(|w| out.support.withdrawn.contains(&w.id))
    {
        end_occupation(&mut out.fighting, world, &w.side_a, &w.side_b);
        out.ended.push(w.id);
    }
    // The occupation and the wrecks follow the control that is left.
    let control = out.fighting.control.clone();
    out.fighting
        .occupied
        .retain(|o| control.iter().any(|&(r, by)| r == o.region && by == o.by));
}

/// Of land casualties (killed, wounded and captured), the killed: about one in four, the order
/// of the First World War's armies, with some three wounded for each death (Clodfelter, *Warfare
/// and Armed Conflicts*). The wounded go back; the invalids among them come with the ledger.
pub const KILLED_PER_MILLE: i64 = 250;

/// Takes `killed` men of `country` out of the population: from the soldiers' pops of the
/// regions where its soldiers are, by the soldiers each holds, and from the pyramid's groups of
/// 20 to 39 years in the same regions.
pub fn bury(world: &mut GameWorld, country: u16, killed: u64) {
    use crate::econ::prof;
    let regions: Vec<usize> = (0..world.econ.regions.len())
        .filter(|&i| world.econ.regions[i].market == country)
        .collect();
    let weights: Vec<Fixed> = regions
        .iter()
        .map(|&i| Fixed::from_int(world.war.soldiers.get(i).copied().unwrap_or(0).max(0)))
        .collect();
    if !weights.iter().any(|w| w.is_positive()) {
        return;
    }
    for (&ri, dead) in regions.iter().zip(apportion(killed, &weights)) {
        let dead = dead as i64;
        if dead == 0 {
            continue;
        }
        let pop = &mut world.econ.regions[ri].pops[prof::SOLDIER];
        let taken = dead.min(pop.size.max(0));
        pop.size -= taken;
        if let Some(ages) = world.demography.ages.get_mut(ri) {
            // Groups 4 to 7: 20 to 39 years.
            let band: i64 = ages[4..8].iter().sum();
            if band > 0 {
                let mut left = taken;
                for a in &mut ages[4..8] {
                    let d = (taken * *a / band).min(*a).min(left);
                    *a -= d;
                    left -= d;
                }
                ages[4] = (ages[4] - left).max(0);
            }
        }
    }
}

/// Takes `share` of every land formation of `c`, elements and equipment alike.
pub fn take_losses(mil: &Military, c: &mut CountryWar, share: Fixed) {
    let keep = (Fixed::ONE - share).max(Fixed::ZERO);
    for f in c
        .formations
        .iter_mut()
        .filter(|f| mil.types[f.unit as usize].domain == Domain::Land)
    {
        f.elements *= keep;
        for e in &mut f.equipment {
            e.1 *= keep;
        }
    }
    c.formations.retain(|f| f.elements.is_positive());
}

impl HashState for Fighting {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.control.len());
        for (r, c) in &self.control {
            h.write_u32(*r);
            h.write_u16(*c);
        }
        h.write_len(self.advance.len());
        for (a, b, km) in &self.advance {
            h.write_u32(*a);
            h.write_u32(*b);
            km.hash_state(h);
        }
        h.write_len(self.postures.len());
        for (c, e, a) in &self.postures {
            h.write_u16(*c);
            h.write_u16(*e);
            a.hash_state(h);
        }
        h.write_len(self.losses.len());
        for (c, l) in &self.losses {
            h.write_u16(*c);
            h.write_u64(*l);
        }
        h.write_len(self.last_week.len());
        for g in &self.last_week {
            h.write_u32(g.from);
            h.write_u32(g.into);
            h.write_u16(g.a);
            h.write_u16(g.b);
            let o = &g.outcome;
            o.engaged.hash_state(h);
            for p in o.power {
                p.hash_state(h);
            }
            for l in o.losses {
                h.write_u64(l);
            }
            o.advance_km.hash_state(h);
        }
        h.write_len(self.still.len());
        for (a, b, w) in &self.still {
            h.write_u32(*a);
            h.write_u32(*b);
            h.write_u16(*w);
        }
        h.write_len(self.support.len());
        for x in &self.support {
            x.hash_state(h);
        }
        h.write_len(self.occupied.len());
        for x in &self.occupied {
            x.hash_state(h);
        }
        h.write_len(self.wrecks.len());
        for x in &self.wrecks {
            x.hash_state(h);
        }
    }
}
