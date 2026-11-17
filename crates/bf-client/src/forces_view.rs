//! The player's armed forces, read from the simulation once a month for the government's
//! forces tab and the journal: what is under arms by unit type, how full its equipment is, how
//! trained it is, the reserve and how much of it has reported.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use bf_sim::war::{Domain, Military, PeaceRules, men};
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
    /// Length of the queue when read: a change reads the forces again before the month ends.
    pub queue_len: usize,
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
        queue_len: c.queue.len(),
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
