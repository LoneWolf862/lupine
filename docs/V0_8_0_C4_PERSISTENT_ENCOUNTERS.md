# Lupine v0.8.0-C4 — Active Encounters in RORK

RORK now records the current room's live encounter: each surviving enemy's **unique instance ID**, enemy definition ID, and remaining HP. It also stores the next available instance ID, preventing reuse after load.

* An encounter is an **in-memory** change, **not an autosave**. Only explicit `save` writes a RORK file.
* `load` restores the saved encounter without rerolling. Dead enemies are absent, preventing duplicated kill rewards from reloading a completed fight.
* Older saves that omit encounter fields deserialize to no active encounter and a next ID of 1. An old save with no encounter is *not* rerolled upon load.
* Leaving a room clears its current encounter. Re-entering triggers a new roll; the engine does not maintain encounters for previously left rooms.
* Saves with invalid or duplicate instance IDs, zero enemy HP, bad enemy references, or HP above the enemy's maximum are rejected, leaving the running game unchanged.
* Active enemies remain invisible as exact HP counts in normal output; ASCII health bars are unchanged.

## Manual regression test

1. `cargo run -- play samples/rpg_rewards`
2. `take dagger`, `south`, `enemies` (guaranteed encounter), `use dagger on goblin`, `enemies`, `save Mid Fight`, `quit`
3. Restart, `load Mid Fight`, `enemies`, `use dagger on goblin` and observe the wounded goblin continues from the saved HP.
4. Defeat the goblin, verify XP, loot and currency awarded once, save to a separate slot, restart and load it; the goblin must remain absent until leaving and returning.

## Future issues

* Static room descriptions may continue mentioning objects already collected. Use dynamic/conditional text in a future polish release.
* Add `spells` as an alias for `spellbook`, and consider typo suggestions.
