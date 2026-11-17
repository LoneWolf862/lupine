use lupine::creator::{add_art, new_game, new_game_wizard};
use lupine::story_loader::load_game;
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp() -> PathBuf {
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("lupine_v07c_{}_{}", std::process::id(), tick))
}
#[test]
fn art_generator_creates_static_text() {
    let dir = temp();
    new_game(&dir, "example", "Example").unwrap();
    let files = add_art(&dir, "sigil", false).unwrap();
    assert_eq!(files.len(), 1);
    assert!(fs::read_to_string(&files[0]).unwrap().contains("/____"));
    assert!(add_art(&dir, "sigil", false).is_err());
    assert!(load_game(&dir).is_ok());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn art_generator_dynamic_files_can_render_from_flags() {
    let dir = temp();
    new_game(&dir, "example", "Example").unwrap();
    let files = add_art(&dir, "sigil", true).unwrap();
    assert_eq!(files.len(), 2);
    let room = dir.join("rooms/start.toml");
    fs::write(
        &room,
        r#"id = "start"
title = "Starting Room"
description = "A carved symbol."
art_file = "sigil_inactive.txt"
[[art_variants]]
when = [{ flag_set = "sigil_active" }]
file = "sigil_active.txt"
[exits]
"#,
    )
    .unwrap();
    let game = load_game(&dir).unwrap();
    let mut state = lupine::state::GameState::new(&game.world);
    let room = game.world.room("start").unwrap();
    let before = lupine::render::describe_room(&game.world, &state, room);
    assert!(before.contains("INACTIVE"), "{before}");
    state.flags.insert("sigil_active".into());
    let after = lupine::render::describe_room(&game.world, &state, room);
    assert!(after.contains("ACTIVE"), "{after}");
    assert!(!after.contains("INACTIVE"), "{after}");
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn bad_art_ids_and_preexisting_files_are_protected() {
    let dir = temp();
    new_game(&dir, "example", "Example").unwrap();
    assert!(add_art(&dir, "../escape", false).is_err());
    let existing = dir.join("art/sigil_active.txt");
    fs::write(&existing, "MY OWN ART").unwrap();
    assert!(add_art(&dir, "sigil", true).is_err());
    assert!(!dir.join("art/sigil_inactive.txt").exists());
    assert_eq!(fs::read_to_string(existing).unwrap(), "MY OWN ART");
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn wizard_creates_valid_game_without_overwriting() {
    let dir = temp();
    let script = format!("{}\ncustom_story\nA Wizard's Tale\n", dir.display());
    let mut input = Cursor::new(script.into_bytes());
    let mut output = Vec::new();
    let path = new_game_wizard(&mut input, &mut output).unwrap();
    assert_eq!(path, dir);
    assert_eq!(load_game(&dir).unwrap().title, "A Wizard's Tale");
    assert!(String::from_utf8(output).unwrap().contains("Created:"));
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn wizard_eof_invalid_id_and_existing_dir_never_overwrite() {
    let dir = temp();
    let mut out = Vec::new();
    let short = format!("{}\n", dir.display());
    assert!(new_game_wizard(&mut Cursor::new(short.as_bytes()), &mut out).is_err());
    assert!(!dir.exists());
    let invalid = format!("{}\n../bad\nTitle\n", dir.display());
    assert!(new_game_wizard(&mut Cursor::new(invalid.as_bytes()), &mut Vec::new()).is_err());
    assert!(!dir.exists());
    new_game(&dir, "existing", "Existing").unwrap();
    let valid = format!("{}\nother\nOverwrite\n", dir.display());
    assert!(new_game_wizard(&mut Cursor::new(valid.as_bytes()), &mut Vec::new()).is_err());
    assert_eq!(load_game(&dir).unwrap().title, "Existing");
    fs::remove_dir_all(dir).unwrap();
}
