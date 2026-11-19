//! The war at sea, minimal (slice C2; `docs/war.pt-BR.md`, "Mar e ar"). Behind `Switches::war`,
//! and run only while a war is open.
//!
//! - **Fleets.** Every country at war keeps its navy in one sea zone, with the fixed mission
//!   "control" (S1, the player's missions, is the next slice). Stronger on the surface than its
//!   main enemy, it takes station at that enemy's gateway zone, the zone most of the enemy's
//!   shortest sea routes to the other countries' ports pass (Germany's is the North Sea): it
//!   blockades. Weaker, it stays at its own gateway: a fleet in being.
//! - **Battle.** A zone with fleets of countries at war with each other is fought with
//!   `naval_step`; ships sunk come off the formations.
//! - **Control.** The country with the most surface power left in a zone holds it.
//! - **Blockade, by pair.** The pair (B, X) is cut when its cheaper way is by sea
//!   (`Geography::by_sea`) and no sea route from B's ports reaches X's with the zones B's enemies
//!   hold closed. A pair by land, or with a free route (Germany and Sweden by the Baltic while
//!   the North Sea is British), stays open. The trade reads [`super::fight::blockaded_pairs`].
//!
//! Everything is integer fixed point and free of randomness.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::calc::{NavalSide, ShipGroup, naval_step};
use super::catalogue::{Domain, Military};
use crate::game::GameWorld;
use crate::sea::{RouteOptions, SeaGraph};

/// What the war at sea needs, built at load.
#[derive(Clone, Debug)]
pub struct SeaSetup {
    pub graph: Arc<SeaGraph>,
    /// For `by_sea`: the cheaper way between two markets.
    pub geo: Arc<crate::econ::trade::Geography>,
}

/// A navy at its station this week.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fleet {
    pub country: u16,
    pub zone: u16,
    pub ships: u64,
    /// The enemy whose gateway it holds (stronger: it blockades); `None` at home (weaker).
    pub against: Option<u16>,
}

/// The war at sea of the last week (`WarState::sea`); empty in peace.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeaWar {
    pub fleets: Vec<Fleet>,
    /// (zone, holder), sorted by zone: the zones with a fleet.
    pub control: Vec<(u16, u16)>,
    /// (blockader, blockaded), sorted: who cuts pairs of whom.
    pub blockades: Vec<(u16, u16)>,
    /// The trade pairs cut, (low, high), sorted.
    pub cut: Vec<(u16, u16)>,
}

impl SeaWar {
    pub fn is_empty(&self) -> bool {
        self.fleets.is_empty()
            && self.control.is_empty()
            && self.blockades.is_empty()
            && self.cut.is_empty()
    }

    /// The holder of `zone`, if a fleet holds it.
    pub fn holder(&self, zone: u16) -> Option<u16> {
        self.control
            .binary_search_by_key(&zone, |(z, _)| *z)
            .ok()
            .map(|i| self.control[i].1)
    }
}

impl HashState for SeaWar {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.fleets.len());
        for f in &self.fleets {
            h.write_u16(f.country);
            h.write_u16(f.zone);
            h.write_u64(f.ships);
            h.write_u16(f.against.unwrap_or(u16::MAX));
        }
        for list in [&self.control, &self.blockades, &self.cut] {
            h.write_len(list.len());
            for (a, b) in list {
                h.write_u16(*a);
                h.write_u16(*b);
            }
        }
    }
}

/// What a week at sea does: the new state and the ships lost, (country, type, generation,
/// ships): the ships sunk and the damage on those afloat, as a share of a ship (a damaged ship
/// is a ship out of action until the repair, which the yards' purchases pay for). By type and
/// generation, not by formation: the week's own step may reorder the formations before the
/// loss is applied.
#[derive(Clone, Debug, Default)]
pub struct SeaWeek {
    pub sea: SeaWar,
    pub sunk: Vec<(u16, u16, u8, Fixed)>,
}

/// The zones of a country's ports, sorted: of its metropolitan states, or of all its states
/// when it has no metropolitan coast.
pub fn port_zones(graph: &SeaGraph, world: &GameWorld, c: u16) -> Vec<u16> {
    let zones = |colonies: bool| -> Vec<u16> {
        let mut z: Vec<u16> = world
            .econ
            .regions
            .iter()
            .filter(|r| r.market == c && (colonies || !r.colony))
            .flat_map(|r| graph.ports_of(r.state).iter().map(|(z, _)| *z))
            .collect();
        z.sort_unstable();
        z.dedup();
        z
    };
    let metro = zones(false);
    if metro.is_empty() { zones(true) } else { metro }
}

/// The ports of every country with a coast, by market index.
fn all_ports(graph: &SeaGraph, world: &GameWorld) -> BTreeMap<u16, Vec<u16>> {
    (0..world.countries.len() as u16)
        .map(|c| (c, port_zones(graph, world, c)))
        .filter(|(_, z)| !z.is_empty())
        .collect()
}

/// A country's gateway: the zone most of its shortest sea routes to the other countries' ports
/// pass (ties to the lower id); `None` without a coast.
pub fn gateway(
    graph: &SeaGraph,
    ports: &BTreeMap<u16, Vec<u16>>,
    c: u16,
    o: &RouteOptions,
) -> Option<u16> {
    let sources = ports.get(&c)?;
    let (dist, prev) = graph.reach(sources, &BTreeSet::new(), o);
    let mut count: BTreeMap<u16, u32> = BTreeMap::new();
    for (&x, zones) in ports {
        if x == c {
            continue;
        }
        let Some(&end) = zones
            .iter()
            .filter(|z| dist[**z as usize] != u32::MAX)
            .min_by_key(|z| (dist[**z as usize], **z))
        else {
            continue;
        };
        let mut at = Some(end);
        while let Some(z) = at {
            *count.entry(z).or_default() += 1;
            at = prev[z as usize];
        }
    }
    count
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(z, _)| z)
        .or_else(|| sources.first().copied())
}

/// A country's ships by formation: (formation index, group).
fn navy(mil: &Military, world: &GameWorld, c: u16) -> Vec<(usize, ShipGroup)> {
    world
        .war
        .countries
        .get(c as usize)
        .map(|cw| {
            cw.formations
                .iter()
                .enumerate()
                .filter(|(_, f)| mil.types[f.unit as usize].domain == Domain::Sea)
                .filter_map(|(i, f)| {
                    let n = f.elements.to_int_floor().max(0) as u64;
                    (n > 0).then(|| (i, mil.ship_group(f.unit, f.generation as usize, n)))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn surface(groups: &[(usize, ShipGroup)]) -> Fixed {
    groups
        .iter()
        .filter(|(_, g)| !g.submerged)
        .map(|(_, g)| (g.gun + g.torpedo) * g.count as i64)
        .sum()
}

/// One week of the war at sea. Pure: reads the world.
pub fn sea_week(setup: &SeaSetup, mil: &Military, world: &GameWorld, date: &str) -> SeaWeek {
    let mut out = SeaWeek::default();
    let wars = &world.war.conflicts.wars;
    if wars.is_empty() {
        return out;
    }
    let at_war = |a: u16, b: u16| world.war.conflicts.at_war(a, b);
    let fighting: BTreeSet<u16> = wars
        .iter()
        .flat_map(|w| w.side_a.iter().chain(&w.side_b).copied())
        .collect();
    let graph = &*setup.graph;
    let o = RouteOptions::open(date);
    let ports = all_ports(graph, world);
    let navies: BTreeMap<u16, Vec<(usize, ShipGroup)>> = fighting
        .iter()
        .map(|&c| (c, navy(mil, world, c)))
        .filter(|(_, n)| !n.is_empty())
        .collect();
    let power: BTreeMap<u16, Fixed> = navies.iter().map(|(&c, n)| (c, surface(n))).collect();
    let mut gateways: BTreeMap<u16, Option<u16>> = BTreeMap::new();
    let mut gate = |c: u16| {
        *gateways
            .entry(c)
            .or_insert_with(|| gateway(graph, &ports, c, &o))
    };
    // Stations.
    for (&c, n) in &navies {
        let enemy = fighting
            .iter()
            .copied()
            .filter(|&e| at_war(c, e))
            .max_by(|a, b| {
                let pa = power.get(a).copied().unwrap_or(Fixed::ZERO);
                let pb = power.get(b).copied().unwrap_or(Fixed::ZERO);
                pa.cmp(&pb).then(b.cmp(a))
            });
        let mine = power[&c];
        let (zone, against) = match enemy {
            Some(e) if mine > power.get(&e).copied().unwrap_or(Fixed::ZERO) => match gate(e) {
                Some(z) => (Some(z), Some(e)),
                None => (gate(c), None),
            },
            _ => (gate(c), None),
        };
        let Some(zone) = zone else {
            continue;
        };
        out.sea.fleets.push(Fleet {
            country: c,
            zone,
            ships: n.iter().map(|(_, g)| g.count).sum(),
            against,
        });
    }
    // Battle and control, zone by zone.
    let mut by_zone: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
    for f in &out.sea.fleets {
        by_zone.entry(f.zone).or_default().push(f.country);
    }
    let mut left: BTreeMap<u16, Fixed> = power.clone();
    for (&zone, here) in &by_zone {
        // The strongest present and its enemies present.
        let lead = *here
            .iter()
            .max_by(|a, b| power[a].cmp(&power[b]).then(b.cmp(a)))
            .unwrap();
        let foes: Vec<u16> = here.iter().copied().filter(|&x| at_war(lead, x)).collect();
        if !foes.is_empty() {
            let friends: Vec<u16> = here
                .iter()
                .copied()
                .filter(|&x| !at_war(lead, x) && foes.iter().any(|&f| at_war(x, f)))
                .collect();
            let side = |members: &[u16]| -> (NavalSide, Vec<(u16, usize)>) {
                let mut groups = Vec::new();
                let mut index = Vec::new();
                let (mut trained, mut ships) = (Fixed::ZERO, 0i64);
                for &m in members {
                    for (fi, g) in &navies[&m] {
                        let t = world.war.countries[m as usize].formations[*fi].training;
                        trained += t * g.count as i64;
                        ships += g.count as i64;
                        groups.push(g.clone());
                        index.push((m, *fi));
                    }
                }
                let training = if ships > 0 {
                    trained / Fixed::from_int(ships)
                } else {
                    Fixed::ONE
                };
                (
                    NavalSide {
                        groups,
                        training,
                        // Fuel is not followed yet: full.
                        supply: Fixed::ONE,
                    },
                    index,
                )
            };
            let (sa, ia) = side(&friends.iter().copied().chain([lead]).collect::<Vec<_>>());
            let (sb, ib) = side(&foes);
            let o = naval_step(&mil.rules, [&sa, &sb]);
            for (i, (side_groups, index)) in [(&sa, &ia), (&sb, &ib)].into_iter().enumerate() {
                for (k, (m, fi)) in index.iter().enumerate() {
                    let g = &side_groups.groups[k];
                    let damaged = if g.tonnage.is_positive() {
                        o.damage_t[i][k] / g.tonnage
                    } else {
                        Fixed::ZERO
                    };
                    let n = Fixed::from_int(o.sunk[i][k] as i64) + damaged;
                    if n.is_positive() {
                        let f = &world.war.countries[*m as usize].formations[*fi];
                        out.sunk.push((*m, f.unit, f.generation, n));
                        if !g.submerged {
                            let lost = (g.gun + g.torpedo) * n;
                            if let Some(p) = left.get_mut(m) {
                                *p = (*p - lost).max(Fixed::ZERO);
                            }
                        }
                    }
                }
            }
        }
        let holder = here
            .iter()
            .copied()
            .filter(|c| left[c].is_positive())
            .max_by(|a, b| left[a].cmp(&left[b]).then(b.cmp(a)));
        if let Some(h) = holder {
            out.sea.control.push((zone, h));
        }
    }
    // The pairs cut: each country at war, with the zones its enemies hold closed.
    let mut cut = Vec::new();
    let mut blockades = Vec::new();
    for &b in &fighting {
        let Some(sources) = ports.get(&b) else {
            continue;
        };
        let blocked: BTreeSet<u16> = out
            .sea
            .control
            .iter()
            .filter(|&&(_, h)| at_war(h, b))
            .map(|&(z, _)| z)
            .collect();
        if blocked.is_empty() {
            continue;
        }
        let (dist, _) = graph.reach(sources, &blocked, &o);
        let mut any = false;
        for (&x, zones) in &ports {
            if x == b || at_war(b, x) || !setup.geo.by_sea(b as usize, x as usize) {
                continue;
            }
            if zones.iter().all(|z| dist[*z as usize] == u32::MAX) {
                cut.push((b.min(x), b.max(x)));
                any = true;
            }
        }
        if any {
            let mut by: BTreeSet<u16> = BTreeSet::new();
            for &(_, h) in out
                .sea
                .control
                .iter()
                .filter(|&&(z, _)| blocked.contains(&z))
            {
                by.insert(h);
            }
            for a in by {
                blockades.push((a, b));
            }
        }
    }
    cut.sort_unstable();
    cut.dedup();
    blockades.sort_unstable();
    blockades.dedup();
    out.sea.cut = cut;
    out.sea.blockades = blockades;
    out
}

/// Takes the ships sunk off the formations of their type and generation, in formation order,
/// their equipment in proportion.
pub fn sink(world: &mut GameWorld, sunk: &[(u16, u16, u8, Fixed)]) {
    for &(c, unit, generation, n) in sunk {
        let Some(cw) = world.war.countries.get_mut(c as usize) else {
            continue;
        };
        let mut left = n;
        for f in cw
            .formations
            .iter_mut()
            .filter(|f| f.unit == unit && f.generation == generation)
        {
            if !left.is_positive() {
                break;
            }
            if !f.elements.is_positive() {
                continue;
            }
            let lost = left.min(f.elements);
            let keep = (f.elements - lost) / f.elements;
            f.elements -= lost;
            left -= lost;
            for e in &mut f.equipment {
                e.1 *= keep;
            }
        }
    }
    for cw in &mut world.war.countries {
        cw.formations.retain(|f| f.elements.is_positive());
    }
}
