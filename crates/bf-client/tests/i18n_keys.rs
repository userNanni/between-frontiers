//! Every message id written literally in the client exists in every locale.
//!
//! Catches a message renamed or removed from the `.ftl` files while code still asks for it,
//! which shows on screen as `‹id›`. Ids built at run time (`country-{tag}`) are not covered.

use std::path::{Path, PathBuf};

use bf_i18n::{I18n, Locale};

fn literal_ids(dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            literal_ids(&path, out);
            continue;
        }
        if path.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for call in ["tr(\"", "tr_args(\n", "tr_args(\"", "attr(\"", "label(\""] {
            let mut rest = text.as_str();
            while let Some(i) = rest.find(call) {
                rest = &rest[i + call.len()..];
                let after = rest.trim_start_matches([' ', '\n']);
                let after = after.strip_prefix('"').unwrap_or(after);
                let id: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect();
                // A literal id is followed by its closing quote; anything else is built at run time.
                if id.contains('-') && after[id.len()..].starts_with('"') {
                    out.push((id, path.clone()));
                }
            }
        }
    }
}

#[test]
fn every_literal_message_id_exists() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut ids = Vec::new();
    literal_ids(&root.join("src"), &mut ids);
    assert!(
        ids.len() > 50,
        "found only {} ids; the scanner is broken",
        ids.len()
    );
    for locale in Locale::ALL {
        let i18n = I18n::load(&root.join("../../assets/i18n"), locale).unwrap();
        let missing: Vec<String> = ids
            .iter()
            .filter(|(id, _)| !i18n.has(id))
            .map(|(id, path)| format!("{id} ({})", path.display()))
            .collect();
        assert!(missing.is_empty(), "{locale} lacks: {missing:#?}");
    }
}

/// Every command has a name and a short label for the drawn keyboard, in every locale.
#[test]
fn every_command_has_a_name_and_a_short_label() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n");
    let registry = bf_client::keys::Registry::builtin();
    for locale in Locale::ALL {
        let i18n = I18n::load(&root, locale).unwrap();
        for def in registry.all() {
            let key = def.name_key();
            assert!(!i18n.tr(&key).starts_with('‹'), "{locale}: {key}");
            assert!(i18n.attr(&key, "short").is_some(), "{locale}: {key}.short");
        }
    }
}

/// Every state of the map has its name (`state-<KEY>`) in every locale, so no screen falls back
/// to the map's name, which is in the language of the place or in English.
#[test]
fn every_state_has_a_name() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join("assets/map/provinces.ron")).unwrap();
    let map: bf_sim::scenario::MapFile = ron::from_str(&text).unwrap();
    assert!(map.states.len() > 800, "only {} states", map.states.len());
    for locale in Locale::ALL {
        let i18n = I18n::load(&root.join("assets/i18n"), locale).unwrap();
        let missing: Vec<&str> = map
            .states
            .iter()
            .filter(|s| !i18n.has(&format!("state-{}", s.key)))
            .map(|s| s.key.as_str())
            .collect();
        assert!(missing.is_empty(), "{locale} lacks names for: {missing:?}");
    }
}

/// Every character of every text in every locale has a glyph in the game's fonts: a character
/// none of them draws shows as a box on screen (the "⋯" of an early text of the colonial war).
#[test]
fn every_character_has_a_glyph() {
    let ctx = egui::Context::default();
    bf_ui_kit::install(&ctx);
    // The fonts load on the first frame.
    let mut first = ctx.run_pass(egui::RawInput::default(), |_| {});
    // No renderer takes the font atlas here: dropped on purpose (a debug build checks it).
    first.textures_delta.clear();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n");
    let mut missing = Vec::new();
    for locale in std::fs::read_dir(&root).unwrap() {
        let locale = locale.unwrap().path();
        for file in std::fs::read_dir(&locale).unwrap() {
            let file = file.unwrap().path();
            if file.extension().is_none_or(|x| x != "ftl") {
                continue;
            }
            let text = std::fs::read_to_string(&file).unwrap();
            for (n, line) in text.lines().enumerate() {
                if line.trim_start().starts_with('#') {
                    continue;
                }
                for c in line.chars() {
                    let font = egui::FontId::proportional(14.0);
                    let has = ctx.fonts_mut(|f| f.has_glyph(&font, c));
                    if !has {
                        missing.push(format!(
                            "{}:{}: {c:?} (U+{:04X})",
                            file.display(),
                            n + 1,
                            c as u32
                        ));
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "no glyph for:\n{}", missing.join("\n"));
}

/// No text a player reads names an owner's decision by its code (O3, G8, N1, CT6…): the code
/// means nothing on screen; the text says what the rule is.
#[test]
fn no_text_names_a_decision_code() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n");
    let prefixes = ["GC", "PO", "CT", "O", "G", "N", "D", "A"];
    let mut found = Vec::new();
    for locale in std::fs::read_dir(&root).unwrap() {
        for file in std::fs::read_dir(locale.unwrap().path()).unwrap() {
            let file = file.unwrap().path();
            if file.extension().is_none_or(|x| x != "ftl") {
                continue;
            }
            let text = std::fs::read_to_string(&file).unwrap();
            for (n, line) in text.lines().enumerate() {
                if line.trim_start().starts_with('#') {
                    continue;
                }
                // A word made of a prefix and one or two digits: "O3", "G8", "CT6", "D13".
                let words = line.split(|c: char| !c.is_ascii_alphanumeric());
                for w in words {
                    let code = prefixes.iter().any(|p| {
                        w.strip_prefix(p).is_some_and(|d| {
                            (1..=2).contains(&d.len()) && d.chars().all(|c| c.is_ascii_digit())
                        })
                    });
                    if code {
                        found.push(format!("{}:{}: {w}", file.display(), n + 1));
                    }
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "decision codes in texts:\n{}",
        found.join("\n")
    );
}

/// English writes the percent sign against the number ("12.5%"), as `gov-pct` does.
#[test]
fn english_percentages_have_no_space() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n/en");
    let mut found = Vec::new();
    for file in std::fs::read_dir(&dir).unwrap() {
        let file = file.unwrap().path();
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let spaced = line.contains("} %")
                || line
                    .char_indices()
                    .any(|(i, c)| c.is_ascii_digit() && line[i + 1..].starts_with(" %"));
            if spaced {
                found.push(format!("{}:{}", file.display(), n + 1));
            }
        }
    }
    assert!(
        found.is_empty(),
        "a space before % in English:\n{}",
        found.join("\n")
    );
}

/// Portuguese writes a space between the number and the percent sign ("12,5 %"), as `gov-pct`
/// does; English writes none (`english_percentages_have_no_space`).
#[test]
fn portuguese_percentages_have_a_space() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n/pt-BR");
    let mut found = Vec::new();
    for file in std::fs::read_dir(&dir).unwrap() {
        let file = file.unwrap().path();
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let tight = line.contains("}%")
                || line
                    .char_indices()
                    .any(|(i, c)| c.is_ascii_digit() && line[i + 1..].starts_with('%'));
            if tight {
                found.push(format!("{}:{}", file.display(), n + 1));
            }
        }
    }
    assert!(
        found.is_empty(),
        "no space before % in Portuguese:\n{}",
        found.join("\n")
    );
}

/// Every good, building and method of the catalogue has its name (`good-<key>`,
/// `building-<key>`, `method-<key>`) in every locale, so no screen shows a raw key (the warhead
/// assembly of #333 came without its names).
#[test]
fn every_good_building_and_method_has_a_name() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let d = bf_sim::data::GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    let cat = &d.catalogue;
    let mut keys: Vec<String> = cat
        .goods
        .iter()
        .map(|g| format!("good-{}", g.key))
        .collect();
    for b in &cat.buildings {
        keys.push(format!("building-{}", b.key));
        keys.extend(b.methods.iter().map(|m| format!("method-{}", m.key)));
    }
    keys.sort();
    keys.dedup();
    for locale in Locale::ALL {
        let i18n = I18n::load(&root.join("assets/i18n"), locale).unwrap();
        let missing: Vec<&str> = keys
            .iter()
            .filter(|k| !i18n.has(k))
            .map(String::as_str)
            .collect();
        assert!(missing.is_empty(), "{locale} lacks names for: {missing:?}");
    }
}
