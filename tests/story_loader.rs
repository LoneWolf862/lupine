use lupine::engine::{execute_command, ActionOutcome};
use lupine::parser::parse_command;
use lupine::state::GameState;
use lupine::story_loader::load_game;

fn example() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("story_example")
}

#[test]
fn plays_external_story_without_engine_changes() {
    let game = load_game(example()).expect("example story must load");
    assert_eq!(game.id, "example_adventure");
    let mut state = GameState::new(&game.world);
    let action = |text: &str, state: &mut GameState| {
        execute_command(&game.world, state, parse_command(text).unwrap())
    };
    assert!(
        matches!(action("north", &mut state), ActionOutcome::Message(message) if message.contains("locked"))
    );
    assert!(
        matches!(action("take key", &mut state), ActionOutcome::Message(message) if message.contains("take"))
    );
    assert!(
        matches!(action("use key on door", &mut state), ActionOutcome::Message(message) if message.contains("unlocks"))
    );
    assert!(
        matches!(action("look", &mut state), ActionOutcome::Message(message) if message.contains("stands open"))
    );
    assert!(
        matches!(action("north", &mut state), ActionOutcome::Message(message) if message.contains("Sample Workshop"))
    );
    assert_eq!(state.current_room, "workshop");
}

#[test]
fn missing_story_directory_has_contextual_error() {
    let mut path = example();
    path.push("nonexistent_subdirectory");
    let error = load_game(&path).unwrap_err();
    assert!(error.contains("game.toml"), "{error}");
}

#[test]
fn detects_bad_direction_with_file_path() {
    let root = std::env::temp_dir().join(format!(
        "lupine-loader-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir_all(root.join("rooms")).unwrap();
    std::fs::create_dir_all(root.join("items")).unwrap();
    std::fs::write(
        root.join("game.toml"),
        "format_version = 1\nid = 'test'\ntitle = 'Test'\nstarting_room = 'a'\n",
    )
    .unwrap();
    std::fs::write(
        root.join("rooms/a.toml"),
        "id='a'\ntitle='A'\ndescription='A'\n[exits]\nsideways='a'\n",
    )
    .unwrap();
    let error = load_game(&root).unwrap_err();
    assert!(
        error.contains("a.toml") && error.contains("sideways"),
        "{error}"
    );
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn rejects_unsupported_format_version() {
    let root = std::env::temp_dir().join(format!("lupine-version-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("game.toml"),
        "format_version = 999\nid = 'test'\ntitle = 'Test'\nstarting_room = 'a'\n",
    )
    .unwrap();
    let error = load_game(&root).unwrap_err();
    assert!(error.contains("format_version"), "{error}");
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn rejects_malformed_toml_with_source_file() {
    let root = std::env::temp_dir().join(format!("lupine-malformed-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("game.toml"), "title = [not valid toml").unwrap();
    let error = load_game(&root).unwrap_err();
    assert!(error.contains("game.toml"), "{error}");
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn rejects_missing_room_reference() {
    let root = std::env::temp_dir().join(format!("lupine-broken-ref-test-{}", std::process::id()));
    std::fs::create_dir_all(root.join("rooms")).unwrap();
    std::fs::create_dir_all(root.join("items")).unwrap();
    std::fs::write(
        root.join("game.toml"),
        "format_version=1\nid='test'\ntitle='Test'\nstarting_room='a'\n",
    )
    .unwrap();
    std::fs::write(
        root.join("rooms/a.toml"),
        "id='a'\ntitle='A'\ndescription='A'\n[exits]\nnorth='missing'\n",
    )
    .unwrap();
    let error = load_game(&root).unwrap_err();
    assert!(
        error.contains("missing") && error.contains("validation"),
        "{error}"
    );
    std::fs::remove_dir_all(&root).unwrap();
}
