//! Standard terminal interface for games that use Lupine.
//!
//! Game authors can call [`run`] to launch a terminal game without implementing
//! command parsing, a read/evaluate/print loop, or error formatting themselves.
use std::io::{self, BufRead, Write};

use crate::engine::{execute_command, ActionOutcome};
use crate::parser::{parse_command, ParseError};
use crate::render::describe_room;
use crate::state::GameState;
use crate::world::World;

/// Run a validated world using the process terminal. All gameplay is offline.
pub fn run(world: World, title: &str) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_with_io(world, title, stdin.lock(), stdout.lock())
}

/// Run against supplied input/output streams, enabling scripted tests and
/// alternative terminal hosts. Invalid worlds are rejected before play begins.
pub fn run_with_io<R: BufRead, W: Write>(
    world: World,
    title: &str,
    mut input: R,
    mut output: W,
) -> io::Result<()> {
    if let Err(errors) = world.validate() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            errors.join("\n"),
        ));
    }
    let mut state = GameState::new(&world);
    writeln!(output, "{title}")?;
    writeln!(output, "Powered by Lupine. Type 'help' for commands.\n")?;
    if let Some(room) = world.room(state.current_room) {
        writeln!(output, "{}\n", describe_room(&world, &state, room))?;
    }
    loop {
        write!(output, "> ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            writeln!(output, "\nGoodbye!")?;
            break;
        }
        match parse_command(&line) {
            Ok(command) => match execute_command(&world, &mut state, command) {
                ActionOutcome::Message(message) => writeln!(output, "{message}\n")?,
                ActionOutcome::Quit(message) => {
                    writeln!(output, "{message}")?;
                    break;
                }
            },
            Err(ParseError::Empty) => continue,
            Err(ParseError::MissingDirection) => {
                writeln!(output, "Where would you like to go? Try 'go north'.\n")?;
            }
            Err(ParseError::MissingTarget(action)) => {
                writeln!(
                    output,
                    "Please specify a target. Try 'help' for the syntax of {action}.\n"
                )?;
            }
            Err(ParseError::Unknown(command)) => {
                writeln!(
                    output,
                    "I don't understand '{command}'. Type 'help' for commands.\n"
                )?;
            }
        }
    }
    Ok(())
}
