//! The state's budget: taxes by class, payroll, purchases, public services and public works.
//!
//! See `docs/state.pt-BR.md` for the rules and their calibration.

use laplace_schelling::Fixed;
use rayon::prelude::*;

use super::catalogue::Catalogue;
use super::prof::{self, P};
use super::state::{Economy, Market, PUBLIC_CATEGORIES, Project, Region, TAX_CLASSES};
use super::{fill, milli};

/// Tax class of each profession (`TAX_CLASSES`): 0 lower, 1 middle, 2 upper.
pub const CLASS_OF: [usize; P] = {
    let mut c = [0; P];
    c[prof::SKILLED] = 1;
    c[prof::CLERK] = 1;
    c[prof::ENGINEER] = 1;
    c[prof::PROFESSIONAL] = 1;
    c[prof::CIVIL_SERVANT] = 1;
    c[prof::MANAGER] = 2;
    c[prof::CAPITALIST] = 2;
    c[prof::LANDOWNER] = 2;
    c
};

/// Public investment categories, in the order of `Budget::priority`.
pub const INFRASTRUCTURE: usize = 0;
pub const EDUCATION: usize = 1;
pub const HEALTH: usize = 2;
pub const HOUSING: usize = 3;
pub const DEFAULT_PRIORITY: [u8; PUBLIC_CATEGORIES] = [3, 2, 2, 3];
/// Highest weight a category takes.
pub const MAX_PRIORITY: u8 = 10;

/// Income tax by class, per mille: what states took in 1925 and in 1975, when the welfare
/// state was built. The rule moves linearly between them and stays after 1975.
const RATES_1925: [i64; TAX_CLASSES] = [50, 90, 150];
const RATES_1975: [i64; TAX_CLASSES] = [120, 220, 350];
/// Highest rate a class pays, by rule or by command.
pub const MAX_TAX: Fixed = milli(600);

pub fn historical_rates(year: i32) -> [Fixed; TAX_CLASSES] {
    let t = i64::from((year - 1925).clamp(0, 50));
    let mut out = [Fixed::ZERO; TAX_CLASSES];
    for c in 0..TAX_CLASSES {
        out[c] = milli(RATES_1925[c] + (RATES_1975[c] - RATES_1925[c]) * t / 50);
    }
    out
}

/// Public pay of a servant, soldier or priest: the average private wage times the profession's
/// wage weight, times this.
pub const PUBLIC_PAY: Fixed = milli(700);
/// The state buys goods (arms, paper, transport) worth this share of its payroll.
pub const GOODS_PER_PAYROLL: Fixed = milli(600);
/// Share of the surplus beyond twice the reserve spent each week.
const SURPLUS_SPENDING: Fixed = milli(100);
/// Weeks of revenue the treasury keeps before it starts public works.
pub const RESERVE_WEEKS: i64 = 13;
/// Revenue smoothing: a thirteenth of the gap each week.
pub const REVENUE_SMOOTHING: i64 = 13;
/// With the treasury in the red, purchases fall by the deficit's size in half-years of
/// revenue, down to this share.
const AUSTERITY_FLOOR: Fixed = milli(250);

/// Pupils a population asks for (school-age share) and hospital care per person per year.
pub const PUPILS_PER_PERSON: Fixed = milli(200);
pub const CARE_PER_PERSON: Fixed = milli(200);

/// Share of the planned purchases the state makes, given its balance.
pub fn austerity(m: &Market) -> Fixed {
    if m.treasury >= Fixed::ZERO || m.budget.revenue_avg <= Fixed::ZERO {
        return Fixed::ONE;
    }
    let depth = -m.treasury / (m.budget.revenue_avg * 26);
    (Fixed::ONE - depth).max(AUSTERITY_FLOOR)
}

/// Money the state means to spend on goods this week: a share of its payroll, plus a tenth of
/// whatever the treasury holds beyond twice its reserve (money the stand-in for a central bank
/// issued and public works did not use).
pub fn goods_budget(m: &Market) -> Fixed {
    let surplus = (m.treasury - m.budget.revenue_avg * (2 * RESERVE_WEEKS)).max(Fixed::ZERO);
    m.budget.payroll * GOODS_PER_PAYROLL * austerity(m) + surplus * SURPLUS_SPENDING
}

/// Building indices the state works with.
pub struct PublicKinds {
    pub school: Option<usize>,
    pub hospital: Option<usize>,
    pub housing: Option<usize>,
    /// Infrastructure the state builds when its output is scarce.
    pub infrastructure: Vec<usize>,
    pub construction: Option<u16>,
}

impl PublicKinds {
    pub fn new(cat: &Catalogue) -> Self {
        Self {
            school: cat.building("school"),
            hospital: cat.building("hospital"),
            housing: cat.building("urban_housing"),
            infrastructure: ["railway", "road_transport", "power_plant"]
                .iter()
                .filter_map(|k| cat.building(k))
                .collect(),
            construction: cat.good("construction"),
        }
    }
}

/// Units a level of `kind` makes in a year here, at full staff (first output).
pub fn level_output(cat: &Catalogue, r: &Region, kind: usize, method: usize) -> Fixed {
    let b = &cat.buildings[kind];
    b.methods[method]
        .outputs
        .first()
        .map_or(Fixed::ZERO, |(_, q)| *q * r.tfp_for(b) * 52)
}

/// What the plants of `kind` here make in a year, as they are staffed.
pub fn yearly_output(cat: &Catalogue, r: &Region, kind: usize) -> Fixed {
    r.plants
        .iter()
        .filter(|p| p.kind as usize == kind)
        .map(|p| level_output(cat, r, kind, p.method as usize) * p.levels * p.staffing)
        .sum()
}

/// Share of the region's need a public service meets, 0–1.
pub fn coverage(cat: &Catalogue, r: &Region, kind: Option<usize>, per_person: Fixed) -> Fixed {
    let Some(kind) = kind else {
        return Fixed::ZERO;
    };
    fill(
        yearly_output(cat, r, kind),
        per_person * r.population().max(1),
    )
}

/// Construction points a level of `kind` costs: two years of the value it adds at base prices,
/// in points at the base price of construction.
pub fn level_points(cat: &Catalogue, kind: usize, construction: u16) -> Fixed {
    let va = super::monthly::base_value_added(cat, kind, 0) * (52 * super::monthly::CAPITAL_YEARS);
    va / cat.goods[construction as usize].base_price
}

/// Most public works a country starts in a month.
const MAX_PUBLIC_BUILDS: usize = 200;
/// Infrastructure is built when its output trades this far above its base price.
const SCARCE_PRICE: Fixed = milli(1100);

/// Public works each country starts this month, from the treasury above its reserve.
pub fn public_works(cat: &Catalogue, econ: &Economy, unlocks: &super::Unlocks<'_>) -> Vec<Project> {
    let kinds = PublicKinds::new(cat);
    let Some(cg) = kinds.construction else {
        return Vec::new();
    };
    let mut by_market: Vec<Vec<usize>> = vec![Vec::new(); econ.markets.len()];
    for (ri, r) in econ.regions.iter().enumerate() {
        by_market[r.market as usize].push(ri);
    }
    let lists: Vec<Vec<Project>> = econ
        .markets
        .par_iter()
        .enumerate()
        .map(|(mi, m)| market_works(cat, econ, &kinds, cg, mi, m, &by_market[mi], unlocks))
        .collect();
    lists.into_iter().flatten().collect()
}

/// A public work worth doing: score, region, kind, method, need one level meets.
type Candidate = (Fixed, usize, usize, u8, Fixed);

#[allow(clippy::too_many_arguments)]
fn market_works(
    cat: &Catalogue,
    econ: &Economy,
    kinds: &PublicKinds,
    cg: u16,
    mi: usize,
    m: &Market,
    regions: &[usize],
    unlocks: &super::Unlocks<'_>,
) -> Vec<Project> {
    let price = m.price[cg as usize];
    let committed: Fixed = m
        .queue
        .iter()
        .filter(|p| p.public)
        .map(|p| p.left * price)
        .sum();
    let reserve = m.budget.revenue_avg * RESERVE_WEEKS;
    let budget = m.treasury - reserve - committed;
    if budget <= Fixed::ZERO || regions.is_empty() {
        return Vec::new();
    }
    // Levels already queued, by region and kind.
    let queued = |ri: usize, kind: usize| -> i64 {
        m.queue
            .iter()
            .filter(|p| p.region as usize == ri && p.kind as usize == kind)
            .count() as i64
    };
    // Candidates per category: (score, region, kind, method, need met by one level).
    let mut cands: [Vec<Candidate>; PUBLIC_CATEGORIES] = std::array::from_fn(|_| Vec::new());
    for (cat_ix, kind, per_person) in [
        (EDUCATION, kinds.school, PUPILS_PER_PERSON),
        (HEALTH, kinds.hospital, CARE_PER_PERSON),
    ] {
        let Some(kind) = kind else { continue };
        for &ri in regions {
            let r = &econ.regions[ri];
            let per_level = level_output(cat, r, kind, 0);
            if per_level <= Fixed::ZERO {
                continue;
            }
            let have = yearly_output(cat, r, kind) + per_level * queued(ri, kind);
            let gap = per_person * r.population() - have;
            if gap > Fixed::ZERO {
                cands[cat_ix].push((gap, ri, kind, 0, per_level));
            }
        }
    }
    if let Some(kind) = kinds.housing {
        for &ri in regions {
            let r = &econ.regions[ri];
            let per_level = super::migration::housing_per_level(cat, r, kind);
            let gap = super::migration::housing_gap(cat, r, kind) - per_level * queued(ri, kind);
            if gap > Fixed::ZERO && per_level > Fixed::ZERO {
                cands[HOUSING].push((gap, ri, kind, 0, per_level));
            }
        }
    }
    for &kind in &kinds.infrastructure {
        let b = &cat.buildings[kind];
        let Some((good, _)) = b.methods[0].outputs.first() else {
            continue;
        };
        let g = *good as usize;
        if m.price[g] < cat.goods[g].base_price * SCARCE_PRICE {
            continue;
        }
        let Some(method) = (0..b.methods.len())
            .rev()
            .find(|&mi2| unlocks.method(cat, mi, kind, mi2))
        else {
            continue;
        };
        // Where people are and the building is scarce.
        for &ri in regions {
            let r = &econ.regions[ri];
            let levels = r
                .plants
                .iter()
                .filter(|p| p.kind as usize == kind)
                .map(|p| p.levels)
                .sum::<i64>()
                + queued(ri, kind);
            let score = Fixed::from_int(r.population() / 1000) / (levels + 1);
            cands[INFRASTRUCTURE].push((score, ri, kind, method as u8, Fixed::ZERO));
        }
    }
    let weights: Vec<i64> = (0..PUBLIC_CATEGORIES)
        .map(|c| {
            if cands[c].is_empty() {
                0
            } else {
                i64::from(m.budget.priority[c])
            }
        })
        .collect();
    let total: i64 = weights.iter().sum();
    if total == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for c in 0..PUBLIC_CATEGORIES {
        if weights[c] == 0 {
            continue;
        }
        let mut money = budget.mul_ratio(weights[c], total);
        let list = &mut cands[c];
        // Infrastructure: a few levels where the score is highest; services: fill the largest
        // gaps first, one level at a time.
        let cap = if c == INFRASTRUCTURE {
            (regions.len() / 4).max(1)
        } else {
            MAX_PUBLIC_BUILDS
        };
        let mut started = 0;
        while started < cap && out.len() < MAX_PUBLIC_BUILDS {
            // Highest score, ties by region then kind: thread-independent.
            let Some(best) = (0..list.len()).max_by(|&a, &b| {
                list[a]
                    .0
                    .cmp(&list[b].0)
                    .then(list[b].1.cmp(&list[a].1))
                    .then(list[b].2.cmp(&list[a].2))
            }) else {
                break;
            };
            let (score, ri, kind, method, per_level) = list[best];
            if score <= Fixed::ZERO {
                break;
            }
            let points = level_points(cat, kind, cg);
            let cost = points * price;
            if points <= Fixed::ZERO || cost > money {
                break;
            }
            money -= cost;
            out.push(Project {
                region: ri as u32,
                kind: kind as u16,
                method,
                public: true,
                points,
                left: points,
            });
            started += 1;
            if c == INFRASTRUCTURE {
                list.swap_remove(best);
            } else {
                list[best].0 = score - per_level;
            }
        }
    }
    out
}

/// The tax rule for the coming year: the historical rates times the fiscal correction, which
/// rises a step after a year deep in deficit and falls a step after a large surplus.
pub fn yearly_taxes(m: &mut Market, year: i32) {
    let b = &mut m.budget;
    let yearly = b.revenue_avg * 52;
    if yearly > Fixed::ZERO {
        if m.treasury < -yearly.mul_ratio(1, 4) {
            b.adjust = (b.adjust + milli(50)).min(milli(1600));
        } else if m.treasury > yearly.mul_ratio(1, 2) {
            b.adjust = (b.adjust - milli(50)).max(milli(600));
        }
    }
    if b.tax_auto {
        let base = historical_rates(year);
        for c in 0..TAX_CLASSES {
            b.tax_rates[c] = (base[c] * b.adjust).min(MAX_TAX);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_rise_until_1975_and_stay() {
        let r25 = historical_rates(1925);
        let r50 = historical_rates(1950);
        let r75 = historical_rates(1975);
        for c in 0..TAX_CLASSES {
            assert!(r25[c] < r50[c] && r50[c] < r75[c]);
            assert!(r25[c] <= r25[TAX_CLASSES - 1], "the rich pay the most");
        }
        assert_eq!(historical_rates(2025), r75);
        assert_eq!(historical_rates(1900), r25);
    }

    #[test]
    fn a_deficit_raises_the_rule_and_a_surplus_lowers_it() {
        let mut m = Market::empty(0);
        m.budget.revenue_avg = Fixed::from_int(100);
        m.treasury = Fixed::from_int(-2000);
        yearly_taxes(&mut m, 1950);
        assert_eq!(m.budget.adjust, milli(1050));
        assert!(m.budget.tax_rates[0] > historical_rates(1950)[0]);
        m.treasury = Fixed::from_int(5000);
        yearly_taxes(&mut m, 1950);
        yearly_taxes(&mut m, 1950);
        assert_eq!(m.budget.adjust, milli(950));
        // Rates the player set are kept.
        m.budget.tax_auto = false;
        m.budget.tax_rates = [milli(10); TAX_CLASSES];
        yearly_taxes(&mut m, 1960);
        assert_eq!(m.budget.tax_rates, [milli(10); TAX_CLASSES]);
    }

    #[test]
    fn purchases_shrink_in_deficit() {
        let mut m = Market::empty(0);
        m.budget.revenue_avg = Fixed::from_int(100);
        m.budget.payroll = Fixed::from_int(100);
        assert_eq!(goods_budget(&m), Fixed::from_int(60));
        m.treasury = Fixed::from_int(-1300);
        assert_eq!(goods_budget(&m), Fixed::from_int(30));
        m.treasury = Fixed::from_int(-100_000);
        assert_eq!(goods_budget(&m), Fixed::from_int(15));
    }
}
