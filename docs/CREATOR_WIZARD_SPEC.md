# Lupine Text Creator Wizard — Expansion Specification

## Current pre-release policy (2026-11-18)

Lupine is unreleased and pre-1.0, with no external users, published games or production deployments requiring development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Phase 1 relocates existing character fields/defaults/calculations without redesigning progression, adds display terminology, and updates creator guidance. No legacy `character.toml` loader. No save-format change is needed. Full wizard and parser upgrades are later phases.


**Status: Phase 2A foundations implemented; broader authoring journey below remains proposed.** `lupine new` and `add room`, `add item`, `add art` already exist. Enhance them rather than replacing them.

## Learning philosophy

**Teach by creating, not by lecturing.** A first-time user should make playable game content, see how Lupine represents it in TOML and `.fig`, and learn enough to edit those files afterward. All help is optional, concise and grounded in real keys that the engine understands. Experienced creators should be able to work quickly without repetitive tutorials.

## Interaction modes

- **Standard (default):** short explanations, sample inputs, file preview.
- **Guided:** more explanatory hints and walkthroughs.
- **Quick:** minimal prompts, still performs validation, confirmation, and safe writes.

Suggested controls: Enter accepts defaults; `?` shows more explanation; `back` returns to the previous question when safe; `skip` omits optional steps; `cancel` aborts without clobbering previous work. Decide how to escape these controls when entering free-form story text.

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

When a step has collected inputs, show the destination path and precisely the representation that **the same serializer will write**, or a clearly labeled subset of it. Do not implement a separately hand-formatted preview that can drift from the output. Explain a key only when useful, e.g. "`id` is the stable internal name; `title` is what the player sees." Do not present nonfunctional example syntax as actual output.

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

## Phase 2A implementation boundary

`new` now collects title, stable ID, location, starting room title and multiline description. Shared prompt, serialization/preview and exclusive staged-write helpers live under `src/creator/`. Preview and disk bytes use the same `DraftFile` contents. Generated projects pass the existing story loader. No dependency or gameplay changes. See CREATOR_QUICKSTART.md for controls, safety and filesystem limitations. Full guided room/item/puzzle/config workflows and quick/guided modes are not implemented in this phase.
