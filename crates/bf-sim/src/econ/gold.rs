//! The gold standard: each country's exchange regime and its rule (`game::Switches::gold_standard`,
//! which needs `payments_settle`; gold standard slice 3, owner's decisions PO2, PO3, PO4, PO6
//! and PO7; docs/state.pt-BR.md, "Padrão-ouro, reservas e balanço de pagamentos").
//!
//! A country floats, is on gold at a parity (the exchange rate it holds against the reference
//! unit, `money::Money::exchange_rate`'s index), or holds a parity behind exchange control. Who
//! is on gold in 1925 and at what parity is data (`data/economy/gold.ron`); after that the
//! rule moves them:
//!
//! - **Cover** ([`cover`]): the central bank's reserves (gold and foreign exchange, as the
//!   statutes of the 1920s counted them; the country's own currency held abroad,
//!   `payments::ReservesState::owed`, is not netted out), valued in local real units at the
//!   real exchange rate, over the monetary base: the money the game issued (`Money::debt_bank`,
//!   the bonds it bought, plus `Money::credit`, what it lent banks) and the rest of the 1925
//!   base (notes and sight liabilities, `GoldRules::base_1925`, held fixed in
//!   `Standing::base_other`).
//! - **The rule on gold** ([`step`], read by `money::week` as a [`Peg`]): the exchange rate
//!   holds the parity; below the legal cover the rate rises above the Taylor rule by some points
//!   per point of cover missing and the bank creates no money (nor lends the treasury); after
//!   some weeks short it withdraws money, and the existing inflation channel turns it into
//!   deflation.
//! - **Leaving** (PO3), by the rule, never by date: cover below the floor for some weeks in a
//!   row, or the main issuer of the foreign exchange the country holds leaves and the fall
//!   expected of that currency would leave the holder's cover below the legal one (contagion,
//!   as sterling in 1931). The rate goes back to the Taylor rule, the exchange rate floats to
//!   purchasing-power parity, the premium rises for some years, and a country with much debt
//!   in foreign currency holds its parity behind exchange control instead of floating.
//! - **Holders lose** (PO7, [`revalue`]): foreign exchange is worth what its exchange rate says;
//!   when a currency falls its holders' reserves fall with it and what its issuer owes abroad
//!   falls by the same amount. Nobody gains from nowhere: the world's net reserves do not move.
//! - **Returning** (PO2): a floating country goes back to gold at the exchange rate of the day
//!   when its inflation has been low and its bank has created no money beyond the rule for long
//!   enough, and its reserves cover enough.
//! - **The player** (PO6): adheres at the prewar parity or the day's rate, leaves (floating or
//!   under control), or hands the choice back to the rule; on gold, the posture "defend" holds
//!   the parity past the floor until the reserves run out, "devalue" moves the parity instead of
//!   leaving.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::Market;
use super::money::{INDEX_BASE, Money};
use super::payments::ReservesState;

/// A country's exchange regime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Regime {
    /// The exchange rate drifts to purchasing-power parity (`money::exchange_target`).
    #[default]
    Float,
    /// On gold: the exchange rate holds the parity, the cover rule sets the rate.
    Gold,
    /// The parity is held behind exchange control: no cover rule and no exit by it; what the
    /// country cannot pay stays blocked abroad in its own currency.
    Control,
}

/// How a country on gold meets a cover below the floor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Posture {
    /// Leaves after the rule's weeks below the floor.
    #[default]
    Rule,
    /// Holds the parity with the rate past the floor, until the reserves run out.
    Defend,
    /// Moves the parity (a devaluation within the standard) instead of leaving.
    Devalue,
}

/// The player's choice of regime (`GameCommand::SetGoldStandard`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoldChoice {
    /// Adhere at the country's prewar parity (`data/economy/gold.ron`); refused without one.
    Prewar,
    /// Adhere at the exchange rate of the day.
    Market,
    /// Leave gold and float.
    Float,
    /// Hold the exchange rate of the day behind exchange control.
    Control,
}

/// One country's standing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    pub regime: Regime,
    /// The exchange rate held on gold or under control (1925 = 100); after leaving, the last
    /// parity held.
    pub parity: Fixed,
    /// The prewar parity at the same index, zero when the data has none.
    pub prewar: Fixed,
    pub posture: Posture,
    /// The player chose the regime: the rule's return leaves it alone (the exit by the cover
    /// still applies).
    pub chosen: bool,
    /// The player's choice, carried out at the next economy week.
    pub pending: Option<GoldChoice>,
    /// Weeks in a row with the cover below the legal cover, and below the floor.
    pub weeks_short: u16,
    pub weeks_floor: u16,
    /// Weeks in a row of low inflation while floating.
    pub weeks_calm: u16,
    /// Points of risk premium an exit left, falling to zero.
    pub premium: Fixed,
    /// The exchange rate, and the currency reforms, when foreign exchange in this currency was
    /// last valued ([`revalue`]).
    pub valued_at: Fixed,
    pub redenominations: u16,
    /// The cover of the last week ([`cover`]), as a ratio.
    pub cover: Fixed,
    /// Points the central bank's rate stood above the Taylor rule last week to hold the parity
    /// ([`defense`]); zero off gold.
    pub defense: Fixed,
    /// The part of the 1925 monetary base (notes and the bank's sight liabilities,
    /// `GoldRules::base_1925`) the game's money does not hold (`Money::debt_bank` and `credit`):
    /// notes backed by gold and by commercial bills, reference units, fixed at the start. Zero
    /// when the data has no base.
    pub base_other: Fixed,
}

impl HashState for Standing {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u8(self.regime as u8);
        h.write_u8(self.posture as u8);
        h.write_u8(u8::from(self.chosen));
        h.write_u8(self.pending.map_or(0, |c| c as u8 + 1));
        for v in [
            self.parity,
            self.prewar,
            self.premium,
            self.valued_at,
            self.cover,
            self.defense,
            self.base_other,
        ] {
            v.hash_state(h);
        }
        h.write_u16(self.weeks_short);
        h.write_u16(self.weeks_floor);
        h.write_u16(self.weeks_calm);
        h.write_u16(self.redenominations);
    }
}

/// What happened to a country's regime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    /// Went (back) on gold, by the rule or the player.
    Join,
    /// Left gold by the cover (the rule, or a defended parity whose reserves ran out).
    Leave,
    /// Left gold because the issuer of most of its foreign exchange did.
    Contagion,
    /// Left for exchange control.
    Control,
    /// Moved its parity within the standard.
    Devalue,
    /// Left by the player's choice.
    Abandon,
}

/// A change of regime, for the record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoldEvent {
    pub year: i32,
    pub month: u8,
    pub market: u16,
    pub change: Change,
    /// The regime after it, and the parity it holds (the exchange rate of the day when it
    /// floats).
    pub regime: Regime,
    pub parity: Fixed,
}

/// `GameWorld::gold`: each country's standing (state, in the hash once the rule has run) and the
/// record of the changes (out of the hash). Empty while `Switches::gold_standard` is off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoldState {
    /// By market index.
    pub countries: Vec<Standing>,
    pub events: Vec<GoldEvent>,
}

/// [`GoldState`] as saved before the monetary base (world versions 41 and 42): each standing
/// without `base_other`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoldStateBeforeBase {
    pub countries: Vec<StandingBeforeBase>,
    pub events: Vec<GoldEvent>,
}

/// [`Standing`] before `base_other`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingBeforeBase {
    pub regime: Regime,
    pub parity: Fixed,
    pub prewar: Fixed,
    pub posture: Posture,
    pub chosen: bool,
    pub pending: Option<GoldChoice>,
    pub weeks_short: u16,
    pub weeks_floor: u16,
    pub weeks_calm: u16,
    pub premium: Fixed,
    pub valued_at: Fixed,
    pub redenominations: u16,
    pub cover: Fixed,
    pub defense: Fixed,
}

impl From<GoldStateBeforeBase> for GoldState {
    /// The old saves had no base: the cover reads the game's money alone, as it did.
    fn from(g: GoldStateBeforeBase) -> Self {
        Self {
            countries: g
                .countries
                .into_iter()
                .map(|s| Standing {
                    regime: s.regime,
                    parity: s.parity,
                    prewar: s.prewar,
                    posture: s.posture,
                    chosen: s.chosen,
                    pending: s.pending,
                    weeks_short: s.weeks_short,
                    weeks_floor: s.weeks_floor,
                    weeks_calm: s.weeks_calm,
                    premium: s.premium,
                    valued_at: s.valued_at,
                    redenominations: s.redenominations,
                    cover: s.cover,
                    defense: s.defense,
                    base_other: Fixed::ZERO,
                })
                .collect(),
            events: g.events,
        }
    }
}

impl From<&GoldState> for GoldStateBeforeBase {
    fn from(g: &GoldState) -> Self {
        Self {
            countries: g
                .countries
                .iter()
                .map(|s| StandingBeforeBase {
                    regime: s.regime,
                    parity: s.parity,
                    prewar: s.prewar,
                    posture: s.posture,
                    chosen: s.chosen,
                    pending: s.pending,
                    weeks_short: s.weeks_short,
                    weeks_floor: s.weeks_floor,
                    weeks_calm: s.weeks_calm,
                    premium: s.premium,
                    valued_at: s.valued_at,
                    redenominations: s.redenominations,
                    cover: s.cover,
                    defense: s.defense,
                })
                .collect(),
            events: g.events.clone(),
        }
    }
}

impl GoldState {
    pub fn is_empty(&self) -> bool {
        self.countries.is_empty()
    }
}

impl HashState for GoldState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.countries.len() as u64);
        for c in &self.countries {
            c.hash_state(h);
        }
    }
}

/// What the gold standard asks of a central bank this week (`money::week`). The default asks
/// nothing: the bank floats on its own rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Peg {
    /// The exchange rate to hold; `None` floats.
    pub parity: Option<Fixed>,
    /// Points above the Taylor rule.
    pub rate_add: Fixed,
    pub creation: Creation,
    /// Points added to the risk premium.
    pub premium: Fixed,
}

impl Peg {
    /// The cover rule binds the bank: no money beyond what it already issued.
    pub fn bound(&self) -> bool {
        self.creation != Creation::Rule
    }
}

/// The money a central bank on gold may create.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Creation {
    /// What its rule creates.
    #[default]
    Rule,
    /// None: the rule's purchases stop at zero, and the treasury gets no loan from the bank.
    Stop,
    /// It withdraws at least this many points of GDP a year.
    Withdraw(Fixed),
}

/// The rule's numbers and the 1925 data (`data/economy/gold.ron`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoldRules {
    /// By market: the regime and parity of 1925, the prewar parity (zero: none), the legal
    /// cover (a ratio), and whether the rule never takes it back to gold.
    pub start: Vec<(Regime, Fixed)>,
    pub prewar: Vec<Fixed>,
    pub legal_cover: Vec<Fixed>,
    pub outside: Vec<bool>,
    /// The floor, as a share of the legal cover.
    pub floor: Fixed,
    /// Points of rate per point (percent) of cover missing, and the most it adds.
    pub rate_per_point: Fixed,
    pub max_add: Fixed,
    /// Weeks short of the legal cover before money is withdrawn, and how much (points of GDP a
    /// year).
    pub withdraw_weeks: u16,
    pub withdraw: Fixed,
    /// Weeks below the floor that take a country off gold.
    pub exit_weeks: u16,
    /// Share of the reserves in one currency above which its issuer's exit can take the holder
    /// off: it does when the fall it expects of that currency (`contagion_loss`) leaves the
    /// holder's own cover below the legal one.
    pub contagion: Fixed,
    pub contagion_loss: Fixed,
    /// Debt in foreign currency over GDP above which a country leaving holds its parity behind
    /// exchange control.
    pub control_debt: Fixed,
    /// Points of premium an exit adds, and the weeks it takes to fade.
    pub exit_premium: Fixed,
    pub premium_weeks: u16,
    /// Return: inflation within this many points of zero (both now and last year) and money
    /// created beyond the rule (`Money::monetized`, points of GDP a year) at most
    /// `calm_monetized`, for this many weeks, and a cover of this multiple of the legal one.
    pub calm_inflation: Fixed,
    pub calm_monetized: Fixed,
    pub calm_weeks: u16,
    pub return_cover: Fixed,
    /// Share by which a devaluation moves the parity.
    pub devaluation: Fixed,
    /// Each market's monetary base at the end of 1925 (notes in circulation and the central
    /// bank's sight liabilities), reference units; zero where the data has none
    /// (`data/economy/reserves.ron`).
    pub base_1925: Vec<Fixed>,
}

/// The cover of one central bank (module doc): its reserves (gold and foreign exchange) at the
/// real exchange rate over the monetary base, the money the game issued (`Money::debt_bank`,
/// `credit`) plus the part of the 1925 base it does not hold (`Standing::base_other`). A bank
/// that issued nothing is fully covered with any reserve.
pub fn cover(reserves: Fixed, money: &Money, base_other: Fixed) -> Fixed {
    let local = reserves.mul_div(money.real_exchange(), INDEX_BASE);
    let issued = money.debt_bank + money.credit + base_other;
    let most = Fixed::from_int(10);
    if issued <= Fixed::ZERO {
        return if local > Fixed::ZERO {
            most
        } else {
            Fixed::ZERO
        };
    }
    (local / issued).clamp(-most, most)
}

/// The issuer of the largest foreign exchange a reserve holds, and its share of the reserve.
fn main_issuer(r: &super::payments::Reserve) -> Option<(usize, Fixed)> {
    let total = r.total();
    if total <= Fixed::ZERO {
        return None;
    }
    let (k, v) =
        r.fx.iter()
            .copied()
            .fold(None::<(u16, Fixed)>, |best, x| match best {
                Some(b) if b.1 >= x.1 => Some(b),
                _ => Some(x),
            })?;
    Some((usize::from(k), v / total))
}

fn leave(
    rules: &GoldRules,
    s: &mut Standing,
    m: &Market,
    change: Change,
    date: laplace_schelling::Date,
    mi: usize,
    events: &mut Vec<GoldEvent>,
) {
    let gdp = m.gdp_last_year;
    let control = gdp > Fixed::ZERO && m.money.debt_fx_local() > gdp * rules.control_debt;
    s.regime = if control {
        Regime::Control
    } else {
        Regime::Float
    };
    s.premium = rules.exit_premium;
    s.weeks_short = 0;
    s.weeks_floor = 0;
    s.weeks_calm = 0;
    events.push(GoldEvent {
        year: date.year,
        month: date.month,
        market: mi as u16,
        change: if control { Change::Control } else { change },
        regime: s.regime,
        parity: s.parity,
    });
}

/// The week's rule (module doc), before the central banks move (`money::week`): the cover, the
/// player's choices, exits, contagion, devaluations and returns; returns what each bank is asked.
/// `reserves` as the week's settlement left them.
pub fn step(
    rules: &GoldRules,
    state: &mut GoldState,
    reserves: &ReservesState,
    markets: &[Market],
    date: laplace_schelling::Date,
) -> Vec<Peg> {
    let n = markets.len();
    if state.is_empty() {
        state.countries = (0..n)
            .map(|mi| {
                let (regime, parity) = rules
                    .start
                    .get(mi)
                    .copied()
                    .unwrap_or((Regime::Float, INDEX_BASE));
                let money = &markets[mi].money;
                let base = rules.base_1925.get(mi).copied().unwrap_or(Fixed::ZERO);
                Standing {
                    regime,
                    parity,
                    base_other: if base > Fixed::ZERO {
                        (base - money.debt_bank - money.credit).max(Fixed::ZERO)
                    } else {
                        Fixed::ZERO
                    },
                    prewar: rules.prewar.get(mi).copied().unwrap_or(Fixed::ZERO),
                    valued_at: markets[mi].money.exchange_rate,
                    redenominations: markets[mi].money.redenominations,
                    ..Standing::default()
                }
            })
            .collect();
    }
    if state.countries.len() < n {
        state.countries.resize(
            n,
            Standing {
                parity: INDEX_BASE,
                valued_at: INDEX_BASE,
                ..Standing::default()
            },
        );
    }
    let net = reserves.net();
    let gross = |mi: usize| {
        reserves
            .markets
            .get(mi)
            .map_or(Fixed::ZERO, super::payments::Reserve::total)
    };
    let legal = |mi: usize| {
        rules
            .legal_cover
            .get(mi)
            .copied()
            .unwrap_or(Fixed::from_ratio(35, 100))
    };
    let fade = if rules.premium_weeks > 0 {
        rules.exit_premium / i64::from(rules.premium_weeks)
    } else {
        rules.exit_premium
    };
    let mut left: Vec<usize> = Vec::new();
    let events = &mut state.events;
    for (mi, (s, m)) in state.countries.iter_mut().zip(markets).enumerate() {
        s.cover = cover(gross(mi), &m.money, s.base_other);
        s.premium = (s.premium - fade).max(Fixed::ZERO);
        // The player's choice.
        if let Some(choice) = s.pending.take() {
            match choice {
                GoldChoice::Prewar | GoldChoice::Market => {
                    let parity = if choice == GoldChoice::Prewar {
                        s.prewar
                    } else {
                        m.money.exchange_rate
                    };
                    if parity > Fixed::ZERO {
                        s.regime = Regime::Gold;
                        s.parity = parity;
                        s.weeks_short = 0;
                        s.weeks_floor = 0;
                        events.push(GoldEvent {
                            year: date.year,
                            month: date.month,
                            market: mi as u16,
                            change: Change::Join,
                            regime: Regime::Gold,
                            parity,
                        });
                    }
                }
                GoldChoice::Float if s.regime != Regime::Float => {
                    leave(rules, s, m, Change::Abandon, date, mi, events);
                    // The player floats: no control by the rule.
                    if s.regime == Regime::Control {
                        s.regime = Regime::Float;
                        if let Some(e) = events.last_mut() {
                            e.change = Change::Abandon;
                            e.regime = Regime::Float;
                        }
                    }
                    left.push(mi);
                }
                GoldChoice::Control => {
                    let was_gold = s.regime == Regime::Gold;
                    s.regime = Regime::Control;
                    if s.parity <= Fixed::ZERO || !was_gold {
                        s.parity = m.money.exchange_rate;
                    }
                    events.push(GoldEvent {
                        year: date.year,
                        month: date.month,
                        market: mi as u16,
                        change: Change::Control,
                        regime: Regime::Control,
                        parity: s.parity,
                    });
                    if was_gold {
                        left.push(mi);
                    }
                }
                GoldChoice::Float => {}
            }
            continue;
        }
        match s.regime {
            Regime::Gold => {
                let legal = legal(mi);
                s.weeks_short = if s.cover < legal {
                    s.weeks_short.saturating_add(1)
                } else {
                    0
                };
                s.weeks_floor = if s.cover < legal * rules.floor {
                    s.weeks_floor.saturating_add(1)
                } else {
                    0
                };
                let gross = reserves
                    .markets
                    .get(mi)
                    .map_or(Fixed::ZERO, super::payments::Reserve::total);
                let out = match s.posture {
                    Posture::Rule => s.weeks_floor >= rules.exit_weeks,
                    Posture::Defend => s.weeks_floor >= rules.exit_weeks && gross <= Fixed::ZERO,
                    Posture::Devalue => {
                        if s.weeks_floor >= rules.exit_weeks {
                            s.parity += s.parity * rules.devaluation;
                            s.weeks_floor = 0;
                            events.push(GoldEvent {
                                year: date.year,
                                month: date.month,
                                market: mi as u16,
                                change: Change::Devalue,
                                regime: Regime::Gold,
                                parity: s.parity,
                            });
                        }
                        false
                    }
                };
                if out {
                    leave(rules, s, m, Change::Leave, date, mi, events);
                    left.push(mi);
                }
            }
            Regime::Float => {
                let calm = |x: Fixed| x.abs() <= rules.calm_inflation;
                s.weeks_calm = if calm(m.money.inflation)
                    && calm(m.money.inflation_last_year)
                    && m.money.monetized <= rules.calm_monetized
                {
                    s.weeks_calm.saturating_add(1)
                } else {
                    0
                };
                let may = !s.chosen && !rules.outside.get(mi).copied().unwrap_or(false);
                if may
                    && net.get(mi).is_some_and(|&r| r > Fixed::ZERO)
                    && s.weeks_calm >= rules.calm_weeks
                    && s.cover >= legal(mi) * rules.return_cover
                {
                    s.regime = Regime::Gold;
                    s.parity = m.money.exchange_rate;
                    s.weeks_short = 0;
                    s.weeks_floor = 0;
                    events.push(GoldEvent {
                        year: date.year,
                        month: date.month,
                        market: mi as u16,
                        change: Change::Join,
                        regime: Regime::Gold,
                        parity: s.parity,
                    });
                }
            }
            Regime::Control => {}
        }
    }
    // Contagion: a country on gold by the rule whose reserves are mostly in the currency of a
    // country that left this week leaves too, and so on down the chain.
    let mut k = 0;
    while k < left.len() {
        let issuer = left[k];
        k += 1;
        for mi in 0..n {
            let s = &state.countries[mi];
            if s.regime != Regime::Gold || s.posture != Posture::Rule {
                continue;
            }
            // The holder follows only if the fall it expects of the issuer's currency would
            // leave its own cover below the legal one; a holder with room takes the loss when
            // it comes (`revalue`) and stays.
            let hit = reserves
                .markets
                .get(mi)
                .and_then(|r| {
                    let (i, share) = main_issuer(r)?;
                    (i == issuer && share >= rules.contagion).then(|| r.total() * share)
                })
                .is_some_and(|held| {
                    let after = gross(mi) - held * rules.contagion_loss;
                    let other = state.countries[mi].base_other;
                    cover(after, &markets[mi].money, other) < legal(mi)
                });
            if hit {
                leave(
                    rules,
                    &mut state.countries[mi],
                    &markets[mi],
                    Change::Contagion,
                    date,
                    mi,
                    &mut state.events,
                );
                left.push(mi);
            }
        }
    }
    state
        .countries
        .iter_mut()
        .enumerate()
        .map(|(mi, s)| {
            s.defense = if s.regime == Regime::Gold {
                defense(rules, legal(mi), s.cover)
            } else {
                Fixed::ZERO
            };
            (mi, &*s)
        })
        .map(|(_, s)| match s.regime {
            Regime::Gold => Peg {
                parity: Some(s.parity),
                rate_add: s.defense,
                creation: if s.weeks_short >= rules.withdraw_weeks {
                    Creation::Withdraw(rules.withdraw)
                } else if s.weeks_short > 0 {
                    Creation::Stop
                } else {
                    Creation::Rule
                },
                premium: s.premium,
            },
            Regime::Control => Peg {
                parity: Some(s.parity),
                premium: s.premium,
                ..Peg::default()
            },
            Regime::Float => Peg {
                premium: s.premium,
                ..Peg::default()
            },
        })
        .collect()
}

/// Points above the Taylor rule a bank on gold sets to hold its parity: `rate_per_point` per
/// point of cover below the legal one, at most `max_add`.
pub fn defense(rules: &GoldRules, legal: Fixed, cover: Fixed) -> Fixed {
    let missing = (legal - cover).max(Fixed::ZERO) * 100;
    (missing * rules.rate_per_point).min(rules.max_add)
}

/// Why a choice of regime cannot be carried out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Adhering at the prewar parity, for a country the data gives none.
    NoPrewarParity,
}

/// What a choice of regime would mean this week, without carrying it out (the screen's cost of
/// each parity, PO6): the parity, the cover the country's reserves would give at it, the
/// points above the Taylor rule that cover asks for on gold, and how far the parity is from the
/// day's exchange rate (positive: the currency held above the market, an overvaluation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preview {
    pub parity: Fixed,
    pub cover: Fixed,
    pub legal: Fixed,
    pub floor: Fixed,
    pub defense: Fixed,
    pub gap: Fixed,
}

/// The cost of `choice` for market `mi` (see [`Preview`]); `Err` when it would be refused.
/// The cover at a parity scales the week's cover by the parity over the day's rate: the same
/// reserves are worth more local money at a weaker currency.
pub fn preview(
    rules: &GoldRules,
    standing: &Standing,
    mi: usize,
    m: &Market,
    choice: GoldChoice,
) -> Result<Preview, Refusal> {
    let today = m.money.exchange_rate.max(Fixed::EPSILON);
    let parity = match choice {
        GoldChoice::Prewar if standing.prewar <= Fixed::ZERO => {
            return Err(Refusal::NoPrewarParity);
        }
        GoldChoice::Prewar => standing.prewar,
        GoldChoice::Market | GoldChoice::Control => today,
        GoldChoice::Float => today,
    };
    let legal = rules
        .legal_cover
        .get(mi)
        .copied()
        .unwrap_or(Fixed::from_ratio(35, 100));
    let cover = standing.cover.mul_div(parity, today);
    let on_gold = matches!(choice, GoldChoice::Prewar | GoldChoice::Market);
    Ok(Preview {
        parity,
        cover,
        legal,
        floor: legal * rules.floor,
        defense: if on_gold {
            defense(rules, legal, cover)
        } else {
            Fixed::ZERO
        },
        gap: (today - parity) / today,
    })
}

/// How likely each country's devaluation looks to lenders (`econ::capital`), 0 to 1: on gold,
/// zero while the cover is at the floor times `1 + band` or above, growing linearly to one at
/// the floor. Floating and under exchange control, zero (the drift and the controls are read
/// elsewhere).
pub fn at_risk(rules: &GoldRules, state: &GoldState, band: Fixed) -> Vec<Fixed> {
    state
        .countries
        .iter()
        .enumerate()
        .map(|(mi, s)| {
            if s.regime != Regime::Gold {
                return Fixed::ZERO;
            }
            let legal = rules
                .legal_cover
                .get(mi)
                .copied()
                .unwrap_or(Fixed::from_ratio(35, 100));
            let floor = legal * rules.floor;
            let span = floor * band;
            if span <= Fixed::ZERO {
                return if s.cover < floor {
                    Fixed::ONE
                } else {
                    Fixed::ZERO
                };
            }
            ((floor + span - s.cover) / span).clamp(Fixed::ZERO, Fixed::ONE)
        })
        .collect()
}

/// After the central banks moved (`money::week`): the foreign exchange in each currency is worth
/// what its exchange rate moved since it was last valued (PO7). A currency that falls costs its
/// holders and lowers what its issuer owes abroad by the same amount, so the world's net
/// reserves do not move; one that rises the other way. A currency reform (three zeros) is no
/// fall.
pub fn revalue(state: &mut GoldState, reserves: &mut ReservesState, markets: &[Market]) {
    for (k, (s, m)) in state.countries.iter_mut().zip(markets).enumerate() {
        let now = m.money.exchange_rate;
        while s.redenominations < m.money.redenominations {
            s.valued_at = s.valued_at.mul_ratio(1, 1000);
            s.redenominations += 1;
        }
        if now <= Fixed::ZERO || s.valued_at <= Fixed::ZERO || now == s.valued_at {
            s.valued_at = now;
            continue;
        }
        let Ok(key) = u16::try_from(k) else { continue };
        for r in &mut reserves.markets {
            if let Ok(i) = r.fx.binary_search_by_key(&key, |x| x.0) {
                let v = r.fx[i].1.mul_div(s.valued_at, now);
                if v > Fixed::ZERO {
                    r.fx[i].1 = v;
                } else {
                    r.fx.remove(i);
                }
            }
        }
        s.valued_at = now;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::econ::payments::Reserve;

    pub fn rules(n: usize) -> GoldRules {
        GoldRules {
            start: vec![(Regime::Gold, INDEX_BASE); n],
            prewar: vec![Fixed::ZERO; n],
            legal_cover: vec![Fixed::from_ratio(40, 100); n],
            outside: vec![false; n],
            floor: Fixed::from_ratio(1, 2),
            rate_per_point: Fixed::from_ratio(15, 100),
            max_add: Fixed::from_int(8),
            withdraw_weeks: 13,
            withdraw: Fixed::from_int(1),
            exit_weeks: 13,
            contagion: Fixed::from_ratio(1, 4),
            contagion_loss: Fixed::from_ratio(3, 10),
            calm_monetized: Fixed::from_ratio(1, 2),
            base_1925: Vec::new(),
            control_debt: Fixed::from_ratio(1, 10),
            exit_premium: Fixed::from_int(1),
            premium_weeks: 156,
            calm_inflation: Fixed::from_int(3),
            calm_weeks: 104,
            return_cover: Fixed::ONE,
            devaluation: Fixed::from_ratio(1, 10),
        }
    }

    fn market(issued: i64) -> Market {
        let mut m = Market::empty(1);
        m.gdp_last_year = Fixed::from_int(1000);
        m.money.debt_bank = Fixed::from_int(issued);
        m
    }

    #[test]
    fn a_preview_gives_each_parity_its_cover_and_its_defense() {
        let r = rules(1);
        let mut m = market(100);
        m.money.exchange_rate = INDEX_BASE;
        let st = Standing {
            cover: Fixed::from_ratio(30, 100),
            prewar: Fixed::from_int(90),
            ..Standing::default()
        };
        // At the day's rate: the week's cover, 10 points short of 40 %, 1.5 points of defense.
        let day = preview(&r, &st, 0, &m, GoldChoice::Market).unwrap();
        assert_eq!(day.cover, Fixed::from_ratio(30, 100));
        assert_eq!(day.defense, Fixed::from_ratio(15, 10));
        assert_eq!(day.gap, Fixed::ZERO);
        assert_eq!(day.floor, Fixed::from_ratio(20, 100));
        // At the prewar parity, 10 % stronger: less cover, more defense, a 10 % overvaluation.
        let pre = preview(&r, &st, 0, &m, GoldChoice::Prewar).unwrap();
        assert_eq!(pre.cover, Fixed::from_ratio(27, 100));
        assert!(pre.defense > day.defense);
        assert_eq!(pre.gap, Fixed::from_ratio(1, 10));
        // Floating asks for no defense; no prewar parity, refused.
        assert_eq!(
            preview(&r, &st, 0, &m, GoldChoice::Float).unwrap().defense,
            Fixed::ZERO
        );
        let none = Standing {
            prewar: Fixed::ZERO,
            ..st
        };
        assert_eq!(
            preview(&r, &none, 0, &m, GoldChoice::Prewar),
            Err(Refusal::NoPrewarParity)
        );
    }

    fn reserves(gold: &[i64]) -> ReservesState {
        ReservesState {
            markets: gold
                .iter()
                .map(|&g| Reserve {
                    gold: Fixed::from_int(g),
                    fx: Vec::new(),
                })
                .collect(),
            ..ReservesState::default()
        }
    }

    #[test]
    fn below_the_cover_the_rate_rises_and_money_stops() {
        let r = rules(2);
        let mut st = GoldState::default();
        let markets = vec![market(100), market(100)];
        // Market 0 covers 50 %, market 1 covers 20 %: 20 points short.
        let res = reserves(&[50, 20]);
        let pegs = step(
            &r,
            &mut st,
            &res,
            &markets,
            laplace_schelling::Date::new(1926, 1, 4),
        );
        assert_eq!(pegs[0].rate_add, Fixed::ZERO);
        assert_eq!(pegs[0].creation, Creation::Rule);
        assert_eq!(pegs[1].rate_add, Fixed::from_int(3));
        assert_eq!(pegs[1].creation, Creation::Stop);
        assert_eq!(pegs[1].parity, Some(INDEX_BASE));
    }

    #[test]
    fn below_the_floor_for_the_rules_weeks_a_country_leaves_and_its_holders_follow() {
        let r = rules(3);
        let mut st = GoldState::default();
        let markets = vec![market(100), market(100), market(100)];
        // Market 1 covers 10 %, under the floor of 20 %; market 2 holds mostly market 1's
        // currency.
        let mut res = reserves(&[100, 10, 10]);
        res.markets[2].fx.push((1, Fixed::from_int(40)));
        let mut date = laplace_schelling::Date::new(1926, 1, 4);
        for week in 0..12 {
            step(&r, &mut st, &res, &markets, date);
            assert_eq!(st.countries[1].regime, Regime::Gold, "week {week}");
            date = laplace_schelling::Date::from_days(date.to_days() + 7);
        }
        let pegs = step(&r, &mut st, &res, &markets, date);
        assert_eq!(st.countries[1].regime, Regime::Float);
        assert_eq!(st.countries[2].regime, Regime::Float, "contagion");
        assert_eq!(st.countries[0].regime, Regime::Gold);
        assert_eq!(pegs[1].parity, None);
        assert_eq!(pegs[1].rate_add, Fixed::ZERO);
        assert_eq!(pegs[1].premium, r.exit_premium);
        let changes: Vec<Change> = st.events.iter().map(|e| e.change).collect();
        assert_eq!(changes, [Change::Leave, Change::Contagion]);
    }

    #[test]
    fn a_defended_parity_holds_until_the_reserves_run_out() {
        let r = rules(1);
        let mut st = GoldState::default();
        let markets = vec![market(100)];
        let mut res = reserves(&[5]);
        step(
            &r,
            &mut st,
            &res,
            &markets,
            laplace_schelling::Date::new(1926, 1, 4),
        );
        st.countries[0].posture = Posture::Defend;
        for _ in 0..30 {
            step(
                &r,
                &mut st,
                &res,
                &markets,
                laplace_schelling::Date::new(1926, 1, 4),
            );
        }
        assert_eq!(st.countries[0].regime, Regime::Gold);
        res.markets[0].gold = Fixed::ZERO;
        step(
            &r,
            &mut st,
            &res,
            &markets,
            laplace_schelling::Date::new(1926, 1, 4),
        );
        assert_eq!(st.countries[0].regime, Regime::Float);
    }

    #[test]
    fn holders_of_a_falling_currency_lose_what_its_issuer_no_longer_owes() {
        let mut markets = vec![market(100), market(100)];
        let mut st = GoldState::default();
        let mut res = reserves(&[100, 0]);
        res.markets[0].fx.push((1, Fixed::from_int(50)));
        step(
            &rules(2),
            &mut st,
            &res,
            &markets,
            laplace_schelling::Date::new(1926, 1, 4),
        );
        let net_before: Fixed = res.net().iter().copied().sum();
        // Market 1's currency falls a fifth: 100 → 125 units per reference unit.
        markets[1].money.exchange_rate = Fixed::from_int(125);
        revalue(&mut st, &mut res, &markets);
        assert_eq!(res.markets[0].fx, [(1, Fixed::from_int(40))]);
        assert_eq!(res.owed(1), Fixed::from_int(40));
        assert_eq!(res.net().iter().copied().sum::<Fixed>(), net_before);
        // A currency reform is no fall.
        markets[1].money.exchange_rate = Fixed::from_int(125).mul_ratio(1, 1000);
        markets[1].money.redenominations = 1;
        revalue(&mut st, &mut res, &markets);
        assert_eq!(res.markets[0].fx, [(1, Fixed::from_int(40))]);
    }

    #[test]
    fn a_holder_with_room_takes_the_loss_and_stays() {
        let r = rules(3);
        let mut st = GoldState::default();
        let markets = vec![market(100), market(100), market(100)];
        // Market 2 holds mostly market 1's currency, but so much gold besides that a fall of 30 %
        // of what it holds still leaves it above the legal cover.
        let mut res = reserves(&[100, 10, 200]);
        res.markets[2].fx.push((1, Fixed::from_int(80)));
        let mut date = laplace_schelling::Date::new(1926, 1, 4);
        for _ in 0..13 {
            step(&r, &mut st, &res, &markets, date);
            date = laplace_schelling::Date::from_days(date.to_days() + 7);
        }
        assert_eq!(st.countries[1].regime, Regime::Float);
        assert_eq!(
            st.countries[2].regime,
            Regime::Gold,
            "no contagion with room"
        );
    }

    #[test]
    fn a_calm_floating_country_with_cover_returns_at_the_days_rate() {
        let mut r = rules(1);
        r.start = vec![(Regime::Float, INDEX_BASE)];
        r.calm_weeks = 3;
        let mut st = GoldState::default();
        let mut markets = vec![market(100)];
        markets[0].money.exchange_rate = Fixed::from_int(120);
        let res = reserves(&[50]);
        for _ in 0..2 {
            step(
                &r,
                &mut st,
                &res,
                &markets,
                laplace_schelling::Date::new(1927, 1, 3),
            );
            assert_eq!(st.countries[0].regime, Regime::Float);
        }
        step(
            &r,
            &mut st,
            &res,
            &markets,
            laplace_schelling::Date::new(1927, 1, 3),
        );
        assert_eq!(st.countries[0].regime, Regime::Gold);
        assert_eq!(st.countries[0].parity, Fixed::from_int(120));
    }
}
