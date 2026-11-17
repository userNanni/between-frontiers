//! Independence by movement (`docs/borders.pt-BR.md`, section 11): the colonial units of 1925,
//! the two marks, the metropole's account, the revolt after a refusal and the player's answer.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::borders::independence::{self, Answer, Demand, How, IndependenceSetup, Unit, units_of};
use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use laplace_schelling::{Date, Fixed};

fn load() -> GameData {
    GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
}

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(load)
}

/// The rules with independence by movement switched on (it ships off).
fn rules_on() -> bf_sim::game::Rules {
    data().rules_with(bf_sim::game::Switches {
        independence: true,
        ..Default::default()
    })
}

/// A new rule enters off: the switch is off by default, and the world of a game keeps the hash
/// of a build without independence by movement.
#[test]
fn independence_ships_off() {
    let d = data();
    let rules = d.rules();
    assert!(!rules.switches.independence);
    assert!(!rules.independence.as_ref().unwrap().rules.enabled);
    assert_eq!(
        bf_sim::game::Switches::parse("independence").unwrap(),
        bf_sim::game::Switches {
            independence: true,
            ..Default::default()
        }
    );
    let mut s = Session::new(d.new_world(), 1925, Date::new(1926, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(1926, 1, 4));
    assert!(s.world().independence.is_empty());
}

fn setup() -> std::sync::Arc<IndependenceSetup> {
    rules_on().independence.expect("independence rules")
}

fn ix(w: &GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
}

fn alive(w: &GameWorld, tag: &str) -> bool {
    w.countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .is_some_and(|c| w.econ.regions.iter().any(|r| usize::from(r.market) == c))
}

/// The world of 1925 with the units in place and `key`'s movement forced to full support.
fn forced(key: &str, demand: Option<Answer>) -> (GameWorld, Unit) {
    let s = setup();
    let mut w = data().new_world();
    let mut units = units_of(&s, &w, &[]);
    let u = units.iter_mut().find(|u| u.key == key).expect(key);
    u.movement.support = 1000;
    u.mobilized = 1000;
    u.demand = demand.map(|answer| Demand {
        since: Date::new(1925, 1, 2),
        answer,
        account: Default::default(),
    });
    let unit = u.clone();
    w.independence.units = units;
    (w, unit)
}

fn session_at_new_year(w: GameWorld) -> Session {
    Session::new(w, 1925, Date::new(1926, 1, 1), 2, Some(rules_on()))
}

#[test]
fn the_colonial_units_of_1925() {
    let w = data().new_world();
    let units = units_of(&setup(), &w, &[]);
    let find = |k: &str| {
        units
            .iter()
            .find(|u| u.key == k)
            .unwrap_or_else(|| panic!("{k}"))
    };
    assert_eq!(usize::from(find("NGA").owner), ix(&w, "GBR"));
    assert_eq!(usize::from(find("KOR").owner), ix(&w, "JAP"));
    assert_eq!(usize::from(find("COD").owner), ix(&w, "BEL"));
    // No region in two units, and every region of a unit is a colony of its owner.
    let mut seen = std::collections::BTreeSet::new();
    for u in &units {
        for &r in &u.regions {
            assert!(seen.insert(r), "{} {r}", u.key);
            let reg = &w.econ.regions[r as usize];
            assert!(reg.colony && reg.market == u.owner, "{}", u.key);
        }
    }
    // A sovereign country of 1925 is not a unit.
    assert!(units.iter().all(|u| u.key != "BRA"));
}

#[test]
fn after_a_refusal_the_revolt_breaks_away_without_agreement() {
    let (w, _) = forced("NGA", Some(Answer::Refused));
    let gbr = ix(&w, "GBR");
    let mut s = session_at_new_year(w);
    s.sim.run_until(Date::new(1926, 1, 4));
    let w = s.world();
    let r = w
        .independence
        .history
        .iter()
        .find(|r| r.unit == "NGA")
        .expect("NGA broke away");
    assert_eq!(r.how, How::Revolt);
    assert_eq!(usize::from(r.owner), gbr);
    assert!(alive(w, "NGA"), "the border operation ran");
    assert!(w.independence.units.iter().all(|u| u.key != "NGA"));
    // Without an agreement nobody signs: the metropole does not recognise the new country.
    let nga = ix(w, "NGA") as u16;
    assert!(!w.treaties.recognition.recognises(gbr as u16, nga));
}

#[test]
fn a_player_metropole_answers_and_the_agreement_releases() {
    let (w, _) = forced("NGA", None);
    let mut s = session_at_new_year(w);
    s.submit(GameCommand::ChoosePlayer { tag: "GBR".into() });
    s.sim.run_until(Date::new(1926, 1, 3));
    let pending = s
        .world()
        .independence
        .units
        .iter()
        .find(|u| u.key == "NGA")
        .and_then(|u| u.demand.as_ref())
        .map(|d| d.answer);
    assert_eq!(
        pending,
        Some(Answer::Pending),
        "the demand waits for the player"
    );
    s.submit(GameCommand::AnswerDemand {
        unit: "NGA".into(),
        answer: Answer::Independence,
    });
    s.sim.run_until(Date::new(1926, 1, 5));
    let w = s.world();
    let r = w
        .independence
        .history
        .iter()
        .find(|r| r.unit == "NGA")
        .unwrap();
    assert_eq!(r.how, How::Agreement);
    assert!(
        alive(w, "NGA"),
        "{:?} {:?}",
        w.borders.refused,
        w.borders.pending
    );
    // The metropole signed the agreement: it recognises the new country.
    let (gbr, nga) = (ix(w, "GBR") as u16, ix(w, "NGA") as u16);
    assert!(w.treaties.recognition.recognises(gbr, nga));
}

#[test]
fn the_international_cost_follows_the_anticolonial_countries_not_the_date() {
    let s = setup();
    let (w, unit) = forced("NGA", None);
    let stance = independence::stances(&s.rules, &w);
    let (base, _) = independence::account(&s, &w, &unit, &stance, Fixed::ZERO);
    assert!(base.international > Fixed::ZERO);
    // More anticolonial countries, more cost: give every holder of colonies but the owner the
    // stance of a country born of an independence.
    let more: Vec<u16> = stance
        .iter()
        .enumerate()
        .map(|(c, &x)| {
            if c == usize::from(unit.owner) {
                0
            } else {
                x.max(s.rules.born_independent)
            }
        })
        .collect();
    let (wider, _) = independence::account(&s, &w, &unit, &more, Fixed::ZERO);
    assert!(wider.international > base.international);
    // A colony without a movement costs nothing abroad.
    let mut quiet = unit.clone();
    quiet.movement.support = 0;
    quiet.mobilized = 0;
    let (none, withdrawn) = independence::account(&s, &w, &quiet, &stance, Fixed::ZERO);
    assert_eq!(none.international, Fixed::ZERO);
    assert!(withdrawn.is_empty());
    // The answer: refuse while the colony is worth more than it costs, then the ladder.
    let mut acc = base.clone();
    acc.value = acc.cost() + Fixed::ONE;
    assert_eq!(independence::answer(&acc, false), Answer::Refused);
    acc.value = Fixed::ZERO;
    assert_eq!(independence::answer(&acc, false), Answer::SelfRule);
    assert_eq!(independence::answer(&acc, true), Answer::Independence);
}

#[test]
fn switched_off_nothing_is_measured() {
    let d = data();
    let mut rules = rules_on();
    let mut setup = (*rules.independence.clone().unwrap()).clone();
    setup.rules.enabled = false;
    rules.independence = Some(std::sync::Arc::new(setup));
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(1925, 1, 4));
    assert!(s.world().independence.is_empty());
}

/// The screen's reading: the pending demand with its deadline, the condition's key, and the price
/// of each answer.
#[test]
fn the_screen_reads_the_demand_and_the_price_of_each_answer() {
    let (w, _) = forced("NGA", None);
    let gbr = ix(&w, "GBR") as u16;
    let mut s = session_at_new_year(w);
    s.submit(GameCommand::ChoosePlayer { tag: "GBR".into() });
    s.sim.run_until(Date::new(1926, 1, 3));
    let setup = setup();
    let w = s.world();
    let views = independence::demands_of(&setup, w, gbr);
    let v = views.iter().find(|v| v.unit == "NGA").expect("pending");
    assert_eq!(v.since, Date::new(1926, 1, 2));
    assert_eq!(v.deadline, Date::new(1927, 1, 2));
    assert!(v.states.iter().any(|s| s == "NGA-WESTERN"));
    assert!(v.mobilized >= v.demand_mark);
    let key = v.weakest.as_deref().expect("a condition");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n");
    let politics = std::fs::read_to_string(root.join("pt-BR/politics.ftl")).unwrap();
    assert!(politics.contains(&format!("\n{key} =")), "{key}");
    let out = independence::answer_effects(&setup, w, "NGA", Answer::Independence).unwrap();
    assert_eq!(out.after.value, Fixed::ZERO);
    assert_eq!(out.after.international, Fixed::ZERO);
    let refuse = independence::answer_effects(&setup, w, "NGA", Answer::Refused).unwrap();
    assert_eq!(refuse.revolt, Some((v.mobilized, v.revolt_mark)));
}

/// Every candidate a unit can be has a name in both languages.
#[test]
fn every_unit_has_a_name() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n");
    let w = data().new_world();
    for lang in ["pt-BR", "en"] {
        let names = std::fs::read_to_string(root.join(lang).join("countries.ftl")).unwrap();
        for u in units_of(&setup(), &w, &[]) {
            assert!(
                names.contains(&format!("\ncountry-{} =", u.key)),
                "{lang} country-{}",
                u.key
            );
        }
        for c in &data().candidates.as_ref().unwrap().countries {
            assert!(
                names.contains(&format!("\ncountry-{} =", c.tag)),
                "{lang} candidate country-{}",
                c.tag
            );
        }
    }
}

/// Sinal 6 do doc: a separação, por revolta ou por acordo, não cria nem destrói gente, dinheiro,
/// dívida ou plantas. Entre 2 e 3 de janeiro de 1926 só rodam os sistemas diários (política,
/// independência, fronteiras), então o que o mundo soma antes e depois tem de ser igual. O acordo
/// vem da resposta do jogador metrópole, dada no dia 3 (a IA refaz a própria conta e não lê uma
/// resposta escrita no mundo).
#[test]
fn a_separation_conserves_by_revolt_and_by_agreement() {
    use bf_sim::borders::Totals;
    for agreement in [false, true] {
        let (w, _) = forced("NGA", (!agreement).then_some(Answer::Refused));
        let mut s = session_at_new_year(w);
        if agreement {
            s.submit(GameCommand::ChoosePlayer { tag: "GBR".into() });
            s.sim.run_until(Date::new(1926, 1, 3));
            s.submit(GameCommand::AnswerDemand {
                unit: "NGA".into(),
                answer: Answer::Independence,
            });
        } else {
            s.sim.run_until(Date::new(1926, 1, 2));
        }
        let before = Totals::of(s.world());
        let done = s.world().borders.changes.len();
        s.sim.run_until(Date::new(1926, 1, 4));
        let w = s.world();
        assert!(
            alive(w, "NGA"),
            "agreement {agreement}: {:?}",
            w.borders.refused
        );
        let mut after = Totals::of(w);
        for c in &w.borders.changes[done..] {
            after.diaspora += c.repatriated.0;
            after.arrived += c.repatriated.1;
        }
        assert_eq!(after, before, "agreement {agreement}: totals changed");
    }
}
