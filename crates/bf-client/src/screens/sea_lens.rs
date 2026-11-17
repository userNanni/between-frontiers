//! The sea lens over the map: the zones (tint, outline, ice and name), the straits and canals
//! (who holds each and why, whether it is open, the treaty that governs it) and a route
//! between two coastal states, with a switch on each strait of the way that redraws it closed.
//! A preview only: nothing here is a command of the game.

use std::collections::BTreeSet;

use bf_i18n::{Arg, I18n};
use bf_sim::sea::{Holders, PassageKind, Route, RouteOptions, Season};
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Ui, Vec2};

use crate::keys;
use crate::map::Camera;
use crate::map::sea::SeaLayer;
use crate::text::country_name;
use crate::world::World;

/// The lens' own state: the route asked for and the straits closed in the preview.
#[derive(Clone, Debug, Default)]
pub struct SeaView {
    /// State ids (1-based) of the route's ends.
    pub from: Option<u16>,
    pub to: Option<u16>,
    /// Straits closed in the preview.
    pub closed: BTreeSet<u16>,
    /// The strait whose card stays open (focused by its key tip).
    pub focus: Option<u16>,
    /// A state clicked that has no port, to say so.
    pub no_port: Option<u16>,
}

impl SeaView {
    /// A coastal state clicked: the start of a new route, or its end.
    pub fn pick(&mut self, layer: &SeaLayer, state: u16) {
        if !layer.graph.is_coastal(state) {
            self.no_port = Some(state);
            return;
        }
        self.no_port = None;
        match (self.from, self.to) {
            (Some(_), None) => self.to = Some(state),
            _ => {
                self.from = Some(state);
                self.to = None;
                self.closed.clear();
            }
        }
    }

    /// Forgets the route and the preview (a command calls this).
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// The options of a route on `date` (ISO) in the season of `month`.
pub fn options(date: &str, month: u8, closed: &BTreeSet<u16>) -> RouteOptions {
    let mut o = RouteOptions::open(date);
    o.closed = closed.clone();
    o.season = if matches!(month, 11 | 12 | 1 | 2 | 3 | 4) {
        Season::Winter
    } else {
        Season::Summer
    };
    o
}

pub struct Sources<'a> {
    pub i18n: &'a I18n,
    pub world: &'a World,
    pub layer: &'a SeaLayer,
    /// The overlay's tiles and the texels each covers.
    pub texture: &'a [(TextureHandle, Rect)],
    pub camera: Camera,
    /// ISO date and month of the game.
    pub date: &'a str,
    pub month: u8,
}

fn year_of(date: &str) -> String {
    date.get(..4).unwrap_or(date).to_string()
}

fn state_center(world: &World, state: u16) -> Option<Pos2> {
    let s = world.states.get(usize::from(state).checked_sub(1)?)?;
    Some(Pos2::new(s.def.center.0, s.def.center.1))
}

fn state_name(i18n: &I18n, world: &World, state: u16) -> String {
    world
        .states
        .get(usize::from(state).saturating_sub(1))
        .map_or_else(|| "—".into(), |s| crate::text::state_name(i18n, &s.def))
}

/// Who controls a passage now, and why: its holders by treaty, or the owners of its shores.
pub fn control_line(src: &Sources<'_>, strait: u16) -> String {
    let i18n = src.i18n;
    let s = &src.layer.graph.file.straits[strait as usize];
    let held = Holders::on(&src.layer.graph, src.date);
    if let Some(tags) = held.0.get(&strait).filter(|t| !t.is_empty()) {
        return s
            .holders
            .iter()
            .filter(|h| tags.contains(&h.tag))
            .map(|h| {
                i18n.tr_args(
                    "sea-held-by-treaty",
                    &[
                        ("country", Arg::String(country_name(i18n, &h.tag))),
                        ("year", Arg::String(year_of(&h.from))),
                    ],
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
    }
    let mut owners: Vec<String> = Vec::new();
    for &st in &s.shores {
        let tag = src
            .world
            .states
            .get(usize::from(st).saturating_sub(1))
            .and_then(|s| s.owner)
            .map(|c| src.world.countries[c].tag().to_owned());
        if let Some(tag) = tag {
            let name = country_name(i18n, &tag);
            if !owners.contains(&name) {
                owners.push(name);
            }
        }
    }
    i18n.tr_args(
        "sea-held-by-shores",
        &[("countries", Arg::String(owners.join(", ")))],
    )
}

/// Whether a passage is open on the date and in the preview, in words.
pub fn open_line(src: &Sources<'_>, strait: u16, closed: &BTreeSet<u16>) -> (String, bool) {
    let i18n = src.i18n;
    let s = &src.layer.graph.file.straits[strait as usize];
    if let Some(opened) = s.opened.as_deref()
        && opened > src.date
    {
        return (
            i18n.tr_args("sea-not-yet", &[("year", Arg::String(year_of(opened)))]),
            false,
        );
    }
    if closed.contains(&strait) {
        return (i18n.tr("sea-closed-preview"), false);
    }
    (i18n.tr("sea-open"), true)
}

/// The card of a passage: name and kind, who controls it, whether it is open, its treaty.
fn strait_card(ui: &mut Ui, src: &Sources<'_>, strait: u16, closed: &BTreeSet<u16>) {
    let i18n = src.i18n;
    let s = &src.layer.graph.file.straits[strait as usize];
    ui.set_max_width(340.0);
    ui.label(
        egui::RichText::new(i18n.tr(&format!("sea-strait-{}", s.key)))
            .font(sans_semibold(size::BODY))
            .color(color::TEXT),
    );
    let kind = i18n.tr(match s.kind {
        PassageKind::Canal => "sea-kind-canal",
        PassageKind::Strait => "sea-kind-strait",
    });
    let (open, is_open) = open_line(src, strait, closed);
    ui.label(
        egui::RichText::new(format!("{kind} · {open}"))
            .font(sans(size::SMALL))
            .color(if is_open {
                color::POSITIVE
            } else {
                color::DANGER
            }),
    );
    ui.label(
        egui::RichText::new(control_line(src, strait))
            .font(sans(size::SMALL))
            .color(color::TEXT),
    );
    ui.label(
        egui::RichText::new(i18n.tr(&format!("sea-regime-{}", s.key)))
            .font(sans(size::CAPTION))
            .color(color::TEXT_MUTED),
    );
}

/// What the overlay did this frame.
#[derive(Default)]
pub struct Response {
    /// A strait's switch in the route panel was flipped.
    pub toggle: Option<u16>,
    pub clear: bool,
}

/// Draws the lens over `map_rect` and the route panel over `screen`.
pub fn show(
    ui: &mut Ui,
    map_rect: Rect,
    screen: Rect,
    src: &Sources<'_>,
    view: &mut SeaView,
) -> Response {
    let mut out = Response::default();
    let layer = src.layer;
    let cam = src.camera;
    let to_screen = |t: Pos2| cam.to_screen(map_rect, t);
    let painter = ui.painter().with_clip_rect(map_rect);

    // Zones: tint, outline and ice, from the overlay.
    for (tex, texels) in src.texture {
        let on_screen = Rect::from_min_max(to_screen(texels.min), to_screen(texels.max));
        if on_screen.intersects(map_rect) {
            painter.image(
                tex.id(),
                on_screen,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }

    // Names, where the zone is big enough on screen to carry one.
    for z in &layer.graph.file.zones {
        let area = z.pixels as f32 * cam.zoom * cam.zoom;
        if area < 9_000.0 {
            continue;
        }
        let at = to_screen(Pos2::new(z.center.0, z.center.1));
        if !map_rect.contains(at) {
            continue;
        }
        let name = src.i18n.tr(&format!("sea-zone-{}", z.key));
        let font = sans(if area > 60_000.0 { 13.0 } else { 11.0 });
        for d in [Vec2::new(1.0, 1.0), Vec2::new(-1.0, -1.0)] {
            painter.text(
                at + d,
                Align2::CENTER_CENTER,
                &name,
                font.clone(),
                Color32::from_black_alpha(160),
            );
        }
        painter.text(
            at,
            Align2::CENTER_CENTER,
            &name,
            font,
            Color32::from_rgb(214, 228, 240),
        );
    }

    // The route, as a line through the zones it crosses.
    let route = match (view.from, view.to) {
        (Some(a), Some(b)) => layer
            .graph
            .route(a, b, &options(src.date, src.month, &view.closed)),
        _ => None,
    };
    if let (Some(r), Some(a), Some(b)) = (&route, view.from, view.to) {
        let mut points: Vec<Pos2> = Vec::new();
        points.extend(state_center(src.world, a));
        // Through each zone's middle, and through the strait or canal between two zones when
        // the way takes one.
        for (i, z) in r.zones.iter().enumerate() {
            if i > 0 {
                let prev = r.zones[i - 1];
                if let Some(st) = r.straits.iter().find(|&&st| {
                    let d = &layer.graph.file.straits[st as usize];
                    (d.a == prev && d.b == *z) || (d.b == prev && d.a == *z)
                }) {
                    points.push(layer.straits[*st as usize]);
                }
            }
            let z = &layer.graph.file.zones[*z as usize - 1];
            points.push(Pos2::new(z.center.0, z.center.1));
        }
        points.extend(state_center(src.world, b));
        let line: Vec<Pos2> = points.into_iter().map(to_screen).collect();
        painter.add(egui::Shape::line(
            line.clone(),
            Stroke::new(5.0, Color32::from_black_alpha(140)),
        ));
        painter.add(egui::Shape::line(
            line,
            Stroke::new(2.5, color::ACCENT_HOVER),
        ));
    }
    for s in [view.from, view.to].into_iter().flatten() {
        if let Some(c) = state_center(src.world, s) {
            painter.circle(
                to_screen(c),
                5.0,
                color::ACCENT,
                Stroke::new(1.5, Color32::BLACK),
            );
        }
    }

    // Straits and canals: a marker each, a card on hover or focus.
    let on_route: BTreeSet<u16> = route
        .as_ref()
        .map_or(BTreeSet::new(), |r| r.straits.iter().copied().collect());
    for (k, s) in layer.graph.file.straits.iter().enumerate() {
        let id = k as u16;
        let at = to_screen(layer.straits[k]);
        if !map_rect.contains(at) {
            continue;
        }
        let (_, open) = open_line(src, id, &view.closed);
        let fill = if !open {
            color::DANGER
        } else if on_route.contains(&id) {
            color::ACCENT_HOVER
        } else {
            Color32::from_rgb(235, 228, 210)
        };
        if !map_rect.contains_rect(Rect::from_center_size(at, Vec2::splat(16.0))) {
            continue;
        }
        let r = Rect::from_center_size(at, Vec2::splat(16.0));
        let resp = ui.interact(r, ui.make_persistent_id(("sea-strait", id)), Sense::click());
        let name = src.i18n.tr(&format!("sea-strait-{}", s.key));
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name.clone()));
        let rad = if resp.hovered() || view.focus == Some(id) {
            7.5
        } else {
            5.5
        };
        match s.kind {
            PassageKind::Canal => {
                painter.rect(
                    Rect::from_center_size(at, Vec2::splat(rad * 2.0)),
                    2,
                    fill,
                    Stroke::new(1.5, Color32::from_black_alpha(200)),
                    egui::StrokeKind::Inside,
                );
            }
            PassageKind::Strait => {
                painter.circle(
                    at,
                    rad,
                    fill,
                    Stroke::new(1.5, Color32::from_black_alpha(200)),
                );
            }
        }
        let typed = keys::item(ui, 'E', format!("sea.strait.{}", s.key), &s.key, r);
        if typed || resp.clicked() {
            view.focus = if view.focus == Some(id) {
                None
            } else {
                Some(id)
            };
        }
        if view.focus != Some(id) {
            resp.on_hover_ui(|ui| strait_card(ui, src, id, &view.closed));
        }
    }
    // The focused strait's card stays open beside its marker.
    if let Some(id) = view.focus {
        let at = to_screen(layer.straits[id as usize]);
        let closed = view.closed.clone();
        egui::Area::new(ui.make_persistent_id("sea-strait-card"))
            .order(egui::Order::Foreground)
            .fixed_pos(at + Vec2::new(14.0, 14.0))
            .constrain_to(screen)
            .show(ui.ctx(), |ui| {
                // Named as a window with the size it took last frame, so what it holds is
                // known to be its own.
                let mem = egui::Id::NULL.with("bf-sea-card-rect");
                if let Some(r) = ui.ctx().data(|d| d.get_temp::<Rect>(mem)) {
                    super::mark_frame(
                        ui,
                        Rect::from_min_size(ui.cursor().min, r.size()),
                        "sea-strait-card",
                    );
                }
                let r = bf_ui_kit::overlay_frame()
                    .show(ui, |ui| strait_card(ui, src, id, &closed))
                    .response
                    .rect;
                ui.ctx().data_mut(|d| d.insert_temp(mem, r));
            });
    }

    route_panel(ui, screen, src, view, route.as_ref(), &mut out);
    out
}

/// The route asked for, its length and the passages on the way, each with a switch that closes
/// it in the preview.
fn route_panel(
    ui: &mut Ui,
    screen: Rect,
    src: &Sources<'_>,
    view: &mut SeaView,
    route: Option<&Route>,
    out: &mut Response,
) {
    let i18n = src.i18n;
    if view.from.is_none() && view.no_port.is_none() {
        return;
    }
    let id = egui::Id::NULL.with("bf-sea-route");
    let height: f32 = ui.ctx().data(|d| d.get_temp(id)).unwrap_or(160.0);
    let width = 420.0f32.min(screen.width() - 2.0 * space::LG);
    let rect = Rect::from_min_size(
        Pos2::new(
            screen.center().x - width / 2.0,
            screen.bottom() - space::LG - height,
        ),
        Vec2::new(width, height),
    );
    keys::cover(ui, rect);
    super::frame(ui, rect, "sea-route");
    ui.interact(rect, id.with("block"), Sense::click_and_drag());
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(space::LG)));
    let ui = &mut child;
    ui.spacing_mut().item_spacing.y = space::XS;
    let top = ui.cursor().top();
    ui.horizontal(|ui| {
        bf_ui_kit::heading(ui, &i18n.tr("sea-route-title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let clear = Button::ghost(&i18n.tr("sea-route-clear"))
                .size(ButtonSize::Small)
                .show(ui);
            if clear.clicked() || keys::single(ui, "sea.route.clear", 'L', clear.rect) {
                out.clear = true;
            }
        });
    });
    if let Some(s) = view.no_port {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "sea-route-no-port",
                &[("state", Arg::String(state_name(src.i18n, src.world, s)))],
            ),
        );
    }
    if let Some(a) = view.from {
        let to = view
            .to
            .map_or_else(|| "…".into(), |b| state_name(src.i18n, src.world, b));
        ui.label(
            egui::RichText::new(i18n.tr_args(
                "sea-route-from-to",
                &[
                    ("from", Arg::String(state_name(src.i18n, src.world, a))),
                    ("to", Arg::String(to)),
                ],
            ))
            .font(sans_semibold(size::SMALL))
            .color(color::TEXT),
        );
    }
    match (view.to, route) {
        (None, _) => bf_ui_kit::small(ui, &i18n.tr("sea-route-pick")),
        (Some(_), None) => ui.label(
            egui::RichText::new(i18n.tr("sea-route-none"))
                .font(sans(size::SMALL))
                .color(color::DANGER),
        ),
        (Some(_), Some(r)) => bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "sea-route-km",
                &[
                    ("km", Arg::String(i18n.int(r.km as i64))),
                    ("zones", Arg::String(i18n.int(r.zones.len() as i64))),
                ],
            ),
        ),
    };
    // The passages on the way, and those closed in the preview, to open them again.
    let mut list: Vec<u16> = route.map_or(Vec::new(), |r| r.straits.clone());
    for c in &view.closed {
        if !list.contains(c) {
            list.push(*c);
        }
    }
    if view.to.is_some() && !list.is_empty() {
        bf_ui_kit::caption(ui, &i18n.tr("sea-route-straits"));
        for s in list {
            let st = &src.layer.graph.file.straits[s as usize];
            let closed = view.closed.contains(&s);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(i18n.tr(&format!("sea-strait-{}", st.key)))
                        .font(sans(size::SMALL))
                        .color(if closed { color::DANGER } else { color::TEXT }),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = i18n.tr(if closed {
                        "sea-route-reopen"
                    } else {
                        "sea-route-close"
                    });
                    let b = Button::secondary(&label).size(ButtonSize::Small).show(ui);
                    let typed =
                        keys::item(ui, 'E', format!("sea.route.{}", st.key), &st.key, b.rect);
                    if b.clicked() || typed {
                        out.toggle = Some(s);
                    }
                });
            });
        }
    }
    if options(src.date, src.month, &BTreeSet::new()).season == Season::Winter {
        bf_ui_kit::small(ui, &i18n.tr("sea-route-winter"));
    }
    ui.label(
        egui::RichText::new(i18n.tr("sea-route-preview"))
            .font(sans(size::CAPTION))
            .color(color::TEXT_MUTED),
    );
    let used = ui.min_rect().bottom() - top + space::LG * 2.0;
    ui.ctx()
        .data_mut(|d| d.insert_temp(id, used.min(screen.height() * 0.6)));
}
