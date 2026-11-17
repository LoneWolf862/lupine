# Lupine

Lupine is a lightweight, offline, terminal-only Rust interactive fiction engine inspired by classic command-based adventures. The engine is reusable and story-independent. HOWLBOUND is a tentative future game title, not engine content.

## Current state

- v0.5: Rust-owned worlds and external TOML-authored adventures, with validation, beginner templates and contextual errors.
- v0.6: Manual-only RORK (`.rork`, TOML) saves; `save`, `load`, and `saves` are player commands. No autosaving, no checkpoint saving, and no save-on-quit. Game-specific compatibility revision checks and staged writes protect saves from ordinary write failures.
- v0.7.0-A candidate: text-only `new` project generator. Requires local validation before publication.
- Packaging, a creator-friendly distribution workflow, and native Linux verification are still planned, not complete.

## Getting started (source checkout)

Install a Rust toolchain and open a terminal in the repository root. Windows CMD examples:

```cmd
cargo run -- help
cargo run -- new my_first_adventure my_first_adventure My First Adventure
cargo run -- validate my_first_adventure
cargo run -- play my_first_adventure
```

The `new` command creates an editable one-room project, safely refusing to overwrite an existing directory. Edit `my_first_adventure\rooms\start.toml`, then re-run `play`; no source-code changes or recompilation are required. Avoid committing your generated local test folder unless you intend to publish that adventure.

Other examples:

```cmd
cargo run
cargo run -- play story_example
cargo run -- validate templates/starter
cargo run --manifest-path samples/independent_game/Cargo.toml
```

In-game commands include `look`, `take`, `inventory`, `use`, `save <name>`, `load <name>`, `saves` and `quit`. Shell/Cargo commands must be entered outside the game.

### Developer checks

```cmd
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
```

## Authoring and project layout

A TOML adventure contains `game.toml`, a `rooms/` folder, an `items/` folder, and optional `interactions.toml`. Each room and item uses its own TOML file. See [Creating a Game](docs/CREATING_A_GAME.md) and [Story Format](docs/STORY_FORMAT_PROPOSAL.md). You can also copy `templates/starter/`.

RORK save files are stored in per-game user-data directories, separate from game installation paths; see [RORK save format](docs/SAVE_FORMAT.md). Saves are always player-initiated.

The engine code is in `src/`; game/engine tests in `tests/`; the neutral Rust fixture is in `examples/`; an independent Rust client is in `samples/independent_game/`.

## Roadmap and course work

See [Development roadmap](docs/ROADMAP.md), [Requirements](docs/REQUIREMENTS.md), and [Creator vision](docs/CREATOR_VISION.md). Lupine's final authoring experience is text-based (no graphical editor planned). The CSE 310 instructor-provided README template, video, and accurate sprint report remain separate course obligations.

## License

MIT — see [LICENSE](LICENSE).
