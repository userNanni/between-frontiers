//! Localisation for Between Frontiers.
//!
//! The simulation emits message ids and arguments; only the client formats text. Messages live in
//! Fluent files under `assets/i18n/<locale>/*.ftl`, and every id must exist in every locale (the
//! `parity` test enforces it).
//!
//! Numbers are formatted here, not by Fluent: Fluent's default number formatting has no digit
//! grouping, and a Brazilian reader expects `1.234.567,8`.

mod number;

use std::fmt;
use std::path::Path;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::{LanguageIdentifier, langid};

pub use number::NumberFormat;

/// Languages shipped with the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Locale {
    #[default]
    PtBr,
    En,
}

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::PtBr, Locale::En];

    /// Directory name under `assets/i18n/`, also the BCP 47 tag.
    pub const fn code(self) -> &'static str {
        match self {
            Locale::PtBr => "pt-BR",
            Locale::En => "en",
        }
    }

    /// The language's own name, shown in the language picker regardless of the current locale.
    pub const fn endonym(self) -> &'static str {
        match self {
            Locale::PtBr => "Português (Brasil)",
            Locale::En => "English",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|l| l.code().eq_ignore_ascii_case(code))
    }

    fn langid(self) -> LanguageIdentifier {
        match self {
            Locale::PtBr => langid!("pt-BR"),
            Locale::En => langid!("en"),
        }
    }

    pub const fn number_format(self) -> NumberFormat {
        match self {
            Locale::PtBr => NumberFormat {
                decimal: ',',
                group: '.',
            },
            Locale::En => NumberFormat {
                decimal: '.',
                group: ',',
            },
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(std::path::PathBuf, std::io::Error),
    Parse(String, Vec<String>),
    Duplicate(String, Vec<String>),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(path, err) => write!(f, "cannot read {}: {err}", path.display()),
            LoadError::Parse(name, errors) => write!(f, "{name}: {}", errors.join("; ")),
            LoadError::Duplicate(name, errors) => {
                write!(f, "{name}: duplicate messages: {}", errors.join("; "))
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// An argument passed to a message.
pub enum Arg<'a> {
    Str(&'a str),
    String(String),
    /// A number used for plural selection. Format it yourself and pass a string too when the
    /// number is also displayed.
    Num(f64),
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(value: &'a str) -> Self {
        Arg::Str(value)
    }
}

impl From<String> for Arg<'_> {
    fn from(value: String) -> Self {
        Arg::String(value)
    }
}

impl From<f64> for Arg<'_> {
    fn from(value: f64) -> Self {
        Arg::Num(value)
    }
}

impl From<i64> for Arg<'_> {
    fn from(value: i64) -> Self {
        Arg::Num(value as f64)
    }
}

/// Messages of one locale.
pub struct I18n {
    locale: Locale,
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    /// Loads every `.ftl` file of `locale` under `root` (the `assets/i18n` directory), in file
    /// name order.
    pub fn load(root: &Path, locale: Locale) -> Result<Self, LoadError> {
        let dir = root.join(locale.code());
        let mut files: Vec<_> = std::fs::read_dir(&dir)
            .map_err(|e| LoadError::Io(dir.clone(), e))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "ftl"))
            .collect();
        files.sort();
        let mut sources = Vec::with_capacity(files.len());
        for path in files {
            let text =
                std::fs::read_to_string(&path).map_err(|e| LoadError::Io(path.clone(), e))?;
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            sources.push((name, text));
        }
        Self::from_sources(locale, sources)
    }

    /// Builds a bundle from `(file name, Fluent source)` pairs.
    pub fn from_sources(
        locale: Locale,
        sources: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Self, LoadError> {
        let mut bundle = FluentBundle::new_concurrent(vec![locale.langid()]);
        // Bidi isolation marks render as boxes in the game font and are useless for pt-BR and en.
        bundle.set_use_isolating(false);
        for (name, text) in sources {
            let resource = FluentResource::try_new(text).map_err(|(_, errors)| {
                LoadError::Parse(name.clone(), errors.iter().map(|e| e.to_string()).collect())
            })?;
            bundle.add_resource(resource).map_err(|errors| {
                LoadError::Duplicate(name.clone(), errors.iter().map(|e| e.to_string()).collect())
            })?;
        }
        Ok(Self { locale, bundle })
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn has(&self, id: &str) -> bool {
        self.bundle.has_message(id)
    }

    /// The message `id`. A missing message shows as `‹id›`, so it is visible on screen.
    pub fn tr(&self, id: &str) -> String {
        self.format(id, None, None)
    }

    pub fn tr_args<'a>(&self, id: &str, args: &[(&str, Arg<'a>)]) -> String {
        self.format(id, None, Some(args))
    }

    /// The attribute `attr` of message `id`, if both exist.
    pub fn attr(&self, id: &str, attr: &str) -> Option<String> {
        let message = self.bundle.get_message(id)?;
        let pattern = message.get_attribute(attr)?.value();
        let mut errors = vec![];
        Some(
            self.bundle
                .format_pattern(pattern, None, &mut errors)
                .into_owned(),
        )
    }

    fn format(&self, id: &str, attr: Option<&str>, args: Option<&[(&str, Arg<'_>)]>) -> String {
        let Some(message) = self.bundle.get_message(id) else {
            return format!("‹{id}›");
        };
        let pattern = match attr {
            Some(a) => message.get_attribute(a).map(|a| a.value()),
            None => message.value(),
        };
        let Some(pattern) = pattern else {
            return format!("‹{id}›");
        };
        let fluent_args = args.map(|args| {
            let mut out = FluentArgs::new();
            for (name, value) in args {
                let value = match value {
                    Arg::Str(s) => FluentValue::from(*s),
                    Arg::String(s) => FluentValue::from(s.clone()),
                    Arg::Num(n) => FluentValue::from(*n),
                };
                out.set(*name, value);
            }
            out
        });
        let mut errors = vec![];
        self.bundle
            .format_pattern(pattern, fluent_args.as_ref(), &mut errors)
            .into_owned()
    }

    /// `1234567.0` → `1.234.567` (pt-BR) or `1,234,567` (en).
    pub fn int(&self, value: i64) -> String {
        self.locale.number_format().int(value)
    }

    pub fn decimal(&self, value: f64, digits: usize) -> String {
        self.locale.number_format().decimal(value, digits)
    }

    /// Large numbers abbreviated for tight spaces: `45,1 mi`, `1,9 bi`, `45.1M`.
    pub fn compact_short(&self, value: f64) -> String {
        self.compact_with(
            value,
            [
                "number-short-thousands",
                "number-short-millions",
                "number-short-billions",
            ],
        )
    }

    /// Large numbers in words: `45,1 milhões`, `1,9 bilhão`, `45.1 million`.
    pub fn compact(&self, value: f64) -> String {
        self.compact_with(
            value,
            ["number-thousands", "number-millions", "number-billions"],
        )
    }

    fn compact_with(&self, value: f64, ids: [&str; 3]) -> String {
        let abs = value.abs();
        let (scaled, id) = if abs >= 1e9 {
            (value / 1e9, ids[2])
        } else if abs >= 1e6 {
            (value / 1e6, ids[1])
        } else if abs >= 1e4 {
            (value / 1e3, ids[0])
        } else {
            return self.int(value.round() as i64);
        };
        let digits = if scaled.abs() >= 100.0 { 0 } else { 1 };
        let rounded = round_to(scaled, digits);
        let text = self.decimal(rounded, digits);
        self.tr_args(
            id,
            &[("value", Arg::String(text)), ("count", Arg::Num(rounded))],
        )
    }
}

fn round_to(value: f64, digits: usize) -> f64 {
    let p = 10f64.powi(digits as i32);
    (value * p).round() / p
}

/// Message ids and their attribute names in a Fluent source, for parity checks.
pub fn message_ids(source: &str) -> Vec<(String, Vec<String>)> {
    let Ok(resource) = FluentResource::try_new(source.to_owned()) else {
        return vec![];
    };
    resource
        .entries()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(m) => Some((
                m.id.name.to_owned(),
                m.attributes.iter().map(|a| a.id.name.to_owned()).collect(),
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle(locale: Locale, src: &str) -> I18n {
        I18n::from_sources(locale, [("t.ftl".to_owned(), src.to_owned())]).unwrap()
    }

    const PT: &str = "number-thousands = { $value } mil\n\
number-millions = { $count ->\n  [one] { $value } milhão\n *[other] { $value } milhões\n}\n\
number-billions = { $count ->\n  [one] { $value } bilhão\n *[other] { $value } bilhões\n}\n\
hello = Olá, { $name }!\n    .tip = dica\n";

    #[test]
    fn formats_messages_and_attributes() {
        let i = bundle(Locale::PtBr, PT);
        assert_eq!(i.tr_args("hello", &[("name", "Rio".into())]), "Olá, Rio!");
        assert_eq!(i.attr("hello", "tip").as_deref(), Some("dica"));
        assert_eq!(i.tr("missing"), "‹missing›");
    }

    #[test]
    fn compact_numbers_follow_portuguese_plural() {
        let i = bundle(Locale::PtBr, PT);
        assert_eq!(i.compact(45_123_000.0), "45,1 milhões");
        assert_eq!(i.compact(1_500_000.0), "1,5 milhão");
        assert_eq!(i.compact(1_912_000_000.0), "1,9 bilhão");
        assert_eq!(i.compact(250_400.0), "250 mil");
        assert_eq!(i.compact(9_876.0), "9.876");
    }
}
