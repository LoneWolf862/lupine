//! Generic test adventure. Not part of the reusable Lupine engine library.
use lupine::commands::Direction;
use lupine::world::{
    Condition, DescriptionVariant, Effect, ExitRule, Interaction, Item, Room, World,
};

pub fn fixture_world() -> World {
    World {
        starting_room: "foyer",
        rooms: vec![
            Room {
                id: "foyer",
                title: "Test Foyer",
                description: "A plain room used to test navigation. A corridor leads north.",
                descriptions: vec![],
                exits: vec![(Direction::North, "corridor")],
            },
            Room {
                id: "corridor",
                title: "Test Corridor",
                description: "A short corridor. The foyer is south; a test chamber is east.",
                descriptions: vec![],
                exits: vec![(Direction::South, "foyer"), (Direction::East, "chamber")],
            },
            Room {
                id: "chamber",
                title: "Test Chamber",
                description:
                    "A chamber used to verify a stateful interaction. A sealed passage leads north.",
                descriptions: vec![DescriptionVariant {
                    when: vec![Condition::FlagSet("panel_activated")],
                    text: "The control panel glows green. The northern passage is now open.",
                }],
                exits: vec![(Direction::West, "corridor"), (Direction::North, "vault")],
            },
            Room {
                id: "vault",
                title: "Test Vault",
                description: "An empty room demonstrating a conditional exit.",
                descriptions: vec![],
                exits: vec![(Direction::South, "chamber")],
            },
        ],
        items: vec![
            Item {
                id: "brass_key",
                name: "brass key",
                aliases: vec!["key", "brass key"],
                description: "An ordinary brass key for engine testing.",
                descriptions: vec![],
                portable: true,
                starting_room: "foyer",
            },
            Item {
                id: "stone_table",
                name: "stone table",
                aliases: vec!["table", "stone table"],
                description: "A sturdy table fixed to the floor.",
                descriptions: vec![],
                portable: false,
                starting_room: "foyer",
            },
            Item {
                id: "test_panel",
                name: "test panel",
                aliases: vec!["panel", "test panel"],
                description: "A stationary panel with a key-shaped slot.",
                descriptions: vec![DescriptionVariant {
                    when: vec![Condition::FlagSet("panel_activated")],
                    text: "The indicator shines green. The mechanism is active.",
                }],
                portable: false,
                starting_room: "chamber",
            },
        ],
        exit_rules: vec![ExitRule {
            from: "chamber",
            direction: Direction::North,
            requires: vec![Condition::FlagSet("panel_activated")],
            blocked_text: "The northern passage is sealed.",
        }],
        interactions: vec![Interaction {
            item: "brass_key",
            target: "test_panel",
            requires: vec![Condition::FlagUnset("panel_activated")],
            effects: vec![Effect::SetFlag("panel_activated")],
            failure_text: "That interaction has already been completed.",
            success_text: "The key turns in the panel. A test indicator lights up.",
        }],
    }
}
