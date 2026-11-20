use lupine::runner::run_with_io_and_saves;
use lupine::save_files::{list_slots, read_slot, slot_path, slot_slug};
use lupine::state::{GameState, ItemLocation};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "../examples/fixture_world.rs"]
mod fixture_world;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn temporary_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let unique_id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);

    std::env::temp_dir().join(format!(
        "lupine_rork_{}_{}_{}",
        std::process::id(),
        nanos,
        unique_id
    ))
}
fn play(root: &std::path::Path, script: &str) -> String {
    let mut output = Vec::new();
    run_with_io_and_saves(
        fixture_world::fixture_world(),
        "Fixture Adventure",
        "fixture",
        root,
        Cursor::new(script.as_bytes()),
        &mut output,
    )
    .unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn saves_and_loads_across_process_sessions() {
    let root = temporary_root();
    let first = play(
        &root,
        "take key\nnorth\neast\nuse key on panel\nsave Forest Camp\nquit\n",
    );
    assert!(first.contains("Saved: forestcamp.fixture.rork"), "{first}");
    let path = slot_path(&root, "fixture", "Forest Camp").unwrap();
    assert!(path.exists());
    let second = play(&root, "saves\nload Forest Camp\ninventory\nnorth\nquit\n");
    assert!(second.contains("Forest Camp"), "{second}");
    assert!(second.contains("Loaded: Forest Camp"), "{second}");
    assert!(second.contains("brass key"), "{second}");
    assert!(second.contains("Test Vault"), "{second}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_named_save_writes_without_questions_including_overwrite() {
    let root = temporary_root();
    play(&root, "save Initial\nquit\n");
    let result = play(&root, "take key\nsave Initial\nquit\n");
    assert!(!result.contains("yes/no"), "{result}");
    assert!(!result.contains("Save name:"), "{result}");
    assert!(result.contains("Saved:"), "{result}");
    let world = fixture_world::fixture_world();
    let state = read_slot(&root, &world, "fixture", "Initial").unwrap();
    assert_eq!(state.location("brass_key"), Some(ItemLocation::Inventory));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cannot_load_wrong_game_or_traverse_paths() {
    let root = temporary_root();
    play(&root, "save Original\nquit\n");
    assert!(read_slot(
        &root,
        &fixture_world::fixture_world(),
        "different",
        "Original"
    )
    .is_err());
    assert!(slot_slug("../other").is_err());
    assert!(slot_slug("..").is_err());
    assert!(slot_path(&root, "../outside", "Original").is_err());
    let slots = list_slots(&root, "fixture").unwrap();
    assert_eq!(slots, vec!["Original"]);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_load_preserves_live_state_and_unsaved_prompt() {
    let root = temporary_root();
    let response = play(
        &root,
        "take key\nload missing\ninventory\nquit\nno\ninventory\nquit\nyes\n",
    );
    assert!(response.contains("Load failed"), "{response}");
    assert!(response.contains("Unsaved progress"), "{response}");
    assert!(
        response.matches("You are carrying:\n  brass key").count() >= 2,
        "{response}"
    );
    assert!(!root.exists()); // no implicit writes
}

#[test]
fn collision_names_do_not_overwrite_one_another() {
    let root = temporary_root();
    play(&root, "save Forest Camp\nquit\n");
    let message = play(&root, "save ForestCamp\nyes\nquit\n");
    assert!(message.contains("collides with existing slot"), "{message}");
    assert_eq!(list_slots(&root, "fixture").unwrap(), vec!["Forest Camp"]);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshot_stays_in_memory_without_save_command() {
    let root = temporary_root();
    let response = play(&root, "take key\nquit\nyes\n");
    assert!(response.contains("Goodbye!"));
    assert!(!root.exists());
    let initial = GameState::new(&fixture_world::fixture_world());
    assert_eq!(
        initial.location("brass_key"),
        Some(ItemLocation::Room("foyer".into()))
    );
}

#[test]
fn first_save_only_prompts_for_a_name() {
    let root = temporary_root();
    let result = play(&root, "save\nnorth\nquit\n");
    assert!(!result.contains("manual save"), "{result}");
    assert!(!result.contains("game command"), "{result}");
    assert!(!result.contains("yes/no"), "{result}");
    assert!(result.contains("Save name:\n"), "{result}");
    assert!(
        !result.contains("Test Corridor"),
        "naming must not move the player: {result}"
    );
    assert!(result.contains("Saved: north.fixture.rork"), "{result}");
    assert!(!result.contains("Unsaved progress"));
    assert_eq!(
        read_slot(&root, &fixture_world::fixture_world(), "fixture", "north")
            .unwrap()
            .current_room,
        "foyer"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn name_cancel_eof_and_explicit_command_names_remain_manual() {
    for ending in ["cancel\nquit\n", "back\nquit\n", "\nquit\n", ""] {
        let root = temporary_root();
        play(&root, &format!("save\n{ending}"));
        assert!(!root.exists());
    }
    let root = temporary_root();
    let result = play(&root, "save north\nquit\n");
    assert!(result.contains("Saved: north.fixture.rork"));
    assert!(!result.contains("looks like a game command"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn confirmation_questions_are_complete_lines_and_declines_preserve_state() {
    let root = temporary_root();
    play(&root, "save camp\nquit\n");
    let original = fs::read(slot_path(&root, "fixture", "camp").unwrap()).unwrap();
    let result = play(
        &root,
        "load camp\ntake key\nsave\nno\ncancel\nload camp\nno\nexit\nmaybe\nno\ninventory\nexit\nyes\n",
    );
    for text in [
        "Overwrite current save 'camp'? (yes/no):\n",
        "Discard unsaved progress and load? (yes/no):\n",
        "Unsaved progress. Quit without saving? (yes/no):\n",
    ] {
        assert!(result.contains(text), "{result}");
    }
    assert!(result.contains("Please answer yes or no."));
    assert!(result.contains("You are carrying:\n  brass key"));
    assert_eq!(
        fs::read(slot_path(&root, "fixture", "camp").unwrap()).unwrap(),
        original
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn current_save_overwrite_and_save_as_follow_successful_writes() {
    let root = temporary_root();
    let result = play(&root, "save Forest Camp\ntake key\nsave\nyes\ndrop key\nsave\nno\nNew Camp\ntake key\nsave\nyes\nquit\n");
    assert_eq!(
        result
            .matches("Overwrite current save 'Forest Camp'?")
            .count(),
        2,
        "{result}"
    );
    assert_eq!(
        result.matches("Overwrite current save 'New Camp'?").count(),
        1,
        "{result}"
    );
    assert_eq!(result.matches("Save name:").count(), 1, "{result}");
    assert!(
        !result.contains("Save exists. Overwrite?"),
        "no duplicate confirmations: {result}"
    );
    for name in ["Forest Camp", "New Camp"] {
        assert_eq!(
            read_slot(&root, &fixture_world::fixture_world(), "fixture", name)
                .unwrap()
                .location("brass_key"),
            Some(ItemLocation::Inventory)
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn load_selects_current_slot_and_failures_or_declines_do_not_replace_it() {
    let root = temporary_root();
    play(&root, "save First\ntake key\nsave Second\nquit\n");
    let second = fs::read(slot_path(&root, "fixture", "Second").unwrap()).unwrap();
    let result = play(
        &root,
        "load First\ntake key\nload Second\nno\nload missing\nsave ../invalid\nsave\nyes\nquit\n",
    );
    assert!(
        result.contains("Overwrite current save 'First'?"),
        "{result}"
    );
    assert_eq!(
        read_slot(&root, &fixture_world::fixture_world(), "fixture", "First")
            .unwrap()
            .location("brass_key"),
        Some(ItemLocation::Inventory)
    );
    assert_eq!(
        fs::read(slot_path(&root, "fixture", "Second").unwrap()).unwrap(),
        second
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn canceled_or_eof_save_preserves_current_slot_and_fresh_sessions_have_none() {
    let root = temporary_root();
    play(&root, "save camp\nquit\n");
    let original = fs::read(slot_path(&root, "fixture", "camp").unwrap()).unwrap();
    for ending in ["", "no\n", "no\ncancel\nquit\nyes\n", "no\n\nquit\nyes\n"] {
        play(&root, &format!("load camp\ntake key\nsave\n{ending}"));
        assert_eq!(
            fs::read(slot_path(&root, "fixture", "camp").unwrap()).unwrap(),
            original
        );
    }
    let result = play(&root, "save\ncancel\nquit\n");
    assert!(result.contains("Save name:"));
    assert!(!result.contains("Overwrite current save"));
    // Naming an existing slot through the prompt still asks before replacing it.
    let result = play(&root, "take key\nsave\ncamp\nno\nquit\nyes\n");
    assert!(result.contains("Save exists. Overwrite?"));
    assert_eq!(
        fs::read(slot_path(&root, "fixture", "camp").unwrap()).unwrap(),
        original
    );
    fs::remove_dir_all(root).unwrap();
}
