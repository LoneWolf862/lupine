use lupine::save::{decode_save, decode_save_with_compat, encode_save, encode_save_with_compat};
use lupine::save_files::{read_slot_with_compat, write_slot_with_compat};
use lupine::state::GameState;
use lupine::story_loader::load_game;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn example() -> lupine::story_loader::LoadedGame {
    load_game(Path::new(env!("CARGO_MANIFEST_DIR")).join("story_example")).unwrap()
}

#[test]
fn old_rork_v1_without_compat_field_remains_readable() {
    let game = example();
    let state = GameState::new(&game.world);
    let new_text = encode_save(&game.world, &state, &game.id, "Camp").unwrap();
    let old_text = new_text
        .lines()
        .filter(|line| !line.starts_with("save_compatibility_version = "))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        decode_save(&game.world, &game.id, &old_text).unwrap(),
        state
    );
    assert_eq!(
        decode_save_with_compat(&game.world, &game.id, &old_text, 1).unwrap(),
        state
    );
}

#[test]
fn mismatched_compat_version_is_rejected_with_helpful_error() {
    let game = example();
    let state = GameState::new(&game.world);
    let text = encode_save_with_compat(&game.world, &state, &game.id, "Camp", 2).unwrap();
    let error = decode_save_with_compat(&game.world, &game.id, &text, 1).unwrap_err();
    assert!(error.contains("compatibility version 2"), "{error}");
    assert!(error.contains("version 1"), "{error}");
    assert_eq!(
        decode_save_with_compat(&game.world, &game.id, &text, 2).unwrap(),
        state
    );
}

#[test]
fn invalid_compat_revision_rejected() {
    let game = example();
    let state = GameState::new(&game.world);
    assert!(encode_save_with_compat(&game.world, &state, &game.id, "Camp", 0).is_err());
    let text = encode_save(&game.world, &state, &game.id, "Camp").unwrap();
    assert!(decode_save_with_compat(&game.world, &game.id, &text, 0).is_err());
}

#[test]
fn manifest_defaults_to_one_and_v1_declares_one() {
    let game = example();
    assert_eq!(game.save_compatibility_version, 1);
}

#[test]
fn save_files_enforce_compatibility_version() {
    let game = example();
    let state = GameState::new(&game.world);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lupine_compat_{}_{}", std::process::id(), nanos));
    write_slot_with_compat(&root, &game.world, &state, &game.id, "Camp", false, 2).unwrap();
    let err = read_slot_with_compat(&root, &game.world, &game.id, "Camp", 1).unwrap_err();
    assert!(err.contains("compatibility version"), "{err}");
    assert_eq!(
        read_slot_with_compat(&root, &game.world, &game.id, "Camp", 2).unwrap(),
        state
    );
    fs::remove_dir_all(root).unwrap();
}
