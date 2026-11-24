//! The AI's orders of new ships and aircraft (`war::ai_orders`, item 11), measured and not
//! tuned: with the rule and without it, for the great powers in 1925, 1935 and 1939, the
//! aircraft and the naval tonnage (and the capital ships and carriers the treaties cap), the
//! depot's value, and the military spending over GDP. Ignored: two runs of 15 years; run in a
//! bench slot and read the table it prints.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::Session;
use bf_sim::war::catalogue::Domain;
use laplace_schelling::{Date, Fixed};

fn load(rule: bool) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    if !rule {
        d.war.as_mut().unwrap().peace.as_mut().unwrap().ai_orders = None;
    }
    d
}

#[test]
#[ignore = "two runs of 15 years; run in the slot"]
fn ai_orders_1925_to_1939() {
    for rule in [true, false] {
        let d = load(rule);
        let mil = &d.war.as_ref().unwrap().military;
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        for year in [1925, 1935, 1939] {
            s.sim.run_until(Date::new(year, 6, 1));
            let w = s.world();
            for tag in ["GER", "GBR", "FRA", "USA", "JAP", "SOV", "ITA"] {
                let Some(ci) = w.countries.iter().position(|(_, c)| c.tag == tag) else {
                    continue;
                };
                let c = &w.war.countries[ci];
                let m = &w.econ.markets[ci];
                let (mut aircraft, mut tons, mut capital, mut carriers) = (0.0, 0.0, 0.0, 0.0);
                for f in c.formations.iter().filter(|f| !f.colonial) {
                    let ty = &mil.types[f.unit as usize];
                    let n = f.elements.to_f64_lossy();
                    match ty.domain {
                        Domain::Air => aircraft += n,
                        Domain::Sea => {
                            let t =
                                n * f64::from(mil.spec(f.unit, usize::from(f.generation)).tonnage);
                            tons += t;
                            match ty.key.as_str() {
                                "battleship" => capital += t,
                                "carrier" => carriers += t,
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                let value = |l: &[(u16, Fixed)]| -> f64 {
                    l.iter()
                        .map(|(g, q)| (*q * m.price[usize::from(*g)]).to_f64_lossy())
                        .sum()
                };
                let gdp = m.gdp_last_year.to_f64_lossy().max(1.0);
                let ordered: f64 = c
                    .queue
                    .iter()
                    .map(|o| (o.ordered - o.delivered).to_f64_lossy())
                    .sum();
                println!(
                    "AI {} {year} {tag}: aircraft {aircraft:.0} | naval {:.0} kt (capital {:.0}, carriers {:.0}) | open orders {ordered:.0} elements | depot {:.2} bn | military spending {:.2} % of GDP",
                    if rule { "rule" } else { "no rule" },
                    tons / 1000.0,
                    capital / 1000.0,
                    carriers / 1000.0,
                    value(&c.depot) / 1e9,
                    value(&c.orders) * 52.0 * 100.0 / gdp
                );
            }
        }
    }
}
