# Lupine

**Lupine** is an open-source, lightweight Rust engine for offline, terminal-based interactive fiction. It is inspired by the *command-driven interaction style* of classic adventures such as Zork, not their copyrighted stories, rooms, puzzles, or prose.

**Lupine is an engine, not a particular game.** Anyone should ultimately be able to create their own adventure with it. **HOWLBOUND** is the tentative title for the first original game we will build to exercise and improve Lupine; its story has not been decided.

## Status — v0.2.0 verified on Windows

The following functionality has been implemented and tested on Windows 10:

- Offline terminal REPL; `help`, `quit`, `look`, and directional navigation.
- Deterministic parser supporting capitalization/spacing normalization, basic synonyms and object phrases.
- Objects with stable IDs, aliases, portable/fixed properties, and examine text.
- `take`, `drop`, `inventory`, and `examine`; ambiguous names do not silently select an object.
- A simple `use <item> on <target>` interaction that persists a state flag during play.
- A **temporary three-room fixture** for testing. The fixture is *not* HOWLBOUND content.

**Verified:** `cargo build`, `cargo test` (10 passing tests), interactive gameplay, `cargo fmt --all`, and `cargo clippy --all-targets -- -D warnings`. **Not yet verified:** CachyOS build, packaged offline releases, independent external story files, or nontechnical authoring.

## Build, run, and check

To develop from source, install the Rust toolchain and Cargo. Windows builds using the MSVC Rust target also require Microsoft C++ build tools. From the repository root:

```sh
cargo run
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

The eventual **player-facing release** will ship as a packaged platform-specific executable; players will not need Cargo or Rust. A release download is **not yet available**.

Try this fixture-world sequence:

```text
look
examine table
take table
take brass key
inventory
north
east
use key on panel
look
west
south
drop key
quit
```

## Repository map

| Path | Responsibility |
|---|---|
| `src/main.rs` | Current terminal app and fixture-world startup |
| `src/lib.rs` | Public Rust module exports |
| `src/parser.rs`, `src/commands.rs` | Normalize/parse typed commands |
| `src/world.rs` | Current world/item/interaction types and temporary fixture |
| `src/state.rs` | Mutable room, item-location, and flag state |
| `src/engine.rs` | Object resolution, validation, and state transitions |
| `src/render.rs` | Player-visible text |
| `tests/` | Navigation and object/interaction integration tests |
| `docs/` | Requirements, architecture, roadmap, authoring vision, handoff |

## For adventure creators

**Current reality:** Creating a custom world currently requires editing Rust definitions and rebuilding. Lupine does **not yet** include a story-file loader, visual editor, or no-code creation workflow.

**Final-release goal:** A person without a programming background can author, validate, test, and package a game without writing Rust or changing engine internals. We plan to support a human-readable story format and accessible tooling; a graphical **Lupine Studio** is a *candidate*, not a committed implementation. Read [Creator Vision](docs/CREATOR_VISION.md).

## Development direction

The next architectural milestone is to separate generic engine behavior from the temporary fixture and introduce reusable conditions, effects, and state-dependent descriptions in a form suitable for later data-driven content. See [Roadmap](docs/ROADMAP.md), [Architecture](docs/ARCHITECTURE.md), and [Requirements](docs/REQUIREMENTS.md).

## CSE 310 note

This project also supports a CSE 310 Rust module. It exercises mutable/immutable values, loops, borrowing, `Vec`, `match`, and structured types. **The instructor-provided README template must be obtained and incorporated before course submission.** This repository README is project documentation, not a substitute for that template.

## License

MIT — see [LICENSE](LICENSE).
