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
        offer_on_enter: None,
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
        "talk guide\ncancel\n",
        "talk guide\n",
        "talk guide\n\n\u{1b}[31m\ncancel\n",
    ] {
        let (state, text) = play(mini(), input);
        assert_eq!(state, baseline, "{text}");
    }
    let (state, text) = play(
        mini(),
        "talk guide\nZoë O'Neil\nL\nstats\ntalk guide\ncancel\n",
    );
    assert_eq!(state.player_name.as_deref(), Some("Zoë O'Neil"));
    assert!(text.contains("Good to meet you, Zoë O'Neil"));
    assert!(text.contains("Welcome back, Zoë O'Neil"));
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
    let (state, text) = play(a, "talk guide\nRex\n");
    assert_eq!(state, baseline, "{text}");
    assert!(!text.contains("Good to meet you, Rex"));
}

#[test]
fn manual_save_restart_restores_name_and_rejects_old_or_invalid_names() {
    let saves = temp();
    let (saved, text) = play_at(mini(), "talk guide\nRex\nL\nsave named\n", &saves);
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
    let baseline = play(mini(), "talk guide\nRex\nL\nuse shop key on door\nwest\n").0;
    let (state, text) = play(
        a,
        "talk guide\nRex\nL\nuse shop key on door\nwest\nbuy steel\ncancel\n",
    );
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
    a.wallet
        .currencies
        .get_mut("coins")
        .unwrap()
        .starting_amount = 2;
    let (state, text) = play(
        a,
        "talk guide\nRex\nL\nuse shop key on door\nwest\nbuy sword from the trader\n",
    );
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
    let (_, text) = play(mini(), "talk guide\nRex\nL\nquit\nyes\n");
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

fn route_before(command: &str) -> String {
    let suite: toml::Value =
        toml::from_str(include_str!("../adventures/mini_adventure/scenarios.toml")).unwrap();
    let commands = suite["scenario"][0]["commands"].as_array().unwrap();
    commands
        .iter()
        .map(|c| c.as_str().unwrap())
        .take_while(|c| *c != command)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn plural_selection_is_not_implicit_bulk_and_filtered_bulk_is_one_turn() {
    let baseline = play(mini(), "").0;
    for script in ["take scrolls\ncancel\n", "take all nonexistent\n", "2\n"] {
        assert_eq!(play(mini(), script).0, baseline);
    }
    let (one, _) = play(mini(), "take scrolls\n1\n");
    assert_eq!(
        one.carried_quantity("fireball_scroll") + one.carried_quantity("healing_scroll"),
        1
    );
    let (both, text) = play(mini(), "take all scrolls\ninventory\n");
    assert_eq!(both.events.turn, 1, "{text}");
    assert!(both.is_carried("fireball_scroll") && both.is_carried("healing_scroll"));
    assert!(!both.is_carried("dagger") && !both.is_carried("shop_key"));
    assert!(text.contains("You are carrying:\n  fireball scroll\n  healing scroll"));
    let (dropped, _) = play(mini(), "take all scrolls\ndrop all scrolls\n");
    assert!(!dropped.is_carried("fireball_scroll") && !dropped.is_carried("healing_scroll"));
    assert_eq!(dropped.events.turn, 2);
}

#[test]
fn persistent_dialogue_settles_each_choice_and_never_repeats_the_gift() {
    let (state, text) = play(
        mini(),
        "talk to the guide\nRex\nwelcome\nshop\nL\nuse shop key on door\ntalk guide\nwelcome\nL\n",
    );
    assert_eq!(state.events.turn, 5, "{text}");
    assert_eq!(state.carried_quantity("shop_key"), 0);
    assert!(state.flags.contains("shop_open"));
    assert!(!state.learned_abilities.contains("light"));
    assert_eq!(text.matches("What is your name?").count(), 1);
    assert_eq!(text.matches("Here is the shop key").count(), 1);
    assert!(!text.contains("cancels):\n> \n"), "Double prompt: {text}");
    assert!(text.contains("Back to gameplay"));
}

#[test]
fn conversation_stops_after_a_choice_moves_the_npc_or_ends_the_game() {
    for effect in [
        Effect::MoveNpc {
            npc: "guide".into(),
            room: Some("woods".into()),
        },
        Effect::EndGame("victory".into()),
    ] {
        let mut a = mini();
        a.game
            .world
            .dialogue
            .npcs
            .get_mut("guide")
            .unwrap()
            .nodes
            .get_mut("topics")
            .unwrap()
            .choices[0]
            .effects
            .push(effect);
        let (state, text) = play(a, "talk guide\nRex\nwelcome\nlook\n");
        assert_eq!(state.events.turn, 2, "{text}");
        assert_eq!(text.matches("Choose a response").count(), 1, "{text}");
    }
}

#[test]
fn invalid_automatic_prompt_and_roomless_offer_are_rejected() {
    let mut a = mini();
    a.game
        .world
        .dialogue
        .npcs
        .get_mut("guide")
        .unwrap()
        .nodes
        .get_mut("topics")
        .unwrap()
        .prompt_choice = Some("welcome".into());
    assert!(a.game.world.validate().is_err());
    let mut a = mini();
    let action = a
        .game
        .world
        .actions
        .iter_mut()
        .find(|a| a.id == "unlock_gate")
        .unwrap();
    action.room = None;
    assert!(a.game.world.validate().is_err());
}

#[test]
fn gate_offer_no_cancel_eof_and_look_are_free_and_yes_unlocks_once() {
    let setup = route_before("yes");
    let baseline = play(mini(), &setup).0;
    for answer in ["", "no\nlook\n", "cancel\n", "0\n", "maybe\nno\n"] {
        let (state, text) = play(mini(), &format!("{setup}{answer}"));
        assert_eq!(state, baseline, "{text}");
        assert_eq!(
            text.matches("Use your brass key to unlock the gate?")
                .count(),
            if answer.starts_with("maybe") { 2 } else { 1 }
        );
    }
    let (state, text) = play(mini(), &format!("{setup}yes\nlook\n"));
    assert!(state.flags.contains("gate_open"), "{text}");
    assert_eq!(state.events.turn, baseline.events.turn + 1);
    assert_eq!(
        text.matches("Use your brass key to unlock the gate?")
            .count(),
        1
    );
    let manual = play(mini(), &format!("{setup}no\nunlock gate with brass key\n")).0;
    assert_eq!(state, manual);
}

#[test]
fn accepted_offer_rolls_back_with_its_later_hooks() {
    let setup = route_before("yes");
    let baseline = play(mini(), &setup).0;
    let mut a = mini();
    a.game.world.events.events.push(Event {
        id: "failed_unlock".into(),
        on: Trigger::Turn,
        once: true,
        requires: vec![lupine::rules::Condition::FlagSet("gate_open".into())],
        effects: vec![Effect::Debit {
            currency: "coins".into(),
            amount: 999,
        }],
        text: String::new(),
    });
    let (state, text) = play(a, &format!("{setup}yes\nlook\n"));
    assert_eq!(state, baseline, "{text}");
    assert!(!text.contains("The gate unlocks."));
    assert_eq!(
        text.matches("Use your brass key to unlock the gate?")
            .count(),
        1
    );
}

#[test]
fn chest_contents_keys_objectives_and_focus_follow_actual_progress() {
    let setup = route_before("take all");
    let (state, text) = play(mini(), &setup);
    assert!(!state.is_carried("chest_key") && !state.is_carried("shop_key"));
    assert!(
        text.contains("You open the sturdy chest.\nIt is open. Contents:\n  brass gate key"),
        "{text}"
    );
    let (state, text) = play(
        mini(),
        &format!("{setup}take all\nequip coat\ninventory\njournal\nlook in chest\n"),
    );
    assert_eq!(state.events.quests["challenge"], "gate");
    assert!(text.contains("Return south to camp, then east to the gate."));
    assert!(text.contains("padded coat (equipped)"));
    assert!(text.contains("Contents:\n  nothing"));
    let setup = route_before("use mana altar");
    let (state, _) = play(
        mini(),
        &format!("{setup}wait\nwait\nuse mana altar\neast\neast\n"),
    );
    assert_eq!(state.current_room, "vault");
    assert!(state.systems.statuses["focus"] >= 4);
    let c = state.character.unwrap();
    assert_eq!(c.hp, c.max_hp);
    assert_eq!(c.mp, c.max_mp);
}

#[test]
fn shops_list_automatically_and_purchased_stock_is_visible_on_look() {
    let setup = route_before("sell trophy");
    let (_, text) = play(mini(), &format!("{setup}sell trophy\nbuy sword\nlook\n"));
    assert_eq!(text.matches("Trader buys from you:").count(), 2, "{text}");
    assert_eq!(text.matches("Trader sells to you:").count(), 2);
    assert!(text.contains("Steel sword (quantity: 1) — Coins: 10 (sold out)"));
    assert!(text.contains("Ration (quantity: 1) — Coins: 2 (10 left)"));
}

#[test]
fn complete_checkpoint_round_trip_and_old_content_revision_rejection() {
    let saves = temp();
    let setup = route_before("attack stone golem with steel sword");
    let (saved, text) = play_at(mini(), &format!("{setup}west\nsave prepared\n"), &saves);
    assert!(text.contains("Saved: prepared"));
    let (loaded, text) = play_at(mini(), "load prepared\n", &saves);
    assert_eq!(loaded, saved, "{text}");
    assert!(!text.contains("Use your brass key to unlock the gate?"));
    let mut old = mini();
    old.game.save_compatibility_version = 1;
    play_at(old, "save old-revision\n", &saves);
    let old_path =
        lupine::save_files::slot_path(&saves, "lupine_mini_adventure", "old-revision").unwrap();
    let old_bytes = std::fs::read(&old_path).unwrap();
    let (state, text) = play_at(mini(), "load old-revision\n", &saves);
    assert_eq!(state, play(mini(), "").0, "{text}");
    assert!(text.contains("Load failed"));
    assert_eq!(old_bytes, std::fs::read(&old_path).unwrap());
    std::fs::remove_dir_all(saves).unwrap();
}

#[test]
fn blank_input_after_victory_is_silent_and_free() {
    let setup = route_before("read journal");
    let (baseline, before) = play(mini(), &setup);
    let (state, text) = play(mini(), &format!("{setup}\n\n"));
    assert_eq!(state, baseline);
    assert_eq!(
        text.matches("This adventure has ended").count(),
        before.matches("This adventure has ended").count()
    );
}

#[test]
fn filtered_bulk_and_a_later_dialogue_choice_roll_back_independently() {
    let mut a = mini();
    a.game.world.events.events.push(Event {
        id: "fail_bulk".into(),
        on: Trigger::Turn,
        once: true,
        requires: vec![lupine::rules::Condition::HasItem("healing_scroll".into())],
        effects: vec![Effect::Debit {
            currency: "coins".into(),
            amount: 999,
        }],
        text: String::new(),
    });
    let (state, text) = play(a, "take all scrolls\n");
    assert_eq!(state, play(mini(), "").0);
    assert!(!text.contains("You take the"), "{text}");
    let mut a = mini();
    a.game
        .world
        .dialogue
        .npcs
        .get_mut("guide")
        .unwrap()
        .nodes
        .get_mut("topics")
        .unwrap()
        .choices[0]
        .effects
        .push(Effect::Debit {
            currency: "coins".into(),
            amount: 999,
        });
    let baseline = play(mini(), "talk guide\nRex\nL\n").0;
    let (state, text) = play(a, "talk guide\nRex\nwelcome\nlook\n");
    assert_eq!(state, baseline, "{text}");
    assert_eq!(text.matches("Here is the shop key").count(), 1);
}
