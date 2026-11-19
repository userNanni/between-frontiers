//! Trade with a partner (`docs/trade.pt-BR.md`): who sells to whom comes out of the weekly
//! clearing of each good, by the cost of arrival, not out of an estimate made after it.
//!
//! Every buyer splits what it seeks among the sellers by `offer × preference`, where the
//! preference follows the cost of arrival (price, freight, tariff) with the inertia of contracts
//! and trading networks. A seller asked for more than it offers cuts every order to it in
//! proportion; what is left of the demand goes a second time to the sellers with something left,
//! in the fixed order of the markets. Everything a market exports or imports is the sum of its
//! row or column of the same matrix, so world exports equal world imports exactly.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

/// What crossed between each pair of countries, at base prices (the unit of GDP): exports of
/// seller `s` to buyer `b` at `s × n + b`, in the year so far and in the last full year. Read by
/// the trade channel of influence (`influence::dependence`) and the screens.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeState {
    pub n: u16,
    pub year: Vec<Fixed>,
    pub last_year: Vec<Fixed>,
    /// Each buyer's preference for each seller, by building group (`buyer × groups + group`):
    /// (seller, share of the buyer's favourite in 65,535ths), only above 1/1,000. It moves each
    /// month part of the way toward the cost of arrival (`step`).
    #[serde(default)]
    pub pref: Vec<Vec<(u16, u16)>>,
}

/// Smallest preference kept, in 65,535ths of the buyer's favourite.
const PREF_FLOOR: u16 = 66;

/// What [`equilibrium`] charges a pair: the home bias, the duties' rule, the governments'
/// tables, and the pairs closed (embargoes, war).
pub type Charges<'a> = (
    Option<Fixed>,
    Option<&'a super::customs::Duties>,
    Option<&'a super::customs::TariffState>,
    Option<&'a [bool]>,
);

/// The equilibrium preferences of the month: for every buyer, group and seller, the cost of
/// arrival to the power −σ, the cost being the seller's price index in the group (price over
/// base price, averaged over its tradable goods) times one plus the group's freight over the
/// effective km. Scaled per buyer and group to the favourite seller. `buyer × groups + group`,
/// then by seller.
pub fn equilibrium(
    cat: &super::Catalogue,
    geo: Option<&Geography>,
    markets: &[super::Market],
    factors: &FreightFactors,
    date: &str,
    links: Option<&[crate::treaty::TariffLink]>,
    (home, duties, tables, closed): Charges<'_>,
) -> Vec<Vec<Fixed>> {
    let groups = crate::economy::BUILDING_GROUPS.len();
    let n = markets.len();
    let mut freight = vec![Fixed::ZERO; groups];
    let mut members: Vec<Vec<usize>> = vec![Vec::new(); groups];
    for (g, good) in cat.goods.iter().enumerate() {
        if let Some(k) = cat.good_group[g]
            && good.tradable
        {
            let k = usize::from(k);
            members[k].push(g);
            freight[k] = freight[k].max(cat.good_freight[g]);
        }
    }
    // Price index of each seller in each group.
    let index: Vec<Vec<Fixed>> = markets
        .iter()
        .map(|m| {
            members
                .iter()
                .map(|gs| {
                    if gs.is_empty() {
                        return Fixed::ONE;
                    }
                    let sum: Fixed = gs
                        .iter()
                        .map(|&g| m.price[g] / cat.goods[g].base_price)
                        .sum();
                    sum / gs.len() as i64
                })
                .collect()
        })
        .collect();
    let reference = i64::from(cat.trade.reference_km.max(1));
    // What depends only on the pair, once a month: whether it is closed, the km, the carry and
    // the tariff (the treaties in force on the date, filtered once).
    let closed = closed.map(<[bool]>::to_vec);
    // From the agreements in force when they are kept (`links`), else from the treaties of
    // `diplomacy.ron` in force on the date: the same relations in 1925, in the same order.
    let owned: Vec<TariffRelation> = links.map_or_else(Vec::new, |l| {
        l.iter().map(TariffRelation::from_link).collect()
    });
    let relations: Vec<&TariffRelation> = match links {
        Some(_) => owned.iter().collect(),
        None => geo.map_or_else(Vec::new, |g| g.relations_on(date)),
    };
    let mut pair: Vec<Option<(i64, Fixed, Fixed)>> = vec![None; n];
    let mut out = vec![vec![Fixed::ZERO; n]; n * groups];
    for b in 0..n {
        // The tariff is charged on the value with freight: one charged alike on every seller
        // cancels out; only a treaty's discrimination moves the shares.
        let own = markets[b]
            .laws
            .effects
            .first()
            .map_or(Fixed::ZERO, |e| e.tariff);
        // With customs duties, the tariff the country charges (its law moved by its 1925 gap,
        // `customs::Duties::rate`): the same one the treasury collects.
        let own = duties.map_or(own, |d| d.rate_of(b, own));
        // A government that fixed goods or partners (CT1, CT2): by group, the mean of its goods'
        // rates (the rule's for those it did not fix), and each partner's points on top. The
        // preference is by group; a good fixed far from the rest of its group moves the group's
        // whole share (docs/trade.pt-BR.md, section 13).
        let table = tables.and_then(|t| t.table(b));
        let own_by_group: Vec<Fixed> = table.map_or_else(Vec::new, |t| {
            members
                .iter()
                .map(|gs| {
                    if gs.is_empty() {
                        return own;
                    }
                    let sum: Fixed = gs.iter().map(|&g| t.good(g).unwrap_or(own)).sum();
                    sum / gs.len() as i64
                })
                .collect()
        });
        for (s, p) in pair.iter_mut().enumerate() {
            // A closed pair has no equilibrium: its preference decays (`step`).
            *p = (s != b && !closed.as_ref().is_some_and(|c| c[s * n + b])).then(|| {
                let km = i64::from(geo.map_or(0, |g| g.km(s, b)));
                let tariff = if geo.is_some() {
                    tariff_by(&relations, b, s, own)
                } else {
                    own
                };
                (km, factors.of_pair(geo, s, b), tariff)
            });
        }
        for k in 0..groups {
            let row = &mut out[b * groups + k];
            for (s, p) in pair.iter().enumerate() {
                let Some((km, carry, tariff)) = *p else {
                    continue;
                };
                let tariff = match table {
                    Some(t) => {
                        let base = (own_by_group[k] + t.partner(s)).max(Fixed::ZERO);
                        if geo.is_some() {
                            tariff_by(&relations, b, s, base)
                        } else {
                            base
                        }
                    }
                    None => tariff,
                };
                row[s] = power(
                    cat,
                    index[s][k]
                        * (Fixed::ONE + freight[k] * km * carry / reference)
                        * (Fixed::ONE + tariff),
                );
            }
            if let Some(h) = home {
                row[b] = h * power(cat, index[b][k]);
            }
            let top = row.iter().copied().max().unwrap_or(Fixed::ZERO);
            if top > Fixed::ZERO {
                for v in row.iter_mut() {
                    *v /= top;
                }
            }
        }
    }
    out
}

/// The cost of arrival to the power −σ, with a floor of a quarter on the cost.
fn power(cat: &super::Catalogue, cost: Fixed) -> Fixed {
    let inverse = Fixed::ONE / cost.max(Fixed::from_ratio(1, 4));
    let mut p = Fixed::ONE;
    for _ in 0..cat.trade.sigma {
        p *= inverse;
    }
    p
}

/// A month of inertia: every preference moves `step` of the way toward the equilibrium; with
/// none yet (the first week, a save from before), it starts at the equilibrium.
pub fn step(old: &[Vec<(u16, u16)>], eq: &[Vec<Fixed>], step: Fixed) -> Vec<Vec<(u16, u16)>> {
    let full = Fixed::from_int(i64::from(u16::MAX));
    eq.iter()
        .enumerate()
        .map(|(i, row)| {
            let mut was = vec![Fixed::ZERO; row.len()];
            let fresh = old.get(i).is_none_or(Vec::is_empty);
            if let Some(o) = old.get(i) {
                for &(s, v) in o {
                    if let Some(w) = was.get_mut(usize::from(s)) {
                        *w = Fixed::from_ratio(i64::from(v), i64::from(u16::MAX));
                    }
                }
            }
            let moved: Vec<Fixed> = row
                .iter()
                .zip(&was)
                .map(|(&e, &w)| if fresh { e } else { w + (e - w) * step })
                .collect();
            let top = moved.iter().copied().max().unwrap_or(Fixed::ZERO);
            if top <= Fixed::ZERO {
                return Vec::new();
            }
            moved
                .iter()
                .enumerate()
                .filter_map(|(s, &v)| {
                    let q = (v / top * full)
                        .to_int_floor()
                        .clamp(0, i64::from(u16::MAX)) as u16;
                    (q >= PREF_FLOOR).then_some((s as u16, q))
                })
                .collect()
        })
        .collect()
}

/// The preferences as one flat table, `(buyer × groups + group) × n + seller`, in shares of the
/// buyer's favourite: what `dense` gives, in a single allocation.
pub fn dense_flat(pref: &[Vec<(u16, u16)>], n: usize) -> Vec<Fixed> {
    let mut out = vec![Fixed::ZERO; pref.len() * n];
    for (row, list) in pref.iter().enumerate() {
        for &(s, v) in list {
            let s = usize::from(s);
            if s < n {
                out[row * n + s] = Fixed::from_ratio(i64::from(v), i64::from(u16::MAX));
            }
        }
    }
    out
}

/// The preferences as a dense table, `buyer × groups + group`, then by seller, in shares of the
/// buyer's favourite.
pub fn dense(pref: &[Vec<(u16, u16)>], n: usize) -> Vec<Vec<Fixed>> {
    pref.iter()
        .map(|list| {
            let mut row = vec![Fixed::ZERO; n];
            for &(s, v) in list {
                if let Some(r) = row.get_mut(usize::from(s)) {
                    *r = Fixed::from_ratio(i64::from(v), i64::from(u16::MAX));
                }
            }
            row
        })
        .collect()
}

impl TradeState {
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Grows to `n` countries, keeping what each pair had (countries born later start at zero).
    pub fn fit(&mut self, n: usize) {
        let old = usize::from(self.n);
        if n <= old {
            return;
        }
        let grow = |v: &[Fixed]| {
            let mut out = vec![Fixed::ZERO; n * n];
            for s in 0..old {
                for b in 0..old {
                    out[s * n + b] = v.get(s * old + b).copied().unwrap_or(Fixed::ZERO);
                }
            }
            out
        };
        self.year = grow(&self.year);
        self.last_year = grow(&self.last_year);
        self.n = n as u16;
    }

    /// Adds `value` (base prices) to the exports of `s` to `b` this year.
    pub fn add(&mut self, s: usize, b: usize, value: Fixed) {
        let n = usize::from(self.n);
        if s < n && b < n {
            self.year[s * n + b] += value;
        }
    }

    /// The year closes: what was traded becomes the last full year.
    pub fn roll_year(&mut self) {
        self.last_year = std::mem::replace(&mut self.year, vec![Fixed::ZERO; self.last_year.len()]);
    }

    /// Exports of `from` to `to` in the last full year, at base prices.
    pub fn exports(&self, from: usize, to: usize) -> Fixed {
        let n = usize::from(self.n);
        if from < n && to < n {
            self.last_year[from * n + to]
        } else {
            Fixed::ZERO
        }
    }
}

impl HashState for TradeState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(u64::from(self.n));
        self.year.hash_state(h);
        self.last_year.hash_state(h);
        h.write_u64(self.pref.len() as u64);
        for list in &self.pref {
            h.write_u64(list.len() as u64);
            for &(s, v) in list {
                h.write_u64(u64::from(s) << 16 | u64::from(v));
            }
        }
    }
}

/// The flows of one good: `flow[s][b]` is what seller `s` sends buyer `b`, in the order of the
/// `offered` and `sought` slices.
pub type Flows = Vec<Vec<Fixed>>;

/// Clears one good between `offered` (by seller) and `sought` (by buyer). `preference(s, b)` is
/// how much buyer `b` leans toward seller `s` (non-negative; equal preferences give the world
/// pool's pro rata). Sellers and buyers are distinct lists; a market appears in at most one.
pub fn clear_good(
    offered: &[Fixed],
    sought: &[Fixed],
    preference: impl Fn(usize, usize) -> Fixed,
) -> Flows {
    let (ns, nb) = (offered.len(), sought.len());
    let mut pref = vec![Fixed::ZERO; ns * nb];
    for b in 0..nb {
        for s in 0..ns {
            pref[b * ns + s] = preference(s, b);
        }
    }
    let mut buf = ClearBuffers::default();
    clear_good_flat(offered, sought, &pref, &mut buf);
    (0..ns)
        .map(|s| buf.flow[s * nb..(s + 1) * nb].to_vec())
        .collect()
}

/// What the clearing of one good leaves, and its scratch, kept from good to good so that the
/// week allocates once (`clear_good_flat`).
#[derive(Default)]
pub struct ClearBuffers {
    /// What seller `s` sends buyer `b`, at `s × buyers + b`.
    pub flow: Vec<Fixed>,
    /// What each seller exports and each buyer imports: the sums of `flow`'s rows and columns.
    pub exports: Vec<Fixed>,
    pub imports: Vec<Fixed>,
    /// Each pass's orders, `s × buyers + b`.
    asked: Vec<Fixed>,
    requested: Vec<Fixed>,
    left_offer: Vec<Fixed>,
    left_want: Vec<Fixed>,
    weights: Vec<Fixed>,
    parts: Vec<Fixed>,
    sellers: Vec<usize>,
    sent: Vec<Fixed>,
    rest: Vec<(i128, usize)>,
}

/// `clear_good` on flat tables: `pref[b × sellers + s]` is how much buyer `b` leans toward
/// seller `s`; the flows land in `buf`. Two passes: the orders as the preferences place them,
/// then what is left of each buyer's demand among the sellers that still have something. The
/// sums of each seller's and each buyer's flows are kept as they are written, in the order the
/// columns and rows would add them: the same integers as summing the matrix after.
pub fn clear_good_flat(
    offered: &[Fixed],
    sought: &[Fixed],
    pref: &[Fixed],
    buf: &mut ClearBuffers,
) {
    let (ns, nb) = (offered.len(), sought.len());
    let reset = |v: &mut Vec<Fixed>, n: usize| {
        v.clear();
        v.resize(n, Fixed::ZERO);
    };
    reset(&mut buf.flow, ns * nb);
    reset(&mut buf.exports, ns);
    reset(&mut buf.imports, nb);
    reset(&mut buf.asked, ns * nb);
    reset(&mut buf.sent, nb);
    buf.left_offer.clear();
    buf.left_offer.extend_from_slice(offered);
    buf.left_want.clear();
    buf.left_want.extend_from_slice(sought);
    for pass in 0..2 {
        if pass > 0 {
            buf.asked.fill(Fixed::ZERO);
        }
        reset(&mut buf.requested, ns);
        for b in 0..nb {
            if buf.left_want[b] <= Fixed::ZERO {
                continue;
            }
            buf.sellers.clear();
            buf.weights.clear();
            let row = &pref[b * ns..(b + 1) * ns];
            for s in 0..ns {
                if buf.left_offer[s] > Fixed::ZERO && row[s] > Fixed::ZERO {
                    buf.sellers.push(s);
                    buf.weights.push(buf.left_offer[s] * row[s]);
                }
            }
            buf.parts.clear();
            buf.parts.resize(buf.weights.len(), Fixed::ZERO);
            split_exact_into(
                buf.left_want[b],
                &buf.weights,
                &mut buf.parts,
                &mut buf.rest,
            );
            for (k, &s) in buf.sellers.iter().enumerate() {
                let q = buf.parts[k];
                buf.asked[s * nb + b] = q;
                buf.requested[s] += q;
            }
        }
        for s in 0..ns {
            if buf.requested[s] <= Fixed::ZERO {
                continue;
            }
            let row = &buf.asked[s * nb..(s + 1) * nb];
            // A seller asked for more than it has cuts every order in proportion.
            let sent: &[Fixed] = if buf.requested[s] > buf.left_offer[s] {
                split_exact_into(buf.left_offer[s], row, &mut buf.sent, &mut buf.rest);
                &buf.sent
            } else {
                row
            };
            let flow = &mut buf.flow[s * nb..(s + 1) * nb];
            let mut total = Fixed::ZERO;
            for b in 0..nb {
                let q = sent[b];
                flow[b] += q;
                total += q;
                buf.imports[b] += q;
            }
            buf.exports[s] += total;
        }
        for s in 0..ns {
            buf.left_offer[s] = (offered[s] - buf.exports[s]).max(Fixed::ZERO);
        }
        for b in 0..nb {
            buf.left_want[b] = (sought[b] - buf.imports[b]).max(Fixed::ZERO);
        }
    }
}

/// Division by a divisor fixed for many numerators, without a hardware division each time: the
/// reciprocal `⌊(2⁶⁴ − 1) / d⌋` is taken once, the quotient estimated by a 64 × 64 → 128-bit
/// multiplication and corrected exactly (it is short by at most two). The same quotient and
/// remainder as `/` and `%`.
pub(crate) struct Divisor {
    d: u64,
    m: u64,
}

impl Divisor {
    pub(crate) fn new(d: u64) -> Self {
        assert!(d > 0, "a divisor of zero");
        Self { d, m: u64::MAX / d }
    }

    #[inline]
    pub(crate) fn div_rem(&self, n: u64) -> (u64, u64) {
        let mut q = ((u128::from(n) * u128::from(self.m)) >> 64) as u64;
        let mut r = n - q * self.d;
        while r >= self.d {
            q += 1;
            r -= self.d;
        }
        (q, r)
    }

    /// `n / d` rounded to nearest, ties up: `Fixed`'s division of two non-negative values
    /// (`n` the dividend's raw times the scale).
    #[inline]
    pub(crate) fn div_round(&self, n: u64) -> u64 {
        let (q, r) = self.div_rem(n);
        if r * 2 >= self.d { q + 1 } else { q }
    }
}

/// Every value of `row` (non-negative) divided by `top` (positive), as `Fixed`'s `/` would.
pub(crate) fn scale_to(row: &mut [Fixed], top: Fixed) {
    let d = Divisor::new(top.raw() as u64);
    for v in row.iter_mut() {
        match v.raw().checked_mul(Fixed::SCALE) {
            Some(n) if n >= 0 => *v = Fixed::from_raw(d.div_round(n as u64) as i64),
            _ => *v /= top,
        }
    }
}

/// `total` split by `weights` exactly into `out` (`rest` is scratch): the floor of each share in
/// raw units, and the units left to the largest remainders (ties to the lower index), so the
/// parts add up to `total` to the last unit and never past it. Fewer units are left than there
/// are weights, so only that many of the largest remainders take one: they are selected, not
/// sorted (the same ones, the order being total).
fn split_exact_into(
    total: Fixed,
    weights: &[Fixed],
    out: &mut [Fixed],
    rest: &mut Vec<(i128, usize)>,
) {
    let sum: i128 = weights.iter().map(|w| i128::from(w.raw().max(0))).sum();
    if total <= Fixed::ZERO || sum <= 0 {
        out.fill(Fixed::ZERO);
        return;
    }
    let t = i128::from(total.raw());
    rest.clear();
    let mut given: i128 = 0;
    // Most products fit in 64 bits: divide there, in hardware, instead of in the 128-bit
    // software routine (the same integers). A zero weight gets nothing and leaves no remainder:
    // it is never among the remainders that take a unit, so it is left out of them.
    let small = u64::try_from(sum).ok().map(Divisor::new);
    for (i, w) in weights.iter().enumerate() {
        let w = w.raw().max(0);
        if w == 0 {
            out[i] = Fixed::ZERO;
            continue;
        }
        let (part, r) = match (&small, total.raw().checked_mul(w)) {
            (Some(d), Some(num)) => {
                let (q, r) = d.div_rem(num as u64);
                (i128::from(q), i128::from(r))
            }
            _ => {
                let num = t * i128::from(w);
                (num / sum, num % sum)
            }
        };
        given += part;
        out[i] = Fixed::from_raw(part as i64);
        rest.push((r, i));
    }
    let left = (t - given) as usize;
    if left == 0 {
        return;
    }
    let order = |a: &(i128, usize), b: &(i128, usize)| b.0.cmp(&a.0).then(a.1.cmp(&b.1));
    if left < rest.len() {
        rest.select_nth_unstable_by(left - 1, order);
    }
    for &(r, i) in &rest[..left.min(rest.len())] {
        if r > 0 {
            out[i] = Fixed::from_raw(out[i].raw() + 1);
        }
    }
}

/// Scales each buyer's preferences (`pref[s][b]`) so that its favourite seller has 1: the weights
/// `offer × preference` then never exceed the offer.
pub fn normalize(pref: &mut [Vec<Fixed>]) {
    let nb = pref.first().map_or(0, Vec::len);
    for b in 0..nb {
        let top = pref.iter().map(|row| row[b]).max().unwrap_or(Fixed::ZERO);
        if top > Fixed::ZERO {
            for row in pref.iter_mut() {
                row[b] /= top;
            }
        }
    }
}

/// What each seller exports and each buyer imports: the sums of the rows and the columns.
pub fn totals(flow: &Flows, buyers: usize) -> (Vec<Fixed>, Vec<Fixed>) {
    let exports = flow.iter().map(|row| row.iter().copied().sum()).collect();
    let imports = (0..buyers)
        .map(|b| flow.iter().map(|row| row[b]).sum())
        .collect();
    (exports, imports)
}

/// The tariff `buyer` charges on `seller`'s goods under `relations` (those in force), from the
/// buyer's own `tariff`: a customs union takes it to zero, a preference lowers it by a margin, a
/// cap holds it under a ceiling, in the order of the treaties.
fn tariff_by(relations: &[&TariffRelation], buyer: usize, seller: usize, tariff: Fixed) -> Fixed {
    let mut t = tariff;
    for r in relations
        .iter()
        .filter(|r| r.giver == buyer && r.receiver == seller)
    {
        t = match r.rule {
            TariffRule::Union => Fixed::ZERO,
            TariffRule::Margin(m) => (t - m).max(Fixed::ZERO),
            TariffRule::Cap(c) => t.min(c),
        };
    }
    t
}

/// Effective distance between countries for freight, in km of land haul, fixed for the
/// scenario, and the tariff relations of its treaties: the sea route between their main ports at a third of the land cost, or the land
/// route between capitals where they share a border, whichever is cheaper. Indexed by the
/// scenario's country order (the markets'); a country born later has none and trades at the
/// mean distance.
#[derive(Clone, Debug, Default)]
pub struct Geography {
    n: usize,
    km: Vec<u32>,
    mean: u32,
    /// Tariff relations by treaty, between country indices.
    relations: Vec<TariffRelation>,
    /// Whether the cheaper way between each pair is by sea (else by land).
    sea: Vec<bool>,
    /// Pairs closed to trade (embargo, sanction, war, no recognition), with their dates. Empty
    /// until the clauses and the war exist (docs/trade.pt-BR.md, section 11).
    closures: Vec<Closure>,
}

/// A pair closed to trade, both ways, while in force.
#[derive(Clone, Debug)]
pub struct Closure {
    pub a: usize,
    pub b: usize,
    pub from: String,
    pub until: Option<String>,
}

/// What lowers the freight this month (docs/trade.pt-BR.md, section 10): the world's diffused
/// transport technique for the sea leg, and each country's own (its technique and the price of
/// its inland transport, which measures its railways, roads and ports) for the land leg and the
/// port.
pub struct FreightFactors {
    /// Multiplier of the sea leg, from the world's `trade` effect weighted by GDP.
    pub sea: Fixed,
    /// Multiplier of each country's land leg and port.
    pub land: Vec<Fixed>,
}

/// How much the whole tree's `trade` effect cuts the freight: 0.7, so that with all of it the
/// freight is 30 % of 1925's (design value, docs/trade.pt-BR.md).
const TECH_CUT: Fixed = Fixed::from_raw(700);

impl FreightFactors {
    /// From each market's `trade` effect (0–1 of the tree), its GDP and the price of its inland
    /// transport over the base price.
    pub fn of(effect: &[Fixed], gdp: &[Fixed], transport: &[Fixed]) -> Self {
        let world: Fixed = gdp.iter().copied().sum();
        let mean: Fixed = if world > Fixed::ZERO {
            effect.iter().zip(gdp).map(|(e, g)| *e * (*g / world)).sum()
        } else {
            Fixed::ZERO
        };
        Self {
            sea: Fixed::ONE - TECH_CUT * mean.clamp(Fixed::ZERO, Fixed::ONE),
            land: effect
                .iter()
                .zip(transport)
                .map(|(&e, &t)| {
                    (Fixed::ONE - TECH_CUT * e.clamp(Fixed::ZERO, Fixed::ONE))
                        * t.clamp(Fixed::from_ratio(1, 2), Fixed::from_int(4))
                })
                .collect(),
        }
    }

    /// The multiplier of the freight between `s` and `b`: the sea leg's, or the land's, times
    /// the mean of the two sides' ports and inland transport.
    pub fn of_pair(&self, geo: Option<&Geography>, s: usize, b: usize) -> Fixed {
        let side = |c: usize| self.land.get(c).copied().unwrap_or(Fixed::ONE);
        let both = (side(s) + side(b)) / 2;
        if geo.is_some_and(|g| g.by_sea(s, b)) {
            self.sea * both
        } else {
            both
        }
    }
}

/// What a treaty does to the tariff `giver` charges on `receiver`'s goods, while in force.
#[derive(Clone, Debug)]
struct TariffRelation {
    giver: usize,
    receiver: usize,
    from: String,
    until: Option<String>,
    rule: TariffRule,
}

impl TariffRelation {
    /// A relation of an agreement in force (`TreatyState::tariff_relations`), in force now.
    fn from_link(l: &crate::treaty::TariffLink) -> Self {
        use crate::treaty::TariffTerm;
        Self {
            giver: usize::from(l.giver),
            receiver: usize::from(l.receiver),
            from: String::new(),
            until: None,
            rule: match l.rule {
                TariffTerm::Union => TariffRule::Union,
                TariffTerm::Margin(m) => TariffRule::Margin(m),
                TariffTerm::Cap(c) => TariffRule::Cap(c),
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TariffRule {
    /// Inside one customs territory: no tariff.
    Union,
    /// The tariff lowered by a margin (imperial preference).
    Margin(Fixed),
    /// The tariff at most a rate (unequal treaties, most favoured nation).
    Cap(Fixed),
}

impl TariffRelation {
    fn from_treaties(
        diplomacy: &crate::diplomacy::DiplomacyFile,
        countries: &[crate::scenario::CountryDef],
    ) -> Vec<Self> {
        use crate::treaty::{ClauseKind, Side};
        let ix = |tag: &crate::scenario::Tag| countries.iter().position(|c| &c.tag == tag);
        let mut out = Vec::new();
        for t in &diplomacy.treaties {
            let side = |s: &Side| -> Vec<usize> {
                let tags: Vec<&crate::scenario::Tag> = match s {
                    Side::Senior => t.senior.iter().collect(),
                    Side::Junior => t.parties.iter().filter(|p| !t.senior.contains(p)).collect(),
                    Side::All => t.parties.iter().collect(),
                    Side::Parties(v) => v.iter().collect(),
                };
                tags.into_iter().filter_map(ix).collect()
            };
            for c in &t.clauses {
                let rule = match &c.kind {
                    ClauseKind::CustomsUnion => TariffRule::Union,
                    ClauseKind::TariffPreference { margin } => TariffRule::Margin(*margin),
                    ClauseKind::TariffCap { rate: Some(r) } => TariffRule::Cap(*r),
                    // A frozen tariff treats every partner alike: nothing to weigh.
                    _ => continue,
                };
                let until = match (&t.until, &c.until) {
                    (Some(a), Some(b)) => Some(a.min(b).clone()),
                    (a, b) => a.clone().or_else(|| b.clone()),
                };
                for g in side(&c.giver) {
                    for r in side(&c.receiver) {
                        if g == r {
                            continue;
                        }
                        out.push(Self {
                            giver: g,
                            receiver: r,
                            from: t.in_force.clone(),
                            until: until.clone(),
                            rule,
                        });
                        // A customs union is free trade both ways.
                        if matches!(rule, TariffRule::Union) {
                            out.push(Self {
                                giver: r,
                                receiver: g,
                                from: t.in_force.clone(),
                                until: until.clone(),
                                rule,
                            });
                        }
                    }
                }
            }
        }
        out
    }
}

impl Geography {
    /// Builds the matrix from the map, the countries and the sea graph. A country's main port is
    /// its metropole's coastal state nearest to the capital.
    pub fn build(
        map: &crate::scenario::MapFile,
        countries: &[crate::scenario::CountryDef],
        seas: Option<&crate::sea::SeaGraph>,
        diplomacy: Option<&crate::diplomacy::DiplomacyFile>,
        params: &crate::economy::TradeDef,
    ) -> Self {
        let n = countries.len();
        let meta = &map.meta;
        let miller = |lat: f64| {
            1.25 * (std::f64::consts::FRAC_PI_4 + 0.4 * lat.to_radians())
                .tan()
                .ln()
        };
        let (top, bottom) = (miller(meta.lat_top), miller(meta.lat_bottom));
        let lon_lat = |c: (f32, f32)| {
            let lon = f64::from(c.0) / f64::from(meta.width.max(1)) * 360.0 - 180.0;
            let y = top - f64::from(c.1) / f64::from(meta.height.max(1)) * (top - bottom);
            let lat = ((y / 1.25).exp().atan() - std::f64::consts::FRAC_PI_4) / 0.4;
            (lon, lat.to_degrees())
        };
        let great_circle = |a: (f64, f64), b: (f64, f64)| {
            let (la1, la2) = (a.1.to_radians(), b.1.to_radians());
            let dl = (b.0 - a.0).to_radians();
            let h = ((la2 - la1) / 2.0).sin().powi(2)
                + la1.cos() * la2.cos() * (dl / 2.0).sin().powi(2);
            2.0 * 6371.0 * h.sqrt().asin()
        };
        let owner: std::collections::BTreeMap<u16, usize> = map
            .states
            .iter()
            .filter_map(|s| {
                countries
                    .iter()
                    .position(|c| c.tag == s.owner)
                    .map(|ci| (s.id, ci))
            })
            .collect();
        let port: Vec<Option<u16>> = countries
            .iter()
            .enumerate()
            .map(|(ci, c)| {
                let seas = seas?;
                map.states
                    .iter()
                    .filter(|s| owner.get(&s.id) == Some(&ci) && !s.colony)
                    .filter(|s| !seas.ports_of(s.id).is_empty())
                    .map(|s| (great_circle(lon_lat(s.center), c.capital_lon_lat), s.id))
                    .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                    .map(|(_, id)| id)
            })
            .collect();
        let mut land = vec![false; n * n];
        for s in &map.states {
            let Some(&a) = owner.get(&s.id) else { continue };
            for nb in &s.neighbours {
                if let Some(&b) = owner.get(nb)
                    && a != b
                {
                    land[a * n + b] = true;
                    land[b * n + a] = true;
                }
            }
        }
        let open = crate::sea::RouteOptions::open("1925-01-01");
        let mut km = vec![0u32; n * n];
        let mut sea = vec![false; n * n];
        for a in 0..n {
            for b in 0..n {
                if a == b {
                    continue;
                }
                let gc = great_circle(countries[a].capital_lon_lat, countries[b].capital_lon_lat);
                let by_land = gc * f64::from(params.land_detour);
                let by_sea = match (seas, port[a], port[b]) {
                    (Some(g), Some(pa), Some(pb)) => g
                        .route(pa, pb, &open)
                        .map(|r| f64::from(r.km) * f64::from(params.sea_per_land)),
                    _ => None,
                };
                let (d, is_sea) = match by_sea {
                    Some(sea) if land[a * n + b] && by_land < sea => (by_land, false),
                    Some(sea) => (sea, true),
                    // No sea route: the land haul to the partner (through its neighbours).
                    None => (by_land, false),
                };
                km[a * n + b] = d.round() as u32;
                sea[a * n + b] = is_sea;
            }
        }
        let pairs = (n * n).saturating_sub(n).max(1) as u64;
        let mean = (km.iter().map(|&k| u64::from(k)).sum::<u64>() / pairs) as u32;
        let relations =
            diplomacy.map_or_else(Vec::new, |d| TariffRelation::from_treaties(d, countries));
        Self {
            n,
            km,
            mean,
            relations,
            sea,
            closures: Vec::new(),
        }
    }

    /// Closes the pair `a`–`b` to trade, both ways, from `from` (`YYYY-MM-DD`) until `until`
    /// (none: for good). The slot of embargoes, sanctions and war (section 11).
    pub fn close(&mut self, a: usize, b: usize, from: &str, until: Option<&str>) {
        self.closures.push(Closure {
            a,
            b,
            from: from.to_string(),
            until: until.map(str::to_string),
        });
    }

    /// Whether the cheaper way from `a` to `b` is by sea.
    pub fn by_sea(&self, a: usize, b: usize) -> bool {
        a < self.n && b < self.n && self.sea[a * self.n + b]
    }

    /// Whether trade between `a` and `b` is closed on `date` (either way).
    pub fn closed(&self, a: usize, b: usize, date: &str) -> bool {
        self.closures.iter().any(|c| {
            ((c.a == a && c.b == b) || (c.a == b && c.b == a))
                && crate::diplomacy::covers(&c.from, c.until.as_deref(), date)
        })
    }

    /// The tariff `buyer` charges on `seller`'s goods on `date`, from the buyer's own `tariff`
    /// and the treaties between them.
    pub fn tariff(&self, buyer: usize, seller: usize, tariff: Fixed, date: &str) -> Fixed {
        tariff_by(&self.relations_on(date), buyer, seller, tariff)
    }

    /// The tariff relations in force on `date`, in the order of the treaties.
    fn relations_on(&self, date: &str) -> Vec<&TariffRelation> {
        self.relations
            .iter()
            .filter(|r| crate::diplomacy::covers(&r.from, r.until.as_deref(), date))
            .collect()
    }

    /// The pairs closed on `date` by the dated closures, plus `extra` (both ways): the
    /// embargoes, the pairs at war and those a blockade cuts (`war::fight::blockaded_pairs`).
    /// `None` when nothing is closed.
    pub fn closed_with(&self, date: &str, n: usize, extra: &[(u16, u16)]) -> Option<Vec<bool>> {
        let mut out = self.closed_on(date, n);
        for &(a, b) in extra {
            let (a, b) = (usize::from(a), usize::from(b));
            if a < n && b < n && a != b {
                let t = out.get_or_insert_with(|| vec![false; n * n]);
                t[a * n + b] = true;
                t[b * n + a] = true;
            }
        }
        out
    }

    /// The pairs among `n` markets closed on `date` (either way), as an `n × n` table, as
    /// `closed` answers them; `None` when none is.
    pub fn closed_on(&self, date: &str, n: usize) -> Option<Vec<bool>> {
        let mut out: Option<Vec<bool>> = None;
        for c in self
            .closures
            .iter()
            .filter(|c| crate::diplomacy::covers(&c.from, c.until.as_deref(), date))
        {
            if c.a < n && c.b < n {
                let t = out.get_or_insert_with(|| vec![false; n * n]);
                t[c.a * n + c.b] = true;
                t[c.b * n + c.a] = true;
            }
        }
        out
    }

    /// Effective km from `a` to `b`.
    pub fn km(&self, a: usize, b: usize) -> u32 {
        if a == b {
            0
        } else if a < self.n && b < self.n {
            self.km[a * self.n + b]
        } else {
            self.mean
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::milli;

    fn f(v: i64) -> Fixed {
        Fixed::from_int(v)
    }

    /// The world pool of before: what is traded is the smaller side, shared pro rata.
    fn pool(offered: &[Fixed], sought: &[Fixed]) -> (Vec<Fixed>, Vec<Fixed>) {
        let o: Fixed = offered.iter().copied().sum();
        let s: Fixed = sought.iter().copied().sum();
        let traded = o.min(s);
        (
            offered.iter().map(|x| x.mul_div(traded, o)).collect(),
            sought.iter().map(|x| x.mul_div(traded, s)).collect(),
        )
    }

    fn close(a: Fixed, b: Fixed) -> bool {
        (a - b).abs() <= Fixed::from_ratio(1, 1_000_000)
    }

    #[test]
    fn equal_preferences_give_the_pool() {
        for (offered, sought) in [
            (vec![f(30), f(70)], vec![f(20), f(50), f(10)]),
            (vec![f(5), f(7), f(1)], vec![f(40), f(60)]),
            (vec![f(100)], vec![f(1), f(2), f(3)]),
        ] {
            let flow = clear_good(&offered, &sought, |_, _| Fixed::ONE);
            let (exp, imp) = totals(&flow, sought.len());
            let (pe, pi) = pool(&offered, &sought);
            for (a, b) in exp.iter().zip(&pe) {
                assert!(close(*a, *b), "exports {a:?} against the pool's {b:?}");
            }
            for (a, b) in imp.iter().zip(&pi) {
                assert!(close(*a, *b), "imports {a:?} against the pool's {b:?}");
            }
        }
    }

    #[test]
    fn what_is_exported_is_what_is_imported_and_within_both_sides() {
        // Deterministic pseudo-random markets and preferences.
        let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut next = |m: u64| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x % m) as i64
        };
        for _ in 0..200 {
            let ns = 1 + next(8) as usize;
            let nb = 1 + next(8) as usize;
            let offered: Vec<Fixed> = (0..ns).map(|_| milli(next(500_000))).collect();
            let sought: Vec<Fixed> = (0..nb).map(|_| milli(next(500_000))).collect();
            let pref: Vec<Vec<Fixed>> = (0..ns)
                .map(|_| (0..nb).map(|_| milli(next(2_000))).collect())
                .collect();
            let flow = clear_good(&offered, &sought, |s, b| pref[s][b]);
            let (exp, imp) = totals(&flow, nb);
            let world_exports: Fixed = exp.iter().copied().sum();
            let world_imports: Fixed = imp.iter().copied().sum();
            assert_eq!(world_exports, world_imports);
            for (e, o) in exp.iter().zip(&offered) {
                assert!(*e <= *o, "a seller sent more than it offered");
            }
            for (i, s) in imp.iter().zip(&sought) {
                assert!(*i <= *s, "a buyer got more than it sought");
            }
            assert!(flow.iter().flatten().all(|q| *q >= Fixed::ZERO));
        }
    }

    #[test]
    fn the_cheaper_seller_sells_more_but_not_everything() {
        let offered = [f(100), f(100)];
        let sought = [f(100)];
        // The first seller's goods arrive at half the preference of the second's.
        let flow = clear_good(&offered, &sought, |s, _| {
            if s == 0 { milli(500) } else { Fixed::ONE }
        });
        assert!(flow[1][0] > flow[0][0]);
        assert!(flow[0][0] > Fixed::ZERO);
        assert!(close(flow[0][0] + flow[1][0], f(100)));
    }

    #[test]
    fn a_full_seller_sends_the_rest_of_the_demand_elsewhere() {
        // The buyer prefers the small seller, which runs out; the second pass buys the rest
        // from the other.
        let offered = [f(10), f(100)];
        let sought = [f(60)];
        let flow = clear_good(
            &offered,
            &sought,
            |s, _| if s == 0 { f(100) } else { Fixed::ONE },
        );
        assert!(close(flow[0][0], f(10)));
        assert!(close(flow[0][0] + flow[1][0], f(60)));
    }

    #[test]
    fn a_closed_pair_trades_nothing() {
        let offered = [f(50), f(50)];
        let sought = [f(40), f(40)];
        let flow = clear_good(&offered, &sought, |s, b| {
            if s == 0 && b == 1 {
                Fixed::ZERO
            } else {
                Fixed::ONE
            }
        });
        assert_eq!(flow[0][1], Fixed::ZERO);
    }

    #[test]
    fn a_preference_moves_a_sixth_of_the_way_each_month() {
        let eq = vec![vec![Fixed::ONE, milli(400)]];
        // A month ago the buyer leaned all the way to the first seller.
        let old = vec![vec![(0u16, u16::MAX)]];
        let new = step(&old, &eq, Fixed::from_ratio(1, 6));
        let table = dense(&new, 2);
        // 0 + (0.4 − 0) / 6 ≈ 0.0667 of the favourite.
        let second = table[0][1];
        assert!(
            (second - Fixed::from_ratio(1, 15)).abs() < milli(2),
            "{second:?}"
        );
        assert_eq!(table[0][0], Fixed::ONE);
    }

    #[test]
    fn with_no_preference_yet_it_starts_at_the_equilibrium() {
        let eq = vec![vec![Fixed::ONE, milli(400), milli(0)]];
        let new = step(&[], &eq, Fixed::from_ratio(1, 6));
        let table = dense(&new, 3);
        assert!((table[0][1] - milli(400)).abs() < milli(1));
        // Below the floor nothing is kept.
        assert_eq!(table[0][2], Fixed::ZERO);
        assert_eq!(new[0].len(), 2);
    }

    #[test]
    fn half_a_year_of_inertia_covers_about_two_thirds_of_a_change() {
        let eq = vec![vec![Fixed::ONE, Fixed::ONE]];
        let mut pref = vec![vec![(0u16, u16::MAX)]];
        for _ in 0..6 {
            pref = step(&pref, &eq, Fixed::from_ratio(1, 6));
        }
        let second = dense(&pref, 2)[0][1];
        // 1 − (5/6)^6 ≈ 0.665.
        assert!((second - milli(665)).abs() < milli(5), "{second:?}");
    }

    #[test]
    fn treaties_lower_the_tariff_between_their_parties_while_in_force() {
        let rel = |giver, receiver, rule| TariffRelation {
            giver,
            receiver,
            from: "1932-08-20".to_string(),
            until: Some("1947-10-30".to_string()),
            rule,
        };
        let geo = Geography {
            n: 3,
            km: vec![0; 9],
            mean: 0,
            sea: vec![false; 9],
            closures: Vec::new(),
            relations: vec![
                rel(0, 1, TariffRule::Margin(milli(100))),
                rel(0, 2, TariffRule::Union),
                rel(1, 2, TariffRule::Cap(milli(50))),
            ],
        };
        let own = milli(250);
        assert_eq!(geo.tariff(0, 1, own, "1935-01-01"), milli(150));
        assert_eq!(geo.tariff(0, 2, own, "1935-01-01"), Fixed::ZERO);
        assert_eq!(geo.tariff(1, 2, own, "1935-01-01"), milli(50));
        // Before and after the treaty, and the other way round: the buyer's own tariff.
        assert_eq!(geo.tariff(0, 1, own, "1930-01-01"), own);
        assert_eq!(geo.tariff(0, 1, own, "1950-01-01"), own);
        assert_eq!(geo.tariff(1, 0, own, "1935-01-01"), own);
    }

    fn geography_with_a_closure() -> Geography {
        Geography {
            n: 2,
            km: vec![0; 4],
            mean: 0,
            relations: Vec::new(),
            sea: vec![false; 4],
            closures: vec![Closure {
                a: 0,
                b: 1,
                from: "1940-06-10".to_string(),
                until: Some("1945-05-08".to_string()),
            }],
        }
    }

    #[test]
    fn closing_a_pair_cuts_its_trade_in_the_week_both_ways() {
        let geo = geography_with_a_closure();
        assert!(!geo.closed(0, 1, "1940-06-09"));
        assert!(geo.closed(0, 1, "1940-06-10"));
        assert!(geo.closed(1, 0, "1942-01-01"));
        assert!(!geo.closed(0, 1, "1945-05-08"));
        // Whatever the stored preference, the week's weight is zero: nothing crosses.
        let stored = Fixed::ONE;
        let weight = |s: usize, b: usize| {
            if geo.closed(s, b, "1941-01-06") {
                Fixed::ZERO
            } else {
                stored
            }
        };
        let flow = clear_good(&[Fixed::from_int(100)], &[Fixed::from_int(50)], |_, _| {
            weight(0, 1)
        });
        assert_eq!(flow[0][0], Fixed::ZERO);
    }

    #[test]
    fn reopening_does_not_bring_trade_back_at_once() {
        // Two sellers the buyer liked alike; the first is closed off for six months.
        let mut pref = vec![vec![(0u16, u16::MAX), (1u16, u16::MAX)]];
        let closed_eq = vec![vec![Fixed::ZERO, Fixed::ONE]];
        for _ in 0..6 {
            pref = step(&pref, &closed_eq, Fixed::from_ratio(1, 6));
        }
        let decayed = dense(&pref, 2)[0][0];
        // (5/6)^6 ≈ 0.335 of what it was.
        assert!((decayed - milli(335)).abs() < milli(5), "{decayed:?}");
        // The pair reopens: one month later it has come back only a sixth of the way.
        let open_eq = vec![vec![Fixed::ONE, Fixed::ONE]];
        let back = dense(&step(&pref, &open_eq, Fixed::from_ratio(1, 6)), 2)[0][0];
        let expected = decayed + (Fixed::ONE - decayed) / 6;
        assert!(
            (back - expected).abs() < milli(3),
            "{back:?} against {expected:?}"
        );
        assert!(back < milli(500));
    }

    #[test]
    fn the_whole_tree_cuts_sea_freight_to_three_tenths() {
        let none = FreightFactors::of(&[Fixed::ZERO], &[Fixed::ONE], &[Fixed::ONE]);
        assert_eq!(none.sea, Fixed::ONE);
        let all = FreightFactors::of(
            &[Fixed::ONE, Fixed::ONE],
            &[Fixed::ONE, Fixed::ONE],
            &[Fixed::ONE, Fixed::ONE],
        );
        assert!((all.sea - milli(300)).abs() < milli(1));
        // Scarce inland transport (twice the base price) doubles the land leg and the port.
        let scarce = FreightFactors::of(&[Fixed::ZERO], &[Fixed::ONE], &[Fixed::from_int(2)]);
        assert_eq!(scarce.land[0], Fixed::from_int(2));
    }
}

#[cfg(test)]
mod divisor_tests {
    use super::*;

    #[test]
    fn the_reciprocal_divides_as_the_hardware_does() {
        let mut rng = laplace_schelling::Rng::from_seed(7);
        for _ in 0..200_000 {
            let d = (rng.next_u64() >> (rng.next_u64() % 64)).max(1);
            let n = rng.next_u64() >> (rng.next_u64() % 64);
            let div = Divisor::new(d);
            assert_eq!(div.div_rem(n), (n / d, n % d), "{n} / {d}");
        }
        for d in [1u64, 2, 3, 7, 1000, u64::MAX, u64::MAX - 1] {
            let div = Divisor::new(d);
            for n in [0u64, 1, d - 1, d, d.saturating_add(1), u64::MAX] {
                assert_eq!(div.div_rem(n), (n / d, n % d), "{n} / {d}");
            }
        }
    }

    #[test]
    fn scaling_to_the_favourite_is_fixed_division() {
        let mut rng = laplace_schelling::Rng::from_seed(11);
        for _ in 0..20_000 {
            let top = Fixed::from_raw((rng.next_u64() % 1_000_000) as i64 + 1);
            let mut row: Vec<Fixed> = (0..8)
                .map(|_| Fixed::from_raw((rng.next_u64() % 2_000_000) as i64))
                .collect();
            let want: Vec<Fixed> = row.iter().map(|v| *v / top).collect();
            scale_to(&mut row, top);
            assert_eq!(row, want);
        }
    }
}

/// The tariff relations in force by pair, looked up without scanning the treaties: for the
/// weekly duties of each good (`customs::Charge`). Built once a week when duties are collected.
#[derive(Clone, Debug, Default)]
pub struct PairRules {
    n: usize,
    /// `seller × n + buyer` → index into `rules` (0: no relation).
    at: Vec<u32>,
    rules: Vec<Vec<TariffRule>>,
}

impl PairRules {
    /// From the agreements in force when they are kept (`links`), else from the treaties of
    /// `diplomacy.ron` in force on `date`, as `equilibrium` reads them.
    pub fn new(
        geo: Option<&Geography>,
        date: &str,
        links: Option<&[crate::treaty::TariffLink]>,
        n: usize,
    ) -> Self {
        let owned: Vec<TariffRelation> = links.map_or_else(Vec::new, |l| {
            l.iter().map(TariffRelation::from_link).collect()
        });
        let relations: Vec<&TariffRelation> = match links {
            Some(_) => owned.iter().collect(),
            None => geo.map_or_else(Vec::new, |g| g.relations_on(date)),
        };
        let mut at = vec![0u32; n * n];
        let mut rules: Vec<Vec<TariffRule>> = vec![Vec::new()];
        for r in relations {
            if r.giver >= n || r.receiver >= n {
                continue;
            }
            let k = r.receiver * n + r.giver;
            if at[k] == 0 {
                at[k] = u32::try_from(rules.len()).unwrap_or(u32::MAX);
                rules.push(Vec::new());
            }
            if let Some(list) = rules.get_mut(at[k] as usize) {
                list.push(r.rule);
            }
        }
        Self { n, at, rules }
    }

    /// The tariff `buyer` charges on `seller`'s goods from its own `tariff`, as `tariff_by`.
    pub fn apply(&self, seller: usize, buyer: usize, tariff: Fixed) -> Fixed {
        let k = seller * self.n + buyer;
        let Some(list) = self.at.get(k).and_then(|&i| self.rules.get(i as usize)) else {
            return tariff;
        };
        list.iter().fold(tariff, |t, r| match *r {
            TariffRule::Union => Fixed::ZERO,
            TariffRule::Margin(m) => (t - m).max(Fixed::ZERO),
            TariffRule::Cap(c) => t.min(c),
        })
    }
}
