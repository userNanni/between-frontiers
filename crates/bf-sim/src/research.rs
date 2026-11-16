//! Phase 2: technologies and how far each country has taken each one.
//!
//! A technology moves, country by country, through four stages inspired by the TRL scale:
//! science (the idea is understood), prototype (it was made to work once), industrial (it is
//! made at scale, so its methods can be built) and diffused (it is everywhere, at full
//! productivity). Knowing and producing are different things: a country can buy and use what
//! it does not make. Without a buyer, from the market or from the state, a technology stalls at
//! the prototype: that is the valley of death.
//!
//! Data in `data/research/technologies.ron`; names in `assets/i18n/*/research.ftl`
//! (`tech-<key>` and `tech-<key>-desc`, `tech-field-<field>`, `tech-era-<n>`, `tech-stage-<stage>`).

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

/// Fields of research, in display order.
pub const FIELDS: [&str; 12] = [
    "agriculture",
    "extraction",
    "energy",
    "materials",
    "chemistry",
    "machinery",
    "electrical",
    "electronics",
    "transport",
    "medicine",
    "military",
    "society",
];

/// Eras by first year; era 0 holds what was known before the game starts.
pub const ERAS: [i32; 6] = [0, 1925, 1945, 1965, 1985, 2005];

/// What a technology changes besides unlocking methods and goods, per mille.
pub const EFFECTS: [&str; 5] = [
    // Fewer deaths (vaccines, antibiotics, sanitation).
    "mortality",
    // Fewer births (the pill, family planning).
    "fertility",
    // Faster growth of literacy (mass schooling, printing, broadcasting).
    "literacy",
    // More research per researcher (computers, the internet).
    "research",
    // Cheaper trade between markets (containers, jet freight).
    "trade",
];

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Stage {
    #[default]
    Unknown,
    Science,
    Prototype,
    Industrial,
    Diffused,
}

impl Stage {
    pub const ALL: [Stage; 5] = [
        Stage::Unknown,
        Stage::Science,
        Stage::Prototype,
        Stage::Industrial,
        Stage::Diffused,
    ];

    /// The i18n suffix: `tech-stage-<key>`.
    pub fn key(self) -> &'static str {
        match self {
            Stage::Unknown => "unknown",
            Stage::Science => "science",
            Stage::Prototype => "prototype",
            Stage::Industrial => "industrial",
            Stage::Diffused => "diffused",
        }
    }

    pub fn next(self) -> Option<Stage> {
        match self {
            Stage::Unknown => Some(Stage::Science),
            Stage::Science => Some(Stage::Prototype),
            Stage::Prototype => Some(Stage::Industrial),
            Stage::Industrial => Some(Stage::Diffused),
            Stage::Diffused => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechnologiesFile {
    pub technologies: Vec<TechnologyDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechnologyDef {
    pub key: String,
    /// One of [`FIELDS`].
    pub field: String,
    /// Index into [`ERAS`]: the era the historical breakthrough belongs to.
    pub era: u8,
    /// Technologies that must be at least at the science stage first.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Year it first worked anywhere (prototype), and year it was first made at scale
    /// (industrial). Used to calibrate research speed and to check the 100-year run.
    pub demonstrated: i32,
    pub industrialized: i32,
    /// Research effort to reach the science stage. Relative units: an era-1 breakthrough is about
    /// 100, and effort grows with the era (R&D spending grew far faster than the economy).
    pub cost: u32,
    /// Methods (`building.method`) a country can build with once it reaches the industrial
    /// stage. Methods no technology lists are always available.
    #[serde(default)]
    pub methods: Vec<String>,
    /// Goods that exist in the world once some country reaches the industrial stage.
    #[serde(default)]
    pub goods: Vec<String>,
    /// Productivity gain, per mille, for plants of a building group (`economy::BUILDING_GROUPS`)
    /// in countries where the technology is diffused (half of it at the industrial stage).
    #[serde(default)]
    pub productivity: Vec<(String, u16)>,
    /// Other effects, per mille, keyed by [`EFFECTS`], at the diffused stage.
    #[serde(default)]
    pub effects: Vec<(String, u16)>,
    /// The state can carry it across the valley of death as the buyer (arms, space, nuclear).
    #[serde(default)]
    pub state_buyer: bool,
    /// Stage on 1 January 1925; omitted means unknown everywhere.
    #[serde(default)]
    pub start: Option<Start>,
}

/// Stage in 1925: a default and the countries (by tag) that differ from it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Start {
    pub default: Stage,
    #[serde(default)]
    pub countries: Vec<(String, Stage)>,
}

/// A technology with its references resolved to indices.
#[derive(Clone, Debug)]
pub struct Tech {
    pub key: String,
    pub field: u8,
    pub era: u8,
    pub requires: Vec<u16>,
    pub demonstrated: i32,
    pub industrialized: i32,
    pub cost: u32,
    /// (building index, method index) pairs in the economy catalogue.
    pub methods: Vec<(u16, u8)>,
    pub goods: Vec<u16>,
    /// (building group, per mille).
    pub productivity: Vec<(String, u16)>,
    /// (index into EFFECTS, per mille).
    pub effects: Vec<(u8, u16)>,
    pub state_buyer: bool,
    pub start: Option<Start>,
}

#[derive(Clone, Debug, Default)]
pub struct TechTree {
    pub techs: Vec<Tech>,
    /// For each building, for each method: the technology that unlocks it, if any.
    pub method_tech: Vec<Vec<Option<u16>>>,
    /// For each good: the technologies whose industrial stage brings it into the world.
    pub good_techs: Vec<Vec<u16>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TreeError {
    Duplicate(String),
    UnknownField {
        tech: String,
        field: String,
    },
    UnknownEra {
        tech: String,
        era: u8,
    },
    UnknownRequirement {
        tech: String,
        requires: String,
    },
    UnknownMethod {
        tech: String,
        method: String,
    },
    UnknownGood {
        tech: String,
        good: String,
    },
    UnknownGroup {
        tech: String,
        group: String,
    },
    UnknownEffect {
        tech: String,
        effect: String,
    },
    /// Requirements loop back to the technology.
    Cycle(String),
    /// Two technologies unlock the same method.
    MethodTwice {
        method: String,
        first: String,
        second: String,
    },
    /// Industrialized before demonstrated.
    Dates(String),
}

impl std::fmt::Display for TreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl TechTree {
    pub fn build(file: &TechnologiesFile, cat: &crate::econ::Catalogue) -> Result<Self, TreeError> {
        let defs = &file.technologies;
        let index = |key: &str| defs.iter().position(|t| t.key == key);
        let mut techs = Vec::with_capacity(defs.len());
        let mut method_tech: Vec<Vec<Option<u16>>> = cat
            .buildings
            .iter()
            .map(|b| vec![None; b.methods.len()])
            .collect();
        let mut good_techs = vec![Vec::new(); cat.goods.len()];
        for (ti, d) in defs.iter().enumerate() {
            if index(&d.key) != Some(ti) {
                return Err(TreeError::Duplicate(d.key.clone()));
            }
            let field = FIELDS.iter().position(|f| *f == d.field).ok_or_else(|| {
                TreeError::UnknownField {
                    tech: d.key.clone(),
                    field: d.field.clone(),
                }
            })? as u8;
            if usize::from(d.era) >= ERAS.len() {
                return Err(TreeError::UnknownEra {
                    tech: d.key.clone(),
                    era: d.era,
                });
            }
            if d.industrialized < d.demonstrated {
                return Err(TreeError::Dates(d.key.clone()));
            }
            let requires = d
                .requires
                .iter()
                .map(|r| {
                    index(r)
                        .map(|i| i as u16)
                        .ok_or_else(|| TreeError::UnknownRequirement {
                            tech: d.key.clone(),
                            requires: r.clone(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut methods = Vec::new();
            for m in &d.methods {
                let unknown = || TreeError::UnknownMethod {
                    tech: d.key.clone(),
                    method: m.clone(),
                };
                let (b, mk) = m.split_once('.').ok_or_else(unknown)?;
                let bi = cat.building(b).ok_or_else(unknown)?;
                let mi = cat.buildings[bi]
                    .methods
                    .iter()
                    .position(|x| x.key == mk)
                    .ok_or_else(unknown)?;
                if let Some(first) = method_tech[bi][mi] {
                    return Err(TreeError::MethodTwice {
                        method: m.clone(),
                        first: defs[first as usize].key.clone(),
                        second: d.key.clone(),
                    });
                }
                method_tech[bi][mi] = Some(ti as u16);
                methods.push((bi as u16, mi as u8));
            }
            let mut goods = Vec::new();
            for g in &d.goods {
                let gi = cat.good(g).ok_or_else(|| TreeError::UnknownGood {
                    tech: d.key.clone(),
                    good: g.clone(),
                })?;
                good_techs[gi as usize].push(ti as u16);
                goods.push(gi);
            }
            for (group, _) in &d.productivity {
                if !crate::economy::BUILDING_GROUPS.contains(&group.as_str()) {
                    return Err(TreeError::UnknownGroup {
                        tech: d.key.clone(),
                        group: group.clone(),
                    });
                }
            }
            let effects = d
                .effects
                .iter()
                .map(|(e, v)| {
                    EFFECTS
                        .iter()
                        .position(|x| x == e)
                        .map(|i| (i as u8, *v))
                        .ok_or_else(|| TreeError::UnknownEffect {
                            tech: d.key.clone(),
                            effect: e.clone(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            techs.push(Tech {
                key: d.key.clone(),
                field,
                era: d.era,
                requires,
                demonstrated: d.demonstrated,
                industrialized: d.industrialized,
                cost: d.cost,
                methods,
                goods,
                productivity: d.productivity.clone(),
                effects,
                state_buyer: d.state_buyer,
                start: d.start.clone(),
            });
        }
        // Requirements form a DAG.
        let mut state = vec![0u8; techs.len()];
        fn visit(t: usize, techs: &[Tech], state: &mut [u8]) -> Result<(), TreeError> {
            match state[t] {
                1 => return Err(TreeError::Cycle(techs[t].key.clone())),
                2 => return Ok(()),
                _ => {}
            }
            state[t] = 1;
            for &r in &techs[t].requires {
                visit(r as usize, techs, state)?;
            }
            state[t] = 2;
            Ok(())
        }
        for t in 0..techs.len() {
            visit(t, &techs, &mut state)?;
        }
        Ok(Self {
            techs,
            method_tech,
            good_techs,
        })
    }

    pub fn tech(&self, key: &str) -> Option<usize> {
        self.techs.iter().position(|t| t.key == key)
    }

    /// Stage of `tech` on 1 January 1925 in the country `tag`.
    pub fn start_stage(&self, tech: usize, tag: &str) -> Stage {
        self.techs[tech].start.as_ref().map_or(Stage::Unknown, |s| {
            s.countries
                .iter()
                .find(|(t, _)| t == tag)
                .map_or(s.default, |(_, st)| *st)
        })
    }
}

/// Where every country stands on every technology. Indexed by country (the market index), then
/// by technology.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchState {
    pub stage: Vec<Vec<Stage>>,
    /// Progress toward the next stage, 0–1 of its requirement.
    pub progress: Vec<Vec<Fixed>>,
    /// Technology the country chose to research; `None` lets the country pick by itself.
    pub focus: Vec<Option<u16>>,
    /// Research effort produced last month, by country.
    pub points: Vec<Fixed>,
    /// Year each technology first reached each stage anywhere (0 = not yet), by technology,
    /// indexed by `Stage as usize`.
    pub firsts: Vec<[i32; 5]>,
    /// Strength of each of [`EFFECTS`] by country, 0–1 of what the whole tree gives.
    #[serde(default)]
    pub effects: Vec<[Fixed; 5]>,
}

impl ResearchState {
    /// The 1925 state for the given country tags, in market order.
    pub fn initial<'a>(tree: &TechTree, tags: impl IntoIterator<Item = &'a str>) -> Self {
        let stage: Vec<Vec<Stage>> = tags
            .into_iter()
            .map(|tag| {
                (0..tree.techs.len())
                    .map(|t| tree.start_stage(t, tag))
                    .collect()
            })
            .collect();
        let n = stage.len();
        let mut firsts = vec![[0i32; 5]; tree.techs.len()];
        for (t, first) in firsts.iter_mut().enumerate() {
            for s in Stage::ALL.into_iter().skip(1) {
                if stage.iter().any(|c| c[t] >= s) {
                    first[s as usize] = 1925;
                }
            }
        }
        Self {
            progress: vec![vec![Fixed::ZERO; tree.techs.len()]; n],
            focus: vec![None; n],
            points: vec![Fixed::ZERO; n],
            effects: vec![[Fixed::ZERO; 5]; n],
            stage,
            firsts,
        }
    }

    /// Whether `country` may build with `method` of `building`.
    pub fn can_build(
        &self,
        tree: &TechTree,
        country: usize,
        building: usize,
        method: usize,
    ) -> bool {
        match tree
            .method_tech
            .get(building)
            .and_then(|m| m.get(method))
            .copied()
            .flatten()
        {
            None => true,
            Some(t) => self
                .stage
                .get(country)
                .is_none_or(|c| c[t as usize] >= Stage::Industrial),
        }
    }

    /// Whether `good` exists anywhere in the world (someone can make it).
    pub fn good_exists(&self, tree: &TechTree, good: usize) -> bool {
        let techs = &tree.good_techs[good];
        techs.is_empty()
            || techs
                .iter()
                .any(|&t| self.firsts[t as usize][Stage::Industrial as usize] > 0)
    }

    pub fn is_empty(&self) -> bool {
        self.stage.is_empty()
    }
}

impl HashState for Stage {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u8(*self as u8);
    }
}

impl HashState for ResearchState {
    fn hash_state(&self, h: &mut StateHasher) {
        self.stage.hash_state(h);
        self.progress.hash_state(h);
        h.write_len(self.focus.len());
        for f in &self.focus {
            h.write_i64(f.map_or(-1, i64::from));
        }
        self.points.hash_state(h);
        h.write_len(self.firsts.len());
        for f in &self.firsts {
            for y in f {
                h.write_i64(i64::from(*y));
            }
        }
        h.write_len(self.effects.len());
        for e in &self.effects {
            for v in e {
                v.hash_state(h);
            }
        }
    }
}
