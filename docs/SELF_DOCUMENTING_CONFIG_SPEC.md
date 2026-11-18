# Lupine — Self-Documenting Adventure Configuration

**Status:** Approved product requirement; implementation planned for Phase 2C.2. This document describes desired behavior, not a claim that it already works.

## Intent

Every new adventure should be understandable by inspecting its generated files. The beginner-friendly terminal wizard and direct hand editing are equal, supported workflows. Configuration defaults should be explicit and explained in the files themselves, not hidden in Rust or revealed only after enabling a feature.

## Default project structure

New adventures should create these **directories** even when empty: `config/`, `rooms/`, `items/`, `spells/`, `weapons/`, `enemies/`, `shops/`, and `art/`. Create `game.toml`, `interactions.toml`, the starting room, the project README, and the **currently implemented** FIG configuration files. Do not create unsupported FIG filenames or pretend proposed knobs are implemented.

Example (illustrative; actual room filenames depend on creator input):

```text
my_adventure/
|-- game.toml
|-- interactions.toml
|-- README.txt
|-- config/
|   |-- character.fig
|   `-- terminology.fig
|-- rooms/
|   `-- start.toml
|-- items/
|-- spells/
|-- weapons/
|-- enemies/
|-- shops/
`-- art/
```

The displayed creator tree must reflect disk contents, not an invented fixed tree. Existing adventures must remain valid without new directories or FIG files. The wizard should provide the same configuration fields as the files, not a shadow format.

## Every FIG file is self-documenting

- Generate each **supported** FIG file with explicit, valid default values and concise comments for every setting.
- Explain the setting's purpose, valid values/units and meaningful dependencies, including whether changing it affects save compatibility.
- Keep comments next to the corresponding keys. Prefer ordinary TOML syntax inside `.fig` files.
- Generated files must pass `lupine validate` without special handling of comments.
- Do not generate a FIG file for a category until its schema and runtime behavior actually exist.
- `config/terminology.fig` should explicitly show the existing `display.spellbook` and `display.spells` defaults, with comments that these change presentation, **not commands**.
- `config/character.fig` must show the actual character/progression defaults, plus an explicit opt-in switch as described below. These are examples of intended content; final serialized names must match the implemented schema.

## Preserve narrative-only adventures when character.fig is always present

**Current Phase 1 behavior:** merely having `config/character.fig` enables RPG mechanics. Generating that file everywhere would inadvertently change ordinary narrative games. Phase 2C.2 must change this contract deliberately.

**Approved target behavior:** add an explicit `enabled = false` boolean in `config/character.fig` for newly generated adventures. The file is present for learning and easy manual editing, while narrative-only behavior remains the default. Setting it to `true` enables the existing RPG character/progression behavior. The generated file must contain valid default stats even while disabled.

Migration and fallback must be unambiguous and tested:

1. An existing adventure **without** character.fig remains narrative-only.
2. Existing shipped RPG sample adventures and any legacy character.fig files that previously enabled RPG must retain RPG behavior; either migrate their files to `enabled = true` or preserve legacy missing-key semantics with clear validation/docs. Do **not** silently disable working games.
3. New generated adventures contain `enabled = false` and must play as narrative-only until the creator opts in.
4. `enabled` only toggles RPG availability; it does not automatically write RORK player saves or enable other unrelated modules.
5. The existing FIG loader, validator and game runtime must agree on enabled/disabled behavior. Unknown keys and types should remain actionable validation errors.
6. Changing RPG enablement for a game with existing saves may be a **breaking content change** and should trigger guidance about `save_compatibility_version`; **do not** conflate that with the engine Cargo version or RORK format version.

Suggested generated character.fig values (subject to verifying actual engine defaults):

```toml
# Enable RPG character mechanics. false keeps this an exploration adventure.
enabled = false

# Starting character attributes (AP: 100 means 1.00x damage).
starting_level = 1
max_level = 0
starting_hp = 100
starting_mp = 30
starting_ap = 100

# Gains per level.
hp_per_level = 10
mp_per_level = 5
ap_per_level = 10

# XP requirement: 120 means 20% growth per level.
base_requirement = 100
growth_percent = 120
```

## Terminology example

```toml
# These settings change displayed labels, not command words.
[display]
# Label for the learned-ability collection.
spellbook = "spellbook"
# Label for entries in that collection.
spells = "abilities"
```

## Future FIG categories

`gameplay.fig`, `progression.fig`, `combat.fig`, and `parser.fig` are *proposed*, not implemented as of this specification. Do not create them merely to match an illustrative tree. Consider splitting `character.fig` only via a dedicated schema migration with clear ownership, updates to all bundled examples, tests, docs and deliberate compatibility choices. Avoid duplicate/conflicting sources of truth.

## Acceptance criteria

- A new default adventure generates the standard directories and both currently supported FIGs, with meaningful inline comments and explicit defaults.
- It remains narrative-only; setting `enabled = true` activates existing RPG behavior after validating/reloading.
- Existing RPG sample adventures still validate, play, level and load their manual RORK saves as before.
- An old narrative adventure lacking `config/` continues to load unchanged.
- Editing a FIG with a standard text editor is immediately reflected when validating/playing; there are no wizard-only settings.
- Generated configuration and folder tree are readable on Windows 10 and Linux; files use portable paths and deterministic output.
- Error paths are descriptive for invalid boolean, unknown options, bad values, or incorrectly placed FIG files.
- No autosaving, automatic checkpoints or unintended RORK format/version changes.
- All Rust checks (`cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`) pass. Verify platforms separately; no invented verification claims.
