//! Government tab: debts between governments. What the player owes other governments and what
//! they owe it, line by line, with where each stands; the selected line in detail. Read only:
//! renegotiating goes through the negotiating table in the Agendas when treaties reach it.

use bf_i18n::{Arg, I18n};
use bf_sim::treaty::{Currency, Pledge};
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, radius, size, space};
use egui::{Rect, RichText, Sense, Ui, vec2};

use super::government::TabContext;
use crate::keys;
use crate::ledger_view::{LedgerView, Line, Standing};
use crate::text::{country_name, money};

const ROW: f32 = 30.0;

pub fn treaty_name(i18n: &I18n, key: &str) -> String {
    let id = format!("treaty-{key}");
    if i18n.has(&id) {
        i18n.tr(&id)
    } else {
        key.to_owned()
    }
}

/// "US$ 59 bi": a sum for a narrow column.
fn money_short(i18n: &I18n, x: f64) -> String {
    i18n.tr_args(
        "money-intl-dollars",
        &[("value", Arg::String(i18n.compact_short(x)))],
    )
}

fn pct(i18n: &I18n, x: f64) -> String {
    format!("{} %", i18n.decimal(x * 100.0, 1))
}

fn standing_color(s: Standing) -> egui::Color32 {
    match s {
        Standing::Current => color::POSITIVE,
        Standing::Arrears | Standing::Broken => color::DANGER,
        Standing::Moratorium => color::ACCENT,
    }
}

/// A label confined to `rect`, cut with an ellipsis (the whole text on hover).
fn cell(ui: &mut Ui, rect: Rect, text: RichText, right: bool) -> egui::Response {
    let layout = if right {
        egui::Layout::right_to_left(egui::Align::Center)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    };
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect).layout(layout), |ui| {
        ui.add(egui::Label::new(text).truncate())
    })
    .inner
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(v) = cx.snap.ledger.as_deref() else {
        bf_ui_kit::muted(ui, &i18n.tr(super::government::waiting_key(cx.snap)));
        return;
    };
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &i18n.tr("ledger-basis"));
    bf_ui_kit::gap(ui, space::SM);
    if v.lines.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("ledger-none"));
        return;
    }
    if cx
        .ledger
        .selected
        .is_none_or(|id| !v.lines.iter().any(|l| l.id == id))
    {
        cx.ledger.selected = v.lines.first().map(|l| l.id);
    }
    // Narrow windows: the detail goes under the tables.
    let wide = ui.available_width() > 760.0;
    if wide {
        ui.spacing_mut().item_spacing.x = space::XL;
        ui.columns(2, |cols| {
            tables(&mut cols[0], cx, v);
            detail(&mut cols[1], cx, v);
        });
    } else {
        tables(ui, cx, v);
        bf_ui_kit::divider(ui);
        detail(ui, cx, v);
    }
}

fn tables(ui: &mut Ui, cx: &mut TabContext<'_>, v: &LedgerView) {
    for (by_us, head) in [(true, "ledger-we-owe"), (false, "ledger-owed-to-us")] {
        bf_ui_kit::heading(ui, &cx.i18n.tr(head));
        let lines: Vec<&Line> = v.owed(by_us).collect();
        if lines.is_empty() {
            bf_ui_kit::muted(ui, &cx.i18n.tr("ledger-nothing"));
        } else {
            let w = widths(ui, cx.i18n);
            header(ui, cx.i18n, &w);
            for l in &lines {
                row(ui, cx, l, &w);
            }
            totals(ui, cx.i18n, &lines);
        }
        bf_ui_kit::gap(ui, space::MD);
    }
}

/// Widths of the number columns, reserved by the longest text they can hold in this language
/// (a twelve-digit sum, three-digit years, the headers): a number is never cut.
struct Widths {
    money: f32,
    years: f32,
}

const DOT: f32 = 14.0;
const GAP: f32 = 10.0;

fn widths(ui: &Ui, i18n: &I18n) -> Widths {
    let w = |t: String, size: f32| {
        ui.painter()
            .layout_no_wrap(t, sans(size), color::TEXT)
            .size()
            .x
    };
    let money = [999_999_999_999.0, 99_999_999_999.0, 999_999_999.0]
        .into_iter()
        .map(|x| w(money_short(i18n, x), size::SMALL))
        .chain(
            ["ledger-col-balance", "ledger-col-annuity"]
                .into_iter()
                .map(|k| w(i18n.tr(k), size::CAPTION)),
        )
        .fold(0.0, f32::max);
    let years = w("999".into(), size::SMALL).max(w(i18n.tr("ledger-col-years"), size::CAPTION));
    Widths {
        money: money.ceil() + 2.0,
        years: years.ceil() + 2.0,
    }
}

/// Columns, from the right: standing (a dot), years, annuity, balance; the country takes the
/// rest and is the only text that may be cut.
fn columns(r: Rect, w: &Widths) -> [Rect; 5] {
    let mut right = r.right();
    let mut take = |width: f32| {
        let c = Rect::from_min_max(
            egui::pos2(right - width, r.top()),
            egui::pos2(right, r.bottom()),
        );
        right -= width + GAP;
        c
    };
    let dot = take(DOT);
    let years = take(w.years);
    let annuity = take(w.money);
    let balance = take(w.money);
    let country = Rect::from_min_max(r.min, egui::pos2(right.max(r.left() + 40.0), r.bottom()));
    [country, balance, annuity, years, dot]
}

/// A number, whole: never cut, right-aligned in its reserved column.
fn number(ui: &mut Ui, rect: Rect, text: RichText) {
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.add(egui::Label::new(text).extend());
        },
    );
}

fn header(ui: &mut Ui, i18n: &I18n, w: &Widths) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
    let faint = |k: &str| {
        RichText::new(i18n.tr(k))
            .font(sans(size::CAPTION))
            .color(color::TEXT_FAINT)
    };
    let c = columns(r.shrink2(vec2(space::SM, 0.0)), w);
    cell(ui, c[0], faint("ledger-col-country"), false);
    number(ui, c[1], faint("ledger-col-balance"));
    number(ui, c[2], faint("ledger-col-annuity"));
    number(ui, c[3], faint("ledger-col-years"));
}

fn row(ui: &mut Ui, cx: &mut TabContext<'_>, l: &Line, w: &Widths) {
    let i18n = cx.i18n;
    let selected = cx.ledger.selected == Some(l.id);
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    let name = country_name(i18n, &l.other);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &name));
    if selected || resp.hovered() {
        ui.painter().rect_filled(
            r,
            radius::SM,
            if selected {
                color::ACCENT_SOFT
            } else {
                color::SURFACE_HOVER
            },
        );
    }
    let text = |s: String| RichText::new(s).font(sans(size::SMALL)).color(color::TEXT);
    let c = columns(r.shrink2(vec2(space::SM, 0.0)), w);
    cell(ui, c[0], text(name), false);
    // The table's sums short ("US$ 673 mi"), whole; the detail has them in full.
    number(ui, c[1], text(money_short(i18n, l.principal)));
    number(ui, c[2], text(money_short(i18n, l.annuity)));
    number(ui, c[3], text(l.years_left.to_string()));
    // The standing as a coloured dot; its name on hover and in the detail.
    let dot = ui.interact(
        c[4],
        ui.make_persistent_id(("ledger-dot", l.id)),
        Sense::hover(),
    );
    ui.painter()
        .circle_filled(c[4].center(), 4.5, standing_color(l.standing));
    dot.on_hover_text(i18n.tr(l.standing.key()));
    let typed = keys::item(
        ui,
        'Z',
        format!("ledger.line.{}", l.id),
        &l.id.to_string(),
        r,
    );
    if resp.clicked() || typed {
        cx.ledger.selected = Some(l.id);
    }
}

fn totals(ui: &mut Ui, i18n: &I18n, lines: &[&Line]) {
    let sum = |f: fn(&Line) -> f64| lines.iter().map(|l| f(l)).sum::<f64>();
    let arrears = sum(|l| l.arrears);
    let mut text = i18n.tr_args(
        "ledger-totals",
        &[
            ("balance", Arg::String(money(i18n, sum(|l| l.principal)))),
            ("annuity", Arg::String(money(i18n, sum(|l| l.annuity)))),
        ],
    );
    if arrears > 0.0 {
        text.push_str(" · ");
        text.push_str(&i18n.tr_args(
            "ledger-totals-arrears",
            &[("arrears", Arg::String(money(i18n, arrears)))],
        ));
    }
    bf_ui_kit::small(ui, &text);
}

fn detail(ui: &mut Ui, cx: &mut TabContext<'_>, v: &LedgerView) {
    let i18n = cx.i18n;
    let Some(l) = v.lines.iter().find(|l| Some(l.id) == cx.ledger.selected) else {
        return;
    };
    ui.label(
        RichText::new(i18n.tr_args(
            if l.owed_by_us {
                "ledger-detail-owe"
            } else {
                "ledger-detail-owed"
            },
            &[("country", Arg::String(country_name(i18n, &l.other)))],
        ))
        .font(sans_semibold(size::BODY))
        .color(color::TEXT),
    );
    bf_ui_kit::muted(ui, &treaty_name(i18n, &l.treaty));
    bf_ui_kit::gap(ui, space::SM);
    let row = |ui: &mut Ui, k: &str, value: String| bf_ui_kit::stat_row(ui, &i18n.tr(k), &value);
    row(ui, "ledger-balance", money(i18n, l.principal));
    // The amortized part: what was signed in 1925 less what is left.
    let signed = cx.signed.get(&(
        l.treaty.clone(),
        if l.owed_by_us {
            v.tag.clone()
        } else {
            l.other.clone()
        },
        if l.owed_by_us {
            l.other.clone()
        } else {
            v.tag.clone()
        },
    ));
    match signed {
        Some(s) => {
            let amortized = (s - l.principal).max(0.0);
            bf_ui_kit::stat_row(ui, &i18n.tr("ledger-amortized"), &money(i18n, amortized));
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "ledger-amortized-note",
                    &[("signed", Arg::String(money(i18n, *s)))],
                ),
            );
        }
        None => row(ui, "ledger-amortized", "—".into()),
    }
    // Paid since the signature: ready for the ledger's running total.
    match l.paid {
        Some(p) => row(ui, "ledger-paid", money(i18n, p)),
        None => {
            bf_ui_kit::stat_row(ui, &i18n.tr("ledger-paid"), "—");
            bf_ui_kit::small(ui, &i18n.tr("ledger-paid-pending"));
        }
    }
    row(ui, "ledger-rate", pct(i18n, l.rate));
    row(ui, "ledger-annuity", money(i18n, l.annuity));
    row(ui, "ledger-years", l.years_left.to_string());
    row(
        ui,
        "ledger-left",
        money(i18n, l.annuity * f64::from(l.years_left)),
    );
    let arrears = money(i18n, l.arrears);
    bf_ui_kit::stat_row(ui, &i18n.tr("ledger-arrears"), &arrears);
    ui.horizontal(|ui| {
        bf_ui_kit::small(ui, &i18n.tr("ledger-standing"));
        ui.label(
            RichText::new(i18n.tr(l.standing.key()))
                .font(sans(size::SMALL))
                .color(standing_color(l.standing)),
        )
        .on_hover_text(i18n.tr(&format!("{}-tip", l.standing.key())));
    });

    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("ledger-terms"));
    let currency = match l.currency {
        Currency::Reference => "ledger-currency-gold",
        Currency::Debtor => "ledger-currency-debtor",
        Currency::Creditor => "ledger-currency-creditor",
    };
    row(ui, "ledger-currency", i18n.tr(currency));
    if l.deferrable > 0.0 {
        row(ui, "ledger-deferrable", pct(i18n, l.deferrable));
    }
    if let Some(cap) = l.transfer_cap {
        row(ui, "ledger-transfer-cap", pct(i18n, cap));
    }
    if let Some(p) = l.pledge {
        let k = match p {
            Pledge::Customs => "ledger-pledge-customs",
            Pledge::ConsumptionTaxes => "ledger-pledge-consumption",
            Pledge::Railways => "ledger-pledge-railways",
        };
        row(ui, "ledger-pledge", i18n.tr(k));
    }
    if l.contingent > 0.0 {
        row(ui, "ledger-contingent", money(i18n, l.contingent));
    }
    if l.suspended_until > 0 {
        row(ui, "ledger-moratorium", l.suspended_until.to_string());
    }

    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("ledger-history"));
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "ledger-history-last",
            &[("treaty", Arg::String(treaty_name(i18n, &l.treaty)))],
        ),
    );
    bf_ui_kit::gap(ui, space::SM);
    // Renegotiating opens the negotiation table in the Agendas with the revision on it; with the
    // table switched off the button says so.
    let on = cx.snap.treaty_table.is_some();
    let b = bf_ui_kit::Button::secondary(&i18n.tr("ledger-renegotiate"))
        .enabled(on)
        .show(ui);
    let typed = on && keys::single(ui, "ledger.renegotiate", 'N', b.rect);
    if on && (b.clicked() || typed) {
        *cx.renegotiate = Some((l.other.clone(), revision(cx.tag, l)));
    }
    b.on_disabled_hover_text(i18n.tr("ledger-renegotiate-tip"))
        .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("ledger-renegotiate-open")));
}

/// The proposal "Renegotiate" puts on the table: the creditor revises the terms of the line's
/// treaty, starting from the terms of today.
pub fn revision(me: &str, l: &Line) -> bf_sim::treaty::Proposal {
    use bf_sim::treaty::{Clause, ClauseKind, DebtTerms, Proposal, Side};
    use laplace_schelling::Fixed;
    let x = Fixed::from_f64_lossy;
    let terms = DebtTerms {
        principal: x(l.principal),
        rate: x(l.rate),
        annuity: x(l.annuity),
        years: l.years_left,
        currency: l.currency,
        deferrable: x(l.deferrable),
        transfer_cap: l.transfer_cap.map(x),
        pledge: l.pledge,
        contingent: x(l.contingent),
        grace_years: 0,
    };
    // The creditor gives the revision: the other side when the player owes.
    let (giver, receiver) = if l.owed_by_us {
        (Side::Junior, Side::Senior)
    } else {
        (Side::Senior, Side::Junior)
    };
    Proposal {
        side_a: vec![me.to_owned()],
        side_b: vec![l.other.clone()],
        clauses: vec![Clause {
            kind: ClauseKind::ReviseDebt {
                treaty: l.treaty.clone(),
                terms,
            },
            giver,
            receiver,
            until: None,
            note: None,
        }],
        term_years: None,
        notice_years: 0,
    }
}

/// What the tab keeps between frames.
#[derive(Clone, Copy, Debug, Default)]
pub struct LedgerUi {
    pub selected: Option<u32>,
}
