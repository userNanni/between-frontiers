//! Phase 2 on the engine: research effort, the four stages of every technology in every
//! country, and what they change in the economy (productivity by building group, demography,
//! literacy). Rules in `docs/research.pt-BR.md`.

use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};

use crate::econ::prof;
use crate::econ::week::ECON;
use crate::econ::{Catalogue, Unlocks, milli};
use crate::economy::BUILDING_GROUPS;
use crate::game::GameWorld;
use crate::research::{EFFECTS, ResearchState, Stage, TechTree};

pub const RESEARCH: ResourceId = ResourceId(11);

/// Research effort a thousand literate engineers and professionals produce in a year.
pub const POINTS_PER_THOUSAND: Fixed = milli(500);
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
    colony_tech_tfp: Vec<Vec<Fixed>>,
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
    /// Engineers and professionals at work in the metropole: colonies do no research.
    researchers: i64,
    /// Literacy of the metropole, which weighs its researchers.
    home_literacy: Fixed,
    literacy: Fixed,
    population: i64,
    gdp: Fixed,
    /// Literacy and people of the colonies alone.
    colony_literacy: Fixed,
    colony_population: i64,
}

fn facts(world: &GameWorld) -> Vec<CountryFacts> {
    let econ = &world.econ;
    let mut out: Vec<CountryFacts> = econ
        .markets
        .iter()
        .map(|m| CountryFacts {
            researchers: 0,
            home_literacy: Fixed::ZERO,
            literacy: Fixed::ZERO,
            population: 0,
            gdp: m.gdp_last_year.max(Fixed::ZERO),
            colony_literacy: Fixed::ZERO,
            colony_population: 0,
        })
        .collect();
    for r in &econ.regions {
        let c = &mut out[r.market as usize];
        let people = r.population();
        c.literacy += r.literacy * people;
        c.population += people;
        if r.colony {
            c.colony_literacy += r.literacy * people;
            c.colony_population += people;
        } else {
            c.researchers += r.pops[prof::ENGINEER].employed + r.pops[prof::PROFESSIONAL].employed;
            c.home_literacy += r.literacy * people;
        }
    }
    for c in &mut out {
        if c.population > 0 {
            c.literacy = c.literacy / c.population;
        }
        let home = c.population - c.colony_population;
        if home > 0 {
            c.home_literacy = c.home_literacy / home;
        }
        if c.colony_population > 0 {
            c.colony_literacy = c.colony_literacy / c.colony_population;
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
        let facts = facts(world);
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
            Fixed::from_int(i64::from(tech.cost))
                * (Fixed::ONE
                    - SPILLOVER * known[t] * crate::foreign_capital::science_channel(channel))
                * ahead(year, tech.demonstrated - 3)
        };

        for c in 0..n {
            let f = &facts[c];
            let boost = Fixed::ONE + state.effects[c][3] * 2;
            let points = Fixed::from_int(f.researchers).mul_ratio(1, 1000)
                * f.home_literacy
                * POINTS_PER_THOUSAND
                * boost
                / 12;
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
                    let cost = Fixed::from_int(i64::from(tech.cost))
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

        for (c, f) in facts.iter().enumerate().take(n) {
            colony_step(
                &mut state,
                c,
                f,
                &world.research.stage[c],
                &industrial,
                &diffused,
                imitation_channel(world.econ.markets.get(c)),
            );
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
            .map(|c| group_tfp(tree, tags.get(c).copied(), &state.stage[c]))
            .collect();
        let colony_tech_tfp = (0..n)
            .map(|c| match state.colony_stage.get(c) {
                Some(stage) if facts[c].colony_population > 0 => {
                    group_tfp(tree, tags.get(c).copied(), stage)
                }
                _ => Vec::new(),
            })
            .collect();
        ResearchChanges {
            state,
            tech_tfp,
            colony_tech_tfp,
        }
    }

    fn apply(&self, _ctx: &TickCtx, changes: ResearchChanges, world: &mut GameWorld) {
        world.research = changes.state;
        for ((m, tfp), colony) in world
            .econ
            .markets
            .iter_mut()
            .zip(changes.tech_tfp)
            .zip(changes.colony_tech_tfp)
        {
            m.tech_tfp = tfp;
            m.colony_tech_tfp = colony;
        }
    }
}

/// Productivity from technology by building group for the stages `stage`, relative to where the
/// country `tag` stood in 1925.
fn group_tfp(tree: &TechTree, tag: Option<&str>, stage: &[Stage]) -> Vec<Fixed> {
    BUILDING_GROUPS
        .iter()
        .map(|group| {
            let mut now = Fixed::ONE;
            let mut start = Fixed::ONE;
            for (t, tech) in tree.techs.iter().enumerate() {
                for (g, v) in &tech.productivity {
                    if g == group {
                        let gain = Fixed::from_ratio(i64::from(*v), 1000);
                        now *= Fixed::ONE + gain * weight(stage[t]);
                        let s0 = tag.map_or(Stage::Unknown, |tag| tree.start_stage(t, tag));
                        start *= Fixed::ONE + gain * weight(s0);
                    }
                }
            }
            now / start
        })
        .collect()
}

/// A month of the colonies of country `c`. They do no research of their own: what their
/// metropole (`metropole`, its stages at the start of the month) makes at scale reaches them by
/// the same imitation as anywhere, at the colonies' literacy and through the country's channel,
/// in one step (the firm brings the know-how; nobody there builds the science or the prototype);
/// then it spreads there as it spreads anywhere. A laboratory in a colony adds nothing here.
fn colony_step(
    state: &mut ResearchState,
    c: usize,
    f: &CountryFacts,
    metropole: &[Stage],
    industrial: &[Fixed],
    diffused: &[Fixed],
    channel: Fixed,
) {
    if f.colony_population == 0 || c >= state.colony_stage.len() {
        return;
    }
    let lit = f.colony_literacy;
    let (stages, progress) = (&mut state.colony_stage[c], &mut state.colony_progress[c]);
    for t in 0..stages.len() {
        let p = &mut progress[t];
        if stages[t] < Stage::Industrial && metropole[t] >= Stage::Industrial {
            *p += (milli(300) + lit) * industrial[t] * channel / (IMITATION_YEARS * 12);
            if *p >= Fixed::ONE {
                stages[t] = Stage::Industrial;
                *p = Fixed::ZERO;
            }
        } else if stages[t] == Stage::Industrial {
            *p += (milli(500) + lit) * (Fixed::ONE + diffused[t]) / (DIFFUSION_YEARS * 12);
            if *p >= Fixed::ONE {
                stages[t] = Stage::Diffused;
                *p = Fixed::ZERO;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colonies(literacy: i64) -> CountryFacts {
        CountryFacts {
            researchers: 0,
            home_literacy: milli(literacy),
            literacy: milli(literacy),
            population: 1_000,
            gdp: Fixed::ONE,
            colony_literacy: milli(literacy),
            colony_population: 1_000,
        }
    }

    fn state(stage: Stage) -> ResearchState {
        ResearchState {
            colony_stage: vec![vec![stage]],
            colony_progress: vec![vec![Fixed::ZERO]],
            ..Default::default()
        }
    }

    /// Months until the colonies of a metropole that makes the technology take it up.
    fn months_to_industry(literacy: i64) -> i64 {
        let mut s = state(Stage::Unknown);
        let f = colonies(literacy);
        for month in 1..=1_200 {
            colony_step(
                &mut s,
                0,
                &f,
                &[Stage::Industrial],
                &[milli(500)],
                &[Fixed::ZERO],
                Fixed::ONE,
            );
            if s.colony_stage[0][0] == Stage::Industrial {
                return month;
            }
        }
        i64::MAX
    }

    #[test]
    fn colonies_take_up_what_the_metropole_makes_later_where_few_read() {
        let lettered = months_to_industry(800);
        let unlettered = months_to_industry(100);
        // (0.3 + literacy) × half the world × 1 / 5 years: about nine years.
        assert!((105..=115).contains(&lettered), "{lettered}");
        assert!(unlettered > lettered * 2, "{unlettered} vs {lettered}");
    }

    #[test]
    fn colonies_never_run_ahead_of_their_metropole() {
        let mut s = state(Stage::Unknown);
        for _ in 0..1_200 {
            colony_step(
                &mut s,
                0,
                &colonies(900),
                &[Stage::Prototype],
                &[Fixed::ONE],
                &[Fixed::ONE],
                Fixed::ONE,
            );
        }
        assert_eq!(s.colony_stage[0][0], Stage::Unknown);
    }

    #[test]
    fn a_country_without_colonies_keeps_its_colony_stages() {
        let mut s = state(Stage::Unknown);
        let mut f = colonies(900);
        f.colony_population = 0;
        colony_step(
            &mut s,
            0,
            &f,
            &[Stage::Industrial],
            &[Fixed::ONE],
            &[Fixed::ONE],
            Fixed::ONE,
        );
        assert_eq!(s.colony_progress[0][0], Fixed::ZERO);
    }
}
