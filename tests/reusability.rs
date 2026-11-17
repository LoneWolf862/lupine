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
        starting_room: "pier",
        rooms: vec![Room {
            id: "pier",
            title: "Sample Pier",
            description: "The water is calm.",
            descriptions: vec![DescriptionVariant {
                when: vec![Condition::FlagSet("lamp_lit")],
                text: "A bright lantern lights the pier.",
            }],
            exits: vec![],
        }],
        items: vec![
            Item {
                id: "match",
                name: "match",
                aliases: vec!["match"],
                description: "A match.",
                descriptions: vec![],
                portable: true,
                starting_room: "pier",
            },
            Item {
                id: "lamp",
                name: "lamp",
                aliases: vec!["lamp"],
                description: "An unlit lamp.",
                descriptions: vec![],
                portable: false,
                starting_room: "pier",
            },
        ],
        interactions: vec![Interaction {
            item: "match",
            target: "lamp",
            requires: vec![Condition::FlagUnset("lamp_lit")],
            effects: vec![Effect::SetFlag("lamp_lit")],
            failure_text: "The lamp is already lit.",
            success_text: "The lamp lights.",
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
    world.rooms[0].exits.push((Direction::East, "missing_room"));
    world.items[0].starting_room = "missing_room";
    world.items.push(Item {
        id: "match",
        name: "match two",
        aliases: vec![],
        description: "Duplicate",
        descriptions: vec![],
        portable: true,
        starting_room: "pier",
    });
    let errors = world.validate().unwrap_err().join("\n");
    assert!(errors.contains("Duplicate or empty item id"));
    assert!(errors.contains("missing room"));
}

#[test]
fn invalid_effect_is_rejected_before_play() {
    let mut world = alternate_world();
    world.interactions[0].effects.push(Effect::MoveItemToRoom {
        item: "missing",
        room: "pier",
    });
    assert!(world
        .validate()
        .unwrap_err()
        .iter()
        .any(|error| error.contains("Move effect")));
}
