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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
pub const LIVE_FRONTS: &[Front] = &[Front::Politics];

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
}

/// The ledger: `GameWorld::treaties.debts` once in the tick.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    pub debts: Vec<InterstateDebt>,
    pub next_id: u32,
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
    /// `revenue_week` caps the service where the line has a transfer cap; what passes the cap
    /// goes to arrears. A gold debt loses `world_inflation` a year; a debt in the debtor's
    /// currency loses the debtor's inflation. Money only moves between treasuries: their sum does
    /// not change.
    pub fn service_week(
        &mut self,
        treasuries: &mut [Fixed],
        revenue_week: &[Fixed],
        inflation: &[Fixed],
        world_inflation: Fixed,
        year: i32,
    ) -> Vec<Service> {
        let mut out = Vec::with_capacity(self.debts.len());
        let weekly = |x: Fixed| x.mul_ratio(1, 52);
        for d in &mut self.debts {
            let mut s = Service::default();
            if d.suspended_until < year && d.years_left > 0 {
                let interest = weekly(d.principal * d.rate);
                let scheduled = weekly(d.annuity).max(interest);
                let due = scheduled + d.arrears.mul_ratio(1, 52);
                let cap = d
                    .transfer_cap
                    .map(|c| revenue_week[d.debtor as usize] * c)
                    .unwrap_or(Fixed::MAX);
                let paid = due.min(cap).max(Fixed::ZERO);
                s.paid = paid;
                s.deferred = due - paid;
                treasuries[d.debtor as usize] -= paid;
                treasuries[d.creditor as usize] += paid;
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

    /// Default of `debtor`: `share` of every line it owes is written off. Returns the loss of
    /// each creditor, by country.
    pub fn haircut(&mut self, debtor: u16, share: Fixed) -> BTreeMap<u16, Fixed> {
        let mut lost = BTreeMap::new();
        for d in self.debts.iter_mut().filter(|d| d.debtor == debtor) {
            let cut = (d.principal + d.arrears) * share;
            let from_arrears = cut.min(d.arrears);
            d.arrears -= from_arrears;
            d.principal -= cut - from_arrears;
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

    /// Lines that end: paid off, or out of years with nothing in arrears.
    pub fn settle(&mut self) {
        self.debts.retain(|d| {
            d.principal + d.arrears > Fixed::ZERO && (d.years_left > 0 || d.arrears > Fixed::ZERO)
        });
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

/// The ledger on the start date: the war debts in force, when the switch is on. Country indices
/// are the scenario order.
pub fn ledger_1925(
    debts: &WarDebtsFile,
    policy: &Policy,
    countries: &crate::scenario::CountriesFile,
) -> Ledger {
    let mut ledger = Ledger::default();
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
            &[Fixed::ZERO; 2],
            Fixed::ZERO,
            1931,
        );
        assert_eq!(s[0].paid, Fixed::ZERO);
        assert_eq!(tr[0], Fixed::from_int(100));
        let s = l.service_week(
            &mut tr,
            &[Fixed::from_int(10); 2],
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
                &[Fixed::from_ratio(50, 100), Fixed::ZERO],
                Fixed::ZERO,
                1925,
            );
        }
        let p = l.debts[0].principal.to_f64_lossy();
        assert!(p < 700.0 && p > 550.0, "{p}");
        assert_eq!(l.owed_by(0), l.claims_of(1));
    }
}
