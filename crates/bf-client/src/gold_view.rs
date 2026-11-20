//! The player's exchange regime, reserves and capital, read from the world for the money tab
//! and the Agendas (gold standard slice 5): `econ::gold`, `econ::payments`'s reserves and
//! `econ::capital`. `None` while the gold standard is off; the capital part only with
//! `capital_follows_rate`.

use bf_sim::econ::gold::{self, GoldChoice, GoldRules, Posture, Refusal, Regime};
use bf_sim::game::GameWorld;
use laplace_schelling::Fixed;

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// What a choice of regime would mean this week (`gold::preview`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preview {
    pub parity: f64,
    /// Ratios.
    pub cover: f64,
    pub legal: f64,
    pub floor: f64,
    /// Points above the Taylor rule the cover asks for on gold.
    pub defense: f64,
    /// How far the parity is from the day's exchange rate (positive: overvalued).
    pub gap: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GoldView {
    pub regime: Regime,
    /// The exchange rate held on gold or under control (1925 = 100), and the day's.
    pub parity: f64,
    pub today: f64,
    /// Zero when the data has none.
    pub prewar: f64,
    pub posture: Posture,
    /// The player chose the regime; the rule's return leaves it alone.
    pub chosen: bool,
    pub pending: Option<GoldChoice>,
    /// Ratios: the cover of last week, the legal cover and the floor.
    pub cover: f64,
    pub legal: f64,
    pub floor: f64,
    pub weeks_short: u16,
    pub weeks_floor: u16,
    /// The rule's weeks: short of the legal cover before money is withdrawn, below the floor
    /// before leaving.
    pub withdraw_weeks: u16,
    pub exit_weeks: u16,
    /// Points the rate stood above the Taylor rule to hold the parity.
    pub defense: f64,
    /// Points of risk premium an exit left.
    pub premium: f64,
    /// Reserves, reference units (dollars at base prices): gold, and foreign exchange by the
    /// issuer's tag, largest first.
    pub gold: f64,
    pub fx: Vec<(String, f64)>,
    /// The country's own currency in the other countries' reserves.
    pub owed: f64,
    /// Share by which a devaluation moves the parity.
    pub devaluation: f64,
    /// Points of GDP a year the bank withdraws once short of the legal cover long enough.
    pub withdraw: f64,
    /// The government fixed the policy rate: the cover's points do not apply.
    pub rate_fixed: bool,
    /// Each choice of regime, as `gold::preview` reads it, in the order of [`CHOICES`].
    pub choices: Vec<Result<Preview, Refusal>>,
    pub capital: Option<CapitalView>,
}

/// The choices of regime the Agendas offer, in order.
pub const CHOICES: [GoldChoice; 4] = [
    GoldChoice::Prewar,
    GoldChoice::Market,
    GoldChoice::Float,
    GoldChoice::Control,
];

#[derive(Clone, Debug, PartialEq)]
pub struct CapitalView {
    /// Controls in force this week, and the player's choice (`None`: the rule).
    pub controls: bool,
    pub chosen: Option<bool>,
    /// Claims of the country's savers abroad (long, short) and of savers abroad on it.
    pub abroad: (f64, f64),
    pub from_abroad: (f64, f64),
    /// Points of premium controls add.
    pub control_premium: f64,
}

/// Reads country `ci`; `None` while the gold standard is off.
pub fn read(
    world: &GameWorld,
    ci: usize,
    rules: Option<&GoldRules>,
    capital: Option<&bf_sim::econ::capital::CapitalRules>,
) -> Option<GoldView> {
    let rules = rules?;
    let s = world.gold.countries.get(ci)?;
    let m = world.econ.markets.get(ci)?;
    let legal = rules
        .legal_cover
        .get(ci)
        .copied()
        .unwrap_or(Fixed::from_ratio(35, 100));
    let reserve = world.reserves.markets.get(ci).cloned().unwrap_or_default();
    let tag = |mi: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(mi))
            .map_or_else(String::new, |(_, c)| c.tag.clone())
    };
    let mut fx: Vec<(String, f64)> = reserve.fx.iter().map(|&(mi, x)| (tag(mi), f(x))).collect();
    fx.sort_by(|a, b| b.1.total_cmp(&a.1));
    let choices = CHOICES
        .iter()
        .map(|&c| {
            gold::preview(rules, s, ci, m, c).map(|p| Preview {
                parity: f(p.parity),
                cover: f(p.cover),
                legal: f(p.legal),
                floor: f(p.floor),
                defense: f(p.defense),
                gap: f(p.gap),
            })
        })
        .collect();
    let capital = capital.and_then(|c| {
        let cs = &world.capital;
        if cs.is_empty() {
            return None;
        }
        let ((lo, so), (li, si)) = cs.position(ci);
        Some(CapitalView {
            controls: cs.controls.get(ci).copied().unwrap_or(false),
            chosen: cs.chosen.get(ci).copied().flatten(),
            abroad: (f(lo), f(so)),
            from_abroad: (f(li), f(si)),
            control_premium: f(c.control_premium),
        })
    });
    Some(GoldView {
        regime: s.regime,
        parity: f(s.parity),
        today: f(m.money.exchange_rate),
        prewar: f(s.prewar),
        posture: s.posture,
        chosen: s.chosen,
        pending: s.pending,
        cover: f(s.cover),
        legal: f(legal),
        floor: f(legal * rules.floor),
        weeks_short: s.weeks_short,
        weeks_floor: s.weeks_floor,
        withdraw_weeks: rules.withdraw_weeks,
        exit_weeks: rules.exit_weeks,
        defense: f(s.defense),
        premium: f(s.premium),
        gold: f(reserve.gold),
        fx,
        owed: f(world.reserves.owed(ci)),
        devaluation: f(rules.devaluation),
        withdraw: f(rules.withdraw),
        rate_fixed: m.money.rate_command.is_some(),
        choices,
        capital,
    })
}
