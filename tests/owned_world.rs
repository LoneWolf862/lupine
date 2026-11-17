//! Verifies that the engine can run a world built from runtime-owned strings.
//! No TOML loader is implemented yet; this guards the v0.5.0-A prerequisite.
use lupine::commands::{Command, Direction};
use lupine::engine::{execute_command, ActionOutcome};
use lupine::state::GameState;
use lupine::world::{
    Condition, DescriptionVariant, Effect, ExitRule, Interaction, Item, Room, World,
};

fn dynamic(text: &str) -> String {
    // Construct a new String at runtime rather than borrowing a string literal.
    text.chars().collect()
}

#[test]
fn accepts_runtime_allocated_names_descriptions_and_ids() {
    let world = World {
        starting_room: dynamic("start"),
        rooms: vec![
            Room {
                id: dynamic("start"),
                title: dynamic("Entry"),
                description: dynamic("An unlit room."),
                descriptions: vec![DescriptionVariant {
                    when: vec![Condition::FlagSet(dynamic("unlocked"))],
                    text: dynamic("The passage is open."),
                }],
                art: None,
                art_variants: vec![],
                exits: vec![(Direction::North, dynamic("next"))],
            },
            Room {
                id: dynamic("next"),
                title: dynamic("Second Room"),
                description: dynamic("Another room."),
                descriptions: vec![],
                art: None,
                art_variants: vec![],
                exits: vec![],
            },
        ],
        items: vec![
            Item {
                id: dynamic("key"),
                name: dynamic("key"),
                aliases: vec![dynamic("little key")],
                description: dynamic("An iron key."),
                descriptions: vec![],
                portable: true,
                starting_room: dynamic("start"),
            },
            Item {
                id: dynamic("lock"),
                name: dynamic("lock"),
                aliases: vec![],
                description: dynamic("A sturdy lock."),
                descriptions: vec![],
                portable: false,
                starting_room: dynamic("start"),
            },
        ],
        exit_rules: vec![ExitRule {
            from: dynamic("start"),
            direction: Direction::North,
            requires: vec![Condition::FlagSet(dynamic("unlocked"))],
            blocked_text: dynamic("It is locked."),
        }],
        interactions: vec![Interaction {
            item: dynamic("key"),
            target: dynamic("lock"),
            requires: vec![Condition::FlagUnset(dynamic("unlocked"))],
            effects: vec![Effect::SetFlag(dynamic("unlocked"))],
            failure_text: dynamic("Already unlocked."),
            success_text: dynamic("You hear a click."),
        }],
    };
    assert!(world.validate().is_ok());
    let mut state = GameState::new(&world);
    assert_eq!(
        execute_command(&world, &mut state, Command::Move(Direction::North)),
        ActionOutcome::Message("It is locked.".into())
    );
    execute_command(&world, &mut state, Command::Take(dynamic("little key")));
    assert_eq!(
        execute_command(
            &world,
            &mut state,
            Command::Use {
                item: dynamic("key"),
                target: dynamic("lock"),
            }
        ),
        ActionOutcome::Message("You hear a click.".into())
    );
    assert!(matches!(
        execute_command(&world, &mut state, Command::Look),
        ActionOutcome::Message(message) if message.contains("passage is open")
    ));
    assert!(matches!(
        execute_command(&world, &mut state, Command::Move(Direction::North)),
        ActionOutcome::Message(message) if message.contains("Second Room")
    ));
}
