# Lupine — Current Project Handoff

**Updated:** 2026-11-16  
**Repository:** `LoneWolf862/lupine` (public)  
**Engine:** Lupine (Rust, Cargo)  
**First game:** HOWLBOUND (*tentative name; story not yet designed*)

## What is established

Lupine is a small, deterministic, offline, terminal-based **reusable interactive fiction engine**. It draws on the command-input style of old parser adventures, but no Zork rooms, prose, or puzzles. It should eventually be usable by third parties to build their own original adventures.

A **final-release** requirement, **not an MVP requirement**, is that people with no technical background can make, validate, test, and package games without writing Rust. See `CREATOR_VISION.md`.

## Verified implementation as of v0.2.0

- Windows 10 toolchain installed and native build successful.
- Terminal REPL, navigation, look, help, quit, inventory, examine, take/drop, aliases/synonyms, ambiguity handling, and a simple stateful `use` interaction.
- `cargo test` passed **10 tests**; interactive playthrough passed; formatting and strict Clippy passed.
- Changes committed and pushed on `main` (`6d58bdc`); working tree reported clean.
- **CachyOS/Linux has not yet been verified.** Packaged releases, a data-driven story loader, authoring tools, and HOWLBOUND's narrative are **not implemented**.

## Immediate next task

> Build **Lupine v0.3.0** as a modest architectural improvement, not a new story. Begin by reviewing `docs/REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/CREATOR_VISION.md`, and current modules. Design a clean engine API independent of `fixture_world`, a small generic conditions/effects model, and state-dependent descriptions. Implement incrementally with tests, including more than one independent example world. Preserve existing commands and Windows/Linux compatibility. Do **not** invent HOWLBOUND lore or rush into a full authoring DSL or graphical editor.

## Collaboration and Git workflow

- The user develops at home on Windows 10 and away on a CachyOS laptop. Keep one shared GitHub repository and separate local clones; pull before new work and push completed commits before switching machines.
- User prefers reviewing supplied individual files, patches, or incremental ZIP archives and reporting test output. Codex may be used later but is not the default process.
- Run `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets -- -D warnings` before committing code updates.
- Documents must distinguish **verified behavior**, **next steps**, and **final product aspirations**.

## Academic deliverables

CSE 310 requires mutable/immutable variables, loops, ownership or borrowing, a real `Vec`, `match`, and a stretch feature; project structure naturally supports these. A public repo, **instructor-provided README template**, demonstration video, and accurate sprint/time log are also required. The professor permits eligible semester work in the approved two-week sprint. Do not fabricate dates or hours.

## Project boundaries

No final story decisions, internet-required runtime, large language model, game-specific puzzle code in core engine, or unapproved scope expansion. The authoring tool's interface and story-file format remain open design decisions.
