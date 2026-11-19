//! A completely separate Cargo package using Lupine as a library.
//! No edits to Lupine's parser, rules or original fixture are required.
use lupine::commands::Direction;
use lupine::world::{Condition, DescriptionVariant, Effect, ExitRule, Interaction, Item, Room, World};

fn world() -> World {
    World {
        dialogue: Default::default(),
        events: Default::default(),
        object_defs: Default::default(),
        actions: Vec::new(),
        rules: Default::default(),
        starting_room: "porch".into(),
        rooms: vec![
            Room {
                id: "porch".into(), title: "Sample Porch".into(),
                description: "A small porch faces a locked workshop to the north.".into(),
                descriptions: vec![DescriptionVariant {
                    when: vec![Condition::FlagSet("door_open".into())],
                    text: "The workshop door stands open to the north.".into(),
                }],
                art: None,
                art_variants: vec![],
                exits: vec![(Direction::North, "workshop".into())],
            },
            Room {
                id: "workshop".into(), title: "Sample Workshop".into(),
                description: "A quiet workshop. A doorway leads south.".into(),
                descriptions: vec![],
                art: None,
                art_variants: vec![],
                exits: vec![(Direction::South, "porch".into())],
            },
        ],
        items: vec![
            Item {
                id: "copper_token".into(), name: "copper token".into(), aliases: vec!["token".into()],
                description: "A small copper token with a gear stamped on it.".into(),
                descriptions: vec![], portable: true, starting_room: "porch".into(),
            },
            Item {
                id: "door".into(), name: "workshop door".into(), aliases: vec!["door".into()],
                description: "A door with a token-sized mechanism.".into(),
                descriptions: vec![], portable: false, starting_room: "porch".into(),
            },
        ],
        interactions: vec![Interaction {
            outcomes: Vec::new(),
            item: "copper_token".into(), target: "door".into(),
            requires: vec![Condition::FlagUnset("door_open".into())],
            effects: vec![Effect::SetFlag("door_open".into())],
            failure_text: "The door is already unlocked.".into(),
            success_text: "The token clicks into place and the door unlocks.".into(),
        }],
        exit_rules: vec![ExitRule {
            from: "porch".into(), direction: Direction::North,
            requires: vec![Condition::FlagSet("door_open".into())],
            blocked_text: "The workshop door is still locked.".into(),
        }],
    }
}

fn main() -> std::io::Result<()> {
    lupine::runner::run_with_game_id(world(), "Independent Sample Adventure", "independent_sample")
}
