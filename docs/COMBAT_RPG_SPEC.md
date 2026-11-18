# Lupine RPG / Combat Architecture — Design Specification v0.8

> Historical design/stage record: feature-status and compatibility claims below describe earlier development. Current policy and file paths are in [NEXT_PHASE_REQUIREMENTS.md](NEXT_PHASE_REQUIREMENTS.md) and [CONFIG_FIG_SPEC.md](CONFIG_FIG_SPEC.md). Character configuration now belongs in `config/character.fig`.

**Status:** Design locked for staged implementation; only v0.8.0-A character foundations are included in the accompanying patch. Weapons, spells, enemies and encounters are NOT implemented yet. The system is optional: adventures without `character.toml` remain pure interactive fiction. No game-specific Rust code is necessary for the planned content formats.

## Non-negotiable requirements

1. Everything configured by game authors in TOML, ASCII art in `.txt`; no GUI, external model or network dependency.
2. Combat is turn-based; deterministic core mechanics and injectable/random-seed-controlled encounter selection.
3. **Never autosave**. Only explicit `save` writes a RORK file. Saving within an encounter preserves all live combat state.
4. Physical weapons remain inventory items. An attack requires possession of its weapon; no weapon duplication in inventory.
5. Spellbook is a separate collection of learned *abilities*, not physical items; spells require ownership and enough MP.
6. No passive HP/MP recovery. Level-up fills both; other restoration only through authored items, abilities or scripted effects.
7. New game content must not invalidate legacy narrative adventures; migration of old RORK files must be deliberate and documented.
8. **Enemy exact HP is private by default.** Display name and a fixed-width ASCII health bar (no numeric fraction, numeric remaining %, damage-derived exact HP, or automatic stats dump). Later a designated inspection ability such as `arcane_eye` may reveal numeric HP, attacks and abilities. For all other default descriptions, guard against accidental leaks.

## Enemy health presentation

Default combat output (10 cells, filled proportional to HP):

```text
Southern Woods
Two enemies emerge from the brush!

Wolf    [----      ]
Goblin  [--------  ]

> attack goblin with sword
You strike the Goblin!
Goblin  [------    ]
```

Bar display is an **approximation**, not a covert exact readout. Use integer math to choose 0..10 filled cells; optional status adjectives (healthy, wounded, critical) could replace the bar in accessibility settings later. The above examples are illustrative bars rather than literal starting HP percentages. Enemies with the same display name receive numbered instance labels (`Wolf #1`, `Wolf #2`) so player targeting is unambiguous. Do **not** reveal hit points or damage numbers inflicted on enemies in ordinary combat text unless an authorized reveal mechanic is used. Player HP/MP/AP may be displayed numerically with `stats`.

**Future inspection**: a learned `arcane_eye` ability grants an inspect result with exact current/max HP, defined attacks/abilities and resistances. Inspection authorization must be checked in the engine, not by trusting a UI setting. Designers can specify whether inspection consumes MP or a turn.

## Proposed on-disk layout

```text
game.toml
character.toml                 # optional; absent = narrative-only
rooms/southern_woods.toml
items/iron_sword.toml          # physical pickup
weapons/iron_sword.toml        # combat data for physical item
abilities/fireball.toml        # spellbook definition
items/fireball_scroll.toml     # teaches ability on use
enemies/goblin.toml
enemies/wolf.toml
art/...
interactions.toml
```

### Character and leveling (v0.8.0-A)

Optional `character.toml` uses top-level fields; see `docs/CHARACTER_PROGRESS.md` for the implemented version. `HP` and `MP` are integer point pools; AP is an integer percentage (100 = 1.00x). Level 1 defaults HP 100, MP 30, AP 1.00; increments +10 HP, +5 MP, +0.10 AP. XP needed for each next level starts at 100 and increases 20% per level. `max_level = 0` means no authored cap; bounded numerical arithmetic still applies. Gaining a level fully restores HP/MP. Healing and MP restoration are otherwise explicit only. Round weapon damage to nearest integer (half up). Enemy defense subtracts after scaling, minimum 1 on valid hits. Default values are balancing starting points, not universal constraints.

### Weapon definition (planned v0.8.0-B)

```toml
item = "iron_sword"
base_damage = 10
damage_type = "slashing"
```

The `item` must exist in `items/`. `attack goblin with sword`, `use sword on goblin` and `slash goblin with sword` normalize to a single attack action. Invalid or unowned weapons cannot deal damage and must not advance enemy turns. AP 100 does base damage, AP 200 doubles it. Armor is a separate later feature.

### Ability definition (planned v0.8.0-B)

```toml
id = "fireball"
name = "Fireball"
mp_cost = 10
base_damage = 15
damage_type = "fire"
requires_target = true
```

The spellbook is a persistent set of learned ability IDs. `cast fireball on goblin` verifies the spell is learned and MP is sufficient; only then deduct MP and execute the action. Scroll items teach abilities through an explicit world effect. Skill labels and spellbook display names should eventually be configurable per adventure genre.

### Enemy definition (planned v0.8.0-C)

```toml
id = "goblin"
name = "Goblin"
max_hp = 25
experience = 15
[[attacks]]
name = "Slash"
damage = 5
weight = 70
[[attacks]]
name = "Heavy Strike"
damage = 9
weight = 30
```

Enemy internal stats, attacks and abilities exist and are saved when active; they are not shown to players by default. Weighted attacks should roll only among usable attacks.

### Per-room encounters (planned v0.8.0-C)

```toml
[encounters]
chance = 0.25
min_enemies = 1
max_enemies = 3
allow_duplicates = true

[[encounters.enemies]]
enemy = "goblin"
weight = 40

[[encounters.enemies]]
enemy = "wolf"
weight = 40
requires = [{ flag_unset = "wolf_pack_defeated" }]

[[encounters.enemies]]
enemy = "boar"
weight = 20
```

On **entering** the room, check the encounter chance once; when it succeeds, roll group size and weighted entries independently. This is an encounter table, **not** a defeated-enemy loot table. A different loot table can drop gold and physical items. Authored one-shot bosses and story-triggered events should be supported separately. A room already containing an active encounter must not spawn another merely because `look` is called. Loading a save must never reroll an existing encounter. Later allow repeat/cooldown/once policies and seeded RNG state to avoid save-scumming.

### Turn behavior (planned v0.8.0-C)

Player commands: `attack <enemy> with <weapon>`; `use <weapon> on <enemy>`; `cast <ability> on <enemy>`; `spellbook`; `stats`; `flee` (chance configurable); `use <potion>` when in inventory. Valid committed combat actions advance one turn and surviving enemies counterattack. Invalid parser actions, ambiguous target selection and unowned abilities/items do not advance turns. An enemy's defeat awards XP exactly once and resolves its configured drops. Enemy HP reaching 0 never reveals its maximum HP automatically.

Player death handling, fleeing behavior, spell costs and loot collection need targeted tests and explicit author defaults before public beta. We should avoid irreversible character death by default; prefer a configurable restart/recovery policy.

### RORK schema evolution (planned v0.8.0-D)

Current RORK saves are `save_format_version = 1`; v0.8-A adds an **optional** character table while retaining v1. Saves without a character section remain readable in narrative-only games. To switch a game from narrative-only to RPG, **increment `save_compatibility_version` and migrate old saves explicitly** rather than silently inventing stats. Future RORK fields: learned abilities, active encounter instance IDs and individual HP, encounter/cooldown state, one-time rewards, player stats/XP, deterministic RNG state. Always decode and validate completely before replacing live state. Neither room art nor enemy display bars are saved; the rendered view derives from saved state.

## Roadmap

- **v0.8.0-A**: optional `character.toml`, stats/XP/leveling formulas, player `stats` command, RPG snapshot in RORK, validation/tests.
- **v0.8.0-B**: weapon/ability definitions, inventory checks, spellbook, mana actions, persistent learned abilities.
- **v0.8.0-C**: enemy definitions, weighted encounter tables, turn combat, **non-numeric enemy HP bars**, unique enemy-instance targeting and defeat/rewards.
- **v0.8.0-D**: save/load encounters, loot and abilities; reload invariants; compatibility migration; integration tests.
- **v0.9.0**: Windows/Linux standalone packaging and platform validation.
