//! Headline indicators derived from the state, shown in the top bar.
//!
//! Phase 0 derives them from scenario data; from the economy phase on, the same formulas read
//! the simulated pops instead.

/// Quality of life on a 0–100 scale, in the spirit of the Human Development Index: the geometric
/// mean of an income, a health and an education index.
///
/// - Income: log of GDP per capita (2011 international dollars) between $300 and $40,000, the
///   range of the century from the poorest 1925 economies to rich 2025 ones.
/// - Health: life expectancy at birth between 20 and 85 years.
/// - Education: adult literacy.
///
/// A geometric mean means one very weak dimension cannot be hidden by the other two.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QualityOfLife {
    pub income: f64,
    pub health: f64,
    pub education: f64,
    /// 0–100.
    pub index: f64,
}

pub const INCOME_FLOOR: f64 = 300.0;
pub const INCOME_CEILING: f64 = 40_000.0;
pub const LIFE_FLOOR: f64 = 20.0;
pub const LIFE_CEILING: f64 = 85.0;

pub fn quality_of_life(gdp_per_capita: f64, life_expectancy: f64, literacy: f64) -> QualityOfLife {
    let clamp = |v: f64| v.clamp(0.001, 1.0);
    let income = clamp(
        (gdp_per_capita.max(1.0).ln() - INCOME_FLOOR.ln())
            / (INCOME_CEILING.ln() - INCOME_FLOOR.ln()),
    );
    let health = clamp((life_expectancy - LIFE_FLOOR) / (LIFE_CEILING - LIFE_FLOOR));
    let education = clamp(literacy);
    let index = 100.0 * (income * health * education).cbrt();
    QualityOfLife {
        income,
        health,
        education,
        index,
    }
}

#[cfg(test)]
mod tests {
    use super::quality_of_life;

    #[test]
    fn rich_literate_long_lived_scores_high() {
        let q = quality_of_life(40_000.0, 85.0, 1.0);
        assert!((q.index - 100.0).abs() < 1e-9);
    }

    #[test]
    fn one_weak_dimension_pulls_the_index_down() {
        let balanced = quality_of_life(3_000.0, 50.0, 0.5).index;
        let unbalanced = quality_of_life(10_000.0, 60.0, 0.05).index;
        assert!(unbalanced < balanced, "{unbalanced} vs {balanced}");
    }

    #[test]
    fn plausible_1925_values() {
        // Roughly Brazil and the United Kingdom in 1925.
        let bra = quality_of_life(1_300.0, 33.0, 0.30).index;
        let gbr = quality_of_life(8_200.0, 58.0, 0.95).index;
        assert!((20.0..45.0).contains(&bra), "{bra}");
        assert!((60.0..80.0).contains(&gbr), "{gbr}");
    }
}
