# Lupine v0.8.4 — RPG creator guide

This document is a **practical entry point** for authors. The sample games contain working, validated TOML syntax; use them as the source of truth for individual fields instead of guessing field names. You do **not** need to write Rust to create a story.

## 1. Create and validate an adventure

```cmd
cargo run -- new my_adventure my_adventure My First Adventure
cargo run -- validate my_adventure
cargo run -- play my_adventure
```

The interactive `cargo run -- new` wizard is an alternative to the positional form. `game.toml` defines the stable game ID, title and starting room. Place room definitions in `rooms/` and object definitions in `items/`. Run `validate` after each significant change. A story without `config/character.fig` can remain a pure puzzle/exploration adventure.

## 2. Character and spells

To add an RPG player, use `config/character.fig` following `samples/rpg_spellbook/config/character.fig`. Define weapon items in `items/` and their mechanics in `weapons/`; define spell effects in `spells/`. The player can check `stats`, `weapons`, `spellbook` or `spells`, learn from scrolls, and target enemies in combat. Learned spells, player stats and consumed scrolls are stored **only on explicit `save`**.

## 3. Enemies and encounter behavior

Define enemy stats, attacks, XP and loot under `enemies/`. An encounter table belongs in a room TOML file. For example:

```toml
[encounters]
chance = 100
min_enemies = 1
max_enemies = 1
persistence = "unique"

[[encounters.enemies]]
enemy = "stone_guardian"
weight = 100
```

The example requires an enemy with ID `stone_guardian` in `enemies/`. Choose:

| Persistence | Retreat | After the group is defeated |
|---|---|---|
| `temporary` (default) | Encounter disappears | A new roll may happen on the next entry |
| `room` | Surviving enemies retain IDs and wounds | A later visit may roll a fresh group |
| `unique` | Surviving enemies retain IDs and wounds | That encounter stays cleared |

Use `samples/rpg_persistent/` as a reference. Enemy HP bars intentionally conceal exact numbers. Unambiguous enemy-name fragments can be used as targets (`attack guardian with dagger`); ambiguous names require more detail.

## 4. Currency and shop offers

See `samples/rpg_rewards/game.toml` for currency definitions and `samples/rpg_rewards/shops/` for valid shop syntax. Offers can buy/sell unique items, restore HP or MP, teach spells or set story flags. Failed services (including already-full HP or insufficient funds) must **not** charge currency or consume stock. As of v0.8.4, inventory objects are unique by item ID: buying an item does not create arbitrary duplicate stacks.

## 5. Attach an offer to an object

Author a stationary item in the appropriate room, define an offer in a shop in that room, and add `transaction_interactions.toml`:

```toml
[[bindings]]
item = "mana_altar"
shop = "healer"
offer = "mana"
success_text = "The altar hums as it replenishes your mana."
```

Then a player can `use altar` without entering the shop menu. See `samples/rpg_transactions/` for the complete example. This binding system currently uses `use <object>` and the referenced shop's room; it is not a general-purpose scripting engine.

## 6. Dynamic prose and ASCII art

Room descriptions can use the exact token `{{objects}}` to list the objects *currently present*. Without the token, ordinary static prose is unchanged even after objects move. ASCII art is stored under `art/` and selected via room art fields and optional conditional variants; see `docs/ROOM_ART.md`.

## 7. Manual RORK saves and compatibility

Players explicitly use `save <name>`, `saves` and `load <name>`. Lupine never autosaves or automatically restores an earlier snapshot on death. At 0 HP, normal gameplay and healing/shopping are blocked until the player explicitly loads a prior save or quits. The save records location, inventory, character, wallet, spells, shop purchases, current and retained enemies, and unique encounters cleared.

Keep the stable `game.toml` ID unchanged across compatible content updates. If a story change makes existing saves incompatible, authors should change `save_compatibility_version` deliberately, noting this will reject older saves. Do **not** change it merely because the Lupine engine version changed: RORK versioning is separate from Cargo package versioning. Back up saves before testing incompatible story edits.

## 8. Verification checklist

1. `cargo run -- validate <your-story>` passes with no unknown IDs or invalid TOML.
2. New-game exploration and puzzle interactions work.
3. An attempted unaffordable offer leaves the wallet unchanged; a valid one charges once.
4. A wounded room-persistent enemy stays wounded after retreat; a defeated unique enemy stays absent.
5. `save <name>`, restart, `load <name>` restores inventory, character, currency and encounter state.
6. A death at HP 0 blocks gameplay but permits `saves`, manual `load`, and `quit`.
7. No command implicitly creates or modifies a save file.

The engine's consolidated regression suite runs via `cargo test`; native Linux tests and prebuilt distributions are separate v0.9 work.
