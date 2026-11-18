# Optional Character Progression — v0.8.0-A

Create a file named `config/character.fig` in your story folder to enable RPG character stats. A story **without** this file retains the original narrative-only behavior.

```toml
starting_level = 1
max_level = 50 # use 0 for no authored cap
starting_hp = 100
starting_mp = 30
starting_ap = 100 # AP units: 100 = 1.00x, 200 = 2.00x
hp_per_level = 10
mp_per_level = 5
ap_per_level = 10
base_requirement = 100
growth_percent = 120 # each next level costs ~20% more XP
```

All keys are optional. Run `lupine validate <story>` to validate the file, `lupine play <story>` to use it, and type `stats` during gameplay to view your player level, current HP/MP and AP. Enemy defeats award configured XP through `Progression::award_xp`. There is no passive regeneration or autosaving. Manual RORK saves preserve the character fields. To turn an existing game into an RPG, increase its `save_compatibility_version` and plan an explicit migration for old saves.

Every level-up fills current HP and MP to the new maximum. Without a level-up, only explicit healing/restoration can refill these pools. Damage multipliers are calculated by integer percentage to make mechanics deterministic.

Missing character.fig means no RPG character. An empty character.fig enables all default values (max_level defaults to 0, although the example above chooses 50). Omitted fields use defaults; unknown fields and invalid values fail validation. The former root character.toml format is not loaded; move it to config/character.fig. This relocation does not change progression calculations or RORK format.
