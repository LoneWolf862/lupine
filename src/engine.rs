use crate::commands::Command;
use crate::render::{describe_room, help};
use crate::state::GameState;
use crate::world::{select_description, Item, World};

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

pub fn execute_command(world: &World, state: &mut GameState, command: Command) -> ActionOutcome {
    execute_command_report(world, state, command).0
}
pub fn execute_command_report(
    world: &World,
    state: &mut GameState,
    command: Command,
) -> (ActionOutcome, bool) {
    let mut performed = false;
    let outcome = execute_core(world, state, command, &mut performed);
    (outcome, performed)
}
fn execute_core(
    world: &World,
    state: &mut GameState,
    command: Command,
    performed: &mut bool,
) -> ActionOutcome {
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
                *performed = true;
                state.current_room = next_room.id.clone();
                state.story.visited.insert(next_room.id.clone());
                message(describe_room(world, state, next_room))
            } else {
                message("That exit leads to an unknown room.")
            }
        }
        Command::Inventory => {
            let carried = world
                .items
                .iter()
                .filter(|item| state.is_carried(&item.id) && state.is_accessible(&item.id))
                .map(|item| {
                    if crate::objects::definition(world, &item.id).stackable {
                        format!("{} ({})", item.name, state.carried_quantity(&item.id))
                    } else {
                        item.name.clone()
                    }
                })
                .collect::<Vec<_>>();
            if carried.is_empty() {
                message("You are carrying nothing.")
            } else {
                message(format!("You are carrying: {}", carried.join(", ")))
            }
        }
        Command::Examine(target) => match resolve(world, state, &target) {
            Ok(item) => message(format!(
                "{}{}",
                select_description(&item.description, &item.descriptions, state),
                crate::objects::describe_container(world, state, &item.id)
            )),
            Err(error) => message(error),
        },
        Command::Take(target) => match resolve(world, state, &target) {
            Err(error) => message(error),
            Ok(item) => match crate::objects::manipulate(world, state, "take", &item.id, None, 1) {
                Ok(text) => {
                    *performed = true;
                    message(text)
                }
                Err(e) => message(e),
            },
        },
        Command::Drop(target) => match resolve(world, state, &target) {
            Err(error) => message(error),
            Ok(item) => match crate::objects::manipulate(world, state, "drop", &item.id, None, 1) {
                Ok(text) => {
                    *performed = true;
                    message(text)
                }
                Err(e) => message(e),
            },
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
            if item.portable && !state.is_carried(&item.id) {
                return message(format!("You must be carrying the {} to use it.", item.name));
            }
            match world
                .interactions
                .iter()
                .find(|rule| rule.item == item.id && rule.target == target.id)
            {
                None if crate::objects::definition(world, &target.id)
                    .container
                    .and_then(|c| c.key)
                    .as_deref()
                    == Some(item.id.as_str()) =>
                {
                    // An explicit authored interaction always wins, even when its
                    // requirements fail. Otherwise using this container's key
                    // means unlock, never toggle the lock or silently open it.
                    match crate::objects::manipulate(
                        world,
                        state,
                        "unlock",
                        &target.id,
                        Some(&item.id),
                        1,
                    ) {
                        Ok(text) => {
                            *performed = true;
                            message(text)
                        }
                        Err(error) => message(error),
                    }
                }
                None => message(format!(
                    "Using the {} on the {} has no effect.",
                    item.name, target.name
                )),
                Some(rule) if !rule.requires.iter().all(|condition| condition.met(state)) => {
                    message(&rule.failure_text)
                }
                Some(rule) => {
                    let outcome = rule
                        .outcomes
                        .iter()
                        .find(|o| o.when.iter().all(|c| c.met(state)));
                    let (effects, text) = outcome
                        .map_or((&rule.effects, rule.success_text.as_str()), |o| {
                            (&o.effects, o.text.as_str())
                        });
                    match crate::rules::apply(world, state, effects) {
                        Ok(mut lines) => {
                            *performed = true;
                            lines.insert(0, text.to_owned());
                            message(lines.join("\n"))
                        }
                        Err(e) => message(format!("Action could not complete: {e}")),
                    }
                }
            }
        }
    }
}
