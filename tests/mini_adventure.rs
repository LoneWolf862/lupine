use lupine::{
    adventure::Adventure,
    events::{Event, Trigger},
    rules::Effect,
    state::GameState,
};
use std::{
    io::Cursor,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

fn mini() -> Adventure {
    Adventure::load(Path::new("adventures/mini_adventure")).unwrap()
}
fn play_at(a: Adventure, commands: &str, saves: &Path) -> (GameState, String) {
    let mut output = Vec::new();
    let state = lupine::runner::run_session_with_io(
        a.game.world,
        &a.game.title,
        &a.game.id,
        saves,
        a.game.save_compatibility_version,
        a.game.config,
        a.rpg,
        a.encounters,
        a.wallet,
        a.shops,
        Cursor::new(commands),
        &mut output,
        Some(1),
    )
    .unwrap();
    (state, String::from_utf8(output).unwrap())
}
fn temp() -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "lupine-mini-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
fn play(a: Adventure, commands: &str) -> (GameState, String) {
    let saves = temp();
    let result = play_at(a, commands, &saves);
    assert!(!saves.exists(), "No implicit saving");
    result
}

#[test]
fn combined_scenarios_use_the_real_runner() {
    let root = Path::new("adventures/mini_adventure");
    let mut output = Vec::new();
    lupine::scenario::run(root, &root.join("scenarios.toml"), &mut output)
        .unwrap_or_else(|e| panic!("{e}\n{}", String::from_utf8_lossy(&output)));
}

#[test]
fn ending_inspection_aliases_are_free_and_do_not_execute_authored_effects() {
    let suite: toml::Value =
        toml::from_str(include_str!("../adventures/mini_adventure/scenarios.toml")).unwrap();
    let route = suite["scenario"][0]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let baseline = play(mini(), &route).0;
    assert!(baseline.events.ending.is_some());
    let mut a = mini();
    a.game.world.actions.push(lupine::actions::Action {
        id: "read_coat".into(),
        commands: vec!["read {target}".into(), "search {target}".into()],
        room: Some("vault".into()),
        target: Some("coat".into()),
        item: None,
        requires: vec![],
        effects: vec![Effect::Credit {
            currency: "coins".into(),
            amount: 50,
        }],
        outcomes: vec![],
        success_text: "Unwanted post-ending reward".into(),
        failure_text: "Cannot read".into(),
    });
    let (state, text) = play(
        a,
        &(route + "read coat\nsearch coat\nread journal\nabilities\nskills\njurnal\nwait\n"),
    );
    assert_eq!(state, baseline, "{text}");
    assert!(text.contains("Equip padded coat to reduce incoming damage."));
    assert!(text.contains("Did you mean 'journal'"), "{text}");
    assert!(!text.contains("Unwanted post-ending reward"));
}

#[test]
fn name_cancellation_eof_and_invalid_input_are_read_only() {
    let baseline = play(mini(), "").0;
    for input in [
        "talk guide\nintroduce\ncancel\n",
        "talk guide\nintroduce\n",
        "talk guide\nintroduce\n\n\u{1b}[31m\ncancel\n",
    ] {
        let (state, text) = play(mini(), input);
        assert_eq!(state, baseline, "{text}");
    }
    let (state, text) = play(
        mini(),
        "talk guide\nintroduce\nZoÃ« O'Neil\nstats\ntalk guide\ncancel\n",
    );
    assert_eq!(state.player_name.as_deref(), Some("ZoÃ« O'Neil"));
    assert!(text.contains("Good to meet you, ZoÃ« O'Neil"));
    assert!(text.contains("Welcome, ZoÃ« O'Neil"));
    assert_eq!(state.events.turn, 1);
}

#[test]
fn name_and_choice_roll_back_when_a_later_turn_hook_fails() {
    let baseline = play(mini(), "").0;
    let mut a = mini();
    a.game.world.events.events.push(Event {
        id: "fail".into(),
        on: Trigger::Turn,
        once: true,
        requires: vec![],
        effects: vec![Effect::Debit {
            currency: "coins".into(),
            amount: 999,
        }],
        text: String::new(),
    });
    let (state, text) = play(a, "talk guide\nintroduce\nRex\n");
    assert_eq!(state, baseline, "{text}");
    assert!(!text.contains("Good to meet you, Rex"));
}

#[test]
fn manual_save_restart_restores_name_and_rejects_old_or_invalid_names() {
    let saves = temp();
    let (saved, text) = play_at(mini(), "talk guide\nintroduce\nRex\nsave named\n", &saves);
    assert!(text.contains("Saved: named"), "{text}");
    let (loaded, text) = play_at(mini(), "load named\n", &saves);
    assert_eq!(loaded, saved, "{text}");
    let a = mini();
    let encoded = lupine::save::encode_save(&a.game.world, &saved, &a.game.id, "named").unwrap();
    for bad in [
        encoded.replace("save_format_version = 7", "save_format_version = 6"),
        encoded.replace("player_name = \"Rex\"", "player_name = \"\""),
    ] {
        assert!(lupine::save::decode_save(&a.game.world, &a.game.id, &bad).is_err());
    }
    for name in ["", " ", "\tRex", "Rex\n", "\u{202e}Rex", &"a".repeat(41)] {
        assert!(lupine::dialogue::validate_player_name(name).is_err());
    }
    std::fs::remove_dir_all(saves).unwrap();
}

#[test]
fn learn_all_and_typos_never_consume_scrolls_or_cast_spells() {
    let setup = "take fireball scroll\ntake healing scroll\n";
    let baseline = play(mini(), setup).0;
    let (state, text) = play(mini(), &format!("{setup}learn all\nlearn ball\n"));
    assert_eq!(state, baseline, "{text}");
    let setup = format!("{setup}learn healing scroll\nlearn fireball scroll\n");
    let baseline = play(mini(), &setup).0;
    let (state, text) = play(
        mini(),
        &format!("{setup}cast heling\ncast fireball ar goblin\n"),
    );
    assert_eq!(state, baseline, "{text}");
    assert!(text.contains("healing_touch"), "{text}");
    assert!(text.contains("Spell syntax:"), "{text}");
}

#[test]
fn buy_and_sell_use_direction_and_item_aliases_without_crossing_offers() {
    let sample = || Adventure::load(Path::new("samples/economy_demo")).unwrap();
    let (state, text) = play(sample(), "sell token\n");
    assert_eq!(state.wallet["coins"], 11, "{text}");
    assert_eq!(state.carried_quantity("token"), 0);
    let baseline = play(sample(), "").0;
    for command in ["buy sell\n", "sell nonexistent\n"] {
        let (state, text) = play(sample(), command);
        assert_eq!(state, baseline, "{command}: {text}");
    }
    let (state, text) = play(sample(), "buy token\n");
    assert_eq!(state.wallet["coins"], 8, "{text}");
    assert_eq!(state.carried_quantity("token"), 5);
}

#[test]
fn ambiguous_shop_offers_cancel_without_spending() {
    let mut a = mini();
    let trader = a.shops.shops.get_mut("trader").unwrap();
    let mut second = trader.offers[1].clone();
    second.id = "second_sword".into();
    trader.offers.push(second);
    let baseline = play(mini(), "east\n").0;
    let (state, text) = play(a, "east\nbuy steel\ncancel\n");
    assert_eq!(state, baseline, "{text}");
    assert!(text.contains("Which steel"));
}

#[test]
fn prepared_offer_and_shop_ids_cannot_be_intercepted_by_display_names() {
    let mut a = mini();
    let trader = a.shops.shops.get_mut("trader").unwrap();
    trader.offers[0].name = "sword".into(); // sell display name collides with buy ID
    trader.offers[1].price = 1;
    let mut other = trader.clone();
    other.id = "aaa".into();
    other.name = "trader".into(); // sorted first, but cannot intercept trader's ID
    other.offers[1].price = 99;
    a.shops.shops.insert("aaa".into(), other);
    let (state, text) = play(a, "east\nbuy sword from the trader\n");
    assert_eq!(state.wallet["coins"], 1, "{text}");
    assert!(state.is_carried("sword"));
    assert_eq!(state.shop_purchases["trader:sword"], 1);
}

#[test]
fn startup_events_do_not_produce_a_false_unsaved_warning() {
    for a in [
        mini(),
        Adventure::load(Path::new("samples/economy_demo")).unwrap(),
    ] {
        let (_, text) = play(a, "quit\n");
        assert!(!text.contains("Unsaved progress"), "{text}");
    }
    let (_, text) = play(mini(), "talk guide\nintroduce\nRex\nquit\nyes\n");
    assert!(text.contains("Unsaved progress"));
}

#[test]
fn player_menu_prioritizes_adventures_and_browse_still_finds_fixtures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let choices = lupine::startup::discover(&[root.to_owned()]);
    assert_eq!(choices.len(), 1);
    assert!(choices[0].title.contains("Mini Adventure"));
    assert!(lupine::startup::discover(&[root.join("samples")]).len() > 1);
    let direct = root.join("samples/economy_demo");
    let choices = lupine::startup::discover(&[root.to_owned(), direct.clone()]);
    assert!(choices
        .iter()
        .any(|c| c.path == direct.canonicalize().unwrap()));
}
