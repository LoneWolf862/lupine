# Lupine

A small, offline, story-independent interactive fiction engine written in Rust. **HOWLBOUND** is the tentative title of a future game using this engine; the sample rooms are strictly test fixtures.

## Status

**v0.1.0 foundation:** terminal loop, typed commands, help, look, navigation, safe input handling, and automated parser/navigation tests. Inventory, objects, puzzle interactions, and the enhanced parser are planned but **not yet implemented**.

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
go north
e
west
south
help
quit
```

## Project layout

- `src/main.rs`: terminal I/O loop
- `src/lib.rs`: reusable engine module exports
- `src/parser.rs`: normalization and command grammar
- `src/commands.rs`: typed commands and directions
- `src/world.rs`: immutable test world
- `src/state.rs`: mutable player location
- `src/engine.rs`: validate and execute commands
- `src/render.rs`: terminal text
- `tests/gameplay.rs`: integration test
- `docs/`: requirements, architecture, handoff notes, roadmap

## CSE 310

Code exercises immutable `World`, mutable `GameState`, an input `loop`, borrowing with `&World`/`&mut GameState`, `Vec` for rooms and exits, `match` for command dispatch, and `struct`/`enum`/`Result` types. The course's **instructor-provided README template must replace or be integrated with this temporary README before submission**.

## Next milestone

Add portable/fixed objects, examine/take/drop/inventory, entity resolution, alias handling, and a stateful generic interaction. See `docs/ROADMAP.md`.

## License

MIT; see `LICENSE`.
