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

/// A plant switches when another method earns this much more.
const SWITCH_MARGIN: Fixed = milli(1100);

pub struct Yearly {
    pub cat: Arc<Catalogue>,
    pub tree: Option<Arc<crate::research::TechTree>>,
}

pub struct YearlyChanges {
    methods: Vec<Vec<u8>>,
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
        let methods = econ
            .regions
            .par_iter()
            .map(|r| {
                let m = &econ.markets[r.market as usize];
                r.plants
                    .iter()
                    .map(|p| {
                        let current = {
                            let me = &cat.buildings[p.kind as usize].methods[p.method as usize];
                            let out: Fixed = me
                                .outputs
                                .iter()
                                .map(|(g, q)| *q * m.price[*g as usize])
                                .sum();
                            let inp: Fixed = me
                                .inputs
                                .iter()
                                .map(|(g, q)| *q * m.price[*g as usize])
                                .sum();
                            (out - inp) * plant_tfp(r, m, &cat.buildings[p.kind as usize])
                                - wages[r.market as usize] * me.workers
                        };
                        match best_method(
                            cat,
                            p.kind as usize,
                            &unlocks,
                            r.market as usize,
                            &m.price,
                            &m.buy_ratio,
                            plant_tfp(r, m, &cat.buildings[p.kind as usize]),
                            wages[r.market as usize],
                        ) {
                            Some((mi, profit))
                                if mi != p.method
                                    && profit > current.max(Fixed::ZERO) * SWITCH_MARGIN =>
                            {
                                mi
                            }
                            _ => p.method,
                        }
                    })
                    .collect()
            })
            .collect();
        // Schooling: literacy closes part of its gap each year, faster where people are richer.
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
                let speed = (milli(8) + comfort * milli(25)) * boost;
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
                let gap = ((leader - r.tfp) / r.tfp.max(milli(1))).min(Fixed::from_int(2));
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
            for (p, m) in r.plants.iter_mut().zip(methods) {
                p.method = m;
            }
            r.literacy = lit;
            // The first January has no year behind it.
            if ctx.tick > 0 {
                r.tfp = tfp;
            }
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
