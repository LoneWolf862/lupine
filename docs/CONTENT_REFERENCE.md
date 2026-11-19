# Current authored formats

This is the current development reference. Earlier milestone reports describe
historical snapshots. [Engine expansion](ENGINE_EXPANSION.md) separates implemented
features from approved future work. Author files directly, run `lupine validate
PATH`, then [test scenarios](SCENARIO_TESTING.md) and `lupine play PATH`.

All TOML/FIG files use standard TOML syntax. References use stable IDs, not filenames
or display names. Unknown fields are rejected. Use lowercase ASCII IDs with digits,
underscores or dashes; avoid changing game identity after players have saves.
The shared-rule additions are documented in [RULES.md](RULES.md); they extend the
baseline fields below with nested conditions, effects and ordered interaction outcomes.

## Files and field reference

| File | Required fields | Optional fields / defaults |
|---|---|---|
| game.toml | format_version=1, id, title, starting_room | save_compatibility_version=1; currency; currencies |
| rooms/*.toml | id, title, description | exits={} mapping north/south/east/west/up/down to rooms; descriptions=[]; art or art_file; art_variants=[]; encounters |
| items/*.toml | id, name, description, portable, starting_room | aliases=[]; descriptions=[] |
| interactions.toml | none | interactions=[]; exit_rules=[] |
| actions.toml | actions array; see [actions grammar](ACTIONS.md) | room, target/item bindings, shared rules/outcomes |
| rules.toml | none | flags={} initial booleans; counters={} initial signed integers |
| weapons/*.toml | item, damage (positive integer) | damage_type="physical" (descriptive only, no resistance behavior) |
| spells/*.toml | id, name, mana_cost (nonnegative), kind, power (positive) | description=""; learn_from_item (portable teaching item) |
| enemies/*.toml | id, name, description, level, max_hp (positive) | attack_power=100; defense=0; experience=0; attacks=[]; loot=[]; currency_rewards=[] |
| shops/*.toml | id, name, room, offers | no implicit shop or offers |
| transaction_interactions.toml | none | bindings=[] |
| art/*.txt | UTF-8 text | maximum 64 KiB; no terminal control sequences |
| config/character.fig | none | See [FIG configuration](CONFIG_FIG_SPEC.md); absent disables RPG |
| config/terminology.fig | none | display.spellbook="spellbook", display.spells="abilities" |

Spells use kind="damage", "heal", or "restore_mana". Damage targets enemies;
restoration targets the player. Defining an ability does not teach it. Teaching
consumes the linked item. Weapons and abilities require RPG enablement.

## Rooms, objects and puzzles

```toml
# rooms/hall.toml
id = "hall"
title = "Hall"
description = "A hall. Nearby: {{objects}}."
[exits]
north = "workshop"
```

Each exit is directional; author the reverse separately. Items begin in their
starting room; fixed items cannot be taken. Shop merchandise and enemy loot are
reserved out of ordinary starting placement. Items are unique, not stacks.
`{{objects}}` in room prose expands current room objects; static prose remains static.

```toml
# interactions.toml
[[interactions]]
item = "key"
target = "door"
requires = [{ flag_unset = "door_open" }]
effects = [{ set_flag = "door_open" }]
success_text = "The door unlocks."
failure_text = "The door is already unlocked."

[[exit_rules]]
from = "hall"
direction = "north"
requires = [{ flag_set = "door_open" }]
blocked_text = "The door is locked."
```

Interactions require item, target, success_text and failure_text; requires/effects
default to empty lists. Both objects must be accessible and the used item carried.
One rule per pair; one exit rule per room/direction. Exit rules require an existing
exit. Conditions: has_item, flag_set, flag_unset. All conditions must pass. Effects:
set_flag, clear_flag, move_item_to_room={item="id",room="id"}.
Undeclared-mode flags initially unset. Optional declarations provide initial values
and strict name checks. Interaction outcomes use ordered when/effects/text entries;
see [shared rules](RULES.md) for the complete extended condition/effect vocabulary.

Room/item `[[descriptions]]` require nonempty `when` condition lists and `text`.
First matching variant replaces default prose. Room `[[art_variants]]` require
nonempty `when` and exactly one of `file` (relative to art/) or `text`. Default art
uses exactly one of art or art_file; art paths must remain confined under art/.

## Encounters and rewards

Room `[encounters]`: required chance=0..100; min_enemies=1, max_enemies=1 (1..12,
minimum <= maximum), persistence="temporary". `[[encounters.enemies]]`: enemy,
positive weight, optional requires=[] of flag_set/flag_unset. Relative weights do
not need to total 100. `room` persistence retains survivors; `unique` additionally
remembers completion. Temporary encounters reroll on entry; look does not reroll.

Enemy attacks: name, positive damage and weight. Loot: item and chance=0..100.
Currency rewards: currency, chance=0..100, min and max (nonnegative, min <= max).
Loot references portable unique items. Existing item possession prevents duplicate
drops. Exact enemy HP is private; the runtime shows coarse bars.

## Currency, shops and object bindings

In game.toml, `[currency]` defines the primary currency and `[[currencies]]` adds
others: name required; id="gold", symbol="", starting_amount=0 defaults.
Currency currently requires RPG enabled. Amounts/prices are nonnegative integers.

Each `[[offers]]` needs id, name, type, currency, price. Types and target fields:
item/sell_item => item; restore_hp/restore_mp => positive amount;
learn_ability => ability; set_flag => flag. Optional stock is positive; omitted
stock is unlimited. requires_flags=[] defaults to no requirements. Item offers
reference unique portable items; one item cannot be stocked by multiple offers.
Failed transactions charge nothing. Successful finite offers consume one stock.

Each `[[bindings]]` requires item, shop, offer; success_text/failure_text optionally
override the normal explanation. Object starting room must match the shop room.
`use <object>` activates the offer only in that room. One binding per item.

## Configuration, saves, validation and examples

FIG files are TOML-syntax configuration, never story content or progress. Only the
two documented filenames are supported. New games explicitly disable RPG; older
character FIGs omitting enabled retain enabled behavior. Unsupported knobs fail.
Manual saves live outside adventure folders. Cosmetic changes need no save revision;
structural changes can invalidate saves. Never edit personal RORK files as authoring.

Validation checks schemas/references, not solvability. `story_example/` demonstrates
locked routes, conditional prose/art and scenario assertions; `samples/rpg_spellbook/`
shows learning/abilities; `samples/rpg_rewards/` shows loot/currency/shops;
`samples/rpg_transactions/` shows bound services; `samples/rpg_persistent/` shows
encounter persistence. These are engine fixtures, not HOWLBOUND story content.
