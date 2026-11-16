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
