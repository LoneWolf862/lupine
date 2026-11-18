# FIG configuration foundation

Status: Phase 1 implementation contract. Verification results are recorded in CODEX_HANDOFF.md.

Lupine is unreleased/pre-1.0. Justified breaking changes are permitted; there is no legacy configuration loader. Bundled consumers must use this contract.

## File responsibilities

- `.toml`: adventure content and identity/metadata, including rooms/exits, items, spells/abilities, enemies/encounters and interactions/puzzles.
- `.fig`: game-wide engine settings and presets, with standard TOML syntax.
- `.rork`: mutable gameplay progress, written only by explicit player saving. No autosave, save-on-quit or automatic checkpoints.

## Supported configuration

Only `config/character.fig` and `config/terminology.fig` are supported. No directory or files are needed for a narrative-only game using default terminology. Unsupported `.fig` filenames are errors, not silently ignored options. Configure your editor to treat `*.fig` as TOML for syntax highlighting.

`config/character.fig` has the same top-level fields, defaults and progression calculations as the former development-only `character.toml`. Move that file; do not keep both. The obsolete root file is rejected with migration guidance, never loaded. Presence of character.fig enables RPG; an empty file enables default RPG stats. Absence disables RPG, rather than implicitly creating a character.

```toml
starting_level = 1
max_level = 0 # no authored cap
starting_hp = 100
starting_mp = 30
starting_ap = 100 # 1.00x weapon damage
hp_per_level = 10
mp_per_level = 5
ap_per_level = 10
base_requirement = 100
growth_percent = 120
```

Stats at level L are base + gain * (L - 1). XP cost starts at base_requirement and repeatedly multiplies by growth_percent / 100 with existing integer rounding and bounds. Level-ups consume thresholds and refill HP/MP. This phase does not change arithmetic or add gameplay settings. See CHARACTER_PROGRESS.md.

`config/terminology.fig` supports exactly two optional display settings:

```toml
[display]
spellbook = "Ability Database"
spells = "Programs"
```

Defaults are `spellbook = "spellbook"` and `spells = "abilities"`, matching the existing empty-book message and `Known abilities:` heading. `spellbook` names the collection in its empty-state message and help; `spells` names its entries in the populated heading and help. Labels must be nonblank, at most 80 characters and contain no control characters. They do not rewrite authored prose, stable IDs, spell names, mana labels or other messages.

Both `spellbook` and `spells` commands remain accepted. Display labels are not command aliases. Additional terms and parser vocabulary are future work; unsupported keys are rejected today.

## Resolution and validation

`config.rs` loads per-game immutable settings. `story_loader::LoadedGame` owns the resolved `GameConfig`, containing optional character progression and terminology. The runner reads configuration separately from mutable GameState. No globals, networking, scripts or config serialization in RORK.

Missing optional files use the defaults above; unreadable or malformed files fail. Unknown keys/types and invalid values report the file and relevant key or TOML source location. Validation runs for both play and validate, including narrative games with terminology only. Configuration files are confined to the game's directory.

Changing character balancing during a campaign can affect subsequent level-ups; creators must deliberately manage their game's save compatibility revision for incompatible changes. Relocating identical configuration or changing terminology requires no save-format/revision change. Phase 1 leaves RORK serialization unchanged.

## Creator integration and future scope

`new` and the starter template remain minimal narrative adventures without empty config files. Their instructions explain how to opt in to character settings or terminology. Existing add commands validate FIG through the story loader. Full preview/guided configuration editing, parser aliases and broader balancing settings are later phases.

Required verification: defaults, character fields, terminology output/commands, malformed/unknown settings, missing files, independent games, unchanged progression behavior, RPG regressions, save independence and bundled sample validation.
