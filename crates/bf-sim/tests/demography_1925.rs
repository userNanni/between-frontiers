//! The diaspora of 1925 from the communities: who lives abroad, by origin and destination.
//! `cargo test -p bf-sim --test demography_1925 -- --nocapture` prints the table that
//! `docs/population.pt-BR.md` quotes.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::demography::migration::initial_diaspora;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn diaspora_of_1925() {
    let data = GameData::load(&root()).unwrap();
    let world = data.new_world();
    let c = data.communities.as_ref().expect("communities");
    let start = initial_diaspora(&world.econ, &data.map, c);
    let (diaspora, ambiguous) = (start.diaspora, start.ambiguous);
    let tags: Vec<String> = world.countries.iter().map(|(_, c)| c.tag.clone()).collect();
    let name = |t: u16| {
        let part = if t % 2 == 1 { "/colonies" } else { "" };
        format!("{}{part}", tags[usize::from(t / 2)])
    };
    let total: i64 = diaspora.iter().map(|d| d.people).sum();
    println!(
        "diaspora of 1925: {} entries, {:.1} million people",
        diaspora.len(),
        total as f64 / 1e6
    );
    let mut big: Vec<_> = diaspora.iter().collect();
    big.sort_by_key(|d| -d.people);
    for d in big.iter().take(25) {
        println!(
            "  {:>8.0} k  {} -> {}  ({})",
            d.people as f64 / 1e3,
            name(d.origin),
            name(d.dest),
            c.cultures[usize::from(d.culture)].key
        );
    }
    let bra = tags.iter().position(|t| t == "BRA").unwrap() as u16 * 2;
    println!("in Brazil:");
    for d in diaspora.iter().filter(|d| d.dest == bra) {
        println!(
            "  {:>8.0} k  from {} ({})",
            d.people as f64 / 1e3,
            name(d.origin),
            c.cultures[usize::from(d.culture)].key
        );
    }
    println!("minorities with a road of their own (people in the origin, 1925):");
    for h in &start.homes {
        println!(
            "  {} {} {:.0} k",
            name(h.origin),
            c.cultures[usize::from(h.culture)].key,
            h.people_1925 as f64 / 1e3
        );
    }
    println!("ambiguous homeland (largest territory's share of the culture):");
    for (k, s) in &ambiguous {
        println!("  {k} {:.0} %", s * 100.0);
    }
    assert!(total > 10_000_000, "{total}");
    assert!(
        diaspora
            .iter()
            .any(|d| d.dest == bra && c.cultures[usize::from(d.culture)].key == "japanese")
    );
}
