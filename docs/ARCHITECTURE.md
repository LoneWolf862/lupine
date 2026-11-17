# Lupine Architecture — Reusable Rust Interactive Fiction Engine

**Status:** v0.2.0 implementation verified on Windows; v0.3.0 architecture proposals identified below.  
**Principle:** The engine interprets supported player commands and applies generic rules. A game supplies content. The parser is **not** an AI chatbot.

## System boundaries

```text
                  PLAYER (terminal)
                         |
                   stdin / output
                         |
                 Lupine terminal app
                         |
     normalize -> parse -> resolve -> validate
                         |
                    apply effects
                         |
                mutable GameState
                         |
                      render

       World definitions / game content (immutable)
              supplied to the engine

   FUTURE: authoring workflow -> validated story data
              -> Lupine game runtime
```

- **Parser** determines what was requested, not whether it is possible.
- **Resolver** maps human references to accessible stable entity IDs and never guesses on genuine ambiguity.
- **Rules** determine whether the action is currently allowed.
- **State transition** applies effects only after prerequisites hold.
- **Renderer** produces concise player-visible output reflecting the state.
- **World content** owns names, aliases, descriptions, room connections, and interaction definitions. The engine must not contain a game's story-specific branching code.

Example: `take boulder` is a valid request, but the rules may reject it because a boulder is fixed.

## Current v0.2.0 modules (implemented)

```text
src/
  main.rs       # terminal loop and fixture-world launch
  lib.rs        # module exports
  parser.rs     # normalize, recognize supported grammar and synonyms
  commands.rs   # typed Command/Direction
  world.rs      # World/Room/Item/Interaction plus fixture_world()
  state.rs      # GameState and mutable item placements / flags
  engine.rs     # resolve, validate and execute commands
  render.rs     # help and room output
tests/
  gameplay.rs   # navigation integration test
  objects.rs    # items, ambiguity, rule, and persistent flag tests
```

Current definitions are compiled in as Rust data: rooms, exits, items (stable string IDs, aliases, portable property), and `Interaction { item, target, flag, success_text }`. State separately stores current room, item locations, and flags. The fixture has three rooms and a key/panel interaction. **There is no external story loader, open/locked object system, conditional description evaluator, or graphical creator yet.**

## Design rules for v0.3.0 (proposed)

1. **Engine/library boundary.** A consumer should be able to supply a `World` and initial `GameState` without modifying Lupine parser, engine, or terminal code. Keep a tiny fixture as example/tests, not a built-in game dependency. Consider a straightforward `Engine` facade only if it reduces API complexity.
2. **Stable IDs and clean state.** Resolve names to IDs; store mutable facts in state, not by rewriting world definitions. Avoid IDs tied to display-name spelling.
3. **Generic preconditions and effects.** Prefer a small, typed vocabulary such as `Condition::HasItem`, `Condition::FlagSet`, `Effect::SetFlag`, or `Effect::UnlockExit`, implemented only as needed. These are **proposed illustrations**, not current Rust APIs. Avoid an arbitrary scripting VM.
4. **State-dependent text.** Rooms, objects, and interactions may select different descriptions based on state. Choose simple, deterministic ordering/fallback rules; test them explicitly.
5. **Validation.** Detect broken world references, duplicate IDs, missing start room, or invalid interaction references before gameplay where practicable. Friendly creator diagnostics become particularly important in later data-driven releases.
6. **Backward compatibility.** Existing v0.2 command behavior should remain tested while the fixture moves into an example or helper module.

## Parser contract

| Input | Meaning |
|---|---|
| `look`, `l`, `LOOK` | Current room |
| `n`, `north`, `go north` | Navigate north |
| `i`, `inv`, `inventory` | Show carried items |
| `x brass key`, `examine the brass key` | Inspect item |
| `get key`, `pick up the key` | Take item |
| `drop key` | Drop a carried item |
| `use key on panel` | Request item/target interaction |
| Unknown verb or missing target | Explain without crashing/mutating state |

Grammar is deliberately constrained: normalization, verb synonyms, articles, multiword names, and simple prepositions; **not arbitrary English**. Exact matches take priority; room/inventory scope is action-aware; unresolved ambiguity triggers a clarification response. Fuzzy typos, pronouns, and chained commands remain optional.

## Content separation and creator-ready direction

- **Today:** compiled Rust world definitions; appropriate for early testing and the course module.
- **v0.3.0:** reusable Rust world construction and generic interaction rules, with examples.
- **Later:** choose a human-readable format, parse/load content, validate references and constraints, provide a template/tutorial and playtest workflow.
- **Final:** an approachable no-Rust/no-programming game creation and packaging path. An optional GUI authoring tool (working name **Lupine Studio**) may be considered, but no decision is final.

A future text format must describe *content* and supported rules; it should not force story authors to write new Rust for ordinary puzzles. Avoid locking in TOML/JSON/RON or schema fields until authoring needs are assessed. See [`CREATOR_VISION.md`](CREATOR_VISION.md).

## Saving, distribution, and portability

- Save/load is optional for the MVP. If added, serialize **mutable state**, with an explicit save schema version.
- Distribute separate platform-specific binaries; the *player* requires no installed Rust or Cargo, internet, or account.
- Story data may be embedded in a game executable or packaged locally; a creator-friendly build/export workflow is a later design task.
- Develop and test on Windows 10 and CachyOS; Linux validation has not happened yet.

## Test boundaries

- Parser: normalization, abbreviations, synonyms, malformed input, multiword phrases.
- Resolver: exact/partial alias selection, scope, ambiguity, inaccessible targets.
- Rules/state: portability, inventory, movement, conditional interaction, repeat action, no mutation on failure.
- World quality: multiple independent worlds, broken references, duplicate IDs, initialization.
- Dynamic narration: correct precondition and fallback descriptions.
- Packaging later: real offline launch without development tools.

## Open choices

The precise v0.3 API, interaction rule variants, content format, save/load milestone, packaging workflow, and authoring interface all remain open. HOWLBOUND's story and final title also remain undecided. Keep changes incremental and testable.

## v0.5.0 architecture and future persistence

The v0.5.0-A transition changes world and mutable state identifiers and text from static string references to owned `String` values. Lookups should borrow with `&str`; the loader should later produce the same `World` type used by Rust-authored games. No story-specific parser changes are needed.

Save/load is an **engine-level** future subsystem. Serialize only `GameState` and the metadata necessary for checking schema and game compatibility. The native `.rork` extension denotes TOML-based manual save files. `save`, `load`, and `saves` will be reserved commands handled by the runner/save subsystem, not by game-specific puzzles. No automatic gameplay writes. Validate the entire loaded state before replacing the active state; use robust file replacement and per-game save storage.
