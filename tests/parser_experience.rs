use lupine::{
    adventure::Adventure,
    commands::Command,
    encounters::{EncounterContent, EnemyInstance},
    engine::execute_command,
    parser::parse_command,
    player_input::prepare,
    resolution::{self, Candidate},
    rpg_content::RpgContent,
    state::{GameState, ItemLocation},
    world::World,
};
use std::{io::Cursor, path::Path};
#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn world() -> World {
    let mut w = fixture_world::fixture_world();
    let key = lupine::world::Item {
        id: "iron_key".into(),
        name: "Iron Key".into(),
        aliases: vec!["key".into()],
        description: "A test key.".into(),
        descriptions: vec![],
        portable: true,
        starting_room: w.starting_room.clone(),
    };
    w.items.push(key);
    w
}
fn prep(
    line: &str,
    answers: &str,
    world: &World,
    state: &GameState,
    rpg: &RpgContent,
    encounters: &EncounterContent,
) -> (Option<String>, String) {
    let before = state.clone();
    let mut out = Vec::new();
    let result = prepare(
        line,
        world,
        state,
        rpg,
        encounters,
        &mut Cursor::new(answers),
        &mut out,
    )
    .unwrap();
    assert_eq!(*state, before, "preparation must never mutate");
    (result, String::from_utf8(out).unwrap())
}

#[test]
fn synonyms_case_spaces_articles_and_multiword_names() {
    let w = world();
    let s = GameState::new(&w);
    for line in [
        "take bronze key",
        "pick up the bronze key",
        "grab a bronze key",
    ] {
        assert_eq!(parse_command(line), Ok(Command::Take("bronze key".into())));
    }
    for line in [" PICK  UP the BRASS KEY ", "get brass", "grab brass_key"] {
        assert_eq!(
            prep(line, "", &w, &s, &Default::default(), &Default::default())
                .0
                .as_deref(),
            Some("take brass_key")
        );
    }
}
#[test]
fn exact_ids_then_names_then_partial_and_no_arbitrary_ties() {
    let candidates = vec![
        Candidate::new("key", "Plain Key"),
        Candidate::new("other", "Key"),
        Candidate::new("third", "Key ring"),
    ];
    assert_eq!(resolution::unique("key", &candidates).unwrap(), "key");
    assert_eq!(
        resolution::unique("key ring", &candidates).unwrap(),
        "third"
    );
    assert_eq!(resolution::unique("ring", &candidates).unwrap(), "third");
    assert!(resolution::unique("ke", &candidates).is_err());
    assert!(resolution::matches("", &candidates).is_empty());
}
#[test]
fn clarification_retries_cancels_and_resolves_both_use_targets() {
    let w = world();
    let s = GameState::new(&w);
    for ending in ["cancel\n", "0\n", "back\n", "", "wrong\n\ncancel\n"] {
        let (result, out) = prep(
            "take key",
            ending,
            &w,
            &s,
            &Default::default(),
            &Default::default(),
        );
        assert!(result.is_none());
        assert!(out.contains("Which key"));
    }
    let (result, _) = prep(
        "take key",
        "wrong\n2\n",
        &w,
        &s,
        &Default::default(),
        &Default::default(),
    );
    assert_eq!(result.as_deref(), Some("take iron_key"));
    let (result, _) = prep(
        "use key on key",
        "brass_key\niron_key\n",
        &w,
        &s,
        &Default::default(),
        &Default::default(),
    );
    assert_eq!(result.as_deref(), Some("use brass_key on iron_key"));
}
#[test]
fn role_relevance_and_suggestions_never_reveal_hidden_items_or_mutate() {
    let w = world();
    let mut s = GameState::new(&w);
    s.item_locations
        .insert("brass_key".into(), ItemLocation::Inventory);
    assert_eq!(
        prep(
            "drop key",
            "",
            &w,
            &s,
            &Default::default(),
            &Default::default()
        )
        .0
        .as_deref(),
        Some("drop brass_key")
    );
    assert_eq!(
        prep(
            "take key",
            "",
            &w,
            &s,
            &Default::default(),
            &Default::default()
        )
        .0
        .as_deref(),
        Some("take iron_key")
    );
    let (result, out) = prep(
        "drop brss key",
        "",
        &w,
        &s,
        &Default::default(),
        &Default::default(),
    );
    assert!(result.is_none());
    assert!(out.contains("Did you mean"));
    let (_, out) = prep(
        "examine pnel",
        "",
        &w,
        &s,
        &Default::default(),
        &Default::default(),
    );
    assert!(!out.contains("control panel"));
    let before = s.clone();
    execute_command(&w, &mut s, Command::Take("key_unknown".into()));
    assert_eq!(s, before);
}
#[test]
fn combat_and_spell_aliases_resolve_instances_and_conserve_failed_state() {
    let a = Adventure::load(Path::new("samples/rpg_persistent")).unwrap();
    let mut s = GameState::new(&a.game.world);
    s.character = Some(
        a.game
            .config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    for id in a.rpg.weapons.keys() {
        s.item_locations.insert(id.clone(), ItemLocation::Inventory);
    }
    s.learned_abilities.insert("fireball".into());
    s.current_room = "woods".into();
    s.active_enemies = vec![
        EnemyInstance {
            instance_id: 1,
            enemy_id: "goblin".into(),
            current_hp: 20,
        },
        EnemyInstance {
            instance_id: 2,
            enemy_id: "goblin".into(),
            current_hp: 20,
        },
    ];
    for command in [
        " ATTACK the GOBLIN WITH the DAGGER ",
        "use the dagger on the goblin",
    ] {
        let (result, out) = prep(command, "2\n", &a.game.world, &s, &a.rpg, &a.encounters);
        assert!(out.contains("Which goblin"), "{command}: {result:?} {out}");
        assert_eq!(
            result.as_deref(),
            Some("attack goblin #2 with rusty_dagger")
        );
    }
    for command in [
        "cast fireball on goblin #1",
        "use fireball on goblin #1",
        "cast fire on #1",
    ] {
        assert_eq!(
            prep(command, "", &a.game.world, &s, &a.rpg, &a.encounters)
                .0
                .as_deref(),
            Some("cast fireball on goblin #1")
        );
    }
    for command in [
        "attack gobln with dagger",
        "cast firebll on goblin",
        "attack goblin with daggr",
    ] {
        let (result, out) = prep(command, "", &a.game.world, &s, &a.rpg, &a.encounters);
        assert!(result.is_none());
        assert!(out.contains("Did you mean") || out.contains("available targets"));
    }
}
#[test]
fn visible_items_keep_priority_over_spell_names() {
    let mut a = Adventure::load(Path::new("samples/rpg_persistent")).unwrap();
    let item = lupine::world::Item {
        id: "fireball".into(),
        name: "Fireball".into(),
        aliases: vec![],
        description: "A test item.".into(),
        descriptions: vec![],
        portable: true,
        starting_room: a.game.world.starting_room.clone(),
    };
    a.game.world.items.push(item);
    let mut s = GameState::new(&a.game.world);
    s.learned_abilities.insert("fireball".into());
    s.active_enemies.push(EnemyInstance {
        instance_id: 1,
        enemy_id: "goblin".into(),
        current_hp: 20,
    });
    let (result, _) = prep(
        "use fireball on goblin",
        "",
        &a.game.world,
        &s,
        &a.rpg,
        &a.encounters,
    );
    assert!(
        !result.is_some_and(|r| r.starts_with("cast ")),
        "must not cast when an ordinary item matches"
    );
}
#[test]
fn scripted_runner_clarifies_and_suggests_without_automatic_action() {
    let w = world();
    let mut out = Vec::new();
    let saves = std::env::temp_dir().join(format!("lupine-parser-no-save-{}", std::process::id()));
    lupine::runner::run_with_io_and_saves(
        w,
        "Parser Test",
        "parser_test",
        &saves,
        Cursor::new("take key\ncancel\nexmain table\ntake key\n2\ninventory\nquit\nyes\n"),
        &mut out,
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("Did you mean 'examine table'"), "{text}");
    assert!(text.contains("You take the Iron Key"));
    assert!(!text.contains("You take the brass key"));
    assert!(!saves.exists());
}

#[test]
fn direct_rpg_calls_reject_ambiguous_abilities_without_spending_mana() {
    let mut a = Adventure::load(Path::new("samples/rpg_persistent")).unwrap();
    let mut twin = a.rpg.abilities["fireball"].clone();
    twin.id = "second_flame".into();
    a.rpg.abilities.insert(twin.id.clone(), twin);
    let mut s = GameState::new(&a.game.world);
    s.character = Some(
        a.game
            .config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    s.learned_abilities
        .extend(["fireball".into(), "second_flame".into()]);
    let before = s.clone();
    assert!(a.rpg.cast(&mut s, "fire").contains("Which"));
    assert_eq!(s, before);
    assert_eq!(resolution::distance("exmain", "examine"), Some(2));
    assert_eq!(
        resolution::distance(&"a".repeat(10000), &"a".repeat(10000)),
        None
    );
}
