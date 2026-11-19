//! The generation of each country's army, read from what the country has (front pace, step 4):
//! its officers' schooling, its training, its newer equipment, its motor vehicles, its recent
//! experience of war and the political control of its officer corps after a change of regime.
//! The terms and their weights are `war::doctrine` (`rules.ron`, `doctrine`); this module only
//! measures the inputs in the world, so that the tick, the forces tab and the war screen read
//! the same numbers. Reasoning in `docs/war.pt-BR.md`, "Geração do exército".

use laplace_schelling::Fixed;

use super::catalogue::{Domain, Military, Mobility};
use super::doctrine::{DoctrineInputs, Generation, Readiness, next_generation, readiness};
use super::state::{CountryWar, men};
use crate::game::GameWorld;
use crate::politics::{GOVERNMENT_GROUP, Politics};

/// What holds a term of the readiness back, for the player to see the way up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// The schools do not teach the doctrine yet: its research (`command.doctrine_tech`) is
    /// at this stage, short of Industrial.
    SchoolWithoutDoctrine(crate::research::Stage),
    /// No tradition of command on record: the officers' term reads the people's schooling.
    NoTradition,
    /// The army is not fully trained (recruits dilute it).
    Training,
    /// No formation in a generation of equipment after 1925's.
    NoNewerEquipment,
    /// No formation on wheels or tracks.
    NoMotorized,
    /// No experience of war.
    NoExperience,
    /// The form of government changed: the officer corps answers to politics for this many
    /// more years.
    RegimeChange { years_left: u32 },
}

/// What the player reads: the generation, the readiness term by term, and the inputs.
#[derive(Clone, Debug, PartialEq)]
pub struct ArmyReading {
    /// The generation the army fights with (the one kept in the war state, or the one the
    /// readiness gives from the second when none is kept).
    pub generation: Generation,
    pub readiness: Readiness,
    pub inputs: DoctrineInputs,
    /// The army has a tradition of command on record; without one, the officers' term reads the
    /// schooling of the people (shown as "no tradition on record", not as zero).
    pub tradition: bool,
    /// The army's experience of war, weeks, decayed to today.
    pub experience_weeks: Fixed,
    /// What holds each term back, in the order of the terms.
    pub holds: [Option<Hold>; 6],
    /// The readiness at which the army moves up, and below which it falls back.
    pub up: u32,
    pub down: u32,
}

/// Share of the land men of `c` that `pick` selects.
fn land_share(mil: &Military, c: &CountryWar, pick: impl Fn(Mobility, u8) -> bool) -> Fixed {
    let (mut all, mut some) = (0i64, 0i64);
    for f in &c.formations {
        if mil.types[f.unit as usize].domain != Domain::Land || !f.elements.is_positive() {
            continue;
        }
        let n = men(mil, f);
        all += n;
        if pick(
            mil.spec(f.unit, f.generation as usize).mobility,
            f.generation,
        ) {
            some += n;
        }
    }
    if all > 0 {
        Fixed::from_ratio(some, all)
    } else {
        Fixed::ZERO
    }
}

/// Years since the last change of the form of government of `country` (a law of the
/// `government` group), if any.
fn years_since_regime_change(
    world: &GameWorld,
    politics: Option<&Politics>,
    country: usize,
    today: i64,
) -> Option<i64> {
    let p = politics?;
    let g = p.group(GOVERNMENT_GROUP)? as u16;
    world
        .politics
        .changes
        .iter()
        .rev()
        .find(|ch| {
            usize::from(ch.country) == country
                && p.laws.get(usize::from(ch.to)).is_some_and(|l| l.group == g)
        })
        .map(|ch| today - ch.date.to_days())
}

/// The share of the officer corps of `country` formed to act on intent in 1925, when its
/// army has a tradition on record (`command.tradition_1925`).
pub fn tradition_1925(world: &GameWorld, mil: &Military, country: usize) -> Option<Fixed> {
    let tag = &world.countries.iter().nth(country)?.1.tag;
    mil.rules
        .command
        .tradition_1925
        .iter()
        .find(|(t, _)| t == tag)
        .map(|(_, pm)| Fixed::from_ratio(i64::from(*pm), 1000))
}

/// Training of the land formations of `c`, weighted by their elements, 0 to 1.
pub fn land_training(mil: &Military, c: &CountryWar) -> Fixed {
    let (mut el, mut tr) = (Fixed::ZERO, Fixed::ZERO);
    for f in &c.formations {
        if mil.types[f.unit as usize].domain == Domain::Land && f.elements.is_positive() {
            el += f.elements;
            tr += f.training * f.elements;
        }
    }
    if el.is_positive() {
        tr / el
    } else {
        Fixed::ZERO
    }
}

/// The inputs of `country`'s army with what the war state keeps of it: the experience and,
/// for an army with a recorded tradition, the share of its officers formed to act on intent in
/// place of the schooling of its people (decision G7).
pub fn inputs_with(
    world: &GameWorld,
    mil: &Military,
    politics: Option<&Politics>,
    vehicles: &[u16],
    country: usize,
    kept: &crate::war::fight::ArmyDoctrine,
    today: i64,
) -> DoctrineInputs {
    let weeks = decayed(mil, kept.experience_weeks, today - kept.as_of);
    let mut i = doctrine_inputs(world, mil, politics, vehicles, country, weeks, today);
    if kept.tradition {
        i.officer_education = kept.intent;
    }
    i
}

/// The inputs of the readiness of `country`'s army on day `today` (days of the calendar),
/// with `experience` weeks of war (decayed) behind it; the officers read from the schooling of
/// the people (an army without a recorded tradition).
pub fn doctrine_inputs(
    world: &GameWorld,
    mil: &Military,
    politics: Option<&Politics>,
    vehicles: &[u16],
    country: usize,
    experience_weeks: Fixed,
    today: i64,
) -> DoctrineInputs {
    let d = &mil.rules.doctrine;
    let Some(c) = world.war.countries.get(country) else {
        return DoctrineInputs::default();
    };
    // Officers: a share (`command.untraditioned_share`) of the schooling of the metropole, by
    // its people: the officers come from those who read, but without a staff school's doctrine.
    let (mut people, mut read) = (0i64, Fixed::ZERO);
    for r in world
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == country && !r.colony)
    {
        let p = r.population();
        people += p;
        read += r.literacy * p;
    }
    let officer_education = if people > 0 {
        (read / people).mul_ratio(i64::from(mil.rules.command.untraditioned_share), 1000)
    } else {
        Fixed::ZERO
    };
    let training = land_training(mil, c);
    // Radio: the men in a generation of equipment after 1925's (the radio comes with it).
    let radio = land_share(mil, c, |_, g| g > 0);
    // Motorization: the land men on wheels or tracks as far as their formations hold the
    // vehicles (`composition::equipped`): a formation converted without lorries walks.
    let (wheels, tracks, all) = super::composition::equipped(mil, vehicles, c);
    let motorization = if all.is_positive() {
        ((wheels + tracks) / all).min(Fixed::ONE)
    } else {
        Fixed::ZERO
    };
    let full = Fixed::from_int(i64::from(d.experience_full_weeks.max(1)));
    let experience = (experience_weeks / full).min(Fixed::ONE);
    // Political control of the officer corps: full right after a change of regime, gone after
    // `politicized_years` (purges, commissars, a new loyalty asked of the officers).
    let span = 365 * i64::from(d.politicized_years.max(1));
    let central_control = years_since_regime_change(world, politics, country, today)
        .map_or(Fixed::ZERO, |days| {
            (Fixed::ONE - Fixed::from_ratio(days, span)).max(Fixed::ZERO)
        });
    DoctrineInputs {
        officer_education,
        training,
        radio,
        motorization,
        experience,
        central_control,
    }
}

/// Weeks of experience left after `days` from `weeks`, by the half-life in the rules.
pub fn decayed(mil: &Military, weeks: Fixed, days: i64) -> Fixed {
    let half = 365 * i64::from(mil.rules.doctrine.experience_half_life_years.max(1));
    if days <= 0 || !weeks.is_positive() {
        return weeks;
    }
    // exp(-ln2 · t / half), by halvings and a linear rest: deterministic and close enough.
    let mut w = weeks;
    let mut t = days;
    while t >= half && w.is_positive() {
        w /= Fixed::from_int(2);
        t -= half;
    }
    w - w * Fixed::from_ratio(t, 2 * half)
}

/// The reading of `country`'s army on day `today`: the generation kept in the war state (or
/// the second), the readiness and its inputs.
pub fn army_reading(
    world: &GameWorld,
    mil: &Military,
    politics: Option<&Politics>,
    tree: Option<&crate::research::TechTree>,
    vehicles: &[u16],
    country: usize,
    today: i64,
) -> ArmyReading {
    let kept = world.war.fighting.doctrine_of(country as u16).copied();
    let (inputs, weeks) = match &kept {
        Some(k) => (
            inputs_with(world, mil, politics, vehicles, country, k, today),
            decayed(mil, k.experience_weeks, today - k.as_of),
        ),
        None => {
            let mut i =
                doctrine_inputs(world, mil, politics, vehicles, country, Fixed::ZERO, today);
            if let Some(t) = tradition_1925(world, mil, country) {
                i.officer_education = t;
            }
            (i, Fixed::ZERO)
        }
    };
    let r = readiness(&mil.rules, &inputs);
    let generation = match kept {
        Some(k) => k.generation,
        None => next_generation(&mil.rules, Generation::Second, &r),
    };
    let tradition = tradition_1925(world, mil, country).is_some();
    let stage = tree
        .and_then(|t| t.tech(&mil.rules.command.doctrine_tech))
        .and_then(|t| world.research.stage.get(country)?.get(t).copied());
    let span = 365 * i64::from(mil.rules.doctrine.politicized_years.max(1));
    let left = years_since_regime_change(world, politics, country, today)
        .filter(|&d| d < span)
        .map(|d| ((span - d) / 365) as u32);
    let holds = [
        if !tradition {
            Some(Hold::NoTradition)
        } else {
            stage
                .filter(|&s| s < crate::research::Stage::Industrial)
                .map(Hold::SchoolWithoutDoctrine)
        },
        (inputs.training < Fixed::ONE).then_some(Hold::Training),
        (!inputs.radio.is_positive()).then_some(Hold::NoNewerEquipment),
        (!inputs.motorization.is_positive()).then_some(Hold::NoMotorized),
        (!inputs.experience.is_positive()).then_some(Hold::NoExperience),
        left.map(|years_left| Hold::RegimeChange { years_left }),
    ];
    ArmyReading {
        generation,
        readiness: r,
        inputs,
        tradition,
        experience_weeks: weeks,
        holds,
        up: mil.rules.doctrine.up,
        down: mil.rules.doctrine.down,
    }
}
