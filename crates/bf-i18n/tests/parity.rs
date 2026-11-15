//! Every message and attribute must exist in every shipped locale, and every file must parse.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bf_i18n::{I18n, Locale, message_ids};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n")
}

fn ids(locale: Locale) -> BTreeMap<String, Vec<String>> {
    let dir = root().join(locale.code());
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "ftl") {
            let text = std::fs::read_to_string(&path).unwrap();
            for (id, mut attrs) in message_ids(&text) {
                attrs.sort();
                out.insert(id, attrs);
            }
        }
    }
    out
}

#[test]
fn all_locales_load() {
    for locale in Locale::ALL {
        I18n::load(&root(), locale).unwrap_or_else(|e| panic!("{locale}: {e}"));
    }
}

#[test]
fn all_locales_have_the_same_messages() {
    let reference = ids(Locale::PtBr);
    assert!(!reference.is_empty());
    for locale in Locale::ALL {
        let other = ids(locale);
        let missing: Vec<_> = reference
            .keys()
            .filter(|k| !other.contains_key(*k))
            .collect();
        let extra: Vec<_> = other
            .keys()
            .filter(|k| !reference.contains_key(*k))
            .collect();
        assert!(missing.is_empty(), "{locale} lacks {missing:?}");
        assert!(
            extra.is_empty(),
            "{locale} has messages missing in pt-BR: {extra:?}"
        );
        for (id, attrs) in &reference {
            if let Some(o) = other.get(id) {
                // `.article` may be empty in one language and absent in none; attributes must match.
                assert_eq!(attrs, o, "{locale}: attributes of {id} differ");
            }
        }
    }
}
