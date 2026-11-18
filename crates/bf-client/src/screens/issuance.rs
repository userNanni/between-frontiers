//! Money and debt tab: where the treasury issues its new bonds (D9, `docs/state.pt-BR.md`). The
//! historical rule (home savers first, then the bank, then abroad, lenders abroad taking in
//! dollars the share they demand) or a split fixed by the government between home, abroad in the
//! own currency and abroad in dollars. With the split fixed, last week's bonds sold and left
//! unsold at each market, and a warning when the share asked abroad in the own currency passes
//! what lenders abroad take ("original sin"): the rest goes unsold, the cash falls and austerity
//! follows. Read from `bf_sim::econ::levers::issuance_view`.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::levers::IssuanceView;
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::{color, space};
use egui::{RichText, Ui};

use super::gov_ui::{StepTips, control_row, stepper};
use super::government::TabContext;
use crate::keys;
use crate::text::money;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// Step of a share, in thousandths.
const STEP: i32 = 50;

fn pct(i18n: &I18n, share: f64) -> String {
    format!("{} %", i18n.decimal(share * 100.0, 0))
}

/// The fixed split as the command carries it, from the view's shares of 1.
fn milli(shares: [f64; 3]) -> [u16; 3] {
    shares.map(|s| (s * 1000.0).round() as u16)
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>, view: &IssuanceView) {
    let i18n = cx.i18n;
    let now: Option<[u16; 3]> = cx.drafts.issuance.unwrap_or_else(|| view.fixed.map(milli));
    let mut next = now;

    bf_ui_kit::heading(ui, &i18n.tr("issue-heading"));
    let mut fixed = now.is_some();
    let (changed, rects) = bf_ui_kit::lever_header(
        ui,
        &i18n.tr("issue-lever"),
        &mut fixed,
        &i18n.tr("lever-rule"),
        &i18n.tr("lever-fixed"),
    );
    let mut typed = false;
    for (r, (key, on)) in rects.into_iter().zip([("rule", false), ("fixed", true)]) {
        if keys::item(ui, 'I', format!("gov.issue.{key}"), key, r) && fixed != on {
            fixed = on;
            typed = true;
        }
    }
    if changed || typed {
        // Fixed starts from the rule's own order: everything at home.
        next = fixed.then_some([1000, 0, 0]);
    }
    bf_ui_kit::lever_rule_line(
        ui,
        &i18n.tr("issue-rule"),
        next.map(|_| i18n.tr("issue-note")).as_deref(),
    );
    if let Some([home, local, _]) = next {
        // Two steppers; the dollar share is what is left.
        let mut split = [home, local];
        for (k, key) in ["issue-home", "issue-abroad-local"].into_iter().enumerate() {
            let v = split[k];
            let other = split[1 - k];
            let d = control_row(ui, &i18n.tr(key), |ui| {
                stepper(
                    ui,
                    &pct(i18n, f64::from(v) / 1000.0),
                    &i18n.tr("issue-tip"),
                    v > 0,
                    u32::from(v) + u32::from(other) < 1000,
                    StepTips {
                        family: 'I',
                        id: &format!("gov.issue.{k}"),
                        row: k,
                    },
                )
            });
            if d != 0 {
                split[k] = (i32::from(v) + d * STEP).clamp(0, 1000 - i32::from(other)) as u16;
            }
        }
        let dollar = 1000 - split[0] - split[1];
        next = Some([split[0], split[1], dollar]);
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr("issue-abroad-fx"),
            &pct(i18n, f64::from(dollar) / 1000.0),
        );
        // Original sin: lenders abroad take only so much in the own currency.
        let abroad = u32::from(split[1]) + u32::from(dollar);
        if abroad > 0 && view.local_limit < 1.0 {
            let asked = f64::from(split[1]) / f64::from(abroad);
            if asked > view.local_limit + 1e-9 {
                warn(
                    ui,
                    &i18n.tr_args(
                        "issue-original-sin",
                        &[
                            ("asked", arg(pct(i18n, asked))),
                            ("limit", arg(pct(i18n, view.local_limit))),
                        ],
                    ),
                );
            }
        }
    }
    // Last week, with the split fixed: sold and unsold at each market.
    if view.fixed.is_some() {
        bf_ui_kit::caption(ui, &i18n.tr("issue-last-week"));
        for (key, sold, unsold) in [
            ("issue-at-home", view.home, Some(view.unsold_home)),
            ("issue-at-abroad-local", view.abroad_local, None),
            ("issue-at-abroad-fx", view.abroad_fx, None),
        ] {
            bf_ui_kit::stat_row(ui, &i18n.tr(key), &money(i18n, sold));
            if let Some(u) = unsold.filter(|u| *u > 0.5) {
                warn(
                    ui,
                    &i18n.tr_args("issue-unsold-home", &[("money", arg(money(i18n, u)))]),
                );
            }
        }
        if view.unsold_abroad > 0.5 {
            warn(
                ui,
                &i18n.tr_args(
                    "issue-unsold-abroad",
                    &[("money", arg(money(i18n, view.unsold_abroad)))],
                ),
            );
        }
    }
    if next != now {
        cx.commands.push(GameCommand::SetIssuance {
            tag: cx.tag.to_owned(),
            shares: next,
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.issuance = Some(next);
    }
}

fn warn(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                .color(color::DANGER),
        )
        .wrap(),
    );
    let _ = space::XS;
}
