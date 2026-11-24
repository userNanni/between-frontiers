//! What a government fixes over the budget's historical rule (`docs/state.pt-BR.md`, "Alavancas
//! do orçamento e da dívida"). Each lever is an `Option`: `None` is the rule, and with every lever
//! of every country at `None` the world keeps the main's hash. The values are saved in
//! `GameWorld::levers`; each market carries a copy (`Market::levers`, not saved) so that the
//! functions that read one market (`public::goods_budget`, `money::deficit_room`) see them.
//!
//! First slice: the payroll (D1), public pay (D2), the treasury's reserve and surplus (D4) and the
//! deficit cap (D5). Purchases and the military share (D3) and issuance (D9) have their fields
//! here, with no command yet.

use super::milli;
use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::state::Market;

/// What the treasury does with the cash beyond twice its reserve (D4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurplusUse {
    /// The rule: a tenth of it a week goes to purchases, and to paying back debt when the debt
    /// is above 60 % of GDP.
    #[default]
    Spend,
    /// A tenth of it a week pays back debt, whatever the debt.
    Repay,
    /// It stays in the treasury: a sovereign fund.
    Keep,
}

/// One country's levers. Values in thousandths, as the commands carry them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Levers {
    /// D1: servants and clergy per thousand people (`[servants, clergy]`).
    pub payroll: Option<[u16; 2]>,
    /// D2: public pay over the profession's private wage (the rule is 700).
    pub public_pay: Option<u16>,
    /// D3: the goods budget over the rule's (1000 is the rule), and the military share of it.
    pub purchases: Option<u16>,
    pub military: Option<u16>,
    /// D4: weeks of revenue kept in reserve (the rule is 13), and what the surplus is for.
    pub reserve_weeks: Option<u16>,
    pub surplus: Option<SurplusUse>,
    /// D5: the year's deficit for purchases, in thousandths of GDP.
    pub deficit_cap: Option<u16>,
    /// D9: shares of new issues at home, abroad in the own currency and abroad in dollars.
    pub issuance: Option<[u16; 3]>,
}

/// Share of the taxes a region's servants can collect (`Switches::fiscal_capacity`, owner's
/// decision O1 of 2026-11-17): below the servants the rule asks for (`monthly::STATE_SHARE`), the
/// state collects in proportion to what it has; above, no more. Linear, as the owner put it ("a
/// arrecadação cai na proporção que falta"); the idea is Besley and Persson's fiscal capacity
/// (2009, AER 99(4)). The rule's servants are a fixed share of the people, so a country on the
/// rule collects all but the lag of hiring behind growth.
pub fn collection(r: &super::state::Region) -> Fixed {
    collection_with(r, Fixed::ONE)
}

/// [`collection`] with each servant worth `worth` servants (the offices' computers,
/// `econ::admin`): at most 1.
pub fn collection_with(r: &super::state::Region, worth: Fixed) -> Fixed {
    let share = super::monthly::STATE_SHARE
        .iter()
        .find(|s| s.0 == super::prof::CIVIL_SERVANT)
        .map_or(0, |s| s.1);
    let needed = r.population() * share / 1000;
    if needed <= 0 {
        return Fixed::ONE;
    }
    let have = r.pops[super::prof::CIVIL_SERVANT].size;
    if worth == Fixed::ONE {
        return Fixed::from_ratio(have.min(needed), needed);
    }
    (Fixed::from_ratio(have, needed) * worth).min(Fixed::ONE)
}

/// Floors and ceilings of what a command may fix. Design values for the first slice, each to be
/// replaced by a measured one with its source in the PR that switches the lever's screen on
/// (`docs/state.pt-BR.md`, the table of the commands).
pub const SERVANTS_PER_MILLE: (u16, u16) = (2, 80);
pub const CLERGY_PER_MILLE: (u16, u16) = (0, 15);
pub const PUBLIC_PAY_MILLE: (u16, u16) = (400, 1500);
pub const RESERVE_WEEKS: (u16, u16) = (4, 52);
pub const DEFICIT_CAP_MILLE: (u16, u16) = (0, 100);
/// D3: the goods budget over the rule's, and the military share of it, per mille. Design values:
/// a quarter of the rule's purchases (deep austerity) to twice them; no military goods to 80 % of
/// the goods budget (a war economy's state purchases).
pub const PURCHASES_MILLE: (u16, u16) = (250, 2000);
pub const MILITARY_MILLE: (u16, u16) = (0, 800);

/// Share of the excess of servants or clergy over a fixed payroll that leaves each month:
/// retirements, hiring freezes and lay-offs. Design value (a fortieth a month, about a quarter of
/// the excess in a year), to be calibrated on the cuts of Greece and the United Kingdom of
/// 2010–2015 in the PR that switches the payroll's screen on.
pub const PAYROLL_CUT_PER_MONTH: i64 = 40;

fn clamp((lo, hi): (u16, u16), v: u16) -> u16 {
    v.clamp(lo, hi)
}

impl Levers {
    /// Whether every lever is at its rule.
    pub fn is_rule(&self) -> bool {
        *self == Self::default()
    }

    /// The levers with every fixed value cut to its floor and ceiling.
    pub fn clamped(mut self) -> Self {
        self.payroll = self
            .payroll
            .map(|[s, c]| [clamp(SERVANTS_PER_MILLE, s), clamp(CLERGY_PER_MILLE, c)]);
        self.public_pay = self.public_pay.map(|v| clamp(PUBLIC_PAY_MILLE, v));
        self.reserve_weeks = self.reserve_weeks.map(|v| clamp(RESERVE_WEEKS, v));
        self.deficit_cap = self.deficit_cap.map(|v| clamp(DEFICIT_CAP_MILLE, v));
        self.purchases = self.purchases.map(|v| clamp(PURCHASES_MILLE, v));
        self.military = self.military.map(|v| clamp(MILITARY_MILLE, v));
        // D9: the three shares add up to 1000; all zero is the rule.
        self.issuance = self.issuance.and_then(|s| {
            let total: u32 = s.iter().map(|&v| u32::from(v)).sum();
            if total == 0 {
                return None;
            }
            let mut out = s.map(|v| (u32::from(v) * 1000 / total) as u16);
            let sum: u16 = out.iter().sum();
            out[0] += 1000 - sum;
            Some(out)
        });
        self
    }

    /// Public pay over the private wage (D2).
    pub fn public_pay(&self) -> Fixed {
        self.public_pay
            .map_or(super::public::PUBLIC_PAY, |v| milli(i64::from(v)))
    }

    /// Weeks of revenue the treasury keeps (D4).
    pub fn reserve_weeks(&self) -> i64 {
        self.reserve_weeks
            .map_or(super::public::RESERVE_WEEKS, i64::from)
    }

    /// What the surplus is for (D4).
    pub fn surplus(&self) -> SurplusUse {
        self.surplus.unwrap_or_default()
    }

    /// The deficit target over GDP, when fixed (D5).
    pub fn deficit_cap(&self) -> Option<Fixed> {
        self.deficit_cap.map(|v| milli(i64::from(v)))
    }

    /// The issue split fixed (D9), as shares of 1: home, abroad in the own currency, abroad in
    /// dollars.
    pub fn issuance_shares(&self) -> Option<[Fixed; 3]> {
        self.issuance.map(|s| s.map(|v| milli(i64::from(v))))
    }

    /// Servants and clergy per thousand people, when fixed (D1).
    pub fn payroll(&self) -> Option<[i64; 2]> {
        self.payroll.map(|p| p.map(i64::from))
    }
}

/// Every country's levers, by market index; empty while nobody has fixed anything.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeversState {
    pub countries: Vec<Levers>,
    /// D8: the order the government fixed for its public works, by market (`None` or missing:
    /// arrival order). Works missing from the list come after it, in arrival order.
    pub works_order: Vec<Option<Vec<WorkKey>>>,
    /// D8: public works the governments asked for this month (`GameCommand::AddWork`); the
    /// investment system checks each against what the state may build and queues it.
    pub requests: Vec<WorkRequest>,
}

/// A work in a country's queue, by what it is: its state, building, method and cost in points.
/// Two works with the same key are the same work twice, and either may stand for the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WorkKey {
    pub region: u32,
    pub kind: u16,
    pub method: u8,
    pub points: Fixed,
}

impl WorkKey {
    pub fn of(p: &super::state::Project) -> Self {
        Self {
            region: p.region,
            kind: p.kind,
            method: p.method,
            points: p.points,
        }
    }
}

/// A public work a government asked for (`GameCommand::AddWork`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkRequest {
    pub market: u16,
    pub region: u32,
    pub kind: u16,
}

/// Puts the public works of `m`'s queue in `order`, in the slots the public works already take:
/// the private works keep their places and their arrival order (owner's decision O2: the
/// government has no hand in them). Public works not in `order` follow the listed ones, in their
/// arrival order; a key listed twice takes two works with that key.
pub fn order_queue(m: &mut Market, order: &[WorkKey]) {
    let slots: Vec<usize> = (0..m.queue.len()).filter(|&i| m.queue[i].public).collect();
    if slots.len() < 2 {
        return;
    }
    let mut public: Vec<Option<super::state::Project>> =
        slots.iter().map(|&i| Some(m.queue[i].clone())).collect();
    let mut sorted = Vec::with_capacity(slots.len());
    for key in order {
        if let Some(j) = public
            .iter()
            .position(|p| p.as_ref().is_some_and(|p| WorkKey::of(p) == *key))
        {
            sorted.push(public[j].take().expect("found above"));
        }
    }
    sorted.extend(public.into_iter().flatten());
    for (slot, p) in slots.into_iter().zip(sorted) {
        m.queue[slot] = p;
    }
}

impl LeversState {
    /// No lever fixed anywhere: the state stays out of the hash.
    pub fn is_empty(&self) -> bool {
        self.countries.iter().all(Levers::is_rule)
            && self.works_order.iter().all(Option::is_none)
            && self.requests.is_empty()
    }

    /// The order country `ci` fixed for its public works (D8).
    pub fn order(&self, ci: usize) -> Option<&[WorkKey]> {
        self.works_order.get(ci).and_then(|o| o.as_deref())
    }

    /// Fixes (or, with `None`, hands back to arrival) the order of country `ci`'s public works.
    pub fn set_order(&mut self, ci: usize, order: Option<Vec<WorkKey>>) {
        if self.works_order.len() <= ci {
            self.works_order.resize(ci + 1, None);
        }
        self.works_order[ci] = order;
    }

    /// The levers of country `ci`.
    pub fn of(&self, ci: usize) -> Levers {
        self.countries.get(ci).copied().unwrap_or_default()
    }

    /// Changes the levers of country `ci` and copies them to its market.
    pub fn set(&mut self, markets: &mut [Market], ci: usize, f: impl FnOnce(&mut Levers)) {
        if ci >= markets.len() {
            return;
        }
        if self.countries.len() <= ci {
            self.countries.resize(ci + 1, Levers::default());
        }
        let l = &mut self.countries[ci];
        f(l);
        *l = l.clamped();
        markets[ci].levers = *l;
    }

    /// Copies the saved levers to the markets (after loading a world).
    pub fn sync(&self, markets: &mut [Market]) {
        for (ci, m) in markets.iter_mut().enumerate() {
            m.levers = self.of(ci);
        }
    }
}

impl HashState for LeversState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.countries.len() as u64);
        for l in &self.countries {
            for v in [
                l.public_pay,
                l.purchases,
                l.military,
                l.reserve_weeks,
                l.deficit_cap,
            ] {
                h.write_i64(v.map_or(-1, i64::from));
            }
            for p in [
                l.payroll.map(|p| p.to_vec()),
                l.issuance.map(|p| p.to_vec()),
            ] {
                match p {
                    Some(v) => v.iter().for_each(|x| h.write_i64(i64::from(*x))),
                    None => h.write_i64(-1),
                }
            }
            h.write_i64(l.surplus.map_or(-1, |s| s as i64));
        }
        h.write_u64(self.works_order.len() as u64);
        for o in &self.works_order {
            match o {
                Some(keys) => {
                    h.write_u64(keys.len() as u64);
                    for k in keys {
                        h.write_u32(k.region);
                        h.write_u16(k.kind);
                        h.write_u8(k.method);
                        h.write_i64(k.points.raw());
                    }
                }
                None => h.write_i64(-1),
            }
        }
        h.write_u64(self.requests.len() as u64);
        for r in &self.requests {
            h.write_u16(r.market);
            h.write_u32(r.region);
            h.write_u16(r.kind);
        }
    }
}

/// One lever as the screen shows it: what the rule gives now, what is fixed, and the limits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeverLine {
    pub rule: f64,
    pub fixed: Option<f64>,
    pub floor: f64,
    pub ceiling: f64,
    /// What the fixed value does to the treasury in a week against the rule, in real units
    /// (positive: more cash). `None` on the rule, or where a week's effect is not one number
    /// (the reserve).
    pub week_effect: Option<f64>,
}

/// The first slice's levers of one country, for the budget screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeversView {
    /// Servants and clergy per thousand people.
    pub servants: LeverLine,
    pub clergy: LeverLine,
    /// Public pay over the private wage.
    pub public_pay: LeverLine,
    pub reserve_weeks: LeverLine,
    pub surplus: SurplusUse,
    pub surplus_fixed: bool,
    /// The year's deficit for purchases over GDP.
    pub deficit_cap: LeverLine,
    /// With `Switches::fiscal_capacity` on, the share of last week's taxes not collected for
    /// lack of servants (0–1); `None` with the switch off.
    pub uncollected: Option<f64>,
}

/// What the budget screen reads for country `ci`.
/// The state's purchases (D3) as the budget screen shows them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PurchasesView {
    /// The goods budget over the rule's, per mille (the rule is 1000). Its week's effect is the
    /// purchases the fixed level saves (positive) or adds against the rule's.
    pub goods: LeverLine,
    /// The military share of the goods budget, per mille (the rule is the basket's weight). It
    /// moves money between civilian and military goods; its week's effect is the change in the
    /// military ceiling, not in cash.
    pub military: LeverLine,
    /// This week's goods budget and military ceiling (`public::military_room`), in real units.
    pub budget: f64,
    pub military_room: f64,
}

/// What the budget screen reads of the state's purchases of country market `m` (D3).
pub fn purchases_view(m: &Market, cat: &super::catalogue::Catalogue) -> PurchasesView {
    let l = &m.levers;
    let f = |v: Fixed| v.raw() as f64 / 1000.0;
    let mut rule = m.clone();
    rule.levers.purchases = None;
    rule.levers.military = None;
    let budget = f(super::public::goods_budget(m));
    let room = f(super::public::military_room(m, cat));
    PurchasesView {
        goods: LeverLine {
            rule: 1000.0,
            fixed: l.purchases.map(f64::from),
            floor: f64::from(PURCHASES_MILLE.0),
            ceiling: f64::from(PURCHASES_MILLE.1),
            week_effect: l
                .purchases
                .map(|_| f(super::public::goods_budget(&rule)) - budget),
        },
        military: LeverLine {
            rule: super::public::military_weight(cat) as f64,
            fixed: l.military.map(f64::from),
            floor: f64::from(MILITARY_MILLE.0),
            ceiling: f64::from(MILITARY_MILLE.1),
            week_effect: l
                .military
                .map(|_| room - f(super::public::military_room(&rule, cat))),
        },
        budget,
        military_room: room,
    }
}

/// Where the treasury issues (D9), as the money and debt screen shows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IssuanceView {
    /// The fixed split of new bonds, shares of 1: home, abroad in the own currency, abroad in
    /// dollars. `None` on the rule (home savers first, then the bank, then abroad, and lenders
    /// abroad take the dollar share they demand).
    pub fixed: Option<[f64; 3]>,
    /// Last week with the split fixed, in real units: sold at home (savers and a subordinate
    /// bank), sold abroad in the own currency and in dollars, left unsold at home and abroad,
    /// and the share of local bonds lenders abroad would take (`1 − fx_share`).
    pub home: f64,
    pub abroad_local: f64,
    pub abroad_fx: f64,
    pub unsold_home: f64,
    pub unsold_abroad: f64,
    pub local_limit: f64,
}

/// What the money and debt screen reads of where country market `m` issues (D9).
pub fn issuance_view(m: &Market) -> IssuanceView {
    let f = |v: Fixed| v.raw() as f64 / 1000.0;
    let i = m.issue_last.unwrap_or_default();
    IssuanceView {
        fixed: m.levers.issuance_shares().map(|s| s.map(f)),
        home: f(i.home),
        abroad_local: f(i.abroad_local),
        abroad_fx: f(i.abroad_fx),
        unsold_home: f(i.unsold_home),
        unsold_abroad: f(i.unsold_abroad),
        local_limit: f(i.local_limit),
    }
}

pub fn view(m: &Market) -> LeversView {
    let l = &m.levers;
    let f = |v: Fixed| v.raw() as f64 / 1000.0;
    let line = |rule: f64, fixed: Option<u16>, (lo, hi): (u16, u16)| LeverLine {
        rule,
        fixed: fixed.map(f64::from),
        floor: f64::from(lo),
        ceiling: f64::from(hi),
        week_effect: None,
    };
    // A week of each lever against the rule, by the tick's own sums.
    let b = &m.budget;
    let rule_pay = f(super::public::PUBLIC_PAY);
    // Public pay: last week's payroll was paid at the fixed share; at the rule's it would be
    // payroll × rule / fixed. Cash rises by what the fixed pay saves.
    let pay_effect = l.public_pay.map(|v| {
        let fixed = f64::from(v) / 1000.0;
        let payroll = f(b.payroll);
        payroll * rule_pay / fixed - payroll
    });
    // Servants: those missing against the rule cost no pay but, with the capacity to collect
    // on, the taxes they would have collected. Last week's sums give the second.
    let servants_effect = l.payroll.and(m.uncollected).map(|u| -f(u));
    // Deficit cap: the purchases the cap allows against the rule's, a week.
    let deficit_effect = l.deficit_cap.map(|_| {
        let mut rule = m.clone();
        rule.levers.deficit_cap = None;
        f(super::money::deficit_room(&rule)) - f(super::money::deficit_room(m))
    });
    let share = |p: usize| {
        super::monthly::STATE_SHARE
            .iter()
            .find(|s| s.0 == p)
            .map_or(0.0, |s| s.1 as f64)
    };
    let with = |mut line: LeverLine, e: Option<f64>| {
        line.week_effect = e;
        line
    };
    let uncollected = m.uncollected.map(|u| {
        let due = f(b.revenue) + f(u);
        if due > 0.0 { f(u) / due } else { 0.0 }
    });
    LeversView {
        uncollected,
        servants: with(
            line(
                share(super::prof::CIVIL_SERVANT),
                l.payroll.map(|p| p[0]),
                SERVANTS_PER_MILLE,
            ),
            servants_effect,
        ),
        clergy: line(
            share(super::prof::CLERGY),
            l.payroll.map(|p| p[1]),
            CLERGY_PER_MILLE,
        ),
        public_pay: with(
            line(
                f(super::public::PUBLIC_PAY) * 1000.0,
                l.public_pay,
                PUBLIC_PAY_MILLE,
            ),
            pay_effect,
        ),
        reserve_weeks: line(
            super::public::RESERVE_WEEKS as f64,
            l.reserve_weeks,
            RESERVE_WEEKS,
        ),
        surplus: l.surplus(),
        surplus_fixed: l.surplus.is_some(),
        deficit_cap: with(
            line(
                f(super::money::DEFICIT_TARGET) * 1000.0,
                l.deficit_cap,
                DEFICIT_CAP_MILLE,
            ),
            deficit_effect,
        ),
    }
}

/// What holds a public work back this week (D8, the works screen).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bottleneck {
    /// Nothing: the work gets its full weekly share of points.
    None,
    /// The treasury cannot pay in full: it pays its servants in arrears and starts nothing new
    /// (`public::pay_share` below 1).
    Cash,
    /// The building trade lacked cement or steel last week (`Market::buy_ratio` of the good).
    Cement,
    Steel,
    /// Construction points cost more than 110 % of their base price: the queue is long.
    Price,
}

/// One public work of the queue, as the works screen shows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkLine {
    pub key: WorkKey,
    /// Its public category (`public::INFRASTRUCTURE`…), if the state builds this building.
    pub category: Option<usize>,
    /// Share of its points delivered, 0–1.
    pub progress: f64,
    /// Weeks left at the pace of the last week's deliveries (the queue got `buy_ratio` of the
    /// points it asked for), never fewer than at the full weekly share.
    pub weeks: i64,
    /// Weeks left once the bottleneck clears: at the full weekly share (a work takes at least
    /// `construction::MIN_WEEKS`).
    pub weeks_full: i64,
    pub bottleneck: Bottleneck,
    /// How tight the bottleneck is, 0–1: the share of the cement or steel the building trade
    /// sought that it got last week, the share of its pay the treasury met (`Cash`), or the base
    /// price of construction points over today's (`Price`); 1 with no bottleneck.
    pub met: f64,
    /// With `Cash`: the money the treasury lacks to pay in full again (up to
    /// `public::PAY_RESERVE_WEEKS` of revenue), in real units.
    pub short: Option<f64>,
    /// Share of the points the queue asked for that it got last week, 0–1.
    pub pace: f64,
    /// Placed by the government's order (D8) rather than by arrival.
    pub fixed: bool,
}

/// The public works of market `m`'s queue, in queue order (D8). Private works are not listed:
/// the government has no hand in them (owner's decision O2).
pub fn works(
    cat: &super::catalogue::Catalogue,
    m: &Market,
    order: Option<&[WorkKey]>,
) -> Vec<WorkLine> {
    let kinds = super::public::PublicKinds::new(cat);
    let share = |key: &str| {
        cat.good(key)
            .and_then(|g| m.buy_ratio.get(g as usize).copied())
            .unwrap_or(Fixed::ONE)
    };
    let pace = kinds
        .construction
        .and_then(|cg| m.buy_ratio.get(cg as usize).copied())
        .unwrap_or(Fixed::ONE)
        .max(milli(10));
    let dear = kinds
        .construction
        .is_some_and(|cg| m.price[cg as usize] > cat.goods[cg as usize].base_price * milli(1100));
    // The week tops the treasury up to its reserve, so it hovers just under it; cash holds the
    // works back only when the state pays in arrears (`public::pay_share`).
    let bottleneck = if super::public::pay_share(m) < Fixed::ONE {
        Bottleneck::Cash
    } else if share("cement") < Fixed::ONE {
        Bottleneck::Cement
    } else if share("steel") < Fixed::ONE {
        Bottleneck::Steel
    } else if dear {
        Bottleneck::Price
    } else {
        Bottleneck::None
    };
    let f = |x: Fixed| x.raw() as f64 / 1000.0;
    let (met, short) = match bottleneck {
        Bottleneck::Cash => (
            f(super::public::pay_share(m)),
            Some(f((m.budget.revenue_avg * super::public::PAY_RESERVE_WEEKS
                - m.treasury)
                .max(Fixed::ZERO))),
        ),
        Bottleneck::Cement => (f(share("cement")), None),
        Bottleneck::Steel => (f(share("steel")), None),
        Bottleneck::Price => (
            kinds.construction.map_or(1.0, |cg| {
                f(cat.goods[cg as usize].base_price) / f(m.price[cg as usize]).max(1e-9)
            }),
            None,
        ),
        Bottleneck::None => (1.0, None),
    };
    let ceil = |x: Fixed| (x + Fixed::ONE - Fixed::EPSILON).to_int_floor().max(1);
    let listed = |k: &WorkKey| order.is_some_and(|o| o.contains(k));
    m.queue
        .iter()
        .filter(|p| p.public)
        .map(|p| {
            let key = WorkKey::of(p);
            let full = super::construction::want(p);
            let per_week = full * pace;
            let weeks = if per_week > Fixed::ZERO {
                ceil(p.left / per_week)
            } else {
                0
            };
            let weeks_full = if full > Fixed::ZERO {
                ceil(p.left / full)
            } else {
                0
            };
            WorkLine {
                key,
                category: kinds.category_of(usize::from(p.kind)),
                progress: if p.points > Fixed::ZERO {
                    1.0 - (p.left.raw() as f64 / p.points.raw() as f64)
                } else {
                    1.0
                },
                weeks,
                weeks_full,
                bottleneck,
                met,
                short,
                pace: f(pace),
                fixed: listed(&key),
            }
        })
        .collect()
}

/// The public works the government of country `ci` may ask for, by state (D8, `AddWork`): each
/// building with the cost of one level at today's price of construction points. The tick checks a
/// request against the same list (`public::buildable_in`).
pub fn buildable(
    cat: &super::catalogue::Catalogue,
    world: &crate::game::GameWorld,
    tree: Option<&crate::research::TechTree>,
    year: i32,
    ci: usize,
) -> Vec<(u32, Vec<(u16, Fixed)>)> {
    let econ = &world.econ;
    let Some(m) = econ.markets.get(ci) else {
        return Vec::new();
    };
    let price = super::public::PublicKinds::new(cat)
        .construction
        .map_or(Fixed::ZERO, |cg| m.price[cg as usize]);
    let unlocks = super::Unlocks::new(cat, year, tree, &world.research);
    econ.regions
        .iter()
        .enumerate()
        .filter(|(_, r)| usize::from(r.market) == ci)
        .map(|(ri, _)| {
            let kinds = super::public::buildable_in(cat, econ, &unlocks, ci, ri)
                .into_iter()
                .map(|(k, _, points)| (k as u16, points * price))
                .collect();
            (ri as u32, kinds)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_is_empty_and_limits_cut() {
        let mut s = LeversState::default();
        assert!(s.is_empty());
        let mut markets = vec![Market::empty(0), Market::empty(0)];
        s.set(&mut markets, 1, |l| {
            l.public_pay = Some(5000);
            l.reserve_weeks = Some(1);
        });
        assert!(!s.is_empty());
        assert_eq!(markets[1].levers.public_pay, Some(1500));
        assert_eq!(markets[1].levers.reserve_weeks(), 4);
        assert_eq!(markets[0].levers, Levers::default());
        s.set(&mut markets, 1, |l| *l = Levers::default());
        assert!(s.is_empty(), "back to the rule");
    }
}
