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
    /// Width of the segment, km (the border between its two states); zero when unknown, and the
    /// density then does not count.
    pub width_km: f64,
    /// Weeks the segment has held still, and how dug in its defenders are, 0 to 1 (full at
    /// `front.dig_weeks`).
    pub still_weeks: u16,
    pub dug: f64,
    /// The mobility each side moved at here and the share of its men here in its mobile class,
    /// 0–1, ours first; `None` until a week is fought after a load (`SegmentWeek::mobility`).
    pub pace: Option<[(bf_sim::war::Mobility, f64); 2]>,
}

impl Segment {
    /// Men per km of front of each side, ours first; `None` when the width is unknown.
    pub fn density(&self) -> Option<[f64; 2]> {
        (self.width_km > 0.0).then(|| [self.men[0] / self.width_km, self.men[1] / self.width_km])
    }
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

    /// The front's advance this week as a front moves: the mean over its segments, and the
    /// segment that moved the most (with its sign).
    pub fn advance_mean(&self) -> f64 {
        if self.segments.is_empty() {
            0.0
        } else {
            self.advance_km() / self.segments.len() as f64
        }
    }

    pub fn advance_most(&self) -> f64 {
        self.segments
            .iter()
            .map(|s| s.advance_km)
            .max_by(|a, b| a.abs().total_cmp(&b.abs()))
            .unwrap_or(0.0)
    }
}

/// How near the player's government is to falling by occupation (`support::support_week`): the
/// metropolitan people under the enemy, per mille, against the share that brings it down, lower
/// with the capital taken; and what keeps it standing (a colony still held, an ally fighting).
#[derive(Clone, Debug, PartialEq)]
pub struct Overrun {
    pub share: i64,
    pub capital_taken: bool,
    /// The share that brings it down now (`overrun`, or `capital_overrun` with the capital taken),
    /// and the other one.
    pub threshold: i32,
    pub capital_threshold: i32,
    pub plain_threshold: i32,
    pub colony_held: bool,
    pub ally_fights: bool,
}

/// One side's support for the war (G4), per mille, as the tick wrote it.
#[derive(Clone, Debug, PartialEq)]
pub struct SupportLine {
    pub country: String,
    pub value: i32,
    /// What moved it this week: (term key, per mille), in the order the screen shows them.
    pub terms: Vec<(&'static str, i32)>,
    /// Where the terms pull it.
    pub target: i32,
    /// (interest group key, support per mille, clout per mille), the weightiest first.
    pub groups: Vec<(String, i32, u16)>,
    pub weeks: u32,
    /// Weeks before the government falls, below the collapse.
    pub countdown: Option<u16>,
    pub fallen: Option<Fallen>,
    /// The franchise and the censorship of the laws in force, per mille, and the share of the
    /// gap to the target the support closes in a week (`support::speed`: a free vote hastens
    /// it, censorship slows it).
    pub franchise: i32,
    pub censorship: i32,
    pub speed: i64,
}

/// How a government fell in the war.
#[derive(Clone, Debug, PartialEq)]
pub enum Fallen {
    /// A new form of government (law keys): the security reputation is halved.
    Regime { from: String, to: String },
    /// A republic's cabinet.
    Cabinet,
}

/// An occupied region between the war's sides.
#[derive(Clone, Debug, PartialEq)]
pub struct Occupied {
    pub state: u16,
    pub by: String,
    /// The player's side holds it.
    pub ours: bool,
    pub weeks: u32,
    /// Per mille.
    pub resistance: i32,
    /// Per mille of the garrison needed that is kept.
    pub garrison: i32,
    /// What the occupier took this week, real dollars.
    pub taken: f64,
    /// Per mille of the region's output the occupier takes: the extraction of its weeks
    /// (`occupation::extraction`, rising to the cap) cut by the garrison kept; and the cap.
    pub extraction: i32,
    pub extraction_cap: i32,
}

/// The garrison of one occupier over the regions it holds in this war (item 16): the men they
/// need, the men kept there last week, how many of those its garrison formations give (the
/// rest comes out of its field army, at most half of it), and its open garrison order.
#[derive(Clone, Debug, PartialEq)]
pub struct Garrison {
    pub by: String,
    /// The player's side holds the regions.
    pub ours: bool,
    pub need: u64,
    pub kept: u64,
    pub posts: u64,
    /// Garrison elements ordered and not yet delivered (a thousand men each); zero without an
    /// open order.
    pub ordered: f64,
}

/// A region the war wrecked.
#[derive(Clone, Debug, PartialEq)]
pub struct Wreck {
    pub state: u16,
    /// The player's side's land.
    pub ours: bool,
    /// Per mille of the building levels destroyed.
    pub share: i32,
    pub owed: f64,
    pub civilians: u64,
}

/// N8: a region's productivity lost to a nuclear strike's fallout, a stock that decays.
#[derive(Clone, Debug, PartialEq)]
pub struct Fallout {
    pub state: u16,
    /// The player's side's land.
    pub ours: bool,
    /// Per mille of the productivity lost.
    pub per_mille: u32,
    /// Years for the loss to halve, at the rule's yearly decay.
    pub half_years: u32,
    /// The January in which the loss falls under 1 % (10 per mille), as the simulation decays it.
    pub under_1pc: i32,
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
    /// The support of the player's country and of the enemy's leader; `None` before the tick
    /// wrote it (or without the war's conduct data).
    pub support: [Option<SupportLine>; 2],
    pub occupied: Vec<Occupied>,
    /// Each occupier's garrison, the player's side first.
    pub garrisons: Vec<Garrison>,
    pub wrecks: Vec<Wreck>,
    /// N8: the fallout on either side's land, the worst first.
    pub fallout: Vec<Fallout>,
    /// The nuclear balance with the other side's leader, as the player reads it; `None` when
    /// neither holds a warhead.
    pub nuclear: Option<crate::nuclear_view::Panel>,
    /// The expedition, when this is a colonial war of the player's metropole (G6).
    pub expedition: Option<Expedition>,
}

/// The player's expedition to a colony in revolt: what it landed, toward what, and what it ties.
#[derive(Clone, Debug, PartialEq)]
pub struct Expedition {
    pub colony: String,
    pub landed: u64,
    /// This week's goal and the cap (`expedition_cap` of the land men).
    pub goal: u64,
    pub cap: u64,
    /// The player's order; `None`: the AI's rule (the cover of O3).
    pub wanted: Option<u64>,
    /// The garrison and the landed men in the unit, and that share of the land army (‰).
    pub tied: u64,
    pub share: u16,
    /// The enemy's irregulars under arms.
    pub armed: u64,
    /// No cap: the colony is reached by land.
    pub by_land: bool,
    /// The sea route is cut (slice C2): the metropole's port state, the kept port state and the
    /// key of the zone the enemy holds; no man lands while it is.
    pub cut: Option<(u16, u16, String)>,
}

/// The player's navy this week (slice C2): where it is, why, and what it blockades. The mission
/// is fixed ("control"); the player's command (S1) comes with the next slice.
#[derive(Clone, Debug, PartialEq)]
pub struct FleetLine {
    /// The zone's key (`sea-zone-<key>`).
    pub zone: String,
    pub ships: u64,
    /// The enemy whose gateway it holds (stronger on the surface); `None`: at home (weaker).
    pub against: Option<String>,
    /// The enemy it measured itself against when it stays home.
    pub stronger: Option<String>,
    /// It holds the zone at the end of the week.
    pub holds: bool,
    /// The countries it blockades, and the trade pairs cut.
    pub blockades: Vec<String>,
    pub cut: usize,
    /// The pairs it cuts, with what they traded in a year before (both ways), the largest first.
    pub pairs: Vec<(String, String, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WarView {
    pub me: String,
    pub wars: Vec<WarLine>,
    /// Land casualties of the player's country since its first war.
    pub losses: u64,
    /// The support's thresholds, per mille: peace and collapse.
    pub thresholds: Option<(i32, i32)>,
    /// Weeks from the collapse to the government's fall.
    pub revolution_weeks: u16,
    /// The pace of the fronts (`front` of the military rules): men per km of a full line, and
    /// the weeks a still segment takes to dig in fully.
    pub full_line_men_km: f64,
    pub dig_weeks: u16,
    /// What the fighting looked like when read: a change reads it again.
    pub signature: Signature,
    /// The player's trade cut at sea by an enemy's blockade: (partner, what they traded in a
    /// year, both ways), the largest first.
    pub cut_ours: Vec<(String, f64)>,
    /// The player's nearness to falling by occupation, at war.
    pub overrun: Option<Overrun>,
    /// The player's navy, while at war and with ships.
    pub fleet: Option<FleetLine>,
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

fn support_line(
    world: &GameWorld,
    politics: Option<&bf_sim::politics::Politics>,
    rules: Option<&bf_sim::war::support::SupportRules>,
    country: u16,
    tag: String,
) -> Option<SupportLine> {
    let s = world.war.fighting.support_of(country)?;
    let (franchise, censorship) = politics.map_or((0, 0), |p| {
        let e = bf_sim::polity::effects_in_force(
            p,
            &world.politics,
            usize::from(country),
            bf_sim::politics::Scope::Metropole,
        );
        let at = |k: &str| {
            bf_sim::politics::effect_index(k)
                .and_then(|i| e.get(i))
                .copied()
                .unwrap_or(0)
        };
        (at("franchise"), at("censorship"))
    });
    let speed = rules.map_or(0, |r| bf_sim::war::support::speed(r, franchise, censorship));
    let t = &s.terms;
    let clout = world.politics.clout.get(usize::from(country));
    let mut groups: Vec<(String, i32, u16)> = s
        .groups
        .iter()
        .enumerate()
        .filter_map(|(g, v)| {
            let key = politics?.interest_groups.get(g)?.key.clone();
            let c = clout.and_then(|c| c.get(g)).copied().unwrap_or(0);
            Some((key, *v, c))
        })
        .collect();
    groups.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    let law = |i: u16| {
        politics
            .and_then(|p| p.laws.get(usize::from(i)))
            .map_or_else(|| i.to_string(), |l| l.key.clone())
    };
    Some(SupportLine {
        country: tag,
        value: s.value,
        terms: vec![
            ("start", t.start),
            ("casualties", t.casualties),
            ("duration", t.duration),
            ("deprivation", t.deprivation),
            ("territory", t.territory),
            ("result", t.result),
        ],
        target: t.target,
        groups,
        weeks: s.weeks,
        countdown: s.countdown,
        fallen: s.fallen.map(|f| match f {
            bf_sim::war::support::Fall::Regime { from, to } => Fallen::Regime {
                from: law(from),
                to: law(to),
            },
            bf_sim::war::support::Fall::Cabinet => Fallen::Cabinet,
        }),
        franchise,
        censorship,
        speed,
    })
}

/// The player's wars; `None` without a player or without the war rule.
pub fn read(
    world: &GameWorld,
    tag: &str,
    setup: &FightSetup,
    politics: Option<&bf_sim::politics::Politics>,
    front: Option<&bf_sim::war::FrontRule>,
    mil: Option<&bf_sim::war::Military>,
    days: i64,
) -> Option<WarView> {
    let dig_weeks = front.map_or(0, |r| r.dig_weeks);
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
    // A sea zone's key, from the war at sea's graph; its number without it.
    let zone_key = |z: u16| {
        setup
            .sea
            .as_ref()
            .and_then(|s| s.graph.file.zones.get(usize::from(z).wrapping_sub(1)))
            .map_or_else(|| z.to_string(), |x| x.key.clone())
    };
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
    let wars: Vec<WarLine> = world
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
                                width_km: {
                                    let (x, y) = (state(g.from), state(g.into));
                                    setup
                                        .width_km
                                        .get(&(x.min(y), x.max(y)))
                                        .map_or(0.0, |w| f(*w))
                                },
                                still_weeks: fighting.still_weeks(g.from, g.into),
                                dug: if dig_weeks > 0 {
                                    (f64::from(fighting.still_weeks(g.from, g.into))
                                        / f64::from(dig_weeks))
                                    .min(1.0)
                                } else {
                                    0.0
                                },
                                pace: g.mobility.map(|m| {
                                    let at = |side: usize| {
                                        (m[side], f64::from(g.mobile_share[side]) / 1000.0)
                                    };
                                    [at(us), at(them)]
                                }),
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
            let state = |r: u32| regions.get(r as usize).map(|x| (x.state, x.market));
            let occupation = setup.conduct.as_ref().map(|c| &c.rules.occupation);
            let occupied = fighting
                .occupied
                .iter()
                .filter_map(|o| {
                    let (st, owner) = state(o.region)?;
                    let mine = ours.contains(&o.by) && theirs.contains(&owner);
                    let theirs_ = theirs.contains(&o.by) && ours.contains(&owner);
                    (mine || theirs_).then(|| Occupied {
                        state: st,
                        by: tag_of(o.by),
                        ours: mine,
                        weeks: o.weeks,
                        resistance: o.resistance,
                        garrison: o.garrison,
                        taken: f(o.taken),
                        extraction: occupation.map_or(0, |r| {
                            (i64::from(bf_sim::war::occupation::extraction(r, o.weeks))
                                * i64::from(o.garrison)
                                / 1000) as i32
                        }),
                        extraction_cap: occupation.map_or(0, |r| r.extraction_cap),
                    })
                })
                .collect();
            // Each occupier's garrison: the need over its regions here, the share kept, its
            // garrison formations and its open order of them.
            let mut garrisons: Vec<Garrison> = Vec::new();
            if let Some(r) = occupation {
                let unit_key = r.garrison_unit.as_deref();
                let unit = unit_key.and_then(|k| mil.and_then(|m| m.unit(k)));
                let mut by: Vec<u16> = Vec::new();
                for o in &fighting.occupied {
                    let Some((_, owner)) = state(o.region) else {
                        continue;
                    };
                    let side = (ours.contains(&o.by) && theirs.contains(&owner))
                        || (theirs.contains(&o.by) && ours.contains(&owner));
                    if side && !by.contains(&o.by) {
                        by.push(o.by);
                    }
                }
                for c in by {
                    let (mut need, mut kept) = (0u64, 0u64);
                    for o in fighting.occupied.iter().filter(|o| o.by == c) {
                        let people = regions.get(o.region as usize).map_or(0, |x| x.population());
                        let n = bf_sim::war::occupation::garrison_need(r, people, o.resistance);
                        need += n;
                        kept += n * o.garrison.clamp(0, 1000) as u64 / 1000;
                    }
                    let country = world.war.countries.get(usize::from(c));
                    let posts = match (mil, country) {
                        (Some(m), Some(cw)) => bf_sim::war::fight::posts(m, cw, unit).0,
                        _ => 0,
                    };
                    let ordered = match (unit_key, country) {
                        (Some(k), Some(cw)) => cw
                            .queue
                            .iter()
                            .filter(|q| q.unit == k)
                            .map(|q| f(q.ordered - q.delivered))
                            .sum(),
                        _ => 0.0,
                    };
                    garrisons.push(Garrison {
                        by: tag_of(c),
                        ours: ours.contains(&c),
                        need,
                        kept,
                        posts,
                        ordered,
                    });
                }
                garrisons.sort_by_key(|g| !g.ours);
            }
            let wrecks = fighting
                .wrecks
                .iter()
                .filter_map(|k| {
                    let (st, owner) = state(k.region)?;
                    let side = if ours.contains(&owner) {
                        true
                    } else if theirs.contains(&owner) {
                        false
                    } else {
                        return None;
                    };
                    Some(Wreck {
                        state: st,
                        ours: side,
                        share: k.share,
                        owed: f(k.owed),
                        civilians: k.civilians,
                    })
                })
                .collect();
            let keep = mil.map_or(977, |m| m.rules.nuclear.fallout_keep).min(999);
            let year = laplace_schelling::calendar::Date::from_days(days).year;
            // As `nuclear_use::fallout_year` decays it: each January, `keep` per mille is left.
            let half_years = {
                let (mut l, mut n) = (1_000_000u64, 0u32);
                while l > 500_000 && n < 1000 {
                    l = l * u64::from(keep) / 1000;
                    n += 1;
                }
                n
            };
            let under_1pc = |per_mille: u32| {
                let (mut l, mut n) = (per_mille, 0i32);
                while l >= 10 && n < 1000 {
                    l = l * keep / 1000;
                    n += 1;
                }
                year + n
            };
            let mut fallout: Vec<Fallout> = world
                .war
                .nuclear
                .fallout
                .iter()
                .filter_map(|&(r, per_mille)| {
                    let (st, owner) = state(r)?;
                    let side = if ours.contains(&owner) {
                        true
                    } else if theirs.contains(&owner) {
                        false
                    } else {
                        return None;
                    };
                    Some(Fallout {
                        state: st,
                        ours: side,
                        per_mille,
                        half_years,
                        under_1pc: under_1pc(per_mille),
                    })
                })
                .collect();
            fallout.sort_by(|a, b| b.per_mille.cmp(&a.per_mille).then(a.state.cmp(&b.state)));
            let rules = setup.conduct.as_ref().map(|c| &c.rules.support);
            let support = [
                support_line(world, politics, rules, me, tag.to_owned()),
                theirs
                    .first()
                    .and_then(|e| support_line(world, politics, rules, *e, tag_of(*e))),
            ];
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
                support,
                occupied,
                garrisons,
                wrecks,
                fallout,
                nuclear: mil.and_then(|m| {
                    theirs.first().and_then(|&e| {
                        let mut p = crate::nuclear_view::read(world, m, me, e)?;
                        p.strike = crate::nuclear_view::strike_view(world, m, me, e, days);
                        Some(p)
                    })
                }),
                expedition: world
                    .war
                    .colonial
                    .iter()
                    .find(|k| k.war == w.id && k.owner == me && !k.peace)
                    .map(|k| Expedition {
                        colony: tag_of(k.colony),
                        landed: k.landed,
                        goal: k.goal,
                        cap: k.cap,
                        wanted: k.wanted,
                        tied: k.tied,
                        share: k.commitment,
                        armed: k.armed,
                        by_land: k.by_land,
                        cut: k.cut.map(|(a, b, z)| (a, b, zone_key(z))),
                    }),
            })
        })
        .collect();
    // How near the player's government is to falling by occupation.
    let overrun = (!wars.is_empty())
        .then(|| {
            let rules = &setup.conduct.as_ref()?.rules.support;
            let capital = setup.capitals.get(tag).copied();
            let (mut all, mut taken) = (0i64, 0i64);
            let (mut capital_taken, mut colony_held) = (false, false);
            for (ri, r) in world.econ.regions.iter().enumerate() {
                if r.market != me {
                    continue;
                }
                let lost = fighting.controller(ri as u32, me) != me;
                if r.colony {
                    colony_held |= !lost;
                    continue;
                }
                all += r.population();
                // G9: the people under the enemy as the support reads them, in proportion to
                // the advance before the state changes hands.
                taken += bf_sim::war::support::under_enemy(
                    rules,
                    fighting,
                    &setup.depth_km,
                    r,
                    ri as u32,
                    lost,
                );
                if lost {
                    capital_taken |= Some(r.state) == capital;
                }
            }
            let share = if all > 0 { taken * 1000 / all } else { 0 };
            let ally_fights = world.war.conflicts.wars.iter().any(|w| {
                let side = if w.side_a.contains(&me) {
                    &w.side_a
                } else if w.side_b.contains(&me) {
                    &w.side_b
                } else {
                    return false;
                };
                side.iter().any(|&x| x != me)
            });
            Some(Overrun {
                share,
                capital_taken,
                threshold: if capital_taken {
                    rules.capital_overrun
                } else {
                    rules.overrun
                },
                capital_threshold: rules.capital_overrun,
                plain_threshold: rules.overrun,
                colony_held,
                ally_fights,
            })
        })
        .flatten();
    // What two countries traded in the last full year (or the year so far), both ways.
    let traded = |a: u16, b: u16| {
        let t = &world.econ.trade;
        let n = usize::from(t.n);
        let any = t.last_year.iter().any(|x| x.is_positive());
        let flows = if any { &t.last_year } else { &t.year };
        let at = |s: usize, d: usize| flows.get(s * n + d).map_or(0.0, |x| f(*x));
        let (a, b) = (usize::from(a), usize::from(b));
        if a < n && b < n {
            at(a, b) + at(b, a)
        } else {
            0.0
        }
    };
    Some(WarView {
        me: tag.to_owned(),
        wars,
        losses: lost_of(me),
        thresholds: setup
            .conduct
            .as_ref()
            .map(|c| (c.rules.support.peace, c.rules.support.collapse)),
        revolution_weeks: setup
            .conduct
            .as_ref()
            .map_or(0, |c| c.rules.support.revolution_weeks),
        full_line_men_km: front.map_or(0.0, |r| f(r.full_line_men_km)),
        dig_weeks,
        signature: signature(world, days),
        overrun,
        cut_ours: {
            let mut v: Vec<(String, f64)> = world
                .war
                .sea
                .cut
                .iter()
                .filter_map(|&(x, y)| {
                    let other = if x == me {
                        y
                    } else if y == me {
                        x
                    } else {
                        return None;
                    };
                    Some((tag_of(other), traded(me, other)))
                })
                .collect();
            v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            v
        },
        fleet: {
            let sea = &world.war.sea;
            sea.fleets.iter().find(|f| f.country == me).map(|f| {
                let blockades: Vec<u16> = sea
                    .blockades
                    .iter()
                    .filter(|(a, _)| *a == me)
                    .map(|(_, b)| *b)
                    .collect();
                // The strongest enemy at sea, for the reason of staying home.
                let stronger = (f.against.is_none())
                    .then(|| {
                        sea.fleets
                            .iter()
                            .filter(|x| world.war.conflicts.at_war(me, x.country))
                            .max_by_key(|x| (x.ships, std::cmp::Reverse(x.country)))
                            .map(|x| tag_of(x.country))
                    })
                    .flatten();
                FleetLine {
                    zone: zone_key(f.zone),
                    ships: f.ships,
                    against: f.against.map(tag_of),
                    stronger,
                    holds: sea.holder(f.zone) == Some(me),
                    cut: sea
                        .cut
                        .iter()
                        .filter(|(x, y)| blockades.contains(x) || blockades.contains(y))
                        .count(),
                    pairs: {
                        let mut v: Vec<(String, String, f64)> = sea
                            .cut
                            .iter()
                            .filter(|(x, y)| blockades.contains(x) || blockades.contains(y))
                            .map(|&(x, y)| {
                                // The blockaded side first.
                                let (a, b) = if blockades.contains(&x) {
                                    (x, y)
                                } else {
                                    (y, x)
                                };
                                (tag_of(a), tag_of(b), traded(a, b))
                            })
                            .collect();
                        v.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.1.cmp(&b.1)));
                        v
                    },
                    blockades: blockades.into_iter().map(tag_of).collect(),
                }
            })
        },
    })
}

/// Who holds what in every war, for the map's war lens: the countries at war and the states
/// held by another than their owner (`fighting.control`).
#[derive(Clone, Debug, PartialEq)]
pub struct ControlView {
    pub at_war: Vec<String>,
    /// (map state id, occupier's tag), by state.
    pub occupied: Vec<(u16, String)>,
}

impl ControlView {
    pub fn occupier(&self, state_id: u16) -> Option<&str> {
        self.occupied
            .binary_search_by_key(&state_id, |(s, _)| *s)
            .ok()
            .map(|i| self.occupied[i].1.as_str())
    }
}

/// The control of every war; `None` with no war.
pub fn read_control(world: &GameWorld) -> Option<ControlView> {
    let wars = &world.war.conflicts.wars;
    if wars.is_empty() {
        return None;
    }
    let tag_of = |i: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(i))
            .map(|(_, c)| c.tag.clone())
    };
    let mut at_war: Vec<String> = wars
        .iter()
        .flat_map(|w| w.side_a.iter().chain(&w.side_b))
        .filter_map(|i| tag_of(*i))
        .collect();
    at_war.sort();
    at_war.dedup();
    let mut occupied: Vec<(u16, String)> = world
        .war
        .fighting
        .control
        .iter()
        .filter_map(|(r, by)| {
            let state = world.econ.regions.get(*r as usize)?.state;
            Some((state, tag_of(*by)?))
        })
        .collect();
    occupied.sort();
    occupied.dedup_by_key(|(s, _)| *s);
    Some(ControlView { at_war, occupied })
}
