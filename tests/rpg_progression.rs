use lupine::rpg::Progression;
use lupine::runner::run_with_io_and_saves_progression;
use lupine::save::{decode_save, encode_save};
use lupine::state::GameState;
use lupine::story_loader::load_game;
use std::io::Cursor;

#[test]
fn progression_scales_damage_and_refills_on_level_up() {
    let config = Progression::default();
    let mut p = config.create_character().unwrap();
    assert_eq!((p.max_hp, p.max_mp, p.ap), (100, 30, 100));
    p.hp = 17;
    p.mp = 0;
    assert_eq!(config.award_xp(&mut p, 99), 0);
    assert_eq!((p.hp, p.mp), (17, 0));
    assert_eq!(config.award_xp(&mut p, 1), 1);
    assert_eq!(
        (p.level, p.max_hp, p.max_mp, p.ap, p.hp, p.mp),
        (2, 110, 35, 110, 110, 35)
    );
    assert_eq!(p.weapon_damage(5), 6);
    assert_eq!(p.weapon_damage(10), 11);
}

#[test]
fn level_cap_and_invalid_config_are_handled() {
    let mut config = Progression {
        max_level: 2,
        ..Progression::default()
    };
    let mut p = config.create_character().unwrap();
    assert_eq!(config.award_xp(&mut p, 100_000), 1);
    assert_eq!(p.level, 2);
    assert_eq!(config.next_requirement(2), None);
    config.starting_level = 3;
    assert!(config.validate().is_err());
}

#[test]
fn rork_character_roundtrip_and_legacy_no_character() {
    let game = load_game("story_example").unwrap();
    let mut state = GameState::new(&game.world);
    let old = encode_save(&game.world, &state, &game.id, "Legacy").unwrap();
    assert!(decode_save(&game.world, &game.id, &old)
        .unwrap()
        .character
        .is_none());
    state.character = Some(Progression::default().create_character().unwrap());
    let serialized = encode_save(&game.world, &state, &game.id, "RPG").unwrap();
    let loaded = decode_save(&game.world, &game.id, &serialized).unwrap();
    assert_eq!(loaded, state);
    let tampered = serialized.replace("\nhp = 100\n", "\nhp = 10001\n");
    assert!(decode_save(&game.world, &game.id, &tampered).is_err());
}

#[test]
fn runner_shows_stats_for_opt_in_only_and_never_autosaves() {
    let game = load_game("story_example").unwrap();
    let root = std::env::temp_dir().join(format!("lupine_rpg_no_write_{}", std::process::id()));
    let mut output = Vec::new();
    run_with_io_and_saves_progression(
        game.world,
        "Test",
        "example_adventure",
        &root,
        1,
        Some(Progression::default()),
        Cursor::new(b"stats\nquit\n"),
        &mut output,
    )
    .unwrap();
    assert!(String::from_utf8(output)
        .unwrap()
        .contains("Level 1 | XP 0 | HP 100/100 | MP 30/30 | AP 1.00"));
    assert!(!root.exists());
}

#[test]
fn optional_config_file_is_loaded_and_validated() {
    let root = std::env::temp_dir().join(format!(
        "lupine_rpg_config_{}_{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir_all(&root).unwrap();
    // Validate malformed RPG metadata without duplicating the rest of a game.
    std::fs::write(
        root.join("game.toml"),
        "format_version = 1\nid = \"test\"\ntitle = \"Test\"\nstarting_room = \"start\"\n",
    )
    .unwrap();
    std::fs::write(root.join("character.toml"), "starting_level = 0\n").unwrap();
    assert!(load_game(&root).unwrap_err().contains("starting_level"));
    std::fs::remove_dir_all(&root).unwrap();
}
