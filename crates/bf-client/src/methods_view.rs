//! The production methods of the buildings the player's country has (the economy window's
//! methods tab): for each building, every method with the technology it needs, its inputs,
//! outputs and jobs per level, its owners' profit per level at this month's prices and the
//! country's wage, how many levels run it, and which one the owners would pick now
//! (`monthly::best_method`, the rule of the yearly switch). Read only, once a month.
//!
//! The rule switches a plant once a year when another method earns 10 % more on its capital
//! (`yearly::SWITCH_MARGIN`); the country's investment fund pays the new equipment.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use bf_sim::research::TechTree;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// One method of a building.
#[derive(Clone, Debug, PartialEq)]
pub struct MethodRow {
    pub index: u8,
    pub key: String,
    /// Technology key that unlocks it, if any.
    pub tech: Option<String>,
    /// The country may build with it now.
    pub available: bool,
    /// Per level per year: (good key, quantity).
    pub inputs: Vec<(String, f64)>,
    pub outputs: Vec<(String, f64)>,
    /// Per level: (profession key, workers).
    pub jobs: Vec<(String, i64)>,
    /// Owners' profit of a level over a year at this month's prices, the country's wage and
    /// the mean productivity of its plants.
    pub profit: f64,
    /// Levels and states running it.
    pub levels: i64,
    pub states: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KindMethods {
    pub kind: u16,
    pub key: String,
    pub group: u8,
    pub levels: i64,
    pub methods: Vec<MethodRow>,
    /// The method the owners would pick now, if any can run.
    pub best: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodsView {
    pub month: (i32, u8),
    /// The buildings the country has, most levels first.
    pub kinds: Vec<KindMethods>,
}

pub fn read(
    world: &GameWorld,
    tag: &str,
    cat: &Catalogue,
    tree: Option<&TechTree>,
    date: Date,
) -> Option<MethodsView> {
    let ci = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let econ = &world.econ;
    let m = econ.markets.get(ci)?;
    let unlocks = bf_sim::econ::Unlocks::new(cat, date.year, tree, &world.research);
    let wage = bf_sim::econ::monthly::market_wages(econ, false)
        .get(ci)
        .copied()
        .unwrap_or(Fixed::ZERO);
    let good = |g: u16| {
        cat.goods
            .get(usize::from(g))
            .map_or_else(String::new, |x| x.key.clone())
    };
    // Levels, states and productivity × levels of each building's plants, by method.
    let n = cat.buildings.len();
    let mut levels: Vec<Vec<(i64, usize)>> = cat
        .buildings
        .iter()
        .map(|b| vec![(0, 0); b.methods.len()])
        .collect();
    let mut tfp: Vec<(f64, i64)> = vec![(0.0, 0); n];
    for r in econ.regions.iter().filter(|r| usize::from(r.market) == ci) {
        for p in r.plants.iter().filter(|p| p.levels > 0) {
            let (k, me) = (usize::from(p.kind), usize::from(p.method));
            let Some(b) = cat.buildings.get(k) else {
                continue;
            };
            if let Some(slot) = levels[k].get_mut(me) {
                slot.0 += p.levels;
                slot.1 += 1;
            }
            let t = bf_sim::econ::plant_tfp(r, m, b).to_f64_lossy();
            tfp[k].0 += t * p.levels as f64;
            tfp[k].1 += p.levels;
        }
    }
    // Goods that clear state by state (transport, power before the grid) are judged in each
    // state by the yearly switch; the market's figure for them says nothing: taken as met.
    let buy: Vec<Fixed> = m
        .buy_ratio
        .iter()
        .zip(&cat.goods)
        .map(|(b, g)| if g.local { Fixed::ONE } else { *b })
        .collect();
    let mut kinds = Vec::new();
    for (k, b) in cat.buildings.iter().enumerate() {
        let total: i64 = levels[k].iter().map(|x| x.0).sum();
        if total == 0 || b.public || b.informal || b.methods.len() < 2 {
            continue;
        }
        let mean_tfp = if tfp[k].1 > 0 {
            Fixed::from_f64_lossy(tfp[k].0 / tfp[k].1 as f64)
        } else {
            Fixed::ONE
        };
        let methods = b
            .methods
            .iter()
            .enumerate()
            .map(|(mi, me)| MethodRow {
                index: mi as u8,
                key: me.key.clone(),
                tech: tree
                    .and_then(|t| t.method_tech.get(k)?.get(mi).copied().flatten())
                    .and_then(|t| tree?.techs.get(usize::from(t)))
                    .map(|t| t.key.clone()),
                available: unlocks.method(cat, ci, k, mi),
                inputs: me
                    .inputs
                    .iter()
                    .map(|(g, q)| (good(*g), q.to_f64_lossy() * 52.0))
                    .collect(),
                outputs: me
                    .outputs
                    .iter()
                    .map(|(g, q)| (good(*g), q.to_f64_lossy() * 52.0))
                    .collect(),
                jobs: me
                    .jobs
                    .iter()
                    .map(|(p, w)| {
                        (
                            bf_sim::economy::PROFESSIONS
                                .get(usize::from(*p))
                                .map_or_else(String::new, |s| (*s).to_owned()),
                            *w,
                        )
                    })
                    .collect(),
                profit: me.profit(&m.price, mean_tfp, wage).to_f64_lossy() * 52.0,
                levels: levels[k][mi].0,
                states: levels[k][mi].1,
            })
            .collect();
        let best = bf_sim::econ::monthly::best_method(
            cat,
            k,
            &unlocks,
            ci,
            &m.price,
            &buy,
            mean_tfp,
            wage,
        )
        .map(|(mi, _)| mi);
        kinds.push(KindMethods {
            kind: k as u16,
            key: b.key.clone(),
            group: b.group_ix,
            levels: total,
            methods,
            best,
        });
    }
    kinds.sort_by(|a, b| b.levels.cmp(&a.levels).then(a.kind.cmp(&b.kind)));
    Some(MethodsView {
        month: (date.year, date.month),
        kinds,
    })
}
