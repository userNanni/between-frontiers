//! Keeps `common/shots.rs`, the helper of the theme screenshot files (`shots_<theme>.rs`),
//! compiled and checked by clippy with the other tests, and its paths right without a GPU.

#[path = "common/shots.rs"]
mod shots;

#[test]
fn the_shots_helper_finds_the_game_data_and_writes_by_theme() {
    assert!(shots::root().join("data").is_dir(), "no data folder");
    let dir = shots::out_dir("helper-check");
    assert!(dir.ends_with("helper-check") && dir.is_dir());
    let _ = std::fs::remove_dir(&dir);
}
