//! The 1925 economy built from the scenario: regions, pops, plants and markets.
//!
//! Plants are sized by a world input-output balance: each block (a country's metropole, or its
//! colonies) has a final demand at its 1925 income (households, the state, investment);
//! manufacturing and services meet their own block's demand, while farm and mine goods meet the
//! world's demand, shared among blocks by their land and deposits. That is what makes Malaya
//! grow rubber for Detroit and Australia shear wool for Yorkshire on day one.

use laplace_schelling::Fixed;

use super::catalogue::Catalogue;
use super::prof::{self, P};
use super::state::{Economy, Market, Plant, Region, empty_pops};
use super::{WORKER_SHARE, milli, workers};
use crate::initial::{IndustryFile, PopulationFile};
use crate::resources::{DepositsFile, FARMS, LandFile, RESOURCES};
use crate::scenario::{CountryDef, MapFile};

pub struct Inputs<'a> {
    pub countries: &'a [CountryDef],
    pub map: &'a MapFile,
    pub deposits: Option<&'a DepositsFile>,
    pub land: Option<&'a LandFile>,
    pub population: Option<&'a PopulationFile>,
    pub industry: Option<&'a IndustryFile>,
}

/// Deposit units of `deposits.ron` to the catalogue's good units.
fn deposit_factor(resource: &str) -> f64 {
    match resource {
        "coal" | "iron_ore" | "oil" | "natural_gas" => 1.0e6,
        "tungsten" => 1.5,
        "uranium" => 1179.0,
        _ => 1.0e3,
    }
}

/// Land a farm level works, km² of cropland (pasture for ranches, forest for forestry).
/// Set from 1925 yields: a level of a grain farm harvests 6,000 t at 1.2 t/ha on 50 km²;
/// perennial and garden crops make far more per hectare.
fn land_per_level(farm: &str) -> f64 {
    match farm {
        "livestock_ranch" => 600.0,
        "forestry" => 150.0,
        "dairy_farm" => 80.0,
        "rice_paddy" => 25.0,
        "cotton_plantation" => 17.0,
        "coffee_plantation" | "tea_plantation" | "cocoa_plantation" | "rubber_plantation"
        | "tobacco_plantation" => 8.0,
        "fiber_plantation" => 7.0,
        "sugarcane_plantation" | "sugar_beet_farm" => 4.0,
        "fruit_plantation" | "vineyard" => 3.0,
        _ => 50.0,
    }
}

/// Shares of GDP the start assumes for the state's purchases and for investment; the weekly
/// market spends about the same from taxes and from the investment pool.
const STATE_GOODS_SHARE: f64 = 0.026;
const INVESTMENT_SHARE: f64 = 0.10;
/// Taxes as a share of GDP the start guesses for the treasury's first reserve.
const EXPECTED_REVENUE: f64 = 0.06;
/// Share of household consumption of each fifth of the population, poorest first.
const CONSUMPTION_BY_FIFTH: [f64; 5] = [0.06, 0.10, 0.15, 0.22, 0.47];

fn fx(v: f64) -> Fixed {
    Fixed::from_f64_lossy(v)
}

/// Largest-remainder split of `total` by weights.
fn split(total: i64, weights: &[f64]) -> Vec<i64> {
    let sum: f64 = weights.iter().sum();
    if total <= 0 || sum <= 0.0 {
        return vec![0; weights.len()];
    }
    let raw: Vec<f64> = weights.iter().map(|w| total as f64 * w / sum).collect();
    let mut out: Vec<i64> = raw.iter().map(|r| r.floor() as i64).collect();
    let mut rest = total - out.iter().sum::<i64>();
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by(|&a, &b| {
        (raw[b] - raw[b].floor())
            .total_cmp(&(raw[a] - raw[a].floor()))
            .then(a.cmp(&b))
    });
    for i in order {
        if rest == 0 {
            break;
        }
        out[i] += 1;
        rest -= 1;
    }
    out
}

fn yearly_output(cat: &Catalogue, kind: usize, good: usize) -> f64 {
    cat.buildings[kind].methods[0]
        .outputs
        .iter()
        .find(|(g, _)| *g as usize == good)
        .map_or(0.0, |(_, q)| q.to_f64_lossy() * 52.0)
}

fn best_yearly_output(cat: &Catalogue, kind: usize) -> f64 {
    cat.buildings[kind]
        .methods
        .iter()
        .filter_map(|m| m.outputs.first().map(|(_, q)| q.to_f64_lossy() * 52.0))
        .fold(0.0, f64::max)
}

struct Block {
    market: usize,
    regions: Vec<usize>,
    sizes: Vec<i64>,
    urban: Vec<f64>,
    population: u64,
    gdp_per_capita: f64,
    literacy: f64,
    /// Output per worker relative to the catalogue's 1925 baseline.
    tfp: f64,
    agr: f64,
    market_farming: f64,
    /// Weekly final demand by good.
    final_demand: Vec<f64>,
    /// Farm and mine capacity in levels, by building; mines also their 1925 levels.
    capacity: Vec<f64>,
    mines_1925: Vec<f64>,
}

/// (block, building, levels) a raw good is split into.
type Allocation = Vec<(usize, usize, f64)>;

pub fn build(cat: &Catalogue, inp: &Inputs) -> Economy {
    let ng = cat.goods.len();
    let nb = cat.buildings.len();
    let country_index = |tag: &str| inp.countries.iter().position(|c| c.tag == tag);
    let land_of = |key: &str| {
        inp.land
            .and_then(|l| l.states.iter().find(|s| s.state == key))
    };
    let deposits_of = |key: &str| {
        inp.deposits
            .and_then(|d| d.states.iter().find(|s| s.state == key))
    };
    let pop_of = |key: &str| {
        inp.population
            .and_then(|p| p.states.iter().find(|s| s.state == key))
    };
    let base: Vec<Fixed> = cat.goods.iter().map(|g| g.base_price).collect();
    let base_f: Vec<f64> = base.iter().map(|p| p.to_f64_lossy()).collect();
    let mine_of = |res: usize| {
        cat.buildings
            .iter()
            .position(|b| b.resource == Some(res as u8))
    };
    let farm_of = |farm: usize| {
        cat.buildings
            .iter()
            .position(|b| b.farm == Some(farm as u8))
    };

    // 1. Regions in map order, with their deposits and land in levels.
    let mut regions: Vec<Region> = Vec::new();
    let mut keys: Vec<(String, f64)> = Vec::new();
    for s in &inp.map.states {
        let Some(ci) = country_index(&s.owner) else {
            continue;
        };
        let mut deposits = Vec::new();
        for dep in deposits_of(&s.key).map_or(&[][..], |d| &d.deposits[..]) {
            let Some(res) = RESOURCES.iter().position(|r| *r == dep.resource) else {
                continue;
            };
            let Some(kind) = mine_of(res) else { continue };
            let per_level = best_yearly_output(cat, kind);
            let ceiling = if per_level > 0.0 {
                (f64::from(dep.potential) * deposit_factor(&dep.resource) / per_level).ceil() as i64
            } else {
                0
            };
            deposits.push((res as u8, dep.discovered, ceiling.max(1)));
        }
        let mut land = Vec::new();
        if let Some(l) = land_of(&s.key) {
            for (farm, suit) in &l.farms {
                let Some(fi) = FARMS.iter().position(|f| f == farm) else {
                    continue;
                };
                let ceiling = match farm.as_str() {
                    "fishery" => {
                        if l.coast {
                            (40.0 * f64::from(*suit)).ceil()
                        } else {
                            0.0
                        }
                    }
                    "livestock_ranch" => {
                        f64::from(l.pasture_km2) * f64::from(*suit) / land_per_level(farm)
                    }
                    "forestry" => f64::from(l.forest_km2) * f64::from(*suit) / land_per_level(farm),
                    _ => f64::from(l.cropland_km2) * f64::from(*suit) / land_per_level(farm),
                } as i64;
                if ceiling > 0 {
                    land.push((fi as u8, ceiling));
                }
            }
        }
        regions.push(Region {
            market: ci as u16,
            colony: s.colony,
            pops: empty_pops(),
            plants: Vec::new(),
            literacy: Fixed::ZERO,
            deposits,
            land,
            subsistence: Fixed::ZERO,
            tfp: Fixed::ONE,
            tfp_start: Fixed::ONE,
        });
        keys.push((s.key.clone(), f64::from(s.area_km2)));
    }

    // Staple food per 1,000 people per week, in tonnes.
    let staple = cat
        .needs
        .iter()
        .find(|n| n.key == "staple_food")
        .map_or(0.0, |n| n.per_capita.to_f64_lossy() / 1000.0);

    // 2. Blocks: population, labour structure, final demand, capacity.
    let mut blocks: Vec<Block> = Vec::new();
    for (ci, def) in inp.countries.iter().enumerate() {
        for colony in [false, true] {
            let idx: Vec<usize> = (0..regions.len())
                .filter(|&i| regions[i].market as usize == ci && regions[i].colony == colony)
                .collect();
            if idx.is_empty() {
                continue;
            }
            let (population, gdppc, lit, urb) = if colony {
                match &def.colonies {
                    Some(c) => (
                        c.population,
                        f64::from(c.gdp_per_capita),
                        c.literacy.unwrap_or(0.1),
                        c.urbanization.unwrap_or(0.1),
                    ),
                    None => continue,
                }
            } else {
                (
                    def.population,
                    f64::from(def.gdp_per_capita),
                    def.literacy.unwrap_or(0.3),
                    def.urbanization.unwrap_or(0.2),
                )
            };
            let has_census = idx.iter().all(|&i| pop_of(&keys[i].0).is_some());
            let sizes: Vec<i64> = if has_census {
                idx.iter()
                    .map(|&i| pop_of(&keys[i].0).map_or(0, |p| p.population as i64))
                    .collect()
            } else {
                let w: Vec<f64> = idx
                    .iter()
                    .map(|&i| {
                        land_of(&keys[i].0).map_or(0.0, |l| f64::from(l.cropland_km2))
                            + keys[i].1 * 0.02
                            + 1.0
                    })
                    .collect();
                split(population as i64, &w)
            };
            let urban: Vec<f64> = idx
                .iter()
                .map(|&i| pop_of(&keys[i].0).map_or(f64::from(urb), |p| f64::from(p.urban)))
                .collect();
            let workforce = inp.population.and_then(|p| {
                p.workforce
                    .iter()
                    .find(|w| w.country == def.tag && w.colonies == colony)
            });
            let agr = workforce.map_or_else(
                || (0.85 - gdppc / 12_000.0).clamp(0.08, 0.85),
                |w| f64::from(w.agriculture),
            );
            let market_farming = (gdppc / 6_000.0).clamp(0.2, 0.9);

            let gdp_week = population as f64 * gdppc / 52.0;
            let mut final_demand = vec![0.0f64; ng];
            let in_kind = staple * population as f64 / 1000.0 * agr * (1.0 - market_farming);
            // Households in five income fifths: the poor buy staples, the rich the rest. Each
            // fifth splits into town and country by the census share, for rent and shelter.
            let town = f64::from(urb).clamp(0.0, 1.0);
            for share in CONSUMPTION_BY_FIFTH {
                for (urban, part) in [(true, town), (false, 1.0 - town)] {
                    let plan = super::week::plan_pop_where(
                        cat,
                        &base,
                        1925,
                        (population as f64 / 5.0 * part) as i64,
                        fx(gdp_week * 0.70 * share * part),
                        fx(in_kind / 5.0 * part),
                        urban,
                    );
                    for (g, q) in plan {
                        final_demand[g as usize] += q.to_f64_lossy();
                    }
                }
            }
            // The state buys goods with 40% of its tax take; the rest pays its servants.
            for (key, per_mille) in super::week::STATE_BASKET {
                if let Some(g) = cat
                    .good(key)
                    .filter(|g| cat.goods[*g as usize].available_from <= 1925)
                {
                    final_demand[g as usize] += gdp_week * STATE_GOODS_SHARE * per_mille as f64
                        / 1000.0
                        / base_f[g as usize];
                }
            }
            // Schools and hospitals: enrolment follows literacy, hospital care income.
            for (key, per_person, cover) in [
                (
                    "schooling",
                    super::public::PUPILS_PER_PERSON,
                    f64::from(lit).clamp(0.05, 0.95),
                ),
                (
                    "hospital_care",
                    super::public::CARE_PER_PERSON,
                    (gdppc / 15_000.0).clamp(0.02, 0.5),
                ),
            ] {
                if let Some(g) = cat.good(key) {
                    final_demand[g as usize] +=
                        population as f64 * per_person.to_f64_lossy() * cover / 52.0;
                }
            }
            for (key, share) in super::week::UPKEEP_BASKET
                .iter()
                .map(|(k, s)| (*k, *s as f64 / 1000.0))
            {
                if let Some(g) = cat.good(key) {
                    final_demand[g as usize] +=
                        gdp_week * INVESTMENT_SHARE * share / base_f[g as usize];
                }
            }
            let mut capacity = vec![0.0f64; nb];
            let mut mines_1925 = vec![0.0f64; nb];
            for &ri in &idx {
                for (farm, ceiling) in &regions[ri].land {
                    if let Some(ki) = farm_of(*farm as usize) {
                        capacity[ki] += *ceiling as f64;
                    }
                }
                for (res, discovered, ceiling) in &regions[ri].deposits {
                    if *discovered <= 1925
                        && let Some(ki) = mine_of(*res as usize)
                    {
                        // Ceilings count levels of the best method; 1925 mines use the first.
                        let first = cat.buildings[ki].methods[0]
                            .outputs
                            .first()
                            .map_or(0, |(g, _)| *g as usize);
                        let ratio =
                            best_yearly_output(cat, ki) / yearly_output(cat, ki, first).max(1.0);
                        capacity[ki] += *ceiling as f64 * ratio;
                    }
                }
                for dep in deposits_of(&keys[ri].0).map_or(&[][..], |d| &d.deposits[..]) {
                    if dep.discovered > 1925 || dep.output_1925 <= 0.0 {
                        continue;
                    }
                    let Some(res) = RESOURCES.iter().position(|x| *x == dep.resource) else {
                        continue;
                    };
                    let Some(ki) = mine_of(res) else { continue };
                    let first = cat.buildings[ki].methods[0]
                        .outputs
                        .first()
                        .map_or(0, |(g, _)| *g as usize);
                    let per_level = yearly_output(cat, ki, first).max(1.0);
                    mines_1925[ki] +=
                        f64::from(dep.output_1925) * deposit_factor(&dep.resource) / per_level;
                    let _ = gdppc;
                }
            }
            for ki in 0..nb {
                capacity[ki] = capacity[ki].max(mines_1925[ki]);
            }
            // The catalogue's methods make about $6,000 a worker. The GDP not grown by peasants
            // for themselves is made by the market's workers: that sets the gap.
            let grain_price = cat.good("grain").map_or(700.0, |g| base_f[g as usize]);
            let subsistence = staple / 1000.0 * 52.0 * agr * (1.0 - market_farming) * grain_price;
            let market_workers = WORKER_SHARE.to_f64_lossy() * (1.0 - agr * (1.0 - market_farming));
            let tfp = ((gdppc - subsistence).max(gdppc * 0.3) / (market_workers * 6_000.0))
                .clamp(0.35, 8.0);
            blocks.push(Block {
                tfp,
                market: ci,
                regions: idx,
                sizes,
                urban,
                population,
                gdp_per_capita: gdppc,
                literacy: f64::from(lit),
                agr,
                market_farming,
                final_demand,
                capacity,
                mines_1925,
            });
        }
    }

    // 3. Who makes each good: raw goods (farms, mines) world-wide by capacity; the rest locally,
    //    by the first plant whose first method has it as its main output.
    let raw_producers: Vec<Vec<usize>> = (0..ng)
        .map(|g| {
            cat.buildings
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    (b.farm.is_some() || b.resource.is_some())
                        && b.methods[0].outputs.iter().any(|(o, _)| *o as usize == g)
                })
                .map(|(k, _)| k)
                .collect()
        })
        .collect();
    let local_producer: Vec<Option<usize>> = (0..ng)
        .map(|g| {
            if !raw_producers[g].is_empty() {
                return None;
            }
            cat.buildings.iter().position(|b| {
                b.farm.is_none()
                    && b.resource.is_none()
                    && b.methods[0]
                        .outputs
                        .first()
                        .is_some_and(|(o, _)| *o as usize == g)
            })
        })
        .collect();

    // 4. World input-output balance by fixed-point iteration.
    let nbk = blocks.len();
    // Workers each block can put on commercial farms and in mines.
    let farm_cap: Vec<f64> = blocks
        .iter()
        .map(|b| b.sizes.iter().map(|&s| workers(s)).sum::<i64>() as f64 * b.agr * b.market_farming)
        .collect();
    let mine_cap: Vec<f64> = blocks
        .iter()
        .map(|b| b.sizes.iter().map(|&s| workers(s)).sum::<i64>() as f64 * 0.05)
        .collect();
    let mut lv = vec![vec![0.0f64; nb]; nbk];
    for _ in 0..16 {
        let mut demand: Vec<Vec<f64>> = blocks.iter().map(|b| b.final_demand.clone()).collect();
        for (bi, levels) in lv.iter().enumerate() {
            for (ki, n) in levels.iter().enumerate() {
                if *n > 0.0 {
                    for (g, q) in &cat.buildings[ki].methods[0].inputs {
                        demand[bi][*g as usize] += n * q.to_f64_lossy() * blocks[bi].tfp;
                    }
                }
            }
        }
        let mut next = vec![vec![0.0f64; nb]; nbk];
        let mut raw_alloc: Vec<(f64, Allocation)> = vec![(0.0, Vec::new()); ng];
        let mut residual = vec![0.0f64; ng];
        for g in 0..ng {
            if let Some(ki) = local_producer[g] {
                let per_level = yearly_output(cat, ki, g) / 52.0;
                if per_level > 0.0 {
                    for bi in 0..nbk {
                        next[bi][ki] =
                            f64::max(next[bi][ki], demand[bi][g] / (per_level * blocks[bi].tfp));
                    }
                }
                continue;
            }
            if raw_producers[g].is_empty() {
                continue;
            }
            let world: f64 = demand.iter().map(|d| d[g]).sum();
            // Shares by capacity, mines weighted toward what they already produced in 1925.
            let mut weights = Vec::new();
            for (bi, block) in blocks.iter().enumerate() {
                for &ki in &raw_producers[g] {
                    let cap = block.capacity[ki];
                    if cap > 0.0 {
                        let w = if cat.buildings[ki].resource.is_some() {
                            block.mines_1925[ki] + 0.1 * cap
                        } else {
                            cap
                        };
                        weights.push((bi, ki, w));
                    }
                }
            }
            raw_alloc[g] = (world, weights);
        }
        // Raw goods: shares by capacity; then each block's farms and mines are scaled to the
        // people it has for them, all goods alike; what that leaves unmet goes to blocks with
        // land and people to spare.
        let place = |next: &mut Vec<Vec<f64>>,
                     g: usize,
                     amount: f64,
                     weights: &[(usize, usize, f64)],
                     limit: &dyn Fn(usize, usize) -> f64|
         -> f64 {
            let total: f64 = weights.iter().map(|w| w.2).sum();
            if total <= 0.0 || amount <= 0.0 {
                return 0.0;
            }
            let mut placed = 0.0;
            for (bi, ki, w) in weights {
                let per_level = yearly_output(cat, *ki, g) / 52.0 * blocks[*bi].tfp;
                if per_level <= 0.0 {
                    continue;
                }
                let want = (amount * w / total / per_level).min(limit(*bi, *ki));
                if want > 0.0 {
                    next[*bi][*ki] += want;
                    placed += want * per_level;
                }
            }
            placed
        };
        for g in 0..ng {
            let (world, weights) = &raw_alloc[g];
            let placed = place(&mut next, g, *world, weights, &|bi, ki| {
                let floor = blocks[bi].mines_1925[ki] / blocks[bi].tfp;
                (blocks[bi].capacity[ki] / blocks[bi].tfp).max(floor)
            });
            residual[g] = (world - placed).max(0.0);
        }
        let used = |next: &Vec<Vec<f64>>, bi: usize, farms: bool| -> f64 {
            next[bi]
                .iter()
                .enumerate()
                .filter(|(ki, _)| {
                    if farms {
                        cat.buildings[*ki].farm.is_some()
                    } else {
                        cat.buildings[*ki].resource.is_some()
                    }
                })
                .map(|(ki, n)| n * cat.buildings[ki].methods[0].workers as f64)
                .sum()
        };
        for bi in 0..nbk {
            for farms in [true, false] {
                let cap = if farms { farm_cap[bi] } else { mine_cap[bi] };
                let u = used(&next, bi, farms);
                if u > cap && u > 0.0 {
                    let k = cap / u;
                    for ki in 0..nb {
                        let is = if farms {
                            cat.buildings[ki].farm.is_some()
                        } else {
                            cat.buildings[ki].resource.is_some()
                        };
                        if !is {
                            continue;
                        }
                        let floor = if farms {
                            0.0
                        } else {
                            blocks[bi].mines_1925[ki] / blocks[bi].tfp
                        };
                        let before = next[bi][ki];
                        let after = (before * k).max(floor.min(before));
                        for (g, q) in &cat.buildings[ki].methods[0].outputs {
                            residual[*g as usize] +=
                                (before - after) * q.to_f64_lossy() * blocks[bi].tfp;
                        }
                        next[bi][ki] = after;
                    }
                }
            }
        }
        for g in 0..ng {
            if residual[g] <= 0.0 {
                continue;
            }
            let (_, weights) = &raw_alloc[g];
            let spare: Vec<(usize, usize, f64)> = weights
                .iter()
                .map(|(bi, ki, w)| {
                    let farms = cat.buildings[*ki].farm.is_some();
                    let cap = if farms { farm_cap[*bi] } else { mine_cap[*bi] };
                    let room = (cap - used(&next, *bi, farms)).max(0.0);
                    (*bi, *ki, w * room)
                })
                .collect();
            let snapshot = next.clone();
            place(&mut next, g, residual[g], &spare, &|bi, ki| {
                let farms = cat.buildings[ki].farm.is_some();
                let cap = if farms { farm_cap[bi] } else { mine_cap[bi] };
                let room = (cap - used(&snapshot, bi, farms)).max(0.0)
                    / cat.buildings[ki].methods[0].workers.max(1) as f64;
                room.min((blocks[bi].capacity[ki] / blocks[bi].tfp - snapshot[bi][ki]).max(0.0))
            });
        }
        lv = next;
    }

    if std::env::var_os("BF_SETUP_DEBUG").is_some() {
        let mut demand: Vec<f64> = vec![0.0; ng];
        for (bi, b) in blocks.iter().enumerate() {
            for g in 0..ng {
                demand[g] += b.final_demand[g];
            }
            for (ki, n) in lv[bi].iter().enumerate() {
                for (g, q) in &cat.buildings[ki].methods[0].inputs {
                    demand[*g as usize] += n * q.to_f64_lossy() * b.tfp;
                }
            }
        }
        for key in [
            "natural_rubber",
            "cotton",
            "iron_ore",
            "coal",
            "steel",
            "lead_zinc_ore",
            "grain",
        ] {
            let Some(g) = cat.good(key) else { continue };
            let g = g as usize;
            let mut supply = 0.0;
            let mut cap = 0.0;
            for (bi, b) in blocks.iter().enumerate() {
                for (ki, n) in lv[bi].iter().enumerate() {
                    let per = yearly_output(cat, ki, g) / 52.0 * b.tfp;
                    supply += n * per;
                    cap += b.capacity[ki] * yearly_output(cat, ki, g) / 52.0;
                }
            }
            eprintln!(
                "solve {key}: demand/wk {:.0}, planned supply/wk {:.0}, capacity/wk {:.0}",
                demand[g], supply, cap
            );
        }
    }

    // 5. Fit each block into its workforce and lay the plants and people on its regions.
    let housing_kind = cat.building("urban_housing");
    for (bi, b) in blocks.iter().enumerate() {
        let total_workers: i64 = b.sizes.iter().map(|&s| workers(s)).sum();
        let reserved = (total_workers as f64 * b.agr * (1.0 - b.market_farming)) as i64;
        let room = (total_workers - reserved).max(0) as f64;
        // Farms and mines keep their size; manufacturing and services give way if people lack.
        let is_raw =
            |ki: usize| cat.buildings[ki].farm.is_some() || cat.buildings[ki].resource.is_some();
        let need_raw: f64 = lv[bi]
            .iter()
            .enumerate()
            .filter(|(ki, _)| is_raw(*ki))
            .map(|(ki, n)| n * cat.buildings[ki].methods[0].workers as f64)
            .sum();
        let need_other: f64 = lv[bi]
            .iter()
            .enumerate()
            .filter(|(ki, _)| !is_raw(*ki))
            .map(|(ki, n)| n * cat.buildings[ki].methods[0].workers as f64)
            .sum();
        let left = (room * 0.95 - need_raw).max(0.0);
        let k_other = if need_other > left {
            left / need_other
        } else {
            1.0
        };
        if std::env::var_os("BF_SETUP_DEBUG").is_some() {
            eprintln!(
                "block {} colony {}: workers {:.1} M, reserved {:.1} M, raw need {:.1} M, other need {:.1} M, k_other {:.2}, tfp {:.2}",
                inp.countries[b.market].tag,
                regions[b.regions[0]].colony,
                total_workers as f64 / 1e6,
                reserved as f64 / 1e6,
                need_raw / 1e6,
                need_other / 1e6,
                k_other,
                b.tfp
            );
        }
        let urban_weight: Vec<f64> = b
            .sizes
            .iter()
            .zip(&b.urban)
            .map(|(s, u)| *s as f64 * u.max(0.02))
            .collect();
        for (ki, n) in lv[bi].iter().enumerate() {
            let k = if is_raw(ki) { 1.0 } else { k_other };
            let levels = (n * k).round() as i64;
            if levels <= 0 {
                continue;
            }
            let bd = &cat.buildings[ki];
            let w: Vec<f64> = b
                .regions
                .iter()
                .zip(&urban_weight)
                .map(|(&ri, w)| {
                    if let Some(farm) = bd.farm {
                        regions[ri]
                            .land
                            .iter()
                            .filter(|l| l.0 == farm)
                            .map(|l| l.1 as f64)
                            .sum()
                    } else if let Some(res) = bd.resource {
                        regions[ri]
                            .deposits
                            .iter()
                            .filter(|d| d.0 == res && d.1 <= 1925)
                            .map(|d| d.2 as f64)
                            .sum()
                    } else if bd.key == "port" && !land_of(&keys[ri].0).is_some_and(|l| l.coast) {
                        0.0
                    } else {
                        *w
                    }
                })
                .collect();
            for (&ri, n) in b.regions.iter().zip(split(levels, &w)) {
                if n > 0 {
                    let p: &mut Plant = regions[ri].plant_mut(ki as u16);
                    p.levels += n;
                }
            }
        }
        for (pos, &ri) in b.regions.iter().enumerate() {
            let r = &mut regions[ri];
            let size = b.sizes[pos];
            r.literacy = fx(b.literacy);
            r.tfp = fx(b.tfp);
            r.tfp_start = r.tfp;
            let cropland = land_of(&keys[ri].0).map_or(0.0, |l| f64::from(l.cropland_km2));
            let per_head_ha = if size > 0 {
                cropland * 100.0 / size as f64
            } else {
                0.0
            };
            r.subsistence = fx(staple * (per_head_ha / 0.3).clamp(0.4, 1.0) * 0.9);
            let mut jobs = [0i64; P];
            for p in &r.plants {
                for (pi, n) in &cat.buildings[p.kind as usize].methods[0].jobs {
                    jobs[*pi as usize] += *n * p.levels;
                }
            }
            let mut people: Vec<i64> = jobs
                .iter()
                .map(|w| (Fixed::from_int(*w) / WORKER_SHARE).to_int_floor())
                .collect();
            let used: i64 = people.iter().sum();
            if used > size * 9 / 10 && used > 0 {
                // More jobs than people here: thin every plant to fit.
                let k = Fixed::from_ratio(size * 9 / 10, used);
                for p in &mut r.plants {
                    p.levels = (Fixed::from_int(p.levels) * k).to_int_floor();
                }
                r.plants.retain(|p| p.levels > 0);
                jobs = [0; P];
                for p in &r.plants {
                    for (pi, n) in &cat.buildings[p.kind as usize].methods[0].jobs {
                        jobs[*pi as usize] += *n * p.levels;
                    }
                }
                people = jobs
                    .iter()
                    .map(|w| (Fixed::from_int(*w) / WORKER_SHARE).to_int_floor())
                    .collect();
            }
            for (pi, share) in [
                (prof::CIVIL_SERVANT, 12),
                (prof::SOLDIER, 10),
                (prof::CLERGY, 5),
            ] {
                people[pi] += size * share / 1000;
            }
            let used: i64 = people.iter().sum();
            people[prof::PEASANT] += (size - used).max(0);
            for (pi, pop) in r.pops.iter_mut().enumerate() {
                pop.size = people[pi];
                pop.employed = workers(people[pi]).min(jobs[pi]);
                pop.satisfaction = [milli(800); 4];
            }
            // City housing for every town dweller (by profession), whatever the census said.
            if let Some(hk) = housing_kind {
                let per = super::migration::housing_per_level(cat, r, hk);
                if per > Fixed::ZERO {
                    let urban = Fixed::from_int(super::migration::urban_population(r));
                    let levels = (urban / per).to_int_floor() + 1;
                    r.plant_mut(hk as u16).levels = levels;
                }
            }
            for p in &mut r.plants {
                p.staffing = Fixed::ONE;
            }
        }
    }

    // 6. Markets and the money people start with.
    let mut markets = Vec::new();
    for ci in 0..inp.countries.len() {
        let target_gdp: f64 = blocks
            .iter()
            .filter(|b| b.market == ci)
            .map(|b| b.population as f64 * b.gdp_per_capita)
            .sum();
        let weekly = target_gdp / 52.0;
        let pop_total: i64 = regions
            .iter()
            .filter(|r| r.market as usize == ci)
            .map(|r| r.population())
            .sum();
        for r in regions.iter_mut().filter(|r| r.market as usize == ci) {
            for (pi, pop) in r.pops.iter_mut().enumerate() {
                if pop_total == 0 || pop.size == 0 {
                    continue;
                }
                let share = pop.size as f64 / pop_total as f64;
                let income = weekly * share * 0.6 * super::WAGE_WEIGHT[pi].to_f64_lossy();
                pop.income = fx(income);
                pop.savings = fx(income * 6.0);
            }
        }
        // The treasury starts at its reserve, at a revenue guessed from the 1925 tax rates.
        let revenue = fx(target_gdp * EXPECTED_REVENUE / 52.0);
        markets.push(Market {
            price: base.clone(),
            supply: vec![Fixed::ZERO; ng],
            demand: vec![Fixed::ZERO; ng],
            buy_ratio: vec![Fixed::ONE; ng],
            sell_ratio: vec![Fixed::ONE; ng],
            net_import: vec![Fixed::ZERO; ng],
            // A pool that spends its weekly upkeep share at the rate investment flows in.
            pool: fx(target_gdp * INVESTMENT_SHARE
                / 52.0
                / super::week::UPKEEP_SHARE.to_f64_lossy()),
            treasury: revenue * super::public::RESERVE_WEEKS,
            tfp: Fixed::ONE,
            construction: vec![Fixed::ZERO; ng],
            gdp_running: Fixed::ZERO,
            gdp_last_year: fx(target_gdp),
            tech_tfp: Vec::new(),
            gov_workers: 0,
            queue: Vec::new(),
            laws: Default::default(),
            budget: super::state::Budget {
                revenue_avg: revenue,
                ..Default::default()
            },
        });
    }
    Economy {
        regions,
        markets,
        world_price: base,
        history: Vec::new(),
    }
}
