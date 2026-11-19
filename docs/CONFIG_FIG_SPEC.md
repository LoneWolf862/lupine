# FIG configuration foundation

## RPG authoring integration candidate

The guided RPG menus in draft PRs #14-#18 do not change FIG syntax or progression calculations. Explicit RPG enablement and existing progression editing remain in menu 3. Currency definitions remain content in game.toml, edited through R > 4; no currency FIG is introduced. [Authoring verification](RPG_AUTHORING_DEVELOPMENT.md). Earlier FIG editor PR #12 and integration #13 are merged.

Status: the Phase 2C.2 runtime/configuration contract is merged. The Phase 2E-B guided editor is a candidate in PR #12, integrated in #13. Verification is recorded in CREATOR_COMPLETION_REVIEW.md; historical checkpoints remain in CODEX_HANDOFF.md.

Lupine is unreleased/pre-1.0. Justified breaking changes are permitted; there is no legacy configuration loader. Bundled consumers must use this contract.

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

`config.rs` loads per-game immutable settings. `story_loader::LoadedGame` owns the resolved `GameConfig`, containing optional character progression and terminology. The runner reads configuration separately from mutable GameState. No globals, networking, scripts or config serialization in RORK.

Missing optional files use the defaults above; unreadable or malformed files fail. Unknown keys/types and invalid values report the file and relevant key or TOML source location. Validation runs for both play and validate, including narrative games with terminology only. Configuration files are confined to the game's directory.

Changing character balancing during a campaign can affect subsequent level-ups; creators must deliberately manage their game's save compatibility revision for incompatible changes. Relocating identical configuration or changing terminology requires no save-format/revision change. Phase 1 leaves RORK serialization unchanged.

## Creator integration and future scope

`new` creates all eight standard content directories and commented, explicit defaults in both supported FIGs. The starter template shares those same files. RPG remains disabled until enabled is set true. Existing games need not add any directories or FIG files. Existing add commands validate FIG through the story loader. Full preview/guided configuration editing, parser aliases and broader balancing settings are later phases.

Required verification: defaults, character fields, terminology output/commands, malformed/unknown settings, missing files, independent games, unchanged progression behavior, RPG regressions, save independence and bundled sample validation.

<!-- lupine-phase-2c2-spec -->
## Approved next contract (Phase 2C.2 — not implemented yet)

The authoritative requirements are in [`SELF_DOCUMENTING_CONFIG_SPEC.md`](SELF_DOCUMENTING_CONFIG_SPEC.md). Generated character and terminology files share the starter template text; exact previews use those same bytes. Comments beside every setting explain units, accepted values, dependencies and save compatibility. No new configuration categories exist.

Changing RPG enablement for existing campaigns can invalidate saves. Deliberately increase `game.toml` `save_compatibility_version` for incompatible changes; do not change Cargo or RORK format versions. Terminology-only edits need no revision change. The loader rejects root-level character.fig/terminology.fig/combat.fig with placement guidance; files belong under config/.


## Phase 2E-B FIG editor candidate (2026-11-18)

Menu 3 edits character enablement, every current progression field and both implemented display labels. Displayed defaults and explanations come from the same commented templates used for new adventures. Existing omitted-enabled semantics remain true; absent files are only created after confirmed Save. Inline display tables, comments and omitted unrelated defaults survive. Unsupported categories are not generated. Full validation prevents disabling RPG while dependent content remains and checks overflow/cross-field constraints even while disabled.

Branch codex/phase-2e-fig-editor stacks on interactions #11 (after #8-#10). Windows: 193 tests, fmt and strict Clippy pass. Tests cover all fields, exact previews, new missing config directories, legacy semantics, inline labels, cancel, invalid cross-field repair and dependent RPG content. Self-review improved failed-save handling: proposed drafts remain available for correction; stale/recovery conflicts instruct users to discard/reopen after review. No save compatibility or Cargo version is changed automatically. Native Linux/human usability and final integrated acceptance remain pending.

## Engine expansion phase 7: implemented combat settings

New adventures include commented config/combat.fig with minimum_damage=1 and
retaliation="combat". Valid minimum damage is 0..1000000; full immunity still
deals zero. Retaliation supports combat, every_action or none. See [exact behavior,
status ordering and equipment](SYSTEMS.md). This FIG never enables RPG by itself.
Guided editing follows in expansion phase 8. Currency remains game.toml content
and no longer requires RPG.

Current expansion: creator F provides complete-schema multi-file structural/text drafts, repair, duplicate/delete and validation. G edits publication properties; I inspects the room map and typed references. Combat FIG has a dedicated form at 3 > 3. See [PROJECT_EDITOR.md](PROJECT_EDITOR.md) for controls, formatting effects and recovery v2. Individual guided managers remain available.
