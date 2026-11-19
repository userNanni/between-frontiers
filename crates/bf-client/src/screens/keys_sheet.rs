//! The sheet of keys (F1): every command with its keys, by screen, with a search. Read only;
//! the keys change in the settings' keyboard editor. Built from the same registry the keys run
//! on, so it never says a key the game does not answer.

use bf_i18n::Arg;
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, space};
use egui::{Rect, Sense, Ui};

use crate::app::Shared;
use crate::keys::{self, Category, chord_label, id};

/// The open sheet: what the player searched.
#[derive(Clone, Debug, Default)]
pub struct Sheet {
    pub query: String,
}

/// One command on the sheet: its name and its keys, as printed on the board.
pub struct Row {
    pub category: Category,
    pub name: String,
    pub keys: Vec<String>,
}

/// Every command that has a key, by category, in the registry's order; those that `query`
/// finds in their name or keys (case and accents aside).
pub fn rows(shared: &Shared, query: &str) -> Vec<Row> {
    let q = fold(query.trim());
    let layout = shared.keys.config().layout;
    let mut out = Vec::new();
    for category in Category::ALL {
        for def in shared.keys.registry().all() {
            if def.category != category {
                continue;
            }
            let keys: Vec<String> = shared
                .keys
                .keys_of(def.id)
                .iter()
                .map(|c| chord_label(*c, layout, &shared.i18n))
                .collect();
            if keys.is_empty() {
                continue;
            }
            let name = shared.i18n.tr(&def.name_key());
            if !q.is_empty()
                && !fold(&name).contains(&q)
                && !keys.iter().any(|k| fold(k).contains(&q))
            {
                continue;
            }
            out.push(Row {
                category,
                name,
                keys,
            });
        }
    }
    out
}

/// Draws the sheet over `screen`; false when the player closed it.
pub fn show(ui: &mut Ui, screen: Rect, shared: &Shared, sheet: &mut Sheet) -> bool {
    let i18n = &shared.i18n;
    let mut open = true;
    ui.painter()
        .rect_filled(screen, 0, egui::Color32::from_black_alpha(140));
    keys::cover(ui, screen);
    let size = egui::vec2(
        (screen.width() - 2.0 * space::XL).min(720.0),
        (screen.height() - 2.0 * space::XL).min(820.0),
    );
    let panel = Rect::from_center_size(screen.center(), size);
    let block = ui.interact(
        screen,
        ui.make_persistent_id("keys-sheet-block"),
        Sense::click(),
    );
    if block.clicked()
        && !ui
            .input(|i| i.pointer.interact_pos())
            .is_some_and(|p| panel.contains(p))
    {
        open = false;
    }
    super::frame(ui, panel, "keys-sheet");
    super::region(ui, panel.shrink(space::XL), |ui| {
        ui.spacing_mut().item_spacing.y = space::SM;
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("keys-sheet-title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = i18n.tr("keys-sheet-close");
                let tip = keys::tip(ui.ctx(), &label, id::BACK);
                if bf_ui_kit::close_button(ui, &label, &tip).clicked() {
                    open = false;
                }
            });
        });
        let editor = keys::shortcut(ui.ctx(), id::KEYS).unwrap_or_default();
        bf_ui_kit::small(
            ui,
            &i18n.tr_args("keys-sheet-basis", &[("editor", Arg::String(editor))]),
        );
        let field = ui.add(
            egui::TextEdit::singleline(&mut sheet.query)
                .id(ui.make_persistent_id("keys-sheet-search"))
                .font(sans(13.0))
                .hint_text(i18n.tr("keys-sheet-search"))
                .desired_width(f32::INFINITY),
        );
        // The search takes the keyboard on opening.
        if ui.memory(|m| m.focused().is_none()) && !field.lost_focus() {
            field.request_focus();
        }
        let rows = rows(shared, &sheet.query);
        if rows.is_empty() {
            bf_ui_kit::muted(ui, &i18n.tr("keys-sheet-none"));
        }
        egui::ScrollArea::vertical()
            .id_salt("keys-sheet-rows")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let width = ui.available_width();
                let mut last = None;
                for row in &rows {
                    if last != Some(row.category) {
                        ui.add_space(space::SM);
                        ui.label(
                            egui::RichText::new(i18n.tr(row.category.name_key()))
                                .font(sans_semibold(14.0))
                                .color(color::ACCENT),
                        );
                        last = Some(row.category);
                    }
                    ui.horizontal(|ui| {
                        ui.set_width(width);
                        ui.label(
                            egui::RichText::new(&row.name)
                                .font(sans(13.0))
                                .color(color::TEXT),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(row.keys.join(" · "))
                                    .font(sans_semibold(13.0))
                                    .color(color::TEXT_MUTED),
                            );
                        });
                    });
                }
            });
    });
    open
}

/// Folds a name for search: lower case, accents off.
fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}
