//! Births, deaths, ageing and migration between countries. Rules in
//! `docs/population.pt-BR.md` ("Demografia").
//!
//! Every state carries an age pyramid in five-year groups. Mortality by age follows one world
//! schedule moved by a health level that comes from the state's own living standard, schooling,
//! hospitals and the medicine its country uses; fertility follows one world curve moved by
//! child survival, the schooling of the mothers, work off the farm, comfort and contraception.
//! No rate is set by country or by date: the differences come from each state's economy.
//!
//! Migration between countries runs on the same monthly tick: people leave where income is low
//! for where it is several times higher, more toward countries of their language family and
//! within their empire, less far away, and only as far as the laws of both sides let them.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::econ::public::{CARE_PER_PERSON, PublicKinds, coverage};
use crate::econ::week::ECON;
use crate::econ::{Catalogue, Economy, Region, milli, prof, workers};
use crate::game::GameWorld;
use crate::politics::{Politics, Scope};
use crate::polity::{POLITY, effects_in_force};
use crate::research_system::RESEARCH;

pub mod migration;
pub use migration::{Diaspora, Home, Push, Wave, WaveEnd, WaveRecord};

pub const DEMOGRAPHY: ResourceId = ResourceId(13);

/// A country's metropole or its colonies: the unit of migration between countries and of the
/// history series. The one place that knows how a territory maps to a market and back.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct TerritoryId(pub u16);

impl TerritoryId {
    pub const fn new(market: u16, colony: bool) -> Self {
        Self(market * 2 + colony as u16)
    }

    /// The territory of a region.
    pub fn of(r: &Region) -> Self {
        Self::new(r.market, r.colony)
    }

    /// From a dense index (`0..count`).
    pub fn from_index(i: usize) -> Self {
        Self(i as u16)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    pub const fn market(self) -> usize {
        (self.0 / 2) as usize
    }

    pub const fn colony(self) -> bool {
        self.0 % 2 == 1
    }

    /// Territories of `markets` markets.
    pub const fn count(markets: usize) -> usize {
        markets * 2
    }
}

impl HashState for TerritoryId {
    fn hash_state(&self, h: &mut StateHasher) {
        self.0.hash_state(h);
    }
}

/// Five-year age groups: 0–4, 5–9, …, 75–79 and 80 and over.
pub const AGE_GROUPS: usize = 17;
/// First and last group of the mothers (15–19 to 45–49).
const FERTILE: std::ops::RangeInclusive<usize> = 3..=9;
/// Share of a woman's births in each fertile group, per mille (a standard schedule).
const FERTILITY_SHAPE: [i64; 7] = [80, 230, 250, 200, 140, 80, 20];
/// Groups whose people leave and arrive as migrants (20–34).
const MIGRANT_AGES: std::ops::RangeInclusive<usize> = 4..=6;

/// Deaths per 100,000 a year by age group, at health level 0, ¼, ½, ¾ and 1: a world
/// schedule (child deaths, an adult floor and a Gompertz rise with age) whose life expectancy
/// at birth is about 28, 39, 52, 68 and 82 years. Between knots the rates are linear.
const MORTALITY: [[i64; AGE_GROUPS]; 5] = [
    [
        13087, 1431, 627, 1167, 1184, 1211, 1254, 1323, 1434, 1612, 1899, 2361, 3103, 4297, 6216,
        9302, 19442,
    ],
    [
        8529, 938, 414, 820, 834, 857, 894, 954, 1049, 1202, 1449, 1845, 2482, 3507, 5155, 7805,
        16512,
    ],
    [
        4681, 523, 234, 506, 518, 537, 568, 618, 697, 825, 1031, 1363, 1895, 2751, 4128, 6342,
        13616,
    ],
    [
        1709, 201, 95, 237, 247, 262, 287, 327, 391, 493, 659, 925, 1352, 2040, 3146, 4923, 10765,
    ],
    [
        83, 25, 17, 52, 59, 71, 89, 119, 168, 245, 370, 571, 893, 1412, 2247, 3589, 7997,
    ],
];
/// Life expectancy at birth of each knot of [`MORTALITY`], in tenths of a year.
const LIFE_AT_KNOT: [i64; 5] = [277, 389, 525, 680, 820];

/// Health level from living standard alone: comfort (tier 3 needs met), the mothers'
/// schooling, and hunger.
const H_COMFORT: Fixed = milli(380);
const H_SCHOOLING: Fixed = milli(350);
const H_OFFSET: Fixed = milli(50);
const H_HUNGER: Fixed = milli(250);
/// Health level modern medicine reaches: a base, comfort and hospital care.
const H_MODERN: Fixed = milli(700);
const H_MODERN_COMFORT: Fixed = milli(250);
const H_MODERN_CARE: Fixed = milli(130);
/// Share of the medicine a country uses that reaches a state with no hospitals and no schools
/// (mass campaigns: vaccines, DDT, penicillin for yaws, at cents a head); the rest follows care
/// or schooling. Preston (1975): income explains 10 to 25 % of the gain in life expectancy
/// between the 1930s and the 1960s; India went from 32 to 51 years between 1941 and 1975 with
/// a sixth of its women literate and 0.5 hospital beds per thousand.
const MEDICINE_REACH: Fixed = milli(550);
/// Strength of the medicine the richest countries already used in 1925 (vaccination, sanitation,
/// antisepsis): it is part of the living standard in [`health`], so only what comes after it
/// counts as modern medicine.
const MEDICINE_1925: Fixed = milli(200);
/// Strength at which modern medicine is complete: antibiotics, vaccines and insecticides are
/// half of the tree's medicine and did most of the work.
const MEDICINE_FULL: Fixed = milli(800);

/// Children per woman before and after the transition; without modern contraception the
/// transition stops this much higher (withdrawal, late marriage).
const TFR_HIGH: Fixed = milli(6500);
const TFR_LOW: Fixed = milli(1750);
const TFR_WITHOUT_PILL: Fixed = milli(500);
/// Land hunger, the preventive check: where the cropland a farmer has (open and still to clear,
/// farm levels per thousand farmers) is below `LAND_SCARCE` and below the state's own of 1925,
/// fertility falls in proportion to what was lost, by `LAND_BRAKE` at most. Brazil, Argentina
/// and Africa have land to clear and do not brake; rural Asia does. An absolute measure alone
/// would brake the irrigated Nile in 1925, where fertility was 6.5.
const LAND_BRAKE: Fixed = milli(500);
const LAND_SCARCE: Fixed = milli(20_000);
/// Weights of the transition score and where the transition starts and ends.
const F_SURVIVAL: Fixed = milli(300);
const F_SCHOOLING: Fixed = milli(570);
const F_OFF_FARM: Fixed = milli(30);
const F_COMFORT: Fixed = milli(100);
const F_CONTRACEPTION: Fixed = milli(170);
const F_START: Fixed = milli(450);
const F_END: Fixed = milli(900);
/// Child deaths before 5 (per mille) at which survival counts as 0 and as 1 in the score.
const Q5_HIGH: i64 = 500;
const Q5_LOW: i64 = 20;

/// Share of the contraception a country uses that reaches mothers with neither schooling nor a
/// health service; the rest arrives with whichever of the two the state has more of: family
/// planning came through clinics and health posts as much as through schools (barefoot doctors
/// in China, Indonesia, Bangladesh).
const CONTRACEPTION_REACH: Fixed = milli(50);
/// Months for the mothers' schooling to close the gap with today's literacy: girls at school
/// now are mothers in 15 to 20 years.
const SCHOOLING_MONTHS: i64 = 240;
/// Girls go to school after boys where schooling is scarce: women's literacy is
/// `literacy × (1 − GIRLS_GAP × (1 − literacy))`. Census literacy, total and women (World Bank
/// WDI, SE.ADT.LITR.ZS and .FE.ZS): India 1991 48 and 34 %, Egypt 1986 44 and 31 %, Algeria 1987
/// 50 and 36 %, Nigeria 1991 56 and 44 %, Turkey 1990 79 and 69 % (a gap of 0.5 to 0.65); Mexico
/// 1990 88 and 85 % and Brazil 1980 75 and 73 % have less.
const GIRLS_GAP: Fixed = milli(500);

/// Literacy of the women of a region whose literacy is `l`.
pub fn women_literacy(l: Fixed) -> Fixed {
    l * (Fixed::ONE - GIRLS_GAP * (Fixed::ONE - l))
}

/// Age pyramid and migration links of every state, saved and hashed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemographyState {
    /// People in each age group, by region. Rescaled every month to the region's population
    /// (which other systems move: internal migration). Empty before the first month.
    pub ages: Vec<[i64; AGE_GROUPS]>,
    /// Literacy of the mothers: the women's literacy of the region followed with a lag of a generation.
    pub schooling: Vec<Fixed>,
    /// Land per farmer of each region in its first month, the measure of land hunger.
    #[serde(default)]
    pub land_1925: Vec<Fixed>,
    /// Last month's rates by region.
    pub vital: Vec<Vital>,
    /// Territories (`market * 2 + colony`): centre of population and language family.
    pub territories: Vec<Territory>,
    /// People abroad by origin, destination and culture, sorted by them.
    #[serde(default)]
    pub diaspora: Vec<Diaspora>,
    /// Minorities with a road of their own abroad, sorted by origin and culture.
    #[serde(default)]
    pub homes: Vec<Home>,
    /// Mass migrations under way, and those that ended.
    #[serde(default)]
    pub waves: Vec<Wave>,
    #[serde(default)]
    pub waves_ended: Vec<WaveRecord>,
}

/// One term of the fertility score of a territory ([`DemographyState::fertility_breakdown`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FertilityTerm {
    pub key: &'static str,
    pub value: Fixed,
    pub weight: Fixed,
    pub contribution: Fixed,
}

/// Rates of a region, per thousand people a year, and what produced them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vital {
    pub births: Fixed,
    pub deaths: Fixed,
    /// Immigrants minus emigrants.
    pub migration: Fixed,
    /// Life expectancy at birth of this month's mortality, years.
    pub life: Fixed,
    /// Children per woman at this month's fertility.
    pub tfr: Fixed,
    /// Values of [`FERTILITY_TERMS`] this month.
    #[serde(default)]
    pub fertility_terms: [Fixed; 5],
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Territory {
    /// Latitude and longitude of the population centre, degrees.
    pub lat: Fixed,
    pub lon: Fixed,
    /// Index into [`crate::communities::CULTURE_FAMILIES`] of the largest language family;
    /// `u8::MAX` when unknown.
    pub family: u8,
    /// Index into the scenario's cultures of the largest culture; `u16::MAX` when unknown.
    #[serde(default = "no_culture")]
    pub culture: u16,
}

fn no_culture() -> u16 {
    u16::MAX
}

impl HashState for Vital {
    fn hash_state(&self, h: &mut StateHasher) {
        for v in [
            self.births,
            self.deaths,
            self.migration,
            self.life,
            self.tfr,
        ] {
            v.hash_state(h);
        }
        self.fertility_terms.as_slice().hash_state(h);
    }
}

impl HashState for Territory {
    fn hash_state(&self, h: &mut StateHasher) {
        self.lat.hash_state(h);
        self.lon.hash_state(h);
        self.family.hash_state(h);
        self.culture.hash_state(h);
    }
}

impl HashState for DemographyState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.ages.len());
        for a in &self.ages {
            a.as_slice().hash_state(h);
        }
        self.schooling.hash_state(h);
        self.land_1925.hash_state(h);
        self.vital.hash_state(h);
        self.territories.hash_state(h);
        self.diaspora.hash_state(h);
        self.homes.hash_state(h);
        self.waves.hash_state(h);
        self.waves_ended.hash_state(h);
    }
}

impl DemographyState {
    /// Share of a country's people (metropole and colonies) who came from another country or
    /// descend from those who did, 0–1: the diaspora stock whose destination is the country.
    /// Moves within a country (metropole and colonies) do not count. The `immigrant_share` the
    /// politics measures.
    pub fn immigrant_share(&self, econ: &Economy, country: usize) -> Fixed {
        let people: i64 = econ
            .regions
            .iter()
            .filter(|r| usize::from(r.market) == country)
            .map(|r| r.population())
            .sum();
        if people <= 0 {
            return Fixed::ZERO;
        }
        let abroad: i64 = self
            .immigrants_by_origin(country)
            .iter()
            .map(|(_, n)| *n)
            .sum();
        Fixed::from_ratio(abroad.min(people), people)
    }

    /// People of foreign origin in a country by origin country (index of the market), largest
    /// first.
    pub fn immigrants_by_origin(&self, country: usize) -> Vec<(usize, i64)> {
        let mut by = std::collections::BTreeMap::new();
        for d in &self.diaspora {
            let (o, t) = (d.origin.market(), d.dest.market());
            if t == country && o != country && d.people > 0 {
                *by.entry(o).or_insert(0i64) += d.people;
            }
        }
        let mut out: Vec<(usize, i64)> = by.into_iter().collect();
        out.sort_by_key(|&(o, n)| (-n, o));
        out
    }

    /// Why women have the children they have in a territory (`market`, `colony`): every term
    /// of the fertility score with its value (0–1, weighted by population), its weight and its
    /// contribution (value × weight); the transition starts at a score of `F_START` and ends at
    /// `F_END`. Empty before the first month.
    pub fn fertility_breakdown(
        &self,
        econ: &Economy,
        market: u16,
        colony: bool,
    ) -> Vec<FertilityTerm> {
        let mut sum = [0i128; 5];
        let mut people = 0i128;
        for (r, v) in econ.regions.iter().zip(&self.vital) {
            if r.market != market || r.colony != colony {
                continue;
            }
            let n = i128::from(r.population());
            people += n;
            for (s, x) in sum.iter_mut().zip(v.fertility_terms) {
                *s += i128::from(x.raw()) * n;
            }
        }
        if people == 0 {
            return Vec::new();
        }
        FERTILITY_TERMS
            .iter()
            .zip(sum)
            .map(|(&(key, weight), s)| {
                let value = Fixed::from_raw((s / people) as i64);
                FertilityTerm {
                    key,
                    value,
                    weight,
                    contribution: value * weight,
                }
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.territories.is_empty() && self.ages.is_empty()
    }

    /// Life expectancy and children per woman of a territory (`market`, `colony`), weighted by
    /// population; `None` before the first month.
    pub fn territory_rates(&self, econ: &Economy, market: u16, colony: bool) -> Option<Vital> {
        let mut sum = [0i128; 5];
        let mut people = 0i128;
        for (r, v) in econ.regions.iter().zip(&self.vital) {
            if r.market != market || r.colony != colony {
                continue;
            }
            let n = i128::from(r.population());
            people += n;
            for (s, x) in sum
                .iter_mut()
                .zip([v.births, v.deaths, v.migration, v.life, v.tfr])
            {
                *s += i128::from(x.raw()) * n;
            }
        }
        (people > 0).then(|| {
            let f = |s: i128| Fixed::from_raw((s / people) as i64);
            Vital {
                births: f(sum[0]),
                deaths: f(sum[1]),
                migration: f(sum[2]),
                life: f(sum[3]),
                tfr: f(sum[4]),
                // The terms by territory: `fertility_breakdown`.
                fertility_terms: [Fixed::ZERO; 5],
            }
        })
    }
}

/// Static facts of the territories, from the map and the communities, at the start.
pub fn territories(
    econ: &Economy,
    map: &crate::scenario::MapFile,
    communities: Option<&crate::communities::Communities>,
) -> Vec<Territory> {
    let n = TerritoryId::count(econ.markets.len());
    // (people, Σ lat × people, Σ lon × people) and people by language family.
    let mut acc = vec![(0f64, 0f64, 0f64); n];
    let families = crate::communities::CULTURE_FAMILIES.len();
    let mut by_family = vec![vec![0f64; families]; n];
    let cultures = communities.map_or(0, |c| c.cultures.len());
    let mut by_culture = vec![vec![0f64; cultures]; n];
    let meta = &map.meta;
    let miller = |lat: f64| {
        1.25 * (std::f64::consts::FRAC_PI_4 + 0.4 * lat.to_radians())
            .tan()
            .ln()
    };
    let (top, bottom) = (miller(meta.lat_top), miller(meta.lat_bottom));
    for r in &econ.regions {
        let t = TerritoryId::of(r).index();
        let Some(def) = map.states.iter().find(|d| d.id == r.state) else {
            continue;
        };
        let people = r.population() as f64;
        let lon = f64::from(def.center.0) / f64::from(meta.width.max(1)) * 360.0 - 180.0;
        let y = top - f64::from(def.center.1) / f64::from(meta.height.max(1)) * (top - bottom);
        let lat = ((y / 1.25).exp().atan() - std::f64::consts::FRAC_PI_4) / 0.4;
        let a = &mut acc[t];
        a.0 += people;
        a.1 += lat.to_degrees() * people;
        a.2 += lon * people;
        if let Some(c) = communities
            && let Some(mix) = c.states.iter().find(|m| m.state == def.key)
        {
            for &(id, share) in &mix.shares {
                let culture = c.communities[id as usize].0 as usize;
                let fam = &c.cultures[culture].family;
                if let Some(f) = crate::communities::CULTURE_FAMILIES
                    .iter()
                    .position(|k| k == fam)
                {
                    by_family[t][f] += people * f64::from(share);
                }
                by_culture[t][culture] += people * f64::from(share);
            }
        }
    }
    acc.iter()
        .zip(&by_family)
        .zip(&by_culture)
        .map(|((&(p, lat, lon), fam), cul)| {
            let (lat, lon) = if p > 0.0 {
                (lat / p, lon / p)
            } else {
                (0.0, 0.0)
            };
            let family = fam
                .iter()
                .enumerate()
                .filter(|(_, v)| **v > 0.0)
                .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
                .map_or(u8::MAX, |(i, _)| i as u8);
            let culture = cul
                .iter()
                .enumerate()
                .filter(|(_, v)| **v > 0.0)
                .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
                .map_or(u16::MAX, |(i, _)| i as u16);
            Territory {
                lat: Fixed::from_ratio((lat * 10.0).round() as i64, 10),
                lon: Fixed::from_ratio((lon * 10.0).round() as i64, 10),
                family,
                culture,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Rules

/// Health level 0–1 of a region.
pub struct HealthInputs {
    pub comfort: Fixed,
    pub food: Fixed,
    pub schooling: Fixed,
    pub care: Fixed,
    /// Strength of the medicine the country uses, 0–1.
    pub medicine: Fixed,
}

pub fn health(i: &HealthInputs) -> Fixed {
    let before = (i.comfort * H_COMFORT + i.schooling * H_SCHOOLING
        - H_OFFSET
        - (Fixed::ONE - i.food.min(Fixed::ONE)) * H_HUNGER)
        .clamp(Fixed::ZERO, Fixed::ONE);
    let modern = (H_MODERN + i.comfort * H_MODERN_COMFORT + i.care * H_MODERN_CARE).min(Fixed::ONE);
    let reach = MEDICINE_REACH + (Fixed::ONE - MEDICINE_REACH) * i.care.max(i.schooling);
    let modern_medicine = ((i.medicine - MEDICINE_1925) / (MEDICINE_FULL - MEDICINE_1925))
        .clamp(Fixed::ZERO, Fixed::ONE);
    let medicine = modern_medicine * reach;
    (before + (modern - before).max(Fixed::ZERO) * medicine).clamp(Fixed::ZERO, Fixed::ONE)
}

/// Deaths per 100,000 a year in each age group at health level `h`.
pub fn mortality(h: Fixed) -> [i64; AGE_GROUPS] {
    let x = h.clamp(Fixed::ZERO, Fixed::ONE).raw() * 4; // 0..4000
    let k = (x / 1000).min(3) as usize;
    let f = x - k as i64 * 1000;
    std::array::from_fn(|g| (MORTALITY[k][g] * (1000 - f) + MORTALITY[k + 1][g] * f + 500) / 1000)
}

/// Life expectancy at birth at health level `h`, years.
pub fn life_expectancy(h: Fixed) -> Fixed {
    let x = h.clamp(Fixed::ZERO, Fixed::ONE).raw() * 4;
    let k = (x / 1000).min(3) as usize;
    let f = x - k as i64 * 1000;
    Fixed::from_raw((LIFE_AT_KNOT[k] * (1000 - f) + LIFE_AT_KNOT[k + 1] * f) / 10)
}

/// Children per woman.
pub struct FertilityInputs {
    /// Deaths before age 5, per 100,000 a year in the 0–4 group.
    pub child_deaths: i64,
    pub schooling: Fixed,
    pub off_farm: Fixed,
    pub comfort: Fixed,
    /// Contraception the country uses, 0–1, as far as the mothers went to school.
    pub contraception: Fixed,
}

/// What moves fertility, in the order of [`FERTILITY_TERMS`]: each value is 0–1, and the
/// transition score is the sum of value × weight. A new cause (the diffusion of the transition
/// across a language family, a religion's measured effect) enters as one more term here.
pub const FERTILITY_TERMS: [(&str, Fixed); 5] = [
    ("child_survival", F_SURVIVAL),
    ("schooling", F_SCHOOLING),
    ("off_farm", F_OFF_FARM),
    ("comfort", F_COMFORT),
    ("contraception", F_CONTRACEPTION),
];

/// Values of [`FERTILITY_TERMS`] for these inputs.
pub fn fertility_values(i: &FertilityInputs) -> [Fixed; 5] {
    // Share of children dead before 5, per mille: 1 − (1 − 2.5m)/(1 + 2.5m).
    let m = i.child_deaths; // per 100,000
    let q5 = 1000 - (100_000 - 250 * m / 100).max(0) * 1000 / (100_000 + 250 * m / 100);
    let survival = Fixed::from_ratio((Q5_HIGH - q5).clamp(0, Q5_HIGH - Q5_LOW), Q5_HIGH - Q5_LOW);
    [
        survival,
        i.schooling,
        i.off_farm,
        i.comfort,
        i.contraception,
    ]
}

pub fn fertility(i: &FertilityInputs) -> Fixed {
    let values = fertility_values(i);
    let score = values
        .iter()
        .zip(FERTILITY_TERMS)
        .fold(Fixed::ZERO, |acc, (v, (_, w))| acc + *v * w);
    let t = ((score - F_START) / (F_END - F_START)).clamp(Fixed::ZERO, Fixed::ONE);
    let smooth = t * t * (Fixed::from_int(3) - t * 2);
    let floor = TFR_LOW + TFR_WITHOUT_PILL * (Fixed::ONE - i.contraception.min(Fixed::ONE));
    floor + (TFR_HIGH - floor) * (Fixed::ONE - smooth)
}

/// One month of births, deaths and ageing on a pyramid: returns births and deaths.
fn month(
    ages: &mut [i64; AGE_GROUPS],
    deaths_per_100k: &[i64; AGE_GROUPS],
    tfr: Fixed,
) -> (i64, i64) {
    let tfr = tfr.raw(); // thousandths of a child
    let mut births: i128 = 0;
    for (g, shape) in FERTILE.zip(FERTILITY_SHAPE) {
        // Women are half the group; a group spans five years; the month is a twelfth.
        births += i128::from(ages[g]) * i128::from(tfr) * i128::from(shape);
    }
    let births = (births / (2 * 5 * 12 * 1_000_000)) as i64;
    let mut deaths = 0;
    for (a, &m) in ages.iter_mut().zip(deaths_per_100k) {
        let d = (i128::from(*a) * i128::from(m) / (12 * 100_000)) as i64;
        *a -= d;
        deaths += d;
    }
    // A sixtieth of each group grows into the next every month.
    let mut moving = [0i64; AGE_GROUPS];
    for g in 0..AGE_GROUPS - 1 {
        moving[g] = ages[g] / 60;
    }
    for g in 0..AGE_GROUPS - 1 {
        ages[g] -= moving[g];
        ages[g + 1] += moving[g];
    }
    ages[0] += births;
    (births, deaths)
}

/// Years of the past a pyramid is grown over, and the share of today's development (comfort,
/// schooling, work off the farm) the region had at their start.
const PAST_YEARS: i64 = 60;
const PAST_DEVELOPMENT: Fixed = milli(650);

/// The age structure of the start: a population stable at the rates of `PAST_YEARS` ago, when
/// it had `PAST_DEVELOPMENT` of today's development, then grown through those years as the
/// development rose to today's. `rates(d)` gives mortality and fertility at development share
/// `d` of today's.
pub fn initial_pyramid(
    people: i64,
    rates: impl Fn(Fixed) -> ([i64; AGE_GROUPS], Fixed),
) -> [i64; AGE_GROUPS] {
    let mut ages = [1_000_000i64; AGE_GROUPS];
    let (m, f) = rates(PAST_DEVELOPMENT);
    for _ in 0..150 {
        for _ in 0..12 {
            month(&mut ages, &m, f);
        }
        ages = rescale(&ages, 17_000_000);
    }
    for y in 0..PAST_YEARS {
        let d = PAST_DEVELOPMENT + (Fixed::ONE - PAST_DEVELOPMENT).mul_ratio(y, PAST_YEARS);
        let (m, f) = rates(d);
        for _ in 0..12 {
            month(&mut ages, &m, f);
        }
        ages = rescale(&ages, 17_000_000);
    }
    rescale(&ages, people)
}

/// Scales a pyramid to `people`, rounding so that the groups add up exactly.
pub fn rescale(ages: &[i64; AGE_GROUPS], people: i64) -> [i64; AGE_GROUPS] {
    let total: i64 = ages.iter().sum();
    if total <= 0 {
        let mut out = [0; AGE_GROUPS];
        out[5] = people.max(0);
        return out;
    }
    let mut out: [i64; AGE_GROUPS] = std::array::from_fn(|g| {
        (i128::from(ages[g]) * i128::from(people) / i128::from(total)) as i64
    });
    let missing = people - out.iter().sum::<i64>();
    // The remainder goes to the largest group.
    let big = (0..AGE_GROUPS)
        .max_by_key(|&g| (out[g], std::cmp::Reverse(g)))
        .unwrap_or(0);
    out[big] += missing;
    out
}

// ---------------------------------------------------------------------------------------------
// System

pub struct Demography {
    pub cat: Arc<Catalogue>,
    kinds: PublicKinds,
    /// The law catalogue and the indices of the `immigration` and `emigration` effects.
    laws: Option<(Arc<Politics>, usize, usize)>,
    /// Whether each farm of `resources::FARMS` is cropland (not pasture, forest or fishery).
    crop: Vec<bool>,
}

impl Demography {
    pub fn new(cat: Arc<Catalogue>, politics: Option<Arc<Politics>>) -> Self {
        let kinds = PublicKinds::new(&cat);
        let laws = politics.map(|p| {
            let index = |k| crate::politics::effect_index(k).expect("migration law effect");
            (p, index("immigration"), index("emigration"))
        });
        let crop = crate::resources::FARMS
            .iter()
            .map(|f| !matches!(*f, "livestock_ranch" | "forestry" | "fishery"))
            .collect();
        Self {
            cat,
            kinds,
            laws,
            crop,
        }
    }

    /// Cropland a region has and could still clear, in farm levels: open cropland and the
    /// farming frontier (`Region::frontier`).
    fn cropland(&self, r: &Region) -> i64 {
        let crop = |f: u8| self.crop.get(usize::from(f)).copied().unwrap_or(false);
        r.land
            .iter()
            .filter(|l| crop(l.0))
            .map(|l| l.1)
            .sum::<i64>()
            + r.frontier
                .iter()
                .filter(|l| crop(l.0))
                .map(|l| l.1)
                .sum::<i64>()
    }

    /// What the laws in force in `country` let through, per mille of the open flow:
    /// immigration and emigration. Without the politics data, everything is open.
    fn migration_laws(&self, world: &GameWorld, country: usize) -> (i64, i64) {
        match &self.laws {
            Some((p, imm, emi)) if country < world.politics.laws.len() => {
                let e = effects_in_force(p, &world.politics, country, Scope::Metropole);
                (i64::from(e[*imm]), i64::from(e[*emi]))
            }
            _ => (1000, 1000),
        }
    }
}

/// What a region contributes to its territory's migration, and its own month.
pub(crate) struct RegionMonth {
    ages: [i64; AGE_GROUPS],
    schooling: Fixed,
    births: i64,
    deaths: i64,
    life: Fixed,
    tfr: Fixed,
    fertility_terms: [Fixed; 5],
    land_then: Fixed,
    people: i64,
    income: Fixed,
    literacy_people: Fixed,
    /// Basic needs met (tier 1) times people.
    food_people: Fixed,
    idle_workers: i64,
    workers: i64,
    urban: i64,
    /// Labourers able to work, and those in jobs.
    labourers: i64,
    labourers_employed: i64,
    /// The land's ceiling in farm levels, and the people who work the land.
    land_total: i64,
    farmers: i64,
}

pub struct DemographyChanges {
    regions: Vec<RegionMonth>,
    plan: migration::Plan,
}

fn average(r: &Region, tier: usize) -> Fixed {
    let total = r.population().max(1);
    r.pops
        .iter()
        .map(|p| p.satisfaction[tier] * p.size)
        .sum::<Fixed>()
        / total
}

impl System<GameWorld> for Demography {
    type Changes = DemographyChanges;

    fn name(&self) -> &'static str {
        "demography"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 15 }
    }

    fn access(&self) -> Access {
        Access::new()
            .read(ECON)
            .write(ECON)
            .read(RESEARCH)
            .read(POLITY)
            .read(DEMOGRAPHY)
            .write(DEMOGRAPHY)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> DemographyChanges {
        let econ = &world.econ;
        let state = &world.demography;
        let year = ctx.date.year;
        let first = state.ages.len() != econ.regions.len();
        let regions: Vec<RegionMonth> = econ
            .regions
            .par_iter()
            .enumerate()
            .map(|(ri, r)| {
                let people = r.population();
                let women = women_literacy(r.literacy);
                let schooling = match state.schooling.get(ri) {
                    Some(&s) if !first => s + (women - s).mul_ratio(1, SCHOOLING_MONTHS),
                    _ => women,
                };
                let care = coverage(&self.cat, r, self.kinds.hospital, CARE_PER_PERSON);
                let (medicine, contraception) =
                    match world.research.effects.get(usize::from(r.market)) {
                        Some(e) => (
                            e[0],
                            e[1] * (CONTRACEPTION_REACH
                                + (Fixed::ONE - CONTRACEPTION_REACH) * schooling.max(care)),
                        ),
                        // Without research data, medicine spreads from 1935 to 1970.
                        None => (
                            Fixed::from_ratio(i64::from((year - 1935).clamp(0, 35)), 35),
                            Fixed::ZERO,
                        ),
                    };
                let comfort = average(r, 2);
                let food = average(r, 0);
                let off_farm = r.urban_share();
                // Mortality and fertility at a share `d` of today's development.
                let rates_at = |d: Fixed| {
                    let h = health(&HealthInputs {
                        comfort: comfort * d,
                        food,
                        schooling: schooling * d,
                        care: care * d,
                        medicine: medicine * d,
                    });
                    let rates = mortality(h);
                    let tfr = fertility(&FertilityInputs {
                        child_deaths: rates[0],
                        schooling: schooling * d,
                        off_farm: off_farm * d,
                        comfort: comfort * d,
                        contraception: contraception * d,
                    });
                    (h, rates, tfr)
                };
                let (h, rates, tfr) = rates_at(Fixed::ONE);
                // Land hunger: as the land per farmer falls below the state's own of 1925, sons
                // wait for land to marry and families stop earlier (the preventive check).
                let land_total = self.cropland(r);
                let farmers = r.pops[prof::PEASANT].size + r.pops[prof::FARM_LABORER].size;
                let land_now = if farmers > 0 {
                    Fixed::from_ratio(land_total * 1000, farmers)
                } else {
                    Fixed::ZERO
                };
                let land_then = match state.land_1925.get(ri) {
                    Some(&l) if !first => l,
                    _ => land_now,
                };
                let lost = if land_then > Fixed::ZERO && land_now < LAND_SCARCE {
                    (Fixed::ONE - land_now / land_then).clamp(Fixed::ZERO, Fixed::ONE)
                } else {
                    Fixed::ZERO
                };
                let tfr = (tfr * (Fixed::ONE - lost * LAND_BRAKE)).max(TFR_LOW.min(tfr));
                let fertility_terms = fertility_values(&FertilityInputs {
                    child_deaths: rates[0],
                    schooling,
                    off_farm,
                    comfort,
                    contraception,
                });
                let mut ages = if first {
                    initial_pyramid(people, |d| {
                        let (_, m, f) = rates_at(d);
                        (m, f)
                    })
                } else {
                    rescale(&state.ages[ri], people)
                };
                let (births, deaths) = month(&mut ages, &rates, tfr);
                let p = &r.pops[prof::PEASANT];
                let land_total = self.cropland(r);
                let farmers = r.pops[prof::PEASANT].size + r.pops[prof::FARM_LABORER].size;
                let urban: i64 = r
                    .pops
                    .iter()
                    .enumerate()
                    .filter(|(pi, _)| crate::econ::migration::is_urban(*pi))
                    .map(|(_, p)| p.size)
                    .sum();
                RegionMonth {
                    ages,
                    schooling,
                    births,
                    deaths,
                    life: life_expectancy(h),
                    tfr,
                    fertility_terms,
                    land_then,
                    people,
                    income: r.pops.iter().map(|p| p.income).sum(),
                    literacy_people: r.literacy * people,
                    food_people: food * people,
                    idle_workers: (workers(p.size) - p.employed).max(0),
                    workers: r.pops.iter().map(|p| workers(p.size)).sum(),
                    urban,
                    labourers: workers(r.pops[prof::LABORER].size),
                    labourers_employed: r.pops[prof::LABORER].employed,
                    land_total,
                    farmers,
                }
            })
            .collect();
        let laws: Vec<(i64, i64)> = (0..econ.markets.len())
            .map(|c| self.migration_laws(world, c))
            .collect();
        let now = ctx.date.year * 12 + i32::from(ctx.date.month) - 1;
        let plan = migration::plan(world, &regions, &laws, now);
        DemographyChanges { regions, plan }
    }

    fn apply(&self, _ctx: &TickCtx, mut changes: DemographyChanges, world: &mut GameWorld) {
        let state = &mut world.demography;
        let n = world.econ.regions.len();
        state.ages.resize(n, [0; AGE_GROUPS]);
        state.schooling.resize(n, Fixed::ZERO);
        state.land_1925.resize(n, Fixed::ZERO);
        state.vital.resize(n, Vital::default());
        state.diaspora = std::mem::take(&mut changes.plan.diaspora);
        state.waves = std::mem::take(&mut changes.plan.waves);
        state.waves_ended.append(&mut changes.plan.ended);
        for (ri, (r, m)) in world
            .econ
            .regions
            .iter_mut()
            .zip(changes.regions)
            .enumerate()
        {
            let mut ages = m.ages;
            let moved = changes.plan.moves[ri];
            shift_migrants(&mut ages, moved);
            let target: i64 = ages.iter().sum();
            distribute(r, target - m.people, moved);
            // The pyramid and the pops agree on the total.
            state.ages[ri] = rescale(&ages, r.population());
            state.schooling[ri] = m.schooling;
            state.land_1925[ri] = m.land_then;
            let per_year = |n: i64| {
                if m.people > 0 {
                    Fixed::from_ratio(n * 12_000, m.people)
                } else {
                    Fixed::ZERO
                }
            };
            state.vital[ri] = Vital {
                births: per_year(m.births),
                deaths: per_year(m.deaths),
                migration: per_year(moved),
                life: m.life,
                tfr: m.tfr,
                fertility_terms: m.fertility_terms,
            };
        }
    }
}

/// Adds or takes migrants from the 20–34 groups, in proportion to their size.
fn shift_migrants(ages: &mut [i64; AGE_GROUPS], moved: i64) {
    let pool: i64 = ages[MIGRANT_AGES].iter().sum();
    if moved == 0 {
        return;
    }
    let mut left = moved;
    for g in MIGRANT_AGES {
        let share = if pool > 0 {
            (i128::from(moved) * i128::from(ages[g]) / i128::from(pool)) as i64
        } else {
            moved / 3
        };
        ages[g] = (ages[g] + share).max(0);
        left -= share;
    }
    ages[5] = (ages[5] + left).max(0);
}

/// Applies a region's net change to its pops: births and deaths in proportion to every
/// profession; emigrants leave the land and the labourers, immigrants arrive as labourers.
fn distribute(r: &mut Region, change: i64, moved: i64) {
    let natural = change - moved;
    let total = r.population();
    if total > 0 && natural != 0 {
        let mut left = natural;
        for p in r.pops.iter_mut() {
            let d = (i128::from(natural) * i128::from(p.size) / i128::from(total)) as i64;
            p.size = (p.size + d).max(0);
            left -= d;
        }
        let big = (0..r.pops.len())
            .max_by_key(|&i| r.pops[i].size)
            .unwrap_or(0);
        r.pops[big].size = (r.pops[big].size + left).max(0);
    }
    if moved > 0 {
        r.pops[prof::LABORER].size += moved;
    } else if moved < 0 {
        let poor = [prof::PEASANT, prof::FARM_LABORER, prof::LABORER];
        let pool: i64 = poor.iter().map(|&p| r.pops[p].size).sum();
        let mut left = -moved;
        if pool > 0 {
            for &p in &poor {
                let d =
                    ((i128::from(-moved) * i128::from(r.pops[p].size)) / i128::from(pool)) as i64;
                let d = d.min(r.pops[p].size).min(left);
                r.pops[p].size -= d;
                left -= d;
            }
        }
        // What the poor could not cover leaves from the largest pop.
        if left > 0 {
            let big = (0..r.pops.len())
                .max_by_key(|&i| r.pops[i].size)
                .unwrap_or(0);
            r.pops[big].size = (r.pops[big].size - left).max(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mortality_falls_and_life_rises_with_health() {
        let mut last = mortality(Fixed::ZERO);
        let mut life = life_expectancy(Fixed::ZERO);
        for k in 1..=20 {
            let h = Fixed::from_ratio(k, 20);
            let m = mortality(h);
            assert!(m.iter().zip(&last).all(|(a, b)| a <= b));
            assert!(life_expectancy(h) > life);
            last = m;
            life = life_expectancy(h);
        }
        assert_eq!(life_expectancy(Fixed::ONE), Fixed::from_ratio(82, 1));
    }

    fn crude(h: Fixed, tfr: Fixed) -> (i64, i64) {
        let rates = mortality(h);
        let mut ages = initial_pyramid(10_000_000, |_| (rates, tfr));
        let (mut b, mut d) = (0, 0);
        for _ in 0..12 {
            let (x, y) = month(&mut ages, &rates, tfr);
            b += x;
            d += y;
        }
        (b / 10_000, d / 10_000)
    }

    #[test]
    fn crude_rates_of_old_and_new_regimes() {
        // A poor agrarian population of 1925: births in the 40s, deaths around 30.
        let (b, d) = crude(milli(70), milli(6000));
        assert!((40..=47).contains(&b) && (26..=35).contains(&d), "{b} {d}");
        // Medicine without the fertility transition: growth near 3 % a year.
        let (b, d) = crude(milli(500), milli(6000));
        assert!(b - d >= 22, "{b} {d}");
        // An old rich population: few births, deaths above them.
        let (b, d) = crude(milli(950), milli(1600));
        assert!(b < d, "{b} {d}");
    }

    #[test]
    fn fertility_falls_through_the_transition() {
        let high = fertility(&FertilityInputs {
            child_deaths: mortality(milli(70))[0],
            schooling: milli(150),
            off_farm: milli(250),
            comfort: milli(200),
            contraception: Fixed::ZERO,
        });
        let low = fertility(&FertilityInputs {
            child_deaths: mortality(milli(950))[0],
            schooling: milli(990),
            off_farm: milli(900),
            comfort: milli(900),
            contraception: milli(900),
        });
        assert_eq!(high, TFR_HIGH);
        // Past the transition, fertility sits on the floor that contraception allows.
        let floor = TFR_LOW + TFR_WITHOUT_PILL * (Fixed::ONE - milli(900));
        assert_eq!(low, floor);
        assert!(low < TFR_LOW + TFR_WITHOUT_PILL);
    }

    #[test]
    fn rescale_keeps_the_total() {
        let ages: [i64; AGE_GROUPS] = std::array::from_fn(|g| 1000 - g as i64 * 37);
        for people in [0, 1, 999, 123_457, 10_000_001] {
            assert_eq!(rescale(&ages, people).iter().sum::<i64>(), people);
        }
    }

    #[test]
    fn cosine() {
        assert_eq!(migration::cos_deg(Fixed::ZERO), Fixed::ONE);
        let c60 = migration::cos_deg(Fixed::from_int(60));
        assert!((milli(490)..=milli(510)).contains(&c60), "{c60:?}");
        assert!(migration::cos_deg(Fixed::from_int(90)) <= milli(20));
    }

    #[test]
    fn women_read_less_where_few_read() {
        assert_eq!(women_literacy(Fixed::ONE), Fixed::ONE);
        assert_eq!(women_literacy(Fixed::ZERO), Fixed::ZERO);
        // India 1991: 48 % of all, 34 % of women.
        let w = women_literacy(milli(482));
        assert!((milli(330)..=milli(370)).contains(&w), "{w:?}");
    }
}
