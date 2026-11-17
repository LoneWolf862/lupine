use lupine::commands::{Command, Direction};
use lupine::engine::{execute_command, ActionOutcome};
use lupine::parser::parse_command;
use lupine::state::GameState;
#[path = "../examples/fixture_world.rs"]
mod fixture_world;
use fixture_world::fixture_world;

#[test]
fn walkthrough_navigation_and_blocked_exit() {
    let world = fixture_world();
    let mut state = GameState::new(&world);
    assert_eq!(state.current_room, "foyer");

    let north = parse_command("go north").unwrap();
    let outcome = execute_command(&world, &mut state, north);
    assert!(matches!(outcome, ActionOutcome::Message(text) if text.contains("Test Corridor")));
    assert_eq!(state.current_room, "corridor");

    execute_command(&world, &mut state, Command::Move(Direction::East));
    assert_eq!(state.current_room, "chamber");

    let blocked = execute_command(&world, &mut state, Command::Move(Direction::North));
    assert_eq!(
        blocked,
        ActionOutcome::Message("The northern passage is sealed.".to_string())
    );
    assert_eq!(state.current_room, "chamber");

    execute_command(&world, &mut state, Command::Move(Direction::West));
    execute_command(&world, &mut state, Command::Move(Direction::South));
    assert_eq!(state.current_room, "foyer");
}
