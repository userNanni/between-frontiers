//! The state's offices with computers (`game::Switches::computer_efficiency` with
//! `fiscal_capacity`; docs/research.pt-BR.md, "Computação como método"): each civil servant
//! collects as much as `factor` servants did by hand, the factor of the newest computing
//! technology the country has at the industrial stage (`data/economy/administration.ron`), as far
//! as the state found the computers it ordered. The same payroll collects more: efficiency, not
//! capacity. With either switch off nothing here runs.

use laplace_schelling::Fixed;

use super::catalogue::GoodIx;
use crate::research::{ResearchState, Stage};

/// What the rule reads (`data/economy/administration.ron`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdminRules {
    /// The good the offices buy (`computers`).
    pub good: GoodIx,
    /// By technology, oldest first: (technology index, servant's factor, units of the good a
    /// week for a thousand servants).
    pub steps: Vec<(u16, Fixed, Fixed)>,
}

/// The country's step: the factor and the computers a thousand servants use a week, of the newest
/// technology at the industrial stage there; `None` before the first.
pub fn step(
    rules: &AdminRules,
    research: &ResearchState,
    country: usize,
) -> Option<(Fixed, Fixed)> {
    let stages = research.stage.get(country)?;
    rules
        .steps
        .iter()
        .rev()
        .find(|(t, _, _)| {
            stages
                .get(usize::from(*t))
                .is_some_and(|s| *s >= Stage::Industrial)
        })
        .map(|&(_, f, q)| (f, q))
}

/// A servant's worth this week: one plus the step's gain, scaled by the share of the computers
/// the state found last week.
pub fn efficiency(factor: Fixed, found: Fixed) -> Fixed {
    Fixed::ONE + (factor - Fixed::ONE).max(Fixed::ZERO) * found.clamp(Fixed::ZERO, Fixed::ONE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_industrial_step_counts_as_far_as_the_computers_were_found() {
        let rules = AdminRules {
            good: 0,
            steps: vec![
                (0, Fixed::from_ratio(13, 10), Fixed::from_int(10)),
                (1, Fixed::from_ratio(16, 10), Fixed::from_int(60)),
            ],
        };
        let mut research = ResearchState::default();
        research.stage = vec![vec![Stage::Prototype, Stage::Unknown]];
        assert_eq!(step(&rules, &research, 0), None);
        research.stage[0][0] = Stage::Industrial;
        assert_eq!(
            step(&rules, &research, 0),
            Some((Fixed::from_ratio(13, 10), Fixed::from_int(10)))
        );
        research.stage[0][1] = Stage::Diffused;
        assert_eq!(
            step(&rules, &research, 0).unwrap().0,
            Fixed::from_ratio(16, 10)
        );
        assert_eq!(
            efficiency(Fixed::from_ratio(16, 10), Fixed::ONE / 2),
            Fixed::from_ratio(13, 10)
        );
        assert_eq!(
            efficiency(Fixed::from_ratio(16, 10), Fixed::ZERO),
            Fixed::ONE
        );
    }
}
