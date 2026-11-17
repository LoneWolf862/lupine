# RORK save files — future specification (NOT IMPLEMENTED)

**Target:** v0.6.0. This document describes decisions, not a working save system.

- Every Lupine game will support reserved terminal commands `save`, `load`, and `saves`.
- Saving is **always manual**. No autosave on turns, checkpoints, exit, or background events.
- Named slots, confirmation before overwriting existing files, and warnings before discarding unsaved changes are intended.
- File extension `.rork`; TOML content for portability and inspectability.
- Proposed filename: `<normalized-save-name>.<stable-game-id>.rork`; e.g. `forestcamp.howlbound.rork`.
- Stable game ID comes from the story manifest, not the display title. Save names must be normalized safely, with collisions detected rather than silently overwritten.
- Game-specific save directories should live in standard user-data paths. Saves can be copied to compatible Windows and Linux installations.
- Save data will preserve mutable `GameState` (current room, item positions/inventory, flags and supported future state), alongside schema/game IDs and compatibility metadata. Immutable story content is not duplicated.
- On load: parse, check schema/game identity and compatibility, validate references against the current world, then replace the live state only on complete success.
- On save: write safely through a temporary file and replace/rename carefully to avoid damaging existing slots on failure.
- Never interpret TOML save content as executable code; do not let story content choose arbitrary filesystem destinations.

**Open implementation details:** game-version policy, explicit save schema, normalization collision handling, platform-specific path selection, and how a player imports portable save files. These must be finalized and tested when v0.6.0 is implemented.
