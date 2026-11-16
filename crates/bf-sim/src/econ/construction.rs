//! The queue of works: every new level, private or public, waits for construction points made
//! by the building trade (`construction_sector`). Points are bought weekly on the national
//! market; a work takes them in queue order, at most a share of its total each week, so even
//! with plenty of builders it takes half a year. A shortage of cement, steel or timber cuts the
//! builders' output and so delays every work.

use laplace_schelling::Fixed;

use super::state::{Market, Project};

/// The fastest a work can be finished, in weeks.
pub const MIN_WEEKS: i64 = 26;

/// Points a work can take this week.
pub fn want(p: &Project) -> Fixed {
    p.left
        .min(p.points.mul_ratio(1, MIN_WEEKS).max(Fixed::EPSILON))
        .max(Fixed::ZERO)
}

/// Points the whole queue asks for this week.
pub fn queue_want(m: &Market) -> Fixed {
    m.queue.iter().map(want).sum()
}

/// What a week of deliveries did to the queue.
pub struct Delivery {
    /// Works finished this week, in queue order.
    pub done: Vec<Project>,
    /// Money owed for the points, by the pool and by the treasury.
    pub private_cost: Fixed,
    pub public_cost: Fixed,
}

/// Hands out the points bought this week (`share` of what the queue asked for) in queue
/// order, at `price` each, and takes the finished works out of the queue.
pub fn deliver(m: &mut Market, share: Fixed, price: Fixed) -> Delivery {
    let mut points = queue_want(m) * share;
    let mut out = Delivery {
        done: Vec::new(),
        private_cost: Fixed::ZERO,
        public_cost: Fixed::ZERO,
    };
    for p in &mut m.queue {
        if points <= Fixed::ZERO {
            break;
        }
        let got = want(p).min(points);
        points -= got;
        p.left -= got;
        if p.public {
            out.public_cost += got * price;
        } else {
            out.private_cost += got * price;
        }
    }
    let (done, waiting): (Vec<Project>, Vec<Project>) = std::mem::take(&mut m.queue)
        .into_iter()
        .partition(|p| p.left <= Fixed::ZERO);
    m.queue = waiting;
    out.done = done;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(points: i64, public: bool) -> Project {
        Project {
            region: 0,
            kind: 0,
            method: 0,
            public,
            points: Fixed::from_int(points),
            left: Fixed::from_int(points),
        }
    }

    fn market(queue: Vec<Project>) -> Market {
        Market {
            queue,
            ..Market::empty(0)
        }
    }

    #[test]
    fn a_work_takes_at_least_half_a_year() {
        let mut m = market(vec![project(2600, false)]);
        let mut weeks = 0;
        while !m.queue.is_empty() {
            let d = deliver(&mut m, Fixed::ONE, Fixed::ONE);
            weeks += 1;
            assert!(d.public_cost == Fixed::ZERO);
        }
        assert_eq!(weeks, MIN_WEEKS);
    }

    #[test]
    fn a_shortage_delays_in_queue_order() {
        let mut m = market(vec![project(2600, false), project(2600, true)]);
        // Half the points: the first work gets its full share, the second nothing.
        let d = deliver(&mut m, milli_half(), Fixed::ONE);
        assert_eq!(m.queue[0].left, Fixed::from_int(2500));
        assert_eq!(m.queue[1].left, Fixed::from_int(2600));
        assert_eq!(d.private_cost, Fixed::from_int(100));
        assert_eq!(d.public_cost, Fixed::ZERO);
    }

    fn milli_half() -> Fixed {
        Fixed::from_ratio(1, 2)
    }
}
