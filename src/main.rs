use std::io::{self, Write};

use lupine::engine::{execute_command, ActionOutcome};
use lupine::parser::{parse_command, ParseError};
use lupine::state::GameState;
use lupine::world::fixture_world;

fn main() -> io::Result<()> {
    let world = fixture_world();
    let mut state = GameState::new(&world);

    println!("LUPINE v0.1.0 — Interactive Fiction Engine");
    println!("Temporary test world. HOWLBOUND story not yet designed.");
    println!("Type 'help' for commands.\n");
    if let Some(room) = world.room(state.current_room) {
        println!("{}\n", lupine::render::describe_room(room));
    }

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            println!("\nGoodbye!");
            break;
        }
        match parse_command(&input) {
            Ok(command) => match execute_command(&world, &mut state, command) {
                ActionOutcome::Message(message) => println!("{message}\n"),
                ActionOutcome::Quit(message) => {
                    println!("{message}");
                    break;
                }
            },
            Err(ParseError::Empty) => continue,
            Err(ParseError::MissingDirection) => println!("Where would you like to go? Try 'go north'.\n"),
            Err(ParseError::Unknown(command)) => println!("I don't understand '{command}'. Type 'help' for commands.\n"),
        }
    }
    Ok(())
}
