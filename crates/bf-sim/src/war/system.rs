//! The forces in peace, every week (phase 5, slice A): equipment wears and is made good by what
//! the state bought, the reserve reports when mobilized, the size in peace follows the men of
//! service age once a year, and the soldiers each region holds are set every month.

use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::Weekday;
use laplace_schelling::schedule::{Access, Cadence, System, TickCtx};

use super::calc::{AgeBand, apportion, mobilizable};
use super::catalogue::{Domain, Military};
use super::state::{CountryWar, Formation, PeaceRules, PeaceSize, WAR, men};
use crate::game::GameWorld;
use crate::politics::Politics;
use crate::politics::Scope;

pub struct Forces {
    pub cat: Arc<crate::econ::Catalogue>,
    pub military: Arc<Military>,
    pub peace: Arc<PeaceRules>,
    pub politics: Option<Arc<Politics>>,
}

pub struct ForcesChanges {
    countries: Vec<CountryWar>,
    soldiers: Option<Vec<i64>>,
}

/// Merges a list of (good, quantity) into a sorted list.
fn add(list: &mut Vec<(u16, Fixed)>, g: u16, q: Fixed) {
    match list.binary_search_by_key(&g, |(k, _)| *k) {
        Ok(i) => list[i].1 += q,
        Err(i) => list.insert(i, (g, q)),
    }
}

/// Years of age of each group of the demography's pyramid (5-year groups, the last open).
fn bands(ages: &[i64]) -> Vec<AgeBand> {
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

    /// Whether `country` can equip generation `generation` of type `ty`: every technology it needs
    /// made at home (Industrial stage or past it), or diffused somewhere, so it can be bought.
    fn can_equip(&self, world: &GameWorld, country: usize, ty: u16, generation: usize) -> bool {
        let spec = self.military.spec(ty, generation);
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
            let (unit, colonial, training) = (f.unit, f.colonial, f.training);
            let to = match c.formations.iter().position(|g| {
                g.unit == unit
                    && g.generation as usize == next
                    && g.colonial == colonial
                    && !g.called_up
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

    fn country_week(
        &self,
        ctx: &TickCtx,
        world: &GameWorld,
        ci: usize,
        cw: &CountryWar,
    ) -> CountryWar {
        let mil = &self.military;
        let rules = &mil.rules;
        let mut c = cw.clone();
        let Some(m) = world.econ.markets.get(ci) else {
            return c;
        };
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
            if let Some(inf) = mil.unit("infantry") {
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
        if new_year && self.peace.peace_size == PeaceSize::ShareOfServiceAge && !c.mobilized {
            let of_age = self.men_of_service_age(world, ci);
            let land =
                |f: &Formation| mil.types[f.unit as usize].domain == Domain::Land && !f.called_up;
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
        let military_weight: i64 = self
            .cat
            .state_basket
            .iter()
            .filter(|(g, _)| self.cat.military[*g as usize])
            .map(|(_, w)| *w)
            .sum();
        let room = crate::econ::public::goods_budget(m)
            .max(Fixed::ZERO)
            .mul_ratio(military_weight, 1000);
        let weeks = i64::from(self.peace.replace_weeks.max(1));
        let price = |list: &[(u16, Fixed)]| -> Fixed {
            list.iter().map(|(g, q)| *q * m.price[*g as usize]).sum()
        };
        let needs = |c: &CountryWar| {
            let mut orders: Vec<(u16, Fixed)> = Vec::new();
            let mut repair: Vec<(u16, Fixed)> = Vec::new();
            for f in &c.formations {
                let spec = mil.spec(f.unit, f.generation as usize);
                for (g, q) in &spec.upkeep {
                    add(&mut orders, *g, *q * f.elements * self.peace.upkeep_scale);
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
        };
        let (first, _) = needs(&c);
        let spare = room - price(&first);
        if spare.is_positive() && !c.mobilized {
            self.modernize(world, ci, &mut c, spare * weeks, &m.price);
        }
        let (mut orders, mut repair) = needs(&c);
        let cost = price(&orders);
        let mut upgrade: Vec<(u16, Fixed)> = Vec::new();
        if cost > room && cost.is_positive() {
            // Short of the budget, everything is cut alike, so what is ordered is what is bought.
            let cut = room / cost;
            for (_, q) in orders.iter_mut().chain(repair.iter_mut()) {
                *q *= cut;
            }
        } else {
            // The rest buys the military goods in the proportions of the state's military basket
            // of before the forces: the arms industry of 1925 is the size of that demand, and
            // spending it on what the formations hold (mostly ships) would ask the yards for
            // twice what they make, leaving the money unspent. What a formation uses goes to its
            // equipment; ammunition bought this way is used up in training.
            let left = room - cost;
            if left.is_positive() {
                let basket: Vec<(u16, i64)> = self
                    .cat
                    .state_basket
                    .iter()
                    .filter(|(g, _)| self.cat.military[*g as usize])
                    .filter(|(g, _)| m.price[*g as usize].is_positive())
                    .copied()
                    .collect();
                let total: i64 = basket.iter().map(|(_, w)| *w).sum();
                for (g, w) in basket {
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
        c
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
            };
        }
        let countries: Vec<CountryWar> = world
            .war
            .countries
            .iter()
            .enumerate()
            .map(|(ci, cw)| self.country_week(ctx, world, ci, cw))
            .collect();
        let month_start = ctx.date.day <= 7 || world.war.soldiers.is_empty();
        let soldiers = month_start.then(|| place_soldiers(&self.military, &countries, &world.econ));
        ForcesChanges {
            countries,
            soldiers,
        }
    }

    fn apply(&self, _ctx: &TickCtx, changes: ForcesChanges, world: &mut GameWorld) {
        if changes.countries.is_empty() {
            return;
        }
        world.war.countries = changes.countries;
        if let Some(s) = changes.soldiers {
            world.war.soldiers = s;
        }
    }
}

/// What the state buys in a week when it has forces. The civilian part of the state's basket
/// (goods not of the `military` category) keeps the money it had before the forces, its weights
/// over 1,000 of the goods budget; the forces' orders take at most the rest, scaled down together
/// when they cost more. What they leave stays in the treasury, where the budget rules turn it
/// into public works and less debt.
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
