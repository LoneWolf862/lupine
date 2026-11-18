# `.fig` Game Configuration — Design Specification

**Status: Proposal. Not currently implemented.** All example TOML snippets are *candidate schema*, not working files until a schema is approved and implemented.

## File responsibilities

| Format | Contents |
|---|---|
| `*.toml` | Author-defined story: `game.toml`, rooms, items, spells, enemies, interactions and exits |
| `config/*.fig` | Optional **game-wide** configuration: terminology, adjustable rules, balancing |
| `*.rork` | Existing mutable save state, written **only on explicit player request** |

A `.fig` file uses ordinary TOML syntax and the existing Rust `toml`/`serde` ecosystem. It is a distinctive file extension for creators, not a new custom grammar. The runtime should clearly document how to enable TOML editor syntax highlighting for `.fig`.

## Proposed layout

```text
adventure/
  game.toml
  config/
    terminology.fig
    progression.fig
    gameplay.fig
    combat.fig
    parser.fig
  rooms/
  items/
  interactions.toml
```

The filenames are candidates, not mandatory. Do not manufacture empty files or expose settings that the engine does not actually implement. Simple games can rely on defaults without any `config/` directory.

## Configuration resolution contract

1. Load **versioned, documented engine defaults**, preserving current v0.8.4 behavior.
2. Read each optional recognized `.fig` file; merge defined keys only, leave omitted keys at defaults.
3. Validate TOML parsing, key names, data types, ranges, references, duplicate/conflicting aliases, unsupported mechanics, and overflow before gameplay starts.
4. Report errors with configuration file path, table/key and actionable explanation. Do not silently ignore misspelled keys.
5. Store resolved settings as read-only configuration available to runtime systems; never mix it into mutable player state unless a specific game mechanic requires careful save-compatibility planning.
6. Keep stable item/command/save IDs separate from player-facing terminology. Existing core commands should continue working after display renaming.
7. Apply config per game, with no leakage between different games loaded in the same process.
8. Keep configuration local and declarative; never execute scripts or fetch network data through configuration.

## Candidate configuration groups

### Terminology — `terminology.fig`

```toml
[display]
spellbook = "Ability Database"
spells = "Programs"
mana = "Energy"
health = "Hull Integrity"
currency = "Credits"
level = "Rank"
experience = "Research Points"
```

These strings customize output flavor, **not** internal game state or command keywords. Consider pluralization and singular labels in the actual schema rather than naive global string substitution. Audit all relevant displays/help text before declaring a term supported. Different currencies in existing multi-currency wallets may require labels per currency, not a universal `currency` field.

### Progression — `progression.fig`

```toml
[progression]
enabled = true
starting_level = 1

[level_scaling]
base_experience = 100
growth_multiplier = 1.5
maximum_level = 50
```

**Proposed examples only:** First inspect real RPG experience equations, defaults, boundaries, save behavior, and integer overflow constraints. The approved keys must map to meaningful implemented settings, not new speculative mechanics. Protect existing character saves if changing progression rules after a campaign starts.

### Parser vocabulary — `parser.fig`

```toml
[command_aliases]
spellbook = ["database", "programs", "abilities"]
```

Separate **display text** from **accepted command aliases**. Define a stable set of reserved commands; detect collisions with other actions/objects. Do not let custom aliases hijack `save`, `load`, `quit`, or other engine-critical actions. Decide whether this is necessary in the first config release; it is not a prerequisite for terminology support.

### Combat and gameplay — optional

Only expose demonstrably valuable global switches or coefficients already supported by core systems (e.g. a documented existing progression rule). Per-enemy health, particular spell effects, item descriptions, and individual puzzles remain in their respective `.toml` files.

## Wizard behavior

Offer "Use recommended defaults" (minimal setup) or "Customize game-wide settings". For each exposed setting: describe it briefly, show its default, offer an example, validate the answer, show **exact serialized FIG output**, and request confirmation. New authors may skip all configuration and return later by editing the FIG manually.

## Acceptance testing

- No FIG files: all existing sample games and their effective behaviors remain unchanged.
- One hand-edited terminology setting visibly updates a supported UI label and preserves the associated command behavior.
- Invalid key/value, malformed TOML, bad numeric scaling, conflicting aliases, and unrelated game settings fail with clear diagnostics.
- Optional settings do not modify `.rork` save serialization or trigger any autosave.
- A wizard-created FIG works identically when edited manually.

## Architecture questions for Codex to answer before coding

Where should the immutable resolved config live? What labels and equations currently exist? What are the exact old defaults? Which first **smallest** useful FIG subset can we ship? How is saving affected if balancing changes mid-campaign? Should the configuration loader be in `story_loader.rs` or a separate module? Codex should recommend answers, not invent a finalized schema before audit.
