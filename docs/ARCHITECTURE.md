# Lupine Architecture — current implementation and planned extensions

**Status:** v0.5.0-A verified on Windows; v0.5.0-B TOML story loading is a testing candidate.

Lupine is an offline, deterministic, story-independent Rust terminal engine. It is not an LLM or Zork remake. The runtime remains text-only.

## Runtime flow

```text
Rust-authored World -----------+
                                +--> validated World --> GameState --> parser --> rules/effects --> renderer --> terminal
TOML game/rooms/items/rules --+                                  ^
           |                                                     |
      story_loader.rs (v0.5.0-B candidate)                       +-- typed player commands
```

- `src/parser.rs`, `src/commands.rs`: normalize input and parse known grammar into typed commands.
- `src/world.rs`: immutable `World`, `Room`, `Item`, `Interaction`, `ExitRule`, conditions and effects. Content strings/IDs are owned (`String`) and may be created at runtime.
- `src/state.rs`: mutable `GameState` with room ID, item locations/inventory and flags.
- `src/engine.rs`: resolve visible objects, verify preconditions, mutate state, and return outcome text.
- `src/render.rs`: state-dependent room descriptions and help output.
- `src/runner.rs`: reusable standard terminal loop. Rust game developers call `lupine::run(world, title)`.
- `src/story_loader.rs` (v0.5.0-B candidate): parse versioned files via `serde`/`toml` DTOs, convert into `World`, validate before starting. The same engine and runner are used for both content sources.
- `src/main.rs`: fixture launch (`cargo run`) or TOML story launch (`cargo run -- play story_example`).

## File format design

The approved version-1 proposal uses `game.toml`, `rooms/*.toml`, `items/*.toml`, and optional `interactions.toml`. See `STORY_FORMAT_PROPOSAL.md` for fields and examples. A story is *data*, never executable code. Keep schema DTOs distinct from core world structs.

## Creator workflow

Creators edit TOML templates in any editor. Future helper CLI commands may generate templates, validate, test and package games. No graphical authoring application is planned. Players need no Rust toolchain or network once an appropriate binary and local story content are packaged.

## Future save/load subsystem — not implemented

`save`, `load`, `saves` will be reserved engine commands. Manual-only player saves will be TOML-encoded `.rork` files with per-game identifiers and safe validation before state replacement. Save only mutable game state and compatibility metadata, never duplicate immutable world content. **No autosaving**. See `SAVE_FORMAT.md`.

## Quality expectations

Keep both Rust-defined and file-defined games functioning; add automated tests for malformed TOML, missing/duplicate references, invalid directions and end-to-end gameplay. Confirm Windows and Linux separately; do not infer cross-platform testing from passing Windows tests.
