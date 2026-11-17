use lupine::combat::{award_defeat, physical_attack};
use lupine::encounters::{EncounterContent, EnemyInstance, RandomSource};
use lupine::rpg::Progression;
use lupine::rpg_content::RpgContent;
use lupine::state::{GameState, ItemLocation};
use lupine::story_loader::load_game;
use std::path::Path;

struct FixedRng(u32);
impl RandomSource for FixedRng {
    fn below(&mut self, limit: u32) -> u32 {
        self.0.min(limit.saturating_sub(1))
    }
}
fn fixture() -> (
    lupine::story_loader::LoadedGame,
    EncounterContent,
    RpgContent,
    GameState,
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_rewards");
    let game = load_game(&root).unwrap();
    let ids: Vec<String> = game.world.rooms.iter().map(|r| r.id.clone()).collect();
    let encounter = EncounterContent::load(&root, &ids, true).unwrap();
    encounter.validate_loot(&game.world).unwrap();
    let content = RpgContent::load(&root, &game.world, true).unwrap();
    let mut state = GameState::new(&game.world);
    state.character = Some(
        game.progression
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    encounter.prepare_loot(&mut state);
    (game, encounter, content, state)
}
#[test]
fn hidden_loot_drops_and_is_persistent_state() {
    let (game, encounter, _, mut state) = fixture();
    assert_eq!(
        state.location("goblin_trophy"),
        Some(ItemLocation::Consumed)
    );
    state.current_room = "woods".into();
    let enemy = EnemyInstance {
        instance_id: 1,
        enemy_id: "goblin".into(),
        current_hp: 0,
    };
    let result = award_defeat(
        &encounter,
        &game.world,
        game.progression.as_ref(),
        &mut state,
        &enemy,
        &mut FixedRng(0),
    );
    assert!(result.contains("XP"), "{result}");
    assert!(result.contains("drops"), "{result}");
    assert_eq!(
        state.location("goblin_trophy"),
        Some(ItemLocation::Room("woods".into()))
    );
    // A dropped item uses a unique world ID; no duplicate physical copies.
    let repeated = award_defeat(
        &encounter,
        &game.world,
        None,
        &mut state,
        &enemy,
        &mut FixedRng(0),
    );
    assert!(!repeated.contains("drops"), "{repeated}");
    let encoded = lupine::save::encode_save(&game.world, &state, &game.id, "Rewards").unwrap();
    let decoded = lupine::save::decode_save(&game.world, &game.id, &encoded).unwrap();
    assert_eq!(
        decoded.location("goblin_trophy"),
        Some(ItemLocation::Room("woods".into()))
    );
    assert_eq!(decoded.character, state.character);
}
#[test]
fn defeat_removes_enemy_once_before_rewards() {
    let (game, encounter, content, mut state) = fixture();
    state
        .item_locations
        .insert("rusty_dagger".into(), ItemLocation::Inventory);
    let mut active = vec![EnemyInstance {
        instance_id: 1,
        enemy_id: "goblin".into(),
        current_hp: 1,
    }];
    let hit = physical_attack(
        &game.world,
        &content,
        &encounter,
        &mut state,
        &mut active,
        "goblin with dagger",
    );
    assert!(hit.turn_spent);
    assert!(active.is_empty());
    assert!(hit.defeated.is_some());
    let second = physical_attack(
        &game.world,
        &content,
        &encounter,
        &mut state,
        &mut active,
        "goblin with dagger",
    );
    assert!(!second.turn_spent);
    assert!(second.defeated.is_none());
}
#[test]
fn chance_zero_means_no_loot_and_levels_are_capped() {
    let (game, mut encounter, _, mut state) = fixture();
    state.current_room = "woods".into();
    encounter.enemies.get_mut("goblin").unwrap().loot[0].chance = 0;
    let enemy = EnemyInstance {
        instance_id: 1,
        enemy_id: "goblin".into(),
        current_hp: 0,
    };
    let text = award_defeat(
        &encounter,
        &game.world,
        game.progression.as_ref(),
        &mut state,
        &enemy,
        &mut FixedRng(0),
    );
    assert!(!text.contains("drops"));
    assert_eq!(
        state.location("goblin_trophy"),
        Some(ItemLocation::Consumed)
    );
    let config = Progression {
        base_requirement: 1,
        max_level: 2,
        ..Progression::default()
    };
    let character = state.character.as_mut().unwrap();
    let levels = config.award_xp(character, 1_000);
    assert_eq!(levels, 1);
    assert_eq!(character.level, 2);
}
