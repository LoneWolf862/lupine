use lupine::creator::{add_item, add_room, new_game};
use lupine::engine::{execute_command, ActionOutcome};
use lupine::parser::parse_command;
use lupine::render::describe_room;
use lupine::save::{decode_save, encode_save};
use lupine::state::GameState;
use lupine::story_loader::load_game;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn temp() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("lupine_v07b_{}_{}", std::process::id(), nanos))
}
fn fixture() -> lupine::story_loader::LoadedGame {
    load_game(Path::new(env!("CARGO_MANIFEST_DIR")).join("story_example")).unwrap()
}

#[test]
fn flag_changes_art_on_look_and_restored_save() {
    let game = fixture();
    let mut state = GameState::new(&game.world);
    let foyer = game.world.room("foyer").unwrap();
    let before = describe_room(&game.world, &state, foyer);
    assert!(before.contains("LOCKED"), "{before}");
    for cmd in ["take key", "use key on door"] {
        match execute_command(&game.world, &mut state, parse_command(cmd).unwrap()) {
            ActionOutcome::Message(_) => {}
            _ => panic!("unexpected quit"),
        }
    }
    let after = describe_room(&game.world, &state, foyer);
    assert!(after.contains("OPEN"), "{after}");
    assert!(!after.contains("LOCKED"), "{after}");
    let save = encode_save(&game.world, &state, &game.id, "Art Test").unwrap();
    let restored = decode_save(&game.world, &game.id, &save).unwrap();
    assert!(describe_room(&game.world, &restored, foyer).contains("OPEN"));
}

#[test]
fn generated_templates_are_valid_and_do_not_overwrite() {
    let dir = temp();
    new_game(&dir, "art_test", "Art Test").unwrap();
    assert!(dir.join("art").is_dir());
    let room = add_room(&dir, "chamber").unwrap();
    let item = add_item(&dir, "lantern").unwrap();
    assert!(room.exists() && item.exists());
    assert!(add_room(&dir, "chamber").is_err());
    assert!(add_item(&dir, "lantern").is_err());
    assert!(add_room(&dir, "../out").is_err());
    assert!(load_game(&dir).is_ok());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_art_path_escape_and_bad_art_files() {
    let dir = temp();
    new_game(&dir, "art_test", "Art Test").unwrap();
    let room = dir.join("rooms/start.toml");
    let original = fs::read_to_string(&room).unwrap();
    fs::write(&room, format!("art_file = \"../outside.txt\"\n{original}")).unwrap();
    assert!(load_game(&dir).unwrap_err().contains("relative"));
    fs::write(&room, format!("art_file = \"bad.txt\"\n{original}")).unwrap();
    fs::write(dir.join("art/bad.txt"), "hello\u{001b}[31m").unwrap();
    assert!(load_game(&dir).unwrap_err().contains("control characters"));
    fs::write(dir.join("art/bad.txt"), "x".repeat(65537)).unwrap();
    assert!(load_game(&dir).unwrap_err().contains("64 KiB"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn inline_art_and_variant_conditions_work() {
    let dir = temp();
    new_game(&dir, "art_test", "Art Test").unwrap();
    let room = dir.join("rooms/start.toml");
    fs::write(
        &room,
        r#"id = "start"
title = "Start"
description = "Hello"
art = '''
 /\
/__\
'''
[[art_variants]]
when = [{ flag_set = "powered" }]
text = "POWERED"

[exits]
"#,
    )
    .unwrap();
    let game = load_game(&dir).unwrap();
    let mut state = GameState::new(&game.world);
    let room = game.world.room("start").unwrap();
    assert!(describe_room(&game.world, &state, room).contains("/__\\"));
    state.flags.insert("powered".into());
    assert!(describe_room(&game.world, &state, room).contains("POWERED"));
    fs::remove_dir_all(dir).unwrap();
}
