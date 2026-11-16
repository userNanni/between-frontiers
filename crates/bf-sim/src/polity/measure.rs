//! What the movements' causes measure: [`crate::politics::INDICATORS`] read from the pops, the
//! regions and the market of a country, for the metropole or for its colonial regions.

use laplace_schelling::Fixed;

use crate::econ::catalogue::Catalogue;
use crate::econ::migration::is_urban;
use crate::econ::prof::{self, P};
use crate::econ::state::Region;
use crate::econ::{public, workers};

/// Indicator indices, resolved at compile time against `politics::INDICATORS` by the test
/// below.
pub const LITERACY: usize = 0;
pub const URBANIZATION: usize = 1;
pub const UNEMPLOYMENT: usize = 2;
pub const NEEDS_MET: usize = 3;
pub const SCHOOL_COVERAGE: usize = 4;
pub const HOSPITAL_COVERAGE: usize = 5;
pub const RURAL_WAGE_GAP: usize = 6;
pub const TAX_BURDEN: usize = 7;
pub const IMPORT_SHARE: usize = 8;
pub const LANDOWNER_INCOME_SHARE: usize = 9;
/// Read from the politics state, not from the economy.
pub const LEGITIMACY: usize = 14;
pub const FISCAL_STRESS: usize = 16;
pub const GDP_GROWTH: usize = 17;
pub const IMMIGRANT_SHARE: usize = 13;

/// Share of the people of `country` of foreign origin, per mille: the one place politics reads
/// it. `None` until the demography keeps the stock of migrants by origin (#54); then it is
/// `world.demography.immigrant_share(&world.econ, country)` (a `Fixed` 0–1, metropole and
/// colonies, read as of the last 15th).
pub fn immigrant_share(_world: &crate::game::GameWorld, _country: usize) -> Option<i32> {
    None
}

fn one() -> i128 {
    i128::from(Fixed::ONE.raw())
}

/// Sums over the people of one country, metropole or colonies.
#[derive(Clone, Debug, Default)]
pub struct Facts {
    pub size: [i64; P],
    pub employed: [i64; P],
    /// Raw [`Fixed`] sums: weekly income, size × (tiers 1 and 2 met) / 2, size × literacy.
    pub income: [i128; P],
    pub needs: [i128; P],
    pub literacy: [i128; P],
    /// Raw sums of population × coverage of schools and of hospitals.
    pub school: i128,
    pub hospital: i128,
    /// Per mille, of the market: taxes over GDP and imports over the supply of manufactures.
    pub tax_burden: i32,
    pub import_share: i32,
    pub fiscal_stress: i32,
    pub gdp_growth: i32,
    /// `None` while the simulation does not measure it ([`immigrant_share`]).
    pub immigrant_share: Option<i32>,
}

impl Facts {
    pub fn population(&self) -> i64 {
        self.size.iter().sum()
    }

    fn add_region(&mut self, cat: &Catalogue, kinds: (Option<usize>, Option<usize>), r: &Region) {
        let lit = i128::from(r.literacy.raw());
        for (p, pop) in r.pops.iter().enumerate().take(P) {
            let n = pop.size.max(0);
            self.size[p] += n;
            self.employed[p] += pop.employed.max(0);
            self.income[p] += i128::from(pop.income.raw().max(0));
            let met = i128::from((pop.satisfaction[0] + pop.satisfaction[1]).raw()) / 2;
            self.needs[p] += met * i128::from(n);
            self.literacy[p] += lit * i128::from(n);
        }
        let n = i128::from(r.population());
        self.school +=
            i128::from(public::coverage(cat, r, kinds.0, public::PUPILS_PER_PERSON).raw()) * n;
        self.hospital +=
            i128::from(public::coverage(cat, r, kinds.1, public::CARE_PER_PERSON).raw()) * n;
    }

    /// Per mille share of `num` over `den`, `None` with nobody to measure.
    fn ratio(num: i128, den: i128) -> Option<i32> {
        (den > 0).then(|| (num * 1000 / den).clamp(0, 1_000_000) as i32)
    }

    /// Value of indicator `i` among the people `row` recruits (per mille of each profession),
    /// or `None` when the simulation does not measure it yet.
    pub fn indicator(&self, i: usize, row: &[u16; P]) -> Option<i32> {
        let w = |p: usize| i128::from(row[p]);
        let weighted = |v: &[i128; P]| -> Option<i32> {
            let den: i128 = (0..P).map(|p| w(p) * i128::from(self.size[p])).sum();
            let num: i128 = (0..P).map(|p| w(p) * v[p]).sum();
            Self::ratio(num, den * one())
        };
        let total = i128::from(self.population());
        let v = match i {
            LITERACY => weighted(&self.literacy)?,
            NEEDS_MET => weighted(&self.needs)?,
            UNEMPLOYMENT => {
                let (mut idle, mut force) = (0i128, 0i128);
                for p in 0..P {
                    let f = i128::from(workers(self.size[p]));
                    force += w(p) * f;
                    idle += w(p) * (f - i128::from(self.employed[p])).max(0);
                }
                Self::ratio(idle, force)?
            }
            URBANIZATION => Self::ratio(
                (0..P)
                    .filter(|&p| is_urban(p))
                    .map(|p| i128::from(self.size[p]))
                    .sum(),
                total,
            )?,
            SCHOOL_COVERAGE => Self::ratio(self.school, total * one())?,
            HOSPITAL_COVERAGE => Self::ratio(self.hospital, total * one())?,
            RURAL_WAGE_GAP => {
                let rural = [prof::PEASANT, prof::FARM_LABORER];
                let per_head = |town: bool| -> Option<i128> {
                    let ps =
                        (0..P).filter(|&p| is_urban(p) == town && (town || rural.contains(&p)));
                    let (inc, n) = ps.fold((0i128, 0i128), |(a, b), p| {
                        (a + self.income[p], b + i128::from(self.size[p]))
                    });
                    (n > 0 && inc > 0).then(|| inc * 1_000_000 / n)
                };
                Self::ratio(per_head(true)?, per_head(false)?)?.min(10_000)
            }
            TAX_BURDEN => self.tax_burden,
            IMPORT_SHARE => self.import_share,
            FISCAL_STRESS => self.fiscal_stress,
            GDP_GROWTH => self.gdp_growth,
            IMMIGRANT_SHARE => self.immigrant_share?,
            LANDOWNER_INCOME_SHARE => Self::ratio(
                self.income[prof::LANDOWNER],
                self.income[prof::LANDOWNER]
                    + self.income[prof::PEASANT]
                    + self.income[prof::FARM_LABORER],
            )?,
            _ => return None,
        };
        Some(v.clamp(0, crate::politics::INDICATORS[i].max))
    }
}

/// Facts of every country, `[metropole, colonies]`, in market order.
pub fn facts(
    cat: &Catalogue,
    kinds: (Option<usize>, Option<usize>),
    manufactures: &[usize],
    world: &crate::game::GameWorld,
) -> Vec<[Facts; 2]> {
    let (regions, markets) = (&world.econ.regions, &world.econ.markets);
    // GDP of the last two years on record, by market (the history is in year order).
    let mut last: Vec<[i128; 2]> = vec![[0; 2]; markets.len()];
    let mut found = vec![0u8; markets.len()];
    for rec in world.econ.history.iter().rev() {
        let mi = usize::from(rec.market);
        if mi < markets.len() && found[mi] < 2 {
            last[mi][usize::from(found[mi])] = i128::from(rec.gdp.raw());
            found[mi] += 1;
        }
        if found.iter().all(|&n| n == 2) {
            break;
        }
    }
    let mut out: Vec<[Facts; 2]> = vec![Default::default(); markets.len()];
    for r in regions {
        if let Some(f) = out.get_mut(usize::from(r.market)) {
            f[usize::from(r.colony)].add_region(cat, kinds, r);
        }
    }
    for (out_index, (f, m)) in out.iter_mut().zip(markets).enumerate() {
        let gdp = i128::from(m.gdp_last_year.raw());
        let taxes = i128::from(m.budget.revenue_avg.raw()) * 52;
        let tax_burden = Facts::ratio(taxes, gdp).unwrap_or(0).min(1000);
        let (mut imports, mut supply) = (0i128, 0i128);
        for &g in manufactures {
            let price = i128::from(m.price[g].raw());
            imports += i128::from(m.net_import[g].raw().max(0)) * price;
            supply += i128::from(m.supply[g].raw().max(0)) * price;
        }
        let import_share = Facts::ratio(imports, supply).unwrap_or(0).min(1000);
        let austerity = i128::from(public::austerity(m).raw());
        let fiscal_stress = (1000 - austerity * 1000 / one()).clamp(0, 1000) as i32;
        let mi = out_index;
        let gdp_growth = if found[mi] == 2 && last[mi][1] > 0 {
            ((last[mi][0] - last[mi][1]) * 1000 / last[mi][1]).clamp(0, 1000) as i32
        } else {
            0
        };
        let immigrants = immigrant_share(world, mi);
        for x in f.iter_mut() {
            x.tax_burden = tax_burden;
            x.import_share = import_share;
            x.fiscal_stress = fiscal_stress;
            x.gdp_growth = gdp_growth;
            x.immigrant_share = immigrants;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::politics::indicator_index;

    #[test]
    fn indices_match_the_catalogue() {
        for (i, key) in [
            (LITERACY, "literacy"),
            (URBANIZATION, "urbanization"),
            (UNEMPLOYMENT, "unemployment"),
            (NEEDS_MET, "needs_met"),
            (SCHOOL_COVERAGE, "school_coverage"),
            (HOSPITAL_COVERAGE, "hospital_coverage"),
            (RURAL_WAGE_GAP, "rural_wage_gap"),
            (TAX_BURDEN, "tax_burden"),
            (IMPORT_SHARE, "import_share"),
            (LANDOWNER_INCOME_SHARE, "landowner_income_share"),
            (LEGITIMACY, "legitimacy"),
            (FISCAL_STRESS, "fiscal_stress"),
            (GDP_GROWTH, "gdp_growth"),
            (IMMIGRANT_SHARE, "immigrant_share"),
        ] {
            assert_eq!(indicator_index(key), Some(i), "{key}");
        }
    }
}
