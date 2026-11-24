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

use super::calc::{
    AirGroup, AirSide, LandField, LandSide, Posture, Segment, TerrainMix, air_step, apportion,
    land_step, pursuit_km,
};
use super::catalogue::{Domain, Military, Mobility, Terrain};
use super::doctrine::Generation;
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
    /// The laws, for the changes of the form of government (step 4).
    pub politics: Option<Arc<crate::politics::Politics>>,
    /// Index in the tree of `command.doctrine_tech`, the research that opens the doctrine.
    pub doctrine_tech: Option<u16>,
    /// The goods that make a formation move on wheels or tracks (`peace.ron`,
    /// `composition.vehicles`), for the motorization of the generation.
    pub vehicles: Vec<u16>,
    /// The colonial war (G6), when `war.ron` has its rules.
    pub colonial: Option<Arc<super::colonial::ColonialSetup>>,
    /// The war at sea (slice C2), when the sea zones exist.
    pub sea: Option<Arc<super::sea::SeaSetup>>,
    /// The nuclear programme's technologies, resolved (N1, N2).
    pub programme: Option<Arc<super::programme::ProgrammeSetup>>,
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
    /// The generation of each army that has fought, by country (step 4 of the front's pace):
    /// kept from month to month because it moves up and down with a gap (`doctrine.up`,
    /// `doctrine.down`), with the weeks of war behind it.
    pub doctrine: Vec<ArmyDoctrine>,
}

/// The generation of a country's army and its experience of war.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmyDoctrine {
    pub country: u16,
    pub generation: super::doctrine::Generation,
    /// Weeks of war behind the army on `as_of`, decayed by `doctrine.experience_half_life_years`.
    pub experience_weeks: Fixed,
    /// The day (`Date::to_days`) the experience was counted.
    pub as_of: i64,
    /// Share of the officer corps formed to act on the commander's intent, 0 to 1 (decision G7);
    /// zero and unused without a recorded tradition.
    pub intent: Fixed,
    /// The army has a tradition of command on record (`command.tradition_1925`).
    pub tradition: bool,
    /// Until this day the schools revise the doctrine after a defeat, cohorts faster; 0 for
    /// none (the calendar's days are negative before 2000).
    pub reform_until: i64,
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
            && self.doctrine.is_empty()
    }

    /// The generation kept for `country`'s army, if it has fought.
    pub fn doctrine_of(&self, country: u16) -> Option<&ArmyDoctrine> {
        self.doctrine
            .binary_search_by_key(&country, |d| d.country)
            .ok()
            .map(|i| &self.doctrine[i])
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
    /// For the screen, sides A and B: the mobility each moved at on this segment, and the share
    /// of its men here in its mobile class, per mille. Not saved: after a load they read the
    /// next week's.
    #[serde(skip)]
    pub mobility: Option<[Mobility; 2]>,
    #[serde(skip)]
    pub mobile_share: [u16; 2],
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
    /// Aircraft lost over the fronts this week, (country, type, generation, aircraft).
    pub air_lost: Vec<(u16, u16, u8, Fixed)>,
    pub segments: Vec<SegmentWeek>,
    /// What the support decided: governments fallen, peace asked for, wars given up.
    pub support: super::support::SupportOut,
    /// Occupation money: (owner, occupier, amount).
    pub transfers: Vec<super::occupation::Transfer>,
    pub devastation: super::occupation::Devastation,
    /// Scorched earth's state after the week, with its switch on.
    pub scorch: Option<super::scorch::ScorchState>,
}

/// The land strength of a country: men, the fire and protection of what it holds, and its
/// average training, over every land formation.
struct Strength {
    men: u64,
    fire: Fixed,
    protection: Fixed,
    training: Fixed,
    mobility: Mobility,
    /// The mobile class (horse, wheels, tracks) with the most men, if any: what a third
    /// generation puts on its focus segment.
    mobile: Option<Mobility>,
    /// The land men of that class.
    mobile_men: u64,
}

/// The mobile class (horse, wheels, tracks) that holds the most land men of the country, if
/// it has any; on a tie the faster.
pub fn mobile_mobility(mil: &Military, formations: &[Formation]) -> Option<Mobility> {
    let mut by: BTreeMap<Mobility, i64> = BTreeMap::new();
    for f in formations {
        if mil.types[f.unit as usize].domain != Domain::Land || !f.elements.is_positive() {
            continue;
        }
        let m = mil.spec(f.unit, f.generation as usize).mobility;
        if matches!(m, Mobility::Horse | Mobility::Wheeled | Mobility::Tracked) {
            *by.entry(m).or_default() += men(mil, f);
        }
    }
    by.into_iter()
        .filter(|(_, n)| *n > 0)
        .max_by(|x, y| x.1.cmp(&y.1).then(x.0.cmp(&y.0)))
        .map(|(m, _)| m)
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
        mobile: mobile_mobility(mil, &c.formations),
        mobile_men: {
            let class = mobile_mobility(mil, &c.formations);
            land.iter()
                .filter(|f| Some(mil.spec(f.unit, f.generation as usize).mobility) == class)
                .map(|f| men(mil, f))
                .sum::<i64>()
                .max(0) as u64
        },
    }
}

/// The mobility a segment moves at when a second-generation army concentrates its mobile class
/// there (item 12): that class's when its men are at least `share` per mille of the men on the
/// segment, else the army's majority. `mobile_here` of `men_here` are of the class.
pub fn concentrated_mobility(
    majority: Mobility,
    mobile: Option<Mobility>,
    mobile_here: u64,
    men_here: u64,
    share: u32,
) -> Mobility {
    match mobile {
        Some(m)
            if men_here > 0
                && u128::from(mobile_here) * 1000 >= u128::from(men_here) * u128::from(share) =>
        {
            m
        }
        _ => majority,
    }
}

/// The garrison unit type of the occupation rules (item 16), if any.
fn garrison_unit(mil: &Military, r: &super::support::OccupationRules) -> Option<u16> {
    r.garrison_unit.as_deref().and_then(|k| mil.unit(k))
}

/// The men and the fire of a country's formations of the garrison `unit`, colonial ones apart
/// (item 16): they stand as the occupation's garrison before the field army.
pub fn posts(mil: &Military, c: &CountryWar, unit: Option<u16>) -> (u64, Fixed) {
    let Some(unit) = unit else {
        return (0, Fixed::ZERO);
    };
    let list: Vec<&Formation> = c
        .formations
        .iter()
        .filter(|f| f.unit == unit && !f.colonial && f.elements.is_positive())
        .collect();
    let m: i64 = list.iter().map(|f| men(mil, f)).sum();
    let equipment: Vec<(u16, Fixed)> = list.iter().flat_map(|f| f.equipment.clone()).collect();
    (m.max(0) as u64, mil.land_fire(&equipment))
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

/// The blockades at sea this week, (blockader, blockaded), sorted: who holds the sea lanes of
/// whom, for the screen. The trade closes the pairs of [`blockaded_pairs`] (`war::sea`).
pub fn blockaded(world: &GameWorld) -> Vec<(u16, u16)> {
    world.war.sea.blockades.clone()
}

/// The trade pairs a blockade cuts this week, (blockaded, partner) with the lower index first,
/// sorted and without repeats: the pairs whose cheaper way is by sea (`Geography::by_sea`) and
/// whose every sea route passes a zone the blockader controls (`SeaGraph`, with those zones
/// closed). A pair that goes by land, or that has a free route (Germany and Sweden by the
/// Baltic while the North Sea is British), stays open. The trade closes and reopens them
/// (owner's decision CT7).
pub fn blockaded_pairs(world: &GameWorld) -> Vec<(u16, u16)> {
    world.war.sea.cut.clone()
}

/// A month of the command culture and the generation of every army (step 4; decision G7):
/// the officers formed in the schools' doctrine a class a year (in January), the experience of war
/// decaying, a defeat that opens a reform, and the generation moving up or down with its gap.
/// Runs the first week of each month while the war is switched on.
pub fn doctrine_month(
    setup: &FightSetup,
    mil: &Military,
    world: &GameWorld,
    today: i64,
    january: bool,
    f: &mut Fighting,
) {
    let cmd = &mil.rules.command;
    let tradition_of = |ci: usize| super::army::tradition_1925(world, mil, ci);
    let industrial = |ci: usize| {
        setup.doctrine_tech.is_some_and(|t| {
            world
                .research
                .stage
                .get(ci)
                .and_then(|s| s.get(usize::from(t)))
                .is_some_and(|&st| st >= crate::research::Stage::Industrial)
        })
    };
    for (ci, c) in world.war.countries.iter().enumerate() {
        let has_land = c.formations.iter().any(|x| {
            mil.types[x.unit as usize].domain == super::catalogue::Domain::Land
                && x.elements.is_positive()
        });
        if !has_land {
            continue;
        }
        let tradition = tradition_of(ci);
        let at = f.doctrine.binary_search_by_key(&(ci as u16), |d| d.country);
        let mut d = match at {
            Ok(i) => f.doctrine[i],
            Err(_) => ArmyDoctrine {
                country: ci as u16,
                generation: super::doctrine::Generation::Second,
                experience_weeks: Fixed::ZERO,
                as_of: today,
                intent: tradition.unwrap_or(Fixed::ZERO),
                tradition: tradition.is_some(),
                reform_until: 0,
            },
        };
        d.experience_weeks = super::army::decayed(mil, d.experience_weeks, today - d.as_of);
        d.as_of = today;
        if f.support_of(ci as u16).is_some_and(|s| s.fallen.is_some()) {
            d.reform_until = today + 365 * i64::from(cmd.reform_years);
        }
        if let Some(t) = tradition
            && january
        {
            // What the schools teach: the doctrine once the country makes its research, the
            // tradition until then. A class a year leaves the schools, a career's share of the
            // corps, spread as far as the army trains.
            let school = if industrial(ci) { Fixed::ONE } else { t };
            let mut rate = Fixed::from_ratio(1, i64::from(cmd.officer_career_years.max(1)));
            if d.reform_until != 0 && today < d.reform_until {
                rate *= Fixed::from_int(i64::from(cmd.defeat_reform_speed.max(1)));
            }
            let training = super::army::land_training(mil, c);
            d.intent += (school - d.intent) * rate * training;
        }
        let r = super::doctrine::readiness(
            &mil.rules,
            &super::army::inputs_with(
                world,
                mil,
                setup.politics.as_deref(),
                &setup.vehicles,
                ci,
                &d,
                today,
            ),
        );
        d.generation = super::doctrine::next_generation(&mil.rules, d.generation, &r);
        match at {
            Ok(i) => f.doctrine[i] = d,
            Err(i) => f.doctrine.insert(i, d),
        }
    }
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
    // The generation of the armies, the first week of each month (step 4).
    if date.day <= 7 {
        doctrine_month(
            setup,
            mil,
            world,
            date.to_days(),
            date.month == 1,
            &mut out.fighting,
        );
    }
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
                world.war.countries.get(c as usize).map_or((0, 0), |cw| {
                    let (posts, _) = posts(mil, cw, garrison_unit(mil, &k.rules.occupation));
                    (posts, strength(mil, cw).men.saturating_sub(posts))
                })
            })
        });
        conduct_week(
            setup,
            world,
            &mut out,
            held.as_ref(),
            player,
            date.to_days(),
        );
        return out;
    }
    // Each country's land strength, spread evenly over its segments. A metropole fighting its
    // colony overseas (G6) has there only its garrison and the men it landed: those segments
    // are a theatre of their own, keyed by the enemy; the rest share the main army (`u16::MAX`).
    let caps: BTreeMap<(u16, u16), u64> = world
        .war
        .colonial
        .iter()
        .filter(|k| !k.peace)
        .filter_map(|k| super::colonial::theatre_men(world, k).map(|m| ((k.owner, k.colony), m)))
        .collect();
    let theatre = |c: u16, e: u16| {
        if caps.contains_key(&(c, e)) {
            e
        } else {
            u16::MAX
        }
    };
    let mut count: BTreeMap<(u16, u16), u64> = BTreeMap::new();
    for &(_, _, a, b) in &segments {
        *count.entry((a, theatre(a, b))).or_default() += 1;
        *count.entry((b, theatre(b, a))).or_default() += 1;
    }
    let fighters: std::collections::BTreeSet<u16> = count.keys().map(|k| k.0).collect();
    // The garrisons of the occupied land (G3) stand apart from the fronts.
    let held = setup.conduct.as_ref().map(|k| {
        super::occupation::garrisons(&k.rules.occupation, world, &out.fighting, |c| {
            world.war.countries.get(c as usize).map_or((0, 0), |cw| {
                let (posts, _) = posts(mil, cw, garrison_unit(mil, &k.rules.occupation));
                (posts, strength(mil, cw).men.saturating_sub(posts))
            })
        })
    });
    let strengths: BTreeMap<u16, Strength> = fighters
        .iter()
        .filter_map(|&c| {
            world.war.countries.get(c as usize).map(|cw| {
                let mut s = strength(mil, cw);
                if let Some((_, g)) = held.as_ref().and_then(|h| h.get(&c))
                    && s.men > 0
                {
                    // The garrison formations stand first, with their own fire; the rest of
                    // the garrison comes out of the field army, its fire in proportion.
                    let unit = setup
                        .conduct
                        .as_ref()
                        .and_then(|k| garrison_unit(mil, &k.rules.occupation));
                    let (pm, pf) = posts(mil, cw, unit);
                    let from_posts = (*g).min(pm);
                    let from_field = g - from_posts;
                    let post_fire = if pm > 0 {
                        pf * Fixed::from_ratio(from_posts as i64, pm as i64)
                    } else {
                        Fixed::ZERO
                    };
                    let (field_men, field_fire) = (s.men.saturating_sub(pm), s.fire - pf);
                    let field_cut = if field_men > 0 {
                        field_fire * Fixed::from_ratio(from_field as i64, field_men as i64)
                    } else {
                        Fixed::ZERO
                    };
                    s.fire = (s.fire - post_fire - field_cut).max(Fixed::ZERO);
                    s.men = s.men.saturating_sub(*g);
                }
                (c, s)
            })
        })
        .collect();
    // A week of war adds to the experience of every army on a front.
    for &c in &fighters {
        if let Ok(i) = out
            .fighting
            .doctrine
            .binary_search_by_key(&c, |d| d.country)
        {
            out.fighting.doctrine[i].experience_weeks += Fixed::ONE;
        }
    }
    // Posture: the player's as set (defend by default); the AI attacks a segment where its
    // fire there is at least 1.5 times the enemy's there (G5): local superiority, so that two
    // even countries still attack where one is thinner.
    let attacks = |c: u16, e: u16, mine: Fixed, theirs: Fixed| -> bool {
        if Some(c) == player {
            return out.fighting.posture(c, e).unwrap_or(false);
        }
        mine * AI_ATTACK_RATIO.1 >= theirs * AI_ATTACK_RATIO.0 && mine.is_positive()
    };
    // The generation each army fights with this week: a third-generation army fights as a
    // second short of fuel (`doctrine.fuel_min`; radio is not a good yet).
    let generation_of = |c: u16| -> Generation {
        let kept = out
            .fighting
            .doctrine_of(c)
            .map_or(Generation::Second, |d| d.generation);
        let fuel = world
            .econ
            .markets
            .get(c as usize)
            .and_then(|m| m.buy_ratio.get(mil.rules.fuel as usize))
            .copied()
            .unwrap_or(Fixed::ONE)
            .clamp(Fixed::ZERO, Fixed::ONE);
        super::doctrine::effective_generation(&mil.rules, kept, Fixed::ONE, fuel).0
    };
    let generations: BTreeMap<u16, Generation> =
        fighters.iter().map(|&c| (c, generation_of(c))).collect();
    // Each theatre's segments in the order the loop below takes them: width and the enemy's
    // even share of fire, for the third generation's focus.
    let mut segs_of: BTreeMap<(u16, u16), Vec<Segment>> = BTreeMap::new();
    for &(from, into, a, b) in &segments {
        let (sa, sb) = (regions[from as usize].state, regions[into as usize].state);
        let width = setup
            .width_km
            .get(&(sa.min(sb), sa.max(sb)))
            .copied()
            .unwrap_or(Fixed::ONE);
        for (me, enemy) in [(a, b), (b, a)] {
            let e = &strengths[&enemy];
            let enemy_segments: u64 = count
                .iter()
                .filter(|(k, _)| k.0 == enemy)
                .map(|(_, n)| *n)
                .sum();
            segs_of
                .entry((me, theatre(me, enemy)))
                .or_default()
                .push(Segment {
                    length_km: width,
                    enemy_power: e.fire / Fixed::from_int(enemy_segments.max(1) as i64),
                    objective: false,
                });
        }
    }
    // The men of each theatre: a colonial one has its cap, the main army the rest.
    let pool = |c: u16, t: u16| -> u64 {
        let all = strengths[&c].men;
        if t != u16::MAX {
            return caps[&(c, t)].min(all);
        }
        let overseas: u64 = count
            .keys()
            .filter(|k| k.0 == c && k.1 != u16::MAX)
            .map(|k| caps[k].min(all))
            .sum();
        all.saturating_sub(overseas)
    };
    // The men on each segment of a theatre: evenly for a second-generation army; a third
    // concentrates on the focus segment, the weakest per km (`doctrine::allocate_front`).
    let mut focus_of: BTreeMap<(u16, u16), usize> = BTreeMap::new();
    let men_on: BTreeMap<(u16, u16), Vec<u64>> = count
        .iter()
        .map(|(&(c, t), &n)| {
            let men = pool(c, t);
            if generations[&c] == Generation::Third && n > 1 {
                let shares = super::doctrine::allocate_front(
                    &mil.rules,
                    men,
                    &segs_of[&(c, t)],
                    Generation::Third,
                );
                if let Some(i) = shares.iter().position(|x| x.focus) {
                    focus_of.insert((c, t), i);
                }
                ((c, t), shares.iter().map(|x| x.elements).collect())
            } else {
                ((c, t), apportion(men, &vec![Fixed::ONE; n as usize]))
            }
        })
        .collect();
    // Where a second-generation army puts its mobile class (item 12): the segment where the
    // enemy is weakest per km, as the third generation's focus, without the concentration of
    // its other men.
    let conc_of: BTreeMap<(u16, u16), usize> = count
        .keys()
        .filter(|(c, _)| generations[c] == Generation::Second)
        .filter_map(|k| {
            let segs = segs_of.get(k)?;
            (!segs.is_empty()).then(|| (*k, super::doctrine::focus_segment(segs)))
        })
        .collect();
    // The mobility a side moves at on segment `i` of theatre `key`, and the share of its men
    // there in its mobile class, per mille.
    let mobility_on = |c: u16, key: (u16, u16), i: usize, men_here: u64| -> (Mobility, u16) {
        let s = &strengths[&c];
        let focus = focus_of.get(&key) == Some(&i);
        let conc = conc_of.get(&key) == Some(&i);
        if !(focus || conc) || s.men == 0 {
            return (s.mobility, 0);
        }
        let pool_mobile =
            (u128::from(s.mobile_men) * u128::from(pool(c, key.1)) / u128::from(s.men)) as u64;
        let here = pool_mobile.min(men_here);
        let share = if men_here > 0 {
            (u128::from(here) * 1000 / u128::from(men_here)) as u16
        } else {
            0
        };
        // The focus segment of a third-generation army gets its mobile troops, as before.
        let m = if focus {
            s.mobile.unwrap_or(s.mobility)
        } else {
            concentrated_mobility(
                s.mobility,
                s.mobile,
                here,
                men_here,
                mil.rules.doctrine.mobile_share,
            )
        };
        (m, share)
    };
    let mut next: BTreeMap<(u16, u16), usize> = BTreeMap::new();
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
    let supply: BTreeMap<u16, Fixed> = fighters
        .iter()
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
    // The air over the fronts (slice C2): each country's aircraft spread evenly over its
    // segments; on each, `air_step` gives the superiority the land step reads.
    let mut air_segs: BTreeMap<u16, u64> = BTreeMap::new();
    for &(_, _, a, b) in &segments {
        *air_segs.entry(a).or_default() += 1;
        *air_segs.entry(b).or_default() += 1;
    }
    type Wing = (u16, u8, AirGroup);
    let wings: BTreeMap<u16, (Vec<Wing>, Fixed)> = air_segs
        .keys()
        .filter_map(|&c| {
            let cw = world.war.countries.get(c as usize)?;
            let (mut trained, mut planes) = (Fixed::ZERO, 0i64);
            let groups: Vec<Wing> = cw
                .formations
                .iter()
                .filter(|f| mil.types[f.unit as usize].domain == Domain::Air)
                .filter_map(|f| {
                    let n = f.elements.to_int_floor().max(0) as u64;
                    trained += f.training * n as i64;
                    planes += n as i64;
                    (n > 0).then(|| {
                        (
                            f.unit,
                            f.generation,
                            mil.air_group(f.unit, f.generation as usize, n),
                        )
                    })
                })
                .collect();
            let training = if planes > 0 {
                trained / Fixed::from_int(planes)
            } else {
                Fixed::ONE
            };
            (!groups.is_empty()).then_some((c, (groups, training)))
        })
        .collect();
    let air_side = |c: u16| -> (AirSide, Vec<(u16, u8)>) {
        let n = air_segs.get(&c).copied().unwrap_or(1).max(1);
        let mut groups = Vec::new();
        let mut keys = Vec::new();
        let mut training = Fixed::ONE;
        if let Some((ws, t)) = wings.get(&c) {
            training = *t;
            for (u, g, w) in ws {
                let here = w.count / n;
                if here > 0 {
                    groups.push(AirGroup {
                        count: here,
                        ..w.clone()
                    });
                    keys.push((*u, *g));
                }
            }
        }
        (
            AirSide {
                groups,
                training,
                // Fuel is not followed yet: full.
                supply: Fixed::ONE,
            },
            keys,
        )
    };
    let mut air_lost: BTreeMap<(u16, u16, u8), u64> = BTreeMap::new();
    let side = |c: u16, men_here: u64, attack: bool, dug: Fixed, mobility: Mobility| -> LandSide {
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
            mobility,
            fortification: dug,
            landing: false,
        }
    };
    for &(from, into, a, b) in &segments {
        let (ka, kb) = ((a, theatre(a, b)), (b, theatre(b, a)));
        let ia = *next.entry(ka).or_default();
        let ib = *next.entry(kb).or_default();
        *next.get_mut(&ka).unwrap() += 1;
        *next.get_mut(&kb).unwrap() += 1;
        let ma = men_on[&ka].get(ia).copied().unwrap_or(0);
        let mb = men_on[&kb].get(ib).copied().unwrap_or(0);
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
        let (fa_focus, fb_focus) = (
            focus_of.get(&ka) == Some(&ia),
            focus_of.get(&kb) == Some(&ib),
        );
        let ((mob_a, share_a), (mob_b, share_b)) =
            (mobility_on(a, ka, ia, ma), mobility_on(b, kb, ib, mb));
        let (mut sa, mut sb) = (
            side(a, ma, att_a, dug, mob_a),
            side(b, mb, att_b, dug, mob_b),
        );
        // The air over this segment: neutral (one half each) when neither side flies.
        if wings.contains_key(&a) || wings.contains_key(&b) {
            let ((xa, a_keys), (xb, b_keys)) = (air_side(a), air_side(b));
            let o = air_step(&mil.rules, [&xa, &xb]);
            sa.air = o.superiority[0];
            sb.air = o.superiority[1];
            for (c, keys, lost) in [(a, &a_keys, &o.losses[0]), (b, &b_keys, &o.losses[1])] {
                for ((u, g), n) in keys.iter().zip(lost) {
                    if *n > 0 {
                        *air_lost.entry((c, *u, *g)).or_default() += n;
                    }
                }
            }
        }
        let field = field_of(from, into, att_a || !att_b);
        let mut o = land_step(&mil.rules, &field, [&sa, &sb]);
        // Pursuit (step 6): the side that advanced here last week and attacks with the ratio
        // marches on at its marching pace.
        let last = world
            .war
            .fighting
            .last_week
            .iter()
            .find(|g| g.from == from && g.into == into)
            .map_or(Fixed::ZERO, |g| g.outcome.advance_km);
        let still = mil.rules.front.still_km;
        let pa = pursuit_km(&mil.rules, &field, &sa, o.power, last > still);
        let pb = pursuit_km(
            &mil.rules,
            &field,
            &sb,
            [o.power[1], o.power[0]],
            last < -still,
        );
        // Each side's pursuit only ever adds to its own advance (positive for A, negative for B).
        if pa.is_positive() && pa > o.advance_km {
            o.advance_km = pa;
        }
        if pb.is_positive() && -pb < o.advance_km {
            o.advance_km = -pb;
        }
        // A third-generation attacker breaks the focus segment at three times the power and
        // exploits beyond the line (`doctrine::breakthrough`).
        for (i, (c, me, focus, attack)) in [(a, &sa, fa_focus, att_a), (b, &sb, fb_focus, att_b)]
            .into_iter()
            .enumerate()
        {
            if !(focus && attack) {
                continue;
            }
            let bt = super::doctrine::breakthrough(
                &mil.rules,
                &field,
                &super::doctrine::BreakthroughInputs {
                    generation: generations[&c],
                    attacker_power: o.power[i],
                    defender_power: o.power[1 - i],
                    defender_reserve: Fixed::ZERO,
                    mobility: me.mobility,
                },
            );
            if bt.happened {
                if i == 0 {
                    o.advance_km += bt.advance_km;
                } else {
                    o.advance_km -= bt.advance_km;
                }
            }
        }
        *lost.entry(a).or_default() += o.losses[0];
        *lost.entry(b).or_default() += o.losses[1];
        out.segments.push(SegmentWeek {
            from,
            into,
            a,
            b,
            outcome: o,
            mobility: Some([mob_a, mob_b]),
            mobile_share: [share_a, share_b],
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
    conduct_week(
        setup,
        world,
        &mut out,
        held.as_ref(),
        player,
        date.to_days(),
    );
    out.air_lost = air_lost
        .into_iter()
        .map(|((c, u, g), n)| (c, u, g, Fixed::from_int(n as i64)))
        .collect();
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

/// [`Fighting`] as versions 28 and 29 of the world wrote it, before the generation of the armies.
#[derive(Default, Deserialize)]
pub(crate) struct FightingBeforeDoctrine {
    control: Vec<(u32, u16)>,
    advance: Vec<(u32, u32, Fixed)>,
    postures: Vec<(u16, u16, bool)>,
    losses: Vec<(u16, u64)>,
    last_week: Vec<SegmentWeek>,
    still: Vec<(u32, u32, u16)>,
    support: Vec<super::support::Support>,
    occupied: Vec<super::occupation::Occupied>,
    wrecks: Vec<super::occupation::Wreck>,
}

impl From<FightingBeforeDoctrine> for Fighting {
    fn from(f: FightingBeforeDoctrine) -> Self {
        Self {
            control: f.control,
            advance: f.advance,
            postures: f.postures,
            losses: f.losses,
            last_week: f.last_week,
            still: f.still,
            support: f.support,
            occupied: f.occupied,
            wrecks: f.wrecks,
            doctrine: Vec::new(),
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
    day: i64,
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
    // Scorched earth: the regions taken in this week's combat, before any peace hands them back.
    if let Some(rules) = k.rules.occupation.scorched.as_ref() {
        let regions = &world.econ.regions;
        let mut fought = Vec::new();
        let mut taken = Vec::new();
        for g in &out.segments {
            let km = g.outcome.advance_km;
            if km != Fixed::ZERO {
                fought.push(if km.is_positive() { g.into } else { g.from });
            }
            for r in [g.from, g.into] {
                let owner = regions[r as usize].market;
                let before = world.war.fighting.controller(r, owner);
                let after = out.fighting.controller(r, owner);
                if before != after && !taken.iter().any(|(t, _, _)| *t == r) {
                    taken.push((r, before, after));
                }
            }
        }
        let (state, burn) = super::scorch::week(rules, world, &fought, &taken, day);
        out.devastation.scorched = burn;
        out.scorch = Some(state);
    }
    out.support = super::support::support_week(
        k,
        setup,
        world,
        &mut out.fighting,
        &out.segments,
        &out.ended,
        player,
    );
    for w in world.war.conflicts.wars.iter().filter(|w| {
        out.support.withdrawn.contains(&w.id) || out.support.surrendered.contains(&w.id)
    }) {
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
        h.write_len(self.doctrine.len());
        for d in &self.doctrine {
            h.write_u16(d.country);
            h.write_u8(d.generation as u8);
            d.experience_weeks.hash_state(h);
            h.write_i64(d.as_of);
            d.intent.hash_state(h);
            d.tradition.hash_state(h);
            h.write_i64(d.reform_until);
        }
    }
}

#[cfg(test)]
mod concentration_tests {
    use super::*;

    #[test]
    fn the_mobile_class_sets_the_pace_from_a_third_of_the_men() {
        let m = |here, all| {
            concentrated_mobility(Mobility::Foot, Some(Mobility::Horse), here, all, 333)
        };
        assert_eq!(m(40_000, 100_000), Mobility::Horse);
        assert_eq!(m(33_300, 100_000), Mobility::Horse);
        assert_eq!(m(30_000, 100_000), Mobility::Foot);
        assert_eq!(m(0, 0), Mobility::Foot);
        // Without a mobile class the army's majority.
        assert_eq!(
            concentrated_mobility(Mobility::Foot, None, 50_000, 60_000, 333),
            Mobility::Foot
        );
    }
}
