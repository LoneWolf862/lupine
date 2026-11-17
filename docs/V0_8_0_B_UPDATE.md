# Lupine v0.8.0-B — RPG Equipment and Spellbook

This is an **implementation candidate**; verify with `cargo fmt --all`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` on Windows before committing.

## Design

- Optional `weapons/*.toml`: an item becomes a weapon with `item`, positive `damage`, and `damage_type`.
- Optional `spells/*.toml`: `id`, `name`, `description`, `mana_cost`, `kind` (`damage`, `heal`, `restore_mana`), `power`, optional `learn_from_item`.
- `weapons` only displays **carried** weapons and AP-adjusted damage. Weapon attack commands and enemies are **not yet implemented**; these arrive in v0.8.0-C.
- `learn <scroll>` requires the teaching item in inventory, consumes it permanently, and adds the ability to the player's spellbook. The secondary inventory is stored as ability IDs, never item inventory entries.
- `spellbook` lists learned abilities; `cast <ability> [on self]` supports healing and mana restoration and checks MP. Damage spells are defined and learnable but **cannot target enemies until combat exists**; no MP is spent on failed actions.
- `GameState` and RORK v1 now include `learned_abilities`, defaulting to an empty list for old saves. `Consumed` item location persists; legacy `room`/`inventory` values still work.
- Save format remains v1, old RORK files without abilities decode. Saves containing an unknown ability are rejected by the interactive runner. A creator changing the set of item IDs must increment `save_compatibility_version` and may need a migration strategy.
- No automatic saving or resource regeneration. Non-RPG adventures still run unchanged.

## Example

```toml
# weapons/iron_sword.toml
item = "iron_sword"
damage = 10
damage_type = "slashing"
```

```toml
# spells/healing_touch.toml
id = "healing_touch"
name = "Healing Touch"
description = "Recover some HP."
mana_cost = 8
kind = "heal"
power = 20
learn_from_item = "healing_scroll"
```

Commands: `take healing scroll`, `learn healing scroll`, `spellbook`, `cast healing touch`, `stats`, `save Spell Trial`.

## Deferred to later stages

Enemy definitions, attack commands, encounter tables, actual attack resolution, enemy health bars (no numeric HP by default), drops, enemy turns, combat restoration and targeting all remain v0.8.0-C/D work.
