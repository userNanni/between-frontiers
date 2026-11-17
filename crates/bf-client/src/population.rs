//! What the population panel reads: the player's country's age pyramid (whole and by state),
//! its rates, why women have the children they have, and where its people came from and went,
//! copied out of the simulation once a month (`sim::Monthly`).

use std::collections::BTreeMap;

use bf_sim::demography::{AGE_GROUPS, FertilityTerm, TerritoryId};
use bf_sim::game::GameWorld;
use laplace_schelling::calendar::Date;

pub use bf_sim::demography::{Push, WaveEnd};

/// Roads listed in each direction.
pub const ROADS: usize = 6;
/// Ended waves listed.
pub const ENDED: usize = 4;

/// One term of the fertility score, in plain numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    pub key: &'static str,
    pub value: f64,
    pub weight: f64,
    pub contribution: f64,
}

impl From<&FertilityTerm> for Term {
    fn from(t: &FertilityTerm) -> Self {
        Self {
            key: t.key,
            value: t.value.to_f64_lossy(),
            weight: t.weight.to_f64_lossy(),
            contribution: t.contribution.to_f64_lossy(),
        }
    }
}

/// The rates of a territory (metropole or colonies), per thousand a year, of last month.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rates {
    pub births: f64,
    pub deaths: f64,
    pub migration: f64,
    pub life: f64,
    pub tfr: f64,
}

/// The metropole or the colonies of the country.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub colony: bool,
    pub people: i64,
    pub rates: Rates,
    pub terms: Vec<Term>,
}

/// A road of the diaspora, by country.
#[derive(Clone, Debug, PartialEq)]
pub struct Road {
    /// Tag of the other country.
    pub tag: String,
    /// The community: the 1925 stock, everyone who came since and their children.
    pub people: i64,
    /// Arrivals since 1925, without children.
    pub arrived: i64,
}

/// A wave of the country, in or out.
#[derive(Clone, Debug, PartialEq)]
pub struct WaveLine {
    pub from: String,
    pub to: String,
    /// Year and month it opened.
    pub since: (i32, u8),
    pub cause: Push,
    pub people: i64,
    /// For ended waves of one road: when and why the last one ended.
    pub ended: Option<((i32, u8), WaveEnd)>,
    /// Waves of the road it sums (1 for an open wave).
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PopulationView {
    pub month: (i32, u8),
    pub tag: String,
    /// The country's pyramid (metropole and colonies), people by age group.
    pub pyramid: [i64; AGE_GROUPS],
    /// By map state id: its name's index is the state, its pyramid.
    pub states: Vec<(u16, [i64; AGE_GROUPS])>,
    pub parts: Vec<Part>,
    /// Communities of foreign origin living here, by origin, largest first.
    pub immigrants: Vec<Road>,
    /// Communities from here living abroad, by destination, largest first.
    pub emigrants: Vec<Road>,
    pub waves: Vec<WaveLine>,
    /// Ended waves by road, the latest ended first: first opening to last end, all moved.
    pub ended: Vec<WaveLine>,
}

fn month_of(m: i32) -> (i32, u8) {
    (m.div_euclid(12), (m.rem_euclid(12) + 1) as u8)
}

/// Reads the player's country; `None` without a player or before the first demography month.
pub fn read(world: &GameWorld, date: Date) -> Option<PopulationView> {
    let tag = world.player_tag()?;
    let c = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    read_country(world, c, date)
}

/// Reads country `c` (index in the simulation's order).
pub fn read_country(world: &GameWorld, c: usize, date: Date) -> Option<PopulationView> {
    let d = &world.demography;
    let econ = &world.econ;
    if d.ages.is_empty() {
        return None;
    }
    let tag_of = |m: usize| {
        world
            .countries
            .iter()
            .nth(m)
            .map_or_else(|| "?".into(), |(_, c)| c.tag.clone())
    };
    let mut pyramid = [0i64; AGE_GROUPS];
    let mut states: Vec<(u16, [i64; AGE_GROUPS])> = Vec::new();
    for (r, ages) in econ.regions.iter().zip(&d.ages) {
        if usize::from(r.market) != c {
            continue;
        }
        for (p, a) in pyramid.iter_mut().zip(ages) {
            *p += a;
        }
        match states.iter_mut().find(|(s, _)| *s == r.state) {
            Some((_, sum)) => {
                for (p, a) in sum.iter_mut().zip(ages) {
                    *p += a;
                }
            }
            None => states.push((r.state, *ages)),
        }
    }
    states.sort_by_key(|(s, _)| *s);
    let parts = [false, true]
        .into_iter()
        .filter_map(|colony| {
            let v = d.territory_rates(econ, c as u16, colony)?;
            let people = econ
                .regions
                .iter()
                .filter(|r| usize::from(r.market) == c && r.colony == colony)
                .map(|r| r.population())
                .sum();
            Some(Part {
                colony,
                people,
                rates: Rates {
                    births: v.births.to_f64_lossy(),
                    deaths: v.deaths.to_f64_lossy(),
                    migration: v.migration.to_f64_lossy(),
                    life: v.life.to_f64_lossy(),
                    tfr: v.tfr.to_f64_lossy(),
                },
                terms: d
                    .fertility_breakdown(econ, c as u16, colony)
                    .iter()
                    .map(Term::from)
                    .collect(),
            })
        })
        .collect();
    // The diaspora by the other country, in and out (moves within the country do not count).
    let mut in_by: BTreeMap<usize, (i64, i64)> = BTreeMap::new();
    let mut out_by: BTreeMap<usize, (i64, i64)> = BTreeMap::new();
    for x in &d.diaspora {
        let (o, t) = (x.origin.market(), x.dest.market());
        let side = if t == c && o != c {
            in_by.entry(o)
        } else if o == c && t != c {
            out_by.entry(t)
        } else {
            continue;
        };
        let e = side.or_insert((0, 0));
        e.0 += x.people;
        e.1 += x.arrived;
    }
    let roads = |by: BTreeMap<usize, (i64, i64)>| -> Vec<Road> {
        let mut v: Vec<_> = by.into_iter().filter(|(_, (n, _))| *n > 0).collect();
        v.sort_by_key(|&(m, (n, _))| (-n, m));
        v.into_iter()
            .take(ROADS)
            .map(|(m, (people, arrived))| Road {
                tag: tag_of(m),
                people,
                arrived,
            })
            .collect()
    };
    let immigrants = roads(in_by);
    let emigrants = roads(out_by);
    let ours = |o: TerritoryId, t: TerritoryId| o.market() == c || t.market() == c;
    let waves = d
        .waves
        .iter()
        .filter(|w| ours(w.origin, w.dest))
        .map(|w| WaveLine {
            from: tag_of(w.origin.market()),
            to: tag_of(w.dest.market()),
            since: month_of(w.since),
            cause: w.cause,
            people: w.people,
            ended: None,
            count: 1,
        })
        .collect();
    // Ended waves by road (country to country): first opening, last end, everyone they moved.
    let mut by_road: Vec<((usize, usize), WaveLine, i32)> = Vec::new();
    for r in d
        .waves_ended
        .iter()
        .filter(|r| ours(r.wave.origin, r.wave.dest))
    {
        let road = (r.wave.origin.market(), r.wave.dest.market());
        match by_road.iter_mut().find(|(k, _, _)| *k == road) {
            Some((_, line, until)) => {
                line.people += r.wave.people;
                line.count += 1;
                if r.wave.since < (line.since.0 * 12 + i32::from(line.since.1) - 1) {
                    line.since = month_of(r.wave.since);
                    line.cause = r.wave.cause;
                }
                if r.until >= *until {
                    *until = r.until;
                    line.ended = Some((month_of(r.until), r.end));
                }
            }
            None => by_road.push((
                road,
                WaveLine {
                    from: tag_of(road.0),
                    to: tag_of(road.1),
                    since: month_of(r.wave.since),
                    cause: r.wave.cause,
                    people: r.wave.people,
                    ended: Some((month_of(r.until), r.end)),
                    count: 1,
                },
                r.until,
            )),
        }
    }
    by_road.sort_by_key(|(k, _, until)| (std::cmp::Reverse(*until), *k));
    let ended = by_road
        .into_iter()
        .take(ENDED)
        .map(|(_, line, _)| line)
        .collect();
    Some(PopulationView {
        month: (date.year, date.month),
        tag: tag_of(c),
        pyramid,
        states,
        parts,
        immigrants,
        emigrants,
        waves,
        ended,
    })
}
