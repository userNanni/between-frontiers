//! The relations' measures in the country panel (switch `relations`, read only): the country's
//! power (the CINC and its six shares), rank, role, trend and threat, the territory it lost, and
//! its hostility with the player term by term — or, for the player's own country, the countries
//! most hostile to it.

use bf_i18n::{Arg, I18n};
use bf_sim::relations::{Pair, Role, Terms, Weights};
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use egui::Ui;

use crate::sim::RelationsView;
use crate::text::country_name;
use crate::world::World;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn pm(i18n: &I18n, x: f64, decimals: usize) -> String {
    super::budget::pct(i18n, x / 10.0, decimals)
}

/// The terms of a pair, the weighted ones that count, the largest first: (key, points).
fn terms(t: &Terms, w: &Weights) -> Vec<(&'static str, i32)> {
    let mut out = vec![
        ("claim", t.claim * w.claim / 1000),
        ("territory", t.territory * w.territory / 1000),
        ("minority", t.minority * w.minority / 1000),
        ("regime", t.regime * w.regime / 1000),
        ("causes", t.causes * w.causes / 1000),
        ("arms_race", t.arms_race * w.arms_race / 1000),
        ("war", t.war * w.war / 1000),
        ("trade", -(t.trade * w.trade / 1000)),
        ("agreement", -(t.agreement * w.agreement / 1000)),
        ("common_enemy", -(t.common_enemy * w.common_enemy / 1000)),
        ("deterrence", -(t.deterrence * w.deterrence / 1000)),
    ];
    out.retain(|(_, v)| *v != 0);
    out.sort_by_key(|(_, v)| -v.abs());
    out
}

fn pair_line(i18n: &I18n, a: &str, b: &str, p: &Pair, w: &Weights) -> String {
    let parts: Vec<String> = terms(&p.terms, w)
        .into_iter()
        .map(|(k, v)| {
            i18n.tr_args(
                "relations-term",
                &[
                    ("term", arg(i18n.tr(&format!("relations-term-{k}")))),
                    ("points", arg(format!("{v:+}"))),
                ],
            )
        })
        .collect();
    i18n.tr_args(
        "relations-pair",
        &[
            ("a", arg(country_name(i18n, a))),
            ("b", arg(country_name(i18n, b))),
            ("hostility", arg(p.hostility.to_string())),
            ("terms", arg(parts.join(", "))),
        ],
    )
}

pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    _world: &World,
    v: &RelationsView,
    tag: &str,
    player: Option<&str>,
) {
    let index = |t: &str| v.tags.iter().position(|x| x == t).map(|i| i as u16);
    let Some(c) = index(tag) else {
        return;
    };
    let Some(m) = v.measures.countries.iter().find(|m| m.country == c) else {
        return;
    };
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("relations-title"));
    let role = match m.role {
        Role::Leading => "relations-role-leading",
        Role::Great => "relations-role-great",
        Role::Rising => "relations-role-rising",
        Role::Declining => "relations-role-declining",
        Role::Small => "relations-role-small",
    };
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("relations-power"),
        &i18n.tr_args(
            "relations-power-value",
            &[
                ("cinc", arg(pm(i18n, f64::from(m.cinc), 1))),
                ("rank", arg(m.rank.to_string())),
            ],
        ),
    );
    bf_ui_kit::stat_row(ui, &i18n.tr("relations-role"), &i18n.tr(role));
    bf_ui_kit::stat_row(ui, &i18n.tr("relations-trend"), &{
        let sign = if m.trend > 0 { "+" } else { "" };
        format!("{sign}{}", pm(i18n, f64::from(m.trend), 1))
    });
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("relations-threat"),
        &pm(i18n, f64::from(m.threat), 0),
    );
    if m.lost_territory > 0 {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr("relations-lost"),
            &pm(i18n, f64::from(m.lost_territory), 1),
        );
    }
    // The hostility with the player, or the most hostile to the player.
    if let Some(me) = player.and_then(index) {
        let me_tag = player.unwrap_or_default();
        if me != c {
            match v.measures.pair(me, c) {
                Some(p) => {
                    bf_ui_kit::small(ui, &pair_line(i18n, me_tag, tag, p, &v.weights));
                }
                None => {
                    bf_ui_kit::small(
                        ui,
                        &i18n.tr_args(
                            "relations-pair-none",
                            &[
                                ("a", arg(country_name(i18n, me_tag))),
                                ("b", arg(country_name(i18n, tag))),
                            ],
                        ),
                    );
                }
            }
        } else {
            let mut hostile: Vec<&Pair> = v
                .measures
                .pairs
                .iter()
                .filter(|p| (p.a == me || p.b == me) && p.hostility > 0)
                .collect();
            hostile.sort_by_key(|p| -p.hostility);
            if !hostile.is_empty() {
                bf_ui_kit::caption(ui, &i18n.tr("relations-most-hostile"));
            }
            for p in hostile.into_iter().take(5) {
                let other = if p.a == me { p.b } else { p.a };
                let other = v.tags.get(usize::from(other)).map_or("", String::as_str);
                bf_ui_kit::small(ui, &pair_line(i18n, me_tag, other, p, &v.weights));
            }
        }
    }
    // The six shares of the world, as bars.
    let max = m.shares.iter().copied().max().unwrap_or(1).max(1) as f32;
    const LABEL: f32 = 140.0;
    const VALUE: f32 = 48.0;
    let width = (ui.available_width() - LABEL - VALUE - 2.0 * space::XS).max(40.0);
    for (k, share) in bf_sim::relations::COMPONENTS.iter().zip(m.shares) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::XS;
            let (label, _) = ui.allocate_exact_size(egui::vec2(LABEL, 14.0), egui::Sense::hover());
            ui.painter().text(
                label.left_center(),
                egui::Align2::LEFT_CENTER,
                i18n.tr(&format!("relations-share-{k}")),
                sans(size::SMALL),
                color::TEXT_MUTED,
            );
            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, color::SURFACE_HOVER);
            let mut fill = rect;
            fill.set_width(rect.width() * share as f32 / max);
            ui.painter().rect_filled(fill, 2.0, color::ACCENT);
            let (value, _) = ui.allocate_exact_size(egui::vec2(VALUE, 14.0), egui::Sense::hover());
            ui.painter().text(
                value.right_center(),
                egui::Align2::RIGHT_CENTER,
                pm(i18n, f64::from(share), 1),
                sans(size::SMALL),
                color::TEXT,
            );
        });
    }
}
