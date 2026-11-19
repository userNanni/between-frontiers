//! Armed forces in the world (phase 5, slice A): the formations of every country, saved and
//! hashed, in peace. They hold men taken from the pops (`econ::monthly::Labour` hires toward
//! [`WarState::soldiers`]) and equipment bought by the state (`econ::week` buys
//! [`CountryWar::orders`] before the rest of the state's basket). Reasoning in
//! `docs/war.pt-BR.md`, "Forças em paz".

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::schedule::ResourceId;
use serde::{Deserialize, Serialize};

use super::catalogue::{Domain, Military};
use super::forces::{ForcesFile, Station};
use crate::scenario::CountryDef;

/// The forces of every country (`GameWorld::war`).
pub const WAR: ResourceId = ResourceId(16);

/// How the forces sit in peace (`data/military/peace.ron`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeaceRules {
    pub soldiers: SoldierRule,
    pub peace_size: PeaceSize,
    /// Scale the size in peace by the `conscription` effect in force over the one of 1925.
    pub law_scaling: bool,
    /// Weeks to make good the equipment a formation lacks.
    pub replace_weeks: u32,
    /// Years in which the whole of a force moves to a newer generation of equipment when the
    /// country can make it and the budget has room: a share of the elements each week.
    pub modernize_years: u32,
    /// Multiplies what the forces consume in peace (training ammunition, fuel, spares, depots);
    /// 1 keeps the catalogue (`docs/war.pt-BR.md`, "Forças em paz").
    pub upkeep_scale: Fixed,
    /// (country, men under arms per mille of the men of service age), in place of 1925's.
    #[serde(default)]
    pub share_override: Vec<(String, u32)>,
    /// Weeks over which the men of a land order report, a class at a time (slice A2).
    #[serde(default = "default_raise_weeks")]
    pub raise_weeks: u32,
    /// Weeks from laying down to delivery of the ships, aircraft and strategic forces of an
    /// order, by unit type key; the types not listed take [`DEFAULT_BUILD_WEEKS`].
    #[serde(default)]
    pub build_weeks: Vec<(String, u32)>,
    /// The historical rule of the forces' composition; `None`: no conversion.
    #[serde(default)]
    pub composition: Option<super::composition::CompositionRule>,
    /// The nuclear programme (N1, N2); `None`: no programme can start.
    #[serde(default)]
    pub nuclear_programme: Option<super::programme::ProgrammeRule>,
}

fn default_raise_weeks() -> u32 {
    12
}

/// Build weeks of a type not listed in [`PeaceRules::build_weeks`].
pub const DEFAULT_BUILD_WEEKS: u32 = 52;

impl PeaceRules {
    /// Weeks to build one batch of type `key`, at least one.
    pub fn build_weeks(&self, key: &str) -> u32 {
        self.build_weeks
            .iter()
            .find(|(k, _)| k == key)
            .map_or(DEFAULT_BUILD_WEEKS, |(_, w)| *w)
            .max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoldierRule {
    /// The pops hold the men of the formations.
    Forces,
    /// The rule from before the forces: this many per mille of the population are soldiers, and
    /// the state buys arms by its fixed basket. The forces are not loaded.
    PopulationShare(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeaceSize {
    /// Each country keeps the share of its men of service age it had under arms in 1925.
    ShareOfServiceAge,
    /// The numbers of 1925, unchanged.
    Fixed1925,
}

/// One body of the same unit type and generation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Formation {
    /// Index in [`Military::types`].
    pub unit: u16,
    pub generation: u8,
    /// Elements: thousands of men on land, ships at sea, aircraft in the air.
    pub elements: Fixed,
    /// Stationed in the colonies.
    pub colonial: bool,
    /// Reservists called up; they go back to the reserve on demobilization.
    pub called_up: bool,
    /// Equipment held, by good.
    pub equipment: Vec<(u16, Fixed)>,
    /// Training completed, 0 to 1.
    pub training: Fixed,
    /// Raised by an order (slice A2): outside the size in peace, which the January rule keeps
    /// to the share of the men of service age.
    pub raised: bool,
}

/// Forces a country ordered (slice A2), paid from the military budget right after the needs.
/// Land orders raise men a class a week over [`PeaceRules::raise_weeks`], untrained and drawing
/// their equipment from the depot and the purchases; ships, aircraft and strategic forces are
/// built over [`PeaceRules::build_weeks`] and join whole, when their equipment is complete.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForceOrder {
    /// Unit type key in `units.ron`; an order of a type the catalogue lacks is dropped.
    pub unit: String,
    /// The generation laid down: the newest the country could equip in the first week the order
    /// was paid; `None` until then.
    pub generation: Option<u8>,
    pub colonial: bool,
    /// Elements ordered, and of them delivered.
    pub ordered: Fixed,
    pub delivered: Fixed,
    /// Equipment of the elements still being built (ships on the slipway, aircraft on the line).
    pub built: Vec<(u16, Fixed)>,
    /// Of [`CountryWar::orders`], what this order buys this week; credited to `built` next week.
    pub buying: Vec<(u16, Fixed)>,
    /// Weeks in a row the order got nothing for want of room in the military budget.
    pub stalled: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryWar {
    pub formations: Vec<Formation>,
    /// Trained men out of service.
    pub reserve: u64,
    /// Men under arms on land over the men of service age, measured the first January the
    /// pyramid exists (zero before), or taken from `share_override`.
    pub peace_share: Fixed,
    /// The `conscription` effect in force when the share was measured, per mille.
    pub law_then: i32,
    pub mobilized: bool,
    /// Military goods the state buys this week, by good: what the forces consume and what makes
    /// good their equipment. Paid before the rest of the state's goods.
    pub orders: Vec<(u16, Fixed)>,
    /// Of `orders`, the part that goes to the equipment (the rest is consumed).
    pub repair: Vec<(u16, Fixed)>,
    /// Of `orders`, equipment bought above what the formations need: the rest of the military
    /// budget once the needs and the modernization are paid. It goes to the depot.
    pub upgrade: Vec<(u16, Fixed)>,
    /// The war reserve: equipment held in store, by good. Formations that lack equipment draw on
    /// it first (reservists called up, a newer generation, the wear); it wears like the rest.
    pub depot: Vec<(u16, Fixed)>,
    /// Forces ordered and not yet delivered, in the order they were placed (slice A2).
    pub queue: Vec<ForceOrder>,
}

/// The forces of the world. Empty when the forces are not loaded: the economy then runs the
/// rule from before them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarState {
    /// By market (country) index.
    pub countries: Vec<CountryWar>,
    /// Soldiers (workers) each region holds, by region index; set every month.
    pub soldiers: Vec<i64>,
    /// Crises open and wars under way (slice B); empty in peace, and out of the hash then.
    pub conflicts: super::crisis::Conflicts,
    /// The war on land (slice C): control, advance, postures, losses; empty in peace.
    pub fighting: super::fight::Fighting,
    /// The composition the governments fixed and the conversions under way; empty while the
    /// war is switched off.
    pub composition: super::composition::CompositionState,
    /// The colonial wars under way (G6), by war; empty without one.
    pub colonial: Vec<super::colonial::Colonial>,
    /// The war at sea of the last week (slice C2): fleets, control, blockades; empty in peace.
    pub sea: super::sea::SeaWar,
    /// The nuclear programmes under way (N1); empty without one.
    pub nuclear: super::programme::NuclearState,
}

impl WarState {
    pub fn is_empty(&self) -> bool {
        self.countries.is_empty()
    }

    /// The forces of `forces.ron` for the countries of the scenario, in market order, fully
    /// equipped and trained. Reserve rows go to [`CountryWar::reserve`].
    pub fn initial(military: &Military, forces: &ForcesFile, countries: &[CountryDef]) -> Self {
        let countries = countries
            .iter()
            .map(|c| {
                let mut cw = CountryWar::default();
                let Some(f) = forces.country(&c.tag) else {
                    return cw;
                };
                for row in &f.rows {
                    let Some(ty) = military.unit(&row.unit) else {
                        continue;
                    };
                    if row.reserve {
                        cw.reserve += row.count;
                        continue;
                    }
                    let elements = match military.types[ty as usize].domain {
                        Domain::Land => Fixed::from_ratio(row.count as i64, 1000),
                        _ => Fixed::from_int(row.count as i64),
                    };
                    let spec = military.spec(ty, 0);
                    cw.formations.push(Formation {
                        unit: ty,
                        generation: 0,
                        elements,
                        colonial: row.station == Station::Colonies,
                        called_up: false,
                        equipment: spec
                            .equipment
                            .iter()
                            .map(|(g, q)| (*g, *q * elements))
                            .collect(),
                        training: Fixed::ONE,
                        raised: false,
                    });
                }
                cw
            })
            .collect();
        Self {
            countries,
            soldiers: Vec::new(),
            conflicts: Default::default(),
            fighting: Default::default(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }

    /// Military goods ordered this week by country `market`; `None` without forces, which keeps
    /// the state's fixed basket.
    pub fn orders(&self, market: usize) -> Option<&[(u16, Fixed)]> {
        if self.is_empty() {
            return None;
        }
        Some(
            self.countries
                .get(market)
                .map_or(&[][..], |c| &c.orders[..]),
        )
    }

    /// Soldiers (workers) region `region` should hold; `None` without forces.
    pub fn soldiers(&self, region: usize) -> Option<i64> {
        if self.is_empty() {
            return None;
        }
        Some(self.soldiers.get(region).copied().unwrap_or(0))
    }
}

/// Men in a formation: elements times the men per element of its generation.
pub fn men(military: &Military, f: &Formation) -> i64 {
    let spec = military.spec(f.unit, f.generation as usize);
    (f.elements * spec.men as i64).to_int_round()
}

impl HashState for Formation {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.unit);
        h.write_u8(self.generation);
        self.elements.hash_state(h);
        self.colonial.hash_state(h);
        self.called_up.hash_state(h);
        h.write_len(self.equipment.len());
        for (g, q) in &self.equipment {
            h.write_u16(*g);
            q.hash_state(h);
        }
        self.training.hash_state(h);
        // Only when set, and the queue only when it holds orders: a world where nobody ordered
        // anything hashes as before the orders (slice A).
        if self.raised {
            h.write_u8(1);
        }
    }
}

impl HashState for ForceOrder {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.unit.len());
        for b in self.unit.bytes() {
            h.write_u8(b);
        }
        match self.generation {
            None => h.write_u8(0),
            Some(g) => {
                h.write_u8(1);
                h.write_u8(g);
            }
        }
        self.colonial.hash_state(h);
        self.ordered.hash_state(h);
        self.delivered.hash_state(h);
        hash_goods(h, &self.built);
        hash_goods(h, &self.buying);
        h.write_u32(self.stalled);
    }
}

fn hash_goods(h: &mut StateHasher, list: &[(u16, Fixed)]) {
    h.write_len(list.len());
    for (g, q) in list {
        h.write_u16(*g);
        q.hash_state(h);
    }
}

impl HashState for CountryWar {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.formations.len());
        for f in &self.formations {
            f.hash_state(h);
        }
        h.write_u64(self.reserve);
        self.peace_share.hash_state(h);
        h.write_i64(i64::from(self.law_then));
        self.mobilized.hash_state(h);
        hash_goods(h, &self.orders);
        hash_goods(h, &self.repair);
        hash_goods(h, &self.upgrade);
        hash_goods(h, &self.depot);
        if !self.queue.is_empty() {
            h.write_len(self.queue.len());
            for o in &self.queue {
                o.hash_state(h);
            }
        }
    }
}

impl HashState for WarState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.countries.len());
        for c in &self.countries {
            c.hash_state(h);
        }
        h.write_i64_slice(&self.soldiers);
        if !self.conflicts.is_empty() {
            self.conflicts.hash_state(h);
        }
        if !self.fighting.is_empty() {
            self.fighting.hash_state(h);
        }
        if !self.composition.is_empty() {
            self.composition.hash_state(h);
        }
        if !self.colonial.is_empty() {
            h.write_len(self.colonial.len());
            for c in &self.colonial {
                c.hash_state(h);
            }
        }
        if !self.sea.is_empty() {
            self.sea.hash_state(h);
        }
        if !self.nuclear.is_empty() {
            self.nuclear.hash_state(h);
        }
    }
}

/// [`WarState`] as version 30 of the world wrote it, before the war at sea.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeSea {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::Fighting,
    composition: super::composition::CompositionState,
    colonial: Vec<super::colonial::Colonial>,
}

impl From<WarStateBeforeSea> for WarState {
    fn from(w: WarStateBeforeSea) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting,
            composition: w.composition,
            colonial: w.colonial,
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// [`WarState`] as version 33 of the world wrote it, before the colonial war.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeColonial {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::Fighting,
    composition: super::composition::CompositionState,
}

impl From<WarStateBeforeColonial> for WarState {
    fn from(w: WarStateBeforeColonial) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting,
            composition: w.composition,
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// [`WarState`] as version 37 of the world wrote it, before the nuclear tests and the crisis
/// rung.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeTests {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::Fighting,
    composition: super::composition::CompositionState,
    colonial: Vec<super::colonial::Colonial>,
    sea: super::sea::SeaWar,
    nuclear: super::programme::NuclearStateBeforeTests,
}

impl From<WarStateBeforeTests> for WarState {
    fn from(w: WarStateBeforeTests) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting,
            composition: w.composition,
            colonial: w.colonial,
            sea: w.sea,
            nuclear: w.nuclear.into(),
        }
    }
}

/// [`WarState`] as version 38 of the world wrote it, before the nuclear use.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeUse {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::Fighting,
    composition: super::composition::CompositionState,
    colonial: Vec<super::colonial::Colonial>,
    sea: super::sea::SeaWar,
    nuclear: super::programme::NuclearStateBeforeUse,
}

impl From<WarStateBeforeUse> for WarState {
    fn from(w: WarStateBeforeUse) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting,
            composition: w.composition,
            colonial: w.colonial,
            sea: w.sea,
            nuclear: w.nuclear.into(),
        }
    }
}

/// [`WarState`] as version 35 of the world wrote it, before the nuclear programmes.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeNuclear {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::Fighting,
    composition: super::composition::CompositionState,
    colonial: Vec<super::colonial::Colonial>,
    sea: super::sea::SeaWar,
}

impl From<WarStateBeforeNuclear> for WarState {
    fn from(w: WarStateBeforeNuclear) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting,
            composition: w.composition,
            colonial: w.colonial,
            sea: w.sea,
            nuclear: Default::default(),
        }
    }
}

/// [`WarState`] as version 28 of the world wrote it, before the composition of the forces.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeComposition {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::FightingBeforeDoctrine,
}

impl From<WarStateBeforeComposition> for WarState {
    fn from(w: WarStateBeforeComposition) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting.into(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// [`WarState`] as version 26 of the world wrote it, before the entrenchment.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeStill {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::FightingBeforeStill,
}

impl From<WarStateBeforeStill> for WarState {
    fn from(w: WarStateBeforeStill) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting.into(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// [`WarState`] as version 29 of the world wrote it, before the generation of the armies.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeDoctrine {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::FightingBeforeDoctrine,
    composition: super::composition::CompositionState,
}

impl From<WarStateBeforeDoctrine> for WarState {
    fn from(w: WarStateBeforeDoctrine) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting.into(),
            composition: w.composition,
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// [`WarState`] as version 27 of the world wrote it, before the war support and the occupation.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeSupport {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
    fighting: super::fight::FightingBeforeSupport,
}

impl From<WarStateBeforeSupport> for WarState {
    fn from(w: WarStateBeforeSupport) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: w.fighting.into(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// [`WarState`] as versions 15 to 19 of the world wrote it, before the crises (slice B).
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeCrises {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
}

impl From<WarStateBeforeCrises> for WarState {
    fn from(w: WarStateBeforeCrises) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: Default::default(),
            fighting: Default::default(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// [`Formation`] as versions 9 to 14 of the world wrote it, before the orders (no `raised`).
#[derive(Deserialize)]
pub(crate) struct FormationBeforeOrders {
    unit: u16,
    generation: u8,
    elements: Fixed,
    colonial: bool,
    called_up: bool,
    equipment: Vec<(u16, Fixed)>,
    training: Fixed,
}

/// [`CountryWar`] as versions 9 to 14 of the world wrote it, before the orders (no `queue`).
#[derive(Deserialize)]
pub(crate) struct CountryWarBeforeOrders {
    formations: Vec<FormationBeforeOrders>,
    reserve: u64,
    peace_share: Fixed,
    law_then: i32,
    mobilized: bool,
    orders: Vec<(u16, Fixed)>,
    repair: Vec<(u16, Fixed)>,
    upgrade: Vec<(u16, Fixed)>,
    depot: Vec<(u16, Fixed)>,
}

/// [`WarState`] as versions 9 to 14 of the world wrote it, before the orders.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeOrders {
    countries: Vec<CountryWarBeforeOrders>,
    soldiers: Vec<i64>,
}

impl From<WarStateBeforeOrders> for WarState {
    fn from(w: WarStateBeforeOrders) -> Self {
        Self {
            countries: w
                .countries
                .into_iter()
                .map(|c| CountryWar {
                    formations: c
                        .formations
                        .into_iter()
                        .map(|f| Formation {
                            unit: f.unit,
                            generation: f.generation,
                            elements: f.elements,
                            colonial: f.colonial,
                            called_up: f.called_up,
                            equipment: f.equipment,
                            training: f.training,
                            raised: false,
                        })
                        .collect(),
                    reserve: c.reserve,
                    peace_share: c.peace_share,
                    law_then: c.law_then,
                    mobilized: c.mobilized,
                    orders: c.orders,
                    repair: c.repair,
                    upgrade: c.upgrade,
                    depot: c.depot,
                    queue: Vec::new(),
                })
                .collect(),
            soldiers: w.soldiers,
            conflicts: Default::default(),
            fighting: Default::default(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

impl From<WarStateBeforeOrders> for WarStateBeforeCrises {
    fn from(w: WarStateBeforeOrders) -> Self {
        let w: WarState = w.into();
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
        }
    }
}

/// [`super::crisis::Conflicts`] as versions 20 to 22 of the world wrote it, before the calls of
/// third parties (W4).
#[derive(Default, Deserialize)]
pub(crate) struct ConflictsBeforeCalls {
    crises: Vec<super::crisis::Crisis>,
    wars: Vec<super::crisis::War>,
    next_id: u32,
}

/// [`WarState`] as versions 20 to 22 of the world wrote it, before the calls of third parties.
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeCalls {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: ConflictsBeforeCalls,
}

impl From<WarStateBeforeCalls> for WarState {
    fn from(w: WarStateBeforeCalls) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: super::crisis::Conflicts {
                crises: w.conflicts.crises,
                wars: w.conflicts.wars,
                next_id: w.conflicts.next_id,
                calls: Vec::new(),
            },
            fighting: Default::default(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}

/// Men per element of a land formation: every land type of `units.ron` counts its elements in
/// thousands of men.
const LAND_MEN_PER_ELEMENT: u64 = 1000;

/// The elements `disband` would take from formation `index` of `c`, and their equipment in the
/// formation's order; `None` when nothing would be disbanded (no such formation, called-up
/// reservists, none asked).
fn disbanded(c: &CountryWar, index: usize, elements: u32) -> Option<(Fixed, Vec<(u16, Fixed)>)> {
    let f = c.formations.get(index)?;
    if f.called_up || !f.elements.is_positive() || elements == 0 {
        return None;
    }
    let n = Fixed::from_int(i64::from(elements)).min(f.elements);
    let share = n / f.elements;
    Some((
        n,
        f.equipment.iter().map(|(g, q)| (*g, *q * share)).collect(),
    ))
}

/// What disbanding `elements` elements of formation `index` would send to the depot, by good:
/// what the forces tab shows before the click. The same account as [`disband`].
pub fn disband_preview(c: &CountryWar, index: usize, elements: u32) -> Vec<(u16, Fixed)> {
    disbanded(c, index, elements)
        .map(|(_, freed)| freed.into_iter().filter(|(_, q)| q.is_positive()).collect())
        .unwrap_or_default()
}

/// Disbands `elements` whole elements of formation `index` of `c` (fewer if it holds fewer):
/// their share of its equipment goes to the depot, and the men of a raised (ordered) land
/// formation join the reserve. A formation left empty leaves the list. Called-up reservists are
/// left to `Mobilize`. Returns the elements disbanded.
pub fn disband(c: &mut CountryWar, index: usize, elements: u32) -> Fixed {
    let Some((n, freed)) = disbanded(c, index, elements) else {
        return Fixed::ZERO;
    };
    let f = &mut c.formations[index];
    for (e, (_, q)) in f.equipment.iter_mut().zip(&freed) {
        e.1 -= *q;
    }
    f.elements -= n;
    if f.raised {
        c.reserve += (n * LAND_MEN_PER_ELEMENT as i64).to_int_floor().max(0) as u64;
    }
    for (g, q) in freed {
        if q.is_positive() {
            match c.depot.binary_search_by_key(&g, |(k, _)| *k) {
                Ok(i) => c.depot[i].1 += q,
                Err(i) => c.depot.insert(i, (g, q)),
            }
        }
    }
    if !c.formations[index].elements.is_positive() {
        c.formations.remove(index);
    }
    n
}

/// [`WarState`] as versions 23 to 25 of the world wrote it, before the war on land (slice C).
#[derive(Default, Deserialize)]
pub(crate) struct WarStateBeforeFighting {
    countries: Vec<CountryWar>,
    soldiers: Vec<i64>,
    conflicts: super::crisis::Conflicts,
}

impl From<WarStateBeforeFighting> for WarState {
    fn from(w: WarStateBeforeFighting) -> Self {
        Self {
            countries: w.countries,
            soldiers: w.soldiers,
            conflicts: w.conflicts,
            fighting: Default::default(),
            composition: Default::default(),
            colonial: Vec::new(),
            sea: Default::default(),
            nuclear: Default::default(),
        }
    }
}
