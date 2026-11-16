//! The politics state in the running game: the 1925 laws in the world, `SetLaw`, the read API
//! of the interface, and save and reload with the new state.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use bf_sim::polity::{Enactor, possible_laws, stance};
use laplace_schelling::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn session(data: &GameData, threads: usize) -> Session {
    Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        threads,
        Some(data.rules()),
    )
}

#[test]
fn laws_commands_reads_and_saves() {
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let laws = data.laws_1925.as_ref().unwrap();
    let mut s = session(&data, 1);
    let w = s.world();
    assert_eq!(w.politics.laws, laws.laws);
    let gbr = w
        .countries
        .iter()
        .position(|(_, c)| c.tag == "GBR")
        .unwrap();
    let swe = w
        .countries
        .iter()
        .position(|(_, c)| c.tag == "SWE")
        .unwrap();
    let franchise = p.group("franchise").unwrap();
    let universal = p.law("universal_suffrage").unwrap();
    let colonial = p.groups.iter().position(|g| g.colonial).unwrap();

    // Possible laws: the group's, none for a colonial group without colonies.
    assert_eq!(
        possible_laws(p, &w.politics, gbr, franchise),
        p.groups[franchise].laws.as_slice()
    );
    assert!(possible_laws(p, &w.politics, swe, colonial).is_empty());

    // The stance on universal suffrage is the sum of its terms, and franchise is one of them.
    let current = w.politics.law_in_force(gbr, franchise).unwrap();
    assert_ne!(
        usize::from(current),
        universal,
        "GBR had a restricted franchise in 1925"
    );
    let franchise_effect = bf_sim::politics::effect_index("franchise").unwrap() as u8;
    for (ig, g) in p.interest_groups.iter().enumerate() {
        let st = stance(p, &w.politics, gbr, ig, universal);
        assert_eq!(st.total, st.terms.iter().map(|(_, v)| v).sum::<i64>());
        if g.interests.iter().any(|(e, _)| *e == franchise_effect) {
            assert!(
                st.terms.iter().any(|(e, _)| *e == franchise_effect),
                "{}",
                g.key
            );
        }
    }
    let workers = p.interest_group("workers").unwrap();
    let estates = p.interest_group("landed_estates").unwrap();
    assert!(stance(p, &w.politics, gbr, workers, universal).total > 0);
    assert!(stance(p, &w.politics, gbr, estates, universal).total < 0);
    assert_eq!(
        stance(p, &w.politics, gbr, workers, usize::from(current)).total,
        0
    );

    // A colonial law for a country without colonies, an unknown law and an unknown tag do
    // nothing; the newer of two requests for one group wins.
    let colonial_law = p.groups[colonial].laws[0];
    s.submit(GameCommand::SetLaw {
        tag: "SWE".into(),
        law: colonial_law,
    });
    s.submit(GameCommand::SetLaw {
        tag: "GBR".into(),
        law: u16::MAX,
    });
    s.submit(GameCommand::SetLaw {
        tag: "XXX".into(),
        law: 0,
    });
    let other = p.groups[franchise]
        .laws
        .iter()
        .copied()
        .find(|&l| l != current && usize::from(l) != universal)
        .unwrap();
    s.submit(GameCommand::SetLaw {
        tag: "GBR".into(),
        law: other,
    });
    s.submit(GameCommand::SetLaw {
        tag: "GBR".into(),
        law: universal as u16,
    });
    s.sim.run_until(Date::new(1925, 1, 3));
    let w = s.world();
    assert_eq!(
        w.politics.law_in_force(gbr, franchise),
        Some(universal as u16)
    );
    assert!(w.politics.law_in_force(swe, colonial).is_none());
    assert_eq!(w.politics.changes.len(), 1);
    let rec = &w.politics.changes[0];
    assert_eq!(
        (rec.country as usize, rec.from, rec.to),
        (gbr, current, universal as u16)
    );
    assert_eq!(rec.by, Enactor::Command);
    assert!(w.politics.pending.is_empty());

    // Clout is measured on the first of the month and adds up to about 1000.
    s.sim.run_until(Date::new(1925, 2, 2));
    let clout = s.world().politics.clout(gbr);
    let total: u32 = clout.iter().map(|&c| u32::from(c)).sum();
    assert!((990..=1000).contains(&total), "GBR clout {clout:?}");
    assert!(clout[workers] > clout[estates], "GBR clout {clout:?}");

    // Save and reload keep the state and run on identically, with another thread count.
    let bytes = s.save("test", "").unwrap();
    let mut loaded = Session::load(&bytes, 2, Some(data.rules())).unwrap();
    assert_eq!(loaded.world().politics, s.world().politics);
    s.submit(GameCommand::SetLaw {
        tag: "SWE".into(),
        law: universal as u16,
    });
    loaded.submit(GameCommand::SetLaw {
        tag: "SWE".into(),
        law: universal as u16,
    });
    s.sim.run_until(Date::new(1925, 3, 2));
    loaded.sim.run_until(Date::new(1925, 3, 2));
    assert_eq!(loaded.state_hash(), s.state_hash());
}

#[test]
fn laws_reach_the_economy_and_regimes_change() {
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let mut s = session(&data, 2);
    let w = s.world();
    let government = p.group("government").unwrap();
    let taxation = p.group("taxation").unwrap();
    let civil_service = p.group("civil_service").unwrap();

    // The form of government in force is the scenario's.
    for (c, def) in data.countries.countries.iter().enumerate() {
        let l = w.politics.law_in_force(c, government).unwrap();
        assert_eq!(p.laws[usize::from(l)].key, def.government, "{}", def.tag);
    }

    // A country under indirect taxation and patronage: its upper rate is held at 8 % and only
    // 70 % of the assessed tax reaches the treasury, from the first day.
    let law_key = |c: usize, g: usize| {
        p.laws[usize::from(w.politics.law_in_force(c, g).unwrap())]
            .key
            .clone()
    };
    let c = (0..w.politics.laws.len())
        .find(|&c| {
            law_key(c, taxation) == "indirect_taxation" && law_key(c, civil_service) == "patronage"
        })
        .expect("a country with indirect taxation and patronage");
    let free = (0..w.politics.laws.len())
        .find(|&c| law_key(c, civil_service) == "merit_service")
        .unwrap();
    s.sim.run_until(Date::new(1925, 1, 2));
    let w = s.world();
    let m = &w.econ.markets[c];
    let e = &m.laws.effects[0];
    assert_eq!(e.tax_caps[2], bf_sim::econ::milli(80));
    assert!(m.budget.tax_rates[2] <= bf_sim::econ::milli(80));
    assert_eq!(e.tax_collection, bf_sim::econ::milli(700));
    assert_eq!(
        w.econ.markets[free].laws.effects[0].tax_collection,
        laplace_schelling::Fixed::ONE
    );

    // After a month the groups see the tax a ceiling lets the state take, the consumption tax
    // included: lifting the ceilings moves the tax off what the peasants buy and onto the top
    // incomes, and those paid by the state gain from the revenue.
    s.sim.run_until(Date::new(1925, 2, 2));
    let w = s.world();
    let progressive = p.law("progressive_income_tax").unwrap();
    let st = |key: &str| {
        stance(
            p,
            &w.politics,
            c,
            p.interest_group(key).unwrap(),
            progressive,
        )
    };
    assert!(st("peasantry").total > 0);
    assert!(w.econ.markets[c].budget.indirect_rate > laplace_schelling::Fixed::ZERO);
    assert!(st("business").total < 0);
    assert!(st("officialdom").total > 0);

    // A regime falls by command like any law.
    let tag = w.countries.iter().nth(c).unwrap().1.tag.clone();
    let one_party = p.law("one_party_state").unwrap() as u16;
    s.submit(GameCommand::SetLaw {
        tag,
        law: one_party,
    });
    s.sim.run_until(Date::new(1925, 2, 4));
    assert_eq!(
        s.world().politics.law_in_force(c, government),
        Some(one_party)
    );
}
