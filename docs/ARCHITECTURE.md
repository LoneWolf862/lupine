# Architecture Notes — Story-Agnostic Rust Interactive Fiction Engine

## Guiding principle

Build a **small deterministic interpreter for player commands**, not an AI chatbot. The engine interprets supported sentence patterns, resolves entities in context, checks the rules, then changes game state. Keep the story separate.

## Input pipeline

```text
stdin / terminal prompt
        |
        v
normalize (case, whitespace, simple filler words)
        |
        v
parse (verb, targets, prepositions) --> typed Command
        |
        v
resolve object aliases against visible/inventory scope
        |
        v
validate world rules and preconditions
        |
        v
execute action / mutate GameState
        |
        v
format outcome / render in terminal
```

**Key boundary:** The parser decides what the player *asked*; the rules decide whether it is *possible*. `take boulder` should parse successfully but fail because the boulder is fixed.

## Suggested data model

- `World`: immutable definitions for rooms, exits, objects, aliases, and story text.
- `Room`: ID, display name, description, exit mappings, object IDs or initial placements.
- `Item`: ID, name, aliases, examine text, basic properties.
- `GameState`: current room, inventory, object placements, and mutable flags.
- `Command`: an `enum` of intentions (`Look`, `Move(Direction)`, `Examine(Target)`, `Take(Target)`, `Drop(Target)`, `Use { item, target }`, `Help`, `Quit`, etc.).
- `ActionOutcome`: success/failure plus player-visible text; errors can use `Result` or typed variants.

Use stable identifiers internally so that renaming an item doesn't corrupt location or flags. `HashMap` is fine when it helps; ensure a **real `Vec<T>`** also appears in the code for the Rust rubric.

## Recommended module layout (flexible)

```text
src/
  main.rs         # terminal loop, startup and shutdown
  lib.rs          # exports core modules for integration tests
  parser.rs       # normalization, grammar and synonym handling
  commands.rs     # Command and action types
  world.rs        # room/item definitions and fixture world
  state.rs        # mutable state and object locations
  engine.rs       # entity resolution, checks, state transitions
  render.rs       # consistent terminal output
 tests/
   gameplay.rs    # scripted end-to-end game flows
```

Do not create modules solely for their own sake. Collapse files if a smaller layout is clearer.

## Parser behavior contract

| Player input | Intended interpretation |
|---|---|
| `l` / `LOOK` / `look` | `Look` |
| `n` / `north` / `go north` | `Move(North)` |
| `i` / `inv` / `inventory` | `Inventory` |
| `x bronze lever` / `examine the bronze lever` | `Examine("bronze lever")` |
| `get key` / `pick up the key` | `Take("key")` |
| `use key on lock` | `Use { item: "key", target: "lock" }` |
| `xyzzything` | Unknown-command feedback, not a crash |
| `take` | Missing-target guidance |

Commands are **illustrative grammar**, not a promise that any specific item will appear in the actual story.

### Rules for ambiguity

1. Exact alias/name match before partial matching.
2. Prefer objects in the current visible room or inventory, depending on the verb.
3. If multiple valid entities remain, ask the player to specify; don't silently guess.
4. Never treat an unmatched word as proof that a plausible action occurred.

## Content independence

For the first milestone, constructing a small fixture world in Rust is acceptable. Design it so another `World` implementation or loader can replace it. External data files are an optional follow-on; do not build an elaborate scripting language before the core engine works.

## Saving and release

- Saving/loading is an optional next milestone. If implemented, save only mutable state, with a schema version; never require internet.
- Developer builds use Cargo. **Players should run a packaged binary without Cargo/Rust installed**. Cross-platform support means producing platform-specific binaries, not assuming a single binary runs on all operating systems.
- Bundle all required story assets locally and test in an environment with networking disabled.

## Suggested test boundaries

- **Parser:** synonyms, articles, capitalization, spaces, verb/object/preposition forms, bad input.
- **Resolver:** multiword names, aliases, ambiguity, inventory-vs-room visibility.
- **World rules:** fixed item cannot be taken, blocked exit cannot be traversed, absent item cannot be used.
- **State:** take/drop, move, inspect, stateful action persisted across subsequent commands.
- **Integration:** launch fixture world; run a short scripted sequence; assert state changes and responses.

## Implementation order

1. Cargo project + simple terminal loop + `help`/`quit`.
2. World/state types + room traversal and `look`.
3. Typed parser with aliases and friendly errors.
4. Item resolution, inspect, take, drop, inventory.
5. One reusable stateful interaction + tests.
6. Documentation, offline release test, course demo.
7. Only then consider save/load, fuzzy matching, data-driven stories, and extensive content.

## Design decisions to leave open

- Story title and setting.
- Whether world content lives in compiled-in Rust definitions or bundled local text/data files.
- Whether save/load is in the first release.
- Whether to add fuzzy spelling matching in a later version.

The next chat should choose minimal initial defaults, record them, and implement the engine incrementally without writing an elaborate story first.
