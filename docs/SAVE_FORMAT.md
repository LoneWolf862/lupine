# RORK Save Format — v0.6.0-A Working Specification

**Stage A: serialization only (not yet integrated into gameplay).** `src/save.rs` provides `encode_save` and `decode_save`, with TOML-based save format version 1 and validation against the currently loaded `World`. The `save`, `load`, and `saves` commands, filesystem storage, overwrite confirmation, unsaved-progress warnings and cross-platform compatibility policy remain **planned for stages B/C**.

## Locked requirements

- **Manual saves only.** No autosave, save-on-quit, checkpoint save or background save.
- Extension: `.rork`. Filename convention: `<normalized-slot>.<game-id>.rork` (e.g. `forestcamp.howlbound.rork`); normalizing/collisions are not implemented in Stage A.
- Use stable game ID, not game title; keep saves local and portable.
- Three reserved terminal commands: `save`, `load`, `saves`. These are **not yet implemented**.
- Multiple named slots and confirmation before overwriting, with safe write/replace behavior.
- Validate metadata and complete game state before replacing the live session.

## Format v1

The file contains `save_format_version = 1`, `game_id`, `save_name`, `current_room`, `flags`, and an `item_locations` table keyed by stable item ID. Each location is an inline/table object with `kind = "inventory"` or `kind = "room"` and, for room locations, `room = "<room-id>"`. This avoids ambiguity if a story has a room named `inventory`. All item locations must be present exactly once; unknown rooms or items are rejected. Flags are free-form engine state identifiers (world definitions do not currently enumerate them).

Example (partial):

```toml
save_format_version = 1
game_id = "howlbound"
save_name = "Forest Camp"
current_room = "forest"
flags = ["cabin_unlocked"]

[item_locations.lantern]
kind = "inventory"

[item_locations.coin]
kind = "room"
room = "cabin"
```

**Compatibility note:** Stage A verifies save format, game ID, item and room references but does not yet identify game content revisions. The compatibility/version policy must be established before calling the complete save feature production-ready.

**Important:** Do not claim that saving works interactively until stages B/C have been implemented and tested.
