# Lupine v0.8.0-C3: XP and unique-item loot

## What's new

- Enemies award their `experience` on defeat, using the existing `character.toml` progression formula.
- Level-ups update HP, MP and AP, fully restore HP/MP, and respect max_level.
- Enemy `[[loot]]` supports independent percentage drops (`chance = 0..100`) referencing standard portable items in `items/`.
- Loot-only items begin hidden (`Consumed`) in a new playthrough, appear in the room on drop, and can be picked up normally.
- Defeated enemy instances are removed before reward distribution; one enemy death triggers at most one reward payout.
- Items use existing unique world IDs: **no stacks or duplicate copies**. An item already in inventory or a room will not drop again. A previously consumed item may drop in a later encounter.
- All XP/levels and item locations persist via existing explicit RORK saves; **never autosave**. Active encounters remain transient until C4.

## Enemy TOML

```toml
experience = 15

[[loot]]
item = "goblin_trophy"
chance = 100
```

The referenced item must exist in `items/`, be portable, and have a valid starting room (the engine automatically hides configured loot items in fresh games).

## Compatibility

- Existing `samples/rpg_spellbook/` is unchanged: prior saves still load.
- `samples/rpg_rewards/` is a separate save namespace with guaranteed single Goblin on every Southern Woods entry for manual testing.
- `validate` rejects unknown or nonportable loot references.
- A game author adding new item IDs to an existing published world may need to increment `save_compatibility_version`; existing RORK files require the original item set.

## Backlog

- Room descriptions that mention already-collected items must be dynamic.
- Add `spells` as an alias of `spellbook`.
- Consider typo suggestions for attack commands.
- Add stackable/multiple-copy loot in later RPG inventory expansion.
