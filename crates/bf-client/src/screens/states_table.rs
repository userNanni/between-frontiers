//! The economy window's states tab: every state of the player's country, metropole and colonies,
//! with the metrics of the map's state lenses and the estimated GDP; sorted by any column (again
//! flips the order); a line selects the state on the map and opens its panel.

use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, space};
use egui::{Align2, Rect, Sense, Ui, vec2};

use crate::keys;
use crate::state_metric::{self as sm, METRICS};
use crate::world::World;

const ROW: f32 = 26.0;
const NAME_MIN: f32 = 180.0;
const COL: f32 = 104.0;
const SCROLL_GUTTER: f32 = 14.0;
/// The column after the metrics: the GDP the country's figures give the state.
const GDP: usize = METRICS.len();

/// The tab's state: the column sorted by (`None`: by name) and the order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StatesUi {
    pub sort: Option<usize>,
    pub ascending: bool,
}

struct Row {
    state: usize,
    name: String,
    colony: bool,
    values: Vec<Option<f64>>,
}

fn column_key(c: usize) -> String {
    if c == GDP {
        "states-col-gdp".into()
    } else {
        format!("lens-state-{}", METRICS[c])
    }
}

/// Draws the table; the map state id (1-based) of a line clicked.
pub fn show(
    ui: &mut Ui,
    i18n: &bf_i18n::I18n,
    world: Option<&World>,
    player: Option<usize>,
    st: &mut StatesUi,
    search: &mut String,
    place: &mut super::economy::Place,
) -> Option<u16> {
    use super::economy::Place;
    let (Some(world), Some(p)) = (world, player) else {
        bf_ui_kit::muted(ui, &i18n.tr("economy-waiting"));
        return None;
    };
    let Some(view) = world.live_states.as_deref() else {
        bf_ui_kit::muted(ui, &i18n.tr("economy-waiting"));
        return None;
    };
    let country = world.countries.get(p)?;
    let areas = world.state_areas();
    let mut rows: Vec<Row> = country
        .states
        .iter()
        .map(|&s| (s, false))
        .chain(country.colonial_states.iter().map(|&s| (s, true)))
        .filter_map(|(s, colony)| {
            let f = view.states.get(s)?;
            let mut values: Vec<Option<f64>> = (0..METRICS.len())
                .map(|m| sm::value(f, areas.get(s).copied().unwrap_or(0.0), m))
                .collect();
            values.push(f.gdp);
            Some(Row {
                state: s,
                name: crate::text::state_name(i18n, &world.states.get(s)?.def),
                colony,
                values,
            })
        })
        .collect();
    // The search by name and the place, shared with the buildings tab.
    let colonies = !country.colonial_states.is_empty();
    let needle = crate::text::folded(search.trim());
    rows.retain(|r| {
        (needle.is_empty() || crate::text::folded(&r.name).contains(&needle))
            && match *place {
                Place::All => true,
                Place::Metropole => !r.colony,
                Place::Colonies => r.colony,
            }
    });
    match st.sort {
        None => rows.sort_by(|a, b| a.name.cmp(&b.name)),
        Some(c) => rows.sort_by(|a, b| {
            let v = |r: &Row| {
                r.values
                    .get(c)
                    .copied()
                    .flatten()
                    .unwrap_or(f64::NEG_INFINITY)
            };
            v(b).total_cmp(&v(a))
        }),
    }
    if st.ascending {
        rows.reverse();
    }

    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "states-basis",
            &[("n", bf_i18n::Arg::Num(rows.len() as f64))],
        ),
    );
    bf_ui_kit::gap(ui, space::SM);
    super::economy::search_and_place(ui, i18n, search, place, colonies);
    bf_ui_kit::gap(ui, space::XS);
    let width = ui.available_width() - SCROLL_GUTTER;
    let cols = GDP + 1;
    let col = ((width - NAME_MIN) / cols as f32).clamp(64.0, COL);
    let name_w = width - col * cols as f32;
    let x_of = |rect: Rect, c: usize| rect.left() + name_w + col * (c as f32 + 1.0);

    // The header: each column sorts; again flips the order.
    let (head, _) = ui.allocate_exact_size(vec2(width, ROW + 10.0), Sense::hover());
    for c in 0..=cols {
        let (label, sort, left, right) = if c == 0 {
            (
                i18n.tr("states-col-state"),
                None,
                head.left(),
                head.left() + name_w,
            )
        } else {
            let k = c - 1;
            (
                i18n.tr(&column_key(k)),
                Some(k),
                x_of(head, k) - col,
                x_of(head, k),
            )
        };
        let r = Rect::from_x_y_ranges(left..=right, head.y_range());
        let resp = ui.interact(r, ui.make_persistent_id(("states-col", c)), Sense::click());
        let name = label.clone();
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ColumnHeader, true, &name));
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
        let typed = keys::item(ui, 'O', format!("states.sort.{c}"), &c.to_string(), r);
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

    let mut goto = None;
    egui::ScrollArea::vertical()
        .id_salt("states-table")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            super::scroll_keys(ui);
            for (j, row) in rows.iter().enumerate() {
                // "(colony)" only where metropole and colonies are listed together; the line grows
                // with a name that wraps.
                let name = if row.colony && *place == Place::All {
                    i18n.tr_args(
                        "states-colony",
                        &[("state", bf_i18n::Arg::String(row.name.clone()))],
                    )
                } else {
                    row.name.clone()
                };
                let galley = ui
                    .painter()
                    .layout(name, sans(13.0), color::TEXT, name_w - space::SM);
                let height = ROW.max(galley.size().y + 8.0);
                let (r, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(r, 4.0, color::SURFACE_HOVER);
                }
                ui.painter().galley(
                    egui::pos2(r.left(), r.center().y - galley.size().y / 2.0),
                    galley,
                    color::TEXT,
                );
                for (c, v) in row.values.iter().enumerate() {
                    let text = match v {
                        Some(v) if c == GDP => crate::text::money_short(i18n, *v),
                        Some(v) => sm::text(i18n, c, *v),
                        None => "—".into(),
                    };
                    ui.painter().text(
                        egui::pos2(x_of(r, c), r.center().y),
                        Align2::RIGHT_CENTER,
                        text,
                        sans(13.0),
                        color::TEXT,
                    );
                }
                let label = row.name.clone();
                resp.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, label.clone())
                });
                let resp = resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(i18n.tr("economy-goto-state"));
                let typed = keys::item(
                    ui,
                    'T',
                    format!("states.row.{}", row.state),
                    &(j + 1).to_string(),
                    r,
                );
                if resp.clicked() || typed {
                    goto = u16::try_from(row.state + 1).ok();
                }
            }
        });
    goto
}
