//! Technique through the foreign plant (`docs/foreign-capital.pt-BR.md`, C5): where foreign
//! owners hold a fifth of the levels a technology serves, its diffusion runs about 1.2 times as fast
//! (`foreign_capital::FOREIGN_SPILLOVER` at one); switched off, the foreign plants teach nothing.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::foreign_capital::Handover;
use bf_sim::foreign_stakes::Stake;
use bf_sim::game::{GameWorld, Session};
use bf_sim::research::Stage;
use bf_sim::research_system::Research;
use laplace_schelling::schedule::{System, TickCtx};
use laplace_schelling::{Date, Fixed};

fn world() -> (GameData, GameWorld) {
    let data = GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
    let mut s = Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules_with(Default::default())),
    );
    s.sim.run_until(Date::new(1925, 2, 1));
    let w = s.world().clone();
    (data, w)
}

/// Progress the month adds to technology `t` in country `c`.
fn gain(r: &Research, w: &GameWorld, c: usize, t: usize) -> Fixed {
    let at = TickCtx {
        tick: 1,
        date: Date::new(1925, 2, 22),
        seed: 1925,
        stream: 0,
    };
    let before = w.research.progress[c][t];
    let mut w = w.clone();
    let changes = r.compute(&at, &w);
    r.apply(&at, changes, &mut w);
    w.research.progress[c][t] - before
}

#[test]
fn the_foreign_share_of_the_levels_speeds_the_diffusion_by_as_much() {
    let (data, w) = world();
    let rules = data.rules_with(Default::default());
    let tree = rules.tree.clone().unwrap();
    // A country where a technology is industrial (it diffuses) and has one building kind with
    // levels, in a single state of that country.
    let mut found = None;
    'search: for (c, stages) in w.research.stage.iter().enumerate() {
        for (t, s) in stages.iter().enumerate() {
            if *s != Stage::Industrial || w.research.progress[c][t] >= milli_half() {
                continue;
            }
            let mut kinds: Vec<u16> = tree.techs[t].methods.iter().map(|(b, _)| *b).collect();
            kinds.sort_unstable();
            kinds.dedup();
            let plants: Vec<(u32, u16, i64)> = w
                .econ
                .regions
                .iter()
                .enumerate()
                .filter(|(_, r)| usize::from(r.market) == c)
                .flat_map(|(ri, r)| {
                    r.plants
                        .iter()
                        .filter(|p| kinds.contains(&p.kind) && p.levels > 0)
                        .map(move |p| (ri as u32, p.kind, p.levels))
                })
                .collect();
            if !plants.is_empty() {
                found = Some((c, t, plants));
                break 'search;
            }
        }
    }
    let (c, t, plants) = found.expect("an industrial technology with plants");
    let owner = if c == 0 { 1 } else { 0 };
    // A fifth of every plant it serves, rounded down: the share is what the levels give.
    let mut with = w.clone();
    let (mut held, mut all) = (0i64, 0i64);
    for &(region, kind, levels) in &plants {
        with.foreign.add(Stake {
            region,
            kind,
            owner: owner as u16,
            levels: levels / 5,
            built: 1925,
            partner: Fixed::ZERO,
            term: 0,
            handover: Handover::Never,
        });
        held += levels / 5;
        all += levels;
    }
    assert!(held > 0);
    let research = |on: bool| Research {
        cat: rules.catalogue.clone(),
        tree: tree.clone(),
        wage_floor: false,
        foreign_technique: on,
    };
    let plain = gain(&research(true), &w, c, t);
    let faster = gain(&research(true), &with, c, t);
    let off = gain(&research(false), &with, c, t);
    assert!(plain > Fixed::ZERO);
    assert_eq!(
        off, plain,
        "switched off, the foreign plant teaches nothing"
    );
    // The month's progress is a few thousandths: the faster gain is the plain one times
    // 1 + the foreign share, to the last unit.
    let want = plain * (Fixed::ONE + Fixed::from_ratio(held, all));
    assert!(faster > plain);
    assert!(
        (faster - want).abs() <= Fixed::from_ratio(1, 1000),
        "{faster:?} against {want:?}"
    );
}

fn milli_half() -> Fixed {
    Fixed::from_ratio(1, 2)
}
