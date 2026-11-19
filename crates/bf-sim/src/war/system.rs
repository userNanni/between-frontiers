//! The forces in peace, every week (phase 5, slice A): equipment wears and is made good by what
//! the state bought, the reserve reports when mobilized, the size in peace follows the men of
//! service age once a year, and the soldiers each region holds are set every month.

use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::Weekday;
use laplace_schelling::schedule::{Access, Cadence, System, TickCtx};

use super::calc::{AgeBand, apportion, mobilizable};
use super::catalogue::{Domain, Military, Mobility};
use super::state::{CountryWar, Formation, PeaceRules, PeaceSize, WAR, men};
use crate::game::GameWorld;
use crate::politics::Politics;
use crate::politics::Scope;

pub struct Forces {
    pub cat: Arc<crate::econ::Catalogue>,
    pub military: Arc<Military>,
    pub peace: Arc<PeaceRules>,
    pub politics: Option<Arc<Politics>>,
    /// The military goods of the state's basket and their weights over 1,000, resolved at load.
    pub basket: Vec<(u16, i64)>,
    /// The type the reservists report to.
    pub infantry: Option<u16>,
    /// Crises (slice B); `None` without `crisis.ron`.
    pub crisis: Option<Arc<super::crisis::CrisisSetup>>,
    /// The war on land (slice C); `None` with `Switches::war` off.
    pub fight: Option<Arc<super::fight::FightSetup>>,
}

pub struct ForcesChanges {
    countries: Vec<CountryWar>,
    soldiers: Option<Vec<i64>>,
    crises: Option<super::crisis::CrisisWeek>,
    fight: Option<super::fight::FightWeek>,
    /// The conversions under way, every country's, while the war is switched on.
    conversions: Option<Vec<super::composition::Conversion>>,
}

/// [`join`], for the conversions of the composition.
pub(crate) fn join_formation(
    c: &mut CountryWar,
    key: (u16, u8, bool, bool),
    elements: Fixed,
    training: Fixed,
    equipment: Vec<(u16, Fixed)>,
) {
    join(c, key, elements, training, equipment)
}

/// Merges a list of (good, quantity) into a sorted list.
pub fn add_goods(list: &mut Vec<(u16, Fixed)>, g: u16, q: Fixed) {
    add(list, g, q)
}

fn add(list: &mut Vec<(u16, Fixed)>, g: u16, q: Fixed) {
    match list.binary_search_by_key(&g, |(k, _)| *k) {
        Ok(i) => list[i].1 += q,
        Err(i) => list.insert(i, (g, q)),
    }
}

fn held(list: &[(u16, Fixed)], g: u16) -> Fixed {
    list.iter()
        .find(|(k, _)| *k == g)
        .map_or(Fixed::ZERO, |e| e.1)
}

/// Puts `elements` of a type into the formation of the same kind, or a new one, diluting its
/// training by the newcomers' and adding their equipment.
fn join(
    c: &mut CountryWar,
    (unit, generation, colonial, raised): (u16, u8, bool, bool),
    elements: Fixed,
    training: Fixed,
    equipment: Vec<(u16, Fixed)>,
) {
    let i = match c.formations.iter().position(|f| {
        f.unit == unit
            && f.generation == generation
            && f.colonial == colonial
            && f.raised == raised
            && !f.called_up
    }) {
        Some(i) => i,
        None => {
            c.formations.push(Formation {
                unit,
                generation,
                elements: Fixed::ZERO,
                colonial,
                called_up: false,
                equipment: Vec::new(),
                training,
                raised,
            });
            c.formations.len() - 1
        }
    };
    let f = &mut c.formations[i];
    let total = f.elements + elements;
    if total.is_positive() {
        f.training = (f.training * f.elements + training * elements) / total;
    }
    f.elements = total;
    for (g, q) in equipment {
        add(&mut f.equipment, g, q);
    }
}

/// Whether `country` can equip generation `generation` of type `ty`: every technology it needs
/// made at home (Industrial stage or past it), or diffused somewhere, so it can be bought.
pub fn can_equip(
    mil: &Military,
    world: &GameWorld,
    country: usize,
    ty: u16,
    generation: usize,
) -> bool {
    let spec = mil.spec(ty, generation);
    let r = &world.research;
    if r.is_empty() {
        return false;
    }
    spec.requires.iter().all(|&t| {
        let t = t as usize;
        let home = r.stage.get(country).is_some_and(|s| {
            s.get(t)
                .is_some_and(|&st| st >= crate::research::Stage::Industrial)
        });
        let abroad = r
            .firsts
            .get(t)
            .is_some_and(|f| f[crate::research::Stage::Diffused as usize] > 0);
        home || abroad
    })
}

/// The newest generation of type `ty` that `country` can equip, if any.
pub fn newest_generation(mil: &Military, world: &GameWorld, country: usize, ty: u16) -> Option<u8> {
    let n = mil.types[ty as usize].generations.len();
    (0..n)
        .rev()
        .find(|&g| can_equip(mil, world, country, ty, g))
        .map(|g| g as u8)
}

/// A list of (good, quantity), sorted by good.
pub type Goods = Vec<(u16, Fixed)>;

/// What the forces of `c` need this week: what they consume, and the replacement of what they
/// lack over `replace_weeks`; the second list is the replacement alone.
pub fn needs_of(
    mil: &Military,
    peace: &PeaceRules,
    c: &CountryWar,
    combat: bool,
) -> (Goods, Goods) {
    let weeks = i64::from(peace.replace_weeks.max(1));
    let mut orders: Vec<(u16, Fixed)> = Vec::new();
    let mut repair: Vec<(u16, Fixed)> = Vec::new();
    for f in &c.formations {
        let spec = mil.spec(f.unit, f.generation as usize);
        // At war on land, the land forces fire `combat_ammunition` times their training rounds.
        let in_combat =
            combat && mil.types[f.unit as usize].domain == super::catalogue::Domain::Land;
        for (g, q) in &spec.upkeep {
            let q = if in_combat && *g == mil.rules.ammunition {
                *q * mil.rules.combat_ammunition
            } else {
                *q
            };
            add(&mut orders, *g, q * f.elements * peace.upkeep_scale);
        }
        for (g, per) in &spec.equipment {
            let have = f
                .equipment
                .iter()
                .find(|(k, _)| k == g)
                .map_or(Fixed::ZERO, |e| e.1);
            let lack = (*per * f.elements - have).max(Fixed::ZERO);
            if lack.is_positive() {
                add(&mut repair, *g, lack / weeks);
            }
        }
    }
    for (g, q) in &repair {
        add(&mut orders, *g, *q);
    }
    (orders, repair)
}

/// Years of age of each group of the demography's pyramid (5-year groups, the last open).
pub(crate) fn bands(ages: &[i64]) -> Vec<AgeBand> {
    ages.iter()
        .enumerate()
        .map(|(k, people)| AgeBand {
            from: 5 * k as u32,
            to: 5 * k as u32 + 5,
            // Men: half the people of the group.
            men: (*people).max(0) as u64 / 2,
        })
        .collect()
}

impl Forces {
    pub fn new(
        cat: Arc<crate::econ::Catalogue>,
        military: Arc<Military>,
        peace: Arc<PeaceRules>,
        politics: Option<Arc<Politics>>,
        crisis: Option<Arc<super::crisis::CrisisSetup>>,
        fight: Option<Arc<super::fight::FightSetup>>,
    ) -> Self {
        let basket: Vec<(u16, i64)> = cat
            .state_basket
            .iter()
            .filter(|(g, _)| cat.military.get(*g as usize).copied().unwrap_or(false))
            .copied()
            .collect();
        let infantry = military.unit("infantry");
        Self {
            cat,
            military,
            peace,
            politics,
            basket,
            infantry,
            crisis,
            fight,
        }
    }

    fn conscription(&self, world: &GameWorld, country: usize) -> i32 {
        let (Some(p), Some(e)) = (
            &self.politics,
            crate::politics::effect_index("conscription"),
        ) else {
            return 0;
        };
        if world.politics.laws.is_empty() {
            return 0;
        }
        crate::polity::effects_in_force(p, &world.politics, country, Scope::Metropole)[e]
    }

    /// Men of service age in the metropole: the army in peace is drawn from them.
    fn men_of_service_age(&self, world: &GameWorld, country: usize) -> u64 {
        let rules = &self.military.rules;
        world
            .econ
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.market as usize == country && !r.colony)
            .filter_map(|(i, _)| world.demography.ages.get(i))
            .map(|a| mobilizable(rules, &bands(a), 1000, 0).of_age)
            .sum()
    }

    fn can_equip(&self, world: &GameWorld, country: usize, ty: u16, generation: usize) -> bool {
        can_equip(&self.military, world, country, ty, generation)
    }

    /// Moves a share of each formation to the next generation it can equip, as far as `money`
    /// (the room left over the replacement weeks) pays for the equipment the newer generation
    /// needs beyond what the men bring. The moved elements keep their equipment and training;
    /// what they lack is bought as replacement, over the following weeks.
    fn modernize(
        &self,
        world: &GameWorld,
        ci: usize,
        c: &mut CountryWar,
        mut money: Fixed,
        prices: &[Fixed],
        mut class_room: Option<(Fixed, Fixed)>,
    ) {
        let mil = &self.military;
        let per_week = Fixed::from_ratio(1, 52 * i64::from(self.peace.modernize_years.max(1)));
        let n = c.formations.len();
        for i in 0..n {
            if !money.is_positive() {
                break;
            }
            let f = &c.formations[i];
            if f.called_up || !f.elements.is_positive() {
                continue;
            }
            let next = f.generation as usize + 1;
            if next >= mil.types[f.unit as usize].generations.len()
                || !self.can_equip(world, ci, f.unit, next)
            {
                continue;
            }
            let old = &mil.spec(f.unit, f.generation as usize).equipment;
            let new = &mil.spec(f.unit, next).equipment;
            let extra: Vec<(u16, Fixed)> = new
                .iter()
                .map(|(g, q)| {
                    let had = old
                        .iter()
                        .find(|(k, _)| k == g)
                        .map_or(Fixed::ZERO, |e| e.1);
                    (*g, (*q - had).max(Fixed::ZERO))
                })
                .collect();
            let unit_cost: Fixed = extra.iter().map(|(g, q)| *q * prices[*g as usize]).sum();
            let mut moved = f.elements * per_week;
            if unit_cost.is_positive() {
                moved = moved.min(money / unit_cost);
            }
            // A newer generation on another class of mobility (the mechanized infantry from
            // lorries to tracks) moves only as far as the composition's target of that class
            // has room: the modernization does not overfill a bucket the conversions keep.
            let (from_class, to_class) = (
                mil.spec(f.unit, f.generation as usize).mobility,
                mil.spec(f.unit, next).mobility,
            );
            if let Some((rw, rt)) = class_room.as_mut()
                && from_class != to_class
            {
                let room = match to_class {
                    Mobility::Wheeled => Some(rw),
                    Mobility::Tracked => Some(rt),
                    _ => None,
                };
                if let Some(room) = room {
                    moved = moved.min(*room);
                    *room -= moved.max(Fixed::ZERO);
                }
            }
            if !moved.is_positive() {
                continue;
            }
            money -= moved * unit_cost;
            let share = moved / f.elements;
            let f = &mut c.formations[i];
            let carried: Vec<(u16, Fixed)> =
                f.equipment.iter().map(|(g, q)| (*g, *q * share)).collect();
            for (e, (_, q)) in f.equipment.iter_mut().zip(&carried) {
                e.1 -= *q;
            }
            f.elements -= moved;
            let (unit, colonial, training, raised) = (f.unit, f.colonial, f.training, f.raised);
            let to = match c.formations.iter().position(|g| {
                g.unit == unit
                    && g.generation as usize == next
                    && g.colonial == colonial
                    && !g.called_up
                    && g.raised == raised
            }) {
                Some(j) => j,
                None => {
                    c.formations.push(Formation {
                        unit,
                        generation: next as u8,
                        elements: Fixed::ZERO,
                        colonial,
                        called_up: false,
                        equipment: Vec::new(),
                        training,
                        raised,
                    });
                    c.formations.len() - 1
                }
            };
            let t = &mut c.formations[to];
            let total = t.elements + moved;
            t.training = (t.training * t.elements + training * moved) / total;
            t.elements = total;
            for (g, q) in carried {
                add(&mut t.equipment, g, q);
            }
        }
        c.formations
            .retain(|f| f.elements.is_positive() || f.called_up);
    }

    fn newest(&self, world: &GameWorld, country: usize, ty: u16) -> Option<u8> {
        newest_generation(&self.military, world, country, ty)
    }

    /// Ships, aircraft and strategic forces whose equipment is complete join the forces, with
    /// their crews untrained: aircraft one by one, ships and strategic forces the whole order at
    /// once; orders fully delivered leave the queue.
    fn deliver(&self, c: &mut CountryWar) {
        let mil = &self.military;
        let mut done = Vec::new();
        for k in 0..c.queue.len() {
            let o = &c.queue[k];
            let (Some(ty), Some(generation)) = (mil.unit(&o.unit), o.generation) else {
                continue;
            };
            if mil.types[ty as usize].domain == Domain::Land {
                continue;
            }
            let per = &mil.spec(ty, generation as usize).equipment;
            // Whole elements still to deliver, the last one perhaps a part.
            let left = (o.ordered - o.delivered + Fixed::from_raw(Fixed::SCALE - 1)).to_int_floor();
            let ready = per
                .iter()
                .filter(|(_, q)| q.is_positive())
                .map(|(g, q)| (held(&o.built, *g) / *q).to_int_floor())
                .min()
                .unwrap_or(left)
                .min(left);
            // Ships and strategic forces are built side by side and join together at the end
            // of their build weeks; aircraft leave the line one by one.
            let ready = if mil.types[ty as usize].domain == Domain::Air || ready >= left {
                ready
            } else {
                0
            };
            if ready <= 0 {
                continue;
            }
            let n = Fixed::from_int(ready).min(o.ordered - o.delivered);
            let equipment: Vec<(u16, Fixed)> = per.iter().map(|(g, q)| (*g, *q * n)).collect();
            let o = &mut c.queue[k];
            for (g, q) in &equipment {
                if let Ok(i) = o.built.binary_search_by_key(g, |(k, _)| *k) {
                    o.built[i].1 = (o.built[i].1 - *q).max(Fixed::ZERO);
                }
            }
            o.delivered += n;
            let colonial = o.colonial;
            if o.delivered >= o.ordered {
                done.push(k);
            }
            join(
                c,
                (ty, generation, colonial, false),
                n,
                Fixed::ZERO,
                equipment,
            );
        }
        for k in done.into_iter().rev() {
            let o = c.queue.remove(k);
            // What was built beyond the last element goes to the depot.
            for (g, q) in o.built {
                if q.is_positive() {
                    add(&mut c.depot, g, q);
                }
            }
        }
    }

    /// Pays the orders from `money`, the military budget left this week once the needs are
    /// paid, in the order they were placed. Land orders raise a class of men this week, as far
    /// as the money pays the replacement of their equipment over `replace_weeks`; the others
    /// draw on the depot and buy this week's part of their equipment. Returns what is left.
    fn place_orders(
        &self,
        world: &GameWorld,
        ci: usize,
        c: &mut CountryWar,
        mut money: Fixed,
        prices: &[Fixed],
    ) -> Fixed {
        let mil = &self.military;
        let weeks = i64::from(self.peace.replace_weeks.max(1));
        let mut raised = Vec::new();
        let CountryWar { queue, depot, .. } = c;
        for o in queue.iter_mut() {
            o.buying.clear();
            let Some(ty) = mil.unit(&o.unit) else {
                continue;
            };
            if o.generation.is_none() {
                o.generation = self.newest(world, ci, ty);
            }
            let Some(generation) = o.generation else {
                continue;
            };
            if !money.is_positive() {
                continue;
            }
            let per = &mil.spec(ty, generation as usize).equipment;
            let unit_cost: Fixed = per.iter().map(|(g, q)| *q * prices[*g as usize]).sum();
            let left = o.ordered - o.delivered;
            if mil.types[ty as usize].domain == Domain::Land {
                let pace = o.ordered / i64::from(self.peace.raise_weeks.max(1));
                let mut n = pace.min(left);
                if unit_cost.is_positive() {
                    n = n.min(money * weeks / unit_cost);
                }
                if !n.is_positive() {
                    continue;
                }
                money -= n * unit_cost / weeks;
                o.delivered += n;
                raised.push(((ty, generation, o.colonial, true), n));
            } else {
                // This week's part of the whole order, short of what is built: from the depot
                // first, the rest bought. The depot does not shorten the build, it pays for it.
                let build = i64::from(self.peace.build_weeks(&mil.types[ty as usize].key));
                for (g, q) in per {
                    let want = (*q * left - held(&o.built, *g)).max(Fixed::ZERO);
                    let mut pace = (*q * o.ordered / build).min(want);
                    if let Ok(i) = depot.binary_search_by_key(g, |(k, _)| *k) {
                        let take = pace.min(depot[i].1);
                        if take.is_positive() {
                            depot[i].1 -= take;
                            add(&mut o.built, *g, take);
                            pace -= take;
                        }
                    }
                    let p = prices[*g as usize];
                    if !pace.is_positive() || !p.is_positive() {
                        continue;
                    }
                    let q = pace.min(money / p);
                    if q.is_positive() {
                        money -= q * p;
                        add(&mut o.buying, *g, q);
                    }
                }
            }
        }
        for (key, n) in raised {
            // Recruits report untrained and without equipment: the depot and the replacement
            // purchases equip them.
            join(c, key, n, Fixed::ZERO, Vec::new());
        }
        c.queue
            .retain(|o| o.delivered < o.ordered || o.built.iter().any(|(_, q)| q.is_positive()));
        money
    }

    fn country_week(
        &self,
        ctx: &TickCtx,
        world: &GameWorld,
        ci: usize,
        cw: &CountryWar,
    ) -> (CountryWar, Vec<super::composition::Conversion>) {
        let mil = &self.military;
        let rules = &mil.rules;
        let mut c = cw.clone();
        let mut conversions: Vec<super::composition::Conversion> =
            world.war.composition.of(ci as u16).cloned().collect();
        let Some(m) = world.econ.markets.get(ci) else {
            return (c, conversions);
        };
        let newest = |ty: u16| self.newest(world, ci, ty);
        // 1. Last week's purchases make good the equipment, to each formation by what it lacks.
        for (g, q) in &cw.repair {
            let got = *q * m.buy_ratio.get(*g as usize).copied().unwrap_or(Fixed::ZERO);
            let lacks: Vec<Fixed> = c
                .formations
                .iter()
                .map(|f| {
                    let spec = mil.spec(f.unit, f.generation as usize);
                    let want = spec
                        .equipment
                        .iter()
                        .find(|(k, _)| k == g)
                        .map_or(Fixed::ZERO, |(_, per)| *per * f.elements);
                    let have = f
                        .equipment
                        .iter()
                        .find(|(k, _)| k == g)
                        .map_or(Fixed::ZERO, |e| e.1);
                    (want - have).max(Fixed::ZERO)
                })
                .collect();
            let total: Fixed = lacks.iter().copied().sum();
            if !total.is_positive() {
                continue;
            }
            for (f, lack) in c.formations.iter_mut().zip(&lacks) {
                if lack.is_positive() {
                    add(&mut f.equipment, *g, got.mul_div(*lack, total).min(*lack));
                }
            }
        }
        // Equipment bought above the needs goes to the depot (the war reserve); formations that
        // lack equipment draw on it before anything is bought for them: reservists called up,
        // a force moving to a newer generation, and the week's wear.
        for (g, q) in &cw.upgrade {
            let got = *q * m.buy_ratio.get(*g as usize).copied().unwrap_or(Fixed::ZERO);
            add(&mut c.depot, *g, got);
        }
        if !c.depot.is_empty() {
            for f in &mut c.formations {
                for (g, per) in &mil.spec(f.unit, f.generation as usize).equipment {
                    let have = f
                        .equipment
                        .iter()
                        .find(|(k, _)| k == g)
                        .map_or(Fixed::ZERO, |e| e.1);
                    let lack = *per * f.elements - have;
                    if !lack.is_positive() {
                        continue;
                    }
                    if let Ok(i) = c.depot.binary_search_by_key(g, |(k, _)| *k) {
                        let take = lack.min(c.depot[i].1);
                        if take.is_positive() {
                            c.depot[i].1 -= take;
                            add(&mut f.equipment, *g, take);
                        }
                    }
                }
            }
        }
        // Orders (slice A2): last week's purchases go to what is being built, and whatever is
        // complete joins the forces.
        for o in &mut c.queue {
            for (g, q) in std::mem::take(&mut o.buying) {
                let got = q * m.buy_ratio.get(g as usize).copied().unwrap_or(Fixed::ZERO);
                add(&mut o.built, g, got);
            }
        }
        self.deliver(&mut c);
        // 2. Training rises a week at a time toward complete, over the type's training weeks.
        for f in &mut c.formations {
            let weeks = mil
                .spec(f.unit, f.generation as usize)
                .training_weeks
                .max(1);
            f.training = (f.training + Fixed::from_ratio(1, i64::from(weeks))).min(Fixed::ONE);
        }
        // Wear: the catalogue's maintenance share a week, about a tenth of the value a year, in
        // the formations and in the depot alike.
        for f in &mut c.formations {
            for e in &mut f.equipment {
                e.1 -= e.1 * rules.maintenance;
            }
        }
        for e in &mut c.depot {
            e.1 -= e.1 * rules.maintenance;
        }
        // 3. The reserve reports when mobilized, a share each week; goes back when demobilized.
        if c.mobilized && c.reserve > 0 {
            if let Some(inf) = self.infantry {
                let weeks = u64::from(rules.reserve.call_up_weeks.max(1));
                let n = (c.reserve / weeks).max(1000).min(c.reserve);
                c.reserve -= n;
                let add_elements = Fixed::from_ratio(n as i64, 1000);
                match c.formations.iter_mut().find(|f| f.called_up) {
                    Some(f) => f.elements += add_elements,
                    None => c.formations.push(Formation {
                        unit: inf,
                        generation: 0,
                        elements: add_elements,
                        colonial: false,
                        called_up: true,
                        equipment: Vec::new(),
                        training: Fixed::from_ratio(i64::from(rules.reserve.training), 1000),
                        raised: false,
                    }),
                }
            }
        } else if !c.mobilized {
            let back: i64 = c
                .formations
                .iter()
                .filter(|f| f.called_up)
                .map(|f| men(mil, f))
                .sum();
            c.reserve += back.max(0) as u64;
            c.formations.retain(|f| !f.called_up);
        }
        // 4. Once a year, the size in peace follows the men of service age and the law.
        let new_year = ctx.date.month == 1 && ctx.date.day <= 7;
        // The composition of the forces, once a year, while the war is switched on.
        if new_year
            && self.fight.is_some()
            && let Some(rule) = &self.peace.composition
        {
            super::composition::compose(
                mil,
                rule,
                ci as u16,
                &c,
                world.war.composition.fixed_of(ci as u16),
                &newest,
                &mut conversions,
            );
        }
        if new_year && self.peace.peace_size == PeaceSize::ShareOfServiceAge && !c.mobilized {
            let of_age = self.men_of_service_age(world, ci);
            // Forces raised by an order stay outside the size in peace, and so do the colonial
            // ones: a colonial army was set by the metropole's budget and policy, not by its
            // men of service age (scaled by them, the colonies lost hands as the metropole
            // grew: docs/war.pt-BR.md, "Forças em paz").
            let land = |f: &Formation| {
                mil.types[f.unit as usize].domain == Domain::Land
                    && !f.called_up
                    && !f.raised
                    && !f.colonial
            };
            let land_men: i64 = c
                .formations
                .iter()
                .filter(|f| land(f))
                .map(|f| men(mil, f))
                .sum();
            let law = self.conscription(world, ci);
            if of_age > 0 && c.peace_share == Fixed::ZERO {
                let tag = &world
                    .countries
                    .iter()
                    .nth(ci)
                    .map(|(_, s)| s.tag.clone())
                    .unwrap_or_default();
                c.peace_share = match self.peace.share_override.iter().find(|(t, _)| t == tag) {
                    Some((_, pm)) => Fixed::from_ratio(i64::from(*pm), 1000),
                    None => Fixed::from_ratio(land_men, of_age as i64),
                };
                c.law_then = law;
            } else if of_age > 0 && land_men > 0 {
                let mut target = c.peace_share * of_age as i64;
                if self.peace.law_scaling && c.law_then > 0 && law > 0 {
                    target = target.mul_ratio(i64::from(law), i64::from(c.law_then));
                }
                let factor = target / land_men;
                for f in c.formations.iter_mut().filter(|f| land(f)) {
                    f.elements *= factor;
                    // Recruits join untrained: the average falls by the share they are.
                    if factor > Fixed::ONE {
                        f.training /= factor;
                    }
                }
            }
        }
        // 5. This week's orders. The military budget is the part of the goods budget that is not
        // the civilian basket's, as before the forces. It pays first what the forces consume and
        // what makes good their equipment; with room left, part of the force moves to a newer
        // generation, and what is still left buys equipment above the needs.
        // The ceiling of lever D3: the military share the government fixed, or the basket's.
        let room = crate::econ::public::military_room(m, &self.cat);
        let weeks = i64::from(self.peace.replace_weeks.max(1));
        let price = |list: &[(u16, Fixed)]| -> Fixed {
            list.iter().map(|(g, q)| *q * m.price[*g as usize]).sum()
        };
        // At war on land (`Switches::war`), the forces order the ammunition of combat.
        let combat = self.fight.is_some()
            && world
                .war
                .conflicts
                .wars
                .iter()
                .any(|w| w.side_a.contains(&(ci as u16)) || w.side_b.contains(&(ci as u16)));
        let needs = |c: &CountryWar| needs_of(mil, &self.peace, c, combat);
        let (first, _) = needs(&c);
        let mut spare = room - price(&first);
        let before: Vec<(Fixed, Fixed)> = c
            .queue
            .iter()
            .map(|o| (o.delivered, o.built.iter().map(|(_, q)| *q).sum()))
            .collect();
        if spare.is_positive() && !c.queue.is_empty() {
            spare = self.place_orders(world, ci, &mut c, spare, &m.price);
        }
        // The conversions of the composition, from what the orders left; without it they wait,
        // and count the week as stalled.
        if !conversions.is_empty() {
            spare = super::composition::convert_week(
                mil,
                &mut c,
                &mut conversions,
                spare,
                &m.price,
                self.peace.raise_weeks,
                self.peace.replace_weeks,
                &newest,
            );
        }
        // Weeks in a row an order got nothing for want of room in the budget.
        for (o, (delivered, built)) in c.queue.iter_mut().zip(before) {
            let got = o.delivered != delivered
                || !o.buying.is_empty()
                || o.built.iter().map(|(_, q)| *q).sum::<Fixed>() != built;
            o.stalled = if got || o.generation.is_none() {
                0
            } else {
                o.stalled + 1
            };
        }
        if spare.is_positive() && !c.mobilized {
            // The room left in each class of the composition's target, while it runs.
            let class_room = self
                .fight
                .as_ref()
                .and(self.peace.composition.as_ref())
                .and_then(|rule| {
                    let from = mil.unit(&rule.from_unit)?;
                    let (tw, tt) = super::composition::target(
                        rule,
                        world.war.composition.fixed_of(ci as u16),
                        newest(from),
                    )?;
                    let (w, t, all) = super::composition::now(mil, &c, &conversions);
                    Some((
                        (all.mul_ratio(i64::from(tw), 1000) - w).max(Fixed::ZERO),
                        (all.mul_ratio(i64::from(tt), 1000) - t).max(Fixed::ZERO),
                    ))
                });
            self.modernize(world, ci, &mut c, spare * weeks, &m.price, class_room);
        }
        let (mut orders, mut repair) = needs(&c);
        for o in &c.queue {
            for (g, q) in &o.buying {
                add(&mut orders, *g, *q);
            }
        }
        let cost = price(&orders);
        let mut upgrade: Vec<(u16, Fixed)> = Vec::new();
        if cost > room && cost.is_positive() {
            // Short of the budget, everything is cut alike, so what is ordered is what is bought.
            let cut = room / cost;
            for (_, q) in orders.iter_mut().chain(repair.iter_mut()) {
                *q *= cut;
            }
            for o in &mut c.queue {
                for (_, q) in o.buying.iter_mut() {
                    *q *= cut;
                }
            }
        } else {
            // The rest buys the military goods in the proportions of the state's military basket
            // of before the forces: the arms industry of 1925 is the size of that demand, and
            // spending it on what the formations hold (mostly ships) would ask the yards for
            // twice what they make, leaving the money unspent. What a formation uses goes to its
            // equipment; ammunition bought this way is used up in training.
            let left = room - cost;
            if left.is_positive() {
                let priced = || {
                    self.basket
                        .iter()
                        .copied()
                        .filter(|(g, _)| m.price[*g as usize].is_positive())
                };
                let total: i64 = priced().map(|(_, w)| w).sum();
                for (g, w) in priced() {
                    let q = left.mul_ratio(w, total.max(1)) / m.price[g as usize];
                    if q.is_positive() {
                        add(&mut upgrade, g, q);
                        add(&mut orders, g, q);
                    }
                }
            }
        }
        c.upgrade = upgrade;
        c.orders = orders;
        c.repair = repair;
        (c, conversions)
    }
}

/// Soldiers (workers) per region: each country's men, the colonial formations in its colonies
/// and the rest in the metropole, spread by population.
pub fn place_soldiers(
    mil: &Military,
    countries: &[CountryWar],
    econ: &crate::econ::Economy,
) -> Vec<i64> {
    let regions = &econ.regions;
    let mut out = vec![0i64; regions.len()];
    for (ci, c) in countries.iter().enumerate() {
        let (mut metro, mut colony) = (0i64, 0i64);
        for f in &c.formations {
            if f.colonial {
                colony += men(mil, f);
            } else {
                metro += men(mil, f);
            }
        }
        let mine: Vec<usize> = (0..regions.len())
            .filter(|&i| regions[i].market as usize == ci)
            .collect();
        if !mine.iter().any(|&i| regions[i].colony) {
            metro += colony;
            colony = 0;
        }
        for (colonial, total) in [(false, metro), (true, colony)] {
            let place: Vec<usize> = mine
                .iter()
                .copied()
                .filter(|&i| regions[i].colony == colonial)
                .collect();
            let weights: Vec<Fixed> = place
                .iter()
                .map(|&i| Fixed::from_int(regions[i].population()))
                .collect();
            for (&i, n) in place.iter().zip(apportion(total.max(0) as u64, &weights)) {
                out[i] = n as i64;
            }
        }
    }
    out
}

/// Puts the soldiers of the forces in the pops on the start date: each region's soldier pop
/// becomes the one whose workers are the region's soldiers, the difference taken from or given
/// to its peasants (labourers when the peasants run out). People are conserved.
pub fn station_soldiers(mil: &Military, world: &mut GameWorld) {
    use crate::econ::prof;
    let soldiers = place_soldiers(mil, &world.war.countries, &world.econ);
    for (r, &want) in world.econ.regions.iter_mut().zip(&soldiers) {
        let size = (Fixed::from_int(want) / crate::econ::WORKER_SHARE).to_int_round();
        let mut gap = size - r.pops[prof::SOLDIER].size;
        for from in [prof::PEASANT, prof::LABORER] {
            let n = if gap > 0 {
                gap.min(r.pops[from].size)
            } else {
                gap
            };
            r.pops[from].size -= n;
            r.pops[prof::SOLDIER].size += n;
            gap -= n;
            if gap == 0 {
                break;
            }
        }
    }
    world.war.soldiers = soldiers;
}

impl System<GameWorld> for Forces {
    type Changes = ForcesChanges;

    fn name(&self) -> &'static str {
        "forces"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Weekly(Weekday::Monday)
    }

    fn access(&self) -> Access {
        Access::new()
            .read(crate::econ::week::ECON)
            .read(crate::demography::DEMOGRAPHY)
            .read(crate::polity::POLITY)
            .read(crate::research_system::RESEARCH)
            .write(WAR)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> ForcesChanges {
        if world.war.is_empty() {
            return ForcesChanges {
                countries: Vec::new(),
                soldiers: None,
                crises: None,
                fight: None,
                conversions: None,
            };
        }
        // A country born since (a separation, a liberation, a revolt) gets its forces' entry,
        // empty: without it, it had no forces, no military budget and no war support.
        let empty = CountryWar::default();
        let (countries, conversions): (Vec<CountryWar>, Vec<_>) =
            (0..world.countries.len().max(world.war.countries.len()))
                .map(|ci| {
                    let cw = world.war.countries.get(ci).unwrap_or(&empty);
                    self.country_week(ctx, world, ci, cw)
                })
                .unzip();
        let conversions = self
            .fight
            .is_some()
            .then(|| conversions.into_iter().flatten().collect::<Vec<_>>());
        let month_start = ctx.date.day <= 7 || world.war.soldiers.is_empty();
        let soldiers = month_start.then(|| place_soldiers(&self.military, &countries, &world.econ));
        // Crises run only when there are any: a world in peace pays nothing for them.
        let crises = self
            .crisis
            .as_ref()
            .filter(|_| {
                !world.war.conflicts.crises.is_empty() || !world.war.conflicts.calls.is_empty()
            })
            .map(|setup| super::crisis::crisis_week(setup, &self.military, world));
        // The war on land while a war is under way, and the first week of each month the
        // generation of the armies (step 4), in peace too while the war is switched on.
        let fight = self
            .fight
            .as_ref()
            .filter(|_| !world.war.conflicts.wars.is_empty() || ctx.date.day <= 7)
            .map(|setup| super::fight::fight_week(setup, &self.military, world, ctx.date));
        ForcesChanges {
            countries,
            soldiers,
            crises,
            fight,
            conversions,
        }
    }

    fn apply(&self, ctx: &TickCtx, changes: ForcesChanges, world: &mut GameWorld) {
        if changes.countries.is_empty() {
            return;
        }
        world.war.countries = changes.countries;
        let colonial = self.fight.as_ref().and_then(|f| f.colonial.clone());
        // A week after a peace, the colonies the union took back (G6, GC5).
        if let Some(k) = &colonial
            && !world.war.colonial.is_empty()
        {
            super::colonial::reconquered(k, world);
        }
        if let Some(c) = changes.conversions {
            world.war.composition.conversions = c;
        }
        if let Some(week) = changes.fight {
            for (c, share) in &week.lost_share {
                if let Some(cw) = world.war.countries.get_mut(*c as usize) {
                    super::fight::take_losses(&self.military, cw, *share);
                }
            }
            for (c, men) in &week.lost_men {
                let killed = (*men as i64 * super::fight::KILLED_PER_MILLE / 1000).max(0) as u64;
                super::fight::bury(world, *c, killed);
            }
            // Occupation money, from the owner's treasury to the occupier's (G3).
            for (owner, by, amount) in &week.transfers {
                if let Some(m) = world.econ.markets.get_mut(*owner as usize) {
                    m.treasury -= *amount;
                }
                if let Some(m) = world.econ.markets.get_mut(*by as usize) {
                    m.treasury += *amount;
                }
                let (o, b) = (*owner as usize, *by as usize);
                if o.max(b) < world.econ.markets.len() {
                    world.payments.transfer(o, b, ctx.date.year, *amount);
                }
            }
            for (r, levels) in &week.devastation.levels {
                super::occupation::wreck(world, *r, *levels);
            }
            for (r, n) in &week.devastation.civilians {
                super::occupation::kill_civilians(world, *r, *n);
            }
            // A government fallen at the end of a collapse (G4): a new form of government is a
            // change of regime, which the treaties read (owner's decision P3).
            if let Some(k) = self.fight.as_ref().and_then(|f| f.conduct.as_ref()) {
                for (c, fall) in &week.support.falls {
                    if let (super::support::Fall::Regime { from, to }, Some(g)) =
                        (fall, k.government)
                        && let Some(slot) = world
                            .politics
                            .laws
                            .get_mut(*c as usize)
                            .and_then(|v| v.get_mut(g))
                    {
                        *slot = Some(*to);
                        world.politics.changes.push(crate::polity::LawChangeRecord {
                            date: ctx.date,
                            country: *c,
                            from: *from,
                            to: *to,
                            by: crate::polity::Enactor::Movement(k.regime_change.unwrap_or(0)),
                        });
                    }
                }
            }
            let tag = |m: u16| {
                world
                    .countries
                    .iter()
                    .nth(m as usize)
                    .map(|(_, c)| c.tag.clone())
            };
            let asks: Vec<_> = week
                .support
                .proposals
                .iter()
                .filter_map(|(from, to, p)| Some((tag(*from)?, tag(*to)?, p.clone())))
                .collect();
            for (from, to, p) in asks {
                world.treaties.table.propose(&from, &to, p, ctx.date);
            }
            world.war.fighting = week.fighting;
            // A peace signed at the table ends its war: the sides go home from mobilization.
            let ended = &week.ended;
            let gone: Vec<u16> = world
                .war
                .conflicts
                .wars
                .iter()
                .filter(|w| ended.contains(&w.id))
                .flat_map(|w| w.side_a.iter().chain(&w.side_b).copied())
                .collect();
            if let Some(k) = &colonial {
                // The colony that lost: the union, by the treaty's bridge.
                let cat = self.crisis.as_ref().and_then(|c| c.clauses.as_ref());
                for w in world
                    .war
                    .conflicts
                    .wars
                    .clone()
                    .iter()
                    .filter(|w| week.support.surrendered.contains(&w.id))
                {
                    if let Some(cat) = cat
                        && let Ok(s) = crate::treaty_exec::settle(
                            cat,
                            &w.demand,
                            &format!("colonial-{}", w.id),
                        )
                    {
                        crate::treaty_exec::queue(world, &s, ctx.date.year);
                    }
                }
                super::colonial::end(k, world, ended, &week.support.withdrawn);
            }
            world.war.conflicts.wars.retain(|w| !ended.contains(&w.id));
            // Out of the war, out of the support; in peace, no front.
            let wars = &world.war.conflicts.wars;
            if wars.is_empty() {
                world.war.fighting.last_week.clear();
            }
            world.war.fighting.support.retain(|x| {
                wars.iter()
                    .any(|w| w.side_a.contains(&x.country) || w.side_b.contains(&x.country))
            });
            for c in gone {
                if !world
                    .war
                    .conflicts
                    .wars
                    .iter()
                    .any(|w| w.side_a.contains(&c) || w.side_b.contains(&c))
                    && let Some(cw) = world.war.countries.get_mut(c as usize)
                {
                    cw.mobilized = false;
                }
            }
        }
        if let Some(s) = changes.soldiers {
            world.war.soldiers = s;
        }
        if let Some(week) = changes.crises {
            world.war.conflicts = week.conflicts;
            for s in &week.settled {
                crate::treaty_exec::queue(world, s, ctx.date.year);
            }
            for (call, honour) in &week.lapsed {
                if *honour {
                    super::third::join(world, call.war, call.party, call.side);
                } else {
                    super::third::betray(world, call, ctx.date);
                }
            }
        }
        // The colonial wars (G6): a revolt of the last seven days opens one; under way, the
        // metropole lands men and the new country recruits.
        if let Some(k) = &colonial {
            super::colonial::open(k, &self.military, world, ctx.date);
            if !world.war.colonial.is_empty() {
                super::colonial::week(k, &self.military, world);
            }
        }
    }
}

/// What the state buys in a week when it has forces. The civilian part of the state's basket
/// (goods not of the `military` category) keeps the money it had before the forces, its weights
/// over 1,000 of the goods budget; the forces' orders take at most the rest, scaled down together
/// when they cost more. What they leave stays in the treasury, where the budget rules turn it
/// into public works and less debt.
/// [`state_purchases`] with the military share the government fixed (D3): the civilian goods buy
/// with their weights per million (`econ::public::fixed_basket`), and the forces' orders fit in
/// `room` (`econ::public::military_room`).
pub fn state_purchases_fixed(
    cat: &crate::econ::Catalogue,
    basket: impl Iterator<Item = (u16, i64)>,
    military: &[(u16, Fixed)],
    budget: Fixed,
    room: Fixed,
    price: &[Fixed],
) -> Vec<(u16, Fixed)> {
    let budget = budget.max(Fixed::ZERO);
    let mut out: Vec<(u16, Fixed)> = basket
        .filter(|(g, _)| !cat.military.get(*g as usize).copied().unwrap_or(false))
        .map(|(g, w)| (g, budget.mul_ratio(w, 1_000_000) / price[g as usize]))
        .collect();
    let cost: Fixed = military.iter().map(|(g, q)| *q * price[*g as usize]).sum();
    let scale = if cost > room && cost.is_positive() {
        room / cost
    } else {
        Fixed::ONE
    };
    out.extend(
        military
            .iter()
            .filter(|(g, q)| q.is_positive() && price[*g as usize].is_positive())
            .map(|(g, q)| (*g, *q * scale)),
    );
    out
}

pub fn state_purchases<'a>(
    cat: &crate::econ::Catalogue,
    basket: impl Iterator<Item = &'a (u16, i64)>,
    military: &[(u16, Fixed)],
    budget: Fixed,
    price: &[Fixed],
) -> Vec<(u16, Fixed)> {
    let budget = budget.max(Fixed::ZERO);
    let mut out: Vec<(u16, Fixed)> = Vec::new();
    let mut civil_money = Fixed::ZERO;
    for &(g, w) in basket {
        if cat.military.get(g as usize).copied().unwrap_or(false) {
            continue;
        }
        let money = budget.mul_ratio(w, 1000);
        civil_money += money;
        out.push((g, money / price[g as usize]));
    }
    let room = (budget - civil_money).max(Fixed::ZERO);
    let cost: Fixed = military.iter().map(|(g, q)| *q * price[*g as usize]).sum();
    let scale = if cost > room && cost.is_positive() {
        room / cost
    } else {
        Fixed::ONE
    };
    out.extend(
        military
            .iter()
            .filter(|(g, q)| q.is_positive() && price[*g as usize].is_positive())
            .map(|(g, q)| (*g, *q * scale)),
    );
    out
}
