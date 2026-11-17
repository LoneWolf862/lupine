use lupine::encounters::{EncounterContent, RandomSource};
use lupine::state::GameState;
use lupine::story_loader::load_game;
use lupine::wallet::WalletContent;
use std::path::Path;
struct ZeroRng;
impl RandomSource for ZeroRng {
    fn below(&mut self, _limit: u32) -> u32 {
        0
    }
}
fn setup() -> (
    lupine::story_loader::LoadedGame,
    WalletContent,
    EncounterContent,
    GameState,
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_rewards");
    let game = load_game(&root).unwrap();
    let wallet = WalletContent::load(&root, true).unwrap();
    let room_ids = game
        .world
        .rooms
        .iter()
        .map(|r| r.id.clone())
        .collect::<Vec<_>>();
    let encounters = EncounterContent::load(&root, &room_ids, true).unwrap();
    wallet.validate_encounters(&encounters).unwrap();
    let mut state = GameState::new(&game.world);
    state.character = Some(
        game.progression
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    state.wallet = wallet.initial_balances();
    (game, wallet, encounters, state)
}
#[test]
fn wallet_currency_names_starting_values_and_alias_data() {
    let (_, wallet, _, state) = setup();
    assert_eq!(state.wallet["gold"], 25);
    assert_eq!(state.wallet["rubies"], 0);
    assert!(wallet.describe(&state).contains("Gold: 25 G"));
    assert!(wallet.describe(&state).contains("Rubies: 0 R"));
}
#[test]
fn reward_roll_and_checked_credit_debit() {
    let (_, wallet, encounters, mut state) = setup();
    let msg = wallet.award(&encounters, "goblin", &mut state, &mut ZeroRng);
    assert!(msg.contains("5 Gold"), "{msg}");
    assert_eq!(state.wallet["gold"], 30);
    assert!(wallet.debit(&mut state, "gold", 31).is_err());
    assert_eq!(state.wallet["gold"], 30);
    wallet.debit(&mut state, "gold", 30).unwrap();
    assert_eq!(state.wallet["gold"], 0);
    wallet.credit(&mut state, "gold", u64::MAX).unwrap();
    assert!(wallet.credit(&mut state, "gold", 1).is_err());
    assert_eq!(state.wallet["gold"], u64::MAX);
}
#[test]
fn wallet_rork_roundtrip_and_legacy() {
    let (game, wallet, _, mut state) = setup();
    wallet.credit(&mut state, "gold", 15).unwrap();
    let data = lupine::save::encode_save(&game.world, &state, &game.id, "Wallet Test").unwrap();
    let restored = lupine::save::decode_save(&game.world, &game.id, &data).unwrap();
    assert_eq!(restored.wallet, state.wallet);
    // Legacy RORK files had no wallet section.
    let legacy = data.split("[wallet]").next().unwrap();
    let previous = lupine::save::decode_save(&game.world, &game.id, legacy).unwrap();
    assert!(previous.wallet.is_empty());
}
#[test]
fn unknown_currency_in_rork_is_rejected_by_story_wallet() {
    let (game, wallet, _, mut state) = setup();
    state.wallet.insert("unconfigured".into(), 99);
    let data = lupine::save::encode_save(&game.world, &state, &game.id, "Wallet Test").unwrap();
    let decoded = lupine::save::decode_save(&game.world, &game.id, &data).unwrap();
    assert!(wallet.validate_balances(&decoded).is_err());
}
