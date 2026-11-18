use lupine::encounters::{EncounterContent, EncounterPersistence, RandomSource};
use lupine::save::{decode_save, encode_save};
use lupine::state::GameState;
use lupine::story_loader::load_game;
use std::path::Path;

struct AlwaysZero;
impl RandomSource for AlwaysZero {
    fn below(&mut self, _limit: u32) -> u32 {
        0
    }
}

fn sample() -> (lupine::story_loader::LoadedGame, EncounterContent) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_persistent");
    let game = load_game(root.to_str().unwrap()).unwrap();
    let rooms = game
        .world
        .rooms
        .iter()
        .map(|r| r.id.clone())
        .collect::<Vec<_>>();
    let encounters = EncounterContent::load(&root, &rooms, true).unwrap();
    (game, encounters)
}

#[test]
fn sample_parses_and_modes_are_opt_in() {
    let (_game, content) = sample();
    assert_eq!(
        content.tables["woods"].persistence,
        EncounterPersistence::Room
    );
    assert_eq!(
        content.tables["lair"].persistence,
        EncounterPersistence::Unique
    );
    let baseline = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_rewards");
    let root = load_game(baseline.to_str().unwrap()).unwrap();
    let rooms = root
        .world
        .rooms
        .iter()
        .map(|r| r.id.clone())
        .collect::<Vec<_>>();
    let old = EncounterContent::load(&baseline, &rooms, true).unwrap();
    assert_eq!(
        old.tables["woods"].persistence,
        EncounterPersistence::Temporary
    );
}

#[test]
fn room_enemy_retreat_restores_health_and_identity() {
    let (game, content) = sample();
    let mut state = GameState::new(&game.world);
    let mut rng = AlwaysZero;
    state.current_room = "woods".into();
    content.enter_room("woods", &mut state, &mut rng);
    assert_eq!(state.active_enemies.len(), 1);
    state.active_enemies[0].current_hp -= 3;
    let wounded = state.active_enemies[0].clone();
    state.current_room = "camp".into();
    content.leave_room("woods", &mut state);
    assert!(state.active_enemies.is_empty());
    assert_eq!(state.room_encounters["woods"][0], wounded);
    state.current_room = "woods".into();
    content.enter_room("woods", &mut state, &mut rng);
    assert_eq!(state.active_enemies, vec![wounded]);
    assert!(!state.room_encounters.contains_key("woods"));
}

#[test]
fn unique_defeat_does_not_respawn_after_exit_or_rork_load() {
    let (game, content) = sample();
    let mut state = GameState::new(&game.world);
    state.character = Some(
        game.config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    let mut rng = AlwaysZero;
    state.current_room = "lair".into();
    content.enter_room("lair", &mut state, &mut rng);
    assert_eq!(state.active_enemies.len(), 1);
    state.active_enemies.clear(); // Simulates last enemy's removal after actual combat.
    content.record_defeat(&mut state);
    assert!(state.completed_unique_encounters.contains("lair"));
    state.current_room = "woods".into();
    content.leave_room("lair", &mut state);
    let saved = encode_save(&game.world, &state, &game.id, "Victory").unwrap();
    let mut restored = decode_save(&game.world, &game.id, &saved).unwrap();
    content.validate_restored(&restored).unwrap();
    restored.current_room = "lair".into();
    content.enter_room("lair", &mut restored, &mut rng);
    assert!(restored.active_enemies.is_empty());
}

#[test]
fn saved_retreating_boss_survives_but_duplicate_ids_are_rejected() {
    let (game, content) = sample();
    let mut state = GameState::new(&game.world);
    state.character = Some(
        game.config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    let mut rng = AlwaysZero;
    state.current_room = "lair".into();
    content.enter_room("lair", &mut state, &mut rng);
    state.active_enemies[0].current_hp = 7;
    let wounded = state.active_enemies[0].clone();
    state.current_room = "woods".into();
    content.leave_room("lair", &mut state);
    let saved = encode_save(&game.world, &state, &game.id, "Retreat").unwrap();
    let mut restored = decode_save(&game.world, &game.id, &saved).unwrap();
    content.validate_restored(&restored).unwrap();
    restored.current_room = "lair".into();
    content.enter_room("lair", &mut restored, &mut rng);
    assert_eq!(restored.active_enemies[0], wounded);
    restored
        .room_encounters
        .insert("woods".into(), vec![wounded]);
    assert!(content.validate_restored(&restored).is_err());
    assert!(encode_save(&game.world, &restored, &game.id, "Corrupt").is_err());
}

#[test]
fn temporary_mode_still_discards_enemies_on_retreat() {
    let (game, mut content) = sample();
    content.tables.get_mut("woods").unwrap().persistence = EncounterPersistence::Temporary;
    let mut state = GameState::new(&game.world);
    let mut rng = AlwaysZero;
    state.current_room = "woods".into();
    content.enter_room("woods", &mut state, &mut rng);
    let first_id = state.active_enemies[0].instance_id;
    state.current_room = "camp".into();
    content.leave_room("woods", &mut state);
    assert!(state.room_encounters.is_empty());
    state.current_room = "woods".into();
    content.enter_room("woods", &mut state, &mut rng);
    assert!(state.active_enemies[0].instance_id > first_id);
}
