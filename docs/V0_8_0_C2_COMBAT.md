# Lupine v0.8.0-C2 — Combat candidate

- `attack goblin with rusty dagger` / `use rusty dagger on goblin`: carried weapons only, AP-scaled, reduced by enemy defense (minimum 1 damage).
- `cast fireball on goblin`: learned offensive spells only; requires MP. Invalid targets and insufficient mana do not consume resources.
- Multiple same-name enemies require `Goblin #<instance_id>` targeting.
- Surviving enemies retaliate once each after a successful offensive player action. Enemy attacks use author-defined weighted moves.
- Enemy health stays approximate; no exact HP displayed. Player HP is numeric.
- Defeated enemies disappear from the current encounter. C3 will award XP/loot; C4 handles encounter persistence.
- C2 encounters are ephemeral and are deliberately NOT written to RORK. **No autosaves.** Player death currently prevents further attacks until healing or loading; comprehensive death design is deferred.
- Known backlog issue: prose in static room descriptions may mention items removed from the room; use conditional description or dynamic rendering in a later polish update.

## Testing focus

Exercise unknown weapon, uncarried weapon, unknown target, missing target, duplicate targets, zero MP, defeat, and multiple enemy retaliation. Check all invalid combat attempts remain free actions. Verify narrative-only adventures still run.

## Known limitation

Healing/mana support uses the original `cast` route and is not yet a combat turn; C3 should decide whether in-combat recovery consumes a turn. Save/load while fighting does not preserve the encounter until C4.
