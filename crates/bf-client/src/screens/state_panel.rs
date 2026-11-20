//! The state panel: a state clicked on the map, with its country above it (a click on the
//! country, or Esc, goes up to the country panel). Its people, jobs, income and GDP share,
//! schooling and towns; the buildings it has with their levels and workers; the works under way
//! there; and the deposits known. Read only, from the monthly figures of every state.

use bf_i18n::{Arg, I18n};
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Ui};

use crate::flags::Flags;
use crate::keys;
use crate::state_view::StateFigures;
use crate::text::{country_name, money};
use crate::world::World;

/// How many building kinds the panel lists before "and n more".
const PLANTS_SHOWN: usize = 8;

#[derive(Default)]
pub struct Response {
    /// Go up to the country panel.
    pub to_country: bool,
}

fn pct(i18n: &I18n, x: f64) -> String {
    crate::text::pct_text(i18n, i18n.decimal(x * 100.0, 0))
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    world: &World,
    flags: &Flags,
    state: usize,
    figures: Option<&StateFigures>,
    year: i32,
) -> Response {
    let mut out = Response::default();
    let Some(st) = world.states.get(state) else {
        return out;
    };
    ui.spacing_mut().item_spacing.y = space::MD;

    // The country above: a click goes up.
    if let Some(c) = st.owner.and_then(|o| world.countries.get(o)) {
        ui.horizontal(|ui| {
            crate::flags::flag(ui, flags, c, 18.0);
            let label = i18n.tr_args(
                "state-panel-country",
                &[("country", Arg::String(country_name(i18n, c.tag())))],
            );
            let b = Button::ghost(&label).size(ButtonSize::Small).show(ui);
            let tip = keys::tip(ui.ctx(), &i18n.tr("state-panel-up"), keys::id::BACK);
            let b = b.on_hover_text(tip);
            if b.clicked() || keys::single(ui, "state.country", 'P', b.rect) {
                out.to_country = true;
            }
        });
    }
    bf_ui_kit::title(ui, &crate::text::state_name(i18n, &st.def));
    let Some(f) = figures.filter(|f| f.known) else {
        bf_ui_kit::muted(ui, &i18n.tr("state-panel-none"));
        return out;
    };
    if f.colony {
        bf_ui_kit::badge(ui, &i18n.tr("state-panel-colony"), bf_ui_kit::Tone::Neutral);
    }
    bf_ui_kit::divider(ui);

    let half = (ui.available_width() - space::LG) / 2.0;
    egui::Grid::new("state-stats")
        .num_columns(2)
        .min_col_width(half)
        .max_col_width(half)
        .spacing([space::LG, space::LG])
        .show(ui, |ui| {
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("stat-population"),
                &i18n.compact(f.population as f64),
                None,
            );
            let gdp = f.gdp.map_or_else(|| "—".into(), |g| money(i18n, g));
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("state-panel-gdp"),
                &gdp,
                Some(&i18n.tr("state-panel-gdp-note")),
            );
            ui.end_row();
            let income = f
                .income_per_head()
                .map_or_else(|| "—".into(), |x| crate::text::money_exact(i18n, x));
            bf_ui_kit::stat_block(ui, &i18n.tr("state-panel-income"), &income, None);
            let employed = i18n.tr_args(
                "state-panel-employed-share",
                &[(
                    "share",
                    Arg::String(
                        f.employed_share()
                            .map_or_else(|| "—".into(), |x| pct(i18n, x)),
                    ),
                )],
            );
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("state-panel-employed"),
                &i18n.compact(f.employed as f64),
                Some(&employed),
            );
            ui.end_row();
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("state-panel-subsistence"),
                &i18n.compact(f.subsistence as f64),
                Some(&i18n.tr("state-panel-subsistence-note")),
            );
            let jobless = f
                .unemployment()
                .map_or_else(|| "—".into(), |x| pct(i18n, x));
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("state-panel-unemployment"),
                &jobless,
                Some(&i18n.tr_args(
                    "state-panel-unemployment-note",
                    &[("n", Arg::String(i18n.compact(f.jobless as f64)))],
                )),
            );
            ui.end_row();
            bf_ui_kit::stat_block(ui, &i18n.tr("bar-literacy"), &pct(i18n, f.literacy), None);
            bf_ui_kit::stat_block(ui, &i18n.tr("state-panel-urban"), &pct(i18n, f.urban), None);
            ui.end_row();
        });
    bf_ui_kit::divider(ui);

    // Buildings.
    bf_ui_kit::caption(
        ui,
        &i18n.tr_args(
            "state-panel-buildings",
            &[
                ("levels", Arg::String(i18n.int(f.levels()))),
                ("output", Arg::String(money(i18n, f.output))),
                ("state", Arg::String(i18n.int(f.state_levels()))),
            ],
        ),
    );
    if f.plants.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("state-panel-no-buildings"));
    }
    ui.spacing_mut().item_spacing.y = space::XS;
    // A kind per two lines: the panel is narrow and names run long.
    for p in f.plants.iter().take(PLANTS_SHOWN) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(
                RichText::new({
                    let name = i18n.tr(&format!("building-{}", p.key));
                    if p.state_firm {
                        i18n.tr_args(
                            "economy-state-firm-name",
                            &[("building", Arg::String(name))],
                        )
                    } else {
                        name
                    }
                })
                .font(bf_ui_kit::theme::sans(size::SMALL))
                .color(color::TEXT),
            );
            ui.label(
                RichText::new(i18n.tr_args(
                    "state-panel-plant",
                    &[
                        ("levels", Arg::String(p.levels.to_string())),
                        ("workers", Arg::String(i18n.compact_short(p.workers))),
                    ],
                ))
                .font(bf_ui_kit::theme::sans(size::SMALL))
                .color(color::TEXT_MUTED),
            );
        });
    }
    if f.plants.len() > PLANTS_SHOWN {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "state-panel-more",
                &[(
                    "n",
                    Arg::String((f.plants.len() - PLANTS_SHOWN).to_string()),
                )],
            ),
        );
    }
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "state-panel-works",
            &[
                ("public", Arg::String(f.works[0].to_string())),
                ("private", Arg::String(f.works[1].to_string())),
            ],
        ),
    );
    ui.spacing_mut().item_spacing.y = space::MD;
    bf_ui_kit::divider(ui);

    // Deposits known by now.
    bf_ui_kit::caption(ui, &i18n.tr("state-panel-deposits"));
    let mut known: Vec<&bf_sim::resources::Deposit> = world
        .deposits
        .get(state)
        .map(|d| d.iter().filter(|d| d.discovered <= year).collect())
        .unwrap_or_default();
    known.sort_by(|a, b| b.size.cmp(&a.size).then(a.resource.cmp(&b.resource)));
    if known.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("state-panel-no-deposits"));
    } else {
        let line: Vec<String> = known
            .iter()
            .map(|d| {
                i18n.tr_args(
                    "state-panel-deposit",
                    &[
                        (
                            "resource",
                            Arg::String(i18n.tr(&format!("resource-{}", d.resource))),
                        ),
                        ("size", Arg::String(d.size.to_string())),
                    ],
                )
            })
            .collect();
        bf_ui_kit::small(ui, &line.join(" · "));
    }
    out
}
