//! What the forces screen reads about the orders (slice A2): what each type would cost, the room
//! the military budget leaves this week and how each order is going. Read when the screen asks
//! (the client once a month), never in the tick.

use laplace_schelling::Fixed;

use super::catalogue::{Domain, Military};
use super::state::PeaceRules;
use super::system::{needs_of, newest_generation};
use crate::econ::Catalogue;
use crate::game::GameWorld;

/// A type the country could order.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitOffer {
    /// Index in [`Military::types`].
    pub unit: u16,
    /// The newest generation the country can equip; `None` while it lacks the technology (an
    /// order then waits for it).
    pub generation: Option<u8>,
    /// Weeks to deliver an order: the men reporting a class a week on land
    /// ([`PeaceRules::raise_weeks`]), the build weeks otherwise.
    pub weeks: u32,
    /// Men per element.
    pub men: u32,
    /// Equipment of one element at this week's prices.
    pub equipment: Fixed,
    /// A year of one element once delivered: its consumption and the wear of its equipment.
    pub upkeep_year: Fixed,
    /// A year of pay of the men of one element, at the country's soldier pay.
    pub pay_year: Fixed,
}

/// An order in the queue.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderLine {
    /// Position in [`super::CountryWar::queue`] (what `CancelOrder` takes).
    pub index: usize,
    /// `None` when the catalogue lacks the type (the order is dropped).
    pub unit: Option<u16>,
    pub generation: Option<u8>,
    pub colonial: bool,
    pub ordered: Fixed,
    pub delivered: Fixed,
    /// Share of the equipment of what is still to deliver already built, 0 to 1 (ships on the
    /// slipway, aircraft on the line); zero on land.
    pub built: Fixed,
    /// Weeks in a row it got nothing for want of room in the budget.
    pub stalled: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrdersView {
    /// The military budget this week: the military part of the state's goods budget.
    pub room: Fixed,
    /// Of it, what the forces need this week (consumption and replacement).
    pub needs: Fixed,
    /// The room left once the needs are paid: what orders, modernization and the depot share.
    pub spare: Fixed,
    pub queue: Vec<OrderLine>,
    /// Every type of the catalogue, in its order.
    pub offers: Vec<UnitOffer>,
}

/// The orders of `country` (market index); `None` without forces. `wage_floor` is the rule
/// `small_market_wage_floor` (`game::Switches`), so the pay shown is the pay the tick uses.
pub fn orders_view(
    world: &GameWorld,
    cat: &Catalogue,
    mil: &Military,
    peace: &PeaceRules,
    country: usize,
    wage_floor: bool,
) -> Option<OrdersView> {
    let c = world.war.countries.get(country)?;
    let m = world.econ.markets.get(country)?;
    let price = |list: &[(u16, Fixed)]| -> Fixed {
        list.iter().map(|(g, q)| *q * m.price[*g as usize]).sum()
    };
    let room = crate::econ::public::military_room(m, cat);
    let needs = price(&needs_of(mil, peace, c, false).0);
    let wage = crate::econ::monthly::market_wages(&world.econ, wage_floor)
        .get(country)
        .copied()
        .unwrap_or(Fixed::ZERO);
    let soldier_pay = wage
        * crate::econ::WAGE_WEIGHT[crate::econ::prof::SOLDIER]
        * crate::econ::public::PUBLIC_PAY;
    let offers = (0..mil.types.len() as u16)
        .map(|ty| {
            let generation = newest_generation(mil, world, country, ty);
            let spec = mil.spec(ty, generation.unwrap_or(0) as usize);
            let equipment = price(&spec.equipment);
            let consumed = price(&spec.upkeep) * peace.upkeep_scale;
            let t = &mil.types[ty as usize];
            UnitOffer {
                unit: ty,
                generation,
                weeks: match t.domain {
                    Domain::Land => peace.raise_weeks.max(1),
                    _ => peace.build_weeks(&t.key),
                },
                men: spec.men,
                equipment,
                upkeep_year: (consumed + equipment * mil.rules.maintenance) * 52,
                pay_year: soldier_pay * i64::from(spec.men) * 52,
            }
        })
        .collect();
    let queue = c
        .queue
        .iter()
        .enumerate()
        .map(|(index, o)| {
            let unit = mil.unit(&o.unit);
            let built = match (unit, o.generation) {
                (Some(ty), Some(g)) if mil.types[ty as usize].domain != Domain::Land => {
                    let per = &mil.spec(ty, g as usize).equipment;
                    let left = o.ordered - o.delivered;
                    // By value at this week's prices: the goods are in different units.
                    let want: Fixed = per
                        .iter()
                        .map(|(g, q)| *q * left * m.price[*g as usize])
                        .sum();
                    let have: Fixed = per
                        .iter()
                        .map(|(g, q)| {
                            o.built
                                .iter()
                                .find(|(k, _)| k == g)
                                .map_or(Fixed::ZERO, |e| e.1)
                                .min(*q * left)
                                * m.price[*g as usize]
                        })
                        .sum();
                    if want.is_positive() {
                        (have / want).min(Fixed::ONE)
                    } else {
                        Fixed::ZERO
                    }
                }
                _ => Fixed::ZERO,
            };
            OrderLine {
                index,
                unit,
                generation: o.generation,
                colonial: o.colonial,
                ordered: o.ordered,
                delivered: o.delivered,
                built,
                stalled: o.stalled,
            }
        })
        .collect();
    Some(OrdersView {
        room,
        needs,
        spare: room - needs,
        queue,
        offers,
    })
}
