//! Border changes, out of the tick (part A of `docs/borders.pt-BR.md`): the contract of the
//! historical reference `data/history/borders.ron` (every change of sovereignty from 1925 to 2025
//! at the resolution of the map states) and of the catalogue of countries that may be born
//! `data/borders/candidates.ron`.
//!
//! Nothing here is scheduled or saved. The reference is for the scorecard, as the technology years
//! are: no change ever fires by its date. The catalogue gives a country that is born its tag,
//! colour, capital, flag, laws and the states history gave it.
//!
//! Dates are ISO strings, `YYYY-MM-DD`. Names live in `assets/i18n/*/countries.ftl`
//! (`country-<TAG>`, `.article`, `country-<TAG>-capital`), as for the 1925 countries.

pub mod independence;
pub mod op;

pub use op::{BorderRules, BordersState, Heir, Operation, Terms};

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::scenario::{CountriesFile, MapFile, Tag};

/// The changes of sovereignty of the reference, oldest first.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BordersFile {
    pub changes: Vec<SovereigntyChange>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeKind {
    /// States move between two countries that exist (cession, partial annexation, return).
    Transfer,
    /// States form a new or restored country (one line per heir).
    Secession,
    /// A whole country ends inside another; `states` is empty and means all it holds.
    Annexation,
    /// No territory moves: `from` becomes a subject of `to`, or sovereign when `to == from`.
    Status,
    /// The same country takes a new name (`from == to`).
    Rename,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SovereigntyChange {
    pub date: String,
    pub kind: ChangeKind,
    pub from: Tag,
    pub to: Tag,
    /// Map state keys that move whole: a state moves with the majority of its area.
    pub states: Vec<String>,
    /// States that changed hands only in part and did not move, with the share that did.
    pub partial: Vec<(String, f32)>,
    /// Occupying powers at creation or during a status, comma-separated tags.
    pub occupier: Option<String>,
    pub wikidata: String,
    pub source: String,
    pub note: String,
}

/// The countries that may be born after 1925.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidatesFile {
    pub countries: Vec<Candidate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub tag: Tag,
    pub wikidata: String,
    pub color: (u8, u8, u8),
    pub capital_lon_lat: (f64, f64),
    /// `assets/flags/<flag>.png`, attributed in `assets/flags/ATTRIBUTION.md`.
    pub flag: Option<String>,
    /// Form of government at birth (a law of the `government` group); `None` for the rule of
    /// `docs/borders.pt-BR.md`.
    #[serde(default)]
    pub government: Option<String>,
    /// Template of `data/scenario/1925/laws.ron` the country is born with; `None` for the rule of
    /// `docs/borders.pt-BR.md` (the economy laws in force in the territory, `new_state` for the
    /// rest).
    #[serde(default)]
    pub laws: Option<String>,
    /// The states history gave it, at map resolution; empty when it never held a whole state
    /// (microstates, islands below the resolution) or is only a new name of another country.
    pub states: Vec<String>,
    /// For a name a country of 1925 or of the catalogue took later (Iran, Thailand, Russia), that
    /// country: the row only carries the name, the capital and the flag.
    #[serde(default)]
    pub rename_of: Option<Tag>,
    pub born: String,
    pub ended: Option<String>,
    pub source: String,
}

pub fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        && (1..=12).contains(&s[5..7].parse::<u8>().unwrap_or(0))
        && (1..=31).contains(&s[8..10].parse::<u8>().unwrap_or(0))
}

impl BordersFile {
    /// The owner of every map state after the changes up to and including `date`, replayed from
    /// the 1925 map. Changes that do not apply (a state not held by `from`) are skipped here; see
    /// [`BordersFile::validate`].
    pub fn owners_on(&self, map: &MapFile, date: &str) -> BTreeMap<String, Tag> {
        let mut owner: BTreeMap<String, Tag> = map
            .states
            .iter()
            .map(|s| (s.key.clone(), s.owner.clone()))
            .collect();
        for c in self.changes.iter().take_while(|c| c.date.as_str() <= date) {
            apply(&mut owner, c);
        }
        owner
    }

    /// Every rule of the contract; all errors at once.
    pub fn validate(
        &self,
        map: &MapFile,
        countries: &CountriesFile,
        candidates: &CandidatesFile,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let known: BTreeSet<&str> = countries
            .countries
            .iter()
            .map(|c| c.tag.as_str())
            .chain(candidates.countries.iter().map(|c| c.tag.as_str()))
            .collect();
        let mut owner: BTreeMap<String, Tag> = map
            .states
            .iter()
            .map(|s| (s.key.clone(), s.owner.clone()))
            .collect();
        let mut last = countries.start_date.clone();
        // Date each country was last born: a country born of several owners on one day (Libya
        // in 1951, out of the British and the French administrations) takes several lines.
        let mut born: BTreeMap<&str, &str> = BTreeMap::new();
        for (i, c) in self.changes.iter().enumerate() {
            let at = format!(
                "change {i} ({} {:?} {} -> {})",
                c.date, c.kind, c.from, c.to
            );
            if !is_iso_date(&c.date) {
                errors.push(format!("{at}: bad date"));
            } else if c.date < last {
                errors.push(format!("{at}: out of order (after {last})"));
            } else {
                last = c.date.clone();
            }
            if c.date.as_str() > "2025-12-31" {
                errors.push(format!("{at}: after 2025"));
            }
            for t in [&c.from, &c.to] {
                if !known.contains(t.as_str()) {
                    errors.push(format!("{at}: unknown tag {t}"));
                }
            }
            for t in c.occupier.iter().flat_map(|o| o.split(',')) {
                if !known.contains(t.trim()) {
                    errors.push(format!("{at}: unknown occupier {t}"));
                }
            }
            if c.source.trim().is_empty() || !c.wikidata.starts_with('Q') {
                errors.push(format!("{at}: needs a source and a Wikidata item"));
            }
            let held = |tag: &str, owner: &BTreeMap<String, Tag>| {
                owner.values().filter(|o| o.as_str() == tag).count()
            };
            let mut seen = BTreeSet::new();
            for s in &c.states {
                if !seen.insert(s) {
                    errors.push(format!("{at}: state {s} listed twice"));
                }
                match owner.get(s) {
                    None => errors.push(format!("{at}: unknown state {s}")),
                    Some(o) if *o != c.from => {
                        errors.push(format!("{at}: state {s} is held by {o}, not {}", c.from))
                    }
                    _ => {}
                }
            }
            for (s, share) in &c.partial {
                if !owner.contains_key(s) {
                    errors.push(format!("{at}: unknown partial state {s}"));
                }
                if !(*share > 0.0 && *share < 1.0) {
                    errors.push(format!("{at}: partial share of {s} outside (0, 1)"));
                }
            }
            match c.kind {
                ChangeKind::Transfer | ChangeKind::Secession => {
                    if c.states.is_empty() {
                        errors.push(format!("{at}: moves no state"));
                    }
                    if c.from == c.to {
                        errors.push(format!("{at}: from and to are the same"));
                    }
                    if c.kind == ChangeKind::Secession {
                        if held(&c.to, &owner) > 0
                            && born.get(c.to.as_str()) != Some(&c.date.as_str())
                        {
                            errors.push(format!("{at}: {} already holds states", c.to));
                        }
                        born.insert(&c.to, &c.date);
                    }
                    if c.kind == ChangeKind::Transfer && held(&c.to, &owner) == 0 {
                        errors.push(format!("{at}: {} holds no state to receive", c.to));
                    }
                }
                ChangeKind::Annexation => {
                    if !c.states.is_empty() {
                        errors.push(format!("{at}: an annexation lists no states"));
                    }
                    if c.from == c.to {
                        errors.push(format!("{at}: from and to are the same"));
                    }
                }
                ChangeKind::Status => {
                    if !c.states.is_empty() {
                        errors.push(format!("{at}: a status change moves no state"));
                    }
                }
                ChangeKind::Rename => {
                    if !c.states.is_empty() || c.from != c.to {
                        errors.push(format!("{at}: a rename moves nothing and keeps the tag"));
                    }
                }
            }
            apply(&mut owner, c);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn apply(owner: &mut BTreeMap<String, Tag>, c: &SovereigntyChange) {
    match c.kind {
        ChangeKind::Transfer | ChangeKind::Secession => {
            for s in &c.states {
                if let Some(o) = owner.get_mut(s)
                    && *o == c.from
                {
                    o.clone_from(&c.to);
                }
            }
        }
        ChangeKind::Annexation => {
            for o in owner.values_mut() {
                if *o == c.from {
                    o.clone_from(&c.to);
                }
            }
        }
        ChangeKind::Status | ChangeKind::Rename => {}
    }
}

impl CandidatesFile {
    pub fn get(&self, tag: &str) -> Option<&Candidate> {
        self.countries.iter().find(|c| c.tag == tag)
    }

    pub fn validate(&self, map: &MapFile, countries: &CountriesFile) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let old: BTreeSet<&str> = countries.countries.iter().map(|c| c.tag.as_str()).collect();
        let states: BTreeSet<&str> = map.states.iter().map(|s| s.key.as_str()).collect();
        let mut tags = BTreeSet::new();
        for c in &self.countries {
            let at = format!("candidate {}", c.tag);
            if !tags.insert(c.tag.as_str()) {
                errors.push(format!("{at}: listed twice"));
            }
            if old.contains(c.tag.as_str()) {
                errors.push(format!("{at}: tag of a 1925 country"));
            }
            if !(3..=4).contains(&c.tag.len()) || !c.tag.bytes().all(|b| b.is_ascii_uppercase()) {
                errors.push(format!("{at}: tag must be 3 or 4 capital letters"));
            }
            if !c.wikidata.starts_with('Q') || c.source.trim().is_empty() {
                errors.push(format!("{at}: needs a source and a Wikidata item"));
            }
            for d in std::iter::once(&c.born).chain(c.ended.as_ref()) {
                if !is_iso_date(d) {
                    errors.push(format!("{at}: bad date {d:?}"));
                }
            }
            if let Some(e) = &c.ended
                && *e <= c.born
            {
                errors.push(format!("{at}: ends before it is born"));
            }
            let (lon, lat) = c.capital_lon_lat;
            if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
                errors.push(format!("{at}: capital outside the globe"));
            }
            for s in &c.states {
                if !states.contains(s.as_str()) {
                    errors.push(format!("{at}: unknown state {s}"));
                }
            }
            if let Some(r) = &c.rename_of {
                if !old.contains(r.as_str()) && self.get(r).is_none() {
                    errors.push(format!("{at}: rename of unknown {r}"));
                }
                if !c.states.is_empty() {
                    errors.push(format!("{at}: a new name holds no states of its own"));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// What a change of owner must neither create nor destroy, summed over the world: the
/// conservation check of every operation (`docs/borders.pt-BR.md`, "Conservação"). Debt in
/// foreign currency is in reference units, so a split never depends on an exchange rate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub people: i64,
    /// Savings of the pops.
    pub savings: laplace_schelling::Fixed,
    /// Investment pools of the markets.
    pub pool: laplace_schelling::Fixed,
    pub treasury: laplace_schelling::Fixed,
    /// Public debt by holder: at home, at the central bank, abroad in local currency, and in
    /// foreign currency (reference units).
    pub debt: [laplace_schelling::Fixed; 4],
    /// What savers lent to other states.
    pub foreign_assets: laplace_schelling::Fixed,
    /// People in the community rows of the population (`DemographyState::communities`).
    pub community_people: i64,
    /// People abroad (the diaspora) and those of them who arrived since the start.
    pub diaspora: i64,
    pub arrived: i64,
    /// Plant levels, and levels under construction.
    pub levels: i64,
    pub works: usize,
    /// Debts between governments (`Economy::ledger`): claims minus debts, world-wide. Zero by
    /// construction; an annexation that joins a debtor and its creditor removes the line from
    /// both sides at once.
    pub ledger_net: laplace_schelling::Fixed,
}

impl Totals {
    pub fn of(world: &crate::game::GameWorld) -> Self {
        let mut t = Self::default();
        for r in &world.econ.regions {
            for p in &r.pops {
                t.people += p.size;
                t.savings += p.savings;
            }
            t.levels += r.plants.iter().map(|p| p.levels).sum::<i64>();
        }
        t.community_people = world.demography.communities.iter().map(|r| r.total()).sum();
        for d in &world.demography.diaspora {
            t.diaspora += d.people;
            t.arrived += d.arrived;
        }
        let ledger = &world.econ.ledger;
        for c in 0..world.econ.markets.len() as u16 {
            t.ledger_net += ledger.claims_of(c) - ledger.owed_by(c);
        }
        for m in &world.econ.markets {
            let y = &m.money;
            t.pool += m.pool;
            t.treasury += m.treasury;
            for (d, v) in
                t.debt
                    .iter_mut()
                    .zip([y.debt_home, y.debt_bank, y.debt_abroad, y.debt_fx])
            {
                *d += v;
            }
            t.foreign_assets += y.foreign_assets;
            t.works += m.queue.len();
        }
        t
    }
}
