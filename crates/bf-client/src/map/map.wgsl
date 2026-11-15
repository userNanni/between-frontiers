// Political map: one draw call over the map rectangle.
//
// `ids` holds the province id of every texel (0 = water). `palette` and `owners` are indexed by
// province id, laid out in rows of 256; `owners` also carries the state of each province. `fields` is a quarter-resolution distance field:
// r = distance from water to the nearest land, g = distance from land to the nearest border.

struct Uniforms {
    view_origin: vec2<f32>,  // texel at the top-left corner of the rectangle
    view_size: vec2<f32>,    // texels covered by the rectangle
    map_size: vec2<f32>,
    texel_per_px: f32,
    time: f32,
    hovered: u32,
    selected: u32,
    dim: f32,
    relief: f32,
    pastel: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var ids: texture_2d<u32>;
@group(0) @binding(2) var palette: texture_2d<f32>;
@group(0) @binding(3) var owners: texture_2d<u32>;
@group(0) @binding(4) var fields: texture_2d<f32>;
@group(0) @binding(5) var field_sampler: sampler;
@group(0) @binding(6) var relief_tex: texture_2d<f32>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    // Full-rectangle triangle; egui sets the viewport to the callback rectangle.
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    var out: VsOut;
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

fn id_at(t: vec2<f32>) -> u32 {
    let size = vec2<i32>(u.map_size);
    let p = clamp(vec2<i32>(floor(t)), vec2<i32>(0, 0), size - vec2<i32>(1, 1));
    if (t.x < 0.0 || t.y < 0.0 || t.x >= u.map_size.x || t.y >= u.map_size.y) {
        return 0u;
    }
    return textureLoad(ids, p, 0).r;
}

fn table_coord(id: u32) -> vec2<i32> {
    return vec2<i32>(i32(id % 256u), i32(id / 256u));
}

// The owner table holds the owner index in bits 0-14, the colony flag in bit 15 and the state id
// in bits 16-31.
fn owner_of(id: u32) -> u32 {
    if (id == 0u) { return 0u; }
    return textureLoad(owners, table_coord(id), 0).r & 0x7FFFu;
}

fn state_of(id: u32) -> u32 {
    if (id == 0u) { return 0u; }
    return textureLoad(owners, table_coord(id), 0).r >> 16u;
}

// Whether the neighbour `n` is land of another state (water is a coast, not a state border).
fn other_state(n: u32, state: u32) -> bool {
    return n != 0u && state_of(n) != state;
}

fn is_colony(id: u32) -> bool {
    if (id == 0u) { return false; }
    return (textureLoad(owners, table_coord(id), 0).r & 0x8000u) != 0u;
}

fn hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(0.1031, 0.1030));
    let r = q + dot(q, q.yx + 33.33);
    return fract((r.x + r.y) * r.x);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let s = f * f * (3.0 - 2.0 * f);
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

const INK: vec3<f32> = vec3<f32>(0.075, 0.063, 0.051);
const GOLD: vec3<f32> = vec3<f32>(0.902, 0.757, 0.392);
const SEA_DEEP: vec3<f32> = vec3<f32>(0.106, 0.180, 0.231);
const SEA_SHALLOW: vec3<f32> = vec3<f32>(0.255, 0.384, 0.427);
const BEYOND: vec3<f32> = vec3<f32>(0.043, 0.059, 0.078);
// Country colours are softened toward this paper tone, so the relief reads through them.
const PAPER: vec3<f32> = vec3<f32>(0.93, 0.89, 0.80);
const SUNLIT: vec3<f32> = vec3<f32>(1.0, 0.97, 0.90);
const SHADOW_TINT: vec3<f32> = vec3<f32>(0.80, 0.84, 0.95);

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let t = u.view_origin + in.uv * u.view_size;
    if (t.y < 0.0 || t.y >= u.map_size.y || t.x < 0.0 || t.x >= u.map_size.x) {
        return vec4<f32>(BEYOND * (1.0 - u.dim), 1.0);
    }
    let id = id_at(t);
    let field = textureSampleLevel(fields, field_sampler, t / u.map_size, 0.0);
    let grain = value_noise(t * 0.35) * 0.5 + value_noise(t * 0.06) * 0.5;

    var col: vec3<f32>;
    if (id == 0u) {
        // Sea: lighter near the coast, with two thin contour lines like an old atlas.
        let d = field.r;
        col = mix(SEA_SHALLOW, SEA_DEEP, smoothstep(0.0, 0.55, d));
        let ring1 = 1.0 - smoothstep(0.0, 0.012, abs(d - 0.06));
        let ring2 = 1.0 - smoothstep(0.0, 0.010, abs(d - 0.14));
        col = mix(col, SEA_SHALLOW * 1.25, ring1 * 0.35 + ring2 * 0.2);
        col *= 0.94 + grain * 0.08;
    } else {
        let owner = owner_of(id);
        col = mix(textureLoad(palette, table_coord(id), 0).rgb, PAPER, u.pastel);
        // Terrain: the hillshade is centred on 0.5 (flat). Light faces get warmer and brighter,
        // shadows darker and slightly cooler.
        let hill = (textureSampleLevel(relief_tex, field_sampler, t / u.map_size, 0.0).r - 0.5) * 2.0 * u.relief;
        // Small departures are scanning noise on plains: fade them out, keep slopes and ridges.
        let lit = clamp(hill * 1.5, -1.0, 1.0) * smoothstep(0.04, 0.22, abs(hill));
        col = col * (1.0 + lit * 0.55);
        col = mix(col, SUNLIT, max(lit, 0.0) * 0.20);
        col = col * mix(vec3<f32>(1.0), SHADOW_TINT, max(-lit, 0.0) * 0.5);
        // Darker rim just inside every border, like ink soaked into paper.
        let rim = 1.0 - smoothstep(0.0, 0.10, field.g);
        col = mix(col, col * 0.72, rim * 0.55);
        col *= 0.95 + grain * 0.07;
        if (is_colony(id)) {
            // Colonies: same colour as the owner, with diagonal hatching in screen space.
            let stripe = fract((in.pos.x + in.pos.y) / 9.0);
            let ink = smoothstep(0.30, 0.38, stripe) * (1.0 - smoothstep(0.62, 0.70, stripe));
            col = mix(col, col * 0.74, ink * 0.75);
        }
        if (owner != 0u && owner == u.hovered && owner != u.selected) {
            col = mix(col, vec3<f32>(1.0), 0.14);
        }
        if (u.selected != 0u) {
            if (owner == u.selected) {
                col = mix(col, vec3<f32>(1.0), 0.18);
            } else {
                col = mix(col, vec3<f32>(dot(col, vec3<f32>(0.3, 0.59, 0.11))), 0.35) * 0.8;
            }
        }
    }

    // Borders, kept about 1.3 screen pixels wide at every zoom.
    let w = max(u.texel_per_px * 1.3, 0.02);
    let n0 = id_at(t + vec2<f32>(w, 0.0));
    let n1 = id_at(t - vec2<f32>(w, 0.0));
    let n2 = id_at(t + vec2<f32>(0.0, w));
    let n3 = id_at(t - vec2<f32>(0.0, w));
    let me = owner_of(id);
    let o0 = owner_of(n0);
    let o1 = owner_of(n1);
    let o2 = owner_of(n2);
    let o3 = owner_of(n3);
    let country_edge = id != 0u && (o0 != me || o1 != me || o2 != me || o3 != me);
    let province_edge = id != 0u && (n0 != id || n1 != id || n2 != id || n3 != id);
    let coast = id != 0u && (n0 == 0u || n1 == 0u || n2 == 0u || n3 == 0u);
    // State borders: a hint at world scale; from mid zoom on, darker and twice as wide as
    // province borders, so the two read apart.
    let state_k = 1.0 - smoothstep(1.5, 6.0, u.texel_per_px);
    let st = state_of(id);
    let w2 = w * (1.0 + state_k);
    let state_edge = id != 0u && (other_state(n0, st) || other_state(n1, st)
        || other_state(n2, st) || other_state(n3, st)
        || other_state(id_at(t + vec2<f32>(w2, 0.0)), st)
        || other_state(id_at(t - vec2<f32>(w2, 0.0)), st)
        || other_state(id_at(t + vec2<f32>(0.0, w2)), st)
        || other_state(id_at(t - vec2<f32>(0.0, w2)), st));

    if (state_edge && !country_edge) {
        col = mix(col, col * 0.42, 0.12 + state_k * 0.63);
    } else if (province_edge && !country_edge) {
        // Province borders, fainter, fade in as the camera zooms in.
        let k = 1.0 - smoothstep(0.6, 2.2, u.texel_per_px);
        col = mix(col, col * 0.62, k * 0.5);
    }
    if (country_edge) {
        col = mix(col, INK, select(0.85, 0.7, coast));
    }
    if (u.selected != 0u && id != 0u) {
        let sel_edge = (me == u.selected) != (o0 == u.selected)
            || (me == u.selected) != (o1 == u.selected)
            || (me == u.selected) != (o2 == u.selected)
            || (me == u.selected) != (o3 == u.selected);
        if (sel_edge && me == u.selected) {
            col = GOLD;
        }
    }

    return vec4<f32>(col * (1.0 - u.dim), 1.0);
}
