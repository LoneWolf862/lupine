# Lupine Player Parser — Forgiving, Deterministic Input

**Status: Planned improvements; do not replace existing parser.**

## Player-facing objective

Accept sensible alternative wording and return useful feedback without demanding exact "magic phrases." Keep processing deterministic and local; **no LLM**. A parsed intent still needs world-rule validation before mutation.

## Observed current baseline

In current `src/parser.rs`, basic commands already normalize whitespace/case, ignore `the/a/an` for item targets, support directions and aliases (`take/get/grab/pick up`, `examine/inspect/look at`, etc.), and parse `use X on Y`. Other optional RPG gameplay commands are handled elsewhere. Existing behavior and tests must continue to work.

## Work priorities

### 1. Consistent parsing and object resolution

- Preserve and extend reasonable synonyms, articles, multiword item names and input patterns supported by actual gameplay.
- Search relevant **visible room and inventory** objects by action role. Prefer exact ID/name/alias matches; only then consider suitable partial matches.
- When more than one candidate remains, ask a concise numbered clarification instead of guessing. Do **not** execute until the choice is resolved and revalidated.
- Give distinct helpful responses for unknown verb, missing object, ambiguous reference, inaccessible object, impossible state, invalid direction, and unsupported action.
- Audit both basic command parser and optional RPG commands, so the user experience remains consistent.

### 2. Typos and suggestions

- Detect likely misspelled verbs or object names with bounded deterministic matching; suggest a correct command.
- A fuzzy suggestion is **not permission to execute**. Err conservatively for `drop all`, purchases, combat, consumables, and save overwrites.
- Do not suggest secret/hidden objects not currently visible, and do not auto-correct into a different destructive command.

### 3. Later optional refinements

Pronouns like `it`, richer context help, and chained commands such as `take key then open door` are useful but not required to finish the first parser upgrade. If added, preserve explicit sequencing and cancellation semantics.

## Examples (not assertions of current behavior)

```text
> PICK  UP the brass key
You take the brass key.

> take key
Which key?
1. brass key
2. iron key
> 1
You take the brass key.

> exmain door
Did you mean "examine door"?
```

The ambiguous example assumes both keys are visible and applicable. For a single visible key, no clarification is needed.

## Separation of responsibilities

```text
input -> normalize -> identify typed intent -> resolve targets in current world
      -> validate action rules -> mutate state -> render result
```

Do not place hardcoded story objects in grammar or modify story content to compensate for parser weaknesses. Coordinate display terminology from FIG with vocabulary aliases explicitly; renaming a label must not silently remove a longstanding command.

## Required tests

Test case/whitespace/articles, aliases, multiword names, exact-before-partial matching, zero/one/multiple visible candidates, carried versus room targets, canceled/invalid clarification, misspelled verbs and objects, reserved verbs, EOF, and old RPG command paths. **Assert no game-state mutation on failed parses, ambiguity, cancellation, or unconfirmed suggestions.** Existing saves, combat, shop and interaction regression suites must remain green.

## Implementation entry points

Read `src/parser.rs`, `src/commands.rs`, `src/engine.rs`, `src/runner.rs`, and existing parser/gameplay/RPG tests. Begin with a read-only audit to determine where parsing, target resolution and RPG dispatch actually live before changing structure.
