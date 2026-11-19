# Lupine Adventure Creator

Current source: 0.10.0-dev.1 development candidate. Creator work through Phase 2I is merged.
Phase 3 adds startup, player input and export in draft PRs #20–#24; published v0.9.0 binaries
do not include these changes. [Verification](PHASE_3_REVIEW.md).

## Export and share

Choose E, select Windows x64 / Linux x64 / Both, choose compatible local runtimes and a
new destination outside the source adventure, review the files and explicitly confirm.
Enter uses the running engine for its native OS. Other runtimes need matching
lupine.runtime.toml metadata from the same trusted build. No runtime is downloaded.
Share the complete platform folder, optionally zipped. Source files stay untouched;
personal RORK saves and development files are excluded. Missing binaries/invalid content
produce actionable errors. [Full installation/export instructions](INSTALLATION.md).

## Start or reopen

From the source checkout:

```text
cargo run -- new
cargo run -- creator
cargo run -- creator "my_adventure"
```

With a standalone executable, replace `cargo run --` with `.\lupine.exe` on Windows or `./lupine` on Linux. No-argument Lupine opens the startup menu; choose Create or Edit. The explicit fixture command opens the test adventure.

New setup explains the adventure title, destination folder, opening room and description. Enter accepts defaults; `advanced` at welcome exposes custom IDs and direct previews. Review before creating. Existing folders are never overwritten. Successful creation enters the creator; opening an existing project never rewrites it.

## Main menu

| Option | Purpose |
|---|---|
| 1 | Play from a fresh state; `quit` returns here. Load a save explicitly if needed. |
| 2 | Validate all supported story, RPG and FIG content. |
| 3 | Configure character/progression and display terminology. |
| 4 | Create/edit items, aliases, portability and conditional descriptions. |
| 5 | Create/edit spells and teaching-item links. |
| 6 | Create/edit ASCII artwork and room references/variants. |
| 7 | Create/edit rooms, connections, conditional descriptions and encounters. |
| 8 | Help. |
| 9 | Interactions, conditional exits and the guided locked-door recipe. |
| R | Enemies, weapons, shops, currencies and encounter settings. |
| E | Validate, preview and export a Windows/Linux adventure package. |
| 0 | Exit the selected project; return to the launcher if opened through it. |

The real nested ASCII file tree refreshes after completed operations and on returning to the creator. Enter refreshes after external edits. Existing empty directories are shown; absent files are not invented. Inspection is bounded to 2048 entries/eight nested levels; display shows at most 160 entries and abbreviates long names. Links/junctions are not followed. Inspection problems block play/validation/editing.

## Shared controls and drafts

`?` explains fields; Enter accepts the displayed current/default value. `back` returns from a prompt/menu; `cancel` abandons the current operation. EOF discards unfinished work. Prefix literal control words with a backslash. Lists use numbered choices; N creates in content managers. Changes stay in memory until explicit Save confirmation. Existing IDs remain stable; changing a display name does not rename references or filenames.

Most editors use S to save; Room Options uses 6. Every save validates the proposed adventure before publication. Declining Save retains ordinary editor drafts; the locked-door recipe retains its review and offers revision if declined. For ordinary prose, `/multi` begins multiline entry and `/end` finishes. Use the indicated escape for literal controls. Artwork entry has its own whitespace-preserving instructions below.

The exact-preview markers enclose the same bytes used for publication. TOML-aware edits preserve unedited properties, comments and existing CRLF style; changed values may use different quoting. Explicitly deleting a property/entry also removes its attached comments. Reordering variants can move table blocks while retaining their content/comments. Unsupported representations are refused rather than destructively rewritten.

## Rooms and connections

Choose 7, then N or an existing number. New rooms ask for a name, suggested/custom ID and description. A room is any location. `{{objects}}` inserts currently visible object names.

Connections can lead from another room into the selected room or outward from it. Choose a direction and optionally its reverse. Saying no creates a one-way connection and does not remove existing reverse routes. Conflicting routes require explicit replacement approval. Removing an outgoing exit does not automatically remove the reverse exit or its conditional rule; invalid remaining references block Save.

Room Options: 1 name, 2 description, 3 connections, 4 properties, 5 exact previews, 6 Save, 7 advanced settings, 0 discard. Advanced settings edit conditional descriptions and encounter tables. First-match descriptions replace the default when all their conditions pass. Select an existing variant to edit just its text or requirements; remove/reorder variants explicitly.

Encounters require explicitly enabled RPG; select an existing enemy or use N to create one in the room draft. Configure chance 0–100, group size 1–12 with minimum <= maximum, persistence (`temporary`, `room`, `unique`), weighted enemy choices and flag requirements. Weights are relative, not percentages. Return to Room Options to preview/save the whole room draft. Exact enemy HP stays private during gameplay.

## Items

Choose 4, then N or an existing item. New items ask for a unique ID and starting room. Options edit name, description, aliases, portability, starting room and conditional prose. Option 6 inspects the exact TOML; S saves.

Portable objects can be picked up and carried. Fixed objects remain in their room. Aliases are alternate player names: Brass Key might have `key` and `brass key`. Add one at a time; select a numbered alias to remove it after confirmation. Teaching items and weapon items must remain portable in guided editing. Full validation also protects shop/loot/binding references. Starting-room changes affect fresh sessions; existing saves are not rewritten.

## Spells and abilities

Choose 5 after explicitly enabling RPG in configuration. All existing fields are available: ID (creation only), display name, description, MP cost, effect kind, positive power and optional teaching item. There is no spell damage-type setting in the current schema.

Damage requires an enemy target (`cast <spell> on <enemy>`); heal and restore-mana affect the player. Defining a spell does not teach it. First create a portable scroll/item in Item Manager, then choose it as the teaching item. During play, take it and use `learn <item>`; learning consumes it. One item cannot teach multiple spells. Type `remove` at the teaching-link prompt to clear it. Preview with 7 and save with S.

## ASCII artwork

Choose 6, then N or an existing file. Option 1 accepts multiline text directly; `/end` finishes. Spaces, tabs, blank lines and ordinary backslashes are preserved. Escape literal `/end`, `back` or `cancel` control lines with one backslash. Preview with 2 and save with S. Artwork is UTF-8, at most 64 KiB, without terminal control sequences. Existing nested `.txt` files are editable; new files use simple names directly under `art/`.

R selects a room. Set/remove default artwork, add/remove/edit a conditional variant, or move a variant earlier. Replacing existing inline artwork requires confirmation. Variants may use a file or inline text and run in first-match order before the default fallback. Editing conditions retains other requirements. Removing references never deletes the artwork file. Artwork itself is not stored in RORK; saved flags/inventory determine the picture.

## Interactions, flags and locked doors

Choose 9. Option 1 manages interaction and conditional-exit rules. An interaction describes `use <item> on <target>`, requirements, effects and success/failure messages. Each item/target pair has one rule. Both objects must be accessible and the used item must be carried. Conditions support `has_item`, `flag_set`, `flag_unset`; effects support `set_flag`, `clear_flag`, `move_item_to_room`. These are existing engine capabilities, not a separate scripting language.

A flag is a named yes/no memory, initially unset. An interaction can set `door_open`; an exit, alternate description or artwork can require that same flag. All requirements in a list must pass. Edit lists by adding/removing individual entries, then return to the rule menu to preview/save. Conditional exits must already connect rooms; each room/direction has one rule. Removing a rule makes that route unconditional.

Option 2 guides a locked door: select room/direction, select or create a portable key and door object, choose the unlocked flag and write success/failure/blocked messages. Preview all related files and confirm one transaction. Only the selected direction is locked; configure a reverse rule separately if needed. Conflicting existing rules are rejected rather than replaced silently. Cancel/decline writes none of the draft files. Later revisions use the item/rule editors.

## FIG configuration

Choose 3, then Character or Terminology. Menus show current values and defaults; selecting a setting explains its purpose, range and dependencies using the bundled self-documenting FIG templates. P previews, S confirms/validates/saves, 0 discards. Opening absent config does not create files.

`enabled = true` explicitly enables RPG; new games use false. Existing character FIGs that omit enabled retain true. All progression values validate even while disabled. Existing spells/encounters and other dependent RPG content prevent disabling. Starting HP/MP/AP are bases at level 1 plus per-level gains for higher starting levels. AP 100 means 1.00x weapon damage; level cap 0 means no authored cap; XP growth 120 means +20 percent per level.

Terminology exposes only `display.spellbook` and `display.spells`; these change labels, not player commands. No unsupported FIG categories are generated. Existing saves are never rewritten. If a configuration change invalidates existing saves, deliberately manage `game.toml` save_compatibility_version; the editor does not change versions automatically.

## Save safety and recovery

Authored content saves and player saves are separate. **Player progress is never autosaved**, including on quit, EOF or entering/leaving play. Only explicit `save`, `load` and `saves` commands handle RORK.

Editors require a valid project. They snapshot content, reject unsafe paths/links, stage proposed changes privately, run the complete validator, compare original bytes/metadata to catch external edits, then publish complete files without truncation. Modified files retain permissions. Failed publication rolls back when safe. Failed ordinary form saves retain drafts for correction; stale-file/recovery conflicts require discarding/reopening after review.

All editors share `.lupine-room-edit/` as the historical lock/recovery directory. If it remains, stop editing and read RECOVERY.txt and CHANGES.txt. `original/` retains displaced files; `proposed/` is the staged project. COMMITTED.txt means publication/validation completed but cleanup may not have. Preserve external changes, deliberately restore originals/remove only transaction-created files when appropriate, validate, and then remove the reviewed recovery directory. **Never delete recovery data blindly.** There is no automatic recovery command.

Hard-link support is required. Publication is recoverable, not crash-atomic across multiple files. Limits are 8 MiB per snapshotted file and 64 MiB total, plus traversal bounds. Avoid concurrent editing/play while publishing. Checks do not defend against hostile concurrent filesystem replacement. No unrelated files or existing player saves are deleted.

## Windows review and current limits

Follow the [review procedure](CREATOR_COMPLETION_REVIEW.md). The automated `tests/fixtures/creator_acceptance.txt` transcript builds a nontrivial neutral adventure entirely through the CLI and is exercised by `cargo test --test creator_acceptance`. It is a test fixture, not HOWLBOUND story content or a human usability result.

Current limits: stable-ID rename/deletion migrations and guided crash recovery are not implemented; export creates directories, with archives left to the user's normal ZIP tool. Transaction-binding definition editing remains separate; enemy, weapon, shop and currency managers are now available under R. Unusual condition/effect representations that cannot be safely edited receive a limitation message. Full text-file authoring and direct add/validate/play commands remain available.
