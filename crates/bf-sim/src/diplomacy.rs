//! The international world on the start date (phase 4): organizations, treaties, the monetary
//! regime of each country and its average import tariff, as stored in
//! `data/scenario/1925/diplomacy.ron`. Not wired into the simulation yet; see
//! `docs/diplomacy.pt-BR.md` for what each part will feed.
//!
//! Every row carries its dates and its source. Rows describe history: a treaty signed after the
//! start date, or a regime that begins later, is a historical reference for calibration (as the
//! technology years are), not a script; the start state is what is in force on
//! `start_date` ([`DiplomacyFile::treaties_on`], [`DiplomacyFile::regime_on`]).
//!
//! Subject relations (dominions, mandates, protectorates, satellites) live in
//! `countries.ron` ([`crate::scenario::Status`]); a treaty of kind [`TreatyKind::Subjection`]
//! only names the instrument behind one and is checked against it, never restates it.
//!
//! Dates are ISO strings, `YYYY-MM-DD`. Names live in the Fluent files
//! (`assets/i18n/*/diplomacy.ftl`): `org-<key>`, `treaty-<key>`, `treaty-kind-<kind>`,
//! `regime-<kind>`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::scenario::{CountriesFile, Status, Tag};

/// Highest average tariff the data may hold (share of the import value).
pub const MAX_TARIFF: f32 = 0.5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiplomacyFile {
    /// ISO date the file describes; the same as `countries.ron`.
    pub start_date: String,
    pub organizations: Vec<Organization>,
    pub treaties: Vec<Treaty>,
    /// Monetary regime periods; every country has exactly one on `start_date`.
    pub monetary: Vec<MonetaryPeriod>,
    /// Average import tariff in the start year, one row per country.
    pub tariffs: Vec<Tariff>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Organization {
    /// Stable key, `org-<key>` in the Fluent files.
    pub key: String,
    pub kind: OrganizationKind,
    pub founded: String,
    #[serde(default)]
    pub dissolved: Option<String>,
    pub members: Vec<Membership>,
    /// Territories the organization governs or protects itself (the Saar, Danzig).
    #[serde(default)]
    pub wards: Vec<Ward>,
    pub source: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizationKind {
    /// Open to every state (the League of Nations, later the UN).
    Global,
    /// States of one region (the Pan-American Union).
    Regional,
    /// A bloc of allies bound by treaties listed with `bloc` (the Little Entente).
    Alliance,
    /// A monetary union (the Latin Monetary Union).
    Monetary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Membership {
    pub tag: Tag,
    pub joined: String,
    /// Date membership ended (for the League: notice of withdrawal plus two years, under
    /// Article 1 of the Covenant; expulsion or annexation ends it at once).
    #[serde(default)]
    pub left: Option<String>,
    /// Permanent seat on the organization's council.
    #[serde(default)]
    pub permanent_seat: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ward {
    pub tag: Tag,
    pub kind: WardKind,
    pub from: String,
    #[serde(default)]
    pub until: Option<String>,
    pub source: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WardKind {
    /// The organization runs the government (the Saar Governing Commission).
    Governs,
    /// The organization guarantees the constitution and keeps a high commissioner (Danzig).
    Protects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Treaty {
    /// Stable key, `treaty-<key>` in the Fluent files.
    pub key: String,
    pub kind: TreatyKind,
    /// Every state bound by it, at least two.
    pub parties: Vec<Tag>,
    /// For the one-sided kinds ([`TreatyKind::is_asymmetric`]), the parties on the strong side:
    /// guarantors, victors, occupiers, protectors, creditors, the customs territory, the
    /// overlord. Empty for the even kinds.
    #[serde(default)]
    pub senior: Vec<Tag>,
    pub signed: String,
    /// Entry into force; the treaty binds from this date.
    pub in_force: String,
    /// End: expiry, denunciation or replacement. `None` when it outlives what this file records.
    #[serde(default)]
    pub until: Option<String>,
    /// Organization of kind [`OrganizationKind::Alliance`] the treaty belongs to.
    #[serde(default)]
    pub bloc: Option<String>,
    pub source: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreatyKind {
    /// Mutual defence.
    DefensiveAlliance,
    /// Friendship, neutrality or recognition, without a defence clause.
    Friendship,
    /// Promise to consult each other on a region (the Four-Power Treaty).
    Consultation,
    /// Limits on armaments.
    ArmsLimitation,
    /// Guarantors promise to defend a border or a state.
    Guarantee,
    /// Peace imposed by the victors (arms limits, reparations, borders).
    Peace,
    /// Reparations owed to the senior parties.
    Reparations,
    /// Settlement of a war debt owed to the senior party.
    DebtSettlement,
    /// Military occupation of a party's territory by the senior parties.
    Occupation,
    /// The senior party controls the junior's foreign relations, customs or defence without the
    /// junior being its subject in `countries.ron` (Cuba under the Platt Amendment).
    Protection,
    /// The junior parties are inside the senior party's customs territory and apply its tariff.
    CustomsUnion,
    /// Instrument behind a subject relation of `countries.ron`: parties are the subject and its
    /// overlord, the overlord is the senior party.
    Subjection,
}

impl TreatyKind {
    pub const ALL: [TreatyKind; 12] = [
        Self::DefensiveAlliance,
        Self::Friendship,
        Self::Consultation,
        Self::ArmsLimitation,
        Self::Guarantee,
        Self::Peace,
        Self::Reparations,
        Self::DebtSettlement,
        Self::Occupation,
        Self::Protection,
        Self::CustomsUnion,
        Self::Subjection,
    ];

    /// Whether the treaty has a strong side (`senior`).
    pub fn is_asymmetric(self) -> bool {
        !matches!(
            self,
            Self::DefensiveAlliance | Self::Friendship | Self::Consultation | Self::ArmsLimitation
        )
    }

    /// Fluent key, `treaty-kind-<key>`.
    pub fn key(self) -> &'static str {
        match self {
            Self::DefensiveAlliance => "defensive_alliance",
            Self::Friendship => "friendship",
            Self::Consultation => "consultation",
            Self::ArmsLimitation => "arms_limitation",
            Self::Guarantee => "guarantee",
            Self::Peace => "peace",
            Self::Reparations => "reparations",
            Self::DebtSettlement => "debt_settlement",
            Self::Occupation => "occupation",
            Self::Protection => "protection",
            Self::CustomsUnion => "customs_union",
            Self::Subjection => "subjection",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonetaryPeriod {
    pub tag: Tag,
    pub regime: Regime,
    pub from: String,
    /// End of the period; `None` when this file does not record it.
    #[serde(default)]
    pub until: Option<String>,
    /// The date or the parity is an estimate (see the source note).
    #[serde(default)]
    pub estimate: bool,
    pub source: String,
}

/// How a country's money is tied to gold, silver or another currency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Regime {
    /// Convertible into gold, as coin or bullion: grams of fine gold per unit.
    Gold { grams: f64 },
    /// Gold-exchange standard: a gold parity defended with reserves in foreign currency.
    /// `grams` when the parity is legal.
    GoldExchange { grams: Option<f64> },
    /// Fixed to another country's currency (sterling or dollar exchange standard): units per
    /// unit of the anchor. Follows the anchor on and off gold.
    Peg {
        anchor: Tag,
        per_anchor: Option<f64>,
    },
    /// Silver standard: grams of fine silver per unit when known.
    Silver { grams: Option<f64> },
    /// No national money: foreign gold and silver coins (Maria Theresa thalers, sovereigns,
    /// rupees) circulate by weight.
    Specie,
    /// Inconvertible paper money: the rate floats, whatever the law says about gold.
    Paper,
    /// Uses another country's currency (the Indian rupee in Kuwait, the dollar in Panama).
    Foreign { issuer: Tag },
}

impl Regime {
    /// Fluent key, `regime-<key>`.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Gold { .. } => "gold",
            Self::GoldExchange { .. } => "gold_exchange",
            Self::Peg { .. } => "peg",
            Self::Silver { .. } => "silver",
            Self::Specie => "specie",
            Self::Paper => "paper",
            Self::Foreign { .. } => "foreign",
        }
    }

    pub const KEYS: [&'static str; 7] = [
        "gold",
        "gold_exchange",
        "peg",
        "silver",
        "specie",
        "paper",
        "foreign",
    ];

    /// The other country whose money this one follows, if any.
    pub fn follows(&self) -> Option<&str> {
        match self {
            Self::Peg { anchor, .. } => Some(anchor),
            Self::Foreign { issuer } => Some(issuer),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tariff {
    pub tag: Tag,
    /// Import duties over the value of all imports (the Clemens–Williamson measure), 0 to
    /// [`MAX_TARIFF`].
    pub rate: f32,
    /// `false` only when the figure is cited from a source; estimates say how in `source`.
    pub estimate: bool,
    pub source: String,
}

/// Whether `date` is a valid ISO date, `YYYY-MM-DD`.
pub fn is_iso_date(date: &str) -> bool {
    let b = date.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = |r: std::ops::Range<usize>| -> Option<u32> {
        let s = &date[r];
        s.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| s.parse().ok())
            .flatten()
    };
    let (Some(_), Some(m), Some(d)) = (digits(0..4), digits(5..7), digits(8..10)) else {
        return false;
    };
    (1..=12).contains(&m) && (1..=31).contains(&d)
}

/// Whether a period `[from, until)` covers `date`. ISO dates compare as strings.
pub fn covers(from: &str, until: Option<&str>, date: &str) -> bool {
    from <= date && until.is_none_or(|u| date < u)
}

impl DiplomacyFile {
    /// Treaties in force on `date`.
    pub fn treaties_on<'a>(&'a self, date: &'a str) -> impl Iterator<Item = &'a Treaty> + 'a {
        self.treaties
            .iter()
            .filter(move |t| covers(&t.in_force, t.until.as_deref(), date))
    }

    /// Members of organization `key` on `date`.
    pub fn members_on<'a>(&'a self, key: &str, date: &'a str) -> Vec<&'a Membership> {
        self.organizations
            .iter()
            .filter(|o| o.key == key && covers(&o.founded, o.dissolved.as_deref(), date))
            .flat_map(|o| &o.members)
            .filter(|m| covers(&m.joined, m.left.as_deref(), date))
            .collect()
    }

    /// The monetary regime of `tag` on `date`.
    pub fn regime_on(&self, tag: &str, date: &str) -> Option<&MonetaryPeriod> {
        self.monetary
            .iter()
            .find(|p| p.tag == tag && covers(&p.from, p.until.as_deref(), date))
    }

    pub fn tariff(&self, tag: &str) -> Option<&Tariff> {
        self.tariffs.iter().find(|t| t.tag == tag)
    }

    /// Checks the file against itself and the countries of the scenario. Returns every problem
    /// found, one line each.
    pub fn validate(&self, countries: &CountriesFile) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut err = |msg: String| errors.push(msg);
        let tags: BTreeSet<&str> = countries.countries.iter().map(|c| c.tag.as_str()).collect();
        let start = self.start_date.as_str();
        if self.start_date != countries.start_date {
            err(format!(
                "start_date {} differs from countries.ron ({})",
                self.start_date, countries.start_date
            ));
        }

        let check_tag = |at: &str, tag: &str, err: &mut dyn FnMut(String)| {
            if !tags.contains(tag) {
                err(format!("{at}: unknown tag {tag}"));
            }
        };
        let check_period =
            |at: &str, from: &str, until: Option<&str>, err: &mut dyn FnMut(String)| {
                for d in std::iter::once(from).chain(until) {
                    if !is_iso_date(d) {
                        err(format!("{at}: bad date {d:?}"));
                    }
                }
                if let Some(u) = until
                    && u <= from
                {
                    err(format!("{at}: ends ({u}) before it begins ({from})"));
                }
            };
        let check_source = |at: &str, source: &str, err: &mut dyn FnMut(String)| {
            if source.trim().is_empty() {
                err(format!("{at}: no source"));
            }
        };

        // Organizations.
        let mut org_keys = BTreeMap::new();
        for o in &self.organizations {
            let at = format!("organization {}", o.key);
            if org_keys.insert(o.key.as_str(), o.kind).is_some() {
                err(format!("{at}: listed twice"));
            }
            check_period(&at, &o.founded, o.dissolved.as_deref(), &mut err);
            check_source(&at, &o.source, &mut err);
            if o.members.is_empty() {
                err(format!("{at}: no members"));
            }
            let mut seen = BTreeSet::new();
            for m in &o.members {
                let at = format!("{at} member {}", m.tag);
                check_tag(&at, &m.tag, &mut err);
                if !seen.insert(m.tag.as_str()) {
                    err(format!("{at}: listed twice"));
                }
                check_period(&at, &m.joined, m.left.as_deref(), &mut err);
                if m.joined < o.founded {
                    err(format!("{at}: joined before the organization was founded"));
                }
                if let (Some(end), Some(left)) = (&o.dissolved, &m.left)
                    && left > end
                {
                    err(format!("{at}: left after the organization was dissolved"));
                }
            }
            for w in &o.wards {
                let at = format!("{at} ward {}", w.tag);
                check_tag(&at, &w.tag, &mut err);
                check_period(&at, &w.from, w.until.as_deref(), &mut err);
                check_source(&at, &w.source, &mut err);
            }
        }

        // Treaties.
        let mut treaty_keys = BTreeSet::new();
        for t in &self.treaties {
            let at = format!("treaty {}", t.key);
            if !treaty_keys.insert(t.key.as_str()) {
                err(format!("{at}: listed twice"));
            }
            check_source(&at, &t.source, &mut err);
            for d in [&t.signed, &t.in_force] {
                if !is_iso_date(d) {
                    err(format!("{at}: bad date {d:?}"));
                }
            }
            if t.in_force < t.signed {
                err(format!(
                    "{at}: in force ({}) before it was signed ({})",
                    t.in_force, t.signed
                ));
            }
            check_period(&at, &t.in_force, t.until.as_deref(), &mut err);
            let parties: BTreeSet<&str> = t.parties.iter().map(String::as_str).collect();
            if parties.len() != t.parties.len() {
                err(format!("{at}: a party is listed twice"));
            }
            if parties.len() < 2 {
                err(format!("{at}: needs two parties or more"));
            }
            for p in &t.parties {
                check_tag(&at, p, &mut err);
            }
            let senior: BTreeSet<&str> = t.senior.iter().map(String::as_str).collect();
            if let Some(s) = senior.iter().find(|s| !parties.contains(*s)) {
                err(format!("{at}: senior {s} is not a party"));
            }
            if t.kind.is_asymmetric() {
                if senior.is_empty() || senior.len() >= parties.len() {
                    err(format!("{at}: a one-sided treaty needs both sides"));
                }
            } else if !senior.is_empty() {
                err(format!("{at}: an even treaty has no senior side"));
            }
            if let Some(bloc) = &t.bloc
                && org_keys.get(bloc.as_str()) != Some(&OrganizationKind::Alliance)
            {
                err(format!("{at}: bloc {bloc} is not an alliance"));
            }
            if t.kind == TreatyKind::Subjection {
                let (Some(lord), Some(subject)) = (
                    t.senior.first(),
                    t.parties.iter().find(|p| !senior.contains(p.as_str())),
                ) else {
                    continue;
                };
                if parties.len() != 2 {
                    err(format!(
                        "{at}: subjection binds one subject and its overlord"
                    ));
                }
                let status = countries
                    .countries
                    .iter()
                    .find(|c| &c.tag == subject)
                    .map(|c| &c.status);
                let matches =
                    matches!(status, Some(Status::Subject { overlord, .. }) if overlord == lord);
                if covers(&t.in_force, t.until.as_deref(), start) && !matches {
                    err(format!(
                        "{at}: {subject} is not a subject of {lord} in countries.ron"
                    ));
                }
            }
        }

        // Monetary regimes.
        let mut by_tag: BTreeMap<&str, Vec<&MonetaryPeriod>> = BTreeMap::new();
        for p in &self.monetary {
            let at = format!("monetary {} from {}", p.tag, p.from);
            check_tag(&at, &p.tag, &mut err);
            check_period(&at, &p.from, p.until.as_deref(), &mut err);
            check_source(&at, &p.source, &mut err);
            let parity_ok = match &p.regime {
                Regime::Gold { grams } => (0.01..20.0).contains(grams),
                Regime::GoldExchange { grams } => {
                    grams.is_none_or(|g| (0.000_001..20.0).contains(&g))
                }
                Regime::Silver { grams } => grams.is_none_or(|g| (0.1..100.0).contains(&g)),
                Regime::Peg { per_anchor, .. } => {
                    per_anchor.is_none_or(|r| r > 0.0 && r.is_finite())
                }
                Regime::Specie | Regime::Paper | Regime::Foreign { .. } => true,
            };
            if !parity_ok {
                err(format!("{at}: parity out of range"));
            }
            if let Some(other) = p.regime.follows() {
                if other == p.tag {
                    err(format!("{at}: follows itself"));
                }
                check_tag(&at, other, &mut err);
                // The anchor must have money of its own on the day the period starts.
                if let Some(Regime::Foreign { .. }) =
                    self.regime_on(other, &p.from).map(|q| &q.regime)
                {
                    err(format!(
                        "{at}: follows {other}, which has no money of its own"
                    ));
                }
            }
            by_tag.entry(p.tag.as_str()).or_default().push(p);
        }
        for (tag, periods) in &mut by_tag {
            periods.sort_by(|a, b| a.from.cmp(&b.from));
            for pair in periods.windows(2) {
                let end = pair[0].until.as_deref().unwrap_or("9999-12-31");
                if pair[1].from.as_str() < end {
                    err(format!(
                        "monetary {tag}: periods from {} and {} overlap",
                        pair[0].from, pair[1].from
                    ));
                }
            }
        }
        for tag in &tags {
            if self.regime_on(tag, start).is_none() {
                err(format!("monetary {tag}: no regime on {start}"));
            }
        }

        // Tariffs.
        let mut tariff_tags = BTreeSet::new();
        for t in &self.tariffs {
            let at = format!("tariff {}", t.tag);
            check_tag(&at, &t.tag, &mut err);
            if !tariff_tags.insert(t.tag.as_str()) {
                err(format!("{at}: listed twice"));
            }
            if !(t.rate.is_finite() && (0.0..=MAX_TARIFF).contains(&t.rate)) {
                err(format!("{at}: rate {} outside 0..={MAX_TARIFF}", t.rate));
            }
            check_source(&at, &t.source, &mut err);
        }
        for tag in &tags {
            if !tariff_tags.contains(tag) {
                err(format!("tariff {tag}: missing"));
            }
        }
        // A customs union in force puts its junior members under the senior's tariff.
        for t in self
            .treaties_on(start)
            .filter(|t| t.kind == TreatyKind::CustomsUnion)
        {
            let Some(rate) = t
                .senior
                .first()
                .and_then(|s| self.tariff(s))
                .map(|x| x.rate)
            else {
                continue;
            };
            for p in t.parties.iter().filter(|p| !t.senior.contains(p)) {
                if self.tariff(p).is_some_and(|x| x.rate != rate) {
                    err(format!(
                        "treaty {}: {p} applies its own tariff inside a customs union",
                        t.key
                    ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_dates() {
        assert!(is_iso_date("1925-04-28"));
        assert!(!is_iso_date("1925-4-28"));
        assert!(!is_iso_date("1925-13-01"));
        assert!(!is_iso_date("192a-01-01"));
    }

    #[test]
    fn periods_cover_from_but_not_until() {
        assert!(covers("1925-04-28", Some("1931-09-21"), "1925-04-28"));
        assert!(!covers("1925-04-28", Some("1931-09-21"), "1931-09-21"));
        assert!(covers("1900-03-14", None, "2025-01-01"));
        assert!(!covers("1926-09-14", None, "1925-01-01"));
    }
}
