//! Invariants of the treaty contract (`treaty.rs`): the clause catalogue, the 58 treaties of 1925
//! as clauses, the war debts and their ledger, the evaluation, and the texts the player reads. A
//! dormant clause must say on screen why it sleeps and what wakes it; that is tested here.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::diplomacy::DiplomacyFile;
use bf_sim::scenario::{CountriesFile, MapFile};
use bf_sim::treaty::{
    self, Clause, ClauseCatalogue, ClauseKind, DebtTerms, EvalInputs, Front, LIVE_FRONTS, Proposal,
    Side, ValueTerm, WarDebtsFile,
};
use laplace_schelling::Fixed;

const LOCALES: &[&str] = &["pt-BR", "en"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

struct Data {
    cat: ClauseCatalogue,
    diplomacy: DiplomacyFile,
    countries: CountriesFile,
    debts: WarDebtsFile,
}

fn load() -> Data {
    Data {
        cat: read("data/diplomacy/clauses.ron"),
        diplomacy: read("data/scenario/1925/diplomacy.ron"),
        countries: read("data/scenario/1925/countries.ron"),
        debts: read("data/scenario/1925/war_debts.ron"),
    }
}

fn errors_after(edit: impl FnOnce(&mut Data)) -> Vec<String> {
    let mut d = load();
    edit(&mut d);
    treaty::validate_treaties(&d.diplomacy, &d.cat, &d.countries, &d.debts)
        .err()
        .unwrap_or_default()
}

fn assert_refused(errors: &[String], needle: &str) {
    assert!(
        errors.iter().any(|e| e.contains(needle)),
        "expected an error containing {needle:?}, got {errors:?}"
    );
}

#[test]
fn the_catalogue_is_valid() {
    let d = load();
    if let Err(e) = d.cat.validate() {
        panic!("{} problems:\n{}", e.len(), e.join("\n"));
    }
}

#[test]
fn the_1925_treaties_are_clauses() {
    let d = load();
    if let Err(e) = treaty::validate_treaties(&d.diplomacy, &d.cat, &d.countries, &d.debts) {
        panic!("{} problems:\n{}", e.len(), e.join("\n"));
    }
    assert_eq!(d.diplomacy.treaties.len(), 59);
    let clauses: usize = d.diplomacy.treaties.iter().map(|t| t.clauses.len()).sum();
    let bundles = d
        .diplomacy
        .treaties
        .iter()
        .filter(|t| t.bundle.is_some())
        .count();
    eprintln!("59 treaties: {clauses} clauses, {bundles} subject bundles");
}

#[test]
fn every_place_a_clause_names_exists() {
    let d = load();
    let map: MapFile = read("assets/map/provinces.ron");
    let states: BTreeSet<&str> = map.states.iter().map(|s| s.key.as_str()).collect();
    let seas = std::fs::read_to_string(repo_root().join("data/map/sea_zones.ron")).unwrap();
    for t in &d.diplomacy.treaties {
        for c in &t.clauses {
            for p in treaty::places(&c.kind) {
                assert!(
                    states.contains(p.as_str()) || seas.contains(&format!("key: \"{p}\"")),
                    "treaty {}: {} names {p}, neither a state nor a passage",
                    t.key,
                    c.kind.key()
                );
            }
        }
    }
}

#[test]
fn refusals() {
    // A subjection must carry the bundle of its grade.
    let e = errors_after(|d| {
        let t = d
            .diplomacy
            .treaties
            .iter_mut()
            .find(|t| t.key == "anglo_irish_treaty")
            .unwrap();
        t.bundle = Some("protectorate".into());
    });
    assert_refused(&e, "the grade in countries.ron wants");
    // Senior and junior only on one-sided treaties.
    let e = errors_after(|d| {
        let t = d
            .diplomacy
            .treaties
            .iter_mut()
            .find(|t| t.senior.is_empty())
            .unwrap();
        t.clauses[0].giver = Side::Senior;
    });
    assert_refused(&e, "Senior/Junior on an even treaty");
    // A side names parties only.
    let e = errors_after(|d| {
        d.diplomacy.treaties[0].clauses[0].giver = Side::Parties(vec!["ARG".into()]);
    });
    assert_refused(&e, "ARG is not a party");
    // A treaty says what it does.
    let e = errors_after(|d| d.diplomacy.treaties[0].clauses.clear());
    assert_refused(&e, "says nothing it does");
    // The kind has its clause.
    let e = errors_after(|d| {
        let t = d
            .diplomacy
            .treaties
            .iter_mut()
            .find(|t| t.key == "dawes_plan")
            .unwrap();
        t.clauses.retain(|c| c.kind.key() != "debt");
    });
    assert_refused(&e, "without the clause debt");
    // A debt in force has its lines; a line has its treaty.
    let e = errors_after(|d| {
        d.debts
            .lines
            .retain(|l| l.treaty != "baldwin_mellon_settlement")
    });
    assert_refused(&e, "without lines in war_debts.ron");
    let e = errors_after(|d| d.debts.lines[0].treaty = "no_such_treaty".into());
    assert_refused(&e, "unknown treaty");
    let e = errors_after(|d| d.debts.lines[0].share = 0.9);
    assert_refused(&e, "shares add up to");
    // The catalogue has one row per clause kind.
    let mut d = load();
    d.cat.clauses.retain(|c| c.key != "tribute");
    assert_refused(&d.cat.validate().unwrap_err(), "tribute: no catalogue row");
}

#[test]
fn every_text_exists_in_both_languages() {
    let d = load();
    let mut wanted: BTreeSet<String> = BTreeSet::new();
    for k in ClauseKind::KEYS {
        wanted.insert(format!("clause-{k}"));
        wanted.insert(format!("clause-{k}-effect"));
    }
    for f in Front::ALL {
        wanted.insert(format!("front-{}", f.key()));
    }
    for t in ValueTerm::ALL {
        wanted.insert(format!("term-{}", t.key()));
    }
    for b in &d.cat.bundles {
        wanted.insert(format!("bundle-{}", b.key));
    }
    let mut by_locale = Vec::new();
    for locale in LOCALES {
        let ids = fluent_ids(locale, "treaties.ftl");
        for id in &wanted {
            assert!(ids.contains(id), "{locale}: missing {id}");
        }
        by_locale.push(ids);
    }
    assert_eq!(by_locale[0], by_locale[1], "ids differ between locales");
}

#[test]
fn a_dormant_clause_says_what_wakes_it() {
    // Every clause of 1925 either acts in this build or names a front, and that front has its
    // text in both languages (checked above). Count what sleeps, for the record.
    let d = load();
    let mut asleep: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &d.diplomacy.treaties {
        for c in &t.clauses {
            if let Some(f) = d.cat.dormant(c.kind.key(), LIVE_FRONTS) {
                *asleep.entry(f.key()).or_default() += 1;
            }
        }
    }
    eprintln!("dormant clauses of 1925 by front: {asleep:?}");
    assert!(d.cat.dormant("foreign_policy", LIVE_FRONTS).is_none());
    assert_eq!(d.cat.dormant("defence", LIVE_FRONTS), Some(Front::War));
}

#[test]
fn the_ledger_of_1925() {
    let d = load();
    let ledger = treaty::ledger_1925(&d.debts, &d.cat.policy, &d.countries);
    assert_eq!(ledger.debts.len(), 5);
    let idx = |tag: &str| {
        d.countries
            .countries
            .iter()
            .position(|c| c.tag == tag)
            .unwrap() as u16
    };
    // Germany owes 92 % of 50 billion gold marks (the Spa shares in the ledger), in 1925
    // dollars by gold content, at 2011 prices.
    let german = ledger.owed_by(idx("GER")).to_f64_lossy();
    let expected = 0.92 * 50e9 * 0.358423 / 1.504632 * 12.85;
    assert!(
        (german - expected).abs() / expected < 1e-6,
        "{german} vs {expected}"
    );
    // The British debt: US$ 4.6 billion of 1925, a level annuity at 3 % over 60 years.
    let gbr = &ledger.debts[4];
    assert_eq!(gbr.principal.to_f64_lossy(), 4.6e9 * 12.85);
    let a = gbr.annuity.to_f64_lossy() / 12.85;
    assert!(a > 160e6 && a < 170e6, "{a}");
    // Claims equal debts, world-wide.
    let n = d.countries.countries.len() as u16;
    let net: Fixed = (0..n)
        .map(|c| ledger.claims_of(c) - ledger.owed_by(c))
        .sum();
    assert_eq!(net, Fixed::ZERO);
    // The switch off: no war debts.
    let mut off = d.cat.policy.clone();
    off.war_debts_1925 = false;
    assert!(treaty::ledger_1925(&d.debts, &off, &d.countries).is_empty());
}

#[test]
fn ten_years_of_service_conserve_money_and_debt() {
    let d = load();
    let mut ledger = treaty::ledger_1925(&d.debts, &d.cat.policy, &d.countries);
    let n = d.countries.countries.len();
    let mut treasuries = vec![Fixed::from_int(1_000_000_000); n];
    let revenue = vec![Fixed::from_int(40_000_000); n];
    let inflation = vec![Fixed::from_ratio(3, 100); n];
    let money: Fixed = treasuries.iter().copied().sum();
    for week in 0..520 {
        ledger.service_week(
            &mut treasuries,
            &revenue,
            &vec![Fixed::ONE; n],
            &inflation,
            Fixed::from_ratio(2, 100),
            1925 + week / 52,
        );
        assert_eq!(
            treasuries.iter().copied().sum::<Fixed>(),
            money,
            "week {week}"
        );
        let net: Fixed = (0..n as u16)
            .map(|c| ledger.claims_of(c) - ledger.owed_by(c))
            .sum();
        assert_eq!(net, Fixed::ZERO, "week {week}");
    }
}

fn reparation_proposal(principal: i64, annuity: i64) -> Proposal {
    Proposal {
        side_a: vec!["FRA".into(), "GBR".into()],
        side_b: vec!["GER".into()],
        clauses: vec![
            Clause {
                kind: ClauseKind::Debt {
                    terms: Some(DebtTerms {
                        principal: Fixed::from_int(principal),
                        rate: Fixed::ZERO,
                        annuity: Fixed::from_int(annuity),
                        years: 10,
                        currency: Default::default(),
                        deferrable: Fixed::ZERO,
                        transfer_cap: None,
                        pledge: None,
                        contingent: Fixed::ZERO,
                        grace_years: 0,
                    }),
                },
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            },
            Clause {
                kind: ClauseKind::Demilitarized {
                    states: vec!["DEU-RP".into()],
                },
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            },
        ],
        term_years: None,
        notice_years: 0,
    }
}

#[test]
fn the_account_is_open_and_even() {
    let d = load();
    let gdp: BTreeMap<String, Fixed> = [("FRA", 30_000), ("GBR", 40_000), ("GER", 50_000)]
        .into_iter()
        .map(|(t, g)| (t.to_string(), Fixed::from_int(g)))
        .collect();
    let clean = BTreeMap::new();
    let live = [Front::Ledger, Front::Politics];
    let inputs = EvalInputs {
        discount: Fixed::from_ratio(5, 100),
        gdp: &gdp,
        reputation: &clean,
        live: &live,
    };
    let p = reparation_proposal(10_000, 1_000);
    let ger = treaty::evaluate(&d.cat, &inputs, &p, "GER");
    let fra = treaty::evaluate(&d.cat, &inputs, &p, "FRA");
    let gbr = treaty::evaluate(&d.cat, &inputs, &p, "GBR");
    // Money is a transfer: what the creditors gain the debtor loses.
    let money = |v: &treaty::Valuation| v.term(ValueTerm::Money);
    assert!(money(&ger) < Fixed::ZERO);
    assert_eq!(money(&fra), money(&gbr));
    let sum = money(&ger) + money(&fra) + money(&gbr);
    assert!(sum.abs() <= Fixed::from_raw(2), "{sum:?}");
    // The demilitarised zone sleeps until war exists: shown with its front, valued at zero.
    let zone: Vec<_> = ger.lines.iter().filter(|l| l.clause == 1).collect();
    assert!(!zone.is_empty());
    assert!(
        zone.iter()
            .all(|l| l.dormant == Some(Front::War) && l.value == Fixed::ZERO)
    );
    // The same proposal in the same world gives the same account.
    assert_eq!(treaty::evaluate(&d.cat, &inputs, &p, "GER"), ger);
    // A debtor that broke promises is worth less to its creditors.
    // Broken promises of the general kind worth half of Germany's GDP (owner's decision P1).
    let bad: BTreeMap<String, treaty::Reputation> = [(
        "GER".to_string(),
        treaty::Reputation {
            general: Fixed::from_int(25_000),
            security: Fixed::ZERO,
        },
    )]
    .into();
    let doubtful = EvalInputs {
        reputation: &bad,
        ..inputs.clone()
    };
    let fra_doubt = treaty::evaluate(&d.cat, &doubtful, &p, "FRA");
    assert!(fra_doubt.total < fra.total);
    assert!(fra_doubt.term(ValueTerm::Trust) < Fixed::ZERO);
    // A betrayed alliance does not discount a debt: each clause reads the reputation of its kind.
    let traitor: BTreeMap<String, treaty::Reputation> = [(
        "GER".to_string(),
        treaty::Reputation {
            general: Fixed::ZERO,
            security: Fixed::from_int(25_000),
        },
    )]
    .into();
    let only_security = EvalInputs {
        reputation: &traitor,
        ..inputs.clone()
    };
    assert_eq!(treaty::evaluate(&d.cat, &only_security, &p, "FRA"), fra);
    // Without the ledger live, the debt itself sleeps.
    let asleep = EvalInputs {
        live: &[Front::Politics],
        ..inputs
    };
    let ger_asleep = treaty::evaluate(&d.cat, &asleep, &p, "GER");
    assert_eq!(ger_asleep.term(ValueTerm::Money), Fixed::ZERO);
    assert!(
        ger_asleep
            .lines
            .iter()
            .any(|l| l.dormant == Some(Front::Ledger))
    );
}

/// Message ids of a Fluent file, by simple line parsing.
fn fluent_ids(locale: &str, file: &str) -> BTreeSet<String> {
    let path = repo_root().join("assets/i18n").join(locale).join(file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with(' ') && l.contains('='))
        .map(|l| l.split_once('=').unwrap().0.trim().to_owned())
        .collect()
}

#[test]
fn recognition_of_1925() {
    let d = load();
    let file: treaty::RecognitionFile = read("data/scenario/1925/recognition.ron");
    file.validate(&d.countries).unwrap();
    let rec = file.start(&d.countries, &d.cat.policy);
    if let Err(e) = treaty::validate_recognition(&d.diplomacy, &d.cat, &rec, &d.countries) {
        panic!("{} problems:\n{}", e.len(), e.join("\n"));
    }
    let idx = |tag: &str| {
        d.countries
            .countries
            .iter()
            .position(|c| c.tag == tag)
            .unwrap() as u16
    };
    assert!(!rec.recognises(idx("USA"), idx("SOV")));
    assert!(rec.recognises(idx("SOV"), idx("USA")));
    assert!(rec.recognises(idx("SOV"), idx("MGL")));
    assert!(!rec.recognises(idx("GBR"), idx("MGL")));
    assert!(rec.recognises(idx("FRA"), idx("GER")));
    // Mongolia, recognised by the Soviet Union alone, is recognised by a small share of the world.
    let gdp: Vec<Fixed> = d
        .countries
        .countries
        .iter()
        .map(|c| Fixed::from_int(c.population as i64 * c.gdp_per_capita as i64 / 1000))
        .collect();
    let share = rec.recognised_by_share(idx("MGL"), &gdp);
    assert!(
        share > Fixed::ZERO && share < Fixed::from_ratio(1, 10),
        "{share:?}"
    );
    assert_eq!(rec.recognised_by_share(idx("FRA"), &gdp), Fixed::ONE);
    // The other policy: everyone recognises everyone.
    let mut everyone = d.cat.policy.clone();
    everyone.recognition_default = treaty::RecognitionDefault::Everyone;
    assert!(file.start(&d.countries, &everyone).is_empty());
}

#[test]
fn recognition_gates_the_table() {
    let d = load();
    let file: treaty::RecognitionFile = read("data/scenario/1925/recognition.ron");
    let mut rec = file.start(&d.countries, &d.cat.policy);
    let index = |t: &str| {
        d.countries
            .countries
            .iter()
            .position(|c| c.tag == t)
            .map(|i| i as u16)
    };
    let clause = |kind: ClauseKind| Clause {
        kind,
        giver: Side::All,
        receiver: Side::All,
        until: None,
        note: None,
    };
    let p = |clauses| Proposal {
        side_a: vec!["USA".into()],
        side_b: vec!["SOV".into()],
        clauses,
        term_years: None,
        notice_years: 0,
    };
    // No alliance, tariff or loan between the United States and the Soviet Union in 1925...
    let alliance = p(vec![clause(ClauseKind::Defence)]);
    assert!(!treaty::recognition_conflicts(&alliance, &rec, &d.cat, index).is_empty());
    // ...but recognition itself and a one-off exchange may be signed.
    let rec_only = p(vec![
        clause(ClauseKind::Recognition { border: false }),
        clause(ClauseKind::Payment { amount: Fixed::ONE }),
    ]);
    assert!(treaty::recognition_conflicts(&rec_only, &rec, &d.cat, index).is_empty());
    // Once recognised, the alliance may be signed.
    rec.grant(index("USA").unwrap(), index("SOV").unwrap());
    assert!(treaty::recognition_conflicts(&alliance, &rec, &d.cat, index).is_empty());
    // A new country is recognised only by those who signed its creation.
    let n = d.countries.countries.len() as u16;
    rec.on_new_country(
        n,
        n + 1,
        &[index("GBR").unwrap()],
        d.cat.policy.recognition_default,
    );
    assert!(rec.recognises(index("GBR").unwrap(), n));
    assert!(!rec.recognises(index("FRA").unwrap(), n));
    rec.on_annex(n);
    assert!(rec.recognises(index("FRA").unwrap(), n));
}
