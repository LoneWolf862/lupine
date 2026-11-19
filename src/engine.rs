use crate::commands::Command;
use crate::render::{describe_room, help};
use crate::state::{GameState, ItemLocation};
use crate::world::{select_description, Effect, Item, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    Message(String),
    Quit(String),
}

/// Resolve player-supplied names against visible objects or inventory.
/// Exact names/aliases win over partial matches; ambiguity never guesses.
fn resolve<'a>(world: &'a World, state: &GameState, text: &str) -> Result<&'a Item, String> {
    let id = crate::resolution::unique(text, &crate::resolution::items(world, state))?;
    world
        .item(&id)
        .ok_or_else(|| "Resolved item is missing.".into())
}

fn message(text: impl Into<String>) -> ActionOutcome {
    ActionOutcome::Message(text.into())
}

fn apply_effect(effect: &Effect, state: &mut GameState) {
    match effect {
        Effect::SetFlag(flag) => {
            state.flags.insert(flag.clone());
        }
        Effect::ClearFlag(flag) => {
            state.flags.remove(flag);
        }
        Effect::MoveItemToRoom { item, room } => {
            state
                .item_locations
                .insert(item.clone(), ItemLocation::Room(room.clone()));
        }
    }
}

pub fn execute_command(world: &World, state: &mut GameState, command: Command) -> ActionOutcome {
    match command {
        Command::Help => message(help()),
        Command::Quit => ActionOutcome::Quit("Goodbye!".into()),
        Command::Look => match world.room(&state.current_room) {
            Some(room) => message(describe_room(world, state, room)),
            None => message("The current room could not be found."),
        },
        Command::Move(direction) => {
            let Some(room) = world.room(&state.current_room) else {
                return message("The current room could not be found.");
            };
            let Some((_, destination)) = room.exits.iter().find(|(exit, _)| *exit == direction)
            else {
                return message(format!("You cannot go {} from here.", direction.label()));
            };
            if let Some(rule) = world
                .exit_rules
                .iter()
                .find(|rule| rule.from == room.id && rule.direction == direction)
            {
                if !rule.requires.iter().all(|condition| condition.met(state)) {
                    return message(&rule.blocked_text);
                }
            }
            if let Some(next_room) = world.room(destination) {
                state.current_room = next_room.id.clone();
                message(describe_room(world, state, next_room))
            } else {
                message("That exit leads to an unknown room.")
            }
        }
        Command::Inventory => {
            let carried = world
                .items
                .iter()
                .filter(|item| state.location(&item.id) == Some(ItemLocation::Inventory))
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>();
            if carried.is_empty() {
                message("You are carrying nothing.")
            } else {
                message(format!("You are carrying: {}", carried.join(", ")))
            }
        }
        Command::Examine(target) => match resolve(world, state, &target) {
            Ok(item) => message(select_description(
                &item.description,
                &item.descriptions,
                state,
            )),
            Err(error) => message(error),
        },
        Command::Take(target) => match resolve(world, state, &target) {
            Err(error) => message(error),
            Ok(item) if state.location(&item.id) == Some(ItemLocation::Inventory) => {
                message(format!("You are already carrying the {}.", item.name))
            }
            Ok(item) if !item.portable => message(format!("The {} cannot be moved.", item.name)),
            Ok(item) => {
                state
                    .item_locations
                    .insert(item.id.clone(), ItemLocation::Inventory);
                message(format!("You take the {}.", item.name))
            }
        },
        Command::Drop(target) => match resolve(world, state, &target) {
            Err(error) => message(error),
            Ok(item) if state.location(&item.id) != Some(ItemLocation::Inventory) => {
                message(format!("You are not carrying the {}.", item.name))
            }
            Ok(item) => {
                state.item_locations.insert(
                    item.id.clone(),
                    ItemLocation::Room(state.current_room.clone()),
                );
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
            if state.location(&item.id) != Some(ItemLocation::Inventory) {
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
                Some(rule) if !rule.requires.iter().all(|condition| condition.met(state)) => {
                    message(&rule.failure_text)
                }
                Some(rule) => {
                    for effect in &rule.effects {
                        apply_effect(effect, state);
                    }
                    message(&rule.success_text)
                }
            }
        }
    }
}
