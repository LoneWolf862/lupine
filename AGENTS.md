# Lupine — contributor / coding-agent instructions

## Identity and product boundaries

- **Lupine** is a reusable, open-source, offline Rust **interactive fiction engine**, not a particular narrative.
- **HOWLBOUND** is the *tentative* first original adventure and a real-world test of Lupine. Do not invent or commit its setting, plot, characters, puzzles, or lore without the creator's direction.
- Preserve the distinction between **engine**, **example/test fixtures**, **story content**, and any **future authoring tools**.
- The long-term final release must enable **nontechnical authors** to create, validate, test, and package games without writing Rust. This is **not** a v0.3.0 or course-MVP gate. Read `docs/CREATOR_VISION.md`.

## Current pre-release policy (2026-11-18)

Lupine is unreleased and pre-1.0, with no external users, published games or production deployments requiring development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Phase 1 relocates existing character fields/defaults/calculations without redesigning progression, adds display terminology, and updates creator guidance. No legacy `character.toml` loader. No save-format change is needed. Full wizard and parser upgrades are later phases.

## Architecture rules

- Deterministic, small, offline terminal *runtime*: no LLM, cloud calls, accounts, web server, or networking dependency at play time.
- Separate normalization/parsing, object resolution, rule validation, state mutation, and output formatting.
- Do not hardcode a particular game's puzzle or prose in the parser or generic engine. Represent eventual story mechanics using reusable data-oriented conditions/effects.
- Prefer stable identifiers and immutable content definitions distinct from mutable `GameState`.
- Preserve Windows 10 and CachyOS/Linux portability; avoid machine-specific paths, commit `Cargo.lock`, and do not commit `target/`.
- Choose the smallest maintainable implementation. Prefer the Rust standard library unless a dependency solves a concrete need.
- Content is loadable without Rust edits. Use the current FIG configuration specification and validate before play.

## Development workflow

- Read `README.md`, `docs/REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, and `docs/CREATOR_VISION.md` before architectural changes.
- Work in small, reviewable increments. Add tests for parser ambiguity, missing objects, invalid worlds, and state transitions as features arrive.
- Before a milestone is marked verified, run `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets -- -D warnings` on an available Rust toolchain.
- Update documentation and record what is **implemented**, **proposed**, or **not verified**; do not claim features exist until tested.
- The user prefers collaborative file/patch/ZIP updates and reviews changes before implementation; don't assume Codex automation is the default.
- Protect CSE 310 requirements and the instructor README-template obligation; do not invent time logs or submission evidence.

## Current creator implementation candidate (2026-11-18)

Phases 2D-A through 2F are implemented on the stacked draft PRs #8�#13, not merged. See docs/CREATOR_COMPLETION_REVIEW.md and docs/CREATOR_QUICKSTART.md for actual capabilities and verification. Preserve shared Snapshot transactions for all editors, stable existing IDs, exact previews and explicit confirmation. Use current schemas, preserve unedited TOML/FIG properties/comments, never autosave player progress. Conditional descriptions/variants are first-match; table source positions must agree with edited order. Item teaching links require portable items. New RPG enablement remains explicit. Enemy/weapon/shop definition editors, ID migrations and automated recovery remain separate work. Every task still requires an isolated branch, checkpoint pushes, reviewable PR and no self-merge.

## Historical milestones -- not current instructions

The following sections record prior development stages. Their compatibility requirements and feature-status claims are superseded by the current policy above and current source.

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

## Historical handoff — v0.8.4-C (2026-11-17 candidate)

Prior sections are historical milestone instructions, not the current feature inventory. v0.8.4-A and v0.8.4-B were verified by the user on Windows. The v0.8.4-C candidate consolidates defeat text, moves the test module to end-of-file for Clippy, updates documentation and Cargo version, and adds sample-loading regression coverage. Before marking C released, verify with `cargo fmt --all && cargo test && cargo clippy --all-targets -- -D warnings` and interactive play on the user's Windows setup. **Do not change save format or game save compatibility versions just to change the Cargo crate version.** Never autosave. Target v0.9 for standalone Windows/Linux packaging, and do not claim Linux native testing before actually running it.

<!-- lupine-phase-2c2-spec -->
## Approved next phase — self-documenting FIG files (Phase 2C.2)

Read `docs/SELF_DOCUMENTING_CONFIG_SPEC.md` before changing new-adventure scaffolding or FIG activation. Every **new** game should expose implemented configuration defaults in commented FIG files, while narrative-only behavior remains the default. `config/character.fig` currently enables RPG by its mere presence; change this deliberately via an explicit opt-in `enabled` field and test existing RPG games/migrations. Do not create unsupported gameplay/progression/combat/parser FIG files. Keep content TOML, configuration FIG, manual player RORK saves strictly separate. Every Codex task uses an isolated feature branch, frequent checkpoint pushes, a reviewable PR, and no self-merge or direct pushes to `main`.
