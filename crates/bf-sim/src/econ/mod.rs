//! The economy of phase 1 on the engine: production, national markets, wages and consumption,
//! world trade, labour, demography, investment and the change of production methods.
//!
//! See `docs/economy.pt-BR.md` ("Modelo") for the rules and their calibration.

pub mod catalogue;
pub mod monthly;
pub mod setup;
pub mod state;
pub mod week;
pub mod yearly;

use laplace_schelling::Fixed;

pub use catalogue::Catalogue;
pub use state::{Economy, Market, Plant, Pop, Region, YearRecord};

/// Profession indices, in the order of `economy::PROFESSIONS`.
pub mod prof {
    pub const P: usize = 13;
    pub const PEASANT: usize = 0;
    pub const FARM_LABORER: usize = 1;
    pub const LABORER: usize = 2;
    pub const SKILLED: usize = 3;
    pub const CLERK: usize = 4;
    pub const ENGINEER: usize = 5;
    pub const PROFESSIONAL: usize = 6;
    pub const MANAGER: usize = 7;
    pub const CAPITALIST: usize = 8;
    pub const LANDOWNER: usize = 9;
    pub const CIVIL_SERVANT: usize = 10;
    pub const CLERGY: usize = 11;
    pub const SOLDIER: usize = 12;
}

/// Thousandths, for readable constants.
pub const fn milli(n: i64) -> Fixed {
    Fixed::from_raw(n)
}

/// Share of a pop that works or looks for work.
pub const WORKER_SHARE: Fixed = milli(400);
/// Relative wage of each profession inside a building.
pub const WAGE_WEIGHT: [Fixed; prof::P] = [
    milli(600),
    milli(800),
    milli(1000),
    milli(1600),
    milli(1400),
    milli(3000),
    milli(3000),
    milli(5000),
    milli(1000),
    milli(1000),
    milli(1500),
    milli(1000),
    milli(1000),
];
/// Share of income each profession saves.
pub const SAVE_RATE: [Fixed; prof::P] = [
    milli(0),
    milli(20),
    milli(40),
    milli(60),
    milli(80),
    milli(120),
    milli(120),
    milli(150),
    milli(350),
    milli(300),
    milli(60),
    milli(40),
    milli(40),
];
/// Income tax that funds the state's servants and purchases.
pub const TAX_RATE: Fixed = milli(100);
/// Share of value added paid as wages, by building group.
pub fn labor_share(group: &str) -> Fixed {
    match group {
        "agriculture" => milli(700),
        "extraction" => milli(600),
        "services" | "infrastructure" => milli(650),
        _ => milli(600),
    }
}

/// `a / b` capped at 1; 1 when `b` is not positive (nothing was asked).
pub fn fill(a: Fixed, b: Fixed) -> Fixed {
    if b <= Fixed::ZERO || a >= b {
        Fixed::ONE
    } else if a <= Fixed::ZERO {
        Fixed::ZERO
    } else {
        a / b
    }
}

/// Workers among `size` people.
pub fn workers(size: i64) -> i64 {
    (Fixed::from_int(size) * WORKER_SHARE).to_int_floor()
}
