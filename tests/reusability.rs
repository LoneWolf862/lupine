use lupine::commands::{Command, Direction};
use lupine::engine::{execute_command, ActionOutcome};
use lupine::state::GameState;
use lupine::world::{Condition, DescriptionVariant, Effect, Interaction, Item, Room, World};

#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn outcome(world: &World, state: &mut GameState, command: Command) -> String {
    match execute_command(world, state, command) {
        ActionOutcome::Message(text) => text,
        ActionOutcome::Quit(_) => panic!("Unexpected quit"),
    }
}

#[test]
fn fixture_dynamic_descriptions_and_unlock() {
    let world = fixture_world::fixture_world();
    assert!(world.validate().is_ok());
    let mut state = GameState::new(&world);
    outcome(&world, &mut state, Command::Take("brass key".into()));
    outcome(&world, &mut state, Command::Move(Direction::North));
    outcome(&world, &mut state, Command::Move(Direction::East));
    let before = state.clone();
    assert!(outcome(&world, &mut state, Command::Move(Direction::North)).contains("sealed"));
    assert_eq!(state, before);
    assert!(outcome(
        &world,
        &mut state,
        Command::Use {
            item: "key".into(),
            target: "panel".into()
        }
    )
    .contains("indicator"));
    assert!(outcome(&world, &mut state, Command::Look).contains("glows green"));
    assert!(outcome(&world, &mut state, Command::Examine("panel".into())).contains("shines green"));
    assert!(outcome(&world, &mut state, Command::Move(Direction::North)).contains("Test Vault"));
}

/// A completely unrelated adventure uses the same engine APIs and parser.
fn alternate_world() -> World {
    World {
        dialogue: Default::default(),
        events: Default::default(),
        object_defs: Default::default(),
        actions: Vec::new(),
        rules: Default::default(),
        starting_room: "pier".into(),
        rooms: vec![Room {
            id: "pier".into(),
            title: "Sample Pier".into(),
            description: "The water is calm.".into(),
            descriptions: vec![DescriptionVariant {
                when: vec![Condition::FlagSet("lamp_lit".into())],
                text: "A bright lantern lights the pier.".into(),
            }],
            art: None,
            art_variants: vec![],
            exits: vec![],
        }],
        items: vec![
            Item {
                id: "match".into(),
                name: "match".into(),
                aliases: vec!["match".into()],
                description: "A match.".into(),
                descriptions: vec![],
                portable: true,
                starting_room: "pier".into(),
            },
            Item {
                id: "lamp".into(),
                name: "lamp".into(),
                aliases: vec!["lamp".into()],
                description: "An unlit lamp.".into(),
                descriptions: vec![],
                portable: false,
                starting_room: "pier".into(),
            },
        ],
        interactions: vec![Interaction {
            outcomes: Vec::new(),
            item: "match".into(),
            target: "lamp".into(),
            requires: vec![Condition::FlagUnset("lamp_lit".into())],
            effects: vec![Effect::SetFlag("lamp_lit".into())],
            failure_text: "The lamp is already lit.".into(),
            success_text: "The lamp lights.".into(),
        }],
        exit_rules: vec![],
    }
}

#[test]
fn different_world_reuses_engine_without_modifications() {
    let world = alternate_world();
    assert!(world.validate().is_ok());
    let mut state = GameState::new(&world);
    assert!(outcome(&world, &mut state, Command::Look).contains("water is calm"));
    outcome(&world, &mut state, Command::Take("match".into()));
    assert!(outcome(
        &world,
        &mut state,
        Command::Use {
            item: "match".into(),
            target: "lamp".into()
        }
    )
    .contains("lights"));
    assert!(outcome(&world, &mut state, Command::Look).contains("bright lantern"));
    assert!(outcome(
        &world,
        &mut state,
        Command::Use {
            item: "match".into(),
            target: "lamp".into()
        }
    )
    .contains("already lit"));
}

#[test]
fn invalid_world_reports_references_and_duplicates() {
    let mut world = alternate_world();
    world.rooms[0]
        .exits
        .push((Direction::East, "missing_room".into()));
    world.items[0].starting_room = "missing_room".into();
    world.items.push(Item {
        id: "match".into(),
        name: "match two".into(),
        aliases: vec![],
        description: "Duplicate".into(),
        descriptions: vec![],
        portable: true,
        starting_room: "pier".into(),
    });
    let errors = world.validate().unwrap_err().join("\n");
    assert!(errors.contains("Duplicate or empty item id"));
    assert!(errors.contains("missing room"));
}

#[test]
fn invalid_effect_is_rejected_before_play() {
    let mut world = alternate_world();
    world.interactions[0].effects.push(Effect::MoveItemToRoom {
        item: "missing".into(),
        room: "pier".into(),
    });
    assert!(world
        .validate()
        .unwrap_err()
        .iter()
        .any(|error| error.contains("Move effect")));
}
