# Shared story rules

Implemented in engine expansion phase 2. Ordinary interactions, conditional exits,
room/item prose and room art share these typed conditions. Define content directly
in TOML; advanced rule forms currently have no dedicated wizard form. Existing forms
preserve them, and migration follows their schema-aware references.

Optional `rules.toml` declares initial flags and integer counters:

```toml
[flags]
door_open = false
permission = false
[counters]
attempts = 0
```

If flags are declared (nonempty table), all authored flag references must name a
declaration, including shops and encounters. Without declarations, existing free-form
flags begin unset. Counters always require declarations. IDs: lowercase letters,
digits, underscores/dashes, 1..80 bytes. Initial values apply only to fresh sessions.

## Conditions

Each condition is a table with exactly one named operation. A `requires`/`when`
list still requires all its entries. Supported operations:

| TOML | Meaning |
|---|---|
| `{ has_item="key" }` | Item carried |
| `{ flag_set="door_open" }` / `{ flag_unset="door_open" }` | Boolean memory |
| `{ all=[{...}, {...}] }` | All nested conditions |
| `{ any=[{...}, {...}] }` | At least one nested condition |
| `{ not={flag_set="door_open"} }` | Negation |
| `{ in_room="hall" }` | Current room |
| `{ visited="hall" }` | Room visited in this session/save |
| `{ item_in_room={item="key",room="hall"} }` | Item's exact location |
| `{ counter={id="attempts",op="ge",value=3} }` | Integer comparison: eq, ne, lt, le, gt, ge |

`all`/`any` require 1..128 entries; nesting is limited to 16 condition levels.
Unknown fields, operations and references fail validation. Empty top-level condition
lists remain unconditional. Ordered outcomes use first-match priority.

## Effects and outcomes

| TOML | Effect |
|---|---|
| `{ set_flag="door_open" }` / `{ clear_flag="door_open" }` | Change a flag |
| `{ set_counter={id="attempts",value=0} }` | Assign a declared counter |
| `{ add_counter={id="attempts",amount=1} }` | Add signed amount; overflow fails |
| `{ move_item_to_room={item="key",room="hall"} }` | Relocate an object |
| `{ give_item="key" }` | Move a portable item into inventory |
| `{ remove_item="key" }` | Mark an existing available item consumed/unavailable |
| `{ move_player="hall" }` | Move the player and record a visit |
| `{ show_text="A mechanism clicks." }` | Additional success output |

Effects run in authored order on a draft. Every effect must succeed before state or
effect output is published. Failed consumption, overflow or invalid references leave
the original state intact. Maximum 256 effects per outcome and 128 outcomes per
interaction. These effects cannot save progress or execute system code.

```toml
[[interactions]]
item = "key"
target = "door"
requires = [{ any=[{has_item="key"},{flag_set="permission"}] }]
effects = []
success_text = "Nothing else happens."
failure_text = "You cannot do that yet."

[[interactions.outcomes]]
when = [{ flag_unset="door_open" }]
effects = [{set_flag="door_open"}, {add_counter={id="attempts",amount=1}}]
text = "The door unlocks."

[[interactions.outcomes]]
when = []
effects = []
text = "The door is already open."
```

The outer requirements run first. The first matching outcome replaces the outer
effects/success text; no matching outcome uses the outer fallback. Normal `use X on Y`
requires carrying portable X; fixed accessible X is allowed. See [authored actions](ACTIONS.md)
for general verbs and single-object use. New condition/effect vocabulary also works in direct Rust worlds.

## Diagnostics, migration and saves

Validate reports advisory unreachable-room, never-set flag,
contradictory flag and shadowed-outcome warnings. These are conservative aids, not
a solvability proof. All warnings preserve author content and permit intentional
offstage rooms. Migration M includes counters, declared flags, nested conditions and
outcomes, preserving prose and RORK bytes. Renaming can invalidate saved state.

[Events and story progress](EVENTS.md) extend the shared vocabulary with quest, journal,
timer and turn conditions, and timer/quest/journal/ending effects.

RORK serialization is now version 5: [object state](OBJECTS.md), counters and visited
rooms and event progress are persistent. Only version 5 loads in this development engine; keep the matching old engine
and game for older saves. No obsolete loader or automatic migration is introduced.
Game save-compatibility revisions remain author-controlled and independent of Cargo.
The published v0.9.0 engine/tag are unchanged. See [save policy](SAVE_FORMAT.md).

[NPC conditions and effects](DIALOGUE.md) add location/node queries, NPC movement
and conversation transitions. Typed item locations also accept NPC ownership.
