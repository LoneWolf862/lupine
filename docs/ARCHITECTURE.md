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

`new` (including the basic wizard), `add room`, `add item`, `add art`, `validate` and `play` exist. The generator uses editable content files; it does not generate unnecessary configuration or enable RPG implicitly. The full guided authoring journey is future work.

A manually triggered Windows/Linux packaging workflow exists; successful source checks do not establish native Linux verification or a published release. Run formatting, tests, strict Clippy and platform-specific smoke checks and report actual results. Preserve CSE 310 evidence and README-template obligations.
