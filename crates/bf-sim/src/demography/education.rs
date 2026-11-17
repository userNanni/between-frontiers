//! Schooling by cohort and level: who is at school, what each age group finished, and the
//! literacy that follows. Rules in `docs/population.pt-BR.md` ("Ensino por coorte").
//!
//! Every state keeps, for each five-year age group of its pyramid, the shares of its people by
//! the highest level they reached ([`Level`]), and the enrolment of each school level as a share
//! of the ages that go to it. Once a year a fifth of every age group is replaced by the one
//! below (the pyramid ages a sixtieth a month), and those who cross the age at which a level
//! ends take the level the schools gave their cohort. Literacy is not a number of its own any
//! more: it is the share of those 15 and over who read, whatever the way they learned.

use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::AGE_GROUPS;

/// Highest level reached. `None` (no schooling, cannot read) is what is left of each group and
/// is not stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    None = 0,
    /// Reads and writes without having been to school: at home, in the army, in an adult
    /// literacy campaign.
    ReadsNoSchool = 1,
    Primary = 2,
    Secondary = 3,
    Tertiary = 4,
}

/// Levels stored per age group (all but `None`).
pub const STORED: usize = 4;
/// Shares are kept in parts per ten thousand.
pub const WHOLE: u32 = 10_000;

/// School levels with enrolment: primary, secondary, tertiary.
pub const SCHOOL_LEVELS: usize = 3;
/// Age groups each level enrols (`AGE_GROUPS` index): primary 5–14, secondary 15–19, tertiary
/// 20–24 (the UNESCO convention for gross enrolment: the five years after secondary for
/// tertiary).
pub const ENROLS: [std::ops::RangeInclusive<usize>; SCHOOL_LEVELS] = [1..=2, 3..=3, 4..=4];
/// Age group a cohort enters when it has finished each level: 15–19 after primary, 20–24 after
/// secondary, 25–29 after tertiary.
pub const FINISHED_AT: [usize; SCHOOL_LEVELS] = [3, 4, 5];
/// First age group counted as adult for literacy (15 and over), and the groups of working age
/// whose mean years of schooling measure the state's human capital (25–64, as Barro and Lee).
pub const ADULT: usize = 3;
pub const WORKING: std::ops::RangeInclusive<usize> = 5..=12;
/// Years of schooling a level stands for: primary 6, secondary 12, tertiary 16 (the common
/// durations; Barro and Lee use each country's own). Reading without school counts 1.
pub const YEARS: [u32; STORED] = [1, 6, 12, 16];

/// `data/scenario/1925/education.ron`: the schooling of 1925 by unit (country, metropole or
/// colonies), generated from Lee and Lee (2016) by `tools/education/build.py`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EducationFile {
    pub first_year: i32,
    pub step: i32,
    pub units: Vec<EducationUnit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EducationUnit {
    pub tag: String,
    pub colonies: bool,
    /// Share of the unit's 1925 population in countries with data (0 for an estimate).
    pub coverage: f32,
    /// (primary, secondary, tertiary) enrolment ratios, %, every `step` years from
    /// `first_year` to 1925.
    pub enrolment: Vec<(f32, f32, f32)>,
    /// Shares of those 15–64 with no schooling, primary, secondary and tertiary as the highest
    /// level attained, %, in 1925 (checks the start; the simulation grows its own).
    pub attainment_15_64: (f32, f32, f32, f32),
    /// Modern countries the means are over (iso3).
    pub countries: String,
    /// How the unit was estimated when none of its countries has data.
    #[serde(default)]
    pub estimate: Option<String>,
}

impl EducationFile {
    /// The unit's enrolment path, per ten thousand of each level's ages.
    pub fn path(&self, tag: &str, colonies: bool) -> Option<Vec<[u32; SCHOOL_LEVELS]>> {
        let u = self
            .units
            .iter()
            .find(|u| u.tag == tag && u.colonies == colonies)?;
        let per = |v: f32| (f64::from(v) * 100.0).round().clamp(0.0, f64::from(WHOLE)) as u32;
        Some(
            u.enrolment
                .iter()
                .map(|&(p, s, t)| [per(p), per(s), per(t)])
                .collect(),
        )
    }
}

/// Mark a command leaves in `EducationState::campaigns` for a campaign just decided; the
/// demography replaces it with the campaign's years.
pub const CAMPAIGN_STARTS: u8 = u8::MAX;

/// `data/economy/literacy_campaign.ron`: the adult literacy campaign, a budget programme.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignRules {
    /// Share of the illiterate adults (15–49) enrolled each year.
    pub reach: f32,
    /// Share of those enrolled who still read years later.
    pub durable: f32,
    /// Cost per participant, in primary pupil-years (`schooling` at its base price).
    pub cost: f32,
    /// Years a campaign runs once decided.
    pub years: u32,
}

impl CampaignRules {
    /// Share of the illiterate adults taught to read for good each year, per ten thousand.
    pub fn yearly(&self) -> u32 {
        (f64::from(self.reach * self.durable) * f64::from(WHOLE)).round() as u32
    }
}

/// A state's schooling, saved and hashed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schooling {
    /// By age group: shares (per ten thousand) of reads-without-school, primary, secondary and
    /// tertiary as the highest level reached; the rest has none.
    pub levels: Vec<[u16; STORED]>,
    /// Enrolment of each school level as a share of the ages it enrols, per ten thousand.
    pub enrolment: [u16; SCHOOL_LEVELS],
    /// Enrolment the cohort now finishing each level had, averaged over the years it spent at
    /// school (an exponential mean over five years), per ten thousand.
    pub cohort: [u16; SCHOOL_LEVELS],
    /// Adults (15–49) without schooling taught to read by a literacy campaign this year.
    pub campaign: i64,
    /// Share of the children who do not reach school but learn to read anyway, found at the
    /// start to meet the state's literacy, per ten thousand.
    pub informal: u16,
    /// Enrolment the education law aims at for each level, per ten thousand of its ages: what
    /// public works build schools toward.
    pub target: [u16; SCHOOL_LEVELS],
    /// How far below what families ask for at its income ([`demand`]) the state's enrolment was
    /// at the start, per ten thousand; it closes by [`GAP_CLOSES`] a year.
    pub gap: [u16; SCHOOL_LEVELS],
}

impl HashState for Schooling {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.levels.len());
        for g in &self.levels {
            g.as_slice().hash_state(h);
        }
        self.enrolment.as_slice().hash_state(h);
        self.cohort.as_slice().hash_state(h);
        self.campaign.hash_state(h);
        self.informal.hash_state(h);
        self.target.as_slice().hash_state(h);
        self.gap.as_slice().hash_state(h);
    }
}

/// What the schools and the law allow this year, as shares per ten thousand of the ages each
/// level enrols: places the buildings offer and the enrolment the education law aims at.
#[derive(Clone, Copy, Debug, Default)]
pub struct SchoolYear {
    pub places: [u32; SCHOOL_LEVELS],
    /// What families ask for at the country's income ([`demand`]): schools of churches,
    /// communities and private owners, which are not buildings of the economy, open where
    /// families pay; enrolment reaches it less the state's own gap.
    pub asked: [u32; SCHOOL_LEVELS],
    pub target: [u32; SCHOOL_LEVELS],
    /// Share of the illiterate adults (15–49) a literacy campaign teaches this year, per ten
    /// thousand; zero without one.
    pub campaign: u32,
    /// Share of the children who do not reach school but learn to read anyway (a literate
    /// parent, the church), per ten thousand.
    pub informal: u32,
}

/// Enrolment families ask for at a country's income: a logistic in the log of GDP per head
/// fitted to Lee and Lee (2016) against Maddison (2023), 1950–2010, 111 countries, 1,361
/// country-years (root mean square error 0.18 for primary and secondary, 0.13 for tertiary;
/// `docs/population.pt-BR.md`, "Ensino por coorte"), tabulated at GDP per head of 500 to 64,000
/// dollars of 2011 (doubling), per ten thousand of each level's ages; linear in between, flat
/// outside. Integer, so that every platform gets the same.
const DEMAND_GDP: [i64; 8] = [500, 1_000, 2_000, 4_000, 8_000, 16_000, 32_000, 64_000];
const DEMAND: [[u32; 8]; SCHOOL_LEVELS] = [
    [2_931, 4_635, 6_429, 7_895, 8_866, 9_422, 9_714, 9_861],
    [375, 807, 1_654, 3_089, 5_021, 6_946, 8_369, 9_205],
    [46, 113, 276, 654, 1_453, 2_847, 4_632, 6_192],
];

/// Share of a state's gap below [`demand`] that closes each year, per ten thousand: the curve
/// was fitted to 1950–2010, and the countries of 1925 sat below it (calibrated: 3 % a year
/// halves the gap in 23 years; docs).
pub const GAP_CLOSES: u32 = 300;

/// [`DEMAND`] of each level at GDP per head `gdp` (dollars of 2011).
pub fn demand(gdp: i64) -> [u32; SCHOOL_LEVELS] {
    std::array::from_fn(|l| {
        let t = &DEMAND[l];
        if gdp <= DEMAND_GDP[0] {
            return t[0];
        }
        for k in 0..DEMAND_GDP.len() - 1 {
            let (lo, hi) = (DEMAND_GDP[k], DEMAND_GDP[k + 1]);
            if gdp < hi {
                let f = (gdp - lo) * 1000 / (hi - lo);
                return (i64::from(t[k]) + (i64::from(t[k + 1]) - i64::from(t[k])) * f / 1000)
                    as u32;
            }
        }
        t[t.len() - 1]
    })
}

/// Share of the gap between enrolment and what places and law allow that closes in a year:
/// families take a few years to fill a new school (calibrated; docs).
pub const ENROLMENT_SPEED: u32 = 5_000;

fn clamp_share(x: i64) -> u16 {
    x.clamp(0, i64::from(WHOLE)) as u16
}

impl Schooling {
    /// Share of each age group (per ten thousand) at or above `level`.
    fn at_least(g: &[u16; STORED], level: Level) -> u32 {
        let from = (level as usize).saturating_sub(1);
        g[from..].iter().map(|&v| u32::from(v)).sum()
    }

    /// One school year. `ages` is the pyramid now. Enrolment moves toward what places and law
    /// allow; every age group takes a fifth of the one below; those who cross the end of a
    /// level take the level their cohort's enrolment gave them; a campaign teaches some of the
    /// illiterate adults.
    pub fn year(&mut self, ages: &[i64; AGE_GROUPS], y: &SchoolYear) {
        if self.levels.len() != AGE_GROUPS {
            self.levels = vec![[0; STORED]; AGE_GROUPS];
        }
        self.target = std::array::from_fn(|l| y.target[l].min(WHOLE) as u16);
        // Enrolment follows the places (the law's target is what the state builds toward, not a
        // cap on families who already send their children); secondary cannot exceed those who
        // finished primary, tertiary those who finished secondary.
        for l in 0..SCHOOL_LEVELS {
            // The state's distance below what its income asks for closes a little each year.
            self.gap[l] = (u32::from(self.gap[l]) * (WHOLE - GAP_CLOSES) / WHOLE) as u16;
            let private = y.asked[l].saturating_sub(u32::from(self.gap[l]));
            let mut allow = y.places[l].max(private).min(WHOLE);
            if l > 0 {
                let finished = Self::at_least(&self.levels[FINISHED_AT[l - 1]], level_of(l - 1));
                allow = allow.min(finished);
            }
            let e = i64::from(self.enrolment[l]);
            let step = (i64::from(allow) - e) * i64::from(ENROLMENT_SPEED) / i64::from(WHOLE);
            self.enrolment[l] = clamp_share(e + step);
            let c = i64::from(self.cohort[l]);
            self.cohort[l] = clamp_share(c + (i64::from(self.enrolment[l]) - c) / 5);
        }
        // Ageing: a fifth of each group comes from the one below (the oldest group keeps its
        // own and takes the inflow too).
        let old = self.levels.clone();
        for g in (1..AGE_GROUPS).rev() {
            let mut inflow = old[g - 1];
            // Crossing the end of a level: at least that level, with the cohort's enrolment.
            for l in 0..SCHOOL_LEVELS {
                if g == FINISHED_AT[l] {
                    inflow = finish(inflow, l, u32::from(self.cohort[l]), y.informal);
                }
            }
            let stay = if g == AGE_GROUPS - 1 {
                // The open group: its own people weigh more than one fifth of it.
                let n = ages[g].max(0);
                let came = ages[g - 1].max(0) / 5;
                (n, came)
            } else {
                (4, 1)
            };
            let (a, b) = stay;
            let total = (a + b).max(1);
            for k in 0..STORED {
                let v = (i64::from(old[g][k]) * a + i64::from(inflow[k]) * b) / total;
                self.levels[g][k] = clamp_share(v);
            }
        }
        // The youngest groups have reached nothing yet.
        for g in self.levels.iter_mut().take(ADULT) {
            *g = [0; STORED];
        }
        // An adult literacy campaign: some of the illiterate 15–49 learn to read.
        let mut taught = 0i64;
        if y.campaign > 0 {
            for g in ADULT..=9 {
                let none = WHOLE - Self::at_least(&self.levels[g], Level::ReadsNoSchool);
                let moved = none * y.campaign / WHOLE;
                self.levels[g][0] = clamp_share(i64::from(self.levels[g][0]) + i64::from(moved));
                taught += ages[g].max(0) * i64::from(moved) / i64::from(WHOLE);
            }
        }
        self.campaign = taught;
    }

    /// Share of those 15 and over who read and write (per ten thousand of the adults), the
    /// literacy of the history series and of 1925's data.
    pub fn literacy(&self, ages: &[i64; AGE_GROUPS]) -> u32 {
        self.weighted(ages, ADULT..=AGE_GROUPS - 1, |g| {
            u64::from(Self::at_least(g, Level::ReadsNoSchool))
        }) as u32
    }

    /// Share of the people of `groups` who read, per ten thousand.
    pub fn literacy_of(
        &self,
        ages: &[i64; AGE_GROUPS],
        groups: std::ops::RangeInclusive<usize>,
    ) -> u32 {
        self.weighted(ages, groups, |g| {
            u64::from(Self::at_least(g, Level::ReadsNoSchool))
        }) as u32
    }

    /// Mean years of schooling of those 25–64.
    pub fn mean_years(&self, ages: &[i64; AGE_GROUPS]) -> f64 {
        let w = self.weighted(ages, WORKING, |g| {
            g.iter()
                .zip(YEARS)
                .map(|(&s, y)| u64::from(s) * u64::from(y))
                .sum::<u64>()
        });
        w as f64 / f64::from(WHOLE)
    }

    /// Share of those 25–64 with tertiary schooling, per ten thousand.
    pub fn tertiary(&self, ages: &[i64; AGE_GROUPS]) -> u32 {
        self.weighted(ages, WORKING, |g| u64::from(g[STORED - 1])) as u32
    }

    /// People of each age group by highest level, `[none, reads, primary, secondary,
    /// tertiary]`; each group adds up to its people (largest remainder).
    pub fn people(&self, ages: &[i64; AGE_GROUPS]) -> Vec<[i64; STORED + 1]> {
        ages.iter()
            .enumerate()
            .map(|(g, &n)| {
                let s = self.levels.get(g).copied().unwrap_or([0; STORED]);
                let stored: u32 = s.iter().map(|&v| u32::from(v)).sum::<u32>().min(WHOLE);
                let weights: Vec<i128> = std::iter::once(i128::from(WHOLE - stored))
                    .chain(s.iter().map(|&v| i128::from(v)))
                    .collect();
                let v = super::communities::apportion(n.max(0), &weights);
                std::array::from_fn(|k| v[k])
            })
            .collect()
    }

    /// Pupils of each school level: enrolment times the ages it enrols.
    pub fn pupils(&self, ages: &[i64; AGE_GROUPS]) -> [i64; SCHOOL_LEVELS] {
        std::array::from_fn(|l| {
            let n: i64 = ENROLS[l].clone().map(|g| ages[g].max(0)).sum();
            n * i64::from(self.enrolment[l]) / i64::from(WHOLE)
        })
    }

    /// Scales each level, over the adult groups, so that the share of those 15–64 who reached
    /// it is `attained` (per ten thousand: primary, secondary, tertiary).
    fn match_attainment(&mut self, ages: &[i64; AGE_GROUPS], attained: [u32; SCHOOL_LEVELS]) {
        // Cumulative shares by group: at least primary, at least secondary, tertiary.
        let mut cum: Vec<[u32; 3]> = self
            .levels
            .iter()
            .map(|g| {
                let t = u32::from(g[3]);
                let s = t + u32::from(g[2]);
                [s + u32::from(g[1]), s, t]
            })
            .collect();
        let adults = ADULT..=12;
        let total: i64 = adults.clone().map(|g| ages[g].max(0)).sum();
        if total <= 0 {
            return;
        }
        for k in 0..3 {
            let now: i64 = adults
                .clone()
                .map(|g| ages[g].max(0) * i64::from(cum[g][k]))
                .sum::<i64>()
                / total;
            let want = i64::from(attained[k].min(WHOLE));
            for c in cum.iter_mut().skip(ADULT) {
                let cap = if k == 0 { WHOLE } else { c[k - 1] };
                let v = if now > 0 {
                    i64::from(c[k]) * want / now
                } else {
                    want
                };
                c[k] = (v.max(0) as u32).min(cap);
            }
        }
        for g in ADULT..AGE_GROUPS {
            let [c1, c2, c3] = cum[g];
            let reads = u32::from(self.levels[g][0]).min(WHOLE - c1);
            self.levels[g] = [reads as u16, (c1 - c2) as u16, (c2 - c3) as u16, c3 as u16];
        }
    }

    fn weighted(
        &self,
        ages: &[i64; AGE_GROUPS],
        groups: std::ops::RangeInclusive<usize>,
        f: impl Fn(&[u16; STORED]) -> u64,
    ) -> u64 {
        let (mut num, mut den) = (0u128, 0u128);
        for g in groups {
            let n = ages[g].max(0) as u128;
            let s = self.levels.get(g).copied().unwrap_or([0; STORED]);
            num += n * u128::from(f(&s));
            den += n;
        }
        num.checked_div(den).map_or(0, |v| v as u64)
    }
}

/// The schooling of a state at the start: its schools' enrolment of the past, `path` (per ten
/// thousand of each level's ages, every five years up to the start), lived year by year on the
/// start's pyramid, with the share of children who learn to read without school found so that
/// the literacy of those 15 and over is `literacy` (per ten thousand). When schools alone
/// already give more, the informal share is zero and literacy comes out higher.
///
/// `attained`, when known, is the share of those 15–64 who reached primary, secondary and
/// tertiary schooling, complete or not (per ten thousand; Lee and Lee's attainment): each level
/// is then scaled over the adult groups to it, keeping the younger groups ahead of the older.
/// Enrolment alone undershoots it (Lee and Lee count anyone who entered a level, and secondary
/// begins at 12).
pub fn initial(
    ages: &[i64; AGE_GROUPS],
    path: &[[u32; SCHOOL_LEVELS]],
    attained: Option<[u32; SCHOOL_LEVELS]>,
    literacy: u32,
) -> Schooling {
    let years = (path.len().max(1) as u32 - 1) * 5;
    let at = |y: u32| -> [u32; SCHOOL_LEVELS] {
        if path.is_empty() {
            return [0; SCHOOL_LEVELS];
        }
        let i = ((y / 5) as usize).min(path.len() - 1);
        let j = (i + 1).min(path.len() - 1);
        let f = y % 5;
        std::array::from_fn(|l| (path[i][l] * (5 - f) + path[j][l] * f) / 5)
    };
    let enrolment = at(years);
    let grow = |informal: u32| -> Schooling {
        let mut s = Schooling::default();
        for y in 0..=years {
            let target = at(y);
            s.year(
                ages,
                &SchoolYear {
                    places: target,
                    asked: [0; SCHOOL_LEVELS],
                    target,
                    campaign: 0,
                    informal,
                },
            );
        }
        // The start's enrolment as it was, not where it was heading.
        s.enrolment = std::array::from_fn(|l| enrolment[l].min(WHOLE) as u16);
        if let Some(a) = attained {
            s.match_attainment(ages, a);
        }
        s
    };
    let (mut lo, mut hi) = (0u32, WHOLE);
    let mut best = grow(0);
    let mut informal = 0;
    if best.literacy(ages) < literacy {
        best = grow(WHOLE);
        informal = WHOLE;
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            let s = grow(mid);
            if s.literacy(ages) < literacy {
                lo = mid;
            } else {
                hi = mid;
                best = s;
                informal = mid;
            }
        }
    }
    best.informal = informal as u16;
    best
}

/// Schooling of every state and the adult literacy campaigns, saved and hashed
/// (`GameWorld::education`). Empty before the first January of a world with schooling data and
/// in saves from before it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EducationState {
    /// By region (`Economy::regions`).
    pub regions: Vec<Schooling>,
    /// By country (market): years left of its adult literacy campaign, 0 without one.
    pub campaigns: Vec<u8>,
    /// By country: what its campaign cost last month (base prices), for the budget screen.
    pub campaign_cost: Vec<laplace_schelling::Fixed>,
}

impl EducationState {
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty() && self.campaigns.iter().all(|&c| c == 0)
    }
}

impl HashState for EducationState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.regions.len());
        for r in &self.regions {
            r.hash_state(h);
        }
        self.campaigns.as_slice().hash_state(h);
        self.campaign_cost.hash_state(h);
    }
}

/// What the simulation reads to run the schools: the start and the campaign's rules, and the
/// school building of each level (`school`, `secondary_school`, `university`).
#[derive(Debug)]
pub struct EducationRules {
    /// The rule's switch (`data/rules.ron`). Off, a new campaign has no schooling; a world that
    /// already has it (a save from a campaign with the rule on) keeps it.
    pub enabled: bool,
    pub start: EducationFile,
    pub campaign: CampaignRules,
    pub kinds: [Option<usize>; SCHOOL_LEVELS],
}

/// Human capital of a region for the other systems (research, productivity): mean years of
/// schooling of those 25–64 and their share with tertiary schooling (0–1). `None` before the
/// schooling exists.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HumanCapital {
    pub mean_years: f64,
    pub tertiary: f64,
}

/// Places each school level of a region needs to reach its law's target: target × the ages it
/// enrols, in pupils. Zero before the schooling exists.
pub fn places_needed(world: &crate::game::GameWorld, ri: usize) -> [i64; SCHOOL_LEVELS] {
    let (Some(s), Some(ages)) = (
        world.education.regions.get(ri),
        world.demography.ages.get(ri),
    ) else {
        return [0; SCHOOL_LEVELS];
    };
    std::array::from_fn(|l| {
        let n: i64 = ENROLS[l].clone().map(|g| ages[g].max(0)).sum();
        n * i64::from(s.target[l]) / i64::from(WHOLE)
    })
}

/// Share of those 15–64 of region `ri` with at least primary, secondary and tertiary
/// schooling, for the labour market; `None` before the schooling exists.
pub fn qualified(
    world: &crate::game::GameWorld,
    ri: usize,
) -> Option<[laplace_schelling::Fixed; 3]> {
    let s = world.education.regions.get(ri)?;
    let ages = world.demography.ages.get(ri)?;
    let share = |k: usize| {
        s.weighted(ages, ADULT..=12, |g| {
            g[k..].iter().map(|&v| u64::from(v)).sum::<u64>()
        })
    };
    Some(std::array::from_fn(|l| {
        laplace_schelling::Fixed::from_ratio(share(l + 1) as i64, i64::from(WHOLE))
    }))
}

/// What a country's adult literacy campaign is or would be, for the budget screen.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CampaignOutlook {
    /// Years left; 0 without a campaign.
    pub years_left: u8,
    /// Adults 15–49 who cannot read, today.
    pub illiterate: i64,
    /// Adults it enrols in a year, and those of them who will still read years later.
    pub enrolled_per_year: i64,
    pub readers_per_year: i64,
    /// What it costs a year at base prices, and what it cost last month while running.
    pub cost_per_year: f64,
    pub cost_last_month: f64,
    /// Adults taught to read at the last school year.
    pub taught_last_year: i64,
}

/// [`CampaignOutlook`] of country `country` (market index); `None` before schooling exists.
pub fn campaign_outlook(
    world: &crate::game::GameWorld,
    rules: &EducationRules,
    price_of_schooling: f64,
    country: usize,
) -> Option<CampaignOutlook> {
    if world.education.regions.len() != world.econ.regions.len() {
        return None;
    }
    let mut out = CampaignOutlook {
        years_left: world.education.campaigns.get(country).copied().unwrap_or(0),
        cost_last_month: world
            .education
            .campaign_cost
            .get(country)
            .map_or(0.0, |c| c.to_f64_lossy()),
        ..Default::default()
    };
    if out.years_left == CAMPAIGN_STARTS {
        out.years_left = rules.campaign.years.min(200) as u8;
    }
    for ((r, s), ages) in world
        .econ
        .regions
        .iter()
        .zip(&world.education.regions)
        .zip(&world.demography.ages)
    {
        if usize::from(r.market) != country {
            continue;
        }
        for row in &s.people(ages)[ADULT..=9] {
            out.illiterate += row[0];
        }
        out.taught_last_year += s.campaign;
    }
    let n = out.illiterate as f64;
    out.enrolled_per_year = (n * f64::from(rules.campaign.reach)) as i64;
    out.readers_per_year = (n * f64::from(rules.campaign.reach * rules.campaign.durable)) as i64;
    out.cost_per_year =
        n * f64::from(rules.campaign.reach) * f64::from(rules.campaign.cost) * price_of_schooling;
    Some(out)
}

/// [`HumanCapital`] of region `ri` of the world.
pub fn human_capital(world: &crate::game::GameWorld, ri: usize) -> Option<HumanCapital> {
    let s = world.education.regions.get(ri)?;
    let ages = world.demography.ages.get(ri)?;
    Some(HumanCapital {
        mean_years: s.mean_years(ages),
        tertiary: f64::from(s.tertiary(ages)) / f64::from(WHOLE),
    })
}

fn level_of(l: usize) -> Level {
    match l {
        0 => Level::Primary,
        1 => Level::Secondary,
        _ => Level::Tertiary,
    }
}

/// A cohort crossing the end of school level `l`, with `enrolled` of it (per ten thousand)
/// having been at that level: they reach it (secondary and tertiary only from the level below),
/// and after primary, `informal` of the rest learn to read anyway.
fn finish(mut s: [u16; STORED], l: usize, enrolled: u32, informal: u32) -> [u16; STORED] {
    let k = l + 1; // index of the level in `STORED`
    // Who is below the level and can move up: for primary, everyone below; for the others,
    // those exactly at the level below.
    let pool: u32 = if l == 0 {
        WHOLE - s[k..].iter().map(|&v| u32::from(v)).sum::<u32>()
    } else {
        u32::from(s[k - 1])
    };
    let have: u32 = s[k..].iter().map(|&v| u32::from(v)).sum();
    let want = if l == 0 {
        enrolled.min(WHOLE)
    } else {
        enrolled.min(have + pool)
    };
    let up = want.saturating_sub(have).min(pool);
    if up > 0 {
        if l == 0 {
            // Taken from the illiterate first, then from those who read without school.
            let none = WHOLE - s.iter().map(|&v| u32::from(v)).sum::<u32>();
            let from_none = up.min(none);
            let from_reads = (up - from_none).min(u32::from(s[0]));
            s[0] -= from_reads as u16;
        } else {
            s[k - 1] -= up as u16;
        }
        s[k] += up as u16;
    }
    if l == 0 && informal > 0 {
        let none = WHOLE - s.iter().map(|&v| u32::from(v)).sum::<u32>();
        s[0] += (none * informal / WHOLE) as u16;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pyramid() -> [i64; AGE_GROUPS] {
        // A young population: each group 3 % smaller than the one below.
        let mut a = [0i64; AGE_GROUPS];
        let mut n = 1_000_000i64;
        for g in a.iter_mut() {
            *g = n;
            n = n * 97 / 100;
        }
        a
    }

    fn total_ok(s: &Schooling) {
        for g in &s.levels {
            assert!(g.iter().map(|&v| u32::from(v)).sum::<u32>() <= WHOLE);
        }
    }

    #[test]
    fn people_add_up_to_each_group() {
        let mut s = Schooling::default();
        let ages = pyramid();
        let y = SchoolYear {
            places: [WHOLE; 3],
            asked: [0; 3],
            target: [8_000, 2_000, 500],
            campaign: 0,
            informal: 500,
        };
        for _ in 0..40 {
            s.year(&ages, &y);
            total_ok(&s);
            for (g, row) in s.people(&ages).iter().enumerate() {
                assert_eq!(row.iter().sum::<i64>(), ages[g]);
            }
        }
    }

    #[test]
    fn more_places_fill_schools_in_the_year_reach_the_young_in_five_and_adults_slowly() {
        let ages = pyramid();
        let little = SchoolYear {
            places: [2_000, 200, 50],
            asked: [0; 3],
            target: [WHOLE; 3],
            campaign: 0,
            informal: 300,
        };
        let mut s = Schooling::default();
        for _ in 0..60 {
            s.year(&ages, &little);
        }
        let (e0, young0, lit0) = (s.enrolment[0], s.levels[3][1], s.literacy(&ages));
        // Twice the primary places.
        let more = SchoolYear {
            places: [4_000, 200, 50],
            asked: [0; 3],
            ..little
        };
        s.year(&ages, &more);
        assert!(s.enrolment[0] > e0 + 500, "enrolment moves in the year");
        assert!(
            s.levels[3][1] < young0 + 300,
            "the 15–19 have not changed yet"
        );
        for _ in 0..4 {
            s.year(&ages, &more);
        }
        // A child who enters the new school in 1931 is 15–19 only from 1940: in five years
        // the group has moved part of the way, in fifteen most of it.
        let five = s.levels[3][1];
        assert!(
            five > young0 + 300 && five < young0 + 1_000,
            "{young0} -> {five}"
        );
        assert!(
            s.literacy(&ages) < lit0 + 400,
            "adult literacy moves slowly"
        );
        for _ in 0..10 {
            s.year(&ages, &more);
        }
        assert!(
            s.levels[3][1] > young0 + 1_400,
            "most of the way in fifteen years"
        );
    }

    #[test]
    fn the_start_matches_its_literacy_and_the_young_read_more() {
        let ages = pyramid();
        let path: Vec<[u32; 3]> = (0..13)
            .map(|i| [1_000 + 160 * i, 100 + 30 * i, 20 + 7 * i])
            .collect();
        let s = initial(&ages, &path, None, 4_000);
        let lit = s.literacy(&ages);
        assert!((3_950..=4_100).contains(&lit), "{lit}");
        let reads = |g: usize| Schooling::at_least(&s.levels[g], Level::ReadsNoSchool);
        assert!(reads(3) > reads(12), "15–19 read more than 60–64");
        assert_eq!(s.enrolment, [2_920, 460, 104]);
    }

    #[test]
    fn a_campaign_teaches_adults_within_a_few_years() {
        let ages = pyramid();
        let y = SchoolYear {
            places: [2_000, 200, 50],
            asked: [0; 3],
            target: [WHOLE; 3],
            campaign: 0,
            informal: 300,
        };
        let mut s = Schooling::default();
        for _ in 0..60 {
            s.year(&ages, &y);
        }
        let before = s.literacy(&ages);
        let with = SchoolYear {
            campaign: 3_000,
            ..y
        };
        for _ in 0..3 {
            s.year(&ages, &with);
        }
        assert!(s.campaign > 0);
        assert!(
            s.literacy(&ages) > before + 2_000,
            "{} -> {}",
            before,
            s.literacy(&ages)
        );
    }
}
