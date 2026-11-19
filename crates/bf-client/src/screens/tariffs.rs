//! Government tab: trade, the tariff by good and by partner (`docs/trade.pt-BR.md`, section 13;
//! owner's decisions CT1 and CT2). The goods, by category, each on the historical rule (the
//! trade law, from the 1925 tariff) or at a rate the government fixed; and the partners, the
//! largest sellers first, each with the agreements in force for the pair and the points the
//! government adds or takes off every good's tariff. The agreements hold the result: the
//! screen says when they cut what the player asked. Read from `crate::tariff_view`.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::customs::TariffTable;
use bf_sim::game::GameCommand;
use bf_ui_kit::theme::{sans, sans_medium};
use bf_ui_kit::tokens::{color, control, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align, Layout, RichText, Sense, Ui, vec2};
use laplace_schelling::Fixed;

use super::budget::{WEEKS, pct, with_gdp};
use super::country_find::{self, FindUi};
use super::gov_ui::{StepTips, step_width, stepper_wide};
use super::government::TabContext;
use crate::keys::{self, Tip};
use crate::tariff_view::{Category, PartnerLine, TOP_PARTNERS, TariffView, good_rate};
use crate::text::{country_name, money, money_short};

/// The trade tab's state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TariffUi {
    /// The category open, by key: one at a time, so the page stays short.
    pub open: Option<String>,
    /// The rate "Apply to the group" sets, per mille; the open category's mean until moved.
    pub group: Option<u16>,
    pub find: FindUi,
    /// Partners picked by name, listed after the largest.
    pub picked: Vec<String>,
}

/// A good's rate: steps of one percentage point, from zero to 200 %.
const RATE_STEP: i32 = 10;
const RATE_MAX: i32 = 2000;
/// A partner's points: steps of one point, from −100 to +100.
const POINT_STEP: i32 = 10;
const POINT_MAX: i32 = 1000;

const ROW_H: f32 = 28.0;
const GAP: f32 = 8.0;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// A rate, a fraction, as "26,0 %".
fn rate(i18n: &I18n, r: Fixed) -> String {
    pct(i18n, r.to_f64_lossy() * 100.0, 1)
}

/// A per-mille rate rounded to the step, inside the limits.
fn to_step(r: Fixed) -> u16 {
    let v = (r.to_f64_lossy() * 1000.0 / f64::from(RATE_STEP)).round() as i32 * RATE_STEP;
    v.clamp(0, RATE_MAX) as u16
}

/// "+10 pontos", "−5 pontos", "sem ajuste".
fn points(i18n: &I18n, per_mille: i32) -> String {
    if per_mille == 0 {
        return i18n.tr("tariff-points-none");
    }
    let sign = if per_mille > 0 { "+" } else { "−" };
    let n = f64::from(per_mille.abs()) / 10.0;
    i18n.tr_args(
        "tariff-points",
        &[("sign", arg(sign.into())), ("n", Arg::Num(n))],
    )
}

fn good_name(i18n: &I18n, key: &str) -> String {
    i18n.tr(&format!("good-{key}"))
}

/// A cell of a row: `width` wide, its content aligned left or right.
fn cell<R>(ui: &mut Ui, width: f32, right: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let layout = if right {
        Layout::right_to_left(Align::Center)
    } else {
        Layout::left_to_right(Align::Center)
    };
    ui.allocate_ui_with_layout(vec2(width, ROW_H), layout, add)
        .inner
}

fn text_cell(ui: &mut Ui, width: f32, right: bool, text: RichText) {
    cell(ui, width, right, |ui| {
        ui.add(egui::Label::new(text).truncate());
    });
}

fn faint(text: String) -> RichText {
    RichText::new(text)
        .font(sans(size::CAPTION))
        .color(color::TEXT_FAINT)
}

fn body(text: String) -> RichText {
    RichText::new(text)
        .font(sans(size::BODY))
        .color(color::TEXT)
}

/// Widths of the goods' columns, from the right: imports, rate, the stepper, the rule's
/// choice; the name takes what is left.
struct GoodCols {
    imports: f32,
    rate: f32,
    step_value: f32,
    stepper: f32,
    choice: f32,
    name: f32,
}

fn text_w(ui: &Ui, t: &str, font: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(t.to_owned(), font, color::TEXT)
        .size()
        .x
}

fn good_cols(ui: &Ui, i18n: &I18n) -> GoodCols {
    let small = |t: &str| text_w(ui, t, sans(size::BODY));
    let imports = [
        small(&money_short(i18n, 999_000_000_000.0)),
        text_w(ui, &i18n.tr("tariff-col-imports"), sans(size::CAPTION)),
    ]
    .into_iter()
    .fold(0.0, f32::max)
    .ceil()
        + 4.0;
    let rate = [
        small(&pct(i18n, 200.0, 1)),
        text_w(ui, &i18n.tr("tariff-col-rate"), sans(size::CAPTION)),
    ]
    .into_iter()
    .fold(0.0, f32::max)
    .ceil()
        + 4.0;
    let step_value = step_width(ui, &[pct(i18n, 200.0, 0)]);
    // − and + are small buttons beside the value.
    let stepper = step_value + 2.0 * (control::HEIGHT_SM + space::XS) + 4.0;
    let seg = |t: String| text_w(ui, &t, sans_medium(size::SMALL + 1.0)) + space::LG * 2.0;
    let choice = seg(i18n.tr("lever-rule")) + seg(i18n.tr("lever-fixed")) + 12.0;
    let used = imports + rate + stepper + choice + 4.0 * GAP;
    GoodCols {
        imports,
        rate,
        step_value,
        stepper,
        choice,
        name: (ui.available_width() - used).max(96.0),
    }
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(v) = cx.snap.tariffs.clone() else {
        bf_ui_kit::muted(ui, &i18n.tr(super::government::waiting_key(cx.snap)));
        return;
    };
    if v.tag != cx.tag {
        bf_ui_kit::muted(ui, &i18n.tr("gov-waiting"));
        return;
    }
    // What the player set this tick wins over the view until the tick applies it.
    let mut table = cx.drafts.tariffs.clone().unwrap_or_else(|| v.table.clone());
    let mut out: Vec<GameCommand> = Vec::new();
    ui.spacing_mut().item_spacing = vec2(GAP, space::XS);

    summary(ui, i18n, &v, &table, cx.tag);
    bf_ui_kit::gap(ui, space::SM);
    let tag = cx.tag.to_owned();
    let mut set = |table: &mut TariffTable, goods: Vec<u16>, per_mille: Option<u16>| {
        if goods.is_empty() {
            return;
        }
        for &g in &goods {
            table.set_good(g, per_mille);
        }
        out.push(GameCommand::SetGoodsTariff {
            tag: tag.clone(),
            goods,
            per_mille,
        });
    };
    let cols = good_cols(ui, i18n);
    goods_header(ui, i18n, &cols);
    for c in &v.categories {
        category(ui, cx, &v, c, &cols, &mut table, &mut set);
    }

    bf_ui_kit::gap(ui, space::MD);
    let mut partner_out = partners(ui, cx, &v, &mut table);
    out.append(&mut partner_out);

    if !out.is_empty() {
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.tariffs = Some(table);
        cx.commands.extend(out);
    }
}

/// The head: the mean rate, the customs revenue and the trade law.
fn summary(ui: &mut Ui, i18n: &I18n, v: &TariffView, table: &TariffTable, tag: &str) {
    ui.horizontal(|ui| {
        bf_ui_kit::heading(ui, &i18n.tr("gov-tab-trade"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(country_name(i18n, tag))
                    .font(sans_medium(size::BODY))
                    .color(color::TEXT_MUTED),
            );
        });
    });
    // Off, the revenue the table would collect from last year's imports (`would_collect`).
    let yearly = if v.on {
        v.revenue * WEEKS
    } else {
        v.would_collect(table)
    };
    let mut parts = vec![
        i18n.tr_args("tariff-mean", &[("rate", arg(rate(i18n, v.mean(table))))]),
        i18n.tr_args(
            if v.on {
                "tariff-revenue"
            } else {
                "tariff-revenue-would"
            },
            &[(
                "amount",
                arg(with_gdp(i18n, money(i18n, yearly), yearly, v.gdp)),
            )],
        ),
    ];
    if let Some(law) = &v.law {
        parts.push(i18n.tr_args(
            "tariff-law",
            &[("law", arg(i18n.tr(&format!("law-{law}"))))],
        ));
    }
    ui.add(
        egui::Label::new(
            RichText::new(parts.join(" · "))
                .font(sans_medium(size::BODY))
                .color(color::TEXT),
        )
        .wrap(),
    )
    .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("tariff-mean-tip")));
    if !v.on {
        bf_ui_kit::muted(ui, &i18n.tr("tariff-off"));
    }
    bf_ui_kit::small(ui, &i18n.tr("tariff-basis"));
}

fn goods_header(ui: &mut Ui, i18n: &I18n, c: &GoodCols) {
    ui.horizontal(|ui| {
        text_cell(ui, c.name, false, faint(i18n.tr("tariff-col-good")));
        text_cell(ui, c.choice, false, faint(i18n.tr("tariff-col-rule")));
        cell(ui, c.stepper, false, |_| {});
        text_cell(ui, c.rate, true, faint(i18n.tr("tariff-col-rate")));
        cell(ui, c.imports, true, |ui| {
            ui.add(egui::Label::new(faint(i18n.tr("tariff-col-imports"))).truncate())
                .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("tariff-imports-tip")));
        });
    });
}

/// "26,0 %" or "26,0 % a 40,0 %", and how many goods are fixed.
fn category_summary(i18n: &I18n, v: &TariffView, c: &Category, table: &TariffTable) -> String {
    let rates: Vec<Fixed> = c
        .goods
        .iter()
        .map(|g| good_rate(table, v.rule, usize::from(g.good)))
        .collect();
    let lo = rates.iter().copied().min().unwrap_or(v.rule);
    let hi = rates.iter().copied().max().unwrap_or(v.rule);
    let mut s = if lo == hi {
        rate(i18n, lo)
    } else {
        i18n.tr_args(
            "tariff-range",
            &[("low", arg(rate(i18n, lo))), ("high", arg(rate(i18n, hi)))],
        )
    };
    let fixed = c
        .goods
        .iter()
        .filter(|g| table.good(usize::from(g.good)).is_some())
        .count();
    if fixed > 0 {
        s.push_str(" · ");
        s.push_str(&i18n.tr_args(
            "tariff-fixed-count",
            &[
                ("n", Arg::Num(fixed as f64)),
                ("of", Arg::Num(c.goods.len() as f64)),
            ],
        ));
    }
    s
}

/// A category: its head (click or key to open it), and when open the group's controls and its
/// goods.
#[allow(clippy::too_many_arguments)]
fn category(
    ui: &mut Ui,
    cx: &mut TabContext<'_>,
    v: &TariffView,
    c: &Category,
    cols: &GoodCols,
    table: &mut TariffTable,
    set: &mut impl FnMut(&mut TariffTable, Vec<u16>, Option<u16>),
) {
    let i18n = cx.i18n;
    let open = cx.tariffs.open.as_deref() == Some(c.key.as_str());
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click());
    let name = i18n.tr(&format!("good-category-{}", c.key));
    let head = format!("{name} ({})", c.goods.len());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, open, &head));
    let painter = ui.painter();
    if resp.hovered() {
        painter.rect_filled(rect, radius::SM, color::SURFACE_HOVER);
    }
    // The disclosure triangle: down when open, right when closed.
    let at = rect.left_center() + vec2(space::XS + 5.0, 0.0);
    let tri = if open {
        vec![
            at + vec2(-5.0, -3.0),
            at + vec2(5.0, -3.0),
            at + vec2(0.0, 4.0),
        ]
    } else {
        vec![
            at + vec2(-3.0, -5.0),
            at + vec2(4.0, 0.0),
            at + vec2(-3.0, 5.0),
        ]
    };
    painter.add(egui::Shape::convex_polygon(
        tri,
        color::TEXT_MUTED,
        egui::Stroke::NONE,
    ));
    let left = painter.text(
        rect.left_center() + vec2(space::XS + 16.0, 0.0),
        egui::Align2::LEFT_CENTER,
        &head,
        sans_medium(size::BODY),
        color::TEXT,
    );
    let summary = category_summary(i18n, v, c, table);
    let right = rect.right() - space::XS;
    if right - left.right() > 2.0 * GAP {
        painter.text(
            egui::pos2(right, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            &summary,
            sans(size::SMALL),
            color::TEXT_MUTED,
        );
    }
    let typed = keys::item(ui, 'K', format!("gov.tariff.cat.{}", c.key), &c.key, rect);
    if resp.clicked() || typed {
        cx.tariffs.open = if open { None } else { Some(c.key.clone()) };
        cx.tariffs.group = None;
    }
    if !open {
        return;
    }
    let goods: Vec<u16> = c.goods.iter().map(|g| g.good).collect();
    // Apply to the group, back to the rule.
    let mean = {
        let sum: Fixed = goods
            .iter()
            .map(|&g| good_rate(table, v.rule, usize::from(g)))
            .sum();
        sum / goods.len().max(1) as i64
    };
    let mut group = cx.tariffs.group.unwrap_or_else(|| to_step(mean));
    ui.horizontal(|ui| {
        ui.add_space(space::LG);
        ui.label(
            RichText::new(i18n.tr("tariff-group-apply"))
                .font(sans(size::BODY))
                .color(color::TEXT_MUTED),
        );
        let d = stepper_wide(
            ui,
            &pct(i18n, f64::from(group) / 10.0, 0),
            cols.step_value,
            &i18n.tr("tariff-rate-limits"),
            group > 0,
            i32::from(group) < RATE_MAX,
            StepTips {
                family: 'G',
                id: "gov.tariff.group",
                row: 0,
            },
        );
        if d != 0 {
            group = (i32::from(group) + d * RATE_STEP).clamp(0, RATE_MAX) as u16;
            cx.tariffs.group = Some(group);
        }
        let apply = Button::secondary(&i18n.tr("tariff-apply"))
            .size(ButtonSize::Small)
            .show(ui);
        if apply.clicked() || keys::item(ui, 'G', "gov.tariff.group.apply".into(), "a", apply.rect)
        {
            set(table, goods.clone(), Some(group));
        }
        let any_fixed = goods.iter().any(|&g| table.good(usize::from(g)).is_some());
        let back = Button::ghost(&i18n.tr("tariff-back-to-rule"))
            .size(ButtonSize::Small)
            .enabled(any_fixed)
            .show(ui);
        let typed =
            any_fixed && keys::item(ui, 'G', "gov.tariff.group.rule".into(), "r", back.rect);
        if back.clicked() || typed {
            let fixed: Vec<u16> = goods
                .iter()
                .copied()
                .filter(|&g| table.good(usize::from(g)).is_some())
                .collect();
            set(table, fixed, None);
        }
    });
    for (row, g) in c.goods.iter().enumerate() {
        let gi = usize::from(g.good);
        let key = &cx.catalogue.goods[gi].key;
        ui.horizontal(|ui| {
            cell(ui, cols.name, false, |ui| {
                ui.add_space(space::LG);
                ui.add(egui::Label::new(body(good_name(i18n, key))).truncate());
            });
            let now = table.good(gi);
            let mut fixed = now.is_some();
            cell(ui, cols.choice, false, |ui| {
                let options = [
                    (false, i18n.tr("lever-rule")),
                    (true, i18n.tr("lever-fixed")),
                ];
                let tips = [
                    Tip::Item('F', format!("gov.tariff.good.{key}.rule"), key),
                    Tip::Item('F', format!("gov.tariff.good.{key}.fixed"), key),
                ];
                if keys::segmented(ui, &mut fixed, &options, &tips) {
                    set(
                        table,
                        vec![g.good],
                        fixed.then(|| to_step(good_rate(table, v.rule, gi))),
                    );
                }
            });
            cell(ui, cols.stepper, false, |ui| {
                if let Some(r) = table.good(gi) {
                    let pm = (r.to_f64_lossy() * 1000.0).round() as i32;
                    let d = stepper_wide(
                        ui,
                        &pct(i18n, f64::from(pm) / 10.0, 0),
                        cols.step_value,
                        &i18n.tr("tariff-rate-limits"),
                        pm > 0,
                        pm < RATE_MAX,
                        StepTips {
                            family: 'S',
                            id: &format!("gov.tariff.good.{key}"),
                            row,
                        },
                    );
                    if d != 0 {
                        let to = (pm + d * RATE_STEP).clamp(0, RATE_MAX) as u16;
                        set(table, vec![g.good], Some(to));
                    }
                }
            });
            text_cell(
                ui,
                cols.rate,
                true,
                body(rate(i18n, good_rate(table, v.rule, gi))),
            );
            let imports = if g.imports > 0.0 {
                money_short(i18n, g.imports * WEEKS)
            } else {
                "—".into()
            };
            text_cell(ui, cols.imports, true, body(imports));
        });
    }
    bf_ui_kit::gap(ui, space::XS);
}

/// Widths of the partners' columns, from the right: the effective rate, the stepper, the
/// agreement, imports; the name takes what is left.
struct PartnerCols {
    imports: f32,
    terms: f32,
    step_value: f32,
    stepper: f32,
    rate: f32,
    name: f32,
}

fn partner_cols(ui: &Ui, i18n: &I18n) -> PartnerCols {
    let body_w = |t: &str| text_w(ui, t, sans(size::BODY));
    let cap_w = |k: &str| text_w(ui, &i18n.tr(k), sans(size::CAPTION));
    let imports = body_w(&money_short(i18n, 999_000_000_000.0))
        .max(cap_w("tariff-col-imports"))
        .ceil()
        + 4.0;
    let rate = body_w(&pct(i18n, 200.0, 1))
        .max(cap_w("tariff-col-effective"))
        .ceil()
        + 4.0;
    let step_value = step_width(ui, &[points(i18n, -1000), points(i18n, 0)]);
    let stepper = step_value + 2.0 * (control::HEIGHT_SM + space::XS) + 4.0;
    let terms = 200.0;
    let used = imports + rate + stepper + terms + 4.0 * GAP;
    PartnerCols {
        imports,
        terms,
        step_value,
        stepper,
        rate,
        name: (ui.available_width() - used).max(96.0),
    }
}

/// The partners: the largest sellers, those with points, those picked by name.
fn partners(
    ui: &mut Ui,
    cx: &mut TabContext<'_>,
    v: &TariffView,
    table: &mut TariffTable,
) -> Vec<GameCommand> {
    let i18n = cx.i18n;
    let mut out = Vec::new();
    bf_ui_kit::heading(ui, &i18n.tr("tariff-partners"));
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            if v.so_far {
                "tariff-partners-basis-so-far"
            } else {
                "tariff-partners-basis"
            },
            &[
                ("n", Arg::Num(TOP_PARTNERS as f64)),
                ("year", arg(v.year.to_string())),
            ],
        ),
    );
    let shown: Vec<&PartnerLine> = v
        .partners
        .iter()
        .enumerate()
        .filter(|(k, p)| {
            (*k < TOP_PARTNERS && p.imports > 0.0)
                || table.partner(p.country) != Fixed::ZERO
                || cx.tariffs.picked.contains(&p.tag)
        })
        .map(|(_, p)| p)
        .collect();
    let c = partner_cols(ui, i18n);
    ui.horizontal(|ui| {
        text_cell(ui, c.name, false, faint(i18n.tr("tariff-col-partner")));
        text_cell(ui, c.imports, true, faint(i18n.tr("tariff-col-imports")));
        text_cell(ui, c.terms, false, faint(i18n.tr("tariff-col-agreement")));
        text_cell(ui, c.stepper, false, faint(i18n.tr("tariff-col-points")));
        text_cell(ui, c.rate, true, faint(i18n.tr("tariff-col-effective")));
    });
    for (row, p) in shown.iter().enumerate() {
        let pm = (table.partner(p.country).to_f64_lossy() * 1000.0).round() as i32;
        ui.horizontal(|ui| {
            text_cell(ui, c.name, false, body(country_name(i18n, &p.tag)));
            let imports = if p.imports > 0.0 {
                money_short(i18n, p.imports)
            } else {
                "—".into()
            };
            text_cell(ui, c.imports, true, body(imports));
            let terms = if p.terms.is_empty() {
                i18n.tr("tariff-agreement-none")
            } else {
                p.terms
                    .iter()
                    .map(|t| super::trade::term(i18n, *t))
                    .collect::<Vec<_>>()
                    .join(" · ")
            };
            cell(ui, c.terms, false, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(terms)
                            .font(sans(size::SMALL))
                            .color(color::TEXT_MUTED),
                    )
                    .truncate(),
                );
            });
            cell(ui, c.stepper, false, |ui| {
                let d = stepper_wide(
                    ui,
                    &points(i18n, pm),
                    c.step_value,
                    &i18n.tr("tariff-points-tip"),
                    pm > -POINT_MAX,
                    pm < POINT_MAX,
                    StepTips {
                        family: 'P',
                        id: &format!("gov.tariff.partner.{}", p.tag),
                        row,
                    },
                );
                if d != 0 {
                    let to = (pm + d * POINT_STEP).clamp(-POINT_MAX, POINT_MAX) as i16;
                    let to = (to != 0).then_some(to);
                    if let Ok(pi) = u16::try_from(p.country) {
                        table.set_partner(pi, to);
                    }
                    out.push(GameCommand::SetPartnerTariff {
                        tag: cx.tag.to_owned(),
                        partner: p.tag.clone(),
                        per_mille: to,
                    });
                }
            });
            text_cell(ui, c.rate, true, body(rate(i18n, v.effective(table, p))));
        });
        if let Some(cap) = v.limit(table, p) {
            let note = if cap == Fixed::ZERO {
                i18n.tr("tariff-limit-union")
            } else {
                i18n.tr_args("tariff-limit-cap", &[("rate", arg(rate(i18n, cap)))])
            };
            ui.horizontal(|ui| {
                ui.add_space(space::LG);
                ui.label(
                    RichText::new(note)
                        .font(sans(size::SMALL))
                        .color(color::ACCENT),
                );
            });
        }
    }
    if shown.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("tariff-partners-none"));
    }
    if let Some(world) = cx.world
        && let Some(tag) = country_find::show(
            ui,
            i18n,
            world,
            cx.tag,
            &mut cx.tariffs.find,
            "gov.tariff",
            'A',
        )
        && !cx.tariffs.picked.contains(&tag)
    {
        cx.tariffs.picked.push(tag);
    }
    out
}
