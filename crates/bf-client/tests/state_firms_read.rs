//! The state firms in the reading screens: a weapon core plant (a state firm, U2) put in one of
//! Brazil's states shows as the state's in the economy window's buildings and in the state's
//! figures; the methods tab shows the government's choice of method switched off.

use bf_sim::game::{GameCommand, Session};
use laplace_schelling::calendar::Date;

mod common;
use common::layout::root;

#[test]
fn a_state_firm_reads_as_the_states() {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let mut s = Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        1,
        Some(rules.clone()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "BRA".into() });
    s.sim.step();
    let cat = &rules.catalogue;
    let kind = cat
        .building("weapon_core_plant")
        .expect("weapon core plant") as u16;
    assert!(cat.buildings[usize::from(kind)].state_firm);
    let ci = s
        .world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == "BRA")
        .unwrap();
    let region = s
        .world()
        .econ
        .regions
        .iter()
        .position(|r| usize::from(r.market) == ci)
        .unwrap();
    let state = s.world().econ.regions[region].state;
    {
        let p = s.sim.world_mut().econ.regions[region].plant_mut(kind);
        p.levels = 2;
    }
    let date = s.clock().date;
    let b = bf_client::buildings_view::read(s.world(), "BRA", cat, date).unwrap();
    assert_eq!(
        b.state_levels(),
        (2, 0),
        "the state's levels in the buildings"
    );
    let row = b.kinds.iter().find(|k| k.kind == kind).unwrap();
    assert!(row.state_firm);
    let states = bf_client::state_view::read(s.world(), cat, date).unwrap();
    assert_eq!(
        states.states[usize::from(state) - 1].state_levels(),
        2,
        "the state's levels in its state"
    );
}
