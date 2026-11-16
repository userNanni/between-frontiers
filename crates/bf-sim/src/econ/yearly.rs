//! Yearly systems: the change of production methods, schooling, and the statistics.

use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::schedule::{Access, Cadence, System, TickCtx};
use rayon::prelude::*;

use super::catalogue::Catalogue;
use super::monthly::best_method;
use super::state::YearRecord;
use super::week::ECON;
use super::{Unlocks, fill, milli, plant_tfp};
use crate::game::GameWorld;

/// Share of a private plant's levels that wears out each year: capital lasts about sixty years,
/// and what the pool spends on new levels first replaces what wore out. Public buildings are
/// the state's to keep.
pub const DEPRECIATION: Fixed = milli(15);

/// Share of its 1925 distance to the leader a region's institutions can close, at full
/// literacy: the rest of the gap persists (conditional convergence).
const CLOSE_WITH_SCHOOLING: Fixed = milli(500);

/// A plant switches when another method earns this much more.
const SWITCH_MARGIN: Fixed = milli(1100);
/// Literacy speed with no schools, and the gain at full enrolment: half the children in
/// school leaves the phase 1 speed unchanged.
const SCHOOL_BASE: Fixed = milli(600);
const SCHOOL_GAIN: Fixed = milli(800);

pub struct Yearly {
    pub cat: Arc<Catalogue>,
    pub tree: Option<Arc<crate::research::TechTree>>,
}

pub struct YearlyChanges {
    methods: Vec<Vec<u8>>,
    /// Equipment the switches ordered, by market: (good, quantity).
    switches: Vec<Orders>,
    literacy: Vec<Fixed>,
    tfp: Vec<Fixed>,
    records: Vec<YearRecord>,
}

impl System<GameWorld> for Yearly {
    type Changes = YearlyChanges;
    fn name(&self) -> &'static str {
        "economy-year"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Yearly { month: 1, day: 1 }
    }
    fn access(&self) -> Access {
        Access::new().read(ECON).write(ECON)
    }
    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> YearlyChanges {
        let cat = &*self.cat;
        let econ = &world.econ;
        let year = ctx.date.year;
        let unlocks = Unlocks::new(cat, year, self.tree.as_deref(), &world.research);
        let wages = super::monthly::market_wages(econ);
        let national = super::local::national(
            cat,
            self.tree.as_deref(),
            &world.research,
            econ.markets.len(),
        );
        let methods = econ
            .regions
            .par_iter()
            .map(|r| {
                let m = &econ.markets[r.market as usize];
                // Transport and power are judged at the state's price and supply.
                let view = super::local::View::last_week(cat, m, r, &national[r.market as usize]);
                let candidates: Vec<Option<(u8, Fixed)>> = r
                    .plants
                    .iter()
                    .map(|p| {
                        let b = &cat.buildings[p.kind as usize];
                        let (tfp, wage) = (plant_tfp(r, m, b), wages[r.market as usize]);
                        let current = b.methods[p.method as usize].profit(&view.price, tfp, wage);
                        match best_method(
                            cat,
                            p.kind as usize,
                            &unlocks,
                            r.market as usize,
                            &view.price,
                            &view.buy,
                            tfp,
                            wage,
                        ) {
                            Some((mi, profit))
                                if mi != p.method
                                    && profit > current.max(Fixed::ZERO) * SWITCH_MARGIN =>
                            {
                                Some((mi, profit - current))
                            }
                            _ => None,
                        }
                    })
                    .collect();
                let gains: Vec<Fixed> = candidates
                    .iter()
                    .map(|c| c.map_or(Fixed::ZERO, |(_, g)| g))
                    .collect();
                let methods = power_at_hand(cat, r, m, &national[r.market as usize], candidates);
                (methods, gains)
            })
            .collect();
        let (methods, switches) = paid_switches(cat, econ, methods);
        // Schooling: literacy closes part of its gap each year, faster where people are richer
        // and where schools reach more of the children.
        let school = cat.building("school");
        let literacy = econ
            .regions
            .iter()
            .map(|r| {
                let total = r.population().max(1);
                let comfort: Fixed = r
                    .pops
                    .iter()
                    .map(|p| p.satisfaction[1] * p.size)
                    .sum::<Fixed>()
                    / total;
                // Mass schooling, radio and television speed it up.
                let boost = world
                    .research
                    .effects
                    .get(r.market as usize)
                    .map_or(Fixed::ONE, |e| Fixed::ONE + e[2] * milli(300));
                // Schooling beyond what the education law asks for does not count.
                let schools =
                    super::public::coverage(cat, r, school, super::public::PUPILS_PER_PERSON).min(
                        world.econ.markets[r.market as usize]
                            .laws
                            .of(r)
                            .enrolment_target,
                    );
                let reach = if school.is_some() {
                    SCHOOL_BASE + schools * SCHOOL_GAIN
                } else {
                    Fixed::ONE
                };
                let speed = (milli(8) + comfort * milli(25)) * boost * reach;
                r.literacy + (Fixed::ONE - r.literacy) * speed
            })
            .collect();
        // Stand-in for research until phase 2: productivity grows faster where more people can
        // read, and regions behind the leader catch up in proportion to the gap (capped at 2×).
        let leader = econ
            .regions
            .iter()
            .map(|r| r.tfp)
            .max()
            .unwrap_or(Fixed::ONE);
        let leader_start = econ
            .regions
            .iter()
            .map(|r| r.tfp_start)
            .max()
            .unwrap_or(Fixed::ONE);
        let tfp = econ
            .regions
            .iter()
            .map(|r| {
                // With research, technology moves productivity by building group and only the
                // rest of the 1925 gap (institutions, schooling) closes here.
                let own = if world.research.is_empty() {
                    milli(4) + r.literacy * milli(14)
                } else {
                    Fixed::ZERO
                };
                // Convergence is conditional: a region closes in on the share of the leader its
                // institutions allow, which starts at its 1925 standing and rises with schooling
                // (`CLOSE_WITH_SCHOOLING` of the rest at full literacy).
                let standing = (r.tfp_start / leader_start.max(milli(1))).min(Fixed::ONE);
                let reach = standing
                    + (Fixed::ONE - standing) * CLOSE_WITH_SCHOOLING * r.literacy * r.literacy;
                let gap = ((leader * reach - r.tfp) / r.tfp.max(milli(1)))
                    .clamp(Fixed::ZERO, Fixed::from_int(2));
                let catch_up = gap * (milli(5) + r.literacy * milli(20));
                r.tfp * (Fixed::ONE + own + catch_up)
            })
            .collect();
        let mut records = Vec::new();
        for (mi, m) in econ.markets.iter().enumerate() {
            let regions: Vec<_> = econ
                .regions
                .iter()
                .filter(|r| r.market as usize == mi)
                .collect();
            let population: i64 = regions.iter().map(|r| r.population()).sum();
            if population == 0 {
                continue;
            }
            let literacy = regions
                .iter()
                .map(|r| r.literacy * r.population())
                .sum::<Fixed>()
                / population;
            let urban = regions
                .iter()
                .map(|r| r.urban_share() * r.population())
                .sum::<Fixed>()
                / population;
            let basic = regions
                .iter()
                .flat_map(|r| r.pops.iter())
                .map(|p| (p.satisfaction[0] + p.satisfaction[1]) * p.size)
                .sum::<Fixed>()
                / (population * 2);
            records.push(YearRecord {
                year: year - 1,
                market: mi as u16,
                population,
                gdp: m.gdp_running,
                literacy,
                urban,
                basic_needs: fill(basic, Fixed::ONE),
            });
        }
        YearlyChanges {
            methods,
            switches,
            literacy,
            tfp,
            records,
        }
    }
    fn apply(&self, ctx: &TickCtx, changes: YearlyChanges, world: &mut GameWorld) {
        let econ = &mut world.econ;
        for (((r, methods), lit), tfp) in econ
            .regions
            .iter_mut()
            .zip(changes.methods)
            .zip(changes.literacy)
            .zip(changes.tfp)
        {
            r.literacy = lit;
            // The first January has no year behind it: no prices seen yet, and the 1925 plants
            // were sized for the methods they start with.
            if ctx.tick > 0 {
                for (p, m) in r.plants.iter_mut().zip(methods) {
                    p.method = m;
                    if !self.cat.buildings[p.kind as usize].public {
                        p.wear += Fixed::from_int(p.levels) * DEPRECIATION;
                        let gone = p.wear.to_int_floor().min(p.levels);
                        p.levels -= gone;
                        p.wear -= Fixed::from_int(gone);
                    }
                }
                r.tfp = tfp;
            }
        }
        if ctx.tick > 0 {
            for (m, goods) in econ.markets.iter_mut().zip(changes.switches) {
                for (g, q) in goods {
                    m.construction[g as usize] += q;
                }
            }
        }
        // Last year's inflation and the central banks; taxes for the year ahead.
        for m in &mut econ.markets {
            super::money::yearly(m, ctx.date.year);
            super::public::yearly_taxes(m, ctx.date.year);
        }
        // The first January of the run has no full year behind it: keep the 1925 figures.
        if ctx.tick > 0 {
            econ.history.extend(changes.records);
            for m in &mut econ.markets {
                m.gdp_last_year = m.gdp_running;
                m.gdp_running = Fixed::ZERO;
            }
        }
    }
}

/// Goods a market orders: (good, quantity).
type Orders = Vec<(u16, Fixed)>;

/// A new method is new equipment: the switches the plants want (`wanted`: methods by region
/// and the gain per level of each switch) are paid by the country's investment pool, the most
/// profitable for the money first, as far as the pool goes. A switch costs the equipment part
/// (what is not building work) of the construction basket of a level of the new method; it
/// joins the market's construction orders like the machines of a new level.
fn paid_switches(
    cat: &Catalogue,
    econ: &super::Economy,
    wanted: Vec<(Vec<u8>, Vec<Fixed>)>,
) -> (Vec<Vec<u8>>, Vec<Orders>) {
    let mut budget: Vec<Fixed> = econ
        .markets
        .iter()
        .map(|m| m.pool - super::monthly::committed(cat, m))
        .collect();
    // (gain per unit of cost, region, plant, cost)
    let mut offers: Vec<Vec<(Fixed, usize, usize, Fixed)>> = vec![Vec::new(); econ.markets.len()];
    for (ri, (r, (methods, gains))) in econ.regions.iter().zip(&wanted).enumerate() {
        for (i, p) in r.plants.iter().enumerate() {
            let new = methods[i];
            if new == p.method {
                continue;
            }
            let tfp = plant_tfp(
                r,
                &econ.markets[r.market as usize],
                &cat.buildings[p.kind as usize],
            );
            let level_cost = switch_cost(cat, p.kind as usize, new, tfp);
            let cost = level_cost * p.levels;
            let ratio = if level_cost > Fixed::ZERO {
                gains[i] / level_cost
            } else {
                Fixed::MAX
            };
            offers[r.market as usize].push((ratio, ri, i, cost));
        }
    }
    let mut methods: Vec<Vec<u8>> = econ
        .regions
        .iter()
        .map(|r| r.plants.iter().map(|p| p.method).collect())
        .collect();
    let mut orders: Vec<Orders> = vec![Vec::new(); econ.markets.len()];
    for (mi, mut list) in offers.into_iter().enumerate() {
        list.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let m = &econ.markets[mi];
        for (_, ri, i, cost) in list {
            if cost > budget[mi] {
                continue;
            }
            budget[mi] -= cost;
            let r = &econ.regions[ri];
            let p = &r.plants[i];
            let new = wanted[ri].0[i];
            let tfp = plant_tfp(r, m, &cat.buildings[p.kind as usize]);
            methods[ri][i] = new;
            let bd = &cat.buildings[p.kind as usize];
            for &(g, share, work) in &bd.construction {
                if !work {
                    let money =
                        (super::monthly::level_capital(cat, p.kind as usize, new as usize, tfp)
                            * p.levels)
                            .mul_ratio(share, 1000);
                    orders[mi].push((g, money / m.price[g as usize]));
                }
            }
        }
    }
    (methods, orders)
}

/// What switching a level to `method` costs: the equipment share of its capital
/// (`monthly::level_capital`).
fn switch_cost(cat: &Catalogue, kind: usize, method: u8, tfp: Fixed) -> Fixed {
    let equipment: i64 = cat.buildings[kind]
        .construction
        .iter()
        .filter(|(_, _, work)| !work)
        .map(|(_, share, _)| *share)
        .sum();
    super::monthly::level_capital(cat, kind, method as usize, tfp).mul_ratio(equipment, 1000)
}

/// The methods the plants of `r` run next year, from the switches they want (`candidates`, new
/// method and gain). A switch that needs more of the state's own transport or power than the
/// state has to spare waits, the most profitable first: a mill is electrified when there is a
/// line to plug it into.
fn power_at_hand(
    cat: &Catalogue,
    r: &super::Region,
    m: &super::Market,
    national: &[bool],
    candidates: Vec<Option<(u8, Fixed)>>,
) -> Vec<u8> {
    let mut methods: Vec<u8> = r.plants.iter().map(|p| p.method).collect();
    let nl = cat.local.len();
    let scale = |p: &super::Plant| {
        Fixed::from_int(p.levels)
            * p.staffing
            * p.utilization
            * plant_tfp(r, m, &cat.buildings[p.kind as usize])
    };
    let use_of = |p: &super::Plant, method: u8, g: u16| -> Fixed {
        cat.buildings[p.kind as usize].methods[method as usize]
            .inputs
            .iter()
            .filter(|(gi, _)| *gi == g)
            .map(|(_, q)| *q)
            .sum::<Fixed>()
            * scale(p)
    };
    // What the state makes of each local good at full use, less what it used last week.
    let mut spare = vec![Fixed::ZERO; nl];
    for (li, &g) in cat.local.iter().enumerate() {
        if national.get(li).copied().unwrap_or(false) {
            spare[li] = Fixed::MAX;
            continue;
        }
        let made: Fixed = r
            .plants
            .iter()
            .map(|p| {
                let q: Fixed = cat.buildings[p.kind as usize].methods[p.method as usize]
                    .outputs
                    .iter()
                    .filter(|(gi, _)| *gi == g)
                    .map(|(_, q)| *q)
                    .sum();
                q * Fixed::from_int(p.levels)
                    * p.staffing
                    * plant_tfp(r, m, &cat.buildings[p.kind as usize])
            })
            .sum();
        let used = r.local.get(li).map_or(Fixed::ZERO, |l| l.demand);
        spare[li] = (made - used).max(Fixed::ZERO);
    }
    let mut wanted: Vec<(Fixed, usize, u8)> = candidates
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.map(|(mi, gain)| (gain, i, mi)))
        .collect();
    wanted.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (_, i, mi) in wanted {
        let p = &r.plants[i];
        let extra: Vec<Fixed> = cat
            .local
            .iter()
            .map(|&g| use_of(p, mi, g) - use_of(p, p.method, g))
            .collect();
        if extra.iter().zip(&spare).all(|(e, s)| *e <= *s) {
            for (s, e) in spare.iter_mut().zip(&extra) {
                if *s != Fixed::MAX {
                    *s -= *e;
                }
            }
            methods[i] = mi;
        }
    }
    methods
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::catalogue::{Building, Good, Method};
    use crate::econ::state::{LocalGood, empty_pops};
    use crate::econ::{Market, Plant, Region};

    fn good(key: &str, local: bool) -> Good {
        Good {
            key: key.into(),
            category: "energy".into(),
            base_price: Fixed::ONE,
            available_from: 1925,
            tradable: !local,
            local,
            national_with: None,
        }
    }

    fn method(inputs: Vec<(u16, i64)>, outputs: Vec<(u16, i64)>) -> Method {
        let q = |v: Vec<(u16, i64)>| {
            v.into_iter()
                .map(|(g, n)| (g, Fixed::from_int(n)))
                .collect()
        };
        Method {
            available_from: 1925,
            inputs: q(inputs),
            outputs: q(outputs),
            ..Method::default()
        }
    }

    fn building(key: &str, methods: Vec<Method>) -> Building {
        Building {
            key: key.into(),
            group: "light_industry".into(),
            methods,
            ..Building::default()
        }
    }

    fn plant(kind: u16) -> Plant {
        Plant {
            kind,
            levels: 1,
            method: 0,
            staffing: Fixed::ONE,
            input_ratio: Fixed::ONE,
            utilization: Fixed::ONE,
            profit: Fixed::ZERO,
            losing_weeks: 0,
            wear: Fixed::ZERO,
        }
    }

    /// Goods: electricity (local), coal, cloth. A power plant makes 100 a week; two mills burn
    /// coal and could switch to an electric method that takes 50.
    fn world(used_last_week: i64) -> (Catalogue, Region) {
        let mill = || {
            vec![
                method(vec![(1, 10)], vec![(2, 10)]),
                method(vec![(0, 50)], vec![(2, 20)]),
            ]
        };
        let cat = Catalogue {
            goods: vec![
                good("electricity", true),
                good("coal", false),
                good("cloth", false),
            ],
            buildings: vec![
                building("power_plant", vec![method(vec![(1, 50)], vec![(0, 100)])]),
                building("mill_a", mill()),
                building("mill_b", mill()),
            ],
            local: vec![0],
            ..Catalogue::default()
        };
        let region = Region {
            market: 0,
            colony: false,
            pops: empty_pops(),
            plants: vec![plant(0), plant(1), plant(2)],
            literacy: Fixed::ZERO,
            deposits: Vec::new(),
            land: Vec::new(),
            subsistence: Fixed::ZERO,
            tfp: Fixed::ONE,
            tfp_start: Fixed::ONE,
            state: 0,
            neighbours: Vec::new(),
            local: vec![LocalGood {
                price: Fixed::ONE,
                buy: Fixed::ONE,
                sell: Fixed::ONE,
                demand: Fixed::from_int(used_last_week),
            }],
        };
        (cat, region)
    }

    #[test]
    fn plants_electrify_only_as_far_as_the_state_has_power_to_spare() {
        // 70 to spare: room for one mill, the one that gains more.
        let (cat, r) = world(30);
        let wants = vec![
            None,
            Some((1, Fixed::from_int(5))),
            Some((1, Fixed::from_int(9))),
        ];
        let got = power_at_hand(&cat, &r, &Market::empty(3), &[false], wants.clone());
        assert_eq!(got, vec![0, 0, 1]);
        // With the grid across the country, the state's own plants do not limit anyone.
        let got = power_at_hand(&cat, &r, &Market::empty(3), &[true], wants.clone());
        assert_eq!(got, vec![0, 1, 1]);
        // Nothing to spare: both wait.
        let (cat, r) = world(100);
        let got = power_at_hand(&cat, &r, &Market::empty(3), &[false], wants);
        assert_eq!(got, vec![0, 0, 0]);
    }
}
