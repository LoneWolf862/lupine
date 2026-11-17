# Lupine v0.8.3 — room and unique encounters

Each room can opt into one of three encounter persistence modes via `[encounters]`:

```toml
[encounters]
chance = 100
min_enemies = 1
max_enemies = 1
persistence = "unique" # "temporary" (default), "room", "unique"

[[encounters.enemies]]
enemy = "stone_guardian"
weight = 100
```

- **temporary** (default): leaving discards the enemies and their damage. Returning rolls normally. Existing adventures keep this behavior without edits.
- **room**: living enemies remain in their room with the same instance IDs and HP after retreat. When every enemy is defeated, entering again rolls a *new* encounter. Use for guards or areas that replenish after being cleared.
- **unique**: living enemies remain wounded after retreat, but once the *last* enemy in the group is defeated, the room never spawns that encounter again in that save. Use for bosses and story-critical guards.

The current-room enemies, offscreen retained encounters, and completed unique rooms are serialized into RORK. **No autosave is added.** Saving continues to require an explicit `save` command. Old RORK files and existing room TOML without `persistence` remain supported.

## Demo

Launch `lupine play samples/rpg_persistent` (or `cargo run -- play samples/rpg_persistent`). Take the dagger or learn Fireball. Move `south` into Southern Woods (`room` mode), damage the goblin, retreat `north`, then return `south`. The same goblin ID and damage should remain. From Southern Woods go `east` to the Guardian's Lair (`unique` mode). Damage the Stone Guardian and retreat west; return east to check that its HP persists. Defeat the guardian, leave and return: there should be **no new guardian**. Try explicit saves before and after the battle, then reload each to test persistence.

## Limitations

- `room` respawns a fresh encounter after all enemies are defeated and the player later re-enters. `unique` never respawns once the group is defeated.
- This is room-bound persistence, not a full multi-room AI simulation: offscreen enemies do not chase the player or take turns.
- Enemy HP bars remain approximate; this update does not address the cosmetic zero-segment-alive issue.
- Save compatibility revision stays at 1; story authors who change or remove encounter tables or enemy definitions should consider increasing the game's `save_compatibility_version` for incompatible changes.
