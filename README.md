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

### v0.7.0-B candidate: room and item creation + dynamic artwork

Use `cargo run -- add room <story-directory> <room-id>` and `cargo run -- add item <story-directory> <item-id>` to generate editable TOML files. Use `art_file = "symbol.txt"` in a room to load ASCII/UTF-8 art from the game-local `art/` folder. Conditional `[[art_variants]]` can choose different text files based on flags or inventory. Run `cargo run -- play story_example` to test the locked/open door artwork. See [Room Art](docs/ROOM_ART.md) for full instructions. This update is pending local tests.


## Creator improvements (v0.7.0-C test candidate)

Run `lupine new` for a guided terminal setup, or use `lupine new <directory> <game-id> <title...>` for scripting. Use `lupine add art <directory> <id>` to generate a static ASCII `.txt` file or append `--dynamic` for two files driven by an optional flag variant. Copy the generated instructions into the room TOML; no automatic story rewriting. See [`docs/CREATOR_QUICKSTART.md`](docs/CREATOR_QUICKSTART.md) and [`docs/ROOM_ART.md`](docs/ROOM_ART.md). This is not a GUI, and the engine never autosaves.

### Optional RPG progression (v0.8.0-A candidate)

Add `character.toml` to a story to enable player HP, MP, AP, level, and XP. Enter `stats` during play. These character values are stored in manually triggered RORK saves; there is **no autosaving**. Enemies, weapons, spells and ASCII enemy health bars are planned for later v0.8 stages. Refer to `docs/CHARACTER_PROGRESS.md` and `docs/COMBAT_RPG_SPEC.md`.

## v0.8.0-B RPG preview
With `character.toml`, creators may define weapons in `weapons/*.toml` and abilities in `spells/*.toml`. Commands: `weapons`, `learn <scroll>`, `spellbook`, `cast <ability> [on self]`. Offensive attacks require the v0.8.0-C combat system. See `docs/V0_8_0_B_UPDATE.md`.

### v0.8.0-C1 encounter preview (candidate)

Room `[encounters]` tables and `enemies/*.toml` declarations support weighted groups and hidden-health ASCII bars; see `docs/V0_8_0_C1_ENCOUNTERS.md`. C1 encounters are **visual, transient previews**, not combat and not yet persisted in RORK.
