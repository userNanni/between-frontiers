//! A synthetic toy world that exercises the scheduler: ~900 regions × 6 goods, a toy
//! production–consumption–price loop and toy population growth, across daily, weekly, monthly
//! and yearly systems. It is not the game's economy; it only has the shape and the cost
//! profile of one (per-region work, several cadences, conflicting and independent systems).

use laplace_schelling::fixed::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::ids::{Id, Table};
use laplace_schelling::rng::Rng;
use laplace_schelling::schedule::{Access, Cadence, ResourceId, Schedule, System, TickCtx};
use laplace_schelling::{Date, Weekday};
use rayon::prelude::*;

pub const REGIONS: u32 = 900;
pub const GOODS: usize = 6;

/// Units of each good one thousand people need per day.
const NEED_PER_THOUSAND: [i64; GOODS] = [2_000, 800, 300, 100, 50, 20];
/// Starting price of each good.
const BASE_PRICE: [i64; GOODS] = [1, 2, 5, 10, 20, 50];
const PRICE_FLOOR: Fixed = Fixed::from_raw(10);
const PRICE_CEILING: Fixed = Fixed::from_int(100_000);

pub struct Region;
/// One region × good cell.
pub struct Market;

const POP: ResourceId = ResourceId(0);
const WEALTH: ResourceId = ResourceId(1);
const WEATHER: ResourceId = ResourceId(2);
const CAPACITY: ResourceId = ResourceId(3);
const STOCK: ResourceId = ResourceId(4);
const PRICE: ResourceId = ResourceId(5);
const SHORTAGE: ResourceId = ResourceId(6);
const STATS: ResourceId = ResourceId(7);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldStats {
    pub year: i32,
    pub population: i64,
    pub wealth: Fixed,
    pub price_index: Fixed,
    pub markets_at_price_limit: u32,
}

pub struct World {
    pub population: Table<Region, i64>,
    pub wealth: Table<Region, Fixed>,
    pub weather: Table<Region, Fixed>,
    pub capacity: Table<Market, Fixed>,
    pub stock: Table<Market, Fixed>,
    pub price: Table<Market, Fixed>,
    pub shortage: Table<Market, Fixed>,
    pub stats: WorldStats,
}

fn market(region: usize, good: usize) -> usize {
    region * GOODS + good
}

fn need(population: i64, good: usize) -> Fixed {
    Fixed::from_int(population).mul_ratio(NEED_PER_THOUSAND[good], 1_000)
}

impl World {
    /// Deterministic world from the seed.
    pub fn generate(seed: u64) -> Self {
        let mut rng = Rng::stream(seed, 0, laplace_schelling::schedule::stream_id("init"), 0);
        let population = Table::from_fn(REGIONS, |_| {
            // Skewed sizes: a few large regions, many small ones (about 2 billion in all).
            let a = rng.range(30_000, 2_000_000);
            let b = rng.range(1, 4);
            a * b
        });
        let markets = REGIONS * GOODS as u32;
        let mut capacity = Table::from_fn(markets, |_| Fixed::ZERO);
        let mut stock = Table::from_fn(markets, |_| Fixed::ZERO);
        let price = Table::from_fn(markets, |id: Id<Market>| {
            Fixed::from_int(BASE_PRICE[id.index() % GOODS])
        });
        for (region, &people) in population.iter() {
            for good in 0..GOODS {
                let m = Id::new(market(region.index(), good) as u32);
                let daily = need(people, good);
                capacity[m] = daily.mul_ratio(rng.range(700, 1_300), 1_000);
                stock[m] = daily * 10;
            }
        }
        // About a month of spending.
        let wealth = Table::from_fn(REGIONS, |id| Fixed::from_int(population[id] * 250));
        Self {
            population,
            wealth,
            weather: Table::from_fn(REGIONS, |_| Fixed::ONE),
            capacity,
            stock,
            price,
            shortage: Table::from_fn(markets, |_| Fixed::ZERO),
            stats: WorldStats::default(),
        }
    }

    pub fn population_total(&self) -> i64 {
        self.population.as_slice().iter().sum()
    }
}

impl HashState for World {
    fn hash_state(&self, h: &mut StateHasher) {
        h.section("population");
        h.write_i64_slice(self.population.as_slice());
        for (name, column) in [("wealth", &self.wealth), ("weather", &self.weather)] {
            h.section(name);
            column.hash_state(h);
        }
        for (name, column) in [
            ("capacity", &self.capacity),
            ("stock", &self.stock),
            ("price", &self.price),
            ("shortage", &self.shortage),
        ] {
            h.section(name);
            column.hash_state(h);
        }
        h.section("stats");
        h.write_u32(self.stats.year as u32);
        h.write_i64(self.stats.population);
        self.stats.wealth.hash_state(h);
        self.stats.price_index.hash_state(h);
        h.write_u32(self.stats.markets_at_price_limit);
    }
}

/// Collects one value per region in parallel, in region order.
fn per_region<T: Send>(f: impl Fn(usize) -> T + Sync + Send) -> Vec<T> {
    (0..REGIONS as usize).into_par_iter().map(f).collect()
}

/// Daily productivity shock per region.
struct Weather;

impl System<World> for Weather {
    type Changes = Vec<Fixed>;
    fn name(&self) -> &'static str {
        "weather"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }
    fn access(&self) -> Access {
        Access::new().write(WEATHER)
    }
    fn compute(&self, ctx: &TickCtx, _world: &World) -> Vec<Fixed> {
        per_region(|r| Fixed::from_raw(850) + ctx.rng(r as u64).unit().mul_ratio(3, 10))
    }
    fn apply(&self, _ctx: &TickCtx, changes: Vec<Fixed>, world: &mut World) {
        world.weather.as_mut_slice().copy_from_slice(&changes);
    }
}

/// Wages and profits: the value of capacity at current prices, paid daily.
struct Income;

impl System<World> for Income {
    type Changes = Vec<Fixed>;
    fn name(&self) -> &'static str {
        "income"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }
    fn access(&self) -> Access {
        Access::new().read(CAPACITY).read(PRICE).write(WEALTH)
    }
    fn compute(&self, _ctx: &TickCtx, world: &World) -> Vec<Fixed> {
        let (capacity, price) = (world.capacity.as_slice(), world.price.as_slice());
        per_region(|r| {
            (0..GOODS)
                .map(|g| capacity[market(r, g)] * price[market(r, g)])
                .sum::<Fixed>()
        })
    }
    fn apply(&self, _ctx: &TickCtx, changes: Vec<Fixed>, world: &mut World) {
        for (wealth, income) in world.wealth.as_mut_slice().iter_mut().zip(changes) {
            *wealth += income;
        }
    }
}

/// Output = capacity × weather.
struct Production;

impl System<World> for Production {
    type Changes = Vec<[Fixed; GOODS]>;
    fn name(&self) -> &'static str {
        "production"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }
    fn access(&self) -> Access {
        Access::new().read(CAPACITY).read(WEATHER).write(STOCK)
    }
    fn compute(&self, _ctx: &TickCtx, world: &World) -> Self::Changes {
        let (capacity, weather) = (world.capacity.as_slice(), world.weather.as_slice());
        per_region(|r| core::array::from_fn(|g| capacity[market(r, g)] * weather[r]))
    }
    fn apply(&self, _ctx: &TickCtx, changes: Self::Changes, world: &mut World) {
        let stock = world.stock.as_mut_slice();
        for (r, output) in changes.into_iter().enumerate() {
            for (g, amount) in output.into_iter().enumerate() {
                stock[market(r, g)] += amount;
            }
        }
    }
}

/// People buy what they need and can afford; unmet need is recorded; 1% of stock spoils.
struct Consumption;

struct Purchases {
    stock: [Fixed; GOODS],
    shortage: [Fixed; GOODS],
    spent: Fixed,
}

impl System<World> for Consumption {
    type Changes = Vec<Purchases>;
    fn name(&self) -> &'static str {
        "consumption"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }
    fn access(&self) -> Access {
        Access::new()
            .read(POP)
            .read(PRICE)
            .read(STOCK)
            .read(WEALTH)
            .write(STOCK)
            .write(SHORTAGE)
            .write(WEALTH)
    }
    fn compute(&self, _ctx: &TickCtx, world: &World) -> Self::Changes {
        per_region(|r| {
            let people = world.population.as_slice()[r];
            // Spend at most a tenth of savings a day, split across goods by need value.
            let mut budget = world.wealth.as_slice()[r].mul_ratio(1, 10).max(Fixed::ZERO);
            let mut out = Purchases {
                stock: [Fixed::ZERO; GOODS],
                shortage: [Fixed::ZERO; GOODS],
                spent: Fixed::ZERO,
            };
            for g in 0..GOODS {
                let m = market(r, g);
                let price = world.price.as_slice()[m];
                let available = world.stock.as_slice()[m];
                let wanted = need(people, g);
                let affordable = budget.checked_div(price).unwrap_or(Fixed::ZERO);
                let bought = wanted.min(available).min(affordable).max(Fixed::ZERO);
                let cost = bought * price;
                budget -= cost;
                out.spent += cost;
                out.shortage[g] = wanted - bought;
                out.stock[g] = (available - bought).mul_ratio(99, 100);
            }
            out
        })
    }
    fn apply(&self, _ctx: &TickCtx, changes: Self::Changes, world: &mut World) {
        for (r, purchases) in changes.into_iter().enumerate() {
            world.wealth.as_mut_slice()[r] -= purchases.spent;
            for g in 0..GOODS {
                let m = market(r, g);
                world.stock.as_mut_slice()[m] = purchases.stock[g];
                world.shortage.as_mut_slice()[m] += purchases.shortage[g];
            }
        }
    }
}

/// Weekly price discovery: shortage raises the price, excess stock lowers it.
struct Prices;

impl System<World> for Prices {
    type Changes = Vec<[Fixed; GOODS]>;
    fn name(&self) -> &'static str {
        "prices"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Weekly(Weekday::Monday)
    }
    fn access(&self) -> Access {
        Access::new()
            .read(POP)
            .read(STOCK)
            .read(SHORTAGE)
            .read(PRICE)
            .write(PRICE)
            .write(SHORTAGE)
    }
    fn compute(&self, _ctx: &TickCtx, world: &World) -> Self::Changes {
        per_region(|r| {
            let people = world.population.as_slice()[r];
            core::array::from_fn(|g| {
                let m = market(r, g);
                let price = world.price.as_slice()[m];
                let weekly_need = need(people, g) * 7;
                if weekly_need == Fixed::ZERO {
                    return price;
                }
                let shortage = world.shortage.as_slice()[m];
                let stock = world.stock.as_slice()[m];
                let factor = if shortage > Fixed::ZERO {
                    // Up to +10% a week, in proportion to the unmet share of need.
                    Fixed::ONE
                        + shortage
                            .mul_div(Fixed::from_raw(100), weekly_need)
                            .min(Fixed::from_raw(100))
                } else if stock > weekly_need * 3 {
                    Fixed::from_raw(970)
                } else {
                    Fixed::ONE
                };
                (price * factor).clamp(PRICE_FLOOR, PRICE_CEILING)
            })
        })
    }
    fn apply(&self, _ctx: &TickCtx, changes: Self::Changes, world: &mut World) {
        for (r, prices) in changes.into_iter().enumerate() {
            for (g, price) in prices.into_iter().enumerate() {
                world.price.as_mut_slice()[market(r, g)] = price;
            }
        }
        world.shortage.as_mut_slice().fill(Fixed::ZERO);
    }
}

/// Monthly births and deaths; hunger (food shortage) raises deaths.
struct Demography;

impl System<World> for Demography {
    type Changes = Vec<i64>;
    fn name(&self) -> &'static str {
        "demography"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 1 }
    }
    fn access(&self) -> Access {
        Access::new().read(POP).read(SHORTAGE).write(POP)
    }
    fn compute(&self, ctx: &TickCtx, world: &World) -> Vec<i64> {
        per_region(|r| {
            let people = world.population.as_slice()[r];
            let food_need = need(people, 0) * 7;
            let hunger = if food_need > Fixed::ZERO {
                world.shortage.as_slice()[market(r, 0)]
                    .mul_div(Fixed::ONE, food_need)
                    .min(Fixed::ONE)
            } else {
                Fixed::ZERO
            };
            // Births 0.25%, deaths 0.15% a month, plus up to 1% more deaths from hunger, with
            // a per-mille of noise; rates are integer ratios per 100 000.
            let births = 250 + ctx.rng(r as u64).range(-20, 21);
            let deaths = 150 + hunger.mul_ratio(1_000, 1).to_int_round();
            let change = i128::from(people) * i128::from(births - deaths) / 100_000;
            (i128::from(people) + change).clamp(1_000, i128::from(i64::MAX)) as i64
        })
    }
    fn apply(&self, _ctx: &TickCtx, changes: Vec<i64>, world: &mut World) {
        world.population.as_mut_slice().copy_from_slice(&changes);
    }
}

/// Monthly investment: where a good sells above its base price, a region that can pay for it
/// adds 2% capacity (half the time, at random), at 60 days of output per unit; where it sells
/// below half its base price, 2% of the capacity is scrapped.
struct Investment;

struct Plan {
    capacity: [Fixed; GOODS],
    cost: Fixed,
}

impl System<World> for Investment {
    type Changes = Vec<Plan>;
    fn name(&self) -> &'static str {
        "investment"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 15 }
    }
    fn access(&self) -> Access {
        Access::new()
            .read(PRICE)
            .read(WEALTH)
            .read(CAPACITY)
            .write(CAPACITY)
            .write(WEALTH)
    }
    fn compute(&self, ctx: &TickCtx, world: &World) -> Vec<Plan> {
        per_region(|r| {
            let mut rng = ctx.rng(r as u64);
            let mut budget = world.wealth.as_slice()[r].mul_ratio(1, 4);
            let mut plan = Plan {
                capacity: [Fixed::ZERO; GOODS],
                cost: Fixed::ZERO,
            };
            for g in 0..GOODS {
                let m = market(r, g);
                let price = world.price.as_slice()[m];
                let capacity = world.capacity.as_slice()[m];
                let base = Fixed::from_int(BASE_PRICE[g]);
                plan.capacity[g] = if price > base && rng.chance(Fixed::from_raw(500)) {
                    let added = capacity.mul_ratio(2, 100);
                    let cost = added * price * 60;
                    if cost <= budget {
                        budget -= cost;
                        plan.cost += cost;
                        capacity + added
                    } else {
                        capacity
                    }
                } else if price < base / 2 {
                    capacity.mul_ratio(98, 100)
                } else {
                    capacity
                };
            }
            plan
        })
    }
    fn apply(&self, _ctx: &TickCtx, changes: Vec<Plan>, world: &mut World) {
        for (r, plan) in changes.into_iter().enumerate() {
            world.wealth.as_mut_slice()[r] -= plan.cost;
            for (g, capacity) in plan.capacity.into_iter().enumerate() {
                world.capacity.as_mut_slice()[market(r, g)] = capacity;
            }
        }
    }
}

/// Yearly world aggregates (what the report reads).
struct Statistics;

impl System<World> for Statistics {
    type Changes = WorldStats;
    fn name(&self) -> &'static str {
        "statistics"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Yearly { month: 1, day: 1 }
    }
    fn access(&self) -> Access {
        Access::new()
            .read(POP)
            .read(WEALTH)
            .read(PRICE)
            .write(STATS)
    }
    fn compute(&self, ctx: &TickCtx, world: &World) -> WorldStats {
        let prices = world.price.as_slice();
        let relative: Fixed = prices
            .iter()
            .enumerate()
            .map(|(m, &p)| p / BASE_PRICE[m % GOODS])
            .sum();
        WorldStats {
            year: ctx.date.year,
            population: world.population_total(),
            wealth: world.wealth.as_slice().iter().sum(),
            price_index: relative / prices.len() as i64,
            markets_at_price_limit: prices
                .iter()
                .filter(|&&p| p == PRICE_FLOOR || p == PRICE_CEILING)
                .count() as u32,
        }
    }
    fn apply(&self, _ctx: &TickCtx, changes: WorldStats, world: &mut World) {
        world.stats = changes;
    }
}

/// The toy schedule, in declared (apply) order.
pub fn schedule(threads: usize) -> Schedule<World> {
    let mut schedule = Schedule::new(threads);
    schedule
        .add(Weather)
        .add(Income)
        .add(Production)
        .add(Consumption)
        .add(Prices)
        .add(Demography)
        .add(Investment)
        .add(Statistics);
    schedule
}

/// Start date of the benchmark.
pub const START: Date = Date::new(1925, 1, 1);
