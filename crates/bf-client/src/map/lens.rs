//! Map modes ("lenses"): political, one mineral resource, climate, one land use. Each recolours
//! the provinces through the palette table; borders, relief and labels stay.

use bf_sim::resources::{CLIMATES, FARMS, RESOURCES};

use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lens {
    #[default]
    Political,
    /// Index into `RESOURCES`.
    Resource(usize),
    Climate,
    /// Index into `FARMS`.
    Farm(usize),
    /// Goods made and used in the state, live from the simulation: index into [`LOCAL_GOODS`].
    Local(usize),
    /// Sea zones, straits, ice and routes over a quieted political map (`map::sea`).
    Sea,
}

/// Goods the infrastructure lens shows, in the simulation's `Catalogue::local` order (catalogue
/// order; `crates/bf-sim/tests/economy_catalogue.rs` holds it).
pub const LOCAL_GOODS: [&str; 2] = ["electricity", "transportation"];
/// The lens opens on transport.
pub const LOCAL_DEFAULT: usize = 1;

/// Shortage to plenty: red, ochre, green.
pub fn met_color(share: f32) -> [u8; 4] {
    const SHORT: [u8; 3] = [190, 60, 50];
    const HALF: [u8; 3] = [215, 170, 70];
    const MET: [u8; 3] = [80, 150, 80];
    if share < 0.5 {
        mix(SHORT, HALF, share * 2.0)
    } else {
        mix(HALF, MET, (share - 0.5) * 2.0)
    }
}

impl Lens {
    pub fn resource_key(self) -> Option<&'static str> {
        match self {
            Lens::Resource(i) => RESOURCES.get(i).copied(),
            _ => None,
        }
    }

    pub fn farm_key(self) -> Option<&'static str> {
        match self {
            Lens::Farm(i) => FARMS.get(i).copied(),
            _ => None,
        }
    }
}

/// Paper tone for land with nothing to show in the current lens.
pub const EMPTY: [u8; 3] = [196, 188, 170];

/// Köppen colours, close to the usual classification maps so they read at once.
pub const CLIMATE_COLORS: [[u8; 3]; 14] = [
    [0, 70, 160],    // tropical rainforest
    [40, 120, 220],  // tropical monsoon
    [110, 170, 235], // tropical savanna
    [220, 60, 50],   // hot desert
    [240, 150, 150], // cold desert
    [230, 150, 50],  // hot steppe
    [240, 205, 110], // cold steppe
    [235, 225, 60],  // mediterranean
    [140, 210, 90],  // humid subtropical
    [60, 160, 70],   // oceanic
    [60, 190, 200],  // humid continental
    [120, 90, 170],  // subarctic
    [150, 150, 150], // tundra
    [235, 240, 245], // ice cap
];

/// Hue of each resource's ramp.
fn resource_hue(key: &str) -> [u8; 3] {
    match key {
        "coal" | "oil" | "natural_gas" => [40, 40, 48],
        "iron_ore" | "manganese" | "chromium" => [150, 60, 40],
        "copper" => [190, 95, 30],
        "bauxite" => [170, 50, 70],
        "gold" => [200, 150, 20],
        "uranium" => [60, 150, 40],
        "rare_earths" => [110, 40, 160],
        "phosphate" | "potash" | "sulfur" => [160, 150, 20],
        _ => [40, 90, 140],
    }
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), 255]
}

/// Colour of a deposit of `size` (1–5) of `key` on the resource ramp.
pub fn ramp_color(key: &str, size: u8) -> [u8; 4] {
    mix(
        EMPTY,
        resource_hue(key),
        0.25 + 0.75 * (f32::from(size) / 5.0),
    )
}

/// Colour of one state in the lens, `None` meaning "nothing here".
pub fn state_color(world: &World, state: usize, lens: Lens, year: i32) -> Option<[u8; 4]> {
    match lens {
        Lens::Political | Lens::Sea => None,
        Lens::Resource(i) => {
            let key = RESOURCES[i];
            let size = world.deposits[state]
                .iter()
                .filter(|d| d.resource == key && d.discovered <= year)
                .map(|d| d.size)
                .max()?;
            Some(ramp_color(key, size))
        }
        Lens::Climate => {
            let land = world.land[state].as_ref()?;
            let (key, _) = land.climate.iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
            let i = CLIMATES.iter().position(|c| c == key)?;
            let c = CLIMATE_COLORS[i];
            Some([c[0], c[1], c[2], 255])
        }
        Lens::Farm(i) => {
            let key = FARMS[i];
            let land = world.land[state].as_ref()?;
            let s = land.farms.iter().find(|(k, _)| k == key).map(|(_, v)| *v)?;
            if s <= 0.0 {
                return None;
            }
            Some(mix(EMPTY, [40, 120, 50], s))
        }
        Lens::Local(i) => {
            let v = world.live_local.get(state)?.get(i).copied()?;
            (v >= 0.0).then(|| met_color(v))
        }
    }
}

/// Province palette for a lens; political falls back to country colours.
pub fn palette(world: &World, lens: Lens, year: i32) -> Vec<[u8; 4]> {
    let mut out = super::political_palette(world);
    if lens == Lens::Political {
        return out;
    }
    if lens == Lens::Sea {
        // The land stays political, faded, so the sea reads first.
        for c in &mut out {
            for v in c.iter_mut().take(3) {
                *v = ((*v as u16 + 2 * 150) / 3) as u8;
            }
        }
        return out;
    }
    let mut by_state = vec![None; world.states.len()];
    for (i, slot) in by_state.iter_mut().enumerate() {
        *slot = state_color(world, i, lens, year);
    }
    for p in &world.provinces {
        let color = p
            .state
            .and_then(|s| by_state[s])
            .unwrap_or([EMPTY[0], EMPTY[1], EMPTY[2], 255]);
        out[p.def.id as usize] = color;
    }
    out
}
