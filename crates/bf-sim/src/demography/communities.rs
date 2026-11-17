//! Communities in the population: every state's people split by community (culture + religion)
//! and profession, the table `docs/population.pt-BR.md` calls "a linha demográfica". Rules in
//! that document ("Comunidades na simulação").
//!
//! The economy keeps its pops by profession; the rows here are a demographic layer over them.
//! After every demography month the rows of each profession add up to its pop exactly
//! ([`RegionRows::reconcile`]). Births go to the row that has them, immigrants to the row of
//! their culture, and once a year people assimilate from one row to another.

use std::collections::BTreeMap;

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use crate::communities::{CULTURE_FAMILIES, Communities, CommunityId, RELIGION_FAMILIES, RowRule};
use crate::econ::{Region, prof};

/// Id of the "other minorities" row.
pub const OTHERS: CommunityId = CommunityId::MAX;

/// Assimilation per year without resistance, parts per million: culture 1.5 %, religion 0.2 %
/// (decision 3 of `docs/architecture.pt-BR.md`, 2026-11-17).
pub const CULTURE_RATE: i128 = 15_000;
pub const RELIGION_RATE: i128 = 2_000;
/// Factor of the language (or religion) family: the same family as the target 1000, another 500.
const OTHER_FAMILY: i128 = 500;
/// Pull of the state's religion law on conversion to the country's majority religion, per mille,
/// from `church_privilege`: religion law imposed (theocratic law, 900) 1500; an established
/// church that tolerates others (500) 1000; separation or less 500.
fn religion_law(church_privilege: i32) -> i128 {
    if church_privilege >= 900 {
        1500
    } else if church_privilege >= 500 {
        1000
    } else {
        500
    }
}

/// A region's people by community and profession, saved and hashed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionRows {
    /// Community of each row; [`OTHERS`] is the last row when present.
    pub ids: Vec<CommunityId>,
    /// People of each row by profession (`prof`).
    pub people: Vec<[i64; prof::P]>,
    /// Who the "other minorities" are, by community, sorted by id; adds up to that row.
    pub others: Vec<(CommunityId, i64)>,
}

impl HashState for RegionRows {
    fn hash_state(&self, h: &mut StateHasher) {
        self.ids.hash_state(h);
        h.write_len(self.people.len());
        for p in &self.people {
            p.as_slice().hash_state(h);
        }
        h.write_len(self.others.len());
        for (id, n) in &self.others {
            id.hash_state(h);
            n.hash_state(h);
        }
    }
}

/// Splits `total` in proportion to `weights` (largest remainder, ties to the lower index); an
/// all-zero weight list puts everything on the first entry.
pub fn apportion(total: i64, weights: &[i128]) -> Vec<i64> {
    let n = weights.len();
    if n == 0 {
        return Vec::new();
    }
    let sum: i128 = weights.iter().map(|w| (*w).max(0)).sum();
    if sum <= 0 {
        let mut out = vec![0; n];
        out[0] = total;
        return out;
    }
    let t = i128::from(total);
    let mut out: Vec<i64> = weights
        .iter()
        .map(|w| (t * (*w).max(0) / sum) as i64)
        .collect();
    let mut left = total - out.iter().sum::<i64>();
    let mut rest: Vec<(i128, usize)> = weights
        .iter()
        .enumerate()
        .map(|(k, w)| ((t * (*w).max(0)) % sum, k))
        .collect();
    rest.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut k = 0;
    while left > 0 {
        out[rest[k % n].1] += 1;
        left -= 1;
        k += 1;
    }
    while left < 0 {
        // Negative totals do not occur; keep the sum exact anyway.
        let j = rest[k % n].1;
        out[j] -= 1;
        left += 1;
        k += 1;
    }
    out
}

impl RegionRows {
    pub fn total(&self) -> i64 {
        self.people.iter().flatten().sum()
    }

    pub fn row_total(&self, k: usize) -> i64 {
        self.people[k].iter().sum()
    }

    fn row_of(&self, id: CommunityId) -> Option<usize> {
        self.ids.iter().position(|&x| x == id)
    }

    /// Index of the "other minorities" row, added when missing.
    fn others_row(&mut self) -> usize {
        match self.row_of(OTHERS) {
            Some(k) => k,
            None => {
                self.ids.push(OTHERS);
                self.people.push([0; prof::P]);
                self.ids.len() - 1
            }
        }
    }

    /// Makes every profession's rows add up to its pop. A profession that grew takes the
    /// newcomers in the proportions of the whole state (provisional rule for migration between
    /// professions and states, which does not carry communities yet); one that shrank loses
    /// people in proportion to its rows. Then the make-up of "other minorities" is scaled to
    /// that row.
    pub fn reconcile(&mut self, r: &Region) {
        if self.ids.is_empty() {
            return;
        }
        let totals: Vec<i128> = (0..self.ids.len())
            .map(|k| i128::from(self.row_total(k)))
            .collect();
        for (p, pop) in r.pops.iter().enumerate() {
            let have: i64 = self.people.iter().map(|row| row[p]).sum();
            let want = pop.size.max(0);
            if have == want {
                continue;
            }
            let new = if want > have {
                let add = apportion(want - have, &totals);
                self.people
                    .iter()
                    .zip(add)
                    .map(|(row, a)| row[p] + a)
                    .collect::<Vec<_>>()
            } else {
                let w: Vec<i128> = self.people.iter().map(|row| i128::from(row[p])).collect();
                apportion(want, &w)
            };
            for (row, v) in self.people.iter_mut().zip(new) {
                row[p] = v;
            }
        }
        self.scale_others();
    }

    /// Scales the make-up of "other minorities" to the people of that row.
    fn scale_others(&mut self) {
        let Some(k) = self.row_of(OTHERS) else {
            self.others.clear();
            return;
        };
        let total = self.row_total(k);
        if self.others.is_empty() {
            // People without a recorded community: cannot happen with rows built here.
            return;
        }
        let w: Vec<i128> = self.others.iter().map(|(_, n)| i128::from(*n)).collect();
        for ((_, n), v) in self.others.iter_mut().zip(apportion(total, &w)) {
            *n = v;
        }
    }

    /// Adds `n` people of community `id` to profession `p`: its row, or "other minorities".
    pub fn add(&mut self, id: CommunityId, p: usize, n: i64) {
        if n <= 0 {
            return;
        }
        if let Some(k) = self.row_of(id) {
            self.people[k][p] += n;
            return;
        }
        let k = self.others_row();
        self.people[k][p] += n;
        match self.others.binary_search_by_key(&id, |e| e.0) {
            Ok(i) => self.others[i].1 += n,
            Err(i) => self.others.insert(i, (id, n)),
        }
    }

    /// People by community (rows and the make-up of "other minorities"), sorted by id.
    pub fn by_community(&self) -> Vec<(CommunityId, i64)> {
        let mut out: BTreeMap<CommunityId, i64> = BTreeMap::new();
        for (k, &id) in self.ids.iter().enumerate() {
            if id != OTHERS {
                *out.entry(id).or_default() += self.row_total(k);
            }
        }
        for &(id, n) in &self.others {
            *out.entry(id).or_default() += n;
        }
        out.into_iter().collect()
    }
}

/// What the simulation needs from the community catalogue, built once from the data.
#[derive(Debug)]
pub struct CommunityTable {
    pub catalogue: Communities,
    /// By community: index into [`CULTURE_FAMILIES`] of its culture and into
    /// [`RELIGION_FAMILIES`] of its religion (`u8::MAX` unknown).
    pub family: Vec<u8>,
    pub religion_family: Vec<u8>,
    /// Community of `(culture, religion)`.
    pub pair: BTreeMap<(u16, u16), CommunityId>,
    /// By culture: the community its immigrants join, the culture's most numerous community of
    /// 1925 (the diaspora records culture only); `OTHERS` when the culture has none.
    pub arrival: Vec<CommunityId>,
    /// `culture-religion` key of each community, as the politics names its targets.
    pub keys: Vec<String>,
}

impl CommunityTable {
    pub fn new(c: &Communities) -> Self {
        let n = c.communities.len();
        let fam = |key: &str, list: &[&str]| {
            list.iter()
                .position(|k| *k == key)
                .map_or(u8::MAX, |i| i as u8)
        };
        let family = c
            .communities
            .iter()
            .map(|&(cu, _)| fam(&c.cultures[cu as usize].family, CULTURE_FAMILIES))
            .collect();
        let religion_family = c
            .communities
            .iter()
            .map(|&(_, re)| fam(&c.religions[re as usize].family, RELIGION_FAMILIES))
            .collect();
        let pair = c
            .communities
            .iter()
            .enumerate()
            .map(|(i, &k)| (k, i as CommunityId))
            .collect();
        // Weight of each community in 1925: the sum of its shares over the states, a proxy
        // that needs no population (ties to the lower id).
        let mut weight = vec![0f64; n];
        for s in &c.states {
            for &(id, share) in &s.shares {
                weight[id as usize] += f64::from(share);
            }
        }
        let mut arrival = vec![OTHERS; c.cultures.len()];
        let mut best = vec![-1f64; c.cultures.len()];
        for (id, &(cu, _)) in c.communities.iter().enumerate() {
            if weight[id] > best[cu as usize] {
                best[cu as usize] = weight[id];
                arrival[cu as usize] = id as CommunityId;
            }
        }
        let keys = (0..n).map(|i| c.key(i as CommunityId)).collect();
        Self {
            catalogue: c.clone(),
            family,
            religion_family,
            pair,
            arrival,
            keys,
        }
    }

    pub fn culture(&self, id: CommunityId) -> u16 {
        self.catalogue.communities[id as usize].0
    }

    pub fn religion(&self, id: CommunityId) -> u16 {
        self.catalogue.communities[id as usize].1
    }

    /// The rows of every region from the 1925 compositions and the decided row rule, with the
    /// region's pops split among them in proportion to the shares (every community in every
    /// profession alike: no data on who works where).
    pub fn initial_rows(
        &self,
        regions: &[Region],
        state_key: impl Fn(u16) -> Option<String>,
    ) -> Vec<RegionRows> {
        let mut population: BTreeMap<String, f64> = BTreeMap::new();
        for r in regions {
            if let Some(k) = state_key(r.state) {
                *population.entry(k).or_default() += r.population() as f64;
            }
        }
        let rows = self.catalogue.rows(&RowRule::PROPOSED, &population);
        let by_state: BTreeMap<&str, usize> = rows
            .iter()
            .enumerate()
            .map(|(i, s)| (s.state.as_str(), i))
            .collect();
        let mix: BTreeMap<&str, &crate::communities::StateMix> = self
            .catalogue
            .states
            .iter()
            .map(|m| (m.state.as_str(), m))
            .collect();
        regions
            .iter()
            .map(|r| {
                let Some(key) = state_key(r.state) else {
                    return RegionRows::default();
                };
                let (Some(&i), Some(m)) = (by_state.get(key.as_str()), mix.get(key.as_str()))
                else {
                    return RegionRows::default();
                };
                let sr = &rows[i];
                let share = |id: CommunityId| -> i128 {
                    m.shares
                        .iter()
                        .find(|e| e.0 == id)
                        .map_or(0, |e| (f64::from(e.1) * 1e6).round() as i128)
                };
                let mut ids = sr.rows.clone();
                let mut weights: Vec<i128> = ids.iter().map(|&id| share(id)).collect();
                let others_w: i128 = sr
                    .others
                    .iter()
                    .map(|e| (f64::from(e.1) * 1e6).round() as i128)
                    .sum();
                if !sr.others.is_empty() {
                    ids.push(OTHERS);
                    weights.push(others_w);
                }
                let mut people = vec![[0i64; prof::P]; ids.len()];
                for (p, pop) in r.pops.iter().enumerate() {
                    for (row, v) in people.iter_mut().zip(apportion(pop.size.max(0), &weights)) {
                        row[p] = v;
                    }
                }
                let mut out = RegionRows {
                    ids,
                    people,
                    others: Vec::new(),
                };
                if let Some(k) = out.row_of(OTHERS) {
                    let w: Vec<i128> = sr
                        .others
                        .iter()
                        .map(|e| (f64::from(e.1) * 1e6).round() as i128)
                        .collect();
                    let mut others: Vec<(CommunityId, i64)> = sr
                        .others
                        .iter()
                        .map(|e| e.0)
                        .zip(apportion(out.row_total(k), &w))
                        .collect();
                    others.sort_unstable();
                    out.others = others;
                }
                out
            })
            .collect()
    }
}

/// Diffusion of the fertility transition along language families (docs, "Cultura e religião na
/// natalidade"): for each family, its people and those of them living in states already in
/// transition last month, world-wide.
pub struct Diffusion {
    pub people: Vec<i64>,
    pub moving: Vec<i64>,
    /// Population-weighted mean of the family values over the world, the centre of the term.
    pub mean: Fixed,
}

/// Children per woman at or under which a state counts as in transition: a tenth below the
/// pre-transition 6.5 (the Princeton dating of the onset, a 10 % fall).
pub const IN_TRANSITION: Fixed = Fixed::from_raw(5850);

impl Diffusion {
    pub fn new(table: &CommunityTable, rows: &[RegionRows], tfr: &[Fixed]) -> Self {
        let nf = CULTURE_FAMILIES.len();
        let mut people = vec![0i64; nf];
        let mut moving = vec![0i64; nf];
        for (rr, &t) in rows.iter().zip(tfr) {
            let moved = t > Fixed::ZERO && t <= IN_TRANSITION;
            for (id, n) in rr.by_community() {
                let f = table.family[id as usize];
                if usize::from(f) < nf {
                    people[f as usize] += n;
                    if moved {
                        moving[f as usize] += n;
                    }
                }
            }
        }
        let total: i64 = people.iter().sum();
        let mean = if total > 0 {
            Fixed::from_ratio(moving.iter().sum::<i64>(), total)
        } else {
            Fixed::ZERO
        };
        Self {
            people,
            moving,
            mean,
        }
    }

    /// Value of the term for each row of a region, 0–1: the share of its language family
    /// outside this state that already lives in transition ("other minorities": the mean of
    /// its make-up). `own_moving` says whether this state was in transition last month.
    pub fn row_values(
        &self,
        table: &CommunityTable,
        rr: &RegionRows,
        own_moving: bool,
    ) -> Vec<Fixed> {
        let value = |id: CommunityId, n_own: i64| -> Fixed {
            let f = table.family[id as usize] as usize;
            if f >= self.people.len() {
                return self.mean;
            }
            let people = self.people[f] - n_own;
            let moving = self.moving[f] - if own_moving { n_own } else { 0 };
            if people > 0 {
                Fixed::from_ratio(moving.max(0), people)
            } else {
                Fixed::ZERO
            }
        };
        // People of each family in this state, taken out of the world sums.
        let mut own: BTreeMap<u8, i64> = BTreeMap::new();
        for (id, n) in rr.by_community() {
            *own.entry(table.family[id as usize]).or_default() += n;
        }
        let own_of = |id: CommunityId| own.get(&table.family[id as usize]).copied().unwrap_or(0);
        rr.ids
            .iter()
            .map(|&id| {
                if id != OTHERS {
                    return value(id, own_of(id));
                }
                let total: i64 = rr.others.iter().map(|e| e.1).sum();
                if total <= 0 {
                    return self.mean;
                }
                let s: i128 = rr
                    .others
                    .iter()
                    .map(|&(c, n)| i128::from(value(c, own_of(c)).raw()) * i128::from(n))
                    .sum();
                Fixed::from_raw((s / i128::from(total)) as i64)
            })
            .collect()
    }
}

/// What assimilation needs to know about a country, by territory.
pub struct AssimilationLaw {
    /// Primary culture of the territory (its largest), `u16::MAX` when none.
    pub culture: u16,
    /// Majority religion of the territory, `u16::MAX` when none.
    pub religion: u16,
    /// Colonies: no cultural assimilation without an explicit law (docs).
    pub colony: bool,
    /// Pull of the religion law, per mille.
    pub religion_law: i128,
    /// By community targeted by the country's laws: (assimilation per mille, discrimination
    /// per mille). Communities not listed: 1000 and 0.
    pub targeted: BTreeMap<CommunityId, (i128, i128)>,
}

impl AssimilationLaw {
    pub fn new(culture: u16, religion: u16, colony: bool, church_privilege: i32) -> Self {
        Self {
            culture,
            religion,
            colony,
            religion_law: religion_law(church_privilege),
            targeted: BTreeMap::new(),
        }
    }
}

/// People who moved this year, for the record.
#[derive(Clone, Copy, Debug, Default)]
pub struct Assimilated {
    pub culture: i64,
    pub religion: i64,
    /// People whose target community (primary culture with their religion, or their culture
    /// with the majority religion) is not in the catalogue: they stay.
    pub blocked: i64,
}

/// One year of assimilation in a region: every community whose culture is not the territory's
/// primary culture passes a share to (primary culture, same religion), and every community whose
/// religion is not the majority's passes a share to (same culture, majority religion):
///
///     a = rate × law × (1 − discrimination) × (1 − share in the state)² × family
///
/// Rates and factors per mille or per million; integer arithmetic.
pub fn assimilate(
    table: &CommunityTable,
    law: &AssimilationLaw,
    rr: &mut RegionRows,
) -> Assimilated {
    let mut done = Assimilated::default();
    let total = rr.total();
    if total <= 0 {
        return done;
    }
    // Every flow is decided on the year's opening numbers.
    let start = rr.by_community();
    let mut flows: Vec<(CommunityId, CommunityId, i64)> = Vec::new();
    for &(id, n) in &start {
        if n <= 0 {
            continue;
        }
        let (cu, re) = (table.culture(id), table.religion(id));
        let (law_a, disc) = law.targeted.get(&id).copied().unwrap_or((1000, 0));
        let s = i128::from(n) * 1_000_000 / i128::from(total); // share in the state, ppm
        let concentration = (1_000_000 - s) * (1_000_000 - s) / 1_000_000; // ppm
        let open = (1000 - disc).max(0); // per mille
        // Culture: the colonies only under a law that pushes it.
        let culture_on =
            law.culture != u16::MAX && cu != law.culture && (!law.colony || law_a > 1000);
        if culture_on {
            let fam = |c: u16| {
                table
                    .catalogue
                    .cultures
                    .get(c as usize)
                    .map(|d| d.family.as_str())
            };
            let f = if fam(cu) == fam(law.culture) {
                1000
            } else {
                OTHER_FAMILY
            };
            let ppm =
                CULTURE_RATE * law_a / 1000 * open / 1000 * concentration / 1_000_000 * f / 1000;
            let moved = (i128::from(n) * ppm / 1_000_000) as i64;
            if moved > 0 {
                match table.pair.get(&(law.culture, re)) {
                    Some(&to) => flows.push((id, to, moved)),
                    None => done.blocked += moved,
                }
            }
        }
        if law.religion != u16::MAX && re != law.religion {
            let rf = |r: u16| {
                table
                    .catalogue
                    .religions
                    .get(r as usize)
                    .map(|d| d.family.as_str())
            };
            let f = if rf(re) == rf(law.religion) {
                1000
            } else {
                OTHER_FAMILY
            };
            let ppm = RELIGION_RATE * law.religion_law / 1000 * open / 1000 * concentration
                / 1_000_000
                * f
                / 1000;
            let moved = (i128::from(n) * ppm / 1_000_000) as i64;
            if moved > 0 {
                match table.pair.get(&(cu, law.religion)) {
                    Some(&to) => flows.push((id, to, moved)),
                    None => done.blocked += moved,
                }
            }
        }
    }
    for (from, to, n) in flows {
        let taken = take(rr, from, n);
        if table.culture(from) != table.culture(to) {
            done.culture += taken.iter().sum::<i64>();
        } else {
            done.religion += taken.iter().sum::<i64>();
        }
        for (p, &v) in taken.iter().enumerate() {
            rr.add(to, p, v);
        }
    }
    done
}

/// Takes up to `n` people of community `id` out of the region, by profession in proportion to
/// where they are; returns how many left each profession.
fn take(rr: &mut RegionRows, id: CommunityId, n: i64) -> [i64; prof::P] {
    let mut out = [0i64; prof::P];
    if let Some(k) = rr.row_of(id) {
        let n = n.min(rr.row_total(k));
        let w: Vec<i128> = rr.people[k].iter().map(|v| i128::from(*v)).collect();
        for (p, v) in apportion(n, &w).into_iter().enumerate() {
            rr.people[k][p] -= v;
            out[p] = v;
        }
        return out;
    }
    let Some(k) = rr.row_of(OTHERS) else {
        return out;
    };
    let Ok(i) = rr.others.binary_search_by_key(&id, |e| e.0) else {
        return out;
    };
    let n = n.min(rr.others[i].1);
    rr.others[i].1 -= n;
    let w: Vec<i128> = rr.people[k].iter().map(|v| i128::from(*v)).collect();
    for (p, v) in apportion(n, &w).into_iter().enumerate() {
        rr.people[k][p] -= v;
        out[p] = v;
    }
    rr.others.retain(|e| e.1 > 0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apportion_keeps_the_total() {
        assert_eq!(apportion(10, &[1, 1, 1]), vec![4, 3, 3]);
        assert_eq!(apportion(7, &[0, 0]), vec![7, 0]);
        assert_eq!(apportion(100, &[3, 0, 1]).iter().sum::<i64>(), 100);
    }

    #[test]
    fn religion_law_reads_church_privilege() {
        assert_eq!(religion_law(900), 1500);
        assert_eq!(religion_law(500), 1000);
        assert_eq!(religion_law(100), 500);
        assert_eq!(religion_law(-800), 500);
    }
}
