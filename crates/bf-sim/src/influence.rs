//! Influence, blocs and the small country (phase 4), out of the tick: the contract of
//! `docs/influence.pt-BR.md`, as pure functions in fixed point with every term named.
//!
//! - [`dependence`]: what country B loses if A cuts its ties, by channel, with no memory: the debt
//!   B owes A in the ledger of debts between governments; investment, trade and protection stay
//!   **dormant** (named, no value) until their fronts exist.
//! - [`Goodwill`]: what A did for B (aid, debt forgiven, defence honoured) and against it, a stock
//!   that loses a share a year ([`InfluenceRules::goodwill_yearly_retention`]).
//! - [`affinity`]: likeness of regime (laws in force) and of people (B's people of A's main
//!   culture), a factor between the floor and the ceiling of the rules.
//! - [`interest`]: the four together, `influence = (dependence + goodwill) × affinity`, the
//!   reading the treaties' valuation and the strategic AI of phase 6 take.
//! - [`blocs_on`]: the blocs on a date, from the 1925 data: alliances and monetary unions of
//!   `diplomacy.ron`, customs unions under their customs territory, and every overlord with its
//!   subjects.
//!
//! Nothing here is scheduled or saved. Shares are of B's GDP of the last full year.

use std::collections::{BTreeMap, BTreeSet};

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use crate::demography::communities::CommunityTable;
use crate::diplomacy::{DiplomacyFile, OrganizationKind, TreatyKind};
use crate::game::GameWorld;
use crate::scenario::{CountriesFile, Status, Tag};

/// What a proposal is about: its dominant term in the treaties' valuation. Influence counts by
/// issue (Baldwin 1979): a creditor weighs in a debt, a protector in an alliance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Issue {
    Money,
    Trade,
    Territory,
    Security,
    Sovereignty,
}

impl Issue {
    pub const ALL: [Issue; 5] = [
        Issue::Money,
        Issue::Trade,
        Issue::Territory,
        Issue::Security,
        Issue::Sovereignty,
    ];
    pub const COUNT: usize = 5;
}

/// How much each dependence channel counts in one issue, per mille.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueWeights {
    pub issue: Issue,
    pub credit: u32,
    pub investment: u32,
    pub trade: u32,
    pub protection: u32,
}

/// The rules of `data/diplomacy/influence.ron`. Design values, to balance after phase 6.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InfluenceRules {
    /// The weights of the channels in each issue; every issue once.
    pub issues: Vec<IssueWeights>,
    /// Share of goodwill kept from one year to the next, per mille (917: half in 8 years).
    pub goodwill_yearly_retention: u32,
    /// Affinity factor between these two, per mille (800 and 1200: weak, Walt 1987).
    pub affinity_floor: u32,
    pub affinity_ceiling: u32,
    /// Weights of regime and people inside the affinity, per mille.
    pub regime_weight: u32,
    pub people_weight: u32,
    /// Share of B's people of A's main culture at which the people term is full, per mille.
    pub people_full: u32,
    /// What A's influence over B is worth in a proposal of A that B weighs, per mille of
    /// `influence × B's GDP` (the term `Influence` of the treaties' valuation).
    pub proposal_weight: u32,
    /// Most the term may add to a proposal, per mille of the proposal's size: influence makes a
    /// country accept a small loss for a patron, never any loss.
    pub proposal_cap: u32,
    pub source: String,
}

impl InfluenceRules {
    fn weights(&self, issue: Issue) -> IssueWeights {
        self.issues
            .iter()
            .find(|w| w.issue == issue)
            .copied()
            .unwrap_or(IssueWeights {
                issue,
                credit: 0,
                investment: 0,
                trade: 0,
                protection: 0,
            })
    }
}

/// A dependence channel: its value, or the front it waits for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// Share of B's GDP.
    Live(Fixed),
    /// Not measured yet: the i18n key of the front that wakes it (`front-<key>`).
    Dormant(&'static str),
}

impl Channel {
    pub fn value(self) -> Fixed {
        match self {
            Channel::Live(v) => v,
            Channel::Dormant(_) => Fixed::ZERO,
        }
    }
}

/// What B loses if A cuts its ties, channel by channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dependence {
    /// Debt B owes A in the ledger (principal and arrears), over B's GDP.
    pub credit: Channel,
    /// Plants in B owned from A, over B's GDP.
    pub investment: Channel,
    /// B's exports to A over B's GDP, minus A's to B over A's GDP.
    pub trade: Channel,
    /// Defence, bases and troops of A in B.
    pub protection: Channel,
    /// The weighted sum in each issue (`Issue::ALL` order).
    pub by_issue: [Fixed; Issue::COUNT],
}

/// B's GDP of the last full year, or the running one in the first year.
fn gdp(world: &GameWorld, c: usize) -> Fixed {
    world.econ.markets.get(c).map_or(Fixed::ZERO, |m| {
        if m.gdp_last_year > Fixed::ZERO {
            m.gdp_last_year
        } else {
            m.gdp_running
        }
    })
}

fn share(part: Fixed, whole: Fixed) -> Fixed {
    if whole <= Fixed::ZERO || part <= Fixed::ZERO {
        Fixed::ZERO
    } else {
        part / whole
    }
}

fn milli(n: u32) -> Fixed {
    Fixed::from_raw(i64::from(n))
}

/// Dependence of `b` on `a` (country indices) in `year`: each channel is the flow over B's GDP
/// times A's share of that channel in B (Keohane and Nye: vulnerability, what cannot be replaced).
/// Credit: B's debt to A in the ledger, times A's share of B's foreign creditors (the ledger and
/// the bond market; the market counts only while B is not shut out of it after a default).
pub fn dependence(
    world: &GameWorld,
    rules: &InfluenceRules,
    a: usize,
    b: usize,
    year: i32,
) -> Dependence {
    let ledger = &world.econ.ledger;
    let owed: Fixed = ledger
        .debts
        .iter()
        .filter(|d| usize::from(d.debtor) == b && usize::from(d.creditor) == a)
        .map(|d| d.principal + d.arrears)
        .sum();
    let market = world.econ.markets.get(b).map_or(Fixed::ZERO, |m| {
        if m.money.excluded_until > year {
            Fixed::ZERO
        } else {
            m.money.debt_abroad + m.money.debt_fx
        }
    });
    let creditors = ledger.owed_by(b as u16) + market;
    let credit = share(owed, gdp(world, b)) * share(owed, creditors);
    let mut d = Dependence {
        credit: Channel::Live(credit),
        investment: Channel::Dormant("foreign-capital"),
        trade: Channel::Dormant("bilateral-trade"),
        protection: Channel::Dormant("war"),
        by_issue: [Fixed::ZERO; Issue::COUNT],
    };
    for (k, issue) in Issue::ALL.into_iter().enumerate() {
        let w = rules.weights(issue);
        d.by_issue[k] = d.credit.value() * milli(w.credit)
            + d.investment.value() * milli(w.investment)
            + d.trade.value() * milli(w.trade)
            + d.protection.value() * milli(w.protection);
    }
    d
}

/// What A did for B, as a share of B's GDP, kept with a yearly loss.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goodwill(pub Fixed);

impl Goodwill {
    /// An act of `value` (share of B's GDP): positive for aid, forgiveness, defence honoured;
    /// negative for a breach against B.
    pub fn add(&mut self, value: Fixed) {
        self.0 += value;
    }

    /// A year passes.
    pub fn decay_year(&mut self, rules: &InfluenceRules) {
        self.0 *= milli(rules.goodwill_yearly_retention);
    }
}

/// Likeness of A and B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Affinity {
    /// Share of the law groups (not colonial) where A and B hold the same law, 0–1.
    pub regime: Fixed,
    /// Share of B's people of A's main culture, 0–1.
    pub people: Fixed,
    /// The factor, between the floor and the ceiling of the rules.
    pub factor: Fixed,
}

/// Main culture of each country's metropole (most people in its community rows).
pub fn main_cultures(world: &GameWorld, table: &CommunityTable) -> Vec<Option<u16>> {
    let n = world.countries.len();
    let mut by: Vec<BTreeMap<u16, i64>> = vec![BTreeMap::new(); n];
    for (r, rows) in world.econ.regions.iter().zip(&world.demography.communities) {
        if r.colony {
            continue;
        }
        let Some(m) = by.get_mut(usize::from(r.market)) else {
            continue;
        };
        for (id, people) in rows.by_community() {
            *m.entry(table.culture(id)).or_default() += people;
        }
    }
    by.into_iter()
        .map(|m| {
            m.into_iter()
                .max_by(|x, y| x.1.cmp(&y.1).then(y.0.cmp(&x.0)))
                .map(|(c, _)| c)
        })
        .collect()
}

/// Affinity of `a` and `b`. `cultures`: [`main_cultures`]; `colonial`: the law groups that hold
/// only for colonies (left out of the regime likeness).
pub fn affinity(
    world: &GameWorld,
    rules: &InfluenceRules,
    table: Option<&CommunityTable>,
    cultures: &[Option<u16>],
    colonial: &[bool],
    a: usize,
    b: usize,
) -> Affinity {
    let p = &world.politics;
    let regime = match (p.laws.get(a), p.laws.get(b)) {
        (Some(la), Some(lb)) => {
            let (mut same, mut all) = (0i64, 0i64);
            for (g, (x, y)) in la.iter().zip(lb).enumerate() {
                if colonial.get(g).copied().unwrap_or(false) {
                    continue;
                }
                if let (Some(x), Some(y)) = (x, y) {
                    all += 1;
                    if x == y {
                        same += 1;
                    }
                }
            }
            if all > 0 {
                Fixed::from_ratio(same, all)
            } else {
                Fixed::ZERO
            }
        }
        _ => Fixed::ZERO,
    };
    let people = match (table, cultures.get(a).copied().flatten()) {
        (Some(table), Some(culture)) => {
            let (mut of, mut all) = (0i64, 0i64);
            for (r, rows) in world.econ.regions.iter().zip(&world.demography.communities) {
                if usize::from(r.market) != b {
                    continue;
                }
                for (id, n) in rows.by_community() {
                    all += n;
                    if table.culture(id) == culture {
                        of += n;
                    }
                }
            }
            if all > 0 {
                Fixed::from_ratio(of, all)
            } else {
                Fixed::ZERO
            }
        }
        _ => Fixed::ZERO,
    };
    let people_term = (people / milli(rules.people_full.max(1))).min(Fixed::ONE);
    let wr = milli(rules.regime_weight);
    let wp = milli(rules.people_weight);
    let mix = if wr + wp > Fixed::ZERO {
        (regime * wr + people_term * wp) / (wr + wp)
    } else {
        Fixed::ZERO
    };
    let floor = milli(rules.affinity_floor);
    let ceiling = milli(rules.affinity_ceiling);
    Affinity {
        regime,
        people,
        factor: floor + (ceiling - floor) * mix,
    }
}

/// The reading of A over B: what the treaties' valuation (term `Influence`) and the AI of phase 6
/// take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interest {
    pub dependence: Dependence,
    pub goodwill: Goodwill,
    pub affinity: Affinity,
    /// `(dependence + goodwill) × affinity factor` in each issue, share of B's GDP.
    pub influence: [Fixed; Issue::COUNT],
}

/// Interest of `a` in `b`, with the goodwill A holds there.
pub fn interest(dependence: Dependence, goodwill: Goodwill, affinity: Affinity) -> Interest {
    Interest {
        dependence,
        goodwill,
        affinity,
        influence: std::array::from_fn(|k| (dependence.by_issue[k] + goodwill.0) * affinity.factor),
    }
}

/// What A's influence over B is worth in reference units in each issue, before the cap
/// (`influence × B's GDP × proposal_weight`). Zero without influence.
pub fn worth(rules: &InfluenceRules, interest: &Interest, b_gdp: Fixed) -> [Fixed; Issue::COUNT] {
    interest
        .influence
        .map(|i| i.max(Fixed::ZERO) * b_gdp * milli(rules.proposal_weight))
}

/// The influence of every country over every other, ready for the treaties' valuation: (A, B) →
/// worth of A's influence over B in reference units. Only pairs with a live channel or goodwill
/// are in it; a missing pair is zero. Built once a month in the tick (phase 4, etapa 2); the
/// valuation reads it and never computes influence itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InfluenceMap {
    /// Worth by issue (`Issue::ALL` order).
    pub worth: BTreeMap<(u16, u16), [Fixed; Issue::COUNT]>,
    /// The cap, per mille of the proposal's size (`InfluenceRules::proposal_cap`).
    pub cap: u32,
}

impl InfluenceMap {
    /// Every pair with debt in the ledger or goodwill recorded, A over B.
    pub fn build(
        world: &GameWorld,
        rules: &InfluenceRules,
        table: Option<&CommunityTable>,
        colonial: &[bool],
        goodwill: &BTreeMap<(u16, u16), Goodwill>,
        year: i32,
    ) -> Self {
        let cultures = table.map(|t| main_cultures(world, t)).unwrap_or_default();
        let mut pairs: BTreeSet<(u16, u16)> = world
            .econ
            .ledger
            .debts
            .iter()
            .map(|d| (d.creditor, d.debtor))
            .collect();
        pairs.extend(goodwill.keys().copied());
        let mut worth = BTreeMap::new();
        for (a, b) in pairs {
            let (ai, bi) = (usize::from(a), usize::from(b));
            if ai == bi || bi >= world.econ.markets.len() {
                continue;
            }
            let i = interest(
                dependence(world, rules, ai, bi, year),
                goodwill.get(&(a, b)).copied().unwrap_or_default(),
                affinity(world, rules, table, &cultures, colonial, ai, bi),
            );
            let w = self::worth(rules, &i, gdp(world, bi));
            if w.iter().any(|v| *v > Fixed::ZERO) {
                worth.insert((a, b), w);
            }
        }
        Self {
            worth,
            cap: rules.proposal_cap,
        }
    }

    /// The term `Influence` of a proposal of `a` that `b` weighs (`docs/influence.pt-BR.md`,
    /// section 4): the worth of A's influence over B in the proposal's `issue` (its dominant term),
    /// at most `cap` of `size`, the proposal's size
    /// in reference units (the sum of the absolute values of B's other lines). A line of the
    /// proposal, not of a clause.
    pub fn proposal_term(&self, a: u16, b: u16, issue: Issue, size: Fixed) -> Fixed {
        let k = Issue::ALL.iter().position(|i| *i == issue).unwrap_or(0);
        let w = self.worth.get(&(a, b)).map_or(Fixed::ZERO, |w| w[k]);
        w.min(size.abs() * milli(self.cap))
    }
}

/// What ties a bloc together (`docs/influence.pt-BR.md`, section 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlocKind {
    /// Customs union or tariff preference.
    Customs,
    /// Mutual defence.
    Military,
    /// Currency pegged to, or shared with, the leader's.
    Monetary,
    /// An overlord and its subjects.
    Subjects,
}

/// A bloc on a date: a label over treaties with the same bundle and, often, a leader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bloc {
    /// The organization's key, the customs treaty's leader, or `subjects:<overlord>`.
    pub key: String,
    pub kind: BlocKind,
    pub leader: Option<Tag>,
    /// Members, the leader included, sorted.
    pub members: Vec<Tag>,
}

/// The blocs on `date` (ISO), from the scenario's data.
pub fn blocs_on(diplomacy: &DiplomacyFile, countries: &CountriesFile, date: &str) -> Vec<Bloc> {
    let mut out = Vec::new();
    for o in &diplomacy.organizations {
        let kind = match o.kind {
            OrganizationKind::Alliance => BlocKind::Military,
            OrganizationKind::Monetary => BlocKind::Monetary,
            OrganizationKind::Global | OrganizationKind::Regional => continue,
        };
        let mut members: Vec<Tag> = diplomacy
            .members_on(&o.key, date)
            .into_iter()
            .map(|m| m.tag.clone())
            .collect();
        members.sort();
        if members.len() >= 2 {
            out.push(Bloc {
                key: o.key.clone(),
                kind,
                leader: None,
                members,
            });
        }
    }
    let mut customs: BTreeMap<Tag, BTreeSet<Tag>> = BTreeMap::new();
    for t in diplomacy
        .treaties_on(date)
        .filter(|t| t.kind == TreatyKind::CustomsUnion)
    {
        if let Some(leader) = t.senior.first() {
            customs
                .entry(leader.clone())
                .or_default()
                .extend(t.parties.iter().cloned());
        }
    }
    for (leader, members) in customs {
        out.push(Bloc {
            key: format!("customs:{leader}"),
            kind: BlocKind::Customs,
            leader: Some(leader),
            members: members.into_iter().collect(),
        });
    }
    let mut subjects: BTreeMap<Tag, BTreeSet<Tag>> = BTreeMap::new();
    for c in &countries.countries {
        if let Status::Subject { overlord, .. } = &c.status {
            let e = subjects.entry(overlord.clone()).or_default();
            e.insert(overlord.clone());
            e.insert(c.tag.clone());
        }
    }
    for (leader, members) in subjects {
        out.push(Bloc {
            key: format!("subjects:{leader}"),
            kind: BlocKind::Subjects,
            leader: Some(leader),
            members: members.into_iter().collect(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> InfluenceRules {
        InfluenceRules {
            issues: Issue::ALL
                .into_iter()
                .map(|issue| IssueWeights {
                    issue,
                    credit: 1000,
                    investment: 1000,
                    trade: 1000,
                    protection: 1000,
                })
                .collect(),
            goodwill_yearly_retention: 917,
            affinity_floor: 800,
            affinity_ceiling: 1200,
            regime_weight: 500,
            people_weight: 500,
            people_full: 200,
            proposal_weight: 1000,
            proposal_cap: 250,
            source: String::new(),
        }
    }

    #[test]
    fn the_term_is_capped_by_the_proposal() {
        let mut m = InfluenceMap {
            worth: BTreeMap::new(),
            cap: 250,
        };
        let mut w = [Fixed::ZERO; Issue::COUNT];
        w[0] = Fixed::from_int(100);
        m.worth.insert((1, 2), w);
        // Large proposal: the whole worth; small one: a quarter of its size; another issue or no
        // influence: zero.
        let big = Fixed::from_int(1000);
        assert_eq!(
            m.proposal_term(1, 2, Issue::Money, big),
            Fixed::from_int(100)
        );
        assert_eq!(
            m.proposal_term(1, 2, Issue::Money, Fixed::from_int(40)),
            Fixed::from_int(10)
        );
        assert_eq!(m.proposal_term(1, 2, Issue::Security, big), Fixed::ZERO);
        assert_eq!(m.proposal_term(2, 1, Issue::Money, big), Fixed::ZERO);
    }

    #[test]
    fn goodwill_halves_in_about_eight_years() {
        let r = rules();
        let mut g = Goodwill(Fixed::ONE);
        for _ in 0..8 {
            g.decay_year(&r);
        }
        let half = Fixed::from_raw(500);
        assert!((g.0 - half).abs() <= Fixed::from_raw(10), "{:?}", g.0);
    }

    #[test]
    fn influence_is_dependence_and_goodwill_times_affinity() {
        let d = Dependence {
            credit: Channel::Live(Fixed::from_raw(100)),
            investment: Channel::Dormant("foreign-capital"),
            trade: Channel::Dormant("bilateral-trade"),
            protection: Channel::Dormant("war"),
            by_issue: [Fixed::from_raw(100); Issue::COUNT],
        };
        let a = Affinity {
            regime: Fixed::ONE,
            people: Fixed::ZERO,
            factor: Fixed::from_raw(1500),
        };
        let i = interest(d, Goodwill(Fixed::from_raw(20)), a);
        assert_eq!(i.influence[0], Fixed::from_raw(180));
        assert_eq!(d.investment.value(), Fixed::ZERO);
    }
}
