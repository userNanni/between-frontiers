//! What a technology opens and what it takes: the units it brings into service, the
//! technologies it leads to, its gains once diffused, the requirements a country still lacks;
//! and the other way round, the technologies behind a building, a method, a good or a unit
//! searched by name.

use std::collections::BTreeSet;

use bf_i18n::{Arg, I18n};
use bf_sim::econ::Catalogue;
use bf_sim::research::{EFFECTS, Stage, TechTree};
use bf_sim::war::Military;

/// Unit generations that technology `t` brings into service: (unit key, generation index),
/// where it is the last requirement added (the base type, or a later generation).
pub fn units(military: &Military, t: usize) -> Vec<(String, usize)> {
    let t = t as u16;
    let mut out = Vec::new();
    for ty in &military.types {
        for (g, spec) in ty.generations.iter().enumerate() {
            let before = g
                .checked_sub(1)
                .map(|p| ty.generations[p].requires.as_slice())
                .unwrap_or(&[]);
            if spec.requires.contains(&t) && !before.contains(&t) {
                out.push((ty.key.clone(), g));
            }
        }
    }
    out
}

/// A unit generation's name: the type, with its generation from the second on.
pub fn unit_label(i18n: &I18n, key: &str, generation: usize) -> String {
    let name = i18n.tr(&format!("unit-{key}"));
    if generation == 0 {
        name
    } else {
        i18n.tr_args(
            "orders-generation",
            &[
                ("unit", Arg::String(name)),
                ("generation", Arg::String((generation + 1).to_string())),
            ],
        )
    }
}

/// The technologies that require `t` directly.
pub fn leads_to(tree: &TechTree, t: usize) -> Vec<usize> {
    let t = t as u16;
    (0..tree.techs.len())
        .filter(|&d| tree.techs[d].requires.contains(&t))
        .collect()
}

/// Every requirement, direct or not, still below the science stage: what has to be researched
/// before `t` can be, from the first to the last (by level).
pub fn missing(tree: &TechTree, stages: &[Stage], t: usize) -> Vec<usize> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<usize> = tree.techs[t].requires.iter().map(|&r| r as usize).collect();
    while let Some(x) = stack.pop() {
        if stages.get(x).is_some_and(|s| *s >= Stage::Science) || !seen.insert(x) {
            continue;
        }
        stack.extend(tree.techs[x].requires.iter().map(|&r| r as usize));
    }
    let mut out: Vec<usize> = seen.into_iter().collect();
    out.sort_by_key(|&x| (tree.techs[x].level, x));
    out
}

/// The gains of `t` once diffused, said: productivity by building group, then the other
/// effects.
pub fn gains(i18n: &I18n, tree: &TechTree, t: usize) -> Vec<String> {
    let pct = |per_mille: u16| super::budget::pct(i18n, f64::from(per_mille) / 10.0, 1);
    let tech = &tree.techs[t];
    let mut out: Vec<String> = tech
        .productivity
        .iter()
        .map(|(group, pm)| {
            i18n.tr_args(
                "research-gain-productivity",
                &[
                    ("pct", Arg::String(pct(*pm))),
                    (
                        "group",
                        Arg::String(i18n.tr(&format!("building-group-{group}"))),
                    ),
                ],
            )
        })
        .collect();
    out.extend(tech.effects.iter().filter_map(|&(e, pm)| {
        let key = EFFECTS.get(usize::from(e))?;
        Some(i18n.tr_args(
            &format!("research-effect-{key}"),
            &[("pct", Arg::String(pct(pm)))],
        ))
    }));
    out
}

/// Folds a name for search: lower case, accents off.
pub fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

/// The technologies whose name, or the name of something they open (a method, its building,
/// a good, a unit), holds `query`: the way back from what the player wants to the technology
/// behind it. Empty for a blank query.
pub fn search(
    i18n: &I18n,
    tree: &TechTree,
    cat: &Catalogue,
    military: Option<&Military>,
    query: &str,
) -> BTreeSet<usize> {
    let q = fold(query.trim());
    if q.is_empty() {
        return BTreeSet::new();
    }
    let hit = |s: String| fold(&s).contains(&q);
    let mut out = BTreeSet::new();
    for (t, tech) in tree.techs.iter().enumerate() {
        let found = hit(i18n.tr(&format!("tech-{}", tech.key)))
            || tech.methods.iter().any(|&(b, m)| {
                let b = &cat.buildings[b as usize];
                hit(i18n.tr(&format!("method-{}", b.methods[m as usize].key)))
                    || hit(i18n.tr(&format!("building-{}", b.key)))
            })
            || tech
                .goods
                .iter()
                .any(|&g| hit(i18n.tr(&format!("good-{}", cat.goods[g as usize].key))))
            || military.is_some_and(|m| {
                units(m, t)
                    .into_iter()
                    .any(|(k, _)| hit(i18n.tr(&format!("unit-{k}"))))
            });
        if found {
            out.insert(t);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_folds_case_and_accents() {
        assert_eq!(fold("Alumínio Ção"), "aluminio cao");
    }
}
