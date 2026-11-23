//! The use of the weapon (`docs/war.pt-BR.md`, "Arma nuclear na escada de crise", step 4; the
//! owner's N5, N6 and N7): a strike kills and wrecks, its dead weigh on the target's support and
//! not on the user's, the target answers in kind, and the taboo falls at each use and grows back
//! with the years.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use bf_sim::war::Formation;
use bf_sim::war::nuclear_use::Mode;
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.war = true;
        d.switches.treaties_in_force = false;
        d.switches.reputation = true;
        d
    })
}

fn mil() -> &'static bf_sim::war::Military {
    &data().war.as_ref().unwrap().military
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

fn give(w: &mut GameWorld, tag: &str, key: &str, n: i64) {
    let ty = mil().unit(key).unwrap();
    let c = index(w, tag) as usize;
    w.war.countries[c].formations.push(Formation {
        unit: ty,
        // Thermonuclear warheads (generation 1, U3), as the arsenals that deter; the rest
        // has one generation here.
        generation: u8::from(key == "nuclear_warhead"),
        elements: Fixed::from_int(n),
        colonial: false,
        called_up: false,
        equipment: Vec::new(),
        training: Fixed::ONE,
        raised: true,
    });
}

/// Warheads on bombers: they can be launched, but on the ground they give no second strike, so
/// they block no declaration.
fn bombers(w: &mut GameWorld, tag: &str, warheads: i64) {
    give(w, tag, "nuclear_warhead", warheads);
    give(w, tag, "strategic_bomber", warheads);
}

fn warheads(w: &GameWorld, tag: &str) -> i64 {
    let ty = mil().unit("nuclear_warhead").unwrap();
    w.war.countries[index(w, tag) as usize]
        .formations
        .iter()
        .filter(|f| f.unit == ty)
        .map(|f| f.elements.to_int_floor())
        .sum()
}

fn people(w: &GameWorld, tag: &str) -> i64 {
    let m = index(w, tag);
    w.econ
        .regions
        .iter()
        .filter(|r| r.market == m)
        .map(|r| r.population())
        .sum()
}

fn levels(w: &GameWorld, tag: &str) -> i64 {
    let m = index(w, tag);
    w.econ
        .regions
        .iter()
        .filter(|r| r.market == m)
        .flat_map(|r| &r.plants)
        .map(|p| p.levels)
        .sum()
}

/// Germany, played, at war with Poland alone (demanded whole, refused, declared; the treaties
/// off). Hungary against Czechoslovakia ends in two weeks (the shallow capital falls), Germany
/// against Poland lasts.
fn at_war(world: GameWorld) -> Session {
    let d = data();
    let mut s = Session::new(world, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
    s.submit(GameCommand::ChoosePlayer { tag: "GER".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: "GER".into(),
        target: "POL".into(),
        demand: Proposal {
            side_a: vec!["GER".into()],
            side_b: vec!["POL".into()],
            clauses: vec![Clause {
                kind: ClauseKind::Union,
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
        weeks: None,
    });
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "GER".into(),
        crisis: id,
        war: true,
    });
    s.sim.run_until(Date::new(1925, 3, 27));
    assert_eq!(s.world().war.conflicts.wars.len(), 1, "at war");
    s
}

fn later(s: &Session, days: i64) -> Date {
    Date::from_days(s.sim.clock().date.to_days() + days)
}

/// The dead `n` of `by`'s warheads would kill in `on`'s cities now (`war::target`), each counted
/// at `share` per mille.
fn expected(w: &GameWorld, by: &str, on: &str, n: usize, share: i64) -> i64 {
    let rule = &mil().rules.nuclear;
    let (sum, count) =
        bf_sim::war::balance::warhead_emt(mil(), &w.war.countries[index(w, by) as usize]);
    let area = bf_sim::war::target::area_of(rule.lethal_km2_per_emt, (sum / count.max(1)) as u64);
    let cities = bf_sim::war::target::cities(w, index(w, on));
    bf_sim::war::target::plan(&cities, rule.core_density, &vec![area; n])
        .iter()
        .map(|x| x.1 * share / 1000)
        .sum()
}

/// Within a percent: the cities grow a little in the week before the strike.
fn near(got: u64, want: i64) {
    let got = got as i64;
    assert!(
        (got - want).abs() * 100 <= want,
        "dead {got}, the cities' rule {want}"
    );
}

fn launch(s: &mut Session, mode: Mode, n: u32) {
    s.submit(GameCommand::NuclearUse {
        tag: "GER".into(),
        target: "POL".into(),
        mode,
        warheads: n,
    });
    let d = later(s, 8);
    s.sim.run_until(d);
}

/// A countervalue strike kills its warheads' dead in the target's cities (`war::target`), wrecks plants, spends the warheads, stains
/// the user's security reputation, and cuts the taboo.
#[test]
fn countervalue_kills_and_wrecks() {
    let mut w = data().new_world();
    bombers(&mut w, "GER", 4);
    let mut s = at_war(w);
    let (p0, l0, h0) = (
        people(s.world(), "POL"),
        levels(s.world(), "POL"),
        warheads(s.world(), "GER"),
    );
    let stain0 = s
        .world()
        .treaties
        .reputation
        .get("GER")
        .map_or(Fixed::ZERO, |r| r.security);
    let want = expected(s.world(), "GER", "POL", 2, 1000);
    launch(&mut s, Mode::Countervalue, 2);
    let w = s.world();
    let cze = index(w, "POL");
    let dead = w.war.nuclear.dead_of(cze);
    eprintln!(
        "dead {dead}, people {p0} → {}, levels {l0} → {}",
        people(w, "POL"),
        levels(w, "POL")
    );
    near(dead, want);
    assert!(
        people(w, "POL") <= p0 - 300_000,
        "the dead leave the people"
    );
    assert!(levels(w, "POL") < l0, "the cities' plants are wrecked");
    assert_eq!(warheads(w, "GER"), h0 - 2);
    assert!(w.war.nuclear.last_use.is_some());
    assert!(w.war.nuclear.tested(index(w, "GER")), "a use is a test");
    let stain = w
        .treaties
        .reputation
        .get("GER")
        .map_or(Fixed::ZERO, |r| r.security);
    assert!(stain > stain0, "the security reputation is stained");
}

/// Counterforce kills a tenth of the civilians and takes men out of the target's land forces.
#[test]
fn counterforce_spares_civilians_and_hits_the_forces() {
    let mut w = data().new_world();
    bombers(&mut w, "GER", 2);
    let mut s = at_war(w);
    let men0 = bf_sim::war::nuclear_use::land_men(
        mil(),
        &s.world().war.countries[index(s.world(), "POL") as usize],
    );
    let want = expected(s.world(), "GER", "POL", 2, 100);
    launch(&mut s, Mode::Counterforce, 2);
    let w = s.world();
    let cze = index(w, "POL");
    near(w.war.nuclear.dead_of(cze), want);
    let men = bf_sim::war::nuclear_use::land_men(mil(), &w.war.countries[cze as usize]);
    eprintln!("land men {men0} → {men}");
    assert!(men < men0 - 30_000, "the forces are hit");
}

/// No use without a launcher: warheads alone stay in store.
#[test]
fn no_use_without_a_launcher() {
    let mut w = data().new_world();
    give(&mut w, "GER", "nuclear_warhead", 4);
    let mut s = at_war(w);
    launch(&mut s, Mode::Countervalue, 2);
    let w = s.world();
    assert_eq!(w.war.nuclear.dead_of(index(w, "POL")), 0);
    assert_eq!(warheads(w, "GER"), 4);
}

/// N5: the AI answers the next week on the same rung, with as many warheads as it received.
#[test]
fn the_target_answers_in_kind() {
    let mut w = data().new_world();
    bombers(&mut w, "GER", 4);
    bombers(&mut w, "POL", 4);
    let mut s = at_war(w);
    let want = expected(s.world(), "POL", "GER", 2, 100);
    launch(&mut s, Mode::Counterforce, 2);
    let d = later(&s, 8);
    s.sim.run_until(d);
    let w = s.world();
    assert_eq!(warheads(w, "POL"), 2, "two answered two");
    near(w.war.nuclear.dead_of(index(w, "GER")), want);
    // The answer is not answered again.
    let d = later(&s, 15);
    s.sim.run_until(d);
    assert_eq!(warheads(s.world(), "GER"), 2);
}

/// N6: the dead weigh on the target's support for the war, not on the user's.
#[test]
fn the_dead_weigh_on_the_target_only() {
    let support = |s: &Session, tag: &str| {
        let w = s.world();
        let m = index(w, tag);
        w.war
            .fighting
            .support
            .iter()
            .find(|x| x.country == m)
            .map(|x| x.value)
            .unwrap()
    };
    let mut w = data().new_world();
    bombers(&mut w, "GER", 8);
    let mut used = at_war(w.clone());
    let mut plain = at_war(w);
    launch(&mut used, Mode::Countervalue, 8);
    let d = later(&plain, 8);
    plain.sim.run_until(d);
    for _ in 0..4 {
        let d = later(&used, 7);
        used.sim.run_until(d);
        let d = later(&plain, 7);
        plain.sim.run_until(d);
    }
    eprintln!(
        "POL {} with, {} without; GER {} with, {} without",
        support(&used, "POL"),
        support(&plain, "POL"),
        support(&used, "GER"),
        support(&plain, "GER")
    );
    // The target's support falls by its dead; the user's moves only with the course of the war
    // (a few points either way), not by a term of its own.
    assert!(support(&plain, "POL") - support(&used, "POL") > 20);
    assert!((support(&used, "GER") - support(&plain, "GER")).abs() <= 5);
}

/// N7: the taboo grows a twentieth each January while a warhead exists and the year before had no
/// use; a use halves it, and the year of the use does not grow it.
#[test]
fn the_taboo_grows_and_falls() {
    let mut w = data().new_world();
    bombers(&mut w, "GER", 4);
    let mut s = at_war(w);
    // January 1925: warheads in the world, no use in 1924.
    assert_eq!(s.world().war.nuclear.taboo, 50);
    launch(&mut s, Mode::Countervalue, 1);
    assert_eq!(s.world().war.nuclear.taboo, 25, "a use halves it");
    s.sim.run_until(Date::new(1926, 1, 10));
    assert_eq!(s.world().war.nuclear.taboo, 25, "the year of the use");
    s.sim.run_until(Date::new(1928, 1, 10));
    assert_eq!(s.world().war.nuclear.taboo, 125, "two Januaries of 50");
}

/// N7: the AI uses first only with its home occupied beyond the taboo's line: 30% with no taboo,
/// all of it with the taboo whole.
#[test]
fn the_ai_uses_first_when_overrun() {
    use bf_sim::war::nuclear_use::first_uses;
    let mut w = data().new_world();
    bombers(&mut w, "POL", 4);
    let (ger, pol) = (index(&w, "GER"), index(&w, "POL"));
    w.war.conflicts.wars.push(bf_sim::war::crisis::War {
        id: 0,
        side_a: vec![ger],
        side_b: vec![pol],
        crisis: 0,
        demand: Proposal {
            side_a: vec!["GER".into()],
            side_b: vec!["POL".into()],
            clauses: Vec::new(),
            term_years: None,
            notice_years: 0,
        },
        since: (1925, 3, 20),
    });
    assert!(first_uses(mil(), &w, None).is_empty(), "nothing taken");
    // Germany takes Poland's home regions, the most populous first, past 30%.
    let mut home: Vec<(i64, u32)> = w
        .econ
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| r.market == pol && !r.colony)
        .map(|(i, r)| (r.population(), i as u32))
        .collect();
    home.sort_unstable_by(|a, b| b.cmp(a));
    let all: i64 = home.iter().map(|x| x.0).sum();
    let mut taken = 0;
    for (p, r) in home {
        if taken * 10 >= all * 4 {
            break;
        }
        w.war.fighting.control.push((r, ger));
        taken += p;
    }
    w.war.fighting.control.sort_unstable();
    let uses = first_uses(mil(), &w, None);
    assert_eq!(uses.len(), 1);
    assert_eq!(
        (uses[0].by, uses[0].target, uses[0].warheads),
        (pol, ger, 4)
    );
    assert!(
        first_uses(mil(), &w, Some(pol)).is_empty(),
        "the player decides for itself"
    );
    w.war.nuclear.taboo = 1000;
    assert!(
        first_uses(mil(), &w, None).is_empty(),
        "the taboo whole holds"
    );
}

/// N8: a warhead's fallout takes a tenth of the productivity of the region it falls on and
/// three hundredths of the neighbours', kept as a stock that decays with the years (half in
/// 30), the productivity coming back with it.
#[test]
fn the_fallout_takes_productivity_and_decays() {
    let mut w = data().new_world();
    bombers(&mut w, "GER", 4);
    let mut used = at_war(w.clone());
    let mut plain = at_war(w);
    launch(&mut used, Mode::Countervalue, 1);
    let d = later(&plain, 8);
    plain.sim.run_until(d);
    let n = &used.world().war.nuclear;
    let pol = index(used.world(), "POL");
    // The region the warhead fell on: a home region of Poland's with the whole loss.
    let hit = n
        .fallout
        .iter()
        .find(|&&(r, l)| l == 100 && used.world().econ.regions[r as usize].market == pol)
        .map(|&(r, _)| r)
        .unwrap();
    eprintln!("fallout {:?}", n.fallout);
    assert_eq!(n.fallout_of(hit), 100);
    assert!(
        n.fallout.iter().any(|&(r, l)| r != hit && l == 30),
        "the neighbours take three tenths"
    );
    let ratio = |a: &Session, b: &Session, r: u32| {
        a.world().econ.regions[r as usize].tfp.to_f64_lossy()
            / b.world().econ.regions[r as usize].tfp.to_f64_lossy()
    };
    let now = ratio(&used, &plain, hit);
    eprintln!("productivity of the region hit: {now:.3} of the world without the strike");
    assert!((now - 0.9).abs() < 0.005);
    // A year on, the stock decays and the productivity comes back with it.
    used.sim.run_until(Date::new(1926, 1, 20));
    plain.sim.run_until(Date::new(1926, 1, 20));
    let left = used.world().war.nuclear.fallout_of(hit);
    let back = ratio(&used, &plain, hit);
    eprintln!("a year on: fallout {left} per mille, productivity {back:.3}");
    assert_eq!(left, 97);
    assert!(back > now, "it comes back");
}

/// N8, the measure for the PR (no target): France strikes Germany with ten warheads on its
/// cities in March 1925, and Germany's GDP five, ten and thirty years on, with the fallout's loss
/// at 0 (the strike alone), 5 % and 10 % a warhead (the neighbours three tenths of it, as in the
/// data).
#[test]
#[ignore = "long: the N8 measure, thirty years three times; run in the slot"]
fn fallout_gdp_measure() {
    for loss in [0u32, 50, 100] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.war = true;
        let n = &mut d.war.as_mut().unwrap().military.rules.nuclear;
        n.fallout_loss = loss;
        let mut w = d.new_world();
        let ty = d
            .war
            .as_ref()
            .unwrap()
            .military
            .unit("nuclear_warhead")
            .unwrap();
        let bomber = d
            .war
            .as_ref()
            .unwrap()
            .military
            .unit("strategic_bomber")
            .unwrap();
        let fra = w
            .countries
            .iter()
            .position(|(_, c)| c.tag == "FRA")
            .unwrap();
        for (unit, n) in [(ty, 10), (bomber, 10)] {
            w.war.countries[fra].formations.push(Formation {
                unit,
                generation: 0,
                elements: Fixed::from_int(n),
                colonial: false,
                called_up: false,
                equipment: Vec::new(),
                training: Fixed::ONE,
                raised: true,
            });
        }
        let mut s = Session::new(w, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
        s.submit(GameCommand::ChoosePlayer { tag: "FRA".into() });
        s.sim.run_until(Date::new(1925, 3, 2));
        s.submit(GameCommand::Demand {
            tag: "FRA".into(),
            target: "GER".into(),
            demand: Proposal {
                side_a: vec!["FRA".into()],
                side_b: vec!["GER".into()],
                clauses: vec![Clause {
                    kind: ClauseKind::Union,
                    giver: Side::Junior,
                    receiver: Side::Senior,
                    until: None,
                    note: None,
                }],
                term_years: None,
                notice_years: 0,
            },
            weeks: None,
        });
        s.sim.run_until(Date::new(1925, 3, 20));
        let id = s.world().war.conflicts.crises[0].id;
        s.submit(GameCommand::Declare {
            tag: "FRA".into(),
            crisis: id,
            war: true,
        });
        s.sim.run_until(Date::new(1925, 3, 24));
        s.submit(GameCommand::NuclearUse {
            tag: "FRA".into(),
            target: "GER".into(),
            mode: Mode::Countervalue,
            warheads: 10,
        });
        let ger = s
            .world()
            .countries
            .iter()
            .position(|(_, c)| c.tag == "GER")
            .unwrap();
        let gdp = |s: &Session| s.world().econ.markets[ger].gdp_last_year.to_f64_lossy() / 1e9;
        s.sim.run_until(Date::new(1926, 1, 10));
        let g0 = gdp(&s);
        let mut line = format!("N8 loss {loss}: GDP of Germany, billions: 1925 {g0:.1}");
        for year in [1930, 1935, 1955] {
            s.sim.run_until(Date::new(year, 1, 10));
            let stock: u32 = s
                .world()
                .war
                .nuclear
                .fallout
                .iter()
                .filter(|(r, _)| usize::from(s.world().econ.regions[*r as usize].market) == ger)
                .map(|(_, l)| *l)
                .max()
                .unwrap_or(0);
            line += &format!(" | {} {:.1} (worst region {stock}‰)", year - 1, gdp(&s));
        }
        eprintln!("{line}");
    }
}
