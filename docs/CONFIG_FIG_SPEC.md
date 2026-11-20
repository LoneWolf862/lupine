# FIG configuration foundation

This is the current development configuration contract. The generated templates,
runtime validator and creator use the same three supported FIGs. Historical editor
milestone results are in [CREATOR_COMPLETION_REVIEW.md](CREATOR_COMPLETION_REVIEW.md);
current review status is in [ENGINE_EXPANSION.md](ENGINE_EXPANSION.md).

Lupine is pre-1.0. Unsupported configuration fields are rejected; proposed settings
must not be documented as implemented behavior.

## File responsibilities

- `.toml`: adventure content and identity/metadata, including rooms/exits, items, spells/abilities, enemies/encounters and interactions/puzzles.
- `.fig`: game-wide engine settings and presets, with standard TOML syntax.
- `.rork`: mutable gameplay progress, written only by explicit player saving. No autosave, save-on-quit or automatic checkpoints.

## Supported configuration

`config/character.fig`, `config/terminology.fig`, and `config/combat.fig` are supported. No directory or files are needed for a narrative-only game using default terminology. Unsupported `.fig` filenames are errors, not silently ignored options. Configure your editor to treat `*.fig` as TOML for syntax highlighting.

`config/character.fig` has the same top-level fields, defaults and progression calculations as the former development-only `character.toml`. Move that file; do not keep both. The obsolete root file is rejected with migration guidance, never loaded. `enabled = true` enables RPG; `enabled = false` disables it. For existing authored FIGs, omitting `enabled` (including an empty file) retains the original enabled behavior. Absence still disables RPG. All character values are validated even when disabled. Bundled RPG samples explicitly set true; new generated adventures explicitly set false.

```toml
enabled = false # explicit narrative-only default for new adventures
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

`config.rs` loads per-game immutable settings. `story_loader::LoadedGame` owns the resolved `GameConfig`, containing optional character progression, terminology and combat settings. The runner reads configuration separately from mutable GameState. No globals, networking, scripts or config serialization in RORK.

Missing optional files use the defaults above; unreadable or malformed files fail. Unknown keys/types and invalid values report the file and relevant key or TOML source location. Validation runs for both play and validate, including narrative games with terminology only. Configuration files are confined to the game's directory.

Changing character balancing during a campaign can affect subsequent level-ups; creators must deliberately manage their game's save compatibility revision for incompatible changes. Relocating identical configuration or changing terminology requires no save-format/revision change. Configuration-only changes do not revise the RORK serialization format.

## Creator integration

`new` creates nine standard directories, including empty NPCs/, and all three
commented FIG templates. Exact previews and generated files use the same text.
RPG remains disabled until explicitly enabled. Existing games need not add missing
optional directories or FIG files. Root-level character.fig, terminology.fig and
combat.fig are rejected with guidance to place them under config/.

Menu 3 provides Character, Terminology and Combat forms. Their defaults and
explanations come from the bundled templates. P previews, S validates and asks for
confirmation, and 0 discards. Missing files are created only on confirmed publication.
F also supports complete structural/text editing; see [PROJECT_EDITOR.md](PROJECT_EDITOR.md).
No configuration edit writes player saves or automatically changes compatibility revisions.

## Combat settings

New adventures include commented config/combat.fig with minimum_damage=1 and
retaliation="combat". Valid minimum damage is 0..1000000; full immunity still
deals zero. Retaliation supports combat, every_action or none. See [exact behavior,
status ordering and equipment](SYSTEMS.md). This FIG never enables RPG by itself.
Guided editing is available at 3 > 3. Currency remains game.toml content
and no longer requires RPG.

Current expansion: creator F provides complete-schema multi-file structural/text drafts, repair, duplicate/delete and validation. G edits publication properties; I inspects the room map and typed references. Combat FIG has a dedicated form at 3 > 3. See [PROJECT_EDITOR.md](PROJECT_EDITOR.md) for controls, formatting effects and recovery v2. Individual guided managers remain available.

Engine expansion Phase 9: [CREATOR_NAVIGATION.md](CREATOR_NAVIGATION.md) describes searchable/paged selectors, visible draft status, room-centered browsing and verified local runtime discovery. These controls share the same explicit publication boundary. Publication still requires explicit confirmation.
