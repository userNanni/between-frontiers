//! The politics catalogue (laws, interest groups, movements) and the laws in force in 1925: no
//! broken reference, no law without a group, no country without a law in each group, no unknown
//! effect, names in every language, and stances that follow from the effects.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::politics::{
    Bound, CATEGORIES, EFFECTS, Goal, INDICATORS, InterestGroupsFile, LawsFile, MovementsFile,
    Politics, PoliticsError, ScenarioLawsFile, Scope,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(path: &str) -> T {
    let text = std::fs::read_to_string(root().join(path)).unwrap();
    ron::from_str(&text).unwrap()
}

struct Files {
    laws: LawsFile,
    groups: InterestGroupsFile,
    movements: MovementsFile,
    scenario: ScenarioLawsFile,
}

fn files() -> Files {
    Files {
        laws: read("data/politics/laws.ron"),
        groups: read("data/politics/interest_groups.ron"),
        movements: read("data/politics/movements.ron"),
        scenario: read("data/scenario/1925/laws.ron"),
    }
}

fn build(f: &Files) -> Result<Politics, PoliticsError> {
    Politics::build(&f.laws, &f.groups, &f.movements)
}

fn resolve(f: &Files) -> Result<(), PoliticsError> {
    let data = GameData::load(&root()).unwrap();
    build(f)?
        .resolve(&f.scenario, &data.countries.countries)
        .map(|_| ())
}

#[test]
fn catalogue_loads_and_every_country_holds_one_law_per_group() {
    let data = GameData::load(&root()).unwrap();
    let politics = data.politics.as_ref().expect("data/politics/laws.ron");
    let laws = data
        .laws_1925
        .as_ref()
        .expect("data/scenario/1925/laws.ron");
    assert_eq!(laws.laws.len(), data.countries.countries.len());
    for (c, country) in data.countries.countries.iter().enumerate() {
        for (g, group) in politics.groups.iter().enumerate() {
            let held = laws.laws[c][g];
            let expected = !group.colonial || country.colonies.is_some();
            assert_eq!(held.is_some(), expected, "{} {}", country.tag, group.key);
            if let Some(l) = held {
                assert_eq!(usize::from(politics.laws[usize::from(l)].group), g);
            }
        }
    }
}

#[test]
fn every_effect_is_used_and_every_category_has_a_group() {
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    for (e, kind) in EFFECTS.iter().enumerate() {
        assert!(
            p.laws
                .iter()
                .any(|l| l.effects.iter().any(|(x, _)| usize::from(*x) == e)),
            "no law changes {}",
            kind.key
        );
    }
    for (c, cat) in CATEGORIES.iter().enumerate() {
        assert!(
            p.groups.iter().any(|g| usize::from(g.category) == c),
            "category {cat} is empty"
        );
    }
}

#[test]
fn a_law_with_an_unknown_group_is_refused() {
    let mut f = files();
    f.laws.laws[0].group = "nowhere".into();
    assert!(matches!(
        build(&f),
        Err(PoliticsError::UnknownGroup { group, .. }) if group == "nowhere"
    ));
}

#[test]
fn a_group_without_a_choice_is_refused() {
    let mut f = files();
    let g = f.laws.groups[0].key.clone();
    let first = f
        .laws
        .laws
        .iter()
        .find(|l| l.group == g)
        .unwrap()
        .key
        .clone();
    f.laws.laws.retain(|l| l.group != g || l.key == first);
    assert_eq!(build(&f).unwrap_err(), PoliticsError::NoChoice(g));
}

#[test]
fn unknown_effects_are_refused_everywhere() {
    let mut f = files();
    f.laws.laws[0].effects.push(("happiness".into(), 100));
    assert!(matches!(
        build(&f),
        Err(PoliticsError::UnknownEffect { effect, .. }) if effect == "happiness"
    ));

    let mut f = files();
    f.groups.groups[0].interests.push(("happiness".into(), 1));
    assert!(matches!(
        build(&f),
        Err(PoliticsError::UnknownEffect { effect, .. }) if effect == "happiness"
    ));

    let mut f = files();
    f.movements.movements[0]
        .cause
        .push(("happiness".into(), Bound::AtLeast(10)));
    assert!(matches!(
        build(&f),
        Err(PoliticsError::UnknownMeasure { measure, .. }) if measure == "happiness"
    ));
}

#[test]
fn out_of_range_and_misplaced_effects_are_refused() {
    let mut f = files();
    let law = f
        .laws
        .laws
        .iter_mut()
        .find(|l| l.key == "internal_passports")
        .unwrap();
    law.effects[0].1 = 9000;
    assert!(matches!(
        build(&f),
        Err(PoliticsError::EffectOutOfRange { .. })
    ));

    // Colonial effects only in colonial groups.
    let mut f = files();
    f.laws.laws[0]
        .effects
        .push(("colonial_tribute".into(), 100));
    assert!(matches!(
        build(&f),
        Err(PoliticsError::ColonialEffect { .. })
    ));
}

#[test]
fn broken_references_are_refused() {
    // A movement for a law that does not exist.
    let mut f = files();
    f.movements.movements[0].goal = Goal::Law("utopia".into());
    assert!(matches!(
        build(&f),
        Err(PoliticsError::UnknownLaw { law, .. }) if law == "utopia"
    ));

    // An interest group drawn from a profession that does not exist.
    let mut f = files();
    f.groups.groups[0].professions.push(("knight".into(), 10));
    assert!(matches!(
        build(&f),
        Err(PoliticsError::UnknownProfession { profession, .. }) if profession == "knight"
    ));

    // Shares of a profession that do not add up to everyone.
    let mut f = files();
    f.groups.groups[0].professions[0].1 -= 1;
    assert!(matches!(
        build(&f),
        Err(PoliticsError::ProfessionShares { .. })
    ));

    // A community id that cannot be one.
    let mut f = files();
    f.groups.groups[0].community = Some("Not An Id".into());
    assert!(matches!(build(&f), Err(PoliticsError::BadCommunity { .. })));

    // The scenario naming a law, a template or a country that does not exist.
    let mut f = files();
    f.scenario.countries[0].laws.push("utopia".into());
    assert!(matches!(
        resolve(&f),
        Err(PoliticsError::UnknownLaw { law, .. }) if law == "utopia"
    ));
    let mut f = files();
    f.scenario.countries[0].template = Some("nowhere".into());
    assert!(matches!(
        resolve(&f),
        Err(PoliticsError::UnknownTemplate { .. })
    ));
    let mut f = files();
    f.scenario.countries[0].tag = "XXX".into();
    assert!(matches!(resolve(&f), Err(PoliticsError::UnknownTag(t)) if t == "XXX"));
    let mut f = files();
    f.scenario.history[0].law = "utopia".into();
    assert!(matches!(resolve(&f), Err(PoliticsError::UnknownLaw { .. })));
}

#[test]
fn a_country_missing_a_group_is_refused() {
    // USA lists every law itself, without a template.
    let mut f = files();
    let usa = f
        .scenario
        .countries
        .iter_mut()
        .find(|c| c.tag == "USA")
        .unwrap();
    usa.laws.retain(|l| l != "free_press");
    assert!(matches!(
        resolve(&f),
        Err(PoliticsError::MissingGroup { tag, group }) if tag == "USA" && group == "press"
    ));

    let mut f = files();
    f.scenario.countries.retain(|c| c.tag != "BRA");
    assert_eq!(
        resolve(&f).unwrap_err(),
        PoliticsError::MissingCountry("BRA".into())
    );

    // Two laws of one group.
    let mut f = files();
    let usa = f
        .scenario
        .countries
        .iter_mut()
        .find(|c| c.tag == "USA")
        .unwrap();
    usa.laws.push("prior_censorship".into());
    assert!(matches!(resolve(&f), Err(PoliticsError::LawTwice { .. })));

    // A colonial law for a country without colonies.
    let mut f = files();
    let che = f
        .scenario
        .countries
        .iter_mut()
        .find(|c| c.tag == "CHE")
        .unwrap();
    che.laws.push("direct_rule".into());
    assert!(matches!(
        resolve(&f),
        Err(PoliticsError::NoColonies { tag, .. }) if tag == "CHE"
    ));

    // Targets for a law that targets no one.
    let mut f = files();
    let che = f
        .scenario
        .countries
        .iter_mut()
        .find(|c| c.tag == "CHE")
        .unwrap();
    che.targets.push(("press".into(), vec!["anyone".into()]));
    assert!(matches!(
        resolve(&f),
        Err(PoliticsError::UntargetedLaw { .. })
    ));
}

#[test]
fn every_dated_change_changes_something() {
    let f = files();
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let laws = data.laws_1925.as_ref().unwrap();
    let mut history: Vec<_> = f.scenario.history.iter().collect();
    history.sort_by(|a, b| (&a.tag, &a.date).cmp(&(&b.tag, &b.date)));
    let mut current: BTreeMap<&str, Vec<Option<u16>>> = BTreeMap::new();
    for h in history {
        let c = data
            .countries
            .countries
            .iter()
            .position(|c| c.tag == h.tag)
            .unwrap();
        let set = current
            .entry(&h.tag)
            .or_insert_with(|| laws.laws[c].clone());
        let l = p.law(&h.law).unwrap();
        let g = usize::from(p.laws[l].group);
        assert!(
            set[g].is_some(),
            "{} {}: {} is a colonial law for a country without colonies",
            h.tag,
            h.date,
            h.law
        );
        assert_ne!(
            set[g],
            Some(l as u16),
            "{} {}: {} is already in force",
            h.tag,
            h.date,
            h.law
        );
        set[g] = Some(l as u16);
    }
}

#[test]
fn stances_follow_the_effects() {
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let prefers = |group: &str, a: &str, b: &str| {
        let g = p.interest_group(group).unwrap();
        let (sa, sb) = (p.score(g, p.law(a).unwrap()), p.score(g, p.law(b).unwrap()));
        assert!(sa > sb, "{group} should prefer {a} ({sa}) to {b} ({sb})");
    };
    prefers("landed_estates", "great_estates", "land_reform");
    prefers("peasantry", "land_reform", "great_estates");
    prefers("peasantry", "freehold_farming", "bonded_tenure");
    prefers("workers", "collective_bargaining", "unions_banned");
    prefers("business", "unions_banned", "collective_bargaining");
    prefers("business", "laissez_faire", "planned_economy");
    prefers("workers", "eight_hour_day", "unregulated_labour");
    prefers("professional_classes", "free_press", "prior_censorship");
    prefers("churches", "state_church", "state_atheism");
    prefers("officialdom", "merit_service", "patronage");
    prefers("military", "universal_service", "volunteer_army");
    prefers(
        "landed_estates",
        "indirect_taxation",
        "steep_progressive_tax",
    );

    // The stance is the sum of its terms, each naming an effect.
    let g = p.interest_group("workers").unwrap();
    let l = p.law("labour_code").unwrap();
    let terms = p.terms(g, l);
    assert!(!terms.is_empty());
    assert_eq!(terms.iter().map(|(_, v)| v).sum::<i64>(), p.score(g, l));
}

#[test]
fn internal_passports_match_the_migration_rule() {
    // econ::migration::MigrationLaw::Restricted lets 1/4 of the flow through within a state and
    // 1/10 between states; the law that will drive it must say the same.
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let l = p.law("internal_passports").unwrap();
    let e = |k: &str| p.law_effect(l, EFFECTS.iter().position(|x| x.key == k).unwrap());
    assert_eq!(e("migration_within_state"), 250);
    assert_eq!(e("migration_between_states"), 100);
}

#[test]
fn colonies_live_under_colonial_and_economic_laws_only() {
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let laws = data.laws_1925.as_ref().unwrap();
    let fra = data
        .countries
        .countries
        .iter()
        .position(|c| c.tag == "FRA")
        .unwrap();
    let idx = |k: &str| EFFECTS.iter().position(|x| x.key == k).unwrap();
    let metropole = p.effects(&laws.laws[fra], Scope::Metropole);
    let colonies = p.effects(&laws.laws[fra], Scope::Colonies);
    // Men vote in France; no one votes in its colonies.
    assert!(metropole[idx("franchise")] > 0);
    assert_eq!(colonies[idx("franchise")], 0);
    // The tariff is the same; forced labour only in the colonies.
    assert_eq!(metropole[idx("tariff")], colonies[idx("tariff")]);
    assert_eq!(metropole[idx("unfree_labour")], 0);
    assert!(colonies[idx("unfree_labour")] > 0);
}

#[test]
fn no_movement_wants_a_law_that_keeps_its_cause() {
    // Once its law is enacted, a movement's cause must stop holding; otherwise it never ends.
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    for m in &p.movements {
        if let bf_sim::politics::GoalRef::Law(l) = m.goal {
            for &(measure, bound) in &m.cause {
                if let bf_sim::politics::Measure::Effect(e) = measure {
                    let v = p.law_effect(usize::from(l), usize::from(e));
                    let holds = match bound {
                        Bound::AtLeast(t) => v >= t,
                        Bound::AtMost(t) => v <= t,
                    };
                    assert!(
                        !holds,
                        "{}: its goal {} keeps its own cause ({})",
                        m.key,
                        p.laws[usize::from(l)].key,
                        EFFECTS[usize::from(e)].key
                    );
                }
            }
        }
    }
}

#[test]
fn every_name_exists_in_every_language() {
    // With every switch that adds to the catalogue: the movement against the war is in it only
    // with `anti_war`.
    let data = GameData::load_with(&root(), "anti_war").unwrap();
    let p = data.politics.as_ref().unwrap();
    let mut ids = BTreeSet::new();
    for c in CATEGORIES {
        ids.insert(format!("law-category-{c}"));
    }
    for g in &p.groups {
        ids.insert(format!("law-group-{}", g.key));
    }
    for l in &p.laws {
        ids.insert(format!("law-{}", l.key));
        ids.insert(format!("law-{}-desc", l.key));
    }
    for e in &EFFECTS {
        ids.insert(format!("law-effect-{}", e.key));
    }
    for i in &INDICATORS {
        ids.insert(format!("indicator-{}", i.key));
    }
    for g in &p.interest_groups {
        ids.insert(format!("ig-{}", g.key));
        ids.insert(format!("ig-{}-desc", g.key));
    }
    for m in &p.movements {
        ids.insert(format!("movement-{}", m.key));
        ids.insert(format!("movement-{}-desc", m.key));
    }
    for lang in ["pt-BR", "en"] {
        let text = std::fs::read_to_string(root().join(format!("assets/i18n/{lang}/politics.ftl")))
            .unwrap();
        let messages: BTreeMap<&str, &str> = text
            .lines()
            .filter_map(|l| l.split_once(" = "))
            .map(|(id, value)| (id.trim(), value.trim()))
            .collect();
        for id in &ids {
            assert!(
                messages.get(id.as_str()).is_some_and(|v| !v.is_empty()),
                "{lang} lacks {id}"
            );
        }
        for id in messages.keys() {
            assert!(ids.contains(*id), "{lang}: {id} names nothing");
        }
    }
}
