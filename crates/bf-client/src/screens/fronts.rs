//! The fronts as objects of command (switch `fronts`, G2), a tab of the war window: for each
//! front, the share of the land army it is to have (`GameCommand::SetFrontShare`), what stands
//! on its line and what is on the way, the men per km, the posture, and the enemy state it aims
//! at (`SetFrontObjective`); the reserve off the lines (`SetReserve`), with the state it
//! reinforced; and how fast the army moves to its new places.

use bf_i18n::{Arg, I18n};
use bf_sim::game::GameCommand;
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Ui};

use super::war::Response;
use crate::keys;
use crate::text::country_name;
use crate::war_view::{FrontsView, WarView};
use crate::world::World;

/// A click of − or + moves a share by this, per mille.
const STEP: u16 = 50;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn pm(i18n: &I18n, x: u16) -> String {
    super::budget::pct(i18n, f64::from(x) / 10.0, 0)
}

/// − value +, with the key tips of the row; returns the new value when a button moved it.
fn stepper(ui: &mut Ui, i18n: &I18n, value: u16, row: &str, id: &str) -> Option<u16> {
    let mut out = None;
    ui.horizontal(|ui| {
        let down = Button::ghost("−").size(ButtonSize::Small).show(ui);
        if (down.clicked() || keys::item(ui, 'P', format!("{id}.less"), row, down.rect))
            && value > 0
        {
            out = Some(value.saturating_sub(STEP));
        }
        ui.label(RichText::new(pm(i18n, value)).font(sans(size::BODY)));
        let up = Button::ghost("+").size(ButtonSize::Small).show(ui);
        if (up.clicked() || keys::item(ui, 'Q', format!("{id}.more"), row, up.rect)) && value < 1000
        {
            out = Some((value + STEP).min(1000));
        }
    });
    out
}

fn muted(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(sans(size::SMALL))
            .color(color::TEXT_MUTED),
    );
}

pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    v: &WarView,
    f: &FrontsView,
    out: &mut Response,
) {
    ui.horizontal(|ui| {
        bf_ui_kit::heading(ui, &i18n.tr("fronts-title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            muted(
                ui,
                &i18n.tr_args("fronts-army", &[("men", arg(i18n.compact(f.army_men)))]),
            );
        });
    });
    bf_ui_kit::small(ui, &i18n.tr("fronts-basis"));
    if f.rows.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("fronts-none"));
    }
    let me = v.me.clone();
    egui::Grid::new("fronts-grid")
        .num_columns(6)
        .striped(true)
        .spacing([space::LG, space::XS])
        .show(ui, |ui| {
            for key in [
                "fronts-col-front",
                "fronts-col-share",
                "fronts-col-line",
                "fronts-col-transit",
                "fronts-col-density",
                "fronts-col-posture",
            ] {
                muted(ui, &i18n.tr(key));
            }
            ui.end_row();
            for (k, r) in f.rows.iter().enumerate() {
                let row = (k + 1).to_string();
                ui.label(country_name(i18n, &r.enemy));
                let shown = r.share.unwrap_or(r.now);
                ui.horizontal(|ui| {
                    if let Some(share) =
                        stepper(ui, i18n, shown, &row, &format!("fronts.{}", r.enemy))
                    {
                        out.commands.push(GameCommand::SetFrontShare {
                            tag: me.clone(),
                            enemy: r.enemy.clone(),
                            share,
                        });
                    }
                    if r.share.is_none() {
                        muted(ui, &i18n.tr("fronts-no-order"));
                    }
                });
                ui.label(pm(i18n, r.now));
                ui.label(if r.transit > 0 {
                    pm(i18n, r.transit)
                } else {
                    "—".into()
                });
                ui.label(
                    r.men_km
                        .map_or_else(|| "—".into(), |m| i18n.int(m.round() as i64)),
                );
                ui.label(i18n.tr(if r.attack { "war-attack" } else { "war-hold" }));
                ui.end_row();
                // The objective: a click goes to the next enemy state on the front, then to none.
                ui.label("");
                let name = |s: u16| super::war::state_name(i18n, world, s);
                let label = match r.objective {
                    Some(s) => i18n.tr_args(
                        "fronts-objective",
                        &[
                            ("state", arg(name(s))),
                            ("n", arg(r.to_objective.to_string())),
                            ("of", arg(r.segments.to_string())),
                        ],
                    ),
                    None => i18n.tr("fronts-objective-none"),
                };
                let b = Button::secondary(&label).size(ButtonSize::Small).show(ui);
                if (b.clicked()
                    || keys::item(
                        ui,
                        'O',
                        format!("fronts.{}.objective", r.enemy),
                        &row,
                        b.rect,
                    ))
                    && !r.states.is_empty()
                {
                    let next = match r
                        .objective
                        .and_then(|s| r.states.iter().position(|x| *x == s))
                    {
                        None => Some(r.states[0]),
                        Some(i) if i + 1 < r.states.len() => Some(r.states[i + 1]),
                        Some(_) => None,
                    };
                    out.commands.push(GameCommand::SetFrontObjective {
                        tag: me.clone(),
                        enemy: r.enemy.clone(),
                        state: next,
                    });
                }
                ui.end_row();
            }
            // The reserve.
            ui.label(RichText::new(i18n.tr("fronts-reserve")).strong());
            if let Some(share) = stepper(ui, i18n, f.reserve, "0", "fronts.reserve") {
                out.commands.push(GameCommand::SetReserve {
                    tag: me.clone(),
                    share,
                });
            }
            ui.label(pm(i18n, f.reserve_now));
            ui.label(match f.reinforced {
                Some(s) => i18n.tr_args(
                    "fronts-reinforced",
                    &[("state", arg(super::war::state_name(i18n, world, s)))],
                ),
                None => "—".into(),
            });
            ui.end_row();
        });
    let set: u16 = f.rows.iter().filter_map(|r| r.share).sum::<u16>() + f.reserve;
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "fronts-sum",
            &[
                ("set", arg(pm(i18n, set))),
                ("rest", arg(pm(i18n, f.rest))),
                ("transit", arg(pm(i18n, f.transit))),
            ],
        ),
    );
    // How long the slowest front takes to reach its share.
    let slowest = f
        .rows
        .iter()
        .filter(|r| r.transit > 0)
        .max_by_key(|r| r.transit);
    let pace = match slowest {
        Some(r) if f.per_week > 0 => i18n.tr_args(
            "fronts-pace-front",
            &[
                ("pace", arg(pm(i18n, f.per_week))),
                ("front", arg(country_name(i18n, &r.enemy))),
                ("share", arg(pm(i18n, r.share.unwrap_or(r.now)))),
                (
                    "weeks",
                    arg(u32::from(r.transit)
                        .div_ceil(u32::from(f.per_week))
                        .to_string()),
                ),
                (
                    "count",
                    Arg::Num(f64::from(
                        u32::from(r.transit).div_ceil(u32::from(f.per_week)),
                    )),
                ),
            ],
        ),
        _ => i18n.tr_args("fronts-pace", &[("pace", arg(pm(i18n, f.per_week)))]),
    };
    bf_ui_kit::small(ui, &pace);
    bf_ui_kit::small(ui, &i18n.tr("fronts-posture-note"));
}
