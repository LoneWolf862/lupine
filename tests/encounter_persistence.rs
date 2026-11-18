//! C4: encounter state is part of explicit RORK snapshots, not autosaved.
use lupine::encounters::{EncounterContent, EnemyInstance};
use lupine::save::{decode_save, encode_save};
use lupine::state::GameState;
use lupine::story_loader::load_game;
use std::path::Path;

fn fixture() -> (
    lupine::story_loader::LoadedGame,
    EncounterContent,
    GameState,
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_rewards");
    let game = load_game(&root).unwrap();
    let ids = game
        .world
        .rooms
        .iter()
        .map(|room| room.id.clone())
        .collect::<Vec<_>>();
    let encounters = EncounterContent::load(&root, &ids, true).unwrap();
    let mut state = GameState::new(&game.world);
    state.character = Some(
        game.config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    state.current_room = "woods".into();
    (game, encounters, state)
}

#[test]
fn wounded_group_roundtrips_with_distinct_ids_and_next_id() {
    let (game, _, mut state) = fixture();
    state.next_enemy_id = 14;
    state.active_enemies = vec![
        EnemyInstance {
            instance_id: 11,
            enemy_id: "goblin".into(),
            current_hp: 3,
        },
        EnemyInstance {
            instance_id: 13,
            enemy_id: "wolf".into(),
            current_hp: 5,
        },
    ];
    let data = encode_save(&game.world, &state, &game.id, "Mid Fight").unwrap();
    let loaded = decode_save(&game.world, &game.id, &data).unwrap();
    assert_eq!(loaded.active_enemies, state.active_enemies);
    assert_eq!(loaded.next_enemy_id, 14);
}

#[test]
fn old_save_without_encounter_fields_remains_readable() {
    let (game, _, state) = fixture();
    let data = encode_save(&game.world, &state, &game.id, "Old").unwrap();
    let filtered = data
        .lines()
        .filter(|line| !line.starts_with("next_enemy_id = "))
        .collect::<Vec<_>>()
        .join("\n");
    let loaded = decode_save(&game.world, &game.id, &filtered).unwrap();
    assert!(loaded.active_enemies.is_empty());
    assert_eq!(loaded.next_enemy_id, 1);
}

#[test]
fn duplicate_dead_or_future_instance_ids_are_rejected() {
    let (game, _, mut state) = fixture();
    state.next_enemy_id = 4;
    state.active_enemies = vec![EnemyInstance {
        instance_id: 2,
        enemy_id: "goblin".into(),
        current_hp: 5,
    }];
    let valid = encode_save(&game.world, &state, &game.id, "Inspect").unwrap();
    let future = valid.replace("instance_id = 2", "instance_id = 4");
    assert!(decode_save(&game.world, &game.id, &future).is_err());
    let dead = valid.replace("current_hp = 5", "current_hp = 0");
    assert!(decode_save(&game.world, &game.id, &dead).is_err());
    let duplicate = valid.replace("next_enemy_id = 4", "next_enemy_id = 3");
    // existing instance 2 is still valid under 3, so this is valid.
    assert!(decode_save(&game.world, &game.id, &duplicate).is_ok());
}

#[test]
fn defeated_enemy_is_not_resurrected_by_save_roundtrip() {
    let (game, _, mut state) = fixture();
    state.next_enemy_id = 3;
    state.active_enemies = vec![EnemyInstance {
        instance_id: 2,
        enemy_id: "goblin".into(),
        current_hp: 1,
    }];
    state.active_enemies.clear();
    let data = encode_save(&game.world, &state, &game.id, "Defeated").unwrap();
    let loaded = decode_save(&game.world, &game.id, &data).unwrap();
    assert!(loaded.active_enemies.is_empty());
    assert_eq!(loaded.next_enemy_id, 3);
}
