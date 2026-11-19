//! The population panel of the HUD: the player's age pyramid (whole or by state), its rates,
//! why women have as many children as they do, and where its people came from and went.
//! Everything comes from [`crate::population::PopulationView`], read once a month.

use bf_i18n::{Arg, I18n};
use bf_sim::demography::AGE_GROUPS;
use bf_ui_kit::theme::{sans, sans_medium};
use bf_ui_kit::tokens::{color, size, space};
use egui::{Align2, Rect, RichText, Sense, Ui, vec2};

use crate::keys;
use crate::population::{
    ClassTfr, CommunityLine, Part, PopulationView, Push, Road, WaveEnd, WaveLine,
};
use crate::text::country_name;
use crate::world::World;

/// Width the panel asks for.
pub const WIDTH: f32 = 440.0;

/// Below this width the population panel and the country panel do not fit side by side: the
/// one opened last shows.
pub const SIDE_BY_SIDE: f32 = 1200.0;

/// What the panel keeps between frames: the state whose pyramid shows (`None`: the country).
#[derive(Clone, Copy, Debug, Default)]
pub struct Scope(pub Option<u16>);

const ROW: f32 = 13.0;
const LABEL: f32 = 46.0;
const SHARE: f32 = 52.0;

pub fn show(
    ui: &mut Ui,
    rect: Rect,
    i18n: &I18n,
    world: &World,
    view: &PopulationView,
    scope: &mut Scope,
) {
    super::frame(ui, rect, "population");
    ui.interact(
        rect,
        ui.make_persistent_id("population-panel"),
        Sense::click_and_drag(),
    );
    super::region(ui, rect.shrink(space::LG), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("population-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| body(ui, i18n, world, view, scope));
    });
}

fn body(ui: &mut Ui, i18n: &I18n, world: &World, view: &PopulationView, scope: &mut Scope) {
    ui.spacing_mut().item_spacing.y = space::SM;
    bf_ui_kit::title(ui, &i18n.tr("population-title"));
    bf_ui_kit::muted(
        ui,
        &i18n.tr_args(
            "population-subtitle",
            &[
                ("country", Arg::String(country_name(i18n, &view.tag))),
                ("month", Arg::String(month(i18n, view.month))),
            ],
        ),
    );
    bf_ui_kit::divider(ui);

    pyramid_section(ui, i18n, world, view, scope);
    class_section(ui, i18n, world, view, *scope);
    community_section(ui, i18n, world, view, *scope);
    schooling_section(ui, i18n, world, view, *scope);
    bf_ui_kit::divider(ui);
    for part in &view.parts {
        rates_section(ui, i18n, part, view.parts.len() > 1);
        fertility_section(ui, i18n, part);
        bf_ui_kit::divider(ui);
    }
    migration_section(ui, i18n, view);
}

fn month(i18n: &I18n, (year, month): (i32, u8)) -> String {
    i18n.tr_args(
        "population-month",
        &[
            ("month", Arg::String(i18n.tr(&format!("month-{month}")))),
            ("year", Arg::String(year.to_string())),
        ],
    )
}

fn state_name(i18n: &I18n, world: &World, state: u16) -> String {
    world
        .states
        .get(usize::from(state).saturating_sub(1))
        .map_or_else(|| "—".into(), |s| crate::text::state_name(i18n, &s.def))
}

fn pyramid_section(
    ui: &mut Ui,
    i18n: &I18n,
    world: &World,
    view: &PopulationView,
    scope: &mut Scope,
) {
    bf_ui_kit::heading(ui, &i18n.tr("population-pyramid"));
    // States by people, largest first; the stepper walks the country, then each state.
    let mut states: Vec<&(u16, [i64; AGE_GROUPS])> = view.states.iter().collect();
    states.sort_by_key(|(s, a)| (-a.iter().sum::<i64>(), *s));
    let mut at = scope
        .0
        .and_then(|s| states.iter().position(|(x, _)| *x == s))
        .map_or(0, |i| i + 1);
    if scope.0.is_some() && at == 0 {
        scope.0 = None;
    }
    let steps = states.len() + 1;
    let name = match at {
        0 => i18n.tr("population-whole-country"),
        i => state_name(i18n, world, states[i - 1].0),
    };
    ui.horizontal(|ui| {
        let side = bf_ui_kit::tokens::control::HEIGHT_SM;
        let back = ui.add_sized([side, side], egui::Button::new("‹"));
        let back_typed = keys::item_beside(
            ui,
            'Z',
            "population.state.back".into(),
            "B",
            back.rect,
            keys::Side::Left,
        );
        ui.add_sized(
            [
                ui.available_width() - side - ui.spacing().item_spacing.x,
                side,
            ],
            egui::Label::new(
                RichText::new(&name)
                    .font(sans_medium(size::BODY))
                    .color(color::TEXT),
            )
            .truncate(),
        );
        let next = ui.add_sized([side, side], egui::Button::new("›"));
        let next_typed = keys::item_beside(
            ui,
            'Z',
            "population.state.next".into(),
            "N",
            next.rect,
            keys::Side::Right,
        );
        if back.clicked() || back_typed {
            at = (at + steps - 1) % steps;
        }
        if next.clicked() || next_typed {
            at = (at + 1) % steps;
        }
    });
    scope.0 = (at > 0).then(|| states[at - 1].0);
    let ages = match at {
        0 => &view.pyramid,
        i => &states[i - 1].1,
    };
    // Coloured by schooling when there is schooling: a bar per level, from none to tertiary.
    let levels = match scope.0 {
        None => view.schooling.as_ref().map(|s| &s.by_age),
        Some(st) => view
            .state_schooling
            .iter()
            .find(|(x, _)| *x == st)
            .map(|(_, s)| &s.by_age),
    };
    pyramid(ui, i18n, ages, levels);
    if levels.is_some() {
        level_legend(ui, i18n);
    }
    bf_ui_kit::caption(ui, &i18n.tr("population-no-sexes"));
}

/// Colours of the levels of schooling in the pyramid: none, reads without school, primary,
/// secondary, tertiary (a ramp from grey to light gold).
const LEVEL_COLOURS: [egui::Color32; 5] = [
    color::BORDER_STRONG,
    color::TEXT_FAINT,
    color::ACCENT_PRESSED,
    color::ACCENT,
    color::ACCENT_HOVER,
];
const LEVEL_KEYS: [&str; 5] = [
    "population-level-none",
    "population-level-reads",
    "population-level-primary",
    "population-level-secondary",
    "population-level-tertiary",
];

fn level_legend(ui: &mut Ui, i18n: &I18n) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        for (c, k) in LEVEL_COLOURS.iter().zip(LEVEL_KEYS) {
            let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
            ui.painter().rect_filled(r, 1.0, *c);
            ui.label(
                RichText::new(i18n.tr(k))
                    .font(sans(size::CAPTION))
                    .color(color::TEXT_MUTED),
            );
        }
    });
}

/// Schooling of the country or of the state the pyramid shows: literacy, pupils by level and
/// the share of their ages, mean years, and jobs waiting for graduates.
fn schooling_section(ui: &mut Ui, i18n: &I18n, world: &World, view: &PopulationView, scope: Scope) {
    let (place, s) = match scope.0 {
        None => (i18n.tr("population-whole-country"), view.schooling.as_ref()),
        Some(st) => (
            state_name(i18n, world, st),
            view.state_schooling
                .iter()
                .find(|(x, _)| *x == st)
                .map(|(_, s)| s),
        ),
    };
    if view.schooling.is_none() {
        return;
    }
    ui.add_space(space::XS);
    ui.label(
        RichText::new(i18n.tr_args("population-schooling", &[("place", Arg::String(place))]))
            .font(sans_medium(size::BODY))
            .color(color::TEXT),
    )
    .on_hover_text(i18n.tr("population-schooling-note"));
    let Some(s) = s else {
        bf_ui_kit::muted(ui, &i18n.tr("population-no-data"));
        return;
    };
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("population-literacy"),
        &crate::text::pct_text(i18n, i18n.decimal(s.literacy * 100.0, 1)),
    );
    for (l, key) in [
        "population-pupils-primary",
        "population-pupils-secondary",
        "population-pupils-tertiary",
    ]
    .iter()
    .enumerate()
    {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr(key),
            &i18n.tr_args(
                "population-pupils-value",
                &[
                    ("pupils", Arg::String(i18n.compact(s.pupils[l] as f64))),
                    (
                        "share",
                        Arg::String(i18n.decimal(s.enrolment[l] * 100.0, 1)),
                    ),
                ],
            ),
        );
    }
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("population-mean-years"),
        &i18n.tr_args(
            "population-years",
            &[("n", Arg::String(i18n.decimal(s.mean_years, 1)))],
        ),
    );
    let missing: i64 = s.missing_graduates.iter().sum();
    if missing > 0 {
        ui.label(
            RichText::new(i18n.tr_args(
                "population-missing-graduates",
                &[
                    (
                        "primary",
                        Arg::String(i18n.compact(s.missing_graduates[0] as f64)),
                    ),
                    (
                        "secondary",
                        Arg::String(i18n.compact(s.missing_graduates[1] as f64)),
                    ),
                    (
                        "tertiary",
                        Arg::String(i18n.compact(s.missing_graduates[2] as f64)),
                    ),
                ],
            ))
            .font(sans(size::SMALL))
            .color(color::DANGER),
        );
    }
    bf_ui_kit::small(ui, &i18n.tr("population-schooling-short"));
}

/// Who lives here: the largest communities (culture and religion) of the country or of the state
/// the pyramid shows, with their share and people.
fn community_section(ui: &mut Ui, i18n: &I18n, world: &World, view: &PopulationView, scope: Scope) {
    let (place, lines): (String, &[CommunityLine]) = match scope.0 {
        None => (i18n.tr("population-whole-country"), &view.communities),
        Some(s) => (
            state_name(i18n, world, s),
            view.state_communities
                .iter()
                .find(|(x, _)| *x == s)
                .map_or(&[][..], |(_, l)| l),
        ),
    };
    ui.add_space(space::XS);
    ui.label(
        RichText::new(i18n.tr_args("population-communities", &[("place", Arg::String(place))]))
            .font(sans_medium(size::BODY))
            .color(color::TEXT),
    )
    .on_hover_text(i18n.tr("population-communities-note"));
    if lines.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("population-communities-wait"));
        return;
    }
    let width = ui.available_width() - space::MD;
    let line = 16.0;
    let (rect, _) =
        ui.allocate_exact_size(vec2(width, line * (lines.len() + 1) as f32), Sense::hover());
    let p = ui.painter_at(rect);
    // Fixed columns, right-aligned: the bar, people ("53,0 milhões" fits in 92) and share.
    let bar_w = 56.0;
    let people_x = rect.right() - bar_w - space::SM;
    let share_x = people_x - 92.0 - space::SM;
    let mid = |row: usize| rect.top() + line * row as f32 + line / 2.0;
    for (x, align, key) in [
        (rect.left(), Align2::LEFT_CENTER, "population-community"),
        (share_x, Align2::RIGHT_CENTER, "population-class-share"),
        (people_x, Align2::RIGHT_CENTER, "population-people"),
    ] {
        p.text(
            egui::pos2(x, mid(0)),
            align,
            i18n.tr(key),
            sans(size::CAPTION),
            color::TEXT_FAINT,
        );
    }
    let max = lines.iter().map(|l| l.share).fold(0.0, f64::max).max(0.001);
    for (row, l) in lines.iter().enumerate() {
        let y = mid(row + 1);
        let name = match &l.keys {
            Some((culture, religion)) => i18n.tr_args(
                "community",
                &[
                    (
                        "culture",
                        Arg::String(i18n.tr(&format!("culture-{culture}"))),
                    ),
                    (
                        "religion",
                        Arg::String(i18n.tr(&format!("religion-{religion}"))),
                    ),
                ],
            ),
            None => i18n.tr("population-communities-rest"),
        };
        // The name gives way to the numbers.
        let name_rect = Rect::from_min_max(
            egui::pos2(rect.left(), y - line / 2.0),
            egui::pos2(share_x - 48.0 - space::SM, y + line / 2.0),
        );
        ui.painter_at(name_rect).text(
            egui::pos2(rect.left(), y),
            Align2::LEFT_CENTER,
            name,
            sans(size::SMALL),
            if l.keys.is_some() {
                color::TEXT
            } else {
                color::TEXT_MUTED
            },
        );
        p.text(
            egui::pos2(share_x, y),
            Align2::RIGHT_CENTER,
            crate::text::pct_text(i18n, i18n.decimal(l.share * 100.0, 1)),
            sans_medium(size::SMALL),
            color::TEXT,
        );
        p.text(
            egui::pos2(people_x, y),
            Align2::RIGHT_CENTER,
            i18n.compact(l.people as f64),
            sans(size::SMALL),
            color::TEXT_MUTED,
        );
        let len = (bar_w * (l.share / max) as f32).max(1.0);
        p.rect_filled(
            Rect::from_min_size(egui::pos2(people_x + space::SM, y - 4.0), vec2(len, 8.0)),
            1.0,
            color::ACCENT,
        );
    }
    bf_ui_kit::small(ui, &i18n.tr("population-communities-short"));
}

/// Children per woman of each profession, of the country or of the state the pyramid shows.
fn class_section(ui: &mut Ui, i18n: &I18n, world: &World, view: &PopulationView, scope: Scope) {
    let (place, lines): (String, &[ClassTfr]) = match scope.0 {
        None => (i18n.tr("population-whole-country"), &view.classes),
        Some(s) => (
            state_name(i18n, world, s),
            view.state_classes
                .iter()
                .find(|(x, _)| *x == s)
                .map_or(&[][..], |(_, l)| l),
        ),
    };
    ui.add_space(space::XS);
    ui.label(
        RichText::new(i18n.tr_args("population-by-class", &[("place", Arg::String(place))]))
            .font(sans_medium(size::BODY))
            .color(color::TEXT),
    )
    .on_hover_text(i18n.tr("population-by-class-note"));
    if lines.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("population-by-class-wait"));
        return;
    }
    // One painted row per class: name, share, children per woman and a bar for the gradient.
    let width = ui.available_width() - space::MD;
    let line = 16.0;
    let (rect, _) =
        ui.allocate_exact_size(vec2(width, line * (lines.len() + 1) as f32), Sense::hover());
    let p = ui.painter_at(rect);
    let bar_w = 90.0;
    let tfr_x = rect.right() - bar_w - space::SM;
    // The share column is narrow: "100,0 %" fits, and the header of the children column, wider
    // than its numbers, spreads over the share's empty left side.
    let share_x = tfr_x - 110.0;
    let max = lines.iter().map(|l| l.tfr).fold(0.0, f64::max).max(0.1);
    let mid = |row: usize| rect.top() + line * row as f32 + line / 2.0;
    for (x, align, key) in [
        (rect.left(), Align2::LEFT_CENTER, "population-class"),
        (share_x, Align2::RIGHT_CENTER, "population-class-share"),
        (tfr_x, Align2::RIGHT_CENTER, "population-class-tfr"),
    ] {
        p.text(
            egui::pos2(x, mid(0)),
            align,
            i18n.tr(key),
            sans(size::CAPTION),
            color::TEXT_FAINT,
        );
    }
    for (row, l) in lines.iter().enumerate() {
        let y = mid(row + 1);
        let key = bf_sim::economy::PROFESSIONS
            .get(l.prof)
            .copied()
            .unwrap_or("?");
        p.text(
            egui::pos2(rect.left(), y),
            Align2::LEFT_CENTER,
            i18n.tr(&format!("profession-{key}")),
            sans(size::SMALL),
            color::TEXT,
        );
        p.text(
            egui::pos2(share_x, y),
            Align2::RIGHT_CENTER,
            crate::text::pct_text(i18n, i18n.decimal(l.share * 100.0, 1)),
            sans(size::SMALL),
            color::TEXT_MUTED,
        );
        p.text(
            egui::pos2(tfr_x, y),
            Align2::RIGHT_CENTER,
            i18n.decimal(l.tfr, 2),
            sans_medium(size::SMALL),
            color::TEXT,
        );
        let len = (bar_w * (l.tfr / max) as f32).max(1.0);
        p.rect_filled(
            Rect::from_min_size(egui::pos2(tfr_x + space::SM, y - 4.0), vec2(len, 8.0)),
            1.0,
            color::ACCENT,
        );
    }
    bf_ui_kit::small(ui, &i18n.tr("population-by-class-short"));
}

/// One bar per five-year group, the oldest on top, with its share of the people.
fn pyramid(
    ui: &mut Ui,
    i18n: &I18n,
    ages: &[i64; AGE_GROUPS],
    levels: Option<&[[i64; 5]; AGE_GROUPS]>,
) {
    let total: i64 = ages.iter().sum();
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, ROW * AGE_GROUPS as f32), Sense::hover());
    let p = ui.painter_at(rect);
    if total <= 0 {
        return;
    }
    let max = ages.iter().copied().max().unwrap_or(1).max(1) as f32;
    let bar_w = (width - LABEL - SHARE - space::SM * 2.0).max(10.0);
    for (g, &n) in ages.iter().enumerate() {
        let row = AGE_GROUPS - 1 - g;
        let y = rect.top() + row as f32 * ROW;
        let label = if g == AGE_GROUPS - 1 {
            format!("{}+", g * 5)
        } else {
            format!("{}–{}", g * 5, g * 5 + 4)
        };
        p.text(
            egui::pos2(rect.left() + LABEL, y + ROW / 2.0),
            Align2::RIGHT_CENTER,
            label,
            sans(size::CAPTION),
            color::TEXT_MUTED,
        );
        let x0 = rect.left() + LABEL + space::SM;
        let len = bar_w * n as f32 / max;
        match levels {
            Some(l) => {
                // Stacked by level, in proportion to the group's people.
                let row_total: i64 = l[g].iter().sum::<i64>().max(1);
                let mut x = x0;
                for (k, c) in LEVEL_COLOURS.iter().enumerate() {
                    let w = len * l[g][k] as f32 / row_total as f32;
                    if w > 0.0 {
                        p.rect_filled(
                            Rect::from_min_size(egui::pos2(x, y + 1.5), vec2(w, ROW - 3.0)),
                            0.0,
                            *c,
                        );
                    }
                    x += w;
                }
            }
            None => {
                p.rect_filled(
                    Rect::from_min_size(egui::pos2(x0, y + 1.5), vec2(len.max(1.0), ROW - 3.0)),
                    1.0,
                    color::ACCENT,
                );
            }
        }
        p.text(
            egui::pos2(rect.right(), y + ROW / 2.0),
            Align2::RIGHT_CENTER,
            crate::text::pct_text(i18n, i18n.decimal(n as f64 * 100.0 / total as f64, 1)),
            sans(size::CAPTION),
            color::TEXT_MUTED,
        );
    }
}

fn rates_section(ui: &mut Ui, i18n: &I18n, part: &Part, split: bool) {
    let head = if !split {
        i18n.tr("population-rates")
    } else if part.colony {
        i18n.tr("population-rates-colonies")
    } else {
        i18n.tr("population-rates-metropole")
    };
    bf_ui_kit::heading(ui, &head);
    let r = &part.rates;
    let per_mille = |v: f64| format!("{} ‰", i18n.decimal(v, 1));
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("population-people"),
        &i18n.compact(part.people as f64),
    );
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("population-life"),
        &i18n.tr_args(
            "population-years",
            &[("n", Arg::String(i18n.decimal(r.life, 1)))],
        ),
    );
    bf_ui_kit::stat_row(ui, &i18n.tr("population-tfr"), &i18n.decimal(r.tfr, 2));
    bf_ui_kit::stat_row(ui, &i18n.tr("population-births"), &per_mille(r.births));
    bf_ui_kit::stat_row(ui, &i18n.tr("population-deaths"), &per_mille(r.deaths));
    let net = if r.migration > 0.0 {
        format!("+{}", per_mille(r.migration))
    } else {
        per_mille(r.migration)
    };
    bf_ui_kit::stat_row(ui, &i18n.tr("population-migration"), &net);
    bf_ui_kit::caption(ui, &i18n.tr("population-rates-when"));
}

fn fertility_section(ui: &mut Ui, i18n: &I18n, part: &Part) {
    bf_ui_kit::heading(ui, &i18n.tr("population-why"));
    if part.terms.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("population-no-data"));
        return;
    }
    let score: f64 = part.terms.iter().map(|t| t.contribution).sum();
    bf_ui_kit::body(
        ui,
        &i18n.tr_args(
            "population-why-lead",
            &[
                ("tfr", Arg::String(i18n.decimal(part.rates.tfr, 2))),
                ("score", Arg::String(i18n.decimal(score, 2))),
            ],
        ),
    );
    let w = ui.available_width();
    let num = 58.0;
    egui::Grid::new(("population-terms", part.colony))
        .num_columns(4)
        .min_col_width(num)
        .spacing([space::SM, 2.0])
        .show(ui, |ui| {
            let head = |ui: &mut Ui, k: &str| {
                ui.label(
                    RichText::new(i18n.tr(k))
                        .font(sans(size::CAPTION))
                        .color(color::TEXT_FAINT),
                );
            };
            ui.allocate_ui(vec2(w - num * 3.0 - space::SM * 3.0, 0.0), |ui| {
                head(ui, "population-term");
            });
            head(ui, "population-value");
            head(ui, "population-weight");
            head(ui, "population-contribution");
            ui.end_row();
            for t in &part.terms {
                ui.label(
                    RichText::new(i18n.tr(&format!("population-term-{}", t.key.replace('_', "-"))))
                        .font(sans_medium(size::SMALL))
                        .color(color::TEXT),
                );
                for v in [t.value, t.weight, t.contribution] {
                    ui.label(
                        RichText::new(i18n.decimal(v, 2))
                            .font(sans(size::SMALL))
                            .color(color::TEXT),
                    );
                }
                ui.end_row();
            }
            // Causes the simulation weighs but does not show, and causes it does not model yet.
            for (key, why) in [
                ("population-term-land", "population-hidden"),
                ("population-term-class", "population-by-class-above"),
                ("population-term-religion", "population-no-net-measure"),
            ] {
                let faint = |ui: &mut Ui, s: String| {
                    ui.label(
                        RichText::new(s)
                            .font(sans(size::SMALL))
                            .color(color::TEXT_FAINT),
                    );
                };
                faint(ui, i18n.tr(key));
                faint(ui, "—".into());
                faint(ui, "—".into());
                faint(ui, i18n.tr(why));
                ui.end_row();
            }
        });
    // The cause of each term, in a line.
    ui.add_space(space::XS);
    for t in &part.terms {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                &format!("population-cause-{}", t.key.replace('_', "-")),
                &[("value", Arg::String(i18n.decimal(t.value, 2)))],
            ),
        );
    }
    bf_ui_kit::caption(ui, &i18n.tr("population-why-note"));
}

fn roads(ui: &mut Ui, i18n: &I18n, head: &str, note: &str, list: &[Road]) {
    ui.label(
        RichText::new(i18n.tr(head))
            .font(sans_medium(size::BODY))
            .color(color::TEXT),
    );
    bf_ui_kit::caption(ui, &i18n.tr(note));
    if list.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("population-none"));
        return;
    }
    let num = 96.0;
    // Room on the right for the scroll bar.
    let name_w = (ui.available_width() - num * 2.0 - space::SM * 3.0 - space::MD).max(60.0);
    egui::Grid::new(("population-roads", head))
        .num_columns(3)
        .spacing([space::SM, 2.0])
        .show(ui, |ui| {
            let cell = |ui: &mut Ui, w: f32, text: String, faint: bool, right: bool| {
                let layout = if right {
                    egui::Layout::right_to_left(egui::Align::Center)
                } else {
                    egui::Layout::left_to_right(egui::Align::Center)
                };
                ui.allocate_ui_with_layout(vec2(w, 0.0), layout, |ui| {
                    ui.set_min_width(w);
                    ui.add(
                        egui::Label::new(
                            RichText::new(text)
                                .font(sans(if faint { size::CAPTION } else { size::SMALL }))
                                .color(if faint {
                                    color::TEXT_FAINT
                                } else {
                                    color::TEXT
                                }),
                        )
                        .truncate(),
                    );
                });
            };
            cell(ui, name_w, String::new(), true, false);
            cell(ui, num, i18n.tr("population-with-descendants"), true, true);
            cell(ui, num, i18n.tr("population-arrived"), true, true);
            ui.end_row();
            for r in list {
                cell(ui, name_w, country_name(i18n, &r.tag), false, false);
                cell(ui, num, i18n.compact(r.people as f64), false, true);
                cell(ui, num, i18n.compact(r.arrived as f64), false, true);
                ui.end_row();
            }
        });
}

fn wave_line(i18n: &I18n, w: &WaveLine) -> String {
    let cause = match w.cause {
        Push::Land => "population-push-land",
        Push::Hunger => "population-push-hunger",
        Push::Poverty => "population-push-poverty",
        Push::Frontier => "population-push-frontier",
        Push::Chain => "population-push-chain",
    };
    let mut args = vec![
        ("from", Arg::String(country_name(i18n, &w.from))),
        ("to", Arg::String(country_name(i18n, &w.to))),
        ("since", Arg::String(w.since.0.to_string())),
        ("people", Arg::String(i18n.compact(w.people as f64))),
        ("cause", Arg::String(i18n.tr(cause))),
    ];
    match w.ended {
        None => i18n.tr_args("population-wave", &args),
        Some(((until, _), end)) => {
            let end = match end {
                WaveEnd::PressureEased => "population-end-eased",
                WaveEnd::DoorClosed => "population-end-closed",
                WaveEnd::Full => "population-end-full",
                WaveEnd::NoPull => "population-end-no-pull",
                WaveEnd::Border => "population-end-border",
            };
            args.push(("until", Arg::String(until.to_string())));
            args.push(("count", Arg::Num(f64::from(w.count))));
            args.push(("end", Arg::String(i18n.tr(end))));
            i18n.tr_args("population-wave-ended", &args)
        }
    }
}

fn migration_section(ui: &mut Ui, i18n: &I18n, view: &PopulationView) {
    bf_ui_kit::heading(ui, &i18n.tr("population-roads"));
    roads(
        ui,
        i18n,
        "population-immigrants",
        "population-immigrants-note",
        &view.immigrants,
    );
    ui.add_space(space::XS);
    roads(
        ui,
        i18n,
        "population-emigrants",
        "population-emigrants-note",
        &view.emigrants,
    );
    ui.add_space(space::XS);
    ui.label(
        RichText::new(i18n.tr("population-waves"))
            .font(sans_medium(size::BODY))
            .color(color::TEXT),
    );
    if view.waves.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("population-no-waves"));
    }
    for w in &view.waves {
        bf_ui_kit::small(ui, &wave_line(i18n, w));
    }
    if !view.ended.is_empty() {
        ui.label(
            RichText::new(i18n.tr("population-waves-ended"))
                .font(sans_medium(size::BODY))
                .color(color::TEXT),
        );
        for w in &view.ended {
            bf_ui_kit::small(ui, &wave_line(i18n, w));
        }
    }
}
