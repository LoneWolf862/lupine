use lupine::render::describe_room;
use lupine::state::{GameState, ItemLocation};
use lupine::story_loader::load_game;

#[test]
fn room_object_placeholder_updates_after_items_move() {
    let mut game = load_game("samples/rpg_spellbook").unwrap();
    let camp_id = game.world.starting_room.clone();
    let room = game
        .world
        .rooms
        .iter_mut()
        .find(|r| r.id == camp_id)
        .unwrap();
    room.description = "Nearby: {{objects}}.".into();
    let mut state = GameState::new(&game.world);
    let first = describe_room(&game.world, &state, game.world.room(&camp_id).unwrap());
    assert!(first.contains("rusty dagger"), "{first}");
    for item in &game.world.items {
        if state.location(&item.id) == Some(ItemLocation::Room(camp_id.clone())) {
            state
                .item_locations
                .insert(item.id.clone(), ItemLocation::Inventory);
        }
    }
    let second = describe_room(&game.world, &state, game.world.room(&camp_id).unwrap());
    assert!(second.contains("Nearby: nothing."), "{second}");
    assert!(!second.contains("Nearby: rusty dagger"), "{second}");
}

#[test]
fn defeat_blocks_gameplay_but_allows_manual_load_and_quit() {
    use lupine::encounters::{
        EncounterContent, EncounterEntry, EncounterPersistence, EncounterTable, EnemyAttack,
    };
    use lupine::rpg_content::RpgContent;
    use lupine::runner::run_configured_with_io;
    use lupine::shops::ShopContent;
    use lupine::wallet::WalletContent;
    use std::io::Cursor;

    let root = std::path::Path::new("samples/rpg_persistent");
    let game = load_game(root).unwrap();
    let rpg = RpgContent::load(root, &game.world, true).unwrap();
    let ids = game
        .world
        .rooms
        .iter()
        .map(|room| room.id.clone())
        .collect::<Vec<_>>();
    let mut encounters = EncounterContent::load(root, &ids, true).unwrap();
    let goblin = encounters.enemies.get_mut("goblin").unwrap();
    goblin.attacks = vec![EnemyAttack {
        name: "Fatal Blow".into(),
        damage: 1000,
        weight: 1,
    }];
    encounters.tables.insert(
        "camp".into(),
        EncounterTable {
            persistence: EncounterPersistence::Temporary,
            chance: 100,
            min_enemies: 1,
            max_enemies: 1,
            enemies: vec![EncounterEntry {
                enemy: "goblin".into(),
                weight: 1,
                requires: vec![],
            }],
        },
    );
    let wallet = WalletContent::load(root, true).unwrap();
    let shops = ShopContent::load(root, &game.world, &wallet, &rpg).unwrap();
    let save_root = std::env::temp_dir().join(format!("lupine_defeat_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&save_root);
    let script = "take dagger\nattack goblin with dagger\nbuy healing\ntake all\nsave Cheating\nsaves\nload not_found\nquit\nyes\n";
    let mut output = Vec::new();
    run_configured_with_io(
        game.world,
        &game.title,
        &game.id,
        &save_root,
        game.save_compatibility_version,
        game.config,
        rpg,
        encounters,
        wallet,
        shops,
        Cursor::new(script),
        &mut output,
    )
    .unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("YOU HAVE BEEN DEFEATED"), "{text}");
    assert_eq!(text.matches("YOU HAVE BEEN DEFEATED").count(), 1, "{text}");
    let after_death = text.split_once("YOU HAVE BEEN DEFEATED").unwrap().1;
    assert!(
        !after_death.contains("Enemies:"),
        "defeat must end combat output: {text}"
    );
    assert!(!text.contains("You cannot attack until restored"), "{text}");
    assert!(
        text.matches("You have been defeated. Use").count() >= 3,
        "{text}"
    );
    assert!(text.contains("No saved games."), "{text}");
    assert!(text.contains("Load failed"), "{text}");
    assert!(
        !save_root.exists(),
        "defeat must never cause an implicit save"
    );
}
