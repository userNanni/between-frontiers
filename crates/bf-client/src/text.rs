//! Formatting helpers on top of the message bundle. No user-visible text is built elsewhere.

use bf_i18n::{Arg, I18n};
use laplace_schelling::calendar::Date;

use crate::world::{Country, World};

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

/// Difficulty from the economic rank among sovereign countries.
pub fn difficulty_key(world: &World, country: &Country) -> &'static str {
    if country.is_subject() {
        return "difficulty-subject";
    }
    let rank = world
        .countries
        .iter()
        .filter(|c| !c.is_subject() && c.total_gdp() > country.total_gdp())
        .count()
        + 1;
    match rank {
        1..=8 => "difficulty-great",
        9..=20 => "difficulty-major",
        _ => "difficulty-minor",
    }
}
