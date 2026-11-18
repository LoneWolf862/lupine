use lupine::rpg_content::RpgContent;
use lupine::save::{decode_save, encode_save};
use lupine::shops::ShopContent;
use lupine::state::GameState;
use lupine::story_loader::load_game;
use lupine::wallet::WalletContent;
use std::path::Path;

fn fixture() -> (
    lupine::story_loader::LoadedGame,
    WalletContent,
    ShopContent,
    RpgContent,
    GameState,
) {
    let path = Path::new("samples/rpg_rewards");
    let game = load_game(path).unwrap();
    let wallet = WalletContent::load(path, true).unwrap();
    let rpg = RpgContent::load(path, &game.world, true).unwrap();
    let shops = ShopContent::load(path, &game.world, &wallet, &rpg).unwrap();
    let mut state = GameState::new(&game.world);
    state.wallet = wallet.initial_balances();
    state.character = Some(
        game.config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    shops.prepare_inventory(&mut state);
    (game, wallet, shops, rpg, state)
}

#[test]
fn healing_full_hp_does_not_charge() {
    let (_game, wallet, shops, _rpg, mut state) = fixture();
    let before = state.clone();
    assert!(shops.transact(&mut state, &wallet, "healing", "").is_err());
    assert_eq!(state, before);
    state.character.as_mut().unwrap().hp = 50;
    shops.transact(&mut state, &wallet, "healing", "").unwrap();
    assert_eq!(state.character.as_ref().unwrap().hp, 100);
    assert_eq!(state.wallet["gold"], 15);
}
#[test]
fn stock_one_and_rork_roundtrip() {
    let (game, wallet, shops, _rpg, mut state) = fixture();
    shops
        .transact(&mut state, &wallet, "camp_permit", "")
        .unwrap();
    assert!(state.flags.contains("camp_permit"));
    let prior = state.clone();
    assert!(shops
        .transact(&mut state, &wallet, "camp_permit", "")
        .is_err());
    assert_eq!(state, prior);
    let serialized = encode_save(&game.world, &state, &game.id, "Shop Test").unwrap();
    let restored = decode_save(&game.world, &game.id, &serialized).unwrap();
    shops.validate_saved(&restored).unwrap();
    assert_eq!(state, restored);
}
#[test]
fn invalid_shop_or_room_cannot_spend() {
    let (_game, wallet, shops, _rpg, mut state) = fixture();
    let before = state.clone();
    assert!(shops
        .transact(&mut state, &wallet, "healing", "missing")
        .is_err());
    assert_eq!(state, before);
    state.current_room = "woods".into();
    let before = state.clone();
    assert!(shops.transact(&mut state, &wallet, "healing", "").is_err());
    assert_eq!(state, before);
}
