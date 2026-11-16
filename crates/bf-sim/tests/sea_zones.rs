//! The sea graph generated in `assets/map/seas.ron`, against the map and the rules in
//! `data/map/sea_zones.ron`: every port reaches every port, closing a strait changes the route of
//! whoever depended on it, ice closes the White Sea in winter but not Murmansk, and control of a
//! strait follows the owner of its shores.

use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::scenario::MapFile;
use bf_sim::sea::{Holders, Ice, Route, RouteOptions, SeaGraph, SeasFile, Season};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn load() -> &'static (SeaGraph, MapFile) {
    static DATA: OnceLock<(SeaGraph, MapFile)> = OnceLock::new();
    DATA.get_or_init(|| {
        let file: SeasFile = read("assets/map/seas.ron");
        (
            SeaGraph::new(file).unwrap(),
            read("assets/map/provinces.ron"),
        )
    })
}

fn state(key: &str) -> u16 {
    let (g, map) = load();
    let id = map
        .states
        .iter()
        .find(|s| s.key == key)
        .unwrap_or_else(|| panic!("no state {key}"))
        .id;
    assert!(g.is_coastal(id), "{key} has no port");
    id
}

fn route(from: &str, to: &str, o: &RouteOptions) -> Option<Route> {
    load().0.route(state(from), state(to), o)
}

fn open() -> RouteOptions {
    RouteOptions::open("1925-01-01")
}

fn closing(keys: &[&str]) -> RouteOptions {
    let g = &load().0;
    let mut o = open();
    o.closed = keys.iter().map(|k| g.strait(k).unwrap()).collect();
    o
}

fn zones(r: &Route) -> Vec<&'static str> {
    let g = &load().0;
    r.zones
        .iter()
        .map(|z| g.file.zones[*z as usize - 1].key.as_str())
        .collect()
}

#[test]
fn every_port_reaches_every_port() {
    let (g, map) = load();
    // Breadth first over every zone through every edge, from the first open-sea zone.
    let n = g.file.zones.len();
    let mut seen = vec![false; n + 1];
    let start = g.zone("north_atlantic_central").unwrap();
    let mut queue = VecDeque::from([start]);
    seen[start as usize] = true;
    while let Some(z) = queue.pop_front() {
        for e in &g.file.edges {
            let other = if e.a == z {
                e.b
            } else if e.b == z {
                e.a
            } else {
                continue;
            };
            if !seen[other as usize] {
                seen[other as usize] = true;
                queue.push_back(other);
            }
        }
    }
    for z in &g.file.zones {
        assert_eq!(seen[z.id as usize], !z.isolated, "zone {}", z.key);
    }
    let isolated: Vec<&str> = g
        .file
        .zones
        .iter()
        .filter(|z| z.isolated)
        .map(|z| z.key.as_str())
        .collect();
    assert_eq!(isolated, ["caspian_sea"]);
    // Every port is on a zone, and the coastal states of land.ron mostly have one.
    assert!(
        g.file
            .ports
            .iter()
            .all(|p| p.zone >= 1 && p.zone as usize <= n)
    );
    let coastal: BTreeSet<u16> = g.file.ports.iter().map(|p| p.state).collect();
    assert!(
        coastal.len() * 2 > map.states.len(),
        "{} coastal of {}",
        coastal.len(),
        map.states.len()
    );
}

#[test]
fn every_strait_has_shores_and_joins_two_zones() {
    let (g, map) = load();
    assert_eq!(g.file.straits.len(), 24);
    for s in &g.file.straits {
        assert!(s.a != s.b && s.a != 0 && s.b != 0, "{}", s.key);
        assert!(!s.shores.is_empty(), "{}", s.key);
        assert!(s.shores.iter().all(|id| (*id as usize) <= map.states.len()));
        let edges = g
            .file
            .edges
            .iter()
            .filter(|e| e.strait == Some(s.id))
            .count();
        assert_eq!(edges, 1, "{}", s.key);
    }
    // A declared strait replaces the open contact between its two zones.
    for s in &g.file.straits {
        let (a, b) = (s.a.min(s.b), s.a.max(s.b));
        assert!(
            !g.file
                .edges
                .iter()
                .any(|e| e.a == a && e.b == b && e.strait.is_none()),
            "{}",
            s.key
        );
    }
}

#[test]
fn closing_suez_sends_bombay_round_the_cape() {
    let via_suez = route("IND-GUJARAT", "GBR-KENT-SUSSEX", &open()).unwrap();
    assert!(zones(&via_suez).contains(&"red_sea"));
    let round = route("IND-GUJARAT", "GBR-KENT-SUSSEX", &closing(&["suez_canal"])).unwrap();
    assert!(!zones(&round).contains(&"red_sea"));
    assert!(
        zones(&round)
            .iter()
            .any(|z| z.starts_with("south_atlantic"))
    );
    assert!(
        round.km > via_suez.km + 3000,
        "{} vs {}",
        round.km,
        via_suez.km
    );
}

#[test]
fn closing_malacca_goes_through_sunda_or_lombok() {
    let g = &load().0;
    let open_route = route("IND-BENGAL", "CHN-GD", &open()).unwrap();
    assert!(open_route.straits.contains(&g.strait("malacca").unwrap()));
    let round = route("IND-BENGAL", "CHN-GD", &closing(&["malacca"])).unwrap();
    let used: Vec<u16> = round.straits.clone();
    assert!(
        used.contains(&g.strait("sunda").unwrap()) || used.contains(&g.strait("lombok").unwrap()),
        "{:?}",
        zones(&round)
    );
    assert!(round.km > open_route.km);
}

#[test]
fn closing_the_turkish_straits_shuts_the_black_sea() {
    assert!(route("RUS-KRYM", "FRA-PROVENCE", &open()).is_some());
    assert!(route("RUS-KRYM", "FRA-PROVENCE", &closing(&["bosporus"])).is_none());
    assert!(route("RUS-KRYM", "FRA-PROVENCE", &closing(&["dardanelles"])).is_none());
}

#[test]
fn murmansk_stays_open_in_winter_and_arkhangelsk_does_not() {
    let g = &load().0;
    assert_eq!(
        g.file.zones[g.zone("barents_south").unwrap() as usize - 1].ice,
        Ice::Free
    );
    let mut winter = open();
    winter.season = Season::Winter;
    assert!(route("RUS-MUR", "GBR-KENT-SUSSEX", &winter).is_some());
    assert!(route("RUS-ARKHANGELSK", "GBR-KENT-SUSSEX", &open()).is_some());
    assert!(route("RUS-ARKHANGELSK", "GBR-KENT-SUSSEX", &winter).is_none());
    winter.icebreakers = true;
    assert!(route("RUS-ARKHANGELSK", "GBR-KENT-SUSSEX", &winter).is_some());
    // Leningrad is closed in winter too, by the ice of the Gulf of Finland.
    winter.icebreakers = false;
    assert!(route("RUS-LENINGRAD", "GBR-KENT-SUSSEX", &winter).is_none());
}

fn tags(v: &[&str]) -> BTreeSet<String> {
    v.iter().map(|t| t.to_string()).collect()
}

#[test]
fn control_follows_the_shores_and_either_shore_closes() {
    let (g, map) = load();
    let owner = |s: u16| map.states[s as usize - 1].owner.as_str();
    let held = Holders::on(g, "1925-01-01");
    let oresund = g.strait("oresund").unwrap();
    let belts = g.strait("great_belt").unwrap();
    assert_eq!(g.controllers(oresund, owner, &held), tags(&["DEN", "SWE"]));
    // Sweden alone closes the Øresund, not the Belts: the Baltic stays open.
    let mut o = open();
    o.closed = g.closed_by(&BTreeSet::from(["SWE"]), owner, &held);
    assert!(o.closed.contains(&oresund) && !o.closed.contains(&belts));
    assert!(route("RUS-LENINGRAD", "GBR-KENT-SUSSEX", &o).is_some());
    // Denmark closes both; the Baltic still has the Kiel Canal, which Germany holds.
    o.closed = g.closed_by(&BTreeSet::from(["DEN"]), owner, &held);
    let via_kiel = route("RUS-LENINGRAD", "GBR-KENT-SUSSEX", &o).unwrap();
    assert!(via_kiel.straits.contains(&g.strait("kiel_canal").unwrap()));
    o.closed = g.closed_by(&BTreeSet::from(["DEN", "GER"]), owner, &held);
    assert!(route("RUS-LENINGRAD", "GBR-KENT-SUSSEX", &o).is_none());
}

#[test]
fn a_holder_by_treaty_comes_before_the_shores() {
    let (g, map) = load();
    let owner = |s: u16| map.states[s as usize - 1].owner.as_str();
    let suez = g.strait("suez_canal").unwrap();
    let panama = g.strait("panama_canal").unwrap();
    // In 1925 the British hold the Suez Canal zone and the United States the Panama Canal
    // Zone: Egypt alone does not close Suez, nor Panama its canal.
    let held = Holders::on(g, "1925-01-01");
    assert_eq!(g.controllers(suez, owner, &held), tags(&["GBR"]));
    assert_eq!(g.controllers(panama, owner, &held), tags(&["USA"]));
    assert!(
        !g.closed_by(&BTreeSet::from(["EGY"]), owner, &held)
            .contains(&suez)
    );
    assert!(
        g.closed_by(&BTreeSet::from(["GBR"]), owner, &held)
            .contains(&suez)
    );
    assert!(
        !g.closed_by(&BTreeSet::from(["PAN"]), owner, &held)
            .contains(&panama)
    );
    // Without the holders (phase 4 removed them), the owner of the shore closes.
    let none = Holders::none();
    assert!(
        g.closed_by(&BTreeSet::from(["EGY"]), owner, &none)
            .contains(&suez)
    );
    assert!(
        g.closed_by(&BTreeSet::from(["PAN"]), owner, &none)
            .contains(&panama)
    );
    // Only these two are held in 1925.
    assert_eq!(held.0.keys().copied().collect::<Vec<_>>(), {
        let mut v = vec![suez, panama];
        v.sort();
        v
    });
}

#[test]
fn panama_opens_in_1914_and_magellan_has_the_horn_behind_it() {
    let before = route("CHL-CENTRO", "USA-NY", &RouteOptions::open("1913-01-01"));
    let after = route("CHL-CENTRO", "USA-NY", &open()).unwrap();
    let before = before.unwrap();
    assert!(after.km < before.km);
    assert!(zones(&after).contains(&"caribbean_sea"));
    // Closing Magellan still leaves the way round Cape Horn.
    let round = route(
        "CHL-CENTRO",
        "ARG-BUENOS-AIRES",
        &closing(&["magellan", "panama_canal"]),
    );
    assert!(round.is_some());
}
