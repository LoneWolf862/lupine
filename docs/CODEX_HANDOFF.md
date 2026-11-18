# Lupine — Codex Development Handoff

## Current pre-release policy (2026-11-18)

Lupine is unreleased and pre-1.0, with no external users, published games or production deployments requiring development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Phase 1 relocates existing character fields/defaults/calculations without redesigning progression, adds display terminology, and updates creator guidance. No legacy `character.toml` loader. No save-format change is needed. Full wizard and parser upgrades are later phases.

**Status:** Phase 1 implemented and automatically verified on Windows; pending user review. Wizard/parser expansion remains future work.

## Project identity and baseline

- Repository: `LoneWolf862/lupine`.
- **Existing** reusable offline terminal interactive fiction engine in Rust. **Not** a new Zork clone or a new scaffold.
- v0.8.4 shown in `README.md`; v0.8.4-C was awaiting final Windows verification in reviewed notes. Verify actual head before work.
- HOWLBOUND is a tentative first original game, not built-in engine content; its story hasn't been specified.
- Core and optional RPG features, save files, TOML authoring commands, and the guided `new` wizard already exist. Do not implement them a second time.

## Read before work

1. `README.md` and `AGENTS.md`.
2. `docs/NEXT_PHASE_REQUIREMENTS.md`.
3. `docs/CONFIG_FIG_SPEC.md`.
4. `docs/CREATOR_WIZARD_SPEC.md`.
5. `docs/PARSER_IMPROVEMENTS_SPEC.md`.
6. Existing `docs/REQUIREMENTS.md`, `docs/CREATOR_VISION.md`, `docs/CREATOR_QUICKSTART.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md` and relevant current source/tests.

**Important:** Some older documentation describes historical milestones as not implemented even though the README and newer source show them implemented. Flag inconsistencies; do not regress behavior based on stale notes.

## Workstreams

1. Per-game, TOML-under-the-hood `.fig` configurations for suitable game-wide settings and flavoring.
2. Expand existing **text-only** creator wizard to build rooms, exits, items, interactions and optional FIG settings; display **actual file previews** to passively teach authoring.
3. Improve deterministic parser/object resolution, ambiguity handling, typo suggestions and errors; never require an LLM.
4. Low-risk cleanup with regression tests and updated current-state documentation.

## Hard constraints

- Offline terminal gameplay; no GUI, internet, accounts or LLM service.
- Authoring is text/TOML/FIG; no story-specific engine code.
- `.rork` retains explicit **manual only** saves; **absolutely no autosave**.
- Do not modify `.rork` format or compatibility versions as a side-effect.
- Migrate bundled RPG games to `config/character.fig`; no duplicate legacy loader. Preserve gameplay, not obsolete development formats.
- No destructive overwrites or file traversal via creator input.
- Keep tests and changes small, reviewable and accurately reported.
- Prefer architectural proposals before changes where the design is undecided.

## Phase 1 implementation and verification (2026-11-18)

Implemented configuration foundation; pending user review. `config.rs` loads immutable per-game GameConfig; LoadedGame carries config instead of a separate progression field. Full runner entry points are run_configured and run_configured_with_io. Existing convenience entry points use default terminology. Spellbook rendering takes explicit display terms.

All five bundled character files moved to config/character.fig with unchanged contents. Root character.toml is rejected with move guidance, not parsed. The RPG transactions sample demonstrates terminology.fig. Generated and starter narrative adventures remain minimal with no unnecessary config files; their instructions explain opt-in.

Windows checks executed successfully: cargo fmt --all -- --check; cargo test (114 tests); cargo clippy --all-targets -- -D warnings. CLI validation passed for story_example, templates/starter and all four RPG samples. Scripted RPG play verified custom empty/populated spellbook labels and unchanged commands. The independent Rust sample compiled after refreshing its stale local Lupine package version in its Cargo.lock; its initial --locked check had correctly refused that update.

No RORK serialization/version or progression arithmetic changes. No dependency additions, full wizard overhaul or parser upgrades. Native Linux, human interactive acceptance and published package execution were not performed. Existing extreme-XP arithmetic limitations identified in the audit are unchanged and remain separate work.

## Course obligations

CSE 310 professor allows this semester's preexisting Lupine work to count as a self-contained two-week sprint. Document real Rust features (mutable/immutable variables, loops, borrowing/ownership, `Vec`, `match`, at least one stretch), honest hours, GitHub source, course-template README, and demo/code walkthrough. Don't invent evidence.

## Validation for later implementation

`cargo fmt --all -- --check` · `cargo test` · `cargo clippy --all-targets -- -D warnings`, plus sample play/validation and manual platform checks. Never state these passed without running them.
