# Lupine v0.4.0 — Independent Game Support

This is an **unverified development candidate** until built and tested on
Windows and CachyOS. It is based on the confirmed v0.3.0 source layout.

## Changes

- Public `lupine::run(World, title)` entry point for standalone games.
- `runner::run_with_io` allows scripted terminal tests using injected streams.
- Lupine's built-in executable becomes a thin wrapper around the generic runner.
- `samples/independent_game` is a separate Cargo package with its own world
  and Rust program; no parser or engine changes are required.
- Engine rejects invalid worlds before displaying game content.
- `docs/EXTERNAL_GAME.md` explains local dependency setup.
- No external crates or runtime network requirements were added.

## Verify on Windows

```cmd
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --manifest-path samples/independent_game/Cargo.toml
```

The separate game's Cargo manifest must be checked independently:

```cmd
cargo test --manifest-path samples/independent_game/Cargo.toml
```

## Manual script

`north` (blocked) -> `take token` -> `use token on door` -> `north` -> `quit`.

Do not commit this milestone until its builds, tests, and manual session pass.
