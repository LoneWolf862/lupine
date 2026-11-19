# Project files, repair and publication

Creator **F** is a file-first editor for every implemented TOML/FIG schema,
including actions, nested rules, objects/containers, events, timers, quests,
journals, endings, NPC dialogue, shops, abilities, equipment and statuses.
Existing individual guided managers remain available. F provides structural
editing and exact text replacement; it is not a separate bespoke form for each
engine feature. See [CONTENT_REFERENCE.md](CONTENT_REFERENCE.md) and the bundled
`samples/` for complete, validated authoring examples.

F captures one baseline and holds all changes in memory. Edit several files before
validation when definitions and references must change together. **V** validates
the proposed whole adventure. **P** shows original and exact proposed bytes,
including deleted files. **S** validates, previews, then asks for explicit
confirmation before publishing the entire batch. Any invalid proposal is refused.
There is no partial publish option. Earlier confirmed transactions remain saved.
`0`/`back` in a file returns to the project draft; at the project menu it discards
the batch. `cancel` or EOF discards all unsaved changes in this operation.

## Structural editing

Choose Edit/create and a relative path, then list field paths. `/metadata/author`
addresses a table key; `/actions/0/effects/0` addresses an array entry. Indices are
zero based. `-` appends to an array; `/` selects the root table. Escape a literal
slash in a key as `~1` and a literal tilde as `~0`.

Set accepts one TOML literal, such as `"New title"`, `3`,
`[{ set_flag = "opened" }]` or `{ on = { enter = "start" }, id = "arrival" }`.
Its parent must already exist. Remove deletes a key or array entry. Copy requires
a new destination and prompts for a new ID when copying a definition with an
`id` field. Copying a whole definition file creates a new file and stable ID;
references continue to point at the original. Review associated weapon/equipment
definitions separately. NPCs live in individual `NPCs/*.toml` files. Their nested nodes can be
copied to a new key (for example `/nodes/hello` to `/nodes/goodbye`). Full validation detects collisions and broken references.

Structural edits reserialize that file, removing comments and normalizing layout;
the preview shows this before publication. Use exact text replacement to retain
your own formatting/comments. Paste until `/end` (or `.end`); prefix a literal control line with a backslash.
`back` keeps this file unchanged while retaining other draft files. Pasted lines use LF. Syntax errors can remain in a draft, but cannot be
published. This also allows an invalid existing project to be repaired without
requiring it to load first. Unsafe filesystem layouts/links still require manual
attention; F never bypasses path protections.

Use **M** for a deliberate existing-ID or weapon-association migration. Ordinary
guided forms retain stable IDs. Advanced raw/structural edits do not automatically
redirect references. Delete previews typed ID occurrences and stages removal;
repair every remaining reference before publishing. **I** or `lupine inspect PATH`
shows the room graph and schema-typed ID occurrences, excluding narrative text.
This is static author inspection, not proof that every branch is reachable.

## Recovery

Snapshot journal version 2 records hashes of baseline and proposal, including
absent files for deletions, and whether the baseline loaded successfully. A repair
may start from invalid content. Restoring that baseline preserves its bytes and
reports that it still needs repair; completing a proposal always requires a valid
adventure. Missing, corrupt, older or externally changed evidence refuses automatic
recovery. Keep it for manual review. Existing player RORK files are never included,
rewritten or removed. RORK remains version 6; editor metadata does not revise saves.

## Publication properties

Creator **G** edits the title and optional `game.toml` publication metadata:

```toml
[metadata]
author = ""
description = ""
version = ""
credits = ""
license = ""
instructions = ""
```

Each value is text, up to 64 KiB, allowing newline/tab but no other terminal
controls. Multiline TOML strings are supported. Empty values are optional; no
authorship or license is invented. The version is an author-facing label, separate
from engine version, format version and save compatibility revision.

Export always includes `ABOUT_GAME.txt`; nonempty credits, license and instructions
become `GAME_CREDITS.txt`, `GAME_LICENSE.txt` and `GAME_INSTRUCTIONS.txt` in each
platform folder/ZIP. The engine license and dependency notices remain separate.
No external file paths or network fetches are involved.

Creator **3 > 3** edits implemented combat FIG settings. F also supports all three
FIGs. `lupine test PATH SCENARIOS.toml --trace` prints each scenario's transcript,
seed and final mutable state for debugging. This author-only output can reveal
hidden state; normal player output continues to hide exact enemy HP. Tests cannot
read or write RORK slots, and trace writes only to the supplied output stream.

Searchable file selection is available at F > L. [CREATOR_NAVIGATION.md](CREATOR_NAVIGATION.md) documents shared search, draft status and room-centered navigation.
