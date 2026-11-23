//! What the "Balance of deterrence" panel reads (`bf_sim::war::balance`): the nuclear balance
//! between the player and the other side of a crisis or a war, as the player reads it (its own
//! arsenal whole, the other's untested arsenal at the opacity of N3). Read only.

use bf_sim::game::GameWorld;
use bf_sim::war::{Military, Rung};

/// One side as the panel shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct PanelSide {
    pub tag: String,
    /// Warheads as the player reads them (its own whole; the other's at the opacity).
    pub warheads: u64,
    /// Untested: read at the opacity (N3).
    pub opaque: bool,
    /// The protectors whose arsenals the umbrella adds (N9).
    pub protectors: Vec<String>,
    /// What it still delivers after the other strikes first, and what the other holds as
    /// unacceptable.
    pub delivered: f64,
    pub unacceptable_to_other: f64,
    pub secure: bool,
    pub first_strike_incentive: bool,
    /// The highest rung it can climb against the other.
    pub ceiling: Rung,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Panel {
    /// The player's side first.
    pub sides: [PanelSide; 2],
    pub mutual: bool,
    /// Escalation risk per week of direct war, per mille.
    pub risk_per_week: u32,
    /// The opacity of an untested arsenal, per mille (N3).
    pub opaque_weight: u32,
    /// In a war, what a use would do (step 4): `None` in a crisis or without a warhead ready.
    pub strike: Option<StrikeView>,
}

/// What the player reads before a use (step 4; N5–N7).
#[derive(Clone, Debug, PartialEq)]
pub struct StrikeView {
    pub target: String,
    /// Warheads on launchers, ready to go.
    pub deliverable: u64,
    /// Dead a warhead that arrives makes, on the cities and on the forces.
    pub per_countervalue: u64,
    pub per_counterforce: u64,
    /// Men of the target's land forces a counterforce warhead puts out.
    pub counterforce_men: u64,
    /// The target's people: no strike kills more.
    pub target_people: u64,
    /// The stain on the security reputation, in years of our forces' cost, per mille.
    pub stain: u32,
    /// The taboo, per mille, and the years since the last use in the world (N7).
    pub taboo: u32,
    pub years_since: Option<i64>,
}

/// The use `me` could make on `other`, `days` into the calendar.
pub fn strike_view(
    world: &GameWorld,
    mil: &Military,
    me: u16,
    other: u16,
    days: i64,
) -> Option<StrikeView> {
    let c = world.war.countries.get(usize::from(me))?;
    let deliverable = bf_sim::war::nuclear_use::deliverable(mil, c);
    if deliverable == 0 {
        return None;
    }
    let n = &mil.rules.nuclear;
    let nuc = &world.war.nuclear;
    // U3: the dead of one of our warheads, at their mean equivalent megatons.
    let (sum, count) = bf_sim::war::balance::warhead_emt(mil, c);
    let emt = if count > 0 {
        (sum / count).max(0) as u64
    } else {
        1000
    };
    // The density of the target: the dead of that warhead on the target's largest city.
    let cities = bf_sim::war::target::cities(world, other);
    let area = bf_sim::war::target::area_of(n.lethal_km2_per_emt, emt);
    let per = bf_sim::war::target::plan(&cities, n.core_density, &[area])
        .first()
        .map_or(0, |x| x.1.max(0) as u64);
    Some(StrikeView {
        target: world
            .countries
            .iter()
            .nth(usize::from(other))
            .map_or_else(String::new, |(_, c)| c.tag.clone()),
        deliverable,
        per_countervalue: per,
        per_counterforce: per * u64::from(n.counterforce_civilians) / 1000,
        counterforce_men: u64::from(n.counterforce_men),
        target_people: world
            .econ
            .regions
            .iter()
            .filter(|r| r.market == other)
            .map(|r| r.population().max(0) as u64)
            .sum(),
        stain: n.use_stain,
        taboo: nuc.taboo,
        years_since: nuc.last_use.map(|(y, m, d)| {
            (days - laplace_schelling::calendar::Date::new(y, m, d).to_days()).max(0) / 365
        }),
    })
}

/// The balance between `me` and `other`; `None` when neither holds a warhead.
pub fn read(world: &GameWorld, mil: &Military, me: u16, other: u16) -> Option<Panel> {
    let b = bf_sim::war::balance::balance(mil, world, me, other, Some(me))?;
    let tag = |m: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(m))
            .map_or_else(String::new, |(_, c)| c.tag.clone())
    };
    let side = |i: usize| {
        let s = &b.sides[i];
        PanelSide {
            tag: tag(s.country),
            warheads: s.warheads,
            opaque: s.opaque,
            protectors: s.protectors.iter().map(|&p| tag(p)).collect(),
            delivered: b.deterrence.second_strike[i].delivered.to_f64_lossy(),
            unacceptable_to_other: b.sides[1 - i].unacceptable.to_f64_lossy(),
            secure: b.deterrence.secure[i],
            first_strike_incentive: b.deterrence.first_strike_incentive[i],
            ceiling: b.limits.ceiling[i],
        }
    };
    Some(Panel {
        sides: [side(0), side(1)],
        mutual: b.deterrence.mutual,
        risk_per_week: b.limits.risk_per_week,
        opaque_weight: mil.rules.nuclear.opaque_weight,
        strike: None,
    })
}

pub fn rung_key(r: Rung) -> &'static str {
    match r {
        Rung::Tension => "nuclear-rung-tension",
        Rung::Rivalry => "nuclear-rung-rivalry",
        Rung::ArmsRace => "nuclear-rung-arms_race",
        Rung::Crisis => "nuclear-rung-crisis",
        Rung::War => "nuclear-rung-war",
        Rung::NuclearCoercion => "nuclear-rung-coercion",
        Rung::NuclearUse => "nuclear-rung-use",
    }
}
