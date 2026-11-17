//! Country summary shown in the sidebar when a country is selected on the map.

use bf_i18n::Arg;
use bf_ui_kit::{Tone, tokens::space};
use egui::Ui;

use crate::app::Shared;
use crate::text::{country_name, difficulty_key, economy_source, money, money_exact, status_line};
use crate::world::Source;

/// What the player asked for in the panel.
pub enum Action {
    /// Play this country from now on.
    Switch(String),
    /// Free one of the player's states (`StateDef::key`), independent or as a subject.
    Liberate {
        by: String,
        state: String,
        subject: bool,
    },
}

/// `player`: the played country; `selected_state`: the state clicked last.
pub fn show(
    shared: &Shared,
    ui: &mut Ui,
    country: usize,
    player: Option<usize>,
    selected_state: Option<usize>,
) -> Option<Action> {
    let (Some(world), i18n) = (&shared.world, &shared.i18n) else {
        return None;
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
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("stat-population"),
                &i18n.compact(f.population),
                Some(&i18n.tr_args(
                    "stat-rank-population",
                    &[("rank", Arg::String(f.rank_population.to_string()))],
                )),
            );
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("stat-gdp"),
                &money(i18n, f.gdp),
                Some(&i18n.tr_args(
                    "stat-rank-gdp",
                    &[("rank", Arg::String(f.rank_gdp.to_string()))],
                )),
            );
            ui.end_row();
            bf_ui_kit::stat_block(
                ui,
                &i18n.tr("stat-gdp-per-capita"),
                &money_exact(i18n, f.gdp_per_capita()),
                Some(&match table.source {
                    Source::Scenario => i18n.tr("stat-gdp-unit"),
                    Source::Live { .. } => i18n.tr("stat-gdp-unit-live"),
                }),
            );
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
    borders(shared, ui, country, player, selected_state)
}

/// Play another country, or free a state of one's own (`docs/borders.pt-BR.md`, decisions 3
/// and 4). Only in a campaign: the scenario has no player yet.
fn borders(
    shared: &Shared,
    ui: &mut Ui,
    country: usize,
    player: Option<usize>,
    selected_state: Option<usize>,
) -> Option<Action> {
    use crate::keys::{self, commands::id};
    use bf_ui_kit::{Button, ButtonSize};
    let (world, i18n) = (shared.world.as_ref()?, &shared.i18n);
    world.live.as_ref()?;
    let player = player?;
    let c = &world.countries[country];
    if c.states.is_empty() && c.colonial_states.is_empty() {
        return None;
    }
    bf_ui_kit::divider(ui);
    bf_ui_kit::caption(ui, &i18n.tr("borders-title"));
    let mut out = None;
    if country != player {
        let label = i18n.tr_args(
            "borders-switch",
            &[("country", Arg::String(country_name(i18n, c.tag())))],
        );
        let detail = i18n.tr("borders-switch-detail");
        let b = Button::secondary(&label)
            .detail(Some(&detail))
            .size(ButtonSize::Small)
            .show(ui);
        let b = b.on_hover_text(keys::tip(ui.ctx(), &label, id::SWITCH_COUNTRY));
        if keys::pressed(ui, id::SWITCH_COUNTRY, &b) || keys::fired(ui.ctx(), id::SWITCH_COUNTRY) {
            out = Some(Action::Switch(c.tag().to_owned()));
        }
        return out;
    }
    let mine = selected_state
        .and_then(|s| world.states.get(s))
        .filter(|s| s.owner == Some(player));
    let held = c.states.len() + c.colonial_states.len();
    let Some(st) = mine.filter(|_| held > 1) else {
        bf_ui_kit::small(ui, &i18n.tr("borders-select-state"));
        return None;
    };
    let name = crate::text::state_name(i18n, &st.def);
    for (cmd, key, subject) in [
        (id::LIBERATE, "borders-liberate", false),
        (id::LIBERATE_SUBJECT, "borders-liberate-subject", true),
    ] {
        let label = i18n.tr_args(key, &[("state", Arg::String(name.clone()))]);
        let b = Button::secondary(&label).size(ButtonSize::Small).show(ui);
        let b = b.on_hover_text(keys::tip(ui.ctx(), &label, cmd));
        if keys::pressed(ui, cmd, &b) || keys::fired(ui.ctx(), cmd) {
            out = Some(Action::Liberate {
                by: c.tag().to_owned(),
                state: st.def.key.clone(),
                subject,
            });
        }
    }
    bf_ui_kit::small(ui, &i18n.tr("borders-liberate-detail"));
    out
}
