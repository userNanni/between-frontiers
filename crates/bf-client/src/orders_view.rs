//! The player's orders of forces, for the forces tab: the simulation's reading
//! (`bf_sim::war::orders_view`) turned into what the screen shows: the queue with what each order
//! delivered and how far it is, and the types the country can equip with what one element costs
//! to build and, once it serves, a year.
//!
//! Read with the forces, once a month and when the queue changes length.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use bf_sim::war::{Domain, Military, PeaceRules};
use laplace_schelling::Fixed;

/// Weeks in a year.
const WEEKS: f64 = 52.0;

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// How far an order is, in the way it really advances.
#[derive(Clone, Debug, PartialEq)]
pub enum Progress {
    /// Land: the men report a class at a time; `reported` of the elements ordered, 0–1.
    Raising { reported: f64 },
    /// Ships, aircraft and strategic forces: built whole. `built` is the equipment on the
    /// slipway over the equipment of the elements not yet delivered, by value, 0–1.
    Building { built: f64 },
}

/// One order of the queue, in the order it is paid.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderRow {
    /// Position in the queue, as `CancelOrder` takes it.
    pub index: u32,
    pub unit: String,
    pub domain: Domain,
    /// Generation laid down, 1 for that of 1925; `None` until the first week paid.
    pub generation: Option<u8>,
    pub colonial: bool,
    pub ordered: f64,
    pub delivered: f64,
    pub progress: Progress,
    /// Weeks to deliver, from the type's offer.
    pub weeks: Option<u32>,
    /// Weeks in a row it got nothing for want of room in the military budget.
    pub waiting_weeks: u32,
    /// (year, month) of delivery at the pace so far: today plus the weeks of what is left.
    /// Later when the order waits for room in the budget.
    pub expected: Option<(i32, u8)>,
}

/// (year, month) `weeks` weeks after `date`.
pub fn weeks_after(date: laplace_schelling::Date, weeks: u32) -> (i32, u8) {
    let d = laplace_schelling::Date::from_days(date.to_days() + 7 * i64::from(weeks));
    (d.year, d.month)
}

/// A type the player can order, with the price of one element.
#[derive(Clone, Debug, PartialEq)]
pub struct Orderable {
    pub unit: String,
    pub domain: Domain,
    /// Generation it would be laid down at, 1 for that of 1925.
    pub generation: u8,
    /// Men per element (crew and ground staff for ships and aircraft).
    pub men: u32,
    /// Weeks to deliver: the men reporting on land, the build otherwise.
    pub weeks: u32,
    /// On land, the training that follows the reporting.
    pub training_weeks: Option<u32>,
    /// Equipment of one element at this week's prices, real dollars.
    pub build_cost: f64,
    /// A year of one element once it serves: consumption and wear of its equipment.
    pub upkeep_year: f64,
    /// A year of pay of its men.
    pub pay_year: f64,
}

impl Orderable {
    pub fn yearly_cost(&self) -> f64 {
        self.upkeep_year + self.pay_year
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrdersView {
    pub rows: Vec<OrderRow>,
    pub orderable: Vec<Orderable>,
    /// The country holds colonies: colonial forces can be ordered.
    pub colonies: bool,
    /// The military budget, a year at this week's pace, real dollars.
    pub military_year: f64,
    /// What is left this week once the forces' needs are paid: what orders, modernization and
    /// the depot share.
    pub spare_week: f64,
}

/// The player's orders, or `None` without forces.
pub fn read(
    world: &GameWorld,
    date: laplace_schelling::Date,
    ci: usize,
    cat: &Catalogue,
    military: &Military,
    peace: &PeaceRules,
    wage_floor: bool,
) -> Option<OrdersView> {
    let v = bf_sim::war::orders_view(world, cat, military, peace, ci, wage_floor)?;
    let key = |ty: u16| &military.types[ty as usize];
    let offer_weeks = |ty: u16| v.offers.iter().find(|o| o.unit == ty).map(|o| o.weeks);
    let rows = v
        .queue
        .iter()
        .filter_map(|o| {
            let ty = o.unit?;
            let t = key(ty);
            let progress = if t.domain == Domain::Land {
                Progress::Raising {
                    reported: if o.ordered.is_positive() {
                        f(o.delivered) / f(o.ordered)
                    } else {
                        0.0
                    },
                }
            } else {
                Progress::Building { built: f(o.built) }
            };
            Some(OrderRow {
                index: o.index as u32,
                unit: t.key.clone(),
                domain: t.domain,
                generation: o.generation.map(|g| g + 1),
                colonial: o.colonial,
                ordered: f(o.ordered),
                delivered: f(o.delivered),
                weeks: offer_weeks(ty),
                waiting_weeks: o.stalled,
                expected: offer_weeks(ty).map(|w| {
                    let done = match &progress {
                        Progress::Raising { reported } => *reported,
                        Progress::Building { built } => *built,
                    };
                    weeks_after(date, (f64::from(w) * (1.0 - done)).ceil().max(0.0) as u32)
                }),
                progress,
            })
        })
        .collect();
    let mut orderable: Vec<Orderable> = v
        .offers
        .iter()
        .filter_map(|o| {
            let g = o.generation?;
            let t = key(o.unit);
            Some(Orderable {
                unit: t.key.clone(),
                domain: t.domain,
                generation: g + 1,
                men: o.men,
                weeks: o.weeks,
                training_weeks: (t.domain == Domain::Land)
                    .then(|| military.spec(o.unit, g as usize).training_weeks),
                build_cost: f(o.equipment),
                upkeep_year: f(o.upkeep_year),
                pay_year: f(o.pay_year),
            })
        })
        .collect();
    orderable.sort_by_key(|o| o.domain);
    let colonies = world
        .econ
        .regions
        .iter()
        .any(|r| r.market as usize == ci && r.colony);
    Some(OrdersView {
        rows,
        orderable,
        colonies,
        military_year: f(v.room) * WEEKS,
        spare_week: f(v.spare),
    })
}

/// An order large enough to ask twice: its build cost is at least this share of a year of the
/// military budget, or it takes at least this many weeks (a battleship, a carrier).
pub const LARGE_SHARE: f64 = 0.10;
pub const LARGE_WEEKS: u32 = 104;

/// Whether ordering `elements` of `o` asks for a confirmation.
pub fn is_large(o: &Orderable, elements: u32, military_year: f64) -> bool {
    let cost = o.build_cost * f64::from(elements);
    o.weeks >= LARGE_WEEKS || (military_year > 0.0 && cost >= military_year * LARGE_SHARE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn battleship() -> Orderable {
        Orderable {
            unit: "battleship".into(),
            domain: Domain::Sea,
            generation: 1,
            men: 1200,
            weeks: 156,
            training_weeks: None,
            build_cost: 30.0e6,
            upkeep_year: 4.0e6,
            pay_year: 1.0e6,
        }
    }

    #[test]
    fn a_battleship_asks_twice_and_a_small_order_does_not() {
        assert!(is_large(&battleship(), 1, 1.0e12));
        let infantry = Orderable {
            unit: "infantry".into(),
            domain: Domain::Land,
            weeks: 12,
            training_weeks: Some(16),
            build_cost: 1.0e6,
            ..battleship()
        };
        assert!(!is_large(&infantry, 2, 100.0e6));
        assert!(is_large(&infantry, 10, 100.0e6));
    }
}
