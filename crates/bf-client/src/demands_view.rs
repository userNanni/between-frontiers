//! What the demand screen reads: the player's pending independence demands (`bf_sim::borders::
//! independence::demands_of`) and the price of each answer (`answer_effects`), computed by the
//! simulation once a month.

use bf_sim::borders::independence::{Answer, DemandView, Effects, IndependenceSetup};
use bf_sim::game::GameWorld;
use laplace_schelling::calendar::Date;

/// The answers the player can give, in the order the screen offers them.
pub const ANSWERS: [Answer; 4] = [
    Answer::SelfRule,
    Answer::Dominion,
    Answer::Independence,
    Answer::Refused,
];

#[derive(Clone, Debug, PartialEq)]
pub struct Demand {
    pub view: DemandView,
    /// The price of each of [`ANSWERS`], in that order.
    pub effects: Vec<Option<Effects>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DemandsView {
    pub month: (i32, u8),
    pub tag: String,
    pub demands: Vec<Demand>,
    /// Every country's tag in the simulation's order, for the payers' names.
    pub tags: Vec<String>,
}

/// The demands pending on country `tag`; `None` without a player or the independence rules.
pub fn read(
    world: &GameWorld,
    setup: Option<&IndependenceSetup>,
    tag: Option<&str>,
    date: Date,
) -> Option<DemandsView> {
    let setup = setup?;
    let tag = tag?;
    let owner = world.countries.iter().position(|(_, c)| c.tag == tag)? as u16;
    let demands = bf_sim::borders::independence::demands_of(setup, world, owner)
        .into_iter()
        // An answer given and not yet acted on takes the demand off the list.
        .filter(|v| !world.independence.answers.iter().any(|(u, _)| *u == v.unit))
        .map(|view| {
            let effects = ANSWERS
                .iter()
                .map(|a| {
                    bf_sim::borders::independence::answer_effects(setup, world, &view.unit, *a)
                })
                .collect();
            Demand { view, effects }
        })
        .collect();
    Some(DemandsView {
        month: (date.year, date.month),
        tag: tag.to_owned(),
        demands,
        tags: world.countries.iter().map(|(_, c)| c.tag.clone()).collect(),
    })
}

pub fn answer_key(a: Answer) -> &'static str {
    match a {
        Answer::Pending => "independence-answer-pending",
        Answer::SelfRule => "independence-answer-self_rule",
        Answer::Dominion => "independence-answer-dominion",
        Answer::Independence => "independence-answer-independence",
        Answer::Refused => "independence-answer-refused",
    }
}
