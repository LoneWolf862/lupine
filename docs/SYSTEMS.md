# Economy, abilities, equipment and statuses

Currencies and zero-mana world abilities work without RPG character settings.
Character statistics, weapons, damage/healing abilities, armor and statuses still
require `enabled=true` in config/character.fig. Try the narrative-only example:
`lupine test samples/economy_demo samples/economy_demo/scenarios.toml`.

## Shared resources and rewards

All [shared-rule](RULES.md) contexts, including dialogue, events, consumables,
authored actions and world abilities, accept these additions:

| Condition | Meaning |
|---|---|
| `{balance={currency="coins",op="ge",value=2}}` | Wallet balance comparison |
| `{knows_ability="light"}` | Ability is learned |
| `{hp={op="lt",value=50}}` / `{mp={op="ge",value=5}}` | Player HP/MP comparison |
| `{equipped="coat"}` | Item occupies its equipment slot |
| `{status_active="poison"}` | Player has an active status |

| Effect | Meaning |
|---|---|
| `{credit={currency="coins",amount=5}}` | Add currency; overflow fails |
| `{debit={currency="coins",amount=2}}` | Spend currency; insufficient funds fails |
| `{award_xp=10}` | Apply configured progression, including existing level-up restoration |
| `{restore_hp=10}` / `{restore_mp=5}` | Restore, capped at configured maximum |
| `{spend_mp=5}` | Spend MP; insufficient MP fails |
| `{damage_player={amount=10,damage_type="fire"}}` | Damage using armor/resistance rules |
| `{teach_ability="light"}` | Learn a defined ability; already known is harmless |
| `{apply_status="poison"}` / `{clear_status="poison"}` | Apply/reset duration, or remove status |
| `{equip="coat"}` / `{unequip="body"}` | Equip an accessible carried item, or clear its slot |
| `{spawn_item={item="token",quantity=3,location={kind="room",id="hall"}}}` | Create units at a typed location, preserving other stacks |

References validate before play against the assembled definitions. HP/MP/XP effects
require character progression. Transactions stage effects in order, and any failure
rolls back state and success output. These operations never save progress.

## Shops and loot quantities

Shop offers accept `quantity=1` (default), shared `requires=[]` and `effects=[]` in
addition to their existing fields. Item and sell_item offers transfer the specified
units for the stated price. Stock counts successful transactions, not units. Unique
items require quantity 1; stacks support 1..1,000,000. Selling requires accessible
carried units; it does not empty closed bags or discard all copies of an item.

`type="effects"` offers must contain at least one shared effect. They debit their
price and run effects as one transaction. Other offer types run their additional
effects after their ordinary operation, within the same transaction. Failed effects
preserve money, stock and the ordinary operation. Restoration services still reject
full HP/MP without charging and require character configuration.

Stacks are never removed from initial placement just because a shop sells them or
an enemy drops them. Stack purchases create additional units; multiple shops may
stock the same stackable item. Existing unique merchandise/loot with a legacy
starting_room is reserved offstage as before. An explicit `initial` placement is
respected; an already present unique object cannot also be purchased/dropped.

Enemy loot accepts `quantity=1`. Stack drops add units in the current room; unique
drops remain limited to an unavailable object. Enemy `rewards=[]` adds shared
effects. XP, rolled loot, currency and those effects publish as one reward batch.
On reward failure the enemy remains defeated, while the entire reward batch rolls
back with a diagnostic. The total reward batch must fit 256 effects. Enemy defeat
hooks then use the normal event system. No automatic retry or saving occurs.

## World abilities

```toml
# spells/light.toml
id = "light"
name = "Light"
kind = "world"
description = "Illuminate the test lamp."
mana_cost = 0 # default; positive costs require a character
target = "lamp" # optional fixed item ID; omit for an untargeted ability
requires = [{flag_unset="lit"}]
effects = [{debit={currency="coins",amount=1}}, {set_flag="lit"}]
# learn_from_item = "light_scroll" # optional teaching item
```

Use `cast light on lamp`, or `cast <ability>` for an untargeted definition. The ability
must be learned and its target accessible. Target resolution uses item IDs/names;
failed resolution, requirements or effects spend neither mana nor a turn. A successful
world ability spends one action turn and participates in normal events. Defining an
ability does not teach it: use a teaching item, service, or teach_ability effect.
Ordinary item use retains priority. World abilities use cast explicitly.

The existing damage/heal/restore_mana kinds retain their behavior. World-only
target/requires/effects fields are rejected on those kinds rather than ignored.
Damage abilities additionally accept `damage_type="physical"` and optional
`on_hit_status="poison"`. Only damage abilities accept on_hit_status.

## Equipment and damage

```toml
# systems.toml
[equipment.coat] # key is an existing unique portable item ID
slot = "body" # author-defined stable slot ID
defense = 3 # default 0
[equipment.coat.resistances]
fire = 50

[statuses.poison]
name = "Poison"
turns = 3
damage = 2 # default 0 per successful action turn
damage_type = "poison" # default physical
attack_percent = 100 # default; 0..1000, 100 means unchanged
stun = false # default; prevents offensive actions/retaliation
```

`equip <item>` replaces the item in its slot. `unequip <slot>` clears it.
`equipment` and `status` show equipped items and player status durations. Equipping
requires possession and access; dropping, giving away, hiding or closing a bag around
equipment removes its benefit and clears the invalid equipment association. Equipping
and unequipping spend a turn; missing/unchanged selections and cancellation are free.
Weapon attacks still name a carried accessible weapon; there is no forced weapon slot.

Weapons retain their existing damage_type, which now affects resistance. Enemies
accept `[resistances]` maps and attacks accept `damage_type="physical"` plus optional
`status="poison"`. Resistances are signed percentages from -100 (double damage) to
100 (immune). Equipment defense adds across slots; resistances add and clamp to that
range. Damage subtracts defense, multiplies by the remaining percentage, rounds half
up, and applies minimum_damage. Full immunity always deals zero. Missing resistance
means zero. Exact enemy HP remains private in player output.

## Status and combat timing

Statuses replace their existing remaining duration when reapplied; they do not stack
as duplicate instances. Each successful action ticks active statuses once, including
an application made by that action. Reading, failed actions, cancellation and manual
save/load do not tick them. Effects applied by later event/retaliation hooks start
ticking on the next successful action. Author durations with this ordering in mind.

Player attack modifiers apply in sorted status-ID order. Stun makes a valid attempted
offensive action spend its turn without damage or MP cost, allowing the status to
expire. Failed target/weapon resolution remains free. Status damage uses armor and
resistance. Defeated enemies are removed, awarded once and trigger defeat hooks.
Enemy modifiers active at the start of a tick still affect that turn's retaliation,
including their final turn. Persistent room encounters preserve remaining statuses
while away; only active enemies tick. Status progress survives manual saves.

```toml
# config/combat.fig -- generated with commented defaults for new adventures
minimum_damage = 1 # 0..1000000; full immunity remains zero
retaliation = "combat" # combat | every_action | none
```

`combat` preserves retaliation after successful offensive player actions.
`every_action` also retaliates after successful movement, wait, dialogue, equipment
and other actions. `none` disables enemy retaliation. Read-only and failed commands
are free in every mode. This FIG does not enable RPG itself. Defaults preserve
existing adventures that do not opt into new content or settings.

## Saves and author tools

RORK **6** adds equipment/player statuses and per-enemy status durations. Definition,
location, duration and node validation precedes replacing live state. Narrative wallets
and learned world abilities can be saved without a character. No existing saves,
game compatibility revisions, or published releases are rewritten automatically.

Snapshot/export include systems.toml and config/combat.fig. Migration includes status
IDs, equipment item keys, ability targets, typed reward references and nested shop
requirements/effects. Renames remain explicit previews followed by confirmation.
Scenario assertions add wallet, learned_abilities, equipped, statuses, and player
hp/mp. Full guided new-system editing follows in expansion phase 8.
