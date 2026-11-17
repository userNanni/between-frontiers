//! Phase 2 on the engine: research effort, the four stages of every technology in every
//! country, and what they change in the economy (productivity by building group, demography,
//! literacy). Rules in `docs/research.pt-BR.md`.

use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};

use crate::econ::week::ECON;
use crate::econ::{Catalogue, Unlocks, milli};
use crate::economy::BUILDING_GROUPS;
use crate::game::GameWorld;
use crate::research::{EFFECTS, ResearchState, Stage, TechTree};

pub const RESEARCH: ResourceId = ResourceId(11);

/// Research points a year of a thousand literate researchers at the bench makes, before the
/// duplication of effort. Calibrated so that the leader keeps the dates of the frontier
/// (`docs/research.pt-BR.md`, "Pesquisa que custa").
pub const POINTS_PER_EFFORT: Fixed = milli(26_800);

/// `effort^(3/4)`: doubling the researchers less than doubles the ideas, because they step on
/// each other's toes (the duplication of the idea production function, `λ = 3/4` in the
/// robustness case of Bloom, Jones, Van Reenen and Webb 2020).
fn duplicated(effort: Fixed) -> Fixed {
    let sqrt = |x: Fixed| -> Fixed {
        let raw = u128::try_from(x.raw().max(0)).unwrap_or(0) * 1000;
        Fixed::from_raw(i64::try_from(raw.isqrt()).unwrap_or(i64::MAX))
    };
    let root = sqrt(effort);
    root * sqrt(root)
}
/// Share of the effort spent on new science; the rest builds prototypes of what is known.
const SCIENCE_SHARE: Fixed = milli(700);
/// How much cheaper it is to learn what the whole world already knows.
const SPILLOVER: Fixed = milli(750);
/// A prototype costs this share of the science.
const PROTOTYPE_COST: Fixed = milli(500);
/// Years from industry to everywhere in an average country.
const DIFFUSION_YEARS: i64 = 10;
/// Years an average literate country takes to copy a stage of what others already make.
const IMITATION_YEARS: i64 = 5;
/// Years for a technology made abroad to reach full use at home (vaccines, the pill) without
/// being made there.
const ADOPTION_YEARS: i64 = 15;

pub struct Research {
    pub cat: Arc<Catalogue>,
    pub tree: Arc<TechTree>,
}

pub struct ResearchChanges {
    state: ResearchState,
    tech_tfp: Vec<Vec<Fixed>>,
}

/// Share of today's imitation rate a country keeps through its trade and foreign capital laws
/// (`foreign_capital::imitation_channel`, `docs/late-industrialization.pt-BR.md`). A market
/// without politics keeps all of it.
fn imitation_channel(m: Option<&crate::econ::Market>) -> Fixed {
    m.map_or(Fixed::ONE, |m| {
        let laws = &m.laws.effects[0];
        crate::foreign_capital::imitation_channel(laws.tariff, laws.foreign_investment)
    })
}

/// Weight of a stage in productivity and effects.
fn weight(stage: Stage) -> Fixed {
    match stage {
        Stage::Industrial => milli(500),
        Stage::Diffused => Fixed::ONE,
        _ => Fixed::ZERO,
    }
}

/// Cost multiplier for being ahead of history: five years early doubles the cost.
fn ahead(year: i32, history: i32) -> Fixed {
    if year >= history {
        return Fixed::ONE;
    }
    let d = Fixed::from_ratio(i64::from(history - year), 5);
    Fixed::ONE + d * d
}

struct CountryFacts {
    /// Researchers at the country's laboratories, as they are staffed.
    research: i64,
    literacy: Fixed,
    population: i64,
    gdp: Fixed,
}

fn facts(cat: &Catalogue, world: &GameWorld) -> Vec<CountryFacts> {
    let econ = &world.econ;
    let mut out: Vec<CountryFacts> = econ
        .markets
        .iter()
        .map(|m| CountryFacts {
            research: 0,
            literacy: Fixed::ZERO,
            population: 0,
            gdp: m.gdp_last_year.max(Fixed::ZERO),
        })
        .collect();
    for r in &econ.regions {
        let c = &mut out[r.market as usize];
        let people = r.population();
        // Only the metropole's laboratories count: a colony does not research.
        if let Some(lab) = cat.laboratory.filter(|_| !r.colony) {
            let workers = cat.buildings[usize::from(lab)].methods[0].workers;
            c.research += r
                .plants
                .iter()
                .filter(|p| p.kind == lab)
                .map(|p| (Fixed::from_int(p.levels * workers) * p.staffing).to_int_floor())
                .sum::<i64>();
        }
        c.literacy += r.literacy * people;
        c.population += people;
    }
    for c in &mut out {
        if c.population > 0 {
            c.literacy = c.literacy / c.population;
        }
    }
    out
}

impl System<GameWorld> for Research {
    type Changes = ResearchChanges;

    fn name(&self) -> &'static str {
        "research"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 22 }
    }

    fn access(&self) -> Access {
        Access::new()
            .read(ECON)
            .write(ECON)
            .read(RESEARCH)
            .write(RESEARCH)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> ResearchChanges {
        let tree = &*self.tree;
        let cat = &*self.cat;
        let year = ctx.date.year;
        let mut state = world.research.clone();
        let n = state.stage.len().min(world.econ.markets.len());
        let nt = tree.techs.len();
        let facts = facts(cat, world);
        let world_gdp: Fixed = facts.iter().map(|f| f.gdp).sum::<Fixed>().max(milli(1));
        let unlocks = Unlocks::new(cat, year, Some(tree), &world.research);

        // Share of world GDP at or past each stage, by technology.
        let share_at = |stage: Stage| -> Vec<Fixed> {
            (0..nt)
                .map(|t| {
                    let g: Fixed = (0..n)
                        .filter(|&c| world.research.stage[c][t] >= stage)
                        .map(|c| facts[c].gdp)
                        .sum();
                    g / world_gdp
                })
                .collect()
        };
        let known = share_at(Stage::Science);
        let industrial = share_at(Stage::Industrial);
        let diffused = share_at(Stage::Diffused);

        // Demand for each good in each market and in the world, at market prices.
        let ng = cat.goods.len();
        let mut world_demand = vec![Fixed::ZERO; ng];
        for m in &world.econ.markets {
            for (g, d) in world_demand.iter_mut().enumerate() {
                *d += m.demand[g] * m.price[g];
            }
        }
        // Goods each technology serves: what its methods make, and the goods it brings.
        let served: Vec<Vec<u16>> = tree
            .techs
            .iter()
            .map(|t| {
                let mut goods: Vec<u16> = t.goods.clone();
                for &(b, m) in &t.methods {
                    for (g, _) in &cat.buildings[b as usize].methods[m as usize].outputs {
                        goods.push(*g);
                    }
                }
                goods.sort_unstable();
                goods.dedup();
                goods
            })
            .collect();

        let requirements_met = |c: usize, t: usize, stages: &[Stage]| {
            tree.techs[t]
                .requires
                .iter()
                .all(|&r| stages[r as usize] >= Stage::Science)
                && c < n
        };
        // What the world already makes reaches a country through the channel its laws open
        // (trade and foreign capital): the cheaper prototypes, the push of foreign industry and
        // imitation scale with it. Published science crosses closed borders far more easily: its
        // spillover takes the same channel lifted to a high floor. Fully open, both are one.
        let science_cost = |t: usize, channel: Fixed| -> Fixed {
            let tech = &tree.techs[t];
            Fixed::from_int(i64::from(tech.research_cost))
                * (Fixed::ONE
                    - SPILLOVER * known[t] * crate::foreign_capital::science_channel(channel))
                * ahead(year, tech.demonstrated - 3)
        };

        for c in 0..n {
            let f = &facts[c];
            let boost = Fixed::ONE + state.effects[c][3] * 2;
            // Research is the researchers the laboratories employ, which the state and the firms
            // pay for (spending over the skilled wage, as Bloom et al. measure research effort),
            // not every engineer in the country; literate ones research better.
            let effort = Fixed::from_ratio(f.research, 1000) * f.literacy;
            let points = duplicated(effort) * POINTS_PER_EFFORT * boost / 12;
            state.points[c] = points;
            let stages = state.stage[c].clone();
            let channel = imitation_channel(world.econ.markets.get(c));

            // New science: the chosen technology, or the cheapest open one.
            let open = |t: usize| stages[t] == Stage::Unknown && requirements_met(c, t, &stages);
            let target = state.focus[c]
                .map(usize::from)
                .filter(|&t| t < nt && open(t))
                .or_else(|| {
                    (0..nt)
                        .filter(|&t| open(t))
                        .min_by_key(|&t| (science_cost(t, channel), t))
                });
            if let Some(t) = target {
                state.progress[c][t] +=
                    points * SCIENCE_SHARE / science_cost(t, channel).max(milli(1));
            }

            // Prototypes of what is understood.
            let at_science: Vec<usize> = (0..nt).filter(|&t| stages[t] == Stage::Science).collect();
            if !at_science.is_empty() {
                let each = points * (Fixed::ONE - SCIENCE_SHARE) / at_science.len() as i64;
                for &t in &at_science {
                    let tech = &tree.techs[t];
                    let cost = Fixed::from_int(i64::from(tech.research_cost))
                        * PROTOTYPE_COST
                        * (Fixed::ONE - SPILLOVER * industrial[t] * channel)
                        * ahead(year, tech.demonstrated);
                    state.progress[c][t] += each / cost.max(milli(1));
                }
            }

            // Prototype to industry: needs a buyer.
            let per_head = if f.population > 0 {
                f.gdp / f.population
            } else {
                Fixed::ZERO
            };
            let m = &world.econ.markets[c];
            for t in 0..nt {
                if stages[t] != Stage::Prototype {
                    continue;
                }
                let tech = &tree.techs[t];
                let existing: Vec<usize> = served[t]
                    .iter()
                    .map(|g| *g as usize)
                    .filter(|&g| unlocks.goods[g])
                    .collect();
                let mut buyer = if !existing.is_empty() {
                    let mine: Fixed = existing.iter().map(|&g| m.demand[g] * m.price[g]).sum();
                    let all: Fixed = existing.iter().map(|&g| world_demand[g]).sum();
                    if all > Fixed::ZERO {
                        (mine / (all * milli(30))).clamp(Fixed::ZERO, milli(1500))
                    } else {
                        Fixed::ZERO
                    }
                } else if !served[t].is_empty() {
                    // A good nobody makes yet: rich consumers are the first buyers.
                    (per_head / 6_000).clamp(Fixed::ZERO, milli(1500))
                } else {
                    (milli(500) + f.literacy).min(milli(1500))
                };
                if tech.state_buyer {
                    buyer = buyer.max((f.gdp / world_gdp * 8).clamp(Fixed::ZERO, milli(1500)));
                }
                let years = i64::from((tech.industrialized - tech.demonstrated).max(2));
                let rate = buyer * (Fixed::ONE + industrial[t] * channel * 2)
                    / (years * 12)
                    / ahead(year, tech.industrialized);
                state.progress[c][t] += rate;
            }

            // Imitation: what is made at scale abroad is copied, a stage at a time, faster where
            // people read; no own research needed; through the same channel.
            for t in 0..nt {
                if stages[t] < Stage::Industrial && industrial[t] > Fixed::ZERO {
                    state.progress[c][t] += (milli(300) + f.literacy) * industrial[t] * channel
                        / (IMITATION_YEARS * 12);
                }
            }

            // Industry to everywhere.
            for t in 0..nt {
                if stages[t] == Stage::Industrial {
                    state.progress[c][t] += (milli(500) + f.literacy) * (Fixed::ONE + diffused[t])
                        / (DIFFUSION_YEARS * 12);
                }
            }

            // Stages advance when progress fills.
            for t in 0..nt {
                if state.progress[c][t] >= Fixed::ONE
                    && let Some(next) = state.stage[c][t].next()
                {
                    state.stage[c][t] = next;
                    state.progress[c][t] = Fixed::ZERO;
                    let first = &mut state.firsts[t][next as usize];
                    if *first == 0 {
                        *first = year;
                    }
                }
            }
        }

        // Effects, as a share of what the whole tree can give.
        let mut totals = [Fixed::ZERO; 5];
        for tech in &tree.techs {
            for &(e, v) in &tech.effects {
                totals[e as usize] += Fixed::from_int(i64::from(v));
            }
        }
        for c in 0..n {
            let mut e = [Fixed::ZERO; 5];
            for (t, tech) in tree.techs.iter().enumerate() {
                // What others make is bought and used: vaccines, antibiotics and the pill spread
                // over the years after the first factory, faster where people read.
                let first = state.firsts[t][Stage::Industrial as usize];
                let adoption = if first > 0 {
                    let years =
                        Fixed::from_ratio(i64::from(year - first), ADOPTION_YEARS).min(Fixed::ONE);
                    years * (milli(400) + facts[c].literacy * milli(600))
                } else {
                    Fixed::ZERO
                };
                let w = weight(state.stage[c][t]).max(adoption);
                for &(k, v) in &tech.effects {
                    e[k as usize] += Fixed::from_int(i64::from(v)) * w;
                }
            }
            for k in 0..EFFECTS.len() {
                state.effects[c][k] = if totals[k] > Fixed::ZERO {
                    e[k] / totals[k]
                } else {
                    Fixed::ZERO
                };
            }
        }

        // Productivity by building group, relative to where the country stood in 1925.
        let tags: Vec<&str> = world
            .countries
            .iter()
            .map(|(_, c)| c.tag.as_str())
            .collect();
        let tech_tfp = (0..n)
            .map(|c| {
                BUILDING_GROUPS
                    .iter()
                    .map(|group| {
                        let mut now = Fixed::ONE;
                        let mut start = Fixed::ONE;
                        for (t, tech) in tree.techs.iter().enumerate() {
                            for (g, v) in &tech.productivity {
                                if g == group {
                                    let gain = Fixed::from_ratio(i64::from(*v), 1000);
                                    now *= Fixed::ONE + gain * weight(state.stage[c][t]);
                                    let s0 = tags
                                        .get(c)
                                        .map_or(Stage::Unknown, |tag| tree.start_stage(t, tag));
                                    start *= Fixed::ONE + gain * weight(s0);
                                }
                            }
                        }
                        now / start
                    })
                    .collect()
            })
            .collect();
        ResearchChanges { state, tech_tfp }
    }

    fn apply(&self, _ctx: &TickCtx, changes: ResearchChanges, world: &mut GameWorld) {
        world.research = changes.state;
        for (m, tfp) in world.econ.markets.iter_mut().zip(changes.tech_tfp) {
            m.tech_tfp = tfp;
        }
    }
}
