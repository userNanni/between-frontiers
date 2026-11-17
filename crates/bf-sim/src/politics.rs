//! Phase 3: national politics. Laws, interest groups and movements, as a contract and a
//! catalogue; what changes in the game, and the system that moves it, is in `crate::polity`.
//!
//! - A **law** belongs to exactly one **law group**, and every country holds exactly one law of
//!   each group (the colonial groups only for countries with colonies). A law does nothing by
//!   itself: it declares **effects**, numbers keyed by [`EFFECTS`], on things the simulation has
//!   (migration, the budget, incomes) or will have (unions, media, communities, colonies).
//! - An **interest group** is a bloc of people with a shared material interest, drawn from the 13
//!   professions of `economy::PROFESSIONS` (and, optionally, from one community, by id). Its
//!   stance on a law is not a label: it is the sum, over the law's effects, of how much each
//!   effect helps or hurts the group ([`Politics::score`]). The stance is always explainable
//!   effect by effect ([`Politics::terms`]).
//! - A **movement** wants one law (or independence) and grows only while its **cause** holds: a
//!   list of measurable conditions on the state of the simulation ([`INDICATORS`]) or on the laws
//!   in force ([`EFFECTS`]). No movement is drawn by lot; the weakest condition sets its pace.
//!
//! Data in `data/politics/{laws,interest_groups,movements}.ron`; laws in force on 1 January 1925
//! in `data/scenario/1925/laws.ron`. Names in `assets/i18n/*/politics.ftl`: `law-category-<key>`,
//! `law-group-<key>`, `law-<key>` and `law-<key>-desc`, `law-effect-<key>`, `indicator-<key>`,
//! `ig-<key>` and `ig-<key>-desc`, `movement-<key>` and `movement-<key>-desc`. Rules and sources in
//! `docs/politics.pt-BR.md`.

use serde::{Deserialize, Serialize};

use crate::economy::PROFESSIONS;
use crate::scenario::CountryDef;

/// The law group of the forms of government (`CountryDef::government` keys).
pub const GOVERNMENT_GROUP: &str = "government";

/// Categories of law groups, in display order.
pub const CATEGORIES: [&str; 7] = [
    "economy",
    "labour",
    "welfare",
    "rights",
    "information",
    "state",
    "empire",
];

/// The part of the simulation an effect or an indicator belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum System {
    /// Internal migration (`econ::migration`).
    Migration,
    /// Taxes, payroll, schools and hospitals (`econ::public`).
    Budget,
    /// Wages, profits and rents of the pops (`econ`).
    Income,
    /// National markets and trade (`econ`).
    Market,
    /// Unions, hours, minimum wage, child and unfree labour (phase 3).
    Labour,
    /// Pensions, unemployment pay and medicines (phase 3).
    Welfare,
    /// Firms owned by the state, with profit to the treasury (phase 3).
    StateEnterprise,
    /// Money, inflation and the central bank (phase 3c).
    Money,
    /// Communities: discrimination, segregation, assimilation (phase 3).
    Communities,
    /// Vote, repression, church, corruption (phase 3, this module).
    Politics,
    /// Press, radio, television and internet (phase 3).
    Media,
    /// Colonial rule and decolonization (phase 3).
    Colonies,
    /// Tariffs, foreign capital, borders (phase 4).
    Borders,
    /// Armies (phase 5).
    Military,
}

impl System {
    /// The i18n-free key used in docs and tooling.
    pub fn key(self) -> &'static str {
        match self {
            System::Migration => "migration",
            System::Budget => "budget",
            System::Income => "income",
            System::Market => "market",
            System::Labour => "labour",
            System::Welfare => "welfare",
            System::StateEnterprise => "state_enterprise",
            System::Money => "money",
            System::Communities => "communities",
            System::Politics => "politics",
            System::Media => "media",
            System::Colonies => "colonies",
            System::Borders => "borders",
            System::Military => "military",
        }
    }
}

/// How the values that several laws give to one effect add up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combine {
    /// Per mille multipliers: 1000 is neutral, 250 lets a quarter through. They multiply.
    Multiply,
    /// Per mille shares added to zero: 300 is 30 %. Negative values take away.
    Add,
    /// Per mille ceilings: 1000 is no ceiling; the lowest one wins.
    Cap,
}

/// What a law can change.
#[derive(Clone, Copy, Debug)]
pub struct EffectKind {
    pub key: &'static str,
    pub combine: Combine,
    pub system: System,
    /// The code that already holds the quantity the effect will change; `None` when the system
    /// is still to be built.
    pub existing: Option<&'static str>,
}

impl EffectKind {
    /// Value with no law acting.
    pub const fn neutral(&self) -> i32 {
        match self.combine {
            Combine::Multiply | Combine::Cap => 1000,
            Combine::Add => 0,
        }
    }

    /// Allowed values, inclusive, for one law and for the combined value.
    pub const fn range(&self) -> (i32, i32) {
        match self.combine {
            Combine::Multiply => (0, 5000),
            Combine::Add => (-1000, 1000),
            Combine::Cap => (0, 1000),
        }
    }

    /// Distance from neutral, the unit of every stance.
    pub const fn deviation(&self, value: i32) -> i32 {
        value - self.neutral()
    }
}

const fn effect(
    key: &'static str,
    combine: Combine,
    system: System,
    existing: Option<&'static str>,
) -> EffectKind {
    EffectKind {
        key,
        combine,
        system,
        existing,
    }
}

use Combine::{Add, Cap, Multiply};

/// Every effect a law may declare, per mille.
pub const EFFECTS: [EffectKind; 40] = [
    // Share of the open migration flows let through (the `MigrationLaw` reach).
    effect(
        "migration_within_state",
        Multiply,
        System::Migration,
        Some("econ::migration::MigrationLaw"),
    ),
    effect(
        "migration_between_states",
        Multiply,
        System::Migration,
        Some("econ::migration::MigrationLaw"),
    ),
    // Highest income tax rate of each class (`econ::public::CLASS_OF`).
    effect(
        "tax_cap_lower",
        Cap,
        System::Budget,
        Some("econ::state::Budget::tax_rates"),
    ),
    effect(
        "tax_cap_middle",
        Cap,
        System::Budget,
        Some("econ::state::Budget::tax_rates"),
    ),
    effect(
        "tax_cap_upper",
        Cap,
        System::Budget,
        Some("econ::state::Budget::tax_rates"),
    ),
    // Share of the assessed tax that reaches the treasury.
    effect(
        "tax_collection",
        Multiply,
        System::Budget,
        Some("econ::public"),
    ),
    // Share of hospital care and of schooling the treasury pays (today it pays all).
    effect(
        "health_state_share",
        Add,
        System::Budget,
        Some("econ::public"),
    ),
    effect(
        "education_state_share",
        Add,
        System::Budget,
        Some("econ::public"),
    ),
    // Share of the school-age children the law wants in school.
    effect(
        "school_enrolment_target",
        Add,
        System::Budget,
        Some("econ::public"),
    ),
    // Landowners' share of farm income, relative to 1925 free tenancy.
    effect(
        "landowner_rent",
        Multiply,
        System::Income,
        Some("econ::state::Pop::income"),
    ),
    // Share of the price of medicines the state pays.
    effect("medicine_subsidy", Add, System::Welfare, None),
    // Old-age pension and unemployment pay, as a share of the average wage.
    effect("pension", Add, System::Welfare, None),
    effect("unemployment_benefit", Add, System::Welfare, None),
    // Minimum wage as a share of the average wage.
    effect("wage_floor", Add, System::Labour, None),
    // Weight of unions in setting wages.
    effect("union_bargaining", Add, System::Labour, None),
    // Hours worked relative to the 1925 average (48-hour week ~ 950).
    effect("working_hours", Multiply, System::Labour, None),
    // Share of children 10–14 at work.
    effect("child_labour", Add, System::Labour, None),
    // Share of labour that is not free (serfdom, slavery, corvée).
    effect("unfree_labour", Add, System::Labour, None),
    // Share of existing industry owned by the state, share of new investment it makes, and
    // share of the profit of state firms that goes to the treasury.
    effect("state_ownership", Add, System::StateEnterprise, None),
    effect("state_investment_share", Add, System::StateEnterprise, None),
    effect(
        "state_profit_to_treasury",
        Add,
        System::StateEnterprise,
        None,
    ),
    // Ad valorem tariff on imports, and the pull of foreign capital.
    effect("tariff", Add, System::Borders, None),
    effect("foreign_investment", Multiply, System::Borders, None),
    // Share of would-be immigrants and emigrants let through.
    effect("immigration", Multiply, System::Borders, None),
    effect("emigration", Multiply, System::Borders, None),
    // Wage penalty and share of jobs closed to the communities a law targets; speed of
    // assimilation (`docs/architecture.pt-BR.md`, "Ninguém some em outras minorias").
    effect("discrimination", Add, System::Communities, None),
    effect("segregation", Add, System::Communities, None),
    effect("assimilation", Multiply, System::Communities, None),
    // Share of adults with a vote that can change the government.
    effect("franchise", Add, System::Politics, None),
    // How hard the state puts down movements and strikes.
    effect("repression", Add, System::Politics, None),
    // Public money and legal privilege of the established religion (negative: restricted).
    effect("church_privilege", Add, System::Politics, None),
    // Share of the budget lost to graft.
    effect("corruption", Add, System::Politics, None),
    // Share of what is printed and broadcast the state vets, and share of the media it owns.
    effect("censorship", Add, System::Media, None),
    effect("state_media", Add, System::Media, None),
    // Share of the men of service age called up.
    effect("conscription", Add, System::Military, None),
    // Share of colonial revenue sent to the metropole, and share of colonial offices held by
    // the colonized.
    effect("colonial_tribute", Add, System::Colonies, None),
    effect("colonial_self_rule", Add, System::Colonies, None),
    // Share of the pioneer's clearing the land law lets him keep: free settlement on public
    // land is full; estates that claim the frontier leave the settler little.
    effect(
        "frontier_access",
        Multiply,
        System::Income,
        Some("econ::monthly::pioneers"),
    ),
    // Share of research and development the treasury pays (the firms' fund pays the rest), and
    // how much more research the state wants above its base share of revenue
    // (`docs/research.pt-BR.md`, "Pesquisa que custa").
    effect(
        "research_state_share",
        Add,
        System::Budget,
        Some("econ::public"),
    ),
    effect(
        "research_priority",
        Add,
        System::Budget,
        Some("econ::public"),
    ),
];

pub fn effect_index(key: &str) -> Option<usize> {
    EFFECTS.iter().position(|e| e.key == key)
}

pub fn effect_kind(index: usize) -> &'static EffectKind {
    &EFFECTS[index]
}

/// What a movement's cause can measure, per mille of the population it draws from (or of the
/// country, for country-wide quantities), up to `max`.
#[derive(Clone, Copy, Debug)]
pub struct IndicatorKind {
    pub key: &'static str,
    pub max: i32,
    pub system: System,
    /// Where the simulation already computes it; `None` when it is still to be built.
    pub existing: Option<&'static str>,
}

const fn indicator(
    key: &'static str,
    max: i32,
    system: System,
    existing: Option<&'static str>,
) -> IndicatorKind {
    IndicatorKind {
        key,
        max,
        system,
        existing,
    }
}

pub const INDICATORS: [IndicatorKind; 18] = [
    indicator(
        "literacy",
        1000,
        System::Income,
        Some("econ::state::Region::literacy"),
    ),
    // Share of the people in urban professions.
    indicator(
        "urbanization",
        1000,
        System::Migration,
        Some("econ::migration::is_urban"),
    ),
    // Share of the working people without a job.
    indicator(
        "unemployment",
        1000,
        System::Income,
        Some("econ::state::Pop::employed"),
    ),
    // Share of the basic needs met.
    indicator(
        "needs_met",
        1000,
        System::Income,
        Some("econ::state::Pop::satisfaction"),
    ),
    indicator(
        "school_coverage",
        1000,
        System::Budget,
        Some("econ::public::coverage"),
    ),
    indicator(
        "hospital_coverage",
        1000,
        System::Budget,
        Some("econ::public::coverage"),
    ),
    // Town pay over country pay (1500 = 1.5 times), as the migration rule reads it.
    indicator(
        "rural_wage_gap",
        10000,
        System::Migration,
        Some("econ::migration"),
    ),
    // Taxes over GDP.
    indicator(
        "tax_burden",
        1000,
        System::Budget,
        Some("econ::state::Budget::revenue"),
    ),
    // Imports over the national supply of manufactures.
    indicator(
        "import_share",
        1000,
        System::Market,
        Some("econ::state::Market::net_import"),
    ),
    // Landowners' share of farm income.
    indicator(
        "landowner_income_share",
        1000,
        System::Income,
        Some("econ::state::Pop::income"),
    ),
    // Yearly rise of prices, 1000 = prices doubled.
    indicator("price_change", 1000, System::Money, None),
    // Income share of the richest tenth.
    indicator("inequality", 1000, System::Income, None),
    // Share of the people the media reaches.
    indicator("media_reach", 1000, System::Media, None),
    // Share of the people of foreign origin (`polity::measure::immigrant_share`, zero until the
    // demography keeps the stock of migrants by origin).
    indicator("immigrant_share", 1000, System::Communities, None),
    // Consent to the laws in force (`polity`): 1000 less what unanswered movements mobilize.
    indicator(
        "legitimacy",
        1000,
        System::Politics,
        Some("polity::PoliticsState::legitimacy"),
    ),
    // Share of the people of the country's 1914 lands now under another state (needs the
    // borders of phase 4).
    indicator("lost_territory", 1000, System::Borders, None),
    // How far the treasury has to cut what it buys and pays (1000 less `econ::public::austerity`):
    // cash it could not borrow, and interest eating the revenue.
    indicator(
        "fiscal_stress",
        1000,
        System::Budget,
        Some("econ::public::austerity"),
    ),
    // Growth of GDP over the last full year, per mille (30 = 3 %); a fall reads as 0.
    indicator(
        "gdp_growth",
        1000,
        System::Income,
        Some("econ::state::Economy::history"),
    ),
];

pub fn indicator_index(key: &str) -> Option<usize> {
    INDICATORS.iter().position(|i| i.key == key)
}

// ---------------------------------------------------------------------------------------------
// Files

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LawsFile {
    pub groups: Vec<LawGroupDef>,
    pub laws: Vec<LawDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LawGroupDef {
    pub key: String,
    /// One of [`CATEGORIES`].
    pub category: String,
    /// Held only by countries with colonies, and applied only in their colonial regions.
    #[serde(default)]
    pub colonial: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LawDef {
    pub key: String,
    /// Key of a [`LawGroupDef`].
    pub group: String,
    /// The effects fall only on the communities the scenario names for this law's group in
    /// each country (`CountryLaws::targets`), not on everyone.
    #[serde(default)]
    pub targeted: bool,
    /// (effect key of [`EFFECTS`], per mille). Effects of the colonies system only in colonial
    /// groups.
    pub effects: Vec<(String, i32)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterestGroupsFile {
    pub groups: Vec<InterestGroupDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterestGroupDef {
    pub key: String,
    /// (profession, per mille of the profession that belongs to the group). Among the groups
    /// without a community, the shares of each profession add up to 1000: everyone belongs to
    /// one bloc, and no one to two.
    pub professions: Vec<(String, u16)>,
    /// Community id (culture + religion; `docs/architecture.pt-BR.md`): the group draws only
    /// from it, taking its shares out of the general groups for that community.
    #[serde(default)]
    pub community: Option<String>,
    /// (effect key, weight from -3 to 3): how much a rise of the effect helps (+) or hurts (-)
    /// the group's members.
    pub interests: Vec<(String, i8)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementsFile {
    pub movements: Vec<MovementDef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Goal {
    /// Enact this law.
    Law(String),
    /// End colonial or subject rule over the people the movement draws from.
    Independence,
    /// Win back territory and standing lost (needs borders and war; never won before them).
    Revision,
}

/// Whom a movement draws from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Among {
    /// Everyone in the metropole (filtered by profession).
    #[default]
    Everyone,
    /// People under colonial rule: the colonial regions of an owner, and every region of a
    /// subject country.
    Colonies,
    /// The communities a law in force targets.
    Targeted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bound {
    AtLeast(i32),
    AtMost(i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementDef {
    pub key: String,
    pub goal: Goal,
    #[serde(default)]
    pub among: Among,
    /// (profession, per mille of it that can join); empty means every profession in full.
    #[serde(default)]
    pub professions: Vec<(String, u16)>,
    /// All must hold: (indicator key of [`INDICATORS`] or effect key of [`EFFECTS`] measured on
    /// the laws in force, bound).
    pub cause: Vec<(String, Bound)>,
    /// Per mille of the people it draws from that join per year when every condition is met
    /// by the widest margin.
    pub growth: u16,
    /// Years for support to halve once the cause no longer holds: movements outlive their
    /// cause.
    pub persistence: u16,
}

/// Laws in force on the scenario start date: `data/scenario/1925/laws.ron`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioLawsFile {
    /// Law sets shared by several countries; a country overrides group by group.
    pub templates: Vec<LawTemplate>,
    pub countries: Vec<CountryLaws>,
    /// Dated law changes after the start, as reference for calibration and for the AI. Nothing
    /// fires them.
    #[serde(default)]
    pub history: Vec<LawChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LawTemplate {
    pub key: String,
    pub laws: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountryLaws {
    pub tag: String,
    #[serde(default)]
    pub template: Option<String>,
    /// Laws replacing the template's in their group, or the whole set without a template.
    #[serde(default)]
    pub laws: Vec<String>,
    /// (law group, community ids) for the targeted law of that group.
    #[serde(default)]
    pub targets: Vec<(String, Vec<String>)>,
    /// The acts and constitutions behind the choice.
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawChange {
    pub tag: String,
    /// `YYYY-MM-DD`, after the start date.
    pub date: String,
    pub law: String,
    pub source: String,
}

// ---------------------------------------------------------------------------------------------
// Resolved catalogue

#[derive(Clone, Debug)]
pub struct LawGroup {
    pub key: String,
    pub category: u8,
    pub colonial: bool,
    /// Its laws, in file order.
    pub laws: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct Law {
    pub key: String,
    pub group: u16,
    pub targeted: bool,
    /// (index into [`EFFECTS`], per mille).
    pub effects: Vec<(u8, i32)>,
}

#[derive(Clone, Debug)]
pub struct InterestGroup {
    pub key: String,
    /// (profession index, per mille).
    pub professions: Vec<(u8, u16)>,
    pub community: Option<String>,
    /// (effect index, weight).
    pub interests: Vec<(u8, i8)>,
}

/// What a condition measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Measure {
    Indicator(u8),
    Effect(u8),
}

impl Measure {
    /// Bounds of the measured value.
    pub fn range(self) -> (i32, i32) {
        match self {
            Measure::Indicator(i) => (0, INDICATORS[usize::from(i)].max),
            Measure::Effect(e) => effect_kind(usize::from(e)).range(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalRef {
    Law(u16),
    Independence,
    Revision,
}

#[derive(Clone, Debug)]
pub struct MovementKind {
    pub key: String,
    pub goal: GoalRef,
    pub among: Among,
    pub professions: Vec<(u8, u16)>,
    pub cause: Vec<(Measure, Bound)>,
    pub growth: u16,
    pub persistence: u16,
}

impl MovementKind {
    /// How far the cause holds, per mille: 0 when any condition fails, else the smallest margin
    /// among the conditions (the weakest one sets the pace, and the interface names it).
    /// `value` gives the current value of each measure.
    pub fn pressure(&self, value: impl Fn(Measure) -> i32) -> i32 {
        self.cause
            .iter()
            .map(|&(m, b)| margin(b, value(m), m.range()))
            .min()
            .unwrap_or(0)
    }
}

/// How far `value` is past the bound, per mille of the room between the bound and the end of
/// the range; 0 when the condition fails. A condition just met gives 1.
pub fn margin(bound: Bound, value: i32, (lo, hi): (i32, i32)) -> i32 {
    let (past, room) = match bound {
        Bound::AtLeast(t) => (value - t, hi - t),
        Bound::AtMost(t) => (t - value, t - lo),
    };
    if past < 0 {
        0
    } else if room <= 0 {
        1000
    } else {
        (i64::from(past) * 1000 / i64::from(room)).clamp(1, 1000) as i32
    }
}

/// Combines the values several laws give to one effect ([`Combine`]).
pub fn combine(kind: &EffectKind, values: impl IntoIterator<Item = i32>) -> i32 {
    let (lo, hi) = kind.range();
    let v = match kind.combine {
        Combine::Multiply => values
            .into_iter()
            .fold(1000i64, |acc, v| acc * i64::from(v) / 1000),
        Combine::Add => values.into_iter().map(i64::from).sum(),
        Combine::Cap => values.into_iter().map(i64::from).fold(1000, i64::min),
    };
    v.clamp(i64::from(lo), i64::from(hi)) as i32
}

/// Whom a law's effects fall on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// The metropole's people outside any targeted community.
    Metropole,
    /// A community targeted by the country's targeted laws, in the metropole.
    Targeted,
    /// The colonial regions.
    Colonies,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PoliticsError {
    Duplicate(String),
    UnknownCategory {
        group: String,
        category: String,
    },
    UnknownGroup {
        law: String,
        group: String,
    },
    /// A law group with fewer than two laws: there is nothing to choose.
    NoChoice(String),
    UnknownEffect {
        owner: String,
        effect: String,
    },
    RepeatedEffect {
        owner: String,
        effect: String,
    },
    EffectOutOfRange {
        law: String,
        effect: String,
        value: i32,
    },
    /// An effect of the colonies system in a law outside the colonial groups.
    ColonialEffect {
        law: String,
        effect: String,
    },
    UnknownProfession {
        owner: String,
        profession: String,
    },
    /// Shares of one profession among the general interest groups not adding up to 1000.
    ProfessionShares {
        profession: String,
        total: u32,
    },
    BadShare {
        owner: String,
        profession: String,
    },
    BadWeight {
        group: String,
        effect: String,
        weight: i8,
    },
    NoInterests(String),
    BadCommunity {
        owner: String,
        community: String,
    },
    UnknownLaw {
        owner: String,
        law: String,
    },
    UnknownMeasure {
        movement: String,
        measure: String,
    },
    BoundOutOfRange {
        movement: String,
        measure: String,
    },
    NoCause(String),
    BadPace(String),
    /// A movement for a colonial law must draw from the colonies, and independence too.
    WrongAmong(String),
    // Scenario.
    UnknownTemplate {
        tag: String,
        template: String,
    },
    UnknownTag(String),
    MissingCountry(String),
    LawTwice {
        owner: String,
        group: String,
    },
    MissingGroup {
        tag: String,
        group: String,
    },
    /// A colonial law for a country without colonies.
    NoColonies {
        tag: String,
        law: String,
    },
    /// Targets for a group whose law in force is not targeted.
    UntargetedLaw {
        tag: String,
        group: String,
    },
    BadDate {
        tag: String,
        date: String,
    },
    NoSource(String),
}

impl std::fmt::Display for PoliticsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PoliticsError {}

#[derive(Clone, Debug, Default)]
pub struct Politics {
    pub groups: Vec<LawGroup>,
    pub laws: Vec<Law>,
    pub interest_groups: Vec<InterestGroup>,
    pub movements: Vec<MovementKind>,
}

fn profession(owner: &str, key: &str) -> Result<u8, PoliticsError> {
    PROFESSIONS
        .iter()
        .position(|p| *p == key)
        .map(|p| p as u8)
        .ok_or_else(|| PoliticsError::UnknownProfession {
            owner: owner.to_owned(),
            profession: key.to_owned(),
        })
}

fn shares(owner: &str, list: &[(String, u16)]) -> Result<Vec<(u8, u16)>, PoliticsError> {
    let mut out: Vec<(u8, u16)> = Vec::with_capacity(list.len());
    for (p, share) in list {
        let pi = profession(owner, p)?;
        if *share == 0 || *share > 1000 || out.iter().any(|(q, _)| *q == pi) {
            return Err(PoliticsError::BadShare {
                owner: owner.to_owned(),
                profession: p.clone(),
            });
        }
        out.push((pi, *share));
    }
    Ok(out)
}

/// A community id: lower-case ASCII letters, digits and `_`, starting with a letter.
pub fn is_community_id(id: &str) -> bool {
    id.starts_with(|c: char| c.is_ascii_lowercase())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// `YYYY-MM-DD` with a plausible month and day.
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: std::ops::Range<usize>| s[r].parse::<u32>().ok();
    matches!(
        (num(0..4), num(5..7), num(8..10)),
        (Some(_), Some(1..=12), Some(1..=31))
    )
}

impl Politics {
    pub fn build(
        laws: &LawsFile,
        interest_groups: &InterestGroupsFile,
        movements: &MovementsFile,
    ) -> Result<Self, PoliticsError> {
        let mut out = Politics::default();
        for (gi, g) in laws.groups.iter().enumerate() {
            if laws.groups.iter().position(|x| x.key == g.key) != Some(gi) {
                return Err(PoliticsError::Duplicate(g.key.clone()));
            }
            let category = CATEGORIES
                .iter()
                .position(|c| *c == g.category)
                .ok_or_else(|| PoliticsError::UnknownCategory {
                    group: g.key.clone(),
                    category: g.category.clone(),
                })? as u8;
            out.groups.push(LawGroup {
                key: g.key.clone(),
                category,
                colonial: g.colonial,
                laws: Vec::new(),
            });
        }
        for (li, l) in laws.laws.iter().enumerate() {
            if laws.laws.iter().position(|x| x.key == l.key) != Some(li) {
                return Err(PoliticsError::Duplicate(l.key.clone()));
            }
            let group = out
                .groups
                .iter()
                .position(|g| g.key == l.group)
                .ok_or_else(|| PoliticsError::UnknownGroup {
                    law: l.key.clone(),
                    group: l.group.clone(),
                })?;
            let mut effects: Vec<(u8, i32)> = Vec::with_capacity(l.effects.len());
            for (e, v) in &l.effects {
                let ei = effect_index(e).ok_or_else(|| PoliticsError::UnknownEffect {
                    owner: l.key.clone(),
                    effect: e.clone(),
                })?;
                if effects.iter().any(|(x, _)| usize::from(*x) == ei) {
                    return Err(PoliticsError::RepeatedEffect {
                        owner: l.key.clone(),
                        effect: e.clone(),
                    });
                }
                let (lo, hi) = effect_kind(ei).range();
                if !(lo..=hi).contains(v) {
                    return Err(PoliticsError::EffectOutOfRange {
                        law: l.key.clone(),
                        effect: e.clone(),
                        value: *v,
                    });
                }
                if EFFECTS[ei].system == System::Colonies && !out.groups[group].colonial {
                    return Err(PoliticsError::ColonialEffect {
                        law: l.key.clone(),
                        effect: e.clone(),
                    });
                }
                effects.push((ei as u8, *v));
            }
            out.groups[group].laws.push(li as u16);
            out.laws.push(Law {
                key: l.key.clone(),
                group: group as u16,
                targeted: l.targeted,
                effects,
            });
        }
        if let Some(g) = out.groups.iter().find(|g| g.laws.len() < 2) {
            return Err(PoliticsError::NoChoice(g.key.clone()));
        }

        let mut totals = [0u32; PROFESSIONS.len()];
        for (gi, g) in interest_groups.groups.iter().enumerate() {
            if interest_groups.groups.iter().position(|x| x.key == g.key) != Some(gi) {
                return Err(PoliticsError::Duplicate(g.key.clone()));
            }
            let professions = shares(&g.key, &g.professions)?;
            if let Some(c) = &g.community {
                if !is_community_id(c) {
                    return Err(PoliticsError::BadCommunity {
                        owner: g.key.clone(),
                        community: c.clone(),
                    });
                }
            } else {
                for (p, s) in &professions {
                    totals[usize::from(*p)] += u32::from(*s);
                }
            }
            if g.interests.is_empty() {
                return Err(PoliticsError::NoInterests(g.key.clone()));
            }
            let mut interests: Vec<(u8, i8)> = Vec::with_capacity(g.interests.len());
            for (e, w) in &g.interests {
                let ei = effect_index(e).ok_or_else(|| PoliticsError::UnknownEffect {
                    owner: g.key.clone(),
                    effect: e.clone(),
                })?;
                if interests.iter().any(|(x, _)| usize::from(*x) == ei) {
                    return Err(PoliticsError::RepeatedEffect {
                        owner: g.key.clone(),
                        effect: e.clone(),
                    });
                }
                if *w == 0 || !(-3..=3).contains(w) {
                    return Err(PoliticsError::BadWeight {
                        group: g.key.clone(),
                        effect: e.clone(),
                        weight: *w,
                    });
                }
                interests.push((ei as u8, *w));
            }
            out.interest_groups.push(InterestGroup {
                key: g.key.clone(),
                professions,
                community: g.community.clone(),
                interests,
            });
        }
        for (p, total) in totals.iter().enumerate() {
            if *total != 1000 {
                return Err(PoliticsError::ProfessionShares {
                    profession: PROFESSIONS[p].to_owned(),
                    total: *total,
                });
            }
        }

        for (mi, m) in movements.movements.iter().enumerate() {
            if movements.movements.iter().position(|x| x.key == m.key) != Some(mi) {
                return Err(PoliticsError::Duplicate(m.key.clone()));
            }
            let goal = match &m.goal {
                Goal::Independence => {
                    if m.among != Among::Colonies {
                        return Err(PoliticsError::WrongAmong(m.key.clone()));
                    }
                    GoalRef::Independence
                }
                Goal::Revision => {
                    if m.among == Among::Colonies {
                        return Err(PoliticsError::WrongAmong(m.key.clone()));
                    }
                    GoalRef::Revision
                }
                Goal::Law(l) => {
                    let li = out.law(l).ok_or_else(|| PoliticsError::UnknownLaw {
                        owner: m.key.clone(),
                        law: l.clone(),
                    })?;
                    let colonial = out.groups[usize::from(out.laws[li].group)].colonial;
                    if colonial != (m.among == Among::Colonies) {
                        return Err(PoliticsError::WrongAmong(m.key.clone()));
                    }
                    GoalRef::Law(li as u16)
                }
            };
            let professions = shares(&m.key, &m.professions)?;
            if m.cause.is_empty() {
                return Err(PoliticsError::NoCause(m.key.clone()));
            }
            let mut cause = Vec::with_capacity(m.cause.len());
            for (key, bound) in &m.cause {
                let measure = indicator_index(key)
                    .map(|i| Measure::Indicator(i as u8))
                    .or_else(|| effect_index(key).map(|e| Measure::Effect(e as u8)))
                    .ok_or_else(|| PoliticsError::UnknownMeasure {
                        movement: m.key.clone(),
                        measure: key.clone(),
                    })?;
                let (lo, hi) = measure.range();
                let (Bound::AtLeast(t) | Bound::AtMost(t)) = *bound;
                if !(lo..=hi).contains(&t) {
                    return Err(PoliticsError::BoundOutOfRange {
                        movement: m.key.clone(),
                        measure: key.clone(),
                    });
                }
                cause.push((measure, *bound));
            }
            if m.growth == 0 || m.growth > 1000 || m.persistence == 0 {
                return Err(PoliticsError::BadPace(m.key.clone()));
            }
            out.movements.push(MovementKind {
                key: m.key.clone(),
                goal,
                among: m.among,
                professions,
                cause,
                growth: m.growth,
                persistence: m.persistence,
            });
        }
        Ok(out)
    }

    pub fn law(&self, key: &str) -> Option<usize> {
        self.laws.iter().position(|l| l.key == key)
    }

    pub fn group(&self, key: &str) -> Option<usize> {
        self.groups.iter().position(|g| g.key == key)
    }

    pub fn interest_group(&self, key: &str) -> Option<usize> {
        self.interest_groups.iter().position(|g| g.key == key)
    }

    pub fn movement(&self, key: &str) -> Option<usize> {
        self.movements.iter().position(|m| m.key == key)
    }

    /// Value of `law`'s effect `effect`, neutral when it declares none.
    pub fn law_effect(&self, law: usize, effect: usize) -> i32 {
        self.laws[law]
            .effects
            .iter()
            .find(|(e, _)| usize::from(*e) == effect)
            .map_or(effect_kind(effect).neutral(), |(_, v)| *v)
    }

    /// Whether `law` acts on the people of `scope`.
    pub fn applies(&self, law: usize, scope: Scope) -> bool {
        let l = &self.laws[law];
        let group = &self.groups[usize::from(l.group)];
        match scope {
            Scope::Metropole => !group.colonial && !l.targeted,
            Scope::Targeted => !group.colonial,
            // Colonies live under the colonial laws and the metropole's economic policy; the
            // metropole's labour, welfare, rights and media laws stop at its borders.
            Scope::Colonies => {
                group.colonial || CATEGORIES[usize::from(group.category)] == "economy"
            }
        }
    }

    /// The value of every effect for the people of `scope` under `laws` (one per group, `None`
    /// for a group the country does not hold), indexed like [`EFFECTS`].
    pub fn effects(&self, laws: &[Option<u16>], scope: Scope) -> Vec<i32> {
        let active: Vec<usize> = laws
            .iter()
            .flatten()
            .map(|&l| usize::from(l))
            .filter(|&l| self.applies(l, scope))
            .collect();
        EFFECTS
            .iter()
            .enumerate()
            .map(|(e, kind)| {
                combine(
                    kind,
                    active.iter().filter_map(|&l| {
                        self.laws[l]
                            .effects
                            .iter()
                            .find(|(x, _)| usize::from(*x) == e)
                            .map(|(_, v)| *v)
                    }),
                )
            })
            .collect()
    }

    /// Like [`Self::effects`], but `None` for an effect no law acting on `scope` declares: the
    /// simulation then keeps the rule it runs without politics.
    pub fn declared(&self, laws: &[Option<u16>], scope: Scope) -> Vec<Option<i32>> {
        let active: Vec<usize> = laws
            .iter()
            .flatten()
            .map(|&l| usize::from(l))
            .filter(|&l| self.applies(l, scope))
            .collect();
        EFFECTS
            .iter()
            .enumerate()
            .map(|(e, kind)| {
                let mut values = active.iter().filter_map(|&l| {
                    self.laws[l]
                        .effects
                        .iter()
                        .find(|(x, _)| usize::from(*x) == e)
                        .map(|(_, v)| *v)
                });
                let first = values.next()?;
                Some(combine(kind, std::iter::once(first).chain(values)))
            })
            .collect()
    }

    /// Each effect's share of `group`'s stance on `law`: weight × distance from neutral.
    pub fn terms(&self, group: usize, law: usize) -> Vec<(u8, i64)> {
        self.interest_groups[group]
            .interests
            .iter()
            .filter_map(|&(e, w)| {
                let d = effect_kind(usize::from(e)).deviation(self.law_effect(law, usize::from(e)));
                (d != 0).then_some((e, i64::from(w) * i64::from(d)))
            })
            .collect()
    }

    /// How much `group` gains from `law`, against no law at all: the sum of [`Self::terms`].
    /// Compare two laws of a group to get the stance on changing from one to the other.
    pub fn score(&self, group: usize, law: usize) -> i64 {
        self.terms(group, law).iter().map(|(_, v)| v).sum()
    }

    /// The laws in force on the start date, per country (in the order of `countries`) and per
    /// law group; colonial groups are `None` for countries without colonies.
    pub fn resolve(
        &self,
        file: &ScenarioLawsFile,
        countries: &[CountryDef],
    ) -> Result<LawsInForce, PoliticsError> {
        let lookup = |owner: &str, key: &str| {
            self.law(key).ok_or_else(|| PoliticsError::UnknownLaw {
                owner: owner.to_owned(),
                law: key.to_owned(),
            })
        };
        let mut templates: Vec<(&str, Vec<Option<u16>>)> = Vec::new();
        for t in &file.templates {
            if templates.iter().any(|(k, _)| *k == t.key) {
                return Err(PoliticsError::Duplicate(t.key.clone()));
            }
            let mut set = vec![None; self.groups.len()];
            for key in &t.laws {
                let l = lookup(&t.key, key)?;
                let g = usize::from(self.laws[l].group);
                if set[g].replace(l as u16).is_some() {
                    return Err(PoliticsError::LawTwice {
                        owner: t.key.clone(),
                        group: self.groups[g].key.clone(),
                    });
                }
            }
            templates.push((&t.key, set));
        }
        for (ci, c) in file.countries.iter().enumerate() {
            if file.countries.iter().position(|x| x.tag == c.tag) != Some(ci) {
                return Err(PoliticsError::Duplicate(c.tag.clone()));
            }
            if !countries.iter().any(|d| d.tag == c.tag) {
                return Err(PoliticsError::UnknownTag(c.tag.clone()));
            }
        }
        let mut laws = Vec::with_capacity(countries.len());
        let mut targets = Vec::with_capacity(countries.len());
        for d in countries {
            let c = file
                .countries
                .iter()
                .find(|c| c.tag == d.tag)
                .ok_or_else(|| PoliticsError::MissingCountry(d.tag.clone()))?;
            if c.source.trim().is_empty() {
                return Err(PoliticsError::NoSource(c.tag.clone()));
            }
            let mut set = match &c.template {
                None => vec![None; self.groups.len()],
                Some(t) => templates
                    .iter()
                    .find(|(k, _)| k == t)
                    .map(|(_, s)| s.clone())
                    .ok_or_else(|| PoliticsError::UnknownTemplate {
                        tag: c.tag.clone(),
                        template: t.clone(),
                    })?,
            };
            let mut seen = vec![false; self.groups.len()];
            for key in &c.laws {
                let l = lookup(&c.tag, key)?;
                let g = usize::from(self.laws[l].group);
                if std::mem::replace(&mut seen[g], true) {
                    return Err(PoliticsError::LawTwice {
                        owner: c.tag.clone(),
                        group: self.groups[g].key.clone(),
                    });
                }
                set[g] = Some(l as u16);
            }
            // The form of government comes from countries.ron unless the laws name one.
            if let Some(g) = self.group(GOVERNMENT_GROUP)
                && set[g].is_none()
            {
                set[g] = Some(lookup(&c.tag, &d.government)? as u16);
            }
            let has_colonies = d.colonies.is_some();
            for (g, group) in self.groups.iter().enumerate() {
                match (set[g], group.colonial && !has_colonies) {
                    (None, false) => {
                        return Err(PoliticsError::MissingGroup {
                            tag: c.tag.clone(),
                            group: group.key.clone(),
                        });
                    }
                    (Some(l), true) => {
                        return Err(PoliticsError::NoColonies {
                            tag: c.tag.clone(),
                            law: self.laws[usize::from(l)].key.clone(),
                        });
                    }
                    _ => {}
                }
            }
            let mut country_targets = Vec::with_capacity(c.targets.len());
            for (group, ids) in &c.targets {
                let g = self
                    .group(group)
                    .ok_or_else(|| PoliticsError::UnknownGroup {
                        law: c.tag.clone(),
                        group: group.clone(),
                    })?;
                if !set[g].is_some_and(|l| self.laws[usize::from(l)].targeted) {
                    return Err(PoliticsError::UntargetedLaw {
                        tag: c.tag.clone(),
                        group: group.clone(),
                    });
                }
                if let Some(bad) = ids.iter().find(|id| !is_community_id(id)) {
                    return Err(PoliticsError::BadCommunity {
                        owner: c.tag.clone(),
                        community: bad.clone(),
                    });
                }
                country_targets.push((g as u16, ids.clone()));
            }
            laws.push(set);
            targets.push(country_targets);
        }
        for h in &file.history {
            if !countries.iter().any(|d| d.tag == h.tag) {
                return Err(PoliticsError::UnknownTag(h.tag.clone()));
            }
            lookup(&h.tag, &h.law)?;
            if !is_date(&h.date)
                || h.date.as_str() <= "1925-01-01"
                || h.date.as_str() > "2025-12-31"
            {
                return Err(PoliticsError::BadDate {
                    tag: h.tag.clone(),
                    date: h.date.clone(),
                });
            }
            if h.source.trim().is_empty() {
                return Err(PoliticsError::NoSource(format!("{} {}", h.tag, h.date)));
            }
        }
        Ok(LawsInForce {
            laws,
            targets,
            history: file.history.clone(),
        })
    }
}

/// Laws in force, by country (scenario order) and law group.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LawsInForce {
    pub laws: Vec<Vec<Option<u16>>>,
    /// (law group, community ids) per country.
    pub targets: Vec<Vec<(u16, Vec<String>)>>,
    /// Dated changes after the start, as reference: nothing fires them.
    pub history: Vec<LawChange>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margin_is_zero_below_and_grows_past_the_bound() {
        assert_eq!(margin(Bound::AtLeast(500), 400, (0, 1000)), 0);
        assert_eq!(margin(Bound::AtLeast(500), 500, (0, 1000)), 1);
        assert_eq!(margin(Bound::AtLeast(500), 750, (0, 1000)), 500);
        assert_eq!(margin(Bound::AtMost(400), 100, (0, 1000)), 750);
        assert_eq!(margin(Bound::AtMost(400), 401, (0, 1000)), 0);
    }

    #[test]
    fn combine_follows_the_kind() {
        let mul = effect_kind(effect_index("migration_within_state").unwrap());
        assert_eq!(combine(mul, [250, 400]), 100);
        assert_eq!(combine(mul, []), 1000);
        let cap = effect_kind(effect_index("tax_cap_upper").unwrap());
        assert_eq!(combine(cap, [600, 350]), 350);
        let add = effect_kind(effect_index("censorship").unwrap());
        assert_eq!(combine(add, [700, 400]), 1000);
        assert_eq!(combine(add, [-300]), -300);
    }

    #[test]
    fn effect_keys_are_unique() {
        for (i, e) in EFFECTS.iter().enumerate() {
            assert_eq!(effect_index(e.key), Some(i), "{}", e.key);
            assert!(
                indicator_index(e.key).is_none(),
                "{} is also an indicator",
                e.key
            );
        }
    }

    #[test]
    fn dates() {
        assert!(is_date("1932-12-27"));
        assert!(!is_date("1932-13-01"));
        assert!(!is_date("1932-1-01"));
    }
}
