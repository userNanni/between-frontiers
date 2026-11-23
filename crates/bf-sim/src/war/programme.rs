//! The nuclear programme (`docs/war.pt-BR.md`, "Arma nuclear na escada de crise", step 2;
//! questions N1 and N2, open). Behind `Switches::war`.
//!
//! A government sets a share of its GDP a year for the "Project" (`GameCommand::
//! NuclearProgramme`). The money is spent every week, outside the ceiling of the military
//! budget (it is a programme on top of it):
//!
//! - **Research.** Until the bomb can be built, it buys the goods of a laboratory (the
//!   programme's basket). What the market delivers is credited, each month, to the first
//!   technology of the chain still to be understood or prototyped (fission, enrichment, the
//!   bomb, the thermonuclear weapon), at `dollars_per_cost` a point of its cost, with the
//!   research's own penalty for being ahead of history. From prototype to industry the
//!   programme is the state buyer at `buyer`.
//! - **Plants.** Once enrichment is industrial, the programme asks for a level of the enrichment
//!   plant in its most populous metropolitan state, and another whenever the market delivers
//!   less than `plant_shortage` of the fissile material it asks for. The works queue builds them,
//!   paid by the treasury.
//! - **Warheads.** Once the bomb can be built, `stock_share` of the money keeps an order of
//!   warheads (A2) going: it buys their equipment, the fissile material first, from the same
//!   market as the power plants. The fissile material on the market limits the stock.
//!
//! N2: weapons (`weapons`) are imitated at `imitation` of the usual pace; sharing by a clause
//! between allies waits for the treaties' clause.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

/// `nuclear_programme` in `data/military/peace.ron`. Per mille unless named otherwise.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgrammeRule {
    /// The technologies the programme pushes, in order.
    pub chain: Vec<String>,
    /// N2: the weapons among them, imitated at `imitation` of the usual pace.
    pub weapons: Vec<String>,
    pub imitation: u32,
    /// The goods the research money buys, with their weights.
    pub basket: Vec<(String, u32)>,
    /// Dollars (base prices) delivered per point of a technology's cost.
    pub dollars_per_cost: u32,
    /// The state as buyer of the chain from prototype to industry.
    pub buyer: u32,
    /// Of the money, the share that keeps the warhead order going once the bomb can be built.
    pub stock_share: u32,
    /// The highest share of GDP a year a government may set.
    pub max_share: u32,
    /// The enrichment plant, and the delivered share of the fissile material below which the
    /// programme asks for another level.
    pub plant: String,
    pub plant_shortage: u32,
    /// Weeks between two requests of a plant (or of a mine).
    pub plant_weeks: u16,
    /// U1 (the owner, 2026-11-19): the mine the programme opens with the state's capital, in a
    /// deposit already known, at home first and then in a colony, when the market delivers less
    /// than `mine_shortage` per mille of the ore its plants ask for. Abroad (by the capital
    /// between countries) waits for that rule.
    #[serde(default)]
    pub mine: Option<String>,
    #[serde(default = "default_mine_shortage")]
    pub mine_shortage: u32,
    /// The assembly of the warhead (the missing step between the core and the weapon): the
    /// plant that makes the non-nuclear assembly of a warhead (the lenses, the initiator, the
    /// fuzing and the case), which the programme asks for once its core plants run, and again
    /// when the market delivers less than `assembly_shortage` per mille of what its warhead
    /// order asks for.
    #[serde(default)]
    pub assembly: Option<String>,
    #[serde(default = "default_mine_shortage")]
    pub assembly_shortage: u32,
    pub warhead: String,
    /// Warheads ordered at a time.
    pub batch: u32,
}

fn default_mine_shortage() -> u32 {
    900
}

/// The programme's technologies, resolved against the tree at load.
#[derive(Clone, Debug)]
pub struct ProgrammeSetup {
    pub rule: ProgrammeRule,
    /// Tree indices of `chain` and of `weapons`.
    pub chain: Vec<usize>,
    pub weapons: Vec<usize>,
}

impl ProgrammeSetup {
    pub fn new(rule: ProgrammeRule, tree: &crate::research::TechTree) -> Self {
        let ix = |keys: &[String]| -> Vec<usize> {
            keys.iter()
                .filter_map(|k| tree.techs.iter().position(|t| t.key == *k))
                .collect()
        };
        Self {
            chain: ix(&rule.chain),
            weapons: ix(&rule.weapons),
            rule,
        }
    }
}

/// One country's programme (`WarState::nuclear`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Programme {
    pub country: u16,
    /// Share of GDP a year, per mille.
    pub share: u16,
    /// (year, month, day) it started.
    pub since: (i32, u8, u8),
    /// Money delivered so far (base prices), research and warheads.
    pub spent: Fixed,
    /// Research money delivered, the last four weeks or so (each week adds what was delivered
    /// and keeps three quarters of the rest): the month's credit.
    pub month: Fixed,
    /// What the programme buys this week: the laboratory's goods and the warheads' equipment.
    pub buying: Vec<(u16, Fixed)>,
    /// Of `buying`, the laboratory's goods (the research credit).
    pub research: Vec<(u16, Fixed)>,
    /// Weeks until it may ask for another plant.
    pub plant_wait: u16,
}

/// The programmes under way, by country.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NuclearState {
    pub programmes: Vec<Programme>,
    /// N3: the countries that tested a weapon, sorted: their arsenal is known to all.
    pub tested: Vec<u16>,
    /// Tests asked for by command, checked the next week against the warheads held.
    pub test_requests: Vec<u16>,
    /// (demander, target) of the crises this week whose demander's ceiling against the target
    /// is below war (`war::balance`): the declaration is blocked. Sorted.
    pub blocked: Vec<(u16, u16)>,
    /// N7: the taboo, per mille: grows with the years without use, falls at each use.
    pub taboo: u32,
    /// (year, month, day) of the last use in the world.
    pub last_use: Option<(i32, u8, u8)>,
    /// The dead of the strikes each country received, sorted: they weigh on its support for the
    /// war (the casualties term).
    pub dead: Vec<(u16, u64)>,
    /// Uses asked for by command or by the AI, and the answers (N5), for the next week.
    pub uses: Vec<super::nuclear_use::Use>,
    /// N8: (region, per mille of its productivity lost to the fallout), sorted.
    pub fallout: Vec<(u32, u32)>,
}

impl NuclearState {
    pub fn is_empty(&self) -> bool {
        self.programmes.is_empty()
            && self.tested.is_empty()
            && self.test_requests.is_empty()
            && self.blocked.is_empty()
            && !self.used()
            && self.fallout.is_empty()
    }

    /// Whether anything of the use is held: a world without it hashes as before.
    fn used(&self) -> bool {
        self.taboo != 0 || self.last_use.is_some() || !self.dead.is_empty() || !self.uses.is_empty()
    }

    /// The fallout of `region`, per mille of its productivity (N8).
    pub fn fallout_of(&self, region: u32) -> u32 {
        self.fallout
            .binary_search_by_key(&region, |(r, _)| *r)
            .map_or(0, |i| self.fallout[i].1)
    }

    /// Sets `region`'s fallout; 0 removes it.
    pub fn set_fallout(&mut self, region: u32, level: u32) {
        match self.fallout.binary_search_by_key(&region, |(r, _)| *r) {
            Ok(i) if level == 0 => {
                self.fallout.remove(i);
            }
            Ok(i) => self.fallout[i].1 = level,
            Err(i) if level > 0 => self.fallout.insert(i, (region, level)),
            Err(_) => {}
        }
    }

    /// The dead of the strikes `country` received.
    pub fn dead_of(&self, country: u16) -> u64 {
        self.dead
            .binary_search_by_key(&country, |(c, _)| *c)
            .map_or(0, |i| self.dead[i].1)
    }

    pub fn add_dead(&mut self, country: u16, n: u64) {
        if n == 0 {
            return;
        }
        match self.dead.binary_search_by_key(&country, |(c, _)| *c) {
            Ok(i) => self.dead[i].1 += n,
            Err(i) => self.dead.insert(i, (country, n)),
        }
    }

    /// Whether `country` tested a weapon (N3).
    pub fn tested(&self, country: u16) -> bool {
        self.tested.binary_search(&country).is_ok()
    }

    /// Whether `by`'s declaration on `target` is blocked by the nuclear ceiling.
    pub fn blocked(&self, by: u16, target: u16) -> bool {
        self.blocked.binary_search(&(by, target)).is_ok()
    }

    pub fn of(&self, country: u16) -> Option<&Programme> {
        self.programmes.iter().find(|p| p.country == country)
    }

    /// Sets `country`'s share (per mille of GDP a year), or ends its programme with `None` or 0.
    pub fn set(&mut self, country: u16, share: Option<u16>, since: (i32, u8, u8)) {
        match share.filter(|&s| s > 0) {
            Some(s) => match self.programmes.iter_mut().find(|p| p.country == country) {
                Some(p) => p.share = s,
                None => {
                    self.programmes.push(Programme {
                        country,
                        share: s,
                        since,
                        ..Default::default()
                    });
                    self.programmes.sort_by_key(|p| p.country);
                }
            },
            None => self.programmes.retain(|p| p.country != country),
        }
    }
}

fn hash_goods(h: &mut StateHasher, list: &[(u16, Fixed)]) {
    h.write_len(list.len());
    for (g, q) in list {
        h.write_u16(*g);
        q.hash_state(h);
    }
}

impl HashState for NuclearState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.programmes.len());
        for p in &self.programmes {
            h.write_u16(p.country);
            h.write_u16(p.share);
            h.write_i64(i64::from(p.since.0));
            h.write_u8(p.since.1);
            h.write_u8(p.since.2);
            p.spent.hash_state(h);
            p.month.hash_state(h);
            hash_goods(h, &p.buying);
            hash_goods(h, &p.research);
            h.write_u16(p.plant_wait);
        }
        // Only when any: a world without tests hashes as before them.
        if !self.tested.is_empty() || !self.test_requests.is_empty() || !self.blocked.is_empty() {
            h.write_len(self.tested.len());
            for c in &self.tested {
                h.write_u16(*c);
            }
            h.write_len(self.test_requests.len());
            for c in &self.test_requests {
                h.write_u16(*c);
            }
            h.write_len(self.blocked.len());
            for (a, b) in &self.blocked {
                h.write_u16(*a);
                h.write_u16(*b);
            }
        }
        // Only when any: a world without a use hashes as before it.
        if self.used() {
            h.write_u32(self.taboo);
            match self.last_use {
                Some((y, m, d)) => {
                    h.write_u8(1);
                    h.write_i64(i64::from(y));
                    h.write_u8(m);
                    h.write_u8(d);
                }
                None => h.write_u8(0),
            }
            h.write_len(self.dead.len());
            for (c, n) in &self.dead {
                h.write_u16(*c);
                h.write_u64(*n);
            }
            h.write_len(self.uses.len());
            for u in &self.uses {
                h.write_u16(u.by);
                h.write_u16(u.target);
                h.write_u8(u.mode as u8);
                h.write_u32(u.warheads);
                h.write_u8(u8::from(u.answer));
            }
        }
        // Only when any: a world without fallout hashes as before it.
        if !self.fallout.is_empty() {
            h.write_len(self.fallout.len());
            for (r, l) in &self.fallout {
                h.write_u32(*r);
                h.write_u32(*l);
            }
        }
    }
}

/// [`NuclearState`] as version 39 of the world wrote it, before the fallout.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct NuclearStateBeforeFallout {
    programmes: Vec<Programme>,
    tested: Vec<u16>,
    test_requests: Vec<u16>,
    blocked: Vec<(u16, u16)>,
    taboo: u32,
    last_use: Option<(i32, u8, u8)>,
    dead: Vec<(u16, u64)>,
    uses: Vec<super::nuclear_use::Use>,
}

impl From<NuclearStateBeforeFallout> for NuclearState {
    fn from(n: NuclearStateBeforeFallout) -> Self {
        Self {
            programmes: n.programmes,
            tested: n.tested,
            test_requests: n.test_requests,
            blocked: n.blocked,
            taboo: n.taboo,
            last_use: n.last_use,
            dead: n.dead,
            uses: n.uses,
            fallout: Vec::new(),
        }
    }
}

/// [`NuclearState`] as version 38 of the world wrote it, before the use.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct NuclearStateBeforeUse {
    programmes: Vec<Programme>,
    tested: Vec<u16>,
    test_requests: Vec<u16>,
    blocked: Vec<(u16, u16)>,
}

impl From<NuclearStateBeforeUse> for NuclearState {
    fn from(n: NuclearStateBeforeUse) -> Self {
        Self {
            programmes: n.programmes,
            tested: n.tested,
            test_requests: n.test_requests,
            blocked: n.blocked,
            ..Default::default()
        }
    }
}

/// [`NuclearState`] as version 37 of the world wrote it, before the tests and the crisis rung.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct NuclearStateBeforeTests {
    programmes: Vec<Programme>,
}

impl From<NuclearStateBeforeTests> for NuclearState {
    fn from(n: NuclearStateBeforeTests) -> Self {
        Self {
            programmes: n.programmes,
            ..Default::default()
        }
    }
}

/// The money a programme of `share` per mille spends in a week of a year of `gdp`.
pub fn weekly_money(gdp: Fixed, share: u16) -> Fixed {
    gdp.max(Fixed::ZERO).mul_ratio(i64::from(share), 1000) / 52
}

/// Spreads `money` over a basket of (good, weight) at `prices`: what it buys of each.
pub fn basket_buy(basket: &[(u16, u32)], money: Fixed, prices: &[Fixed]) -> Vec<(u16, Fixed)> {
    let total: i64 = basket
        .iter()
        .filter(|(g, _)| prices[*g as usize].is_positive())
        .map(|(_, w)| i64::from(*w))
        .sum();
    if total == 0 || !money.is_positive() {
        return Vec::new();
    }
    basket
        .iter()
        .filter(|(g, _)| prices[*g as usize].is_positive())
        .map(|(g, w)| {
            (
                *g,
                money.mul_ratio(i64::from(*w), total) / prices[*g as usize],
            )
        })
        .filter(|(_, q)| q.is_positive())
        .collect()
}
