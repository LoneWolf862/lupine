use crate::commands::Command;
use crate::render::{describe_room, help};
use crate::state::GameState;
use crate::world::World;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    Message(String),
    Quit(String),
}

pub fn execute_command(world: &World, state: &mut GameState, command: Command) -> ActionOutcome {
    match command {
        Command::Help => ActionOutcome::Message(help().into()),
        Command::Quit => ActionOutcome::Quit("Goodbye!".into()),
        Command::Look => ActionOutcome::Message(
            world.room(state.current_room).map(describe_room)
                .unwrap_or_else(|| "The current room could not be found.".into()),
        ),
        Command::Move(direction) => {
            let Some(room) = world.room(state.current_room) else {
                return ActionOutcome::Message("The current room could not be found.".into());
            };
            let Some((_, destination)) = room.exits.iter().find(|(exit, _)| *exit == direction) else {
                return ActionOutcome::Message(format!("You cannot go {} from here.", direction.label()));
            };
            if let Some(next_room) = world.room(*destination) {
                state.current_room = next_room.id;
                ActionOutcome::Message(describe_room(next_room))
            } else {
                ActionOutcome::Message("That exit leads to an unknown room.".into())
            }
        }
    }
}
