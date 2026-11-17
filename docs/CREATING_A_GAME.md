# Creating a game with Lupine (Rust-authored prototype, v0.3.0)

**Status:** Developer-oriented preview. Nontechnical authoring and story-file loading are *future goals*, not supported in this release.

Lupine is a library crate. The command parser, rule executor, state manager, and terminal rendering don't contain any story-specific items or rooms. The bundled *generic* example is in `examples/fixture_world.rs`, imported by the demo executable in `src/main.rs`.

## Current authoring API

1. Construct a `lupine::world::World` with a stable starting-room ID, room definitions, item definitions, and interactions.
2. Call `world.validate()` **before** starting. Treat every returned error as an authoring issue.
3. Initialize `GameState::new(&world)`; do not mutate the immutable `World` when the player acts.
4. Parse the player's text using `parser::parse_command` and pass the resulting typed `Command` into `engine::execute_command`.
5. Display the returned `ActionOutcome`. Call `render::describe_room` for the opening description.

See `examples/fixture_world.rs` for an actual complete world definition and `tests/reusability.rs` for a second independent world.

### Mechanics supported in v0.3.0

- A room lists exits as `(Direction, RoomId)` pairs.
- An `ExitRule` may require all `Condition` values to pass before traversing an exit.
- An `Interaction` is triggered by `use <item> on <target>` and may require all conditions before applying all effects.
- `Condition`: `HasItem`, `FlagSet`, `FlagUnset`.
- `Effect`: `SetFlag`, `ClearFlag`, `MoveItemToRoom`.
- `DescriptionVariant` overrides the default room/item description when all its conditions pass; the **first matching variant** wins.
- Items have aliases and a portability flag. Players may carry, inspect, take, and drop them.

All IDs currently use `&'static str` so they must be known at compile time. This is **intentional for the first engine API**; no TOML/JSON parser, authoring GUI, save/load, or arbitrary text-driven story loading exists yet.

### Limitations to keep in mind

- The supported interaction verb is `use` with `on`; there is no generic custom-verb registry yet.
- Conditions combine using AND. OR/NOT expression trees are not supported (other than the explicit `FlagUnset`).
- Conditions are evaluated before an interaction applies its effects. Author-defined effect lists should not rely on intermediate effects being validated.
- Exit rules govern traversal; `look` still lists the direction of a blocked exit.
- The validator checks IDs, links, duplicate IDs/directions/interactions, and references. It does not prove that every puzzle is solvable or that every room can be reached.
- The code currently uses `std` only, with no runtime networking.

## Next: nontechnical authoring

The eventual Creator Release will allow authoring and packaging adventures without Rust. The eventual file format, validator UX, and optional graphical Lupine Studio have **not** been chosen or implemented. See `docs/CREATOR_VISION.md`.
