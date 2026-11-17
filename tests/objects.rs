use lupine::commands::Command;
use lupine::engine::{execute_command, ActionOutcome};
use lupine::parser::parse_command;
use lupine::state::{GameState, ItemLocation};
use lupine::world::{Item, World};
#[path = "../examples/fixture_world.rs"]
mod fixture_world;
use fixture_world::fixture_world;

fn run(world: &World, state: &mut GameState, input: &str) -> String {
    let command = parse_command(input).unwrap();
    match execute_command(world, state, command) {
        ActionOutcome::Message(text) => text,
        ActionOutcome::Quit(_) => panic!("unexpected quit"),
    }
}

#[test]
fn take_examine_drop_and_return_to_room() {
    let world = fixture_world();
    let mut state = GameState::new(&world);
    assert!(run(&world, &mut state, "look").contains("brass key"));
    assert!(run(&world, &mut state, "examine brass key").contains("ordinary brass key"));
    assert!(run(&world, &mut state, "pick up the key").contains("You take"));
    assert_eq!(state.location("brass_key"), Some(ItemLocation::Inventory));
    assert!(run(&world, &mut state, "look").contains("Objects: stone table"));
    assert!(run(&world, &mut state, "inventory").contains("brass key"));
    run(&world, &mut state, "north");
    assert!(run(&world, &mut state, "drop key").contains("You drop"));
    assert_eq!(
        state.location("brass_key"),
        Some(ItemLocation::Room("corridor".into()))
    );
    run(&world, &mut state, "south");
    assert!(run(&world, &mut state, "inventory").contains("nothing"));
    assert!(run(&world, &mut state, "examine key").contains("don't see"));
    run(&world, &mut state, "north");
    assert!(run(&world, &mut state, "look").contains("brass key"));
}

#[test]
fn fixed_or_inaccessible_objects_cannot_move() {
    let world = fixture_world();
    let mut state = GameState::new(&world);
    assert!(run(&world, &mut state, "take table").contains("cannot be moved"));
    assert_eq!(
        state.location("stone_table"),
        Some(ItemLocation::Room("foyer".into()))
    );
    assert!(run(&world, &mut state, "drop key").contains("not carrying"));
    assert!(run(&world, &mut state, "examine panel").contains("don't see"));
}

#[test]
fn stateful_interaction_requires_item_and_persists() {
    let world = fixture_world();
    let mut state = GameState::new(&world);
    run(&world, &mut state, "north");
    run(&world, &mut state, "east");
    assert!(run(&world, &mut state, "use key on panel").contains("don't see"));
    run(&world, &mut state, "west");
    run(&world, &mut state, "south");
    run(&world, &mut state, "get brass key");
    run(&world, &mut state, "north");
    run(&world, &mut state, "east");
    assert!(run(&world, &mut state, "use the key on the panel").contains("indicator lights up"));
    assert!(state.flags.contains("panel_activated"));
    assert!(run(&world, &mut state, "use key on panel").contains("already been completed"));
}

#[test]
fn ambiguous_aliases_do_not_mutate_state() {
    let mut world = fixture_world();
    world.items.push(Item {
        id: "iron_key".into(),
        name: "iron key".into(),
        aliases: vec!["key".into(), "iron key".into()],
        description: "A second key.".into(),
        descriptions: vec![],
        portable: true,
        starting_room: "foyer".into(),
    });
    let mut state = GameState::new(&world);
    let before = state.clone();
    assert!(run(&world, &mut state, "take key").contains("brass key or iron key"));
    assert_eq!(state, before);
    assert!(run(&world, &mut state, "take iron key").contains("You take"));
    assert_eq!(state.location("iron_key"), Some(ItemLocation::Inventory));
}

#[test]
fn invalid_use_and_unknown_target_preserve_state() {
    let world = fixture_world();
    let mut state = GameState::new(&world);
    let before = state.clone();
    assert!(run(&world, &mut state, "take imaginary sword").contains("don't see"));
    assert_eq!(state, before);
    assert!(matches!(
        execute_command(&world, &mut state, Command::Inventory),
        ActionOutcome::Message(_)
    ));
    assert_eq!(state, before);
}
