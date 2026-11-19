//! The colonial war (G6, `docs/war.pt-BR.md`, "Guerra colonial") on the real scenario: Algeria
//! revolts against France. The revolt is put in by hand (the independence's path to it takes
//! decades of a movement): the states the garrison does not hold secede, and the revolt's record
//! goes into the history, as `borders::independence` writes them.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::borders::independence::{How, Record};
use bf_sim::borders::{Operation, Terms};
use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::war::colonial;
use laplace_schelling::Date;

fn load(tweak: impl FnOnce(&mut GameData)) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    tweak(&mut d);
    d
}

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| load(|_| {}))
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

fn setup(d: &GameData) -> std::sync::Arc<colonial::ColonialSetup> {
    d.rules()
        .fight
        .as_ref()
        .and_then(|f| f.colonial.clone())
        .expect("the colonial rules in war.ron")
}

/// Algeria's regions.
fn algeria(d: &GameData, w: &GameWorld) -> Vec<u32> {
    let states: BTreeSet<u16> = ["DZA-ALGER", "DZA-CONSTANTINE", "DZA-ORAN", "DZA-SUD"]
        .iter()
        .map(|s| d.border_rules.state_ids[*s])
        .collect();
    w.econ
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| states.contains(&r.state))
        .map(|(i, _)| i as u32)
        .collect()
}

fn week(s: &mut Session) {
    for _ in 0..7 {
        s.sim.step();
    }
}

fn state_key(d: &GameData, w: &GameWorld, r: u32) -> String {
    let id = w.econ.regions[r as usize].state;
    d.border_rules
        .state_ids
        .iter()
        .find(|(_, v)| **v == id)
        .map(|(k, _)| k.clone())
        .unwrap()
}

/// A month in, with the soldiers placed; then Algeria revolts: the garrison's states stay
/// French, the rest secede as DZA, and the record goes into the history.
fn revolt(d: &GameData) -> (Session, Vec<u32>, Vec<u32>) {
    revolt_of(d, "DZA", "FRA")
}

/// The regions of candidate `unit` held by `owner`.
fn unit_regions(d: &GameData, w: &GameWorld, unit: &str, owner: u16) -> Vec<u32> {
    let k = setup(d);
    let states: BTreeSet<u16> = k.candidates[unit].iter().copied().collect();
    w.econ
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| states.contains(&r.state) && r.market == owner)
        .map(|(i, _)| i as u32)
        .collect()
}

fn revolt_of(d: &GameData, unit: &str, owner_tag: &str) -> (Session, Vec<u32>, Vec<u32>) {
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1925, 2, 3));
    let k = setup(d);
    let w = s.world();
    let owner = index(w, owner_tag);
    let regions = unit_regions(d, w, unit, owner);
    let (kept, released) = colonial::partition(w, &regions, &k.coastal, k.rules.garrison);
    let states: Vec<String> = released.iter().map(|&r| state_key(d, w, r)).collect();
    s.submit(GameCommand::Borders {
        op: Operation::Secede {
            from: owner_tag.into(),
            states,
            tag: unit.into(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        },
    });
    s.sim.step();
    s.sim.world_mut().independence.history.push(Record {
        date: Date::new(1925, 2, 4),
        unit: unit.into(),
        owner,
        how: How::Revolt,
    });
    (s, kept, released)
}

/// The conferences to the end (G6 criteria): a colonial war run until it ends or `years` pass,
/// a line a year: support of both sides, the unit's regions each holds (by control), the
/// irregulars armed, the men landed, the casualties of each side.
fn to_the_end(d: &GameData, unit: &str, owner: &str, years: i32) {
    let (mut s, kept, released) = revolt_of(d, unit, owner);
    let w = s.world();
    let names = |rs: &[u32]| rs.iter().map(|&r| state_key(d, w, r)).collect::<Vec<_>>();
    eprintln!("{unit}: kept {:?}, proclaimed {:?}", names(&kept), names(&released));
    let o = index(w, owner);
    let c = index(w, unit);
    let all: Vec<u32> = kept.iter().chain(&released).copied().collect();
    let k = setup(d);
    let mil = &d.war.as_ref().unwrap().military;
    eprintln!("date | support {owner} {unit} | regions {owner}/{unit} | irregulars (armed) | landed | tied (‰ of the army ×2) | casualties {owner} {unit}");
    let mut end = None;
    for week_n in 0..(52 * years) {
        week(&mut s);
        let w = s.world();
        let going = w.war.colonial.iter().any(|x| x.colony == c && !x.peace);
        if week_n % 52 == 0 || !going {
            let sup = |m: u16| w.war.fighting.support_of(m).map_or(-1, |x| x.value);
            let held = |m: u16| {
                all.iter()
                    .filter(|&&r| w.war.fighting.controller(r, w.econ.regions[r as usize].market) == m)
                    .count()
            };
            let (men, armed) = w
                .war
                .countries
                .get(c as usize)
                .map_or((0, 0), |cw| colonial::irregulars(mil, k.irregular, cw));
            let rec = w.war.colonial.iter().find(|x| x.colony == c);
            let landed = rec.map_or(0, |x| x.landed);
            let (tied, commitment) = rec.map_or((0, 0), |x| (x.tied, x.commitment));
            let lost = |m: u16| {
                w.war
                    .fighting
                    .losses
                    .iter()
                    .find(|(x, _)| *x == m)
                    .map_or(0, |(_, l)| *l)
            };
            let dt = s.sim.clock().date;
            eprintln!(
                "{}-{:02} | {} {} | {}/{} | {} ({}) | {} | {} ({}) | {} {}",
                dt.year,
                dt.month,
                sup(o),
                sup(c),
                held(o),
                held(c),
                men,
                armed,
                landed,
                tied,
                commitment,
                lost(o),
                lost(c)
            );
        }
        if !going {
            end = Some(s.sim.clock().date);
            break;
        }
    }
    let w = s.world();
    let owner_holds = all.iter().filter(|&&r| w.econ.regions[r as usize].market == o).count();
    eprintln!("{unit}: ended {end:?}; {owner} owns {owner_holds} of {} regions", all.len());
}

fn g8() -> &'static GameData {
    static D: OnceLock<GameData> = OnceLock::new();
    D.get_or_init(|| {
        load(|d| {
            let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
            c.support.duration_by_commitment = true;
        })
    })
}

fn dosed(g8: bool) -> GameData {
    load(|d| {
        let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
        c.support.duration_by_commitment = g8;
        c.colonial.as_mut().unwrap().cover_target = 1000;
    })
}

#[test]
#[ignore = "long: the G6 conference, the expedition dosed to the cover of O3"]
fn dosed_to_the_end() {
    for g8 in [false, true] {
        let d = dosed(g8);
        eprintln!("== dosed, G8 {g8}");
        to_the_end(&d, "DZA", "FRA", 15);
        to_the_end(&d, "MYS", "GBR", 15);
    }
}

#[test]
#[ignore = "long: the G6 conference, run in the slot"]
fn algeria_to_the_end() {
    to_the_end(data(), "DZA", "FRA", 15);
}

#[test]
#[ignore = "long: the G6 conference, run in the slot"]
fn malaya_to_the_end() {
    to_the_end(data(), "MYS", "GBR", 15);
}

#[test]
#[ignore = "long: the G6 conference with G8 (duration by commitment), run in the slot"]
fn algeria_to_the_end_g8() {
    to_the_end(g8(), "DZA", "FRA", 15);
}

#[test]
#[ignore = "long: the G6 conference with G8 (duration by commitment), run in the slot"]
fn malaya_to_the_end_g8() {
    to_the_end(g8(), "MYS", "GBR", 15);
}

/// At the revolt the garrison holds the ports first; the interior (the Sahara) proclaims.
#[test]
fn the_garrison_keeps_the_ports() {
    let d = data();
    let (s, kept, released) = revolt(d);
    let w = s.world();
    let k = setup(d);
    let names = |rs: &[u32]| rs.iter().map(|&r| state_key(d, w, r)).collect::<Vec<_>>();
    eprintln!("kept {:?}, proclaimed {:?}", names(&kept), names(&released));
    assert!(!released.is_empty(), "the movement takes a state at least");
    for &r in &kept {
        assert!(
            k.coastal.contains(&w.econ.regions[r as usize].state),
            "{} kept before a port",
            state_key(d, w, r)
        );
    }
    assert!(names(&released).contains(&"DZA-SUD".to_owned()));
}

/// The revolt opens a war: France demands the union, Algeria fights with irregulars, half
/// armed; France starts at the colonial support, Algeria at the proclaimed's.
#[test]
fn a_revolt_opens_a_colonial_war() {
    let d = data();
    let (mut s, kept, _) = revolt(d);
    s.sim.run_until(Date::new(1925, 2, 17));
    let w = s.world();
    let (fra, dza) = (index(w, "FRA"), index(w, "DZA"));
    let war = w
        .war
        .conflicts
        .wars
        .iter()
        .find(|x| x.side_a == [fra] && x.side_b == [dza])
        .expect("a colonial war");
    let c = w.war.colonial.iter().find(|c| c.war == war.id).unwrap();
    assert_eq!(c.kept, kept);
    let k = setup(d);
    let mil = &d.war.as_ref().unwrap().military;
    let (men, armed) = colonial::irregulars(mil, k.irregular, &w.war.countries[dza as usize]);
    let want = colonial::combatants(&k.rules, mil, w, dza, 800);
    eprintln!(
        "irregulars {men} ({armed} armed) of {want}; landed {}",
        c.landed
    );
    assert!(men > 0 && armed < men, "irregulars, not all armed");
    let fs = w.war.fighting.support_of(fra).unwrap().value;
    let ds = w.war.fighting.support_of(dza).unwrap().value;
    eprintln!("support FRA {fs}, DZA {ds}");
    assert!(fs < ds, "the metropole starts below the proclaimed");
}

/// France lands men by the ports it holds, a thousand a port a week, up to the cap; on the
/// colonial segments it fights with no more than the garrison and the men landed.
#[test]
fn the_expedition_lands_by_the_ports() {
    let d = data();
    let (mut s, kept, _) = revolt(d);
    s.sim.run_until(Date::new(1925, 2, 10));
    let landed0 = s.world().war.colonial[0].landed;
    s.sim.run_until(Date::new(1925, 4, 14));
    let w = s.world();
    let k = setup(d);
    let c = &w.war.colonial[0];
    let ports = kept
        .iter()
        .filter(|&&r| {
            let reg = &w.econ.regions[r as usize];
            reg.market == c.owner && k.coastal.contains(&reg.state)
        })
        .count() as u64;
    eprintln!(
        "landed {landed0} → {} with {ports} ports; theatre {:?}",
        c.landed,
        colonial::theatre_men(w, c)
    );
    let mil = &d.war.as_ref().unwrap().military;
    let land: i64 = w.war.countries[c.owner as usize]
        .formations
        .iter()
        .filter(|f| mil.types[f.unit as usize].domain == bf_sim::war::Domain::Land)
        .map(|f| bf_sim::war::men(mil, f))
        .sum();
    eprintln!(
        "France's land men {land}; the cap {}",
        land * i64::from(k.rules.expedition_cap) / 1000
    );
    assert!(!c.by_land, "Algeria is overseas");
    assert!(c.landed > landed0, "men land");
    assert!(c.landed - landed0 <= 9 * ports * u64::from(k.rules.men_per_port_week));
    // France's losses on the colonial segments come out of the theatre's men only.
    let theatre = colonial::theatre_men(w, c).unwrap();
    let lost: u64 = w
        .war
        .fighting
        .last_week
        .iter()
        .map(|g| {
            if (g.a, g.b) == (c.owner, c.colony) {
                g.outcome.losses[0]
            } else if (g.b, g.a) == (c.owner, c.colony) {
                g.outcome.losses[1]
            } else {
                0
            }
        })
        .sum();
    assert!(lost <= theatre, "{lost} lost of a theatre of {theatre}");
}

/// People and money of the world, and every region with a living owner.
fn sums(w: &GameWorld) -> (i64, i128) {
    for r in &w.econ.regions {
        assert!(
            (r.market as usize) < w.countries.len(),
            "an orphan region (market {})",
            r.market
        );
    }
    let people = w.econ.regions.iter().map(|r| r.population()).sum();
    let money = w
        .econ
        .markets
        .iter()
        .map(|m| i128::from(m.treasury.raw()))
        .sum();
    (people, money)
}

/// The metropole gives up (its support below the peace mark): the states it still holds in
/// Algeria go to the new country, which it recognises. No region is left without its owner, and
/// the transfer moves no one and no money.
#[test]
fn the_metropole_gives_up_and_the_colony_takes_the_rest() {
    static D: OnceLock<GameData> = OnceLock::new();
    let d = D.get_or_init(|| {
        load(|d| {
            let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
            c.support.colonial = 250;
        })
    });
    let (mut s, kept, _) = revolt(d);
    let fra = index(s.world(), "FRA");
    let dza = index(s.world(), "DZA");
    s.sim.run_until(Date::new(1925, 2, 10));
    let mut ended = None;
    for _ in 0..30 {
        week(&mut s);
        if s.world().war.colonial.is_empty() {
            ended = Some(s.sim.clock().date);
            break;
        }
    }
    let at = ended.expect("France gave up");
    let before = sums(s.world());
    s.sim.step();
    let w = s.world();
    let after = sums(w);
    for &r in &kept {
        assert_eq!(w.econ.regions[r as usize].market, dza, "a kept state goes");
    }
    assert!(
        w.treaties.recognition.recognises(fra, dza),
        "France recognises it"
    );
    assert!(!w.war.conflicts.at_war(fra, dza));
    eprintln!("ended {at:?}; people {before:?} → {after:?}");
    assert_eq!(before, after, "the transfer moves no one and no money");
}

/// Algeria loses (its support below the peace mark): it asks for the union, France signs, and
/// the unit is a French colony again, its movement starting at half the mark of the revolt (GC5).
#[test]
fn the_colony_loses_and_is_a_colony_again() {
    static D: OnceLock<GameData> = OnceLock::new();
    let d = D.get_or_init(|| {
        load(|d| {
            let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
            c.colonial.as_mut().unwrap().proclaimed = 200;
            c.support.ask_weeks = 1;
            // The peace goes through the treaties' table.
            d.switches.negotiation = true;
        })
    });
    let (mut s, _, _) = revolt(d);
    let fra = index(s.world(), "FRA");
    let regions = algeria(d, s.world());
    // The week the war ends, then day by day to the union: the day it goes through moves no
    // one and no money.
    let mut moved = None;
    for _ in 0..40 {
        week(&mut s);
        if s.world().war.colonial.iter().any(|c| c.peace) {
            for _ in 0..7 {
                let before = sums(s.world());
                s.sim.step();
                let w = s.world();
                if regions
                    .iter()
                    .all(|&r| w.econ.regions[r as usize].market == fra)
                {
                    moved = Some((before, sums(w)));
                    break;
                }
            }
            break;
        }
    }
    let (before, after) = moved.expect("the union went through");
    eprintln!("union: {before:?} → {after:?}");
    assert_eq!(before, after, "people and money close");
    week(&mut s);
    let done = s.world().war.colonial.is_empty();
    assert!(done, "the union went through");
    let w = s.world();
    for &r in &regions {
        assert!(w.econ.regions[r as usize].colony, "a colony again");
    }
    let u = w
        .independence
        .units
        .iter()
        .find(|u| u.key == "DZA")
        .expect("the unit again");
    let k = setup(d);
    assert_eq!(
        u.movement.support,
        (u32::from(k.revolt) * k.rules.movement_after_reconquest / 1000) as u16
    );
    sums(w);
}

/// The combatants in absolute numbers against the sources: Algeria's ALN (15,000 to 40,000) and
/// Malaya's MNLA (5,000 to 10,000). Measured on the 1925 people; the doc scales them.
#[test]
fn combatants_against_the_sources() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1925, 2, 3));
    let w = s.world();
    let k = setup(d);
    let mil = &d.war.as_ref().unwrap().military;
    for (tag, owner) in [("DZA", "FRA"), ("MYS", "GBR")] {
        let o = index(w, owner);
        let n = colonial::combatants_in(&k.rules, mil, w, &k.candidates[tag], o, 800);
        eprintln!("{tag}: {n} combatants at support 800 (1925 people)");
        assert!(n > 0);
    }
}

/// The front lost, the guerrilla goes on (G6): France takes the Sahara, Algeria holds no land,
/// and the war goes on in the states France holds, costing it men every week and keeping its
/// garrison tied there.
#[test]
fn the_guerrilla_goes_on_after_the_front() {
    let d = data();
    let (mut s, _, _) = revolt(d);
    for _ in 0..30 {
        week(&mut s);
    }
    let lost = |w: &GameWorld, m: u16| {
        w.war
            .fighting
            .losses
            .iter()
            .find(|(x, _)| *x == m)
            .map_or(0, |(_, l)| *l)
    };
    let w = s.world();
    let (fra, dza) = (index(w, "FRA"), index(w, "DZA"));
    let before = lost(w, fra);
    let c = w.war.colonial.iter().find(|c| c.colony == dza).expect("still at war");
    assert!(
        c.regions
            .iter()
            .all(|&r| w.war.fighting.controller(r, w.econ.regions[r as usize].market) == fra),
        "France holds the whole unit"
    );
    assert!(c.armed > 0 && c.tied > 0);
    for _ in 0..4 {
        week(&mut s);
    }
    let w = s.world();
    assert!(w.war.conflicts.at_war(fra, dza), "the war goes on");
    assert!(lost(w, fra) > before, "the guerrilla costs France men");
}

/// A colony with no one left under arms has lost (G6): the union goes through.
#[test]
fn a_colony_without_arms_surrenders() {
    static D: OnceLock<GameData> = OnceLock::new();
    let d = D.get_or_init(|| {
        load(|d| {
            let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
            let k = c.colonial.as_mut().unwrap();
            k.recruit_week = 0;
            k.exchange_base = 400_000;
            c.support.ask_weeks = 1;
        })
    });
    let (mut s, _, _) = revolt(d);
    let fra = index(s.world(), "FRA");
    let regions = algeria(d, s.world());
    for _ in 0..30 {
        week(&mut s);
    }
    let w = s.world();
    assert!(
        regions
            .iter()
            .all(|&r| w.econ.regions[r as usize].market == fra),
        "the union went through"
    );
}
