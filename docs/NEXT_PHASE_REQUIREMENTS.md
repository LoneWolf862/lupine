# Lupine — Next Development Phase Requirements

This document records the original creator/parser expansion plan. Those phases
are implemented. Current development is 0.11.0-dev.12; use the
[documentation index](README.md) and [playthrough review](PLAYTHROUGH_REVIEW.md)
for present capabilities and remaining acceptance. The historical baseline and
future-tense milestones below are preserved as the original plan.

## Historical pre-release policy (2026-11-18)

Lupine is unreleased and pre-1.0, with no external users, published games or production deployments requiring development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Phase 1 relocates existing character fields/defaults/calculations without redesigning progression, adds display terminology, and updates creator guidance. No legacy `character.toml` loader. No save-format change is needed. Full wizard and parser upgrades are later phases.

**Status:** Phase 1 merged. Phase 2A.1 beginner/advanced wizard refinement implemented on the existing PR #2 branch; verification recorded in CODEX_HANDOFF.md. The full Workstream B journey remains future work.
**Baseline:** Existing Lupine v0.8.4 Rust engine; v0.8.4-C described as a Windows verification candidate in the reviewed repository (November 17, 2026).  
**Scope:** Enhance the existing repository, not rebuild the engine or write the story for HOWLBOUND.

## Purpose

Lupine is a reusable, open-source, offline, terminal-based interactive fiction engine. This phase prioritizes usability for **game creators** and **players** over adding RPG features. Authors should be able to create games without writing Rust; the player should not need an internet connection, account, API, or LLM. HOWLBOUND is a tentative future story, kept independent of the engine.

## Current features to retain

The current README identifies a TOML story loader and validator, `new` wizard, `add room`, `add item`, `add art`, player exploration/inventory/interactions, manual named `.rork` saves, and optional RPG features including progression, spells, combat, shops, and persistent encounters. Codex must verify precise implementations in source before modifying them. Earlier milestone notes in `AGENTS.md`, `ARCHITECTURE.md`, and `ROADMAP.md` include historical status, not an up-to-date feature inventory.

## Core product rules

1. **Terminal-only gameplay and creator wizard.** No graphical editor requirement.
2. **Offline player runtime.** No network calls or inference services needed at playtime; keep the executable reasonably small and portable (Windows 10 and Linux first).
3. **Story independence.** The same Lupine build runs unrelated adventures without game-specific Rust branches.
4. **Data roles:** `.toml` for story content; optional `.fig` for game-wide flavor/configuration (using TOML syntax); `.rork` for existing manual player saves. Do not reinterpret saves as config.
5. **No automatic saving.** Preserve the explicit `save`, `load`, and `saves` commands; no autosave, checkpoint save, or save-on-quit.
6. **Pre-release evolution.** Make justified API/format changes without obsolete-format loaders; migrate all bundled consumers. Preserve gameplay and default behavior. Narrative games need no FIG files; RPG games opt in with `config/character.fig`.
7. **Safe authoring.** No silent overwrites; guard paths, IDs and input; canceling or an error must not destroy user-created files.
8. **One underlying schema.** The wizard produces ordinary editable files that the same runtime/validator consumes; no hidden wizard-only logic.

## Workstream A — Game-wide `.fig` files

Create a configuration mechanism for engine-wide presentation and meaningful gameplay knobs, with defaults and validation. Candidate settings include display terms (spellbook, mana, currency), command aliases where safe, optional feature toggles, and progression scaling where the existing implementation supports it. Differentiate game-wide knobs from data for a particular item, spell, enemy, or room. Files are optional and hand-editable. See `CONFIG_FIG_SPEC.md`.

**Acceptance:** A narrative adventure with no config works; migrated RPG examples retain their behavior; a user can hand-edit an example FIG setting and see the documented effect; invalid settings produce understandable errors with file/key names; tests cover defaults and compatibility.

## Workstream B — Guided creator wizard

Extend the *existing* `new`/creator commands to help authors establish a project, make rooms, link rooms, create items, and make an introductory interaction/puzzle. Explain inputs without long lectures. After each step show the **actual serialized TOML or FIG output** (or clearly labeled excerpt generated by the same serializer) before writing it. This is passive TOML education: the author should gradually become confident editing generated files directly. Offer examples, defaults, `?` help, skip/cancel/back behavior where safe, and a short/quick mode. See `CREATOR_WIZARD_SPEC.md`.

**Acceptance:** A beginner makes a two-room adventure, links rooms, creates a portable item and a working example interaction, inspects generated files, validates, and plays. Explicitly permit one-way exits. No existing content is overwritten unexpectedly.

## Workstream C — Player parser upgrades

Build upon the tested deterministic parser. Keep command synonyms, filler-word handling, directions, multiword targets, and RPG commands. Improve contextual object matching, ambiguity clarification, helpful mistakes/typo suggestions, and clear errors. Keep normalization/parsing, object resolution, rule checking, execution, and rendering conceptually separate. Do not require or incorporate an LLM. See `PARSER_IMPROVEMENTS_SPEC.md`.

**Acceptance:** Natural supported variants resolve to the same action; ambiguous targets request disambiguation without a state change; a typo suggests the intended phrase; uncertain input cannot accidentally trigger dangerous/destructive actions; existing tests still pass.

## Workstream D — General cleanup

Audit existing implementation and tests. Identify repeated code, confusing interfaces, inconsistent diagnostics, stale current-state descriptions, and missing regression coverage. Apply small justified refactors, not an architectural rewrite. Do not modify `.rork` formats or unrelated RPG behavior merely as cleanup.

## Suggested staged delivery

- **0: Read-only audit:** inventory current features, gaps, risk, interfaces, and an ordered PR plan. No source edits until reviewed.
- **1: Config design/loader:** minimal FIG schema, default behavior, validation, tests; expand categories only as warranted.
- **2: Wizard:** reuse authoring validation/serialization, generate an actually playable example workflow.
- **3: Parser:** incremental improvements and tests, including RPG-command integration.
- **4: Cleanup/release review:** docs, regression tests, cross-platform checks.

Keep PRs independently reviewable and maintain a record of already implemented / candidate / verified status.

## Verification

Run on a real toolchain: `cargo fmt --all -- --check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`. Validate and play current sample games. Test wizard-generated games end-to-end. Smoke-test existing `.rork` slots **without overwriting them**. Windows and Linux test claims require respective verification; standalone distribution is a later v0.9 goal.

## CSE 310 Rust module

Professor has approved using work already done on Lupine **this semester** as a self-contained two-week sprint. Show evidence of mutable and immutable variables; loops; functions transferring ownership or borrowing references; a `Vec`; the `match` keyword; and at least one stretch feature (`struct`, `enum`, or `Option`/`Result` returned from a function). Keep an honest time log, public repository, course-template README, and demo plus code walkthrough video. Do not fabricate recorded hours or verification outcomes.
