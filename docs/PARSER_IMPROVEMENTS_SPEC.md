# Lupine Player Parser — Forgiving, Deterministic Input

## Policy recorded before Phase 3 (historical)

Lupine is unreleased and pre-1.0, with no external users, published games or production deployments requiring development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Phase 1 relocates existing character fields/defaults/calculations without redesigning progression, adds display terminology, and updates creator guidance. No legacy `character.toml` loader. No save-format change is needed. Full wizard and parser upgrades are later phases.


**Status: Phase 3C is an implemented development candidate.** See [actual behavior and verification](PHASE_3_REVIEW.md). The design below is retained; pronouns and chaining remain future work.

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

Read `src/parser.rs`, `src/commands.rs`, `src/engine.rs`, `src/runner.rs`, and existing parser/gameplay/RPG tests. The Phase 0 audit is complete. Phase 3C adds resolution.rs and player_input.rs before existing action validation.
