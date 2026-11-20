//! The journal window (J): every entry of the player's country on the left, the one picked on
//! the right with where it comes from, the conditions that open and close it measured against
//! their limits, its milestones, the forces that push it, and a pin for the side panel.

use crate::keys::{self, id};
use bf_i18n::{Arg as I18nArg, I18n};
use bf_ui_kit::theme::{sans, sans_medium};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align2, Color32, Rect, Sense, Stroke, Ui, Vec2};

use super::region;
use crate::journal::{Arg, Cmp, Condition, Entry, Journal, Kind, Label, Tone, Value};
use crate::text::{date_long, money};
use crate::world::World;

/// Window size: two columns, short enough for 1280 × 720 under the HUD.
const WINDOW: Vec2 = Vec2::new(1000.0, 680.0);
const LIST_W: f32 = 330.0;
/// The smallest the player may make the window.
const MIN_WINDOW: Vec2 = Vec2::new(720.0, 420.0);
const SCROLL_GUTTER: f32 = 16.0;

/// The open window: the entry picked.
#[derive(Clone, Debug, Default)]
pub struct Window {
    pub picked: Option<String>,
    /// The decision to free land, while it is open.
    pub liberate: Liberate,
    /// A decision clicked once and waiting for its confirmation (the entry id).
    pub confirm: Option<String>,
    /// The answer to a demand of independence, while it is open.
    pub demand: super::demand::DemandUi,
    /// The demand on another country being drafted (crisis).
    pub crisis_demand: super::crisis::DemandUi,
    /// A crisis's choice waiting for its confirmation.
    pub crisis: super::crisis::CrisisUi,
    /// A call's choice waiting for its confirmation.
    pub call: super::crisis::CallUi,
    /// The negotiation table: the draft, a counter, an answer waiting for its confirmation.
    pub table: super::treaty_table::TableUi,
    /// The agreements screen: a denunciation being weighed.
    pub agreements: super::agreements::AgreementsUi,
    /// The embargo decision: the country picked, a lift asked.
    pub embargo: super::embargo::EmbargoUi,
    /// The gold standard decision: the choice waiting for its confirmation.
    pub gold: super::gold_decision::GoldUi,
}

/// The id the decision "Libertar território" takes in `Window::picked`.
pub const LIBERATE: &str = "decision:liberate";
/// The prefix of the id a demand of independence takes in `Window::picked`, before the unit.
pub const DEMAND: &str = "decision:demand:";
/// The id the decision to demand of another country takes in `Window::picked`.
pub const CRISIS_DEMAND: &str = "decision:exigency";
/// The prefix of a crisis in `Window::picked`, before its id.
pub const CRISIS: &str = "crisis:";
/// The prefix of a call to honour an agreement in `Window::picked`, before the war's id.
pub const CALL: &str = "call:";
/// `Window::picked` set by a war's button: the war window opens.
pub const OPEN_WAR: &str = "open:war";
/// The id the agreements screen takes in `Window::picked`.
pub const AGREEMENTS: &str = "decision:agreements";
/// The id the gold standard decision takes in `Window::picked`.
pub const GOLD: &str = "decision:gold";
/// The id the embargo decision takes in `Window::picked`.
pub const EMBARGO: &str = "decision:embargo";
/// The id the negotiation table takes in `Window::picked`.
pub const TABLE: &str = "decision:treaty";
/// The prefix of a proposal received in `Window::picked`, before its id.
pub const RECEIVED: &str = "proposal:";

/// The decision to free land: the option picked (index into the view's options), independent
/// or subject, and whether the player is at the confirmation step.
#[derive(Clone, Debug, Default)]
pub struct Liberate {
    pub option: Option<usize>,
    pub subject: bool,
    pub confirming: bool,
}

#[derive(Default)]
pub struct Response {
    pub close: bool,
    /// The player finished moving or resizing the window.
    pub place_done: bool,
    /// Entry ids whose pin was toggled.
    pub toggle_pin: Vec<String>,
    pub open: Option<crate::journal::Screen>,
    /// The player confirmed freeing land: (states, the catalogue's tag, as a subject).
    pub liberate: Option<(Vec<String>, Option<String>, bool)>,
    /// The player confirmed mobilizing (`true`) or sending the reserve home (`false`).
    pub mobilize: Option<bool>,
    /// The player confirmed ending the nuclear programme.
    pub programme_end: bool,
    /// The player confirmed a nuclear test.
    pub nuclear_test: bool,
    /// The player confirmed an answer to a demand of independence: (unit, answer).
    pub answer: Option<(String, bf_sim::borders::independence::Answer)>,
    /// The player confirmed a demand on another country: the target and the proposal.
    pub crisis_demand: Option<(String, bf_sim::treaty::Proposal)>,
    /// The player, target of a crisis, accepted (`true`) or refused it.
    pub crisis_answer: Option<(u32, bool)>,
    /// The player, after a refusal, declared war (`true`) or backed down.
    pub declare: Option<(u32, bool)>,
    /// The player, called by an agreement into war `0`, honoured it (`true`) or not.
    pub call_answer: Option<(u32, bool)>,
    /// The demand being drafted, to weigh (`None` off the demand screen).
    pub draft: Option<(String, bf_sim::treaty::Proposal)>,
    /// The player proposed an agreement, or answered a proposal received.
    pub treaty: Option<super::treaty_table::Sent>,
    /// Open the war window (from a war's line).
    pub open_war: bool,
    /// The player confirmed denouncing this agreement.
    pub denounce: Option<u32>,
    /// The player confirmed imposing (`true`) or lifting an embargo on this country.
    pub embargo: Option<(String, bool)>,
    /// The player confirmed a choice of exchange regime (`None` inside: back to the rule).
    pub gold: Option<Option<bf_sim::econ::gold::GoldChoice>>,
    /// The proposal on the negotiation table, to weigh (`None` off the table).
    pub treaty_draft: Option<(String, bf_sim::treaty::Proposal)>,
}

/// What the journal shows of the negotiation table.
#[derive(Clone, Copy, Default)]
pub struct TableInput<'a> {
    /// `None` with the `negotiation` rule off.
    pub view: Option<&'a crate::treaty_view::TableView>,
    pub draft: Option<&'a crate::treaty_view::Weighed>,
    pub catalogue: Option<&'a bf_sim::treaty::ClauseCatalogue>,
    pub today: Option<laplace_schelling::calendar::Date>,
    /// The player's agreements, breaches and reputations.
    pub agreements: Option<&'a crate::agreements_view::AgreementsView>,
    /// The player's trade by partner and embargoes (the embargo decision).
    pub trade: Option<&'a crate::tariff_view::TariffView>,
    /// The exchange regime and its choices; `None` with the gold standard off.
    pub gold: Option<&'a crate::gold_view::GoldView>,
}

/// What the journal shows of crises: the player's, the draft's weighing, and the country picked
/// on the map as the target of a demand.
#[derive(Clone, Copy, Default)]
pub struct CrisisInput<'a> {
    pub view: Option<&'a crate::crisis_view::CrisisView>,
    pub draft: Option<&'a crate::crisis_view::DraftWeighing>,
    pub me: Option<&'a str>,
    pub target: Option<&'a str>,
}

/// Formats a value.
pub fn value(i18n: &I18n, v: Value) -> String {
    let pct = |p: f64| {
        let digits = if p.abs() < 10.0 { 1 } else { 0 };
        i18n.tr_args(
            "gov-pct",
            &[("value", I18nArg::String(i18n.decimal(p, digits)))],
        )
    };
    // A value never breaks across lines.
    let out = match v {
        Value::Share(s) => pct(s * 100.0),
        Value::Percent(p) => pct(p),
        Value::Money(m) if m < 0.0 => format!("−{}", money(i18n, -m)),
        Value::Money(m) => money(i18n, m),
        Value::Weeks(w) => i18n.tr_args(
            "journal-weeks",
            &[("weeks", I18nArg::String(i18n.int(w.round() as i64)))],
        ),
        Value::Count(n) => i18n.int(n),
        Value::Year(y) => y.to_string(),
        Value::Number(x) => i18n.decimal(x, 2),
    };
    out.replace(' ', "\u{a0}")
}

/// Formats a label.
pub fn text(i18n: &I18n, world: Option<&World>, l: &Label) -> String {
    let args: Vec<(&str, I18nArg<'_>)> = l
        .args
        .iter()
        .map(|(name, a)| {
            let s = match a {
                Arg::Value(v) => value(i18n, *v),
                Arg::Key(id) => i18n.tr(id),
                Arg::State(id) => world
                    .and_then(|w| w.states.get(usize::from(*id).checked_sub(1)?))
                    .map(|s| crate::text::state_name(i18n, &s.def))
                    .unwrap_or_else(|| "—".into()),
                Arg::Date(d) => date_long(i18n, *d),
            };
            (*name, I18nArg::String(s))
        })
        .collect();
    i18n.tr_args(&l.id, &args)
}

pub fn tone_color(tone: Tone) -> Color32 {
    match tone {
        Tone::Neutral => color::ACCENT,
        Tone::Warning => color::ACCENT_HOVER,
        Tone::Danger => color::DANGER,
    }
}

/// The group an entry is listed under.
fn group_key(kind: &Kind) -> &'static str {
    match kind {
        Kind::Works | Kind::LargeWork => "journal-group-works",
        Kind::Research => "journal-group-research",
        Kind::Fiscal(_) | Kind::Gold => "journal-group-fiscal",
        Kind::Movement(_) | Kind::PendingLaw(_) | Kind::LawChanges => "journal-group-politics",
        Kind::Forces { .. } | Kind::Revolt(_) | Kind::NuclearProgramme { .. } => {
            "journal-group-forces"
        }
    }
}

/// Agendas first: entries with a condition and a consequence. Then what is only followed.
const AGENDAS: [&str; 3] = [
    "journal-group-politics",
    "journal-group-fiscal",
    "journal-group-forces",
];
const FOLLOWED: [&str; 2] = ["journal-group-works", "journal-group-research"];

fn is_agenda(kind: &Kind) -> bool {
    AGENDAS.contains(&group_key(kind))
}

/// One line of text no wider than `width`, ended with "…" when it does not fit; the flag
/// says whether it was cut. A number is never cut: when the text holds one, the "…" falls in
/// the words before it ("Fábrica de máq… em 55 estados"), unless the number's part alone does
/// not fit.
pub fn one_line(
    p: &egui::Painter,
    text: &str,
    font: egui::FontId,
    tint: Color32,
    width: f32,
) -> (std::sync::Arc<egui::Galley>, bool) {
    let cut_job = |text: &str, width: f32| {
        let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font.clone(), tint);
        job.wrap = egui::text::TextWrapping {
            max_width: width.max(10.0),
            max_rows: 1,
            break_anywhere: false,
            overflow_character: Some('…'),
        };
        p.layout_job(job)
    };
    // The galley keeps the whole text when it elides: the width says whether it fits.
    let fits = p
        .layout_no_wrap(text.to_owned(), font.clone(), tint)
        .size()
        .x
        <= width.max(10.0);
    let galley = cut_job(text, width);
    if fits {
        return (galley, false);
    }
    if let Some(at) = number_tail(text) {
        let tail = &text[at..];
        let tail_w = p
            .layout_no_wrap(tail.to_owned(), font.clone(), tint)
            .size()
            .x;
        let room = width - tail_w;
        if room >= 24.0 {
            let head = fit_words(p, &text[..at], &font, tint, room);
            // The tail starts at a space: "Fábrica de máq…" + " em 55 estados".
            return (p.layout_no_wrap(format!("{head}{tail}"), font, tint), true);
        }
    }
    (galley, true)
}

/// The longest start of `text`, by characters, that with "…" fits in `width`.
fn fit_words(
    p: &egui::Painter,
    text: &str,
    font: &egui::FontId,
    tint: Color32,
    width: f32,
) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut n = chars.len();
    loop {
        let head: String = chars[..n].iter().collect::<String>().trim_end().to_owned() + "…";
        if n == 0 || p.layout_no_wrap(head.clone(), font.clone(), tint).size().x <= width {
            return head;
        }
        n -= 1;
    }
}

/// Where the part of `text` that holds its first number starts, at the space before it: the
/// word with the first digit, with a short word before it (" em 55 estados"); `None` without
/// a digit or when the number opens the text.
pub fn number_tail(text: &str) -> Option<usize> {
    let digit = text.find(|c: char| c.is_ascii_digit())?;
    let word = text[..digit].rfind(' ').map_or(0, |i| i + 1);
    if word == 0 {
        return None;
    }
    let before = text[..word - 1].rfind(' ').map_or(0, |i| i + 1);
    let short = text[before..word - 1].chars().count() <= 3 && before > 0;
    Some(if short { before - 1 } else { word - 1 })
}

/// The mark at the head of a line: drawn, since the fonts lack the geometric glyphs.
#[derive(Clone, Copy)]
pub enum Mark {
    /// A filled dot: the condition holds.
    Holds,
    /// A ring: it does not.
    Fails,
    /// A small dot: the simulation does not tell.
    Unknown,
    /// A filled diamond: a milestone reached.
    Reached,
    /// An empty diamond: one still ahead.
    Ahead,
    Plus,
    Minus,
}

pub fn mark(ui: &mut Ui, m: Mark, tint: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 16.0), Sense::hover());
    let c = rect.center();
    let p = ui.painter();
    let diamond = |r: f32| {
        vec![
            c + Vec2::new(0.0, -r),
            c + Vec2::new(r, 0.0),
            c + Vec2::new(0.0, r),
            c + Vec2::new(-r, 0.0),
        ]
    };
    match m {
        Mark::Holds => {
            p.circle_filled(c, 4.0, tint);
        }
        Mark::Fails => {
            p.circle_stroke(c, 3.5, Stroke::new(1.3, tint));
        }
        Mark::Unknown => {
            p.circle_filled(c, 1.8, tint);
        }
        Mark::Reached => {
            p.add(egui::Shape::convex_polygon(
                diamond(4.5),
                tint,
                Stroke::NONE,
            ));
        }
        Mark::Ahead => {
            p.add(egui::Shape::closed_line(
                diamond(4.0),
                Stroke::new(1.3, tint),
            ));
        }
        Mark::Plus => {
            p.hline(c.x - 4.0..=c.x + 4.0, c.y, Stroke::new(1.6, tint));
            p.vline(c.x, c.y - 4.0..=c.y + 4.0, Stroke::new(1.6, tint));
        }
        Mark::Minus => {
            p.hline(c.x - 4.0..=c.x + 4.0, c.y, Stroke::new(1.6, tint));
        }
    }
}

/// A thin progress bar across `width`.
pub fn bar(ui: &mut Ui, share: f32, fill: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 5.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, radius::SM, color::SURFACE_SUNKEN);
    let mut done = rect;
    done.set_width(rect.width() * share.clamp(0.0, 1.0));
    if done.width() > 0.5 {
        p.rect_filled(done, radius::SM, fill);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    world: Option<&World>,
    journal: Option<&Journal>,
    liberation: Option<&crate::sim::LiberationView>,
    demands: Option<&crate::demands_view::DemandsView>,
    crises: CrisisInput<'_>,
    table: TableInput<'_>,
    pinned: &[String],
    window: &mut Window,
    place: &mut crate::hud::WindowPlace,
) -> Response {
    let mut out = Response::default();
    let top = screen.top() + super::top_bar::LEFT_BOTTOM;
    let size = WINDOW.min(Vec2::new(
        screen.width() - 32.0,
        screen.bottom() - top - space::LG,
    ));
    let default = Rect::from_min_size(egui::pos2(screen.center().x - size.x / 2.0, top), size);
    let (rect, moved) = super::window_frame(ui, "journal", default, screen, place, MIN_WINDOW);
    out.place_done |= moved;
    if window.picked.as_deref() == Some(OPEN_WAR) {
        window.picked = None;
        out.open_war = true;
    }
    region(ui, rect.shrink(space::XL), |ui| {
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("journal-title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = i18n.tr("gov-close");
                let tip = keys::tip(ui.ctx(), &label, crate::keys::id::BACK);
                let close = bf_ui_kit::close_button(ui, &label, &tip);
                if close.clicked() || keys::single(ui, "journal.close", 'X', close.rect) {
                    out.close = true;
                }
            });
        });
        bf_ui_kit::divider(ui);
        // A decision takes the whole window: it is rare, and it asks for attention.
        if window.picked.as_deref() == Some(LIBERATE) {
            let back = Button::ghost(&format!("←  {}", i18n.tr("journal-agendas")))
                .size(ButtonSize::Small)
                .show(ui);
            if back.clicked() || keys::single(ui, "journal.decision.back", 'B', back.rect) {
                window.picked = None;
                window.liberate = Liberate::default();
                return;
            }
            super::liberate::show(ui, i18n, world, liberation, &mut window.liberate, &mut out);
            return;
        }
        let demand = window
            .picked
            .as_deref()
            .and_then(|p| p.strip_prefix(DEMAND))
            .and_then(|unit| demands?.demands.iter().find(|d| d.view.unit == unit));
        if let (Some(d), Some(all)) = (demand, demands) {
            let back = Button::ghost(&format!("←  {}", i18n.tr("journal-agendas")))
                .size(ButtonSize::Small)
                .show(ui);
            if back.clicked() || keys::single(ui, "journal.decision.back", 'B', back.rect) {
                window.picked = None;
                window.demand = Default::default();
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("demand")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Clear of the scroll bar.
                    ui.set_max_width(ui.available_width() - space::LG);
                    super::scroll_keys(ui);
                    super::demand::show(ui, i18n, world, &all.tags, d, &mut window.demand, &mut out)
                });
            return;
        }
        // The agreements screen takes the whole window too.
        if let Some(view) = table
            .agreements
            .filter(|_| window.picked.as_deref() == Some(AGREEMENTS))
        {
            let back = Button::ghost(&format!("←  {}", i18n.tr("journal-agendas")))
                .size(ButtonSize::Small)
                .show(ui);
            if back.clicked() || keys::single(ui, "journal.decision.back", 'B', back.rect) {
                window.picked = None;
                window.agreements = Default::default();
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("agreements")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - space::LG);
                    super::agreements::show(ui, i18n, view, &mut window.agreements, &mut out);
                });
            return;
        }
        // The gold standard decision takes the whole window too.
        if let Some(g) = table
            .gold
            .filter(|_| window.picked.as_deref() == Some(GOLD))
        {
            let back = Button::ghost(&format!("←  {}", i18n.tr("journal-agendas")))
                .size(ButtonSize::Small)
                .show(ui);
            if back.clicked() || keys::single(ui, "journal.decision.back", 'B', back.rect) {
                window.picked = None;
                window.gold = Default::default();
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("gold-decision")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - space::LG);
                    super::scroll_keys(ui);
                    out.gold = super::gold_decision::show(ui, i18n, g, &mut window.gold);
                });
            return;
        }
        // The embargo decision takes the whole window too.
        if let Some(view) = table
            .trade
            .filter(|_| window.picked.as_deref() == Some(EMBARGO))
        {
            let back = Button::ghost(&format!("←  {}", i18n.tr("journal-agendas")))
                .size(ButtonSize::Small)
                .show(ui);
            if back.clicked() || keys::single(ui, "journal.decision.back", 'B', back.rect) {
                window.picked = None;
                window.embargo = Default::default();
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("embargo")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - space::LG);
                    super::scroll_keys(ui);
                    super::embargo::show(ui, i18n, world, view, &mut window.embargo, &mut out);
                });
            return;
        }
        // The negotiation table and a proposal received take the whole window too.
        let picked_received = window
            .picked
            .as_deref()
            .and_then(|p| p.strip_prefix(RECEIVED))
            .and_then(|id| id.parse::<u32>().ok())
            .and_then(|id| table.view?.incoming.iter().find(|l| l.id == id));
        let on_table = window.picked.as_deref() == Some(TABLE) || window.table.counter.is_some();
        if let Some(view) = table.view.filter(|_| on_table || picked_received.is_some()) {
            let back = Button::ghost(&format!("←  {}", i18n.tr("journal-agendas")))
                .size(ButtonSize::Small)
                .show(ui);
            if back.clicked() || keys::single(ui, "journal.decision.back", 'B', back.rect) {
                window.picked = None;
                window.table = Default::default();
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("treaty-table")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - space::LG);
                    super::scroll_keys(ui);
                    if on_table {
                        if let Some(me) = crises.me {
                            out.treaty_draft = super::treaty_table::show_table(
                                ui,
                                i18n,
                                world,
                                table.catalogue,
                                me,
                                crises.target,
                                view,
                                table.draft,
                                &mut window.table,
                                &mut out,
                            );
                        }
                    } else if let Some(l) = picked_received {
                        super::treaty_table::show_received(
                            ui,
                            i18n,
                            world,
                            l,
                            &mut window.table,
                            &mut out,
                        );
                    }
                });
            // An answer sent, or a counter dropped, goes back to the agendas.
            if out.treaty.is_some() {
                window.picked = None;
            }
            return;
        }
        // The decision to demand of another country, and a crisis, take the whole window too.
        let picked_crisis = window
            .picked
            .as_deref()
            .and_then(|p| p.strip_prefix(CRISIS))
            .and_then(|id| id.parse::<u32>().ok())
            .and_then(|id| crises.view?.crises.iter().find(|c| c.id == id));
        let picked_call = window
            .picked
            .as_deref()
            .and_then(|p| p.strip_prefix(CALL))
            .and_then(|id| id.parse::<u32>().ok())
            .and_then(|id| crises.view?.calls.iter().find(|c| c.war == id));
        let on_demand = window.picked.as_deref() == Some(CRISIS_DEMAND);
        if on_demand || picked_crisis.is_some() || picked_call.is_some() {
            let back = Button::ghost(&format!("←  {}", i18n.tr("journal-agendas")))
                .size(ButtonSize::Small)
                .show(ui);
            if back.clicked() || keys::single(ui, "journal.decision.back", 'B', back.rect) {
                window.picked = None;
                window.crisis_demand = Default::default();
                window.crisis = Default::default();
                window.call = Default::default();
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("crisis")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - space::LG);
                    super::scroll_keys(ui);
                    if let Some(c) = picked_call {
                        super::crisis::show_call(ui, i18n, c, &mut window.call, &mut out);
                    } else if let Some(l) = picked_crisis {
                        super::crisis::show_crisis(
                            ui,
                            i18n,
                            world,
                            l,
                            &mut window.crisis,
                            &mut out,
                        );
                    } else if let Some(me) = crises.me {
                        out.draft = super::crisis::show_demand(
                            ui,
                            i18n,
                            world,
                            me,
                            crises.target,
                            crises.draft,
                            &mut window.crisis_demand,
                            &mut out,
                        );
                    }
                });
            return;
        }
        let decisions = |ui: &mut Ui, window: &mut Window| {
            bf_ui_kit::heading(ui, &i18n.tr("decisions-title"));
            bf_ui_kit::small(ui, &i18n.tr("decisions-hint"));
            let b = Button::secondary(&i18n.tr("liberate-title"))
                .size(ButtonSize::Small)
                .show(ui);
            if b.clicked() || keys::single(ui, "journal.decision.liberate", 'D', b.rect) {
                window.picked = Some(LIBERATE.to_owned());
                window.liberate = Liberate::default();
            }
            // A colony's demand waits for an answer: one button each, in the danger colour.
            for (k, d) in demands.iter().flat_map(|d| &d.demands).enumerate() {
                let label = i18n.tr_args(
                    "independence-demand-title",
                    &[(
                        "unit",
                        bf_i18n::Arg::String(crate::text::country_name(i18n, &d.view.unit)),
                    )],
                );
                let b = Button::danger(&label).size(ButtonSize::Small).show(ui);
                if b.clicked()
                    || keys::item(
                        ui,
                        'D',
                        format!("journal.decision.demand.{}", d.view.unit),
                        &(k + 1).to_string(),
                        b.rect,
                    )
                {
                    window.picked = Some(format!("{DEMAND}{}", d.view.unit));
                    window.demand = Default::default();
                }
            }
            // The country's agreements, breaches and reputations, and the way to denounce.
            if let Some(a) = table.agreements {
                let b = Button::secondary(&i18n.tr_args(
                    "agreements-open",
                    &[("n", I18nArg::String(a.in_force.len().to_string()))],
                ))
                .size(ButtonSize::Small)
                .show(ui);
                if b.clicked() || keys::single(ui, "journal.decision.agreements", 'O', b.rect) {
                    window.picked = Some(AGREEMENTS.to_owned());
                    window.agreements = Default::default();
                }
            }
            // The gold standard: adhere or leave, with what each choice means this week.
            if let Some(g) = table.gold {
                let b = Button::secondary(&super::gold_decision::open_label(i18n, g))
                    .size(ButtonSize::Small)
                    .show(ui);
                if b.clicked() || keys::single(ui, "journal.decision.gold", 'U', b.rect) {
                    window.picked = Some(GOLD.to_owned());
                    window.gold = Default::default();
                }
            }
            // Embargoes: imposing one, and those in force by the country and on it.
            if let Some(t) = table.trade {
                let b = Button::secondary(&super::embargo::open_label(i18n, t))
                    .size(ButtonSize::Small)
                    .show(ui);
                if b.clicked() || keys::single(ui, "journal.decision.embargo", 'M', b.rect) {
                    window.picked = Some(EMBARGO.to_owned());
                    window.embargo.target = None;
                    window.embargo.lift = None;
                }
            }
            // With the negotiation rule on: propose an agreement to the country picked on the map,
            // the proposals received and the player's own.
            if let Some(v) = table.view {
                let label = match crises.target.filter(|t| Some(*t) != crises.me) {
                    Some(t) => i18n.tr_args(
                        "table-propose-to",
                        &[
                            (
                                "country",
                                I18nArg::String(crate::text::country_name(i18n, t)),
                            ),
                            (
                                "article",
                                I18nArg::String(crate::text::country_article(i18n, t)),
                            ),
                        ],
                    ),
                    None => i18n.tr("table-title"),
                };
                let b = Button::secondary(&label).size(ButtonSize::Small).show(ui);
                if b.clicked() || keys::single(ui, "journal.decision.treaty", 'T', b.rect) {
                    window.picked = Some(TABLE.to_owned());
                    window.table.counter = None;
                    window.table.fixed = false;
                }
                for (k, l) in v.incoming.iter().enumerate() {
                    let label = table.today.map_or_else(String::new, |d| {
                        super::treaty_table::received_label(i18n, l, d)
                    });
                    let b = Button::danger(&label).size(ButtonSize::Small).show(ui);
                    if b.clicked()
                        || keys::item(
                            ui,
                            'P',
                            format!("journal.proposal.{}", l.id),
                            &(k + 1).to_string(),
                            b.rect,
                        )
                    {
                        window.picked = Some(format!("{RECEIVED}{}", l.id));
                        window.table = Default::default();
                    }
                }
                if !v.outgoing.is_empty() || !v.closed.is_empty() {
                    bf_ui_kit::gap(ui, space::SM);
                    bf_ui_kit::caption(ui, &i18n.tr("table-section"));
                    for l in &v.outgoing {
                        bf_ui_kit::small(ui, &super::treaty_table::outgoing_label(i18n, l));
                    }
                    if let Some(me) = crises.me {
                        for c in &v.closed {
                            bf_ui_kit::small(ui, &super::treaty_table::closed_label(i18n, me, c));
                        }
                    }
                }
            }
            // With the crisis rule on: demand of the country picked on the map, and the crises
            // and wars the country is in.
            if let Some(v) = crises.view {
                let label = match crises.target {
                    Some(t) => i18n.tr_args(
                        "crisis-demand-from",
                        &[
                            (
                                "country",
                                I18nArg::String(crate::text::country_name(i18n, t)),
                            ),
                            (
                                "article",
                                I18nArg::String(crate::text::country_article(i18n, t)),
                            ),
                        ],
                    ),
                    None => i18n.tr("crisis-demand-title"),
                };
                let b = Button::secondary(&label).size(ButtonSize::Small).show(ui);
                if b.clicked() || keys::single(ui, "journal.decision.exigency", 'E', b.rect) {
                    window.picked = Some(CRISIS_DEMAND.to_owned());
                }
                if !v.crises.is_empty() || !v.wars.is_empty() || !v.calls.is_empty() {
                    bf_ui_kit::gap(ui, space::SM);
                    bf_ui_kit::caption(ui, &i18n.tr("crisis-section"));
                }
                // A call by an agreement waits for an answer, like a demand.
                for (k, c) in v.calls.iter().enumerate() {
                    let b = Button::danger(&super::crisis::call_label(i18n, c))
                        .size(ButtonSize::Small)
                        .show(ui);
                    if b.clicked()
                        || keys::item(
                            ui,
                            'H',
                            format!("journal.call.{}", c.war),
                            &(k + 1).to_string(),
                            b.rect,
                        )
                    {
                        window.picked = Some(format!("{CALL}{}", c.war));
                    }
                }
                for (k, c) in v.crises.iter().enumerate() {
                    let b = Button::danger(&super::crisis::list_label(i18n, c))
                        .size(ButtonSize::Small)
                        .show(ui);
                    if b.clicked()
                        || keys::item(
                            ui,
                            'K',
                            format!("journal.crisis.{}", c.id),
                            &(k + 1).to_string(),
                            b.rect,
                        )
                    {
                        window.picked = Some(format!("{CRISIS}{}", c.id));
                    }
                }
                for (k, w) in v.wars.iter().enumerate() {
                    let names = |s: &[String]| {
                        s.iter()
                            .map(|t| crate::text::country_name(i18n, t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    let label = i18n.tr_args(
                        "crisis-war-line",
                        &[
                            ("a", I18nArg::String(names(&w.side_a))),
                            ("b", I18nArg::String(names(&w.side_b))),
                            ("year", I18nArg::String(w.since.0.to_string())),
                        ],
                    );
                    // The war's window: its fronts, postures and peace.
                    let b = Button::danger(&label).size(ButtonSize::Small).show(ui);
                    if b.clicked()
                        || keys::item(
                            ui,
                            'G',
                            format!("journal.war.{}", w.id),
                            &(k + 1).to_string(),
                            b.rect,
                        )
                    {
                        window.picked = Some(OPEN_WAR.to_owned());
                    }
                }
            }
        };
        let Some(journal) = journal else {
            bf_ui_kit::muted(ui, &i18n.tr("journal-not-ready"));
            return;
        };
        if journal.entries.is_empty() {
            bf_ui_kit::muted(ui, &i18n.tr("journal-empty"));
            bf_ui_kit::gap(ui, space::LG);
            decisions(ui, window);
            return;
        }
        let picked = window
            .picked
            .as_deref()
            .and_then(|id| journal.entry(id))
            .or_else(|| journal.entries.iter().find(|e| is_agenda(&e.kind)))
            .unwrap_or(&journal.entries[0]);
        let picked_id = picked.id.clone();
        let body = ui.available_rect_before_wrap();
        let list = Rect::from_min_size(body.min, Vec2::new(LIST_W, body.height()));
        let detail = Rect::from_min_max(egui::pos2(list.right() + space::XL, body.top()), body.max);
        ui.painter().vline(
            list.right() + space::XL / 2.0,
            body.y_range(),
            Stroke::new(1.0, color::BORDER),
        );
        region(ui, list, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("journal-list")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(LIST_W - SCROLL_GUTTER);
                    super::scroll_keys(ui);
                    decisions(ui, window);
                    bf_ui_kit::gap(ui, space::SM);
                    bf_ui_kit::heading(ui, &i18n.tr("journal-agendas"));
                    if !journal.entries.iter().any(|e| is_agenda(&e.kind)) {
                        bf_ui_kit::small(ui, &i18n.tr("journal-agendas-none"));
                        bf_ui_kit::gap(ui, space::SM);
                    }
                    for (i, g) in AGENDAS.iter().chain(&FOLLOWED).enumerate() {
                        if i == AGENDAS.len() {
                            bf_ui_kit::gap(ui, space::SM);
                            bf_ui_kit::heading(ui, &i18n.tr("journal-followed"));
                            bf_ui_kit::small(ui, &i18n.tr("journal-followed-hint"));
                            bf_ui_kit::gap(ui, space::XS);
                        }
                        let g = *g;
                        let entries: Vec<&Entry> = journal
                            .entries
                            .iter()
                            .filter(|e| group_key(&e.kind) == g)
                            .collect();
                        if entries.is_empty() {
                            continue;
                        }
                        bf_ui_kit::caption(ui, &i18n.tr(g));
                        for e in entries {
                            let pin = pinned.contains(&e.id);
                            let row = list_row(ui, i18n, world, e, e.id == picked_id, pin);
                            let id = format!("journal.entry.{}", e.id);
                            if row.clicked() || keys::item(ui, 'H', id, &e.id, row.rect) {
                                window.picked = Some(e.id.clone());
                            }
                        }
                        bf_ui_kit::gap(ui, space::SM);
                    }
                });
        });
        region(ui, detail, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("journal-detail")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(detail.width() - SCROLL_GUTTER);
                    super::scroll_keys(ui);
                    entry_detail(ui, i18n, world, picked, pinned, &mut out);
                    decision(ui, i18n, picked, &mut window.confirm, &mut out);
                });
        });
    });
    out.place_done |= super::window_grip(ui, "journal", rect, place, MIN_WINDOW);
    out
}

fn list_row(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    e: &Entry,
    selected: bool,
    pinned: bool,
) -> egui::Response {
    let title = text(i18n, world, &e.title);
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 46.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title.clone()));
    let p = ui.painter();
    let fill = if selected {
        color::ACCENT_SOFT
    } else if response.hovered() {
        color::SURFACE_HOVER
    } else {
        color::SURFACE_RAISED
    };
    p.rect(
        rect,
        radius::MD,
        fill,
        Stroke::new(
            1.0,
            if selected {
                color::ACCENT.gamma_multiply(0.7)
            } else {
                color::BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );
    let mark = Rect::from_min_size(rect.min + Vec2::new(8.0, 10.0), Vec2::new(3.0, 26.0));
    p.rect_filled(mark, 1.0, tone_color(e.tone));
    let x = rect.left() + 20.0;
    let title_w = width - 20.0 - if pinned { 54.0 } else { 10.0 };
    let (galley, mut cut) = one_line(p, &title, sans_medium(size::SMALL), color::TEXT, title_w);
    p.galley(egui::pos2(x, rect.top() + 6.0), galley, color::TEXT);
    let mut full = title.clone();
    if pinned {
        p.text(
            egui::pos2(rect.right() - 10.0, rect.top() + 8.0),
            Align2::RIGHT_TOP,
            i18n.tr("journal-pinned"),
            sans(size::CAPTION),
            color::ACCENT,
        );
    }
    if let Some((share, line)) = &e.progress {
        let line = text(i18n, world, line);
        let (g, line_cut) = one_line(
            p,
            &line,
            sans(size::CAPTION),
            color::TEXT_MUTED,
            width - 30.0,
        );
        p.galley(egui::pos2(x, rect.top() + 25.0), g, color::TEXT_MUTED);
        cut |= line_cut;
        full = format!("{full}\n{line}");
        let track = Rect::from_min_size(
            egui::pos2(x, rect.bottom() - 6.0),
            Vec2::new(width - 30.0, 3.0),
        );
        p.rect_filled(track, 1.0, color::SURFACE_SUNKEN);
        let mut done = track;
        done.set_width(track.width() * share.clamp(0.0, 1.0));
        p.rect_filled(done, 1.0, tone_color(e.tone));
    }
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if cut {
        response.on_hover_text(bf_ui_kit::tooltip_text(&full))
    } else {
        response
    }
}

fn condition_line(ui: &mut Ui, i18n: &I18n, world: Option<&World>, c: &Condition) {
    let (glyph, tint) = match c.holds {
        Some(true) => (Mark::Holds, color::POSITIVE),
        Some(false) => (Mark::Fails, color::TEXT_MUTED),
        None => (Mark::Unknown, color::TEXT_FAINT),
    };
    let mut parts = vec![text(i18n, world, &c.label)];
    if let Some(m) = c.measured {
        parts.push(i18n.tr_args(
            "journal-measured",
            &[("value", I18nArg::String(value(i18n, m)))],
        ));
    }
    if let Some((cmp, limit)) = c.limit {
        parts.push(i18n.tr_args(
            match cmp {
                Cmp::AtLeast => "journal-limit-at-least",
                Cmp::AtMost => "journal-limit-at-most",
            },
            &[("value", I18nArg::String(value(i18n, limit)))],
        ));
        // Both sides round alike: say how far apart they are.
        if let Some(m) = c.measured
            && value(i18n, m) == value(i18n, limit)
            && let Some(diff) = difference(m, limit)
        {
            parts.push(i18n.tr_args(
                "journal-difference",
                &[("value", I18nArg::String(value(i18n, diff)))],
            ));
        }
    }
    if c.holds.is_none() {
        parts.push(i18n.tr("journal-unknown"));
    }
    mark_line(ui, glyph, tint, parts.join(" · "), None);
}

/// The measured value minus the limit, in the same unit; `None` when they are equal.
fn difference(m: Value, limit: Value) -> Option<Value> {
    let d = match (m, limit) {
        (Value::Money(a), Value::Money(b)) => Value::Money(a - b),
        (Value::Share(a), Value::Share(b)) => Value::Share(a - b),
        (Value::Percent(a), Value::Percent(b)) => Value::Percent(a - b),
        (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
        _ => return None,
    };
    let zero = match d {
        Value::Money(x) | Value::Share(x) | Value::Percent(x) | Value::Number(x) => x == 0.0,
        _ => true,
    };
    (!zero).then_some(d)
}

/// A line with its mark on the left, wrapped text, and a value on the right.
fn mark_line(ui: &mut Ui, glyph: Mark, tint: Color32, line: String, right: Option<String>) {
    let width = ui.available_width();
    let p = ui.painter().clone();
    let right = right.map(|r| p.layout_no_wrap(r, sans(size::SMALL), color::TEXT_MUTED));
    let right_w = right.as_ref().map_or(0.0, |g| g.size().x + space::MD);
    let text_w = (width - 12.0 - space::SM - right_w).max(40.0);
    let galley = p.layout(line, sans(size::SMALL), color::TEXT, text_w);
    let height = galley.size().y.max(16.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let mark_rect = Rect::from_min_size(rect.min, Vec2::new(12.0, 16.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(mark_rect));
    mark(&mut child, glyph, tint);
    p.galley(
        rect.min + Vec2::new(12.0 + space::SM, 0.0),
        galley,
        color::TEXT,
    );
    if let Some(g) = right {
        p.galley(
            egui::pos2(rect.right() - g.size().x, rect.top()),
            g,
            color::TEXT_MUTED,
        );
    }
}

fn section(ui: &mut Ui, i18n: &I18n, key: &str) {
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::caption(ui, &i18n.tr(key));
}

/// The decision of an agenda, in two steps: the first click asks, the second confirms. The
/// forces' entry has one (mobilize, or send the reserve home), and the nuclear programme's
/// (test the weapon, end the programme).
fn decision(ui: &mut Ui, i18n: &I18n, e: &Entry, confirm: &mut Option<String>, out: &mut Response) {
    if let Kind::NuclearProgramme { can_test } = e.kind {
        if can_test {
            two_steps(
                ui,
                i18n,
                e,
                confirm,
                [
                    "journal-programme-test",
                    "journal-programme-test-confirm",
                    "journal-programme-test-explain",
                ],
                ("journal.decision.test", 'T'),
                || out.nuclear_test = true,
            );
        }
        two_steps(
            ui,
            i18n,
            e,
            confirm,
            [
                "journal-programme-end",
                "journal-programme-end-confirm",
                "journal-programme-end-explain",
            ],
            ("journal.decision", 'D'),
            || out.programme_end = true,
        );
        return;
    }
    let Kind::Forces { mobilized } = e.kind else {
        return;
    };
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("journal-decision"));
    let (ask, sure, explain) = if mobilized {
        (
            "journal-demobilize",
            "journal-demobilize-confirm",
            "journal-demobilize-explain",
        )
    } else {
        (
            "journal-mobilize",
            "journal-mobilize-confirm",
            "journal-mobilize-explain",
        )
    };
    bf_ui_kit::small(ui, &i18n.tr(explain));
    if confirm.as_deref() == Some(e.id.as_str()) {
        ui.horizontal(|ui| {
            let yes = Button::primary(&i18n.tr(sure))
                .size(ButtonSize::Small)
                .show(ui);
            if yes.clicked() || keys::single(ui, "journal.decision.yes", 'Y', yes.rect) {
                out.mobilize = Some(!mobilized);
                *confirm = None;
            }
            let no = Button::ghost(&i18n.tr("journal-decision-cancel"))
                .size(ButtonSize::Small)
                .show(ui);
            if no.clicked() || keys::single(ui, "journal.decision.no", 'N', no.rect) {
                *confirm = None;
            }
        });
    } else {
        let first = Button::secondary(&i18n.tr(ask))
            .size(ButtonSize::Small)
            .show(ui);
        if first.clicked() || keys::single(ui, "journal.decision", 'D', first.rect) {
            *confirm = Some(e.id.clone());
        }
    }
}

/// One decision in two steps: `keys` are the first button, the confirmation and the sentence
/// that says what changes; `id` and `letter` the shortcut; `done` runs on the confirmation.
fn two_steps(
    ui: &mut Ui,
    i18n: &I18n,
    e: &Entry,
    confirm: &mut Option<String>,
    keys: [&str; 3],
    (id, letter): (&str, char),
    done: impl FnOnce(),
) {
    let [ask, sure, explain] = keys;
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("journal-decision"));
    bf_ui_kit::small(ui, &i18n.tr(explain));
    let asked = format!("{}#{ask}", e.id);
    if confirm.as_deref() == Some(asked.as_str()) {
        ui.horizontal(|ui| {
            let yes = Button::primary(&i18n.tr(sure))
                .size(ButtonSize::Small)
                .show(ui);
            if yes.clicked() || keys::single(ui, &format!("{id}.yes"), 'Y', yes.rect) {
                done();
                *confirm = None;
                return;
            }
            let no = Button::ghost(&i18n.tr("journal-decision-cancel"))
                .size(ButtonSize::Small)
                .show(ui);
            if no.clicked() || keys::single(ui, &format!("{id}.no"), 'N', no.rect) {
                *confirm = None;
            }
        });
    } else {
        let first = Button::secondary(&i18n.tr(ask))
            .size(ButtonSize::Small)
            .show(ui);
        if first.clicked() || keys::single(ui, id, letter, first.rect) {
            *confirm = Some(asked);
        }
    }
}

fn entry_detail(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    e: &Entry,
    pinned: &[String],
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::XS;
    let is_pinned = pinned.contains(&e.id);
    ui.horizontal(|ui| {
        bf_ui_kit::heading(ui, &text(i18n, world, &e.title));
    });
    ui.horizontal(|ui| {
        let unpin = i18n.tr("journal-unpin");
        let pin_label = i18n.tr("journal-pin");
        let pin = if is_pinned {
            Button::secondary(&unpin)
        } else {
            Button::primary(&pin_label)
        };
        let pin = pin.size(ButtonSize::Small).show(ui);
        let label = if is_pinned { &unpin } else { &pin_label };
        let pin = pin.on_hover_text(keys::tip(ui.ctx(), label, id::JOURNAL_PIN));
        if keys::pressed(ui, id::JOURNAL_PIN, &pin) || keys::fired(ui.ctx(), id::JOURNAL_PIN) {
            out.toggle_pin.push(e.id.clone());
        }
        let go = match e.screen {
            crate::journal::Screen::Budget => "journal-open-budget",
            crate::journal::Screen::Money => "journal-open-money",
            crate::journal::Screen::Laws => "journal-open-laws",
            crate::journal::Screen::Research => "journal-open-research",
            crate::journal::Screen::Forces => "journal-open-forces",
        };
        let open = Button::ghost(&i18n.tr(go)).size(ButtonSize::Small).show(ui);
        let open = open.on_hover_text(keys::tip(ui.ctx(), &i18n.tr(go), id::JOURNAL_GO));
        if keys::pressed(ui, id::JOURNAL_GO, &open) || keys::fired(ui.ctx(), id::JOURNAL_GO) {
            out.open = Some(e.screen);
        }
    });
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::small(ui, &text(i18n, world, &e.cause));
    if let Some((share, line)) = &e.progress {
        bf_ui_kit::gap(ui, space::SM);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(text(i18n, world, line))
                    .font(sans_medium(size::SMALL))
                    .color(color::TEXT),
            );
        });
        let w = ui.available_width();
        bar(ui, *share, tone_color(e.tone), w);
    }
    if !e.enter.is_empty() {
        section(ui, i18n, "journal-enter");
        for c in &e.enter {
            condition_line(ui, i18n, world, c);
        }
    }
    if !e.milestones.is_empty() {
        section(ui, i18n, "journal-milestones");
        for m in &e.milestones {
            let mut line = text(i18n, world, &m.label);
            if let Some(d) = m.date {
                line = format!("{} · {line}", date_long(i18n, d));
            }
            let (glyph, tint) = if m.reached {
                (Mark::Reached, color::POSITIVE)
            } else {
                (Mark::Ahead, color::TEXT_MUTED)
            };
            let right = m.progress.map(|p| value(i18n, Value::Share(f64::from(p))));
            mark_line(ui, glyph, tint, line, right);
        }
    }
    if !e.forces.is_empty() {
        section(ui, i18n, "journal-forces");
        for f in &e.forces {
            let (glyph, tint) = if f.helps {
                (Mark::Plus, color::POSITIVE)
            } else {
                (Mark::Minus, color::DANGER)
            };
            let right = f.value.map(|v| value(i18n, v));
            mark_line(ui, glyph, tint, text(i18n, world, &f.label), right);
        }
    }
    if !e.exit.is_empty() {
        section(ui, i18n, "journal-exit");
        for c in &e.exit {
            condition_line(ui, i18n, world, c);
        }
    }
    if let Some(c) = &e.completes {
        section(ui, i18n, "journal-completes");
        bf_ui_kit::small(ui, &text(i18n, world, c));
    }
}

#[cfg(test)]
mod number_tail_tests {
    use super::number_tail;

    #[test]
    fn the_number_and_its_short_word_stay() {
        let t = "Fábrica de máquinas em 55 estados";
        assert_eq!(&t[number_tail(t).unwrap()..], " em 55 estados");
        let t = "Usina elétrica ×12 · Budapeste";
        assert_eq!(&t[number_tail(t).unwrap()..], " ×12 · Budapeste");
        assert_eq!(number_tail("Ferrovia · Budapeste"), None);
        assert_eq!(number_tail("55 estados"), None);
    }
}
