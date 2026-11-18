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
}

/// What [`stock`] reads: the queue's length, the formations' number and their elements.
pub type Stock = (usize, usize, Fixed);

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
        self.formations
            .iter()
            .filter(|f| f.row == row && !f.called_up)
            .map(|f| f.elements.to_f64_lossy().ceil().max(0.0) as u32)
            .sum()
    }

    /// Disbanding `count` elements of row `row`: raised formations first (their men keep their
    /// training in the reserve, and the peace army would come back in January), then the
    /// others, each in whole elements. `None` when nothing would leave.
    pub fn disband(&self, row: usize, count: u32) -> Option<Disband> {
        let mut of: Vec<&FormationRef> = self
            .formations
            .iter()
            .filter(|f| f.row == row && !f.called_up && f.elements.is_positive())
            .collect();
        of.sort_by_key(|f| (!f.raised, std::cmp::Reverse(f.index)));
        let mut left = count;
        let mut d = Disband::default();
        let mut goods: Vec<(u16, Fixed)> = Vec::new();
        for f in of {
            if left == 0 {
                break;
            }
            let whole = f.elements.to_f64_lossy().ceil().max(1.0) as u32;
            let n = left.min(whole);
            left -= n;
            let taken = f.elements.to_f64_lossy().min(f64::from(n));
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
    date: Date,
    cat: &Catalogue,
    military: &Military,
    peace: &PeaceRules,
    wage_floor: bool,
) -> Option<ForcesView> {
    let tag = world.player_tag()?;
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
    Some(ForcesView {
        month: (date.year, date.month),
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
    fn called_up_reservists_are_not_disbanded_here() {
        let mut v = view(vec![formation(5, false, 10)]);
        v.formations[0].called_up = true;
        assert_eq!(v.disbandable(0), 0);
        assert!(v.disband(0, 3).is_none());
    }
}
