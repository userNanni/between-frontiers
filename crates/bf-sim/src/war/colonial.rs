//! The colonial war (`docs/war.pt-BR.md`, "Guerra colonial"; owner's decision G6, questions
//! GC1 to GC5). Behind `Switches::war`.
//!
//! A unit that revolts after a refusal (`borders::independence`, `How::Revolt`) splits: the
//! metropole keeps the states its garrison covers (ports first, then the most populous, at the
//! O3 minimum per thousand people), the movement proclaims the rest. The forces system reads the
//! revolt's record and opens a war: the metropole demands the union back, the new country fights
//! for its existence.
//!
//! - The metropole fights overseas with its colonial garrison and the men it lands: the landed
//!   men grow every week by `men_per_port_week` for each port it holds in the unit, up to
//!   `expedition_cap` of its land men. A colony it reaches by land has no cap.
//! - The proclaimed country fights with irregulars: a share (`armed_share`) of its men of service
//!   age, times its war support (the people behind it), recruited a share of the gap a week and
//!   held back by the arms it has.
//! - Two endings. The metropole gives up (its support below the peace threshold): the states it
//!   still holds in the unit go to the new country, which it recognises. The new country loses
//!   (the union, signed at the table): the unit is a colony again and its movement starts again
//!   from `movement_after_reconquest` of the mark of the revolt (GC5).
//!
//! Everything is integer fixed point and free of randomness.

use std::collections::{BTreeMap, BTreeSet};

use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::{Date, Fixed};
use serde::{Deserialize, Serialize};

use super::catalogue::{Domain, Military};
use super::state::{CountryWar, Formation, men};
use crate::game::GameWorld;
use crate::treaty::{Clause, ClauseKind, Proposal, Side};

/// `colonial` in `data/military/war.ron`. Per mille unless named otherwise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonialRules {
    /// Men per thousand people a state needs to be held at the revolt (O3, the calm minimum).
    pub garrison: u32,
    /// Men landed a week for each port the metropole holds in the unit.
    pub men_per_port_week: u32,
    /// The landed men's cap, per mille of the metropole's land men.
    pub expedition_cap: u32,
    /// Combatants per mille of the men of service age, at full mobilization.
    pub armed_share: u32,
    /// Of those combatants, the share under arms at the proclamation.
    pub initial_share: u32,
    /// Of the gap to the combatants, the share recruited a week at full support.
    pub recruit_week: u32,
    /// War support of the proclaimed country at the start (GC2).
    pub proclaimed: i32,
    /// GC1: the colonial formations in the unit go over to the movement by its mobilization.
    pub defect: bool,
    /// GC3: the share of the irregulars' arms they have at the proclamation.
    pub equipment_at_birth: u32,
    /// GC5: the movement after a reconquest, per mille of the mark of the revolt.
    pub movement_after_reconquest: u32,
    /// The guerrilla in the states the metropole holds: its casualties a week per armed
    /// irregular there, per mille.
    pub ambush: u32,
    /// The irregulars' casualties for each of the metropole's, per mille: `exchange_base` plus
    /// `exchange_per_cover` times the cover (the metropole's men in the unit over `hostile` per
    /// thousand of the people it holds there, O3).
    pub exchange_base: u32,
    pub exchange_per_cover: u32,
    pub hostile: u32,
    /// The AI lands men until its men in the unit reach this cover (per mille of `hostile` per
    /// thousand of the people it holds there); 0 lands up to the cap.
    #[serde(default)]
    pub cover_target: u32,
}

/// What the colonial war needs from the map and the independence, built at load.
#[derive(Clone, Debug)]
pub struct ColonialSetup {
    pub rules: ColonialRules,
    /// States with a port.
    pub coastal: BTreeSet<u16>,
    /// The states of each candidate country, by tag (`borders::catalogue`).
    pub candidates: BTreeMap<String, Vec<u16>>,
    /// Land neighbours of each state.
    pub neighbours: std::sync::Arc<BTreeMap<u16, Vec<u16>>>,
    /// The mark of the revolt (`independence.ron`), per mille: the mobilization at the
    /// proclamation, which the record does not keep.
    pub revolt: u16,
    /// The irregulars' unit type.
    pub irregular: Option<u16>,
    /// The key of each state, by id.
    pub names: BTreeMap<u16, String>,
}

/// A colonial war under way (`WarState::colonial`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Colonial {
    /// The war's id.
    pub war: u32,
    /// The metropole and the proclaimed country, market indices.
    pub owner: u16,
    pub colony: u16,
    /// The candidate's tag.
    pub unit: String,
    /// Every region of the unit, and of them those the metropole kept at the revolt.
    pub regions: Vec<u32>,
    pub kept: Vec<u32>,
    /// Men landed so far.
    pub landed: u64,
    /// The metropole reaches the colony by land: no cap.
    pub by_land: bool,
    /// Mobilization of the movement, per mille.
    pub mobilization: u16,
    /// The war ended in a peace: the unit waits for the union to go through to be a colony
    /// again (`reconquered`).
    pub peace: bool,
    /// The new country's irregulars under arms, last week: at zero the colony has lost.
    pub armed: u64,
    /// The metropole's men tied in the unit (its garrison there and the men landed), and that
    /// share of its land men times two, per mille, at most 1,000 (G8, the weight of duration).
    pub tied: u64,
    pub commitment: u16,
}

impl HashState for Colonial {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u32(self.war);
        h.write_u16(self.owner);
        h.write_u16(self.colony);
        h.write_len(self.unit.len());
        for b in self.unit.bytes() {
            h.write_u8(b);
        }
        h.write_len(self.regions.len());
        for r in &self.regions {
            h.write_u32(*r);
        }
        h.write_len(self.kept.len());
        for r in &self.kept {
            h.write_u32(*r);
        }
        h.write_u64(self.landed);
        self.by_land.hash_state(h);
        h.write_u16(self.mobilization);
        self.peace.hash_state(h);
        h.write_u64(self.armed);
        h.write_u64(self.tied);
        h.write_u16(self.commitment);
    }
}

/// The split at the revolt: (kept, released). The garrison is the soldiers the metropole holds
/// in the unit's regions; each state it keeps takes `garrison` men per thousand people, ports
/// first, then by people (most first). The first port (the most populous) is always kept: the
/// administration and the fleet are there ([`first_port`]). The movement always takes one state
/// at least: the last in that order.
pub fn partition(
    world: &GameWorld,
    regions: &[u32],
    coastal: &BTreeSet<u16>,
    garrison: u32,
) -> (Vec<u32>, Vec<u32>) {
    let rs = &world.econ.regions;
    let men: i64 = regions
        .iter()
        .map(|&r| world.war.soldiers.get(r as usize).copied().unwrap_or(0))
        .sum();
    if men <= 0 || regions.len() < 2 {
        return (Vec::new(), regions.to_vec());
    }
    let port = first_port(world, regions, coastal);
    let mut order = regions.to_vec();
    order.sort_by_key(|&r| {
        let reg = &rs[r as usize];
        (!coastal.contains(&reg.state), -reg.population(), r)
    });
    let mut left = men;
    let mut kept = Vec::new();
    let mut released = Vec::new();
    for &r in &order[..order.len() - 1] {
        let need = (rs[r as usize].population() * i64::from(garrison) + 999) / 1000;
        if need <= left || Some(r) == port {
            left = (left - need).max(0);
            kept.push(r);
        } else {
            released.push(r);
        }
    }
    released.push(order[order.len() - 1]);
    kept.sort_unstable();
    released.sort_unstable();
    (kept, released)
}

/// The port the metropole keeps whatever its garrison: the most populous coastal region of the
/// unit (ties by index). `None` without a coast.
pub fn first_port(world: &GameWorld, regions: &[u32], coastal: &BTreeSet<u16>) -> Option<u32> {
    let rs = &world.econ.regions;
    regions
        .iter()
        .copied()
        .filter(|&r| coastal.contains(&rs[r as usize].state))
        .min_by_key(|&r| (-rs[r as usize].population(), r))
}

/// Whether the metropole reaches any of `regions` by land from its own metropolitan regions.
pub fn by_land(setup: &ColonialSetup, world: &GameWorld, owner: u16, regions: &[u32]) -> bool {
    let rs = &world.econ.regions;
    let target: BTreeSet<u16> = regions.iter().map(|&r| rs[r as usize].state).collect();
    let state_of: BTreeMap<u16, &crate::econ::Region> = rs.iter().map(|r| (r.state, r)).collect();
    // Walk the land of the metropole and its colonies from the metropolitan states.
    let mut seen: BTreeSet<u16> = rs
        .iter()
        .filter(|r| r.market == owner && !r.colony)
        .map(|r| r.state)
        .collect();
    let mut stack: Vec<u16> = seen.iter().copied().collect();
    while let Some(s) = stack.pop() {
        for n in setup.neighbours.get(&s).into_iter().flatten() {
            if target.contains(n) {
                return true;
            }
            let ours = state_of.get(n).is_some_and(|r| r.market == owner);
            if ours && seen.insert(*n) {
                stack.push(*n);
            }
        }
    }
    false
}

/// Combatants the proclaimed country can raise: its war support (standing for the
/// mobilization) times `armed_share` of its men of service age.
pub fn combatants(
    rules: &ColonialRules,
    mil: &Military,
    world: &GameWorld,
    colony: u16,
    support: u16,
) -> u64 {
    combatants_where(rules, mil, world, |r| r.market == colony, support)
}

/// [`combatants`] over the people of `regions`, whoever holds them: the whole unit, the states
/// the metropole keeps included (the movement recruits there too).
pub fn combatants_of(
    rules: &ColonialRules,
    mil: &Military,
    world: &GameWorld,
    regions: &[u32],
    support: u16,
) -> u64 {
    let set: BTreeSet<u32> = regions.iter().copied().collect();
    let of_age: u64 = world
        .econ
        .regions
        .iter()
        .enumerate()
        .filter(|(i, _)| set.contains(&(*i as u32)))
        .filter_map(|(i, _)| world.demography.ages.get(i))
        .map(|a| super::calc::mobilizable(&mil.rules, &super::system::bands(a), 1000, 0).of_age)
        .sum();
    (u128::from(of_age) * u128::from(rules.armed_share) * u128::from(support) / 1_000_000) as u64
}

/// [`combatants`] of the people of `states` held by `owner`: the measure of a unit before it
/// revolts.
pub fn combatants_in(
    rules: &ColonialRules,
    mil: &Military,
    world: &GameWorld,
    states: &[u16],
    owner: u16,
    support: u16,
) -> u64 {
    combatants_where(
        rules,
        mil,
        world,
        |r| r.market == owner && states.contains(&r.state),
        support,
    )
}

fn combatants_where(
    rules: &ColonialRules,
    mil: &Military,
    world: &GameWorld,
    take: impl Fn(&crate::econ::Region) -> bool,
    support: u16,
) -> u64 {
    let of_age: u64 = world
        .econ
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| take(r))
        .filter_map(|(i, _)| world.demography.ages.get(i))
        .map(|a| super::calc::mobilizable(&mil.rules, &super::system::bands(a), 1000, 0).of_age)
        .sum();
    (u128::from(of_age) * u128::from(rules.armed_share) * u128::from(support) / 1_000_000) as u64
}

/// The irregulars of a country: (men, men armed).
pub fn irregulars(mil: &Military, irregular: Option<u16>, c: &CountryWar) -> (u64, u64) {
    let Some(ty) = irregular else {
        return (0, 0);
    };
    let mut total = (0u64, 0u64);
    for f in c.formations.iter().filter(|f| f.unit == ty) {
        let n = men(mil, f).max(0) as u64;
        total.0 += n;
        total.1 += (Fixed::from_int(n as i64) * armed_share(mil, f))
            .to_int_trunc()
            .max(0) as u64;
    }
    total
}

/// The share of a formation's men with their arms: the scarcest good of its equipment, 0 to 1.
fn armed_share(mil: &Military, f: &Formation) -> Fixed {
    let spec = mil.spec(f.unit, f.generation as usize);
    if !f.elements.is_positive() {
        return Fixed::ZERO;
    }
    spec.equipment
        .iter()
        .map(|(g, per)| {
            let want = *per * f.elements;
            let have = f
                .equipment
                .iter()
                .find(|(k, _)| k == g)
                .map_or(Fixed::ZERO, |e| e.1);
            if want.is_positive() {
                (have / want).min(Fixed::ONE)
            } else {
                Fixed::ONE
            }
        })
        .min()
        .unwrap_or(Fixed::ONE)
}

/// The men the metropole can put on the colonial segments: its garrison left in the unit (the
/// soldiers of the regions it still holds there) and the men landed. `None` reached by land.
pub fn theatre_men(world: &GameWorld, c: &Colonial) -> Option<u64> {
    if c.by_land {
        return None;
    }
    let garrison: i64 = c
        .regions
        .iter()
        .filter(|&&r| world.econ.regions[r as usize].market == c.owner)
        .map(|&r| world.war.soldiers.get(r as usize).copied().unwrap_or(0))
        .sum();
    Some(garrison.max(0) as u64 + c.landed)
}

fn tag(world: &GameWorld, m: u16) -> Option<String> {
    world
        .countries
        .iter()
        .nth(usize::from(m))
        .map(|(_, c)| c.tag.clone())
}

fn index_of(world: &GameWorld, t: &str) -> Option<u16> {
    world
        .countries
        .iter()
        .position(|(_, c)| c.tag == t)
        .map(|i| i as u16)
}

/// The demand of a colonial war: the new country back in the metropole.
pub fn union_demand(owner: &str, colony: &str) -> Proposal {
    Proposal {
        side_a: vec![owner.to_owned()],
        side_b: vec![colony.to_owned()],
        clauses: vec![Clause {
            kind: ClauseKind::Union,
            giver: Side::Junior,
            receiver: Side::Senior,
            until: None,
            note: None,
        }],
        term_years: None,
        notice_years: 0,
    }
}

/// Opens a colonial war for every revolt recorded in the seven days to `date`: the metropole
/// demands the union, the proclaimed country gets its irregulars (and, by GC1, the colonial
/// formations that go over to it).
pub fn open(setup: &ColonialSetup, mil: &Military, world: &mut GameWorld, date: Date) {
    let since = date.to_days() - 7;
    let revolts: Vec<(String, u16)> = world
        .independence
        .history
        .iter()
        .filter(|r| {
            r.how == crate::borders::independence::How::Revolt
                && r.date.to_days() > since
                && r.date.to_days() <= date.to_days()
        })
        .map(|r| (r.unit.clone(), r.owner))
        .collect();
    for (unit, owner) in revolts {
        let Some(colony) = index_of(world, &unit) else {
            continue;
        };
        if world.war.colonial.iter().any(|c| c.colony == colony)
            || world.war.conflicts.at_war(owner, colony)
        {
            continue;
        }
        let states: BTreeSet<u16> = setup
            .candidates
            .get(&unit)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        let regions: Vec<u32> = world
            .econ
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| states.contains(&r.state) && (r.market == owner || r.market == colony))
            .map(|(i, _)| i as u32)
            .collect();
        if !regions
            .iter()
            .any(|&r| world.econ.regions[r as usize].market == colony)
        {
            continue;
        }
        let kept: Vec<u32> = regions
            .iter()
            .copied()
            .filter(|&r| world.econ.regions[r as usize].market == owner)
            .collect();
        let (Some(a), Some(b)) = (tag(world, owner), tag(world, colony)) else {
            continue;
        };
        let wid = super::crisis::open_war(
            world,
            vec![owner],
            vec![colony],
            u32::MAX,
            union_demand(&a, &b),
            date,
            false,
        );
        let mobilization = setup.revolt;
        let by_land = by_land(setup, world, owner, &regions);
        birth_forces(setup, mil, world, owner, colony, &regions, mobilization);
        let armed = world
            .war
            .countries
            .get(colony as usize)
            .map_or(0, |cw| irregulars(mil, setup.irregular, cw).1);
        world.war.colonial.push(Colonial {
            war: wid,
            owner,
            colony,
            unit,
            regions,
            kept,
            landed: 0,
            by_land,
            mobilization,
            peace: false,
            armed,
            tied: 0,
            commitment: 0,
        });
    }
}

/// The forces of the proclaimed country at birth: the irregulars (`initial_share` of its
/// combatants, `equipment_at_birth` of their arms) and, by GC1, the share of the metropole's
/// colonial formations the unit holds times the mobilization.
fn birth_forces(
    setup: &ColonialSetup,
    mil: &Military,
    world: &mut GameWorld,
    owner: u16,
    colony: u16,
    regions: &[u32],
    mobilization: u16,
) {
    let r = &setup.rules;
    let n = world.countries.len().max(world.war.countries.len());
    world.war.countries.resize_with(n, CountryWar::default);
    if r.defect {
        // The unit's share of the metropole's colonial people.
        let (mut unit_people, mut all) = (0i64, 0i64);
        for (i, reg) in world.econ.regions.iter().enumerate() {
            let in_unit = regions.contains(&(i as u32));
            if (reg.market == owner && reg.colony) || in_unit {
                all += reg.population();
                if in_unit {
                    unit_people += reg.population();
                }
            }
        }
        let share = if all > 0 {
            Fixed::from_ratio(unit_people, all) * Fixed::from_ratio(i64::from(mobilization), 1000)
        } else {
            Fixed::ZERO
        };
        let mut gone = Vec::new();
        if let Some(cw) = world.war.countries.get_mut(owner as usize) {
            for f in cw
                .formations
                .iter_mut()
                .filter(|f| f.colonial && mil.types[f.unit as usize].domain == Domain::Land)
            {
                let take = f.elements * share;
                if !take.is_positive() {
                    continue;
                }
                let part = take / f.elements;
                let equipment: Vec<(u16, Fixed)> =
                    f.equipment.iter().map(|(g, q)| (*g, *q * part)).collect();
                for (e, (_, q)) in f.equipment.iter_mut().zip(&equipment) {
                    e.1 -= *q;
                }
                f.elements -= take;
                gone.push(Formation {
                    unit: f.unit,
                    generation: f.generation,
                    elements: take,
                    colonial: false,
                    called_up: false,
                    equipment,
                    training: f.training,
                    raised: true,
                });
            }
            cw.formations.retain(|f| f.elements.is_positive());
        }
        if let Some(cw) = world.war.countries.get_mut(colony as usize) {
            cw.formations.extend(gone);
        }
    }
    let Some(ty) = setup.irregular else {
        return;
    };
    let want = combatants_of(r, mil, world, regions, r.proclaimed.clamp(0, 1000) as u16);
    let first = (u128::from(want) * u128::from(r.initial_share) / 1000) as i64;
    if first <= 0 {
        return;
    }
    let elements = Fixed::from_ratio(first, i64::from(mil.spec(ty, 0).men.max(1)));
    let arms = Fixed::from_ratio(i64::from(r.equipment_at_birth), 1000);
    let equipment = mil
        .spec(ty, 0)
        .equipment
        .iter()
        .map(|(g, per)| (*g, *per * elements * arms))
        .collect();
    if let Some(cw) = world.war.countries.get_mut(colony as usize) {
        cw.mobilized = true;
        cw.formations.push(Formation {
            unit: ty,
            generation: 0,
            elements,
            colonial: false,
            called_up: false,
            equipment,
            training: Fixed::ONE,
            raised: true,
        });
    }
}

/// A week of every colonial war: the metropole lands men by its ports, the proclaimed country
/// recruits irregulars toward its combatants, drawn by its support and held back by its arms.
pub fn week(setup: &ColonialSetup, mil: &Military, world: &mut GameWorld) {
    let r = &setup.rules;
    let wars = std::mem::take(&mut world.war.colonial);
    let mut out = Vec::with_capacity(wars.len());
    for mut c in wars {
        if c.peace || !world.war.conflicts.wars.iter().any(|w| w.id == c.war) {
            out.push(c);
            continue;
        }
        let land_men: i64 = world.war.countries.get(c.owner as usize).map_or(0, |cw| {
            cw.formations
                .iter()
                .filter(|f| mil.types[f.unit as usize].domain == Domain::Land)
                .map(|f| men(mil, f))
                .sum()
        });
        // Landing: by the ports the metropole still holds and controls in the unit.
        if !c.by_land {
            let ports = c
                .regions
                .iter()
                .filter(|&&ri| {
                    let reg = &world.econ.regions[ri as usize];
                    reg.market == c.owner
                        && setup.coastal.contains(&reg.state)
                        && world.war.fighting.controller(ri, reg.market) == c.owner
                })
                .count() as u64;
            let mut cap = (land_men.max(0) as u64) * u64::from(r.expedition_cap) / 1000;
            if r.cover_target > 0 {
                let held: i64 = c
                    .regions
                    .iter()
                    .filter(|&&ri| {
                        let reg = &world.econ.regions[ri as usize];
                        world.war.fighting.controller(ri, reg.market) == c.owner
                    })
                    .map(|&ri| world.econ.regions[ri as usize].population())
                    .sum();
                let garrison = theatre_men(world, &c).unwrap_or(0).saturating_sub(c.landed);
                let want = (held.max(0) as u64) * u64::from(r.hostile) / 1000
                    * u64::from(r.cover_target)
                    / 1000;
                cap = cap.min(want.saturating_sub(garrison));
            }
            c.landed = (c.landed + ports * u64::from(r.men_per_port_week)).min(cap);
        }
        // Recruitment.
        if let Some(ty) = setup.irregular {
            // The people behind the new country: its war support stands for the mobilization.
            let support = world
                .war
                .fighting
                .support_of(c.colony)
                .map_or(r.proclaimed, |s| s.value)
                .clamp(0, 1000);
            let want = combatants_of(r, mil, world, &c.regions, support as u16);
            if let Some(cw) = world.war.countries.get_mut(c.colony as usize) {
                let (have, armed) = irregulars(mil, Some(ty), cw);
                let gap = want.saturating_sub(have);
                // Without arms the recruits wait: the share armed holds the pace back.
                let arms = if have > 0 {
                    Fixed::from_ratio(armed as i64, have as i64)
                } else {
                    Fixed::ONE
                };
                let n =
                    (Fixed::from_int((u128::from(gap) * u128::from(r.recruit_week) / 1000) as i64)
                        * arms)
                        .to_int_trunc();
                if n > 0 {
                    let add = Fixed::from_ratio(n, i64::from(mil.spec(ty, 0).men.max(1)));
                    match cw.formations.iter_mut().find(|f| f.unit == ty) {
                        Some(f) => {
                            let total = f.elements + add;
                            f.training = (f.training * f.elements) / total;
                            f.elements = total;
                        }
                        None => cw.formations.push(Formation {
                            unit: ty,
                            generation: 0,
                            elements: add,
                            colonial: false,
                            called_up: false,
                            equipment: Vec::new(),
                            training: Fixed::ZERO,
                            raised: true,
                        }),
                    }
                }
            }
        }
        guerrilla(setup, mil, world, &mut c, land_men);
        out.push(c);
    }
    world.war.colonial = out;
}

/// A week of the guerrilla in the unit's regions the metropole holds (G6, the orchestrator's
/// note of 2026-11-19): the irregulars of their people ambush, the metropole's men there hunt
/// them. The metropole's casualties are `ambush` per mille of those armed irregulars; theirs are
/// as many times the metropole's as the exchange, which rises with the cover (O3). Both go into
/// the losses that war support reads (G4), and are buried. Also records the men the metropole
/// keeps tied in the unit and the weight of duration (G8).
fn guerrilla(
    setup: &ColonialSetup,
    mil: &Military,
    world: &mut GameWorld,
    c: &mut Colonial,
    land: i64,
) {
    let r = &setup.rules;
    c.tied = theatre_men(world, c).unwrap_or_else(|| {
        c.regions
            .iter()
            .filter(|&&ri| world.econ.regions[ri as usize].market == c.owner)
            .map(|&ri| {
                world
                    .war
                    .soldiers
                    .get(ri as usize)
                    .copied()
                    .unwrap_or(0)
                    .max(0) as u64
            })
            .sum()
    });
    c.commitment = if land > 0 {
        (u128::from(c.tied) * 2000 / land as u128).min(1000) as u16
    } else {
        1000
    };
    let (mut all, mut held) = (0i64, 0i64);
    for &ri in &c.regions {
        let reg = &world.econ.regions[ri as usize];
        let p = reg.population();
        all += p;
        if world.war.fighting.controller(ri, reg.market) == c.owner {
            held += p;
        }
    }
    let Some(cw) = world.war.countries.get(c.colony as usize) else {
        return;
    };
    let (men_now, armed) = irregulars(mil, setup.irregular, cw);
    c.armed = armed;
    if all <= 0 || held <= 0 || armed == 0 || c.tied == 0 {
        return;
    }
    let armed_here = (u128::from(armed) * held as u128 / all as u128) as u64;
    let theirs_men = (armed_here * u64::from(r.ambush) / 1000).min(c.tied);
    let need = (held as u64 * u64::from(r.hostile) / 1000).max(1);
    let exchange = u64::from(r.exchange_base) + u64::from(r.exchange_per_cover) * c.tied / need;
    let irregular_men = (theirs_men * exchange / 1000).min(men_now);
    if theirs_men > 0 && land > 0 {
        if let Some(cw) = world.war.countries.get_mut(c.owner as usize) {
            super::fight::take_losses(mil, cw, Fixed::from_ratio(theirs_men as i64, land));
        }
        lose(world, c.owner, theirs_men);
    }
    if irregular_men > 0 {
        if let (Some(ty), Some(cw)) = (
            setup.irregular,
            world.war.countries.get_mut(c.colony as usize),
        ) {
            let share = Fixed::from_ratio(irregular_men as i64, men_now as i64);
            for f in cw.formations.iter_mut().filter(|f| f.unit == ty) {
                let keep = Fixed::ONE - share;
                f.elements *= keep;
                for e in &mut f.equipment {
                    e.1 *= keep;
                }
            }
            cw.formations.retain(|f| f.elements.is_positive());
        }
        lose(world, c.colony, irregular_men);
        if let Some(cw) = world.war.countries.get(c.colony as usize) {
            c.armed = irregulars(mil, setup.irregular, cw).1;
        }
    }
}

/// Casualties of `m` into the war's losses, the killed among them buried.
fn lose(world: &mut GameWorld, m: u16, n: u64) {
    let f = &mut world.war.fighting;
    match f.losses.binary_search_by_key(&m, |(k, _)| *k) {
        Ok(i) => f.losses[i].1 += n,
        Err(i) => f.losses.insert(i, (m, n)),
    }
    let killed = (n as i64 * super::fight::KILLED_PER_MILLE / 1000).max(0) as u64;
    super::fight::bury(world, m, killed);
}

/// The colonial wars that ended this week. `withdrawn`: the metropole gave up, and the states it
/// still holds in the unit go to the new country, which it recognises. Otherwise a peace was
/// signed: if the union went through (the new country is gone), the unit is a colony again and
/// its movement starts from `movement_after_reconquest` of the mark (GC5).
pub fn end(setup: &ColonialSetup, world: &mut GameWorld, ended: &[u32], withdrawn: &[u32]) {
    let wars = std::mem::take(&mut world.war.colonial);
    let mut keep = Vec::with_capacity(wars.len());
    for mut c in wars {
        if c.peace || !ended.contains(&c.war) {
            keep.push(c);
            continue;
        }
        let (Some(owner), Some(colony)) = (tag(world, c.owner), tag(world, c.colony)) else {
            continue;
        };
        if !withdrawn.contains(&c.war) {
            // The union is executed by the treaty's bridge, with the borders' operations: the
            // unit waits for it (`reconquered`).
            c.peace = true;
            keep.push(c);
            continue;
        }
        let states: Vec<String> = c
            .regions
            .iter()
            .filter(|&&r| world.econ.regions[r as usize].market == c.owner)
            .filter_map(|&r| setup.names.get(&world.econ.regions[r as usize].state))
            .cloned()
            .collect();
        if !states.is_empty() {
            world
                .borders
                .pending
                .push(crate::borders::op::Operation::Transfer {
                    from: owner,
                    to: colony,
                    states,
                    colony: Some(false),
                    terms: Default::default(),
                });
        }
        world.treaties.recognition.grant(c.owner, c.colony);
    }
    world.war.colonial = keep;
}

/// A week after a peace, once the union went through: the unit's regions back in the metropole are colonies again, and
/// the unit's movement starts from `movement_after_reconquest` of the mark of the revolt (GC5).
pub fn reconquered(setup: &ColonialSetup, world: &mut GameWorld) {
    let wars = std::mem::take(&mut world.war.colonial);
    let mut keep = Vec::with_capacity(wars.len());
    for c in wars {
        if !c.peace {
            keep.push(c);
            continue;
        }
        // A week after the peace the union has gone through, or the peace was another: the
        // new country standing, nothing to reconquer.
        if world.econ.regions.iter().any(|r| r.market == c.colony) {
            continue;
        }
        for &r in &c.regions {
            let reg = &mut world.econ.regions[r as usize];
            if reg.market == c.owner {
                reg.colony = true;
            }
        }
        let support = (u32::from(setup.revolt) * setup.rules.movement_after_reconquest / 1000)
            .min(1000) as u16;
        let units = &mut world.independence.units;
        let at = units.iter().position(|u| u.key == c.unit);
        let u = crate::borders::independence::Unit {
            key: c.unit.clone(),
            owner: c.owner,
            regions: c.regions.clone(),
            movement: crate::polity::MovementState {
                support,
                ..Default::default()
            },
            mobilized: support,
            demand: None,
        };
        match at {
            Some(i) => units[i] = u,
            None => units.push(u),
        }
    }
    world.war.colonial = keep;
}
