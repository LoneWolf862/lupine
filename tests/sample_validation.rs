//! Validate all shipped TOML adventures using the same loaders as `lupine validate`.
use std::path::Path;

#[test]
fn bundled_story_samples_pass_all_content_validators() {
    for name in [
        "rpg_spellbook",
        "rules_demo",
        "objects_demo",
        "events_demo",
        "dialogue_demo",
        "economy_demo",
        "rpg_rewards",
        "rpg_transactions",
        "rpg_persistent",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(name);
        let game =
            lupine::story_loader::load_game(&path).unwrap_or_else(|e| panic!("{name}: story: {e}"));
        let has_rpg = game.config.character.is_some();
        let rpg = lupine::rpg_content::RpgContent::load(&path, &game.world, has_rpg)
            .unwrap_or_else(|e| panic!("{name}: RPG: {e}"));
        let room_ids: Vec<String> = game.world.rooms.iter().map(|r| r.id.clone()).collect();
        let encounters = lupine::encounters::EncounterContent::load(&path, &room_ids, has_rpg)
            .unwrap_or_else(|e| panic!("{name}: encounters: {e}"));
        encounters
            .validate_loot(&game.world)
            .unwrap_or_else(|e| panic!("{name}: loot: {e}"));
        let wallet = lupine::wallet::WalletContent::load(&path, has_rpg)
            .unwrap_or_else(|e| panic!("{name}: wallet: {e}"));
        wallet
            .validate_encounters(&encounters)
            .unwrap_or_else(|e| panic!("{name}: currency rewards: {e}"));
        lupine::shops::ShopContent::load(&path, &game.world, &wallet, &rpg)
            .unwrap_or_else(|e| panic!("{name}: shops/interactions: {e}"));
    }
}
