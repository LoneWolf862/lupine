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
    let script = format!(
        "advanced\nA Wizard's Tale\ncustom_story\n{}\n\n\n1\n",
        dir.display()
    );
    let mut input = Cursor::new(script.into_bytes());
    let mut output = Vec::new();
    let path = new_game_wizard(&mut input, &mut output).unwrap();
    assert_eq!(path, dir);
    assert_eq!(load_game(&dir).unwrap().title, "A Wizard's Tale");
    assert!(String::from_utf8(output)
        .unwrap()
        .contains("ADVENTURE CREATED!"));
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn wizard_eof_invalid_id_and_existing_dir_never_overwrite() {
    let dir = temp();
    let mut out = Vec::new();
    let short = format!("{}\n", dir.display());
    assert!(new_game_wizard(&mut Cursor::new(short.as_bytes()), &mut out).is_err());
    assert!(!dir.exists());
    let invalid = "advanced\nTitle\n../bad\ncancel\n";
    assert!(new_game_wizard(&mut Cursor::new(invalid.as_bytes()), &mut Vec::new()).is_err());
    assert!(!dir.exists());
    new_game(&dir, "existing", "Existing").unwrap();
    let valid = format!("advanced\nOverwrite\nother\n{}\ncancel\n", dir.display());
    assert!(new_game_wizard(&mut Cursor::new(valid.as_bytes()), &mut Vec::new()).is_err());
    assert_eq!(load_game(&dir).unwrap().title, "Existing");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn previews_equal_files_and_special_text_plays() {
    let dir = temp();
    let script = format!("advanced\nThe \"Quoted\" Adventure\nquoted\n{}\n雪 & Stars\n/multi\nA \\\"quote\\\" and a path C:\\stories.\n\nSecond line: café.\n\\cancel\n/end\n1\n", dir.display());
    let mut output = Vec::new();
    new_game_wizard(&mut Cursor::new(script), &mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    for name in [
        "game.toml",
        "rooms/start.toml",
        "interactions.toml",
        "README.txt",
    ] {
        let marker = format!("Preview: {}\n", dir.join(name).display());
        let section = output.split_once(&marker).unwrap().1;
        let contents = section
            .split_once("--- exact contents ---\n")
            .unwrap()
            .1
            .split_once("--- end contents ---")
            .unwrap()
            .0;
        assert_eq!(contents.as_bytes(), fs::read(dir.join(name)).unwrap());
    }
    let game = load_game(&dir).unwrap();
    assert_eq!(game.world.rooms[0].title, "雪 & Stars");
    assert!(game.world.rooms[0]
        .description
        .contains("\n\nSecond line: café.\ncancel"));
    let mut played = Vec::new();
    let saves = dir.join("no_automatic_saves");
    lupine::runner::run_with_io_and_saves(
        game.world,
        &game.title,
        &game.id,
        &saves,
        Cursor::new("look\nquit\n"),
        &mut played,
    )
    .unwrap();
    assert!(String::from_utf8(played)
        .unwrap()
        .contains("Second line: café."));
    assert!(!saves.exists());
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_lupine"))
        .arg("validate")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn defaults_help_and_invalid_input_retry() {
    let dir = temp();
    let script = format!(
        "advanced\n?\n\nBAD ID\n?\n\n{}\n\n\nmaybe\n?\n1\n",
        dir.display()
    );
    let mut output = Vec::new();
    new_game_wizard(&mut Cursor::new(script), &mut output).unwrap();
    let game = load_game(&dir).unwrap();
    assert_eq!(game.title, "My Adventure");
    assert_eq!(game.id, "my_adventure");
    assert_eq!(game.world.rooms[0].title, "Starting Room");
    assert_eq!(
        game.world.rooms[0].description,
        "You stand at the beginning of your adventure."
    );
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Keep this ID stable"));
    assert!(output.contains("Choose 1, 2, 3 or 4"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cancellation_and_eof_at_every_step_create_nothing() {
    let dir = temp();
    let answers = [
        "advanced".to_owned(),
        "Title".to_owned(),
        "test".into(),
        dir.display().to_string(),
        "Room".into(),
        "Description".into(),
    ];
    for step in 0..=answers.len() {
        let prefix = answers[..step]
            .iter()
            .map(|s| format!("{s}\n"))
            .collect::<String>();
        for ending in ["", "cancel\n"] {
            let error = new_game_wizard(
                &mut Cursor::new(format!("{prefix}{ending}")),
                &mut Vec::new(),
            )
            .unwrap_err();
            assert!(error.contains("canceled"));
            assert!(!dir.exists());
        }
    }
    for ending in ["4\n", "cancel\n"] {
        let script = format!("advanced\nTitle\ntest\n{}\n\n\n{ending}", dir.display());
        assert!(new_game_wizard(&mut Cursor::new(script), &mut Vec::new()).is_err());
        assert!(!dir.exists());
    }
}

#[test]
fn unsafe_and_missing_parent_paths_fail_without_changes() {
    let root = temp();
    fs::create_dir(&root).unwrap();
    for path in [
        root.join("../escape"),
        root.join("missing/project"),
        root.join("bad:name"),
    ] {
        assert!(new_game(&path, "test", "Test").is_err());
    }
    fs::write(root.join("occupied"), "keep").unwrap();
    assert!(new_game(&root.join("occupied"), "test", "Test").is_err());
    assert_eq!(fs::read_to_string(root.join("occupied")).unwrap(), "keep");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn output_failure_before_confirmation_creates_nothing() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("broken output"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let dir = temp();
    let script = format!("advanced\nTitle\ntest\n{}\n\n\n1\n", dir.display());
    assert!(new_game_wizard(&mut Cursor::new(script), &mut Broken)
        .unwrap_err()
        .contains("broken output"));
    assert!(!dir.exists());
}

#[test]
fn destination_symlink_is_rejected() {
    let root = temp();
    fs::create_dir(&root).unwrap();
    let link = root.join("link");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&root, &link).unwrap();
    #[cfg(windows)]
    {
        // Junctions do not require Windows Developer Mode or symlink privileges.
        let result = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&root)
            .output()
            .unwrap();
        assert!(result.status.success());
    }
    assert!(new_game(&link.join("escape"), "test", "Test")
        .unwrap_err()
        .contains("Symlink/reparse"));
    assert!(!root.join("escape").exists());
    #[cfg(windows)]
    fs::remove_dir(&link).unwrap();
    #[cfg(unix)]
    fs::remove_file(&link).unwrap();
    fs::remove_dir(root).unwrap();
}
