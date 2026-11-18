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
use crate::diplomacy::{DiplomacyFile, OrganizationKind};
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
    /// The trade and investment channels read the world (`docs/influence.pt-BR.md`, 13c). On
    /// only with `game::Switches::influence_ties` (written by `GameData::rules_with`, never read
    /// from the data): off, both stay dormant and the hash is the one without them.
    #[serde(default)]
    pub live_ties: bool,
    /// The foreign service: missions, teams, the cost at home.
    pub service: ServiceRules,
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

/// The front key of a channel whose front is in the game but switched off in `data/rules.ron`
/// (`front-rules-off`), as against a front still to come (`war`).
pub const RULES_OFF: &str = "rules-off";

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

/// What the trade and investment channels read, built once per pass over the pairs (the month's
/// map, a frame of the window): trade by pair from `Economy::trade` and foreign capital by pair
/// from the stakes (`foreign_stakes::owned_shares`).
#[derive(Clone, Debug, Default)]
pub struct Ties {
    /// Trade by pair is in the world: the trade channel is live.
    pub trade_live: bool,
    /// Read the year so far, not the last full year: the first year of a game, before any
    /// January 1st closed one.
    pub running: bool,
    /// Each country's exports to every other, in the year read, base prices.
    pub exports: Vec<Fixed>,
    /// Foreign capital by pair; `None` without the economy catalogue (investment dormant).
    pub owned: Option<crate::foreign_stakes::OwnedShares>,
}

impl Ties {
    /// The world's ties in `year`. Without the catalogue the investment channel stays dormant.
    pub fn of(
        world: &GameWorld,
        cat: Option<&crate::econ::catalogue::Catalogue>,
        year: i32,
    ) -> Self {
        let t = &world.econ.trade;
        let n = usize::from(t.n);
        let running = t.last_year.iter().all(|v| *v == Fixed::ZERO);
        let flows = if running { &t.year } else { &t.last_year };
        let exports = (0..n)
            .map(|s| {
                (0..n)
                    .filter(|b| *b != s)
                    .map(|b| flows.get(s * n + b).copied().unwrap_or(Fixed::ZERO))
                    .sum()
            })
            .collect();
        Self {
            trade_live: !t.is_empty(),
            running,
            exports,
            owned: cat.map(|c| crate::foreign_stakes::owned_shares(world, c, year)),
        }
    }

    /// The ties the rules let the channels read: the world's with `live_ties`, none without.
    pub fn under(setup: &InfluenceSetup, world: &GameWorld, year: i32) -> Self {
        if setup.rules.live_ties {
            Self::of(world, setup.catalogue.as_deref(), year)
        } else {
            Self::default()
        }
    }

    /// Exports of `from` to `to` in the year read, base prices.
    pub fn flow(&self, world: &GameWorld, from: usize, to: usize) -> Fixed {
        let t = &world.econ.trade;
        let n = usize::from(t.n);
        if from >= n || to >= n || from == to {
            return Fixed::ZERO;
        }
        let v = if self.running { &t.year } else { &t.last_year };
        v.get(from * n + to).copied().unwrap_or(Fixed::ZERO)
    }

    /// The pairs (A, B) the two channels can make live: A buys from B, or A owns plants in B.
    pub fn pairs(&self, world: &GameWorld) -> BTreeSet<(u16, u16)> {
        let mut out = BTreeSet::new();
        if self.trade_live {
            let n = usize::from(world.econ.trade.n);
            for b in 0..n {
                for a in 0..n {
                    if self.flow(world, b, a) > Fixed::ZERO {
                        out.insert((a as u16, b as u16));
                    }
                }
            }
        }
        if let Some(o) = &self.owned {
            out.extend(o.pair.keys().copied());
        }
        out
    }
}

/// Trade channel of A over B: B's exports to A over B's GDP, times A's share of B's exports
/// (what B could not sell elsewhere at once), minus the same of A toward B (Hirschman 1945: the
/// asymmetry is the power). Never below zero: the other side is the pair (B, A).
pub fn trade_dependence(world: &GameWorld, ties: &Ties, a: usize, b: usize) -> Fixed {
    let exp = |c: usize| ties.exports.get(c).copied().unwrap_or(Fixed::ZERO);
    let x_ba = ties.flow(world, b, a);
    let x_ab = ties.flow(world, a, b);
    let of_b = share(x_ba, gdp(world, b)) * share(x_ba, exp(b));
    let of_a = share(x_ab, gdp(world, a)) * share(x_ab, exp(a));
    (of_b - of_a).max(Fixed::ZERO)
}

/// Investment channel of A over B: the plants of B owned from A over B's GDP, times A's share of
/// B's foreign capital (an owner among many weighs less than the only one).
pub fn investment_dependence(
    owned: &crate::foreign_stakes::OwnedShares,
    a: usize,
    b: usize,
) -> Fixed {
    let mine = owned.of(a as u16, b as u16);
    mine * share(mine, owned.foreign(b as u16))
}

/// Dependence of `b` on `a` (country indices) in `year`: each channel is the flow over B's GDP
/// times A's share of that channel in B (Keohane and Nye: vulnerability, what cannot be replaced).
/// Credit: B's debt to A in the ledger, times A's share of B's foreign creditors (the ledger and
/// the bond market; the market counts only while B is not shut out of it after a default).
/// Trade and investment: [`trade_dependence`] and [`investment_dependence`], read from `ties`.
pub fn dependence(
    world: &GameWorld,
    rules: &InfluenceRules,
    ties: &Ties,
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
        // Both fronts are in the game: dormant here means a rule switched off
        // (`influence_ties`, and `trade_by_pair` for trade), not a front still to come.
        investment: match &ties.owned {
            Some(o) => Channel::Live(investment_dependence(o, a, b)),
            None => Channel::Dormant(RULES_OFF),
        },
        trade: if ties.trade_live {
            Channel::Live(trade_dependence(world, ties, a, b))
        } else {
            Channel::Dormant(RULES_OFF)
        },
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
    /// Every pair with debt in the ledger, goodwill recorded, trade or foreign capital, A over B.
    pub fn build(
        world: &GameWorld,
        rules: &InfluenceRules,
        table: Option<&CommunityTable>,
        colonial: &[bool],
        goodwill: &BTreeMap<(u16, u16), Goodwill>,
        ties: &Ties,
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
        pairs.extend(ties.pairs(world));
        let mut worth = BTreeMap::new();
        for (a, b) in pairs {
            let (ai, bi) = (usize::from(a), usize::from(b));
            if ai == bi || bi >= world.econ.markets.len() {
                continue;
            }
            let i = interest(
                dependence(world, rules, ties, ai, bi, year),
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
    /// The organization's key, `<kind>:<leader>`, `<kind>:<members>`, or `subjects:<overlord>`.
    pub key: String,
    pub kind: BlocKind,
    pub leader: Option<Tag>,
    /// Members, the leader included, sorted.
    pub members: Vec<Tag>,
}

/// The name a treaty's clauses give it on the screen, after the Correlates of War and ATOP
/// (`docs/wiki/decisoes/aliancas.md`). A label computed from the clauses; the rule stays in them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AllianceLabel {
    /// Defence of one side only, or a guarantee of a border.
    Guarantee,
    /// Mutual defence alone (COW I; ATOP defence).
    DefencePact,
    /// Mutual defence and more: consultation, bases, troops, passage, foreign policy.
    Alliance,
    /// Non-aggression without defence (COW IIb).
    NonAggressionPact,
    /// Consultation alone (COW III; ATOP consultation).
    Entente,
}

/// The alliance label of a treaty's clauses, or `None` when it promises nothing of security.
pub fn alliance_label(clauses: &[crate::treaty::Clause]) -> Option<AllianceLabel> {
    use crate::treaty::{ClauseKind as K, Side};
    let has = |f: &dyn Fn(&K) -> bool| clauses.iter().any(|c| f(&c.kind));
    let mutual_defence = clauses
        .iter()
        .any(|c| matches!(c.kind, K::Defence) && c.giver == Side::All);
    let one_sided = has(&|k| matches!(k, K::Defence | K::BorderGuarantee));
    let more = has(&|k| {
        matches!(
            k,
            K::Consultation | K::Base { .. } | K::TroopsQuota | K::Passage | K::ForeignPolicy
        )
    });
    Some(if mutual_defence && more {
        AllianceLabel::Alliance
    } else if mutual_defence {
        AllianceLabel::DefencePact
    } else if one_sided {
        AllianceLabel::Guarantee
    } else if has(&|k| matches!(k, K::NonAggression)) {
        AllianceLabel::NonAggressionPact
    } else if has(&|k| matches!(k, K::Consultation)) {
        AllianceLabel::Entente
    } else {
        return None;
    })
}

/// The Correlates of War type (Formal Alliances v4.1), from the strongest obligation: defence is
/// type I, neutrality IIa, non-aggression IIb, consultation alone III. Offence without defence has
/// no COW type (ATOP calls it an offensive pact).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CowType {
    I,
    IIa,
    IIb,
    III,
}

/// The COW type of a treaty's clauses, or `None`.
pub fn cow_type(clauses: &[crate::treaty::Clause]) -> Option<CowType> {
    use crate::treaty::ClauseKind as K;
    let has = |f: &dyn Fn(&K) -> bool| clauses.iter().any(|c| f(&c.kind));
    if has(&|k| matches!(k, K::Defence | K::BorderGuarantee)) {
        Some(CowType::I)
    } else if has(&|k| matches!(k, K::Neutrality)) {
        Some(CowType::IIa)
    } else if has(&|k| matches!(k, K::NonAggression)) {
        Some(CowType::IIb)
    } else if has(&|k| matches!(k, K::Consultation)) {
        Some(CowType::III)
    } else {
        None
    }
}

/// What bundle ties a treaty into a bloc, and its leader (the senior side), if any.
fn bundle_of(t: &crate::diplomacy::Treaty) -> Option<(BlocKind, bool)> {
    use crate::treaty::{ClauseKind as K, Side};
    let any = |f: &dyn Fn(&K) -> bool| t.clauses.iter().any(|c| f(&c.kind));
    if any(&|k| matches!(k, K::CustomsUnion | K::TariffPreference { .. })) {
        Some((BlocKind::Customs, true))
    } else if any(&|k| matches!(k, K::CurrencyPeg | K::MonetaryUnion)) {
        Some((BlocKind::Monetary, !any(&|k| matches!(k, K::MonetaryUnion))))
    } else if t
        .clauses
        .iter()
        .any(|c| matches!(c.kind, K::Defence) && c.giver == Side::All)
    {
        Some((BlocKind::Military, false))
    } else if any(&|k| matches!(k, K::Defence | K::BorderGuarantee)) {
        Some((BlocKind::Military, true))
    } else {
        None
    }
}

/// The maximal cliques of an undirected graph, each sorted (Bron–Kerbosch with a pivot, in
/// `BTreeSet` order for determinism).
fn cliques(adj: &BTreeMap<Tag, BTreeSet<Tag>>) -> Vec<Vec<Tag>> {
    fn go(
        adj: &BTreeMap<Tag, BTreeSet<Tag>>,
        r: Vec<Tag>,
        mut p: BTreeSet<Tag>,
        mut x: BTreeSet<Tag>,
        out: &mut Vec<Vec<Tag>>,
    ) {
        if p.is_empty() && x.is_empty() {
            let mut r = r;
            r.sort();
            out.push(r);
            return;
        }
        let empty = BTreeSet::new();
        let pivot = p.iter().chain(&x).next().cloned().unwrap_or_default();
        let np = adj.get(&pivot).unwrap_or(&empty);
        let candidates: Vec<Tag> = p.difference(np).cloned().collect();
        for v in candidates {
            let nv = adj.get(&v).unwrap_or(&empty);
            let mut r2 = r.clone();
            r2.push(v.clone());
            go(
                adj,
                r2,
                p.intersection(nv).cloned().collect(),
                x.intersection(nv).cloned().collect(),
                out,
            );
            p.remove(&v);
            x.insert(v);
        }
    }
    let mut out = Vec::new();
    go(
        adj,
        Vec::new(),
        adj.keys().cloned().collect(),
        BTreeSet::new(),
        &mut out,
    );
    out
}

/// The blocs over a set of treaties in force (`docs/influence.pt-BR.md`, section 5): a label
/// read only from the clauses, never a structure of its own.
///
/// - **With a leader** (the senior side of customs, a currency peg, a one-sided defence or
///   guarantee): the leader and everyone tied to it by that bundle. Customs and money make a
///   bloc from one member; defence from two (one guarantee alone is a guarantee).
/// - **Without a leader** (mutual defence, a monetary union): the countries all tied to each
///   other by the bundle (maximal cliques), from three. A pair alone is an alliance, not a bloc:
///   the Little Entente is a bloc, the Estonian–Latvian pact of 1923 an alliance.
/// - **Subjects:** an overlord and its subjects, given as (overlord, subject) pairs.
/// - A bloc whose treaties all name the same organization takes its key (`little_entente`);
///   an organization of kind `Monetary` without treaties in the clause form is a bloc by its
///   members (the Latin Union, dormant until 1927).
pub fn blocs_from<'a>(
    treaties: impl IntoIterator<Item = &'a crate::diplomacy::Treaty>,
    monetary_orgs: impl IntoIterator<Item = (String, Vec<Tag>)>,
    subjects: impl IntoIterator<Item = (Tag, Tag)>,
) -> Vec<Bloc> {
    let mut led: BTreeMap<(BlocKind, Tag), BTreeSet<Tag>> = BTreeMap::new();
    let mut even: BTreeMap<BlocKind, BTreeMap<Tag, BTreeSet<Tag>>> = BTreeMap::new();
    let mut org_of: BTreeMap<(Tag, Tag), BTreeSet<Option<String>>> = BTreeMap::new();
    for t in treaties {
        let Some((kind, with_leader)) = bundle_of(t) else {
            continue;
        };
        match (with_leader, t.senior.first()) {
            (true, Some(leader)) => {
                led.entry((kind, leader.clone()))
                    .or_default()
                    .extend(t.parties.iter().filter(|p| *p != leader).cloned());
            }
            _ => {
                let g = even.entry(kind).or_default();
                for a in &t.parties {
                    for b in &t.parties {
                        if a != b {
                            g.entry(a.clone()).or_default().insert(b.clone());
                            org_of
                                .entry((a.clone(), b.clone()))
                                .or_default()
                                .insert(t.bloc.clone());
                        }
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for ((kind, leader), members) in led {
        let least = if kind == BlocKind::Military { 2 } else { 1 };
        if members.len() < least {
            continue;
        }
        let prefix = match kind {
            BlocKind::Customs => "customs",
            BlocKind::Monetary => "monetary",
            BlocKind::Military => "military",
            BlocKind::Subjects => "subjects",
        };
        let mut all: Vec<Tag> = members.into_iter().collect();
        all.push(leader.clone());
        all.sort();
        out.push(Bloc {
            key: format!("{prefix}:{leader}"),
            kind,
            leader: Some(leader),
            members: all,
        });
    }
    for (kind, adj) in even {
        for members in cliques(&adj) {
            if members.len() < 3 {
                continue;
            }
            let mut orgs = BTreeSet::new();
            for a in &members {
                for b in &members {
                    if a != b {
                        orgs.extend(org_of.get(&(a.clone(), b.clone())).into_iter().flatten());
                    }
                }
            }
            let key = match orgs.iter().collect::<Vec<_>>().as_slice() {
                [Some(o)] => o.clone(),
                _ => format!("{:?}:{}", kind, members.join("-")).to_lowercase(),
            };
            out.push(Bloc {
                key,
                kind,
                leader: None,
                members,
            });
        }
    }
    for (key, mut members) in monetary_orgs {
        members.sort();
        if members.len() >= 2 && out.iter().all(|b| b.key != key) {
            out.push(Bloc {
                key,
                kind: BlocKind::Monetary,
                leader: None,
                members,
            });
        }
    }
    let mut by_overlord: BTreeMap<Tag, BTreeSet<Tag>> = BTreeMap::new();
    for (overlord, subject) in subjects {
        let e = by_overlord.entry(overlord.clone()).or_default();
        e.insert(overlord);
        e.insert(subject);
    }
    for (leader, members) in by_overlord {
        out.push(Bloc {
            key: format!("subjects:{leader}"),
            kind: BlocKind::Subjects,
            leader: Some(leader),
            members: members.into_iter().collect(),
        });
    }
    out
}

/// The blocs on `date` (ISO), from the scenario's data: [`blocs_from`] over the treaties in force
/// that day.
pub fn blocs_on(diplomacy: &DiplomacyFile, countries: &CountriesFile, date: &str) -> Vec<Bloc> {
    let monetary = diplomacy
        .organizations
        .iter()
        .filter(|o| o.kind == OrganizationKind::Monetary)
        .map(|o| {
            let m = diplomacy
                .members_on(&o.key, date)
                .into_iter()
                .map(|m| m.tag.clone())
                .collect();
            (o.key.clone(), m)
        });
    let subjects = countries.countries.iter().filter_map(|c| match &c.status {
        Status::Subject { overlord, .. } => Some((overlord.clone(), c.tag.clone())),
        _ => None,
    });
    blocs_from(diplomacy.treaties_on(date), monetary, subjects)
}

fn iso(d: laplace_schelling::Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

fn parse_iso(s: &str) -> Option<laplace_schelling::Date> {
    let mut it = s.splitn(3, '-');
    let y = it.next()?.parse().ok()?;
    let m = it.next()?.parse().ok()?;
    let d = it.next()?.get(..2)?.parse().ok()?;
    Some(laplace_schelling::Date::new(y, m, d))
}

/// One member of a bloc, as the "Blocs" lens shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlocMember {
    /// Country index.
    pub country: u16,
    /// When its first treaty with another member of the bloc came into force (for a subject,
    /// since when it is one; for a monetary organization, the founding).
    pub since: laplace_schelling::Date,
    /// Keys of the treaties in force that tie it to the other members (`treaty-<key>`).
    pub treaties: Vec<String>,
}

/// A bloc for the map: a stable key (`bloc-<key>` in the Fluent files, the same from month to
/// month while the bloc keeps its members and leader), the members by index and the dates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlocView {
    pub key: String,
    pub kind: BlocKind,
    pub leader: Option<u16>,
    pub members: Vec<BlocMember>,
    /// When the bloc as it stands came to be: the last link of a bloc without a leader (the
    /// Little Entente in June 1921), the first of a bloc with one.
    pub since: laplace_schelling::Date,
}

/// The blocs on `date` with the world's countries and subjects: [`blocs_from`] over the treaties
/// in force, the subjects of the world (after border changes), countries by index. Until the
/// treaties keep the agreements in the world, the treaties come from the scenario's data; the
/// return type stays when they do.
pub fn bloc_views(
    diplomacy: &DiplomacyFile,
    borders: &crate::borders::BorderRules,
    world: &GameWorld,
    date: laplace_schelling::Date,
) -> Vec<BlocView> {
    let day = iso(date);
    let index: BTreeMap<&str, u16> = world
        .countries
        .iter()
        .enumerate()
        .map(|(i, (_, c))| (c.tag.as_str(), i as u16))
        .collect();
    let tag = |i: usize| world.countries.iter().nth(i).map(|(_, c)| c.tag.clone());
    let subject_of = |c: usize| {
        let list = if world.borders.subjects.is_empty() {
            &borders.subjects
        } else {
            &world.borders.subjects
        };
        list.get(c).and_then(|s| s.clone())
    };
    let subjects: Vec<(Tag, Tag)> = (0..world.countries.len())
        .filter_map(|c| {
            let s = subject_of(c)?;
            Some((tag(usize::from(s.overlord))?, tag(c)?))
        })
        .collect();
    let monetary: Vec<(String, Vec<Tag>)> = diplomacy
        .organizations
        .iter()
        .filter(|o| o.kind == OrganizationKind::Monetary)
        .map(|o| {
            let m = diplomacy
                .members_on(&o.key, &day)
                .into_iter()
                .map(|m| m.tag.clone())
                .collect();
            (o.key.clone(), m)
        })
        .collect();
    let in_force: Vec<&crate::diplomacy::Treaty> = diplomacy.treaties_on(&day).collect();
    let blocs = blocs_from(in_force.iter().copied(), monetary, subjects);
    let mut out = Vec::new();
    for b in blocs {
        let ix = |t: &Tag| index.get(t.as_str()).copied();
        let founded = diplomacy
            .organizations
            .iter()
            .find(|o| o.key == b.key)
            .and_then(|o| parse_iso(&o.founded));
        let ties = |a: &Tag, other: &Tag| {
            in_force
                .iter()
                .filter(|t| bundle_of(t).is_some_and(|(k, _)| k == b.kind))
                .filter(|t| t.parties.contains(a) && t.parties.contains(other))
                .copied()
                .collect::<Vec<_>>()
        };
        let mut members = Vec::new();
        let mut pair_starts = Vec::new();
        for m in &b.members {
            let Some(country) = ix(m) else { continue };
            let mut treaties = BTreeSet::new();
            let mut since: Option<laplace_schelling::Date> = None;
            if b.kind == BlocKind::Subjects {
                since = subject_of(usize::from(country)).map(|s| s.since);
            }
            for other in b.members.iter().filter(|o| *o != m) {
                let linking = ties(m, other);
                let first = linking.iter().filter_map(|t| parse_iso(&t.in_force)).min();
                if m < other {
                    pair_starts.extend(first);
                }
                for t in linking {
                    treaties.insert(t.key.clone());
                }
                since = match (since, first) {
                    (Some(a), Some(f)) => Some(a.min(f)),
                    (a, f) => a.or(f),
                };
            }
            members.push(BlocMember {
                country,
                since: since.or(founded).unwrap_or(date),
                treaties: treaties.into_iter().collect(),
            });
        }
        let since = if b.leader.is_none() && !pair_starts.is_empty() {
            pair_starts.iter().copied().max().unwrap_or(date)
        } else {
            members
                .iter()
                .filter(|m| Some(m.country) != b.leader.as_ref().and_then(ix))
                .map(|m| m.since)
                .min()
                .or(founded)
                .unwrap_or(date)
        };
        out.push(BlocView {
            key: b.key.clone(),
            kind: b.kind,
            leader: b.leader.as_ref().and_then(ix),
            members,
            since,
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
            live_ties: false,
            service: ServiceRules {
                great_powers: 7,
                missions_per_team: 10,
                without_mission: 500,
                efficiency_gain: 1000,
                domestic_weight: 200,
                stance_unit: 500,
            },
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

// ---------------------------------------------------------------------------------------------
// The limits of diplomacy (`docs/influence.pt-BR.md`, section 12; decisions 4 to 6 of section 13)
// ---------------------------------------------------------------------------------------------

/// The foreign service's rules, in `data/diplomacy/influence.ron`. Design values.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceRules {
    /// The largest economies that count as great powers for the missions of 1925.
    pub great_powers: usize,
    /// Missions abroad one negotiating team serves.
    pub missions_per_team: u32,
    /// Share of goodwill an act yields without a mission in the receiver, per mille.
    pub without_mission: u32,
    /// How much administrative efficiency (the research effect `research`: computers, the
    /// internet) adds to the teams, per mille at full effect.
    pub efficiency_gain: u32,
    /// Legitimacy a rapprochement costs at full rejection by every group that counts, per mille.
    pub domestic_weight: u32,
    /// The stance of a group that counts as full rejection (the politics' stance unit).
    pub stance_unit: i64,
}

/// A diplomatic mission between two countries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Mission {
    Legation,
    Embassy,
}

/// The missions of 1925 by rule (the Correlates of War data may not be redistributed): great
/// powers hold embassies with each other and legations in every other sovereign capital; the
/// others hold legations with their land neighbours, their overlord and the great powers. Pairs
/// are (lower index, higher index).
pub fn missions_1925(
    world: &GameWorld,
    service: &ServiceRules,
    borders: &crate::borders::BorderRules,
) -> BTreeMap<(u16, u16), Mission> {
    let n = world.countries.len();
    let held: Vec<bool> = (0..n)
        .map(|c| {
            world
                .econ
                .regions
                .iter()
                .any(|r| usize::from(r.market) == c)
        })
        .collect();
    let mut by_gdp: Vec<usize> = (0..n).filter(|&c| held[c]).collect();
    by_gdp.sort_by(|&a, &b| gdp(world, b).cmp(&gdp(world, a)).then(a.cmp(&b)));
    let great: BTreeSet<usize> = by_gdp.iter().take(service.great_powers).copied().collect();
    let sovereign = |c: usize| held[c] && borders.overlord_of(world, c).is_none();
    let mut out = BTreeMap::new();
    let mut put = |a: usize, b: usize, m: Mission| {
        if a == b {
            return;
        }
        let key = (a.min(b) as u16, a.max(b) as u16);
        let e = out.entry(key).or_insert(m);
        *e = (*e).max(m);
    };
    for &g in &great {
        for (c, _) in held.iter().enumerate().filter(|(c, h)| **h && *c != g) {
            if great.contains(&c) {
                put(g, c, Mission::Embassy);
            } else if sovereign(c) {
                put(g, c, Mission::Legation);
            }
        }
    }
    // Land neighbours, from the map's state borders.
    let by_state: BTreeMap<u16, usize> = world
        .econ
        .regions
        .iter()
        .map(|r| (r.state, usize::from(r.market)))
        .collect();
    for r in &world.econ.regions {
        let a = usize::from(r.market);
        for s in borders.neighbours.get(&r.state).into_iter().flatten() {
            if let Some(&b) = by_state.get(s) {
                put(a, b, Mission::Legation);
            }
        }
    }
    for c in 0..n {
        if let Some(o) = borders.overlord_of(world, c) {
            put(c, o, Mission::Legation);
        }
    }
    out
}

/// Missions country `c` holds abroad.
pub fn missions_of(missions: &BTreeMap<(u16, u16), Mission>, c: u16) -> usize {
    missions.keys().filter(|(a, b)| *a == c || *b == c).count()
}

/// Negotiations `c` can hold at once (decision 5): one team per `missions_per_team` missions
/// abroad, at least one, more with administrative efficiency (the research effect `research`).
pub fn negotiation_slots(
    world: &GameWorld,
    service: &ServiceRules,
    missions: &BTreeMap<(u16, u16), Mission>,
    c: usize,
) -> u16 {
    let teams =
        1 + missions_of(missions, c as u16) as i64 / i64::from(service.missions_per_team.max(1));
    // `effects` holds, by country, the strength of each research effect, 0–1; index 3 is
    // `research` (`research::EFFECTS`).
    let effect = world.research.effects.get(c).map_or(Fixed::ZERO, |e| e[3]);
    let gain = Fixed::ONE + effect * milli(service.efficiency_gain);
    (Fixed::from_int(teams) * gain)
        .to_int_floor()
        .clamp(1, i64::from(u16::MAX)) as u16
}

/// Share of goodwill an act of `a` toward `b` yields (decision 4): whole with a mission, less
/// without.
pub fn act_yield(
    missions: &BTreeMap<(u16, u16), Mission>,
    service: &ServiceRules,
    a: u16,
    b: u16,
) -> Fixed {
    if missions.contains_key(&(a.min(b), a.max(b))) {
        Fixed::ONE
    } else {
        milli(service.without_mission)
    }
}

/// What a rapprochement of `a` with `b` costs `a` at home, in legitimacy per mille (decision 6):
/// for each law of `b`'s regime, how much the interest groups of `a` that count (by clout) would
/// oppose adopting it, the same stances the politics measures for laws (Putnam 1988).
pub fn domestic_cost(
    world: &GameWorld,
    politics: &crate::politics::Politics,
    service: &ServiceRules,
    a: usize,
    b: usize,
) -> Fixed {
    let s = &world.politics;
    let (Some(la), Some(lb), Some(clout)) = (s.laws.get(a), s.laws.get(b), s.clout.get(a)) else {
        return Fixed::ZERO;
    };
    let unit = service.stance_unit.max(1);
    let (mut against, mut groups) = (0i64, 0i64);
    for (g, (x, y)) in la.iter().zip(lb).enumerate() {
        let (Some(x), Some(y)) = (x, y) else { continue };
        if politics.groups.get(g).is_some_and(|gr| gr.colonial) || x == y {
            continue;
        }
        groups += 1;
        let st = crate::polity::stances(politics, s, a, usize::from(*y));
        for (ig, stance) in st.iter().enumerate() {
            let w = i64::from(clout.get(ig).copied().unwrap_or(0));
            let rejection = (-stance.total).clamp(0, unit) * 1000 / unit;
            against += w * rejection / 1000;
        }
    }
    let total_groups = la
        .iter()
        .zip(lb)
        .filter(|(x, y)| x.is_some() && y.is_some())
        .count() as i64;
    if total_groups == 0 || groups == 0 {
        return Fixed::ZERO;
    }
    // Average over the law groups of the regime: per mille of full rejection by all who count.
    Fixed::from_ratio(against, total_groups * 1000) * milli(service.domestic_weight)
}

// In the tick (phase 4, etapa 2)
// ---------------------------------------------------------------------------------------------

/// The influence state of the world: the goodwill A holds in B (only pairs with an act), and the
/// map of the month the treaties' valuation reads. Saved; hashed only when not empty, so a world
/// with no influence (the rules missing, or `proposal_weight` zero and no act) keeps its hash.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfluenceState {
    pub goodwill: BTreeMap<(u16, u16), Goodwill>,
    /// Worth of A's influence over B by issue, reference units, as of the last 16th.
    pub map: BTreeMap<(u16, u16), [Fixed; Issue::COUNT]>,
}

impl InfluenceState {
    pub fn is_empty(&self) -> bool {
        self.goodwill.is_empty() && self.map.is_empty()
    }

    /// The map as the treaties' valuation takes it, with the cap of the rules.
    pub fn as_map(&self, rules: &InfluenceRules) -> InfluenceMap {
        InfluenceMap {
            worth: self.map.clone(),
            cap: rules.proposal_cap,
        }
    }
}

impl laplace_schelling::hash::HashState for InfluenceState {
    fn hash_state(&self, h: &mut laplace_schelling::hash::StateHasher) {
        h.write_len(self.goodwill.len());
        for ((a, b), g) in &self.goodwill {
            h.write_u16(*a);
            h.write_u16(*b);
            g.0.hash_state(h);
        }
        h.write_len(self.map.len());
        for ((a, b), w) in &self.map {
            h.write_u16(*a);
            h.write_u16(*b);
            w.as_slice().hash_state(h);
        }
    }
}

/// What the influence system reads besides the world.
#[derive(Clone)]
pub struct InfluenceSetup {
    pub rules: InfluenceRules,
    pub table: Option<std::sync::Arc<CommunityTable>>,
    /// Law groups that hold only for colonies.
    pub colonial: Vec<bool>,
    /// The economy catalogue, for the value of foreign plants (investment channel); `None`
    /// leaves the channel dormant.
    pub catalogue: Option<std::sync::Arc<crate::econ::catalogue::Catalogue>>,
    /// The overlords of the start date (`BorderRules::subjects`), read by the lenses while the
    /// world has not changed a border yet.
    pub borders: Option<std::sync::Arc<crate::borders::BorderRules>>,
}

/// A dependence channel by name, for the influence window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelKind {
    Credit,
    Investment,
    Trade,
    Protection,
}

impl ChannelKind {
    pub const ALL: [ChannelKind; 4] = [
        ChannelKind::Credit,
        ChannelKind::Investment,
        ChannelKind::Trade,
        ChannelKind::Protection,
    ];
}

/// One bar: a channel in one issue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelLine {
    pub kind: ChannelKind,
    /// The channel's value (share of B's GDP), or the front it waits for.
    pub channel: Channel,
    /// How much it counts in this issue, per mille (`data/diplomacy/influence.ron`).
    pub weight: u32,
    /// `value × weight × affinity factor`, share of B's GDP; zero while dormant.
    pub part: Fixed,
}

/// Where A's influence over B in one issue comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssueReading {
    pub issue: Issue,
    pub channels: [ChannelLine; 4],
    /// `goodwill × affinity factor`, share of B's GDP (negative after a breach).
    pub goodwill: Fixed,
    /// The sum: the influence of A over B in this issue, share of B's GDP.
    pub influence: Fixed,
}

/// Everything the influence window shows of A over B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairReading {
    pub interest: Interest,
    /// `Issue::ALL` order.
    pub issues: [IssueReading; Issue::COUNT],
    /// The term the treaties' valuation takes, reference units, from the map of the last 16th
    /// (zero for a pair not in it).
    pub worth: [Fixed; Issue::COUNT],
}

/// The influence of `a` over `b` (country indices) channel by channel, issue by issue, with the
/// goodwill held in the world and the given `year` (the date's). The window reads this; the
/// parts of each issue add up to its influence, a share of B's GDP.
pub fn channels(
    world: &GameWorld,
    setup: &InfluenceSetup,
    a: usize,
    b: usize,
    year: i32,
) -> PairReading {
    let rules = &setup.rules;
    let cultures = setup
        .table
        .as_deref()
        .map(|t| main_cultures(world, t))
        .unwrap_or_default();
    let key = (a as u16, b as u16);
    let ties = Ties::under(setup, world, year);
    let i = interest(
        dependence(world, rules, &ties, a, b, year),
        world
            .influence
            .goodwill
            .get(&key)
            .copied()
            .unwrap_or_default(),
        affinity(
            world,
            rules,
            setup.table.as_deref(),
            &cultures,
            &setup.colonial,
            a,
            b,
        ),
    );
    let d = i.dependence;
    let f = i.affinity.factor;
    let issues = std::array::from_fn(|k| {
        let issue = Issue::ALL[k];
        let w = rules.weights(issue);
        let line = |kind, channel: Channel, weight: u32| ChannelLine {
            kind,
            channel,
            weight,
            part: channel.value() * milli(weight) * f,
        };
        IssueReading {
            issue,
            channels: [
                line(ChannelKind::Credit, d.credit, w.credit),
                line(ChannelKind::Investment, d.investment, w.investment),
                line(ChannelKind::Trade, d.trade, w.trade),
                line(ChannelKind::Protection, d.protection, w.protection),
            ],
            goodwill: i.goodwill.0 * f,
            influence: i.influence[k],
        }
    });
    PairReading {
        interest: i,
        issues,
        worth: world
            .influence
            .map
            .get(&key)
            .copied()
            .unwrap_or([Fixed::ZERO; Issue::COUNT]),
    }
}

/// Influence of `a` over `b` in each issue, share of B's GDP, with the cultures computed once.
fn influence_pair(
    world: &GameWorld,
    setup: &InfluenceSetup,
    cultures: &[Option<u16>],
    ties: &Ties,
    a: usize,
    b: usize,
    year: i32,
) -> [Fixed; Issue::COUNT] {
    let rules = &setup.rules;
    interest(
        dependence(world, rules, ties, a, b, year),
        world
            .influence
            .goodwill
            .get(&(a as u16, b as u16))
            .copied()
            .unwrap_or_default(),
        affinity(
            world,
            rules,
            setup.table.as_deref(),
            cultures,
            &setup.colonial,
            a,
            b,
        ),
    )
    .influence
}

/// The pairs the lenses read: a creditor over its debtor in the ledger, every pair with goodwill
/// recorded, the pairs of trade and foreign capital the ties read, and every overlord over its
/// subjects (with no channel measured, its reading names the fronts it waits for). Every other
/// pair is zero.
fn live_pairs(world: &GameWorld, setup: &InfluenceSetup, ties: &Ties) -> BTreeSet<(u16, u16)> {
    let mut pairs: BTreeSet<(u16, u16)> = world
        .econ
        .ledger
        .debts
        .iter()
        .map(|d| (d.creditor, d.debtor))
        .collect();
    pairs.extend(world.influence.goodwill.keys().copied());
    pairs.extend(ties.pairs(world));
    // Every overlord over its subjects, measured or not: the lens shows the pair with its
    // channels (dormant ones with their front), never as "nothing measured".
    if let Some(b) = &setup.borders {
        pairs.extend(
            (0..world.countries.len())
                .filter_map(|c| b.overlord_of(world, c).map(|o| (o as u16, c as u16))),
        );
    }
    pairs.retain(|(a, b)| a != b);
    pairs
}

/// For the "Influence" lens: the influence of `a` over every country that holds any, by issue,
/// share of each one's GDP. Countries left out are zero. One pass for the whole map (the cultures
/// once), so a monthly call costs about one `channels`.
pub fn influence_over(
    world: &GameWorld,
    setup: &InfluenceSetup,
    a: usize,
    year: i32,
) -> Vec<(u16, [Fixed; Issue::COUNT])> {
    let cultures = setup
        .table
        .as_deref()
        .map(|t| main_cultures(world, t))
        .unwrap_or_default();
    let ties = Ties::under(setup, world, year);
    live_pairs(world, setup, &ties)
        .into_iter()
        .filter(|(x, _)| usize::from(*x) == a)
        .map(|(_, b)| {
            (
                b,
                influence_pair(world, setup, &cultures, &ties, a, usize::from(b), year),
            )
        })
        .collect()
}

/// The other way: the influence of every country that holds any over `b`.
pub fn influence_on(
    world: &GameWorld,
    setup: &InfluenceSetup,
    b: usize,
    year: i32,
) -> Vec<(u16, [Fixed; Issue::COUNT])> {
    let cultures = setup
        .table
        .as_deref()
        .map(|t| main_cultures(world, t))
        .unwrap_or_default();
    let ties = Ties::under(setup, world, year);
    live_pairs(world, setup, &ties)
        .into_iter()
        .filter(|(_, y)| usize::from(*y) == b)
        .map(|(a, _)| {
            (
                a,
                influence_pair(world, setup, &cultures, &ties, usize::from(a), b, year),
            )
        })
        .collect()
}

/// The state of the world's influence.
pub const INFLUENCE: laplace_schelling::schedule::ResourceId =
    laplace_schelling::schedule::ResourceId(15);

/// Every month on the 16th, after the demography of the 15th: the goodwill loses its yearly share
/// in January, and the map of the month is built again.
pub struct InfluenceSystem {
    pub setup: std::sync::Arc<InfluenceSetup>,
}

impl laplace_schelling::schedule::System<GameWorld> for InfluenceSystem {
    type Changes = InfluenceState;

    fn name(&self) -> &'static str {
        "influence"
    }

    fn cadence(&self) -> laplace_schelling::schedule::Cadence {
        laplace_schelling::schedule::Cadence::Monthly { day: 16 }
    }

    fn access(&self) -> laplace_schelling::schedule::Access {
        laplace_schelling::schedule::Access::new()
            .read(crate::econ::week::ECON)
            .read(crate::polity::POLITY)
            .read(crate::demography::DEMOGRAPHY)
            .read(crate::foreign_stakes::FOREIGN)
            .write(INFLUENCE)
    }

    fn compute(
        &self,
        ctx: &laplace_schelling::schedule::TickCtx,
        world: &GameWorld,
    ) -> InfluenceState {
        let s = &self.setup;
        let mut goodwill = world.influence.goodwill.clone();
        if ctx.date.month == 1 {
            for g in goodwill.values_mut() {
                g.decay_year(&s.rules);
            }
            goodwill.retain(|_, g| g.0 != Fixed::ZERO);
        }
        let map = InfluenceMap::build(
            world,
            &s.rules,
            s.table.as_deref(),
            &s.colonial,
            &goodwill,
            &Ties::under(s, world, ctx.date.year),
            ctx.date.year,
        );
        InfluenceState {
            goodwill,
            map: map.worth,
        }
    }

    fn apply(
        &self,
        _ctx: &laplace_schelling::schedule::TickCtx,
        changes: InfluenceState,
        world: &mut GameWorld,
    ) {
        world.influence = changes;
    }
}
