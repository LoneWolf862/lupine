# RORK Save Format — v0.6.0-A Working Specification

Phase 4 creator ID migration changes authored content only. It never searches for or
rewrites player RORK files, changes game identity, or increments compatibility revisions.
Room/item/ability/enemy/currency/shop/offer renames may invalidate old saved references;
renamed flags may remain loadable but change behavior. Authors must deliberately decide
when to increment game.toml save_compatibility_version to reject old saves. No automatic
compatibility is promised. Creator features do not change RORK serialization.
See [migration safety and verification](PHASE_4_REVIEW.md).

Current command behavior (2026-11-19): saving remains exclusively player-initiated.
Bare `save` offers to overwrite the last successfully saved/loaded slot in this session;
no asks `Save name:`. Fresh sessions ask for a name immediately. Explicit `save NAME`
writes that named slot without another question, including replacing the same name.
This session-only slot selection does not change RORK serialization or compatibility.
The stage records below describe earlier behavior; see [current player guidance](INSTALLATION.md).

> Historical design/stage record: feature-status and compatibility claims below describe earlier development. Current policy and file paths are in [NEXT_PHASE_REQUIREMENTS.md](NEXT_PHASE_REQUIREMENTS.md) and [CONFIG_FIG_SPEC.md](CONFIG_FIG_SPEC.md). Character configuration now belongs in `config/character.fig`.

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

## Stage B implementation candidate — interactive commands

`src/save_files.rs` and `src/runner.rs` now implement manual-only `save [name]`, `load [name]`, and `saves` in the terminal interface. Named slots are confined to per-game user-data directories and use normalized `.rork` filenames. An overwrite requires an explicit confirmation, and loading an existing slot checks all saved state before changing the session. If gameplay state changed since the last save/load, quitting or loading asks for confirmation to discard those changes. No implicit saving occurs.

**Stage B is not production-ready persistence:** overwriting still writes directly to the existing file. Atomic writes, robust crash safety, and game-content compatibility checks are reserved for v0.6.0-C. See `docs/V0_6_0_B_UPDATE.md`.

## Stage C — Reliability and content compatibility (candidate)

- A `game.toml` manifest may include `save_compatibility_version = 1`; omitting the field defaults it to 1 for existing games.
- New `.rork` files record `save_compatibility_version` alongside `save_format_version = 1` and `game_id`. Older RORK v1 saves that omit the compatibility field are treated as revision 1.
- A save's compatibility revision must equal the current game's revision to load. Revision changes do **not** migrate data or promise compatibility when item IDs/rooms are removed; structural validation remains mandatory. Story authors should increment the revision for breaking game-state changes, not cosmetic text edits.
- The writer stages a synced temporary sibling file, then atomically publishes it on supported local filesystems. A failed staging/rename attempt leaves an existing save untouched. A failed new-slot operation leaves no playable partial `.rork` file.
- File safety is best-effort: crash/power-loss durability depends on the OS/filesystem; a crash may leave ignored `.tmp` files. No backups, migrators, or cryptographic integrity are implemented yet.
- Saving remains strictly **manual-only**. The `save`, `load`, and `saves` terminal verbs remain reserved engine commands.
