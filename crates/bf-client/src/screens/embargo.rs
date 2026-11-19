//! Embargoes, in the journal (owner's decision CT6; `docs/trade.pt-BR.md`): a rare decision,
//! outside the trade tab. Imposing one picks a country by name and says in words what it cuts
//! (last year's trade with it both ways) before the second step; a restore point is written
//! before the command goes. Below, the embargoes in force by the country and on it; the
//! country's own lift in two steps too (no restore point: lifting gives back what imposing took,
//! slowly).

use bf_i18n::{Arg, I18n};
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Ui};

use super::journal::Response;
use crate::keys;
use crate::tariff_view::TariffView;
use crate::text::{country_article, country_name, money};
use crate::world::World;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// The decision's state: the search, the country picked (at the confirmation step), the
/// embargo whose lifting is asked (its target), and the last order sent until the simulation
/// shows it (target, imposed).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmbargoUi {
    pub find: super::country_find::FindUi,
    pub target: Option<String>,
    pub lift: Option<String>,
    pub sent: Option<(String, bool)>,
}

/// The decision's button in the Agendas: "Embargo comercial (n em vigor)".
pub fn open_label(i18n: &I18n, view: &TariffView) -> String {
    i18n.tr_args(
        "embargo-open",
        &[("n", arg(view.embargoes.len().to_string()))],
    )
}

fn wrapped(ui: &mut Ui, text: String, tint: egui::Color32) {
    ui.add(egui::Label::new(RichText::new(text).font(sans(size::SMALL)).color(tint)).wrap());
}

/// What the country sold to `tag` and bought from it in the view's year.
fn trade_with(view: &TariffView, tag: &str) -> (f64, f64) {
    view.partners
        .iter()
        .find(|p| p.tag == tag)
        .map_or((0.0, 0.0), |p| (p.exports, p.imports))
}

/// The trade an embargo on `tag` cuts, in words: "No ano passado o país vendeu X à Argentina e
/// comprou Y dela."
fn cut_line(i18n: &I18n, view: &TariffView, tag: &str) -> String {
    let (sold, bought) = trade_with(view, tag);
    i18n.tr_args(
        if view.so_far {
            "embargo-trade-so-far"
        } else {
            "embargo-trade"
        },
        &[
            ("country", arg(country_name(i18n, tag))),
            ("article", arg(country_article(i18n, tag))),
            ("sold", arg(nbsp(money(i18n, sold)))),
            ("bought", arg(nbsp(money(i18n, bought)))),
        ],
    )
}

/// A number never breaks across lines.
fn nbsp(s: String) -> String {
    s.replace(' ', "\u{a0}")
}

pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    view: &TariffView,
    st: &mut EmbargoUi,
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::SM;
    bf_ui_kit::title(ui, &i18n.tr("embargo-title"));
    bf_ui_kit::small(ui, &i18n.tr("embargo-basis"));
    // The order sent shows until the simulation has taken it.
    if let Some((target, on)) = st.sent.clone() {
        let mine = |(by, t): &(String, String)| *by == view.tag && *t == target;
        if view.embargoes.iter().any(mine) == on {
            st.sent = None;
        } else {
            let args = [
                ("country", arg(country_name(i18n, &target))),
                ("article", arg(country_article(i18n, &target))),
            ];
            let key = if on {
                "embargo-sent"
            } else {
                "embargo-lift-sent"
            };
            wrapped(ui, i18n.tr_args(key, &args), color::ACCENT);
        }
    }
    // Impose: a country by name, then what it does in words.
    bf_ui_kit::heading(ui, &i18n.tr("embargo-impose"));
    match st.target.clone() {
        None => {
            if let Some(w) = world
                && let Some(tag) =
                    super::country_find::show(ui, i18n, w, &view.tag, &mut st.find, "embargo", 'C')
            {
                st.target = Some(tag);
                st.lift = None;
            }
        }
        Some(target) => confirm(ui, i18n, view, &target, st, out),
    }
    // In force: by the country, each with its lift, and on it.
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "embargo-in-force",
            &[("n", arg(view.embargoes.len().to_string()))],
        ),
    );
    if view.embargoes.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("embargo-none"));
    }
    for (k, (by, on)) in view.embargoes.iter().enumerate() {
        bf_ui_kit::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::XS;
            let mine = *by == view.tag;
            let other = if mine { on } else { by };
            let line = i18n.tr_args(
                if mine {
                    "embargo-by-us"
                } else {
                    "embargo-on-us"
                },
                &[
                    ("country", arg(country_name(i18n, other))),
                    ("article", arg(country_article(i18n, other))),
                ],
            );
            ui.strong(line);
            if mine {
                lift(ui, i18n, on, k, st, out);
            } else {
                bf_ui_kit::small(ui, &i18n.tr("embargo-on-us-note"));
            }
        });
    }
}

/// The second step of imposing: what it does, and the two buttons.
fn confirm(
    ui: &mut Ui,
    i18n: &I18n,
    view: &TariffView,
    target: &str,
    st: &mut EmbargoUi,
    out: &mut Response,
) {
    let args = [
        ("country", arg(country_name(i18n, target))),
        ("article", arg(country_article(i18n, target))),
    ];
    let already = view
        .embargoes
        .iter()
        .any(|(by, on)| *by == view.tag && on == target);
    bf_ui_kit::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.strong(i18n.tr_args("embargo-on", &args));
        if already {
            bf_ui_kit::small(ui, &i18n.tr_args("embargo-already", &args));
        } else {
            wrapped(ui, i18n.tr_args("embargo-explain", &args), color::DANGER);
            wrapped(ui, cut_line(i18n, view, target), color::TEXT);
            wrapped(ui, i18n.tr("embargo-lifted-slowly"), color::TEXT_MUTED);
            bf_ui_kit::small(ui, &i18n.tr("embargo-restore-point"));
        }
        ui.horizontal_wrapped(|ui| {
            if !already {
                let yes = Button::danger(&i18n.tr("embargo-yes"))
                    .size(ButtonSize::Medium)
                    .show(ui);
                if yes.clicked() || keys::single(ui, "embargo.yes", 'Y', yes.rect) {
                    out.embargo = Some((target.to_owned(), true));
                    *st = EmbargoUi {
                        sent: Some((target.to_owned(), true)),
                        ..EmbargoUi::default()
                    };
                    return;
                }
            }
            let no = Button::ghost(&i18n.tr("embargo-no"))
                .size(ButtonSize::Medium)
                .show(ui);
            if no.clicked() || keys::single(ui, "embargo.no", 'N', no.rect) {
                st.target = None;
            }
        });
    });
}

/// "Levantar…", then what lifting does and the two buttons.
fn lift(ui: &mut Ui, i18n: &I18n, on: &str, k: usize, st: &mut EmbargoUi, out: &mut Response) {
    if st.lift.as_deref() != Some(on) {
        let b = Button::ghost(&i18n.tr("embargo-lift"))
            .size(ButtonSize::Small)
            .show(ui);
        if b.clicked()
            || keys::item(
                ui,
                'L',
                format!("embargo.lift.{on}"),
                &(k + 1).to_string(),
                b.rect,
            )
        {
            st.lift = Some(on.to_owned());
            st.target = None;
        }
        return;
    }
    wrapped(
        ui,
        i18n.tr_args(
            "embargo-lift-explain",
            &[
                ("country", arg(country_name(i18n, on))),
                ("article", arg(country_article(i18n, on))),
            ],
        ),
        color::TEXT,
    );
    ui.horizontal_wrapped(|ui| {
        let yes = Button::primary(&i18n.tr("embargo-lift-yes"))
            .size(ButtonSize::Medium)
            .show(ui);
        if yes.clicked() || keys::single(ui, "embargo.lift.yes", 'Y', yes.rect) {
            out.embargo = Some((on.to_owned(), false));
            st.lift = None;
            st.sent = Some((on.to_owned(), false));
            return;
        }
        let no = Button::ghost(&i18n.tr("embargo-lift-no"))
            .size(ButtonSize::Medium)
            .show(ui);
        if no.clicked() || keys::single(ui, "embargo.lift.no", 'N', no.rect) {
            st.lift = None;
        }
    });
}
