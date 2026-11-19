//! The nuclear balance read from the forces (`docs/war.pt-BR.md`, "Arma nuclear na escada de
//! crise", step 1). Read only: it changes nothing in the world, and the crisis and war screens
//! show it before anyone escalates.
//!
//! Each country's warheads (`nuclear_warhead`) are put on its launchers (missile submarines
//! first, the most survivable, then missiles, then strategic bombers, each up to what it carries)
//! and become the [`NuclearForce`]s of [`super::nuclear`]; its missile defence and its people
//! (the unacceptable damage) complete its [`NuclearSide`]. For a pair, [`balance`] gives each
//! side's second strike, who is secure, the ceiling of each on the crisis ladder and the risk per
//! week of a direct war.
//!
//! - N3 (to the owner): an arsenal not yet tested counts `opaque_weight` for the others. No test
//!   exists yet in the game: every arsenal is opaque.
//! - N9 (to the owner): with `umbrella`, the protectors that a defence clause would call in add
//!   their forces to the protected side.
//! - Every launcher reaches (no ranges on the map yet).

use laplace_schelling::Fixed;

use super::catalogue::Military;
use super::nuclear::{
    Basing, Deterrence, EscalationLimits, NuclearForce, NuclearSide, deterrence, escalation_limits,
};
use super::state::CountryWar;
use crate::game::GameWorld;

/// One side of the balance, as the other reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct BalanceSide {
    pub country: u16,
    /// Its own warheads, and those of the protectors added (N9).
    pub warheads: u64,
    /// The countries whose forces the umbrella adds.
    pub protectors: Vec<u16>,
    /// Untested: it counts `opaque_weight` (N3).
    pub opaque: bool,
    /// Warheads delivered on it that it holds as unacceptable.
    pub unacceptable: Fixed,
    pub side: NuclearSide,
}

/// The balance of a pair: who deters whom, how far each can climb.
#[derive(Clone, Debug, PartialEq)]
pub struct Balance {
    pub sides: [BalanceSide; 2],
    pub deterrence: Deterrence,
    pub limits: EscalationLimits,
}

/// Count of the elements of `key` a country holds.
fn count(mil: &Military, c: &CountryWar, key: &str) -> u64 {
    let Some(ty) = mil.unit(key) else {
        return 0;
    };
    c.formations
        .iter()
        .filter(|f| f.unit == ty)
        .map(|f| f.elements.to_int_floor().max(0) as u64)
        .sum()
}

/// A country's forces as launchers with their warheads, and its warheads, scaled by `weight`
/// per mille (the opacity of N3).
fn forces(mil: &Military, c: &CountryWar, weight: u32) -> (Vec<NuclearForce>, u64) {
    let n = &mil.rules.nuclear;
    let scale = |x: u64| x * u64::from(weight) / 1000;
    let mut warheads = scale(count(mil, c, "nuclear_warhead"));
    let total = warheads;
    let mut out = Vec::new();
    for (key, basing, per, alert) in [
        (
            "missile_submarine",
            Basing::Submarine,
            n.warheads_per_submarine,
            n.submarine_alert,
        ),
        ("ballistic_missile", Basing::Silo, n.warheads_per_missile, 0),
        (
            "strategic_bomber",
            Basing::Bomber,
            n.warheads_per_bomber,
            n.bomber_alert,
        ),
    ] {
        let launchers = scale(count(mil, c, key));
        if launchers == 0 || warheads == 0 || per == 0 {
            continue;
        }
        // As many launchers armed as the warheads fill, each with its full load.
        let armed = launchers.min(warheads / u64::from(per)).max(1);
        let load = (warheads / armed).min(u64::from(per)) as u32;
        warheads -= armed * u64::from(load);
        out.push(NuclearForce {
            basing,
            launchers: armed,
            warheads_per_launcher: load,
            alert: Fixed::from_ratio(i64::from(alert), 1000),
            reach: Fixed::ONE,
        });
    }
    (out, total)
}

/// [`forces`] for the use (`war::nuclear_use`).
pub(crate) fn forces_of(mil: &Military, c: &CountryWar, weight: u32) -> (Vec<NuclearForce>, u64) {
    forces(mil, c, weight)
}

/// [`defense`] for the use (`war::nuclear_use`).
pub(crate) fn defense_of(mil: &Military, c: &CountryWar, incoming: u64) -> Fixed {
    defense(mil, c, incoming)
}

/// U3: (the sum of the equivalent megatons per mille of a country's warheads, their count).
pub fn warhead_emt(mil: &Military, c: &CountryWar) -> (i128, i128) {
    let Some(ty) = mil.unit("nuclear_warhead") else {
        return (0, 0);
    };
    let (mut sum, mut count) = (0i128, 0i128);
    for f in c.formations.iter().filter(|f| f.unit == ty) {
        let n = i128::from(f.elements.to_int_floor().max(0));
        sum += n * i128::from(mil.spec(ty, usize::from(f.generation)).emt_mille());
        count += n;
    }
    (sum, count)
}

/// The share of incoming warheads a country's missile defence stops against `incoming`.
fn defense(mil: &Military, c: &CountryWar, incoming: u64) -> Fixed {
    let n = &mil.rules.nuclear;
    let sites = count(mil, c, "missile_defense");
    if sites == 0 || incoming == 0 {
        return Fixed::ZERO;
    }
    let stopped = sites * u64::from(n.interceptors_per_site) * u64::from(n.interceptor_kill) / 1000;
    Fixed::from_ratio(stopped.min(incoming) as i64, incoming as i64)
}

/// The protectors a defence clause would call in if `attacker` declared on `protected` (N9).
fn protectors(world: &GameWorld, attacker: u16, protected: u16) -> Vec<u16> {
    let tag = |m: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(m))
            .map(|(_, c)| c.tag.clone())
    };
    let (Some(a), Some(p)) = (tag(attacker), tag(protected)) else {
        return Vec::new();
    };
    let mut out: Vec<u16> = super::third::who_would_join(world, &a, &p)
        .into_iter()
        .filter(|e| e.side == Some(super::third::CallSide::Defender) && !e.suspended && !e.barred)
        .filter_map(|e| {
            world
                .countries
                .iter()
                .position(|(_, c)| c.tag == e.party)
                .map(|i| i as u16)
        })
        .filter(|&m| m != attacker && m != protected)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The nuclear balance between `a` and `b` as `viewer` reads it: its own forces whole, the
/// others' untested arsenals at `opaque_weight` (N3); `None` for an outside reader. `None` when
/// neither side (with its protectors) holds a warhead.
pub fn balance(
    mil: &Military,
    world: &GameWorld,
    a: u16,
    b: u16,
    viewer: Option<u16>,
) -> Option<Balance> {
    let n = &mil.rules.nuclear;
    let empty = CountryWar::default();
    let cw = |m: u16| world.war.countries.get(usize::from(m)).unwrap_or(&empty);
    // Every arsenal but the reader's own and the tested ones is opaque (N3).
    let weight = |m: u16| {
        if Some(m) == viewer || world.war.nuclear.tested(m) {
            1000
        } else {
            n.opaque_weight
        }
    };
    let side_of = |me: u16, foe: u16| -> (Vec<NuclearForce>, u64, Vec<u16>) {
        let mut members = vec![me];
        let prot = if n.umbrella {
            protectors(world, foe, me)
        } else {
            Vec::new()
        };
        members.extend(&prot);
        let mut forces_all = Vec::new();
        let mut warheads = 0;
        for m in members {
            let (f, w) = forces(mil, cw(m), weight(m));
            forces_all.extend(f);
            warheads += w;
        }
        (forces_all, warheads, prot)
    };
    let (fa, wa, pa) = side_of(a, b);
    let (fb, wb, pb) = side_of(b, a);
    if wa == 0 && wb == 0 {
        return None;
    }
    let people = |m: u16| -> i64 {
        world
            .econ
            .regions
            .iter()
            .filter(|r| r.market == m)
            .map(|r| r.population())
            .sum()
    };
    // U3: the mean equivalent megatons of a side's warheads (its own and its protectors'),
    // per mille; one when it has none.
    let emt = |me: u16, prot: &[u16]| -> i128 {
        let (mut sum, mut count) = (0i128, 0i128);
        for m in std::iter::once(me).chain(prot.iter().copied()) {
            let (s, c) = warhead_emt(mil, cw(m));
            sum += s;
            count += c;
        }
        if count > 0 {
            (sum / count).max(1)
        } else {
            1000
        }
    };
    let (ea, eb) = (emt(a, &pa), emt(b, &pb));
    // The warheads of the other side's mean power that kill the share held unacceptable, at
    // `killed_per_warhead` a equivalent megaton.
    let unacceptable = |m: u16, foe_emt: i128| {
        let dead = i128::from(people(m)) * i128::from(n.unacceptable_share) / 1000;
        let per = (i128::from(n.killed_per_warhead.max(1)) * foe_emt / 1000).max(1);
        Fixed::from_int(((dead + per - 1) / per).max(1) as i64)
    };
    let first = |f: &[NuclearForce]| super::nuclear::first_strike_warheads(f);
    let make = |me: u16, f: Vec<NuclearForce>, incoming: u64, foe_emt: i128| NuclearSide {
        forces: f,
        defense: defense(mil, cw(me), incoming),
        command: Fixed::from_ratio(i64::from(n.command), 1000),
        unacceptable: unacceptable(me, foe_emt),
    };
    let (ia, ib) = (first(&fb), first(&fa));
    let sa = make(a, fa, ia, eb);
    let sb = make(b, fb, ib, ea);
    let d = deterrence(&mil.rules, [&sa, &sb]);
    let limits = escalation_limits(&mil.rules, &d, [wa > 0, wb > 0]);
    Some(Balance {
        sides: [
            BalanceSide {
                country: a,
                warheads: wa,
                protectors: pa,
                opaque: Some(a) != viewer && !world.war.nuclear.tested(a),
                unacceptable: sa.unacceptable,
                side: sa,
            },
            BalanceSide {
                country: b,
                warheads: wb,
                protectors: pb,
                opaque: Some(b) != viewer && !world.war.nuclear.tested(b),
                unacceptable: sb.unacceptable,
                side: sb,
            },
        ],
        deterrence: d,
        limits,
    })
}
