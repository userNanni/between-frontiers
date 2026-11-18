//! Treaties as sets of clauses (phase 4). Design: `docs/treaties.pt-BR.md`.
//!
//! An agreement is a list of [`Clause`]s, each with the side that gives and the side that
//! receives. Peace, independence, recognition, subjection and alliance are not kinds of treaty:
//! they are agreements with certain clauses. This module holds the contract, out of the tick:
//!
//! - the clause catalogue ([`ClauseCatalogue`], `data/diplomacy/clauses.ron`): for every
//!   [`ClauseKind`], whether it acts once or holds while the treaty is in force, the front of the
//!   simulation it needs ([`Front`]: a clause whose front is not live is *dormant* and the screen
//!   says why and what wakes it), its weight in sovereignty, what the interest groups weigh in it,
//!   the subject bundles and the owner's switches ([`Policy`]);
//! - the proposal and its evaluation ([`Proposal`], [`evaluate`]): the same open account for the
//!   AI and for the player, term by term, clause by clause;
//! - the ledger of debts between states ([`Ledger`], [`InterstateDebt`]): debtor and creditor by
//!   name, conserved by construction (each line is one debt and one claim of the same amount);
//! - the war debts of 1925 ([`WarDebtsFile`], `data/scenario/1925/war_debts.ron`), switched by
//!   [`Policy::war_debts_1925`].
//!
//! Everything is integer fixed point and free of randomness. Names and texts live in
//! `assets/i18n/*/treaties.ftl`: `clause-<key>`, `clause-<key>-effect`, `front-<key>`,
//! `term-<key>`, `bundle-<key>`.

use std::collections::{BTreeMap, BTreeSet};

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use crate::scenario::{SubjectKind, Tag};

// ---------------------------------------------------------------------------------------------
// Clauses
// ---------------------------------------------------------------------------------------------

/// Who gives or receives a clause, among the parties of a treaty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// The strong side (`Treaty::senior`): guarantors, victors, overlords, creditors.
    Senior,
    /// The other parties.
    Junior,
    /// Every party, to every other party (a mutual clause).
    All,
    /// The named parties.
    Parties(Vec<Tag>),
}

/// One clause: what it does, who gives it and who receives it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clause {
    pub kind: ClauseKind,
    pub giver: Side,
    pub receiver: Side,
    /// The clause ends before the treaty (the evacuation of a zone, a frozen tariff).
    #[serde(default)]
    pub until: Option<String>,
    /// What the data could not say exactly (a zone covering states only in part).
    #[serde(default)]
    pub note: Option<String>,
}

/// Limit on armed forces: absolute, or a share of another party's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmsCap {
    pub party: Tag,
    pub category: ArmsCategory,
    /// Tonnage (ships) or men (army).
    pub max: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArmsCategory {
    /// Men under arms.
    Army,
    /// Standard displacement of capital ships, tons.
    CapitalShips,
    /// Standard displacement of aircraft carriers, tons.
    Carriers,
}

/// What a clause does. The mechanics live here; limits, weights and texts in the catalogue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClauseKind {
    // Once, on entry into force.
    /// The states pass from the giver to the receiver (`borders::Transfer`).
    CedeStates { states: Vec<String> },
    /// The states become a new country, sovereign or a subject in `bundle` (`borders::Secede`).
    ReleaseCountry {
        states: Vec<String>,
        tag: Tag,
        bundle: Option<String>,
    },
    /// The giver joins the receiver (`borders::Annex`).
    Union,
    /// A debt of the giver to the receiver, in the ledger. The amounts of the 1925 debts live in
    /// `war_debts.ron` under the treaty's key; a debt written in play carries its terms.
    Debt { terms: Option<DebtTerms> },
    /// The receiver takes over this share of the giver's market debt (succession).
    AssumeDebt { share: Fixed },
    /// The creditor (giver) changes the terms of the debts of `treaty` (moratorium, rescheduling,
    /// forgiveness).
    ReviseDebt { treaty: String, terms: DebtTerms },
    /// One payment from the giver's treasury to the receiver's, in reference units.
    Payment { amount: Fixed },
    /// Techniques taken or handed over (`borders::TechniqueTerm`).
    Techniques,
    // While in force.
    /// The giver recognises the receiver as a state; with `border`, its present borders too.
    Recognition { border: bool },
    /// The giver starts no crisis and no war against the receiver.
    NonAggression,
    /// The giver goes to war if the receiver is attacked.
    Defence,
    /// The giver consults the receiver before acting in the region.
    Consultation,
    /// The giver defends the receiver's border against any party.
    BorderGuarantee,
    /// The giver keeps no troops nor fortifications in the states.
    Demilitarized { states: Vec<String> },
    /// Ceilings on the giver's forces.
    ArmsLimit { caps: Vec<ArmsCap> },
    /// The receiver keeps a base and a garrison in the giver's states (or sea zone).
    Base { at: Vec<String> },
    /// The giver lets the receiver's forces through.
    Passage,
    /// The receiver occupies the giver's states as a guarantee.
    Occupation { states: Vec<String> },
    /// The giver's tariff on the receiver's goods at most `rate`; `None`: frozen where it was.
    TariffCap { rate: Option<Fixed> },
    /// The giver lowers its tariff for the receiver by `margin`.
    TariffPreference { margin: Fixed },
    /// The giver applies the receiver's tariff and joins its customs territory.
    CustomsUnion,
    /// The giver's currency is tied to the receiver's.
    CurrencyPeg,
    /// A common currency.
    MonetaryUnion,
    /// The giver does not take the receiver's capital without paying.
    CapitalProtection,
    /// The receiver's officials collect and spend part of the giver's revenue (customs
    /// receivership, reparation commission).
    FinancialControl,
    /// A yearly payment from the giver to the receiver, in reference units.
    Tribute { yearly: Fixed },
    /// The giver signs no treaty without the receiver.
    ForeignPolicy,
    /// The receiver may impose a law or a government on the giver.
    Intervention,
    /// The giver protects a community (`communities`, by id) from targeted laws.
    Minorities { community: Option<u16> },
    /// The giver puts troops at the receiver's disposal.
    TroopsQuota,
}

/// Terms of a debt between states, in reference units (the economy's 2011 dollars at 1925 base
/// prices, as `Money::debt_fx`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebtTerms {
    pub principal: Fixed,
    /// Yearly interest, as a fraction.
    pub rate: Fixed,
    /// Yearly payment due (interest and amortization).
    pub annuity: Fixed,
    pub years: u16,
    #[serde(default)]
    pub currency: Currency,
    /// Share of the annuity the debtor may defer (the Young plan).
    #[serde(default)]
    pub deferrable: Fixed,
    /// Share of the debtor's revenue above which the service is deferred, not defaulted (Dawes).
    #[serde(default)]
    pub transfer_cap: Option<Fixed>,
    #[serde(default)]
    pub pledge: Option<Pledge>,
    /// Nominal part, outside every sum until a condition of the treaty is met (the C bonds).
    #[serde(default)]
    pub contingent: Fixed,
    /// Years without service from the start (moratorium).
    #[serde(default)]
    pub grace_years: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Currency {
    /// Gold value: reference units (as `Money::debt_fx`).
    #[default]
    Reference,
    /// The debtor's currency: its inflation erodes the debt.
    Debtor,
    /// The creditor's currency.
    Creditor,
}

/// Revenue pledged to the service before it reaches the budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Pledge {
    Customs,
    ConsumptionTaxes,
    Railways,
}

impl ClauseKind {
    /// Catalogue key: `clause-<key>` in the Fluent files.
    pub fn key(&self) -> &'static str {
        use ClauseKind::*;
        match self {
            CedeStates { .. } => "cede_states",
            ReleaseCountry { .. } => "release_country",
            Union => "union",
            Debt { .. } => "debt",
            AssumeDebt { .. } => "assume_debt",
            ReviseDebt { .. } => "revise_debt",
            Payment { .. } => "payment",
            Techniques => "techniques",
            Recognition { .. } => "recognition",
            NonAggression => "non_aggression",
            Defence => "defence",
            Consultation => "consultation",
            BorderGuarantee => "border_guarantee",
            Demilitarized { .. } => "demilitarized",
            ArmsLimit { .. } => "arms_limit",
            Base { .. } => "base",
            Passage => "passage",
            Occupation { .. } => "occupation",
            TariffCap { .. } => "tariff_cap",
            TariffPreference { .. } => "tariff_preference",
            CustomsUnion => "customs_union",
            CurrencyPeg => "currency_peg",
            MonetaryUnion => "monetary_union",
            CapitalProtection => "capital_protection",
            FinancialControl => "financial_control",
            Tribute { .. } => "tribute",
            ForeignPolicy => "foreign_policy",
            Intervention => "intervention",
            Minorities { .. } => "minorities",
            TroopsQuota => "troops_quota",
        }
    }

    /// Every catalogue key, in declaration order.
    pub const KEYS: [&'static str; 30] = [
        "cede_states",
        "release_country",
        "union",
        "debt",
        "assume_debt",
        "revise_debt",
        "payment",
        "techniques",
        "recognition",
        "non_aggression",
        "defence",
        "consultation",
        "border_guarantee",
        "demilitarized",
        "arms_limit",
        "base",
        "passage",
        "occupation",
        "tariff_cap",
        "tariff_preference",
        "customs_union",
        "currency_peg",
        "monetary_union",
        "capital_protection",
        "financial_control",
        "tribute",
        "foreign_policy",
        "intervention",
        "minorities",
        "troops_quota",
    ];
}

// ---------------------------------------------------------------------------------------------
// The catalogue
// ---------------------------------------------------------------------------------------------

/// A part of the simulation a clause needs. A clause whose front is not live is dormant: it is
/// signed, shown, and has no effect, and the screen says which front wakes it (`front-<key>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Front {
    /// The border operations (borders part B).
    Borders,
    /// The ledger of debts between states, in the weekly tick.
    Ledger,
    /// Recognition between states.
    Recognition,
    /// The tariff inside the world trade.
    Trade,
    /// The monetary regime driving the exchange rate.
    Money,
    /// Forces, crises and war (phase 5).
    War,
    /// Communities inside the pops.
    Communities,
    /// Foreign capital in the tick.
    ForeignCapital,
    /// Research with its transfers.
    Research,
    /// The interest groups and the laws (already live).
    Politics,
}

impl Front {
    pub const ALL: [Front; 10] = [
        Front::Borders,
        Front::Ledger,
        Front::Recognition,
        Front::Trade,
        Front::Money,
        Front::War,
        Front::Communities,
        Front::ForeignCapital,
        Front::Research,
        Front::Politics,
    ];

    /// `front-<key>` in the Fluent files: why the clause sleeps and what wakes it.
    pub fn key(self) -> &'static str {
        match self {
            Front::Borders => "borders",
            Front::Ledger => "ledger",
            Front::Recognition => "recognition",
            Front::Trade => "trade",
            Front::Money => "money",
            Front::War => "war",
            Front::Communities => "communities",
            Front::ForeignCapital => "foreign_capital",
            Front::Research => "research",
            Front::Politics => "politics",
        }
    }
}

/// The fronts live in this build. Grows as each front enters the tick; a clause reads it to
/// know whether it acts ([`ClauseCatalogue::dormant`]).
pub const LIVE_FRONTS: &[Front] = &[
    Front::Politics,
    Front::Ledger,
    Front::Borders,
    Front::Recognition,
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClauseCatalogue {
    pub clauses: Vec<ClauseDef>,
    /// Subject grades as bundles of clauses, from the overlord's point of view (the subject
    /// gives, the overlord receives, unless the line says `overlord_gives`).
    pub bundles: Vec<Bundle>,
    pub policy: Policy,
    /// Weights that turn the unpriced terms into reference units, shown on the screen.
    pub weights: Weights,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClauseDef {
    pub key: String,
    /// Acts once on entry into force (`true`) or holds while the treaty is in force.
    pub once: bool,
    /// Fronts the clause needs to act; empty: always acts.
    pub needs: Vec<Front>,
    /// Points of sovereignty the giver yields (design choice; the screen shows them).
    #[serde(default)]
    pub sovereignty: u16,
    /// What the interest groups weigh: effect key of `politics::EFFECTS` or one of the two new
    /// ones (`sovereignty`, `burden`), with the weight for the giver, −3 to 3.
    #[serde(default)]
    pub concerns: Vec<(String, i8)>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub key: String,
    /// The subject grades of `countries.ron` this bundle stands for.
    pub grades: Vec<SubjectKind>,
    pub clauses: Vec<BundleLine>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleLine {
    pub clause: String,
    /// The overlord gives (defence of a dominion); by default the subject gives.
    #[serde(default)]
    pub overlord_gives: bool,
}

/// The owner's switches (decisions 3 to 6 of the design), with the recommended defaults.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Policy {
    pub subject_exit: SubjectExit,
    pub recognition_default: RecognitionDefault,
    pub ratify_below_threshold: RatifyBelow,
    /// The war debts of 1925 (`war_debts.ron`) enter the ledger on the start date.
    pub war_debts_1925: bool,
    /// Each line keeps what was paid and written off, and a line that ends stays closed in the
    /// ledger instead of leaving it. Off by default while it is judged (the owner's rule of
    /// 2026-11-17: a new rule enters off, with the hash of the main).
    #[serde(default)]
    pub ledger_record: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectExit {
    /// Before war exists, a denunciation holds only when the pressure to leave passes the
    /// overlord's control (recommended).
    PressureOverControl,
    /// Any denunciation holds.
    Free,
    /// Only by negotiation.
    NegotiatedOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognitionDefault {
    /// Everyone recognises everyone in 1925 except the listed pairs; a new country is recognised
    /// only by the signatories of the treaty that made it (recommended).
    AllExceptListed,
    /// Everyone recognises everyone, always.
    Everyone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatifyBelow {
    /// The player may ratify below the threshold paying legitimacy; the AI never does
    /// (recommended).
    PlayerOnly,
    /// Nobody ratifies below the threshold.
    Nobody,
    /// Ratification is automatic.
    Automatic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Weights {
    /// Value of one point of sovereignty, as a share of the giver's yearly GDP.
    pub sovereignty_point: Fixed,
    /// Years over which a continuous clause without a term is valued.
    pub horizon_years: u16,
    /// Yearly decay of the breach memory.
    pub reputation_decay: Fixed,
}

impl ClauseCatalogue {
    pub fn get(&self, key: &str) -> Option<&ClauseDef> {
        self.clauses.iter().find(|c| c.key == key)
    }

    pub fn bundle(&self, key: &str) -> Option<&Bundle> {
        self.bundles.iter().find(|b| b.key == key)
    }

    /// The bundle that stands for a subject grade of `countries.ron`.
    pub fn bundle_for(&self, grade: &SubjectKind) -> Option<&Bundle> {
        self.bundles.iter().find(|b| b.grades.contains(grade))
    }

    /// The first front a clause needs that is not live: why it sleeps. `None`: it acts.
    pub fn dormant(&self, key: &str, live: &[Front]) -> Option<Front> {
        self.get(key)?
            .needs
            .iter()
            .copied()
            .find(|f| !live.contains(f))
    }

    /// Checks the catalogue: one row per clause kind and no other, a source on each, weights in
    /// range, bundles naming known clauses, every subject grade in exactly one bundle.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut seen = BTreeSet::new();
        for c in &self.clauses {
            if !seen.insert(c.key.as_str()) {
                errors.push(format!("clause {}: listed twice", c.key));
            }
            if !ClauseKind::KEYS.contains(&c.key.as_str()) {
                errors.push(format!("clause {}: no such clause kind", c.key));
            }
            if c.source.trim().is_empty() {
                errors.push(format!("clause {}: no source", c.key));
            }
            for (effect, w) in &c.concerns {
                if !(-3..=3).contains(w) {
                    errors.push(format!(
                        "clause {}: weight {w} of {effect} out of -3..3",
                        c.key
                    ));
                }
                if !is_concern(effect) {
                    errors.push(format!("clause {}: unknown concern {effect}", c.key));
                }
            }
        }
        for k in ClauseKind::KEYS {
            if !seen.contains(k) {
                errors.push(format!("clause kind {k}: no catalogue row"));
            }
        }
        let mut grades: BTreeMap<String, &str> = BTreeMap::new();
        for b in &self.bundles {
            if b.source.trim().is_empty() {
                errors.push(format!("bundle {}: no source", b.key));
            }
            for l in &b.clauses {
                match self.get(&l.clause) {
                    None => errors.push(format!("bundle {}: unknown clause {}", b.key, l.clause)),
                    Some(d) if d.once => {
                        errors.push(format!("bundle {}: {} acts once", b.key, l.clause))
                    }
                    _ => {}
                }
            }
            for g in &b.grades {
                if let Some(other) = grades.insert(format!("{g:?}"), &b.key) {
                    errors.push(format!("grade {g:?} in bundles {other} and {}", b.key));
                }
            }
        }
        for g in [
            SubjectKind::Dominion,
            SubjectKind::Mandate,
            SubjectKind::Protectorate,
            SubjectKind::Satellite,
            SubjectKind::CrownColony,
        ] {
            if !grades.contains_key(&format!("{g:?}")) {
                errors.push(format!("grade {g:?}: no bundle"));
            }
        }
        if self.weights.horizon_years == 0 || self.weights.horizon_years > 99 {
            errors.push("weights: horizon_years out of 1..99".into());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// The two concerns treaties add to the effects the interest groups already weigh.
pub const NEW_CONCERNS: [&str; 2] = ["sovereignty", "burden"];

fn is_concern(key: &str) -> bool {
    NEW_CONCERNS.contains(&key) || crate::politics::EFFECTS.iter().any(|e| e.key == key)
}

// ---------------------------------------------------------------------------------------------
// Proposal and evaluation
// ---------------------------------------------------------------------------------------------

/// An agreement on the table: two sides, the clauses each gives, the term.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    /// Side A proposes; side B answers.
    pub side_a: Vec<Tag>,
    pub side_b: Vec<Tag>,
    /// Clauses, with `Side::Senior` read as side A and `Side::Junior` as side B.
    pub clauses: Vec<Clause>,
    /// Years; `None`: indefinite.
    pub term_years: Option<u16>,
    /// Years of notice to denounce.
    pub notice_years: u16,
}

/// The named terms of a valuation, `term-<key>` in the Fluent files.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValueTerm {
    /// Present value of the payments (debt, tribute, payment).
    Money,
    /// Tariff revenue and trade surplus.
    Trade,
    /// Value added and people of the states that change hands.
    Territory,
    /// Defence promised against the measured threat.
    Security,
    /// Clauses that yield foreign policy, bases, intervention.
    Sovereignty,
    /// Opposition of the interest groups at home, in legitimacy.
    Politics,
    /// Discount on what the other side promises for later, by its breaches.
    Trust,
}

impl ValueTerm {
    pub const ALL: [ValueTerm; 7] = [
        ValueTerm::Money,
        ValueTerm::Trade,
        ValueTerm::Territory,
        ValueTerm::Security,
        ValueTerm::Sovereignty,
        ValueTerm::Politics,
        ValueTerm::Trust,
    ];

    pub fn key(self) -> &'static str {
        match self {
            ValueTerm::Money => "money",
            ValueTerm::Trade => "trade",
            ValueTerm::Territory => "territory",
            ValueTerm::Security => "security",
            ValueTerm::Sovereignty => "sovereignty",
            ValueTerm::Politics => "politics",
            ValueTerm::Trust => "trust",
        }
    }
}

/// One line of the account: a term of one clause, for one party, in reference units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueLine {
    pub clause: usize,
    pub term: ValueTerm,
    pub value: Fixed,
    /// The clause is dormant: valued at zero, and this front would wake it.
    pub dormant: Option<Front>,
}

/// The account of one party on a proposal: every line and the total. The screen shows the lines.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Valuation {
    pub party: Tag,
    pub lines: Vec<ValueLine>,
    pub total: Fixed,
}

impl Valuation {
    /// Sum of the lines of one term.
    pub fn term(&self, term: ValueTerm) -> Fixed {
        self.lines
            .iter()
            .filter(|l| l.term == term)
            .map(|l| l.value)
            .sum()
    }
}

/// What the evaluation reads of the world, gathered by the caller.
#[derive(Clone, Debug)]
pub struct EvalInputs<'a> {
    /// The world rate used to discount, as a yearly fraction.
    pub discount: Fixed,
    /// Yearly GDP of each party, reference units.
    pub gdp: &'a BTreeMap<Tag, Fixed>,
    /// Breach memory of each party (0: clean).
    pub reputation: &'a BTreeMap<Tag, Fixed>,
    pub live: &'a [Front],
}

/// Present value of `years` yearly payments of `annuity` at `rate`, paid at the end of each year.
/// Integer loop: the same on every machine.
pub fn present_value(annuity: Fixed, rate: Fixed, years: u16) -> Fixed {
    let mut factor = Fixed::ONE;
    let mut total = Fixed::ZERO;
    let step = Fixed::ONE + rate.max(Fixed::ZERO);
    for _ in 0..years {
        factor = factor.mul_div(Fixed::ONE, step);
        total += annuity * factor;
    }
    total
}

fn side_members<'a>(side: &'a Side, p: &'a Proposal) -> Vec<&'a str> {
    match side {
        Side::Senior => p.side_a.iter().map(String::as_str).collect(),
        Side::Junior => p.side_b.iter().map(String::as_str).collect(),
        Side::All => p
            .side_a
            .iter()
            .chain(&p.side_b)
            .map(String::as_str)
            .collect(),
        Side::Parties(t) => t.iter().map(String::as_str).collect(),
    }
}

/// The account of `party` on `proposal`. The same function for the AI and the player: the AI
/// accepts when the total is positive (and ratification passes); the player sees every line.
///
/// Priced now: money (present value of debts, tributes and payments, split among the members of
/// the receiving and the giving side) and sovereignty (catalogue points times the weight times
/// the giver's GDP). The other terms appear as their fronts enter the tick; until then the
/// clause shows as dormant.
pub fn evaluate(
    cat: &ClauseCatalogue,
    inputs: &EvalInputs,
    proposal: &Proposal,
    party: &str,
) -> Valuation {
    let mut lines = Vec::new();
    let horizon = proposal
        .term_years
        .unwrap_or(cat.weights.horizon_years)
        .min(cat.weights.horizon_years);
    for (ci, c) in proposal.clauses.iter().enumerate() {
        let givers = side_members(&c.giver, proposal);
        let receivers = side_members(&c.receiver, proposal);
        let gives = givers.contains(&party);
        let receives = receivers.contains(&party);
        if !gives && !receives {
            continue;
        }
        let key = c.kind.key();
        let dormant = cat.dormant(key, inputs.live);
        // Money: what flows from the giving side to the receiving side, split evenly.
        let flow = match &c.kind {
            ClauseKind::Debt { terms: Some(t) } => Some(present_value(
                t.annuity,
                inputs.discount,
                t.years.saturating_sub(t.grace_years),
            )),
            ClauseKind::Payment { amount } => Some(*amount),
            ClauseKind::Tribute { yearly } => {
                Some(present_value(*yearly, inputs.discount, horizon))
            }
            _ => None,
        };
        if let Some(pv) = flow {
            let mut value = Fixed::ZERO;
            if receives && !receivers.is_empty() {
                // Promises paid later are discounted by the payer's breaches.
                let trust = givers
                    .iter()
                    .map(|g| inputs.reputation.get(*g).copied().unwrap_or_default())
                    .max()
                    .unwrap_or_default();
                let kept = (Fixed::ONE - trust).clamp(Fixed::ZERO, Fixed::ONE);
                let share = pv.mul_ratio(1, receivers.len() as i64);
                value += share;
                if kept < Fixed::ONE {
                    lines.push(ValueLine {
                        clause: ci,
                        term: ValueTerm::Trust,
                        value: -(share - share * kept),
                        dormant,
                    });
                }
            }
            if gives && !givers.is_empty() {
                value -= pv.mul_ratio(1, givers.len() as i64);
            }
            lines.push(ValueLine {
                clause: ci,
                term: ValueTerm::Money,
                value: if dormant.is_some() {
                    Fixed::ZERO
                } else {
                    value
                },
                dormant,
            });
        }
        // Sovereignty: the giver yields points of its GDP.
        let points = cat.get(key).map(|d| d.sovereignty).unwrap_or(0);
        if points > 0 && gives {
            let gdp = inputs.gdp.get(party).copied().unwrap_or_default();
            let value = -(gdp * cat.weights.sovereignty_point * Fixed::from_int(points as i64));
            lines.push(ValueLine {
                clause: ci,
                term: ValueTerm::Sovereignty,
                value: if dormant.is_some() {
                    Fixed::ZERO
                } else {
                    value
                },
                dormant,
            });
        }
    }
    let total = lines.iter().map(|l| l.value).sum();
    Valuation {
        party: party.to_string(),
        lines,
        total,
    }
}

// ---------------------------------------------------------------------------------------------
// The ledger of debts between states
// ---------------------------------------------------------------------------------------------

/// One debt between two governments. It is at once the debtor's debt and the creditor's claim:
/// every change touches it once, so the two never differ.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterstateDebt {
    pub id: u32,
    /// Country indices (the scenario order, stable).
    pub debtor: u16,
    pub creditor: u16,
    /// Key of the treaty that made or last revised it.
    pub treaty: String,
    pub currency: Currency,
    /// Outstanding principal, reference units.
    pub principal: Fixed,
    pub rate: Fixed,
    /// Yearly payment due.
    pub annuity: Fixed,
    pub years_left: u16,
    pub deferrable: Fixed,
    pub transfer_cap: Option<Fixed>,
    pub pledge: Option<Pledge>,
    /// Nominal part, outside the sums.
    pub contingent: Fixed,
    /// Service due and not paid.
    pub arrears: Fixed,
    /// Year until which the service is suspended (moratorium); 0: none.
    pub suspended_until: i32,
    /// Service paid since the line was written (since the start date for the lines of 1925),
    /// reference units, interest and principal together.
    pub paid: Fixed,
    /// Written off since the line was written: defaults, forgiveness and repudiation.
    pub written_off: Fixed,
    /// Open, or closed and kept for the record.
    pub status: Obligation,
}

/// Where a line of the ledger stands. Open lines are `Ledger::debts`: only they owe, are
/// served, count in the sums and can be cut. A closed line is in `Ledger::closed`, the record of
/// what happened (`paid`, `written_off`).
/// Arrears and moratorium are not states: they are read off the open line's fields.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Obligation {
    #[default]
    Open,
    /// Paid off, or the term ran out with nothing in arrears.
    Closed { year: i32 },
    /// The debtor denounced the clause: the whole line was written off at once, a breach
    /// with that creditor (`docs/treaties.pt-BR.md`, section 5, "repúdio").
    Repudiated { year: i32 },
}

impl Obligation {
    fn hash_code(self) -> (u8, i32) {
        match self {
            Obligation::Open => (0, 0),
            Obligation::Closed { year } => (1, year),
            Obligation::Repudiated { year } => (2, year),
        }
    }
}

impl InterstateDebt {
    pub fn is_open(&self) -> bool {
        self.status == Obligation::Open
    }

    /// A new open line, nothing paid nor in arrears (the id is the ledger's).
    pub fn new(debtor: u16, creditor: u16, treaty: String, terms: &DebtTerms) -> Self {
        InterstateDebt {
            id: 0,
            debtor,
            creditor,
            treaty,
            currency: terms.currency,
            principal: terms.principal,
            rate: terms.rate,
            annuity: terms.annuity,
            years_left: terms.years,
            deferrable: terms.deferrable,
            transfer_cap: terms.transfer_cap,
            pledge: terms.pledge,
            contingent: terms.contingent,
            arrears: Fixed::ZERO,
            suspended_until: 0,
            paid: Fixed::ZERO,
            written_off: Fixed::ZERO,
            status: Obligation::Open,
        }
    }
}

/// A line as saves of format 8 and before wrote it: no record of what was paid or written off,
/// and a line that ended was dropped (so every line read is open).
#[derive(Serialize, Deserialize)]
pub(crate) struct DebtBeforeObligations {
    id: u32,
    debtor: u16,
    creditor: u16,
    treaty: String,
    currency: Currency,
    principal: Fixed,
    rate: Fixed,
    annuity: Fixed,
    years_left: u16,
    deferrable: Fixed,
    transfer_cap: Option<Fixed>,
    pledge: Option<Pledge>,
    contingent: Fixed,
    arrears: Fixed,
    suspended_until: i32,
}

/// The ledger of format 8 and before.
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct LedgerBeforeObligations {
    debts: Vec<DebtBeforeObligations>,
    next_id: u32,
    year: i32,
}

impl From<LedgerBeforeObligations> for Ledger {
    fn from(l: LedgerBeforeObligations) -> Self {
        Ledger {
            debts: l
                .debts
                .into_iter()
                .map(|d| InterstateDebt {
                    id: d.id,
                    debtor: d.debtor,
                    creditor: d.creditor,
                    treaty: d.treaty,
                    currency: d.currency,
                    principal: d.principal,
                    rate: d.rate,
                    annuity: d.annuity,
                    years_left: d.years_left,
                    deferrable: d.deferrable,
                    transfer_cap: d.transfer_cap,
                    pledge: d.pledge,
                    contingent: d.contingent,
                    arrears: d.arrears,
                    suspended_until: d.suspended_until,
                    paid: Fixed::ZERO,
                    written_off: Fixed::ZERO,
                    status: Obligation::Open,
                })
                .collect(),
            next_id: l.next_id,
            year: l.year,
            record: false,
            closed: Vec::new(),
        }
    }
}

/// Reads a ledger written in the layout of formats 14 and before.
pub(crate) fn ledger_before_obligations<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Ledger, D::Error> {
    LedgerBeforeObligations::deserialize(d).map(Into::into)
}

impl LedgerBeforeObligations {
    /// The old layout of an open ledger (for the tests that write old saves).
    #[cfg(test)]
    pub(crate) fn of(l: &Ledger) -> Self {
        LedgerBeforeObligations {
            debts: l
                .debts
                .iter()
                .map(|d| DebtBeforeObligations {
                    id: d.id,
                    debtor: d.debtor,
                    creditor: d.creditor,
                    treaty: d.treaty.clone(),
                    currency: d.currency,
                    principal: d.principal,
                    rate: d.rate,
                    annuity: d.annuity,
                    years_left: d.years_left,
                    deferrable: d.deferrable,
                    transfer_cap: d.transfer_cap,
                    pledge: d.pledge,
                    contingent: d.contingent,
                    arrears: d.arrears,
                    suspended_until: d.suspended_until,
                })
                .collect(),
            next_id: l.next_id,
            year: l.year,
        }
    }
}

/// The ledger: `GameWorld::treaties.debts` once in the tick.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    pub debts: Vec<InterstateDebt>,
    pub next_id: u32,
    /// Year of the last service: on a new year the lines lose a year of their term.
    #[serde(default)]
    pub year: i32,
    /// The lines keep their record (`Policy::ledger_record`): what was paid and written off,
    /// and a closed line moves to `closed`. Off, a line that ends leaves the ledger and nothing
    /// is recorded.
    #[serde(default)]
    pub record: bool,
    /// Lines that ended (paid off, out of term, repudiated), in order of closing: the record.
    /// Kept apart so that the weekly service walks only the open lines (`debts`), however long
    /// the game.
    #[serde(default)]
    pub closed: Vec<InterstateDebt>,
}

impl HashState for InterstateDebt {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u32(self.id);
        h.write_u16(self.debtor);
        h.write_u16(self.creditor);
        h.write_str(&self.treaty);
        h.write_u8(self.currency as u8);
        self.principal.hash_state(h);
        self.rate.hash_state(h);
        self.annuity.hash_state(h);
        h.write_u16(self.years_left);
        self.deferrable.hash_state(h);
        self.transfer_cap.hash_state(h);
        h.write_u8(self.pledge.map_or(0, |p| p as u8 + 1));
        self.contingent.hash_state(h);
        self.arrears.hash_state(h);
        h.write_i64(i64::from(self.suspended_until));
        // The record enters only where there is one: off, a line hashes as it did.
        if self.paid != Fixed::ZERO || self.written_off != Fixed::ZERO || !self.is_open() {
            self.paid.hash_state(h);
            self.written_off.hash_state(h);
            let (k, y) = self.status.hash_code();
            h.write_u8(k);
            h.write_i64(i64::from(y));
        }
    }
}

impl HashState for Ledger {
    fn hash_state(&self, h: &mut StateHasher) {
        self.debts.hash_state(h);
        h.write_u32(self.next_id);
        h.write_i64(i64::from(self.year));
        if self.record || !self.closed.is_empty() {
            h.write_u8(u8::from(self.record));
            self.closed.hash_state(h);
        }
    }
}

/// One week's service of one line: paid (moved between treasuries) and deferred to arrears.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Service {
    pub paid: Fixed,
    pub deferred: Fixed,
}

impl Ledger {
    pub fn is_empty(&self) -> bool {
        self.debts.is_empty()
    }

    /// Adds a debt, returns its id.
    pub fn create(&mut self, mut debt: InterstateDebt) -> u32 {
        debt.id = self.next_id;
        self.next_id += 1;
        self.debts.push(debt);
        self.next_id - 1
    }

    /// What `country` owes other governments, principal plus arrears.
    pub fn owed_by(&self, country: u16) -> Fixed {
        self.debts
            .iter()
            .filter(|d| d.debtor == country)
            .map(|d| d.principal + d.arrears)
            .sum()
    }

    /// What other governments owe `country`'s treasury (`Money::claims` of the borders design,
    /// as a sum of the ledger instead of a stored field).
    pub fn claims_of(&self, country: u16) -> Fixed {
        self.debts
            .iter()
            .filter(|d| d.creditor == country)
            .map(|d| d.principal + d.arrears)
            .sum()
    }

    /// One week of service, out of the treasuries (indexed by country) into the creditors'.
    /// `exchange` is each debtor's real exchange rate over parity (1: at parity): a weak currency
    /// makes the same payment cost more of the treasury, as `Money::debt_fx_local` does, and the
    /// creditor receives what left the debtor (the market's foreign debt works the same way).
    /// `revenue_week` caps the service where the line has a transfer cap; interest that passes
    /// the cap goes to arrears. A gold debt loses `world_inflation` a year (fractions, not
    /// percent); a debt in the debtor's currency loses the debtor's inflation. Money only moves
    /// between treasuries: their sum does not change.
    pub fn service_week(
        &mut self,
        treasuries: &mut [Fixed],
        revenue_week: &[Fixed],
        exchange: &[Fixed],
        inflation: &[Fixed],
        world_inflation: Fixed,
        year: i32,
    ) -> Vec<Service> {
        let mut out = Vec::with_capacity(self.debts.len());
        let weekly = |x: Fixed| x.mul_ratio(1, 52);
        let new_year = self.year != 0 && year > self.year;
        self.year = year;
        let record = self.record;
        for d in &mut self.debts {
            let mut s = Service::default();
            if new_year {
                d.years_left = d.years_left.saturating_sub(1);
            }
            if d.suspended_until < year && d.years_left > 0 {
                let interest = weekly(d.principal * d.rate);
                let scheduled = weekly(d.annuity).max(interest).min(d.principal + interest);
                let due = scheduled + d.arrears.mul_ratio(1, 52);
                let ex = exchange[d.debtor as usize].max(Fixed::EPSILON);
                let cap = d
                    .transfer_cap
                    .map(|c| (revenue_week[d.debtor as usize] * c).mul_div(Fixed::ONE, ex))
                    .unwrap_or(Fixed::MAX);
                let paid = due.min(cap).max(Fixed::ZERO);
                s.paid = paid * ex;
                s.deferred = due - paid;
                treasuries[d.debtor as usize] -= s.paid;
                treasuries[d.creditor as usize] += s.paid;
                if record {
                    d.paid += paid;
                }
                // The schedule first (interest, then principal), then the arrears. Interest not
                // paid goes to arrears; amortization not paid stays in the principal.
                let on_schedule = paid.min(scheduled);
                let on_interest = on_schedule.min(interest);
                d.principal = (d.principal - (on_schedule - on_interest)).max(Fixed::ZERO);
                d.arrears += interest - on_interest;
                d.arrears = (d.arrears - (paid - on_schedule)).max(Fixed::ZERO);
            }
            let erosion = match d.currency {
                Currency::Reference | Currency::Creditor => world_inflation,
                Currency::Debtor => inflation[d.debtor as usize],
            }
            .max(Fixed::ZERO);
            d.principal -= weekly(d.principal * erosion);
            out.push(s);
        }
        out
    }

    /// Yearly service `debtor` owes on its lines in force, reference units (for the default
    /// limit of `econ::money`).
    pub fn yearly_service(&self, debtor: u16, year: i32) -> Fixed {
        self.debts
            .iter()
            .filter(|d| d.debtor == debtor && d.suspended_until < year && d.years_left > 0)
            .map(|d| {
                let interest = d.principal * d.rate;
                d.annuity.max(interest).min(d.principal + interest) + d.arrears
            })
            .sum()
    }

    /// Default of `debtor`: `share` of every line it owes is written off. Returns the loss of
    /// each creditor, by country.
    pub fn haircut(&mut self, debtor: u16, share: Fixed) -> BTreeMap<u16, Fixed> {
        let mut lost = BTreeMap::new();
        let record = self.record;
        for d in self.debts.iter_mut().filter(|d| d.debtor == debtor) {
            let cut = (d.principal + d.arrears) * share;
            let from_arrears = cut.min(d.arrears);
            d.arrears -= from_arrears;
            d.principal -= cut - from_arrears;
            if record {
                d.written_off += cut;
            }
            *lost.entry(d.creditor).or_insert(Fixed::ZERO) += cut;
        }
        lost
    }

    /// `annexed` joins `into`: its lines pass to `into`, as debtor and as creditor; a line that
    /// ends owed by `into` to itself disappears from both sides at once.
    pub fn on_annex(&mut self, annexed: u16, into: u16) {
        for d in &mut self.debts {
            if d.debtor == annexed {
                d.debtor = into;
            }
            if d.creditor == annexed {
                d.creditor = into;
            }
        }
        self.debts.retain(|d| d.debtor != d.creditor);
    }

    /// Lines that end in `year`: paid off, or out of years with nothing in arrears. They close
    /// and stay as the record; what principal the term left is written off (the erosion of a
    /// schedule that outran its principal, or a line served to its end).
    pub fn settle(&mut self, year: i32) {
        if !self.record {
            // Without the record, as before: a line that ends leaves both sides at once.
            self.debts.retain(|d| {
                d.principal + d.arrears > Fixed::ZERO
                    && (d.years_left > 0 || d.arrears > Fixed::ZERO)
            });
            return;
        }
        let mut i = 0;
        while i < self.debts.len() {
            let d = &self.debts[i];
            let done = d.principal + d.arrears <= Fixed::ZERO
                || (d.years_left == 0 && d.arrears <= Fixed::ZERO);
            if done {
                let mut d = self.debts.remove(i);
                d.written_off += d.principal.max(Fixed::ZERO);
                d.principal = Fixed::ZERO;
                d.status = Obligation::Closed { year };
                self.closed.push(d);
            } else {
                i += 1;
            }
        }
    }

    /// `debtor` repudiates line `id`: the whole of it is written off at once and the line closes
    /// as repudiated. Returns the creditor and its loss; `None` if the line is not `debtor`'s
    /// or not open. Both sides of the line go together, so the ledger still sums to zero.
    pub fn repudiate(&mut self, id: u32, debtor: u16, year: i32) -> Option<(u16, Fixed)> {
        let i = self
            .debts
            .iter()
            .position(|d| d.id == id && d.debtor == debtor)?;
        let mut d = self.debts.remove(i);
        let loss = d.principal + d.arrears;
        let creditor = d.creditor;
        if self.record {
            d.written_off += loss;
            d.principal = Fixed::ZERO;
            d.arrears = Fixed::ZERO;
            d.status = Obligation::Repudiated { year };
            self.closed.push(d);
        }
        Some((creditor, loss))
    }
}

// ---------------------------------------------------------------------------------------------
// Recognition
// ---------------------------------------------------------------------------------------------

/// Who does not recognise whom. Only the exceptions are kept: in 1925 nearly everyone recognises
/// everyone (owner's decision 4). A country that is not recognised is not a stored state: it is
/// the share of the world that recognises it ([`Recognition::recognised_by_share`]).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recognition {
    /// `(from, to)`: `from` does not recognise `to`. Country indices.
    pub withheld: BTreeSet<(u16, u16)>,
}

impl HashState for Recognition {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.withheld.len());
        for (a, b) in &self.withheld {
            h.write_u16(*a);
            h.write_u16(*b);
        }
    }
}

impl Recognition {
    pub fn is_empty(&self) -> bool {
        self.withheld.is_empty()
    }

    pub fn recognises(&self, from: u16, to: u16) -> bool {
        from == to || !self.withheld.contains(&(from, to))
    }

    /// Both recognise each other: the condition for any clause but recognition itself and
    /// one-off exchanges.
    pub fn mutual(&self, a: u16, b: u16) -> bool {
        self.recognises(a, b) && self.recognises(b, a)
    }

    pub fn grant(&mut self, from: u16, to: u16) {
        self.withheld.remove(&(from, to));
    }

    pub fn withhold(&mut self, from: u16, to: u16) {
        if from != to {
            self.withheld.insert((from, to));
        }
    }

    /// Share of the world's GDP, other than its own, held by the countries that recognise `to`.
    pub fn recognised_by_share(&self, to: u16, gdp: &[Fixed]) -> Fixed {
        let mut all = Fixed::ZERO;
        let mut yes = Fixed::ZERO;
        for (i, g) in gdp.iter().enumerate() {
            let i = i as u16;
            if i == to {
                continue;
            }
            all += *g;
            if self.recognises(i, to) {
                yes += *g;
            }
        }
        if all > Fixed::ZERO {
            yes / all
        } else {
            Fixed::ONE
        }
    }

    /// A country is born (`borders::Secede`): recognised only by the signatories of the treaty
    /// that made it (none when it leaves by a movement), under the decided policy.
    pub fn on_new_country(
        &mut self,
        new: u16,
        countries: u16,
        signatories: &[u16],
        policy: RecognitionDefault,
    ) {
        if policy == RecognitionDefault::Everyone {
            return;
        }
        for c in 0..countries {
            if c != new && !signatories.contains(&c) {
                self.withhold(c, new);
            }
        }
    }

    /// `annexed` is gone: the pairs that named it go with it.
    pub fn on_annex(&mut self, annexed: u16) {
        self.withheld
            .retain(|(a, b)| *a != annexed && *b != annexed);
    }
}

/// Whether a clause may be signed between two governments that do not recognise each other:
/// only recognition itself and the one-off exchanges (a payment, a cession, a debt) may.
pub fn allowed_without_recognition(kind: &ClauseKind, cat: &ClauseCatalogue) -> bool {
    matches!(kind, ClauseKind::Recognition { .. }) || cat.get(kind.key()).is_some_and(|d| d.once)
}

/// The clauses of `proposal` that recognition forbids: `(clause, giver, receiver)` for each pair
/// of a giver and a receiver that do not recognise each other. `index` turns a tag into the
/// country index.
pub fn recognition_conflicts(
    proposal: &Proposal,
    rec: &Recognition,
    cat: &ClauseCatalogue,
    index: impl Fn(&str) -> Option<u16>,
) -> Vec<(usize, Tag, Tag)> {
    let mut out = Vec::new();
    for (ci, c) in proposal.clauses.iter().enumerate() {
        if allowed_without_recognition(&c.kind, cat) {
            continue;
        }
        for g in side_members(&c.giver, proposal) {
            for r in side_members(&c.receiver, proposal) {
                if g == r {
                    continue;
                }
                if let (Some(a), Some(b)) = (index(g), index(r))
                    && !rec.mutual(a, b)
                {
                    out.push((ci, g.to_string(), r.to_string()));
                }
            }
        }
    }
    out
}

/// Who did not recognise whom on the start date, `data/scenario/1925/recognition.ron`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecognitionFile {
    pub start_date: String,
    pub withheld: Vec<Withheld>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Withheld {
    /// The country not recognised.
    pub to: Tag,
    /// Who withholds recognition: these countries, or everyone but `except`.
    pub by: WithheldBy,
    /// When the history granted it: a reference for the scorecard, never a script.
    #[serde(default)]
    pub until: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WithheldBy {
    Countries(Vec<Tag>),
    AllExcept(Vec<Tag>),
}

impl RecognitionFile {
    /// The recognition state on the start date.
    pub fn start(
        &self,
        countries: &crate::scenario::CountriesFile,
        policy: &Policy,
    ) -> Recognition {
        let mut rec = Recognition::default();
        if policy.recognition_default == RecognitionDefault::Everyone {
            return rec;
        }
        let index = |t: &str| {
            countries
                .countries
                .iter()
                .position(|c| c.tag == t)
                .map(|i| i as u16)
        };
        for w in &self.withheld {
            let Some(to) = index(&w.to) else { continue };
            match &w.by {
                WithheldBy::Countries(list) => {
                    for f in list.iter().filter_map(|t| index(t)) {
                        rec.withhold(f, to);
                    }
                }
                WithheldBy::AllExcept(except) => {
                    for (i, c) in countries.countries.iter().enumerate() {
                        if !except.contains(&c.tag) {
                            rec.withhold(i as u16, to);
                        }
                    }
                }
            }
        }
        rec
    }

    pub fn validate(&self, countries: &crate::scenario::CountriesFile) -> Result<(), Vec<String>> {
        let tags: BTreeSet<&str> = countries.countries.iter().map(|c| c.tag.as_str()).collect();
        let mut errors = Vec::new();
        for w in &self.withheld {
            let at = format!("recognition of {}", w.to);
            let named = match &w.by {
                WithheldBy::Countries(l) | WithheldBy::AllExcept(l) => l,
            };
            for t in std::iter::once(&w.to).chain(named) {
                if !tags.contains(t.as_str()) {
                    errors.push(format!("{at}: unknown tag {t}"));
                }
            }
            if matches!(&w.by, WithheldBy::Countries(l) if l.is_empty()) {
                errors.push(format!("{at}: nobody withholds"));
            }
            if w.source.trim().is_empty() {
                errors.push(format!("{at}: no source"));
            }
            if let Some(u) = &w.until
                && !crate::diplomacy::is_iso_date(u)
            {
                errors.push(format!("{at}: bad date {u}"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// The diplomatic state of the world that is not money: recognition now; agreements in force,
/// proposals and the memory of breaches as they enter the tick. `GameWorld::treaties`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatyState {
    pub recognition: Recognition,
    /// The agreements in force signed in play, in order of signature, with the clauses that
    /// hold while they are in force (`treaty_exec::queue` writes them).
    pub agreements: Vec<Agreement>,
    /// Agreements that ended, in order of ending: the record, apart from those in force.
    pub ended: Vec<Agreement>,
    pub next_agreement: u32,
}

impl TreatyState {
    pub fn is_empty(&self) -> bool {
        self.recognition.is_empty() && self.next_agreement == 0
    }

    /// Records a signed agreement; returns its id.
    pub fn sign(&mut self, mut a: Agreement) -> u32 {
        let id = self.next_agreement;
        a.id = id;
        self.next_agreement += 1;
        self.agreements.push(a);
        id
    }

    /// The term runs out in `year`: an agreement with a fixed term ends on the anniversary of
    /// its signature (no tacit renewal yet) and moves to `ended`.
    pub fn expire(&mut self, year: i32) {
        let mut i = 0;
        while i < self.agreements.len() {
            let a = &self.agreements[i];
            if a.term_years
                .is_some_and(|t| year >= a.signed + i32::from(t))
            {
                let mut a = self.agreements.remove(i);
                a.status = AgreementStatus::Ended { year };
                self.ended.push(a);
            } else {
                i += 1;
            }
        }
    }
}

/// The treaty state of format 8: recognition only.
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct TreatyStateBeforeAgreements {
    pub(crate) recognition: Recognition,
}

impl From<TreatyStateBeforeAgreements> for TreatyState {
    fn from(t: TreatyStateBeforeAgreements) -> Self {
        TreatyState {
            recognition: t.recognition,
            ..Default::default()
        }
    }
}

/// A signed agreement: its parties, its term, and what of it still holds. The one-off clauses
/// (territory, debt, payment, revision) acted on signature and are in the ledger and the border
/// record; `kept` are the clauses that hold while it is in force, read with `Senior` as
/// `side_a` and `Junior` as `side_b`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agreement {
    pub id: u32,
    /// The key its debt lines carry (`InterstateDebt::treaty`).
    pub key: String,
    pub side_a: Vec<Tag>,
    pub side_b: Vec<Tag>,
    /// Year of signature.
    pub signed: i32,
    pub term_years: Option<u16>,
    pub notice_years: u16,
    pub kept: Vec<Clause>,
    pub status: AgreementStatus,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementStatus {
    #[default]
    InForce,
    /// The term ran out, or every party denounced it.
    Ended { year: i32 },
}

impl HashState for Agreement {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u32(self.id);
        h.write_str(&self.key);
        h.write_len(self.side_a.len());
        for t in &self.side_a {
            h.write_str(t);
        }
        h.write_len(self.side_b.len());
        for t in &self.side_b {
            h.write_str(t);
        }
        h.write_i64(i64::from(self.signed));
        h.write_u16(self.term_years.map_or(0, |y| y + 1));
        h.write_u16(self.notice_years);
        // The clauses are data with many shapes: their saved bytes are their state.
        let bytes = laplace_schelling::save::encode_snapshot(&self.kept).unwrap_or_default();
        h.write_bytes(&bytes);
        match self.status {
            AgreementStatus::InForce => h.write_u8(0),
            AgreementStatus::Ended { year } => {
                h.write_u8(1);
                h.write_i64(i64::from(year));
            }
        }
    }
}

impl HashState for TreatyState {
    fn hash_state(&self, h: &mut StateHasher) {
        self.recognition.hash_state(h);
        // Worlds without agreements keep the hash of the recognition alone.
        if self.next_agreement > 0 {
            self.agreements.hash_state(h);
            self.ended.hash_state(h);
            h.write_u32(self.next_agreement);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// War debts of 1925
// ---------------------------------------------------------------------------------------------

/// The debts between governments on the start date, `data/scenario/1925/war_debts.ron`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WarDebtsFile {
    pub start_date: String,
    /// Grams of fine gold in a 1925 dollar.
    pub dollar_gold_grams: f64,
    /// Reference units (the economy's 2011 dollars at 1925 base prices) per 1925 dollar.
    pub price_factor: f64,
    pub lines: Vec<WarDebtLine>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WarDebtLine {
    /// Key of the treaty in `diplomacy.ron`.
    pub treaty: String,
    pub debtor: Tag,
    pub creditor: Tag,
    /// Amounts in the treaty's own money, with its gold content.
    pub unit: String,
    pub unit_gold_grams: f64,
    pub principal: f64,
    /// Yearly payment; `None`: the level annuity that pays the principal off in `years`.
    #[serde(default)]
    pub annuity: Option<f64>,
    #[serde(default)]
    pub contingent: f64,
    pub rate: f64,
    pub years: u16,
    /// Share of the treaty's total this creditor receives (Spa percentages), 0 to 1.
    #[serde(default = "one")]
    pub share: f64,
    #[serde(default)]
    pub pledge: Option<Pledge>,
    #[serde(default)]
    pub transfer_cap: Option<f64>,
    pub source: String,
}

fn one() -> f64 {
    1.0
}

/// The yearly payment that pays `principal` off in `years` at `rate`. Products only (no `powf`),
/// so every platform gives the same bits.
pub fn level_annuity(principal: f64, rate: f64, years: u16) -> f64 {
    if years == 0 {
        return principal;
    }
    if rate <= 0.0 {
        return principal / f64::from(years);
    }
    let mut growth = 1.0;
    for _ in 0..years {
        growth *= 1.0 + rate;
    }
    principal * rate * growth / (growth - 1.0)
}

impl WarDebtLine {
    /// The line in reference units: amounts times the creditor's share, into 1925 dollars by
    /// gold content, then into reference units by the price factor.
    pub fn terms(&self, dollar_gold_grams: f64, price_factor: f64) -> DebtTerms {
        let k = self.share * self.unit_gold_grams / dollar_gold_grams * price_factor;
        let annuity = self
            .annuity
            .unwrap_or_else(|| level_annuity(self.principal, self.rate, self.years));
        DebtTerms {
            principal: Fixed::from_f64_lossy(self.principal * k),
            rate: Fixed::from_f64_lossy(self.rate),
            annuity: Fixed::from_f64_lossy(annuity * k),
            years: self.years,
            currency: Currency::Reference,
            deferrable: Fixed::ZERO,
            transfer_cap: self.transfer_cap.map(Fixed::from_f64_lossy),
            pledge: self.pledge,
            contingent: Fixed::from_f64_lossy(self.contingent * k),
            grace_years: 0,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The 1925 treaties against the catalogue
// ---------------------------------------------------------------------------------------------

/// Places a clause names: states of the map or passages of the sea graph (canals, straits).
pub fn places(kind: &ClauseKind) -> &[String] {
    match kind {
        ClauseKind::CedeStates { states }
        | ClauseKind::ReleaseCountry { states, .. }
        | ClauseKind::Demilitarized { states }
        | ClauseKind::Occupation { states } => states,
        ClauseKind::Base { at } => at,
        _ => &[],
    }
}

/// Checks the treaties of `diplomacy.ron` against the clause catalogue, the countries and the war
/// debts: every treaty says what it does; sides name parties; a subjection carries the bundle of
/// its grade; a treaty's historical kind has the clause that makes it that kind; every debt
/// clause in force on the start date has its lines, and every line its treaty.
pub fn validate_treaties(
    diplomacy: &crate::diplomacy::DiplomacyFile,
    cat: &ClauseCatalogue,
    countries: &crate::scenario::CountriesFile,
    debts: &WarDebtsFile,
) -> Result<(), Vec<String>> {
    use crate::diplomacy::TreatyKind as K;
    use crate::scenario::Status;
    let mut errors = Vec::new();
    let grade_of: BTreeMap<&str, &SubjectKind> = countries
        .countries
        .iter()
        .filter_map(|c| match &c.status {
            Status::Subject { kind, .. } => Some((c.tag.as_str(), kind)),
            Status::Sovereign => None,
        })
        .collect();
    let tags: BTreeSet<&str> = countries.countries.iter().map(|c| c.tag.as_str()).collect();
    for t in &diplomacy.treaties {
        let at = format!("treaty {}", t.key);
        let parties: BTreeSet<&str> = t.parties.iter().map(String::as_str).collect();
        match (&t.bundle, t.clauses.is_empty()) {
            (Some(_), false) => errors.push(format!("{at}: both a bundle and clauses")),
            (None, true) => errors.push(format!("{at}: says nothing it does (no clause)")),
            _ => {}
        }
        if t.kind == K::Subjection {
            let junior = t.parties.iter().find(|p| !t.senior.contains(p));
            let wanted = junior
                .and_then(|j| grade_of.get(j.as_str()))
                .and_then(|g| cat.bundle_for(g))
                .map(|b| b.key.as_str());
            if t.bundle.as_deref() != wanted {
                errors.push(format!(
                    "{at}: bundle {:?}, the grade in countries.ron wants {wanted:?}",
                    t.bundle
                ));
            }
        } else if let Some(b) = &t.bundle {
            errors.push(format!(
                "{at}: bundle {b} on a treaty that is not a subjection"
            ));
        }
        if let Some(b) = &t.bundle
            && cat.bundle(b).is_none()
        {
            errors.push(format!("{at}: unknown bundle {b}"));
        }
        let check_side = |side: &Side, errors: &mut Vec<String>| match side {
            Side::Senior | Side::Junior if t.senior.is_empty() => {
                errors.push(format!("{at}: Senior/Junior on an even treaty"))
            }
            Side::Parties(p) if p.is_empty() => errors.push(format!("{at}: empty side")),
            Side::Parties(p) => {
                for x in p.iter().filter(|x| !parties.contains(x.as_str())) {
                    errors.push(format!("{at}: {x} is not a party"));
                }
            }
            _ => {}
        };
        for c in &t.clauses {
            check_side(&c.giver, &mut errors);
            check_side(&c.receiver, &mut errors);
            // A mutual clause among some parties names them on both sides; a single party or
            // one whole side given to itself is a mistake.
            let mutual = matches!(&c.giver, Side::All)
                || matches!(&c.giver, Side::Parties(p) if p.len() > 1);
            if c.giver == c.receiver && !mutual {
                errors.push(format!("{at}: {} given to its own giver", c.kind.key()));
            }
            if let Some(u) = &c.until
                && (!crate::diplomacy::is_iso_date(u) || *u <= t.in_force)
            {
                errors.push(format!("{at}: {} ends at a bad date {u}", c.kind.key()));
            }
        }
        let has = |key: &str| t.clauses.iter().any(|c| c.kind.key() == key);
        let wanted: &[&str] = match t.kind {
            K::DefensiveAlliance => &["defence"],
            K::ArmsLimitation => &["arms_limit"],
            K::Guarantee => &["border_guarantee"],
            K::Reparations | K::DebtSettlement => &["debt"],
            K::Occupation => &["occupation"],
            K::CustomsUnion => &["customs_union"],
            K::Consultation => &["consultation"],
            _ => &[],
        };
        for w in wanted {
            if !has(w) {
                errors.push(format!(
                    "{at}: a {:?} treaty without the clause {w}",
                    t.kind
                ));
            }
        }
        if t.kind == K::Protection
            && ![
                "foreign_policy",
                "intervention",
                "base",
                "financial_control",
            ]
            .iter()
            .any(|k| has(k))
        {
            errors.push(format!("{at}: a protection without a protecting clause"));
        }
        let debt_lines = debts.lines.iter().filter(|l| l.treaty == t.key).count();
        let in_force =
            crate::diplomacy::covers(&t.in_force, t.until.as_deref(), &diplomacy.start_date);
        if has("debt") && in_force && debt_lines == 0 {
            errors.push(format!(
                "{at}: a debt in force without lines in war_debts.ron"
            ));
        }
        if debt_lines > 0 && !has("debt") {
            errors.push(format!("{at}: lines in war_debts.ron but no debt clause"));
        }
    }
    let keys: BTreeSet<&str> = diplomacy.treaties.iter().map(|t| t.key.as_str()).collect();
    let mut shares: BTreeMap<&str, f64> = BTreeMap::new();
    for (i, l) in debts.lines.iter().enumerate() {
        let at = format!("war debt {i} ({})", l.treaty);
        if !keys.contains(l.treaty.as_str()) {
            errors.push(format!("{at}: unknown treaty"));
        }
        for tag in [&l.debtor, &l.creditor] {
            if !tags.contains(tag.as_str()) {
                errors.push(format!("{at}: unknown tag {tag}"));
            }
        }
        if l.debtor == l.creditor {
            errors.push(format!("{at}: owed to itself"));
        }
        if !(l.share > 0.0 && l.share <= 1.0) {
            errors.push(format!("{at}: share {} out of (0, 1]", l.share));
        }
        *shares.entry(l.treaty.as_str()).or_default() += l.share;
        if l.years == 0 || l.principal <= 0.0 || l.unit_gold_grams <= 0.0 {
            errors.push(format!("{at}: no years, no principal or no gold content"));
        }
        if !(0.0..0.2).contains(&l.rate) {
            errors.push(format!("{at}: rate {} out of 0..20 %", l.rate));
        }
        if l.source.trim().is_empty() {
            errors.push(format!("{at}: no source"));
        }
    }
    for (t, s) in shares {
        if s > 1.0 + 1e-9 {
            errors.push(format!("war debts of {t}: shares add up to {s}"));
        }
    }
    if debts.start_date != diplomacy.start_date {
        errors.push("war_debts.ron: start_date differs from diplomacy.ron".into());
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Checks the 1925 treaties in force against recognition: a continuous clause other than
/// recognition needs the two governments to recognise each other, and a recognition clause in
/// force cannot be withheld.
pub fn validate_recognition(
    diplomacy: &crate::diplomacy::DiplomacyFile,
    cat: &ClauseCatalogue,
    rec: &Recognition,
    countries: &crate::scenario::CountriesFile,
) -> Result<(), Vec<String>> {
    let index = |t: &str| {
        countries
            .countries
            .iter()
            .position(|c| c.tag == t)
            .map(|i| i as u16)
    };
    let mut errors = Vec::new();
    for t in diplomacy.treaties_on(&diplomacy.start_date) {
        let proposal = Proposal {
            side_a: t.senior.clone(),
            side_b: t
                .parties
                .iter()
                .filter(|p| !t.senior.contains(p))
                .cloned()
                .collect(),
            clauses: t.clauses.clone(),
            term_years: None,
            notice_years: 0,
        };
        for (ci, g, r) in recognition_conflicts(&proposal, rec, cat, index) {
            errors.push(format!(
                "treaty {}: {} between {g} and {r}, who do not recognise each other",
                t.key,
                t.clauses[ci].kind.key()
            ));
        }
        for c in &t.clauses {
            if let ClauseKind::Recognition { .. } = c.kind {
                for g in side_members(&c.giver, &proposal) {
                    for r in side_members(&c.receiver, &proposal) {
                        if let (Some(a), Some(b)) = (index(g), index(r))
                            && !rec.recognises(a, b)
                        {
                            errors.push(format!(
                                "treaty {}: {g} recognises {r} by treaty but withholds it",
                                t.key
                            ));
                        }
                    }
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

/// The ledger on the start date: the war debts in force, when the switch is on. Country indices
/// are the scenario order.
pub fn ledger_1925(
    debts: &WarDebtsFile,
    policy: &Policy,
    countries: &crate::scenario::CountriesFile,
) -> Ledger {
    let mut ledger = Ledger {
        record: policy.ledger_record,
        ..Default::default()
    };
    if !policy.war_debts_1925 {
        return ledger;
    }
    let index = |tag: &str| {
        countries
            .countries
            .iter()
            .position(|c| c.tag == tag)
            .map(|i| i as u16)
    };
    for l in &debts.lines {
        let (Some(debtor), Some(creditor)) = (index(&l.debtor), index(&l.creditor)) else {
            continue;
        };
        let t = l.terms(debts.dollar_gold_grams, debts.price_factor);
        ledger.create(InterstateDebt {
            id: 0,
            debtor,
            creditor,
            treaty: l.treaty.clone(),
            currency: t.currency,
            principal: t.principal,
            rate: t.rate,
            annuity: t.annuity,
            years_left: t.years,
            deferrable: t.deferrable,
            transfer_cap: t.transfer_cap,
            pledge: t.pledge,
            contingent: t.contingent,
            arrears: Fixed::ZERO,
            suspended_until: 0,
            paid: Fixed::ZERO,
            written_off: Fixed::ZERO,
            status: Default::default(),
        });
    }
    ledger
}

#[cfg(test)]
mod tests {
    use super::*;

    fn debt(debtor: u16, creditor: u16, principal: i64, annuity: i64) -> InterstateDebt {
        InterstateDebt {
            id: 0,
            debtor,
            creditor,
            treaty: "t".into(),
            currency: Currency::Reference,
            principal: Fixed::from_int(principal),
            rate: Fixed::from_ratio(5, 100),
            annuity: Fixed::from_int(annuity),
            years_left: 30,
            deferrable: Fixed::ZERO,
            transfer_cap: None,
            pledge: None,
            contingent: Fixed::ZERO,
            arrears: Fixed::ZERO,
            suspended_until: 0,
            paid: Fixed::ZERO,
            written_off: Fixed::ZERO,
            status: Default::default(),
        }
    }

    fn balance(l: &Ledger, n: u16) -> Fixed {
        (0..n).map(|c| l.claims_of(c) - l.owed_by(c)).sum()
    }

    #[test]
    fn present_value_of_an_annuity() {
        // 100 a year for 10 years at 5 %: 772.17.
        let pv = present_value(Fixed::from_int(100), Fixed::from_ratio(5, 100), 10);
        assert!((pv.to_f64_lossy() - 772.17).abs() < 0.5, "{pv:?}");
        assert_eq!(
            present_value(Fixed::from_int(100), Fixed::ZERO, 3),
            Fixed::from_int(300)
        );
    }

    #[test]
    fn the_ledger_is_conserved_week_by_week() {
        let mut l = Ledger::default();
        l.create(debt(0, 1, 10_000, 600));
        l.create(debt(0, 2, 4_000, 300));
        let mut t = l.debts[1].clone();
        t.debtor = 2;
        t.creditor = 1;
        t.transfer_cap = Some(Fixed::from_ratio(1, 1000));
        l.create(t);
        let mut treasuries = vec![Fixed::from_int(50_000); 3];
        let revenue = vec![Fixed::from_int(2_000); 3];
        let inflation = vec![Fixed::from_ratio(10, 100); 3];
        let before: Fixed = treasuries.iter().copied().sum();
        for week in 0..520 {
            l.service_week(
                &mut treasuries,
                &revenue,
                &[Fixed::ONE; 3],
                &inflation,
                Fixed::from_ratio(2, 100),
                1925 + week / 52,
            );
            assert_eq!(treasuries.iter().copied().sum::<Fixed>(), before);
            assert_eq!(balance(&l, 3), Fixed::ZERO);
        }
        // The capped line built arrears instead of defaulting.
        assert!(l.debts[2].arrears > Fixed::ZERO);
        let lost = l.haircut(0, Fixed::from_ratio(1, 2));
        assert_eq!(lost.len(), 2);
        assert_eq!(balance(&l, 3), Fixed::ZERO);
        l.on_annex(2, 1);
        assert!(l.debts.iter().all(|d| d.debtor != d.creditor));
        assert_eq!(balance(&l, 3), Fixed::ZERO);
    }

    #[test]
    fn a_moratorium_stops_the_service() {
        let mut l = Ledger::default();
        let mut d = debt(0, 1, 1_000, 100);
        d.suspended_until = 1931;
        l.create(d);
        let mut tr = vec![Fixed::from_int(100); 2];
        let s = l.service_week(
            &mut tr,
            &[Fixed::from_int(10); 2],
            &[Fixed::ONE; 3],
            &[Fixed::ZERO; 2],
            Fixed::ZERO,
            1931,
        );
        assert_eq!(s[0].paid, Fixed::ZERO);
        assert_eq!(tr[0], Fixed::from_int(100));
        let s = l.service_week(
            &mut tr,
            &[Fixed::from_int(10); 2],
            &[Fixed::ONE; 3],
            &[Fixed::ZERO; 2],
            Fixed::ZERO,
            1932,
        );
        assert!(s[0].paid > Fixed::ZERO);
    }

    #[test]
    fn a_debt_in_the_debtors_money_erodes_with_its_inflation() {
        let mut l = Ledger::default();
        let mut d = debt(0, 1, 1_000, 0);
        d.rate = Fixed::ZERO;
        d.currency = Currency::Debtor;
        l.create(d);
        let mut tr = vec![Fixed::ZERO; 2];
        for _ in 0..52 {
            l.service_week(
                &mut tr,
                &[Fixed::ZERO; 2],
                &[Fixed::ONE; 3],
                &[Fixed::from_ratio(50, 100), Fixed::ZERO],
                Fixed::ZERO,
                1925,
            );
        }
        let p = l.debts[0].principal.to_f64_lossy();
        assert!(p < 700.0 && p > 550.0, "{p}");
        assert_eq!(l.owed_by(0), l.claims_of(1));
    }

    /// What left the debtor's treasury, at parity, is what the line records as paid; the
    /// principal at the start is what was amortized plus what is left plus what was written off.
    #[test]
    fn a_line_records_what_was_paid_and_written_off() {
        let mut l = Ledger {
            record: true,
            ..Default::default()
        };
        let mut d = debt(0, 1, 1_000, 200);
        d.years_left = 3;
        l.create(d);
        let mut tr = vec![Fixed::from_int(10_000); 2];
        let start = tr[0];
        for week in 0..(52 * 4) {
            let year = 1925 + week / 52;
            l.service_week(
                &mut tr,
                &[Fixed::from_int(10); 2],
                &[Fixed::ONE; 2],
                &[Fixed::ZERO; 2],
                Fixed::ZERO,
                year,
            );
            l.settle(year);
        }
        assert!(l.debts.is_empty());
        let d = &l.closed[0];
        assert_eq!(start - tr[0], d.paid);
        assert!(
            matches!(d.status, Obligation::Closed { .. }),
            "{:?}",
            d.status
        );
        // A closed line stays in the record, and owes nothing.
        assert_eq!(l.owed_by(0), Fixed::ZERO);
        assert_eq!(l.claims_of(1), Fixed::ZERO);
        // Nothing more moves on a closed line.
        let before = tr.clone();
        let s = l.service_week(
            &mut tr,
            &[Fixed::from_int(10); 2],
            &[Fixed::ONE; 2],
            &[Fixed::ZERO; 2],
            Fixed::ZERO,
            1930,
        );
        assert!(s.is_empty());
        assert_eq!(tr, before);
    }

    #[test]
    fn a_repudiation_writes_the_line_off_on_both_sides() {
        let mut l = Ledger {
            record: true,
            ..Default::default()
        };
        let a = l.create(debt(0, 1, 1_000, 100));
        l.create(debt(0, 2, 500, 50));
        l.debts[0].arrears = Fixed::from_int(30);
        // Only the debtor repudiates.
        assert_eq!(l.repudiate(a, 1, 1931), None);
        let (creditor, loss) = l.repudiate(a, 0, 1931).unwrap();
        assert_eq!((creditor, loss), (1, Fixed::from_int(1_030)));
        assert_eq!(l.closed[0].status, Obligation::Repudiated { year: 1931 });
        assert_eq!(l.closed[0].written_off, Fixed::from_int(1_030));
        assert_eq!(l.claims_of(1), Fixed::ZERO);
        assert_eq!(l.owed_by(0), Fixed::from_int(500));
        assert_eq!(balance(&l, 3), Fixed::ZERO);
        // Once.
        assert_eq!(l.repudiate(a, 0, 1932), None);
        // A default after it cuts only the open line.
        l.haircut(0, Fixed::from_ratio(1, 2));
        assert_eq!(l.closed[0].written_off, Fixed::from_int(1_030));
        assert_eq!(l.debts[0].written_off, Fixed::from_int(250));
        assert_eq!(l.yearly_service(0, 1932), l.debts[0].annuity);
    }

    #[test]
    fn agreements_are_recorded_in_order_and_end_with_their_term() {
        let mut t = TreatyState::default();
        assert!(t.is_empty());
        let a = |key: &str, term| Agreement {
            id: 99,
            key: key.into(),
            side_a: vec!["FRA".into()],
            side_b: vec!["GER".into()],
            signed: 1926,
            term_years: term,
            notice_years: 0,
            kept: Vec::new(),
            status: AgreementStatus::InForce,
        };
        assert_eq!(t.sign(a("x", Some(10))), 0);
        assert_eq!(t.sign(a("y", None)), 1);
        assert!(!t.is_empty());
        t.expire(1935);
        assert_eq!(t.agreements.len(), 2);
        t.expire(1936);
        assert_eq!(
            t.agreements
                .iter()
                .map(|a| a.key.as_str())
                .collect::<Vec<_>>(),
            ["y"]
        );
        assert_eq!(t.ended[0].status, AgreementStatus::Ended { year: 1936 });
        let mut h1 = StateHasher::new();
        t.hash_state(&mut h1);
        let mut h2 = StateHasher::new();
        TreatyState::default().hash_state(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }

    /// Without the record (the default), a line that ends leaves the ledger as before and the
    /// lines hash as they did.
    #[test]
    fn without_the_record_the_ledger_is_as_before() {
        let mut l = Ledger::default();
        let mut d = debt(0, 1, 1_000, 200);
        d.years_left = 3;
        let fresh = d.clone();
        l.create(d);
        let mut tr = vec![Fixed::from_int(10_000); 2];
        for week in 0..(52 * 4) {
            let year = 1925 + week / 52;
            l.service_week(
                &mut tr,
                &[Fixed::from_int(10); 2],
                &[Fixed::ONE; 2],
                &[Fixed::ZERO; 2],
                Fixed::ZERO,
                year,
            );
            if week == 10 {
                assert_eq!(l.debts[0].paid, Fixed::ZERO);
            }
            l.settle(year);
        }
        assert!(l.debts.is_empty());
        let a = l.create(fresh.clone());
        assert!(l.repudiate(a, 0, 1930).is_some());
        assert!(l.debts.is_empty() && l.closed.is_empty());
        // A line without a record hashes without the new fields.
        let hash = |d: &InterstateDebt| {
            let mut h = StateHasher::new();
            d.hash_state(&mut h);
            h.finish()
        };
        let mut recorded = fresh.clone();
        recorded.paid = Fixed::ONE;
        assert_ne!(hash(&fresh), hash(&recorded));
    }
}
