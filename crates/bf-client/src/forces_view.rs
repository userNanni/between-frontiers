//! The player's armed forces, read from the simulation once a month for the government's
//! forces tab and the journal: what is under arms by unit type, how full its equipment is, how
//! trained it is, the reserve and how much of it has reported.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use bf_sim::war::{CountryWar, Domain, Military, PeaceRules, men};
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// One unit type of the player's forces.
#[derive(Clone, Debug, PartialEq)]
pub struct ForceRow {
    /// Unit key (`unit-<key>` in the Fluent files).
    pub unit: String,
    pub domain: Domain,
    /// Generation of the equipment, 1 for that of 1925.
    pub generation: u8,
    /// Elements: thousands of men, ships or aircraft.
    pub elements: f64,
    pub men: i64,
    /// Equipment held over equipment needed, by value; above 1 with equipment bought beyond the
    /// needs.
    pub equipment: f64,
    /// Training, 0–1, weighted by men.
    pub training: f64,
    /// Reservists called up.
    pub called_up: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForcesView {
    pub month: (i32, u8),
    /// The composition, while the war is switched on and the rule is in the data.
    pub composition: Option<CompositionView>,
    pub rows: Vec<ForceRow>,
    pub men: i64,
    pub reserve: i64,
    /// Reservists who have reported.
    pub called_up: i64,
    pub mobilized: bool,
    /// What the state spends on the forces' goods this week, real dollars.
    pub weekly_goods: f64,
    /// Men under arms over the men of service age (measured each January), 0 before.
    pub peace_share: f64,
    /// The war reserve in store, at this week's prices, real dollars.
    pub depot: f64,
    /// The orders of forces and what can be ordered.
    pub orders: Option<crate::orders_view::OrdersView>,
    /// The queue and the formations when read: a change (an order placed or cancelled, a
    /// formation disbanded or grown) reads the forces again before the month ends.
    pub stock: Stock,
    /// Each formation, as the disband step needs it.
    pub formations: Vec<FormationRef>,
    /// The country's forces when read, for the simulation's own account of what a disband sends
    /// to the depot (`war::disband_preview`).
    pub war: CountryWar,
    /// This week's price of each good, real dollars.
    pub prices: Vec<f64>,
    /// The army's generation and readiness, with the war rule on.
    pub army: Option<crate::army_view::ArmyView>,
    /// The nuclear programme (N1), while the war is switched on and the rule is in the data.
    pub programme: Option<ProgrammeView>,
}

/// The nuclear programme as the forces tab shows it (`war::programme`).
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammeView {
    /// Share of GDP a year, per mille; 0 without a programme. And the most it may be.
    pub share: u16,
    pub max_share: u16,
    /// What it costs a year at this share, and what it delivered so far, real dollars.
    pub money_year: f64,
    pub spent: f64,
    /// Each technology of the chain: its key, stage and progress (0–1).
    pub steps: Vec<(String, bf_sim::research::Stage, f64)>,
    /// Months to finish the step under way at the month's credit; `None` without credit yet or
    /// with the chain done.
    pub months_left: Option<f64>,
    /// Warheads in the forces, and ordered not yet delivered.
    pub warheads: f64,
    pub ordered: f64,
    /// The weapon core plants by method (high enrichment, production reactor): levels and the
    /// cores they make a year at this week's pace.
    pub core_plants: Vec<CorePlant>,
    /// The warhead's core (its main equipment): the price and the share the market delivered.
    pub fissile_price: f64,
    pub fissile_delivered: f64,
    /// Where the plants' uranium ore comes from; `None` when no method of the plant takes ore.
    pub ore: Option<OreView>,
    /// N3: the country tested a weapon; or asked for a test, to go through next week.
    pub tested: bool,
    pub test_asked: bool,
}

/// The weapon core plants of one method.
#[derive(Clone, Debug, PartialEq)]
pub struct CorePlant {
    /// Method key (`method-<key>`).
    pub method: String,
    pub levels: i64,
    pub cores_year: f64,
}

/// The uranium ore of the programme: what the country mines, what its core plants take, the
/// market's stock (`econ::stock`, with `market_stock`), its price and the share delivered.
#[derive(Clone, Debug, PartialEq)]
pub struct OreView {
    pub mined_year: f64,
    pub used_year: f64,
    pub stock: f64,
    pub price: f64,
    pub delivered: f64,
}

/// The programme of country `ci`, with its rule and the tree.
pub fn programme_view(
    world: &GameWorld,
    cat: &Catalogue,
    military: &Military,
    setup: &bf_sim::war::programme::ProgrammeSetup,
    tree: &bf_sim::research::TechTree,
    ci: usize,
) -> Option<ProgrammeView> {
    use bf_sim::research::Stage;
    let rule = &setup.rule;
    let m = world.econ.markets.get(ci)?;
    let p = world.war.nuclear.of(ci as u16);
    let share = p.map_or(0, |p| p.share.min(rule.max_share as u16));
    let stage = |t: usize| world.research.stage.get(ci).and_then(|s| s.get(t)).copied();
    let progress = |t: usize| {
        world
            .research
            .progress
            .get(ci)
            .and_then(|s| s.get(t))
            .map_or(0.0, |x| f(*x))
    };
    let steps: Vec<(String, Stage, f64)> = setup
        .chain
        .iter()
        .filter_map(|&t| Some((tree.techs[t].key.clone(), stage(t)?, progress(t))))
        .collect();
    // The step under way: the first still to understand or to prototype.
    let months_left = p.and_then(|p| {
        let credit = f(p.month);
        let &t = setup
            .chain
            .iter()
            .find(|&&t| matches!(stage(t), Some(Stage::Unknown | Stage::Science)))?;
        let base = if stage(t) == Some(Stage::Unknown) {
            1.0
        } else {
            0.5
        };
        let cost = f64::from(tree.techs[t].cost) * f64::from(rule.dollars_per_cost) * base;
        (credit > 0.0).then(|| (1.0 - progress(t)).max(0.0) * cost / credit)
    });
    let warhead = military.unit(&rule.warhead);
    let c = world.war.countries.get(ci)?;
    let warheads = c
        .formations
        .iter()
        .filter(|x| Some(x.unit) == warhead)
        .map(|x| f(x.elements))
        .sum();
    let ordered = c
        .queue
        .iter()
        .filter(|o| o.unit == rule.warhead)
        .map(|o| f(o.ordered - o.delivered))
        .sum();
    // The core plants by method, and the ore: what the country's plants make of each good and
    // take of it a year, at this week's pace.
    let plant = cat.building(&rule.plant);
    let ore = plant.and_then(|k| {
        cat.buildings[k]
            .methods
            .iter()
            .flat_map(|m| m.inputs.iter().map(|(g, _)| usize::from(*g)))
            .find(|&g| cat.goods.get(g).is_some_and(|d| d.key == "uranium_ore"))
    });
    let mut core_plants: Vec<CorePlant> = Vec::new();
    let (mut mined, mut used) = (0.0, 0.0);
    for r in world
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == ci)
    {
        for x in &r.plants {
            let Some(b) = cat.buildings.get(usize::from(x.kind)) else {
                continue;
            };
            let Some(method) = b.methods.get(usize::from(x.method)) else {
                continue;
            };
            let tfp = bf_sim::econ::plant_tfp(r, m, b);
            let scale =
                f(Fixed::from_int(x.levels) * x.staffing * tfp * x.utilization * x.input_ratio)
                    * 52.0;
            if let Some(g) = ore {
                mined += method
                    .outputs
                    .iter()
                    .filter(|(o, _)| usize::from(*o) == g)
                    .map(|(_, q)| f(*q) * scale)
                    .sum::<f64>();
            }
            if Some(usize::from(x.kind)) != plant {
                continue;
            }
            if let Some(g) = ore {
                used += method
                    .inputs
                    .iter()
                    .filter(|(i, _)| usize::from(*i) == g)
                    .map(|(_, q)| f(*q) * scale)
                    .sum::<f64>();
            }
            let cores: f64 = method.outputs.iter().map(|(_, q)| f(*q) * scale).sum();
            match core_plants.iter_mut().find(|c| c.method == method.key) {
                Some(c) => {
                    c.levels += x.levels;
                    c.cores_year += cores;
                }
                None => core_plants.push(CorePlant {
                    method: method.key.clone(),
                    levels: x.levels,
                    cores_year: cores,
                }),
            }
        }
    }
    let ore = ore.map(|g| OreView {
        mined_year: mined,
        used_year: used,
        stock: f(world.stocks.of(ci, g)),
        price: f(m.price[g]),
        delivered: f(m.buy_ratio[g]),
    });
    let fissile = military.unit(&rule.warhead).and_then(|ty| {
        military
            .spec(ty, 0)
            .equipment
            .iter()
            .max_by_key(|(_, q)| *q)
            .map(|(g, _)| usize::from(*g))
    });
    Some(ProgrammeView {
        share,
        max_share: rule.max_share as u16,
        money_year: f(m.gdp_last_year) * f64::from(share) / 1000.0,
        spent: p.map_or(0.0, |p| f(p.spent)),
        steps,
        months_left,
        warheads,
        ordered,
        core_plants,
        ore,
        fissile_price: fissile.map_or(0.0, |g| f(m.price[g])),
        fissile_delivered: fissile.map_or(0.0, |g| f(m.buy_ratio[g])),
        tested: world.war.nuclear.tested(ci as u16),
        test_asked: world.war.nuclear.test_requests.contains(&(ci as u16)),
    })
}

/// What [`stock`] reads: the queue's length, the formations' number and their elements.
pub type Stock = (usize, usize, Fixed);

/// The composition of the land forces (`war::composition`): what the army has on wheels and on
/// tracks, the target, and the infantry in conversion.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionView {
    /// Shares of the land men at home on wheels and on tracks, 0–1, each formation counted by
    /// the vehicles it holds.
    pub wheels: f64,
    pub tracks: f64,
    /// The government's target, per mille, when fixed.
    pub fixed: Option<[u16; 2]>,
    /// The target that applies (the government's or the rule's), per mille; `None` without the
    /// rule or before the country makes the infantry's first generation.
    pub target: Option<(u32, u32)>,
    /// The rule's target, per mille, whether or not the government fixed its own.
    pub rule_target: Option<(u32, u32)>,
    pub converting: Vec<Converting>,
    /// A type the country equips moves on wheels, and one on tracks: without one, that part
    /// of the target stays unconverted.
    pub wheels_type: bool,
    pub tracks_type: bool,
}

/// A conversion under way.
#[derive(Clone, Debug, PartialEq)]
pub struct Converting {
    pub unit: String,
    /// Elements ordered and delivered.
    pub ordered: f64,
    pub delivered: f64,
    /// Weeks in a row it got nothing, and the likeliest reason, read from the forces as they
    /// stand (the simulation keeps only the count).
    pub stalled: u32,
    pub why: Option<Stall>,
}

/// Why a conversion gets nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stall {
    /// The country cannot yet equip the new type.
    NoMaker,
    /// Nothing left of the military budget after the forces' needs and the orders.
    NoRoom,
    /// No infantry left to convert.
    NoInfantry,
    /// The room left does not pay the vehicles at this week's prices.
    Prices,
}

/// The queue and the formations of `c`, to know when the forces changed.
pub fn stock(c: &CountryWar) -> Stock {
    (
        c.queue.len(),
        c.formations.len(),
        c.formations.iter().map(|f| f.elements).sum(),
    )
}

/// One formation of the player's forces.
#[derive(Clone, Debug, PartialEq)]
pub struct FormationRef {
    /// Index in `CountryWar::formations`: what `DisbandForces` names.
    pub index: u32,
    /// The row of [`ForcesView::rows`] it is counted in.
    pub row: usize,
    pub elements: Fixed,
    /// Men in one element.
    pub men_each: u32,
    /// Raised by an order: its men join the reserve when disbanded. The others are of the peace
    /// army, which the January rule brings back to its size in peace.
    pub raised: bool,
    pub called_up: bool,
}

/// What disbanding some elements of a row does, before the click.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Disband {
    /// `(formation, elements)` to send as `DisbandForces`, the highest index first, so a
    /// formation left empty and taken off the list does not move the ones still to come.
    pub commands: Vec<(u32, u32)>,
    pub domain: Option<Domain>,
    /// Elements that leave.
    pub elements: f64,
    /// Equipment to the depot by good: `(good, quantity, value)`, the dearest first.
    pub goods: Vec<(u16, f64, f64)>,
    /// Value of all of it, at this week's prices.
    pub value: f64,
    /// Men of raised formations, who join the reserve.
    pub to_reserve: i64,
    /// Men of the peace army, who go home until January brings the army back to its size.
    pub home: i64,
}

impl ForcesView {
    /// The most elements of row `row` a disband can take (called-up reservists go home by
    /// demobilization, not here).
    pub fn disbandable(&self, row: usize) -> u32 {
        let total: Fixed = self
            .formations
            .iter()
            .filter(|f| f.row == row && !f.called_up)
            .map(|f| f.elements.max(Fixed::ZERO))
            .sum();
        total.to_f64_lossy().ceil() as u32
    }

    /// Disbanding `count` elements of row `row`: raised formations first (their men keep their
    /// training in the reserve, and the peace army would come back in January), the largest
    /// first, so the count is met in whole elements and a sliver left by modernization does not
    /// take an element's place. `None` when nothing would leave.
    pub fn disband(&self, row: usize, count: u32) -> Option<Disband> {
        let mut of: Vec<&FormationRef> = self
            .formations
            .iter()
            .filter(|f| f.row == row && !f.called_up && f.elements.is_positive())
            .collect();
        of.sort_by_key(|f| std::cmp::Reverse((f.raised, f.elements, f.index)));
        let mut left = Fixed::from_int(i64::from(count));
        let mut d = Disband::default();
        let mut goods: Vec<(u16, Fixed)> = Vec::new();
        for f in of {
            if !left.is_positive() {
                break;
            }
            // `DisbandForces` takes whole elements, as many as the formation holds at most.
            let n = left.min(f.elements).to_f64_lossy().ceil().max(1.0) as u32;
            let taken = f.elements.min(Fixed::from_int(i64::from(n)));
            left -= taken;
            let taken = taken.to_f64_lossy();
            d.elements += taken;
            let men = (taken * f64::from(f.men_each)).round() as i64;
            if self.rows.get(row).map(|r| r.domain) == Some(Domain::Land) {
                if f.raised {
                    d.to_reserve += men;
                } else {
                    d.home += men;
                }
            }
            for (g, q) in bf_sim::war::disband_preview(&self.war, f.index as usize, n) {
                match goods.iter_mut().find(|(k, _)| *k == g) {
                    Some(e) => e.1 += q,
                    None => goods.push((g, q)),
                }
            }
            d.commands.push((f.index, n));
        }
        if d.commands.is_empty() {
            return None;
        }
        d.commands.sort_by_key(|(i, _)| std::cmp::Reverse(*i));
        d.domain = self.rows.get(row).map(|r| r.domain);
        d.goods = goods
            .into_iter()
            .map(|(g, q)| {
                let q = f(q);
                (
                    g,
                    q,
                    q * self.prices.get(g as usize).copied().unwrap_or(0.0),
                )
            })
            .collect();
        d.goods
            .sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
        d.value = d.goods.iter().map(|g| g.2).sum();
        Some(d)
    }
}

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// The player's forces; `None` without a player or without forces.
pub fn read(
    world: &GameWorld,
    tag: &str,
    date: Date,
    cat: &Catalogue,
    military: &Military,
    peace: &PeaceRules,
    wage_floor: bool,
) -> Option<ForcesView> {
    let ci = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let c = world.war.countries.get(ci)?;
    let m = world.econ.markets.get(ci)?;
    let mut rows: Vec<ForceRow> = Vec::new();
    for form in &c.formations {
        let ty = &military.types[form.unit as usize];
        let spec = military.spec(form.unit, form.generation as usize);
        let need: Vec<(u16, Fixed)> = spec
            .equipment
            .iter()
            .map(|(g, q)| (*g, *q * form.elements))
            .collect();
        let need_value = f(military.value_lots(&need));
        let held_value = f(military.value_lots(&form.equipment));
        let n = men(military, form);
        let row = rows.iter_mut().find(|r| {
            r.unit == ty.key && r.called_up == form.called_up && r.generation == form.generation + 1
        });
        let equipment = if need_value > 0.0 {
            held_value / need_value
        } else {
            1.0
        };
        match row {
            Some(r) => {
                let total = (r.men + n).max(1) as f64;
                r.equipment = (r.equipment * r.men as f64 + equipment * n as f64) / total;
                r.training = (r.training * r.men as f64 + f(form.training) * n as f64) / total;
                r.elements += f(form.elements);
                r.men += n;
            }
            None => rows.push(ForceRow {
                unit: ty.key.clone(),
                domain: ty.domain,
                generation: form.generation + 1,
                elements: f(form.elements),
                men: n,
                equipment,
                training: f(form.training),
                called_up: form.called_up,
            }),
        }
    }
    rows.sort_by(|a, b| {
        (a.domain, a.called_up, &a.unit, a.generation).cmp(&(
            b.domain,
            b.called_up,
            &b.unit,
            b.generation,
        ))
    });
    let formations = c
        .formations
        .iter()
        .enumerate()
        .filter_map(|(i, form)| {
            let ty = &military.types[form.unit as usize];
            let row = rows.iter().position(|r| {
                r.unit == ty.key
                    && r.called_up == form.called_up
                    && r.generation == form.generation + 1
            })?;
            Some(FormationRef {
                index: i as u32,
                row,
                elements: form.elements,
                men_each: military.spec(form.unit, form.generation as usize).men,
                raised: form.raised,
                called_up: form.called_up,
            })
        })
        .collect();
    let weekly_goods = c
        .orders
        .iter()
        .map(|(g, q)| f(*q * m.price[*g as usize]))
        .sum();
    let depot = c
        .depot
        .iter()
        .map(|(g, q)| f(*q * m.price[*g as usize]))
        .sum();
    let orders = crate::orders_view::read(world, date, ci, cat, military, peace, wage_floor);
    let composition = peace.composition.as_ref().map(|rule| {
        let list: Vec<_> = world.war.composition.of(ci as u16).cloned().collect();
        // What the army moves on: the formations by the equipment they hold.
        let vehicles = bf_sim::war::composition::vehicle_goods(rule, cat);
        let (w, t, all) = bf_sim::war::composition::equipped(military, &vehicles, c);
        let share = |x: Fixed| {
            if all.is_positive() {
                f(x) / f(all)
            } else {
                0.0
            }
        };
        let fixed = world.war.composition.fixed_of(ci as u16);
        let newest = military
            .unit(&rule.from_unit)
            .and_then(|ty| bf_sim::war::system::newest_generation(military, world, ci, ty));
        let newest_of = |ty: u16| bf_sim::war::system::newest_generation(military, world, ci, ty);
        let has = |class| {
            bf_sim::war::composition::class_unit(military, rule, class, &newest_of).is_some()
        };
        CompositionView {
            wheels_type: has(bf_sim::war::Mobility::Wheeled),
            tracks_type: has(bf_sim::war::Mobility::Tracked),
            wheels: share(w),
            tracks: share(t),
            fixed,
            target: bf_sim::war::composition::target(rule, fixed, newest),
            rule_target: bf_sim::war::composition::target(rule, None, newest),
            converting: list
                .iter()
                .map(|x| {
                    let source: Fixed = military.unit(&x.from).map_or(Fixed::ZERO, |ty| {
                        c.formations
                            .iter()
                            .filter(|g| g.unit == ty)
                            .map(|g| g.elements)
                            .sum()
                    });
                    let why = (x.stalled > 0).then(|| {
                        if x.generation.is_none() {
                            Stall::NoMaker
                        } else if !source.is_positive() {
                            Stall::NoInfantry
                        } else if orders.as_ref().is_none_or(|o| o.spare_week <= 0.0) {
                            Stall::NoRoom
                        } else {
                            Stall::Prices
                        }
                    });
                    Converting {
                        unit: x.unit.clone(),
                        ordered: f(x.ordered),
                        delivered: f(x.delivered),
                        stalled: x.stalled,
                        why,
                    }
                })
                .collect(),
        }
    });
    Some(ForcesView {
        month: (date.year, date.month),
        composition,
        orders,
        stock: stock(c),
        formations,
        war: c.clone(),
        prices: m.price.iter().map(|p| f(*p)).collect(),
        men: rows.iter().map(|r| r.men).sum(),
        called_up: rows.iter().filter(|r| r.called_up).map(|r| r.men).sum(),
        reserve: c.reserve as i64,
        mobilized: c.mobilized,
        weekly_goods,
        peace_share: f(c.peace_share),
        depot,
        rows,
        army: None,
        programme: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_sim::war::Formation;

    fn formation(elements: i64, raised: bool, steel: i64) -> Formation {
        Formation {
            unit: 0,
            generation: 0,
            elements: Fixed::from_int(elements),
            colonial: false,
            called_up: false,
            equipment: vec![(3, Fixed::from_int(steel))],
            training: Fixed::ONE,
            raised,
        }
    }

    fn view(formations: Vec<Formation>) -> ForcesView {
        let war = CountryWar {
            formations,
            ..CountryWar::default()
        };
        ForcesView {
            month: (1926, 1),
            composition: None,
            rows: vec![ForceRow {
                unit: "infantry".into(),
                domain: Domain::Land,
                generation: 1,
                elements: war
                    .formations
                    .iter()
                    .map(|f| f.elements.to_f64_lossy())
                    .sum(),
                men: 0,
                equipment: 1.0,
                training: 1.0,
                called_up: false,
            }],
            men: 0,
            reserve: 0,
            called_up: 0,
            mobilized: false,
            weekly_goods: 0.0,
            peace_share: 0.0,
            depot: 0.0,
            army: None,
            orders: None,
            stock: stock(&war),
            formations: war
                .formations
                .iter()
                .enumerate()
                .map(|(i, f)| FormationRef {
                    index: i as u32,
                    row: 0,
                    elements: f.elements,
                    men_each: 1000,
                    raised: f.raised,
                    called_up: false,
                })
                .collect(),
            prices: vec![0.0, 0.0, 0.0, 2.0],
            programme: None,
            war,
        }
    }

    #[test]
    fn a_disband_takes_the_raised_first_and_sends_the_highest_index_first() {
        // The peace army (10 elements) and two orders (4 and 6), all of one row.
        let v = view(vec![
            formation(10, false, 100),
            formation(4, true, 40),
            formation(6, true, 30),
        ]);
        assert_eq!(v.disbandable(0), 20);
        let d = v.disband(0, 12).unwrap();
        // The raised ones whole (index 2 then 1), and 2 of the peace army.
        assert_eq!(d.commands, vec![(2, 6), (1, 4), (0, 2)]);
        assert_eq!(d.to_reserve, 10_000);
        assert_eq!(d.home, 2_000);
        // The same account as the simulation's: 30 + 40 + 2/10 of 100 of steel at 2 each.
        assert_eq!(d.goods.len(), 1);
        assert!((d.goods[0].1 - 90.0).abs() < 1e-6, "{:?}", d.goods);
        assert!((d.value - 180.0).abs() < 1e-6);
        assert!(v.disband(0, 0).is_none());
    }

    #[test]
    fn a_sliver_does_not_take_an_elements_place() {
        // A peace formation of 3 and a sliver of 0.25 left by modernization.
        let mut sliver = formation(0, false, 1);
        sliver.elements = Fixed::from_ratio(1, 4);
        let v = view(vec![formation(3, false, 30), sliver]);
        assert_eq!(v.disbandable(0), 4);
        let d = v.disband(0, 1).unwrap();
        assert_eq!(d.commands, vec![(0, 1)]);
        assert!((d.elements - 1.0).abs() < 1e-9);
        // All of it: the sliver goes in one asked element.
        let d = v.disband(0, 4).unwrap();
        assert_eq!(d.commands, vec![(1, 1), (0, 3)]);
        assert!((d.elements - 3.25).abs() < 1e-9);
    }

    #[test]
    fn called_up_reservists_are_not_disbanded_here() {
        let mut v = view(vec![formation(5, false, 10)]);
        v.formations[0].called_up = true;
        assert_eq!(v.disbandable(0), 0);
        assert!(v.disband(0, 3).is_none());
    }
}
