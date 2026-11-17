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

/// Interest takes this share of revenue before purchases are cut, point for point.
const INTEREST_COMFORT: Fixed = milli(100);

/// Share of the planned purchases the state makes, given its balance (cash it could not
/// borrow) and the share of revenue its interest takes.
pub fn austerity(m: &Market) -> Fixed {
    if m.budget.revenue_avg <= Fixed::ZERO {
        return Fixed::ONE;
    }
    let depth = if m.treasury < Fixed::ZERO {
        -m.treasury / (m.budget.revenue_avg * 26)
    } else {
        Fixed::ZERO
    };
    let burden = (super::money::interest_share(m) - INTEREST_COMFORT).max(Fixed::ZERO);
    (Fixed::ONE - depth - burden).max(AUSTERITY_FLOOR)
}

/// Share of its pay and of its part of schools and hospitals the state can meet this week.
/// When cash falls below [`PAY_RESERVE_WEEKS`] of revenue it pays less, a quarter less for each
/// week of revenue short, down to [`PAY_FLOOR`]: servants paid in arrears, fees for what
/// the treasury no longer covers. Steep enough that the cash settles above zero, interest
/// included.
pub fn pay_share(m: &Market) -> Fixed {
    let reserve = m.budget.revenue_avg * PAY_RESERVE_WEEKS;
    if m.treasury >= reserve || m.budget.revenue_avg <= Fixed::ZERO {
        return Fixed::ONE;
    }
    let depth = (reserve - m.treasury) / (m.budget.revenue_avg * 4);
    (Fixed::ONE - depth).max(PAY_FLOOR)
}

/// Least share of its pay a treasury out of cash still meets.
///
/// Stopgap: 10 % lets a state whose servants and soldiers cost more than any tax its law allows
/// (China in 1925: payroll 30 % of GDP, revenue under 7 %) keep its cash above zero. The cause in
/// history was different: a party state lived off its own firms and forced grain purchases.
/// State firms now pay the treasury (`LawEffects::state_ownership`), but nothing yet makes a
/// revolutionary regime nationalize (phase 7, regime agendas); until then this floor holds.
pub const PAY_FLOOR: Fixed = milli(100);
/// Weeks of revenue under which the treasury starts paying in arrears.
pub const PAY_RESERVE_WEEKS: i64 = 8;

/// Money the state means to spend on goods this week. The budget rule: revenue plus the
/// deficit the debt rule allows (`money::deficit_room`), minus payroll, public services,
/// public works and interest; at least a share of its payroll, cut when the treasury could
/// not borrow or interest weighs. On top, a tenth of whatever the treasury holds beyond twice
/// its reserve.
pub fn goods_budget(m: &Market) -> Fixed {
    let b = &m.budget;
    let surplus = (m.treasury - b.revenue_avg * (2 * RESERVE_WEEKS)).max(Fixed::ZERO);
    let planned = b.revenue_avg + super::money::deficit_room(m)
        - b.payroll
        - b.services
        - b.works
        - b.interest;
    (b.payroll * GOODS_PER_PAYROLL).max(planned) * austerity(m) + surplus * SURPLUS_SPENDING
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
            construction: cat.construction,
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

/// Construction points a level of `kind` costs (`monthly::level_capital`), in points at the
/// base price of construction.
pub fn level_points(cat: &Catalogue, kind: usize, construction: u16, tfp: Fixed) -> Fixed {
    super::monthly::level_capital(cat, kind, 0, tfp) / cat.goods[construction as usize].base_price
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
    // Cash above the reserve, plus what the state may borrow for works while debt and rates
    // are low.
    let budget = m.treasury - reserve - committed + super::money::borrowing_room(m);
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
    let national = unlocks.tech.map_or_else(
        || vec![false; cat.local.len()],
        |(tree, research)| {
            super::local::national(cat, Some(tree), research, econ.markets.len()).swap_remove(mi)
        },
    );
    for &kind in &kinds.infrastructure {
        let b = &cat.buildings[kind];
        let Some((good, _)) = b.methods[0].outputs.first() else {
            continue;
        };
        let g = *good as usize;
        // A good that clears state by state is built where the state lacks it.
        let local = super::local::local_output(cat, kind).filter(|&li| !national[li]);
        if local.is_none() && m.price[g] < cat.goods[g].base_price * SCARCE_PRICE {
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
            let score = match local {
                Some(li) => {
                    let Some(l) = r.local.get(li) else { continue };
                    let unmet = l.demand * (Fixed::ONE - l.buy);
                    if unmet <= Fixed::ZERO {
                        continue;
                    }
                    unmet * cat.goods[g].base_price / (queued(ri, kind) + 1)
                }
                None => Fixed::from_int(r.population() / 1000) / (levels + 1),
            };
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
    let colonial = colonial_purse(econ, m, regions);
    let mut out = Vec::new();
    for c in 0..PUBLIC_CATEGORIES {
        if weights[c] == 0 {
            continue;
        }
        let mut money = budget.mul_ratio(weights[c], total);
        // Public services in the colonies (schools, hospitals: any category whose buildings
        // make a public good) come out of the colony's own revenue, less what the colonial law
        // sends home and what serves the colonizers; the metropole's revenue stays at home
        // (`docs/state.pt-BR.md`, "Orçamento colonial").
        let services = cands[c].iter().any(|x| public_service(cat, x.2));
        let mut colony_money = if services {
            let m_col = money * colonial.share;
            money -= m_col;
            m_col * colonial.natives
        } else {
            Fixed::ZERO
        };
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
            let r = &econ.regions[ri];
            let tfp = super::plant_tfp(r, &econ.markets[r.market as usize], &cat.buildings[kind]);
            let points = level_points(cat, kind, cg, tfp);
            let cost = points * price;
            let purse = if services && r.colony {
                &mut colony_money
            } else {
                &mut money
            };
            if points <= Fixed::ZERO {
                break;
            }
            if cost > *purse {
                if !services {
                    break;
                }
                // This purse is spent: its regions wait, the other purse goes on.
                let colony = r.colony;
                for x in list.iter_mut() {
                    if econ.regions[x.1].colony == colony {
                        x.0 = Fixed::ZERO;
                    }
                }
                continue;
            }
            *purse -= cost;
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

/// Whether `kind` is a public service: its first output is a public good (`Catalogue::
/// public_goods`), which the state buys and pays by the laws. Its works in the colonies come out
/// of the colonial purse.
pub fn public_service(cat: &Catalogue, kind: usize) -> bool {
    cat.buildings[kind].methods.first().is_some_and(|m| {
        m.outputs
            .first()
            .is_some_and(|(g, _)| cat.public_goods.contains(g))
    })
}

/// How a country's public works money divides between metropole and colonies.
pub struct ColonialPurse {
    /// Colonies' share of the income tax base (incomes × class rates × collection).
    pub share: Fixed,
    /// Of the colonies' part, the share left for the natives' schools and hospitals: what the
    /// colonial law neither sends home (`colonial_tribute`) nor spends on the colonizers
    /// (`discrimination`).
    pub natives: Fixed,
}

/// The colonial purse of market `m` over its `regions`. Zero share without colonies.
pub fn colonial_purse(econ: &Economy, m: &Market, regions: &[usize]) -> ColonialPurse {
    let (mut all, mut col) = (Fixed::ZERO, Fixed::ZERO);
    for &ri in regions {
        let r = &econ.regions[ri];
        let collection = m.laws.of(r).tax_collection;
        let base: Fixed = r
            .pops
            .iter()
            .enumerate()
            .map(|(p, pop)| pop.income * m.budget.tax_rates[CLASS_OF[p]])
            .sum::<Fixed>()
            * collection;
        all += base;
        if r.colony {
            col += base;
        }
    }
    let share = if all > Fixed::ZERO {
        fill(col, all)
    } else {
        Fixed::ZERO
    };
    let law = &m.laws.effects[1];
    let natives = ((Fixed::ONE - law.colonial_tribute) * (Fixed::ONE - law.discrimination))
        .clamp(Fixed::ZERO, Fixed::ONE);
    ColonialPurse { share, natives }
}

/// Bounds of the fiscal correction.
const ADJUST_FLOOR: Fixed = milli(600);
pub const ADJUST_CEILING: Fixed = milli(2200);

/// The tax rule for the coming year: the historical rates times the fiscal correction. It
/// answers to the cost of the debt: a step up when interest takes more than 12 % of revenue,
/// two when it takes more than a quarter or the treasury could not borrow (cash below zero),
/// and a step down with little debt, cheap interest and cash half again above the reserve.
pub fn yearly_taxes(m: &mut Market, year: i32) {
    let interest = super::money::interest_share(m);
    let debt_ratio = if m.gdp_last_year > Fixed::ZERO {
        m.budget.debt / m.gdp_last_year
    } else {
        Fixed::ZERO
    };
    let b = &mut m.budget;
    let yearly = b.revenue_avg * 52;
    if yearly > Fixed::ZERO {
        let step = if interest > milli(250) || m.treasury < Fixed::ZERO {
            milli(100)
        } else if interest > milli(120) {
            milli(50)
        } else if interest < milli(40)
            && debt_ratio < milli(400)
            && m.treasury > yearly.mul_ratio(3, 8)
        {
            -milli(50)
        } else {
            Fixed::ZERO
        };
        b.adjust = (b.adjust + step).clamp(ADJUST_FLOOR, ADJUST_CEILING);
    }
    if b.tax_auto {
        let base = historical_rates(year);
        for c in 0..TAX_CLASSES {
            b.tax_rates[c] = (base[c] * b.adjust).min(MAX_TAX);
        }
    }
    cap_taxes(m);
}

/// Holds the class rates under the ceilings the taxation law sets.
///
/// What the ceilings cut is not lost: states held to low income taxes lived off duties and
/// excises. The mean cut across the three classes becomes a tax on consumption
/// (`Budget::indirect_rate`), at most [`INDIRECT_CEILING`]; it falls on what the pops buy, so
/// those who save pay less.
pub fn cap_taxes(m: &mut Market) {
    let caps = m.laws.effects[0].tax_caps;
    let mut cut = Fixed::ZERO;
    for (r, cap) in m.budget.tax_rates.iter_mut().zip(caps) {
        let capped = (*r).min(cap);
        cut += *r - capped;
        *r = capped;
    }
    m.budget.indirect_rate = (cut / TAX_CLASSES as i64).min(INDIRECT_CEILING);
}

/// Highest tax on consumption: the highest rate the historical rule ever asks of the lower
/// class (1975), which pays mostly through what it buys. Derived from the scenario; tariffs on
/// trade come with borders (phase 4).
pub const INDIRECT_CEILING: Fixed = milli(RATES_1975[0]);

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
    fn the_cost_of_debt_raises_the_rule_and_a_surplus_lowers_it() {
        let mut m = Market::empty(0);
        m.budget.revenue_avg = Fixed::from_int(100);
        m.budget.interest = Fixed::from_int(15);
        yearly_taxes(&mut m, 1950);
        assert_eq!(m.budget.adjust, milli(1050));
        assert!(m.budget.tax_rates[0] > historical_rates(1950)[0]);
        // Cash it could not borrow counts twice.
        m.budget.interest = Fixed::ZERO;
        m.treasury = Fixed::from_int(-10);
        yearly_taxes(&mut m, 1950);
        assert_eq!(m.budget.adjust, milli(1150));
        m.treasury = Fixed::from_int(5000);
        yearly_taxes(&mut m, 1950);
        yearly_taxes(&mut m, 1950);
        yearly_taxes(&mut m, 1950);
        assert_eq!(m.budget.adjust, milli(1000));
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
        // Interest above a tenth of revenue cuts purchases too.
        m.treasury = Fixed::ZERO;
        m.budget.interest = Fixed::from_int(30);
        assert_eq!(goods_budget(&m), Fixed::from_int(48));
    }

    #[test]
    fn pay_falls_below_the_reserve_and_stops_at_the_floor() {
        let mut m = Market::empty(0);
        m.budget.revenue_avg = Fixed::from_int(100);
        m.treasury = Fixed::from_int(800);
        assert_eq!(pay_share(&m), Fixed::ONE);
        // Two weeks of revenue short of the reserve: half pay.
        m.treasury = Fixed::from_int(600);
        assert_eq!(pay_share(&m), milli(500));
        m.treasury = Fixed::from_int(-10_000);
        assert_eq!(pay_share(&m), PAY_FLOOR);
    }

    fn region(colony: bool, income: i64) -> Region {
        let mut pops = super::super::state::empty_pops();
        pops[prof::PEASANT].income = Fixed::from_int(income);
        Region {
            market: 0,
            colony,
            pops,
            plants: Vec::new(),
            literacy: Fixed::ZERO,
            deposits: Vec::new(),
            land: Vec::new(),
            frontier: Vec::new(),
            wild_km2: [0; 2],
            cleared_km2: [0; 2],
            pioneer_km2: Fixed::ZERO,
            subsistence: Fixed::ZERO,
            tfp: Fixed::ONE,
            tfp_start: Fixed::ONE,
            state: 0,
            neighbours: Vec::new(),
            local: Vec::new(),
        }
    }

    #[test]
    fn colonies_pay_for_their_own_schools_less_what_the_law_takes() {
        let mut m = Market::empty(0);
        m.budget.tax_rates = [milli(100); TAX_CLASSES];
        let econ = Economy {
            regions: vec![region(false, 300), region(true, 100)],
            markets: Vec::new(),
            world_price: Vec::new(),
            history: Vec::new(),
        };
        // No colonial law: a quarter of the tax base, all of it for the natives.
        let p = colonial_purse(&econ, &m, &[0, 1]);
        assert_eq!(p.share, milli(250));
        assert_eq!(p.natives, Fixed::ONE);
        // Indirect rule: a fifth sent home, 30 % for the colonizers.
        m.laws.effects[1].colonial_tribute = milli(200);
        m.laws.effects[1].discrimination = milli(300);
        let p = colonial_purse(&econ, &m, &[0, 1]);
        assert_eq!(p.natives, milli(560));
        // Without colonies, nothing.
        assert_eq!(colonial_purse(&econ, &m, &[0]).share, Fixed::ZERO);
    }
}
