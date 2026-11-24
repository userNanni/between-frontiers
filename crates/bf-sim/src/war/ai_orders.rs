//! The AI orders new formations (item 11 of the war's missing functions; the orchestrator's
//! design of 2026-11-23). In peace only the land army follows its men of service age; the fleet
//! and the air force kept their 1925 elements and only changed generation, so whatever was left
//! of the military ceiling bought goods for the depot. Each January a country without a player
//! turns a share of that leftover into orders of new ships and aircraft (the A2 queue, the same
//! the player uses), split between the sea and the air by one rule for everyone: the sea weighs
//! more with the share of the metropole's states that have a port and with the people in its
//! colonies. Within each service the order keeps the shape the force has (each type by its share
//! of the service's value), in the newest generation the country can equip. The treaties in force
//! cap what they cap: capital ships and carriers by tonnage (Washington, 1922), men under arms
//! (Versailles, art. 160).
//!
//! Every number of `AiOrderRule` is a DESIGN VALUE, without a source: the split of military
//! spending by service has no page here, and a table by country is calibration.

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use super::catalogue::{Domain, Military};
use super::state::{CountryWar, ForceOrder};
use crate::game::GameWorld;
use crate::treaty::{ArmsCategory, ClauseKind};

/// `data/military/peace.ron`, `ai_orders`. Per mille throughout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiOrderRule {
    /// Share of the year's leftover (the room above the needs that went to the depot) ordered
    /// as new formations.
    pub share: u32,
    /// The sea's weight: a base, plus this per mille of the metropole's states with a port, plus
    /// this per mille of the people in colonies.
    pub sea_base: u32,
    pub sea_coast: u32,
    pub sea_colonies: u32,
    /// The air's weight.
    pub air_base: u32,
}

/// The caps the treaties in force put on `tag`: (category, max), the lowest of each.
pub fn caps(world: &GameWorld, tag: &str) -> Vec<(ArmsCategory, u32)> {
    let mut out: Vec<(ArmsCategory, u32)> = Vec::new();
    for a in &world.treaties.agreements {
        for c in &a.kept {
            let ClauseKind::ArmsLimit { caps } = &c.kind else {
                continue;
            };
            for cap in caps.iter().filter(|x| x.party == tag) {
                match out.iter_mut().find(|(k, _)| *k == cap.category) {
                    Some(e) => e.1 = e.1.min(cap.max),
                    None => out.push((cap.category, cap.max)),
                }
            }
        }
    }
    out
}

/// The capped category of a unit type, by key.
fn category(key: &str) -> Option<ArmsCategory> {
    match key {
        "battleship" => Some(ArmsCategory::CapitalShips),
        "carrier" => Some(ArmsCategory::Carriers),
        _ => None,
    }
}

/// The sea's and the air's weights for country `ci` (`coastal`: map states with a port).
pub fn weights(
    rule: &AiOrderRule,
    world: &GameWorld,
    ci: usize,
    coastal: Option<&std::collections::BTreeSet<u16>>,
) -> (u64, u64) {
    let (mut states, mut ports, mut home, mut colony) = (0u64, 0u64, 0i64, 0i64);
    let mut seen = std::collections::BTreeSet::new();
    for r in world
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == ci)
    {
        if r.colony {
            colony += r.population();
            continue;
        }
        home += r.population();
        if seen.insert(r.state) {
            states += 1;
            if coastal.is_some_and(|c| c.contains(&r.state)) {
                ports += 1;
            }
        }
    }
    let coast = ports * 1000 / states.max(1);
    let colonies = (colony.max(0) as u64) * 1000 / ((home + colony).max(1) as u64);
    let sea = u64::from(rule.sea_base)
        + coast * u64::from(rule.sea_coast) / 1000
        + colonies * u64::from(rule.sea_colonies) / 1000;
    (sea, u64::from(rule.air_base))
}

/// The orders of country `ci` for the year, from `budget` (a year's money for new formations):
/// by service, each type by its share of the service's value, whole elements in the newest
/// generation it can equip, within the treaty caps; none of a type that already has an order
/// open. `newest(ty)` gives that generation (`None`: the type cannot be equipped).
#[allow(clippy::too_many_arguments)]
pub fn orders(
    mil: &Military,
    c: &CountryWar,
    budget: Fixed,
    weights: (u64, u64),
    caps: &[(ArmsCategory, u32)],
    prices: &[Fixed],
    newest: &dyn Fn(u16) -> Option<u8>,
) -> Vec<ForceOrder> {
    let mut out = Vec::new();
    if !budget.is_positive() || weights.0 + weights.1 == 0 {
        return out;
    }
    for (domain, w) in [(Domain::Sea, weights.0), (Domain::Air, weights.1)] {
        let money = budget.mul_ratio(w as i64, (weights.0 + weights.1) as i64);
        // The service's types and their value now (elements at the price of their equipment).
        let cost_of = |ty: u16, g: u8| -> Fixed {
            mil.spec(ty, usize::from(g))
                .equipment
                .iter()
                .map(|(x, q)| *q * prices.get(usize::from(*x)).copied().unwrap_or(Fixed::ZERO))
                .sum()
        };
        let mut types: Vec<(u16, Fixed)> = Vec::new();
        for f in c
            .formations
            .iter()
            .filter(|f| !f.colonial && mil.types[f.unit as usize].domain == domain)
        {
            let value = f.elements * cost_of(f.unit, f.generation);
            match types.iter_mut().find(|(t, _)| *t == f.unit) {
                Some(e) => e.1 += value,
                None => types.push((f.unit, value)),
            }
        }
        types.sort_by_key(|(t, _)| *t);
        let total: Fixed = types.iter().map(|(_, v)| *v).sum();
        if !total.is_positive() {
            continue;
        }
        for (ty, value) in types {
            let key = &mil.types[ty as usize].key;
            if c.queue
                .iter()
                .any(|o| &o.unit == key && o.ordered > o.delivered)
            {
                continue;
            }
            let Some(generation) = newest(ty) else {
                continue;
            };
            let spec = mil.spec(ty, usize::from(generation));
            let cost = cost_of(ty, generation);
            if !cost.is_positive() {
                continue;
            }
            let mut n = (money * value / total / cost).to_int_floor();
            // The treaty caps, by tonnage.
            if let Some(cat) = category(key)
                && let Some(&(_, max)) = caps.iter().find(|(k, _)| *k == cat)
            {
                let tonnage = i64::from(spec.tonnage.max(1));
                let now: i64 = c
                    .formations
                    .iter()
                    .filter(|f| f.unit == ty)
                    .map(|f| {
                        f.elements.to_int_floor()
                            * i64::from(mil.spec(ty, usize::from(f.generation)).tonnage)
                    })
                    .sum();
                n = n.min(((i64::from(max) - now) / tonnage).max(0));
            }
            if n >= 1 {
                out.push(ForceOrder {
                    unit: key.clone(),
                    generation: Some(generation),
                    colonial: false,
                    ordered: Fixed::from_int(n),
                    delivered: Fixed::ZERO,
                    built: Vec::new(),
                    buying: Vec::new(),
                    stalled: 0,
                });
            }
        }
    }
    out
}
