# Lupine v0.8.0-C1 — Enemy Definitions and Encounter Tables

This is the **encounter foundation**, not complete combat. Enemies are declared as `enemies/<id>.toml` with `id`, `name`, `description`, `level`, `max_hp`, optional `attack_power` (percent), `defense`, `experience`, and `[[attacks]]` containing `name`, `damage`, and `weight`.

Rooms may declare a `[encounters]` section with `chance` (0–100 percent, independently rolled on **each successful room entry**), `min_enemies`, `max_enemies` (1–12), and `[[encounters.enemies]]` entries with `enemy`, positive `weight`, and optional `requires = [{ flag_set = "..." }]` / `flag_unset`. Selection weights are relative probabilities **conditional on an encounter**. Selection is with replacement, so duplicates can spawn. If no conditional entries qualify, there is no encounter. A `character.toml` is required for encounter-enabled stories. Noncombat stories remain unchanged.

`lupine validate <path>` now checks enemy definitions, references, weights and group sizes. `lupine play <path>` samples encounters on room entry and renders enemies with an approximate ten-cell ASCII HP bar and numbered instance labels. `look` displays currently visible enemies and `enemies` lists them again. **No exact enemy HP is displayed.** The full values remain internal for future combat and inspection abilities.

**C1 known limitation:** Encounter instances are currently ephemeral visual previews; they do not attack, block travel, award XP or survive saves/reloads. Their persistence and combat semantics will be implemented in C2–D. Do not ship an encounter-enabled game as complete until those milestones are tested. To avoid altering RORK's existing layout, this stage does not write encounter data to saves. Re-entering the same room rerolls its encounter.

**Existing backlog:** Static prose can mention an item even after the item is collected or consumed; add conditional descriptions or future dynamic object-aware prose. See `samples/rpg_spellbook/rooms/camp.toml` for neutral example prose.
