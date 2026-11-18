# Lupine architecture

Lupine is an unreleased, offline terminal engine. Justified pre-1.0 API/format changes are allowed; migrate bundled consumers rather than adding obsolete-format loaders. Preserve gameplay and keep the engine story-independent.

## Content, configuration and state

- `story_loader.rs` reads TOML content into validated immutable `World` definitions, with stable IDs, rooms, items, conditions/effects and artwork.
- `config.rs` reads optional TOML-syntax `config/character.fig` and `config/terminology.fig` into per-game `GameConfig`. LoadedGame owns this alongside world/identity. Character progression is optional; missing terminology uses defaults. See CONFIG_FIG_SPEC.md.
- `rpg_content.rs`, `encounters.rs`, `wallet.rs` and `shops.rs` load optional story definitions. Currency definitions remain in game.toml; individual spell/enemy/item mechanics remain content.
- `state.rs` holds mutable location, inventory, flags, character, learned abilities, wallet, purchases and encounter progress. Configuration is not player state.
- `parser.rs` parses ordinary commands; `engine.rs` resolves objects, checks rules and applies effects. RPG/reserved commands currently dispatch in `runner.rs` and domain modules. Parser unification is future work.
- `render.rs` and domain output functions render results. Terminology is passed explicitly to relevant output; no global string replacement.
- `runner.rs` provides terminal and scriptable entry points, and reads immutable config separately from GameState. Full entry points are `run_configured` and `run_configured_with_io`; convenience entry points supply default terminology. Rust-defined worlds remain supported.
- `save.rs` validates/serializes RORK state; `save_files.rs` stages, syncs and publishes manual slots. The runner validates a complete load before replacement. Never autosave, save on quit or checkpoint automatically.

## Creator and distribution

`new` (including the Phase 2A.1 beginner/advanced setup and optional exact previews), `add room`, `add item`, `add art`, `validate` and `play` exist. The generator uses editable content files; it does not generate unnecessary configuration or enable RPG implicitly. The full guided authoring journey is future work.

A manually triggered Windows/Linux packaging workflow exists; successful source checks do not establish native Linux verification or a published release. Run formatting, tests, strict Clippy and platform-specific smoke checks and report actual results. Preserve CSE 310 evidence and README-template obligations.

Creator foundation: `creator/wizard.rs` collects and validates input and previews serialized `DraftFile` values; `creator/files.rs` stages, syncs and exclusively publishes files. `creator.rs` assembles a minimal project and validates it with the story loader. TOML serialization also supports future FIG drafts without adding configuration editing today. Project rollback tracks created paths and never recursively removes a project directory.

Phase 2A.1: `creator/setup.rs` owns the beginner/advanced step flow, local sibling-ID checks, review navigation and platform-quoted completion instructions. `wizard.rs` returns an optional answer (`None` means back); cancellation/EOF remain explanatory errors for library callers and friendly successful exits in the CLI. Gameplay starts only when separately requested with `play`.

## Phase 2B creator navigation

`creator/session.rs` holds the selected absolute project path and borrows one input/output pair through setup, menu actions and gameplay. `creator` opens Create/Open navigation; `creator PATH` opens a project directly; interactive CLI `new` enters the session after setup. The public setup-only `new_game_wizard` API and noninteractive generators retain their behavior. No-argument CLI execution still runs the engine fixture.

`adventure.rs` assembles the existing story/FIG, RPG, encounters, loot, wallet and shop validators once for CLI play, CLI validation and creator actions. It holds immutable loaded content and delegates gameplay to `runner::run_configured_with_io`. No player-parser, combat, progression, or save changes. Every creator validation/play reloads files. Each play begins a fresh in-memory state, with manual load/save available through the unchanged runner.

`creator/tree.rs` inspects recognized paths read-only, without following symlinks or Windows reparse points. Tree output and traversal are bounded. Unsafe/incompletely inspected projects can be viewed but cannot be played/validated through the creator. Root links are rejected before opening. Invalid TOML can still be opened to receive existing loader diagnostics. The session writes no existing project files and creates no configuration files.

Existing-file persistence belongs to Phase 2C and later. Its review gate must cover comment/unknown-content preservation (or a clearly offered nondestructive alternative), original-byte comparison to detect external edits, exact previews and confirmation, full staged validation before publication, and tested rollback/recovery for changes spanning several files. The exclusive-new-file writer is not an existing-file editing implementation.
