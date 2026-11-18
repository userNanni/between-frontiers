//! The player's wars on land (phase 5, slice C; `docs/war.pt-BR.md`, "A guerra no tick"), for
//! the war window: each war's sides and the demand at stake, and one front per enemy country
//! with the player's posture, the forces of both sides, the losses and the advance of the last
//! week on each segment (`war::fight::Fighting::last_week`, read as the tick left it, never
//! worked out again), how far each segment has gone into its state, and the losses since the
//! first war. Read in the simulation's thread when the week changes.

use bf_sim::game::GameWorld;
use bf_sim::treaty::Proposal;
use bf_sim::war::fight::FightSetup;
use laplace_schelling::Fixed;

/// One segment of a front, from the player's side.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// The player's side's state and the enemy's (map state ids).
    pub ours: u16,
    pub theirs: u16,
    /// Fought this week (someone attacked).
    pub engaged: bool,
    /// Men (thousands × 1,000) and power of each side: ours first.
    pub men: [f64; 2],
    pub power: [f64; 2],
    /// The six terms of each side's power (`calc::LandTerms::named`): ours first.
    pub terms: [[(&'static str, f64); 6]; 2],
    /// Casualties this week: ours first.
    pub losses: [u64; 2],
    /// Kilometres the front moved this week; positive when the player's side advanced.
    pub advance_km: f64,
    /// How far the player's side has gone into the enemy's state, and the enemy into ours, over
    /// the state's depth (the state changes hands at 1).
    pub into_theirs: f64,
    pub into_ours: f64,
}

/// The front against one enemy country.
#[derive(Clone, Debug, PartialEq)]
pub struct Front {
    pub enemy: String,
    /// The player's posture: attack, or hold (the default).
    pub attack: bool,
    pub segments: Vec<Segment>,
    /// Land casualties of the enemy since its first war.
    pub enemy_losses: u64,
}

impl Front {
    pub fn men(&self, side: usize) -> f64 {
        self.segments.iter().map(|s| s.men[side]).sum()
    }
    pub fn power(&self, side: usize) -> f64 {
        self.segments.iter().map(|s| s.power[side]).sum()
    }
    pub fn week_losses(&self, side: usize) -> u64 {
        self.segments.iter().map(|s| s.losses[side]).sum()
    }
    /// Net kilometres the player's side gained this week over the front.
    pub fn advance_km(&self) -> f64 {
        self.segments.iter().map(|s| s.advance_km).sum()
    }
}

/// One war of the player's.
#[derive(Clone, Debug, PartialEq)]
pub struct WarLine {
    pub id: u32,
    /// The player's side and the other, leaders first.
    pub ours: Vec<String>,
    pub theirs: Vec<String>,
    /// The side that declared is the player's.
    pub declared: bool,
    pub since: (i32, u8, u8),
    pub weeks: i64,
    pub demand: Proposal,
    pub fronts: Vec<Front>,
    /// States of the player's side held by the enemy, and of the enemy's held by the player's
    /// side (map state ids).
    pub lost: Vec<u16>,
    pub taken: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WarView {
    pub me: String,
    pub wars: Vec<WarLine>,
    /// Land casualties of the player's country since its first war.
    pub losses: u64,
    /// What the fighting looked like when read: a change reads it again.
    pub signature: Signature,
}

pub type Signature = (usize, usize, usize, usize, i64);

pub fn signature(world: &GameWorld, days: i64) -> Signature {
    let f = &world.war.fighting;
    (
        world.war.conflicts.wars.len(),
        f.last_week.len(),
        f.control.len(),
        f.postures.len(),
        days / 7,
    )
}

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// The player's wars; `None` without a player or without the war rule.
pub fn read(world: &GameWorld, setup: &FightSetup, days: i64) -> Option<WarView> {
    let tag = world.player_tag()?;
    let me = world.countries.iter().position(|(_, c)| c.tag == tag)? as u16;
    let tag_of = |i: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(i))
            .map_or_else(String::new, |(_, c)| c.tag.clone())
    };
    let fighting = &world.war.fighting;
    let regions = &world.econ.regions;
    let lost_of = |c: u16| {
        fighting
            .losses
            .iter()
            .find(|(k, _)| *k == c)
            .map_or(0, |(_, l)| *l)
    };
    let advance = |from: u32, into: u32| {
        fighting
            .advance
            .iter()
            .find(|(a, b, _)| *a == from && *b == into)
            .map_or(0.0, |(_, _, k)| f(*k))
    };
    let depth = |r: u32| {
        regions
            .get(r as usize)
            .and_then(|x| setup.depth_km.get(&x.state))
            .map_or(0.0, |d| f(*d))
    };
    let share = |from: u32, into: u32| {
        let d = depth(into);
        if d > 0.0 {
            (advance(from, into) / d).clamp(0.0, 1.0)
        } else {
            0.0
        }
    };
    let wars = world
        .war
        .conflicts
        .wars
        .iter()
        .filter_map(|w| {
            let (ours, theirs, declared) = if w.side_a.contains(&me) {
                (&w.side_a, &w.side_b, true)
            } else if w.side_b.contains(&me) {
                (&w.side_b, &w.side_a, false)
            } else {
                return None;
            };
            // One front per enemy country the player's country touches this week; the enemies
            // it does not touch get a front without segments (posture still set).
            let fronts = theirs
                .iter()
                .map(|&enemy| {
                    let segments = fighting
                        .last_week
                        .iter()
                        .filter_map(|g| {
                            let (us, them, from_ours) = if g.a == me && g.b == enemy {
                                (0usize, 1usize, true)
                            } else if g.b == me && g.a == enemy {
                                (1, 0, false)
                            } else {
                                return None;
                            };
                            let o = &g.outcome;
                            let (r_ours, r_theirs) = if from_ours {
                                (g.from, g.into)
                            } else {
                                (g.into, g.from)
                            };
                            let terms = |side: usize| o.terms[side].named().map(|(k, v)| (k, f(v)));
                            let state = |r: u32| regions.get(r as usize).map_or(0, |x| x.state);
                            Some(Segment {
                                ours: state(r_ours),
                                theirs: state(r_theirs),
                                engaged: o.engaged,
                                men: [
                                    f(o.terms[us].number) * 1000.0,
                                    f(o.terms[them].number) * 1000.0,
                                ],
                                power: [f(o.power[us]), f(o.power[them])],
                                terms: [terms(us), terms(them)],
                                losses: [o.losses[us], o.losses[them]],
                                advance_km: if us == 0 {
                                    f(o.advance_km)
                                } else {
                                    -f(o.advance_km)
                                },
                                into_theirs: share(r_ours, r_theirs),
                                into_ours: share(r_theirs, r_ours),
                            })
                        })
                        .collect();
                    Front {
                        enemy: tag_of(enemy),
                        attack: fighting.posture(me, enemy).unwrap_or(false),
                        segments,
                        enemy_losses: lost_of(enemy),
                    }
                })
                .collect();
            // Occupied states between the two sides.
            let mut lost = Vec::new();
            let mut taken = Vec::new();
            for &(r, by) in &fighting.control {
                let Some(region) = regions.get(r as usize) else {
                    continue;
                };
                if ours.contains(&region.market) && theirs.contains(&by) {
                    lost.push(region.state);
                } else if theirs.contains(&region.market) && ours.contains(&by) {
                    taken.push(region.state);
                }
            }
            let since = laplace_schelling::calendar::Date::new(w.since.0, w.since.1, w.since.2);
            Some(WarLine {
                id: w.id,
                ours: ours.iter().map(|i| tag_of(*i)).collect(),
                theirs: theirs.iter().map(|i| tag_of(*i)).collect(),
                declared,
                since: w.since,
                weeks: (days - since.to_days()).max(0) / 7,
                demand: w.demand.clone(),
                fronts,
                lost,
                taken,
            })
        })
        .collect();
    Some(WarView {
        me: tag.to_owned(),
        wars,
        losses: lost_of(me),
        signature: signature(world, days),
    })
}
