//! The keys of the catalogues the world indexes by position (technologies, goods, buildings),
//! written with the snapshot so that a save outlives a catalogue that changed: technologies
//! realign by key; goods and buildings may only be added at the end. A save the catalogue can
//! no longer serve is refused before the game starts, never left to fail in play.

use serde::{Deserialize, Serialize};

use crate::game::{GameWorld, Rules};
use crate::research::Stage;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogKeys {
    pub techs: Vec<String>,
    pub goods: Vec<String>,
    pub buildings: Vec<String>,
}

impl CatalogKeys {
    pub fn of(rules: &Rules) -> Self {
        Self {
            techs: rules
                .tree
                .as_ref()
                .map(|t| t.techs.iter().map(|x| x.key.clone()).collect())
                .unwrap_or_default(),
            goods: rules
                .catalogue
                .goods
                .iter()
                .map(|g| g.key.clone())
                .collect(),
            buildings: rules
                .catalogue
                .buildings
                .iter()
                .map(|b| b.key.clone())
                .collect(),
        }
    }
}

/// Which catalogue a save cannot be read against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogError {
    Techs,
    Goods,
    Buildings,
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CatalogError::Techs => "the technology tree changed in a way the save cannot follow",
            CatalogError::Goods => "the goods changed in a way the save cannot follow",
            CatalogError::Buildings => "the buildings changed in a way the save cannot follow",
        })
    }
}

/// Fits a loaded world to the catalogues of `rules`: technologies by key when the save has its
/// keys; otherwise (and for goods and buildings) only what grew at the end is accepted.
pub fn fit(
    world: &mut GameWorld,
    saved: Option<&CatalogKeys>,
    rules: &Rules,
) -> Result<(), CatalogError> {
    let now = CatalogKeys::of(rules);
    fit_techs(world, saved.map(|s| s.techs.as_slice()), &now.techs)?;
    // Goods and buildings: a save of before may lack the last ones (they are filled in), never
    // have them elsewhere.
    match saved {
        Some(s) if !now.goods.starts_with(&s.goods) => return Err(CatalogError::Goods),
        Some(s) if !now.buildings.starts_with(&s.buildings) => {
            return Err(CatalogError::Buildings);
        }
        _ => {}
    }
    if world.econ.world_price.len() > now.goods.len() {
        return Err(CatalogError::Goods);
    }
    let buildings = now.buildings.len();
    if world
        .econ
        .regions
        .iter()
        .flat_map(|r| &r.plants)
        .any(|p| usize::from(p.kind) >= buildings)
    {
        return Err(CatalogError::Buildings);
    }
    crate::econ::compat::fit_goods(&mut world.econ, &rules.catalogue);
    Ok(())
}

fn fit_techs(
    world: &mut GameWorld,
    saved: Option<&[String]>,
    now: &[String],
) -> Result<(), CatalogError> {
    let r = &mut world.research;
    if r.stage.is_empty() || now.is_empty() {
        return Ok(());
    }
    let fits = |r: &crate::research::ResearchState| {
        r.firsts.len() == now.len()
            && r.stage.iter().all(|s| s.len() == now.len())
            && r.progress.iter().all(|p| p.len() == now.len())
    };
    let Some(saved) = saved.filter(|s| *s != now) else {
        // The same tree, or a save without its keys: it fits as it is, or not at all.
        return if fits(r) {
            Ok(())
        } else {
            Err(CatalogError::Techs)
        };
    };
    if r.firsts.len() != saved.len() || r.stage.iter().any(|s| s.len() != saved.len()) {
        return Err(CatalogError::Techs);
    }
    // Where each technology of today was in the save; a new one starts unknown everywhere.
    let from: Vec<Option<usize>> = now
        .iter()
        .map(|k| saved.iter().position(|s| s == k))
        .collect();
    let to = |old: u16| -> Option<u16> {
        from.iter()
            .position(|f| *f == Some(usize::from(old)))
            .and_then(|i| u16::try_from(i).ok())
    };
    // Techniques carried in border operations name technologies by position too.
    let mut ops: Vec<&mut crate::borders::op::Operation> = Vec::new();
    let b = &mut world.borders;
    ops.extend(b.pending.iter_mut());
    ops.extend(b.changes.iter_mut().map(|c| &mut c.op));
    ops.extend(b.refused.iter_mut().map(|c| &mut c.op));
    for op in ops {
        if let Some(terms) = crate::borders::op::terms_mut(op) {
            for t in &mut terms.techniques {
                t.tech = to(t.tech).ok_or(CatalogError::Techs)?;
            }
        }
    }
    let r = &mut world.research;
    for s in &mut r.stage {
        let old = std::mem::take(s);
        *s = from
            .iter()
            .map(|f| f.map_or(Stage::Unknown, |i| old[i]))
            .collect();
    }
    for p in &mut r.progress {
        let old = std::mem::take(p);
        *p = from
            .iter()
            .map(|f| f.and_then(|i| old.get(i).copied()).unwrap_or_default())
            .collect();
    }
    let firsts = std::mem::take(&mut r.firsts);
    r.firsts = from
        .iter()
        .map(|f| f.map_or([0; 5], |i| firsts[i]))
        .collect();
    for f in &mut r.focus {
        *f = f.and_then(to);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{GAME_ID, GameCommand, LoadError, Session};
    use laplace_schelling::Date;
    use laplace_schelling::save::SaveFile;
    use std::sync::Arc;

    /// The rules with a technology inserted at `k` of the tree, everything after it shifted, as
    /// a change to the tree in the middle does.
    fn with_tech_inserted(rules: &Rules, k: usize) -> Rules {
        let mut tree = (**rules.tree.as_ref().unwrap()).clone();
        let shift = |t: &mut u16| {
            if usize::from(*t) >= k {
                *t += 1;
            }
        };
        for t in &mut tree.techs {
            t.requires.iter_mut().for_each(shift);
        }
        for m in &mut tree.method_tech {
            m.iter_mut().flatten().for_each(shift);
        }
        for g in &mut tree.good_techs {
            g.iter_mut().for_each(shift);
        }
        let mut new = tree.techs[k].clone();
        new.key = "inserted_for_the_test".into();
        new.methods.clear();
        new.goods.clear();
        new.effects.clear();
        new.productivity.clear();
        new.start = None;
        tree.techs.insert(k, new);
        Rules {
            tree: Some(Arc::new(tree)),
            ..rules.clone()
        }
    }

    #[test]
    fn a_technology_inserted_in_the_tree_realigns_by_key_or_is_refused() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let data = crate::data::GameData::load(&root).unwrap();
        let rules = data.rules();
        let mut s = Session::new(
            data.new_world(),
            1925,
            Date::new(1925, 1, 1),
            1,
            Some(rules.clone()),
        );
        s.submit(GameCommand::ChoosePlayer { tag: "BRA".into() });
        s.sim.run_until(Date::new(1925, 3, 1));
        let bytes = s.save("test", "catalogue").unwrap();
        let k = 5;
        let changed = with_tech_inserted(&rules, k);
        let keys = CatalogKeys::of(&changed).techs;

        // With its keys, the save realigns: each technology keeps its stage by key.
        let loaded = Session::load(&bytes, 1, Some(changed.clone())).expect("realigned");
        let (before, after) = (&s.world().research, &loaded.world().research);
        let old_keys = CatalogKeys::of(&rules).techs;
        for (c, stages) in after.stage.iter().enumerate() {
            assert_eq!(stages.len(), keys.len());
            assert_eq!(stages[k], Stage::Unknown);
            for (j, key) in keys.iter().enumerate().filter(|(j, _)| *j != k) {
                let i = old_keys.iter().position(|x| x == key).unwrap();
                assert_eq!(stages[j], before.stage[c][i], "{key}");
            }
        }
        // And it plays on.
        let mut loaded = loaded;
        loaded.sim.run_until(Date::new(1925, 6, 1));

        // Without its keys (a save written before them), it is refused, not played.
        let mut file: SaveFile<GameCommand> = SaveFile::decode(&bytes, GAME_ID).unwrap();
        file.snapshot = laplace_schelling::save::encode_snapshot(s.world()).unwrap();
        let legacy = file.encode().unwrap();
        assert!(Session::load(&legacy, 1, Some(rules.clone())).is_ok());
        assert!(matches!(
            Session::load(&legacy, 1, Some(changed)),
            Err(LoadError::Catalog(CatalogError::Techs))
        ));
    }
}
