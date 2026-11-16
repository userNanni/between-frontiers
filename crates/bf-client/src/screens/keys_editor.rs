//! Keyboard editor, opened from the settings: a drawn keyboard with every key that triggers a
//! command coloured by the command's category, one modifier layer at a time; the list of
//! commands with a search; and the selected command's keys, with conflicts, undo, defaults,
//! profiles, layouts and export to a file.

use bf_ui_kit::theme::{sans, sans_medium, sans_semibold};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::text::LayoutJob;
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use super::{centered, region};
use crate::app::Shared;
use crate::keys::keymap::{Conflict, Export, PROFILES, profile_name_key};
use crate::keys::{
    Category, Chord, KeyConfig, Layout, Leader, Mods, PhysKey, chord_label, id, key_label,
};

/// What a captured key goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Capture {
    Command(&'static str),
    Leader,
}

/// The editor's state while it is open.
#[derive(Default)]
pub struct State {
    selected: Option<&'static str>,
    layer: Mods,
    search: String,
    capture: Option<Capture>,
    /// A key asked for and already taken, waiting for the player to swap or cancel.
    pending: Option<Conflict>,
    undo: Vec<KeyConfig>,
    file: Option<String>,
    note: Option<(String, bool)>,
}

impl State {
    /// Opens with `cmd` selected and the layer `layer` showing.
    pub fn with(cmd: &'static str, layer: Mods) -> Self {
        Self {
            selected: Some(cmd),
            layer,
            ..Self::default()
        }
    }
}

/// Colour of each category on the keyboard and in the list.
pub fn category_color(c: Category) -> Color32 {
    match c {
        Category::Time => Color32::from_rgb(0xD2, 0x8C, 0x3C),
        Category::Map => Color32::from_rgb(0x4E, 0x9C, 0x8B),
        Category::Government => Color32::from_rgb(0xB8, 0x5F, 0x5A),
        Category::Windows => Color32::from_rgb(0x5D, 0x80, 0xBC),
        Category::Research => Color32::from_rgb(0x91, 0x6E, 0xB6),
        Category::System => Color32::from_rgb(0x84, 0x90, 0x9C),
    }
}

pub fn show(shared: &mut Shared, ui: &mut Ui, rect: Rect) {
    let Some(mut st) = shared.key_editor.take() else {
        return;
    };
    let size = Vec2::new(1320.0, 840.0).min(rect.size() - Vec2::splat(24.0));
    let panel = centered(rect, size);
    super::frame(ui, panel, "keys-editor");
    ui.interact(
        panel,
        ui.make_persistent_id("keys-editor"),
        Sense::click_and_drag(),
    );
    let mut close = false;
    capture_keys(shared, ui, &mut st);
    region(ui, panel.shrink(space::XL), |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(space::SM, space::SM);
        close = header(shared, ui, &mut st);
        toolbar(shared, ui, &mut st);
        let kb_h = (ui.available_height() * 0.5).min(330.0);
        let kb = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), kb_h));
        keyboard(shared, ui, &mut st, kb);
        ui.advance_cursor_after_rect(kb);
        bf_ui_kit::divider(ui);
        let rest = ui.available_rect_before_wrap();
        let list_w = (rest.width() * 0.5).min(560.0);
        let list = Rect::from_min_size(rest.min, Vec2::new(list_w, rest.height()));
        let detail = Rect::from_min_max(Pos2::new(list.right() + space::XL, rest.top()), rest.max);
        region(ui, list, |ui| command_list(shared, ui, &mut st));
        region(ui, detail, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("keys-detail")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(detail.width() - 12.0);
                    details(shared, ui, &mut st);
                });
        });
    });
    shared.keys.capturing = st.capture.is_some();
    if st.capture.is_none() && st.pending.is_none() && crate::keys::fired(ui.ctx(), id::BACK) {
        close = true;
    }
    if !close {
        shared.key_editor = Some(st);
    } else {
        shared.keys.capturing = false;
    }
}

/// A key pressed while the editor listens goes to the command (or the leader).
fn capture_keys(shared: &mut Shared, ui: &Ui, st: &mut State) {
    let Some(target) = st.capture else {
        return;
    };
    let events = ui.input(|i| i.events.clone());
    for e in events {
        let egui::Event::Key {
            key,
            physical_key,
            pressed: true,
            repeat: false,
            modifiers,
        } = e
        else {
            continue;
        };
        let Some(phys) = physical_key
            .and_then(PhysKey::from_egui)
            .or_else(|| PhysKey::from_egui(key))
        else {
            continue;
        };
        if phys == PhysKey::Escape {
            st.capture = None;
            return;
        }
        match target {
            Capture::Leader => {
                set_leader(shared, st, leader_of(phys));
                st.capture = None;
                return;
            }
            Capture::Command(cmd) if !phys.is_modifier() => {
                assign(
                    shared,
                    st,
                    cmd,
                    Chord::with_mods(phys, Mods::from_egui(modifiers)),
                );
                st.capture = None;
                return;
            }
            Capture::Command(_) => {}
        }
    }
}

/// The leader setting for a key: the left Alt is the default leader, any other key (AltGr
/// included) stands for itself.
fn leader_of(key: PhysKey) -> Leader {
    if key == PhysKey::AltLeft {
        Leader::Alt
    } else {
        Leader::Key(key)
    }
}

/// The leader's name, saying "left Alt" so it is never taken for AltGr.
pub fn leader_name(shared: &Shared, leader: Leader) -> String {
    match leader {
        Leader::Alt => shared.tr("keys-alt-left"),
        Leader::Key(PhysKey::AltRight) => "AltGr".to_owned(),
        Leader::Key(k) => key_label(k, shared.keys.config().layout, &shared.i18n),
    }
}

fn set_leader(shared: &mut Shared, st: &mut State, leader: Leader) {
    st.undo.push(shared.keys.config().clone());
    shared.keys.edit(|c, _| c.leader = leader);
}

/// Gives `chord` to `cmd`, or holds the conflict for the player to decide.
fn assign(shared: &mut Shared, st: &mut State, cmd: &'static str, chord: Chord) {
    let before = shared.keys.config().clone();
    match shared.keys.edit(|c, r| c.assign(r, cmd, chord, false)) {
        Ok(()) => {
            st.undo.push(before);
            st.pending = None;
        }
        Err(conflict) => st.pending = Some(conflict),
    }
}

/// The command's label on a drawn key: short, so neighbouring keys stay apart.
fn short_name(shared: &Shared, cmd: &str) -> String {
    let key = crate::keys::commands::name_key(cmd);
    shared
        .i18n
        .attr(&key, "short")
        .unwrap_or_else(|| shared.tr(&key))
}

fn name(shared: &Shared, cmd: &str) -> String {
    shared.tr(&crate::keys::commands::name_key(cmd))
}

fn header(shared: &mut Shared, ui: &mut Ui, st: &mut State) -> bool {
    let mut close = false;
    // Wraps on a narrow screen rather than leaving the window.
    ui.horizontal_wrapped(|ui| {
        bf_ui_kit::title(ui, &shared.tr("keys-title"));
        ui.add_space(space::LG);
        let mut profile = shared.keys.config().profile.clone();
        let options: Vec<(String, String)> = PROFILES
            .iter()
            .map(|p| ((*p).to_owned(), shared.tr(&profile_name_key(p))))
            .collect();
        if bf_ui_kit::segmented(ui, &mut profile, &options) {
            st.undo.push(shared.keys.config().clone());
            shared.keys.edit(|c, _| c.profile = profile);
            st.pending = None;
        }
        ui.add_space(space::SM);
        let mut layout = shared.keys.config().layout;
        let options: Vec<(Layout, String)> = Layout::ALL
            .iter()
            .map(|l| (*l, l.name().to_owned()))
            .collect();
        if bf_ui_kit::segmented(ui, &mut layout, &options) {
            shared.keys.edit(|c, _| c.layout = layout);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let done = Button::primary(&shared.tr("settings-done"))
                .size(ButtonSize::Small)
                .show(ui);
            if done.clicked() || crate::keys::single(ui, "keys.done", 'D', done.rect) {
                close = true;
            }
            let reset = Button::ghost(&shared.tr("keys-reset-all"))
                .size(ButtonSize::Small)
                .show(ui);
            if reset.clicked() {
                st.undo.push(shared.keys.config().clone());
                shared.keys.edit(|c, _| c.reset_all());
                st.pending = None;
            }
            let undo = Button::secondary(&shared.tr("keys-undo"))
                .size(ButtonSize::Small)
                .enabled(!st.undo.is_empty())
                .show(ui);
            if undo.clicked()
                && let Some(old) = st.undo.pop()
            {
                shared.keys.edit(|c, _| *c = old);
                st.pending = None;
            }
        });
    });
    close
}

/// Modifier layers, the leader key and the legend.
fn toolbar(shared: &mut Shared, ui: &mut Ui, st: &mut State) {
    ui.horizontal_wrapped(|ui| {
        bf_ui_kit::caption(ui, &shared.tr("keys-layer"));
        for (label, on) in [
            ("Ctrl", &mut st.layer.ctrl),
            ("Shift", &mut st.layer.shift),
            ("Alt", &mut st.layer.alt),
        ] {
            if chip(ui, label, *on, None).clicked() {
                *on = !*on;
            }
        }
        ui.add_space(space::LG);
        bf_ui_kit::caption(ui, &shared.tr("keys-leader"));
        let current = shared.keys.config().leader;
        let listening = st.capture == Some(Capture::Leader);
        let tip = shared.i18n.tr_args(
            "keys-leader-tip",
            &[("key", bf_i18n::Arg::String(leader_name(shared, current)))],
        );
        for choice in [Leader::Alt, Leader::Key(PhysKey::AltRight)] {
            let r = chip(
                ui,
                &leader_name(shared, choice),
                current == choice && !listening,
                None,
            )
            .on_hover_text(bf_ui_kit::tooltip_text(&tip));
            if r.clicked() {
                set_leader(shared, st, choice);
                st.capture = None;
            }
        }
        let other = !matches!(current, Leader::Alt | Leader::Key(PhysKey::AltRight));
        let text = if listening {
            shared.tr("keys-leader-pick")
        } else if other {
            leader_name(shared, current)
        } else {
            shared.tr("keys-leader-other")
        };
        let r =
            chip(ui, &text, other || listening, None).on_hover_text(bf_ui_kit::tooltip_text(&tip));
        if r.clicked() {
            st.capture = if listening {
                None
            } else {
                Some(Capture::Leader)
            };
        }
        ui.add_space(space::LG);
        for c in Category::ALL {
            legend(ui, &shared.tr(c.name_key()), category_color(c));
        }
    });
}

fn chip(ui: &mut Ui, text: &str, on: bool, tone: Option<Color32>) -> egui::Response {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_owned(), sans_medium(size::SMALL), color::TEXT);
    let (r, resp) = ui.allocate_exact_size(Vec2::new(galley.size().x + 20.0, 26.0), Sense::click());
    let t = text.to_owned();
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, t.clone()));
    let fill = if on {
        color::ACCENT_SOFT
    } else if resp.hovered() {
        color::SURFACE_HOVER
    } else {
        color::SURFACE_RAISED
    };
    let stroke = if on {
        color::ACCENT
    } else {
        tone.unwrap_or(color::BORDER)
    };
    ui.painter().rect(
        r,
        radius::MD,
        fill,
        Stroke::new(1.0, stroke),
        egui::StrokeKind::Inside,
    );
    ui.painter().galley(
        r.center() - galley.size() / 2.0,
        galley,
        if on { color::ACCENT } else { color::TEXT },
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn legend(ui: &mut Ui, text: &str, c: Color32) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
    ui.painter().rect_filled(r, radius::SM, c);
    ui.label(
        egui::RichText::new(text)
            .font(sans(size::SMALL))
            .color(color::TEXT_MUTED),
    );
    ui.add_space(space::XS);
}

/// The drawn keyboard for the current layout and layer.
fn keyboard(shared: &mut Shared, ui: &mut Ui, st: &mut State, area: Rect) {
    let layout = shared.keys.config().layout;
    let rows = layout.rows();
    // 15 units of board, half a unit of gap, 3 of navigation keys.
    let unit = (area.width() / 18.6).min(62.0);
    let gap_y = unit * 0.3;
    let kh = ((area.height() - 22.0 - gap_y) / 6.0).min(unit * 0.86);
    let pad = (unit * 0.07).max(2.0);
    let total_w = unit * 18.5;
    let origin = Pos2::new(area.center().x - total_w / 2.0, area.top());
    let conflicts = shared.keys.config().conflicts(shared.keys.registry());
    let mut hovered: Option<PhysKey> = None;
    let mut clicked: Option<PhysKey> = None;
    let mut draw = |ui: &mut Ui, key: PhysKey, r: Rect, tall: Option<Rect>| {
        let resp = ui.interact(
            r,
            ui.make_persistent_id(("kb", key, r.top() as i32)),
            Sense::click(),
        );
        let name = format!("{} ({})", key_label(key, layout, &shared.i18n), key.code());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name.clone()));
        let resp = match tall {
            Some(extra) => {
                let e = ui.interact(
                    extra,
                    ui.make_persistent_id(("kb-tall", key)),
                    Sense::click(),
                );
                resp.union(e)
            }
            None => resp,
        };
        if resp.hovered() || resp.has_focus() {
            hovered = Some(key);
        }
        if resp.clicked() {
            clicked = Some(key);
        }
        key_cap(
            shared,
            ui,
            st,
            key,
            r,
            tall,
            layout,
            &conflicts,
            resp.hovered() || resp.has_focus(),
        );
    };
    for (row_i, row) in rows.iter().enumerate() {
        let y = origin.y
            + if row_i == 0 {
                0.0
            } else {
                gap_y + kh * row_i as f32
            };
        let mut x = origin.x;
        for (key, w) in row {
            let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(unit * w, kh)).shrink(pad);
            x += unit * w;
            let Some(key) = key else { continue };
            if *key == PhysKey::Enter && layout == Layout::Abnt2 {
                if row_i == 2 {
                    // The tall ISO Enter: drawn once, spanning two rows.
                    let lower = Rect::from_min_size(
                        Pos2::new(r.left() + unit * 0.25, r.top()),
                        Vec2::new(unit * 1.25 - pad * 2.0, kh * 2.0 - pad * 2.0),
                    );
                    draw(ui, *key, r, Some(lower));
                }
                continue;
            }
            draw(ui, *key, r, None);
        }
    }
    let nav_x = origin.x + unit * 15.5;
    for (i, row) in Layout::nav_rows().iter().enumerate() {
        let y = origin.y + gap_y + kh * (i + 1) as f32;
        for (j, key) in row.iter().enumerate() {
            if let Some(key) = key {
                let r =
                    Rect::from_min_size(Pos2::new(nav_x + unit * j as f32, y), Vec2::new(unit, kh))
                        .shrink(pad);
                draw(ui, *key, r, None);
            }
        }
    }
    // What the key under the pointer (or focused) does.
    let info_y = origin.y + gap_y + kh * 6.0 + 4.0;
    let info = match hovered {
        Some(k) if !k.is_modifier() => {
            let chord = Chord::with_mods(k, st.layer);
            let label = chord_label(chord, layout, &shared.i18n);
            match shared.keys.holder(chord) {
                Some(cmd) => format!("{label} — {}", name(shared, cmd)),
                None => format!("{label} — {}", shared.tr("keys-free")),
            }
        }
        Some(_) => shared.tr("keys-modifier-tip"),
        None => match st.selected {
            Some(_) => shared.tr("keys-click-to-assign"),
            None => shared.tr("keys-hover-tip"),
        },
    };
    ui.painter().text(
        Pos2::new(origin.x + pad, info_y),
        Align2::LEFT_TOP,
        info,
        sans(size::SMALL),
        color::TEXT_MUTED,
    );
    if let Some(k) = clicked
        && st.capture == Some(Capture::Leader)
    {
        set_leader(shared, st, leader_of(k));
        st.capture = None;
    } else if let Some(k) = clicked {
        if k.is_modifier() {
            use PhysKey::*;
            match k {
                ControlLeft | ControlRight => st.layer.ctrl = !st.layer.ctrl,
                ShiftLeft | ShiftRight => st.layer.shift = !st.layer.shift,
                // AltGr types characters: it is not the Alt layer.
                AltLeft => st.layer.alt = !st.layer.alt,
                _ => {}
            }
        } else {
            let chord = Chord::with_mods(k, st.layer);
            match st.selected {
                Some(cmd) => assign(shared, st, cmd, chord),
                None => st.selected = shared.keys.holder(chord),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn key_cap(
    shared: &Shared,
    ui: &Ui,
    st: &State,
    key: PhysKey,
    r: Rect,
    tall: Option<Rect>,
    layout: Layout,
    conflicts: &[Conflict],
    hot: bool,
) {
    let painter = ui.painter();
    let chord = Chord::with_mods(key, st.layer);
    let holder = if key.is_modifier() {
        None
    } else {
        shared.keys.holder(chord)
    };
    let category = holder
        .and_then(|c| shared.keys.registry().get(c))
        .map(|d| d.category);
    let layer_on = {
        use PhysKey::*;
        match key {
            ControlLeft | ControlRight => st.layer.ctrl,
            ShiftLeft | ShiftRight => st.layer.shift,
            AltLeft => st.layer.alt,
            _ => false,
        }
    };
    let base = match category {
        Some(c) => mix(color::SURFACE_RAISED, category_color(c), 0.62),
        None if layer_on => color::ACCENT_SOFT,
        None => color::SURFACE_RAISED,
    };
    let fill = if hot {
        mix(base, Color32::WHITE, 0.12)
    } else {
        base
    };
    let selected = holder.is_some() && holder == st.selected;
    let conflict = conflicts.iter().any(|c| c.chord == chord);
    let pending = st.pending.is_some_and(|p| p.chord == chord);
    let is_leader = key == crate::keys::leader::leader_key(shared.keys.config().leader);
    let stroke = if is_leader {
        Stroke::new(2.0, color::POSITIVE)
    } else if pending || conflict {
        Stroke::new(2.0, color::DANGER)
    } else if selected {
        Stroke::new(2.0, color::ACCENT_HOVER)
    } else if layer_on {
        Stroke::new(1.5, color::ACCENT)
    } else {
        Stroke::new(1.0, color::BORDER_STRONG.gamma_multiply(0.7))
    };
    // A soft edge under the cap so the board reads as keys.
    let shapes: Vec<Rect> = std::iter::once(r).chain(tall).collect();
    for s in &shapes {
        painter.rect_filled(s.translate(Vec2::new(0.0, 2.0)), radius::MD, color::SHADOW);
    }
    for s in &shapes {
        painter.rect(*s, radius::MD, fill, Stroke::NONE, egui::StrokeKind::Inside);
    }
    for s in &shapes {
        painter.rect_stroke(*s, radius::MD, stroke, egui::StrokeKind::Inside);
    }
    if let Some(lower) = tall {
        // Hide the seam between the two parts of the tall Enter.
        let seam = Rect::from_min_max(
            Pos2::new(lower.left() + stroke.width, r.bottom() - 6.0),
            Pos2::new(lower.right() - stroke.width, r.bottom() + 6.0),
        );
        painter.rect_filled(seam, 0.0, fill);
    }
    // Legend: shifted character above, base below, or the key's name.
    let ink = if category.is_some() {
        color::TEXT
    } else {
        color::TEXT_MUTED
    };
    let tl = r.left_top() + Vec2::new(6.0, 4.0);
    match layout.legend(key) {
        Some((base, Some(shifted))) if holder.is_none() && r.height() > 34.0 => {
            painter.text(
                tl,
                Align2::LEFT_TOP,
                shifted,
                sans(10.5),
                ink.gamma_multiply(0.7),
            );
            painter.text(
                tl + Vec2::new(0.0, 12.0),
                Align2::LEFT_TOP,
                base,
                sans_semibold(13.0),
                ink,
            );
        }
        _ => {
            painter.text(
                tl,
                Align2::LEFT_TOP,
                key_label(key, layout, &shared.i18n),
                sans_semibold(if key.letter().is_some() { 13.0 } else { 11.0 }),
                ink,
            );
        }
    }
    let tag = if is_leader {
        Some(shared.tr("keys-leader-tag"))
    } else {
        holder.map(|cmd| short_name(shared, cmd))
    };
    if let Some(label) = tag {
        let width = r.width() - 8.0;
        let mut job = LayoutJob::simple_singleline(label, sans(9.5), color::TEXT);
        job.wrap = egui::text::TextWrapping::truncate_at_width(width);
        let g = painter.layout_job(job);
        painter.galley(
            Pos2::new(r.left() + 4.0, r.bottom() - g.size().y - 3.0),
            g,
            color::TEXT,
        );
    }
    if hot && let Some(cmd) = holder {
        let text = format!(
            "{}\n{}",
            name(shared, cmd),
            chord_label(chord, layout, &shared.i18n)
        );
        egui::Tooltip::always_open(
            ui.ctx().clone(),
            ui.layer_id(),
            egui::Id::unique(("kb-tip", key)),
            egui::PopupAnchor::Position(r.center_bottom() + Vec2::new(0.0, 6.0)),
        )
        .show(|ui| {
            ui.label(bf_ui_kit::tooltip_text(&text));
        });
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn command_list(shared: &mut Shared, ui: &mut Ui, st: &mut State) {
    ui.horizontal(|ui| {
        bf_ui_kit::caption(ui, &shared.tr("keys-search"));
        let w = ui.available_width();
        bf_ui_kit::text_field(ui, &mut st.search, w);
    });
    let needle = st.search.to_lowercase();
    let layout = shared.keys.config().layout;
    egui::ScrollArea::vertical()
        .id_salt("keys-list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for cat in Category::ALL {
                let defs: Vec<(&'static str, String, Vec<Chord>)> = shared
                    .keys
                    .registry()
                    .all()
                    .iter()
                    .filter(|d| d.category == cat)
                    .map(|d| (d.id, name(shared, d.id), shared.keys.keys_of(d.id).to_vec()))
                    .filter(|(_, n, keys)| {
                        needle.is_empty()
                            || n.to_lowercase().contains(&needle)
                            || keys.iter().any(|k| {
                                chord_label(*k, layout, &shared.i18n)
                                    .to_lowercase()
                                    .contains(&needle)
                            })
                    })
                    .collect();
                if defs.is_empty() {
                    continue;
                }
                bf_ui_kit::gap(ui, space::XS);
                bf_ui_kit::caption(ui, &shared.tr(cat.name_key()));
                for (cmd, label, keys) in defs {
                    let w = ui.available_width();
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 26.0), Sense::click());
                    let on = st.selected == Some(cmd);
                    let l = label.clone();
                    resp.widget_info(|| {
                        egui::WidgetInfo::selected(egui::Role::Button, true, on, l.clone())
                    });
                    let painter = ui.painter();
                    if on {
                        painter.rect_filled(r, radius::SM, color::ACCENT_SOFT);
                    } else if resp.hovered() {
                        painter.rect_filled(r, radius::SM, color::SURFACE_HOVER);
                    }
                    painter.rect_filled(
                        Rect::from_center_size(
                            Pos2::new(r.left() + 8.0, r.center().y),
                            Vec2::splat(8.0),
                        ),
                        radius::SM,
                        category_color(cat),
                    );
                    painter.text(
                        Pos2::new(r.left() + 20.0, r.center().y),
                        Align2::LEFT_CENTER,
                        &label,
                        sans(size::SMALL),
                        if on { color::ACCENT } else { color::TEXT },
                    );
                    let mut x = r.right() - 4.0;
                    for k in keys.iter().rev() {
                        x = keycap_label(
                            ui,
                            &chord_label(*k, layout, &shared.i18n),
                            x,
                            r.center().y,
                        );
                    }
                    if keys.is_empty() {
                        painter.text(
                            Pos2::new(x, r.center().y),
                            Align2::RIGHT_CENTER,
                            shared.tr("keys-none"),
                            sans(size::SMALL),
                            color::TEXT_FAINT,
                        );
                    }
                    if resp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        st.selected = Some(cmd);
                        st.pending = None;
                        st.capture = None;
                    }
                }
            }
        });
}

/// A small key label drawn right-aligned at `right`; returns where the next one ends.
fn keycap_label(ui: &Ui, text: &str, right: f32, y: f32) -> f32 {
    let painter = ui.painter();
    let g = painter.layout_no_wrap(text.to_owned(), sans_medium(11.5), color::TEXT);
    let r = Rect::from_min_size(
        Pos2::new(right - g.size().x - 12.0, y - 10.0),
        Vec2::new(g.size().x + 12.0, 20.0),
    );
    painter.rect(
        r,
        radius::SM,
        color::SURFACE_SUNKEN,
        Stroke::new(1.0, color::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );
    painter.galley(r.center() - g.size() / 2.0, g, color::TEXT);
    r.left() - 4.0
}

fn details(shared: &mut Shared, ui: &mut Ui, st: &mut State) {
    let layout = shared.keys.config().layout;
    if let Some(p) = st.pending {
        conflict_box(shared, ui, st, p, layout);
        bf_ui_kit::gap(ui, space::SM);
    }
    match st
        .selected
        .and_then(|c| shared.keys.registry().get(c).cloned())
    {
        None => {
            bf_ui_kit::muted(ui, &shared.tr("keys-pick-command"));
        }
        Some(def) => {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                ui.painter()
                    .rect_filled(r, radius::SM, category_color(def.category));
                bf_ui_kit::caption(ui, &shared.tr(def.category.name_key()));
            });
            bf_ui_kit::heading(ui, &name(shared, def.id));
            let keys = shared.keys.keys_of(def.id).to_vec();
            ui.horizontal_wrapped(|ui| {
                if keys.is_empty() {
                    bf_ui_kit::muted(ui, &shared.tr("keys-none"));
                }
                for k in keys {
                    let label = format!("{}  ×", chord_label(k, layout, &shared.i18n));
                    let r = chip(ui, &label, false, None)
                        .on_hover_text(bf_ui_kit::tooltip_text(&shared.tr("keys-remove")));
                    if r.clicked() {
                        st.undo.push(shared.keys.config().clone());
                        shared.keys.edit(|c, _| c.unbind(&def, k));
                    }
                }
            });
            ui.horizontal(|ui| {
                let listening = st.capture == Some(Capture::Command(def.id));
                let text = if listening {
                    shared.tr("keys-press-key")
                } else {
                    shared.tr("keys-record")
                };
                let record = if listening {
                    Button::primary(&text)
                } else {
                    Button::secondary(&text)
                }
                .size(ButtonSize::Small)
                .show(ui);
                if record.clicked() {
                    st.capture = if listening {
                        None
                    } else {
                        Some(Capture::Command(def.id))
                    };
                }
                if Button::ghost(&shared.tr("keys-reset-one"))
                    .size(ButtonSize::Small)
                    .show(ui)
                    .clicked()
                {
                    st.undo.push(shared.keys.config().clone());
                    shared.keys.edit(|c, _| c.reset(def.id));
                }
            });
            bf_ui_kit::small(ui, &shared.tr("keys-assign-hint"));
        }
    }
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::divider(ui);
    files(shared, ui, st);
}

fn conflict_box(shared: &mut Shared, ui: &mut Ui, st: &mut State, p: Conflict, layout: Layout) {
    let frame = egui::Frame::new()
        .fill(color::DANGER_SOFT)
        .stroke(Stroke::new(1.0, color::DANGER))
        .corner_radius(radius::MD)
        .inner_margin(egui::Margin::same(space::MD as i8));
    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        let key = chord_label(p.chord, layout, &shared.i18n);
        let mine = shared.keys.keys_of(p.wanted_by).first().copied();
        let text = shared.i18n.tr_args(
            "keys-conflict",
            &[
                ("key", bf_i18n::Arg::String(key)),
                ("held", bf_i18n::Arg::String(name(shared, p.held_by))),
                ("wanted", bf_i18n::Arg::String(name(shared, p.wanted_by))),
            ],
        );
        ui.label(
            egui::RichText::new(text)
                .font(sans_medium(size::SMALL))
                .color(color::TEXT),
        );
        let swap = match mine {
            Some(k) => shared.i18n.tr_args(
                "keys-swap-gets",
                &[
                    ("held", bf_i18n::Arg::String(name(shared, p.held_by))),
                    (
                        "key",
                        bf_i18n::Arg::String(chord_label(k, layout, &shared.i18n)),
                    ),
                ],
            ),
            None => shared.i18n.tr_args(
                "keys-swap-none",
                &[("held", bf_i18n::Arg::String(name(shared, p.held_by)))],
            ),
        };
        bf_ui_kit::small(ui, &swap);
        ui.horizontal(|ui| {
            if Button::danger(&shared.tr("keys-swap"))
                .size(ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                let before = shared.keys.config().clone();
                let _ = shared
                    .keys
                    .edit(|c, r| c.assign(r, p.wanted_by, p.chord, true));
                st.undo.push(before);
                st.pending = None;
            }
            if Button::ghost(&shared.tr("keys-cancel"))
                .size(ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                st.pending = None;
            }
        });
    });
}

/// Export and import of the active profile.
fn files(shared: &mut Shared, ui: &mut Ui, st: &mut State) {
    bf_ui_kit::caption(ui, &shared.tr("keys-file"));
    // The file goes in the player's config folder unless a full path is typed.
    let folder = shared.paths.config.clone();
    let profile = shared.keys.config().profile.clone();
    let name = st.file.get_or_insert_with(|| format!("keys-{profile}.ron"));
    let w = ui.available_width();
    bf_ui_kit::text_field(ui, name, w);
    let path = folder.join(name.as_str());
    ui.horizontal(|ui| {
        bf_ui_kit::small(
            ui,
            &shared.i18n.tr_args(
                "keys-folder",
                &[("folder", bf_i18n::Arg::String(home_short(&folder)))],
            ),
        );
        if Button::ghost(&shared.tr("keys-open-folder"))
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
            && let Err(e) = open_folder(&folder)
        {
            st.note = Some((e, false));
        }
    });
    ui.horizontal(|ui| {
        if Button::secondary(&shared.tr("keys-export"))
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
        {
            let export = shared.keys.config().export();
            let result = ron::ser::to_string_pretty(&export, Default::default())
                .map_err(|e| e.to_string())
                .and_then(|t| std::fs::write(&path, t).map_err(|e| e.to_string()));
            st.note = Some(match result {
                Ok(()) => (shared.tr("keys-exported"), true),
                Err(e) => (e, false),
            });
        }
        if Button::secondary(&shared.tr("keys-import"))
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
        {
            let read = std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|t| ron::from_str::<Export>(&t).map_err(|e| e.to_string()));
            st.note = Some(match read {
                Ok(export) => {
                    st.undo.push(shared.keys.config().clone());
                    shared.keys.edit(|c, r| c.import(r, export));
                    (shared.tr("keys-imported"), true)
                }
                Err(e) => (e, false),
            });
        }
    });
    if let Some((text, ok)) = &st.note {
        if *ok {
            bf_ui_kit::small(ui, text);
        } else {
            bf_ui_kit::error_text(ui, text);
        }
    }
}

/// The folder with the home directory written as `~`.
fn home_short(path: &std::path::Path) -> String {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    match home.and_then(|h| path.strip_prefix(h).ok().map(std::path::Path::to_path_buf)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// Opens the folder in the system's file manager.
fn open_folder(path: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let program = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(path)
        .spawn()
        .map(drop)
        .map_err(|e| format!("{program}: {e}"))
}
