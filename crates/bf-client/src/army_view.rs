//! The generation of the player's army (`docs/war.pt-BR.md`, "Geração do exército"): second or
//! third, the readiness term by term with what the player does in peace to move each, what holds
//! each back, and the thresholds (`war::army::army_reading`). Read with the forces, once a month,
//! only with the war rule on.

use bf_sim::game::GameWorld;
use bf_sim::war::army::{Hold, army_reading};
use bf_sim::war::doctrine::Generation;

/// One term of the readiness.
#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    /// `officer_education`, `training`, `radio`, `motorization`, `experience` or
    /// `central_control` (which takes away).
    pub key: &'static str,
    /// Points it gives (or takes), and the most it can.
    pub points: u32,
    pub most: u32,
    /// The cause behind it, 0 to 1.
    pub input: f64,
    pub hold: Option<HoldView>,
}

/// What holds a term back, for the screen.
#[derive(Clone, Debug, PartialEq)]
pub enum HoldView {
    /// The schools do not teach the doctrine: the research's stage key (`tech-stage-…`).
    SchoolWithoutDoctrine(&'static str),
    NoTradition,
    Training,
    NoNewerEquipment,
    NoMotorized,
    NoExperience,
    RegimeChange {
        years_left: u32,
    },
}

impl HoldView {
    /// The text key of the hold.
    pub fn key(&self) -> &'static str {
        match self {
            HoldView::SchoolWithoutDoctrine(_) => "army-hold-school",
            HoldView::NoTradition => "army-hold-tradition",
            HoldView::Training => "army-hold-training",
            HoldView::NoNewerEquipment => "army-hold-equipment",
            HoldView::NoMotorized => "army-hold-motorized",
            HoldView::NoExperience => "army-hold-experience",
            HoldView::RegimeChange { .. } => "army-hold-regime",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArmyView {
    /// 2 or 3.
    pub generation: u8,
    pub total: u32,
    /// It moves up at `up` and falls back below `down`.
    pub up: u32,
    pub down: u32,
    pub terms: Vec<Term>,
    pub tradition: bool,
    pub experience_weeks: f64,
}

impl ArmyView {
    /// Points short of the next step: up to the third, or above the fall back to the second.
    pub fn short_of_up(&self) -> u32 {
        self.up.saturating_sub(self.total)
    }
}

const KEYS: [&str; 6] = [
    "officer_education",
    "training",
    "radio",
    "motorization",
    "experience",
    "central_control",
];

pub fn read(
    world: &GameWorld,
    mil: &bf_sim::war::Military,
    politics: Option<&bf_sim::politics::Politics>,
    tree: Option<&bf_sim::research::TechTree>,
    vehicles: &[u16],
    country: usize,
    today: i64,
) -> ArmyView {
    let r = army_reading(world, mil, politics, tree, vehicles, country, today);
    let d = &mil.rules.doctrine;
    let f = |x: laplace_schelling::Fixed| x.to_f64_lossy();
    let i = &r.inputs;
    let p = &r.readiness;
    let rows = [
        (
            p.officer_education,
            d.officer_education,
            f(i.officer_education),
        ),
        (p.training, d.training, f(i.training)),
        (p.radio, d.radio, f(i.radio)),
        (p.motorization, d.motorization, f(i.motorization)),
        (p.experience, d.experience, f(i.experience)),
        (p.central_control, d.central_control, f(i.central_control)),
    ];
    let terms = KEYS
        .iter()
        .zip(rows)
        .zip(r.holds.iter())
        .map(|((key, (points, most, input)), hold)| Term {
            key,
            points,
            most,
            input,
            hold: hold.as_ref().map(|h| match h {
                Hold::SchoolWithoutDoctrine(s) => HoldView::SchoolWithoutDoctrine(s.key()),
                Hold::NoTradition => HoldView::NoTradition,
                Hold::Training => HoldView::Training,
                Hold::NoNewerEquipment => HoldView::NoNewerEquipment,
                Hold::NoMotorized => HoldView::NoMotorized,
                Hold::NoExperience => HoldView::NoExperience,
                Hold::RegimeChange { years_left } => HoldView::RegimeChange {
                    years_left: *years_left,
                },
            }),
        })
        .collect();
    ArmyView {
        generation: match r.generation {
            Generation::Second => 2,
            Generation::Third => 3,
        },
        total: p.total,
        up: r.up,
        down: r.down,
        terms,
        tradition: r.tradition,
        experience_weeks: f(r.experience_weeks),
    }
}
