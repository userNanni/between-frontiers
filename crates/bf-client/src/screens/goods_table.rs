//! The economy window's goods tab: the player's market good by good (`goods_view`), with the
//! price against the base, what the country makes and uses, imports, exports and the stock of a
//! storable good; filtered by category and by name, sorted by any column. A good clicked opens
//! who makes and who takes it, and "See in Buildings" goes to the buildings tab on a maker.

use bf_i18n::{Arg, I18n};
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align2, Rect, Sense, Ui, vec2};

use crate::goods_view::{GoodRow, GoodsView};
use crate::keys;
use crate::text::money_short;

const ROW: f32 = 26.0;
const NAME_MIN: f32 = 170.0;
const COL: f32 = 104.0;
const SCROLL_GUTTER: f32 = 14.0;
/// Columns after the name, by their message key.
const COLUMNS: [&str; 6] = [
    "goods-col-price",
    "goods-col-made",
    "goods-col-used",
    "goods-col-imports",
    "goods-col-exports",
    "goods-col-stock",
];

/// The tab's state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GoodsUi {
    /// Column sorted by (`None`: by name); smallest first.
    pub sort: Option<usize>,
    pub ascending: bool,
    pub category: Option<String>,
    pub query: String,
    /// The good opened on who makes and takes it.
    pub open: Option<u16>,
}

fn value(r: &GoodRow, c: usize) -> Option<f64> {
    match c {
        0 => Some(r.price),
        1 => Some(r.made),
        2 => Some(r.used),
        3 => Some(r.imports),
        4 => Some(r.exports),
        _ => r.stock,
    }
}

fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'ô' | 'õ' => 'o',
            'ú' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

fn pct(i18n: &I18n, share: f64) -> String {
    crate::text::pct_text(i18n, i18n.decimal(share * 100.0, 0))
}

/// Draws the tab; a building (catalogue index) to show in the buildings tab.
pub fn show(ui: &mut Ui, i18n: &I18n, v: &GoodsView, st: &mut GoodsUi) -> Option<u16> {
    let mut goto = None;
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &i18n.tr("goods-basis"));
    bf_ui_kit::gap(ui, space::SM);

    // Categories, then the search by name.
    let mut categories: Vec<&str> = Vec::new();
    for g in &v.goods {
        if !categories.contains(&g.category.as_str()) {
            categories.push(&g.category);
        }
    }
    ui.horizontal_wrapped(|ui| {
        let mut chip = |ui: &mut Ui, label: String, cat: Option<&str>, key: String| {
            let on = st.category.as_deref() == cat;
            let b = if on {
                Button::primary(&label)
            } else {
                Button::secondary(&label)
            }
            .size(ButtonSize::Small)
            .show(ui);
            let id = format!("goods.category.{}", cat.unwrap_or("all"));
            if b.clicked() || keys::item(ui, 'S', id, &key, b.rect) {
                st.category = cat.map(str::to_owned);
            }
        };
        chip(ui, i18n.tr("goods-category-all"), None, "0".into());
        for (k, c) in categories.iter().enumerate() {
            chip(
                ui,
                i18n.tr(&format!("good-category-{c}")),
                Some(c),
                (k + 1).to_string(),
            );
        }
    });
    ui.horizontal(|ui| {
        bf_ui_kit::small(ui, &i18n.tr("goods-search"));
        bf_ui_kit::text_field(ui, &mut st.query, 220.0);
    });
    bf_ui_kit::gap(ui, space::SM);

    let name = |g: &GoodRow| i18n.tr(&format!("good-{}", g.key));
    let q = fold(st.query.trim());
    let mut rows: Vec<&GoodRow> = v
        .goods
        .iter()
        .filter(|g| st.category.as_deref().is_none_or(|c| c == g.category))
        .filter(|g| q.is_empty() || fold(&name(g)).contains(&q))
        .collect();
    match st.sort {
        None => rows.sort_by_key(|g| name(g)),
        Some(c) => rows.sort_by(|a, b| {
            let v = |r: &GoodRow| value(r, c).unwrap_or(f64::NEG_INFINITY);
            v(b).total_cmp(&v(a))
        }),
    }
    if st.ascending {
        rows.reverse();
    }

    // The good opened: who makes it and who takes it.
    if let Some(open) = st.open
        && let Some(g) = v.goods.iter().find(|g| g.good == open)
    {
        goto = detail(ui, i18n, g, st);
        bf_ui_kit::gap(ui, space::SM);
    }

    let width = ui.available_width() - SCROLL_GUTTER;
    let col = ((width - NAME_MIN) / COLUMNS.len() as f32).clamp(70.0, COL);
    let name_w = width - col * COLUMNS.len() as f32;
    let x_of = |rect: Rect, c: usize| rect.left() + name_w + col * (c as f32 + 1.0);

    // The header: each column sorts; again flips the order.
    let (head, _) = ui.allocate_exact_size(vec2(width, ROW + 10.0), Sense::hover());
    for c in 0..=COLUMNS.len() {
        let (label, sort, left, right) = if c == 0 {
            (
                i18n.tr("goods-col-good"),
                None,
                head.left(),
                head.left() + name_w,
            )
        } else {
            let k = c - 1;
            (
                i18n.tr(COLUMNS[k]),
                Some(k),
                x_of(head, k) - col,
                x_of(head, k),
            )
        };
        let r = Rect::from_x_y_ranges(left..=right, head.y_range());
        let resp = ui.interact(r, ui.make_persistent_id(("goods-col", c)), Sense::click());
        let named = label.clone();
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ColumnHeader, true, &named));
        let on = st.sort == sort;
        let arrow = if on {
            if st.ascending { " ↑" } else { " ↓" }
        } else {
            ""
        };
        let galley = ui.painter().layout(
            format!("{label}{arrow}"),
            sans_semibold(11.5),
            if on || resp.hovered() {
                color::ACCENT
            } else {
                color::TEXT_FAINT
            },
            r.width() - 6.0,
        );
        let pos = if c == 0 {
            egui::pos2(r.left(), r.bottom() - galley.size().y)
        } else {
            egui::pos2(r.right() - galley.size().x, r.bottom() - galley.size().y)
        };
        ui.painter().galley(pos, galley, color::TEXT);
        let typed = keys::item(ui, 'O', format!("goods.sort.{c}"), &c.to_string(), r);
        if resp.clicked() || typed {
            if on {
                st.ascending = !st.ascending;
            } else {
                st.sort = sort;
                st.ascending = sort.is_none();
            }
        }
    }
    bf_ui_kit::divider(ui);

    egui::ScrollArea::vertical()
        .id_salt("goods-table")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            super::scroll_keys(ui);
            if rows.is_empty() {
                bf_ui_kit::muted(ui, &i18n.tr("goods-none"));
            }
            for (j, g) in rows.iter().enumerate() {
                let (r, resp) = ui.allocate_exact_size(vec2(width, ROW), Sense::click());
                let open = st.open == Some(g.good);
                if open || resp.hovered() {
                    ui.painter().rect_filled(
                        r,
                        4.0,
                        if open {
                            color::ACCENT_SOFT
                        } else {
                            color::SURFACE_HOVER
                        },
                    );
                }
                let label = name(g);
                let galley =
                    ui.painter()
                        .layout(label.clone(), sans(13.0), color::TEXT, name_w - space::SM);
                ui.painter().galley(
                    egui::pos2(r.left(), r.center().y - galley.size().y / 2.0),
                    galley,
                    color::TEXT,
                );
                for c in 0..COLUMNS.len() {
                    let (text, tone) = match (c, value(g, c)) {
                        (0, Some(p)) => (
                            pct(i18n, p),
                            if p > 1.25 {
                                color::DANGER
                            } else if p < 0.8 {
                                color::POSITIVE
                            } else {
                                color::TEXT
                            },
                        ),
                        (_, Some(x)) if x > 0.0 => (money_short(i18n, x), color::TEXT),
                        (_, Some(_)) => ("—".into(), color::TEXT_FAINT),
                        (_, None) => (String::new(), color::TEXT_FAINT),
                    };
                    ui.painter().text(
                        egui::pos2(x_of(r, c), r.center().y),
                        Align2::RIGHT_CENTER,
                        text,
                        sans(13.0),
                        tone,
                    );
                }
                resp.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, label.clone())
                });
                let tip = i18n.tr_args(
                    "goods-row-tip",
                    &[
                        ("price", Arg::String(pct(i18n, g.price))),
                        ("world", Arg::String(pct(i18n, g.world))),
                    ],
                );
                let resp = resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(tip);
                let typed = keys::item(
                    ui,
                    'T',
                    format!("goods.row.{}", g.key),
                    &(j + 1).to_string(),
                    r,
                );
                if resp.clicked() || typed {
                    st.open = if open { None } else { Some(g.good) };
                }
            }
        });
    goto
}

/// The good opened: the buildings that make it, each with a way to the buildings tab, and those
/// that take it.
fn detail(ui: &mut Ui, i18n: &I18n, g: &GoodRow, st: &mut GoodsUi) -> Option<u16> {
    let mut goto = None;
    let name = i18n.tr(&format!("good-{}", g.key));
    egui::Frame::new()
        .fill(color::SURFACE_SUNKEN)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::same(space::MD as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                bf_ui_kit::heading(ui, &name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close = Button::ghost(&i18n.tr("goods-close"))
                        .size(ButtonSize::Small)
                        .show(ui);
                    if close.clicked() || keys::single(ui, "goods.close", 'X', close.rect) {
                        st.open = None;
                    }
                });
            });
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "goods-row-tip",
                    &[
                        ("price", Arg::String(pct(i18n, g.price))),
                        ("world", Arg::String(pct(i18n, g.world))),
                    ],
                ),
            );
            bf_ui_kit::caption(ui, &i18n.tr("goods-makers"));
            if g.makers.is_empty() {
                bf_ui_kit::small(ui, &i18n.tr("goods-makers-none"));
            }
            ui.horizontal_wrapped(|ui| {
                for (k, (b, key)) in g.makers.iter().enumerate() {
                    let label = i18n.tr_args(
                        "goods-see-in-buildings",
                        &[("building", Arg::String(i18n.tr(&format!("building-{key}"))))],
                    );
                    let b_resp = Button::secondary(&label).size(ButtonSize::Small).show(ui);
                    if b_resp.clicked()
                        || keys::item(
                            ui,
                            'M',
                            format!("goods.maker.{b}"),
                            &(k + 1).to_string(),
                            b_resp.rect,
                        )
                    {
                        goto = Some(*b);
                    }
                }
            });
            bf_ui_kit::caption(ui, &i18n.tr("goods-takers"));
            let takers: Vec<String> = g
                .takers
                .iter()
                .map(|k| i18n.tr(&format!("building-{k}")))
                .collect();
            bf_ui_kit::small(
                ui,
                &if takers.is_empty() {
                    i18n.tr("goods-takers-none")
                } else {
                    takers.join(", ")
                },
            );
        });
    goto
}
