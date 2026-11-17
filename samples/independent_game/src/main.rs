//! A completely separate Cargo package using Lupine as a library.
//! No edits to Lupine's parser, rules or original fixture are required.
use lupine::commands::Direction;
use lupine::world::{Condition, DescriptionVariant, Effect, ExitRule, Interaction, Item, Room, World};

fn world() -> World {
    World {
        starting_room: "porch",
        rooms: vec![
            Room {
                id: "porch", title: "Sample Porch",
                description: "A small porch faces a locked workshop to the north.",
                descriptions: vec![DescriptionVariant {
                    when: vec![Condition::FlagSet("door_open")],
                    text: "The workshop door stands open to the north.",
                }],
                exits: vec![(Direction::North, "workshop")],
            },
            Room {
                id: "workshop", title: "Sample Workshop",
                description: "A quiet workshop. A doorway leads south.",
                descriptions: vec![],
                exits: vec![(Direction::South, "porch")],
            },
        ],
        items: vec![
            Item {
                id: "copper_token", name: "copper token", aliases: vec!["token"],
                description: "A small copper token with a gear stamped on it.",
                descriptions: vec![], portable: true, starting_room: "porch",
            },
            Item {
                id: "door", name: "workshop door", aliases: vec!["door"],
                description: "A door with a token-sized mechanism.",
                descriptions: vec![], portable: false, starting_room: "porch",
            },
        ],
        interactions: vec![Interaction {
            item: "copper_token", target: "door",
            requires: vec![Condition::FlagUnset("door_open")],
            effects: vec![Effect::SetFlag("door_open")],
            failure_text: "The door is already unlocked.",
            success_text: "The token clicks into place and the door unlocks.",
        }],
        exit_rules: vec![ExitRule {
            from: "porch", direction: Direction::North,
            requires: vec![Condition::FlagSet("door_open")],
            blocked_text: "The workshop door is still locked.",
        }],
    }
}

fn main() -> std::io::Result<()> {
    lupine::run(world(), "Independent Sample Adventure")
}
