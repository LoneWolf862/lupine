# Lupine architecture

Engine expansion: [approved sequence and status](ENGINE_EXPANSION.md). `scenario.rs`
loads strict TOML test suites and invokes the real runner with an explicit random
seed, returning final in-memory state for assertions. Existing runner APIs preserve
their signatures/default randomness. Scenarios never save implicitly.

## Current development architecture (0.11.0-dev.11)

Lupine is an offline terminal engine. Immutable TOML content is separate from mutable
GameState; supported FIG files hold configuration. Manual RORK persistence remains in
save.rs/save_files.rs; serialization version 7 adds the optional player name to objects, story, NPCs and combat progress.
`systems.rs` assembles resource definitions, equipment/status content and combat settings.
World owns the immutable rule context (RPG definitions, wallet and progression) so shared
effects validate across modules. story_loader loads it before world validation; Adventure
reuses it. Rust callers supplying optional content to the runner have it installed in
that context before validation. Direct save/rule callers must populate world.systems
with their definitions too; no player state is used as an implicit content registry.
`dialogue.rs` loads NPC definitions, resolves visible characters and applies one atomic
choice at a time. An authored `ask_name` choice collects and validates input before
committing the name and effects together; cancellation and failed turn hooks roll back.
NPC placements and conversation nodes are saved separately from content.
`events.rs` owns deterministic bounded hooks, action turns, timers, quests and endings.
The runner processes successful action hooks before timer ticks, then turn hooks;
reading, cancellation and manual save/load are free. runner/turn.rs settles status
and reward consequences, retaliation and events inside the original command's
checkpoint. The runner restores GameState and its EncounterRng position on a rule
failure; mutation output is staged until success. Input preparation remains live.
See [turn transactions](TURN_TRANSACTIONS.md) and [event contract](EVENTS.md).
`objects.rs` owns placements/quantities, container state, visibility and atomic transfers.
There is one canonical mutable placement ledger; unique items are one-unit stacks. `rules.rs` owns typed nested conditions, atomic effect application,
ordered outcomes and initial declarations from rules.toml. Existing interactions,
prose/art and exit rules share the same conditions. `diagnostics.rs` supplies advisory
checks without rejecting intentional offstage content. Nested references and declaration
keys are covered by schema-aware migration and Snapshot/export inventory.

- adventure.rs assembles story, FIG, RPG, encounter, wallet and shop validation for CLI,
  startup, creator and export. Fresh play never silently loads a save.
- startup.rs discovers local adventures and reuses existing creator sessions and gameplay.
- terminal.rs wraps CLI stdin with memory-only terminal editing. Generic BufRead/Write
  APIs and redirected input retain scripted operation.
- actions.rs loads bounded authored command patterns, resolves immutable object bindings
  and invokes shared effects. Built-in verbs remain reserved; normal item use retains priority.
- parser.rs handles ordinary intent grammar. resolution.rs performs pure eligible
  ID/name/alias matching and bounded advisory typo matching. player_input.rs clarifies
  references using immutable state before runner/engine/combat/RPG validation and mutation.
- render.rs and domain renderers produce output and configured terminology. No game prose
  or puzzle logic is hardcoded in parser rules.
- Creator editors preserve exact previews and explicit Save through Snapshot's stale
  checks, staging, full validation and rollback/recovery. Ordinary edits keep IDs stable;
  creator/migration.rs uses explicit schema paths for deliberate cross-file renames.
  creator/binding_edit.rs authors existing shop bindings without duplicating transactions.
- room_transaction/recovery.rs extends Snapshot with a versioned hashed journal and
  retained baseline. Confirmed recovery validates complete inventories before restoring
  or completing; older/corrupt/missing evidence and external changes are refused.
- creator/export.rs preflights immutable content/runtime bytes, validates copied adventures,
  checks platform headers/build metadata/SHA-256 and publishes only to a new directory.
  archive.rs writes bounded stored ZIP entries; export_workspace.rs permits cleanup
  only of recorded unchanged staging files. Both folders and ZIPs stage before publication.
  It excludes saves/development files and marks failed publication incomplete. It never
  compiles, downloads or executes foreign runtimes. Metadata is not an authenticity signature.

See [Phase 3 review](PHASE_3_REVIEW.md) and [installation/export](INSTALLATION.md) for exact
behavior and limitations. The portable workflow emits native Windows/Linux development
artifacts; it never publishes a release. Content, configuration and manual saves remain separate.

## Historical implementation notes

The following records earlier stage boundaries, not current missing features.

## Historical Phase 2B creator navigation

`creator/session.rs` holds the selected absolute project path and borrows one input/output pair through setup, menu actions and gameplay. `creator` opens Create/Open navigation; `creator PATH` opens a project directly; interactive CLI `new` enters the session after setup. The public setup-only `new_game_wizard` API and noninteractive generators retain their behavior. No-argument CLI execution still runs the engine fixture.

`adventure.rs` assembles the existing story/FIG, RPG, encounters, loot, wallet and shop validators once for CLI play, CLI validation and creator actions. It holds immutable loaded content and delegates gameplay to `runner::run_configured_with_io`. No player-parser, combat, progression, or save changes. Every creator validation/play reloads files. Each play begins a fresh in-memory state, with manual load/save available through the unchanged runner.

`creator/tree.rs` inspects recognized paths read-only, without following symlinks or Windows reparse points. Tree output and traversal are bounded. Unsafe/incompletely inspected projects can be viewed but cannot be played/validated through the creator. Root links are rejected before opening. Invalid TOML can still be opened to receive existing loader diagnostics. The session writes no existing project files and creates no configuration files.

Existing-file persistence belongs to Phase 2C and later. Its review gate must cover comment/unknown-content preservation (or a clearly offered nondestructive alternative), original-byte comparison to detect external edits, exact previews and confirmation, full staged validation before publication, and tested rollback/recovery for changes spanning several files. The exclusive-new-file writer is not an existing-file editing implementation.

## Phase 2C room drafts and publication (shared foundation)

`creator/room_edit.rs` owns a room draft set keyed by stable IDs, with real filenames and original documents. The editor reuses `Wizard` prompts and `DraftFile` previews. `toml_edit` 0.22 is now a direct dependency (it was already a transitive dependency of toml 0.8); it preserves tables, comments, inline exits and advanced properties while only selected values change. Value decorations are copied when replacing scalars. Unchanged documents are not published; CRLF style is retained for changed CRLF documents. Modified scalar quoting may change. Existing IDs are not renamed.

`creator/room_transaction.rs` captures recognized project files, bytes, modification times and permissions after safe-path inspection. Limits are 8 MiB per file, 64 MiB total and the existing traversal bounds. Save reserves an exclusive `.lupine-room-edit/` directory, stages a private proposed project and runs `Adventure::load` against it. Original bytes/metadata and directory inventory are rechecked before publication. Each affected existing room is renamed into `original/` and the complete synced proposed file is exclusively hard-linked into place. This does not truncate an authored file or overwrite a destination created concurrently. Permissions are preserved. A full post-publication load verifies the result.

On handled failure, published bytes are compared before removal, and retained originals are restored with exclusive hard links. External changes are never overwritten during rollback; rollback conflicts retain recovery data and return explicit instructions. Tests inject failures before later publications and cause a real filesystem destination collision after an earlier file is published. Cleanup removes only transaction-owned files and empty directories, never recursively deleting unexpected files.

This is recoverable multi-file publication, not a crash-atomic transaction. A crash/power loss can leave a partially published project. RECOVERY.txt and CHANGES.txt describe retained originals and targets; COMMITTED.txt marks a fully published/validated result awaiting cleanup. A remaining recovery folder blocks further room editing. Recovery is deliberate/manual; neither automatic rollback at startup nor automatic resume is implemented. Hard-link support is required. Filesystem checks and the cooperative edit lock do not prevent hostile concurrent path swaps, and directory-entry durability on power loss is not guaranteed. Do not play or concurrently edit while publication/recovery is in progress.

Preexisting invalid projects cannot be edited through the room manager; staged-validation failures are labeled proposed errors. No item/spell/art/FIG/puzzle editor, runtime parser changes, or RORK writes are introduced. The shared runtime loader remains the source of truth.


## Phase 2C.2 FIG activation and generated defaults

GameConfig reads character.fig as a TOML table, consumes the single boolean enabled key, and passes the remaining fields through the existing strict Progression deserializer/validator. Disabled files still validate; only enabled progression becomes GameConfig.character. Missing-key enabled semantics preserve older authored FIGs without a second format loader. Runtime, CLI validation and creator operations share that resolved Option, so generation does not accidentally activate RPG. No configuration fields are added to player state or saves.

The three commented FIGs under templates/starter/config are included verbatim as DraftFile content by the generator. All nine standard folders are created/tracked by the existing exclusive writer and rollback path. Both preview and write consume the same text. Root-level supported FIG names report placement guidance; supported filenames and value schemas otherwise retain strict validation.


## Current Phase 2D–2F editor architecture

`content_edit.rs` shares comment-preserving Document drafts, exact previews, object selection, confirmation and item/spell manager navigation. `spell_edit.rs`, `art_edit.rs`, `interaction_edit.rs`, `room_advanced.rs` and `fig_edit.rs` expose current runtime fields through Wizard prompts. `rules_ui.rs` edits existing table/inline-array collections and condition lists without a competing schema. Priority changes refresh TOML table positions so serialized order agrees with the draft.

The existing `room_transaction.rs` Snapshot is now the one publication boundary for rooms, items, spells, supported FIGs, interactions.toml and artwork. A strict target allowlist plus normal relative path components prevents arbitrary writes; nested artwork edits require an existing snapshotted file. Missing direct content directories are created only during a confirmed transaction and removed on handled rollback when still empty. Root rules and all related files share the same staged/full validation, stale checks, permissions, rollback and recovery behavior. The historical `.lupine-room-edit/` lock name remains unchanged for discoverable recovery. Unreferenced edited artwork is explicitly validated with the runtime artwork validator too.

FIG defaults and field explanations come from bundled commented templates; runtime validators remain authoritative for constraints. Existing omitted fields are preserved. Sibling game-ID discovery is advisory and limited to readable regular neighboring adventures; unreadable/linked siblings produce guidance rather than blocking a safe new destination. Destination checks remain strict. No files change on open or mere navigation.

Existing invalid projects must be repaired before editing. Exact IDs are not renamed, unsupported representations are refused, and crash recovery is still manual. New tests cover mixed-category rollback and real CLI authoring/play/save/reopen. The only runtime behavior fix is case normalization of authored object names/aliases during resolution, required for capitalized creator names; parser grammar/ambiguity semantics are unchanged. No save versions, player-save triggers or RPG arithmetic changed.

## Project drafts and publication metadata

`creator/project_edit.rs` uses one Snapshot and `Option<String>` file proposals: Some writes, None deletes. Existing write-only callers delegate to this shared transaction. Proposed projects must load before any publication. Recovery journal v2 includes baseline validity and absent-file inventories, allowing exact restoration of invalid projects under repair. Foreign changes are never overwritten. `story_loader::Metadata` is immutable game publication information, excluded from mutable state and RORK revisioning. Export generates separate authored publication documents.

Room-centered navigation (`creator/room_hub.rs`) holds no mutable draft between editors. It reloads saved content after each editor and uses schema-typed references. Shared selectors paginate/filter without mutating content. Runtime discovery checks only two known relative locations and reuses existing binary verification.

NPC definition files (0.11.0-dev.9): dialogue::Content loads one strict definition
per NPCs/*.toml, removing its top-level id into the existing NPC map. Runtime and
RORK 7 state remain keyed by authored NPC/node IDs, never paths. Snapshot, export,
inspection and schema-aware migration include these files. The old dialogue.toml
is rejected by the loader but retained as an editor repair/deletion target.
