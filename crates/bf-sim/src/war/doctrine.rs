//! Generations of war (decision 1 of `docs/war.pt-BR.md`). An army's generation is its state,
//! with causes: an officer corps educated to act on intent, training, radio, motor vehicles and
//! its own experience of war; central political control of the officer corps holds it back. The
//! generation changes how a front moves:
//!
//! - **Second:** a linear front of fire and attrition, commanded from the centre. The front's
//!   elements follow the enemy ([`allocate_front`] by length, threat and objective) and the line
//!   moves segment by segment ([`crate::war::land_step`]).
//! - **Third:** units act on initiative within the commander's intent; the aim is the enemy's
//!   centre of gravity, not the line. The front concentrates on one focus segment, the weakest
//!   per kilometre on the way to the objective, and a segment broken there opens an exploitation
//!   that bypasses its neighbours, which are cut from supply ([`breakthrough`]).
//!
//! No draw anywhere: the readiness, the focus and the breakthrough come with their terms.

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use super::calc::{LandField, Segment, SegmentShare, apportion, terrain_speed};
use super::catalogue::{Mobility, Rules};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Generation {
    Second,
    Third,
}

/// The causes of an army's readiness for the third generation, each 0 to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DoctrineInputs {
    /// Share of officers with staff education that trains them to act on intent.
    pub officer_education: Fixed,
    /// Training completed by the army, on average.
    pub training: Fixed,
    /// Share of formations with radio.
    pub radio: Fixed,
    /// Share of the army's elements that move on wheels or tracks.
    pub motorization: Fixed,
    /// Experience of war in the last years, decaying (the defeated learn the most).
    pub experience: Fixed,
    /// Political control over the officer corps: commissars, purges, orders from the capital.
    pub central_control: Fixed,
}

/// Points per term (per mille); `total` = the sum, never below zero, at most 1,000.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Readiness {
    pub officer_education: u32,
    pub training: u32,
    pub radio: u32,
    pub motorization: u32,
    pub experience: u32,
    /// Points taken away.
    pub central_control: u32,
    pub total: u32,
}

fn pts(weight: u32, x: Fixed) -> u32 {
    (x.max(Fixed::ZERO).min(Fixed::ONE) * weight as i64).to_int_floor() as u32
}

pub fn readiness(rules: &Rules, i: &DoctrineInputs) -> Readiness {
    let d = &rules.doctrine;
    let mut r = Readiness {
        officer_education: pts(d.officer_education, i.officer_education),
        training: pts(d.training, i.training),
        radio: pts(d.radio, i.radio),
        motorization: pts(d.motorization, i.motorization),
        experience: pts(d.experience, i.experience),
        central_control: pts(d.central_control, i.central_control),
        total: 0,
    };
    let plus = r.officer_education + r.training + r.radio + r.motorization + r.experience;
    r.total = plus.saturating_sub(r.central_control).min(1000);
    r
}

/// The generation next year: up at `up`, back down below `down`, unchanged in between.
pub fn next_generation(rules: &Rules, current: Generation, readiness: &Readiness) -> Generation {
    match current {
        Generation::Second if readiness.total >= rules.doctrine.up => Generation::Third,
        Generation::Third if readiness.total < rules.doctrine.down => Generation::Second,
        g => g,
    }
}

/// Why a third-generation army fights as a second this week.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Degraded {
    /// Without radio the units cannot act on intent and still hold together.
    Radio,
    /// Without fuel there is no exploitation.
    Fuel,
}

/// The generation a front fights with this week, given the fills of radio and of fuel.
pub fn effective_generation(
    rules: &Rules,
    generation: Generation,
    radio: Fixed,
    fuel: Fixed,
) -> (Generation, Option<Degraded>) {
    let below = |x: Fixed, min: u32| x < Fixed::from_ratio(min as i64, 1000);
    match generation {
        Generation::Second => (Generation::Second, None),
        Generation::Third if below(radio, rules.doctrine.radio_min) => {
            (Generation::Second, Some(Degraded::Radio))
        }
        Generation::Third if below(fuel, rules.doctrine.fuel_min) => {
            (Generation::Second, Some(Degraded::Fuel))
        }
        Generation::Third => (Generation::Third, None),
    }
}

/// Spreads a front's elements over its segments by the front's generation.
///
/// - Second: by length × threat × objective pull ([`crate::war::allocate`]).
/// - Third: every segment keeps the `min_share` floor; of the rest, `focus_share` goes to the
///   focus segment and the remainder screens the others by length alone. The focus is the
///   segment with the least enemy power per kilometre among those that lead to the objective
///   (the centre of gravity), or among all when none does; ties go to the lower index.
pub fn allocate_front(
    rules: &Rules,
    elements: u64,
    segments: &[Segment],
    generation: Generation,
) -> Vec<SegmentShare> {
    if generation == Generation::Second || segments.len() < 2 {
        return super::calc::allocate(rules, elements, segments);
    }
    let n = segments.len() as u64;
    let floor =
        ((elements as u128 * rules.min_share.raw() as u128 / 1000) as u64).min(elements / n);
    let rest = elements - floor * n;
    let focus = focus_segment(segments);
    let to_focus = (rest as u128 * rules.doctrine.focus_share.min(1000) as u128 / 1000) as u64;
    let screen: Vec<Fixed> = segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i == focus {
                Fixed::ZERO
            } else {
                s.length_km.max(Fixed::ZERO)
            }
        })
        .collect();
    let mut spread = apportion(rest - to_focus, &screen);
    spread[focus] += to_focus;
    segments
        .iter()
        .zip(spread)
        .enumerate()
        .map(|(i, (s, r))| SegmentShare {
            weight: if i == focus {
                Fixed::ZERO
            } else {
                s.length_km.max(Fixed::ZERO)
            },
            floor,
            elements: floor + r,
            focus: i == focus,
        })
        .collect()
}

/// The focus segment of a third-generation front: the least enemy power per kilometre among the
/// segments that lead to the objective (all of them when none does); the first on a tie.
pub fn focus_segment(segments: &[Segment]) -> usize {
    let any_objective = segments.iter().any(|s| s.objective);
    let density = |s: &Segment| s.enemy_power.max(Fixed::ZERO) / s.length_km.max(Fixed::ONE);
    segments
        .iter()
        .enumerate()
        .filter(|(_, s)| s.objective || !any_objective)
        .min_by(|a, b| density(a.1).cmp(&density(b.1)).then(a.0.cmp(&b.0)))
        .map_or(0, |(i, _)| i)
}

/// What decides a breakthrough on one segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreakthroughInputs {
    /// The attacker's effective generation this week ([`effective_generation`]).
    pub generation: Generation,
    /// Powers of the week's land step on the segment ([`crate::war::LandOutcome::power`]).
    pub attacker_power: Fixed,
    pub defender_power: Fixed,
    /// Power of the defender's reserve that can reach the segment this week.
    pub defender_reserve: Fixed,
    pub mobility: Mobility,
}

/// The terms of a breakthrough, each shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breakthrough {
    pub happened: bool,
    /// The attacker fights in the third generation this week.
    pub third_generation: bool,
    /// Attacker's power over the defender's with the reserve committed.
    pub ratio: Option<Fixed>,
    pub ratio_needed: Fixed,
    /// The reserve brings the ratio back under what a breakthrough needs.
    pub reserve_holds: bool,
    /// The attacker moves fast enough to exploit (horse, wheels, tracks).
    pub mobile: bool,
    /// Kilometres of the exploitation this week; zero without a breakthrough.
    pub advance_km: Fixed,
    /// The segments on both sides are bypassed: their supply is cut until the gap closes.
    pub bypasses_neighbours: bool,
}

/// Whether a segment breaks this week. Only a third-generation, mobile attacker breaks through,
/// and only where its power is `breakthrough_ratio` times the defender's with the reserve
/// thrown in: a continuous front with depth and reserves, in prepared terrain (already in the
/// defender's power), holds.
pub fn breakthrough(rules: &Rules, field: &LandField, i: &BreakthroughInputs) -> Breakthrough {
    let d = &rules.doctrine;
    let third = i.generation == Generation::Third;
    let mobile = matches!(
        i.mobility,
        Mobility::Horse | Mobility::Wheeled | Mobility::Tracked
    );
    let defence = i.defender_power.max(Fixed::ZERO) + i.defender_reserve.max(Fixed::ZERO);
    let ratio = defence
        .is_positive()
        .then(|| i.attacker_power.max(Fixed::ZERO) / defence);
    let enough_without_reserve = i
        .defender_power
        .is_positive()
        .then(|| i.attacker_power / i.defender_power >= d.breakthrough_ratio)
        .unwrap_or(i.attacker_power.is_positive());
    let enough = ratio.map_or(i.attacker_power.is_positive(), |r| {
        r >= d.breakthrough_ratio
    });
    let reserve_holds = enough_without_reserve && !enough;
    let happened = third && mobile && enough;
    let advance_km = if happened {
        rules.base_speed(i.mobility)
            * d.exploitation_speed
            * terrain_speed(rules, field, i.mobility)
    } else {
        Fixed::ZERO
    };
    Breakthrough {
        happened,
        third_generation: third,
        ratio,
        ratio_needed: d.breakthrough_ratio,
        reserve_holds,
        mobile,
        advance_km,
        bypasses_neighbours: happened,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::war::calc::TerrainMix;
    use crate::war::calc::tests::rules;
    use crate::war::catalogue::Terrain;

    fn pm(n: i64) -> Fixed {
        Fixed::from_ratio(n, 1000)
    }

    fn open() -> LandField {
        LandField {
            terrain: TerrainMix::only(Terrain::Open),
            winter: false,
        }
    }

    #[test]
    fn readiness_adds_causes_and_central_control_holds_it_back() {
        let r = rules();
        let ready = DoctrineInputs {
            officer_education: Fixed::ONE,
            training: pm(800),
            radio: pm(700),
            motorization: pm(500),
            experience: pm(300),
            central_control: Fixed::ZERO,
        };
        let s = readiness(&r, &ready);
        assert_eq!(s.total, 300 + 160 + 140 + 100 + 30);
        assert_eq!(
            next_generation(&r, Generation::Second, &s),
            Generation::Third
        );
        // The same army under political control of its officers stays behind.
        let purged = readiness(
            &r,
            &DoctrineInputs {
                central_control: pm(500),
                ..ready
            },
        );
        assert_eq!(purged.central_control, 200);
        assert_eq!(
            next_generation(&r, Generation::Second, &purged),
            Generation::Second
        );
        // Hysteresis: a third-generation army keeps it until readiness falls below `down`.
        assert_eq!(
            next_generation(&r, Generation::Third, &purged),
            Generation::Third
        );
        let broken = readiness(&r, &DoctrineInputs::default());
        assert_eq!(
            next_generation(&r, Generation::Third, &broken),
            Generation::Second
        );
    }

    #[test]
    fn third_generation_without_radio_or_fuel_fights_as_second() {
        let r = rules();
        assert_eq!(
            effective_generation(&r, Generation::Third, Fixed::ONE, Fixed::ONE),
            (Generation::Third, None)
        );
        assert_eq!(
            effective_generation(&r, Generation::Third, pm(200), Fixed::ONE),
            (Generation::Second, Some(Degraded::Radio))
        );
        assert_eq!(
            effective_generation(&r, Generation::Third, Fixed::ONE, pm(100)),
            (Generation::Second, Some(Degraded::Fuel))
        );
        // Degraded, it cannot break through even with the power to.
        let (g, _) = effective_generation(&r, Generation::Third, Fixed::ONE, Fixed::ZERO);
        let b = breakthrough(
            &r,
            &open(),
            &BreakthroughInputs {
                generation: g,
                attacker_power: Fixed::from_int(100),
                defender_power: Fixed::from_int(10),
                defender_reserve: Fixed::ZERO,
                mobility: Mobility::Tracked,
            },
        );
        assert!(!b.happened && !b.third_generation);
    }

    #[test]
    fn a_reserve_behind_a_prepared_front_stops_the_breakthrough() {
        let r = rules();
        let input = BreakthroughInputs {
            generation: Generation::Third,
            attacker_power: Fixed::from_int(90),
            defender_power: Fixed::from_int(20),
            defender_reserve: Fixed::ZERO,
            mobility: Mobility::Tracked,
        };
        let b = breakthrough(&r, &open(), &input);
        assert!(b.happened && b.bypasses_neighbours);
        assert_eq!(b.advance_km, Fixed::from_int(300));
        let held = breakthrough(
            &r,
            &open(),
            &BreakthroughInputs {
                defender_reserve: Fixed::from_int(15),
                ..input
            },
        );
        assert!(!held.happened && held.reserve_holds);
        // Infantry on foot does not exploit.
        let foot = breakthrough(
            &r,
            &open(),
            &BreakthroughInputs {
                mobility: Mobility::Foot,
                ..input
            },
        );
        assert!(!foot.happened && !foot.mobile);
        // A second-generation attacker with the same power only pushes the line.
        let second = breakthrough(
            &r,
            &open(),
            &BreakthroughInputs {
                generation: Generation::Second,
                ..input
            },
        );
        assert!(!second.happened);
    }

    #[test]
    fn a_third_generation_front_concentrates_on_the_gap_to_the_objective() {
        let r = rules();
        let seg = |km: i64, enemy: i64, objective| Segment {
            length_km: Fixed::from_int(km),
            enemy_power: Fixed::from_int(enemy),
            objective,
        };
        // Segment 2 leads to the objective and is thin; segment 3 is thinner but leads nowhere.
        let segments = [
            seg(100, 40, false),
            seg(100, 40, true),
            seg(100, 10, true),
            seg(100, 1, false),
        ];
        assert_eq!(focus_segment(&segments), 2);
        let third = allocate_front(&r, 100, &segments, Generation::Third);
        let second = allocate_front(&r, 100, &segments, Generation::Second);
        assert_eq!(third.iter().map(|s| s.elements).sum::<u64>(), 100);
        assert!(third[2].focus && third.iter().filter(|s| s.focus).count() == 1);
        // 5 floor each (20), 80 left: 48 on the focus, 32 screening by length.
        assert_eq!(third[2].elements, 5 + 48);
        assert!(third[2].elements > second[2].elements);
        // The second generation masses where the enemy is.
        assert!(second[1].elements > second[2].elements);
        for _ in 0..10 {
            assert_eq!(allocate_front(&r, 100, &segments, Generation::Third), third);
        }
    }
}
