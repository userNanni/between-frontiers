//! Research: the technologies of one era, grouped by field, with the stage the player's country
//! has reached in each (science, prototype, industrial, diffused) and the focus of its research.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::Catalogue;
use bf_sim::research::{ERAS, FIELDS, Stage, TechTree};
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, radius, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::sim::ResearchView;

const CARD: Vec2 = Vec2::new(250.0, 92.0);

#[derive(Default)]
pub struct ResearchResponse {
    pub close: bool,
    /// The player picked a focus (`Some(None)` hands the choice back to the country).
    pub focus: Option<Option<u16>>,
}

pub fn stage_color(stage: Stage) -> Color32 {
    match stage {
        Stage::Unknown => color::TEXT_FAINT,
        Stage::Science => Color32::from_rgb(0x7A, 0x9C, 0xC6),
        Stage::Prototype => color::ACCENT,
        Stage::Industrial => color::POSITIVE,
        Stage::Diffused => Color32::from_rgb(0xA8, 0xD8, 0x8E),
    }
}

/// Era shown when the screen opens: the one the calendar is in.
pub fn current_era(year: i32) -> u8 {
    ERAS.iter().rposition(|start| *start <= year).unwrap_or(0) as u8
}

fn open(tree: &TechTree, view: &ResearchView, t: usize) -> bool {
    view.stages[t] == Stage::Unknown
        && tree.techs[t]
            .requires
            .iter()
            .all(|&r| view.stages[r as usize] >= Stage::Science)
}

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    tree: &TechTree,
    cat: &Catalogue,
    view: &ResearchView,
    era: &mut u8,
) -> ResearchResponse {
    let mut out = ResearchResponse::default();
    ui.painter()
        .rect_filled(screen, 0, Color32::from_black_alpha(120));
    let size = Vec2::new(
        (screen.width() - 80.0).min(CARD.x * 4.0 + space::MD * 3.0 + space::XL * 2.0 + 16.0),
        screen.height() - 120.0,
    );
    let panel = Rect::from_center_size(screen.center() + Vec2::new(0.0, 30.0), size);
    bf_ui_kit::paint_overlay(ui, panel);
    let block = ui.interact(
        screen,
        ui.make_persistent_id("research-block"),
        Sense::click(),
    );
    if block.clicked()
        && !ui
            .input(|i| i.pointer.interact_pos())
            .is_some_and(|p| panel.contains(p))
    {
        out.close = true;
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(panel.shrink(space::XL)));
    let ui = &mut child;
    ui.spacing_mut().item_spacing.y = space::SM;

    // Header: title, effort, focus.
    ui.horizontal(|ui| {
        bf_ui_kit::title(ui, &i18n.tr("research-title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::ghost(&i18n.tr("research-close"))
                .size(ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                out.close = true;
            }
        });
    });
    let focus = view
        .focus
        .map(|t| i18n.tr(&format!("tech-{}", tree.techs[t as usize].key)))
        .unwrap_or_else(|| i18n.tr("research-focus-auto"));
    bf_ui_kit::muted(
        ui,
        &i18n.tr_args(
            "research-summary",
            &[
                ("points", Arg::String(format!("{:.0}", view.points))),
                ("focus", Arg::String(focus)),
            ],
        ),
    );
    if view.focus.is_some()
        && Button::secondary(&i18n.tr("research-focus-clear"))
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
    {
        out.focus = Some(None);
    }

    // Era tabs.
    let eras: Vec<(u8, String)> = (0..ERAS.len() as u8)
        .map(|e| (e, i18n.tr(&format!("tech-era-{e}"))))
        .collect();
    bf_ui_kit::segmented(ui, era, &eras);
    bf_ui_kit::gap(ui, space::XS);

    // The panel's width, not the width of what was drawn above (the era tabs).
    let content_w = ui.max_rect().width();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(content_w - 16.0);
            let per_row = ((content_w - 16.0 + space::MD) / (CARD.x + space::MD))
                .floor()
                .max(1.0) as usize;
            for (f, field) in FIELDS.iter().enumerate() {
                let techs: Vec<usize> = tree
                    .techs
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.era == *era && usize::from(x.field) == f)
                    .map(|(t, _)| t)
                    .collect();
                if techs.is_empty() {
                    continue;
                }
                bf_ui_kit::caption(ui, &i18n.tr(&format!("tech-field-{field}")));
                for row in techs.chunks(per_row) {
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(content_w - 16.0, CARD.y), Sense::hover());
                    for (k, &t) in row.iter().enumerate() {
                        let r = Rect::from_min_size(
                            rect.min + Vec2::new(k as f32 * (CARD.x + space::MD), 0.0),
                            CARD,
                        );
                        if let Some(f) = card(ui, r, i18n, tree, cat, view, t) {
                            out.focus = Some(f);
                        }
                    }
                }
                bf_ui_kit::gap(ui, space::SM);
            }
        });
    out
}

fn card(
    ui: &mut Ui,
    r: Rect,
    i18n: &I18n,
    tree: &TechTree,
    cat: &Catalogue,
    view: &ResearchView,
    t: usize,
) -> Option<Option<u16>> {
    let tech = &tree.techs[t];
    let stage = view.stages[t];
    let focused = view.focus == Some(t as u16);
    let can_focus = open(tree, view, t);
    let response = ui.interact(r, ui.make_persistent_id(("tech", t)), Sense::click());
    let painter = ui.painter();
    let fill = if response.hovered() {
        color::SURFACE_HOVER
    } else {
        color::SURFACE_RAISED
    };
    let border = if focused {
        color::ACCENT
    } else {
        color::BORDER
    };
    painter.rect(
        r,
        radius::MD,
        fill,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

    let inner = r.shrink(space::SM);
    let name = i18n.tr(&format!("tech-{}", tech.key));
    let name_color = if stage == Stage::Unknown && !can_focus {
        color::TEXT_MUTED
    } else {
        color::TEXT
    };
    painter.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        &name,
        sans_semibold(14.0),
        name_color,
    );
    painter.text(
        Pos2::new(inner.right(), inner.top() + 27.0),
        Align2::RIGHT_TOP,
        format!("{} · {}", tech.demonstrated, tech.industrialized),
        sans(11.5),
        color::TEXT_FAINT,
    );
    // Stage label and the four-step track.
    let stage_y = inner.top() + 26.0;
    painter.text(
        Pos2::new(inner.left(), stage_y),
        Align2::LEFT_TOP,
        i18n.tr(&format!("tech-stage-{}", stage.key())),
        sans(12.5),
        stage_color(stage),
    );
    let track = Rect::from_min_size(
        Pos2::new(inner.left(), inner.bottom() - 30.0),
        Vec2::new(inner.width(), 6.0),
    );
    let step_w = (track.width() - 3.0 * 3.0) / 4.0;
    for (i, s) in Stage::ALL.iter().skip(1).enumerate() {
        let seg = Rect::from_min_size(
            Pos2::new(track.left() + i as f32 * (step_w + 3.0), track.top()),
            Vec2::new(step_w, track.height()),
        );
        painter.rect_filled(seg, 2.0, color::SURFACE_SUNKEN);
        let filled = if stage >= *s {
            1.0
        } else if stage.next() == Some(*s) {
            view.progress[t].clamp(0.0, 1.0)
        } else {
            0.0
        };
        if filled > 0.0 {
            let mut done = seg;
            done.set_width(seg.width() * filled);
            painter.rect_filled(done, 2.0, stage_color(*s));
        }
    }
    // Focus control.
    let mut picked = None;
    let button_rect = Rect::from_min_size(
        Pos2::new(inner.left(), inner.bottom() - 18.0),
        Vec2::new(inner.width(), 18.0),
    );
    if focused {
        painter.text(
            button_rect.left_center(),
            Align2::LEFT_CENTER,
            i18n.tr("research-focused"),
            sans(12.0),
            color::ACCENT,
        );
    } else if can_focus {
        let hover = response.hovered();
        painter.text(
            button_rect.left_center(),
            Align2::LEFT_CENTER,
            i18n.tr("research-set-focus"),
            sans(12.0),
            if hover {
                color::ACCENT_HOVER
            } else {
                color::TEXT_MUTED
            },
        );
        if response.clicked() {
            picked = Some(Some(t as u16));
        }
    } else if stage == Stage::Unknown {
        painter.text(
            button_rect.left_center(),
            Align2::LEFT_CENTER,
            i18n.tr("research-locked"),
            sans(12.0),
            color::TEXT_FAINT,
        );
    }
    response.on_hover_ui(|ui| tooltip(ui, i18n, tree, cat, view, t));
    picked
}

fn tooltip(
    ui: &mut Ui,
    i18n: &I18n,
    tree: &TechTree,
    cat: &Catalogue,
    view: &ResearchView,
    t: usize,
) {
    let tech = &tree.techs[t];
    ui.set_max_width(320.0);
    ui.label(
        egui::RichText::new(i18n.tr(&format!("tech-{}", tech.key)))
            .font(sans_semibold(14.0))
            .color(color::TEXT),
    );
    ui.label(
        egui::RichText::new(i18n.tr(&format!("tech-{}-desc", tech.key)))
            .font(sans(13.0))
            .color(color::TEXT_MUTED),
    );
    ui.add_space(space::XS);
    let names = |list: Vec<String>| list.join(", ");
    if !tech.requires.is_empty() {
        let req = names(
            tech.requires
                .iter()
                .map(|&r| i18n.tr(&format!("tech-{}", tree.techs[r as usize].key)))
                .collect(),
        );
        ui.label(
            egui::RichText::new(i18n.tr_args("research-requires", &[("list", Arg::String(req))]))
                .font(sans(12.5)),
        );
    }
    let mut unlocks: Vec<String> = tech
        .methods
        .iter()
        .map(|&(b, m)| {
            let b = &cat.buildings[b as usize];
            format!(
                "{} ({})",
                i18n.tr(&format!("method-{}", b.methods[m as usize].key)),
                i18n.tr(&format!("building-{}", b.key))
            )
        })
        .collect();
    unlocks.extend(
        tech.goods
            .iter()
            .map(|&g| i18n.tr(&format!("good-{}", cat.goods[g as usize].key))),
    );
    if !unlocks.is_empty() {
        ui.label(
            egui::RichText::new(
                i18n.tr_args("research-unlocks", &[("list", Arg::String(names(unlocks)))]),
            )
            .font(sans(12.5)),
        );
    }
    ui.label(
        egui::RichText::new(i18n.tr_args(
            "research-history",
            &[
                ("demonstrated", Arg::String(tech.demonstrated.to_string())),
                (
                    "industrialized",
                    Arg::String(tech.industrialized.to_string()),
                ),
            ],
        ))
        .font(sans(12.5))
        .color(color::TEXT_FAINT),
    );
    let first = view.firsts[t][Stage::Industrial as usize];
    if first > 0 {
        ui.label(
            egui::RichText::new(i18n.tr_args(
                "research-first-industrial",
                &[("year", Arg::String(first.to_string()))],
            ))
            .font(sans(12.5))
            .color(color::TEXT_FAINT),
        );
    }
    if tech.state_buyer {
        ui.label(
            egui::RichText::new(i18n.tr("research-state-buyer"))
                .font(sans(12.5))
                .color(color::TEXT_FAINT),
        );
    }
}
