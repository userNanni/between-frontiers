//! The world's schema is the one of its format version: a change to what `GameWorld` saves
//! without a new `WORLD_VERSION` fails here. After raising the version (and freezing the old
//! layout, see `bf_sim::save_format`), `BF_BLESS=1 cargo test -p bf-sim --test world_schema`
//! writes the new version's schema; an existing version's schema is never rewritten.

use std::path::PathBuf;

use bf_sim::game::GameWorld;
use bf_sim::save_format::WORLD_VERSION;
use serde_reflection::{Tracer, TracerConfig};

fn schema() -> String {
    let mut tracer = Tracer::new(TracerConfig::default());
    // Each enum is traced once per variant; the world reaches them only through their first
    // variant. A new enum in the world shows up as missing variants: add it here.
    macro_rules! trace {
        ($($t:ty),* $(,)?) => {$(
            tracer
                .trace_simple_type::<$t>()
                .unwrap_or_else(|e| panic!("tracing {}: {e}", stringify!($t)));
        )*};
    }
    trace!(
        GameWorld,
        bf_sim::econ::money::CentralBank,
        bf_sim::treaty::Currency,
        bf_sim::borders::op::DebtTerm,
        bf_sim::polity::Enactor,
        bf_sim::econ::migration::MigrationLaw,
        bf_sim::borders::op::Operation,
        bf_sim::treaty::Pledge,
        bf_sim::demography::Push,
        bf_sim::research::Stage,
        bf_sim::scenario::SubjectKind,
        bf_sim::demography::WaveEnd,
        bf_sim::foreign_capital::Handover,
        bf_sim::treaty::Side,
        bf_sim::treaty::ClauseKind,
        bf_sim::treaty::ArmsCategory,
        bf_sim::treaty::Obligation,
        bf_sim::treaty::AgreementStatus,
        bf_sim::borders::independence::Answer,
        bf_sim::borders::independence::How,
        bf_sim::econ::levers::SurplusUse,
        bf_sim::war::Stage,
    );
    let registry = tracer
        .registry()
        .unwrap_or_else(|e| panic!("add the enums to trace!: {e}"));
    let mut out = String::new();
    for (name, format) in &registry {
        out.push_str(&format!("{name}: {format:?}\n"));
    }
    out
}

#[test]
fn the_world_schema_matches_its_format_version() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/schema");
    let file = dir.join(format!("world-v{WORLD_VERSION}.txt"));
    let now = schema();
    match std::fs::read_to_string(&file) {
        Ok(saved) => {
            // A checkout on Windows may turn the file's line ends into CRLF.
            if saved.replace("\r\n", "\n") != now {
                // The schema now, to compare with the saved one.
                let seen = std::env::temp_dir().join(format!("world-v{WORLD_VERSION}-now.txt"));
                let _ = std::fs::write(&seen, &now);
                panic!(
                    "GameWorld no longer saves as world format {WORLD_VERSION} (its schema now is in \
                     {}). Raise WORLD_VERSION, freeze the old layout in save_format::compat, \
                     write the migration and run BF_BLESS=1 cargo test -p bf-sim --test \
                     world_schema (see docs/saves.pt-BR.md).",
                    seen.display()
                );
            }
        }
        Err(_) if std::env::var_os("BF_BLESS").is_some() => {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(&file, now).unwrap();
        }
        Err(e) => panic!(
            "{}: {e}; bless it with BF_BLESS=1 once the version is raised",
            file.display()
        ),
    }
}
