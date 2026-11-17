# Lupine

**Lupine** is an open-source, lightweight Rust engine for offline terminal-based interactive fiction. Its parser-driven interface is inspired by classic adventure games such as Zork; game content and mechanics are original. **HOWLBOUND** is the tentative title of a separate future game, not the built-in fixture.

## Development status

- **v0.5.0-A verified on Windows:** owned world/state data, reusable conditions and effects, state-dependent descriptions, independent Rust-authored games and reusable terminal runner. 18 tests passed, interactive testing passed, strict Clippy passed. Commit `97b9476`.
- **v0.5.0-B verified on Windows:** versioned TOML story loading, example adventure, 24 passing tests, strict Clippy and interactive verification; commit `655a5df`.
- **v0.5.0-C candidate — requires local verification:** `validate` command, improved diagnostics, template and creator guide.
- **Not implemented yet:** `save`, `load`, `saves`, `.rork` files, template generator, creator packaging, and graphical interfaces (none planned). Linux testing and packaging remain outstanding.

## Development commands

Install Rust/Cargo to develop from source. On Windows, the MSVC Rust toolchain may require Microsoft C++ build tools. From the repository root:

```sh
cargo run
cargo run -- play story_example
cargo run -- validate story_example
cargo run --manifest-path samples/independent_game/Cargo.toml
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
```

The first command launches the existing Rust fixture; `cargo run -- play story_example` launches a TOML-authored game from disk. **In-game commands are different from shell commands**; exit the adventure (`quit`) before launching a new Cargo process.

Try the file-authored example:

```text
north
look
take key
use key on door
look
north
quit
```

The north passage is locked until the key unlocks the door. You can change the room prose in `story_example/rooms/foyer.toml` and relaunch without changing Rust source.

## Authoring your own adventure

Copy `story_example/` to another directory. Edit `game.toml`, add room files under `rooms/`, and item files under `items/`; define optional interactions and exit rules in `interactions.toml`. Launch with `cargo run -- play <directory>` during development, or a compiled binary with `lupine play <directory>`.

Start with [`docs/CREATING_A_GAME.md`](docs/CREATING_A_GAME.md) and copy `templates/starter/`. Check errors without launching with `cargo run -- validate <directory>`.

The versioned TOML format is specified in [`docs/STORY_FORMAT_PROPOSAL.md`](docs/STORY_FORMAT_PROPOSAL.md). See [`docs/CREATOR_VISION.md`](docs/CREATOR_VISION.md) for the final goal of easy text-only authoring without programming knowledge or requiring Cargo for players.

## Repository layout

| Path | Responsibility |
|---|---|
| `src/parser.rs`, `src/commands.rs` | Deterministic player commands |
| `src/world.rs`, `src/state.rs` | Content definitions and mutable play state |
| `src/engine.rs`, `src/render.rs` | Rules, transitions, and narration |
| `src/runner.rs` | Shared terminal runtime |
| `src/story_loader.rs` | TOML story parser and validation (v0.5.0-B candidate) |
| `examples/fixture_world.rs` | Built-in test world |
| `samples/independent_game/` | Separate Rust-authored sample game |
| `story_example/` | TOML-authored sample game |
| `templates/starter/` | Copyable beginner adventure |
| `tests/` | Unit and integration tests |
| `docs/` | Requirements, architecture, format specs and roadmap |

## Future saves: RORK

Manual `save`, `load`, and `saves` are **planned**, not yet available. Native saves will have the TOML-based `.rork` extension, with filenames such as `forestcamp.howlbound.rork`. **No autosaving**, by design. See [`docs/SAVE_FORMAT.md`](docs/SAVE_FORMAT.md).

## CSE 310

Lupine is also a Rust programming course project. The instructor-provided README template and actual video / sprint reporting remain required for course submission; this project README is not a substitute.

## License

MIT — see [LICENSE](LICENSE).
