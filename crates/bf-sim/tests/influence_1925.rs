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

/// Blocs are read from the clauses: a pair alone is an alliance, the French web of pacts is not one
/// bloc, and the label of each pact follows its clauses.
#[test]
fn blocs_come_from_the_clauses() {
    let d = data();
    let dip = d.diplomacy.as_ref().unwrap();
    let blocs = influence::blocs_on(dip, &d.countries, "1925-01-01");
    let military: Vec<_> = blocs
        .iter()
        .filter(|b| b.kind == BlocKind::Military)
        .collect();
    // Only the Little Entente: the Estonian-Latvian pact is a pair, and France's pacts with
    // Poland, Belgium and Czechoslovakia do not tie those three to each other.
    assert_eq!(military.len(), 1, "{military:?}");
    assert_eq!(military[0].key, "little_entente");
    assert_eq!(military[0].leader, None);
    let label = |key: &str| {
        let t = dip.treaties.iter().find(|t| t.key == key).unwrap();
        influence::alliance_label(&t.clauses)
    };
    assert_eq!(
        label("cze_yug_alliance"),
        Some(influence::AllianceLabel::DefencePact)
    );
    assert_eq!(
        label("estonian_latvian_alliance"),
        Some(influence::AllianceLabel::DefencePact)
    );
    assert_eq!(label("belgium_luxembourg_union"), None);
    let cow = |key: &str| {
        let t = dip.treaties.iter().find(|t| t.key == key).unwrap();
        influence::cow_type(&t.clauses)
    };
    assert_eq!(cow("cze_yug_alliance"), Some(influence::CowType::I));
    assert_eq!(cow("belgium_luxembourg_union"), None);
    // A member who leaves: without the Yugoslav-Romanian treaty the triangle is gone, and so is
    // the bloc (a denunciation takes the member out when the treaty ends).
    let without = dip
        .treaties_on("1925-01-01")
        .filter(|t| t.key != "rom_yug_alliance");
    let after = influence::blocs_from(without, Vec::new(), Vec::new());
    assert!(after.iter().all(|b| b.key != "little_entente"));
    // The Latin Union ends on 1927-01-01.
    let later = influence::blocs_on(dip, &d.countries, "1927-06-01");
    assert!(later.iter().all(|b| b.key != "latin_monetary_union"));
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
    let dep = influence::dependence(w, rules, &influence::Ties::default(), a, b, 1925);
    assert!(matches!(dep.credit, Channel::Live(v) if v > Fixed::ZERO));
    assert!(matches!(dep.trade, Channel::Dormant(_)));
    assert!(matches!(dep.investment, Channel::Dormant(_)));
    assert!(matches!(dep.protection, Channel::Dormant(_)));
    assert!(dep.by_issue[0] > Fixed::ZERO);
    // The other way round, the creditor owes nothing it is not recorded to owe.
    assert!(
        influence::dependence(w, rules, &influence::Ties::default(), b, a, 1925)
            .credit
            .value()
            >= Fixed::ZERO
    );
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
    let dep = influence::dependence(w, rules, &influence::Ties::default(), a, b, 1925);
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
        &influence::Ties::default(),
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
    let alone = influence::dependence(&w, rules, &influence::Ties::default(), a, b, 1925)
        .credit
        .value();
    // B owes as much again to a third creditor: A's share halves, and so on.
    let mut other = debt.clone();
    other.creditor = if debt.creditor == 0 { 1 } else { 0 };
    if other.creditor == debt.debtor {
        other.creditor = 2;
    }
    w.econ.ledger.create(other);
    let shared = influence::dependence(&w, rules, &influence::Ties::default(), a, b, 1925)
        .credit
        .value();
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

/// The "Blocs" lens: countries by index, the treaties that tie each member, and the date the bloc
/// as it stands came to be (the Little Entente closes with the Romanian-Yugoslav treaty).
#[test]
fn bloc_views_for_the_map() {
    let d = data();
    let w = world();
    let views = influence::bloc_views(
        d.diplomacy.as_ref().unwrap(),
        &d.border_rules,
        w,
        Date::new(1925, 1, 1),
    );
    let le = views.iter().find(|b| b.key == "little_entente").unwrap();
    assert_eq!(le.leader, None);
    assert_eq!(le.since, Date::new(1921, 6, 7));
    let members: Vec<usize> = le.members.iter().map(|m| usize::from(m.country)).collect();
    assert_eq!(members, [ix("CZE"), ix("ROM"), ix("YUG")]);
    let cze = &le.members[0];
    assert_eq!(cze.since, Date::new(1920, 8, 14));
    assert_eq!(cze.treaties, ["cze_rom_alliance", "cze_yug_alliance"]);
    let gbr = views.iter().find(|b| b.key == "subjects:GBR").unwrap();
    assert_eq!(gbr.leader, Some(ix("GBR") as u16));
    assert!(
        gbr.members
            .iter()
            .any(|m| usize::from(m.country) == ix("NZL"))
    );
}

/// The "Influence" lens: one pass gives, for each country, what `channels` gives pair by pair.
#[test]
fn influence_over_and_on_match_the_pair_reading() {
    let d = data();
    let w = world();
    let rules = d.rules();
    let setup = rules.influence.as_deref().unwrap();
    let debt = w.econ.ledger.debts.first().unwrap();
    let (a, b) = (usize::from(debt.creditor), usize::from(debt.debtor));
    let over = influence::influence_over(w, setup, a, 1925);
    let (_, v) = over.iter().find(|(x, _)| usize::from(*x) == b).unwrap();
    let pair = influence::channels(w, setup, a, b, 1925);
    assert_eq!(*v, pair.interest.influence);
    let on = influence::influence_on(w, setup, b, 1925);
    assert!(on.iter().any(|(x, u)| usize::from(*x) == a && u == v));
    assert!(v[Issue::Money as usize] > Fixed::ZERO);
}

/// B's GDP as the influence reads it (last full year, or the running one in the first year).
fn gdp_of(w: &GameWorld, c: usize) -> Fixed {
    let m = &w.econ.markets[c];
    if m.gdp_last_year > Fixed::ZERO {
        m.gdp_last_year
    } else {
        m.gdp_running
    }
}

/// The trade channel (`docs/influence.pt-BR.md`, 13c): B's exports to A over B's GDP times A's
/// share of them, minus the same of A toward B; zero when the flows are symmetric in weight.
#[test]
fn the_trade_channel_is_the_asymmetry_of_the_flows() {
    let mut w = world().clone();
    let (a, b, c) = (ix("GER"), ix("YUG"), ix("ITA"));
    w.econ.trade = Default::default();
    w.econ.trade.fit(w.econ.markets.len());
    let gb = gdp_of(&w, b);
    assert!(gb > Fixed::ZERO);
    let tenth = gb / Fixed::from_int(10);
    // Yugoslavia sells a tenth of its GDP, all of it to Germany, and buys nothing back.
    w.econ.trade.add(b, a, tenth);
    w.econ.trade.roll_year();
    let ties = influence::Ties::of(&w, None, 1926);
    let alone = influence::trade_dependence(&w, &ties, a, b);
    assert_eq!(alone, Fixed::from_ratio(1, 10));
    assert_eq!(influence::trade_dependence(&w, &ties, b, a), Fixed::ZERO);
    // Half of it to Italy instead: Germany's share halves and so does the flow, a quarter.
    w.econ.trade.year = w.econ.trade.last_year.clone();
    let n = usize::from(w.econ.trade.n);
    w.econ.trade.year[b * n + a] = tenth / Fixed::from_int(2);
    w.econ.trade.year[b * n + c] = tenth / Fixed::from_int(2);
    w.econ.trade.roll_year();
    let ties = influence::Ties::of(&w, None, 1926);
    let shared = influence::trade_dependence(&w, &ties, a, b);
    assert_eq!(shared, Fixed::from_ratio(1, 40));
    // The live channel enters the dependence; without trade by pair it is dormant.
    let rules = data().influence.as_ref().unwrap();
    let dep = influence::dependence(&w, rules, &ties, a, b, 1926);
    assert_eq!(dep.trade, Channel::Live(shared));
    assert!(matches!(dep.investment, Channel::Dormant(_)));
    let none = influence::dependence(world(), rules, &influence::Ties::default(), a, b, 1926);
    assert!(matches!(none.trade, Channel::Dormant(_)));
}

/// The investment channel: what A owns in B over B's GDP, times A's share of B's foreign capital.
#[test]
fn the_investment_channel_weighs_the_only_owner_more() {
    let mut owned = bf_sim::foreign_stakes::OwnedShares {
        total: vec![Fixed::ZERO; 4],
        ..Default::default()
    };
    let f = Fixed::from_ratio(1, 10);
    owned.pair.insert((0, 3), f);
    owned.total[3] = f;
    assert_eq!(influence::investment_dependence(&owned, 0, 3), f);
    owned.pair.insert((1, 3), f);
    owned.total[3] = f + f;
    assert_eq!(
        influence::investment_dependence(&owned, 0, 3),
        f / Fixed::from_int(2)
    );
    assert_eq!(influence::investment_dependence(&owned, 2, 3), Fixed::ZERO);
}

/// Neutrality of the new channels: with their weights at zero the month's map is the map without
/// them, pair for pair (the bench checks the 10-year hash against the head of #113).
#[test]
fn with_zero_weights_the_new_channels_leave_the_map_as_it_was() {
    let mut w = world().clone();
    let n = w.econ.markets.len();
    w.econ.trade = Default::default();
    w.econ.trade.fit(n);
    for s in 0..n.min(40) {
        for b in 0..n.min(40) {
            if s != b {
                let v = gdp_of(&w, s) / Fixed::from_int(100);
                w.econ.trade.add(s, b, v);
            }
        }
    }
    w.econ.trade.roll_year();
    let d = data();
    let mut rules = d.influence.clone().unwrap();
    for i in &mut rules.issues {
        i.trade = 0;
        i.investment = 0;
    }
    let build = |ties: &influence::Ties| {
        influence::InfluenceMap::build(
            &w,
            &rules,
            d.community_table.as_deref(),
            &colonial(),
            &Default::default(),
            ties,
            1926,
        )
    };
    let with = build(&influence::Ties::of(&w, Some(&d.catalogue), 1926));
    let without = build(&influence::Ties::default());
    assert_eq!(with, without);
}

/// The lens reads every overlord over its subjects, measured or not: Britain over the Raj is in
/// `influence_over` (zero while its channels are dormant), not left out as "nothing measured".
#[test]
fn the_lens_reads_the_overlord_over_its_subjects() {
    let d = data();
    let w = world();
    let rules = d.rules();
    let setup = rules.influence.as_deref().unwrap();
    let gbr = ix("GBR");
    let subjects: Vec<u16> = (0..w.countries.len())
        .filter(|c| d.border_rules.overlord_of(w, *c) == Some(gbr))
        .map(|c| c as u16)
        .collect();
    assert!(!subjects.is_empty(), "Britain has subjects in 1925");
    let over = influence::influence_over(w, setup, gbr, 1925);
    for s in subjects {
        assert!(over.iter().any(|(b, _)| *b == s), "subject {s} left out");
    }
}
