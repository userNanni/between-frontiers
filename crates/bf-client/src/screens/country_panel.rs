//! Country summary shown in the sidebar when a country is selected on the map.

use bf_i18n::Arg;
use bf_ui_kit::{Tone, tokens::space};
use egui::Ui;

use crate::app::Shared;
use crate::text::{country_name, difficulty_key, economy_source, money, money_exact, status_line};
use crate::world::Source;

pub fn show(shared: &Shared, ui: &mut Ui, country: usize) {
    let (Some(world), i18n) = (&shared.world, &shared.i18n) else {
        return;
    };
    let c = &world.countries[country];
    // The simulation's figures in a campaign, the 1925 scenario's when choosing a country.
    let table = world.figures();
    let f = &table.countries[country];
    let tag = c.tag();
    ui.spacing_mut().item_spacing.y = space::MD;

    ui.horizontal(|ui| {
        crate::flags::flag(ui, &shared.flags, c, 32.0);
        ui.add_space(space::XS);
        bf_ui_kit::title(ui, &country_name(i18n, tag));
    });
    ui.horizontal_wrapped(|ui| {
        let status = status_line(i18n, world, c);
        bf_ui_kit::badge(
            ui,
            &status,
            if c.is_subject() {
                Tone::Neutral
            } else {
                Tone::Accent
            },
        );
        bf_ui_kit::badge(ui, &i18n.tr(difficulty_key(world, c)), Tone::Neutral);
    });
    bf_ui_kit::divider(ui);

    let half = (ui.available_width() - space::LG) / 2.0;
    egui::Grid::new("country-stats")
        .num_columns(2)
        .min_col_width(half)
        .max_col_width(half)
        .spacing([space::LG, space::LG])
        .show(ui, |ui| {
            // The figure's closed years, on hover.
            let r = ui.scope(|ui| {
                bf_ui_kit::stat_block(
                    ui,
                    &i18n.tr("stat-population"),
                    &i18n.compact(f.population),
                    Some(&i18n.tr_args(
                        "stat-rank-population",
                        &[("rank", Arg::String(f.rank_population.to_string()))],
                    )),
                );
            });
            r.response.on_hover_ui(|ui| {
                super::top_bar::figure_chart(
                    ui,
                    i18n,
                    world,
                    country,
                    super::top_bar::Figure::Population,
                )
            });
            // The figure's closed years, on hover.
            let r = ui.scope(|ui| {
                bf_ui_kit::stat_block(
                    ui,
                    &i18n.tr("stat-gdp"),
                    &money(i18n, f.gdp),
                    Some(&i18n.tr_args(
                        "stat-rank-gdp",
                        &[("rank", Arg::String(f.rank_gdp.to_string()))],
                    )),
                );
            });
            r.response.on_hover_ui(|ui| {
                super::top_bar::figure_chart(ui, i18n, world, country, super::top_bar::Figure::Gdp)
            });
            ui.end_row();
            // The figure's closed years, on hover.
            let r = ui.scope(|ui| {
                bf_ui_kit::stat_block(
                    ui,
                    &i18n.tr("stat-gdp-per-capita"),
                    &money_exact(i18n, f.gdp_per_capita()),
                    Some(&match table.source {
                        Source::Scenario => i18n.tr("stat-gdp-unit"),
                        Source::Live { .. } => i18n.tr("stat-gdp-unit-live"),
                    }),
                );
            });
            r.response.on_hover_ui(|ui| {
                super::top_bar::figure_chart(
                    ui,
                    i18n,
                    world,
                    country,
                    super::top_bar::Figure::GdpPerCapita,
                )
            });
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("stat-area"),
                &i18n.tr_args(
                    "stat-area-value",
                    &[("value", Arg::String(i18n.int(c.area_km2.round() as i64)))],
                ),
                None,
            );
            ui.end_row();
        });
    bf_ui_kit::divider(ui);

    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("stat-capital"),
        &i18n.tr(&format!("country-{tag}-capital")),
    );
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("stat-government"),
        &i18n.tr(&format!("government-{}", c.def.government)),
    );
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("stat-states"),
        &i18n.int(c.states.len() as i64),
    );
    if c.has_colonies() {
        let value = i18n.tr_args(
            "country-colonies-value",
            &[
                (
                    "population",
                    Arg::String(i18n.compact(f.colonial_population)),
                ),
                (
                    "states",
                    Arg::String(i18n.int(c.colonial_states.len() as i64)),
                ),
            ],
        );
        bf_ui_kit::stat_row(ui, &i18n.tr("country-colonies"), &value);
    }
    if !c.subjects.is_empty() {
        let names: Vec<String> = c
            .subjects
            .iter()
            .map(|s| country_name(i18n, world.countries[*s].tag()))
            .collect();
        bf_ui_kit::caption(ui, &i18n.tr("stat-subjects"));
        bf_ui_kit::small(ui, &names.join(" · "));
    }
    bf_ui_kit::gap(ui, space::XS);
    for line in economy_source(i18n, table.source) {
        bf_ui_kit::small(ui, &line);
    }
}
