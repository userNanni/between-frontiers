//! Government tab: money. Drawn inside the government window (`government.rs`).

use egui::Ui;

use super::government::TabContext;

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    bf_ui_kit::muted(ui, &cx.i18n.tr("gov-coming"));
}
