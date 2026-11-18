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
    /// Influence of the country in focus over each other one (`on`: of each over it), as a
    /// share of the influenced country's GDP. `issue` 0 is the largest issue, 1–5 one issue.
    Influence {
        issue: usize,
        on: bool,
    },
    /// Blocs of one kind (index into [`crate::blocs_view::KINDS`]); customs ties hatched.
    Blocs(usize),
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
        Lens::Political | Lens::Sea | Lens::Influence { .. } | Lens::Blocs(_) => None,
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
    if let Lens::Influence { issue, on } = lens {
        return influence_palette(world, out, issue, on);
    }
    if let Lens::Blocs(kind) = lens {
        return blocs_palette(world, out, kind);
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

/// The country the influence lens looks at.
pub const FOCUS: [u8; 3] = [205, 165, 70];
/// Influence ramp: from the paper of "none measured" to deep blue.
pub const INFLUENCE_HIGH: [u8; 3] = [30, 70, 140];
/// The ramp is logarithmic, from 0.01 % to 10 % of the influenced country's GDP.
pub const INFLUENCE_DECADES: [f64; 4] = [0.0001, 0.001, 0.01, 0.1];

/// Where a share of GDP falls on the influence ramp, 0–1.
pub fn influence_t(share: f64) -> f32 {
    if share <= 0.0 {
        return 0.0;
    }
    ((share.log10() + 4.0) / 3.0).clamp(0.0, 1.0) as f32
}

pub fn influence_color(share: f64) -> [u8; 4] {
    mix(EMPTY, INFLUENCE_HIGH, 0.12 + 0.88 * influence_t(share))
}

/// The share for country `tag` in the lens: over it (or of it over the focus, `on`), the given
/// issue or the largest. `None` where nothing is measured.
pub fn influence_of(
    view: &crate::influence_view::InfluenceView,
    tag: &str,
    issue: usize,
    on: bool,
) -> Option<f64> {
    let pairs = if on { &view.on } else { &view.over };
    let p = pairs.iter().find(|p| p.other == tag)?;
    let v = if issue == 0 {
        p.top()
    } else {
        p.issues.get(issue - 1)?.influence
    };
    (v > 0.0).then_some(v)
}

fn influence_palette(world: &World, mut out: Vec<[u8; 4]>, issue: usize, on: bool) -> Vec<[u8; 4]> {
    let empty = [EMPTY[0], EMPTY[1], EMPTY[2], 255];
    let view = world.live_influence.as_deref();
    for p in &world.provinces {
        let color = match (p.owner, view) {
            (Some(c), Some(v)) => {
                let tag = world.countries[c].tag();
                if tag == v.tag {
                    [FOCUS[0], FOCUS[1], FOCUS[2], 255]
                } else {
                    influence_of(v, tag, issue, on).map_or(empty, influence_color)
                }
            }
            _ => empty,
        };
        out[p.def.id as usize] = color;
    }
    out
}

/// Colours of the blocs, in the order the legend lists them; each comes with its name there.
pub const BLOC_COLORS: [[u8; 3]; 10] = [
    [178, 62, 52],
    [52, 96, 170],
    [70, 140, 70],
    [150, 90, 170],
    [210, 140, 40],
    [40, 150, 150],
    [140, 100, 60],
    [200, 90, 140],
    [100, 110, 40],
    [90, 90, 90],
];

/// Colour of the bloc at `i` of the kind's list: the leader's full, the members' lighter.
pub fn bloc_color(i: usize, leader: bool) -> [u8; 4] {
    let c = BLOC_COLORS[i % BLOC_COLORS.len()];
    if leader {
        [c[0], c[1], c[2], 255]
    } else {
        mix(c, [236, 228, 210], 0.35)
    }
}

/// Alpha that asks the shader to hatch a province (customs ties in the blocs lens).
pub const HATCH_ALPHA: u8 = 128;

/// The blocs of the kind, in the order the legend lists them: largest first.
pub fn blocs_of_kind(view: &crate::blocs_view::BlocsView, kind: usize) -> Vec<usize> {
    let Some(k) = crate::blocs_view::KINDS.get(kind) else {
        return Vec::new();
    };
    let mut list: Vec<usize> = (0..view.blocs.len())
        .filter(|&i| view.blocs[i].kind == *k)
        .collect();
    list.sort_by(|a, b| {
        view.blocs[*b]
            .members
            .len()
            .cmp(&view.blocs[*a].members.len())
            .then_with(|| view.blocs[*a].key.cmp(&view.blocs[*b].key))
    });
    list
}

fn blocs_palette(world: &World, mut out: Vec<[u8; 4]>, kind: usize) -> Vec<[u8; 4]> {
    let empty = [EMPTY[0], EMPTY[1], EMPTY[2], 255];
    let Some(view) = world.live_blocs.as_deref() else {
        for p in &world.provinces {
            out[p.def.id as usize] = empty;
        }
        return out;
    };
    let order = blocs_of_kind(view, kind);
    let k = crate::blocs_view::KINDS[kind.min(crate::blocs_view::KINDS.len() - 1)];
    let member = crate::blocs_view::by_country(view, k);
    let customs = if k == bf_sim::influence::BlocKind::Military {
        crate::blocs_view::in_customs(view)
    } else {
        Vec::new()
    };
    for p in &world.provinces {
        let mut color = empty;
        if let Some(c) = p.owner {
            let tag = world.countries[c].tag();
            if let Some((_, b)) = member.iter().find(|(t, _)| t == tag)
                && let Some(i) = order.iter().position(|x| x == b)
            {
                // A bloc without a leader paints every member in the full tone.
                let leader = view.blocs[*b].leader.as_deref().is_none_or(|l| l == tag);
                color = bloc_color(i, leader);
            }
            if customs.contains(&tag) {
                color[3] = HATCH_ALPHA;
            }
        }
        out[p.def.id as usize] = color;
    }
    out
}
