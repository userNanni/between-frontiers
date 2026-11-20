//! "Alt: shortcuts · F1: list": a chip of the HUD from the first campaign until the player has
//! used the shortcuts a few times, or for good once pinned; and the one-line banner over the key
//! tips that light by themselves the first time.

use bf_i18n::Arg;
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, radius, space};
use egui::{Align2, Rect, Sense, Stroke, Ui, Vec2};

use crate::app::Shared;
use crate::keys::{self, id};

/// Uses after which the chip goes, unless pinned.
pub const FADE_AFTER: u32 = 5;

const H: f32 = 34.0;

/// The chip is on: introduced and not yet used enough, or pinned.
pub fn shown(shared: &Shared) -> bool {
    let s = &shared.settings;
    s.shortcut_chip_pinned || s.shortcut_uses.is_some_and(|n| n < FADE_AFTER)
}

fn text(shared: &Shared, ctx: &egui::Context) -> String {
    let sheet = keys::shortcut(ctx, id::KEYS_SHEET).unwrap_or_default();
    let leader = crate::screens::keys_editor::leader_name(shared, shared.keys.config().leader);
    shared.i18n.tr_args(
        "shortcuts-chip",
        &[
            ("leader", Arg::String(leader)),
            ("sheet", Arg::String(sheet)),
        ],
    )
}

fn pin_label(shared: &Shared) -> String {
    shared.i18n.tr(if shared.settings.shortcut_chip_pinned {
        "shortcuts-chip-unpin"
    } else {
        "shortcuts-chip-pin"
    })
}

/// The chip's size: its line and the pin.
pub fn size(shared: &Shared, ctx: &egui::Context) -> Vec2 {
    let w = |t: String, f| ctx.fonts_mut(|fonts| fonts.layout_no_wrap(t, f, color::TEXT).size().x);
    let line = w(text(shared, ctx), sans_semibold(13.0));
    let pin = w(pin_label(shared), sans(12.0));
    Vec2::new(line + pin + space::LG * 3.0 + space::MD, H)
}

/// What the player did on the chip.
#[derive(Default)]
pub struct ChipOut {
    pub open_sheet: bool,
    pub toggle_pin: bool,
}

pub fn show(ui: &mut Ui, rect: Rect, shared: &Shared) -> ChipOut {
    let mut out = ChipOut::default();
    let painter = ui.painter();
    painter.rect(
        rect,
        radius::MD,
        color::SURFACE,
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    super::mark_frame(ui, rect, "shortcuts");
    let pin = pin_label(shared);
    let pin_w = ui
        .painter()
        .layout_no_wrap(pin.clone(), sans(12.0), color::TEXT_MUTED)
        .size()
        .x
        + space::MD;
    let line_rect = Rect::from_min_max(
        rect.min,
        egui::pos2(rect.right() - pin_w - space::MD, rect.bottom()),
    );
    let line = ui.interact(
        line_rect,
        ui.make_persistent_id("shortcuts-chip-line"),
        Sense::click(),
    );
    ui.painter().text(
        egui::pos2(rect.left() + space::LG, rect.center().y),
        Align2::LEFT_CENTER,
        text(shared, ui.ctx()),
        sans_semibold(13.0),
        if line.hovered() {
            color::ACCENT
        } else {
            color::TEXT
        },
    );
    let leader = crate::screens::keys_editor::leader_name(shared, shared.keys.config().leader);
    let line = line.on_hover_text(
        shared
            .i18n
            .tr_args("shortcuts-chip-tip", &[("leader", Arg::String(leader))]),
    );
    if line.clicked() {
        out.open_sheet = true;
    }
    let pin_rect = Rect::from_min_max(
        egui::pos2(rect.right() - pin_w - space::MD, rect.top()),
        rect.max,
    );
    let pin_r = ui.interact(
        pin_rect,
        ui.make_persistent_id("shortcuts-chip-pin"),
        Sense::click(),
    );
    ui.painter().text(
        pin_rect.center(),
        Align2::CENTER_CENTER,
        pin,
        sans(12.0),
        if pin_r.hovered() {
            color::ACCENT
        } else {
            color::TEXT_MUTED
        },
    );
    if pin_r.clicked() || keys::single(ui, "hud.shortcuts.pin", 'F', pin_rect) {
        out.toggle_pin = true;
    }
    out
}

/// The banner over the key tips lit by themselves: one line, above the chip's place.
pub fn banner(ui: &mut Ui, screen: Rect, shared: &Shared) {
    let text = shared.i18n.tr_args(
        "shortcuts-first-banner",
        &[(
            "sheet",
            Arg::String(keys::shortcut(ui.ctx(), id::KEYS_SHEET).unwrap_or_default()),
        )],
    );
    let galley = ui.painter().layout(
        text,
        sans_semibold(14.0),
        color::ON_ACCENT,
        (screen.width() - 4.0 * space::XL).max(200.0),
    );
    let size = galley.size() + Vec2::new(space::XL * 2.0, space::MD * 2.0);
    let rect = Rect::from_center_size(
        egui::pos2(
            screen.center().x,
            screen.bottom() - H - crate::hud::GAP * 2.0 - size.y / 2.0,
        ),
        size,
    );
    ui.painter().rect_filled(rect, radius::MD, color::ACCENT);
    ui.painter().galley(
        rect.min + Vec2::new(space::XL, space::MD),
        galley,
        color::ON_ACCENT,
    );
}
