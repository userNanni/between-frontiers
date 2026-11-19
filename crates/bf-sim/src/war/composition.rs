//! The composition of the forces (front pace, after step 4): once a year the infantry converts
//! toward a share of the land men on wheels and on tracks. The government may fix the target
//! (`GameCommand::SetComposition`); otherwise the historical rule gives it, by the newest
//! generation of the infantry's equipment the country makes. The army changes equipment, it does
//! not grow: the men come from the infantry with their training, the equipment that serves goes
//! with them, the rest to the depot, and the new type's lack is bought as replacement, so a
//! formation shows in conversion, part equipped, until the market delivers. Behind the war
//! switch; reasoning in `docs/war.pt-BR.md`, "Composição das forças".

use std::collections::BTreeMap;

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::catalogue::{Domain, Military, Mobility};
use super::state::CountryWar;

/// The historical rule (`data/military/peace.ron`, `composition`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompositionRule {
    /// (generation of the infantry's equipment the country makes, wheels per mille, tracks per
    /// mille of the land men), in ascending generations; the newest reached applies.
    pub targets: Vec<(u8, u32, u32)>,
    /// Land men converted a year at most, per mille of the land men.
    pub per_year: u32,
    /// The types the infantry converts into, on wheels and on tracks, and the one it converts.
    pub wheels_unit: String,
    pub tracks_unit: String,
    pub from_unit: String,
}

/// A conversion under way: `ordered` elements of `from` becoming `unit`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversion {
    pub country: u16,
    pub unit: String,
    pub from: String,
    /// The generation laid down, the newest the country could equip when the conversion was
    /// first paid; `None` until then.
    pub generation: Option<u8>,
    pub ordered: Fixed,
    pub delivered: Fixed,
    /// Weeks in a row it got nothing: no room in the military budget, or no infantry left.
    pub stalled: u32,
}

/// The targets the governments fixed and the conversions under way, sorted by country.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionState {
    /// (country, [wheels, tracks] per mille): the government's target in place of the rule.
    pub fixed: Vec<(u16, [u16; 2])>,
    pub conversions: Vec<Conversion>,
}

impl CompositionState {
    pub fn is_empty(&self) -> bool {
        self.fixed.is_empty() && self.conversions.is_empty()
    }

    /// The target `country`'s government fixed, if any.
    pub fn fixed_of(&self, country: u16) -> Option<[u16; 2]> {
        self.fixed
            .binary_search_by_key(&country, |(c, _)| *c)
            .ok()
            .map(|i| self.fixed[i].1)
    }

    /// Fixes `country`'s target, or returns it to the rule (`None`). Each share at most 1,000,
    /// both together at most 1,000.
    pub fn set(&mut self, country: u16, target: Option<[u16; 2]>) {
        let at = self.fixed.binary_search_by_key(&country, |(c, _)| *c);
        match (at, target) {
            (Ok(i), None) => {
                self.fixed.remove(i);
            }
            (Err(_), None) => {}
            (at, Some([w, t])) => {
                let w = w.min(1000);
                let v = (country, [w, t.min(1000 - w)]);
                match at {
                    Ok(i) => self.fixed[i] = v,
                    Err(i) => self.fixed.insert(i, v),
                }
            }
        }
    }

    /// The conversions of `country`.
    pub fn of(&self, country: u16) -> impl Iterator<Item = &Conversion> {
        self.conversions
            .iter()
            .filter(move |x| x.country == country)
    }
}

/// The land elements of `c` at home (not called up, not colonial) on wheels, on tracks, and
/// all of them; the conversions under way count as done.
pub fn now(mil: &Military, c: &CountryWar, conversions: &[Conversion]) -> (Fixed, Fixed, Fixed) {
    let (mut wheels, mut tracks, mut all) = (Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
    let mut count = |m: Mobility, n: Fixed| match m {
        Mobility::Wheeled => wheels += n,
        Mobility::Tracked => tracks += n,
        _ => {}
    };
    for f in &c.formations {
        if mil.types[f.unit as usize].domain != Domain::Land || f.called_up || f.colonial {
            continue;
        }
        all += f.elements;
        count(mil.spec(f.unit, f.generation as usize).mobility, f.elements);
    }
    for x in conversions {
        if let Some(ty) = mil.unit(&x.unit) {
            let g = usize::from(x.generation.unwrap_or(0))
                .min(mil.types[ty as usize].generations.len() - 1);
            count(mil.spec(ty, g).mobility, x.ordered - x.delivered);
        }
    }
    (wheels, tracks, all)
}

/// The year's new conversions of `country` toward the target: the government's when fixed,
/// the rule's otherwise (the target of the newest generation `newest` gives for the infantry).
/// At most `per_year` of the land men a year, never more than the infantry at home, and only
/// into the types the country can equip.
pub fn compose(
    mil: &Military,
    rule: &CompositionRule,
    country: u16,
    c: &CountryWar,
    fixed: Option<[u16; 2]>,
    newest: &dyn Fn(u16) -> Option<u8>,
    conversions: &mut Vec<Conversion>,
) {
    let (Some(from), Some(wty), Some(tty)) = (
        mil.unit(&rule.from_unit),
        mil.unit(&rule.wheels_unit),
        mil.unit(&rule.tracks_unit),
    ) else {
        return;
    };
    let target = match fixed {
        Some([w, t]) => Some((u32::from(w), u32::from(t))),
        None => newest(from).and_then(|g| {
            rule.targets
                .iter()
                .rfind(|(level, _, _)| *level <= g)
                .map(|(_, w, t)| (*w, *t))
        }),
    };
    let Some((tw, tt)) = target else {
        return;
    };
    let (wheels, tracks, all) = now(mil, c, conversions);
    if !all.is_positive() {
        return;
    }
    let gap =
        |target: u32, have: Fixed| (all.mul_ratio(i64::from(target), 1000) - have).max(Fixed::ZERO);
    let mut gw = if newest(wty).is_some() {
        gap(tw, wheels)
    } else {
        Fixed::ZERO
    };
    let mut gt = if newest(tty).is_some() {
        gap(tt, tracks)
    } else {
        Fixed::ZERO
    };
    let infantry: Fixed = c
        .formations
        .iter()
        .filter(|f| f.unit == from && !f.called_up && !f.colonial)
        .map(|f| f.elements)
        .sum();
    let pending: Fixed = conversions
        .iter()
        .filter(|x| x.from == rule.from_unit)
        .map(|x| x.ordered - x.delivered)
        .sum();
    let cap = all
        .mul_ratio(i64::from(rule.per_year), 1000)
        .min((infantry - pending).max(Fixed::ZERO));
    let want = gw + gt;
    if want > cap && want.is_positive() {
        gw = gw.mul_div(cap, want);
        gt = cap - gw;
    }
    for (unit, n) in [(&rule.wheels_unit, gw), (&rule.tracks_unit, gt)] {
        if n.is_positive() {
            conversions.push(Conversion {
                country,
                unit: unit.clone(),
                from: rule.from_unit.clone(),
                generation: None,
                ordered: n,
                delivered: Fixed::ZERO,
                stalled: 0,
            });
        }
    }
}

/// Pays this week's part of the conversions from `money` (what the military budget leaves
/// after the needs and the orders): a class of men a week over `raise_weeks`, as far as the
/// money pays the replacement of the new type's equipment over `replace_weeks`. Returns what is
/// left; finished conversions leave the list.
#[allow(clippy::too_many_arguments)]
pub fn convert_week(
    mil: &Military,
    c: &mut CountryWar,
    conversions: &mut Vec<Conversion>,
    mut money: Fixed,
    prices: &[Fixed],
    raise_weeks: u32,
    replace_weeks: u32,
    newest: &dyn Fn(u16) -> Option<u8>,
) -> Fixed {
    let weeks = i64::from(replace_weeks.max(1));
    let mut source: BTreeMap<u16, Fixed> = BTreeMap::new();
    for f in c.formations.iter().filter(|f| !f.called_up && !f.colonial) {
        *source.entry(f.unit).or_default() += f.elements;
    }
    let mut moves = Vec::new();
    for x in conversions.iter_mut() {
        let (Some(ty), Some(src)) = (mil.unit(&x.unit), mil.unit(&x.from)) else {
            continue;
        };
        if x.generation.is_none() {
            x.generation = newest(ty);
        }
        let Some(generation) = x.generation else {
            x.stalled += 1;
            continue;
        };
        let per = &mil.spec(ty, usize::from(generation)).equipment;
        let unit_cost: Fixed = per.iter().map(|(g, q)| *q * prices[*g as usize]).sum();
        let avail = source.get(&src).copied().unwrap_or(Fixed::ZERO);
        let mut n = (x.ordered / i64::from(raise_weeks.max(1)))
            .min(x.ordered - x.delivered)
            .min(avail);
        if unit_cost.is_positive() && money.is_positive() {
            n = n.min(money * weeks / unit_cost);
        } else if unit_cost.is_positive() {
            n = Fixed::ZERO;
        }
        if !n.is_positive() {
            x.stalled += 1;
            continue;
        }
        x.stalled = 0;
        money -= n * unit_cost / weeks;
        x.delivered += n;
        source.insert(src, avail - n);
        moves.push((src, ty, generation, n));
    }
    for (src, ty, generation, n) in moves {
        convert(mil, c, src, (ty, generation), n);
    }
    conversions.retain(|x| x.delivered < x.ordered);
    money
}

/// Moves `n` elements of type `src` (at home, not called up) to type `ty` of `generation`: the
/// men keep their training; the equipment the new type uses goes with them, the rest to the
/// depot.
fn convert(mil: &Military, c: &mut CountryWar, src: u16, (ty, generation): (u16, u8), n: Fixed) {
    let home = |f: &super::state::Formation| f.unit == src && !f.called_up && !f.colonial;
    let total: Fixed = c
        .formations
        .iter()
        .filter(|f| home(f))
        .map(|f| f.elements)
        .sum();
    if !total.is_positive() {
        return;
    }
    let n = n.min(total);
    let uses: Vec<u16> = mil
        .spec(ty, usize::from(generation))
        .equipment
        .iter()
        .map(|(g, _)| *g)
        .collect();
    let (mut training, mut carried) = (Fixed::ZERO, Vec::new());
    for f in c.formations.iter_mut().filter(|f| home(f)) {
        let share = n.mul_div(f.elements, total);
        let part = if f.elements.is_positive() {
            share / f.elements
        } else {
            Fixed::ZERO
        };
        training += f.training * share;
        for e in &mut f.equipment {
            let q = e.1 * part;
            e.1 -= q;
            super::system::add_goods(&mut carried, e.0, q);
        }
        f.elements -= share;
    }
    let training = training / n;
    let mut keep = Vec::new();
    for (g, q) in carried {
        if uses.contains(&g) {
            keep.push((g, q));
        } else {
            super::system::add_goods(&mut c.depot, g, q);
        }
    }
    c.formations
        .retain(|f| f.elements.is_positive() || f.called_up);
    super::system::join_formation(c, (ty, generation, false, false), n, training, keep);
}

impl HashState for CompositionState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.fixed.len());
        for (c, [w, t]) in &self.fixed {
            h.write_u16(*c);
            h.write_u16(*w);
            h.write_u16(*t);
        }
        h.write_len(self.conversions.len());
        for x in &self.conversions {
            h.write_u16(x.country);
            for s in [&x.unit, &x.from] {
                h.write_len(s.len());
                for b in s.bytes() {
                    h.write_u8(b);
                }
            }
            match x.generation {
                None => h.write_u8(0),
                Some(g) => {
                    h.write_u8(1);
                    h.write_u8(g);
                }
            }
            x.ordered.hash_state(h);
            x.delivered.hash_state(h);
            h.write_u32(x.stalled);
        }
    }
}
