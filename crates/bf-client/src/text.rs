//! Formatting helpers on top of the message bundle. No user-visible text is built elsewhere.

use bf_i18n::{Arg, I18n};
use laplace_schelling::calendar::Date;

use crate::world::{Country, Source, World};

pub fn date_long(i18n: &I18n, date: Date) -> String {
    i18n.tr_args(
        "date-long",
        &[
            ("day", Arg::Num(date.day as f64)),
            (
                "month",
                Arg::String(i18n.tr(&format!("month-{}", date.month))),
            ),
            ("year", Arg::String(date.year.to_string())),
        ],
    )
}

pub fn country_name(i18n: &I18n, tag: &str) -> String {
    i18n.tr(&format!("country-{tag}"))
}

/// A state's name in the interface language (`state-<KEY>` in `states.ftl`), or the map's.
pub fn state_name(i18n: &I18n, def: &bf_sim::scenario::StateDef) -> String {
    let id = format!("state-{}", def.key);
    if i18n.has(&id) {
        i18n.tr(&id)
    } else {
        def.name.clone()
    }
}

pub fn country_article(i18n: &I18n, tag: &str) -> String {
    i18n.attr(&format!("country-{tag}"), "article")
        .unwrap_or_default()
}

pub fn money(i18n: &I18n, value: f64) -> String {
    i18n.tr_args(
        "money-intl-dollars",
        &[("value", Arg::String(i18n.compact(value)))],
    )
}

pub fn money_exact(i18n: &I18n, value: f64) -> String {
    i18n.tr_args(
        "money-intl-dollars",
        &[("value", Arg::String(i18n.int(value.round() as i64)))],
    )
}

pub fn status_line(i18n: &I18n, world: &World, country: &Country) -> String {
    match (&country.def.status, country.overlord) {
        (bf_sim::scenario::Status::Subject { kind, .. }, Some(o)) => {
            let overlord = world.countries[o].tag();
            i18n.tr_args(
                "country-status-subject",
                &[
                    (
                        "kind",
                        Arg::String(i18n.tr(&format!("subject-kind-{kind:?}"))),
                    ),
                    ("overlord", Arg::String(country_name(i18n, overlord))),
                    ("article", Arg::String(country_article(i18n, overlord))),
                ],
            )
        }
        _ => i18n.tr("country-status-sovereign"),
    }
}

/// Difficulty from the economic rank among sovereign countries on the start date: it describes
/// the position the player took, so it does not move with the simulation.
pub fn difficulty_key(world: &World, country: &Country) -> &'static str {
    if country.is_subject() {
        return "difficulty-subject";
    }
    let own = world
        .countries
        .iter()
        .position(|c| c.tag() == country.tag())
        .map_or(0.0, |i| world.scenario.countries[i].total_gdp());
    let rank = world
        .countries
        .iter()
        .zip(&world.scenario.countries)
        .filter(|(c, f)| !c.is_subject() && f.total_gdp() > own)
        .count()
        + 1;
    match rank {
        1..=8 => "difficulty-great",
        9..=20 => "difficulty-major",
        _ => "difficulty-minor",
    }
}

/// The unit money is counted in, for the tooltips: the scenario's and the simulation's dollars
/// are not the same and never mix.
pub fn money_unit(i18n: &I18n, source: Source) -> String {
    match source {
        Source::Scenario => i18n.tr("unit-scenario"),
        Source::Live { .. } => i18n.tr("unit-live"),
    }
}

/// Where economic figures come from, with the year the simulation's GDP belongs to.
pub fn economy_source(i18n: &I18n, source: Source) -> Vec<String> {
    match source {
        Source::Scenario => vec![i18n.tr("source-economy")],
        Source::Live { gdp_year } => vec![
            i18n.tr("source-live"),
            match gdp_year {
                Some(year) => i18n.tr_args(
                    "source-live-gdp-year",
                    &[("year", Arg::String(year.to_string()))],
                ),
                None => i18n.tr("source-live-gdp-calibration"),
            },
        ],
    }
}

/// Where literacy and urbanization come from.
pub fn social_source(i18n: &I18n, source: Source) -> String {
    match source {
        Source::Scenario => i18n.tr("source-social"),
        Source::Live { .. } => i18n.tr("source-live"),
    }
}
