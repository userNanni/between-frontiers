//! War (phase 5): the unit catalogue, the combat rules, the forces on the start date and the pure
//! functions the war systems will be built from. Not wired into the simulation: nothing here is
//! in the schedule, the `GameWorld` or the save. Design in `docs/war.pt-BR.md`.
//!
//! - [`catalogue`]: `data/military/units.ron` and `data/military/rules.ron`, resolved against the
//!   economy catalogue and the technology tree into [`Military`].
//! - [`forces`]: `data/scenario/1925/forces.ron`, the armies, navies and air forces of 1925.
//! - [`calc`]: costs, mobilization, supply, front allocation, the land, sea and air combat steps
//!   and war exhaustion. Every function is deterministic, in fixed point, with no clock and no
//!   random draw, and returns its result term by term.
//! - [`doctrine`]: the generation of an army (second or third), its causes, and how it changes
//!   the way a front concentrates and breaks through.
//! - [`nuclear`]: second strike hurdle by hurdle, deterrence and the rungs of the crisis ladder
//!   it leaves open.
//! - [`ledger`]: the war's cost, week by week and in total.

pub mod calc;
pub mod catalogue;
pub mod doctrine;
pub mod forces;
pub mod ledger;
pub mod nuclear;
pub mod state;
pub mod system;
pub mod terrain;

use std::path::Path;

pub use calc::*;
pub use catalogue::*;
pub use doctrine::*;
pub use forces::*;
pub use ledger::*;
pub use nuclear::*;
pub use state::*;
pub use system::{Forces, place_soldiers, state_purchases, station_soldiers};
pub use terrain::*;

use crate::econ::Catalogue;
use crate::research::TechTree;
use crate::scenario::CountriesFile;

/// Everything the war data holds, loaded and checked.
#[derive(Clone, Debug)]
pub struct War {
    pub military: Military,
    /// The forces on the start date, when `data/scenario/1925/forces.ron` exists.
    pub forces_1925: Option<ForcesFile>,
    /// Terrain of every state, when `assets/map/terrain.ron` exists.
    pub terrain: Option<TerrainFile>,
    /// How the forces sit in peace, when `data/military/peace.ron` exists.
    pub peace: Option<PeaceRules>,
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Reads `data/military/` and the 1925 forces. `None` when `units.ron` is absent; an error when
/// it is present without the technology tree, or when anything fails to resolve or validate.
pub fn load(
    data: &Path,
    catalogue: &Catalogue,
    tree: Option<&TechTree>,
    countries: &CountriesFile,
) -> Result<Option<War>, String> {
    let units_path = data.join("military/units.ron");
    if !units_path.is_file() {
        return Ok(None);
    }
    let tree = tree.ok_or("data/military needs data/research/technologies.ron")?;
    let units: UnitsFile = read(&units_path)?;
    let rules: RulesFile = read(&data.join("military/rules.ron"))?;
    let military = Military::build(&units, &rules, catalogue, tree)
        .map_err(|e| format!("data/military: {}", e.join("; ")))?;
    let forces_path = data.join("scenario/1925/forces.ron");
    let forces_1925 = if forces_path.is_file() {
        let file: ForcesFile = read(&forces_path)?;
        file.validate(&military, tree, countries)
            .map_err(|e| format!("scenario/1925/forces.ron: {}", e.join("; ")))?;
        Some(file)
    } else {
        None
    };
    let terrain_path = data.join("../assets/map/terrain.ron");
    let terrain = if terrain_path.is_file() {
        Some(read::<TerrainFile>(&terrain_path)?)
    } else {
        None
    };
    let peace_path = data.join("military/peace.ron");
    let peace = if peace_path.is_file() {
        Some(read::<PeaceRules>(&peace_path)?)
    } else {
        None
    };
    Ok(Some(War {
        military,
        forces_1925,
        terrain,
        peace,
    }))
}
