//! The unit catalogue (`data/military/units.ron`) and the combat rules
//! (`data/military/rules.ron`), resolved at load: every good and technology key becomes an index,
//! every generation is spelled out in full, and every rule table is indexed by its enum. A wrong
//! key fails here, never silently in a step.

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use crate::econ::Catalogue;
use crate::econ::catalogue::GoodIx;
use crate::research::TechTree;

// ---------------------------------------------------------------------------------------------
// Enums shared by the files and the functions.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Domain {
    Land,
    Sea,
    Air,
    /// Out of the fronts: missiles, warheads, satellites, missile defence.
    Strategic,
}

impl Domain {
    pub const ALL: [Domain; 4] = [Domain::Land, Domain::Sea, Domain::Air, Domain::Strategic];

    /// What one element of the domain is: the unit the catalogue and the forces count in.
    pub fn element(self) -> &'static str {
        match self {
            Domain::Land => "1,000 men",
            Domain::Sea => "one ship",
            Domain::Air => "one aircraft",
            Domain::Strategic => "one weapon, launcher or satellite",
        }
    }
}

/// How a unit moves; sets its speed and how terrain treats it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Mobility {
    /// Does not leave its post (garrisons, fixed defences).
    Static,
    Foot,
    /// Foot troops trained and equipped for mountain, jungle and landing.
    Light,
    /// Horses: cavalry, horse-drawn guns.
    Horse,
    /// Lorries and other wheeled vehicles.
    Wheeled,
    /// Tracked armour.
    Tracked,
    Ship,
    Aircraft,
}

impl Mobility {
    /// The classes land terrain applies to.
    pub const LAND: [Mobility; 6] = [
        Mobility::Static,
        Mobility::Foot,
        Mobility::Light,
        Mobility::Horse,
        Mobility::Wheeled,
        Mobility::Tracked,
    ];

    fn land_index(self) -> Option<usize> {
        Self::LAND.iter().position(|m| *m == self)
    }

    fn allowed_in(self, domain: Domain) -> bool {
        match domain {
            Domain::Land => self.land_index().is_some(),
            Domain::Sea => self == Mobility::Ship,
            Domain::Air => self == Mobility::Aircraft,
            Domain::Strategic => {
                matches!(self, Mobility::Static | Mobility::Ship | Mobility::Aircraft)
            }
        }
    }
}

/// What a unit's equipment is for; shares per mille of its value (sea, air, strategic) or of the
/// element (land support roles).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Naval guns (and, later, ship-launched land-attack missiles).
    Gun,
    Torpedo,
    /// Anti-submarine warfare.
    Asw,
    AntiAir,
    /// Mine laying and sweeping: interdiction of a sea zone.
    Mine,
    AirToAir,
    GroundAttack,
    /// Strategic strike: industry, transport, cities.
    Strategic,
    AntiShip,
    Recon,
    /// Fortification, bridging and river crossing.
    Engineering,
}

impl Role {
    pub const COUNT: usize = 11;
    pub const ALL: [Role; Role::COUNT] = [
        Role::Gun,
        Role::Torpedo,
        Role::Asw,
        Role::AntiAir,
        Role::Mine,
        Role::AirToAir,
        Role::GroundAttack,
        Role::Strategic,
        Role::AntiShip,
        Role::Recon,
        Role::Engineering,
    ];

    fn index(self) -> usize {
        self as usize
    }

    fn allowed_in(self, domain: Domain) -> bool {
        use Role::*;
        match domain {
            Domain::Land => matches!(self, AntiAir | Engineering),
            Domain::Sea => matches!(self, Gun | Torpedo | Asw | AntiAir | Mine | AntiShip),
            Domain::Air => matches!(
                self,
                AirToAir | GroundAttack | Strategic | AntiShip | Recon | Asw
            ),
            Domain::Strategic => matches!(self, Strategic | Recon | AntiAir),
        }
    }
}

/// Land terrain classes. Open, forest, jungle, desert and arctic come from the climate and forest
/// of `land.ron`; mountain, marsh and urban wait for map data (`docs/war.pt-BR.md`, "Pedidos").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Open,
    Forest,
    Jungle,
    Desert,
    Arctic,
    Mountain,
    Marsh,
    Urban,
}

impl Terrain {
    pub const ALL: [Terrain; 8] = [
        Terrain::Open,
        Terrain::Forest,
        Terrain::Jungle,
        Terrain::Desert,
        Terrain::Arctic,
        Terrain::Mountain,
        Terrain::Marsh,
        Terrain::Urban,
    ];

    fn index(self) -> usize {
        self as usize
    }
}

/// How a leg of a supply route moves goods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Mode {
    Rail,
    Road,
    River,
    Sea,
    Air,
}

impl Mode {
    pub const ALL: [Mode; 5] = [Mode::Rail, Mode::Road, Mode::River, Mode::Sea, Mode::Air];

    fn index(self) -> usize {
        self as usize
    }
}

// ---------------------------------------------------------------------------------------------
// units.ron

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitsFile {
    pub types: Vec<UnitTypeDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitTypeDef {
    /// Stable key, `unit-<key>` in the Fluent files when it gets a name on screen.
    pub key: String,
    pub domain: Domain,
    pub mobility: Mobility,
    /// Technologies that unlock the first generation, all of them; empty when none is needed
    /// (militia, cavalry, the warships already afloat in 1925).
    #[serde(default)]
    pub requires: Vec<String>,
    /// Men per element: crew, gunners, pilots and ground staff included.
    pub men: u32,
    /// Displacement in tonnes, for ships: what it takes to sink one.
    #[serde(default)]
    pub tonnage: u32,
    /// Aircraft a ship carries.
    #[serde(default)]
    pub aircraft: u32,
    /// Tonnes of cargo or troops it carries.
    #[serde(default)]
    pub cargo_t: u32,
    /// Moves under water: only anti-submarine warfare reaches it.
    #[serde(default)]
    pub submerged: bool,
    /// Goods one element is equipped with, in each good's unit (lots of US$ 1,000 for the
    /// military goods).
    pub equipment: Vec<(String, Fixed)>,
    /// Goods one element consumes per week at peace, besides equipment maintenance.
    pub upkeep: Vec<(String, Fixed)>,
    pub training_weeks: u32,
    /// Shares per mille, at most 1,000 in all.
    #[serde(default)]
    pub roles: Vec<(Role, u32)>,
    /// A nuclear warhead's yield, kilotons (U3: each generation more power); 0 for the rest.
    #[serde(default)]
    pub yield_kt: u32,
    /// Later generations, in order; each one replaces only what it names.
    #[serde(default)]
    pub generations: Vec<GenerationDef>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GenerationDef {
    /// Technologies added to those of the previous generation; at least one.
    pub requires: Vec<String>,
    #[serde(default)]
    pub mobility: Option<Mobility>,
    #[serde(default)]
    pub men: Option<u32>,
    #[serde(default)]
    pub tonnage: Option<u32>,
    #[serde(default)]
    pub aircraft: Option<u32>,
    #[serde(default)]
    pub cargo_t: Option<u32>,
    #[serde(default)]
    pub equipment: Option<Vec<(String, Fixed)>>,
    #[serde(default)]
    pub upkeep: Option<Vec<(String, Fixed)>>,
    #[serde(default)]
    pub training_weeks: Option<u32>,
    #[serde(default)]
    pub roles: Option<Vec<(Role, u32)>>,
    #[serde(default)]
    pub yield_kt: Option<u32>,
}

// ---------------------------------------------------------------------------------------------
// rules.ron

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RulesFile {
    /// The goods that play a part by name: ammunition (multiplied in combat), fuel (multiplied
    /// on the move) and armour (protection).
    pub ammunition_good: String,
    pub fuel_good: String,
    pub armor_good: String,
    /// Land fire per unit of each good held (goods not listed add no fire).
    pub fire_per_unit: Vec<(String, Fixed)>,
    /// Fire of one element of the reference unit; the equipment term is a land side's fire per
    /// element over this.
    pub reference_fire: Fixed,
    /// Tonnes per unit of each good a force consumes, for supply.
    pub tonnes_per_unit: Vec<(String, Fixed)>,
    /// Share of the equipment value replaced each week, per mille.
    pub maintenance_per_mille: u32,
    /// Ammunition use in combat, as a multiple of the peace upkeep.
    pub combat_ammunition: Fixed,
    /// Fuel use on the move and in combat, as a multiple of the peace upkeep.
    pub moving_fuel: Fixed,
    /// Protection per share of the equipment value in armour, per mille, and its cap.
    pub protection_per_armor_share: u32,
    pub protection_max: u32,
    /// Factor of a force with no training, per mille; full training is 1,000.
    pub training_floor: u32,
    /// Factor of a force with no ammunition, per mille.
    pub supply_floor: u32,
    /// Most a side loses for having no air superiority at all, per mille.
    pub air_weight: u32,
    /// Bonus of full fortification to the defender, per mille.
    pub fortification_bonus: u32,
    /// Share of its men a side loses in a week of combat at even power; up to twice that when
    /// the enemy holds all the power.
    pub land_lethality: Fixed,
    /// Share of its tonnage a fleet loses in a week of action at even strike, as on land.
    pub naval_lethality: Fixed,
    /// Share of its aircraft a side loses in a week of contest at even superiority, as on land.
    pub air_lethality: Fixed,
    /// Ages of military service, from (inclusive) and to (exclusive).
    pub service_age: (u32, u32),
    /// Share of the men called up who stay in essential work, per mille.
    pub reserved_share: u32,
    /// Loss per 1,000 km of each mode, per mille: fuel the lorries burn, spoilage, pilferage.
    pub mode_loss: Vec<(Mode, u32)>,
    /// Tonne-kilometres one lot of the `transportation` good moves.
    pub tkm_per_transport_lot: Fixed,
    pub terrain: Vec<TerrainRule>,
    /// Köppen climate class of `land.ron` → terrain class.
    pub climate_terrain: Vec<(String, Terrain)>,
    /// Kilometres a week an attacker of each land mobility can advance on open ground.
    pub speed_km_week: Vec<(Mobility, Fixed)>,
    /// Factor a landing attacker takes on top of the terrain, per mille.
    pub amphibious: u32,
    /// Bonus to the defender's terrain when the whole border of the segment runs along a big
    /// river, per mille.
    pub river_defense: u32,
    /// People per km² of a town's built-up area, to turn the urban population of a state into
    /// the urban share of its terrain.
    pub urban_density_km2: Fixed,
    pub winter: WinterRule,
    pub allocation: AllocationRule,
    pub exhaustion: ExhaustionWeights,
    /// Share of the wounded who cannot go back to work, per mille.
    pub invalid_share: u32,
    pub doctrine: DoctrineRule,
    #[serde(default)]
    pub command: CommandRule,
    pub nuclear: NuclearRule,
    pub reserve: ReserveRule,
    pub front: FrontRule,
    /// The catalogue counts these goods in lots of 1925 dollars; the economy prices them in
    /// 2011 dollars. Their quantities in `units.ron` are multiplied by `factor` on load (and
    /// their weight per lot and the reference fire follow), so a unit costs what it did.
    #[serde(default)]
    pub dollars_1925: Option<Dollars1925>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dollars1925 {
    pub factor: Fixed,
    pub goods: Vec<String>,
}

/// The pace of a front (docs/war.pt-BR.md, "Ritmo da frente").
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FrontRule {
    /// Men per km of front at which a defence is a full line: the attacker moves at its base
    /// speed. About a division per 25 km is the least a coherent defence holds (Mearsheimer).
    pub full_line_men_km: Fixed,
    /// Bounds of the density's factor on the speed: a dense defence slows it to `density_min`,
    /// a thin one lets it run to `density_max`.
    pub density_min: Fixed,
    pub density_max: Fixed,
    /// Power of the ratio of the full line to the defender's density: 2, so that a line five
    /// times as dense slows the attacker 25 times (the Somme against the war of movement).
    pub density_power: u32,
    /// Weeks a segment holds still to be fully entrenched (1914: the race to the sea, then the
    /// trenches); a week in which the front moves less than `still_km` counts as still.
    pub dig_weeks: u16,
    pub still_km: Fixed,
    /// Pursuit: a side that advanced on a segment last week and attacks it this week with this
    /// many times the defender's power marches on at its marching pace, the defence broken
    /// (`calc::pursuit_km`).
    pub pursuit_ratio: Fixed,
}

/// Calling up the reserve.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ReserveRule {
    /// Weeks from the order until the whole reserve has reported, in equal shares.
    pub call_up_weeks: u32,
    /// Training of a reservist on arrival, per mille of full training.
    pub training: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainRule {
    pub terrain: Terrain,
    /// Defender's factor.
    pub defense: Fixed,
    /// Attacker's factor by mobility; classes not listed get 1.
    #[serde(default)]
    pub attack: Vec<(Mobility, Fixed)>,
    /// Advance speed factor by mobility; classes not listed get 1.
    #[serde(default)]
    pub speed: Vec<(Mobility, Fixed)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinterRule {
    /// Upkeep multiple in winter.
    pub upkeep: Fixed,
    /// Speed multiple in winter.
    pub speed: Fixed,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AllocationRule {
    /// Share of the front's elements each segment gets first, per mille.
    pub min_share: u32,
    /// Extra weight of a segment that leads to the objective, per mille.
    pub objective_pull: u32,
}

/// Points of exhaustion (per mille) at a full share of each term.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ExhaustionWeights {
    /// Casualties over the men of service age.
    pub casualties: u32,
    /// Men under arms over the men of service age.
    pub mobilization: u32,
    /// Share of consumer needs unmet.
    pub shortage: u32,
    /// Share of the population under occupation.
    pub occupation: u32,
    /// Per year of war.
    pub duration_per_year: u32,
    /// Share of industry destroyed.
    pub industry: u32,
    /// War spending since the war began over a year of output: what the war cost in money,
    /// whoever is winning.
    pub spending: u32,
    /// Rise of the price index since the war began (full at a doubling).
    pub prices: u32,
}

/// Generations of war (`docs/war.pt-BR.md`, "Doutrina"): readiness weights and thresholds, and
/// how a third-generation front concentrates and breaks through. Per mille unless said.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DoctrineRule {
    /// Points of readiness at a full share of each cause.
    pub officer_education: u32,
    pub training: u32,
    pub radio: u32,
    pub motorization: u32,
    pub experience: u32,
    /// Points taken away at full central control of the officer corps.
    pub central_control: u32,
    /// An army moves to the third generation at `up` and falls back below `down`.
    pub up: u32,
    pub down: u32,
    /// Below these fills of radio and fuel a third-generation force fights as a second.
    pub radio_min: u32,
    pub fuel_min: u32,
    /// Share of the front's free elements a third-generation front puts on its focus segment.
    pub focus_share: u32,
    /// Power ratio (attacker over defender, defender's reserve included) that breaks a segment.
    pub breakthrough_ratio: Fixed,
    /// Speed of the exploitation after a breakthrough, over the base speed.
    pub exploitation_speed: Fixed,
    /// Weeks of war that give an army the full experience term.
    pub experience_full_weeks: u32,
    /// Half-life of that experience in peace, years.
    pub experience_half_life_years: u32,
    /// Years after a change of the form of government in which the officer corps is under
    /// political control: full at the change, gone at the end.
    pub politicized_years: u32,
}

/// The command culture of the armies (decision G7, draft): the share of the officer corps
/// formed to act on the commander's intent, in 1925, and how it moves, a cohort at a time.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CommandRule {
    /// (country tag, per mille) in 1925, for the armies with a recorded tradition. The others
    /// read the schooling of their people instead.
    pub tradition_1925: Vec<(String, u32)>,
    /// Per mille of the schooling of the people that an army without a recorded tradition
    /// counts as officers formed to act on intent: a schooled corps, without a staff school's
    /// doctrine.
    pub untraditioned_share: u32,
    /// Years an officer serves: a cohort of 1 / this of the corps comes out of the schools each
    /// year, formed in what they teach.
    pub officer_career_years: u32,
    /// The research that opens the doctrine of mobile, decentralised command: once the country
    /// makes it (Industrial stage), the schools teach it.
    pub doctrine_tech: String,
    /// After a lost war the schools revise the doctrine: the cohorts come this many times as
    /// fast for `reform_years`.
    pub defeat_reform_speed: u32,
    pub reform_years: u32,
}

/// Nuclear deterrence (`docs/war.pt-BR.md`, "Arma nuclear"). Per mille unless said. Design
/// parameters, to be calibrated against the sources the doc cites.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NuclearRule {
    /// Share of the warheads aimed at a hardened silo that destroy it.
    pub silo_kill: u32,
    /// Share of the warheads aimed at a soft target (bomber base, submarine in port, mobile
    /// launcher in garrison) that destroy it.
    pub soft_kill: u32,
    /// Share of bombers that get through air defences, before missile defence.
    pub bomber_penetration: u32,
    /// Escalation risk per week of direct war between two powers that both hold a secure second
    /// strike: not a draw, a cost the decision weighs (the threat that leaves something to
    /// chance).
    pub escalation_risk_per_week: u32,
    /// The balance read from the forces (`war::balance`). N3 (to the owner): what an untested
    /// arsenal counts for the others, per mille.
    #[serde(default = "nuclear_defaults::opaque_weight")]
    pub opaque_weight: u32,
    /// N9 (to the owner): the protectors bound by a defence clause add their forces to the
    /// protected side (the umbrella).
    #[serde(default = "nuclear_defaults::umbrella")]
    pub umbrella: bool,
    /// Unacceptable damage: this share of the people, per mille, at `killed_per_warhead` each.
    #[serde(default = "nuclear_defaults::unacceptable_share")]
    pub unacceptable_share: u32,
    #[serde(default = "nuclear_defaults::killed_per_warhead")]
    pub killed_per_warhead: u32,
    /// Warheads each launcher carries.
    #[serde(default = "nuclear_defaults::per_bomber")]
    pub warheads_per_bomber: u32,
    #[serde(default = "nuclear_defaults::per_missile")]
    pub warheads_per_missile: u32,
    #[serde(default = "nuclear_defaults::per_submarine")]
    pub warheads_per_submarine: u32,
    /// Shares on alert or at sea, per mille, until the doctrine sets them.
    #[serde(default)]
    pub bomber_alert: u32,
    #[serde(default = "nuclear_defaults::submarine_alert")]
    pub submarine_alert: u32,
    /// Share of command that survives a first strike, per mille.
    #[serde(default = "nuclear_defaults::command")]
    pub command: u32,
    /// Interceptors of a missile defence site, and the share of warheads each stops, per mille.
    #[serde(default = "nuclear_defaults::interceptors")]
    pub interceptors_per_site: u32,
    #[serde(default = "nuclear_defaults::interceptor_kill")]
    pub interceptor_kill: u32,
    /// The use (step 4; N5, N6, N7). Civilians a counterforce warhead kills, per mille of
    /// `killed_per_warhead`.
    #[serde(default = "nuclear_defaults::counterforce_civilians")]
    pub counterforce_civilians: u32,
    /// Men of the target's land forces a counterforce warhead puts out of the war.
    #[serde(default = "nuclear_defaults::counterforce_men")]
    pub counterforce_men: u32,
    /// Share of a region's plant levels a countervalue warhead destroys, per mille of the share
    /// of its people it kills.
    #[serde(default = "nuclear_defaults::wreck_factor")]
    pub wreck_factor: u32,
    /// The stain of a use on the user's security reputation, per mille of a year of its forces'
    /// cost (the value of a defence call not answered is one year).
    #[serde(default = "nuclear_defaults::use_stain")]
    pub use_stain: u32,
    /// N7: years without use, with warheads in the world, for the taboo to grow from nothing to
    /// whole; and the share of it each use takes away, per mille.
    #[serde(default = "nuclear_defaults::taboo_years")]
    pub taboo_years: u32,
    #[serde(default = "nuclear_defaults::taboo_cut")]
    pub taboo_cut: u32,
    /// N7: the AI uses first with this share of its home people under the enemy, per mille,
    /// when the taboo is nothing; the line rises to all of it as the taboo grows whole.
    #[serde(default = "nuclear_defaults::first_use_occupied")]
    pub first_use_occupied: u32,
    /// N8, the fallout: the share of a region's productivity a warhead that arrives takes, per
    /// mille; the share of that its neighbouring states take; the most a region loses; and the
    /// share of the loss left after a year (the half-life of caesium-137).
    #[serde(default = "nuclear_defaults::fallout_loss")]
    pub fallout_loss: u32,
    #[serde(default = "nuclear_defaults::fallout_downwind")]
    pub fallout_downwind: u32,
    #[serde(default = "nuclear_defaults::fallout_cap")]
    pub fallout_cap: u32,
    #[serde(default = "nuclear_defaults::fallout_keep")]
    pub fallout_keep: u32,
}

mod nuclear_defaults {
    pub fn opaque_weight() -> u32 {
        500
    }
    pub fn umbrella() -> bool {
        true
    }
    pub fn unacceptable_share() -> u32 {
        250
    }
    pub fn killed_per_warhead() -> u32 {
        185_000
    }
    pub fn per_bomber() -> u32 {
        2
    }
    pub fn per_missile() -> u32 {
        1
    }
    pub fn per_submarine() -> u32 {
        16
    }
    pub fn submarine_alert() -> u32 {
        500
    }
    pub fn command() -> u32 {
        900
    }
    pub fn interceptors() -> u32 {
        100
    }
    pub fn interceptor_kill() -> u32 {
        500
    }
    pub fn counterforce_civilians() -> u32 {
        100
    }
    pub fn counterforce_men() -> u32 {
        20_000
    }
    pub fn wreck_factor() -> u32 {
        2000
    }
    pub fn use_stain() -> u32 {
        2000
    }
    pub fn taboo_years() -> u32 {
        20
    }
    pub fn taboo_cut() -> u32 {
        500
    }
    pub fn first_use_occupied() -> u32 {
        300
    }
    pub fn fallout_loss() -> u32 {
        100
    }
    pub fn fallout_downwind() -> u32 {
        300
    }
    pub fn fallout_cap() -> u32 {
        900
    }
    pub fn fallout_keep() -> u32 {
        977
    }
}

// ---------------------------------------------------------------------------------------------
// Resolved catalogue.

pub type TypeIx = u16;

/// One generation of a unit type, spelled out in full.
#[derive(Clone, Debug)]
pub struct Spec {
    /// Every technology this generation needs (those of earlier generations included).
    pub requires: Vec<u16>,
    pub mobility: Mobility,
    pub men: u32,
    pub tonnage: u32,
    pub aircraft: u32,
    pub cargo_t: u32,
    pub equipment: Vec<(GoodIx, Fixed)>,
    pub upkeep: Vec<(GoodIx, Fixed)>,
    pub training_weeks: u32,
    /// Per mille, indexed by [`Role`].
    pub roles: [u32; Role::COUNT],
    /// A nuclear warhead's yield, kilotons.
    pub yield_kt: u32,
}

impl Spec {
    /// U3: the warhead's equivalent megatons, `Y^(2/3)` with Y in megatons: the area a given
    /// overpressure covers grows with the yield to the 2/3 (Glasstone and Dolan, 1977, §3.61–3.62:
    /// the distance as `W^(1/3)`; the area is that squared), the "equivalent megatonnage" of the
    /// Office of Technology Assessment (1979, p. 45). Per mille.
    pub fn emt_mille(&self) -> i64 {
        // (Y / 1000)^(2/3) × 1000 = cbrt(Y² × 1000), Y in kilotons; integer, for determinism.
        let x = i128::from(self.yield_kt) * i128::from(self.yield_kt) * 1000;
        let (mut lo, mut hi) = (0i128, 1i128 << 21);
        while lo < hi {
            let mid = (lo + hi + 1) / 2;
            if mid * mid * mid <= x {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        lo as i64
    }
}

impl Spec {
    pub fn role(&self, role: Role) -> u32 {
        self.roles[role.index()]
    }
}

#[derive(Clone, Debug)]
pub struct UnitType {
    pub key: String,
    pub domain: Domain,
    pub submerged: bool,
    /// Generation 0 is the base type.
    pub generations: Vec<Spec>,
}

#[derive(Clone, Debug)]
pub struct Rules {
    pub ammunition: GoodIx,
    pub fuel: GoodIx,
    pub armor: GoodIx,
    /// Per good index.
    pub fire_per_unit: Vec<Fixed>,
    pub reference_fire: Fixed,
    /// Per good index; `None` for goods no force consumes.
    pub tonnes_per_unit: Vec<Option<Fixed>>,
    /// Value of one unit of each good in lots of US$ 1,000 (base price ÷ 1,000).
    pub lots_per_unit: Vec<Fixed>,
    /// Per good index: units of today's dollars in one of the catalogue's 1925 lots
    /// (`dollars_1925`), one for the rest. Weight and fire are per 1925 lot.
    pub lot_factor: Vec<Fixed>,
    pub maintenance: Fixed,
    pub combat_ammunition: Fixed,
    pub moving_fuel: Fixed,
    pub protection_per_armor_share: Fixed,
    pub protection_max: Fixed,
    pub training_floor: Fixed,
    pub supply_floor: Fixed,
    pub air_weight: Fixed,
    pub fortification_bonus: Fixed,
    pub land_lethality: Fixed,
    pub naval_lethality: Fixed,
    pub air_lethality: Fixed,
    pub service_age: (u32, u32),
    pub reserved_share: Fixed,
    /// Per [`Mode`], share lost per 1,000 km.
    pub mode_loss: [Fixed; 5],
    pub tkm_per_transport_lot: Fixed,
    /// Per [`Terrain`]: defence, and attack and speed per land [`Mobility`].
    pub defense: [Fixed; 8],
    pub attack: [[Fixed; 6]; 8],
    pub speed: [[Fixed; 6]; 8],
    pub climate_terrain: Vec<(String, Terrain)>,
    /// Per land [`Mobility`].
    pub speed_km_week: [Fixed; 6],
    pub amphibious: Fixed,
    pub river_defense: Fixed,
    pub urban_density_km2: Fixed,
    pub winter_upkeep: Fixed,
    pub winter_speed: Fixed,
    pub min_share: Fixed,
    pub objective_pull: Fixed,
    pub exhaustion: ExhaustionWeights,
    pub invalid_share: Fixed,
    pub doctrine: DoctrineRule,
    pub command: CommandRule,
    pub nuclear: NuclearRule,
    pub reserve: ReserveRule,
    pub front: FrontRule,
}

/// The catalogue and the rules, resolved.
#[derive(Clone, Debug)]
pub struct Military {
    pub types: Vec<UnitType>,
    pub rules: Rules,
    /// Good keys, by index, for messages and tests.
    pub good_keys: Vec<String>,
}

pub(crate) fn per_mille(n: u32) -> Fixed {
    Fixed::from_ratio(n as i64, 1000)
}

impl Military {
    pub fn unit(&self, key: &str) -> Option<TypeIx> {
        self.types
            .iter()
            .position(|t| t.key == key)
            .map(|i| i as TypeIx)
    }

    pub fn spec(&self, ty: TypeIx, generation: usize) -> &Spec {
        &self.types[ty as usize].generations[generation]
    }

    /// Number of types per domain.
    pub fn count(&self, domain: Domain) -> usize {
        self.types.iter().filter(|t| t.domain == domain).count()
    }

    /// Resolves and checks the two files. Every problem is reported, not just the first.
    pub fn build(
        units: &UnitsFile,
        rules_file: &RulesFile,
        cat: &Catalogue,
        tree: &TechTree,
    ) -> Result<Self, Vec<String>> {
        let mut errors = Vec::new();
        let rules = resolve_rules(rules_file, cat, &mut errors);
        let mut types: Vec<UnitType> = Vec::new();
        for def in &units.types {
            if types.iter().any(|t| t.key == def.key) {
                errors.push(format!("unit {}: duplicate key", def.key));
            }
            if let Some(t) = resolve_type(def, cat, tree, &rules, &mut errors) {
                types.push(t);
            }
        }
        let mut rules = rules;
        if let (Some(d), Some(r)) = (&rules_file.dollars_1925, rules.as_mut()) {
            let mut scaled = vec![false; cat.goods.len()];
            for key in &d.goods {
                match cat.good(key) {
                    Some(g) => scaled[g as usize] = true,
                    None => errors.push(format!("rules: dollars_1925: unknown good {key}")),
                }
            }
            for t in &mut types {
                for s in &mut t.generations {
                    for (g, q) in s.equipment.iter_mut().chain(s.upkeep.iter_mut()) {
                        if scaled[*g as usize] {
                            *q *= d.factor;
                        }
                    }
                }
            }
            for (g, f) in r.lot_factor.iter_mut().enumerate() {
                if scaled[g] {
                    *f = d.factor;
                }
            }
        }
        if errors.is_empty() {
            Ok(Self {
                types,
                rules: rules.expect("rules resolve when there are no errors"),
                good_keys: cat.goods.iter().map(|g| g.key.clone()).collect(),
            })
        } else {
            Err(errors)
        }
    }
}

fn goods_list(
    what: &str,
    list: &[(String, Fixed)],
    cat: &Catalogue,
    errors: &mut Vec<String>,
) -> Vec<(GoodIx, Fixed)> {
    let mut out: Vec<(GoodIx, Fixed)> = Vec::new();
    for (key, qty) in list {
        match cat.good(key) {
            Some(g) if out.iter().any(|(o, _)| *o == g) => {
                errors.push(format!("{what}: good {key} listed twice"));
            }
            Some(g) if qty.is_positive() => out.push((g, *qty)),
            Some(_) => errors.push(format!("{what}: good {key} with quantity {qty}")),
            None => errors.push(format!("{what}: unknown good {key}")),
        }
    }
    out.sort_by_key(|(g, _)| *g);
    out
}

fn roles_array(
    what: &str,
    domain: Domain,
    list: &[(Role, u32)],
    errors: &mut Vec<String>,
) -> [u32; Role::COUNT] {
    let mut roles = [0u32; Role::COUNT];
    for (role, share) in list {
        if !role.allowed_in(domain) {
            errors.push(format!(
                "{what}: role {role:?} does not belong to {domain:?}"
            ));
        }
        if roles[role.index()] != 0 {
            errors.push(format!("{what}: role {role:?} listed twice"));
        }
        roles[role.index()] = *share;
    }
    let total: u32 = roles.iter().sum();
    if total > 1000 {
        errors.push(format!("{what}: roles add up to {total} per mille"));
    }
    roles
}

fn techs(what: &str, keys: &[String], tree: &TechTree, errors: &mut Vec<String>) -> Vec<u16> {
    keys.iter()
        .filter_map(|k| {
            let t = tree.tech(k).map(|t| t as u16);
            if t.is_none() {
                errors.push(format!("{what}: unknown technology {k}"));
            }
            t
        })
        .collect()
}

fn resolve_type(
    def: &UnitTypeDef,
    cat: &Catalogue,
    tree: &TechTree,
    rules: &Option<Rules>,
    errors: &mut Vec<String>,
) -> Option<UnitType> {
    let what = format!("unit {}", def.key);
    let domain = def.domain;
    let base = Spec {
        requires: techs(&what, &def.requires, tree, errors),
        mobility: def.mobility,
        men: def.men,
        tonnage: def.tonnage,
        aircraft: def.aircraft,
        cargo_t: def.cargo_t,
        equipment: goods_list(&format!("{what} equipment"), &def.equipment, cat, errors),
        upkeep: goods_list(&format!("{what} upkeep"), &def.upkeep, cat, errors),
        training_weeks: def.training_weeks,
        roles: roles_array(&what, domain, &def.roles, errors),
        yield_kt: def.yield_kt,
    };
    let mut generations = vec![base];
    for (n, g) in def.generations.iter().enumerate() {
        let what = format!("unit {} generation {}", def.key, n + 1);
        let prev = generations.last().expect("generation 0 exists").clone();
        if g.requires.is_empty() {
            errors.push(format!("{what}: no technology"));
        }
        let mut requires = prev.requires.clone();
        for t in techs(&what, &g.requires, tree, errors) {
            if requires.contains(&t) {
                errors.push(format!(
                    "{what}: technology {} already required",
                    tree.techs[t as usize].key
                ));
            }
            requires.push(t);
        }
        generations.push(Spec {
            requires,
            mobility: g.mobility.unwrap_or(prev.mobility),
            men: g.men.unwrap_or(prev.men),
            tonnage: g.tonnage.unwrap_or(prev.tonnage),
            aircraft: g.aircraft.unwrap_or(prev.aircraft),
            cargo_t: g.cargo_t.unwrap_or(prev.cargo_t),
            equipment: g.equipment.as_ref().map_or(prev.equipment, |e| {
                goods_list(&format!("{what} equipment"), e, cat, errors)
            }),
            upkeep: g.upkeep.as_ref().map_or(prev.upkeep, |u| {
                goods_list(&format!("{what} upkeep"), u, cat, errors)
            }),
            training_weeks: g.training_weeks.unwrap_or(prev.training_weeks),
            roles: g
                .roles
                .as_ref()
                .map_or(prev.roles, |r| roles_array(&what, domain, r, errors)),
            yield_kt: g.yield_kt.unwrap_or(prev.yield_kt),
        });
    }
    for (n, s) in generations.iter().enumerate() {
        let what = format!("unit {} generation {n}", def.key);
        if !s.mobility.allowed_in(domain) {
            errors.push(format!("{what}: mobility {:?} in {domain:?}", s.mobility));
        }
        if domain == Domain::Land && s.men != 1000 {
            errors.push(format!(
                "{what}: a land element is 1,000 men, not {}",
                s.men
            ));
        }
        if s.men == 0 {
            errors.push(format!("{what}: no men"));
        }
        if domain == Domain::Sea && s.tonnage == 0 {
            errors.push(format!("{what}: a ship needs a tonnage"));
        }
        if s.equipment.is_empty() {
            errors.push(format!("{what}: no equipment"));
        }
        if let Some(r) = rules {
            for (g, _) in &s.upkeep {
                if r.tonnes_per_unit[*g as usize].is_none() {
                    errors.push(format!(
                        "{what}: upkeep good {} has no weight in tonnes_per_unit",
                        cat.goods[*g as usize].key
                    ));
                }
            }
        }
    }
    Some(UnitType {
        key: def.key.clone(),
        domain,
        submerged: def.submerged,
        generations,
    })
}

fn resolve_rules(file: &RulesFile, cat: &Catalogue, errors: &mut Vec<String>) -> Option<Rules> {
    let n = cat.goods.len();
    let good = |key: &str, errors: &mut Vec<String>| {
        let g = cat.good(key);
        if g.is_none() {
            errors.push(format!("rules: unknown good {key}"));
        }
        g
    };
    let ammunition = good(&file.ammunition_good, errors);
    let fuel = good(&file.fuel_good, errors);
    let armor = good(&file.armor_good, errors);
    let mut fire_per_unit = vec![Fixed::ZERO; n];
    for (g, ix) in goods_list("rules fire_per_unit", &file.fire_per_unit, cat, errors) {
        fire_per_unit[g as usize] = ix;
    }
    let mut tonnes_per_unit = vec![None; n];
    for (g, t) in goods_list("rules tonnes_per_unit", &file.tonnes_per_unit, cat, errors) {
        tonnes_per_unit[g as usize] = Some(t);
    }
    let mut mode_loss = [None; 5];
    for (m, l) in &file.mode_loss {
        if mode_loss[m.index()].replace(per_mille(*l)).is_some() {
            errors.push(format!("rules: mode {m:?} listed twice"));
        }
    }
    if mode_loss.iter().any(Option::is_none) {
        errors.push("rules: mode_loss must list every mode".into());
    }
    let mut defense = [None; 8];
    let mut attack = [[Fixed::ONE; 6]; 8];
    let mut speed = [[Fixed::ONE; 6]; 8];
    for rule in &file.terrain {
        let t = rule.terrain.index();
        if defense[t].replace(rule.defense).is_some() {
            errors.push(format!("rules: terrain {:?} listed twice", rule.terrain));
        }
        for (table, list, what) in [
            (&mut attack, &rule.attack, "attack"),
            (&mut speed, &rule.speed, "speed"),
        ] {
            for (m, f) in list {
                match m.land_index() {
                    Some(i) => table[t][i] = *f,
                    None => errors.push(format!(
                        "rules: terrain {:?} {what} for non-land mobility {m:?}",
                        rule.terrain
                    )),
                }
            }
        }
    }
    if defense.iter().any(Option::is_none) {
        errors.push("rules: terrain must list every terrain class".into());
    }
    let mut speed_km_week = [None; 6];
    for (m, s) in &file.speed_km_week {
        match m.land_index() {
            Some(i) => speed_km_week[i] = Some(*s),
            None => errors.push(format!("rules: speed for non-land mobility {m:?}")),
        }
    }
    if speed_km_week.iter().any(Option::is_none) {
        errors.push("rules: speed_km_week must list every land mobility".into());
    }
    let (from, to) = file.service_age;
    if from >= to {
        errors.push(format!("rules: service_age {from}..{to} is empty"));
    }
    for (what, v) in [
        ("training_floor", file.training_floor),
        ("supply_floor", file.supply_floor),
        ("air_weight", file.air_weight),
        ("reserved_share", file.reserved_share),
        ("protection_max", file.protection_max),
        ("invalid_share", file.invalid_share),
        ("allocation.min_share", file.allocation.min_share),
        ("doctrine.up", file.doctrine.up),
        ("doctrine.focus_share", file.doctrine.focus_share),
        ("nuclear.silo_kill", file.nuclear.silo_kill),
        ("nuclear.soft_kill", file.nuclear.soft_kill),
        (
            "nuclear.bomber_penetration",
            file.nuclear.bomber_penetration,
        ),
        ("reserve.training", file.reserve.training),
    ] {
        if v > 1000 {
            errors.push(format!("rules: {what} {v} is more than 1,000 per mille"));
        }
    }
    if file.reserve.call_up_weeks == 0 {
        errors.push("rules: reserve.call_up_weeks must be at least 1".into());
    }
    if file.doctrine.down > file.doctrine.up {
        errors.push("rules: doctrine.down above doctrine.up".into());
    }
    if !file.urban_density_km2.is_positive() {
        errors.push("rules: urban_density_km2 must be positive".into());
    }
    if !file.reference_fire.is_positive() {
        errors.push("rules: reference_fire must be positive".into());
    }
    let (Some(ammunition), Some(fuel), Some(armor)) = (ammunition, fuel, armor) else {
        return None;
    };
    if mode_loss.iter().any(Option::is_none)
        || defense.iter().any(Option::is_none)
        || speed_km_week.iter().any(Option::is_none)
    {
        return None;
    }
    Some(Rules {
        ammunition,
        fuel,
        armor,
        fire_per_unit,
        reference_fire: file.reference_fire,
        tonnes_per_unit,
        lots_per_unit: cat.goods.iter().map(|g| g.base_price / 1000).collect(),
        lot_factor: vec![Fixed::ONE; cat.goods.len()],
        maintenance: per_mille(file.maintenance_per_mille),
        combat_ammunition: file.combat_ammunition,
        moving_fuel: file.moving_fuel,
        protection_per_armor_share: per_mille(file.protection_per_armor_share),
        protection_max: per_mille(file.protection_max),
        training_floor: per_mille(file.training_floor),
        supply_floor: per_mille(file.supply_floor),
        air_weight: per_mille(file.air_weight),
        fortification_bonus: per_mille(file.fortification_bonus),
        land_lethality: file.land_lethality,
        naval_lethality: file.naval_lethality,
        air_lethality: file.air_lethality,
        service_age: file.service_age,
        reserved_share: per_mille(file.reserved_share),
        mode_loss: mode_loss.map(|m| m.expect("checked")),
        tkm_per_transport_lot: file.tkm_per_transport_lot,
        defense: defense.map(|d| d.expect("checked")),
        attack,
        speed,
        climate_terrain: file.climate_terrain.clone(),
        speed_km_week: speed_km_week.map(|s| s.expect("checked")),
        amphibious: per_mille(file.amphibious),
        river_defense: per_mille(file.river_defense),
        urban_density_km2: file.urban_density_km2,
        winter_upkeep: file.winter.upkeep,
        winter_speed: file.winter.speed,
        min_share: per_mille(file.allocation.min_share),
        objective_pull: per_mille(file.allocation.objective_pull),
        exhaustion: file.exhaustion,
        invalid_share: per_mille(file.invalid_share),
        doctrine: file.doctrine,
        command: file.command.clone(),
        nuclear: file.nuclear,
        reserve: file.reserve,
        front: file.front,
    })
}

impl Rules {
    /// Attack factor of a land mobility on a terrain class.
    pub fn attack_factor(&self, terrain: Terrain, mobility: Mobility) -> Fixed {
        mobility
            .land_index()
            .map_or(Fixed::ONE, |m| self.attack[terrain.index()][m])
    }

    pub fn speed_factor(&self, terrain: Terrain, mobility: Mobility) -> Fixed {
        mobility
            .land_index()
            .map_or(Fixed::ONE, |m| self.speed[terrain.index()][m])
    }

    pub fn defense_factor(&self, terrain: Terrain) -> Fixed {
        self.defense[terrain.index()]
    }

    /// Kilometres a week on open ground.
    pub fn base_speed(&self, mobility: Mobility) -> Fixed {
        mobility
            .land_index()
            .map_or(Fixed::ZERO, |m| self.speed_km_week[m])
    }

    pub fn mode_loss(&self, mode: Mode) -> Fixed {
        self.mode_loss[mode.index()]
    }

    pub fn climate(&self, key: &str) -> Option<Terrain> {
        self.climate_terrain
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, t)| *t)
    }
}
