use lupine::rpg_content::RpgContent;
use lupine::runner::run_configured_with_io;
use lupine::shops::ShopContent;
use lupine::state::GameState;
use lupine::story_loader::load_game;
use lupine::wallet::WalletContent;
use std::io::Cursor;
use std::path::Path;

fn fixture() -> (
    lupine::story_loader::LoadedGame,
    WalletContent,
    ShopContent,
    RpgContent,
    GameState,
) {
    let root = Path::new("samples/rpg_transactions");
    let game = load_game(root).unwrap();
    let wallet = WalletContent::load(root, true).unwrap();
    let rpg = RpgContent::load(root, &game.world, true).unwrap();
    let shops = ShopContent::load(root, &game.world, &wallet, &rpg).unwrap();
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
fn altar_uses_atomic_mana_offer_and_preserves_state_on_failure() {
    let (game, wallet, shops, _, mut state) = fixture();
    let initial = state.clone();
    let message = shops.use_transaction_item(&game.world, &mut state, &wallet, "altar");
    assert!(message.unwrap().contains("full"));
    assert_eq!(state, initial);

    state.character.as_mut().unwrap().mp = 10;
    let message = shops.use_transaction_item(&game.world, &mut state, &wallet, "shrine");
    assert!(message.unwrap().contains("altar hums"));
    assert_eq!(state.character.as_ref().unwrap().mp, 30);
    assert_eq!(state.wallet["gold"], 17);

    // The binding cannot be remotely invoked from a different room.
    state.current_room = "woods".to_owned();
    let before = state.clone();
    let message = shops.use_transaction_item(&game.world, &mut state, &wallet, "altar");
    assert!(message.unwrap().contains("cannot use"));
    assert_eq!(state, before);
}

#[test]
fn altar_does_not_charge_when_funds_are_insufficient() {
    let (game, wallet, shops, _, mut state) = fixture();
    state.wallet.insert("gold".to_owned(), 0);
    state.character.as_mut().unwrap().mp = 5;
    let before = state.clone();
    let message = shops.use_transaction_item(&game.world, &mut state, &wallet, "mana altar");
    assert!(message.is_some());
    assert_eq!(state, before);
}

#[test]
fn scripted_use_altar_reuses_normal_shop_without_implicit_save() {
    let (game, wallet, shops, rpg, _) = fixture();
    let root = Path::new("samples/rpg_transactions");
    let encounter_rooms = game
        .world
        .rooms
        .iter()
        .map(|r| r.id.clone())
        .collect::<Vec<_>>();
    let encounters =
        lupine::encounters::EncounterContent::load(root, &encounter_rooms, true).unwrap();
    let mut output = Vec::new();
    let save_root =
        std::env::temp_dir().join(format!("lupine_story_txn_no_save_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&save_root);
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
        Cursor::new(b"use altar\nwallet\nquit\n"),
        &mut output,
    )
    .unwrap();
    let response = String::from_utf8(output).unwrap();
    assert!(response.contains("already full"), "{response}");
    assert!(response.contains("Gold: 25"), "{response}");
    assert!(
        !save_root.exists(),
        "nothing should be written without save"
    );
}
