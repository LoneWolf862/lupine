# Lupine

A small, offline, story-independent interactive fiction engine written in Rust. **HOWLBOUND** is the tentative title of a future game using this engine; the sample rooms are strictly test fixtures.

## Status

**v0.2.0 — Objects and Inventory (candidate):** includes terminal navigation, a typed parser, portable/fixed objects, inspect/take/drop/inventory, object aliases, ambiguity feedback, and a generic stateful `use <item> on <target>` rule. This update must be compiled and tested locally before treating it as verified.

## Requirements

Rust toolchain with Cargo to **build** the project. Players of a future packaged release will not need Rust or Cargo.

## Run and test

```sh
cargo run
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Try it

```text
look
examine table
take brass key
inventory
go north
e
examine panel
use key on panel
west
south
drop key
look
help
quit
```

## Project layout

- `src/main.rs`: terminal I/O loop
- `src/lib.rs`: reusable engine module exports
- `src/parser.rs`: normalization and command grammar
- `src/commands.rs`: typed commands and directions
- `src/world.rs`: immutable test world
- `src/state.rs`: mutable player position, item locations, and interaction flags
- `src/engine.rs`: validate and execute commands
- `src/render.rs`: terminal text
- `tests/gameplay.rs`: navigation integration test
- `tests/objects.rs`: object and interaction integration tests
- `docs/`: requirements, architecture, handoff notes, roadmap

## CSE 310

Code exercises immutable `World`, mutable `GameState`, an input `loop`, borrowing with `&World`/`&mut GameState`, `Vec` for rooms and exits, `match` for command dispatch, and `struct`/`enum`/`Result` types. The course's **instructor-provided README template must replace or be integrated with this temporary README before submission**.

## Next milestone

Verify Windows and CachyOS builds and tests, review ambiguous object resolution and parser usability, then prepare instructor-template documentation and offline releases. See `docs/ROADMAP.md`.

## License

MIT; see `LICENSE`.
