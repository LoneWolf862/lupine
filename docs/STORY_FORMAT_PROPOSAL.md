# Lupine v0.5 — Story file format proposal

**Status:** Design proposal for review; not implemented in Lupine v0.4.0.  
**Goal:** Let creators describe an adventure in editable local files while preserving the existing deterministic Rust engine and its public API.

## Design principles

- A new creator should be able to create a basic world by editing a small number of readable files.
- A story is **data**, never executable code or an untrusted Rust plugin.
- Every relationship uses a stable lowercase identifier; player-facing names may change.
- `World::validate` remains authoritative after loading. Parsing/format errors additionally report file paths and useful detail.
- Game definitions and runtime state remain separate. Files do not store the player's progress.
- Built-in Rust-authored worlds remain supported. Both pathways produce the same `World` representation.
- Offline playback is mandatory. Packaging into a single executable is a later task.

## Proposed first layout

```text
MyAdventure/
  game.toml
  rooms/
    foyer.toml
    workshop.toml
  items/
    brass_key.toml
    door.toml
  interactions.toml
```

This is intentionally a **directory of TOML files** rather than one massive document. For first implementation, room and item definitions are stored in one file per entity; interactions live in one shared file. The loader scans only the documented directories and file extensions.

## `game.toml`

```toml
format_version = 1
id = "example_adventure"
title = "Example Adventure"
starting_room = "foyer"
```

`format_version` is a story-format version, independent from Lupine's Cargo package version. Unknown future format versions should be rejected with an explanatory error.

## Room: `rooms/foyer.toml`

```toml
id = "foyer"
title = "Test Foyer"
description = "A plain room. A passage leads north."

[exits]
north = "workshop"

[[descriptions]]
when = [{ flag_set = "door_open" }]
text = "The doorway to the north stands open."
```

- `exits` is a direction-to-room map; supported directions initially are `north`, `south`, `east`, `west`, `up`, `down`.
- The first matching `[[descriptions]]` entry wins; when no variant matches, use `description`.
- Each `when` list uses logical AND. Empty `when` is not allowed on a conditional variant to avoid shadowing all later descriptions.
- A room does not directly own items. Item `starting_room` determines initial placement.

## Item: `items/brass_key.toml`

```toml
id = "brass_key"
name = "brass key"
aliases = ["key", "brass key"]
description = "A small brass key, worn smooth."
portable = true
starting_room = "foyer"
```

Nonportable targets, such as doors or panels, use the same item format with `portable = false`.

## Interactions and exit rules: `interactions.toml`

```toml
[[interactions]]
item = "brass_key"
target = "workshop_door"
requires = [{ has_item = "brass_key" }, { flag_unset = "door_open" }]
effects = [{ set_flag = "door_open" }]
success_text = "The key turns; the door unlocks."
failure_text = "The door cannot be unlocked again."

[[exit_rules]]
from = "foyer"
direction = "north"
requires = [{ flag_set = "door_open" }]
blocked_text = "The workshop door is locked."
```

For v0.5, preserve the already-supported conditions and effects only:

| Kind | TOML representation | Meaning |
| --- | --- | --- |
| Condition | `{ has_item = "brass_key" }` | Player carries the item |
| Condition | `{ flag_set = "door_open" }` | Flag is active |
| Condition | `{ flag_unset = "door_open" }` | Flag is not active |
| Effect | `{ set_flag = "door_open" }` | Activate flag |
| Effect | `{ clear_flag = "door_open" }` | Clear flag |
| Effect | `{ move_item_to_room = { item = "brass_key", room = "workshop" } }` | Relocate item |

All listed `requires` conditions must be met. All effects execute in the supplied order after validation. Conditions and effects should use typed variants rather than ad hoc strings at runtime.

## Required internal refactor before the loader

The current v0.4 API uses `&'static str` identifiers and definition fields (`RoomId`, `ItemId`, `FlagId` and prose fields). File content is allocated at runtime and **cannot safely be treated as `'static`**. Do **not** use `Box::leak`, global storage, or fake lifetimes to bypass this.

Recommended design: switch world-owned identifiers and text to owned `String` values; use a consistent owned ID strategy in `GameState` and all lookup methods (borrowing `&str` for queries), while preserving a convenient Rust-authored world API. This is a source-breaking refactor appropriate before v1.0. It should receive focused tests, particularly around cloning state, item relocation, and world lifetimes.

File ingestion will parse into `serde`-compatible DTOs, convert to engine `World`, then call validation. Keep file-format DTOs separate from core engine structs so the TOML schema can evolve without binding every internal detail to serialized field names.

## Error handling

The loader should return structured, contextual diagnostics and never panic on malformed content. Example:

```text
Story error: rooms/foyer.toml
  north exit points to unknown room 'workshp'.
  Did you mean 'workshop'?
```

For v0.5 the minimum is: malformed TOML with file name; duplicate IDs; missing rooms/items; missing start room; invalid direction; invalid/unknown condition or effect; unsupported format version. Suggestions are optional and must not silently change authored data.

## Scope and verification

**v0.5 acceptance checks:**

1. Load this sample story from disk into a `World` without changing the engine's command parser.
2. Execute the same progression as the Rust-authored sample: blocked exit, take key, unlock, dynamic description, navigation.
3. Reject one deliberately malformed file with a file-specific error.
4. Reject broken references and duplicate IDs with useful diagnostics.
5. Keep existing Rust-defined worlds and all v0.4 tests functional after the owned-string refactor.
6. Verify Windows and Linux paths without requiring network access at runtime.

**Not yet part of v0.5:** GUI editor, dialogue trees, nested containers, combat, user-defined scripting, save games, or one-click game packaging.

## Decision requested before implementation

Approve **TOML + one file per room/item + a central interactions file** as an initial *versioned* format. This is the simplest comprehensible starting point, not a promise that the final authoring UX must expose raw TOML.
