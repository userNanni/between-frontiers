//! Picking another country by name, from the keyboard: the screens that act on a country (a
//! demand, a proposal at the negotiation table) take the one selected on the map, or the one
//! found here — a search field, the countries that match, a key tip on each, Enter for the
//! first. So the whole path from a demand to the peace needs no mouse.

use bf_i18n::I18n;
use bf_ui_kit::tokens::space;
use bf_ui_kit::{Button, ButtonSize};
use egui::Ui;

use crate::keys;
use crate::text::country_name;
use crate::world::World;

/// The picker's state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FindUi {
    pub open: bool,
    pub query: String,
}

/// Countries shown at most; the search narrows them.
const SHOWN: usize = 40;

/// A button that opens the search ("Choose a country…", key `open_key`), and the search while
/// open. Returns the tag picked, and closes. `except` is left out of the list (the player's).
pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    world: &World,
    except: &str,
    st: &mut FindUi,
    id: &str,
    open_key: char,
) -> Option<String> {
    if !st.open {
        let b = Button::ghost(&i18n.tr("find-country"))
            .size(ButtonSize::Small)
            .show(ui);
        if b.clicked() || keys::single(ui, &format!("{id}.find"), open_key, b.rect) {
            st.open = true;
            st.query.clear();
        }
        return None;
    }
    let q = st.query.trim().to_lowercase();
    // Countries with land on the map, by name in the interface language.
    let mut found: Vec<(String, String)> = world
        .countries
        .iter()
        .filter(|c| c.tag() != except && !(c.states.is_empty() && c.colonial_states.is_empty()))
        .map(|c| (c.tag().to_owned(), country_name(i18n, c.tag())))
        .filter(|(tag, name)| {
            q.is_empty() || name.to_lowercase().contains(&q) || tag.to_lowercase() == q
        })
        .collect();
    found.sort_by(|a, b| a.1.cmp(&b.1));
    let mut picked = None;
    ui.horizontal(|ui| {
        let field = bf_ui_kit::text_field(ui, &mut st.query, 240.0);
        if !field.has_focus() && !field.lost_focus() {
            field.request_focus();
        }
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            picked = found.first().map(|(t, _)| t.clone());
        }
        let close = Button::ghost(&i18n.tr("find-country-cancel"))
            .size(ButtonSize::Small)
            .show(ui);
        // Esc leaves the key tips first, so the search closes by its button (or a pick).
        if close.clicked() || keys::single(ui, &format!("{id}.find.cancel"), 'V', close.rect) {
            st.open = false;
        }
    });
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "find-country-count",
            &[("n", bf_i18n::Arg::Num(found.len() as f64))],
        ),
    );
    egui::ScrollArea::vertical()
        .id_salt((id, "find"))
        .max_height(160.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(space::XS, space::XS);
                for (k, (tag, name)) in found.iter().take(SHOWN).enumerate() {
                    let b = Button::secondary(name).size(ButtonSize::Small).show(ui);
                    if b.clicked()
                        || keys::item(
                            ui,
                            'Z',
                            format!("{id}.find.{tag}"),
                            &(k + 1).to_string(),
                            b.rect,
                        )
                    {
                        picked = Some(tag.clone());
                    }
                }
            });
        });
    if picked.is_some() {
        st.open = false;
        st.query.clear();
    }
    picked
}
