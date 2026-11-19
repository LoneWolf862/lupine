use lupine::rpg::Progression;
use lupine::rpg_content::RpgContent;
use lupine::save::{decode_save, encode_save};
use lupine::state::{GameState, ItemLocation};
use lupine::story_loader::load_game;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp() -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("lupine_abilities_{}_{}", std::process::id(), id));
    fs::create_dir(&path).unwrap();
    path
}
#[test]
fn weapon_requires_inventory_and_scales_with_ap() {
    let game = load_game("story_example").unwrap();
    let root = temp();
    fs::create_dir(root.join("weapons")).unwrap();
    fs::write(
        root.join("weapons/key.toml"),
        "item = 'brass_key'\ndamage = 5\ndamage_type = 'piercing'\n",
    )
    .unwrap();
    let content = RpgContent::load(&root, &game.world, true).unwrap();
    let mut state = GameState::new(&game.world);
    state.character = Some(Progression::default().create_character().unwrap());
    assert!(content
        .weapons(&game.world, &state)
        .contains("no usable weapons"));
    state.set_location("brass_key".into(), ItemLocation::Inventory);
    assert!(content
        .weapons(&game.world, &state)
        .contains("5 piercing damage"));
    state.character.as_mut().unwrap().ap = 200;
    assert!(content
        .weapons(&game.world, &state)
        .contains("10 piercing damage"));
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn learn_scroll_cast_restore_and_save() {
    let mut game = load_game("story_example").unwrap();
    let root = temp();
    fs::create_dir(root.join("spells")).unwrap();
    fs::write(root.join("spells/heal.toml"), "id = 'heal'\nname = 'Heal'\nmana_cost = 8\nkind = 'heal'\npower = 20\nlearn_from_item = 'brass_key'\n").unwrap();
    let content = RpgContent::load(&root, &game.world, true).unwrap();
    game.world.systems.rpg = content.clone();
    game.world.systems.progression = Some(Progression::default());
    let mut state = GameState::new(&game.world);
    state.character = Some(Progression::default().create_character().unwrap());
    assert!(content
        .learn(&game.world, &mut state, "key")
        .contains("must carry"));
    state.set_location("brass_key".into(), ItemLocation::Inventory);
    assert!(content
        .learn(&game.world, &mut state, "key")
        .contains("learn Heal"));
    assert_eq!(state.location("brass_key"), Some(ItemLocation::Consumed));
    assert!(content
        .spellbook(&state, &Default::default())
        .contains("Heal (8 MP)"));
    state.character.as_mut().unwrap().hp = 50;
    assert!(content.cast(&mut state, "heal").contains("HP: 70/100"));
    assert_eq!(state.character.as_ref().unwrap().mp, 22);
    let encoded = encode_save(&game.world, &state, &game.id, "Ability Test").unwrap();
    let restored = decode_save(&game.world, &game.id, &encoded).unwrap();
    assert_eq!(state, restored);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn invalid_definitions_are_rejected() {
    let game = load_game("story_example").unwrap();
    let root = temp();
    fs::create_dir(root.join("weapons")).unwrap();
    fs::write(
        root.join("weapons/sword.toml"),
        "item = 'missing'\ndamage = 10\n",
    )
    .unwrap();
    assert!(RpgContent::load(&root, &game.world, true)
        .unwrap_err()
        .contains("not a defined item"));
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn old_save_has_empty_spellbook_and_no_consumed_items() {
    let game = load_game("story_example").unwrap();
    let state = GameState::new(&game.world);
    let text = encode_save(&game.world, &state, &game.id, "Old Save").unwrap();
    let old = text
        .lines()
        .filter(|line| !line.starts_with("learned_abilities = "))
        .collect::<Vec<_>>()
        .join("\n");
    let restored = decode_save(&game.world, &game.id, &old).unwrap();
    assert!(restored.learned_abilities.is_empty());
}
#[test]
fn damage_ability_does_not_spend_mp_without_combat() {
    let game = load_game("story_example").unwrap();
    let root = temp();
    fs::create_dir(root.join("spells")).unwrap();
    fs::write(
        root.join("spells/fireball.toml"),
        "id='fireball'\nname='Fireball'\nmana_cost=10\nkind='damage'\npower=15\n",
    )
    .unwrap();
    let content = RpgContent::load(&root, &game.world, true).unwrap();
    let mut state = GameState::new(&game.world);
    state.character = Some(Progression::default().create_character().unwrap());
    state.learned_abilities.insert("fireball".into());
    assert!(content
        .cast(&mut state, "fireball on goblin")
        .contains("no MP spent"));
    assert_eq!(state.character.as_ref().unwrap().mp, 30);
    fs::remove_dir_all(root).unwrap();
}
