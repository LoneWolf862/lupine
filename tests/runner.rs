use lupine::runner::run_with_io;
use lupine::world::World;
use std::io::{self, Cursor};

#[path = "../examples/fixture_world.rs"]
mod fixture_world;

#[test]
fn terminal_runner_executes_a_full_script() {
    let mut output = Vec::new();
    let commands = b"take key\nnorth\neast\nnorth\nuse key on panel\nnorth\nquit\n";
    run_with_io(
        fixture_world::fixture_world(),
        "Fixture Game",
        Cursor::new(commands),
        &mut output,
    )
    .expect("valid game should run");
    let text = String::from_utf8(output).expect("utf8 output");
    assert!(text.contains("Fixture Game"));
    assert!(text.contains("The northern passage is sealed."));
    assert!(text.contains("Test Vault"));
    assert!(text.contains("Goodbye!"));
}

#[test]
fn terminal_runner_rejects_invalid_world_before_any_output() {
    let world = World {
        actions: Vec::new(),
        rules: Default::default(),
        starting_room: "missing".into(),
        rooms: vec![],
        items: vec![],
        interactions: vec![],
        exit_rules: vec![],
    };
    let mut output = Vec::new();
    let error = run_with_io(world, "Bad Game", Cursor::new(b"quit\n"), &mut output)
        .expect_err("invalid game must fail");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(output.is_empty());
}

#[test]
fn terminal_runner_handles_end_of_input() {
    let mut output = Vec::new();
    run_with_io(
        fixture_world::fixture_world(),
        "Fixture Game",
        Cursor::new(b""),
        &mut output,
    )
    .expect("EOF must be handled");
    assert!(String::from_utf8(output).unwrap().contains("Goodbye!"));
}
