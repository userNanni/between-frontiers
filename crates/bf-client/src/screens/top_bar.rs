//! Campaign HUD at the top of the screen, over the map.
//!
//! - Top left: the player's banner with the economic rank under it, and a panel of figures: a
//!   row of headline indicators with bars, then two compact rows whose labels live in the
//!   tooltips.
//! - Top right: date and speed controls.

use bf_i18n::{Arg, I18n};
use bf_sim::indicators::{INCOME_CEILING, INCOME_FLOOR, LIFE_CEILING, LIFE_FLOOR};
use bf_ui_kit::theme::{sans, sans_semibold, serif};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Icon, icon_button, paint_icon};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::flags::{Flags, paint_flag};
use crate::keys::{self, id};
use crate::sim::Snapshot;
use crate::text::{
    country_name, date_long, difficulty_key, economy_source, money_exact, money_short, money_unit,
    social_source, status_line,
};
use crate::world::{Source, World};

const MARGIN: f32 = 12.0;
const BANNER: Vec2 = Vec2::new(84.0, 60.0);
const COL_W: f32 = 138.0;
const COLS: usize = 5;
const ROW1_H: f32 = 46.0;
const ROW_H: f32 = 26.0;
const PAD: f32 = 10.0;
const TIME_H: f32 = 56.0;

/// Rightmost screen x of the country figures on the left.
const HUD_RIGHT: f32 = MARGIN + BANNER.x - 6.0 + COL_W * COLS as f32 + PAD * 2.0 + 6.0;
/// Lowest screen y used by the HUD on the right, for panels placed under it.
pub const RIGHT_BOTTOM: f32 = MARGIN + TIME_H + MARGIN;
/// Lowest screen y used by the country figures on the left, for windows across the screen.
pub const LEFT_BOTTOM: f32 = MARGIN + ROW1_H + ROW_H * 2.0 + PAD * 2.0 + MARGIN;

#[derive(Default)]
pub struct BarResponse {
    pub open_country: bool,
    pub toggle_pause: bool,
    pub speed: Option<u8>,
    pub open_menu: bool,
    pub open_research: bool,
    pub open_government: bool,
    pub open_journal: bool,
    pub open_economy: bool,
    pub open_population: bool,
    /// A block was clicked: the screen its figure comes from.
    pub open: Option<super::tiles::Open>,
}

/// Where the two panels go: the HUD layout's rectangles; `None` hides a panel.
#[derive(Clone, Copy)]
pub struct Places {
    pub corner: Option<Rect>,
    pub time: Option<Rect>,
    /// The figures folded for want of room: the banner alone (it opens the country panel,
    /// which shows the same figures).
    pub banner_only: bool,
}

/// Size of the country figures with the banner.
pub fn corner_size(tiles: usize) -> Vec2 {
    let tiles = tiles.clamp(super::tiles::MIN, super::tiles::MAX);
    let rows = tiles.div_ceil(super::tiles::COLS);
    let cols = tiles.min(super::tiles::COLS);
    // At least as tall as the banner with its medallion.
    let height = (ROW1_H + ROW_H * (rows - 1) as f32 + PAD * 2.0).max(BANNER.y + 30.0);
    Vec2::new(
        BANNER.x - 6.0 + COL_W * cols as f32 + PAD * 2.0 + 6.0,
        height,
    )
}

/// Width of the date: what the country figures leave on the top row, up to its full width. At
/// 1280 px the long date has no room beside the figures; the short one ("2 mar. 1925") fits in
/// what is left, so the panel stays on the top row instead of dropping under the figures.
fn date_width(screen: Rect) -> f32 {
    let room = screen.width() - HUD_RIGHT - MARGIN * 2.0 - PAD * 2.0 - TIME_FIXED;
    room.clamp(DATE_MIN, 210.0)
}

/// The narrowest date: the short form at the smallest size.
const DATE_MIN: f32 = 76.0;

const TIME_BUTTON: f32 = 32.0;
const TIME_SMALL: f32 = 28.0;
const TIME_FIXED: f32 = TIME_SMALL * 2.0 + TIME_BUTTON * 7.0 + space::SM * 9.0;

/// Size of the date and speed panel on `screen`.
pub fn time_size(screen: Rect) -> Vec2 {
    Vec2::new(TIME_FIXED + date_width(screen) + PAD * 2.0, TIME_H)
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    screen: Rect,
    places: Places,
    i18n: &I18n,
    world: Option<&World>,
    flags: &Flags,
    player: Option<usize>,
    extra: Extra<'_>,
) -> BarResponse {
    let mut out = BarResponse::default();
    if let Some(r) = places.time {
        time_panel(ui, screen, r.min, i18n, extra.snap, &mut out);
    }
    if let (Some(world), Some(p), Some(r)) = (world, player, places.corner) {
        if places.banner_only {
            super::mark_frame(ui, r, "country-figures");
            banner_ui(
                ui,
                Rect::from_min_size(r.min, BANNER),
                i18n,
                world,
                flags,
                p,
                &mut out,
            );
        } else {
            corner_panel(ui, r.min, i18n, world, flags, p, &extra, &mut out);
        }
    }
    out
}

/// A tooltip in the shape: title and value, a summary, then the parts the value comes from.
struct Tip {
    title: String,
    value: String,
    summary: Vec<String>,
    breakdown: Vec<(String, String)>,
    footer: Vec<String>,
    /// The figure's closed years, for a chart under the summary.
    series: Option<Series>,
}

/// A figure year by year (`live::YearPoint`), and the lines under its chart, written.
struct Series {
    points: Vec<(i32, f64)>,
    /// "Lowest … · highest … · in the year …", and what the record counts.
    lines: Vec<String>,
    /// The least span the chart's height shows: a change of a tenth of a point does not fill
    /// it (five points for a share, a tenth of the highest value otherwise).
    floor: f64,
}

#[derive(Clone, Copy)]
enum SeriesKind {
    Count,
    Money,
    Share,
    /// Already in percent (inflation, the rate): moves in points.
    Percent,
    Years,
}

struct Item {
    icon: Icon,
    value: String,
    /// 0–1, for the headline row.
    bar: Option<f32>,
    tip: Tip,
}

/// What the blocks read besides the map's figures.
#[derive(Clone, Copy)]
pub struct Extra<'a> {
    pub snap: &'a Snapshot,
    pub tree: Option<&'a bf_sim::research::TechTree>,
    /// The blocks to show, in order.
    pub tiles: &'a [&'static str],
}

/// Blocks that read the treasury, the debt, politics, works and research.
fn extra_items(i18n: &I18n, x: &Extra<'_>) -> Vec<(&'static str, Item)> {
    use super::budget::WEEKS;
    // Money short enough for a block: "−US$ 590 mi".
    let signed_money = |_: &I18n, v: f64| {
        if v < 0.0 {
            format!("−{}", money_short(i18n, -v))
        } else {
            money_short(i18n, v)
        }
    };
    let unknown = i18n.tr("bar-unknown");
    let simple = |key: &'static str, value: String, bar: Option<f32>, summary: Vec<String>| {
        let def = super::tiles::def(key).expect("block");
        (
            key,
            Item {
                icon: def.icon,
                bar,
                tip: Tip {
                    title: i18n.tr(def.name),
                    value: value.clone(),
                    summary,
                    breakdown: Vec::new(),
                    footer: vec![i18n.tr("tile-click")],
                    series: None,
                },
                value,
            },
        )
    };
    // Percent a year, written like the other blocks' shares.
    let percent = |p: f64| pct(i18n, p / 100.0);
    let fx = |x: laplace_schelling::Fixed| x.to_f64_lossy();
    let mut out = Vec::new();
    let g = x.snap.government.as_deref();
    let gdp = g.map_or(0.0, |g| g.gdp_last_year);
    let share_of_gdp = |v: f64| {
        if gdp > 0.0 {
            pct(i18n, v / gdp)
        } else {
            unknown.clone()
        }
    };
    out.push(simple(
        "treasury",
        g.map_or(unknown.clone(), |g| signed_money(i18n, g.treasury)),
        None,
        vec![
            i18n.tr("tile-treasury-tip"),
            g.map_or(String::new(), |g| {
                i18n.tr_args(
                    "tile-treasury-reserve",
                    &[(
                        "reserve",
                        Arg::String(crate::text::money(i18n, g.reserve())),
                    )],
                )
            }),
        ],
    ));
    out.push(simple(
        "balance",
        g.map_or(unknown.clone(), |g| signed_money(i18n, g.balance() * WEEKS)),
        None,
        vec![i18n.tr("tile-balance-tip")],
    ));
    out.push(simple(
        "taxes",
        g.map_or(unknown.clone(), |g| {
            share_of_gdp(fx(g.budget.revenue) * WEEKS)
        }),
        None,
        vec![i18n.tr("tile-taxes-tip")],
    ));
    // The public debt in dollars, as the owner asked to see it; the share of GDP, who holds it
    // and the interest of a year on hover.
    let mut debt = simple(
        "debt",
        g.map_or(unknown.clone(), |g| {
            money_short(i18n, fx(g.money.total_debt()))
        }),
        None,
        vec![
            i18n.tr("tile-debt-tip"),
            g.map_or(String::new(), |g| {
                i18n.tr_args(
                    "tile-debt-total",
                    &[
                        (
                            "amount",
                            Arg::String(crate::text::money(i18n, fx(g.money.total_debt()))),
                        ),
                        ("gdp", Arg::String(share_of_gdp(fx(g.money.total_debt())))),
                    ],
                )
            }),
        ],
    );
    if let Some(g) = g {
        let m = &g.money;
        debt.1.tip.breakdown = [
            ("gov-debt-home", fx(m.debt_home)),
            ("gov-debt-bank", fx(m.debt_bank)),
            ("gov-debt-abroad", fx(m.debt_abroad)),
            ("gov-debt-fx", fx(m.debt_fx_local())),
        ]
        .into_iter()
        .filter(|(_, v)| *v > 0.0)
        .map(|(k, v)| (i18n.tr(k), money_short(i18n, v)))
        .chain(std::iter::once((
            i18n.tr("tile-debt-interest"),
            money_short(i18n, fx(g.budget.interest) * WEEKS),
        )))
        .collect();
    }
    out.push(debt);
    out.push(simple(
        "inflation",
        g.map_or(unknown.clone(), |g| percent(fx(g.money.inflation))),
        None,
        vec![i18n.tr("tile-inflation-tip")],
    ));
    out.push(simple(
        "policy-rate",
        g.map_or(unknown.clone(), |g| percent(fx(g.money.policy_rate))),
        None,
        vec![i18n.tr("tile-policy-rate-tip")],
    ));
    let laws = x.snap.laws.as_deref();
    out.push(simple(
        "legitimacy",
        laws.map_or(unknown.clone(), |l| {
            pct(i18n, f64::from(l.legitimacy) / 1000.0)
        }),
        laws.map(|l| f32::from(l.legitimacy) / 1000.0),
        vec![i18n.tr("tile-legitimacy-tip")],
    ));
    let mut movements = simple(
        "movements",
        laws.map_or(unknown.clone(), |l| i18n.int(l.movements.len() as i64)),
        None,
        vec![i18n.tr("tile-movements-tip")],
    );
    if let (Some(l), Some(cat)) = (laws, crate::laws_view::Catalogue::shared())
        && let Some(view) = cat.country(&l.tag, Some(l))
    {
        movements.1.tip.breakdown = l
            .movements
            .iter()
            .map(|(k, m)| {
                (
                    i18n.tr(&format!("movement-{}", view.movement_key(usize::from(*k)))),
                    pct(i18n, f64::from(m.support) / 1000.0),
                )
            })
            .collect();
    }
    out.push(movements);
    let journal = x.snap.journal.as_deref();
    out.push(simple(
        "works",
        journal.map_or(unknown.clone(), |j| {
            i18n.int((j.works.public + j.works.private) as i64)
        }),
        None,
        vec![journal.map_or(String::new(), |j| {
            i18n.tr_args(
                "outliner-works-count",
                &[
                    ("public", Arg::String(i18n.int(j.works.public as i64))),
                    ("private", Arg::String(i18n.int(j.works.private as i64))),
                ],
            )
        })],
    ));
    // The focus, or the technology the country is closest to on its own.
    let research = journal.and_then(|j| j.research.first());
    let tech = research
        .and_then(|r| x.tree?.techs.get(usize::from(r.tech)))
        .map(|t| i18n.tr(&format!("tech-{}", t.key)));
    out.push(simple(
        "research",
        research.map_or(unknown.clone(), |r| pct(i18n, f64::from(r.progress))),
        research.map(|r| r.progress),
        vec![tech.unwrap_or_else(|| i18n.tr("outliner-research-none"))],
    ));
    out
}

fn rank(i18n: &I18n, r: usize) -> String {
    i18n.tr_args("bar-rank", &[("rank", Arg::String(r.to_string()))])
}

fn pct(i18n: &I18n, share: f64) -> String {
    let v = share * 100.0;
    let digits = if v < 10.0 { 1 } else { 0 };
    i18n.tr_args(
        "bar-percent",
        &[("value", Arg::String(i18n.decimal(v, digits)))],
    )
}

fn years(i18n: &I18n, v: f64) -> String {
    i18n.tr_args("bar-years", &[("value", Arg::String(i18n.decimal(v, 1)))])
}

/// Figures of the country `p`: the simulation's during a campaign, read once a month.
fn items(i18n: &I18n, world: &World, p: usize) -> (Vec<Item>, Vec<Item>, Vec<Item>) {
    let c = &world.countries[p];
    let table = world.figures();
    let f = &table.countries[p];
    let live = matches!(table.source, Source::Live { .. });
    let unknown = i18n.tr("bar-unknown");
    let social = social_source(i18n, table.source);
    // Life expectancy: the simulation's last closed year once there is one (the chart's source
    // too); the 1925 data, said so, before it.
    let mut life_source = if f.life_simulated {
        vec![i18n.tr("hud-life-simulated")]
    } else if live {
        vec![i18n.tr("hud-life-scenario"), i18n.tr("source-social")]
    } else {
        vec![i18n.tr("source-social")]
    };
    if p >= world.scenario_countries && !f.life_simulated {
        life_source.insert(0, i18n.tr("hud-life-inherited"));
    }
    let economy = economy_source(i18n, table.source);
    let unit = money_unit(i18n, table.source);
    let due = i18n.tr("hud-due-to");
    let pop = f.population;
    let gdppc = f.gdp_per_capita();
    let tip = |title: &str,
               value: &str,
               summary: Vec<String>,
               breakdown: Vec<(String, String)>,
               footer: Vec<String>| Tip {
        title: i18n.tr(title),
        value: value.to_owned(),
        summary,
        breakdown,
        footer,
        series: None,
    };

    // Row 1: headline indicators with bars.
    let quality_value = f
        .quality
        .map_or(unknown.clone(), |q| i18n.decimal(q.index, 1));
    let quality_breakdown = match f.quality {
        Some(q) => vec![
            (
                format!(
                    "{} ({})",
                    i18n.tr("bar-quality-income"),
                    money_exact(i18n, gdppc)
                ),
                i18n.decimal(q.income, 2),
            ),
            (
                format!(
                    "{} ({})",
                    i18n.tr("bar-quality-health"),
                    f.life_expectancy
                        .map_or(unknown.clone(), |l| years(i18n, l))
                ),
                i18n.decimal(q.health, 2),
            ),
            (
                format!(
                    "{} ({})",
                    i18n.tr("bar-quality-education"),
                    f.literacy.map_or(unknown.clone(), |l| pct(i18n, l))
                ),
                i18n.decimal(q.education, 2),
            ),
        ],
        None => vec![],
    };
    let mut quality_summary = vec![i18n.tr("bar-quality-tip")];
    if f.quality.is_none() {
        quality_summary.push(i18n.tr("bar-quality-missing"));
    } else if live && !f.life_simulated {
        quality_summary.push(i18n.tr("hud-quality-life-scenario"));
    }
    if !quality_breakdown.is_empty() {
        quality_summary.push(due.clone());
    }
    let row1 = vec![
        Item {
            icon: Icon::Heart,
            value: quality_value.clone(),
            bar: f.quality.map(|q| (q.index / 100.0) as f32),
            tip: tip(
                "bar-quality",
                &quality_value,
                quality_summary,
                quality_breakdown,
                vec![
                    f.rank_quality.map(|r| rank(i18n, r)).unwrap_or_default(),
                    social.clone(),
                ],
            ),
        },
        {
            let v = f.literacy.map_or(unknown.clone(), |l| pct(i18n, l));
            Item {
                icon: Icon::Book,
                bar: f.literacy.map(|l| l as f32),
                tip: tip(
                    "bar-literacy",
                    &v,
                    vec![i18n.tr("bar-literacy-tip")],
                    vec![],
                    vec![social.clone()],
                ),
                value: v,
            }
        },
        {
            let v = f
                .life_expectancy
                .map_or(unknown.clone(), |l| years(i18n, l));
            Item {
                icon: Icon::Hourglass,
                bar: f.life_expectancy.map(|l| {
                    ((l - LIFE_FLOOR) / (LIFE_CEILING - LIFE_FLOOR)).clamp(0.0, 1.0) as f32
                }),
                tip: tip(
                    "bar-life",
                    &v,
                    vec![i18n.tr("bar-life-tip"), i18n.tr("hud-life-bar")],
                    vec![],
                    life_source,
                ),
                value: v,
            }
        },
        {
            let v = f.urbanization.map_or(unknown.clone(), |u| pct(i18n, u));
            Item {
                icon: Icon::City,
                bar: f.urbanization.map(|u| u as f32),
                tip: tip(
                    "bar-urban",
                    &v,
                    vec![i18n.tr("bar-urban-tip")],
                    vec![],
                    vec![social.clone()],
                ),
                value: v,
            }
        },
        {
            let v = money_exact(i18n, gdppc);
            let bar = ((gdppc.max(1.0).ln() - INCOME_FLOOR.ln())
                / (INCOME_CEILING.ln() - INCOME_FLOOR.ln()))
            .clamp(0.0, 1.0) as f32;
            Item {
                icon: Icon::Coin,
                bar: Some(bar),
                tip: tip(
                    "hud-per-capita",
                    &v,
                    vec![i18n.tr("hud-per-capita-tip"), unit.clone()],
                    vec![],
                    economy.clone(),
                ),
                value: v,
            }
        },
    ];

    // Row 2: size of the country.
    let gdp = money_short(i18n, f.gdp);
    let population = i18n.compact_short(pop);
    let has_colonies = c.has_colonies();
    let split = |metro: String, colonies: String, total: String| -> Vec<(String, String)> {
        if has_colonies {
            vec![
                (i18n.tr("hud-metropole"), metro),
                (i18n.tr("hud-colonies"), colonies),
                (i18n.tr("hud-total"), total),
            ]
        } else {
            vec![]
        }
    };
    let density = i18n.tr_args(
        "hud-density-value",
        &[(
            "value",
            Arg::String(i18n.decimal(pop / c.area_km2.max(1.0), 1)),
        )],
    );
    let area = i18n.tr_args(
        "stat-area-value",
        &[("value", Arg::String(i18n.compact_short(c.area_km2)))],
    );
    let states = i18n.int(c.states.len() as i64);
    let area_of = |v: f64| {
        i18n.tr_args(
            "stat-area-value",
            &[("value", Arg::String(i18n.compact_short(v)))],
        )
    };
    let row2 = vec![
        Item {
            icon: Icon::Chart,
            bar: None,
            tip: tip(
                "bar-gdp",
                &gdp,
                [
                    vec![i18n.tr("bar-gdp-tip"), unit.clone()],
                    if live && has_colonies {
                        vec![i18n.tr("hud-gdp-split")]
                    } else {
                        vec![]
                    },
                    vec![due.clone()],
                ]
                .concat(),
                [
                    vec![
                        (i18n.tr("bar-population"), i18n.compact(pop)),
                        (
                            i18n.tr("hud-per-capita"),
                            format!("× {}", money_exact(i18n, gdppc)),
                        ),
                    ],
                    split(
                        money_short(i18n, f.gdp),
                        money_short(i18n, f.colonial_gdp),
                        money_short(i18n, f.total_gdp()),
                    ),
                ]
                .concat(),
                [vec![rank(i18n, f.rank_gdp)], economy.clone()].concat(),
            ),
            value: gdp,
        },
        Item {
            icon: Icon::People,
            bar: None,
            tip: tip(
                "bar-population",
                &population,
                vec![i18n.tr("bar-population-tip"), i18n.int(pop as i64)],
                split(
                    i18n.compact(pop),
                    i18n.compact(f.colonial_population),
                    i18n.compact(f.total_population()),
                ),
                vec![rank(i18n, f.rank_population), economy[0].clone()],
            ),
            value: population,
        },
        Item {
            icon: Icon::Density,
            bar: None,
            tip: tip(
                "hud-density",
                &density,
                vec![i18n.tr("hud-density-tip")],
                vec![],
                vec![],
            ),
            value: density,
        },
        Item {
            icon: Icon::Map,
            bar: None,
            tip: tip(
                "hud-area",
                &area,
                vec![
                    i18n.tr("hud-area-tip"),
                    i18n.tr_args(
                        "stat-area-value",
                        &[("value", Arg::String(i18n.int(c.area_km2.round() as i64)))],
                    ),
                ],
                split(
                    area_of(c.area_km2),
                    area_of(c.colonial_area_km2),
                    area_of(c.area_km2 + c.colonial_area_km2),
                ),
                vec![],
            ),
            value: area,
        },
        Item {
            icon: Icon::Grid,
            bar: None,
            tip: tip(
                "hud-states",
                &states,
                vec![i18n.tr("hud-states-tip")],
                split(
                    i18n.int(c.states.len() as i64),
                    i18n.int(c.colonial_states.len() as i64),
                    i18n.int((c.states.len() + c.colonial_states.len()) as i64),
                ),
                vec![],
            ),
            value: states,
        },
    ];

    // Row 3: place in the world.
    let gdp_share = pct(i18n, f.total_gdp() / table.world_gdp.max(1.0));
    let pop_share = pct(i18n, f.total_population() / table.world_population.max(1.0));
    let rank_value =
        |r: usize| i18n.tr_args("hud-rank-value", &[("rank", Arg::String(r.to_string()))]);
    let quality_rank = f.rank_quality.map_or(unknown.clone(), rank_value);
    let population_rank = rank_value(f.rank_population);
    let subjects: Vec<(String, String)> = c
        .subjects
        .iter()
        .map(|&s| {
            let sc = &world.countries[s];
            (country_name(i18n, sc.tag()), status_line(i18n, world, sc))
        })
        .collect();
    let subjects_value = i18n.int(subjects.len() as i64);
    let row3 = vec![
        Item {
            icon: Icon::Pie,
            bar: None,
            tip: tip(
                "hud-gdp-share",
                &gdp_share,
                vec![
                    i18n.tr("hud-gdp-share-tip"),
                    i18n.tr("hud-includes-colonies"),
                ],
                vec![],
                [
                    vec![i18n.tr_args(
                        "hud-world-total",
                        &[("value", Arg::String(money_short(i18n, table.world_gdp)))],
                    )],
                    economy.clone(),
                ]
                .concat(),
            ),
            value: gdp_share,
        },
        Item {
            icon: Icon::Globe,
            bar: None,
            tip: tip(
                "hud-pop-share",
                &pop_share,
                vec![
                    i18n.tr("hud-pop-share-tip"),
                    i18n.tr("hud-includes-colonies"),
                ],
                vec![],
                vec![i18n.tr_args(
                    "hud-world-total",
                    &[("value", Arg::String(i18n.compact(table.world_population)))],
                )],
            ),
            value: pop_share,
        },
        Item {
            icon: Icon::Heart,
            bar: None,
            tip: tip(
                "hud-rank-quality",
                &quality_rank,
                vec![i18n.tr("bar-quality-tip")],
                vec![],
                vec![],
            ),
            value: quality_rank,
        },
        Item {
            icon: Icon::People,
            bar: None,
            tip: tip(
                "hud-rank-population",
                &population_rank,
                vec![i18n.tr("bar-population-tip")],
                vec![],
                vec![],
            ),
            value: population_rank,
        },
        Item {
            icon: Icon::Crown,
            bar: None,
            tip: tip(
                "hud-subjects",
                &subjects_value,
                vec![if subjects.is_empty() {
                    i18n.tr("hud-subjects-none")
                } else {
                    i18n.tr("hud-subjects-tip")
                }],
                subjects,
                vec![],
            ),
            value: subjects_value,
        },
    ];
    // The figures the simulation keeps year by year get their chart.
    let (mut row1, mut row2, row3) = (row1, row2, row3);
    let chart_of = |which: Figure| figure_series(i18n, table, p, which);
    row1[1].tip.series = chart_of(Figure::Literacy);
    row1[2].tip.series = chart_of(Figure::Life);
    row1[3].tip.series = chart_of(Figure::Urban);
    row1[4].tip.series = chart_of(Figure::GdpPerCapita);
    row2[0].tip.series = chart_of(Figure::Gdp);
    row2[1].tip.series = chart_of(Figure::Population);
    (row1, row2, row3)
}

#[allow(clippy::too_many_arguments)]
fn corner_panel(
    ui: &mut Ui,
    origin: Pos2,
    i18n: &I18n,
    world: &World,
    flags: &Flags,
    p: usize,
    extra: &Extra<'_>,
    out: &mut BarResponse,
) {
    let (row1, row2, row3) = items(i18n, world, p);
    // Every block by its key; the panel takes the ones chosen, in their order.
    let mut all: Vec<(&'static str, Item)> = super::tiles::FIGURES
        .iter()
        .copied()
        .zip(row1.into_iter().chain(row2).chain(row3))
        .collect();
    let mut extra_all = extra_items(i18n, extra);
    // The money blocks kept year by year get their chart too.
    let table = world.figures();
    for (key, item) in &mut extra_all {
        let which = match *key {
            "debt" => Figure::Debt,
            "treasury" => Figure::Treasury,
            "inflation" => Figure::Inflation,
            "policy-rate" => Figure::Rate,
            _ => continue,
        };
        item.tip.series = figure_series(i18n, table, p, which);
    }
    all.extend(extra_all);
    let mut chosen: Vec<(&'static str, Item)> = Vec::new();
    for key in extra.tiles {
        if let Some(i) = all.iter().position(|(k, _)| k == key) {
            chosen.push(all.swap_remove(i));
        }
    }
    let size = corner_size(chosen.len());
    let banner = Rect::from_min_size(origin, BANNER);
    let panel = Rect::from_min_size(
        Pos2::new(banner.right() - 6.0, banner.top()),
        Vec2::new(size.x - BANNER.x + 6.0, size.y),
    );

    // Panel behind the figures; it also blocks clicks from reaching the map.
    bf_ui_kit::paint_overlay(ui, panel);
    // The banner is part of it.
    super::mark_frame(
        ui,
        panel.union(Rect::from_min_size(origin, BANNER)),
        "country-figures",
    );
    ui.interact(
        panel,
        ui.make_persistent_id("hud-panel"),
        Sense::click_and_drag(),
    );
    let left = panel.left() + 6.0 + PAD;
    let rows_top = panel.top() + PAD;
    let rule = |ui: &Ui, y: f32| {
        ui.painter().hline(
            (left - 2.0)..=(panel.right() - PAD),
            y,
            Stroke::new(1.0, color::BORDER),
        );
    };

    // The first row large, with bars; the others compact.
    for (n, (key, item)) in chosen.iter().enumerate() {
        let (row, col) = (n / COLS, n % COLS);
        let cell = if row == 0 {
            Rect::from_min_size(
                Pos2::new(left + COL_W * col as f32, rows_top),
                Vec2::new(COL_W - 6.0, ROW1_H - 4.0),
            )
        } else {
            Rect::from_min_size(
                Pos2::new(
                    left + COL_W * col as f32,
                    rows_top + ROW1_H + 2.0 + ROW_H * (row - 1) as f32,
                ),
                Vec2::new(COL_W - 6.0, ROW_H - 2.0),
            )
        };
        // The figures that always had a bar keep its track even when empty.
        let track = item.bar.is_some() || super::tiles::DEFAULT[..COLS].contains(key);
        let open = super::tiles::def(key).and_then(|d| d.open);
        if cell_ui(ui, cell, item, row == 0, track, open.is_some(), key) {
            out.open = open;
        }
    }
    if chosen.len() > COLS {
        rule(ui, rows_top + ROW1_H);
    }

    banner_ui(ui, banner, i18n, world, flags, p, out);
}

/// Draws a block; true when it was clicked and it opens a screen.
fn cell_ui(
    ui: &mut Ui,
    cell: Rect,
    item: &Item,
    headline: bool,
    track: bool,
    opens: bool,
    key: &str,
) -> bool {
    let response = ui.interact(
        cell,
        ui.make_persistent_id(("hud-cell", key)),
        if opens {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let a11y = format!("{}: {}", item.tip.title, item.value);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, a11y.clone()));
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(cell.expand(2.0), radius::SM, color::SURFACE_HOVER);
    }
    let icon_size = if headline { 22.0 } else { 16.0 };
    let icon_rect = Rect::from_min_size(
        Pos2::new(
            cell.left() + 2.0,
            cell.top()
                + if headline {
                    4.0
                } else {
                    (cell.height() - 16.0) / 2.0
                },
        ),
        Vec2::splat(icon_size),
    );
    paint_icon(
        ui,
        item.icon,
        icon_rect,
        if headline {
            color::ACCENT
        } else {
            color::TEXT_MUTED
        },
    );
    let tx = icon_rect.right() + if headline { 8.0 } else { 6.0 };
    // The value at its size when it fits; smaller, then cut, when it does not.
    let room = cell.right() - tx - 2.0;
    let (mut size, floor) = if headline { (16.0, 12.0) } else { (13.0, 11.0) };
    let painter = ui.painter();
    while size > floor
        && painter
            .layout_no_wrap(item.value.clone(), sans_semibold(size), color::TEXT)
            .size()
            .x
            > room
    {
        size -= 1.0;
    }
    let (galley, _) =
        super::journal::one_line(painter, &item.value, sans_semibold(size), color::TEXT, room);
    painter.galley(
        Pos2::new(tx, icon_rect.center().y - galley.size().y / 2.0),
        galley,
        color::TEXT,
    );
    if headline && track {
        let track = Rect::from_min_size(
            Pos2::new(tx, cell.bottom() - 9.0),
            Vec2::new(cell.right() - tx - 6.0, 4.0),
        );
        painter.rect_filled(track, 2, color::SURFACE_SUNKEN);
        if let Some(f) = item.bar {
            let mut fill = track;
            fill.set_width(track.width() * f.clamp(0.0, 1.0));
            painter.rect_filled(fill, 2, bar_color(f));
        }
    }
    let response = if opens {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    };
    response
        .on_hover_ui(|ui| tooltip(ui, item.icon, &item.tip))
        .clicked()
}

/// Low values red, middle brass, high green: readable at a glance without reading the number.
fn bar_color(f: f32) -> Color32 {
    if f < 0.33 {
        color::DANGER
    } else if f < 0.66 {
        color::ACCENT
    } else {
        color::POSITIVE
    }
}

fn tooltip(ui: &mut Ui, icon: Icon, tip: &Tip) {
    ui.set_max_width(340.0);
    ui.spacing_mut().item_spacing.y = space::XS + 1.0;
    ui.spacing_mut().interact_size.y = 16.0;
    ui.horizontal(|ui| {
        bf_ui_kit::icon(ui, icon, 18.0, color::ACCENT);
        ui.label(
            egui::RichText::new(&tip.title)
                .font(sans_semibold(size::BODY))
                .color(color::TEXT),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(&tip.value)
                    .font(sans_semibold(size::BODY))
                    .color(color::ACCENT),
            );
        });
    });
    bf_ui_kit::divider(ui);
    for line in tip.summary.iter().filter(|l| !l.is_empty()) {
        ui.label(
            egui::RichText::new(line)
                .font(sans(size::SMALL))
                .color(color::TEXT),
        );
    }
    for (label, value) in &tip.breakdown {
        ui.horizontal(|ui| {
            ui.add_space(space::MD);
            ui.label(
                egui::RichText::new(label)
                    .font(sans(size::SMALL))
                    .color(color::TEXT_MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(value)
                        .font(sans_semibold(size::SMALL))
                        .color(color::ACCENT),
                );
            });
        });
    }
    if let Some(series) = &tip.series {
        chart(ui, series);
    }
    let footer: Vec<&String> = tip.footer.iter().filter(|l| !l.is_empty()).collect();
    if !footer.is_empty() {
        ui.add_space(space::XS);
        for line in footer {
            ui.label(
                egui::RichText::new(line)
                    .font(sans(size::CAPTION))
                    .color(color::TEXT_FAINT),
            );
        }
    }
}

fn banner_ui(
    ui: &mut Ui,
    banner: Rect,
    i18n: &I18n,
    world: &World,
    flags: &Flags,
    p: usize,
    out: &mut BarResponse,
) {
    let c = &world.countries[p];
    let rank_gdp = world.figures().countries[p].rank_gdp;
    let response = ui.interact(banner, ui.make_persistent_id("hud-banner"), Sense::click());
    ui.painter().add(
        egui::Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: color::SHADOW,
        }
        .as_shape(banner, radius::MD),
    );
    paint_flag(ui, banner, flags, c, color::ACCENT);
    if response.hovered() {
        ui.painter()
            .rect_filled(banner, radius::SM, Color32::from_white_alpha(16));
    }
    ui.painter().rect_stroke(
        banner,
        radius::SM,
        Stroke::new(1.5, color::ACCENT),
        egui::StrokeKind::Inside,
    );

    // Economic rank medallion under the banner.
    let center = Pos2::new(banner.center().x, banner.bottom() + 12.0);
    let medal = Rect::from_center_size(center, Vec2::splat(34.0));
    let medal_response = ui.interact(medal, ui.make_persistent_id("hud-medal"), Sense::hover());
    let painter = ui.painter();
    painter.circle(
        center,
        17.0,
        color::SURFACE_RAISED,
        Stroke::new(2.0, color::ACCENT),
    );
    painter.circle_stroke(
        center,
        13.5,
        Stroke::new(1.0, color::ACCENT.gamma_multiply(0.5)),
    );
    painter.text(
        center,
        Align2::CENTER_CENTER,
        rank_gdp.to_string(),
        serif(17.0),
        color::TEXT,
    );
    let medal_tip = format!(
        "{}\n{}",
        i18n.tr_args(
            "hud-medal-tip",
            &[("rank", Arg::String(rank_gdp.to_string()))]
        ),
        i18n.tr(difficulty_key(world, c))
    );
    medal_response.on_hover_text(bf_ui_kit::tooltip_text(&medal_tip));

    let tip = format!(
        "{}\n{}\n{}\n\n{}",
        country_name(i18n, c.tag()),
        status_line(i18n, world, c),
        i18n.tr(&format!("government-{}", c.def.government)),
        i18n.tr("bar-country-tip")
    );
    let tip = keys::tip(ui.ctx(), &tip, id::COUNTRY);
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(bf_ui_kit::tooltip_text(&tip));
    if keys::pressed(ui, id::COUNTRY, &response) {
        out.open_country = true;
    }
}

/// Date and speed, in a panel at the top right.
fn time_panel(
    ui: &mut Ui,
    screen: Rect,
    origin: Pos2,
    i18n: &I18n,
    snap: &Snapshot,
    out: &mut BarResponse,
) {
    let button = TIME_BUTTON;
    let small = TIME_SMALL;
    let date_w = date_width(screen);
    let panel = Rect::from_min_size(origin, time_size(screen));
    super::frame(ui, panel, "time");
    ui.interact(
        panel,
        ui.make_persistent_id("hud-time"),
        Sense::click_and_drag(),
    );

    let y = panel.center().y;
    let mut x = panel.left() + PAD;
    let mut next = |w: f32, h: f32| {
        let r = Rect::from_min_size(Pos2::new(x, y - h / 2.0), Vec2::new(w, h));
        x += w + space::SM;
        r
    };
    let minus_rect = next(small, small);
    let date_rect = next(date_w, TIME_H - 14.0);
    let plus_rect = next(small, small);
    let pause_rect = next(button, button);
    let government_rect = next(button, button);
    let economy_rect = next(button, button);
    let journal_rect = next(button, button);
    let population_rect = next(button, button);
    let research_rect = next(button, button);
    let menu_rect = next(button, button);

    let mut at = |r: Rect, f: &mut dyn FnMut(&mut Ui)| {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
        f(&mut child);
    };
    // Each button triggers its command; the tooltip shows the command's key.
    let command = |ui: &mut Ui, icon: Icon, side: f32, text: &str, cmd: &'static str, on: bool| {
        let tip = keys::tip(ui.ctx(), text, cmd);
        let response = icon_button(ui, icon, side, &tip, on);
        keys::badge(ui, response.rect, cmd);
        keys::pressed(ui, cmd, &response)
    };
    at(minus_rect, &mut |ui| {
        if command(
            ui,
            Icon::Minus,
            small,
            &i18n.tr("bar-speed-down"),
            id::SPEED_DOWN,
            false,
        ) {
            out.speed = Some(snap.speed.saturating_sub(1).max(1));
        }
    });
    at(plus_rect, &mut |ui| {
        if command(
            ui,
            Icon::Plus,
            small,
            &i18n.tr("bar-speed-up"),
            id::SPEED_UP,
            false,
        ) {
            out.speed = Some((snap.speed + 1).min(5));
        }
    });
    at(pause_rect, &mut |ui| {
        let (icon, tip) = if snap.paused {
            (Icon::Play, i18n.tr("bar-play"))
        } else {
            (Icon::Pause, i18n.tr("bar-pause"))
        };
        if command(ui, icon, button, &tip, id::PAUSE, snap.paused) {
            out.toggle_pause = true;
        }
    });
    at(government_rect, &mut |ui| {
        let text = i18n.tr("bar-government");
        if command(ui, Icon::Shield, button, &text, id::GOVERNMENT, false) {
            out.open_government = true;
        }
    });
    at(economy_rect, &mut |ui| {
        let text = i18n.tr("bar-economy");
        if command(ui, Icon::Coins, button, &text, id::ECONOMY, false) {
            out.open_economy = true;
        }
    });
    at(journal_rect, &mut |ui| {
        if command(
            ui,
            Icon::Book,
            button,
            &i18n.tr("bar-journal"),
            id::JOURNAL,
            false,
        ) {
            out.open_journal = true;
        }
    });
    at(population_rect, &mut |ui| {
        if command(
            ui,
            Icon::People,
            button,
            &i18n.tr("bar-population-panel"),
            id::POPULATION,
            false,
        ) {
            out.open_population = true;
        }
    });
    at(research_rect, &mut |ui| {
        let text = i18n.tr("bar-research");
        if command(ui, Icon::Flask, button, &text, id::RESEARCH, false) {
            out.open_research = true;
        }
    });
    at(menu_rect, &mut |ui| {
        if command(
            ui,
            Icon::Menu,
            button,
            &i18n.tr("bar-menu"),
            id::MENU,
            false,
        ) {
            out.open_menu = true;
        }
    });

    let response = ui.interact(date_rect, ui.make_persistent_id("hud-date"), Sense::click());
    let painter = ui.painter();
    painter.rect(
        date_rect,
        radius::MD,
        if response.hovered() {
            color::SURFACE_HOVER
        } else {
            color::SURFACE_SUNKEN
        },
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    // The full size when it fits; smaller on a narrow screen; the short form when even the
    // smallest long one does not.
    let fits = |text: &str, font: f32| {
        painter
            .layout_no_wrap(text.to_owned(), serif(font), color::TEXT)
            .size()
            .x
            <= date_w - 12.0
    };
    let mut date = date_long(i18n, snap.date);
    if !fits(&date, 13.0) {
        date = crate::text::date_short(i18n, snap.date);
    }
    let mut font = 19.0;
    while font > 13.0 && !fits(&date, font) {
        font -= 1.0;
    }
    painter.text(
        Pos2::new(date_rect.center().x, date_rect.top() + 15.0),
        Align2::CENTER_CENTER,
        date,
        serif(font),
        if snap.paused {
            color::TEXT_MUTED
        } else {
            color::TEXT
        },
    );
    if snap.paused {
        painter.text(
            Pos2::new(date_rect.center().x, date_rect.bottom() - 9.0),
            Align2::CENTER_CENTER,
            i18n.tr("game-paused").to_uppercase(),
            sans_semibold(9.5),
            color::ACCENT,
        );
    } else {
        let pip = Vec2::new(16.0, 4.0);
        let total = pip.x * 5.0 + 4.0 * 4.0;
        for i in 0..5u8 {
            let min = Pos2::new(
                date_rect.center().x - total / 2.0 + i as f32 * (pip.x + 4.0),
                date_rect.bottom() - 11.0,
            );
            let fill = if i < snap.speed {
                color::ACCENT
            } else {
                color::BORDER_STRONG
            };
            painter.rect_filled(Rect::from_min_size(min, pip), 2, fill);
        }
    }
    let tip = format!(
        "{}\n{}",
        i18n.tr_args("bar-date-tip", &[("speed", Arg::Num(snap.speed as f64))]),
        i18n.tr_args(
            "hud-engine",
            &[
                ("tick", Arg::String(i18n.int(snap.tick as i64))),
                ("hash", Arg::String(snap.hash.clone())),
            ],
        )
    );
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(bf_ui_kit::tooltip_text(&tip))
        .clicked()
    {
        out.toggle_pause = true;
    }
}

/// The chart of a figure's closed years: a line over a faint band, the first and last years under
/// it, and the lines written for it.
fn chart(ui: &mut Ui, series: &Series) {
    ui.add_space(space::XS);
    if series.points.len() >= 2 {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 56.0), egui::Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, radius::SM, color::SURFACE_SUNKEN);
        let lo = series
            .points
            .iter()
            .map(|x| x.1)
            .fold(f64::INFINITY, f64::min);
        let hi = series
            .points
            .iter()
            .map(|x| x.1)
            .fold(f64::NEG_INFINITY, f64::max);
        // At least the floor's span, centred on the values: a flat figure draws flat.
        let span = (hi - lo).max(series.floor).max(f64::EPSILON);
        let lo = (lo + hi) / 2.0 - span / 2.0;
        let (first, last) = (series.points[0].0, series.points[series.points.len() - 1].0);
        let years = f64::from((last - first).max(1));
        let inner = rect.shrink2(egui::vec2(6.0, 6.0));
        let at = |(y, v): (i32, f64)| {
            egui::pos2(
                inner.left() + inner.width() * (f64::from(y - first) / years) as f32,
                inner.bottom() - inner.height() * ((v - lo) / span) as f32,
            )
        };
        let line: Vec<egui::Pos2> = series.points.iter().map(|x| at(*x)).collect();
        p.add(egui::Shape::line(
            line.clone(),
            egui::Stroke::new(1.5, color::ACCENT),
        ));
        if let Some(end) = line.last() {
            p.circle_filled(*end, 2.5, color::ACCENT);
        }
    }
    for l in &series.lines {
        ui.label(
            egui::RichText::new(l)
                .font(sans(size::CAPTION))
                .color(color::TEXT_MUTED),
        );
    }
}

/// A figure the simulation records each year (`live::YearPoint`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Figure {
    Population,
    Gdp,
    GdpPerCapita,
    Literacy,
    Urban,
    Life,
    Debt,
    Treasury,
    Inflation,
    Rate,
}

/// The chart of a figure of country `p`, built from its closed years; `None` outside a
/// campaign. The yearly record counts the whole market: with colonies it is not the metropole
/// the blocks show, and a chart of it would mislead, so a country with colonies gets a line
/// saying so instead (until the simulation records the metropole apart).
fn figure_series(
    i18n: &I18n,
    table: &crate::world::FigureTable,
    p: usize,
    which: Figure,
) -> Option<Series> {
    if !matches!(table.source, Source::Live { .. }) {
        return None;
    }
    // A country with colonies: the metropole's own numbers, which the blocks show, from the
    // year the simulation began to keep them apart; before that its years are left out.
    let colonies = table
        .countries
        .get(p)
        .is_some_and(|f| f.colonial_population > 0.0);
    let history = table.history.get(p).map(Vec::as_slice).unwrap_or(&[]);
    type Read = fn(&crate::live::YearPoint, bool) -> Option<f64>;
    let (kind, value): (SeriesKind, Read) = match which {
        Figure::Population => (SeriesKind::Count, |h, c| {
            if c {
                h.series.map(|s| s.population_metro)
            } else {
                Some(h.population)
            }
        }),
        Figure::Gdp => (SeriesKind::Money, |h, c| {
            if c {
                h.series.map(|s| s.gdp_metro)
            } else {
                Some(h.gdp)
            }
        }),
        Figure::GdpPerCapita => (SeriesKind::Money, |h, c| {
            let (gdp, pop) = if c {
                let s = h.series?;
                (s.gdp_metro, s.population_metro)
            } else {
                (h.gdp, h.population)
            };
            (pop > 0.0).then(|| gdp / pop)
        }),
        Figure::Literacy => (SeriesKind::Share, |h, c| {
            if c {
                h.series.map(|s| s.literacy_metro)
            } else {
                Some(h.literacy)
            }
        }),
        Figure::Urban => (SeriesKind::Share, |h, c| {
            if c {
                h.series.map(|s| s.urban_metro)
            } else {
                Some(h.urban)
            }
        }),
        // The metropole's, as the block's number (`FigureTable::live`).
        Figure::Life => (SeriesKind::Years, |h, _| h.series.map(|s| s.life_metro)),
        Figure::Debt => (SeriesKind::Money, |h, _| h.series.map(|s| s.debt)),
        Figure::Treasury => (SeriesKind::Money, |h, _| h.series.map(|s| s.treasury)),
        Figure::Inflation => (SeriesKind::Percent, |h, _| h.series.map(|s| s.inflation)),
        Figure::Rate => (SeriesKind::Percent, |h, _| h.series.map(|s| s.rate)),
    };
    let write = |v: f64| match kind {
        SeriesKind::Count => i18n.compact(v),
        SeriesKind::Money => crate::text::money_short(i18n, v),
        SeriesKind::Share => pct(i18n, v),
        SeriesKind::Percent => crate::text::pct_text(i18n, i18n.decimal(v, 1)),
        SeriesKind::Years => i18n.tr_args(
            "hud-series-years",
            &[("n", Arg::String(i18n.decimal(v, 1)))],
        ),
    };
    let points: Vec<(i32, f64)> = history
        .iter()
        .filter_map(|h| Some((h.year, value(h, colonies)?)))
        .collect();
    let mut lines = Vec::new();
    if points.len() >= 2 {
        let lo = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let hi = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let (last, before) = (points[points.len() - 1].1, points[points.len() - 2].1);
        let sign = |d: f64| if d >= 0.0 { "+" } else { "−" };
        let change = match kind {
            // A share moves in points; a count or a sum in percent.
            SeriesKind::Share => {
                let d = (last - before) * 100.0;
                format!("{}{} p.p.", sign(d), i18n.decimal(d.abs(), 1))
            }
            SeriesKind::Percent => {
                let d = last - before;
                format!("{}{} p.p.", sign(d), i18n.decimal(d.abs(), 1))
            }
            SeriesKind::Years => {
                let d = last - before;
                format!("{}{}", sign(d), i18n.decimal(d.abs(), 1))
            }
            _ if before > 0.0 => {
                let d = (last / before - 1.0) * 100.0;
                format!(
                    "{}{}",
                    sign(d),
                    crate::text::pct_text(i18n, i18n.decimal(d.abs(), 1))
                )
            }
            _ => "—".into(),
        };
        lines.push(i18n.tr_args(
            "hud-series-stats",
            &[
                ("low", Arg::String(write(lo))),
                ("high", Arg::String(write(hi))),
                ("year", Arg::String(points[points.len() - 1].0.to_string())),
                ("change", Arg::String(change)),
            ],
        ));
        lines.push(i18n.tr_args(
            "hud-series-scope",
            &[
                ("from", Arg::String(points[0].0.to_string())),
                ("to", Arg::String(points[points.len() - 1].0.to_string())),
            ],
        ));
    } else {
        lines.push(i18n.tr("hud-series-few"));
    }
    if colonies
        && matches!(
            which,
            Figure::Population
                | Figure::Gdp
                | Figure::GdpPerCapita
                | Figure::Literacy
                | Figure::Urban
                | Figure::Life
        )
    {
        lines.push(i18n.tr("hud-series-metro"));
    }
    let top = points.iter().map(|p| p.1.abs()).fold(0.0, f64::max);
    let floor = match kind {
        SeriesKind::Share => 0.05,
        SeriesKind::Percent => 5.0,
        SeriesKind::Years => 5.0,
        _ => top * 0.1,
    };
    Some(Series {
        points,
        lines,
        floor,
    })
}

/// The chart of a figure, as the HUD's tips draw it, for another panel's hover (the country
/// panel's headline numbers); nothing outside a campaign.
pub fn figure_chart(ui: &mut Ui, i18n: &I18n, world: &World, p: usize, which: Figure) {
    if let Some(s) = figure_series(i18n, world.figures(), p, which) {
        ui.set_max_width(340.0);
        chart(ui, &s);
    }
}
