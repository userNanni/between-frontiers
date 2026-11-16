//! Communities: who lives in each state, as a culture and a religion taken together.
//!
//! A community is a pair culture + religion treated as one identity ("Irish Catholic", "Bengali
//! Sunni"), so the link between the two is kept by construction. The population table of the
//! simulation will be keyed by state × community × profession; this module is the contract that
//! comes before it: the lists, the composition of every state on the start date and the rule
//! that decides which communities get a row of their own and which go into the state's "other
//! minorities" row. Nothing here runs in the tick yet.
//!
//! - `data/culture/cultures.ron`: [`CulturesFile`].
//! - `data/culture/religions.ron`: [`ReligionsFile`].
//! - `data/scenario/1925/communities.ron`: [`CommunitiesFile`], written by
//!   `tools/communities/generate.py` from the sourced tables in `tools/communities/spec/`.
//! - Names in `assets/i18n/*/communities.ftl`: `culture-<key>`, `religion-<key>`,
//!   `culture-family-<key>`, `religion-family-<key>`.
//!
//! Rules and proposed parameters: `docs/population.pt-BR.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::scenario::{MapFile, Tag};

/// Language families that group the cultures (assimilation is easier inside a family).
pub const CULTURE_FAMILIES: &[&str] = &[
    "germanic",
    "romance",
    "slavic",
    "baltic",
    "celtic",
    "hellenic",
    "albanian",
    "armenian",
    "iranian",
    "indo_aryan",
    "dravidian",
    "turkic",
    "mongolic",
    "tungusic",
    "uralic",
    "caucasian",
    "semitic",
    "berber",
    "cushitic",
    "nilo_saharan",
    "niger_congo",
    "bantu",
    "khoisan",
    "sino_tibetan",
    "tai_kadai",
    "austroasiatic",
    "hmong_mien",
    "austronesian",
    "papuan",
    "japonic",
    "koreanic",
    "american",
    "creole",
    "other",
];

/// Families of religions.
pub const RELIGION_FAMILIES: &[&str] = &[
    "christian",
    "muslim",
    "jewish",
    "dharmic",
    "iranian",
    "buddhist",
    "east_asian",
    "folk",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CulturesFile {
    pub cultures: Vec<CultureDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CultureDef {
    /// Stable key, snake_case English: `irish`, `han`, `african_american`.
    pub key: String,
    /// One of [`CULTURE_FAMILIES`].
    pub family: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReligionsFile {
    pub religions: Vec<ReligionDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReligionDef {
    pub key: String,
    /// One of [`RELIGION_FAMILIES`].
    pub family: String,
}

/// The communities of a scenario and where they live on the start date.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunitiesFile {
    /// Every community of the scenario, as `(culture, religion)`.
    pub communities: Vec<(String, String)>,
    /// What makes a community matter in a country beyond its size, on the start date.
    #[serde(default)]
    pub relevance: Vec<RelevanceDef>,
    /// The composition of every state of the map.
    pub states: Vec<StateComposition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelevanceDef {
    pub country: Tag,
    /// Whether it holds in the country's colonies (`true`) or in its metropole.
    #[serde(default)]
    pub colonies: bool,
    pub culture: String,
    pub religion: String,
    pub factors: Vec<Factor>,
    /// Why, with the source.
    pub note: String,
}

/// What raises a community's relevance in a country. Concentration (a majority in some state)
/// is not listed in the data: it is computed from the compositions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Factor {
    /// Share of the elite (land, capital, officer corps, bureaucracy) well above its share of
    /// the population.
    Elite,
    /// Its own party, church, movement or militia acting politically.
    Organization,
    /// A distinct legal status, privileged or persecuted.
    LegalStatus,
    /// A homeland or protector abroad that claims it.
    Patron,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateComposition {
    /// State key from `assets/map/provinces.ron`.
    pub state: String,
    /// `(culture, religion, share)`, largest first; the shares sum to 1.
    pub shares: Vec<(String, String, f32)>,
    pub basis: Basis,
    /// Where the shares come from.
    pub source: String,
}

/// How firm a composition is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Basis {
    /// From a census or an official count of 1915–1935.
    Census,
    /// A census for one dimension (language or nationality) with the other inferred, or a
    /// census of a nearby year adjusted.
    Mixed,
    /// No census: estimated, the method in `source`.
    Estimate,
}

/// Index of a community in [`Communities::communities`].
pub type CommunityId = u16;

/// Most a state's shares may miss 1 by.
pub const SHARE_TOLERANCE: f32 = 0.001;

#[derive(Debug, PartialEq)]
pub enum CommunityError {
    DuplicateCulture(String),
    DuplicateReligion(String),
    UnknownCultureFamily {
        culture: String,
        family: String,
    },
    UnknownReligionFamily {
        religion: String,
        family: String,
    },
    DuplicateCommunity {
        culture: String,
        religion: String,
    },
    /// A community names a culture or a religion missing from its list.
    BrokenCommunity {
        culture: String,
        religion: String,
    },
    /// A community that no state has.
    UnusedCommunity {
        culture: String,
        religion: String,
    },
    UnknownState(String),
    DuplicateState(String),
    MissingState(String),
    EmptyState(String),
    /// A state names a community missing from the list.
    UnknownCommunity {
        state: String,
        culture: String,
        religion: String,
    },
    CommunityTwice {
        state: String,
        culture: String,
        religion: String,
    },
    ShareOutOfRange {
        state: String,
        share: f32,
    },
    SharesDoNotSumToOne {
        state: String,
        sum: f32,
    },
    UnknownCountry(String),
    RelevanceUnknownCommunity {
        country: String,
        culture: String,
        religion: String,
    },
    RelevanceTwice {
        country: String,
        culture: String,
        religion: String,
    },
}

impl std::fmt::Display for CommunityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// The communities of a scenario, checked against the map.
#[derive(Clone, Debug)]
pub struct Communities {
    pub cultures: Vec<CultureDef>,
    pub religions: Vec<ReligionDef>,
    /// `(culture index, religion index)`.
    pub communities: Vec<(u16, u16)>,
    /// One per state of the map, in map order.
    pub states: Vec<StateMix>,
    relevance: BTreeMap<(Tag, bool, CommunityId), Vec<Factor>>,
}

#[derive(Clone, Debug)]
pub struct StateMix {
    pub state: String,
    pub owner: Tag,
    pub colony: bool,
    /// Largest first.
    pub shares: Vec<(CommunityId, f32)>,
    pub basis: Basis,
}

impl Communities {
    pub fn build(
        cultures: &CulturesFile,
        religions: &ReligionsFile,
        file: &CommunitiesFile,
        map: &MapFile,
    ) -> Result<Self, CommunityError> {
        let mut culture_ix = BTreeMap::new();
        for (i, c) in cultures.cultures.iter().enumerate() {
            if !CULTURE_FAMILIES.contains(&c.family.as_str()) {
                return Err(CommunityError::UnknownCultureFamily {
                    culture: c.key.clone(),
                    family: c.family.clone(),
                });
            }
            if culture_ix.insert(c.key.as_str(), i as u16).is_some() {
                return Err(CommunityError::DuplicateCulture(c.key.clone()));
            }
        }
        let mut religion_ix = BTreeMap::new();
        for (i, r) in religions.religions.iter().enumerate() {
            if !RELIGION_FAMILIES.contains(&r.family.as_str()) {
                return Err(CommunityError::UnknownReligionFamily {
                    religion: r.key.clone(),
                    family: r.family.clone(),
                });
            }
            if religion_ix.insert(r.key.as_str(), i as u16).is_some() {
                return Err(CommunityError::DuplicateReligion(r.key.clone()));
            }
        }

        let mut communities = Vec::new();
        let mut community_ix: BTreeMap<(&str, &str), CommunityId> = BTreeMap::new();
        for (c, r) in &file.communities {
            let (Some(&ci), Some(&ri)) = (culture_ix.get(c.as_str()), religion_ix.get(r.as_str()))
            else {
                return Err(CommunityError::BrokenCommunity {
                    culture: c.clone(),
                    religion: r.clone(),
                });
            };
            let id = communities.len() as CommunityId;
            if community_ix.insert((c.as_str(), r.as_str()), id).is_some() {
                return Err(CommunityError::DuplicateCommunity {
                    culture: c.clone(),
                    religion: r.clone(),
                });
            }
            communities.push((ci, ri));
        }

        let mut by_state = BTreeMap::new();
        for s in &file.states {
            if by_state.insert(s.state.as_str(), s).is_some() {
                return Err(CommunityError::DuplicateState(s.state.clone()));
            }
        }
        let map_keys: BTreeSet<&str> = map.states.iter().map(|s| s.key.as_str()).collect();
        if let Some(extra) = by_state.keys().find(|k| !map_keys.contains(*k)) {
            return Err(CommunityError::UnknownState((*extra).to_string()));
        }

        let mut used = vec![false; communities.len()];
        let mut states = Vec::with_capacity(map.states.len());
        for def in &map.states {
            let Some(s) = by_state.get(def.key.as_str()) else {
                return Err(CommunityError::MissingState(def.key.clone()));
            };
            if s.shares.is_empty() {
                return Err(CommunityError::EmptyState(s.state.clone()));
            }
            let mut shares = Vec::with_capacity(s.shares.len());
            for (c, r, share) in &s.shares {
                let Some(&id) = community_ix.get(&(c.as_str(), r.as_str())) else {
                    return Err(CommunityError::UnknownCommunity {
                        state: s.state.clone(),
                        culture: c.clone(),
                        religion: r.clone(),
                    });
                };
                if !(share.is_finite() && *share > 0.0 && *share <= 1.0) {
                    return Err(CommunityError::ShareOutOfRange {
                        state: s.state.clone(),
                        share: *share,
                    });
                }
                if shares.iter().any(|&(other, _)| other == id) {
                    return Err(CommunityError::CommunityTwice {
                        state: s.state.clone(),
                        culture: c.clone(),
                        religion: r.clone(),
                    });
                }
                used[id as usize] = true;
                shares.push((id, *share));
            }
            let sum: f32 = shares.iter().map(|(_, v)| v).sum();
            if (sum - 1.0).abs() > SHARE_TOLERANCE {
                return Err(CommunityError::SharesDoNotSumToOne {
                    state: s.state.clone(),
                    sum,
                });
            }
            shares.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            states.push(StateMix {
                state: def.key.clone(),
                owner: def.owner.clone(),
                colony: def.colony,
                shares,
                basis: s.basis,
            });
        }
        if let Some(i) = used.iter().position(|u| !u) {
            let (c, r) = &file.communities[i];
            return Err(CommunityError::UnusedCommunity {
                culture: c.clone(),
                religion: r.clone(),
            });
        }

        let owners: BTreeSet<&str> = map.states.iter().map(|s| s.owner.as_str()).collect();
        let mut relevance = BTreeMap::new();
        for d in &file.relevance {
            if !owners.contains(d.country.as_str()) {
                return Err(CommunityError::UnknownCountry(d.country.clone()));
            }
            let Some(&id) = community_ix.get(&(d.culture.as_str(), d.religion.as_str())) else {
                return Err(CommunityError::RelevanceUnknownCommunity {
                    country: d.country.clone(),
                    culture: d.culture.clone(),
                    religion: d.religion.clone(),
                });
            };
            let mut factors = d.factors.clone();
            factors.sort();
            factors.dedup();
            if relevance
                .insert((d.country.clone(), d.colonies, id), factors)
                .is_some()
            {
                return Err(CommunityError::RelevanceTwice {
                    country: d.country.clone(),
                    culture: d.culture.clone(),
                    religion: d.religion.clone(),
                });
            }
        }

        Ok(Self {
            cultures: cultures.cultures.clone(),
            religions: religions.religions.clone(),
            communities,
            states,
            relevance,
        })
    }

    /// `culture-religion` key of a community, e.g. `irish-catholic`.
    pub fn key(&self, id: CommunityId) -> String {
        let (c, r) = self.communities[id as usize];
        format!(
            "{}-{}",
            self.cultures[c as usize].key, self.religions[r as usize].key
        )
    }

    pub fn find(&self, culture: &str, religion: &str) -> Option<CommunityId> {
        self.communities
            .iter()
            .position(|&(c, r)| {
                self.cultures[c as usize].key == culture
                    && self.religions[r as usize].key == religion
            })
            .map(|i| i as CommunityId)
    }

    /// Factors listed in the data for a community in a country's metropole or colonies.
    pub fn factors(&self, country: &str, colonies: bool, id: CommunityId) -> &[Factor] {
        self.relevance
            .get(&(country.to_string(), colonies, id))
            .map_or(&[], Vec::as_slice)
    }

    /// Which communities get a row of their own in each state, by the ABC curve of `rule`.
    /// `population` gives the people of each state (it only weighs the national ranking);
    /// states missing from it weigh nothing there.
    pub fn rows(&self, rule: &RowRule, population: &BTreeMap<String, f64>) -> Vec<StateRows> {
        // Markets are ranked apart: a country's metropole and its colonies taken together.
        type Market = (Tag, bool);
        let mut people: BTreeMap<Market, BTreeMap<CommunityId, f64>> = BTreeMap::new();
        let mut majority: BTreeSet<(Tag, CommunityId)> = BTreeSet::new();
        for s in &self.states {
            let pop = population.get(&s.state).copied().unwrap_or(0.0);
            let entry = people.entry((s.owner.clone(), s.colony)).or_default();
            for &(id, share) in &s.shares {
                *entry.entry(id).or_default() += pop * f64::from(share);
                if share > 0.5 {
                    majority.insert((s.owner.clone(), id));
                }
            }
        }
        // Concentration: a majority in some state without being the largest community of the
        // country (metropole and colonies together).
        let mut largest: BTreeMap<&str, (CommunityId, f64)> = BTreeMap::new();
        let mut national: BTreeMap<(&str, CommunityId), f64> = BTreeMap::new();
        for ((owner, _), m) in &people {
            for (&id, &n) in m {
                *national.entry((owner.as_str(), id)).or_default() += n;
            }
        }
        for (&(owner, id), &n) in &national {
            let e = largest.entry(owner).or_insert((id, n));
            if n > e.1 {
                *e = (id, n);
            }
        }
        let relevance = |owner: &str, colony: bool, id: CommunityId| -> f64 {
            let mut r = 1.0 + rule.weights.of(self.factors(owner, colony, id));
            let concentrated = majority.contains(&(owner.to_string(), id))
                && largest.get(owner).is_none_or(|&(top, _)| top != id);
            if concentrated {
                r += rule.weights.concentration;
            }
            r
        };
        let mut national_ab: BTreeSet<(Market, CommunityId)> = BTreeSet::new();
        for (market, m) in &people {
            let ranked: Vec<(CommunityId, f64)> = m
                .iter()
                .map(|(&id, &n)| (id, n * relevance(&market.0, market.1, id)))
                .collect();
            for id in abc(ranked, rule.cut) {
                national_ab.insert((market.clone(), id));
            }
        }

        self.states
            .iter()
            .map(|s| {
                let market = (s.owner.clone(), s.colony);
                let importance: BTreeMap<CommunityId, f64> = s
                    .shares
                    .iter()
                    .map(|&(id, share)| (id, f64::from(share) * relevance(&s.owner, s.colony, id)))
                    .collect();
                let mut chosen: BTreeSet<CommunityId> =
                    abc(importance.iter().map(|(&k, &v)| (k, v)).collect(), rule.cut)
                        .into_iter()
                        .collect();
                for &(id, share) in &s.shares {
                    if share >= rule.national_floor && national_ab.contains(&(market.clone(), id)) {
                        chosen.insert(id);
                    }
                }
                let mut rows: Vec<CommunityId> = chosen.into_iter().collect();
                rows.sort_by(|a, b| importance[b].total_cmp(&importance[a]).then(a.cmp(b)));
                rows.truncate(rule.max_rows);
                let others = s
                    .shares
                    .iter()
                    .filter(|(id, _)| !rows.contains(id))
                    .copied()
                    .collect();
                StateRows {
                    state: s.state.clone(),
                    rows,
                    others,
                }
            })
            .collect()
    }
}

/// Reads and checks the communities from `data` (the `data/` directory) when
/// `scenario/1925/communities.ron` exists; the culture and religion lists are then required.
pub fn load(data: &Path, map: &MapFile) -> Result<Option<Communities>, String> {
    fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }
    let path = data.join("scenario/1925/communities.ron");
    if !path.is_file() {
        return Ok(None);
    }
    let file: CommunitiesFile = read(&path)?;
    let cultures: CulturesFile = read(&data.join("culture/cultures.ron"))?;
    let religions: ReligionsFile = read(&data.join("culture/religions.ron"))?;
    Communities::build(&cultures, &religions, &file, map)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Classes A and B of an ABC curve: by importance, largest first, every item whose cumulative
/// importance before it is under `cut` of the total.
fn abc(mut items: Vec<(CommunityId, f64)>, cut: f64) -> Vec<CommunityId> {
    items.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let total: f64 = items.iter().map(|(_, v)| v).sum();
    let mut out = Vec::new();
    let mut acc = 0.0;
    for (id, v) in items {
        if total <= 0.0 || acc / total < cut {
            out.push(id);
        }
        acc += v;
    }
    out
}

/// The rows of one state.
#[derive(Clone, Debug)]
pub struct StateRows {
    pub state: String,
    /// Communities with a row of their own, most important first.
    pub rows: Vec<CommunityId>,
    /// The make-up of the "other minorities" row; empty when every community has a row.
    pub others: Vec<(CommunityId, f32)>,
}

impl StateRows {
    /// Rows in the population table for this state: one per community plus "other
    /// minorities", each split into `professions` lines at most.
    pub fn lines(&self, professions: usize) -> usize {
        (self.rows.len() + usize::from(!self.others.is_empty())) * professions
    }
}

/// The ABC rule that turns compositions into rows. Proposed values in [`RowRule::PROPOSED`];
/// the reasoning is in `docs/population.pt-BR.md`.
#[derive(Clone, Copy, Debug)]
pub struct RowRule {
    /// Classes A and B: cumulative importance under which a community gets a row.
    pub cut: f64,
    /// Cumulative importance under which a community that already has a row keeps it. Above
    /// `cut`, so keeping a row needs less importance than getting one (hysteresis: a row does
    /// not blink). Used once relevance moves during play; the start date only promotes.
    pub demote_cut: f64,
    /// A community in its country's classes A and B also gets a row in every state where it has
    /// at least this share.
    pub national_floor: f32,
    /// Rows per state at most, "other minorities" not counted.
    pub max_rows: usize,
    pub weights: Weights,
}

/// What each factor adds to a relevance of 1.
#[derive(Clone, Copy, Debug)]
pub struct Weights {
    pub elite: f64,
    pub organization: f64,
    pub legal_status: f64,
    pub patron: f64,
    pub concentration: f64,
}

impl Weights {
    pub fn of(&self, factors: &[Factor]) -> f64 {
        factors
            .iter()
            .map(|f| match f {
                Factor::Elite => self.elite,
                Factor::Organization => self.organization,
                Factor::LegalStatus => self.legal_status,
                Factor::Patron => self.patron,
            })
            .sum()
    }
}

impl RowRule {
    pub const PROPOSED: RowRule = RowRule {
        cut: 0.95,
        demote_cut: 0.97,
        national_floor: 0.02,
        max_rows: 8,
        weights: Weights {
            elite: 3.0,
            organization: 1.0,
            legal_status: 2.0,
            patron: 1.0,
            concentration: 1.0,
        },
    };
}
