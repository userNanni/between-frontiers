//! The war window (phase 5, slice C; `docs/war.pt-BR.md`, "A guerra no tick"): the player's
//! wars, each with its sides, the demand at stake and the states occupied either way; one front
//! per enemy country with the player's posture (hold or attack, by key), the men and the power
//! of both sides, the losses and the advance of the last week, and each segment with the six
//! terms of each side's power, how far the front has gone into each state, its width, each
//! side's men per km and how dug in it is. Above the fronts, each side's war support (its terms,
//! its interest groups, how fast the vote and the censorship let it move, the countdown and the
//! fall), the regions occupied either way with their resistance, garrison and what they yield,
//! and the devastation. Peace goes through the negotiation table.

use bf_i18n::{Arg, I18n};
use bf_sim::game::GameCommand;
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Rect, RichText, Ui, vec2};

use super::region;
use crate::keys::{self, Tip, id};
use crate::text::{country_name, date_long};
use crate::war_view::{Fallen, Front, Occupied, Segment, SupportLine, WarLine, WarView};
use crate::world::World;

const WINDOW: egui::Vec2 = egui::vec2(980.0, 660.0);
const MIN_WINDOW: egui::Vec2 = egui::vec2(560.0, 360.0);
/// A region's devastation is listed from 1 % of its buildings or 100 civilians killed, at most
/// five; the rest are counted.
const WRECK_FLOOR_SHARE: i32 = 10;
const WRECK_FLOOR_CIVILIANS: u64 = 100;
const WRECKS_SHOWN: usize = 5;

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
    // The war's support decides the peace (G4): the first block, each side's.
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::heading(ui, &i18n.tr("war-support"));
    if w.support.iter().all(Option::is_none) {
        bf_ui_kit::small(ui, &i18n.tr("war-support-soon"));
    } else {
        let wide = ui.available_width() >= 680.0;
        // The player's side first, framed; the enemy leader's beside it.
        let side = |ui: &mut Ui, s: &Option<SupportLine>, mine: bool| {
            if let Some(s) = s {
                support(ui, i18n, v, s, mine);
            }
        };
        if wide {
            bf_ui_kit::columns(ui, 2, |cols| {
                side(&mut cols[0], &w.support[0], true);
                side(&mut cols[1], &w.support[1], false);
            });
        } else {
            side(ui, &w.support[0], true);
            side(ui, &w.support[1], false);
        }
        if let Some((peace, collapse)) = v.thresholds {
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "war-support-thresholds",
                    &[
                        ("peace", arg(per_mille(i18n, peace))),
                        ("collapse", arg(per_mille(i18n, collapse))),
                    ],
                ),
            );
        }
    }
    // Occupation: the regions held either way, with what each yields and costs.
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::heading(ui, &i18n.tr("war-occupation"));
    if w.occupied.is_empty() {
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
    } else {
        for (ours, key) in [(true, "war-taken"), (false, "war-lost")] {
            let list: Vec<&Occupied> = w.occupied.iter().filter(|o| o.ours == ours).collect();
            bf_ui_kit::caption(ui, &i18n.tr(key));
            if list.is_empty() {
                bf_ui_kit::small(ui, &i18n.tr("war-states-none"));
            }
            for o in list {
                let line = i18n.tr_args(
                    "war-occupied-line",
                    &[
                        ("state", arg(state_name(i18n, world, o.state))),
                        ("by", arg(country_name(i18n, &o.by))),
                        ("weeks", arg(o.weeks.to_string())),
                        ("count", Arg::Num(f64::from(o.weeks))),
                        ("resistance", arg(per_mille(i18n, o.resistance))),
                        ("garrison", arg(per_mille(i18n, o.garrison))),
                        ("taken", arg(crate::text::money(i18n, o.taken))),
                        ("extraction", arg(per_mille(i18n, o.extraction))),
                        ("cap", arg(per_mille(i18n, o.extraction_cap))),
                    ],
                );
                ui.add(egui::Label::new(RichText::new(line).font(sans(size::SMALL))).wrap())
                    .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("war-occupied-tip")));
            }
        }
    }
    // Devastation: each side's total, then the regions past a floor, the worst first, and how
    // many more lost less.
    let hit: Vec<&crate::war_view::Wreck> = w
        .wrecks
        .iter()
        .filter(|k| k.share > 0 || k.civilians > 0)
        .collect();
    if !hit.is_empty() {
        bf_ui_kit::caption(ui, &i18n.tr("war-devastation"));
        for ours in [true, false] {
            let side: Vec<&&crate::war_view::Wreck> =
                hit.iter().filter(|k| k.ours == ours).collect();
            if side.is_empty() {
                continue;
            }
            let civilians: u64 = side.iter().map(|k| k.civilians).sum();
            bf_ui_kit::stat_row(
                ui,
                &i18n.tr(if ours {
                    "war-devastation-ours"
                } else {
                    "war-devastation-theirs"
                }),
                &i18n.tr_args(
                    "war-devastation-total",
                    &[
                        ("civilians", arg(i18n.int(civilians as i64))),
                        ("count", Arg::Num(civilians as f64)),
                        ("regions", arg(i18n.int(side.len() as i64))),
                        ("n", Arg::Num(side.len() as f64)),
                    ],
                ),
            );
        }
        let mut worst: Vec<&&crate::war_view::Wreck> = hit
            .iter()
            .filter(|k| k.share >= WRECK_FLOOR_SHARE || k.civilians >= WRECK_FLOOR_CIVILIANS)
            .collect();
        worst.sort_by(|a, b| b.share.cmp(&a.share).then(b.civilians.cmp(&a.civilians)));
        let more = hit.len() - worst.len().min(WRECKS_SHOWN);
        for k in worst.into_iter().take(WRECKS_SHOWN) {
            let line = i18n.tr_args(
                if k.ours {
                    "war-wreck-ours"
                } else {
                    "war-wreck-theirs"
                },
                &[
                    ("state", arg(state_name(i18n, world, k.state))),
                    (
                        "share",
                        arg(super::budget::pct(i18n, f64::from(k.share) / 10.0, 1)),
                    ),
                    ("civilians", arg(i18n.int(k.civilians as i64))),
                    ("count", Arg::Num(k.civilians as f64)),
                ],
            );
            ui.add(egui::Label::new(RichText::new(line).font(sans(size::SMALL))).wrap());
        }
        if more > 0 {
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "war-devastation-more",
                    &[
                        ("n", arg(i18n.int(more as i64))),
                        ("count", Arg::Num(more as f64)),
                    ],
                ),
            );
        }
    }
    if let Some(p) = &w.nuclear {
        super::nuclear::show(ui, i18n, p);
    }
    if let Some(e) = &w.expedition {
        expedition(ui, i18n, v, w.id, e, out);
    }
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
                    segment(ui, i18n, world, v, s);
                }
            }
        }
    });
}

/// A per-mille value as a percentage.
fn per_mille(i18n: &I18n, x: i32) -> String {
    super::budget::pct(i18n, f64::from(x) / 10.0, 0)
}

/// One side's support: the value on a bar marked at the peace and the collapse, where the terms
/// pull it, the groups that weigh most, and the government's fall when it comes.
fn support(ui: &mut Ui, i18n: &I18n, v: &WarView, s: &SupportLine, mine: bool) {
    if mine {
        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, color::ACCENT))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| support_body(ui, i18n, v, s, true));
    } else {
        support_body(ui, i18n, v, s, false);
    }
}

fn support_body(ui: &mut Ui, i18n: &I18n, v: &WarView, s: &SupportLine, mine: bool) {
    let thresholds = v.thresholds;
    ui.spacing_mut().item_spacing.y = space::XS;
    ui.horizontal(|ui| {
        let name = country_name(i18n, &s.country);
        ui.strong(if mine {
            i18n.tr_args("war-support-you", &[("country", arg(name))])
        } else {
            name
        });
        ui.label(
            RichText::new(per_mille(i18n, s.value))
                .font(bf_ui_kit::theme::sans_medium(size::BODY))
                .color(match thresholds {
                    Some((_, collapse)) if s.value < collapse => color::DANGER,
                    Some((peace, _)) if s.value < peace => color::ACCENT,
                    _ => color::POSITIVE,
                }),
        );
    });
    // The bar, with a tick at each threshold.
    let width = ui.available_width().min(320.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 3.0, color::SURFACE_SUNKEN);
    let mut fill = rect;
    fill.set_width(rect.width() * (s.value.clamp(0, 1000) as f32 / 1000.0));
    p.rect_filled(fill, 3.0, color::ACCENT);
    if let Some((peace, collapse)) = thresholds {
        for (t, c) in [(peace, color::TEXT_MUTED), (collapse, color::DANGER)] {
            let x = rect.left() + rect.width() * (t as f32 / 1000.0);
            p.vline(x, rect.expand(3.0).y_range(), egui::Stroke::new(1.5, c));
        }
    }
    // Where it tends and why, each part named for the player.
    let terms: Vec<String> = s
        .terms
        .iter()
        .filter(|(_, x)| x.abs() >= 5)
        .map(|(k, x)| {
            let key = match (*k, *x < 0) {
                ("result", true) => "war-support-why-result-down".to_owned(),
                ("result", false) => "war-support-why-result-up".to_owned(),
                ("territory", true) => "war-support-why-territory-lost".to_owned(),
                ("territory", false) => "war-support-why-territory-won".to_owned(),
                (k, _) => format!("war-support-why-{k}"),
            };
            i18n.tr_args(
                "war-support-term",
                &[
                    ("term", arg(i18n.tr(&key))),
                    ("value", arg(signed_whole(*x))),
                ],
            )
        })
        .collect();
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            if s.target < s.value - 5 {
                "war-support-tends-down"
            } else if s.target > s.value + 5 {
                "war-support-tends-up"
            } else {
                "war-support-tends-still"
            },
            &[("target", arg(per_mille(i18n, s.target)))],
        ),
    );
    pieces(ui, &terms, color::TEXT);
    let groups: Vec<String> = s
        .groups
        .iter()
        .take(4)
        .map(|(key, v, clout)| {
            i18n.tr_args(
                "war-support-group",
                &[
                    ("group", arg(i18n.tr(&format!("ig-{key}")))),
                    ("value", arg(per_mille(i18n, *v))),
                    ("clout", arg(per_mille(i18n, i32::from(*clout)))),
                ],
            )
        })
        .collect();
    // One group per piece, so a line breaks between groups and never inside one's numbers.
    pieces(ui, &groups, color::TEXT);
    // How fast it moves, in weeks to go half the way; the vote and the censorship in words.
    if s.speed > 0 {
        let half = (0.5f64.ln() / (1.0 - s.speed as f64 / 1000.0).ln())
            .ceil()
            .max(1.0);
        let mut parts = vec![i18n.tr_args(
            "war-support-pace",
            &[
                ("weeks", arg(i18n.int(half as i64))),
                ("count", Arg::Num(half)),
            ],
        )];
        if s.censorship >= 300 {
            parts.push(i18n.tr(if s.target < s.value {
                "war-support-censored-fall"
            } else {
                "war-support-censored"
            }));
        }
        if s.franchise >= 500 {
            parts.push(i18n.tr("war-support-vote-wide"));
        } else if s.franchise < 200 {
            parts.push(i18n.tr("war-support-vote-narrow"));
        }
        pieces(ui, &parts, color::TEXT_MUTED)
            .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("war-support-speed-tip")));
    }
    // What each threshold means for the player (owner's decision A3: the countdown runs for the
    // player too).
    if mine && let Some((peace, collapse)) = thresholds {
        ui.add(
            egui::Label::new(
                RichText::new(i18n.tr_args(
                    "war-support-you-thresholds",
                    &[
                        ("peace", arg(per_mille(i18n, peace))),
                        ("collapse", arg(per_mille(i18n, collapse))),
                        ("weeks", arg(v.revolution_weeks.to_string())),
                        ("count", Arg::Num(f64::from(v.revolution_weeks))),
                    ],
                ))
                .font(sans(size::SMALL))
                .color(color::ACCENT),
            )
            .halign(egui::Align::LEFT)
            .wrap(),
        );
    }
    if let Some(weeks) = s.countdown {
        let text = i18n.tr_args(
            "war-support-countdown",
            &[
                ("weeks", arg(weeks.to_string())),
                ("count", Arg::Num(f64::from(weeks))),
            ],
        );
        ui.add(
            egui::Label::new(
                RichText::new(text)
                    .font(sans(size::SMALL))
                    .color(color::DANGER),
            )
            .wrap(),
        );
    }
    if let Some(f) = &s.fallen {
        let text = match f {
            Fallen::Regime { to, .. } => i18n.tr_args(
                "war-support-fallen-regime",
                &[("form", arg(i18n.tr(&format!("law-{to}"))))],
            ),
            Fallen::Cabinet => i18n.tr("war-support-fallen-cabinet"),
        };
        ui.add(
            egui::Label::new(
                RichText::new(text)
                    .font(sans(size::SMALL))
                    .color(color::DANGER),
            )
            .wrap(),
        );
    }
}

/// Short texts joined by " · " that wrap only between them, never inside one.
fn pieces(ui: &mut Ui, parts: &[String], tint: egui::Color32) -> egui::Response {
    // The row height comes from the spacing the wrapped layout starts with: set it before.
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
        ui.spacing_mut().interact_size.y = 0.0;
        ui.horizontal_wrapped(|ui| pieces_in(ui, parts, tint))
    })
    .response
}

fn pieces_in(ui: &mut Ui, parts: &[String], tint: egui::Color32) {
    {
        let last = parts.len().saturating_sub(1);
        for (i, part) in parts.iter().enumerate() {
            let text = if i < last {
                format!("{part} · ")
            } else {
                part.clone()
            };
            ui.add(
                egui::Label::new(RichText::new(text).font(sans(size::SMALL)).color(tint))
                    .wrap_mode(egui::TextWrapMode::Extend),
            );
        }
    }
}

/// Per-mille points with a sign, as whole percentage points ("+40", "−9").
fn signed_whole(x: i32) -> String {
    let n = (f64::from(x) / 10.0).round() as i64;
    if n < 0 {
        format!("−{}", -n)
    } else {
        format!("+{n}")
    }
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

fn segment(ui: &mut Ui, i18n: &I18n, world: Option<&World>, v: &WarView, s: &Segment) {
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
    // The pace of the segment: how thick each side stands on its width, and how dug in it is.
    if let Some([ours, theirs]) = s.density() {
        ui.add(
            egui::Label::new(
                RichText::new(i18n.tr_args(
                    "war-segment-density",
                    &[
                        ("width", arg(i18n.int(s.width_km.round() as i64))),
                        ("ours", arg(i18n.int(ours.round() as i64))),
                        ("theirs", arg(i18n.int(theirs.round() as i64))),
                        ("full", arg(i18n.int(v.full_line_men_km.round() as i64))),
                    ],
                ))
                .font(sans(size::SMALL))
                .color(color::TEXT_MUTED),
            )
            .wrap(),
        )
        .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("war-segment-density-tip")));
    }
    if v.dig_weeks > 0 && s.still_weeks > 0 {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "war-segment-dug",
                &[
                    ("share", arg(super::budget::pct(i18n, s.dug * 100.0, 0))),
                    ("weeks", Arg::Num(f64::from(s.still_weeks))),
                    ("full", Arg::Num(f64::from(v.dig_weeks))),
                ],
            ),
        );
        super::journal::bar(ui, s.dug as f32, color::TEXT_MUTED, width);
    }
}

/// Men a step of the expedition's order moves.
const EXPEDITION_STEP: u64 = 10_000;

/// The expedition to a colony in revolt (G6): men landed toward the goal, the cap, the men tied
/// there, and the order: fewer, more, or back to the AI's rule (the cover of O3).
fn expedition(
    ui: &mut Ui,
    i18n: &I18n,
    v: &WarView,
    war: u32,
    e: &crate::war_view::Expedition,
    out: &mut Response,
) {
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "war-expedition",
            &[("country", arg(country_name(i18n, &e.colony)))],
        ),
    );
    let int = |x: u64| i18n.int(x as i64);
    if e.by_land {
        bf_ui_kit::small(ui, &i18n.tr("war-expedition-by-land"));
    } else {
        bf_ui_kit::stat_row(ui, &i18n.tr("war-expedition-landed"), &int(e.landed));
        bf_ui_kit::stat_row(ui, &i18n.tr("war-expedition-goal"), &int(e.goal));
        bf_ui_kit::stat_row(ui, &i18n.tr("war-expedition-cap"), &int(e.cap));
        bf_ui_kit::small(
            ui,
            &i18n.tr(if e.wanted.is_some() {
                "war-expedition-ordered"
            } else {
                "war-expedition-rule"
            }),
        );
    }
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("war-expedition-tied"),
        &i18n.tr_args(
            "war-expedition-tied-value",
            &[
                ("men", arg(int(e.tied))),
                ("share", arg(i18n.decimal(f64::from(e.share) / 10.0, 1))),
            ],
        ),
    );
    bf_ui_kit::stat_row(ui, &i18n.tr("war-expedition-armed"), &int(e.armed));
    if e.by_land {
        return;
    }
    ui.horizontal(|ui| {
        let now = e.wanted.unwrap_or(e.goal);
        let less = Button::ghost(&i18n.tr("war-expedition-less"))
            .size(ButtonSize::Small)
            .show(ui);
        if less.clicked() || keys::item(ui, 'E', "war.expedition.less".into(), "1", less.rect) {
            out.commands.push(GameCommand::SetExpedition {
                tag: v.me.clone(),
                war,
                men: Some(now.saturating_sub(EXPEDITION_STEP)),
            });
        }
        let more = Button::ghost(&i18n.tr("war-expedition-more"))
            .size(ButtonSize::Small)
            .show(ui);
        if more.clicked() || keys::item(ui, 'E', "war.expedition.more".into(), "2", more.rect) {
            out.commands.push(GameCommand::SetExpedition {
                tag: v.me.clone(),
                war,
                men: Some((now + EXPEDITION_STEP).min(e.cap)),
            });
        }
        let rule = Button::ghost(&i18n.tr("war-expedition-to-rule"))
            .size(ButtonSize::Small)
            .enabled(e.wanted.is_some())
            .show(ui);
        if (rule.clicked() || keys::item(ui, 'E', "war.expedition.rule".into(), "3", rule.rect))
            && e.wanted.is_some()
        {
            out.commands.push(GameCommand::SetExpedition {
                tag: v.me.clone(),
                war,
                men: None,
            });
        }
    });
    bf_ui_kit::small(ui, &i18n.tr("war-expedition-note"));
}
