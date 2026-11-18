# Lupine Text Creator Wizard — Expansion Specification

## Current pre-release policy (2026-11-18)

Lupine is unreleased and pre-1.0, with no external users, published games or production deployments requiring development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Phase 1 relocates existing character fields/defaults/calculations without redesigning progression, adds display terminology, and updates creator guidance. No legacy `character.toml` loader. No save-format change is needed. Full wizard and parser upgrades are later phases.


**Status: Phase 2A.1 setup and Phase 2B navigation merged; Phase 2C room editing implemented for review. Later content editors remain proposed.** `lupine new` and `add room`, `add item`, `add art` already exist. Enhance them rather than replacing them.

## Learning philosophy

**Teach by creating, not by lecturing.** A first-time user should make playable game content, see how Lupine represents it in TOML and `.fig`, and learn enough to edit those files afterward. All help is optional, concise and grounded in real keys that the engine understands. Experienced creators should be able to work quickly without repetitive tutorials.

## Implemented interaction modes (Phase 2A.1)

- **Beginner (default):** Enter begins a four-step explained flow. Internal IDs are generated; a plain-language summary offers creation, optional exact file previews, revision or cancellation.
- **Advanced:** type `advanced` at the welcome prompt for shorter field prompts, editable IDs and direct exact previews.

Enter accepts suggestions; `?` gives help; `back` returns to the preceding question; `cancel` stops. Review can revisit all answers without losing them. Prefix literal control words with a backslash. Multiline descriptions remain available through `/multi`, with instructions given on entry. `skip` and broader authoring modes remain proposed.

## Author journey

1. **Create adventure:** directory, permanent game ID, display title, starting room. Explain why IDs must remain stable and titles may change.
2. **Create first room:** title, suggested slug ID, description, optional art. Preview generated `rooms/<id>.toml`.
3. **Add a room and connect exits:** choose direction and existing/new destination. Offer a reverse exit, with explicit one-way option. Preview `[exits]` and, optionally, an ASCII connection diagram.
4. **Create item:** ID, display name, aliases, description, portable flag, starting room. Preview generated `items/<id>.toml`.
5. **Create first interaction/puzzle:** guide a simple prerequisite, effect and response using the **actual existing** `interactions.toml` schema. Explain what happens when preconditions fail and how a flag changes behavior. Preview generated interaction TOML.
6. **Optional FIG configuration:** allow defaults or selective customization. Explain which categories change game flavor or tuning, showing generated `.fig` content.
7. **Validate/play:** invoke the actual validator, explain errors with file/key names, show exact `lupine play <path>` command, and remind authors they can edit TOML/FIG in a text editor at any time.

The basic path must be skippable: creators should still be able to generate a minimal one-room game and edit by hand.

## Exact preview behavior

When requested in Beginner mode, and directly in Advanced mode, show the destination path and precisely the representation that **the same serializer will write**, or a clearly labeled subset of it. Do not implement a separately hand-formatted preview that can drift from the output. Explain a key only when useful, e.g. "`id` is the stable internal name; `title` is what the player sees." Do not present nonfunctional example syntax as actual output.

Illustrative interaction (must be adapted to the real schema):

```text
Room title [Starting Room]: Abandoned Laboratory
Suggested internal ID [abandoned_laboratory]:
Description: A cold room full of flickering monitors.

Preview: rooms/abandoned_laboratory.toml
id = "abandoned_laboratory"
title = "Abandoned Laboratory"
description = "A cold room full of flickering monitors."
[exits]

Create this file? [Y/n]
```

## Safety and validation

- Reuse existing validators, IDs, schema, and files rather than create a second format.
- Existing files are **never** overwritten silently, even when restarting a partially completed session.
- Validate input, directories, symlinks/path traversal, IDs and references before writes.
- Prefer an atomic/staged multi-file commit or a clear recoverable sequence. Never delete content an author edited before a wizard was canceled.
- A generated room exit should never permanently point to a nonexistent room unless explicitly treated as an incomplete draft with a clear diagnostic.
- Failed steps must not create broken objects or damage an otherwise valid project.
- Wizard file creation is distinct from gameplay saves; **player autosaving remains forbidden**.

## Acceptance scenarios

A novice creates a two-room world, connects and optionally unlinks/reorients exits, makes a portable object, sets up an actual supported puzzle and sees it work in play. The user can find and manually edit every generated file. Test quick/guided/default modes, strange titles, multiline descriptions, invalid IDs, reverse exits, missing references, cancellation, interruptions, and preexisting files. Output stays entirely in the terminal.

## Implementation starting points

Inspect `src/creator.rs`, `src/main.rs`, `src/story_loader.rs`, `templates/starter/`, and wizard/creator tests. Share serialization/validation with manual authoring. Use the existing engine condition/effect vocabulary for puzzles; do not invent puzzle syntax or introduce special-case story logic in Rust.

## Phase 2A.1 implementation boundary

`new` guides creators through title, location, starting room and description. Beginner mode automatically generates a stable ID; advanced mode permits customization. Shared prompts, serialized drafts and safe writes remain under `src/creator/`, with the mode/step flow in `setup.rs`. Local sibling-ID checks and folder suffix suggestions prevent common collisions; no global registry is claimed. The resolved destination appears in review. Preview and written bytes use the same `DraftFile` contents.

Generated projects pass the story loader. Completion provides Cargo and exact executable commands with platform-specific shell quoting, without guessing how Lupine was launched. Creation does not automatically launch play. See CREATOR_QUICKSTART.md for navigation, quoting, local collision scope, safety limits and a Windows manual walkthrough. Full guided room/item/puzzle/config workflows remain future work. No dependency, gameplay or save-format changes.

## Phase 2B - persistent session (implemented)

After interactive CLI `new`, the actual project tree and creator menu appear. `creator` offers Create/Open; `creator PATH` reopens an adventure. Option 1 runs the existing runner in-process, returning after quit. Option 2 invokes the same full loader/validator as CLI validation, displaying counts or its diagnostic without changing content. Option 8/help explains navigation. Enter refreshes; 0/back/cancel leaves the project (returning to the launcher when entered through it); EOF exits cleanly.

Phase 2B initially reserved options 3-7. Phase 2C now implements option 7; item/spell/art/FIG editors remain unavailable. The selected project persists through menu actions; tree and content are reloaded after external edits. No automatic load/save occurs. See CREATOR_QUICKSTART.md for file-tree limits and the manual walkthrough.

Foundation PR #2 and navigation PR #3 are now merged. Phase 2C starts from the approved main branch and has its own review PR.

## Phase 2C - guided rooms (implemented candidate)

Option 7 discovers actual room definitions and offers N for new rooms or a numbered existing room. Creation explains display name, suggested/customizable internal ID, multiline description and directional connections. Existing room IDs are fixed; display names/descriptions can change. `{{objects}}` is explained without changing its runtime behavior.

Connections can be authored from an existing room into the selected room or outward from it. The six existing directions are supported, with an optional reverse exit, multiple exits, explicit conflict replacement, and explicit outgoing-exit removal. Removal does not silently remove the reverse edge or conditional exit rules; the staged validator refuses resulting invalid worlds. Disconnected locations are allowed with an unreachable warning. Advanced descriptions/art/encounters and authored comments are preserved rather than rebuilt from a reduced schema.

Room Options provides name/description edits, connections, properties, exact previews, Save and discard. Save summarizes the selected room and affected files, then requires explicit yes (default no). Every affected file is serialized once for preview/publication. Changed files are staged with the rest of the project and pass the existing complete validator before publication; content is validated again afterward. External changes abort the save. Successful operations refresh the real tree and return to Room Manager; selecting a room again allows previewing the saved content. Cancel/back/EOF discard unsaved drafts.

The transaction and recovery contract is documented in ARCHITECTURE.md and CREATOR_QUICKSTART.md. Preexisting invalid projects are refused for editing, not repaired or stripped. Room-ID renaming, full advanced-property editors, and cross-file puzzle migrations remain out of scope. No engine or save version changes.


## Phase 2C.1 - presentation polish (implemented candidate)

The creator uses shared ASCII headings and separated label/input prompts, with unchanged navigation and defaults. The file tree renders real nested directories and basename-only children rather than flattened paths, retaining bounded inspection, skipped-link diagnostics and recognized-content filtering. Root category order stays fixed; children are sorted by name. Room connections show From/To/direction and reverse proposals, with incoming/outgoing summaries and one-way/bidirectional labels. Save review is concise; detailed properties and exact TOML previews remain available. No new editor behavior, dependencies, engine version or save behavior is introduced. See CREATOR_QUICKSTART.md for presentation limits and Windows review steps.

<!-- lupine-phase-2c2-spec -->
## Phase 2C.2 — generated project structure and configuration (implemented candidate)

New adventures should expose actual file structure and **self-documenting** supported FIGs, not require users to discover defaults in separate documentation. Create standard empty content directories, emit commented supported defaults, and keep narrative-only adventures narrative-only via an explicit RPG opt-in setting. See [`SELF_DOCUMENTING_CONFIG_SPEC.md`](SELF_DOCUMENTING_CONFIG_SPEC.md). Future `.fig` categories are not valid placeholders.

The implementation emits both FIGs in the same DraftFile preview/write path as TOML. Every new project includes config, rooms, items, spells, weapons, enemies, shops and art directories. Manual changes to FIGs apply on validation/play reload; a guided FIG value editor remains Phase 2E. Blank Open Adventure paths now explain which folder to enter and retry without a technical path error.

## Phase 2D-A candidate

Item Manager is implemented with confirmed validated transactions, stable IDs, aliases and comment-preserving field edits. See CREATOR_QUICKSTART.md and CODEX_HANDOFF.md. Remaining editors are pending.


## Phase 2D-B spell editor candidate

Menu 5 creates/edits every current spell field: stable ID (creation only), display name, description, MP cost, kind, positive power and optional teaching item. Damage targets an enemy; heal/restore_mana affect the player. No spell damage-type field exists. Teaching items are selected from actual items, must be portable in the guided workflow and are consumed by learn. RPG enablement is explicit; disabled projects receive guidance without modification.

Branch codex/phase-2d-spells stacks on codex/phase-2d-items (PR #8). Windows: 181 tests pass, fmt and strict Clippy pass; native Linux/human usability remain pending. Self-review found that declining Save discarded the draft; both item/spell forms now retain it so authors can revise or confirm later. Tests exercise this, learning, manual save/load, field ranges and preserved comments/IDs. Overall completion remains pending artwork, puzzles, FIG and integration.


## Phase 2D-C artwork editor candidate (2026-11-18)

Menu 6 lists existing artwork (including nested files), creates/edits multiline text, previews exact whitespace and attaches/removes room defaults and conditional variants. First-match variants use guided flag-set, flag-unset or carried-item requirements. Existing inline artwork is replaced only after explicit confirmation. Existing variant tables and inline-array representations are preserved. No artwork is stored in RORK.

Branch codex/phase-2d-artwork stacks on codex/phase-2d-spells (#9), following items #8. Windows: 184 tests, fmt and strict Clippy pass. Self-review identified unreferenced art escaping loader validation; the transaction now calls the same runtime artwork validator even before a file is referenced. Tests cover spaces, blank lines, backslashes, nested files, cancellation, terminal controls, dynamic references and removal. Linux/human usability remain pending. Recovery remains manual; no versions or autosaving changed.


## Phase 2E-A interactions and advanced rooms candidate (2026-11-18)

Main menu 9 opens existing interaction/exit-rule forms and a locked-door recipe. Rules expose used item, target, all existing conditions/effects and outcome text; existing lists/comments survive selected edits. The door workflow can create key/door items and publishes them with rules in one validated recoverable transaction. Room Options 7 adds/edits/removes conditional descriptions and edits encounter chance, counts, persistence, weighted existing enemies and flag requirements. It does not create new enemy definitions.

Branch codex/phase-2e-interactions stacks on artwork #10 (after items #8 and spells #9). Windows: 189 tests, fmt and strict Clippy pass. Scripted acceptance creates connected rooms, a door and key, alternate prose, travels after unlocking and explicitly saves/loads. Cancellation publishes none of the related files. Existing conditional lists/comments and encounter choices survive edits.

Self-review uncovered a real authoring blocker: mixed-case display names without aliases could not be resolved against lowercased player input. The minimal engine fix normalizes authored names/aliases for comparison, preserving exact-match priority and ambiguity. This is covered with uppercase input and ambiguous keys; no parser redesign or new mechanic. An existing navigation test was updated for the intentional ninth menu option. Native Linux and human usability remain pending. No package/RORK versions changed, no autosaving.
