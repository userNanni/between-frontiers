//! The blocs for the "Blocs" lens: `bf_sim::influence::bloc_views` with countries by tag, read
//! once a month and when the borders change.

use bf_sim::game::GameWorld;
use bf_sim::influence::BlocKind;
use laplace_schelling::calendar::Date;

/// One bloc as the lens paints and lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct BlocLine {
    /// Stable key (`org-<key>` names it when the organization has a name).
    pub key: String,
    pub kind: BlocKind,
    pub leader: Option<String>,
    /// Members by tag, the leader included.
    pub members: Vec<String>,
    pub since: Date,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlocsView {
    pub month: (i32, u8),
    /// Border changes when read: a change reads the blocs again.
    pub changes: usize,
    pub blocs: Vec<BlocLine>,
}

pub fn read(
    world: &GameWorld,
    diplomacy: &bf_sim::diplomacy::DiplomacyFile,
    borders: &bf_sim::borders::BorderRules,
    date: Date,
) -> BlocsView {
    let tag = |i: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(i))
            .map(|(_, c)| c.tag.clone())
    };
    let blocs = bf_sim::influence::bloc_views(diplomacy, borders, world, date)
        .into_iter()
        .map(|b| BlocLine {
            key: b.key,
            kind: b.kind,
            leader: b.leader.and_then(tag),
            members: b.members.iter().filter_map(|m| tag(m.country)).collect(),
            since: b.since,
        })
        .collect();
    BlocsView {
        month: (date.year, date.month),
        changes: world.borders.changes.len(),
        blocs,
    }
}

/// The kinds the lens offers, in the order of its list.
pub const KINDS: [BlocKind; 3] = [BlocKind::Military, BlocKind::Monetary, BlocKind::Subjects];

/// For each tag, the bloc of `kind` it belongs to (the largest when it is in more than one),
/// and whether it holds customs ties: what the lens paints.
pub fn by_country(view: &BlocsView, kind: BlocKind) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for (i, b) in view.blocs.iter().enumerate() {
        if b.kind != kind {
            continue;
        }
        for m in &b.members {
            match out.iter_mut().find(|(t, _)| t == m) {
                Some((_, j)) if view.blocs[*j].members.len() >= b.members.len() => {}
                Some((_, j)) => *j = i,
                None => out.push((m.clone(), i)),
            }
        }
    }
    out
}

/// Tags in a customs bloc.
pub fn in_customs(view: &BlocsView) -> Vec<&str> {
    view.blocs
        .iter()
        .filter(|b| b.kind == BlocKind::Customs)
        .flat_map(|b| b.members.iter().map(String::as_str))
        .collect()
}
