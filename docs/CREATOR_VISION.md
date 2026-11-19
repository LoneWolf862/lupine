# Lupine Creator Vision — Nontechnical Game Authoring

Current direction (2026-11-19): direct TOML/FIG authoring is the primary development
workflow. Engine capability takes priority over wizard coverage, which takes priority
over wizard convenience. See [current reference](CONTENT_REFERENCE.md) and
[approved expansion](ENGINE_EXPANSION.md). The final accessibility goal remains.

> Historical design/stage record: feature-status and compatibility claims below describe earlier development. Current policy and file paths are in [NEXT_PHASE_REQUIREMENTS.md](NEXT_PHASE_REQUIREMENTS.md) and [CONFIG_FIG_SPEC.md](CONFIG_FIG_SPEC.md). Character configuration now belongs in `config/character.fig`.

**Status:** Final-release product goal; design direction, **not implemented** and **not an MVP/course acceptance gate**.  
**Project:** Lupine (engine); HOWLBOUND (tentative first game and authoring-system test case).

## Purpose

A person with **no Rust experience and no programming background** should be able to create and publish a complete playable Lupine adventure. Such creators should not have to modify engine source code, compile Rust themselves, learn an API, or hand-debug parser internals. They should be free to focus on writing and designing the adventure.

This goal is compatible with a small, deterministic, terminal-only, fully offline **player runtime**. Authoring tooling may have a different interface and footprint from the shipped game.

## Three audiences

| Audience | Goal | Expected workflow at final release |
|---|---|---|
| Player | Play an adventure | Launch a complete, offline, platform-specific game package; no toolchain, login, or internet |
| Story creator | Make an adventure | Start a template, edit rooms/objects/rules, get actionable errors, test, and export without Rust |
| Rust developer | Extend the engine | Use documented library APIs and extend mechanics without forking parser logic |

## Creator journey (acceptance criteria for the final creator release)

A first-time creator should be able to:

1. **Start** a new adventure from an approachable template, using a documented or guided process.
2. **Define** a title, opening room, rooms/exits, objects, descriptions, and common interactive behaviors without Rust.
3. **Connect and revise** locations, objects, and basic puzzle conditions/effects without manually changing engine source.
4. **Validate** their work before play: missing room references, duplicate IDs, invalid interactions, unreachable starting locations, and malformed content produce readable, location-specific diagnostics with suggested fixes where possible.
5. **Playtest** instantly and repeatedly using the normal Lupine player engine, with deterministic behavior.
6. **Package/export** an offline, immediately playable game for a supported OS using an understandable workflow; the *player* does not need Rust/Cargo. (Packaging systems and supported OS coverage remain to be designed.)
7. **Learn** from a starter project, a short end-to-end tutorial, examples, and a clear command reference.

The final user experience should be assessed with at least one person who has little or no programming experience completing a short new-game walkthrough with minimal assistance. Exact usability targets and distribution details can be set later.

## Content architecture direction

- Use **stable IDs** for rooms, items, interactions, and state flags; names are player-facing display text.
- Keep **immutable content** (descriptions, exits, aliases, definitions, rules) separate from **mutable play state** (location, inventory, open/locked flags, puzzle outcomes).
- Prefer a **data-driven content model** that can be represented in a human-readable local format and validated before play.
- Define reusable **conditions** (e.g., player carries item; flag is set; target is accessible) and **effects** (e.g., set flag; change object state; unlock exit; show response), not arbitrary story-specific Rust branches.
- Allow **state-dependent descriptions** so `look`/`examine` reflect the outcome of actions.
- Give creators sensible defaults for ordinary interactions; more sophisticated mechanics can remain optional.
- Keep the *runtime* story-independent. Separate any future editor from the game executable.

**TOML is approved for initial authoring.** Creators use example/template files and optional future terminal commands; no graphical editor is planned. The example below illustrates the approved v1 TOML schema; the loader is pending v0.5.0-B testing:

```toml
# Illustrative only — there is no Lupine story-file loader yet.
id = "forest_clearing"
title = "Forest Clearing"
description = "Tall pines surround a quiet clearing."
[exits]
north = "cabin"
```

## Authoring interface — text only

- Primary workflow: create and edit readable TOML story files using any text editor.
- Future terminal commands may generate templates, validate references, launch a playtest or package releases.
- No graphical editor, room-map GUI or Lupine Studio application is planned.
- Nontechnical accessibility will be measured through examples, error messages and beginner usability checks; no Rust should be needed for authoring or distributing a game at final release.

## HOWLBOUND as a real-world test

HOWLBOUND should be authored as a separate game using public Lupine mechanisms and, when available, the same game-creation workflow offered to others. Its story is undecided. Any gap that forces a unique engine change should be assessed as a possible missing reusable feature, not immediately patched as special-case HOWLBOUND logic.

## Explicitly out of scope for the current milestone

- A functioning TOML/JSON/RON loader or fixed schema.
- Any graphical editor or visual map editor.
- One-click cross-platform compiler/packager.
- NPC dialogue trees, combat, quests, broad scripting languages, or cloud collaboration.

**Exit condition:** The final release is *not* complete solely because the Rust library is extensible. It must demonstrate the nontechnical creator workflow described above, in addition to reliable offline gameplay.

## Text-only creator workflow and manual save design (confirmed)

Lupine will remain text-based throughout: games play in the terminal, game content is authored using TOML templates and examples, and future project creation/validation tools are terminal commands. A graphical Lupine Studio is **not** a planned requirement.

Game creators do not implement saving. Lupine provides engine-owned manual `save`, `load`, and `saves` commands, with multiple named slots and portable TOML-based `.rork` files. No autosaving, save-on-quit or checkpoints. Any game creation templates should have a stable `game_id` to isolate save files and keep them compatible across title changes.
