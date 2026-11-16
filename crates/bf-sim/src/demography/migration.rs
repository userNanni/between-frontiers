//! Migration between countries: a steady flow by income gap, the pull of the diaspora already
//! abroad (chain migration), and mass waves when the pressure to leave a country passes a
//! threshold. Rules in `docs/population.pt-BR.md` ("Migração entre países").
//!
//! The unit is the territory: a country's metropole or its colonies (`market * 2 + colony`).

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::{RegionMonth, Territory};
use crate::econ::milli;
use crate::game::GameWorld;

/// Income ratio (destination over origin) below which nobody moves, and the extra ratio at
/// which the pull is full.
const GAP_START: Fixed = milli(1500);
const GAP_FULL: Fixed = milli(3000);
/// Most of a territory's people who emigrate in a year in the steady flow, at the full pull.
const EMIGRATION_RATE: i64 = 3; // per mille a year
/// Most immigrants a territory takes in a year from the steady flow, per mille of its people.
const IMMIGRATION_CAP: i64 = 4;
/// Distance at which the pull halves, in km.
const DISTANCE_KM: i64 = 8000;
/// Pull multiplier within a language family and within an empire. The empire gets none: at 2,
/// Belgium, the Netherlands and Portugal sat at the immigration cap for sixty years, while the
/// real colonial flows stayed small until the 1960s.
const SAME_FAMILY: i64 = 2;
const SAME_EMPIRE: i64 = 1;
/// Share of the people who can pay for the passage at no literacy; it grows with literacy
/// squared (who reads hears of the jobs abroad and saves for the ticket).
const AFFORD_BASE: Fixed = milli(50);

/// Land per farmer (the land's ceiling in farm levels over peasants and farm labourers): a
/// destination with `LAND_START` times the origin's starts to pull, with `LAND_START +
/// LAND_FULL` times pulls fully, if it is not poorer than `LAND_MIN_RATIO` of the origin's
/// income (the frontiers of the Americas, Siberia, Australia).
const LAND_START: Fixed = milli(2000);
const LAND_FULL: Fixed = milli(4000);
const LAND_MIN_RATIO: Fixed = milli(300);
/// The diaspora pulls by itself (letters, tickets, a job waiting): at full chain it adds this
/// to the gap, toward a destination at least `NETWORK_MIN_RATIO` of the origin's income.
const NETWORK: Fixed = milli(500);
const NETWORK_MIN_RATIO: Fixed = milli(200);
/// A wave needs at least this pull toward its destination, and a road already open: pioneers
/// there (`WAVE_MIN_CHAIN` of the full chain, 0.5 % of the origin's people) or a passage a law
/// pays. Mass migrations follow the few who went first.
const WAVE_MIN_PULL: Fixed = milli(250);
const WAVE_MIN_CHAIN: Fixed = milli(100);

/// Prepaid passage: a diaspora of `PREPAID_SHARE` per mille of the origin's people, on a
/// road, pays the ticket for everyone who takes it (relatives and labour contractors sent
/// tickets home); a smaller one, in proportion.
const PREPAID_SHARE: i64 = 50;

/// Chain migration: a diaspora of `CHAIN_SHARE` per mille of the origin's people
/// multiplies the pull by `1 + CHAIN`; a smaller one, in proportion. A road with no diaspora
/// starts at `STRANGER`: without someone there, few go (the network theory of migration).
const CHAIN: i64 = 3;
const STRANGER: Fixed = milli(200);
const CHAIN_SHARE: i64 = 50;

/// A law lets a wave through at or above this, per mille of the open flow (quotas do not).
const DOOR: i64 = 500;
/// Pressure to leave at which a wave opens, and below which it ends.
const WAVE_OPEN: Fixed = milli(600);
const WAVE_CLOSE: Fixed = milli(350);
/// Weights of the pressure: idle hands on the land, hunger, and the best pull abroad.
const PRESSURE_LAND: Fixed = milli(1000);
const PRESSURE_HUNGER: Fixed = milli(1000);
const PRESSURE_PULL: Fixed = milli(500);
/// Emigrants of a wave, per mille of the origin a year, at full pressure; most a destination
/// takes from waves, per mille of its people a year; least share of its labourers in jobs
/// for a destination to stay open.
const WAVE_RATE: i64 = 3;
const WAVE_CAP: i64 = 5;
const WAVE_JOBS: Fixed = milli(500);

/// People of one origin living in one destination, by culture: the descendants of who was
/// there in 1925 and everyone who came since, with their children. When pops carry culture,
/// each entry becomes the community `culture` in the destination.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diaspora {
    pub origin: u16,
    pub dest: u16,
    /// Index into the scenario's cultures; `u16::MAX` when the origin has no culture data.
    pub culture: u16,
    pub people: i64,
    /// Arrivals since the start (no children), to compare with the immigration records.
    pub arrived: i64,
}

/// Why a wave opened: the largest term of the pressure to leave.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Push {
    /// Idle hands on the land: more peasants than the farms employ.
    Land,
    Hunger,
    /// Income several times higher abroad.
    Poverty,
}

/// Why a wave ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveEnd {
    PressureEased,
    /// The destination's immigration law or the origin's emigration law shut.
    DoorClosed,
    /// The destination's towns have no jobs left.
    Full,
    /// Income at the destination no longer pays the move.
    NoPull,
}

/// A mass migration from one territory to another, open while the pressure lasts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wave {
    pub origin: u16,
    pub dest: u16,
    /// Months since year 0 (`year * 12 + month - 1`) when it opened.
    pub since: i32,
    pub cause: Push,
    pub people: i64,
}

/// A wave that ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveRecord {
    pub wave: Wave,
    pub until: i32,
    pub end: WaveEnd,
}

impl HashState for Diaspora {
    fn hash_state(&self, h: &mut StateHasher) {
        self.origin.hash_state(h);
        self.dest.hash_state(h);
        self.culture.hash_state(h);
        self.people.hash_state(h);
        self.arrived.hash_state(h);
    }
}

impl HashState for Wave {
    fn hash_state(&self, h: &mut StateHasher) {
        self.origin.hash_state(h);
        self.dest.hash_state(h);
        self.since.hash_state(h);
        (self.cause as u8).hash_state(h);
        self.people.hash_state(h);
    }
}

impl HashState for WaveRecord {
    fn hash_state(&self, h: &mut StateHasher) {
        self.wave.hash_state(h);
        self.until.hash_state(h);
        (self.end as u8).hash_state(h);
    }
}

/// Territory facts of the month.
struct Land {
    people: i64,
    income_per_head: Fixed,
    literacy: Fixed,
    idle: Fixed,
    hunger: Fixed,
    /// Share of the labourers in jobs, to the fourth power: how much room the towns have for
    /// newcomers.
    jobs: Fixed,
    /// Births minus deaths this month.
    natural: i64,
    /// Land ceiling in farm levels per thousand farmers.
    land: Fixed,
}

/// The month's migration.
pub(super) struct Plan {
    /// Net migrants by region (immigrants positive).
    pub moves: Vec<i64>,
    pub diaspora: Vec<Diaspora>,
    pub waves: Vec<Wave>,
    pub ended: Vec<WaveRecord>,
}

/// People abroad by (origin, destination), all cultures together.
fn stock(diaspora: &[Diaspora], nt: usize) -> std::collections::BTreeMap<(u16, u16), i64> {
    let mut out = std::collections::BTreeMap::new();
    for d in diaspora {
        if usize::from(d.origin) < nt && usize::from(d.dest) < nt {
            *out.entry((d.origin, d.dest)).or_insert(0) += d.people;
        }
    }
    out
}

/// How far the road from `i` to `j` is a chain, 0–1: the diaspora on it against
/// `CHAIN_SHARE` of the origin's people (how many families there have someone on that road).
fn chain_share(
    lands: &[Land],
    abroad: &std::collections::BTreeMap<(u16, u16), i64>,
    i: usize,
    j: usize,
) -> Fixed {
    let there = abroad.get(&(i as u16, j as u16)).copied().unwrap_or(0);
    let origin = lands[i].people;
    Fixed::from_ratio(there.min(origin), (origin * CHAIN_SHARE / 1000).max(1)).min(Fixed::ONE)
}

/// Share of a law's value above the open flow (1000 per mille) that pays the passage: an
/// immigration law above 1000 is the destination paying tickets, an emigration law above 1000
/// the origin paying them; 0 to 1.
fn subsidy(law: i64) -> Fixed {
    Fixed::from_ratio((law - 1000).clamp(0, 1000), 1000)
}

/// Share of the passage from `i` to `j` a law pays, 0–1: the origin's sponsored emigration, and
/// the destination's subsidized immigration where the origin lets recruiters work (its
/// emigration law at least open: Italy forbade subsidized emigration to Brazil in 1902).
fn paid(laws: &[(i64, i64)], i: usize, j: usize) -> Fixed {
    let (_, emi) = laws[i / 2];
    let from_destination = if emi >= 1000 {
        subsidy(laws[j / 2].0)
    } else {
        Fixed::ZERO
    };
    (from_destination + subsidy(emi)).min(Fixed::ONE)
}

/// Who can pay the passage from `i` to `j`: the literate, those whose ticket the diaspora on
/// that road prepays, and those whose ticket a law pays.
fn afford(
    lands: &[Land],
    abroad: &std::collections::BTreeMap<(u16, u16), i64>,
    laws: &[(i64, i64)],
    i: usize,
    j: usize,
) -> Fixed {
    let lit = lands[i].literacy;
    let own = AFFORD_BASE + (Fixed::ONE - AFFORD_BASE) * lit * lit;
    let there = abroad.get(&(i as u16, j as u16)).copied().unwrap_or(0);
    let prepaid = Fixed::from_ratio(
        there.min(lands[i].people),
        (lands[i].people * PREPAID_SHARE / 1000).max(1),
    )
    .min(Fixed::ONE);
    let paid = paid(laws, i, j);
    Fixed::ONE - (Fixed::ONE - own) * (Fixed::ONE - prepaid) * (Fixed::ONE - paid)
}

pub(super) fn plan(
    world: &GameWorld,
    regions: &[RegionMonth],
    laws: &[(i64, i64)],
    now: i32,
) -> Plan {
    let econ = &world.econ;
    let state = &world.demography;
    let nt = econ.markets.len() * 2;
    let mut out = Plan {
        moves: vec![0i64; regions.len()],
        diaspora: state.diaspora.clone(),
        waves: Vec::new(),
        ended: Vec::new(),
    };
    if state.territories.len() != nt {
        return out;
    }
    let lands = lands(world, regions, nt);
    let abroad = stock(&state.diaspora, nt);
    let cos: Vec<Fixed> = state.territories.iter().map(|t| cos_deg(t.lat)).collect();
    let pull_of = |i: usize, j: usize| -> Fixed {
        pull(&lands, &state.territories, &cos, &abroad, laws, i, j)
    };

    // Steady flow, by origin; then each destination cut to its cap.
    let mut flows: Vec<(usize, usize, i64)> = Vec::new();
    let mut inflow = vec![0i64; nt];
    let mut best_pull = vec![Fixed::ZERO; nt];
    let mut pulls_of: Vec<Vec<(usize, Fixed)>> = vec![Vec::new(); nt];
    for i in 0..nt {
        if lands[i].people <= 0 {
            continue;
        }
        let pulls: Vec<(usize, Fixed)> = (0..nt)
            .map(|j| (j, pull_of(i, j)))
            .filter(|(_, p)| *p > Fixed::ZERO)
            .collect();
        let best = pulls
            .iter()
            .map(|(_, p)| *p)
            .max()
            .unwrap_or(Fixed::ZERO)
            .min(Fixed::ONE);
        best_pull[i] = best;
        let (_, emi) = laws[i / 2];
        let push = Fixed::ONE + lands[i].idle;
        // Would-be emigrants of the month, before the passage; each road keeps those who can
        // pay for it.
        let emigrants = (Fixed::from_int(lands[i].people) * best * push)
            .mul_ratio(EMIGRATION_RATE * emi.min(1000), 1000 * 1000 * 12)
            .to_int_floor();
        if emigrants > 0 {
            // Shares by pull × size of the destination.
            let weights: Vec<(usize, i128)> = pulls
                .iter()
                .map(|&(j, p)| (j, i128::from(p.raw()) * i128::from(lands[j].people)))
                .collect();
            let total: i128 = weights.iter().map(|(_, w)| *w).sum();
            if total > 0 {
                for (j, w) in weights {
                    let paying = afford(&lands, &abroad, laws, i, j);
                    let n = (i128::from(emigrants) * w * i128::from(paying.raw()) / (total * 1000))
                        as i64;
                    if n > 0 {
                        flows.push((i, j, n));
                        inflow[j] += n;
                    }
                }
            }
        }
        pulls_of[i] = pulls;
    }
    let cap: Vec<i64> = lands
        .iter()
        .map(|l| l.people * IMMIGRATION_CAP / 12_000)
        .collect();
    let mut moved: Vec<(usize, usize, i64)> = flows
        .into_iter()
        .map(|(i, j, n)| {
            let n = if inflow[j] > cap[j] {
                (i128::from(n) * i128::from(cap[j]) / i128::from(inflow[j])) as i64
            } else {
                n
            };
            (i, j, n)
        })
        .collect();

    // Waves: keep, end or open one per origin.
    let pressure = |i: usize| -> (Fixed, Push) {
        let l = &lands[i];
        let terms = [
            (l.idle * PRESSURE_LAND, Push::Land),
            (l.hunger * PRESSURE_HUNGER, Push::Hunger),
            (best_pull[i] * PRESSURE_PULL, Push::Poverty),
        ];
        let total = terms.iter().map(|(v, _)| *v).sum();
        let cause = terms
            .iter()
            .max_by_key(|(v, _)| *v)
            .map_or(Push::Poverty, |(_, c)| *c);
        (total, cause)
    };
    let open_door = |i: usize, j: usize| laws[j / 2].0 >= DOOR && laws[i / 2].1 >= DOOR;
    let road_open = |i: usize, j: usize| {
        paid(laws, i, j) > Fixed::ZERO || chain_share(&lands, &abroad, i, j) >= WAVE_MIN_CHAIN
    };
    let mut wave_in = vec![0i64; nt];
    let mut wave_flows: Vec<(usize, i64)> = Vec::new(); // index into out.waves
    let mut has_wave = vec![false; nt];
    for w in &state.waves {
        let (i, j) = (usize::from(w.origin), usize::from(w.dest));
        if i >= nt || j >= nt {
            continue;
        }
        let (p, _) = pressure(i);
        let end = if !open_door(i, j) {
            Some(WaveEnd::DoorClosed)
        } else if p < WAVE_CLOSE {
            Some(WaveEnd::PressureEased)
        } else if lands[j].jobs < WAVE_JOBS {
            Some(WaveEnd::Full)
        } else if pull_of(i, j) < WAVE_MIN_PULL / 2 {
            Some(WaveEnd::NoPull)
        } else {
            None
        };
        match end {
            Some(end) => out.ended.push(WaveRecord {
                wave: w.clone(),
                until: now,
                end,
            }),
            None => {
                has_wave[i] = true;
                out.waves.push(w.clone());
            }
        }
    }
    for i in 0..nt {
        if has_wave[i] || lands[i].people <= 0 {
            continue;
        }
        let (p, cause) = pressure(i);
        if p < WAVE_OPEN {
            continue;
        }
        // The strongest pull, weighed by how large the destination is next to the origin.
        let size = |j: usize| {
            Fixed::from_ratio(lands[j].people.min(lands[i].people), lands[i].people.max(1))
        };
        let dest = pulls_of[i]
            .iter()
            .filter(|&&(j, p)| {
                p >= WAVE_MIN_PULL
                    && open_door(i, j)
                    && road_open(i, j)
                    && lands[j].jobs >= WAVE_JOBS
            })
            .max_by(|a, b| {
                (a.1 * size(a.0))
                    .cmp(&(b.1 * size(b.0)))
                    .then(b.0.cmp(&a.0))
            })
            .map(|&(j, _)| j);
        if let Some(j) = dest {
            out.waves.push(Wave {
                origin: i as u16,
                dest: j as u16,
                since: now,
                cause,
                people: 0,
            });
        }
    }
    for (k, w) in out.waves.iter().enumerate() {
        let (i, j) = (usize::from(w.origin), usize::from(w.dest));
        let (p, _) = pressure(i);
        let strength = ((p - WAVE_CLOSE) / (WAVE_OPEN - WAVE_CLOSE)).clamp(Fixed::ZERO, Fixed::ONE);
        let paying = afford(&lands, &abroad, laws, i, j);
        let n =
            (Fixed::from_int(lands[i].people) * strength * paying * pull_of(i, j).min(Fixed::ONE))
                .mul_ratio(WAVE_RATE, 12_000)
                .to_int_floor();
        if n > 0 {
            wave_flows.push((k, n));
            wave_in[j] += n;
        }
    }
    let wave_cap: Vec<i64> = lands.iter().map(|l| l.people * WAVE_CAP / 12_000).collect();
    for (k, n) in wave_flows {
        let w = &mut out.waves[k];
        let (i, j) = (usize::from(w.origin), usize::from(w.dest));
        let n = if wave_in[j] > wave_cap[j] {
            (i128::from(n) * i128::from(wave_cap[j]) / i128::from(wave_in[j])) as i64
        } else {
            n
        };
        w.people += n;
        moved.push((i, j, n));
    }

    // Diaspora: grows with the destination's births and deaths, and with the arrivals, who
    // take the cultures already on that road (or the origin's own).
    for d in &mut out.diaspora {
        let l = &lands[usize::from(d.dest).min(nt - 1)];
        if l.people > 0 {
            d.people +=
                (i128::from(d.people) * i128::from(l.natural) / i128::from(l.people)) as i64;
        }
    }
    let mut net = vec![0i64; nt];
    let mut arrivals: std::collections::BTreeMap<(u16, u16), i64> = Default::default();
    for &(i, j, n) in &moved {
        if n <= 0 {
            continue;
        }
        net[i] -= n;
        net[j] += n;
        *arrivals.entry((i as u16, j as u16)).or_insert(0) += n;
    }
    for ((i, j), n) in arrivals {
        let on_road: Vec<usize> = (0..out.diaspora.len())
            .filter(|&k| out.diaspora[k].origin == i && out.diaspora[k].dest == j)
            .collect();
        let total: i64 = on_road.iter().map(|&k| out.diaspora[k].people.max(0)).sum();
        if total > 0 {
            let mut left = n;
            for (x, &k) in on_road.iter().enumerate() {
                let share = if x + 1 == on_road.len() {
                    left
                } else {
                    (i128::from(n) * i128::from(out.diaspora[k].people.max(0)) / i128::from(total))
                        as i64
                };
                out.diaspora[k].people += share;
                out.diaspora[k].arrived += share;
                left -= share;
            }
        } else {
            let culture = state.territories[usize::from(i)].culture;
            match out
                .diaspora
                .iter_mut()
                .find(|d| d.origin == i && d.dest == j && d.culture == culture)
            {
                Some(d) => {
                    d.people += n;
                    d.arrived += n;
                }
                None => out.diaspora.push(Diaspora {
                    origin: i,
                    dest: j,
                    culture,
                    people: n,
                    arrived: n,
                }),
            }
        }
    }
    out.diaspora.sort_by_key(|d| (d.origin, d.dest, d.culture));

    // Into regions: emigrants by population, immigrants by town population.
    let mut people = vec![0i64; nt];
    let mut town = vec![0i64; nt];
    for (r, m) in econ.regions.iter().zip(regions) {
        let t = usize::from(r.market) * 2 + usize::from(r.colony);
        people[t] += m.people;
        town[t] += m.urban;
    }
    for (ri, (r, m)) in econ.regions.iter().zip(regions).enumerate() {
        let t = usize::from(r.market) * 2 + usize::from(r.colony);
        let n = net[t];
        out.moves[ri] = if n < 0 && people[t] > 0 {
            (i128::from(n) * i128::from(m.people) / i128::from(people[t])) as i64
        } else if n > 0 && town[t] > 0 {
            (i128::from(n) * i128::from(m.urban) / i128::from(town[t])) as i64
        } else {
            0
        };
    }
    out
}

/// Territory sums of the month, in region order.
fn lands(world: &GameWorld, regions: &[RegionMonth], nt: usize) -> Vec<Land> {
    let econ = &world.econ;
    let mut people = vec![0i64; nt];
    let mut income = vec![Fixed::ZERO; nt];
    let mut literate = vec![Fixed::ZERO; nt];
    let mut fed = vec![Fixed::ZERO; nt];
    let mut idle = vec![0i64; nt];
    let mut work = vec![0i64; nt];
    let mut natural = vec![0i64; nt];
    let mut lab = vec![(0i64, 0i64); nt];
    let mut land = vec![(0i64, 0i64); nt];
    let mut market_people = vec![0i64; econ.markets.len()];
    let mut market_income = vec![Fixed::ZERO; econ.markets.len()];
    for (r, m) in econ.regions.iter().zip(regions) {
        let t = usize::from(r.market) * 2 + usize::from(r.colony);
        people[t] += m.people;
        income[t] += m.income;
        literate[t] += m.literacy_people;
        fed[t] += m.food_people;
        idle[t] += m.idle_workers;
        work[t] += m.workers;
        natural[t] += m.births - m.deaths;
        lab[t].0 += m.labourers;
        lab[t].1 += m.labourers_employed;
        land[t].0 += m.land_total;
        land[t].1 += m.farmers;
        market_people[usize::from(r.market)] += m.people;
        market_income[usize::from(r.market)] += m.income;
    }
    // Real income per head: the country's GDP per head at base prices, split between
    // metropole and colonies by money income per head.
    (0..nt)
        .map(|t| {
            let mi = t / 2;
            let n = people[t];
            let mut out = Land {
                people: n,
                income_per_head: Fixed::ZERO,
                literacy: Fixed::ZERO,
                idle: Fixed::ZERO,
                hunger: Fixed::ZERO,
                jobs: Fixed::ONE,
                natural: natural[t],
                land: if land[t].1 > 0 {
                    Fixed::from_ratio(land[t].0 * 1000, land[t].1)
                } else {
                    Fixed::ZERO
                },
            };
            if n <= 0 {
                return out;
            }
            out.literacy = literate[t] / n;
            out.hunger = (Fixed::ONE - fed[t] / n).clamp(Fixed::ZERO, Fixed::ONE);
            if work[t] > 0 {
                out.idle = Fixed::from_ratio(idle[t], work[t]);
            }
            if lab[t].0 > 0 {
                let e = Fixed::from_ratio(lab[t].1.min(lab[t].0), lab[t].0);
                out.jobs = e * e * e * e;
            }
            if market_people[mi] > 0 && market_income[mi] > Fixed::ZERO {
                let gdp_pc = econ.markets[mi].gdp_last_year / market_people[mi];
                let relative =
                    income[t].mul_div(Fixed::from_int(market_people[mi]), market_income[mi]) / n;
                out.income_per_head = gdp_pc * relative;
            }
            out
        })
        .collect()
}

/// Pull of destination `j` on the people of `i`: income gap × distance × jobs × language ×
/// diaspora × the destination's immigration law.
fn pull(
    lands: &[Land],
    territories: &[Territory],
    cos: &[Fixed],
    abroad: &std::collections::BTreeMap<(u16, u16), i64>,
    laws: &[(i64, i64)],
    i: usize,
    j: usize,
) -> Fixed {
    let (a, b) = (&lands[i], &lands[j]);
    if i == j || a.people <= 0 || b.people <= 0 || a.income_per_head <= Fixed::ZERO {
        return Fixed::ZERO;
    }
    let ratio = b.income_per_head / a.income_per_head;
    let income = ((ratio - GAP_START) / GAP_FULL).clamp(Fixed::ZERO, Fixed::ONE);
    let land = if ratio >= LAND_MIN_RATIO && a.land > Fixed::ZERO {
        ((b.land / a.land - LAND_START) / LAND_FULL).clamp(Fixed::ZERO, Fixed::ONE)
    } else {
        Fixed::ZERO
    };
    // Emigration a programme of the origin organizes works as a chain: the recruiter is the
    // network. A destination's subsidy pays the ticket (`afford`, the distance) but recruits
    // no one by itself.
    let sponsored = subsidy(laws[i / 2].1);
    let paid = paid(laws, i, j);
    let chain_share = chain_share(lands, abroad, i, j).max(sponsored);
    let network = if ratio >= NETWORK_MIN_RATIO {
        chain_share * NETWORK
    } else {
        Fixed::ZERO
    };
    let gap = (income.max(land) + network).min(Fixed::ONE);
    if gap <= Fixed::ZERO {
        return Fixed::ZERO;
    }
    let (ti, tj) = (&territories[i], &territories[j]);
    // A paid passage makes the distance cheaper in proportion.
    let dlat = (ti.lat - tj.lat) * milli(111_200); // km
    let mut dlon = (ti.lon - tj.lon).abs();
    if dlon > Fixed::from_int(180) {
        dlon = Fixed::from_int(360) - dlon;
    }
    let dlon = dlon * milli(111_200) * ((cos[i] + cos[j]) / 2);
    let d2 = (dlat * dlat + dlon * dlon) / (DISTANCE_KM * DISTANCE_KM);
    let distance = Fixed::ONE / (Fixed::ONE + d2);
    let distance = distance + (Fixed::ONE - distance) * paid;
    let mut link = 1;
    if ti.family != u8::MAX && ti.family == tj.family {
        link *= SAME_FAMILY;
    }
    if i / 2 == j / 2 {
        link *= SAME_EMPIRE;
    }
    let chain = STRANGER + (Fixed::from_int(1 + CHAIN) - STRANGER) * chain_share;
    // Above the open flow the law pays the passage (in `afford` and the distance), not more pull.
    let (imm, _) = laws[j / 2];
    (gap * distance * b.jobs * chain * link).mul_ratio(imm.min(1000), 1000)
}

/// Cosine of a latitude in degrees (|lat| ≤ 90), by its Taylor series.
pub(super) fn cos_deg(lat: Fixed) -> Fixed {
    let x = lat.abs().min(Fixed::from_int(90)) * milli(17)
        + lat.abs().min(Fixed::from_int(90)).mul_ratio(453, 1_000_000);
    let x2 = x * x;
    let x4 = x2 * x2;
    (Fixed::ONE - x2 / 2 + x4 / 24 - x4 * x2 / 720).clamp(Fixed::ZERO, Fixed::ONE)
}

// ---------------------------------------------------------------------------------------------
// 1925

/// Diaspora of 1925 from the communities of every state: a culture living outside its
/// homeland, where it is not native, counts as people of the homeland in that territory.
/// The homeland is the territory with most of the culture; a culture whose homeland holds less
/// than half of it is ambiguous and left out (returned by key, with the homeland's share).
/// Territories next to the homeland by land are left out too: there the culture is a border
/// minority, not people who moved.
pub fn initial_diaspora(
    econ: &crate::econ::Economy,
    map: &crate::scenario::MapFile,
    communities: &crate::communities::Communities,
) -> (Vec<Diaspora>, Vec<(String, f64)>) {
    use std::collections::{BTreeMap, BTreeSet};
    let nt = econ.markets.len() * 2;
    let nc = communities.cultures.len();
    let mut territory_of_state = BTreeMap::new();
    for r in &econ.regions {
        territory_of_state.insert(r.state, usize::from(r.market) * 2 + usize::from(r.colony));
    }
    // Every state: its territory, its people by culture, and the territories it touches by land.
    struct Row {
        t: usize,
        by_culture: Vec<f64>,
        touches: BTreeSet<usize>,
    }
    let mut rows = Vec::new();
    let mut by_territory = vec![vec![0f64; nt]; nc];
    // A culture is native to a territory where it is the largest in some state.
    let mut native = vec![vec![false; nt]; nc];
    for r in &econ.regions {
        let t = usize::from(r.market) * 2 + usize::from(r.colony);
        let Some(def) = map.states.iter().find(|d| d.id == r.state) else {
            continue;
        };
        let Some(mix) = communities.states.iter().find(|m| m.state == def.key) else {
            continue;
        };
        let people = r.population() as f64;
        let mut by_culture = vec![0f64; nc];
        for &(id, share) in &mix.shares {
            let c = communities.communities[id as usize].0 as usize;
            by_culture[c] += people * f64::from(share);
        }
        for c in 0..nc {
            by_territory[c][t] += by_culture[c];
        }
        if let Some((c, _)) = by_culture
            .iter()
            .enumerate()
            .filter(|(_, v)| **v > 0.0)
            .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
        {
            native[c][t] = true;
        }
        let touches = def
            .neighbours
            .iter()
            .filter_map(|n| territory_of_state.get(n).copied())
            .collect();
        rows.push(Row {
            t,
            by_culture,
            touches,
        });
    }
    let mut stock: BTreeMap<(usize, usize, usize), f64> = BTreeMap::new();
    let mut ambiguous = Vec::new();
    for c in 0..nc {
        let total: f64 = by_territory[c].iter().sum();
        if total <= 0.0 {
            continue;
        }
        let (home, most) = by_territory[c]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
            .map(|(t, v)| (t, *v))
            .unwrap_or((0, 0.0));
        // People of the culture outside the homeland, where it is not native, in states that
        // do not touch the homeland: there it is a border minority, not people who moved
        // (Germans in Bohemia, Hungarians in Transylvania).
        let mut abroad = BTreeMap::new();
        for row in &rows {
            let v = row.by_culture[c];
            if v > 0.0 && row.t != home && !native[c][row.t] && !row.touches.contains(&home) {
                *abroad.entry(row.t).or_insert(0.0) += v;
            }
        }
        if abroad.values().sum::<f64>() < 1000.0 {
            continue;
        }
        if most / total < 0.5 {
            ambiguous.push((communities.cultures[c].key.clone(), most / total));
            continue;
        }
        for (t, v) in abroad {
            stock.insert((home, t, c), v);
        }
    }
    let out = stock
        .into_iter()
        .filter(|(_, v)| *v >= 1000.0)
        .map(|((o, d, c), v)| Diaspora {
            origin: o as u16,
            dest: d as u16,
            culture: c as u16,
            people: v.round() as i64,
            arrived: 0,
        })
        .collect();
    (out, ambiguous)
}
