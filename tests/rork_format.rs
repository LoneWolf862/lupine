use lupine::engine::execute_command;
use lupine::parser::parse_command;
use lupine::save::{decode_save, encode_save};
use lupine::state::{GameState, ItemLocation};
use lupine::story_loader::load_game;

fn example() -> lupine::story_loader::LoadedGame {
    load_game(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("story_example")).unwrap()
}

#[test]
fn roundtrip_restores_inventory_flags_and_current_room() {
    let game = example();
    let mut state = GameState::new(&game.world);
    for cmd in ["take key", "use key on door", "north"] {
        execute_command(&game.world, &mut state, parse_command(cmd).unwrap());
    }
    let text = encode_save(&game.world, &state, &game.id, "Forest Camp").unwrap();
    assert!(text.contains("save_format_version = 3"));
    assert!(text.contains("save_name = \"Forest Camp\""));
    let restored = decode_save(&game.world, &game.id, &text).unwrap();
    assert_eq!(state, restored);
    assert_eq!(restored.current_room, "workshop");
    assert_eq!(
        restored.location("brass_key"),
        Some(ItemLocation::Inventory)
    );
    assert!(restored.flags.contains("door_open"));
}

#[test]
fn output_is_deterministic_and_human_readable() {
    let game = example();
    let state = GameState::new(&game.world);
    let a = encode_save(&game.world, &state, &game.id, "My Save").unwrap();
    let b = encode_save(&game.world, &state, &game.id, "My Save").unwrap();
    assert_eq!(a, b);
    assert!(a.contains("[[objects.placements.brass_key]]"));
}

#[test]
fn rejects_wrong_game() {
    let game = example();
    let text = encode_save(&game.world, &GameState::new(&game.world), &game.id, "Camp").unwrap();
    let err = decode_save(&game.world, "another_game", &text).unwrap_err();
    assert!(err.contains("another_game"), "{err}");
}

#[test]
fn rejects_bad_schema_and_malformed_toml() {
    let game = example();
    let text = encode_save(&game.world, &GameState::new(&game.world), &game.id, "Camp").unwrap();
    let err = decode_save(
        &game.world,
        &game.id,
        &text.replace("save_format_version = 3", "save_format_version = 99"),
    )
    .unwrap_err();
    assert!(err.contains("save_format_version"), "{err}");
    assert!(decode_save(&game.world, &game.id, "not valid TOML! [").is_err());
}

#[test]
fn rejects_missing_or_unknown_items_and_rooms() {
    let game = example();
    let mut state = GameState::new(&game.world);
    state.objects.placements.remove("brass_key");
    let err = encode_save(&game.world, &state, &game.id, "Camp").unwrap_err();
    assert!(err.contains("missing location"), "{err}");
    let mut state = GameState::new(&game.world);
    state.current_room = "nonexistent".into();
    assert!(encode_save(&game.world, &state, &game.id, "Camp").is_err());
    let state = GameState::new(&game.world);
    let text = encode_save(&game.world, &state, &game.id, "Camp").unwrap();
    let tampered = text.replace("current_room = \"foyer\"", "current_room = \"nonexistent\"");
    assert!(decode_save(&game.world, &game.id, &tampered)
        .unwrap_err()
        .contains("unknown current room"));
}

#[test]
fn invalid_load_does_not_change_existing_state() {
    let game = example();
    let mut active = GameState::new(&game.world);
    execute_command(&game.world, &mut active, parse_command("take key").unwrap());
    let before = active.clone();
    let err = decode_save(&game.world, &game.id, "broken !!").unwrap_err();
    assert!(err.contains("Invalid RORK"));
    assert_eq!(active, before);
}

#[test]
fn rejects_duplicate_flags_and_invalid_location_shape() {
    let game = example();
    let mut state = GameState::new(&game.world);
    state.flags.insert("door_open".into());
    let text = encode_save(&game.world, &state, &game.id, "Camp").unwrap();
    let tampered = text.replace(
        "flags = [\"door_open\"]",
        "flags = [\"door_open\", \"door_open\"]",
    );
    assert!(decode_save(&game.world, &game.id, &tampered)
        .unwrap_err()
        .contains("duplicate"));
    let bad_location = text.replace("kind = \"room\"", "kind = \"inventory\"");
    assert!(decode_save(&game.world, &game.id, &bad_location)
        .unwrap_err()
        .contains("Invalid RORK"));
}
