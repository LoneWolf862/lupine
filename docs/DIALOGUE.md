# NPCs and dialogue

Author one NPC per `NPCs/*.toml` file, for example `NPCs/guide.toml`.
Use exactly **NPCs/**, including capitalization, on Windows and Linux. The folder
may be absent or empty for games without NPCs; new adventures create it empty.
NPCs are distinct from items and combat enemies;
they have stable IDs, mutable locations and persistent conversation nodes. No RPG
configuration is required. See `samples/dialogue_demo` and its three scenarios.

```toml
id = "guide"
name = "Guide"
aliases = ["helper"] # optional
description = "A neutral demonstration character."
room = "start" # omit to start offstage
requires = [] # optional shared conditions for visibility and conversation
start = "hello"

[nodes.hello]
text = "Would you like an introduction?"
[[nodes.hello.variants]]
when = [{ flag_set="introduced" }]
text = "We have already met."
[[nodes.hello.choices]]
id = "introduce"
label = "Introduce yourself"
requires = [{ flag_unset="introduced" }]
effects = [{ set_flag="introduced" }]
next = "finished" # optional; otherwise stay at the same node
text = "Introduction complete." # optional success output

[nodes.finished]
text = "The introduction has finished."
```

The explicit `id` identifies the NPC everywhere, independently of the filename.
Renaming the file does not rename the NPC. IDs must be unique across the folder;
node IDs are local to each NPC. Keep files directly in NPCs/, without nested
folders or links. Regular non-TOML files are ignored. Definitions are loaded in
sorted filename order; each file is limited to 8 MiB, all NPC files to 64 MiB.

Define referenced flags in rules.toml if using declared flags. Each NPC requires
an ID, nonempty name, description field, starting node and 1..128 nodes. At most 1024 NPCs are
allowed; aliases, variants and choices are capped at 128. Node and choice IDs use
the same stable-ID rules as other content. Choices need unique IDs within a node,
nonempty labels, and valid conditions/effects/next-node references. Unknown fields
are errors. Conditions use the shared bounded nested vocabulary.

## Player interaction

Rooms list currently visible NPCs. `people` lists them without spending a turn.
`talk <name>` or `talk to <name>` resolves ID, name or alias using the normal
deterministic matching rules. Ambiguity prompts for explicit selection. Missing
NPCs never fall back to a hidden or distant character.

Opening a conversation shows the NPC description and the current node text. The
first matching variant replaces the default text. Only choices whose requirements
match are listed. Select a number or exact choice ID, or cancel with 0/back/cancel.
Invalid input asks again; EOF and cancellation change nothing. No choices means
the conversation is read-only. There is no automatic branch selection.

Each confirmed response applies one atomic transaction and returns to the main
prompt. It spends one successful action turn, including a response with no effects.
The next node is remembered for the next `talk`. Timers and event hooks run after
the response. The declared `next` transition is staged before effects; an explicit
`set_dialogue` effect may override it. If an effect fails, the response, transition
and success output all roll back, without spending a turn. Viewing text never
triggers effects. Use flags/conditions for once-only responses.

## Shared rules and transfers

| Rule | Meaning |
|---|---|
| `{ npc_in_room={npc="guide",room="hall"} }` | NPC location matches (independent of visibility requirements) |
| `{ dialogue_node={npc="guide",node="finished"} }` | NPC's current conversation node matches |
| `{ move_npc={npc="guide",room="hall"} }` | Move NPC; omit room to send offstage |
| `{ set_dialogue={npc="guide",node="hello"} }` | Change conversation node from any shared effect context |

Item definitions stay in `items/`; do not duplicate their definitions inside an NPC.
Items can start at `initial={kind="npc",id="guide"}`. NPC-held objects and their
container contents are inaccessible to ordinary take/use/combat commands. Moving
the NPC carries those objects with them. Use typed shared transfers to give or
receive quantities, for example:

```toml
effects = [{ transfer_item = { item="token", quantity=2, from={kind="npc",id="guide"}, to={kind="inventory"} } }]
```

Reverse `from` and `to` to give a loose carried item to the NPC. Transfer locations
are explicit; they do not search nested bags. Authors can require a particular
quantity at a typed location with `item_count`. Normal player object commands
still cannot access NPC inventories. Shared effects are author-controlled, so add
appropriate location/visibility requirements to actions outside dialogue.

Acquiring a gift participates in the existing take/acquisition event mechanism.
Dialogue can also update quests, start timers, move the player, or end the game
through the same effects. No separate dialogue scripting language is introduced.

## Persistence, tests and author tools

RORK **6** retains NPC progress and adds [combat/equipment progress](SYSTEMS.md).
The NPC schema persists exact NPC IDs, locations and node IDs, plus NPC object placements.
Unknown/missing NPCs, nodes and rooms are rejected before replacing live progress.
Offstage locations round-trip. Saving remains exclusively manual; loading does not
replay dialogue. Older saves need their matching engine/game build and are not rewritten.

Scenario assertions accept `npcs={guide={room="start",node="hello"}}`, comparing
the complete NPC state map; omit room for offstage NPCs. Existing quantity assertions
accept the NPC location type. Run `lupine test samples/dialogue_demo
samples/dialogue_demo/scenarios.toml`.

Snapshot, project-file search, room-reference browsing and export include NPCs/*.toml.
Creator F can create/edit, duplicate or delete a whole NPC file, and copy/edit
nodes and choices inside it. Duplication changes the new file's ID but leaves
references pointing to their original targets; review explicit self-references
and item ownership before publishing. Migration M offers `npcs` and
`nodes:<npc_id>` categories, updating typed references and table keys while preserving
prose and RORK bytes. Choice IDs are local input labels; no persistent state refers
to them. F is the complete-schema file/structural editor; no separate NPC form
or automatic behavior generator is implied.

## Moving an existing development adventure

The root `dialogue.toml` layout is no longer loaded. Its presence produces an
explicit migration error rather than silently dropping NPCs. To split it:

1. Create `NPCs/` and one TOML file for each `[npcs.ID]` definition.
2. Replace that outer table header with top-level `id = "ID"`.
3. Remove the `npcs.ID.` prefix from all nested table and array-table headers:
   `[npcs.ID.nodes.hello]` becomes `[nodes.hello]`, for example.
4. Preserve NPC IDs, node IDs, fields, conditions/effects and external references.
5. Remove the old root file after reviewing the split, then validate and run
   your scenarios. In F, stage all new files and deletion of dialogue.toml in
   one batch; validate, preview and explicitly confirm publication.

No user adventure is converted automatically. This file-only move leaves the
runtime model and RORK 6 schema unchanged, and needs no save-compatibility bump.
Existing RORK 6 saves remain usable with otherwise identical content and IDs;
changing NPC/node IDs or other gameplay definitions still needs a separate save
compatibility decision. Never edit the saved progress to perform this split.

All currently supported NPC-specific definition fields live in its file.
Shared flags, events, quests, items and shops retain their own definitions and
are referenced by ID. This layout does not add NPC combat, pathfinding, schedules,
or a direct NPC-to-shop binding.
