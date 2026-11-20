use lupine::{
    adventure::Adventure,
    commands::Command,
    events::{Event, Trigger},
    parser::{parse_command, ParseError},
    rules::{Condition, Effect},
    state::{GameState, ItemLocation},
    world::{Interaction, Item},
};
use std::{
    io::Cursor,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

fn sample() -> Adventure {
    Adventure::load(Path::new("samples/objects_demo")).unwrap()
}

fn play(a: Adventure, commands: &str) -> (GameState, String) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let saves = std::env::temp_dir().join(format!(
        "lupine-parser-feedback-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut output = Vec::new();
    let state = lupine::runner::run_session_with_io(
        a.game.world,
        &a.game.title,
        &a.game.id,
        &saves,
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
    assert!(!saves.exists(), "These commands must never save");
    (state, String::from_utf8(output).unwrap())
}

#[test]
fn inspection_phrases_parse_articles_and_missing_targets() {
    for command in [
        "LOOK IN the chest",
        "look inside a chest",
        "look into chest",
        "read chest",
        "search chest",
    ] {
        assert_eq!(parse_command(command), Ok(Command::Examine("chest".into())));
    }
    for command in [
        "look in",
        "look inside",
        "look into",
        "read",
        "read the",
        "search the",
    ] {
        assert_eq!(
            parse_command(command),
            Err(ParseError::MissingTarget("examine"))
        );
    }
}

#[test]
fn inspection_and_missing_key_guidance_are_free_and_never_reveal_hidden_contents() {
    let (before, _) = play(sample(), "");
    let (after, text) = play(
        sample(),
        "look in chest\nlook inside chest\nsearch chest\nread note\nread gem\nunlock chest\n",
    );
    assert_eq!(before, after, "{text}");
    assert!(text.contains("It is locked."), "{text}");
    assert!(text.contains("unlock chest with <key>"), "{text}");
    assert!(
        !text.contains("A demonstration note.") && !text.contains("A demonstration gem."),
        "{text}"
    );
    assert!(!text.contains("head note"), "{text}");
}

#[test]
fn sample_phrases_unlock_inspect_and_read_through_the_real_runner() {
    let (state, text) = play(sample(), "use the key on the chest\nlook inside chest\nopen chest\nlook into chest\nsearch chest\nsearch the room\nread the note\n");
    assert!(text.contains("You unlock the chest."), "{text}");
    assert!(text.contains("It is closed."), "{text}");
    assert!(text.contains("Contents: gem"), "{text}");
    assert!(text.contains("A demonstration note."), "{text}");
    assert!(state.objects.containers["chest"].open);
    assert!(!state.objects.containers["chest"].locked);
    assert!(state.is_carried("key"));
    assert_eq!(
        state.events.turn, 3,
        "Only unlocking, opening and authored search spend turns"
    );
}

#[test]
fn misspellings_suggest_valid_phrases_without_executing_them() {
    let (before, _) = play(sample(), "");
    let (after, text) = play(
        sample(),
        "serch room\nserch chest\nexamin chest\nreed note\nhed note\n",
    );
    assert_eq!(before, after, "{text}");
    for command in ["search room", "search chest", "examine chest", "read note"] {
        assert!(
            text.contains(&format!("Did you mean '{command}'")),
            "{text}"
        );
    }
    assert!(!text.contains("Did you mean 'head note'"), "{text}");
    assert!(text.contains("suggestions do not run automatically"));
}

#[test]
fn authored_read_and_search_keep_priority_including_failed_requirements() {
    for verb in ["read", "search"] {
        for blocked in [false, true] {
            let mut a = sample();
            let mut action = a.game.world.actions[0].clone();
            action.id = "inspect_key".into();
            action.commands = vec![format!("{verb} {{target}}")];
            action.target = Some("key".into());
            action.success_text = "Authored inspection succeeded.".into();
            action.failure_text = "Authored inspection blocked.".into();
            if blocked {
                action.requires = vec![Condition::HasItem("gem".into())];
            }
            a.game.world.actions.push(action);
            let (state, text) = play(a, &format!("{verb} key\n"));
            assert!(
                text.contains(if blocked {
                    "Authored inspection blocked."
                } else {
                    "Authored inspection succeeded."
                }),
                "{text}"
            );
            assert!(!text.contains("A demonstration key."), "{text}");
            assert_eq!(state.objects.hidden.contains("note"), blocked);
            assert_eq!(state.events.turn, u64::from(!blocked));
        }
    }
}

#[test]
fn authored_item_interaction_wins_over_container_key_fallback() {
    for blocked in [false, true] {
        let mut a = sample();
        a.game.world.interactions.push(Interaction {
            item: "key".into(),
            target: "chest".into(),
            outcomes: vec![],
            requires: if blocked {
                vec![Condition::HasItem("gem".into())]
            } else {
                vec![]
            },
            effects: vec![Effect::RevealItem("note".into())],
            success_text: "The authored interaction runs.".into(),
            failure_text: "The authored interaction refuses.".into(),
        });
        let (state, text) = play(a, "use key on chest\n");
        assert!(state.objects.containers["chest"].locked, "{text}");
        assert!(
            text.contains(if blocked {
                "The authored interaction refuses."
            } else {
                "The authored interaction runs."
            }),
            "{text}"
        );
        assert_eq!(state.objects.hidden.contains("note"), blocked);
    }
}

#[test]
fn key_fallback_requires_accessible_carried_key_and_never_toggles_or_opens() {
    let (state, text) = play(
        sample(),
        "use ration on chest\nuse key on chest\nuse key on chest\n",
    );
    assert!(!state.objects.containers["chest"].locked && !state.objects.containers["chest"].open);
    assert_eq!(state.events.turn, 1, "{text}");
    assert!(text.contains("already closed and unlocked"), "{text}");
    assert_eq!(state.carried_quantity("ration"), 5);
    for contained in [false, true] {
        let mut a = sample();
        a.game.world.object_defs.get_mut("key").unwrap().initial = Some(if contained {
            ItemLocation::Container("chest".into())
        } else {
            ItemLocation::Room("workroom".into())
        });
        let (state, text) = play(a, "use key on chest\n");
        assert!(state.objects.containers["chest"].locked, "{text}");
        assert_eq!(state.events.turn, 0);
    }
}

fn ambiguous() -> Adventure {
    let mut a = sample();
    a.game
        .world
        .items
        .iter_mut()
        .find(|i| i.id == "key")
        .unwrap()
        .aliases
        .push("tool".into());
    a.game.world.items.push(Item {
        id: "other_key".into(),
        name: "Other key".into(),
        aliases: vec!["tool".into()],
        description: "Another key.".into(),
        descriptions: vec![],
        portable: true,
        starting_room: "workroom".into(),
    });
    a
}

#[test]
fn inspection_and_key_ambiguity_require_selection_and_cancel_without_changes() {
    let (before, _) = play(ambiguous(), "");
    for command in ["read tool", "search tool", "use tool on chest"] {
        for answer in ["cancel\n", ""] {
            let (after, text) = play(ambiguous(), &format!("{command}\n{answer}"));
            assert_eq!(before, after, "{text}");
            assert!(text.contains("Which tool"), "{text}");
        }
    }
    let (after, text) = play(ambiguous(), "use tool on chest\nkey\n");
    assert!(!after.objects.containers["chest"].locked, "{text}");
    assert_eq!(after.events.turn, 1);
}

#[test]
fn key_fallback_rolls_back_with_the_rest_of_a_failed_turn() {
    fn failing() -> Adventure {
        let mut a = sample();
        a.game.world.rules.counters.insert("meals".into(), i64::MAX);
        a.game.world.events.events.push(Event {
            id: "failure".into(),
            on: Trigger::Turn,
            once: true,
            requires: vec![],
            effects: vec![Effect::AddCounter {
                id: "meals".into(),
                amount: 1,
            }],
            text: "This must not be published.".into(),
        });
        a
    }
    let (before, _) = play(failing(), "");
    let (after, text) = play(failing(), "use key on chest\n");
    assert_eq!(before, after, "{text}");
    assert!(text.contains("all turn changes were rolled back"), "{text}");
    assert!(
        !text.contains("You unlock the chest.") && !text.contains("This must not be published."),
        "{text}"
    );
}
