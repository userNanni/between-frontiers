//! The imitation channel in the running game: the trade and foreign capital laws reach the
//! economy as `LawEffects::tariff` and `LawEffects::foreign_investment`, an open country keeps
//! today's imitation rate exactly, and closing trade and capital takes it down to the floor
//! (`docs/late-industrialization.pt-BR.md`, "A imitação com canal, primeira versão").

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::milli;
use bf_sim::foreign_capital::{IMITATION_FLOOR, imitation_channel};
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::{Date, Fixed};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn laws_open_and_close_the_imitation_channel() {
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let mut s = Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    s.sim.run_until(Date::new(1925, 1, 2));
    let w = s.world();
    let trade = p.group("trade").unwrap();
    let capital = p.group("foreign_capital").unwrap();
    let ownership = p.group("ownership").unwrap();
    let key = |c: usize, g: usize| {
        p.laws[usize::from(w.politics.law_in_force(c, g).unwrap())]
            .key
            .clone()
    };
    let channel = |c: usize| {
        let e = &w.econ.markets[c].laws.effects[0];
        imitation_channel(e.tariff, e.foreign_investment)
    };

    // The trade law's tariff reaches the economy as it is declared.
    for c in 0..w.politics.laws.len() {
        let want = match key(c, trade).as_str() {
            "free_trade" => 20,
            "revenue_tariffs" => 100,
            "protective_tariffs" => 250,
            "autarky" => 600,
            other => panic!("unknown trade law {other}"),
        };
        assert_eq!(w.econ.markets[c].laws.effects[0].tariff, milli(want));
    }

    // Open trade and capital with a market economy: today's rate, exactly.
    let open = (0..w.politics.laws.len())
        .find(|&c| {
            key(c, trade) != "autarky"
                && key(c, capital) == "open_capital"
                && key(c, ownership) == "laissez_faire"
        })
        .expect("an open country in 1925");
    assert_eq!(
        w.econ.markets[open].laws.effects[0].foreign_investment,
        Fixed::ONE
    );
    assert_eq!(channel(open), Fixed::ONE);

    // Capitulations pull more capital but cannot open the channel past one.
    if let Some(c) = (0..w.politics.laws.len())
        .find(|&c| key(c, capital) == "capitulations" && key(c, trade) != "autarky")
    {
        assert!(w.econ.markets[c].laws.effects[0].foreign_investment > Fixed::ONE);
        assert_eq!(channel(c), Fixed::ONE);
    }

    // Closing the open country by command: capital closed, then a planned economy behind
    // autarky, which leaves only the floor.
    let tag = w.countries.iter().nth(open).unwrap().1.tag.clone();
    for law in ["closed_capital"] {
        s.submit(GameCommand::SetLaw {
            tag: tag.clone(),
            law: p.law(law).unwrap() as u16,
        });
    }
    s.sim.run_until(Date::new(1925, 1, 3));
    let e = &s.world().econ.markets[open].laws.effects[0];
    assert_eq!(e.foreign_investment, milli(100));
    assert_eq!(
        imitation_channel(e.tariff, e.foreign_investment),
        milli(640)
    );

    for law in ["planned_economy", "autarky"] {
        s.submit(GameCommand::SetLaw {
            tag: tag.clone(),
            law: p.law(law).unwrap() as u16,
        });
    }
    s.sim.run_until(Date::new(1925, 1, 4));
    let e = &s.world().econ.markets[open].laws.effects[0];
    assert_eq!(e.tariff, milli(600));
    assert_eq!(
        imitation_channel(e.tariff, e.foreign_investment),
        IMITATION_FLOOR
    );
}
