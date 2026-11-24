//! Relations between countries, measured (`docs/ai.pt-BR.md`, slice 1 of the strategic AI),
//! behind the switch `relations` (off). Read only: nothing in the tick reads these measures yet,
//! and nothing is saved; the interface computes them when it shows them.
//!
//! - **Power:** the Composite Index of National Capability of the Correlates of War (Singer,
//!   Bremer and Stuckey, 1972): the mean of a country's shares of the world's military
//!   spending, military personnel, steel production, primary energy consumption, population and
//!   urban population (NMC Documentation v7.0, "CINC Score", p. 8). Primary energy in coal-ton
//!   equivalents by the NMC's factors (pp. 63-65).
//! - **Lost territory**, per country: its people (its main culture) living in the regions of a
//!   neighbour, and the people of the regions it held in 1925 and no longer holds, over its own.
//! - **Hostility** per pair, 0 to 1000, term by term: territory, minority, regime, causes, arms
//!   race and war push it up; trade, agreements, a common enemy and mutual deterrence pull it
//!   down. The weights are DESIGN VALUES (`data/diplomacy/relations.ron`): no source gives them,
//!   and the calibration moves them later.
//! - **Threat**, per country: the sum over the others of hostility × the power of theirs that
//!   reaches it, over its own power.
//! - **Role**, per country: its rank by power, whether it is a great power, and whether it is
//!   rising or declining by its share of the world's GDP over `trend_years` (the yearly history
//!   the economy keeps; no new state).

use std::collections::{BTreeMap, BTreeSet};

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use crate::econ::catalogue::Catalogue;
use crate::game::GameWorld;

/// The weights of the hostility's terms, per mille of a whole term (DESIGN VALUES).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weights {
    pub territory: i32,
    pub minority: i32,
    pub regime: i32,
    pub causes: i32,
    pub arms_race: i32,
    pub war: i32,
    pub trade: i32,
    pub agreement: i32,
    pub common_enemy: i32,
    pub deterrence: i32,
}

/// `data/diplomacy/relations.ron`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationsRules {
    /// Coal-ton equivalents per unit, per mille: coal (1 t), crude oil (1 t: 1.429), natural gas
    /// (1,000 m³: 1.33) (NMC Documentation v7.0, pp. 63-65).
    pub coal_tce: u32,
    pub crude_tce: u32,
    pub gas_tce: u32,
    pub weights: Weights,
    /// Lost territory or minority abroad, per mille of the own people, that gives a whole term.
    pub people_full: u32,
    /// Military spending per mille of GDP from which, and at which whole, the arms race term
    /// counts.
    pub effort_from: u32,
    pub effort_full: u32,
    /// Trade dependence (the pair's trade over the smaller GDP, per mille) that gives a whole
    /// cooling term.
    pub trade_full: u32,
    /// Hostility from which a country counts as an enemy (for the common enemy).
    pub enemy_from: i32,
    /// Of the power of a country that is not a neighbour, the part that reaches, per mille.
    pub reach_far: u32,
    /// A great power holds at least this share of the world's power, per mille.
    pub great_power: u32,
    /// Years over which the trend is read, and the change of the share of the world's GDP, per
    /// mille of the share, that makes a country rising (or, negative, declining).
    pub trend_years: i32,
    pub rising: i32,
}

/// What the measures need from the data, made once.
#[derive(Clone, Debug)]
pub struct RelationsSetup {
    pub rules: RelationsRules,
    /// The owner of each region in 1925.
    pub owner_1925: Vec<u16>,
    /// Pairs of neighbouring states (`war::terrain::BordersFile`).
    pub state_borders: Vec<(u16, u16)>,
    /// Goods: coal, crude oil, natural gas, steel.
    pub goods: [Option<u16>; 4],
    /// The unit type of a nuclear warhead, if any.
    pub warhead: Option<u16>,
}

/// The six components of the CINC, in order.
pub const COMPONENTS: [&str; 6] = [
    "military_spending",
    "military_personnel",
    "steel",
    "energy",
    "population",
    "urban_population",
];

/// A country's measures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountryMeasures {
    pub country: u16,
    /// Its share of the world in each component, per mille, and their mean (the CINC).
    pub shares: [u32; 6],
    pub cinc: u32,
    /// 1 for the strongest.
    pub rank: u16,
    pub great_power: bool,
    /// Change of its share of the world's GDP over `trend_years`, per mille of the share.
    pub trend: i32,
    pub role: Role,
    /// Lost territory, per mille of its own people.
    pub lost_territory: u32,
    /// Threat, per mille of its own power.
    pub threat: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The strongest.
    Leading,
    /// A great power, neither rising nor declining.
    Great,
    Rising,
    Declining,
    Small,
}

/// The terms of a pair's hostility, each 0 to 1000 before its weight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Terms {
    pub territory: i32,
    pub minority: i32,
    pub regime: i32,
    pub causes: i32,
    pub arms_race: i32,
    pub war: i32,
    pub trade: i32,
    pub agreement: i32,
    pub common_enemy: i32,
    pub deterrence: i32,
}

impl Terms {
    /// The weighted sum, 0 to 1000: the terms that push up less those that pull down.
    pub fn hostility(&self, w: &Weights) -> i32 {
        let up = self.territory * w.territory
            + self.minority * w.minority
            + self.regime * w.regime
            + self.causes * w.causes
            + self.arms_race * w.arms_race
            + self.war * w.war;
        let down = self.trade * w.trade
            + self.agreement * w.agreement
            + self.common_enemy * w.common_enemy
            + self.deterrence * w.deterrence;
        ((up - down) / 1000).clamp(0, 1000)
    }
}

/// A pair's hostility (`a` < `b`), with its terms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    pub a: u16,
    pub b: u16,
    pub terms: Terms,
    pub hostility: i32,
}

/// Every measure, for the world as it is.
#[derive(Clone, Debug, Default)]
pub struct Relations {
    pub countries: Vec<CountryMeasures>,
    /// Every pair with a term, sorted by (a, b).
    pub pairs: Vec<Pair>,
}

impl Relations {
    pub fn pair(&self, a: u16, b: u16) -> Option<&Pair> {
        let k = (a.min(b), a.max(b));
        self.pairs
            .binary_search_by_key(&k, |p| (p.a, p.b))
            .ok()
            .map(|i| &self.pairs[i])
    }

    pub fn hostility(&self, a: u16, b: u16) -> i32 {
        self.pair(a, b).map_or(0, |p| p.hostility)
    }
}

fn per_mille(num: i128, den: i128) -> u32 {
    if den <= 0 {
        0
    } else {
        (num * 1000 / den).clamp(0, 1_000_000) as u32
    }
}

fn scaled(v: u32, full: u32) -> i32 {
    (i64::from(v) * 1000 / i64::from(full.max(1))).clamp(0, 1000) as i32
}

/// The six components of each country, in their own units (spending in money, the rest in
/// people, tonnes and coal-ton equivalents).
fn components(s: &RelationsSetup, cat: &Catalogue, world: &GameWorld) -> Vec<[i128; 6]> {
    let n = world.countries.len();
    let mut out = vec![[0i128; 6]; n];
    let one = i128::from(Fixed::ONE.raw());
    for (c, m) in world.econ.markets.iter().enumerate().take(n) {
        out[c][0] = i128::from(crate::econ::public::military_room(m, cat).raw().max(0));
        let at = |g: Option<u16>, v: &[Fixed]| -> i128 {
            g.and_then(|g| v.get(usize::from(g)))
                .map_or(0, |q| i128::from(q.raw().max(0)))
        };
        let [coal, crude, gas, steel] = s.goods;
        out[c][2] = at(steel, &m.supply);
        let r = &s.rules;
        out[c][3] = (at(coal, &m.demand) * i128::from(r.coal_tce)
            + at(crude, &m.demand) * i128::from(r.crude_tce)
            + at(gas, &m.demand) * i128::from(r.gas_tce))
            / 1000;
    }
    for r in &world.econ.regions {
        let c = usize::from(r.market);
        if c >= n {
            continue;
        }
        for (p, pop) in r.pops.iter().enumerate() {
            let size = i128::from(pop.size.max(0)) * one;
            if p == crate::econ::prof::SOLDIER {
                out[c][1] += size;
            }
            out[c][4] += size;
            if crate::econ::migration::is_urban(p) {
                out[c][5] += size;
            }
        }
    }
    out
}

/// Neighbouring countries, by their regions' states.
fn neighbours(s: &RelationsSetup, world: &GameWorld) -> BTreeSet<(u16, u16)> {
    let mut owner: BTreeMap<u16, BTreeSet<u16>> = BTreeMap::new();
    for r in &world.econ.regions {
        owner.entry(r.state).or_default().insert(r.market);
    }
    let mut out = BTreeSet::new();
    for &(x, y) in &s.state_borders {
        let (Some(a), Some(b)) = (owner.get(&x), owner.get(&y)) else {
            continue;
        };
        for &p in a {
            for &q in b {
                if p != q {
                    out.insert((p.min(q), p.max(q)));
                }
            }
        }
    }
    out
}

/// Measures the world.
pub fn measure(
    s: &RelationsSetup,
    cat: &Catalogue,
    world: &GameWorld,
    table: Option<&crate::demography::communities::CommunityTable>,
    year: i32,
) -> Relations {
    let r = &s.rules;
    let n = world.countries.len();
    // Power.
    let comp = components(s, cat, world);
    let totals: [i128; 6] = std::array::from_fn(|k| comp.iter().map(|x| x[k]).sum());
    let shares: Vec<[u32; 6]> = comp
        .iter()
        .map(|x| std::array::from_fn(|k| per_mille(x[k], totals[k])))
        .collect();
    let cinc: Vec<u32> = shares.iter().map(|x| x.iter().sum::<u32>() / 6).collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&x, &y| cinc[y].cmp(&cinc[x]).then(x.cmp(&y)));
    let mut rank = vec![0u16; n];
    for (i, &c) in order.iter().enumerate() {
        rank[c] = i as u16 + 1;
    }
    // Trend: the share of the world's GDP now against `trend_years` ago.
    let gdp_share = |y: i32| -> BTreeMap<u16, i128> {
        let rows: Vec<_> = world.econ.history.iter().filter(|h| h.year == y).collect();
        let all: i128 = rows.iter().map(|h| i128::from(h.gdp.raw().max(0))).sum();
        rows.iter()
            .map(|h| {
                (
                    h.market,
                    if all > 0 {
                        i128::from(h.gdp.raw().max(0)) * 1_000_000 / all
                    } else {
                        0
                    },
                )
            })
            .collect()
    };
    let last = world.econ.history.iter().map(|h| h.year).max();
    let (now, then) = match last {
        Some(y) => (gdp_share(y), gdp_share(y - r.trend_years)),
        None => (BTreeMap::new(), BTreeMap::new()),
    };
    let trend = |c: u16| -> i32 {
        match (now.get(&c), then.get(&c)) {
            (Some(&x), Some(&y)) if y > 0 => ((x - y) * 1000 / y).clamp(-100_000, 100_000) as i32,
            _ => 0,
        }
    };
    let near = neighbours(s, world);
    let is_near = |a: u16, b: u16| near.contains(&(a.min(b), a.max(b)));
    // Each country's main culture, and its people.
    let cultures = table.map(|t| crate::influence::main_cultures(world, t));
    let mut people = vec![0i128; n];
    for reg in &world.econ.regions {
        if usize::from(reg.market) < n {
            people[usize::from(reg.market)] += i128::from(reg.population().max(0));
        }
    }
    // (country, region's owner) -> people of the country's culture there.
    let mut abroad: BTreeMap<(u16, u16), i128> = BTreeMap::new();
    if let (Some(t), Some(cult)) = (table, cultures.as_ref()) {
        for (reg, rows) in world.econ.regions.iter().zip(&world.demography.communities) {
            for (id, k) in rows.by_community() {
                let culture = t.culture(id);
                for (c, main) in cult.iter().enumerate() {
                    // A minority: the country's culture in a neighbour's metropole where it is
                    // not the neighbour's own (the Germans of Germany are no Swiss minority, the
                    // Arabs of a French colony no minority of a neighbour in France).
                    let theirs = cult.get(usize::from(reg.market)).copied().flatten();
                    if *main == Some(culture)
                        && c as u16 != reg.market
                        && !reg.colony
                        && theirs != Some(culture)
                        && is_near(c as u16, reg.market)
                    {
                        *abroad.entry((c as u16, reg.market)).or_default() += i128::from(k.max(0));
                    }
                }
            }
        }
    }
    // (country, holder) -> people of the regions the country held in 1925 that the holder
    // owns or controls now.
    let mut taken: BTreeMap<(u16, u16), i128> = BTreeMap::new();
    for (ri, reg) in world.econ.regions.iter().enumerate() {
        let Some(&was) = s.owner_1925.get(ri) else {
            continue;
        };
        let holder = world.war.fighting.controller(ri as u32, reg.market);
        if holder != was {
            *taken.entry((was, holder)).or_default() += i128::from(reg.population().max(0));
        }
    }
    let lost = |a: u16, b: u16| -> (u32, u32) {
        let own = people[usize::from(a)];
        (
            per_mille(taken.get(&(a, b)).copied().unwrap_or(0), own),
            per_mille(abroad.get(&(a, b)).copied().unwrap_or(0), own),
        )
    };
    // Military effort, per mille of GDP.
    let effort: Vec<u32> = world
        .econ
        .markets
        .iter()
        .take(n)
        .map(|m| {
            per_mille(
                i128::from(crate::econ::public::military_room(m, cat).raw().max(0)) * 52,
                i128::from(m.gdp_last_year.raw().max(0)),
            )
        })
        .collect();
    let warheads: Vec<bool> = (0..n)
        .map(|c| {
            s.warhead.is_some_and(|w| {
                world.war.countries.get(c).is_some_and(|cw| {
                    cw.formations
                        .iter()
                        .any(|f| f.unit == w && f.elements.is_positive())
                })
            })
        })
        .collect();
    let tag = |c: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(c))
            .map(|(_, x)| x.tag.as_str())
            .unwrap_or("")
    };
    let laws = &world.politics.laws;
    let ties = crate::influence::Ties::of(world, Some(cat), year);
    let flows = |a: usize, b: usize| -> i128 {
        let t = &world.econ.trade;
        let k = usize::from(t.n);
        let f = if ties.running { &t.year } else { &t.last_year };
        let at = |x: usize, y: usize| f.get(x * k + y).map_or(0, |v| i128::from(v.raw().max(0)));
        if a < k && b < k {
            at(a, b) + at(b, a)
        } else {
            0
        }
    };
    // The pairs: first without the common enemy, then with it.
    let mut pairs: Vec<Pair> = Vec::new();
    for a in 0..n as u16 {
        for b in a + 1..n as u16 {
            let (ta, ma) = lost(a, b);
            let (tb, mb) = lost(b, a);
            let mut t = Terms {
                territory: scaled(ta.max(tb), r.people_full),
                minority: scaled(ma.max(mb), r.people_full),
                ..Default::default()
            };
            // Regimes clash where they meet: neighbours, or a great power and anyone.
            let meet = is_near(a, b)
                || cinc[usize::from(a)] >= r.great_power
                || cinc[usize::from(b)] >= r.great_power;
            if let (true, Some(la), Some(lb)) =
                (meet, laws.get(usize::from(a)), laws.get(usize::from(b)))
            {
                let (mut same, mut all) = (0i64, 0i64);
                for (x, y) in la.iter().zip(lb) {
                    if let (Some(x), Some(y)) = (x, y) {
                        all += 1;
                        if x == y {
                            same += 1;
                        }
                    }
                }
                if all > 0 {
                    t.regime = (1000 - same * 1000 / all) as i32;
                }
            }
            let (ga, gb) = (tag(a), tag(b));
            t.causes = if world.treaties.causes.iter().any(|x| {
                (x.holder.as_str() == ga && x.against.as_str() == gb)
                    || (x.holder.as_str() == gb && x.against.as_str() == ga)
            }) {
                1000
            } else {
                0
            };
            t.war = if world.war.conflicts.at_war(a, b) {
                1000
            } else {
                0
            };
            // The arms race weighs between those already in conflict (land, people, a cause,
            // a war): the higher military effort of the two.
            if t.territory + t.minority + t.causes + t.war > 0 {
                let e = effort[usize::from(a)].max(effort[usize::from(b)]);
                t.arms_race = scaled(
                    e.saturating_sub(r.effort_from),
                    r.effort_full - r.effort_from,
                );
            }
            let smaller = world.econ.markets[usize::from(a)]
                .gdp_last_year
                .min(world.econ.markets[usize::from(b)].gdp_last_year);
            t.trade = scaled(
                per_mille(
                    flows(usize::from(a), usize::from(b)),
                    i128::from(smaller.raw().max(0)),
                ),
                r.trade_full,
            );
            t.agreement = if world.treaties.agreements.iter().any(|x| {
                let on = |s: &[String], k: &str| s.iter().any(|y| y == k);
                (on(&x.side_a, ga) && on(&x.side_b, gb)) || (on(&x.side_a, gb) && on(&x.side_b, ga))
            }) {
                1000
            } else {
                0
            };
            t.deterrence = if warheads[usize::from(a)] && warheads[usize::from(b)] {
                1000
            } else {
                0
            };
            if t != Terms::default() {
                pairs.push(Pair {
                    a,
                    b,
                    terms: t,
                    hostility: t.hostility(&r.weights),
                });
            }
        }
    }
    let first: BTreeMap<(u16, u16), i32> =
        pairs.iter().map(|p| ((p.a, p.b), p.hostility)).collect();
    let h0 = |x: u16, y: u16| first.get(&(x.min(y), x.max(y))).copied().unwrap_or(0);
    for p in &mut pairs {
        if (0..n as u16).any(|c| {
            c != p.a && c != p.b && h0(p.a, c) >= r.enemy_from && h0(p.b, c) >= r.enemy_from
        }) {
            p.terms.common_enemy = 1000;
            p.hostility = p.terms.hostility(&r.weights);
        }
    }
    let rel = Relations {
        countries: Vec::new(),
        pairs,
    };
    let countries = (0..n as u16)
        .map(|c| {
            let ci = usize::from(c);
            let own = i128::from(cinc[ci].max(1));
            let threat: i128 = (0..n as u16)
                .filter(|&o| o != c)
                .map(|o| {
                    let reach = if is_near(c, o) || warheads[usize::from(o)] {
                        1000
                    } else {
                        i128::from(r.reach_far)
                    };
                    i128::from(rel.hostility(c, o)) * reach * i128::from(cinc[usize::from(o)])
                        / 1_000_000
                })
                .sum();
            let lost_all: u32 = (0..n as u16)
                .filter(|&o| o != c)
                .map(|o| {
                    let (t, m) = lost(c, o);
                    t + m
                })
                .sum();
            let great = cinc[ci] >= r.great_power;
            let tr = trend(c);
            let role = if rank[ci] == 1 {
                Role::Leading
            } else if tr >= r.rising {
                Role::Rising
            } else if tr <= -r.rising {
                Role::Declining
            } else if great {
                Role::Great
            } else {
                Role::Small
            };
            CountryMeasures {
                country: c,
                shares: shares[ci],
                cinc: cinc[ci],
                rank: rank[ci],
                great_power: great,
                trend: tr,
                role,
                lost_territory: lost_all,
                threat: (threat * 1000 / own).clamp(0, 1_000_000) as u32,
            }
        })
        .collect();
    Relations {
        countries,
        pairs: rel.pairs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w() -> Weights {
        Weights {
            territory: 300,
            minority: 200,
            regime: 150,
            causes: 200,
            arms_race: 150,
            war: 1000,
            trade: 200,
            agreement: 200,
            common_enemy: 150,
            deterrence: 200,
        }
    }

    #[test]
    fn the_terms_add_up_and_the_sum_stays_in_range() {
        let t = Terms {
            territory: 1000,
            minority: 500,
            regime: 1000,
            ..Default::default()
        };
        assert_eq!(t.hostility(&w()), 300 + 100 + 150);
        let calm = Terms {
            trade: 1000,
            agreement: 1000,
            ..t
        };
        assert_eq!(calm.hostility(&w()), 150);
        let war = Terms {
            war: 1000,
            territory: 1000,
            ..Default::default()
        };
        assert_eq!(war.hostility(&w()), 1000);
        assert_eq!(Terms::default().hostility(&w()), 0);
    }
}
