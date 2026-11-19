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
