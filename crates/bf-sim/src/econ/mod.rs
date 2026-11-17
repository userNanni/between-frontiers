//! The economy of phase 1 on the engine: production, national markets, wages and consumption,
//! world trade, labour, demography, investment and the change of production methods.
//!
//! See `docs/economy.pt-BR.md` ("Modelo") for the rules and their calibration.

pub mod catalogue;
pub(crate) mod compat;
pub mod construction;
pub mod local;
pub mod migration;
pub mod money;
pub mod monthly;
pub mod public;
pub mod setup;
pub mod state;
pub mod trade;
pub mod week;
pub mod yearly;

use laplace_schelling::Fixed;

pub use catalogue::{Building, Catalogue};
pub use state::{Budget, Economy, Laws, Market, Plant, Pop, Project, Region, YearRecord};

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
/// Share of a plant's profit paid out to its owners; the rest stays with the firm, in the
/// country's investment pool.
pub const PAYOUT: Fixed = milli(750);
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
/// Share of value added paid as wages, by building group.
pub fn labor_share(group: &str) -> Fixed {
    match group {
        "agriculture" => milli(700),
        "extraction" => milli(600),
        "services" | "infrastructure" => milli(650),
        _ => milli(600),
    }
}

/// The limits of a price, as shares of the base price. Prices move a quarter of the way each
/// week toward base × (1 + 0.75 × tension), within the band; to the ceiling when nothing is
/// offered, to the floor when nothing is asked, to base when neither.
pub struct PriceBand {
    pub floor: Fixed,
    pub ceiling: Fixed,
}

impl PriceBand {
    const SPEED: Fixed = milli(250);
    const RESPONSE: Fixed = milli(750);

    /// Next week's price. `tension` (shortage over the smaller side, negative for a glut) is
    /// asked for only when both sides are positive.
    pub fn next(
        &self,
        old: Fixed,
        base: Fixed,
        supply: Fixed,
        demand: Fixed,
        tension: impl FnOnce() -> Fixed,
    ) -> Fixed {
        let target = if supply <= Fixed::ZERO && demand <= Fixed::ZERO {
            base
        } else if supply <= Fixed::ZERO {
            base * self.ceiling
        } else if demand <= Fixed::ZERO {
            base * self.floor
        } else {
            base * (Fixed::ONE + tension() * Self::RESPONSE).clamp(self.floor, self.ceiling)
        };
        old + (target - old) * Self::SPEED
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

/// What exists and what each country may build: by year without research data, by the stage of
/// each technology with it.
pub struct Unlocks<'a> {
    year: i32,
    tech: Option<(
        &'a crate::research::TechTree,
        &'a crate::research::ResearchState,
    )>,
    /// Goods that exist in the world (someone can make them).
    pub goods: Vec<bool>,
}

impl<'a> Unlocks<'a> {
    pub fn new(
        cat: &Catalogue,
        year: i32,
        tree: Option<&'a crate::research::TechTree>,
        research: &'a crate::research::ResearchState,
    ) -> Self {
        let tech = tree.filter(|_| !research.is_empty()).map(|t| (t, research));
        let goods = (0..cat.goods.len())
            .map(|g| match tech {
                Some((t, r)) => r.good_exists(t, g),
                None => cat.goods[g].available_from <= year,
            })
            .collect();
        Self { year, tech, goods }
    }

    /// Calendar rules only: what history had made by `year`.
    pub fn by_year(cat: &Catalogue, year: i32) -> Unlocks<'static> {
        Unlocks {
            year,
            tech: None,
            goods: cat.goods.iter().map(|g| g.available_from <= year).collect(),
        }
    }

    /// Whether `country` may build `building` with `method`.
    pub fn method(&self, cat: &Catalogue, country: usize, building: usize, method: usize) -> bool {
        match self.tech {
            Some((t, r)) => r.can_build(t, country, building, method),
            None => cat.buildings[building].methods[method].available_from <= self.year,
        }
    }
}

/// Productivity of a plant of building `b` in region `r` of market `m`.
pub fn plant_tfp(r: &Region, m: &Market, b: &Building) -> Fixed {
    r.tfp_for(b)
        * m.tech_tfp
            .get(usize::from(b.group_ix))
            .copied()
            .unwrap_or(Fixed::ONE)
}
