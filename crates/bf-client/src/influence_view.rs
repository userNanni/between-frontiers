//! What the influence window reads: the influence the player's country holds over others and
//! others hold over it, issue by issue, and where each comes from (credit, investment, trade,
//! protection, goodwill), copied out of the simulation once a month. Shares of the GDP of the
//! country influenced, the native unit of `bf_sim::influence`.

use bf_sim::game::GameWorld;
use bf_sim::influence::{Channel, InfluenceSetup, Issue, PairReading};
use laplace_schelling::calendar::Date;

/// One channel of one issue.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelRow {
    /// Index into `ChannelKind::ALL`.
    pub kind: usize,
    /// Measured value, share of B's GDP; `None` while dormant.
    pub value: Option<f64>,
    /// The front a dormant channel waits for.
    pub front: Option<&'static str>,
    /// Weight in the issue, per mille.
    pub weight: u32,
    /// What it adds to the issue, share of B's GDP.
    pub part: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IssueRow {
    pub channels: Vec<ChannelRow>,
    pub goodwill: f64,
    pub influence: f64,
}

/// A over B in full.
#[derive(Clone, Debug, PartialEq)]
pub struct Pair {
    /// Tag of the other country (B when the player is A, A when the player is B).
    pub other: String,
    /// `Issue::ALL` order.
    pub issues: Vec<IssueRow>,
    pub regime: f64,
    pub people: f64,
    pub factor: f64,
    /// The term of a proposal, reference units, as of the last 16th.
    pub worth: Vec<f64>,
}

impl Pair {
    /// The largest issue, the one the list sorts and colours by.
    pub fn top(&self) -> f64 {
        self.issues
            .iter()
            .map(|i| i.influence)
            .fold(f64::NEG_INFINITY, f64::max)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InfluenceView {
    pub month: (i32, u8),
    pub tag: String,
    /// The player over others, largest first.
    pub over: Vec<Pair>,
    /// Others over the player, largest first.
    pub on: Vec<Pair>,
}

pub const ISSUES: usize = Issue::COUNT;

fn pair(r: &PairReading, other: String) -> Pair {
    let f = |x: laplace_schelling::Fixed| x.to_f64_lossy();
    Pair {
        other,
        issues: r
            .issues
            .iter()
            .map(|i| IssueRow {
                channels: i
                    .channels
                    .iter()
                    .map(|c| ChannelRow {
                        kind: bf_sim::influence::ChannelKind::ALL
                            .iter()
                            .position(|k| *k == c.kind)
                            .unwrap_or(0),
                        value: match c.channel {
                            Channel::Live(v) => Some(f(v)),
                            Channel::Dormant(_) => None,
                        },
                        front: match c.channel {
                            Channel::Live(_) => None,
                            Channel::Dormant(k) => Some(k),
                        },
                        weight: c.weight,
                        part: f(c.part),
                    })
                    .collect(),
                goodwill: f(i.goodwill),
                influence: f(i.influence),
            })
            .collect(),
        regime: f(r.interest.affinity.regime),
        people: f(r.interest.affinity.people),
        factor: f(r.interest.affinity.factor),
        worth: r.worth.iter().map(|w| f(*w)).collect(),
    }
}

/// Reads the influence around country `tag`; `None` without a player or without influence.
pub fn read(
    world: &GameWorld,
    setup: Option<&InfluenceSetup>,
    tag: Option<&str>,
    date: Date,
) -> Option<InfluenceView> {
    let setup = setup?;
    let tag = tag?;
    let me = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let tag_of = |i: usize| {
        world
            .countries
            .iter()
            .nth(i)
            .map_or_else(|| "?".into(), |(_, c)| c.tag.clone())
    };
    let year = date.year;
    let mut over: Vec<Pair> = bf_sim::influence::influence_over(world, setup, me, year)
        .into_iter()
        .filter(|(b, _)| usize::from(*b) != me)
        .map(|(b, _)| {
            let b = usize::from(b);
            pair(
                &bf_sim::influence::channels(world, setup, me, b, year),
                tag_of(b),
            )
        })
        .collect();
    let mut on: Vec<Pair> = bf_sim::influence::influence_on(world, setup, me, year)
        .into_iter()
        .filter(|(a, _)| usize::from(*a) != me)
        .map(|(a, _)| {
            let a = usize::from(a);
            pair(
                &bf_sim::influence::channels(world, setup, a, me, year),
                tag_of(a),
            )
        })
        .collect();
    let by_top = |a: &Pair, b: &Pair| b.top().total_cmp(&a.top()).then(a.other.cmp(&b.other));
    over.sort_by(by_top);
    on.sort_by(by_top);
    Some(InfluenceView {
        month: (date.year, date.month),
        tag: tag.to_owned(),
        over,
        on,
    })
}
