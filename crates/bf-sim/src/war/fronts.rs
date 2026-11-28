//! Fronts drawn by the player and divisions sent one by one (G2; the owner's FR1 to FR11 of
//! 2026-11-28), behind the switch `fronts` (off). Design in `docs/war.pt-BR.md`, "Frentes
//! desenhadas".
//!
//! - FR1: a front is a set of the player's own states on the line of contact, marked on the
//!   map. Its segments are the pairs (its state, an enemy state next to it).
//! - FR2 and FR7: the land army is cut into divisions of `division_elements` (some 15,000 men),
//!   each with a number; the player sends each to a front, to the reserve, or to none.
//! - FR3: a front may aim at an enemy state; the segments into it weigh `objective_weight`.
//! - FR4: each week `reserve_step` of the reserve goes to the front that lost most ground.
//! - FR5 and FR10: a division that changes place spends weeks in transit, by the distance by
//!   rail (`rail_km_per_week`); in transit it does not fight.
//! - FR6: the AI keeps the rule of before: no fronts, no divisions.
//! - FR8: a stretch of the line no front covers stands empty: the enemy walks in if it attacks.
//!   The emptiness only comes from the player's act; the screen warns in red.
//! - FR9: when the line moves the fronts follow it: a state lost leaves its front, the own states
//!   that come to touch the enemy join the front of a neighbour, a state taken joins the front
//!   that took it.
//! - FR11: a new division (an order, a mobilization) goes to the reserve.
//!
//! The divisions are a layer over the formations: the formations keep the men, the equipment and
//! the training as before, and the divisions of one kind (unit, generation, call-up, order) share
//! them. Their elements always sum to the formation's. With the switch off, or with no player at
//! war, nothing here exists, and the game is the one before.

use std::collections::{BTreeMap, BTreeSet};

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::catalogue::{Domain, Military, Mobility};
use super::fight::Fighting;
use super::state::{CountryWar, Formation};
use crate::game::GameWorld;

/// `data/military/war.ron`, `fronts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontRules {
    /// Elements (thousands of men on land) of a division (FR7). A division of 1925 was some
    /// 10,000 to 20,000 men.
    pub division_elements: u32,
    /// Km a division moves by rail in a week, for its weeks in transit (FR10). DESIGN VALUE.
    pub rail_km_per_week: u32,
    /// Of the reserve, the part that goes to the front that lost most ground each week, per
    /// mille (FR4). DESIGN VALUE.
    pub reserve_step: u32,
    /// Weight of a segment into the front's objective, per mille of the others' (FR3). DESIGN
    /// VALUE.
    pub objective_weight: u32,
}

/// The kind of formation a division is part of: its unit, generation, and how it was raised.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Kind {
    pub unit: u16,
    pub generation: u8,
    pub called_up: bool,
    pub raised: bool,
}

impl Kind {
    pub fn of(f: &Formation) -> Self {
        Self {
            unit: f.unit,
            generation: f.generation,
            called_up: f.called_up,
            raised: f.raised,
        }
    }
}

/// Where a division stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Place {
    /// On the front of this number.
    Front(u32),
    /// In the reserve: off the line; part of it goes each week where ground is lost (FR4).
    Reserve,
    /// Sent nowhere: it does not fight and does not reinforce.
    Idle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Division {
    pub id: u32,
    pub kind: Kind,
    pub elements: Fixed,
    pub place: Place,
    /// Weeks left in transit before it reaches its place.
    pub transit: u16,
    /// Where it left from, while the tick has not yet worked out its weeks in transit (a
    /// command sent it).
    pub from: Option<Place>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Front {
    pub id: u32,
    /// The enemy it was made for (its name on the screen).
    pub enemy: u16,
    /// The own states on the line it covers, sorted.
    pub states: Vec<u16>,
    /// Attack, or hold (the default).
    pub attack: bool,
    /// The enemy state it aims at.
    pub objective: Option<u16>,
}

/// The player's fronts and divisions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryFronts {
    pub country: u16,
    pub fronts: Vec<Front>,
    pub divisions: Vec<Division>,
    pub next_id: u32,
    /// States of the line no front covers, with the km of their segments (FR8), last week.
    pub uncovered: Vec<(u16, u32)>,
    /// The front the reserve reinforced last week.
    pub reinforced: Option<u32>,
    /// States of the line the player left out of every front (FR8): the line's following (FR9)
    /// does not fill them back. Sorted.
    pub left_open: Vec<u16>,
    /// The fronts were laid down at the start of the war.
    pub started: bool,
}

/// Every country's fronts (only the player's, today); empty with the switch off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontsState {
    pub countries: Vec<CountryFronts>,
}

impl FrontsState {
    pub fn is_empty(&self) -> bool {
        self.countries.is_empty()
    }

    pub fn of(&self, country: u16) -> Option<&CountryFronts> {
        self.countries.iter().find(|c| c.country == country)
    }

    pub fn of_mut(&mut self, country: u16) -> Option<&mut CountryFronts> {
        self.countries.iter_mut().find(|c| c.country == country)
    }
}

impl CountryFronts {
    fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn front(&self, id: u32) -> Option<&Front> {
        self.fronts.iter().find(|f| f.id == id)
    }

    /// The front that covers own state `s`.
    pub fn front_of(&self, s: u16) -> Option<u32> {
        self.fronts
            .iter()
            .find(|f| f.states.binary_search(&s).is_ok())
            .map(|f| f.id)
    }

    /// Draws a front: a new one, or `id`'s states replaced. A state leaves the front it was in.
    pub fn draw(&mut self, id: Option<u32>, enemy: u16, states: &[u16]) -> u32 {
        let mut states: Vec<u16> = states.to_vec();
        states.sort_unstable();
        states.dedup();
        let before: Vec<u16> = self.covered();
        for f in &mut self.fronts {
            if Some(f.id) != id {
                f.states.retain(|s| states.binary_search(s).is_err());
            }
        }
        self.left_open.retain(|s| states.binary_search(s).is_err());
        match id.and_then(|i| self.fronts.iter_mut().find(|f| f.id == i)) {
            Some(f) => {
                f.states = states;
                let id = f.id;
                self.leave_open(&before);
                id
            }
            None => {
                let id = self.new_id();
                self.fronts.push(Front {
                    id,
                    enemy,
                    states,
                    attack: false,
                    objective: None,
                });
                id
            }
        }
    }

    /// Every state some front covers, sorted.
    fn covered(&self) -> Vec<u16> {
        let mut v: Vec<u16> = self.fronts.iter().flat_map(|f| f.states.clone()).collect();
        v.sort_unstable();
        v
    }

    /// The states of `before` no front covers any more stay open, by the player's act.
    fn leave_open(&mut self, before: &[u16]) {
        let now = self.covered();
        for s in before {
            if now.binary_search(s).is_err()
                && let Err(i) = self.left_open.binary_search(s)
            {
                self.left_open.insert(i, *s);
            }
        }
    }

    /// Deletes front `id`: its divisions are sent nowhere (they do not fight), and its stretch of
    /// the line stands empty (FR8).
    pub fn delete(&mut self, id: u32) {
        let before = self.covered();
        self.fronts.retain(|f| f.id != id);
        self.leave_open(&before);
        for d in &mut self.divisions {
            if d.place == Place::Front(id) {
                d.place = Place::Idle;
                d.transit = 0;
            }
        }
    }

    /// Sends divisions to `to`, with the weeks in transit `weeks(from, to)` gives.
    pub fn send(&mut self, ids: &[u32], to: Place, weeks: impl Fn(Place, Place) -> u16) {
        if let Place::Front(f) = to
            && self.front(f).is_none()
        {
            return;
        }
        for d in &mut self.divisions {
            if ids.contains(&d.id) && d.place != to {
                d.transit = if to == Place::Idle {
                    0
                } else {
                    weeks(d.place, to)
                };
                d.place = to;
                d.from = None;
            }
        }
    }

    /// The player's order: sends divisions to `to`; the tick works out their weeks in transit.
    pub fn order(&mut self, ids: &[u32], to: Place) {
        if let Place::Front(f) = to
            && self.front(f).is_none()
        {
            return;
        }
        for d in &mut self.divisions {
            if ids.contains(&d.id) && d.place != to {
                if to == Place::Idle {
                    d.transit = 0;
                    d.from = None;
                } else {
                    d.from = Some(d.from.unwrap_or(d.place));
                    d.transit = u16::MAX;
                }
                d.place = to;
            }
        }
    }

    /// Divisions on each front, ready (not in transit): (front, elements by kind).
    pub fn on_line(&self) -> BTreeMap<u32, Vec<(Kind, Fixed)>> {
        let mut out: BTreeMap<u32, Vec<(Kind, Fixed)>> = BTreeMap::new();
        for d in &self.divisions {
            if let Place::Front(f) = d.place
                && d.transit == 0
            {
                out.entry(f).or_default().push((d.kind, d.elements));
            }
        }
        out
    }
}

impl HashState for FrontsState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.countries.len());
        for c in &self.countries {
            h.write_u16(c.country);
            h.write_len(c.fronts.len());
            for f in &c.fronts {
                h.write_u32(f.id);
                h.write_u16(f.enemy);
                h.write_len(f.states.len());
                for s in &f.states {
                    h.write_u16(*s);
                }
                f.attack.hash_state(h);
                h.write_u16(f.objective.unwrap_or(u16::MAX));
            }
            h.write_len(c.divisions.len());
            for d in &c.divisions {
                h.write_u32(d.id);
                h.write_u16(d.kind.unit);
                h.write_u8(d.kind.generation);
                d.kind.called_up.hash_state(h);
                d.kind.raised.hash_state(h);
                d.elements.hash_state(h);
                match d.place {
                    Place::Front(f) => {
                        h.write_u8(0);
                        h.write_u32(f);
                    }
                    Place::Reserve => h.write_u8(1),
                    Place::Idle => h.write_u8(2),
                }
                h.write_u16(d.transit);
                match d.from {
                    None => h.write_u8(3),
                    Some(Place::Front(f)) => {
                        h.write_u8(0);
                        h.write_u32(f);
                    }
                    Some(Place::Reserve) => h.write_u8(1),
                    Some(Place::Idle) => h.write_u8(2),
                }
            }
            h.write_u32(c.next_id);
            h.write_len(c.uncovered.len());
            for (s, km) in &c.uncovered {
                h.write_u16(*s);
                h.write_u32(*km);
            }
            h.write_u32(c.reinforced.unwrap_or(u32::MAX));
            c.started.hash_state(h);
            h.write_len(c.left_open.len());
            for s in &c.left_open {
                h.write_u16(*s);
            }
        }
    }
}

/// The metropolitan land formations of a country that its divisions share, by kind.
pub fn land_kinds(mil: &Military, cw: &CountryWar) -> BTreeMap<Kind, Fixed> {
    let mut out: BTreeMap<Kind, Fixed> = BTreeMap::new();
    for f in &cw.formations {
        if mil.types[f.unit as usize].domain == Domain::Land && !f.colonial {
            *out.entry(Kind::of(f)).or_default() += f.elements.max(Fixed::ZERO);
        }
    }
    out
}

/// Keeps the divisions' elements equal to the formations', kind by kind: what the formations
/// gained becomes new divisions at `new_place` (FR11: the reserve, once the war is under way);
/// what they lost comes off the divisions of the kind in proportion, and an emptied division goes.
pub fn sync_divisions(
    rules: &FrontRules,
    cf: &mut CountryFronts,
    kinds: &BTreeMap<Kind, Fixed>,
    new_place: impl Fn(&CountryFronts) -> Place,
) {
    let size = Fixed::from_int(i64::from(rules.division_elements.max(1)));
    let mut held: BTreeMap<Kind, Fixed> = BTreeMap::new();
    for d in &cf.divisions {
        *held.entry(d.kind).or_default() += d.elements;
    }
    let all: BTreeSet<Kind> = kinds.keys().chain(held.keys()).copied().collect();
    for kind in all {
        let want = kinds.get(&kind).copied().unwrap_or(Fixed::ZERO);
        let have = held.get(&kind).copied().unwrap_or(Fixed::ZERO);
        if want > have {
            let mut left = want - have;
            // Replacements fill the divisions of the kind up to their size first, in number
            // order; what is left makes new divisions (FR11).
            for d in cf.divisions.iter_mut().filter(|d| d.kind == kind) {
                if !left.is_positive() {
                    break;
                }
                let room = (size - d.elements).max(Fixed::ZERO);
                let add = room.min(left);
                d.elements += add;
                left -= add;
            }
            while left.is_positive() {
                let e = left.min(size);
                left -= e;
                let id = cf.new_id();
                let place = new_place(cf);
                cf.divisions.push(Division {
                    id,
                    kind,
                    elements: e,
                    place,
                    transit: 0,
                    from: None,
                });
            }
        } else if want < have {
            let keep = if have.is_positive() {
                want / have
            } else {
                Fixed::ZERO
            };
            let mut total = Fixed::ZERO;
            let mut last = None;
            for (i, d) in cf.divisions.iter_mut().enumerate() {
                if d.kind == kind {
                    d.elements = (d.elements * keep).max(Fixed::ZERO);
                    total += d.elements;
                    last = Some(i);
                }
            }
            // The rounding goes to the last division of the kind, so the sum is exact.
            if let Some(i) = last {
                cf.divisions[i].elements =
                    (cf.divisions[i].elements + want - total).max(Fixed::ZERO);
            }
        }
    }
    cf.divisions
        .retain(|d| d.elements >= Fixed::from_ratio(1, 100));
}

/// The own states of `country` on the line, and the enemies each touches: states it holds next to
/// a state held by a country at war with it.
pub fn contact(
    world: &GameWorld,
    f: &Fighting,
    neighbours: &BTreeMap<u16, Vec<u16>>,
    country: u16,
) -> BTreeMap<u16, BTreeSet<u16>> {
    let regions = &world.econ.regions;
    let by_state: BTreeMap<u16, u32> = regions
        .iter()
        .enumerate()
        .map(|(i, r)| (r.state, i as u32))
        .collect();
    let holder = |s: u16| {
        by_state
            .get(&s)
            .map(|&r| f.controller(r, regions[r as usize].market))
    };
    let enemies: BTreeSet<u16> = world
        .war
        .conflicts
        .wars
        .iter()
        .flat_map(|w| {
            if w.side_a.contains(&country) {
                w.side_b.clone()
            } else if w.side_b.contains(&country) {
                w.side_a.clone()
            } else {
                Vec::new()
            }
        })
        .collect();
    let mut out: BTreeMap<u16, BTreeSet<u16>> = BTreeMap::new();
    for (&s, near) in neighbours {
        if holder(s) != Some(country) {
            continue;
        }
        for n in near {
            if let Some(e) = holder(*n)
                && enemies.contains(&e)
            {
                out.entry(s).or_default().insert(e);
            }
        }
    }
    out
}

/// A week of the fronts following the line (FR9) and of new contact. At the first week of war
/// (no fronts yet) one front per enemy covers the whole contact, and the divisions are shared
/// among them by their states, as the old rule spreads the men. A new enemy gets a front for its
/// contact, with the reserve. Returns the fronts that lost states this week.
pub fn follow_line(
    cf: &mut CountryFronts,
    line: &BTreeMap<u16, BTreeSet<u16>>,
    neighbours: &BTreeMap<u16, Vec<u16>>,
    taken: &BTreeSet<u16>,
) -> BTreeSet<u32> {
    let first = !cf.started;
    cf.started = true;
    let mut lost: BTreeSet<u32> = BTreeSet::new();
    cf.left_open.retain(|s| line.contains_key(s));
    // States lost to the enemy or off the line leave their front; a lost one counts as ground lost.
    for f in &mut cf.fronts {
        let before = f.states.len();
        f.states.retain(|s| line.contains_key(s));
        if f.states.len() < before {
            lost.insert(f.id);
        }
    }
    // States taken this week join the front of a neighbour (the one that took them).
    let mut free: Vec<u16> = line
        .keys()
        .copied()
        .filter(|s| cf.front_of(*s).is_none() && cf.left_open.binary_search(s).is_err())
        .collect();
    let mut changed = true;
    while changed {
        changed = false;
        let mut still = Vec::new();
        for s in free {
            let near = neighbours.get(&s).map(Vec::as_slice).unwrap_or(&[]);
            let joined = (taken.contains(&s) || !first)
                .then(|| near.iter().find_map(|n| cf.front_of(*n)))
                .flatten();
            match joined {
                Some(id) => {
                    let f = cf
                        .fronts
                        .iter_mut()
                        .find(|f| f.id == id)
                        .expect("known front");
                    if let Err(i) = f.states.binary_search(&s) {
                        f.states.insert(i, s);
                    }
                    changed = true;
                }
                None => still.push(s),
            }
        }
        free = still;
    }
    // A contact with an enemy no front faces gets a front of its own (at the start, every enemy).
    let faced: BTreeSet<u16> = cf.fronts.iter().map(|f| f.enemy).collect();
    let mut by_enemy: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
    for s in &free {
        if let Some(e) = line[s].iter().find(|e| !faced.contains(e)) {
            by_enemy.entry(*e).or_default().push(*s);
        }
    }
    let mut created = Vec::new();
    for (e, states) in by_enemy {
        created.push(cf.draw(None, e, &states));
    }
    if first {
        // Every division to the fronts, by their states, in turn.
        let ids: Vec<u32> = cf.fronts.iter().map(|f| f.id).collect();
        let weights: Vec<usize> = cf.fronts.iter().map(|f| f.states.len().max(1)).collect();
        let total: usize = weights.iter().sum();
        let n = cf.divisions.len();
        let mut k = 0usize;
        for (i, id) in ids.iter().enumerate() {
            let take = if i + 1 == ids.len() {
                n - k
            } else {
                n * weights[i] / total.max(1)
            };
            for d in cf.divisions.iter_mut().skip(k).take(take) {
                d.place = Place::Front(*id);
                d.transit = 0;
            }
            k += take;
        }
    } else {
        // A new front takes the reserve.
        for id in created {
            for d in &mut cf.divisions {
                if d.place == Place::Reserve {
                    d.place = Place::Front(id);
                }
            }
        }
    }
    // What no front covers stands empty (FR8).
    cf.uncovered = line
        .keys()
        .filter(|s| cf.front_of(**s).is_none())
        .map(|s| (*s, 0))
        .collect();
    lost
}

/// A week of transit and of the reserve (FR4): the divisions on the way come one week nearer;
/// `reserve_step` of the reserve leaves for the front that lost most ground, `weeks` away.
pub fn move_week(
    rules: &FrontRules,
    cf: &mut CountryFronts,
    lost: &BTreeSet<u32>,
    weeks: impl Fn(Place, Place) -> u16,
) {
    for d in &mut cf.divisions {
        if d.transit != u16::MAX {
            d.transit = d.transit.saturating_sub(1);
        }
    }
    cf.reinforced = None;
    let Some(&target) = lost.iter().next() else {
        return;
    };
    let reserve: Vec<u32> = cf
        .divisions
        .iter()
        .filter(|d| d.place == Place::Reserve && d.transit == 0)
        .map(|d| d.id)
        .collect();
    let n = (reserve.len() as u64 * u64::from(rules.reserve_step)).div_ceil(1000) as usize;
    if n == 0 {
        return;
    }
    cf.send(
        &reserve[..n.min(reserve.len())],
        Place::Front(target),
        weeks,
    );
    cf.reinforced = Some(target);
}

/// What stands on a front this week, from its divisions ready there: men, fire and the class of
/// most of its men.
#[derive(Clone, Copy, Debug)]
pub struct FrontForce {
    pub men: u64,
    pub fire: Fixed,
    pub mobility: Mobility,
}

/// The force of each front, its divisions' share of their formations' men and fire.
pub fn forces(mil: &Military, cw: &CountryWar, cf: &CountryFronts) -> BTreeMap<u32, FrontForce> {
    // Fire, men and elements of each kind.
    let mut kind: BTreeMap<Kind, (Fixed, i64, Fixed)> = BTreeMap::new();
    for f in &cw.formations {
        if mil.types[f.unit as usize].domain != Domain::Land || f.colonial {
            continue;
        }
        let e = kind
            .entry(Kind::of(f))
            .or_insert((Fixed::ZERO, 0, Fixed::ZERO));
        e.0 += mil.land_fire(&f.equipment);
        e.1 += super::state::men(mil, f);
        e.2 += f.elements;
    }
    let mut out = BTreeMap::new();
    for (front, ds) in cf.on_line() {
        let (mut men, mut fire) = (0i64, Fixed::ZERO);
        let mut by_class: BTreeMap<Mobility, i64> = BTreeMap::new();
        for (k, el) in ds {
            let Some(&(kf, km, ke)) = kind.get(&k) else {
                continue;
            };
            if !ke.is_positive() {
                continue;
            }
            let part = (el / ke).min(Fixed::ONE);
            let m = (Fixed::from_int(km) * part).to_int_round();
            men += m;
            fire += kf * part;
            *by_class
                .entry(mil.spec(k.unit, k.generation as usize).mobility)
                .or_default() += m;
        }
        let mobility = by_class
            .into_iter()
            .max_by(|x, y| x.1.cmp(&y.1).then(y.0.cmp(&x.0)))
            .map_or(Mobility::Foot, |(m, _)| m);
        out.insert(
            front,
            FrontForce {
                men: men.max(0) as u64,
                fire,
                mobility,
            },
        );
    }
    out
}

/// A week's losses on the fronts: each division ready on a front loses the front's share, and
/// the formations of its kind lose those elements, with their equipment in proportion. The
/// divisions elsewhere (the reserve, in transit, sent nowhere) lose nothing.
pub fn take_front_losses(
    mil: &Military,
    cw: &mut CountryWar,
    cf: &mut CountryFronts,
    losses: &BTreeMap<u32, (u64, u64)>,
) {
    let mut cut_by_kind: BTreeMap<Kind, Fixed> = BTreeMap::new();
    for d in &mut cf.divisions {
        let Place::Front(f) = d.place else { continue };
        if d.transit > 0 {
            continue;
        }
        let Some(&(lost, men)) = losses.get(&f) else {
            continue;
        };
        if men == 0 || lost == 0 {
            continue;
        }
        let cut = d.elements * Fixed::from_ratio(lost.min(men) as i64, men as i64);
        d.elements -= cut;
        *cut_by_kind.entry(d.kind).or_default() += cut;
    }
    for (kind, cut) in cut_by_kind {
        let total: Fixed = cw
            .formations
            .iter()
            .filter(|f| {
                mil.types[f.unit as usize].domain == Domain::Land
                    && !f.colonial
                    && Kind::of(f) == kind
            })
            .map(|f| f.elements)
            .sum();
        if !total.is_positive() {
            continue;
        }
        let keep = ((total - cut) / total).clamp(Fixed::ZERO, Fixed::ONE);
        for f in cw.formations.iter_mut().filter(|f| {
            mil.types[f.unit as usize].domain == Domain::Land && !f.colonial && Kind::of(f) == kind
        }) {
            f.elements *= keep;
            for e in &mut f.equipment {
                e.1 *= keep;
            }
        }
    }
    cw.formations.retain(|f| f.elements.is_positive());
    cf.divisions
        .retain(|d| d.elements >= Fixed::from_ratio(1, 100));
}

/// Weeks by rail between two places: the distance between the centres of their fronts, by
/// `rail_km_per_week`, at least one. The reserve stands at the capital.
pub fn transit_weeks(
    rules: &FrontRules,
    cf: &CountryFronts,
    centre_km: &BTreeMap<u16, (i64, i64)>,
    capital: Option<u16>,
    from: Place,
    to: Place,
) -> u16 {
    let at = |p: Place| -> Option<(i64, i64)> {
        let states: Vec<u16> = match p {
            Place::Front(id) => cf.front(id)?.states.clone(),
            Place::Reserve | Place::Idle => vec![capital?],
        };
        let pts: Vec<(i64, i64)> = states
            .iter()
            .filter_map(|s| centre_km.get(s))
            .copied()
            .collect();
        if pts.is_empty() {
            return None;
        }
        let n = pts.len() as i64;
        Some((
            pts.iter().map(|p| p.0).sum::<i64>() / n,
            pts.iter().map(|p| p.1).sum::<i64>() / n,
        ))
    };
    let km = match (at(from), at(to)) {
        (Some(a), Some(b)) => {
            let (dx, dy) = ((a.0 - b.0).abs(), (a.1 - b.1).abs());
            // Octagonal distance: within 4 % of the straight line, in integers.
            dx.max(dy) + dx.min(dy) * 41 / 100
        }
        _ => 0,
    };
    (km / i64::from(rules.rail_km_per_week.max(1)) + 1).min(i64::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> FrontRules {
        FrontRules {
            division_elements: 15,
            rail_km_per_week: 1000,
            reserve_step: 500,
            objective_weight: 2000,
        }
    }

    fn kind(unit: u16) -> Kind {
        Kind {
            unit,
            generation: 0,
            called_up: false,
            raised: false,
        }
    }

    #[test]
    fn the_divisions_always_sum_to_the_formations() {
        let r = rules();
        let mut cf = CountryFronts::default();
        let mut kinds = BTreeMap::new();
        kinds.insert(kind(1), Fixed::from_int(100));
        kinds.insert(kind(2), Fixed::from_int(31));
        sync_divisions(&r, &mut cf, &kinds, |_| Place::Reserve);
        let sum = |cf: &CountryFronts, k| -> Fixed {
            cf.divisions
                .iter()
                .filter(|d| d.kind == k)
                .map(|d| d.elements)
                .sum()
        };
        assert_eq!(sum(&cf, kind(1)), Fixed::from_int(100));
        assert_eq!(cf.divisions.iter().filter(|d| d.kind == kind(1)).count(), 7);
        // Losses come off in proportion; a gain makes a new division.
        kinds.insert(kind(1), Fixed::from_int(73));
        kinds.insert(kind(2), Fixed::from_int(40));
        sync_divisions(&r, &mut cf, &kinds, |_| Place::Reserve);
        assert_eq!(sum(&cf, kind(1)), Fixed::from_int(73));
        assert_eq!(sum(&cf, kind(2)), Fixed::from_int(40));
    }

    #[test]
    fn a_deleted_front_leaves_its_divisions_idle() {
        let mut cf = CountryFronts::default();
        let id = cf.draw(None, 3, &[5, 6]);
        cf.divisions.push(Division {
            id: 99,
            kind: kind(1),
            elements: Fixed::from_int(15),
            place: Place::Front(id),
            transit: 0,
            from: None,
        });
        cf.delete(id);
        assert_eq!(cf.divisions[0].place, Place::Idle);
        assert!(cf.front_of(5).is_none());
    }

    #[test]
    fn a_stretch_left_open_stays_open() {
        let mut cf = CountryFronts::default();
        let a = cf.draw(None, 3, &[5, 6]);
        cf.draw(Some(a), 3, &[5]);
        assert_eq!(cf.left_open, vec![6]);
        let mut line = BTreeMap::new();
        for s in [5u16, 6] {
            line.insert(s, BTreeSet::from([3u16]));
        }
        let near = BTreeMap::from([(5u16, vec![6u16]), (6, vec![5])]);
        follow_line(&mut cf, &line, &near, &BTreeSet::new());
        assert!(
            cf.front_of(6).is_none(),
            "the line's following does not fill it back"
        );
        assert_eq!(cf.uncovered, vec![(6, 0)]);
    }

    #[test]
    fn a_state_drawn_leaves_its_old_front() {
        let mut cf = CountryFronts::default();
        let a = cf.draw(None, 3, &[5, 6, 7]);
        let b = cf.draw(None, 3, &[7, 8]);
        assert_eq!(cf.front(a).unwrap().states, vec![5, 6]);
        assert_eq!(cf.front_of(7), Some(b));
    }
}
