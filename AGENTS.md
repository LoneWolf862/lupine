# Lupine — contributor / coding-agent instructions

## Identity and product boundaries

- **Lupine** is a reusable, open-source, offline Rust **interactive fiction engine**, not a particular narrative.
- **HOWLBOUND** is the *tentative* first original adventure and a real-world test of Lupine. Do not invent or commit its setting, plot, characters, puzzles, or lore without the creator's direction.
- Preserve the distinction between **engine**, **example/test fixtures**, **story content**, and any **future authoring tools**.
- The long-term final release must enable **nontechnical authors** to create, validate, test, and package games without writing Rust. This is **not** a v0.3.0 or course-MVP gate. Read `docs/CREATOR_VISION.md`.

## Current pre-release policy (2026-11-18)

Lupine is pre-1.0. The published v0.9.0 standalone release remains unchanged; current development uses 0.11.0-dev.10. No known external games require development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Character/progression and terminology FIGs, the guided creator and Phase 3 parser are implemented. No legacy character.toml loader. Do not alter RORK format or game save compatibility versions for engine metadata changes.

## Architecture rules

- Deterministic, small, offline terminal *runtime*: no LLM, cloud calls, accounts, web server, or networking dependency at play time.
- Separate normalization/parsing, object resolution, rule validation, state mutation, and output formatting.
- Do not hardcode a particular game's puzzle or prose in the parser or generic engine. Represent eventual story mechanics using reusable data-oriented conditions/effects.
- Prefer stable identifiers and immutable content definitions distinct from mutable `GameState`.
- Preserve Windows 10 and CachyOS/Linux portability; avoid machine-specific paths, commit `Cargo.lock`, and do not commit `target/`.
- Choose the smallest maintainable implementation. Prefer the Rust standard library unless a dependency solves a concrete need.
- Content is loadable without Rust edits. Use the current FIG configuration specification and validate before play.

## Development workflow

- Engine expansion and stabilization through PR #43 are merged. Current approved work
  is status-documentation correction and the complete-adventure/platform acceptance
  plan, with automated verification and human Windows/CachyOS handoff; see
  docs/ACCEPTANCE_PLAN.md and docs/STABILIZATION.md. File-based TOML/FIG authoring is primary.
  Use isolated branches/worktrees, checkpoint pushes and draft PRs targeting main
  (or their immediate dependency when stacked). Do not merge or publish a release.

- Read `README.md`, `docs/REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, and `docs/CREATOR_VISION.md` before architectural changes.
- Work in small, reviewable increments. Add tests for parser ambiguity, missing objects, invalid worlds, and state transitions as features arrive.
- Before a milestone is marked verified, run `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets -- -D warnings` on an available Rust toolchain.
- Update documentation and record what is **implemented**, **proposed**, or **not verified**; do not claim features exist until tested.
- Phase 4 authorizes autonomous implementation/test/refinement. Use isolated worktrees under the user's Codex worktrees directory, one feature branch per major phase, checkpoint pushes and draft PRs targeting their immediate dependency. Never modify the owner's primary checkout, push to main, force-push published history, self-merge, or publish a release without human approval.
- Protect CSE 310 requirements and the instructor README-template obligation; do not invent time logs or submission evidence.

## Current creator and player boundaries

Creator work through Phase 2I (PRs #8–#19) is merged. Preserve Snapshot publication,
exact authored previews, stable IDs and explicit confirmation. Related-manager prerequisite
saves remain separate. Enemies, weapons, shops and currencies have guided editors;
currencies remain in game.toml. Pending enemies publish with room drafts. Do not rename
IDs or weapon associations through ordinary forms, rewrite saves or introduce autosaving.
Phase 4 T manages transaction bindings; M deliberately migrates supported IDs and weapon
associations through schema-aware previews and confirmed Snapshot publication. Preserve
RORK bytes and compatibility revisions; never promise automatic save compatibility.

Phase 3 startup, history, deterministic resolution and offline export are merged in
PRs #20–#24. Read docs/PHASE_3_REVIEW.md for their original verification.
No-argument startup is a menu; fixture explicitly runs the Rust test adventure. Preserve
redirected input and scriptable BufRead APIs. Suggestions/clarification never mutate state;
ordinary item use has priority over spell aliases. Exact enemy HP stays private.
Keep questions on completed output lines above the terminal editor's input prompt.
Bare save offers to overwrite the last successfully saved/loaded slot in this play session;
no asks for a name, and a fresh session asks for a name immediately. Explicit save NAME
authorizes writing that slot without another question. Keep name/collision validation.
Suggestions stay advisory;
missing weapon selection and cancellation must not spend a turn or mutate game state.

Export requires a new destination outside the adventure and explicit confirmation. Never
include RORK saves, silently download runtimes or relabel a binary for another OS. Preserve
target/version/digest checks and the incomplete-export marker. Keep dependency notices with
exported engines; regenerate them from Cargo.lock after dependency changes. Version metadata
comes from Cargo; keep the published v0.9.0 tag immutable. Official release publication is
separate from development CI artifacts.

Phase 4 is merged through PR #30; read docs/PHASE_4_REVIEW.md for historical verification.
Recovery extends .lupine-room-edit with hashed baseline/proposal evidence. Never overwrite
external edits or guess through missing/corrupt/older records. Export supports stored ZIPs
and folders, private staging and an incomplete publication marker. Cleanup may delete only
a recorded, unchanged inventory. Preserve path/size limits, runtime checks and notices.

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
## Historical approved phase — self-documenting FIG files (Phase 2C.2)

Read `docs/SELF_DOCUMENTING_CONFIG_SPEC.md` before changing new-adventure scaffolding or FIG activation. Every **new** game should expose implemented configuration defaults in commented FIG files, while narrative-only behavior remains the default. `config/character.fig` currently enables RPG by its mere presence; change this deliberately via an explicit opt-in `enabled` field and test existing RPG games/migrations. Do not create unsupported gameplay/progression/combat/parser FIG files. Keep content TOML, configuration FIG, manual player RORK saves strictly separate. Every Codex task uses an isolated feature branch, frequent checkpoint pushes, a reviewable PR, and no self-merge or direct pushes to `main`.

## Merged engine expansion boundaries

Phases 1-9 and follow-ups are merged through PR #42. Read
`docs/ENGINE_EXPANSION.md` for historical verification and `docs/STABILIZATION.md`
for current work. The original review worktrees are retired. Current RORK is
6 for added engine state; metadata/editor changes do not revise it. Recovery journal
v2 supports deletion and an explicitly recorded invalid baseline; never guess through
older, corrupt or externally changed evidence. Project Files F supplies complete-schema
structural/text editing, with exact previews and confirmed multi-file publication;
ordinary guided forms retain stable IDs. Room navigation does not share hidden drafts.
Keep runtime discovery bounded, offline and subject to target/version/digest checks.

NPC file organization: NPCs/*.toml now holds one NPC per file with a top-level id. Preserve exact NPCs directory casing and stable IDs independent of filenames. Root dialogue.toml is rejected, not loaded; it remains an explicit editor repair/deletion target. Keep definitions separate from RORK 6 runtime progress. See docs/DIALOGUE.md.
