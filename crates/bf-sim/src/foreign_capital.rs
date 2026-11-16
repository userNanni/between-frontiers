//! Phase 4 contract, out of the tick: capital that crosses borders, the technique it carries and
//! what a law or an expropriation does to the next flow. Design and sources in
//! `docs/late-industrialization.pt-BR.md`.
//!
//! Nothing here is scheduled or saved. These are pure functions in fixed point, with every
//! return split into named terms, so that the screen can show why a fund invests where it does:
//!
//! - [`site_return`]: the yearly return a fund expects from a level built abroad, term by term
//!   (the wage gap at the productivity its technique reaches there, freight, the law, the memory
//!   of expropriation, the relation between the two countries, the diaspora).
//! - [`allocate`]: how much of the money a fund has left after building at home goes to each
//!   site, in proportion to the excess return over home times the pull of the law, capped by what
//!   each site can take.
//! - [`expropriation_premium`]: the premium a country pays, for years, after taking foreign
//!   plants without paying for them.
//! - [`local_share`]: the share of a mixed plant the local partner holds as the contract runs.
//! - [`transfer_rate`]: how much faster a technology spreads in a country where foreign or mixed
//!   plants use it.
//! - [`remittance`]: the part of a plant's profit that leaves for the owner's country.
//!
//! The constants are design values, to be calibrated against the scorecard once the flow runs
//! (the inflow series is `data/history/fdi.csv`).

use laplace_schelling::Fixed;

use crate::econ::milli;

/// Years of the value a level adds that the level costs (`econ::monthly::level_capital`).
pub const CAPITAL_YEARS: i64 = crate::econ::monthly::CAPITAL_YEARS;

/// Premium, in yearly return, of a country whose law screens foreign capital as much as a closed
/// one would (`foreign_investment` effect at zero). A law with pull 1 or more adds nothing.
pub const LAW_PREMIUM: Fixed = milli(80);
/// Cost, in yearly return, of handing over all of a plant's technique to a local partner.
pub const TRANSFER_COST: Fixed = milli(30);
/// Premium, in yearly return, for taking all foreign capital without pay, fresh.
pub const EXPROPRIATION_PREMIUM: Fixed = milli(150);
/// Years for the memory of an expropriation to halve.
pub const EXPROPRIATION_HALF_LIFE: i64 = 10;
/// Premium, in yearly return, of investing in a country at the worst relation (-1).
pub const RELATION_PREMIUM: Fixed = milli(60);
/// Share of the risk premiums a full diaspora link takes away (it knows the place, the language
/// and whom to trust).
pub const DIASPORA_TRUST: Fixed = milli(500);
/// Most of the money left after building at home that a fund sends abroad in a month.
pub const ABROAD_CAP: Fixed = milli(500);
/// Diffusion and imitation are this much faster at home when every level of the technology's
/// buildings is foreign-owned.
pub const FOREIGN_SPILLOVER: Fixed = milli(1000);
/// And this much more when the law makes the owner hand the technique to a local partner.
pub const PARTNER_SPILLOVER: Fixed = milli(1000);

/// What the law of the destination lets a foreigner do. Built from the law effects in force.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Openness {
    /// The combined `foreign_investment` effect: 1 is an open country.
    pub pull: Fixed,
    /// The largest share of a plant a foreigner may own; zero closes the country.
    pub max_foreign_share: Fixed,
    /// Years until the plant passes to the local partner; zero means never.
    pub term_years: i64,
    /// Share of the plant's technique the partner must receive, 0 to 1.
    pub transfer: Fixed,
}

impl Openness {
    /// No foreign ownership at all.
    pub const CLOSED: Self = Self {
        pull: Fixed::ZERO,
        max_foreign_share: Fixed::ZERO,
        term_years: 0,
        transfer: Fixed::ZERO,
    };
    /// Whole ownership, no term, no transfer.
    pub const OPEN: Self = Self {
        pull: Fixed::ONE,
        max_foreign_share: Fixed::ONE,
        term_years: 0,
        transfer: Fixed::ZERO,
    };

    pub fn is_closed(&self) -> bool {
        self.max_foreign_share <= Fixed::ZERO || self.pull <= Fixed::ZERO
    }
}

/// A place a fund could build a level: one building kind in one state of another country.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Site {
    /// Destination country (market index), for the order of the allocation.
    pub dest: u16,
    /// Value a level adds in a week at productivity 1, at the prices the plant sells at.
    pub value_added: Fixed,
    /// Workers a level employs.
    pub workers: i64,
    /// Weekly market wage at the destination, in the fund's money.
    pub wage: Fixed,
    /// Productivity the fund's technique reaches there: its own technology for the building
    /// group, times the destination's base productivity (schooling, institutions) and the share
    /// of it the destination can absorb (literacy, power and transport at hand).
    pub reach: Fixed,
    /// Share of the value lost carrying the goods to the buyers (tariff and freight), 0 to 1.
    pub freight: Fixed,
    /// The destination's premium for past expropriations ([`expropriation_premium`]).
    pub expropriation: Fixed,
    /// Relation between the two countries, -1 (hostile) to 1 (allied).
    pub relation: Fixed,
    /// Diaspora link of the fund's people in the destination, 0 to 1 (the chain share of
    /// `demography::migration`, read in reverse: people of the origin living there).
    pub diaspora: Fixed,
    pub openness: Openness,
    /// Most money the site can take this month (idle hands there times capital per worker).
    pub room: Fixed,
}

/// The expected yearly return of a site, split into the terms the screen shows. Every term is in
/// yearly return (0.080 is 8 % a year); `net = gross − law − transfer − term − expropriation −
/// relation + diaspora`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SiteReturn {
    /// Unit labour cost there: wage × workers over the value a level adds at the reach.
    pub labour_cost: Fixed,
    /// `(1 − labour_cost) / CAPITAL_YEARS`: what the wage gap and the technique give.
    pub gross: Fixed,
    pub law: Fixed,
    pub transfer: Fixed,
    /// The capital paid back within the term before the plant passes to the partner.
    pub term: Fixed,
    pub expropriation: Fixed,
    pub relation: Fixed,
    /// What the diaspora takes off the risk premiums (positive).
    pub diaspora: Fixed,
    pub net: Fixed,
}

/// The yearly return a fund expects from a level at `site`.
pub fn site_return(site: &Site) -> SiteReturn {
    let value = site.value_added * (Fixed::ONE - site.freight.clamp(Fixed::ZERO, Fixed::ONE));
    let output = value * site.reach;
    if output <= Fixed::ZERO {
        return SiteReturn::default();
    }
    let labour_cost = (site.wage * site.workers / output).max(Fixed::ZERO);
    let gross = (Fixed::ONE - labour_cost) / CAPITAL_YEARS;
    let o = &site.openness;
    let law = LAW_PREMIUM * (Fixed::ONE - o.pull.min(Fixed::ONE)).max(Fixed::ZERO);
    let transfer = TRANSFER_COST * o.transfer.clamp(Fixed::ZERO, Fixed::ONE);
    let term = if o.term_years > 0 {
        Fixed::ONE / o.term_years
    } else {
        Fixed::ZERO
    };
    let expropriation = site.expropriation.max(Fixed::ZERO);
    let relation = RELATION_PREMIUM * (-site.relation).clamp(Fixed::ZERO, Fixed::ONE);
    let risk = law + expropriation + relation;
    let diaspora = risk * DIASPORA_TRUST * site.diaspora.clamp(Fixed::ZERO, Fixed::ONE);
    let net = gross - law - transfer - term - expropriation - relation + diaspora;
    SiteReturn {
        labour_cost,
        gross,
        law,
        transfer,
        term,
        expropriation,
        relation,
        diaspora,
        net,
    }
}

/// One site's share of a fund's money abroad, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub dest: u16,
    pub ret: SiteReturn,
    /// `ret.net` less the return the fund gets at home; nothing goes where this is not positive.
    pub excess: Fixed,
    /// `excess × pull`: the weight of the site in the split.
    pub weight: Fixed,
    pub amount: Fixed,
}

/// Splits `free` (the fund's money left after the builds at home) among `sites`. At most
/// [`ABROAD_CAP`] of it leaves; each site gets a part in proportion to its weight, never more
/// than its room, and what a full site cannot take goes to the others. A closed country and a
/// site that does not beat `home_return` get nothing. The result follows the order of `sites`;
/// the split does not depend on it.
pub fn allocate(free: Fixed, home_return: Fixed, sites: &[Site]) -> Vec<Placement> {
    let mut out: Vec<Placement> = sites
        .iter()
        .map(|s| {
            let ret = site_return(s);
            let excess = ret.net - home_return;
            let weight = if s.openness.is_closed() || excess <= Fixed::ZERO || s.room <= Fixed::ZERO
            {
                Fixed::ZERO
            } else {
                excess * s.openness.pull.min(Fixed::ONE)
            };
            Placement {
                dest: s.dest,
                ret,
                excess,
                weight,
                amount: Fixed::ZERO,
            }
        })
        .collect();
    let mut left = free.max(Fixed::ZERO) * ABROAD_CAP;
    let mut open: Vec<usize> = (0..out.len())
        .filter(|&i| out[i].weight > Fixed::ZERO)
        .collect();
    // Water filling: a pass gives every open site its proportional part; a site that fills up
    // leaves the next pass with what it could not take. At most one pass per site.
    while left > Fixed::ZERO && !open.is_empty() {
        let total: Fixed = open.iter().map(|&i| out[i].weight).sum();
        if total <= Fixed::ZERO {
            break;
        }
        let pass = left;
        let mut full = Vec::new();
        for &i in &open {
            let want = pass.mul_div(out[i].weight, total);
            let room = sites[i].room - out[i].amount;
            let give = want.min(room).max(Fixed::ZERO);
            out[i].amount += give;
            left -= give;
            if out[i].amount >= sites[i].room {
                full.push(i);
            }
        }
        if full.is_empty() {
            break;
        }
        open.retain(|i| !full.contains(i));
    }
    out
}

/// An expropriation a country made: how long ago, and the share of the foreign capital it held
/// that it took without paying (0 to 1; a buyout at a fair price is not one).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expropriation {
    pub years_ago: i64,
    pub share: Fixed,
}

/// `0.5 ^ (years / half_life)`, halving by whole half-lives and linear inside one.
fn decay(years: i64, half_life: i64) -> Fixed {
    if years <= 0 {
        return Fixed::ONE;
    }
    let halves = years / half_life;
    if halves >= 30 {
        return Fixed::ZERO;
    }
    let base = Fixed::from_raw(Fixed::ONE.raw() >> halves);
    let next = Fixed::from_raw(base.raw() / 2);
    base - (base - next).mul_ratio(years % half_life, half_life)
}

/// The premium a country pays in yearly return for its past expropriations: each one is
/// `share × EXPROPRIATION_PREMIUM`, halving every [`EXPROPRIATION_HALF_LIFE`] years.
pub fn expropriation_premium(history: &[Expropriation]) -> Fixed {
    history
        .iter()
        .map(|e| {
            EXPROPRIATION_PREMIUM
                * e.share.clamp(Fixed::ZERO, Fixed::ONE)
                * decay(e.years_ago, EXPROPRIATION_HALF_LIFE)
        })
        .sum()
}

/// How a mixed plant passes to the local partner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handover {
    /// The foreign owner keeps its share.
    Never,
    /// The plant passes whole to the partner when the term ends.
    AtTerm,
    /// The partner's share grows evenly to the whole plant over the term.
    Gradual,
}

/// The local partner's share of a mixed plant `elapsed` years into a contract of `term` years
/// that started with the partner holding `start`.
pub fn local_share(start: Fixed, term: i64, elapsed: i64, handover: Handover) -> Fixed {
    let start = start.clamp(Fixed::ZERO, Fixed::ONE);
    if term <= 0 {
        return start;
    }
    match handover {
        Handover::Never => start,
        Handover::AtTerm if elapsed >= term => Fixed::ONE,
        Handover::AtTerm => start,
        Handover::Gradual => start + (Fixed::ONE - start).mul_ratio(elapsed.clamp(0, term), term),
    }
}

/// How much faster a technology spreads in a country (imitation and diffusion alike) where
/// `foreign_share` of the levels of the buildings that use it belongs to foreign or mixed owners,
/// and the law makes them hand over `transfer` of the technique.
pub fn transfer_rate(foreign_share: Fixed, transfer: Fixed) -> Fixed {
    let share = foreign_share.clamp(Fixed::ZERO, Fixed::ONE);
    Fixed::ONE
        + share * FOREIGN_SPILLOVER
        + share * transfer.clamp(Fixed::ZERO, Fixed::ONE) * PARTNER_SPILLOVER
}

/// The part of `profit` that leaves for the owner's country: the foreign share of it, less what
/// the owner reinvests where the plant is.
pub fn remittance(profit: Fixed, foreign_share: Fixed, reinvested: Fixed) -> Fixed {
    if profit <= Fixed::ZERO {
        return Fixed::ZERO;
    }
    profit
        * foreign_share.clamp(Fixed::ZERO, Fixed::ONE)
        * (Fixed::ONE - reinvested.clamp(Fixed::ZERO, Fixed::ONE))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(dest: u16, wage: i64, reach: i64) -> Site {
        Site {
            dest,
            value_added: Fixed::from_int(100),
            workers: 10,
            wage: Fixed::from_int(wage),
            reach: milli(reach),
            freight: milli(50),
            expropriation: Fixed::ZERO,
            relation: Fixed::ZERO,
            diaspora: Fixed::ZERO,
            openness: Openness::OPEN,
            room: Fixed::from_int(1_000_000),
        }
    }

    /// A home plant like `site(_, wage, reach)` without freight: its return is what a fund
    /// gives up by going abroad.
    fn home(wage: i64, reach: i64) -> Fixed {
        let mut s = site(0, wage, reach);
        s.freight = Fixed::ZERO;
        site_return(&s).net
    }

    #[test]
    fn closed_law_gets_nothing() {
        let mut s = site(1, 1, 1000);
        s.openness = Openness::CLOSED;
        let p = allocate(Fixed::from_int(1000), home(8, 1000), &[s]);
        assert_eq!(p[0].amount, Fixed::ZERO);
        assert!(
            p[0].excess > Fixed::ZERO,
            "the return is there, the law shuts it"
        );
    }

    #[test]
    fn lower_wage_with_the_same_technique_draws_more() {
        let cheap = site(1, 2, 1000);
        let dear = site(2, 5, 1000);
        let p = allocate(Fixed::from_int(1000), home(8, 1000), &[cheap, dear]);
        assert!(p[0].ret.gross > p[1].ret.gross);
        assert!(p[0].amount > p[1].amount);
        assert!(p[1].amount > Fixed::ZERO);
    }

    #[test]
    fn cheap_labour_without_the_technique_draws_nothing() {
        // Wages a quarter of home, but the technique only reaches a fifth of its productivity:
        // the unit labour cost is higher, not lower.
        let p = allocate(Fixed::from_int(1000), home(8, 1000), &[site(1, 2, 200)]);
        assert_eq!(p[0].amount, Fixed::ZERO);
    }

    #[test]
    fn rich_open_country_is_not_a_destination_for_its_wages() {
        // Same wage and productivity as home: freight alone makes it worse.
        let p = allocate(Fixed::from_int(1000), home(8, 1000), &[site(1, 8, 1000)]);
        assert!(p[0].excess <= Fixed::ZERO);
        assert_eq!(p[0].amount, Fixed::ZERO);
    }

    #[test]
    fn expropriation_memory_reduces_the_flow_and_fades() {
        let clean = site(1, 2, 1000);
        let mut burnt = clean;
        burnt.dest = 2;
        let fresh = [Expropriation {
            years_ago: 0,
            share: Fixed::ONE,
        }];
        burnt.expropriation = expropriation_premium(&fresh);
        let p = allocate(Fixed::from_int(1000), home(8, 1000), &[clean, burnt]);
        assert!(p[1].amount < p[0].amount);
        assert_eq!(p[1].ret.expropriation, EXPROPRIATION_PREMIUM);

        let at = |y| {
            expropriation_premium(&[Expropriation {
                years_ago: y,
                share: Fixed::ONE,
            }])
        };
        assert_eq!(at(EXPROPRIATION_HALF_LIFE), EXPROPRIATION_PREMIUM / 2);
        assert!(at(5) < at(0) && at(5) > at(10));
        assert!(at(40) < at(20));
    }

    #[test]
    fn the_law_and_the_contract_show_as_terms() {
        let mut s = site(1, 2, 1000);
        s.openness = Openness {
            pull: milli(600),
            max_foreign_share: milli(490),
            term_years: 15,
            transfer: Fixed::ONE,
        };
        s.relation = -Fixed::ONE;
        let r = site_return(&s);
        assert_eq!(r.law, LAW_PREMIUM * milli(400));
        assert_eq!(r.transfer, TRANSFER_COST);
        assert_eq!(r.term, Fixed::ONE / 15);
        assert_eq!(r.relation, RELATION_PREMIUM);
        assert_eq!(
            r.net,
            r.gross - r.law - r.transfer - r.term - r.expropriation - r.relation + r.diaspora
        );
    }

    #[test]
    fn diaspora_takes_off_part_of_the_risk() {
        let mut s = site(1, 2, 1000);
        s.relation = -Fixed::ONE;
        let without = site_return(&s);
        s.diaspora = Fixed::ONE;
        let with = site_return(&s);
        assert_eq!(with.diaspora, RELATION_PREMIUM * DIASPORA_TRUST);
        assert!(with.net > without.net);
    }

    #[test]
    fn allocation_respects_the_cap_and_the_room_and_not_the_order() {
        let mut a = site(1, 2, 1000);
        a.room = Fixed::from_int(100);
        let b = site(2, 3, 1000);
        let c = site(3, 4, 1000);
        let free = Fixed::from_int(1000);
        let p = allocate(free, home(8, 1000), &[a, b, c]);
        let total: Fixed = p.iter().map(|x| x.amount).sum();
        assert!(total <= free * ABROAD_CAP);
        assert!(total >= free * ABROAD_CAP - milli(10));
        assert_eq!(
            p[0].amount,
            Fixed::from_int(100),
            "the full site takes its room"
        );
        let q = allocate(free, home(8, 1000), &[c, a, b]);
        assert_eq!(q[1].amount, p[0].amount);
        assert_eq!(q[2].amount, p[1].amount);
        assert_eq!(q[0].amount, p[2].amount);
    }

    #[test]
    fn mixed_plant_spreads_the_technique_faster() {
        assert_eq!(transfer_rate(Fixed::ZERO, Fixed::ONE), Fixed::ONE);
        let owned = transfer_rate(milli(200), Fixed::ZERO);
        let mixed = transfer_rate(milli(200), Fixed::ONE);
        assert!(owned > Fixed::ONE);
        assert!(mixed > owned);
    }

    #[test]
    fn the_partner_takes_the_plant_as_the_contract_says() {
        let start = milli(510);
        assert_eq!(local_share(start, 15, 30, Handover::Never), start);
        assert_eq!(local_share(start, 15, 14, Handover::AtTerm), start);
        assert_eq!(local_share(start, 15, 15, Handover::AtTerm), Fixed::ONE);
        let half = local_share(start, 10, 5, Handover::Gradual);
        assert!(half > start && half < Fixed::ONE);
        assert_eq!(local_share(start, 10, 10, Handover::Gradual), Fixed::ONE);
    }

    #[test]
    fn remittance_is_the_foreign_share_not_reinvested() {
        let r = remittance(Fixed::from_int(100), milli(490), milli(500));
        assert_eq!(r, Fixed::from_int(100) * milli(490) * milli(500));
        assert_eq!(
            remittance(-Fixed::ONE, Fixed::ONE, Fixed::ZERO),
            Fixed::ZERO
        );
    }
}
