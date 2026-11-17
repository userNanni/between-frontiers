//! Internal migration: people leave the land for the city when town pay beats country pay by
//! enough, within their state and toward the best-paying cities of their country, as far as
//! city housing allows. See `docs/state.pt-BR.md` ("Migração interna").

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use super::catalogue::Catalogue;
use super::prof;
use super::state::{Economy, Region};
use super::{WORKER_SHARE, milli, workers};

/// The country's law on moving house: open, or restricted by internal passports (Soviet
/// propiska, Chinese hukou, pass laws).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationLaw {
    #[default]
    Open,
    Restricted,
}

impl MigrationLaw {
    /// Share of the open flows the law lets through, within a state and between states.
    fn reach(self) -> (Fixed, Fixed) {
        match self {
            MigrationLaw::Open => (Fixed::ONE, Fixed::ONE),
            MigrationLaw::Restricted => (milli(250), milli(100)),
        }
    }
}

/// Whether a profession lives in town (the definition of `Region::urban_share`).
pub const fn is_urban(pi: usize) -> bool {
    !matches!(pi, prof::PEASANT | prof::FARM_LABORER | prof::LANDOWNER)
}

/// Housing a city dweller rents in a year (the `urban_housing` need).
pub const HOUSING_PER_PERSON: Fixed = Fixed::ONE;
/// Slums and crowding: a city holds this many people per housing place before it pushes the
/// jobless back to the land.
pub const CROWDING: Fixed = milli(1250);
/// Town pay per head must beat country pay per head by this factor before anyone moves.
const PREMIUM: Fixed = milli(1500);
/// Monthly share of the idle peasants, per 10,000, who move when town pay beats country pay by
/// twice the premium: to the state's own towns, and to the best cities of the country. Town pay
/// is the labourers' income per head, jobless included: the expected wage, the urban wage times
/// the chance of finding work (Harris–Todaro). At full pull, 1.7 % of the idle peasants a year,
/// about 0.8 % of the rural population: net rural–urban migration in developing countries after
/// 1960 was 1.5 % of the urban population a year (Jedwab, Christiaensen and Gindelsky 2017,
/// *Journal of Urban Economics* 98, table A.3), or 0.4 % (1960) to 1.0 % (2000) of the rural
/// population.
const LOCAL_RATE: i64 = 10;
const DISTANT_RATE: i64 = 4;
/// Cities a country's migrants head for each month.
const DESTINATIONS: usize = 3;

pub fn urban_population(r: &Region) -> i64 {
    r.pops
        .iter()
        .enumerate()
        .filter(|(pi, _)| is_urban(*pi))
        .map(|(_, p)| p.size)
        .sum()
}

/// People one housing level shelters here.
pub fn housing_per_level(cat: &Catalogue, r: &Region, kind: usize) -> Fixed {
    super::public::level_output(cat, r, kind, 0) / HOUSING_PER_PERSON
}

/// People the region's city housing shelters.
pub fn housing_capacity(cat: &Catalogue, r: &Region, kind: Option<usize>) -> Fixed {
    let Some(kind) = kind else {
        return Fixed::MAX;
    };
    let levels: i64 = r
        .plants
        .iter()
        .filter(|p| p.kind as usize == kind)
        .map(|p| p.levels)
        .sum();
    housing_per_level(cat, r, kind) * levels
}

/// City dwellers without a housing place.
pub fn housing_gap(cat: &Catalogue, r: &Region, kind: usize) -> Fixed {
    Fixed::from_int(urban_population(r)) - housing_capacity(cat, r, Some(kind))
}

/// Room for newcomers before the city is overcrowded (negative when it already is).
pub fn room(cat: &Catalogue, r: &Region, kind: Option<usize>) -> i64 {
    if kind.is_none() {
        return i64::MAX / 4;
    }
    (housing_capacity(cat, r, kind) * CROWDING).to_int_floor() - urban_population(r)
}

/// Weekly income per head of the town's working people and of the country's, each with the
/// food the jobless grow (shared by everyone, as the market shares it) at the base price.
fn pay(cat: &Catalogue, r: &Region, m: &super::state::Market) -> (Fixed, Fixed) {
    let in_kind = cat.grain.map_or(Fixed::ZERO, |g| {
        super::week::subsistence_food(cat, r, m) * cat.goods[g as usize].base_price
            / r.population().max(1)
    });
    let per_head = |ps: &[usize]| -> Fixed {
        let size: i64 = ps.iter().map(|&p| r.pops[p].size).sum();
        if size <= 0 {
            return Fixed::ZERO;
        }
        ps.iter().map(|&p| r.pops[p].income).sum::<Fixed>() / size + in_kind
    };
    (
        per_head(&[prof::LABORER]),
        per_head(&[prof::PEASANT, prof::FARM_LABORER]),
    )
}

/// How much more town pays, 0 below the premium, 1 at twice the premium and above.
fn pull(town: Fixed, country: Fixed) -> Fixed {
    if country <= Fixed::ZERO {
        return if town > Fixed::ZERO {
            Fixed::ONE
        } else {
            Fixed::ZERO
        };
    }
    let ratio = town / (country * PREMIUM);
    (ratio - Fixed::ONE).clamp(Fixed::ZERO, Fixed::ONE)
}

/// This month's moves of one region: idle peasants who leave (to their own towns or to other
/// states), labourers who arrive from other states, and whether the city is pushing its jobless
/// back to the land.
#[derive(Clone, Debug, Default)]
pub struct Moves {
    pub to_town: i64,
    pub leaving: i64,
    pub arriving: i64,
    /// Town pays no better than the country, or the city is overcrowded.
    pub push_back: bool,
    /// People beyond the crowding limit.
    pub crowded: i64,
}

/// Plans the month's migration of every region, country by country.
pub fn plan(cat: &Catalogue, econ: &Economy) -> Vec<Moves> {
    let housing = cat.building("urban_housing");
    let n = econ.regions.len();
    let mut moves = vec![Moves::default(); n];
    let mut rooms = vec![0i64; n];
    let mut pays = vec![(Fixed::ZERO, Fixed::ZERO); n];
    let mut idle = vec![0i64; n];
    for (ri, r) in econ.regions.iter().enumerate() {
        rooms[ri] = room(cat, r, housing);
        pays[ri] = pay(cat, r, &econ.markets[r.market as usize]);
        let p = &r.pops[prof::PEASANT];
        let idle_workers = (workers(p.size) - p.employed).max(0);
        idle[ri] = (Fixed::from_int(idle_workers) / WORKER_SHARE)
            .to_int_floor()
            .min(p.size);
        let (town, country) = pays[ri];
        moves[ri].crowded = (-rooms[ri]).max(0);
        moves[ri].push_back = rooms[ri] < 0 || town <= country;
    }
    let mut by_market: Vec<Vec<usize>> = vec![Vec::new(); econ.markets.len()];
    for (ri, r) in econ.regions.iter().enumerate() {
        by_market[r.market as usize].push(ri);
    }
    for (mi, regions) in by_market.iter().enumerate() {
        let laws = &econ.markets[mi].laws;
        let (local, distant) = laws.migration.reach();
        // Within the state first.
        for &ri in regions {
            let (town, country) = pays[ri];
            let local = local * laws.of(&econ.regions[ri]).migration_reach[0];
            let want = (Fixed::from_int(idle[ri]) * local * pull(town, country))
                .mul_ratio(LOCAL_RATE, 10_000)
                .to_int_floor();
            let n = want.min(rooms[ri].max(0)).min(idle[ri]);
            if n > 0 {
                moves[ri].to_town = n;
                rooms[ri] -= n;
                idle[ri] -= n;
            }
        }
        // Then toward the best-paying cities of the country with room to spare.
        let mut dest: Vec<usize> = regions
            .iter()
            .copied()
            .filter(|&ri| rooms[ri] > 0 && pays[ri].0 > Fixed::ZERO)
            .collect();
        dest.sort_by(|&a, &b| pays[b].0.cmp(&pays[a].0).then(a.cmp(&b)));
        dest.truncate(DESTINATIONS);
        for &ri in regions {
            for &di in &dest {
                if di == ri || idle[ri] <= 0 || rooms[di] <= 0 {
                    continue;
                }
                let distant = distant * laws.of(&econ.regions[ri]).migration_reach[1];
                let want = (Fixed::from_int(idle[ri]) * distant * pull(pays[di].0, pays[ri].1))
                    .mul_ratio(DISTANT_RATE, 10_000 * DESTINATIONS as i64)
                    .to_int_floor();
                let n = want.min(rooms[di]).min(idle[ri]);
                if n > 0 {
                    moves[ri].leaving += n;
                    moves[di].arriving += n;
                    rooms[di] -= n;
                    idle[ri] -= n;
                }
            }
        }
    }
    moves
}

/// Applies a region's moves to its (size, employed) table before the month's labour.
pub fn apply(pops: &mut [(i64, i64)], m: &Moves) {
    let out = (m.to_town + m.leaving).min(pops[prof::PEASANT].0);
    pops[prof::PEASANT].0 -= out;
    pops[prof::LABORER].0 += m.to_town.min(out) + m.arriving;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::prof::P;

    #[test]
    fn town_pulls_only_past_the_premium() {
        let c = Fixed::from_int(10);
        assert_eq!(pull(Fixed::from_int(14), c), Fixed::ZERO);
        assert_eq!(pull(Fixed::from_int(15), c), Fixed::ZERO);
        assert!(pull(Fixed::from_int(20), c) > Fixed::ZERO);
        assert_eq!(pull(Fixed::from_int(30), c), Fixed::ONE);
        assert_eq!(pull(Fixed::from_int(100), c), Fixed::ONE);
        assert_eq!(pull(Fixed::from_int(1), Fixed::ZERO), Fixed::ONE);
    }

    #[test]
    fn moving_keeps_everyone() {
        let mut pops = vec![(0i64, 0i64); P];
        pops[prof::PEASANT].0 = 1000;
        pops[prof::LABORER].0 = 100;
        apply(
            &mut pops,
            &Moves {
                to_town: 50,
                leaving: 30,
                arriving: 0,
                ..Default::default()
            },
        );
        assert_eq!(pops[prof::PEASANT].0, 920);
        assert_eq!(pops[prof::LABORER].0, 150);
    }
}
