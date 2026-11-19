# Objects, containers and quantities

Implemented in engine expansion phase 4. Existing items remain unique objects
with quantity 1 and their original `starting_room`. Optional fields are authored
in `items/*.toml`; no Rust or wizard action is needed.

```toml
id = "ration"
name = "ration"
description = "A packed meal."
portable = true
initial = { kind = "inventory" }
stackable = true
quantity = 5

[consumable]
requires = []
effects = [{ add_counter = { id = "meals", amount = 1 } }]
text = "You eat one ration."
failure_text = "You cannot eat that now."
```

Declare `meals` in `rules.toml [counters]`. Consumption removes **one** accessible
carried unit and applies its effects atomically. A failed requirement or effect
does not consume the item or publish success text. Teaching scrolls also consume
one unit. Story effects are currently available; RPG reward/effect integration is
expansion phase 7.

Placement is either `starting_room = "room_id"` or `initial`, never both:

| Initial value | Meaning |
|---|---|
| `{ kind = "room", id = "workroom" }` | In that room |
| `{ kind = "inventory" }` | Carried at the start |
| `{ kind = "container", id = "chest" }` | Inside a defined container |
| `{ kind = "consumed" }` | Offstage/unavailable; no active quantity |

`hidden = true` excludes an item from object lists, resolution, suggestions and
bulk collection until a `reveal_item` effect. It does not rewrite static prose;
authors should use conditional descriptions or `{{objects}}` where appropriate.

```toml
id = "chest"
name = "chest"
description = "A storage chest."
portable = false
starting_room = "workroom"

[container]
open = false
locked = true
key = "key"       # optional portable item ID; absence permits authored lock effects
capacity = 5      # optional maximum direct contained units, not weight
```

Containers may be portable bags. Contents remain inside when the container moves.
Closed or locked containers block access to contents, including when carried.
Nesting is limited to 16; cycles are invalid. Capacity counts directly contained
units; a contained bag counts as one unit and enforces its own capacity separately.
Containers cannot stack or be consumables. Fixed items cannot be carried or placed
inside containers. `examine chest` shows its state and visible contents when open.

Player commands:

- `take ration`, `drop ration`: one unit; ordinary unique-item behavior is unchanged.
- `take 3 ration`, `drop 2 ration`: explicit quantities.
- `put 2 ration in chest`, `take ration from chest`: transfer between locations.
- `open chest`, `close chest`, `unlock chest with key`, `lock chest with key`.
- `consume ration`, `eat ration`, `drink ration`, or bare `use ration`: one unit.
  Existing transaction bindings and authored single-object use take precedence.
- `take all`: accessible portable items, including visible contents of open containers.
  `drop all`: loose inventory, with contents staying in carried containers as they drop.

Names/aliases and clarification use the normal resolver. Tools must be accessible
and carried. Transfers prefer loose inventory for a carried source, and the current
room for a take source; `from <container>` selects a particular source. Explicit
quantities refer to that source, not an implicit combination of multiple locations.
Failures and cancellations do not mutate state. Authored actions may deliberately
override object verbs; use distinct patterns when the built-in behavior is desired.

Unique items have quantity 1. Only portable non-container items may set
`stackable = true`; initial quantity is 1–1,000,000. Each ID has a bounded total of
1,000,000 active units, split across any valid rooms, inventory and containers.
Inventory, room lists and container descriptions display counts for stacks.

## Shared object conditions and effects

Conditions work everywhere shared rules are accepted:

```toml
requires = [
  { item_visible = "key" },
  { container_open = "chest" },
  { not = { container_locked = "chest" } },
  { item_count = { item = "ration", op = "ge", value = 2 } }
]
```

`item_count` defaults to the total carried quantity, including contents of carried
containers. Add `location = { kind = "container", id = "chest" }` (or a room or
inventory placement) to check a specific location. `has_item` means physical
possession, including contained units; `item_visible` additionally checks access.
`item_in_room` checks a positive direct room quantity, even if other units are carried.

```toml
effects = [
  { reveal_item = "note" },
  { hide_item = "note" },
  { set_container = { item = "chest", open = false, locked = false } },
  { add_item = { item = "ration", quantity = 2 } },
  { remove_quantity = { item = "ration", quantity = 1 } },
  { transfer_item = { item = "ration", quantity = 1, from = { kind = "inventory" }, to = { kind = "container", id = "chest" } } }
]
```

`add_item` creates units in inventory; unique duplicates fail. `remove_quantity`
consumes accessible carried units. `transfer_item` moves a specified quantity and
validates capacity/cycles; authored effects can explicitly place items inside closed
containers. `set_container` must change at least one state field and cannot leave
the container both open and locked. `give_item`, `remove_item` and
`move_item_to_room` retain whole-object semantics, affecting all active units of an
ID; relocating an unavailable ID restores one unit. Use quantity effects for stacks.

RORK format **4** retains the phase-4 object schema and adds [event progress](EVENTS.md).
The object schema persists the canonical placement/quantity ledger, hidden items
and container state. Every save validates the full ledger; unsupported older
formats require their matching engine/game build. No save files are automatically
rewritten. Game compatibility revisions remain independent of engine versions.

Snapshot publication, export and ID migration preserve these authored fields.
The basic item form retains unedited properties; choosing a starting room explicitly
replaces `initial`. Full guided controls are expansion phase 8. Use
`lupine test samples/objects_demo samples/objects_demo/scenarios.toml` for executable
examples. Scenario `quantities` assertions accept `{item, location, quantity}` entries;
optional `hidden` asserts the complete hidden-ID set.
