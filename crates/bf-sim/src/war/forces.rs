//! Armies, navies and air forces on the start date (`data/scenario/1925/forces.ron`).
//!
//! Each row is one unit type of one country, counted in the element of its domain: men for land
//! (the catalogue's element is 1,000 men), ships for the sea, aircraft for the air. Every row has
//! a source; a row whose number is not stated by the source carries `estimate` with the method.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::catalogue::{Domain, Military};
use crate::research::{Stage, TechTree};
use crate::scenario::{CountriesFile, Tag};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForcesFile {
    /// ISO date the file describes; the same as `countries.ron`.
    pub start_date: String,
    pub countries: Vec<CountryForces>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountryForces {
    pub tag: Tag,
    pub rows: Vec<ForceRow>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Station {
    /// In the metropole (or the whole country, for one without colonies).
    #[default]
    Metropole,
    /// Raised or stationed in the colonies.
    Colonies,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForceRow {
    /// Unit type key in `data/military/units.ron`.
    #[serde(rename = "type")]
    pub unit: String,
    /// Men (land), ships (sea) or aircraft (air and strategic).
    pub count: u64,
    #[serde(default)]
    pub station: Station,
    /// Total displacement of the ships of the row, in tonnes, when the source gives it. The
    /// catalogue's tonnage is a class average; a navy of small torpedo boats counted as
    /// destroyers keeps its real weight here.
    #[serde(default)]
    pub tonnage: Option<u64>,
    /// Trained men out of service (reserve, militia, territorials), called up on mobilization
    /// ([`crate::war::call_up`]). Land only.
    #[serde(default)]
    pub reserve: bool,
    pub source: String,
    /// How the number was estimated, when the source does not state it.
    #[serde(default)]
    pub estimate: Option<String>,
}

impl ForceRow {
    /// Elements of the catalogue: count ÷ 1,000 for land, the count itself otherwise.
    pub fn elements(&self, military: &Military) -> u64 {
        match military
            .unit(&self.unit)
            .map(|t| military.types[t as usize].domain)
        {
            Some(Domain::Land) => self.count.div_ceil(1000),
            _ => self.count,
        }
    }
}

impl ForcesFile {
    pub fn country(&self, tag: &str) -> Option<&CountryForces> {
        self.countries.iter().find(|c| c.tag == tag)
    }

    /// Men in the reserve of a country.
    pub fn reserve(&self, tag: &str) -> u64 {
        self.country(tag).map_or(0, |c| {
            c.rows.iter().filter(|r| r.reserve).map(|r| r.count).sum()
        })
    }

    /// Sum of the counts of one domain for a country, in service (the reserve apart).
    pub fn total(&self, military: &Military, tag: &str, domain: Domain) -> u64 {
        self.country(tag).map_or(0, |c| {
            c.rows
                .iter()
                .filter(|r| !r.reserve)
                .filter(|r| {
                    military
                        .unit(&r.unit)
                        .is_some_and(|t| military.types[t as usize].domain == domain)
                })
                .map(|r| r.count)
                .sum()
        })
    }

    /// Rows with `estimate`, and rows without.
    pub fn estimated_and_sourced(&self) -> (usize, usize) {
        let rows = self.countries.iter().flat_map(|c| &c.rows);
        let estimated = rows.clone().filter(|r| r.estimate.is_some()).count();
        (estimated, rows.count() - estimated)
    }

    /// Checks the file against the catalogue, the tree and the countries. Every problem is
    /// reported.
    pub fn validate(
        &self,
        military: &Military,
        tree: &TechTree,
        countries: &CountriesFile,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.start_date != countries.start_date {
            errors.push(format!(
                "start_date {} differs from countries.ron {}",
                self.start_date, countries.start_date
            ));
        }
        let known: BTreeSet<&str> = countries.countries.iter().map(|c| c.tag.as_str()).collect();
        let mut seen = BTreeSet::new();
        for c in &self.countries {
            if !known.contains(c.tag.as_str()) {
                errors.push(format!("{}: not a country of the scenario", c.tag));
            }
            if !seen.insert(c.tag.as_str()) {
                errors.push(format!("{}: listed twice", c.tag));
            }
            let has_colonies = countries
                .countries
                .iter()
                .find(|d| d.tag == c.tag)
                .is_some_and(|d| d.colonies.is_some());
            let mut rows = BTreeSet::new();
            for r in &c.rows {
                let what = format!("{} {}", c.tag, r.unit);
                if !rows.insert((r.unit.as_str(), r.station as u8, r.reserve)) {
                    errors.push(format!("{what}: listed twice for the same station"));
                }
                if r.count == 0 {
                    errors.push(format!("{what}: count is zero"));
                }
                if r.source.trim().is_empty() {
                    errors.push(format!("{what}: no source"));
                }
                if r.estimate.as_ref().is_some_and(|e| e.trim().is_empty()) {
                    errors.push(format!("{what}: empty estimate method"));
                }
                if r.station == Station::Colonies && !has_colonies {
                    errors.push(format!(
                        "{what}: stationed in colonies the country does not have"
                    ));
                }
                let Some(ty) = military.unit(&r.unit) else {
                    errors.push(format!("{what}: unknown unit type"));
                    continue;
                };
                if r.reserve && military.types[ty as usize].domain != Domain::Land {
                    errors.push(format!("{what}: only land forces have a reserve"));
                }
                // The first generation must be known in the country on the start date.
                for &t in &military.spec(ty, 0).requires {
                    if tree.start_stage(t as usize, &c.tag) == Stage::Unknown {
                        errors.push(format!(
                            "{what}: needs {}, unknown in {} on the start date",
                            tree.techs[t as usize].key, c.tag
                        ));
                    }
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
