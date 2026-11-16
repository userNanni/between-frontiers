//! The pure functions of the war: what a unit costs, who can be called up, what reaches a front,
//! how a front's elements are spread over its segments, one week of combat on land, at sea and in
//! the air, and war exhaustion. Deterministic and in fixed point: no clock, no random draw, and
//! every result carries the terms it was made of, which is what the screens will show.
//!
//! Rules and their numbers live in `data/military/rules.ron` ([`Rules`]); the reasoning is in
//! `docs/war.pt-BR.md`.

use laplace_schelling::Fixed;

use super::catalogue::{Military, Mobility, Mode, Role, Rules, Spec, Terrain, TypeIx};
use crate::econ::catalogue::GoodIx;

fn clamp01(x: Fixed) -> Fixed {
    x.max(Fixed::ZERO).min(Fixed::ONE)
}

/// `floor + (1 − floor) × x`, with `x` clamped to 0..1.
fn lift(floor: Fixed, x: Fixed) -> Fixed {
    floor + (Fixed::ONE - floor) * clamp01(x)
}

fn ratio(num: Fixed, den: Fixed) -> Option<Fixed> {
    den.is_positive().then(|| num / den)
}

fn count(n: u64) -> Fixed {
    Fixed::from_int(n as i64)
}

/// Adds `qty` of good `g` to a list kept sorted by good.
fn add_good(list: &mut Vec<(GoodIx, Fixed)>, g: GoodIx, qty: Fixed) {
    match list.binary_search_by_key(&g, |(k, _)| *k) {
        Ok(i) => list[i].1 += qty,
        Err(i) => list.insert(i, (g, qty)),
    }
}

/// Splits `total` in proportion to `weights` (non-negative) by largest remainder; ties go to the
/// lower index. Equal shares when every weight is zero.
pub fn apportion(total: u64, weights: &[Fixed]) -> Vec<u64> {
    if weights.is_empty() {
        return Vec::new();
    }
    let raw: Vec<u128> = weights.iter().map(|w| w.raw().max(0) as u128).collect();
    let sum: u128 = raw.iter().sum();
    let (raw, sum) = if sum == 0 {
        (vec![1u128; weights.len()], weights.len() as u128)
    } else {
        (raw, sum)
    };
    let t = total as u128;
    let mut out: Vec<u64> = raw.iter().map(|w| (t * w / sum) as u64).collect();
    let mut left = total - out.iter().sum::<u64>();
    let mut order: Vec<usize> = (0..raw.len()).collect();
    order.sort_by(|&a, &b| {
        let ra = t * raw[a] % sum;
        let rb = t * raw[b] % sum;
        rb.cmp(&ra).then(a.cmp(&b))
    });
    for i in order {
        if left == 0 {
            break;
        }
        out[i] += 1;
        left -= 1;
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Costs.

/// What raising some elements of a unit takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaiseCost {
    pub men: u64,
    pub goods: Vec<(GoodIx, Fixed)>,
    pub training_weeks: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    Peace,
    /// On the move: fuel multiplied by `moving_fuel`.
    Moving,
    /// In combat: fuel as on the move, ammunition multiplied by `combat_ammunition`.
    Combat,
}

impl Military {
    /// Men and equipment to raise `elements` of one generation of a unit type.
    pub fn raise_cost(&self, ty: TypeIx, generation: usize, elements: u64) -> RaiseCost {
        let s = self.spec(ty, generation);
        RaiseCost {
            men: s.men as u64 * elements,
            goods: s
                .equipment
                .iter()
                .map(|(g, q)| (*g, *q * elements as i64))
                .collect(),
            training_weeks: s.training_weeks,
        }
    }

    /// Goods `elements` consume in a week: the type's upkeep plus the maintenance of its
    /// equipment, with ammunition and fuel scaled by the activity and everything by winter.
    pub fn upkeep(
        &self,
        ty: TypeIx,
        generation: usize,
        elements: u64,
        activity: Activity,
        winter: bool,
    ) -> Vec<(GoodIx, Fixed)> {
        let r = &self.rules;
        let s = self.spec(ty, generation);
        let mut out: Vec<(GoodIx, Fixed)> = s.upkeep.clone();
        for (g, q) in &s.equipment {
            add_good(&mut out, *g, *q * r.maintenance);
        }
        for (g, q) in &mut out {
            if *g == r.ammunition && activity == Activity::Combat {
                *q *= r.combat_ammunition;
            }
            if *g == r.fuel && activity != Activity::Peace {
                *q *= r.moving_fuel;
            }
            if winter {
                *q *= r.winter_upkeep;
            }
            *q = *q * elements as i64;
        }
        out
    }

    /// Weight of a list of goods, in tonnes. Goods without a weight count as nothing.
    pub fn tonnes(&self, goods: &[(GoodIx, Fixed)]) -> Fixed {
        goods
            .iter()
            .filter_map(|(g, q)| self.rules.tonnes_per_unit[*g as usize].map(|t| *q * t))
            .sum()
    }

    /// Value of a list of goods in lots of US$ 1,000 at base prices.
    pub fn value_lots(&self, goods: &[(GoodIx, Fixed)]) -> Fixed {
        goods
            .iter()
            .map(|(g, q)| *q * self.rules.lots_per_unit[*g as usize])
            .sum()
    }

    /// Land fire of the equipment held: each good times its `fire_per_unit`.
    pub fn land_fire(&self, held: &[(GoodIx, Fixed)]) -> Fixed {
        held.iter()
            .map(|(g, q)| *q * self.rules.fire_per_unit[*g as usize])
            .sum()
    }

    /// Protection of the equipment held: the armour share of its value times
    /// `protection_per_armor_share`, up to `protection_max`.
    pub fn protection(&self, held: &[(GoodIx, Fixed)]) -> Fixed {
        let total = self.value_lots(held);
        let armor: Fixed = held
            .iter()
            .filter(|(g, _)| *g == self.rules.armor)
            .map(|(g, q)| *q * self.rules.lots_per_unit[*g as usize])
            .sum();
        ratio(armor, total).map_or(Fixed::ZERO, |share| {
            (share * self.rules.protection_per_armor_share).min(self.rules.protection_max)
        })
    }

    /// A land side made of `elements` of one generation, fully equipped.
    pub fn land_side(&self, ty: TypeIx, generation: usize, elements: u64) -> LandSide {
        let s = self.spec(ty, generation);
        let held: Vec<(GoodIx, Fixed)> = s
            .equipment
            .iter()
            .map(|(g, q)| (*g, *q * elements as i64))
            .collect();
        LandSide {
            men: s.men as u64 * elements,
            fire: self.land_fire(&held),
            protection: self.protection(&held),
            training: Fixed::ONE,
            supply: Fixed::ONE,
            air: Fixed::from_ratio(1, 2),
            posture: Posture::Defend,
            mobility: s.mobility,
            fortification: Fixed::ZERO,
            landing: false,
        }
    }

    fn role_value(&self, s: &Spec, role: Role) -> Fixed {
        self.value_lots(&s.equipment) * Fixed::from_ratio(s.role(role) as i64, 1000)
    }

    /// `count` ships of one generation. The air wing of a carrier is added with
    /// [`ShipGroup::with_air_wing`].
    pub fn ship_group(&self, ty: TypeIx, generation: usize, count: u64) -> ShipGroup {
        let s = self.spec(ty, generation);
        ShipGroup {
            count,
            tonnage: Fixed::from_int(s.tonnage as i64),
            gun: self.role_value(s, Role::Gun) + self.role_value(s, Role::AntiShip),
            torpedo: self.role_value(s, Role::Torpedo),
            asw: self.role_value(s, Role::Asw),
            anti_air: self.role_value(s, Role::AntiAir),
            air: Fixed::ZERO,
            aircraft: s.aircraft,
            submerged: self.types[ty as usize].submerged,
        }
    }

    /// `count` aircraft of one generation.
    pub fn air_group(&self, ty: TypeIx, generation: usize, count: u64) -> AirGroup {
        let s = self.spec(ty, generation);
        AirGroup {
            count,
            air_to_air: self.role_value(s, Role::AirToAir),
            ground: self.role_value(s, Role::GroundAttack),
            strategic: self.role_value(s, Role::Strategic),
            anti_ship: self.role_value(s, Role::AntiShip),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Mobilization.

/// Men in an age band, ages `from` (inclusive) to `to` (exclusive), spread evenly inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgeBand {
    pub from: u32,
    pub to: u32,
    pub men: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mobilization {
    /// Men within `service_age`.
    pub of_age: u64,
    /// Of age × the `conscription` effect of the law in force.
    pub callable: u64,
    /// Callable men kept in essential work.
    pub reserved: u64,
    /// Already under arms.
    pub serving: u64,
    /// Callable − reserved − serving.
    pub available: u64,
}

/// Who can be called up, from the male age pyramid and the `conscription` effect (per mille) of
/// the law in force.
pub fn mobilizable(
    rules: &Rules,
    bands: &[AgeBand],
    conscription_per_mille: u32,
    serving: u64,
) -> Mobilization {
    let (lo, hi) = rules.service_age;
    let of_age: u64 = bands
        .iter()
        .filter(|b| b.to > b.from)
        .map(|b| {
            let overlap = hi.min(b.to).saturating_sub(lo.max(b.from)) as u128;
            (b.men as u128 * overlap / (b.to - b.from) as u128) as u64
        })
        .sum();
    let callable = (of_age as u128 * conscription_per_mille.min(1000) as u128 / 1000) as u64;
    let reserved = (callable as u128 * rules.reserved_share.raw() as u128 / 1000) as u64;
    Mobilization {
        of_age,
        callable,
        reserved,
        serving,
        available: callable.saturating_sub(reserved).saturating_sub(serving),
    }
}

// ---------------------------------------------------------------------------------------------
// Supply.

/// One leg of the route from the depot to a front.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leg {
    pub mode: Mode,
    pub km: Fixed,
    /// Tonne-kilometres a week the leg can give the front.
    pub capacity_tkm: Fixed,
    /// Extra share lost on the leg (submarines, blockade, bombing), 0 to 1.
    pub interdiction: Fixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Supply {
    pub need: Fixed,
    pub delivered: Fixed,
    /// Delivered ÷ need, at most 1.
    pub fill: Fixed,
    /// Tonnes a week each leg can carry (capacity ÷ distance).
    pub throughput: Vec<Fixed>,
    /// The leg with the least throughput, the first one on a tie.
    pub bottleneck: Option<usize>,
    /// Share of what is sent that arrives.
    pub survival: Fixed,
}

/// Tonne-kilometres that `lots` of the `transportation` good move.
pub fn transport_capacity(rules: &Rules, lots: Fixed) -> Fixed {
    lots * rules.tkm_per_transport_lot
}

/// What reaches a front that needs `need_t` tonnes a week along `legs`.
pub fn supply(rules: &Rules, need_t: Fixed, legs: &[Leg]) -> Supply {
    let need = need_t.max(Fixed::ZERO);
    let throughput: Vec<Fixed> = legs
        .iter()
        .map(|l| l.capacity_tkm.max(Fixed::ZERO) / l.km.max(Fixed::ONE))
        .collect();
    let bottleneck = throughput
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(&b.0)))
        .map(|(i, _)| i);
    let survival = legs.iter().fold(Fixed::ONE, |s, l| {
        let lost = (rules.mode_loss(l.mode) * l.km.max(Fixed::ZERO) / 1000 + l.interdiction)
            .min(Fixed::ONE)
            .max(Fixed::ZERO);
        s * (Fixed::ONE - lost)
    });
    let delivered = match bottleneck {
        None => need,
        Some(_) if !survival.is_positive() => Fixed::ZERO,
        Some(b) => {
            let wanted = need / survival;
            (wanted.min(throughput[b]) * survival).min(need)
        }
    };
    Supply {
        need,
        delivered,
        fill: ratio(delivered, need).map_or(Fixed::ONE, |f| f.min(Fixed::ONE)),
        throughput,
        bottleneck,
        survival,
    }
}

// ---------------------------------------------------------------------------------------------
// Terrain.

/// Shares of terrain classes in a segment; they need not add to 1.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainMix(pub Vec<(Terrain, Fixed)>);

impl TerrainMix {
    pub fn only(t: Terrain) -> Self {
        Self(vec![(t, Fixed::ONE)])
    }

    /// Average of `f` weighted by the shares; `f(Open)` when there are none.
    pub fn weighted(&self, f: impl Fn(Terrain) -> Fixed) -> Fixed {
        let total: Fixed = self.0.iter().map(|(_, s)| (*s).max(Fixed::ZERO)).sum();
        if !total.is_positive() {
            return f(Terrain::Open);
        }
        self.0
            .iter()
            .map(|(t, s)| f(*t) * (*s).max(Fixed::ZERO))
            .sum::<Fixed>()
            / total
    }
}

/// The terrain of a state from its climate shares and forest share (`land.ron`), read at load:
/// forest takes its share, the climates share the rest; an unknown climate counts as open.
pub fn terrain_of_state(rules: &Rules, climate: &[(String, f32)], forest_share: f32) -> TerrainMix {
    let forest = Fixed::from_f64_lossy(forest_share.clamp(0.0, 1.0) as f64);
    let rest = Fixed::ONE - forest;
    let mut mix: Vec<(Terrain, Fixed)> = Vec::new();
    let mut add = |t: Terrain, s: Fixed| match mix.iter_mut().find(|(k, _)| *k == t) {
        Some(e) => e.1 += s,
        None => mix.push((t, s)),
    };
    if forest.is_positive() {
        add(Terrain::Forest, forest);
    }
    for (key, share) in climate {
        let t = rules.climate(key).unwrap_or(Terrain::Open);
        add(t, Fixed::from_f64_lossy(*share as f64) * rest);
    }
    mix.sort_by_key(|(t, _)| *t);
    TerrainMix(mix)
}

// ---------------------------------------------------------------------------------------------
// Fronts and segments.

/// A pair of neighbouring states on opposite sides: where combat and supply run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub length_km: Fixed,
    /// Land power the enemy has on the segment.
    pub enemy_power: Fixed,
    /// The segment leads to the front's objective.
    pub objective: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentShare {
    /// Length × (1 + enemy power ÷ the front's mean) × (1 + objective pull).
    pub weight: Fixed,
    /// The `min_share` floor every segment gets first.
    pub floor: u64,
    pub elements: u64,
    /// The focus segment of a third-generation front ([`crate::war::allocate_front`]).
    pub focus: bool,
}

/// Spreads a front's land elements over its segments, in the order given (the caller keeps
/// segments in a fixed order, for example by state id).
pub fn allocate(rules: &Rules, elements: u64, segments: &[Segment]) -> Vec<SegmentShare> {
    let n = segments.len() as u64;
    if n == 0 {
        return Vec::new();
    }
    let total_enemy: Fixed = segments
        .iter()
        .map(|s| s.enemy_power.max(Fixed::ZERO))
        .sum();
    let mean_enemy = total_enemy / n as i64;
    let weights: Vec<Fixed> = segments
        .iter()
        .map(|s| {
            let threat = Fixed::ONE
                + ratio(s.enemy_power.max(Fixed::ZERO), mean_enemy).unwrap_or(Fixed::ZERO);
            let pull = if s.objective {
                Fixed::ONE + rules.objective_pull
            } else {
                Fixed::ONE
            };
            s.length_km.max(Fixed::ZERO) * threat * pull
        })
        .collect();
    let floor =
        ((elements as u128 * rules.min_share.raw() as u128 / 1000) as u64).min(elements / n);
    let rest = apportion(elements - floor * n, &weights);
    weights
        .iter()
        .zip(rest)
        .map(|(w, r)| SegmentShare {
            weight: *w,
            floor,
            elements: floor + r,
            focus: false,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Land.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Posture {
    Attack,
    Defend,
}

/// One side of a segment.
#[derive(Clone, Debug, PartialEq)]
pub struct LandSide {
    pub men: u64,
    /// Fire of the equipment the side actually holds ([`Military::land_fire`]).
    pub fire: Fixed,
    /// 0 to 1 ([`Military::protection`]).
    pub protection: Fixed,
    /// Training completed, 0 to 1.
    pub training: Fixed,
    /// Ammunition delivered over ammunition needed, 0 to 1 ([`Supply::fill`]).
    pub supply: Fixed,
    /// Air superiority over the segment, 0 to 1 ([`AirOutcome::superiority`]).
    pub air: Fixed,
    pub posture: Posture,
    /// The mobility that sets the side's speed and terrain penalties.
    pub mobility: Mobility,
    /// Prepared defences, 0 to 1.
    pub fortification: Fixed,
    /// Attacking from the sea.
    pub landing: bool,
}

/// The factors of a side's power; their product is the power.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LandTerms {
    /// Thousands of men.
    pub number: Fixed,
    /// Fire per element over the reference element's.
    pub equipment: Fixed,
    pub training: Fixed,
    pub supply: Fixed,
    pub terrain: Fixed,
    pub air: Fixed,
}

impl LandTerms {
    pub fn power(&self) -> Fixed {
        self.number * self.equipment * self.training * self.supply * self.terrain * self.air
    }

    /// The terms with their keys, in display order.
    pub fn named(&self) -> [(&'static str, Fixed); 6] {
        [
            ("number", self.number),
            ("equipment", self.equipment),
            ("training", self.training),
            ("supply", self.supply),
            ("terrain", self.terrain),
            ("air", self.air),
        ]
    }

    /// Each term over the other side's, `None` where the other's is zero. Their product is the
    /// ratio of the two powers.
    pub fn over(&self, other: &LandTerms) -> [(&'static str, Option<Fixed>); 6] {
        let o = other.named();
        let mut out = [("", None); 6];
        for (i, (k, v)) in self.named().into_iter().enumerate() {
            out[i] = (k, ratio(v, o[i].1));
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LandOutcome {
    pub engaged: bool,
    pub terms: [LandTerms; 2],
    pub power: [Fixed; 2],
    /// Casualties (killed, wounded, captured) of each side; never more than its men.
    pub losses: [u64; 2],
    /// Kilometres the front moves; positive when side 0 advances.
    pub advance_km: Fixed,
}

/// The conditions of the segment.
#[derive(Clone, Debug, PartialEq)]
pub struct LandField {
    pub terrain: TerrainMix,
    pub winter: bool,
}

fn land_terms(rules: &Rules, field: &LandField, me: &LandSide, other: &LandSide) -> LandTerms {
    let number = Fixed::from_ratio(me.men as i64, 1000);
    let equipment = ratio(me.fire, number)
        .and_then(|per| ratio(per, rules.reference_fire))
        .unwrap_or(Fixed::ZERO);
    let defending = me.posture == Posture::Defend && other.posture == Posture::Attack;
    let terrain = if defending {
        field.terrain.weighted(|t| rules.defense_factor(t))
            * (Fixed::ONE + rules.fortification_bonus * clamp01(me.fortification))
    } else {
        let attack = field
            .terrain
            .weighted(|t| rules.attack_factor(t, me.mobility));
        if me.landing {
            attack * rules.amphibious
        } else {
            attack
        }
    };
    LandTerms {
        number,
        equipment,
        training: lift(rules.training_floor, me.training),
        supply: lift(rules.supply_floor, me.supply),
        terrain,
        air: Fixed::ONE - rules.air_weight * (Fixed::ONE - clamp01(me.air)),
    }
}

/// Share of the two powers that is `mine`; `None` when both are zero.
fn share(mine: Fixed, theirs: Fixed) -> Option<Fixed> {
    ratio(mine, mine + theirs)
}

/// Losses of a side: `land_lethality` of its men at even power, up to twice that when the enemy
/// has all of it, less its protection. Scale-free: an army of 2025 against one of 2025 bleeds
/// at the same rate as one of 1925 against one of 1925.
fn casualties(rules: &Rules, enemy_share: Fixed, me: &LandSide) -> u64 {
    let rate = rules.land_lethality * enemy_share * 2 * (Fixed::ONE - clamp01(me.protection));
    let lost = Fixed::from_int(me.men as i64) * rate.min(Fixed::ONE);
    (lost.to_int_floor().max(0) as u64).min(me.men)
}

fn advance(rules: &Rules, field: &LandField, a: &LandSide, pa: Fixed, pd: Fixed) -> Fixed {
    if pa <= pd || !pa.is_positive() {
        return Fixed::ZERO;
    }
    rules.base_speed(a.mobility) * terrain_speed(rules, field, a.mobility) * (Fixed::ONE - pd / pa)
}

/// Speed factor of a mobility on the segment: terrain and winter.
pub(crate) fn terrain_speed(rules: &Rules, field: &LandField, mobility: Mobility) -> Fixed {
    field.terrain.weighted(|t| rules.speed_factor(t, mobility))
        * if field.winter {
            rules.winter_speed
        } else {
            Fixed::ONE
        }
}

/// One week of combat on a segment. Nothing happens unless one side attacks.
pub fn land_step(rules: &Rules, field: &LandField, sides: [&LandSide; 2]) -> LandOutcome {
    let terms = [
        land_terms(rules, field, sides[0], sides[1]),
        land_terms(rules, field, sides[1], sides[0]),
    ];
    let power = [terms[0].power(), terms[1].power()];
    let engaged = sides.iter().any(|s| s.posture == Posture::Attack);
    if !engaged {
        return LandOutcome {
            engaged,
            terms,
            power,
            losses: [0, 0],
            advance_km: Fixed::ZERO,
        };
    }
    let enemy = [
        share(power[1], power[0]).unwrap_or(Fixed::ZERO),
        share(power[0], power[1]).unwrap_or(Fixed::ZERO),
    ];
    let losses = [
        casualties(rules, enemy[0], sides[0]),
        casualties(rules, enemy[1], sides[1]),
    ];
    let forward = if sides[0].posture == Posture::Attack {
        advance(rules, field, sides[0], power[0], power[1])
    } else {
        Fixed::ZERO
    };
    let back = if sides[1].posture == Posture::Attack {
        advance(rules, field, sides[1], power[1], power[0])
    } else {
        Fixed::ZERO
    };
    LandOutcome {
        engaged,
        terms,
        power,
        losses,
        advance_km: forward - back,
    }
}

// ---------------------------------------------------------------------------------------------
// Sea.

/// Ships of one type and generation; firepower per ship.
#[derive(Clone, Debug, PartialEq)]
pub struct ShipGroup {
    pub count: u64,
    /// Tonnes per ship: the damage that sinks one.
    pub tonnage: Fixed,
    /// Guns and anti-ship missiles.
    pub gun: Fixed,
    pub torpedo: Fixed,
    pub asw: Fixed,
    pub anti_air: Fixed,
    /// Strike of the air wing on board, per ship.
    pub air: Fixed,
    /// Aircraft each ship can carry.
    pub aircraft: u32,
    pub submerged: bool,
}

impl ShipGroup {
    /// Puts an air wing on board: as many of `wing`'s aircraft as the ships carry strike with
    /// their anti-ship power.
    pub fn with_air_wing(mut self, wing: &AirGroup) -> Self {
        if self.count > 0 {
            let aboard = wing.count.min(self.aircraft as u64 * self.count);
            self.air = wing.anti_ship * aboard as i64 / self.count as i64;
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavalSide {
    pub groups: Vec<ShipGroup>,
    pub training: Fixed,
    /// Fuel delivered over fuel needed.
    pub supply: Fixed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NavalTerms {
    pub ships: u64,
    /// Gun fire after training and supply.
    pub gun: Fixed,
    pub torpedo: Fixed,
    /// Air strike that got through the other side's anti-air.
    pub air: Fixed,
    pub asw: Fixed,
    pub training: Fixed,
    pub supply: Fixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavalOutcome {
    pub terms: [NavalTerms; 2],
    /// Ships sunk, per group, in the order given.
    pub sunk: [Vec<u64>; 2],
    /// Damage in tonnes left on ships still afloat, per group.
    pub damage_t: [Vec<Fixed>; 2],
}

fn sum_ships(side: &NavalSide, f: impl Fn(&ShipGroup) -> Fixed) -> Fixed {
    side.groups.iter().map(|g| f(g) * g.count as i64).sum()
}

fn naval_terms(rules: &Rules, me: &NavalSide, other: &NavalSide) -> NavalTerms {
    let training = lift(rules.training_floor, me.training);
    let supply = lift(rules.supply_floor, me.supply);
    let f = training * supply;
    let air_raw = sum_ships(me, |g| g.air);
    let aa = sum_ships(other, |g| g.anti_air);
    let through = ratio(air_raw, air_raw + aa).unwrap_or(Fixed::ZERO);
    NavalTerms {
        ships: me.groups.iter().map(|g| g.count).sum(),
        gun: sum_ships(me, |g| g.gun) * f,
        torpedo: sum_ships(me, |g| g.torpedo) * f,
        air: air_raw * through * f,
        asw: sum_ships(me, |g| g.asw) * f,
        training,
        supply,
    }
}

/// Spreads `damage` tonnes over the groups that `hit` selects, by their share of tonnage.
fn sink(
    damage: Fixed,
    side: &NavalSide,
    hit: impl Fn(&ShipGroup) -> bool,
    sunk: &mut [u64],
    left: &mut [Fixed],
) {
    let total: Fixed = side
        .groups
        .iter()
        .filter(|g| hit(g))
        .map(|g| g.tonnage * g.count as i64)
        .sum();
    if !total.is_positive() || !damage.is_positive() {
        return;
    }
    for (i, g) in side.groups.iter().enumerate() {
        if !hit(g) || !g.tonnage.is_positive() {
            continue;
        }
        let fleet = g.tonnage * g.count as i64;
        let d = damage.mul_div(fleet, total);
        let s = ((d / g.tonnage).to_int_floor().max(0) as u64).min(g.count);
        sunk[i] = s;
        left[i] = (d - g.tonnage * s as i64)
            .max(Fixed::ZERO)
            .min(g.tonnage * (g.count - s) as i64);
    }
}

/// One week of action between two fleets in a sea zone.
pub fn naval_step(rules: &Rules, sides: [&NavalSide; 2]) -> NavalOutcome {
    let terms = [
        naval_terms(rules, sides[0], sides[1]),
        naval_terms(rules, sides[1], sides[0]),
    ];
    let mut sunk = [
        vec![0; sides[0].groups.len()],
        vec![0; sides[1].groups.len()],
    ];
    let mut damage_t = [
        vec![Fixed::ZERO; sides[0].groups.len()],
        vec![Fixed::ZERO; sides[1].groups.len()],
    ];
    // Torpedoes of the submarines: what an anti-submarine force fights against.
    let sub_torpedo = |side: &NavalSide| -> Fixed {
        side.groups
            .iter()
            .filter(|g| g.submerged)
            .map(|g| g.torpedo * g.count as i64)
            .sum()
    };
    let tonnes = |side: &NavalSide, under: bool| -> Fixed {
        side.groups
            .iter()
            .filter(|g| g.submerged == under)
            .map(|g| g.tonnage * g.count as i64)
            .sum()
    };
    let strike = |x: &NavalTerms| x.gun + x.torpedo + x.air;
    let rate =
        |s: Option<Fixed>| (rules.naval_lethality * s.unwrap_or(Fixed::ZERO) * 2).min(Fixed::ONE);
    for (me, foe) in [(0, 1), (1, 0)] {
        let (t, mine) = (&terms[foe], &terms[me]);
        // Each fleet loses `naval_lethality` of its tonnage at even strike, up to twice that.
        let surface = tonnes(sides[me], false) * rate(share(strike(t), strike(mine)));
        let under = tonnes(sides[me], true)
            * rate(share(
                t.asw,
                sub_torpedo(sides[me]) * mine.training * mine.supply,
            ));
        let (s, d) = (&mut sunk[me], &mut damage_t[me]);
        sink(surface, sides[me], |g| !g.submerged, s, d);
        sink(under, sides[me], |g| g.submerged, s, d);
    }
    NavalOutcome {
        terms,
        sunk,
        damage_t,
    }
}

// ---------------------------------------------------------------------------------------------
// Air.

/// Aircraft of one type and generation; power per aircraft by role.
#[derive(Clone, Debug, PartialEq)]
pub struct AirGroup {
    pub count: u64,
    pub air_to_air: Fixed,
    pub ground: Fixed,
    pub strategic: Fixed,
    pub anti_ship: Fixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AirSide {
    pub groups: Vec<AirGroup>,
    pub training: Fixed,
    /// Fuel delivered over fuel needed.
    pub supply: Fixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AirOutcome {
    /// Air-to-air power after training and supply.
    pub air_to_air: [Fixed; 2],
    /// Own air-to-air over both sides', 0 to 1 (one half when neither has any).
    pub superiority: [Fixed; 2],
    /// Aircraft lost, per group.
    pub losses: [Vec<u64>; 2],
    /// Strikes of the surviving aircraft, times superiority.
    pub ground: [Fixed; 2],
    pub strategic: [Fixed; 2],
    pub anti_ship: [Fixed; 2],
}

/// One week of contest over a zone.
pub fn air_step(rules: &Rules, sides: [&AirSide; 2]) -> AirOutcome {
    let factor =
        |s: &AirSide| lift(rules.training_floor, s.training) * lift(rules.supply_floor, s.supply);
    let f = [factor(sides[0]), factor(sides[1])];
    let aa = [0, 1].map(|i| {
        sides[i]
            .groups
            .iter()
            .map(|g| g.air_to_air * g.count as i64)
            .sum::<Fixed>()
            * f[i]
    });
    let superiority = match ratio(aa[0], aa[0] + aa[1]) {
        Some(s) => [s, Fixed::ONE - s],
        None => [Fixed::from_ratio(1, 2); 2],
    };
    let losses = [0, 1].map(|me| {
        let side = sides[me];
        let fleet: u64 = side.groups.iter().map(|g| g.count).sum();
        // `air_lethality` of the fleet at even superiority, up to twice that.
        let rate = (rules.air_lethality * superiority[1 - me] * 2).min(Fixed::ONE);
        let downed = ((count(fleet) * rate).to_int_floor().max(0) as u64).min(fleet);
        let counts: Vec<Fixed> = side.groups.iter().map(|g| count(g.count)).collect();
        apportion(downed, &counts)
    });
    let strike = |me: usize, role: fn(&AirGroup) -> Fixed| {
        sides[me]
            .groups
            .iter()
            .zip(&losses[me])
            .map(|(g, l)| role(g) * (g.count - l) as i64)
            .sum::<Fixed>()
            * f[me]
            * superiority[me]
    };
    AirOutcome {
        air_to_air: aa,
        superiority,
        ground: [strike(0, |g| g.ground), strike(1, |g| g.ground)],
        strategic: [strike(0, |g| g.strategic), strike(1, |g| g.strategic)],
        anti_ship: [strike(0, |g| g.anti_ship), strike(1, |g| g.anti_ship)],
        losses,
    }
}

// ---------------------------------------------------------------------------------------------
// Exhaustion.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExhaustionInputs {
    /// Casualties since the war began.
    pub casualties: u64,
    /// Men under arms now.
    pub under_arms: u64,
    /// Men of service age.
    pub men_of_age: u64,
    /// Share of consumer needs unmet, 0 to 1.
    pub shortage: Fixed,
    /// Share of the population under enemy occupation, 0 to 1.
    pub occupied: Fixed,
    pub weeks: u32,
    /// Share of industry destroyed, 0 to 1.
    pub industry_lost: Fixed,
    /// War spending since the war began over a year of output ([`WarLedger::spending`]).
    pub spending_share: Fixed,
    /// Rise of the price index since the war began, 0 at no rise, 1 at a doubling.
    pub price_rise: Fixed,
}

/// Points of exhaustion per term, per mille; `total` is their sum, at most 1,000.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Exhaustion {
    pub casualties: u32,
    pub mobilization: u32,
    pub shortage: u32,
    pub occupation: u32,
    pub duration: u32,
    pub industry: u32,
    pub spending: u32,
    pub prices: u32,
    pub total: u32,
}

fn points(weight: u32, share: Fixed) -> u32 {
    (clamp01(share) * weight as i64).to_int_floor() as u32
}

fn share_of(part: u64, whole: u64) -> Fixed {
    if whole == 0 {
        if part == 0 { Fixed::ZERO } else { Fixed::ONE }
    } else {
        Fixed::from_ratio(part.min(whole) as i64, whole as i64)
    }
}

pub fn exhaustion(rules: &Rules, i: &ExhaustionInputs) -> Exhaustion {
    let w = &rules.exhaustion;
    let mut e = Exhaustion {
        casualties: points(w.casualties, share_of(i.casualties, i.men_of_age)),
        mobilization: points(w.mobilization, share_of(i.under_arms, i.men_of_age)),
        shortage: points(w.shortage, i.shortage),
        occupation: points(w.occupation, i.occupied),
        duration: (w.duration_per_year as u64 * i.weeks as u64 / 52) as u32,
        industry: points(w.industry, i.industry_lost),
        spending: points(w.spending, i.spending_share),
        prices: points(w.prices, i.price_rise),
        total: 0,
    };
    e.total = (e.casualties
        + e.mobilization
        + e.shortage
        + e.occupation
        + e.duration
        + e.industry
        + e.spending
        + e.prices)
        .min(1000);
    e
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::war::catalogue::{DoctrineRule, ExhaustionWeights, NuclearRule};

    fn pm(n: i64) -> Fixed {
        Fixed::from_ratio(n, 1000)
    }

    pub(crate) fn rules() -> Rules {
        let mut attack = [[Fixed::ONE; 6]; 8];
        attack[Terrain::Forest as usize][Mobility::Tracked as usize] = pm(700);
        Rules {
            ammunition: 1,
            fuel: 2,
            armor: 3,
            fire_per_unit: vec![Fixed::ONE, Fixed::ZERO, Fixed::ZERO, Fixed::from_int(3)],
            reference_fire: Fixed::from_int(100),
            tonnes_per_unit: vec![Some(Fixed::ONE); 4],
            lots_per_unit: vec![Fixed::ONE; 4],
            maintenance: pm(2),
            combat_ammunition: Fixed::from_int(10),
            moving_fuel: Fixed::from_int(3),
            protection_per_armor_share: pm(800),
            protection_max: pm(600),
            training_floor: pm(500),
            supply_floor: pm(250),
            air_weight: pm(250),
            fortification_bonus: pm(500),
            land_lethality: pm(20),
            naval_lethality: pm(50),
            air_lethality: pm(100),
            service_age: (18, 45),
            reserved_share: pm(200),
            mode_loss: [pm(10), pm(100), pm(20), pm(10), pm(50)],
            tkm_per_transport_lot: Fixed::from_int(50_000),
            defense: [
                Fixed::ONE,
                pm(1250),
                pm(1500),
                pm(1100),
                pm(1200),
                pm(2000),
                pm(1300),
                pm(1500),
            ],
            attack,
            speed: [[Fixed::ONE; 6]; 8],
            climate_terrain: vec![("tropical_rainforest".into(), Terrain::Jungle)],
            speed_km_week: [
                Fixed::ZERO,
                Fixed::from_int(50),
                Fixed::from_int(50),
                Fixed::from_int(80),
                Fixed::from_int(150),
                Fixed::from_int(150),
            ],
            amphibious: pm(600),
            winter_upkeep: pm(1300),
            winter_speed: pm(600),
            min_share: pm(50),
            objective_pull: pm(1000),
            exhaustion: ExhaustionWeights {
                casualties: 600,
                mobilization: 200,
                shortage: 300,
                occupation: 400,
                duration_per_year: 50,
                industry: 200,
                spending: 300,
                prices: 200,
            },
            invalid_share: pm(150),
            doctrine: DoctrineRule {
                officer_education: 300,
                training: 200,
                radio: 200,
                motorization: 200,
                experience: 100,
                central_control: 400,
                up: 650,
                down: 450,
                radio_min: 500,
                fuel_min: 500,
                focus_share: 600,
                breakthrough_ratio: Fixed::from_int(3),
                exploitation_speed: Fixed::from_int(2),
            },
            nuclear: NuclearRule {
                silo_kill: 500,
                soft_kill: 900,
                bomber_penetration: 600,
                escalation_risk_per_week: 5,
            },
        }
    }

    fn side(men: u64, posture: Posture) -> LandSide {
        LandSide {
            men,
            fire: Fixed::from_int(100) * (men / 1000) as i64,
            protection: Fixed::ZERO,
            training: Fixed::ONE,
            supply: Fixed::ONE,
            air: pm(500),
            posture,
            mobility: Mobility::Foot,
            fortification: Fixed::ZERO,
            landing: false,
        }
    }

    fn open() -> LandField {
        LandField {
            terrain: TerrainMix::only(Terrain::Open),
            winter: false,
        }
    }

    #[test]
    fn land_terms_multiply_to_the_power_and_show_the_ratio() {
        let r = rules();
        let a = side(20_000, Posture::Attack);
        let mut d = side(10_000, Posture::Defend);
        d.supply = Fixed::ZERO;
        let o = land_step(
            &r,
            &LandField {
                terrain: TerrainMix::only(Terrain::Forest),
                winter: false,
            },
            [&a, &d],
        );
        assert_eq!(o.terms[0].number, Fixed::from_int(20));
        assert_eq!(o.terms[0].equipment, Fixed::ONE);
        assert_eq!(o.terms[1].supply, pm(250));
        assert_eq!(o.terms[1].terrain, pm(1250));
        assert_eq!(o.power[0], o.terms[0].power());
        let ratios = o.terms[0].over(&o.terms[1]);
        assert_eq!(ratios[0], ("number", Some(Fixed::from_int(2))));
        assert_eq!(ratios[3], ("supply", Some(Fixed::from_int(4))));
        assert!(o.advance_km.is_positive());
    }

    #[test]
    fn nothing_happens_when_both_defend() {
        let r = rules();
        let o = land_step(
            &r,
            &open(),
            [&side(5000, Posture::Defend), &side(5000, Posture::Defend)],
        );
        assert!(!o.engaged);
        assert_eq!(o.losses, [0, 0]);
        assert_eq!(o.advance_km, Fixed::ZERO);
    }

    #[test]
    fn more_supply_never_hurts() {
        let r = rules();
        let d = side(10_000, Posture::Defend);
        let mut last: Option<LandOutcome> = None;
        for s in [0, 100, 250, 500, 750, 1000] {
            let mut a = side(12_000, Posture::Attack);
            a.supply = pm(s);
            let o = land_step(&r, &open(), [&a, &d]);
            if let Some(prev) = &last {
                assert!(o.power[0] >= prev.power[0]);
                assert!(o.losses[1] >= prev.losses[1]);
                assert!(o.losses[0] <= prev.losses[0]);
                assert!(o.advance_km >= prev.advance_km);
            }
            last = Some(o);
        }
    }

    #[test]
    fn a_side_without_ammunition_loses() {
        let r = rules();
        let a = side(10_000, Posture::Attack);
        let mut b = side(10_000, Posture::Attack);
        b.supply = Fixed::ZERO;
        let o = land_step(&r, &open(), [&a, &b]);
        assert!(o.losses[1] > o.losses[0]);
        assert!(o.advance_km.is_positive());
    }

    #[test]
    fn swapping_sides_swaps_the_result() {
        let r = rules();
        let field = LandField {
            terrain: TerrainMix(vec![(Terrain::Forest, pm(400)), (Terrain::Open, pm(600))]),
            winter: true,
        };
        let mut a = side(15_000, Posture::Attack);
        a.mobility = Mobility::Tracked;
        a.protection = pm(300);
        let mut d = side(9_000, Posture::Defend);
        d.fortification = pm(500);
        d.training = pm(400);
        let o = land_step(&r, &field, [&a, &d]);
        let s = land_step(&r, &field, [&d, &a]);
        assert_eq!(o.terms, [s.terms[1], s.terms[0]]);
        assert_eq!(o.losses, [s.losses[1], s.losses[0]]);
        assert_eq!(o.advance_km, -s.advance_km);
    }

    #[test]
    fn losses_never_exceed_the_men() {
        let r = rules();
        let a = side(1_000_000, Posture::Attack);
        let d = side(1_000, Posture::Defend);
        let o = land_step(&r, &open(), [&a, &d]);
        assert!(o.losses[1] >= 39 && o.losses[1] <= 1_000);
        let empty = side(0, Posture::Defend);
        let o = land_step(&r, &open(), [&a, &empty]);
        assert_eq!(o.losses[1], 0);
        assert_eq!(o.terms[1].equipment, Fixed::ZERO);
    }

    #[test]
    fn mobilization_cuts_bands_and_applies_the_law() {
        let r = rules();
        let bands = [
            AgeBand {
                from: 15,
                to: 20,
                men: 1000,
            },
            AgeBand {
                from: 20,
                to: 40,
                men: 4000,
            },
            AgeBand {
                from: 40,
                to: 50,
                men: 1000,
            },
        ];
        // 15–20: ages 18 and 19 → 400; 20–40: all; 40–50: 40 to 44 → 500.
        let m = mobilizable(&r, &bands, 300, 100);
        assert_eq!(m.of_age, 4900);
        assert_eq!(m.callable, 1470);
        assert_eq!(m.reserved, 294);
        assert_eq!(m.available, 1470 - 294 - 100);
        assert_eq!(mobilizable(&r, &bands, 2000, 0).callable, 4900);
        assert_eq!(mobilizable(&r, &bands, 20, 1_000_000).available, 0);
    }

    #[test]
    fn supply_names_the_bottleneck_and_never_falls_with_capacity() {
        let r = rules();
        let leg = |mode, km: i64, cap: i64| Leg {
            mode,
            km: Fixed::from_int(km),
            capacity_tkm: Fixed::from_int(cap),
            interdiction: Fixed::ZERO,
        };
        let s = supply(
            &r,
            Fixed::from_int(1000),
            &[
                leg(Mode::Rail, 500, 1_000_000),
                leg(Mode::Road, 100, 50_000),
            ],
        );
        assert_eq!(
            s.throughput,
            vec![Fixed::from_int(2000), Fixed::from_int(500)]
        );
        assert_eq!(s.bottleneck, Some(1));
        assert!(s.fill < Fixed::ONE);
        assert_eq!(s.delivered, Fixed::from_int(500) * s.survival);
        let mut last = Fixed::ZERO;
        for cap in [0, 10_000, 50_000, 100_000, 200_000, 1_000_000] {
            let s = supply(
                &r,
                Fixed::from_int(1000),
                &[leg(Mode::Rail, 500, 1_000_000), leg(Mode::Road, 100, cap)],
            );
            assert!(s.delivered >= last);
            assert!(s.delivered <= s.need);
            last = s.delivered;
        }
        let local = supply(&r, Fixed::from_int(10), &[]);
        assert_eq!(local.fill, Fixed::ONE);
        let cut = supply(
            &r,
            Fixed::from_int(10),
            &[Leg {
                interdiction: Fixed::ONE,
                ..leg(Mode::Sea, 10, 1000)
            }],
        );
        assert_eq!(cut.delivered, Fixed::ZERO);
    }

    #[test]
    fn allocation_follows_length_threat_and_objective_and_is_stable() {
        let r = rules();
        let seg = |km: i64, enemy: i64, objective| Segment {
            length_km: Fixed::from_int(km),
            enemy_power: Fixed::from_int(enemy),
            objective,
        };
        let segments = [
            seg(100, 10, false),
            seg(100, 30, false),
            seg(100, 10, true),
            seg(300, 10, false),
        ];
        let a = allocate(&r, 200, &segments);
        assert_eq!(a.iter().map(|s| s.elements).sum::<u64>(), 200);
        assert!(a.iter().all(|s| s.elements >= s.floor && s.floor == 10));
        assert!(a[1].elements > a[0].elements);
        assert!(a[2].elements > a[0].elements);
        assert!(a[3].elements > a[0].elements);
        // Same front, same command, same numbers: the same split next week.
        for _ in 0..10 {
            assert_eq!(allocate(&r, 200, &segments), a);
        }
        // Fewer elements than segments: no segment gets more than one more than another.
        let few = allocate(&r, 3, &segments);
        assert_eq!(few.iter().map(|s| s.elements).sum::<u64>(), 3);
        assert!(allocate(&r, 10, &[]).is_empty());
    }

    #[test]
    fn apportion_is_exact_and_breaks_ties_by_order() {
        assert_eq!(
            apportion(10, &[Fixed::ONE, Fixed::ONE, Fixed::ONE]),
            vec![4, 3, 3]
        );
        assert_eq!(apportion(5, &[Fixed::ZERO, Fixed::ZERO]), vec![3, 2]);
        assert_eq!(
            apportion(7, &[Fixed::from_int(3), Fixed::ONE])
                .iter()
                .sum::<u64>(),
            7
        );
    }

    fn ships(
        count: u64,
        tonnage: i64,
        gun: i64,
        torpedo: i64,
        asw: i64,
        submerged: bool,
    ) -> ShipGroup {
        ShipGroup {
            count,
            tonnage: Fixed::from_int(tonnage),
            gun: Fixed::from_int(gun),
            torpedo: Fixed::from_int(torpedo),
            asw: Fixed::from_int(asw),
            anti_air: Fixed::ZERO,
            air: Fixed::ZERO,
            aircraft: 0,
            submerged,
        }
    }

    #[test]
    fn naval_step_sinks_by_tonnage_and_only_asw_reaches_submarines() {
        let r = rules();
        let a = NavalSide {
            groups: vec![ships(4, 30_000, 20_000, 0, 0, false)],
            training: Fixed::ONE,
            supply: Fixed::ONE,
        };
        let b = NavalSide {
            groups: vec![
                ships(2, 30_000, 10_000, 0, 0, false),
                ships(10, 1_000, 0, 1_000, 0, true),
            ],
            training: Fixed::ONE,
            supply: Fixed::ONE,
        };
        let o = naval_step(&r, [&a, &b]);
        // a strikes 80,000 against b's 20,000 + 10,000: share 8/11, so b's 60,000 surface
        // tonnes lose 0.05 × 2 × 8/11, about 4,360 t, no ship sunk; a has no anti-submarine
        // force, so the submarines are untouched.
        assert_eq!(o.sunk[1], vec![0, 0]);
        assert!(
            o.damage_t[1][0] > Fixed::from_int(4300) && o.damage_t[1][0] < Fixed::from_int(4400)
        );
        assert_eq!(o.damage_t[1][1], Fixed::ZERO);
        let s = naval_step(&r, [&b, &a]);
        assert_eq!(o.sunk, [s.sunk[1].clone(), s.sunk[0].clone()]);
        assert_eq!(o.terms, [s.terms[1], s.terms[0]]);
        // Overwhelming fire sinks no more ships than there are.
        let big = NavalSide {
            groups: vec![ships(100, 30_000, 1_000_000, 0, 1_000_000, false)],
            ..a.clone()
        };
        let o = naval_step(&r, [&big, &b]);
        assert!(o.sunk[1].iter().zip(&b.groups).all(|(s, g)| *s <= g.count));
        assert!(o.sunk[1][1] > 0);
    }

    #[test]
    fn air_step_gives_superiority_and_bounded_losses() {
        let r = rules();
        let planes = |count, a2a: i64, ground: i64| AirGroup {
            count,
            air_to_air: Fixed::from_int(a2a),
            ground: Fixed::from_int(ground),
            strategic: Fixed::ZERO,
            anti_ship: Fixed::ZERO,
        };
        let a = AirSide {
            groups: vec![planes(300, 30, 5)],
            training: Fixed::ONE,
            supply: Fixed::ONE,
        };
        let b = AirSide {
            groups: vec![planes(100, 30, 5), planes(50, 0, 40)],
            training: Fixed::ONE,
            supply: Fixed::ONE,
        };
        let o = air_step(&r, [&a, &b]);
        assert_eq!(o.superiority[0], pm(750));
        // 150 aircraft × 0.1 × 2 × 0.75 = 22.5.
        assert_eq!(o.losses[1].iter().sum::<u64>(), 22);
        assert!(
            o.losses[1]
                .iter()
                .zip(&b.groups)
                .all(|(l, g)| *l <= g.count)
        );
        let s = air_step(&r, [&b, &a]);
        assert_eq!(o.losses, [s.losses[1].clone(), s.losses[0].clone()]);
        assert_eq!(o.ground, [s.ground[1], s.ground[0]]);
        let none = AirSide {
            groups: vec![],
            training: Fixed::ONE,
            supply: Fixed::ONE,
        };
        assert_eq!(air_step(&r, [&none, &none]).superiority, [pm(500), pm(500)]);
        let swarm = AirSide {
            groups: vec![planes(100_000, 1000, 0)],
            ..a.clone()
        };
        assert_eq!(air_step(&r, [&swarm, &b]).losses[1].iter().sum::<u64>(), 30);
    }

    #[test]
    fn exhaustion_adds_named_terms_up_to_a_thousand() {
        let r = rules();
        let e = exhaustion(
            &r,
            &ExhaustionInputs {
                casualties: 1_000_000,
                under_arms: 4_000_000,
                men_of_age: 10_000_000,
                shortage: pm(200),
                occupied: pm(100),
                weeks: 104,
                industry_lost: Fixed::ZERO,
                spending_share: Fixed::ZERO,
                price_rise: Fixed::ZERO,
            },
        );
        assert_eq!(
            e,
            Exhaustion {
                casualties: 60,
                mobilization: 80,
                shortage: 60,
                occupation: 40,
                duration: 100,
                industry: 0,
                spending: 0,
                prices: 0,
                total: 340
            }
        );
        let worst = exhaustion(
            &r,
            &ExhaustionInputs {
                casualties: 10,
                under_arms: 10,
                men_of_age: 10,
                shortage: Fixed::ONE,
                occupied: Fixed::ONE,
                weeks: 520,
                industry_lost: Fixed::ONE,
                spending_share: Fixed::ONE,
                price_rise: Fixed::ONE,
            },
        );
        assert_eq!(worst.total, 1000);
    }

    #[test]
    fn terrain_of_state_splits_forest_and_climates() {
        let r = rules();
        let mix = terrain_of_state(
            &r,
            &[("tropical_rainforest".into(), 0.5), ("oceanic".into(), 0.5)],
            0.2,
        );
        assert_eq!(
            mix.0,
            vec![
                (Terrain::Open, pm(400)),
                (Terrain::Forest, pm(200)),
                (Terrain::Jungle, pm(400))
            ]
        );
        assert_eq!(mix.weighted(|t| r.defense_factor(t)), pm(1250));
    }
}
