use crate::commands::Command;
use crate::render::{describe_room, help};
use crate::state::{GameState, ItemLocation};
use crate::world::{Item, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    Message(String),
    Quit(String),
}

/// Resolve player-supplied names only against the objects currently accessible.
/// Exact names/aliases win over partial matches; ambiguity never guesses.
fn resolve<'a>(world: &'a World, state: &GameState, text: &str) -> Result<&'a Item, String> {
    let accessible: Vec<&Item> = world
        .items
        .iter()
        .filter(|item| state.is_accessible(item.id))
        .collect();
    let exact: Vec<&Item> = accessible
        .iter()
        .copied()
        .filter(|item| item.name == text || item.aliases.contains(&text))
        .collect();
    let candidates = if exact.is_empty() {
        accessible
            .into_iter()
            .filter(|item| {
                item.name.contains(text) || item.aliases.iter().any(|alias| alias.contains(text))
            })
            .collect::<Vec<_>>()
    } else {
        exact
    };
    match candidates.as_slice() {
        [] => Err(format!("You don't see '{text}' here or in your inventory.")),
        [item] => Ok(item),
        _ => Err(format!(
            "Which {text} do you mean: {}?",
            candidates
                .iter()
                .map(|item| item.name)
                .collect::<Vec<_>>()
                .join(" or ")
        )),
    }
}

fn message(text: impl Into<String>) -> ActionOutcome {
    ActionOutcome::Message(text.into())
}

pub fn execute_command(world: &World, state: &mut GameState, command: Command) -> ActionOutcome {
    match command {
        Command::Help => message(help()),
        Command::Quit => ActionOutcome::Quit("Goodbye!".into()),
        Command::Look => match world.room(state.current_room) {
            Some(room) => message(describe_room(world, state, room)),
            None => message("The current room could not be found."),
        },
        Command::Move(direction) => {
            let Some(room) = world.room(state.current_room) else {
                return message("The current room could not be found.");
            };
            let Some((_, destination)) = room.exits.iter().find(|(exit, _)| *exit == direction)
            else {
                return message(format!("You cannot go {} from here.", direction.label()));
            };
            if let Some(next_room) = world.room(destination) {
                state.current_room = next_room.id;
                message(describe_room(world, state, next_room))
            } else {
                message("That exit leads to an unknown room.")
            }
        }
        Command::Inventory => {
            let carried = world
                .items
                .iter()
                .filter(|item| state.location(item.id) == Some(ItemLocation::Inventory))
                .map(|item| item.name)
                .collect::<Vec<_>>();
            if carried.is_empty() {
                message("You are carrying nothing.")
            } else {
                message(format!("You are carrying: {}", carried.join(", ")))
            }
        }
        Command::Examine(target) => match resolve(world, state, &target) {
            Ok(item) => message(item.description),
            Err(error) => message(error),
        },
        Command::Take(target) => match resolve(world, state, &target) {
            Err(error) => message(error),
            Ok(item) if state.location(item.id) == Some(ItemLocation::Inventory) => {
                message(format!("You are already carrying the {}.", item.name))
            }
            Ok(item) if !item.portable => message(format!("The {} cannot be moved.", item.name)),
            Ok(item) => {
                state
                    .item_locations
                    .insert(item.id, ItemLocation::Inventory);
                message(format!("You take the {}.", item.name))
            }
        },
        Command::Drop(target) => match resolve(world, state, &target) {
            Err(error) => message(error),
            Ok(item) if state.location(item.id) != Some(ItemLocation::Inventory) => {
                message(format!("You are not carrying the {}.", item.name))
            }
            Ok(item) => {
                state
                    .item_locations
                    .insert(item.id, ItemLocation::Room(state.current_room));
                message(format!("You drop the {}.", item.name))
            }
        },
        Command::Use { item, target } => {
            let item = match resolve(world, state, &item) {
                Ok(item) => item,
                Err(error) => return message(error),
            };
            let target = match resolve(world, state, &target) {
                Ok(target) => target,
                Err(error) => return message(error),
            };
            if state.location(item.id) != Some(ItemLocation::Inventory) {
                return message(format!("You must be carrying the {} to use it.", item.name));
            }
            match world
                .interactions
                .iter()
                .find(|rule| rule.item == item.id && rule.target == target.id)
            {
                None => message(format!(
                    "Using the {} on the {} has no effect.",
                    item.name, target.name
                )),
                Some(rule) if state.flags.contains(rule.flag) => {
                    message("That interaction has already been completed.")
                }
                Some(rule) => {
                    state.flags.insert(rule.flag);
                    message(rule.success_text)
                }
            }
        }
    }
}
