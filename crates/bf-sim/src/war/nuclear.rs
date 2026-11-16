//! Nuclear deterrence (decision 4 of `docs/war.pt-BR.md`). Deterrence is not a combat bonus: it
//! is a measured second strike, and it decides which rungs of the crisis ladder two states can
//! climb against each other.
//!
//! - [`second_strike`]: what one side still delivers after the other strikes first, hurdle by
//!   hurdle: survive the first strike (hardened silo, submarine at sea, bomber on alert, mobile
//!   launcher dispersed), reach the target, get through the defences.
//! - [`deterrence`]: each side's second strike against what the other holds as unacceptable
//!   damage; mutual, one-sided or none; and who gains by striking first.
//! - [`escalation_limits`]: the highest rung each side can climb against the other, and the
//!   escalation risk a direct war between two secure powers carries each week, a cost the
//!   decision weighs rather than a draw.
//!
//! Expected values throughout: no draw.

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use super::calc::apportion;
use super::catalogue::Rules;

/// How a part of the force is based, which decides how it survives a first strike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Basing {
    /// Hardened silo: always a target, hard to kill.
    Silo,
    /// Submarine: safe at sea, a soft target in port.
    Submarine,
    /// Bomber: safe on alert (airborne or ready to launch on warning), a soft target on the
    /// ground; must get through air defences.
    Bomber,
    /// Mobile launcher: safe when dispersed, a soft target in garrison.
    Mobile,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NuclearForce {
    pub basing: Basing,
    pub launchers: u64,
    pub warheads_per_launcher: u32,
    /// Share on alert, at sea or dispersed: out of reach of a first strike. Ignored for silos.
    pub alert: Fixed,
    /// Share of the launchers with the range to reach the other side.
    pub reach: Fixed,
}

/// One part of the force, hurdle by hurdle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hurdles {
    pub basing: Basing,
    pub launchers: u64,
    /// Exposed launchers the first strike destroyed.
    pub destroyed: u64,
    pub surviving: u64,
    /// Warheads of the survivors that can reach the target.
    pub reaching: Fixed,
    /// Warheads that get through air defences (bombers) and missile defence.
    pub delivered: Fixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SecondStrike {
    pub parts: Vec<Hurdles>,
    /// Share of command and control that survives to order the retaliation.
    pub command: Fixed,
    pub delivered: Fixed,
}

/// What `forces` still deliver after the other side spends `first_strike` warheads on them
/// (counterforce), facing a missile defence that stops `defense` of what comes (0 to 1).
///
/// The attacker's warheads are spread over the exposed launchers in proportion to their number,
/// largest remainder first: every silo, and the share of submarines, bombers and mobile launchers
/// not on alert. A warhead destroys a silo with `silo_kill`, a soft target with `soft_kill`.
pub fn second_strike(
    rules: &Rules,
    forces: &[NuclearForce],
    first_strike: u64,
    defense: Fixed,
    command: Fixed,
) -> SecondStrike {
    let n = &rules.nuclear;
    let clamp = |x: Fixed| x.max(Fixed::ZERO).min(Fixed::ONE);
    let exposed: Vec<u64> = forces
        .iter()
        .map(|f| match f.basing {
            Basing::Silo => f.launchers,
            _ => {
                let safe = (Fixed::from_int(f.launchers as i64) * clamp(f.alert)).to_int_floor();
                f.launchers - (safe.max(0) as u64).min(f.launchers)
            }
        })
        .collect();
    let aimed = apportion(
        if exposed.iter().sum::<u64>() == 0 {
            0
        } else {
            first_strike
        },
        &exposed
            .iter()
            .map(|e| Fixed::from_int(*e as i64))
            .collect::<Vec<_>>(),
    );
    let mut parts = Vec::with_capacity(forces.len());
    let command = clamp(command);
    for ((f, e), a) in forces.iter().zip(&exposed).zip(aimed) {
        let kill = if f.basing == Basing::Silo {
            n.silo_kill
        } else {
            n.soft_kill
        };
        let destroyed = ((a as u128 * kill.min(1000) as u128 / 1000) as u64).min(*e);
        let surviving = f.launchers - destroyed;
        let reaching =
            Fixed::from_int(surviving as i64 * f.warheads_per_launcher as i64) * clamp(f.reach);
        let through_air = if f.basing == Basing::Bomber {
            Fixed::from_ratio(n.bomber_penetration as i64, 1000)
        } else {
            Fixed::ONE
        };
        let delivered = reaching * through_air * (Fixed::ONE - clamp(defense)) * command;
        parts.push(Hurdles {
            basing: f.basing,
            launchers: f.launchers,
            destroyed,
            surviving,
            reaching,
            delivered,
        });
    }
    SecondStrike {
        delivered: parts.iter().map(|p| p.delivered).sum(),
        parts,
        command,
    }
}

/// Everything a first strike can throw: the warheads of every launcher that reaches.
pub fn first_strike_warheads(forces: &[NuclearForce]) -> u64 {
    forces
        .iter()
        .map(|f| {
            (Fixed::from_int(f.launchers as i64 * f.warheads_per_launcher as i64)
                * f.reach.max(Fixed::ZERO).min(Fixed::ONE))
            .to_int_floor()
            .max(0) as u64
        })
        .sum()
}

/// One side of a nuclear dyad.
#[derive(Clone, Debug, PartialEq)]
pub struct NuclearSide {
    pub forces: Vec<NuclearForce>,
    /// Share of incoming warheads its missile defence stops, 0 to 1.
    pub defense: Fixed,
    /// Share of its command that survives a first strike, 0 to 1.
    pub command: Fixed,
    /// Warheads delivered on it that it holds as unacceptable damage: its cities and industry,
    /// counted by the map (the number of targets a countervalue strike needs).
    pub unacceptable: Fixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Deterrence {
    /// What each side delivers after the other strikes first with everything.
    pub second_strike: [SecondStrike; 2],
    /// Each side's second strike reaches the other's unacceptable damage.
    pub secure: [bool; 2],
    /// Both secure: the balance of terror holds.
    pub mutual: bool,
    /// A side gains by striking first: it is secure and the other is not.
    pub first_strike_incentive: [bool; 2],
}

pub fn deterrence(rules: &Rules, sides: [&NuclearSide; 2]) -> Deterrence {
    let strike = |me: usize| {
        let foe = 1 - me;
        second_strike(
            rules,
            &sides[me].forces,
            first_strike_warheads(&sides[foe].forces),
            sides[foe].defense,
            sides[me].command,
        )
    };
    let ss = [strike(0), strike(1)];
    let secure = [0, 1].map(|me| {
        let foe = 1 - me;
        sides[foe].unacceptable.is_positive() && ss[me].delivered >= sides[foe].unacceptable
    });
    Deterrence {
        mutual: secure[0] && secure[1],
        first_strike_incentive: [secure[0] && !secure[1], secure[1] && !secure[0]],
        secure,
        second_strike: ss,
    }
}

/// The crisis ladder of `docs/history-rails.pt-BR.md`, with the nuclear rungs on top.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rung {
    Tension,
    Rivalry,
    ArmsRace,
    Crisis,
    /// Direct conventional war between the two.
    War,
    /// Nuclear threat to compel (an ultimatum backed by a first strike the other cannot answer).
    NuclearCoercion,
    NuclearUse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscalationLimits {
    /// Highest rung each side can climb against the other.
    pub ceiling: [Rung; 2],
    /// Escalation risk per week of direct war, per mille: zero unless both are nuclear.
    pub risk_per_week: u32,
}

/// Which rungs deterrence leaves open:
/// - neither side secure: the ladder is open up to war; a side that can strike without an
///   answer can also coerce;
/// - one side secure: it can coerce; the other stops at crisis (war would invite the strike it
///   cannot answer);
/// - both secure: war stays open but carries `escalation_risk_per_week`; coercion is not
///   credible for either, and use is suicide, so the ceiling for both is war.
pub fn escalation_limits(rules: &Rules, d: &Deterrence, armed: [bool; 2]) -> EscalationLimits {
    let ceiling = [0, 1].map(|me| {
        let foe = 1 - me;
        match (d.secure[me], d.secure[foe]) {
            (true, false) => Rung::NuclearUse,
            (false, true) => Rung::Crisis,
            (true, true) => Rung::War,
            (false, false) if armed[me] && !armed[foe] => Rung::NuclearCoercion,
            (false, false) => Rung::War,
        }
    });
    EscalationLimits {
        ceiling,
        risk_per_week: if armed[0] && armed[1] {
            rules.nuclear.escalation_risk_per_week
        } else {
            0
        },
    }
}

/// Share of the targets a countervalue strike of `delivered` warheads destroys, given the number
/// of targets (cities and industrial centres) the map counts for the victim.
pub fn countervalue_share(delivered: Fixed, targets: u64) -> Fixed {
    if targets == 0 {
        return Fixed::ZERO;
    }
    (delivered / targets as i64)
        .max(Fixed::ZERO)
        .min(Fixed::ONE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::war::calc::tests::rules;

    fn pm(n: i64) -> Fixed {
        Fixed::from_ratio(n, 1000)
    }

    fn force(basing: Basing, launchers: u64, alert: i64) -> NuclearForce {
        NuclearForce {
            basing,
            launchers,
            warheads_per_launcher: 1,
            alert: pm(alert),
            reach: Fixed::ONE,
        }
    }

    fn side(forces: Vec<NuclearForce>, unacceptable: i64) -> NuclearSide {
        NuclearSide {
            forces,
            defense: Fixed::ZERO,
            command: Fixed::ONE,
            unacceptable: Fixed::from_int(unacceptable),
        }
    }

    #[test]
    fn submarines_at_sea_survive_and_silos_need_two_warheads() {
        let r = rules();
        let forces = [
            force(Basing::Silo, 100, 0),
            force(Basing::Submarine, 10, 600),
        ];
        let s = second_strike(&r, &forces, 200, Fixed::ZERO, Fixed::ONE);
        // 104 exposed (100 silos, 4 submarines in port): 192 + 8 warheads.
        assert_eq!(s.parts[0].destroyed, 96);
        assert_eq!(s.parts[1].destroyed, 4);
        assert_eq!(s.parts[1].surviving, 6);
        assert_eq!(s.delivered, Fixed::from_int(10));
        // Without the submarines at sea the same strike leaves only four silos.
        let ashore = [force(Basing::Silo, 100, 0), force(Basing::Submarine, 10, 0)];
        let s = second_strike(&r, &ashore, 200, Fixed::ZERO, Fixed::ONE);
        assert!(s.delivered < Fixed::from_int(10));
    }

    #[test]
    fn deterrence_is_mutual_only_with_secure_second_strikes() {
        let r = rules();
        let a = side(
            vec![
                force(Basing::Submarine, 20, 700),
                force(Basing::Silo, 50, 0),
            ],
            10,
        );
        let b = side(vec![force(Basing::Bomber, 40, 0)], 10);
        let d = deterrence(&r, [&a, &b]);
        assert!(d.secure[0] && !d.secure[1]);
        assert_eq!(d.first_strike_incentive, [true, false]);
        let l = escalation_limits(&r, &d, [true, true]);
        assert_eq!(l.ceiling, [Rung::NuclearUse, Rung::Crisis]);
        // Bombers on alert make b secure too: war stays open, with a weekly risk.
        let b = side(vec![force(Basing::Bomber, 40, 600)], 10);
        let d = deterrence(&r, [&a, &b]);
        assert!(d.mutual);
        let l = escalation_limits(&r, &d, [true, true]);
        assert_eq!(l.ceiling, [Rung::War, Rung::War]);
        assert_eq!(l.risk_per_week, 5);
        // Swapping the sides swaps the result.
        let s = deterrence(&r, [&b, &a]);
        assert_eq!(s.secure, [d.secure[1], d.secure[0]]);
    }

    #[test]
    fn missile_defense_can_break_the_other_sides_second_strike() {
        let r = rules();
        let a = side(vec![force(Basing::Silo, 30, 0)], 10);
        let b = side(vec![force(Basing::Mobile, 30, 600)], 10);
        let d = deterrence(&r, [&a, &b]);
        assert!(d.secure[1]);
        // a builds a defence that stops two thirds: b's survivors no longer suffice.
        let shielded = NuclearSide {
            defense: pm(667),
            ..a.clone()
        };
        let d = deterrence(&r, [&shielded, &b]);
        assert!(!d.secure[1]);
        assert!(d.first_strike_incentive[0] || !d.secure[0]);
    }

    #[test]
    fn a_monopoly_can_coerce_and_countervalue_saturates() {
        let r = rules();
        let none = Deterrence {
            second_strike: [
                SecondStrike {
                    parts: vec![],
                    command: Fixed::ONE,
                    delivered: Fixed::ZERO,
                },
                SecondStrike {
                    parts: vec![],
                    command: Fixed::ONE,
                    delivered: Fixed::ZERO,
                },
            ],
            secure: [false, false],
            mutual: false,
            first_strike_incentive: [false, false],
        };
        let l = escalation_limits(&r, &none, [true, false]);
        assert_eq!(l.ceiling, [Rung::NuclearCoercion, Rung::War]);
        assert_eq!(l.risk_per_week, 0);
        assert_eq!(countervalue_share(Fixed::from_int(5), 10), pm(500));
        assert_eq!(countervalue_share(Fixed::from_int(50), 10), Fixed::ONE);
    }
}
