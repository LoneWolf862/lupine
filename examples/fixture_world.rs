//! Generic test adventure. Not part of the reusable Lupine engine library.
use lupine::commands::Direction;
use lupine::world::{
    Condition, DescriptionVariant, Effect, ExitRule, Interaction, Item, Room, World,
};

pub fn fixture_world() -> World {
    World {
        rules: Default::default(),
        starting_room: "foyer".into(),
        rooms: vec![
            Room {
                id: "foyer".into(),
                title: "Test Foyer".into(),
                description: "A plain room used to test navigation. A corridor leads north.".into(),
                descriptions: vec![],
                art: None,
                art_variants: vec![],
                exits: vec![(Direction::North, "corridor".into())],
            },
            Room {
                id: "corridor".into(),
                title: "Test Corridor".into(),
                description: "A short corridor. The foyer is south; a test chamber is east.".into(),
                descriptions: vec![],
                art: None,
                art_variants: vec![],
                exits: vec![
                    (Direction::South, "foyer".into()),
                    (Direction::East, "chamber".into()),
                ],
            },
            Room {
                id: "chamber".into(),
                title: "Test Chamber".into(),
                description:
                    "A chamber used to verify a stateful interaction. A sealed passage leads north."
                        .into(),
                descriptions: vec![DescriptionVariant {
                    when: vec![Condition::FlagSet("panel_activated".into())],
                    text: "The control panel glows green. The northern passage is now open.".into(),
                }],
                art: None,
                art_variants: vec![],
                exits: vec![
                    (Direction::West, "corridor".into()),
                    (Direction::North, "vault".into()),
                ],
            },
            Room {
                id: "vault".into(),
                title: "Test Vault".into(),
                description: "An empty room demonstrating a conditional exit.".into(),
                descriptions: vec![],
                art: None,
                art_variants: vec![],
                exits: vec![(Direction::South, "chamber".into())],
            },
        ],
        items: vec![
            Item {
                id: "brass_key".into(),
                name: "brass key".into(),
                aliases: vec!["key".into(), "brass key".into()],
                description: "An ordinary brass key for engine testing.".into(),
                descriptions: vec![],
                portable: true,
                starting_room: "foyer".into(),
            },
            Item {
                id: "stone_table".into(),
                name: "stone table".into(),
                aliases: vec!["table".into(), "stone table".into()],
                description: "A sturdy table fixed to the floor.".into(),
                descriptions: vec![],
                portable: false,
                starting_room: "foyer".into(),
            },
            Item {
                id: "test_panel".into(),
                name: "test panel".into(),
                aliases: vec!["panel".into(), "test panel".into()],
                description: "A stationary panel with a key-shaped slot.".into(),
                descriptions: vec![DescriptionVariant {
                    when: vec![Condition::FlagSet("panel_activated".into())],
                    text: "The indicator shines green. The mechanism is active.".into(),
                }],
                portable: false,
                starting_room: "chamber".into(),
            },
        ],
        exit_rules: vec![ExitRule {
            from: "chamber".into(),
            direction: Direction::North,
            requires: vec![Condition::FlagSet("panel_activated".into())],
            blocked_text: "The northern passage is sealed.".into(),
        }],
        interactions: vec![Interaction {
            outcomes: Vec::new(),
            item: "brass_key".into(),
            target: "test_panel".into(),
            requires: vec![Condition::FlagUnset("panel_activated".into())],
            effects: vec![Effect::SetFlag("panel_activated".into())],
            failure_text: "That interaction has already been completed.".into(),
            success_text: "The key turns in the panel. A test indicator lights up.".into(),
        }],
    }
}
