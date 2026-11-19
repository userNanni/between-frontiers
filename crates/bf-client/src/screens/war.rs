//! The war window (phase 5, slice C; `docs/war.pt-BR.md`, "A guerra no tick"): the player's
//! wars, each with its sides, the demand at stake and the states occupied either way; one front
//! per enemy country with the player's posture (hold or attack, by key), the men and the power
//! of both sides, the losses and the advance of the last week, and each segment with the six
//! terms of each side's power and how far the front has gone into each state. What the war's
//! support and the occupation's yield will show has its place, filled by the next part of the
//! simulation. Peace goes through the negotiation table.

use bf_i18n::{Arg, I18n};
use bf_sim::game::GameCommand;
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Rect, RichText, Ui, vec2};

use super::region;
use crate::keys::{self, Tip, id};
use crate::text::{country_name, date_long};
use crate::war_view::{Front, Segment, WarLine, WarView};
use crate::world::World;

const WINDOW: egui::Vec2 = egui::vec2(980.0, 660.0);
const MIN_WINDOW: egui::Vec2 = egui::vec2(560.0, 360.0);

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

#[derive(Clone, Debug, Default)]
pub struct Window {
    /// The war shown (its id); the first when none.
    pub war: Option<u32>,
    /// The fronts whose segments are open, by enemy tag.
    pub open: Vec<String>,
}

#[derive(Default)]
pub struct Response {
    pub close: bool,
    pub place_done: bool,
    pub commands: Vec<GameCommand>,
    /// Propose peace to this country at the negotiation table.
    pub peace: Option<String>,
}

fn names(i18n: &I18n, tags: &[String]) -> String {
    tags.iter()
        .map(|t| country_name(i18n, t))
        .collect::<Vec<_>>()
        .join(", ")
}

fn state_name(i18n: &I18n, world: Option<&World>, id: u16) -> String {
    world
        .and_then(|w| w.states.get(usize::from(id).checked_sub(1)?))
        .map_or_else(|| id.to_string(), |s| crate::text::state_name(i18n, &s.def))
}

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    world: Option<&World>,
    view: Option<&WarView>,
    window: &mut Window,
    place: &mut crate::hud::WindowPlace,
) -> Response {
    let mut out = Response::default();
    let top = screen.top() + super::top_bar::LEFT_BOTTOM;
    let size = WINDOW.min(vec2(
        screen.width() - 32.0,
        screen.bottom() - top - space::LG,
    ));
    let default = Rect::from_min_size(egui::pos2(screen.center().x - size.x / 2.0, top), size);
    let (rect, moved) = super::window_frame(ui, "war", default, screen, place, MIN_WINDOW);
    out.place_done |= moved;
    region(ui, rect.shrink(space::XL), |ui| {
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("war-title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = i18n.tr("gov-close");
                let tip = keys::tip(ui.ctx(), &label, id::BACK);
                let close = bf_ui_kit::close_button(ui, &label, &tip);
                if close.clicked() || keys::single(ui, "war.close", 'X', close.rect) {
                    out.close = true;
                }
            });
        });
        bf_ui_kit::divider(ui);
        let Some(v) = view else {
            bf_ui_kit::muted(ui, &i18n.tr("war-off"));
            return;
        };
        if v.wars.is_empty() {
            bf_ui_kit::muted(ui, &i18n.tr("war-none"));
            return;
        }
        // More than one war: pick it.
        if v.wars.len() > 1 {
            let options: Vec<(u32, String)> = v
                .wars
                .iter()
                .map(|w| (w.id, names(i18n, &w.theirs)))
                .collect();
            let keys_: Vec<String> = (1..=options.len()).map(|k| k.to_string()).collect();
            let tips: Vec<Tip<'_>> = options
                .iter()
                .zip(&keys_)
                .map(|((id, _), k)| Tip::Item('Z', format!("war.pick.{id}"), k))
                .collect();
            let mut picked = window.war.unwrap_or(v.wars[0].id);
            if keys::segmented(ui, &mut picked, &options, &tips) {
                window.war = Some(picked);
            }
        }
        let w = window
            .war
            .and_then(|id| v.wars.iter().find(|w| w.id == id))
            .unwrap_or(&v.wars[0]);
        egui::ScrollArea::vertical()
            .id_salt("war")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width() - space::LG);
                super::scroll_keys(ui);
                ui.spacing_mut().item_spacing.y = space::SM;
                war(ui, i18n, world, v, w, window, &mut out);
            });
    });
    out.place_done |= super::window_grip(ui, "war", rect, place, MIN_WINDOW);
    out
}

fn war(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    v: &WarView,
    w: &WarLine,
    window: &mut Window,
    out: &mut Response,
) {
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "war-sides",
            &[
                ("ours", arg(names(i18n, &w.ours))),
                ("theirs", arg(names(i18n, &w.theirs))),
            ],
        ),
    );
    let since = laplace_schelling::calendar::Date::new(w.since.0, w.since.1, w.since.2);
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            if w.declared {
                "war-since-declared"
            } else {
                "war-since-declared-on"
            },
            &[
                ("date", arg(date_long(i18n, since))),
                ("weeks", arg(w.weeks.to_string())),
                ("count", Arg::Num(w.weeks as f64)),
            ],
        ),
    );
    let at_stake: Vec<String> = w
        .demand
        .clauses
        .iter()
        .map(|c| i18n.tr(&format!("clause-{}", c.kind.key())))
        .collect();
    if !at_stake.is_empty() {
        bf_ui_kit::stat_row(ui, &i18n.tr("war-at-stake"), &at_stake.join(" · "));
    }
    // The war's support decides the peace (G4); its reading comes with the next part.
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::caption(ui, &i18n.tr("war-support"));
    bf_ui_kit::small(ui, &i18n.tr("war-support-soon"));
    // Occupation: the states held either way; what they yield comes with the next part.
    bf_ui_kit::caption(ui, &i18n.tr("war-occupation"));
    for (list, key) in [(&w.taken, "war-taken"), (&w.lost, "war-lost")] {
        let shown: Vec<String> = list.iter().map(|s| state_name(i18n, world, *s)).collect();
        let value = if shown.is_empty() {
            i18n.tr("war-states-none")
        } else {
            shown.join(", ")
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(i18n.tr(key))
                    .font(sans(size::SMALL))
                    .color(color::TEXT_MUTED),
            );
            ui.add(egui::Label::new(RichText::new(value).font(sans(size::SMALL))).wrap());
        });
    }
    bf_ui_kit::small(ui, &i18n.tr("war-occupation-soon"));
    // The fronts.
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::heading(ui, &i18n.tr("war-fronts"));
    bf_ui_kit::small(ui, &i18n.tr("war-fronts-basis"));
    for (k, f) in w.fronts.iter().enumerate() {
        front(ui, i18n, world, v, k, f, window, out);
    }
    bf_ui_kit::stat_row(ui, &i18n.tr("war-our-losses"), &i18n.int(v.losses as i64));
    // Peace, at the negotiation table, with the other side's leader.
    bf_ui_kit::gap(ui, space::SM);
    if let Some(leader) = w.theirs.first() {
        let b = Button::secondary(
            &i18n.tr_args("war-peace", &[("country", arg(country_name(i18n, leader)))]),
        )
        .size(ButtonSize::Medium)
        .show(ui);
        if b.clicked() || keys::single(ui, "war.peace", 'Q', b.rect) {
            out.peace = Some(leader.clone());
        }
        bf_ui_kit::small(ui, &i18n.tr("war-peace-note"));
    }
}

#[allow(clippy::too_many_arguments)]
fn front(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    v: &WarView,
    k: usize,
    f: &Front,
    window: &mut Window,
    out: &mut Response,
) {
    bf_ui_kit::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.horizontal(|ui| {
            ui.strong(i18n.tr_args(
                "war-front",
                &[("country", arg(country_name(i18n, &f.enemy)))],
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Hold or attack: two buttons, the one in force marked; F + the front's number
                // switches it.
                let (hold, attack) = (i18n.tr("war-hold"), i18n.tr("war-attack"));
                let a = if f.attack {
                    Button::primary(&attack)
                } else {
                    Button::ghost(&attack)
                }
                .size(ButtonSize::Small)
                .show(ui);
                let h = if f.attack {
                    Button::ghost(&hold)
                } else {
                    Button::primary(&hold)
                }
                .size(ButtonSize::Small)
                .show(ui);
                let typed = keys::item(
                    ui,
                    'F',
                    format!("war.posture.{}", f.enemy),
                    &(k + 1).to_string(),
                    a.rect.union(h.rect),
                );
                let want = if typed {
                    Some(!f.attack)
                } else if a.clicked() && !f.attack {
                    Some(true)
                } else if h.clicked() && f.attack {
                    Some(false)
                } else {
                    None
                };
                if let Some(attack) = want {
                    out.commands.push(GameCommand::SetPosture {
                        tag: v.me.clone(),
                        enemy: f.enemy.clone(),
                        attack,
                    });
                }
                bf_ui_kit::small(ui, &i18n.tr("war-posture"));
            });
        });
        if f.attack {
            bf_ui_kit::small(ui, &i18n.tr("war-attack-note"));
        } else {
            bf_ui_kit::small(ui, &i18n.tr("war-hold-note"));
        }
        if f.segments.is_empty() {
            bf_ui_kit::small(ui, &i18n.tr("war-front-none"));
        } else {
            let both = |a: String, b: String| {
                i18n.tr_args("war-both", &[("ours", arg(a)), ("theirs", arg(b))])
            };
            let int = |x: f64| i18n.int(x.round() as i64);
            bf_ui_kit::stat_row(ui, &i18n.tr("war-men"), &both(int(f.men(0)), int(f.men(1))));
            bf_ui_kit::stat_row(
                ui,
                &i18n.tr("war-power"),
                &both(i18n.decimal(f.power(0), 1), i18n.decimal(f.power(1), 1)),
            );
            bf_ui_kit::stat_row(
                ui,
                &i18n.tr("war-week-losses"),
                &both(
                    i18n.int(f.week_losses(0) as i64),
                    i18n.int(f.week_losses(1) as i64),
                ),
            );
            // The sum over the segments, not a distance on the map: said on hover.
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(i18n.tr("war-advance"))
                        .font(sans(size::BODY))
                        .color(color::TEXT_MUTED),
                )
                .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("war-advance-tip")));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(km(i18n, f.advance_km()))
                            .font(bf_ui_kit::theme::sans_medium(size::BODY))
                            .color(color::TEXT),
                    )
                    .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("war-advance-tip")));
                });
            });
        }
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr("war-enemy-losses"),
            &i18n.int(f.enemy_losses as i64),
        );
        if !f.segments.is_empty() {
            let open = window.open.contains(&f.enemy);
            let label = i18n.tr_args(
                if open {
                    "war-segments-hide"
                } else {
                    "war-segments-show"
                },
                &[
                    ("n", arg(f.segments.len().to_string())),
                    ("count", Arg::Num(f.segments.len() as f64)),
                ],
            );
            let b = Button::ghost(&label).size(ButtonSize::Small).show(ui);
            if b.clicked()
                || keys::item(
                    ui,
                    'S',
                    format!("war.segments.{}", f.enemy),
                    &(k + 1).to_string(),
                    b.rect,
                )
            {
                if open {
                    window.open.retain(|e| *e != f.enemy);
                } else {
                    window.open.push(f.enemy.clone());
                }
            }
            if open {
                for s in &f.segments {
                    segment(ui, i18n, world, s);
                }
            }
        }
    });
}

/// Kilometres with a sign: positive when the player's side advanced.
fn km(i18n: &I18n, x: f64) -> String {
    let n = i18n.decimal(x.abs(), 1);
    let key = if x > 0.05 {
        "war-km-gained"
    } else if x < -0.05 {
        "war-km-lost"
    } else {
        "war-km-none"
    };
    i18n.tr_args(key, &[("km", arg(n))])
}

fn segment(ui: &mut Ui, i18n: &I18n, world: Option<&World>, s: &Segment) {
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(i18n.tr_args(
                "war-segment",
                &[
                    ("ours", arg(state_name(i18n, world, s.ours))),
                    ("theirs", arg(state_name(i18n, world, s.theirs))),
                ],
            ))
            .font(sans(size::BODY)),
        );
        if !s.engaged {
            bf_ui_kit::small(ui, &i18n.tr("war-quiet"));
        }
    });
    // The power of each side and its six terms, on hover.
    let terms = |side: usize| {
        s.terms[side]
            .iter()
            .map(|(k, x)| {
                i18n.tr_args(
                    "war-term",
                    &[
                        ("term", arg(i18n.tr(&format!("war-term-{k}")))),
                        ("value", arg(i18n.decimal(*x, 2))),
                    ],
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let line = i18n.tr_args(
        "war-segment-line",
        &[
            ("ours", arg(i18n.decimal(s.power[0], 1))),
            ("theirs", arg(i18n.decimal(s.power[1], 1))),
            ("lost", arg(i18n.int(s.losses[0] as i64))),
            ("killed", arg(i18n.int(s.losses[1] as i64))),
            ("advance", arg(km(i18n, s.advance_km))),
        ],
    );
    let tip = format!(
        "{}\n{}\n\n{}\n{}",
        i18n.tr("war-terms-ours"),
        terms(0),
        i18n.tr("war-terms-theirs"),
        terms(1)
    );
    ui.add(egui::Label::new(RichText::new(line).font(sans(size::SMALL))).wrap())
        .on_hover_text(bf_ui_kit::tooltip_text(&tip));
    let width = ui.available_width().min(320.0);
    for (share, key, tint) in [
        (s.into_theirs, "war-into-theirs", color::ACCENT),
        (s.into_ours, "war-into-ours", color::DANGER),
    ] {
        if share <= 0.0 {
            continue;
        }
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                key,
                &[("share", arg(super::budget::pct(i18n, share * 100.0, 0)))],
            ),
        );
        super::journal::bar(ui, share as f32, tint, width);
    }
}
