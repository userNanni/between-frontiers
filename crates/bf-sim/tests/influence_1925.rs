//! Influence and blocs on the 1925 world (`docs/influence.pt-BR.md`), out of the tick: the blocs
//! the scenario holds, credit as the one live channel, affinity of regime and people, and the
//! dormant channels named as dormant.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameWorld, Session};
use bf_sim::influence::{self, BlocKind, Channel, Goodwill, Issue};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

/// A month in: GDP, community rows and the ledger are all filled.
fn world() -> &'static GameWorld {
    static WORLD: OnceLock<GameWorld> = OnceLock::new();
    WORLD.get_or_init(|| {
        let d = data();
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        s.sim.run_until(Date::new(1925, 2, 1));
        s.world().clone()
    })
}

fn ix(tag: &str) -> usize {
    world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .unwrap()
}

fn colonial() -> Vec<bool> {
    data()
        .politics
        .as_ref()
        .map(|p| p.groups.iter().map(|g| g.colonial).collect())
        .unwrap_or_default()
}

#[test]
fn the_blocs_of_1925() {
    let d = data();
    let blocs = influence::blocs_on(d.diplomacy.as_ref().unwrap(), &d.countries, "1925-01-01");
    let find = |key: &str| {
        blocs
            .iter()
            .find(|b| b.key == key)
            .unwrap_or_else(|| panic!("{key}"))
    };
    let le = find("little_entente");
    assert_eq!(le.kind, BlocKind::Military);
    assert_eq!(le.members, ["CZE", "ROM", "YUG"]);
    assert_eq!(find("latin_monetary_union").kind, BlocKind::Monetary);
    let bel = find("customs:BEL");
    assert_eq!(bel.leader.as_deref(), Some("BEL"));
    assert_eq!(bel.members, ["BEL", "LUX"]);
    let fra = find("customs:FRA");
    assert!(fra.members.iter().any(|m| m == "MCO") && fra.members.iter().any(|m| m == "SAA"));
    let gbr = find("subjects:GBR");
    assert_eq!(gbr.kind, BlocKind::Subjects);
    for dominion in ["CAN", "AUS", "NZL", "SAF", "IRL"] {
        assert!(gbr.members.iter().any(|m| m == dominion), "{dominion}");
    }
    // Organizations open to all are not blocs.
    assert!(
        blocs
            .iter()
            .all(|b| b.key != "league_of_nations" && b.key != "pan_american_union")
    );
}

#[test]
fn credit_is_live_and_the_rest_dormant() {
    let w = world();
    let rules = data().influence.as_ref().unwrap();
    // Some government owes another in the 1925 ledger (the war debts).
    let d = w
        .econ
        .ledger
        .debts
        .first()
        .expect("the 1925 ledger has lines");
    let (a, b) = (usize::from(d.creditor), usize::from(d.debtor));
    let dep = influence::dependence(w, rules, a, b, 1925);
    assert!(matches!(dep.credit, Channel::Live(v) if v > Fixed::ZERO));
    assert!(matches!(dep.trade, Channel::Dormant(_)));
    assert!(matches!(dep.investment, Channel::Dormant(_)));
    assert!(matches!(dep.protection, Channel::Dormant(_)));
    assert!(dep.by_issue[0] > Fixed::ZERO);
    // The other way round, the creditor owes nothing it is not recorded to owe.
    assert!(influence::dependence(w, rules, b, a, 1925).credit.value() >= Fixed::ZERO);
}

#[test]
fn affinity_of_regime_and_people() {
    let w = world();
    let d = data();
    let rules = d.influence.as_ref().unwrap();
    let table = d.community_table.as_deref();
    let cultures = influence::main_cultures(w, table.expect("communities"));
    let col = colonial();
    let aff =
        |a: &str, b: &str| influence::affinity(w, rules, table, &cultures, &col, ix(a), ix(b));
    let (gbr_nzl, gbr_jap) = (aff("GBR", "NZL"), aff("GBR", "JAP"));
    assert!(gbr_nzl.people > gbr_jap.people, "{gbr_nzl:?} {gbr_jap:?}");
    // Regime likeness is symmetric and a share.
    let (x, y) = (aff("FRA", "BEL"), aff("BEL", "FRA"));
    assert_eq!(x.regime, y.regime);
    assert!(x.regime >= Fixed::ZERO && x.regime <= Fixed::ONE);
    // The factor stays between the floor and the ceiling.
    for a in [gbr_nzl, gbr_jap, x, y] {
        assert!(a.factor >= Fixed::from_raw(i64::from(rules.affinity_floor)));
        assert!(a.factor <= Fixed::from_raw(i64::from(rules.affinity_ceiling)));
    }
}

#[test]
fn influence_follows_credit_and_goodwill() {
    let w = world();
    let d = data();
    let rules = d.influence.as_ref().unwrap();
    let table = d.community_table.as_deref();
    let cultures = influence::main_cultures(w, table.unwrap());
    let col = colonial();
    let debt = w.econ.ledger.debts.first().unwrap();
    let (a, b) = (usize::from(debt.creditor), usize::from(debt.debtor));
    let aff = influence::affinity(w, rules, table, &cultures, &col, a, b);
    let dep = influence::dependence(w, rules, a, b, 1925);
    let without = influence::interest(dep, Goodwill::default(), aff);
    let mut g = Goodwill::default();
    g.add(Fixed::from_raw(10));
    let with = influence::interest(dep, g, aff);
    assert!(without.influence[0] > Fixed::ZERO);
    assert!(with.influence[0] > without.influence[0]);
}

#[test]
fn the_map_holds_the_ledger_pairs_and_caps_the_term() {
    let w = world();
    let d = data();
    let rules = d.influence.as_ref().unwrap();
    let map = influence::InfluenceMap::build(
        w,
        rules,
        d.community_table.as_deref(),
        &colonial(),
        &Default::default(),
        1925,
    );
    let debt = w.econ.ledger.debts.first().unwrap();
    let worth = map.worth[&(debt.creditor, debt.debtor)];
    assert!(worth[0] > Fixed::ZERO);
    // Credit weighs more in money than in security (influence by issue).
    assert!(worth[0] > worth[3]);
    let small = Fixed::from_int(1);
    assert!(map.proposal_term(debt.creditor, debt.debtor, Issue::Money, small) <= small);
    // A pair with no channel and no goodwill is not in the map: its term is zero.
    let (bra, arg) = (ix("BRA") as u16, ix("ARG") as u16);
    assert_eq!(
        map.proposal_term(bra, arg, Issue::Money, Fixed::from_int(1_000_000)),
        Fixed::ZERO
    );
}

/// Vulnerability: the same debt weighs more on a debtor who owes nobody else.
#[test]
fn credit_is_vulnerability_not_size() {
    let mut w = world().clone();
    let rules = data().influence.as_ref().unwrap();
    let debt = w.econ.ledger.debts[0].clone();
    let (a, b) = (usize::from(debt.creditor), usize::from(debt.debtor));
    let alone = influence::dependence(&w, rules, a, b, 1925).credit.value();
    // B owes as much again to a third creditor: A's share halves, and so on.
    let mut other = debt.clone();
    other.creditor = if debt.creditor == 0 { 1 } else { 0 };
    if other.creditor == debt.debtor {
        other.creditor = 2;
    }
    w.econ.ledger.create(other);
    let shared = influence::dependence(&w, rules, a, b, 1925).credit.value();
    assert!(shared < alone, "{shared:?} {alone:?}");
}

#[test]
fn the_missions_of_1925_by_rule() {
    use influence::Mission;
    let w = world();
    let d = data();
    let service = &d.influence.as_ref().unwrap().service;
    let m = influence::missions_1925(w, service, &d.border_rules);
    let pair = |a: &str, b: &str| {
        let (a, b) = (ix(a) as u16, ix(b) as u16);
        m.get(&(a.min(b), a.max(b))).copied()
    };
    assert_eq!(pair("GBR", "FRA"), Some(Mission::Embassy));
    assert_eq!(pair("GBR", "NZL"), Some(Mission::Legation), "overlord");
    assert_eq!(pair("BRA", "ARG"), Some(Mission::Legation), "neighbours");
    assert_eq!(pair("NZL", "PRY"), None, "far apart and small");
    // A great power serves more negotiations at once than a small country.
    let slots = |t: &str| influence::negotiation_slots(w, service, &m, ix(t));
    assert!(
        slots("GBR") > slots("LUX"),
        "{} {}",
        slots("GBR"),
        slots("LUX")
    );
    assert!(slots("LUX") >= 1);
    // Without a mission an act yields less.
    let (nzl, pry) = (ix("NZL") as u16, ix("PRY") as u16);
    assert!(influence::act_yield(&m, service, nzl, pry) < Fixed::ONE);
}

#[test]
fn rapprochement_with_an_opposite_regime_costs_more_at_home() {
    let w = world();
    let d = data();
    let p = d.politics.as_ref().unwrap();
    let service = &d.influence.as_ref().unwrap().service;
    let cost = |a: &str, b: &str| influence::domestic_cost(w, p, service, ix(a), ix(b));
    assert!(
        cost("USA", "SOV") > cost("USA", "CAN"),
        "{:?} {:?}",
        cost("USA", "SOV"),
        cost("USA", "CAN")
    );
    assert_eq!(cost("USA", "USA"), Fixed::ZERO);
}
