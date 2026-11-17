# Lupine — contributor / coding-agent instructions

## Identity and product boundaries

- **Lupine** is a reusable, open-source, offline Rust **interactive fiction engine**, not a particular narrative.
- **HOWLBOUND** is the *tentative* first original adventure and a real-world test of Lupine. Do not invent or commit its setting, plot, characters, puzzles, or lore without the creator's direction.
- Preserve the distinction between **engine**, **example/test fixtures**, **story content**, and any **future authoring tools**.
- The long-term final release must enable **nontechnical authors** to create, validate, test, and package games without writing Rust. This is **not** a v0.3.0 or course-MVP gate. Read `docs/CREATOR_VISION.md`.

## Architecture rules

- Deterministic, small, offline terminal *runtime*: no LLM, cloud calls, accounts, web server, or networking dependency at play time.
- Separate normalization/parsing, object resolution, rule validation, state mutation, and output formatting.
- Do not hardcode a particular game's puzzle or prose in the parser or generic engine. Represent eventual story mechanics using reusable data-oriented conditions/effects.
- Prefer stable identifiers and immutable content definitions distinct from mutable `GameState`.
- Preserve Windows 10 and CachyOS/Linux portability; avoid machine-specific paths, commit `Cargo.lock`, and do not commit `target/`.
- Choose the smallest maintainable implementation. Prefer the Rust standard library unless a dependency solves a concrete need.
- Future game content should be loadable without editing engine Rust code. **The version-1 TOML schema is approved; the v0.5.0-B loader requires validation before marking it released.**

## Development workflow

- Read `README.md`, `docs/REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, and `docs/CREATOR_VISION.md` before architectural changes.
- Work in small, reviewable increments. Add tests for parser ambiguity, missing objects, invalid worlds, and state transitions as features arrive.
- Before a milestone is marked verified, run `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets -- -D warnings` on an available Rust toolchain.
- Update documentation and record what is **implemented**, **proposed**, or **not verified**; do not claim features exist until tested.
- The user prefers collaborative file/patch/ZIP updates and reviews changes before implementation; don't assume Codex automation is the default.
- Protect CSE 310 requirements and the instructor README-template obligation; do not invent time logs or submission evidence.

## Scope control

- v0.5.0-A is verified and pushed; focus on v0.5.0-B TOML loading, then strengthen validation in v0.5.0-C.
- Preserve the Rust-defined engine API and tests. Avoid story-specific parser branches.
- `save`, `load`, and `saves` and `.rork` are planned for v0.6.0 and **not yet implemented**.
- No graphical creator interface, autosave, network features, combat, or broad scripting language.

## Decisions locked in after v0.4.0

- v0.5.0-A owns world/state strings in preparation for TOML loading; migrate existing sample worlds with `.into()` and keep the public engine story-independent.
- The creator interface is text-based (TOML templates and terminal utilities), **not** a GUI.
- Engine-level manual `save`, `load`, and `saves` will use TOML-based `.rork` saves. No autosaving under any circumstances.
- Protect compatibility and avoid claiming a save loader or TOML story loader exists before implemented and tested.

## v0.5.0-C notes

- TOML story loading (`play PATH`) was verified in v0.5.0-B, commit `655a5df`.
- `validate PATH` and the creator template are v0.5.0-C candidates, not yet user-verified.
- Keep validation errors specific to file paths and identifiers; do not alter content automatically.

## v0.6.0-A RORK boundaries

- `src/save.rs` serializes and validates state; it does not write files or run the terminal save commands yet.
- Saving must **never** run automatically. Gameplay `save`, `load`, `saves` are reserved for a later patch; maintain this distinction.
- Validate the entire save before replacing the active state.

## v0.6.0-B implementation notes

- The terminal runner intercepts reserved save verbs; never implement them in story-specific rules.
- Explicit game IDs (`run_with_game_id`) should be used for stable per-game save paths.
- Preserve **no autosaving**, including on quit, EOF, command execution or game loading.
- v0.6.0-B filesystem overwrite is not atomic; Stage C must harden this before calling RORK production-ready.

## v0.6.0-C RORK reliability

- Preserve manual-only `save`/`load`/`saves`, `forestcamp.<game-id>.rork`, compatibility with Stage B files that omit the content revision (defaults to 1), and the existing 41 tests.
- `save_compatibility_version` is independent of save serialization `save_format_version`. Mismatch is a load error; never partially alter current state.
- Do not truncate existing RORK save files; stage, sync, and publish. No implicit automatic saves.
- New stage is a candidate until `cargo fmt`, `cargo test`, and strict Clippy pass on the target platform.


## v0.7.0-B candidate: optional room art

- Prefer plain `art/*.txt` artwork in game folders, referenced from rooms with `art_file` (relative to art/).
- Dynamic `[[art_variants]]` use the existing condition vocabulary, with first-match priority and default artwork fallback.
- Do not treat artwork as mutable or write it into RORK; RORK already saves the flags and inventory that control art selection.
- Enforce relative, confined art paths; reject terminal control characters and overly large art.
- Creating room/item templates must never overwrite existing content or leave the game folder.

## RPG roadmap

Build an optional TOML-configured RPG system, detailed in `docs/COMBAT_RPG_SPEC.md`. Keep exact enemy HP PRIVATE by default; show approximate ASCII health bars, not numeric enemy HP. A future explicitly learned inspection ability (Arcane Eye) may reveal exact values. Never autosave. Progress through v0.8-A character foundations, B weapons/abilities, C encounters/combat, D RORK integration.

- v0.8.0-B: RPG abilities in `src/rpg_content.rs`; RORK includes `learned_abilities`; `ItemLocation::Consumed` for one-use teaching scrolls. Never autosave. Do not imply damage spells currently attack enemies.
