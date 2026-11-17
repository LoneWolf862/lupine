# Lupine Development Roadmap

**Versions here describe planned milestones, not guaranteed release dates.** Mark a milestone complete only after implementation and verification. HOWLBOUND is the tentative title of an original game used to exercise the reusable engine.

## v0.1.0 — Foundation — COMPLETE (Windows)

- [x] Cargo project, terminal loop, basic typed commands, `help`, `quit`, `look`, navigation
- [x] Three neutral fixture rooms and navigation test
- [x] Public repository `LoneWolf862/lupine` created
- [x] Windows build and four tests passed

## v0.2.0 — Objects and Inventory — COMPLETE (Windows)

- [x] Portable/fixed objects, stable IDs, item locations, inventory, take/drop/examine
- [x] Alias/multiword matching, synonym handling, ambiguity feedback
- [x] One simple `use` rule producing persistent state; integration tests
- [x] Windows `cargo build`, `cargo test` (10 passing), interactive run, formatting, strict Clippy
- [x] Pushed commit `6d58bdc` to `main`
- [ ] Independently verify the same build and tests on CachyOS/Linux

## v0.3.0 — Reusable Engine Architecture — NEXT

- [ ] Make the generic engine usable independently of the bundled test fixture
- [ ] Define stable, documented engine-facing world creation/initialization API
- [ ] Separate fixture world from core library logic (keep a runnable example)
- [ ] Design and implement a **small**, reusable conditions/effects model for interactions
- [ ] Implement conditional room/object descriptions reflecting world state
- [ ] Add tests for at least two distinct fixture worlds and for invalid/blocked state transitions
- [ ] Refine parser/resolver edge cases without pretending to accept arbitrary English
- [ ] Document how a Rust developer builds a basic adventure with the current API

**Not a v0.3 gate:** external story files, graphical authoring, general-purpose scripting language, or no-code creator workflow.

## v0.4.x — Integration and Developer Release Preparation — PLANNED

- [ ] Windows and CachyOS/Linux builds and automated tests
- [ ] Complete offline player-package test (no network, Cargo, or Rust on player machine)
- [ ] Developer-facing examples, API documentation, contribution workflow, and release packaging
- [ ] Confirm license and make introductory documentation approachable
- [ ] Incorporate the **instructor-provided CSE 310 README template**, demo/video, and accurate sprint/time documentation for course submission

## Post-MVP — Data-Driven Game Creation — PLANNED, VERSION UNASSIGNED

- [ ] Choose a human-readable local content format based on authoring usability
- [ ] Introduce content loading and schema/versioning while retaining the Rust API
- [ ] Add actionable content validation (IDs, missing references, rules, startup state)
- [ ] Provide a starter story, step-by-step tutorial, and basic playtest workflow
- [ ] Prototype HOWLBOUND using the same mechanisms available to outside creators

## Final Creator Release — REQUIRED BEFORE CALLING LUPINE COMPLETE

- [ ] A person without a programming background can create rooms, objects, and puzzles without editing Rust
- [ ] Creators can validate and playtest with understandable errors
- [ ] Creators can export an immediately playable offline game through an approachable workflow
- [ ] A beginner-friendly starter project and tutorial have been usability-tested
- [ ] Evaluate guided text authoring versus **possible** Lupine Studio graphical editor (decision open)

See [`CREATOR_VISION.md`](CREATOR_VISION.md) for explicit acceptance criteria. Save/load, advanced grammar, combat, and other story mechanics remain optional until justified by a real game.

## Refined milestones and locked design decisions

- **v0.5.0-A (current candidate):** Owned world and game-state text/IDs; update fixtures, sample game, tests, and documentation.
- **v0.5.0-B:** TOML game/room/item loader, mapping to the same `World` API (Rust-authored games remain supported).
- **v0.5.0-C:** Data-driven conditions/interactions, actionable format validation and multiple example stories.
- **v0.6.0:** Built-in **manual-only** `save`, `load`, `saves`; named per-game TOML-based `.rork` files (e.g. `forestcamp.howlbound.rork`); safety, validation and versioning. **No autosaving**.
- **Later:** Text-only creator templates, terminal commands, testing and standalone game packaging. No graphical studio planned.
