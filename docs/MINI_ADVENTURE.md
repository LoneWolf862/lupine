# Lupine Mini Adventure

Development 0.11.0-dev.12 candidate, based on the creator's dev.11 actual and
example playthroughs. This eight-room training adventure combines the human
player checks. It is not HOWLBOUND and establishes no original-game lore.
Smaller samples remain developer regression fixtures.

## Start and goal

Extract the complete development package. Launch `lupine.exe` on Windows, or
`chmod u+x lupine` then `./lupine` on Linux. Choose **Play an Adventure**, then
**Lupine Mini Adventure — Start Here**. From source:
`cargo run -- play adventures/mini_adventure`.

Defeat the stone golem. `talk to the guide` immediately asks your name.
Cancel/0/back or EOF leaves the entire introduction unchanged. A valid name
confirms the introduction and gives the shop key exactly once. The guide's menu
stays open after each response; L/leave returns to gameplay. Each chosen response
spends one turn; leaving is free and does not undo previous responses.

Use `journal` for your current objective and route, `spellbook` or `abilities`
for known spells and costs, `look` for exits, and `help` for commands. There is
no deadline. Reading, clarification and canceled/failed commands are free.
Nothing saves automatically.

```text
                  Storeroom
                      |
Shop — Training Camp — Gate — Workshop — Approach — Golem
                      |
                    Woods
```

The storeroom's only exit is south. The gate uses a **brass** key.

## Optional walkthrough

1. `talk guide`, enter your name, then L. Take dagger and `take all scrolls`.
   `take scrolls` instead asks which one. `learn fire scroll` and `learn heal`
   study them individually; `learn all` lists the carried teaching commands.
2. Go south. `use dagger on goblin` until defeated, or `cast fireball at goblin`.
   Take trophy, search stump, and take chest key. Return north.
3. `use shop key on door`, then west. The key stays in the lock; the shop
   remains open. Its listing appears automatically. Sell trophy, then buy sword
   if desired. You start with 0 coins, gain 3 from the goblin, and earn 8 from
   the trophy. The sword costs 10, leaving 1 for Light. Rations cost 2 each,
   with stock 10; buying them cannot spend the last odd coin before Light.
   Buying the sword is optional. Return east to camp.
4. Visit the storeroom north of camp. Return south and talk guide; choose
   `chest` for help, then L. The guide teaches Light only after this visit.
   Return north. `cast light on lamp` spends 1 coin once and reveals the chest.
   `unlock chest with chest key` consumes that key into its lock.
   `open chest` immediately lists contents. `take all` collects the brass key,
   padded coat and two rations. Equip padded coat. Eat ration uses one ration.
5. Return south to camp, then east. Answer yes to unlock the gate immediately.
   No/cancel leaves it locked without spending a turn. Manual
   `unlock gate with brass key` or `use brass key on gate` also works.
6. Go east to the workshop. Use mana altar restores HP/MP free and refreshes
   Focus for several actions. Go east to the quiet approach; optionally
   `save before-boss`. Go east again to the vault.
7. Attack stone golem with steel sword until defeated, or use Fireball and
   your dagger. Retreat west twice to rest at the altar if necessary. The
   golem retains its wounds; defeated enemies do not respawn.
8. After victory, inventory, journal, stats, abilities and inspection remain
   available. Blank Enter is silent. Save victory is optional. Quit returns
   to startup, with confirmation for unsaved progress.

## Windows and CachyOS human acceptance

Use one complete playthrough on each OS. Record the artifact run/commit,
`lupine --version`, OS and terminal. This is a development build, not v0.9.0.

- Check readability at your normal terminal width, ideally also at 80 columns.
  Authored prose targets 72–76 columns; narrower terminals can still wrap.
  Check room spacing, journal paragraphs, inventory and automatic shop listings.
- Cancel the first name prompt, then introduce yourself with your preferred
  name, including spaces/accents. Confirm there is one editable prompt and
  the question stays above it. Ask several guide topics and leave with L.
- Try take scrolls and cancel, take all scrolls, learn all, and a bare number
  outside a menu. No command should silently choose or repeat a prior action.
- Follow the route. Check the shop and chest keys disappear only after unlocking,
  sold-out sword stock, the Light help topic, the south-only storeroom exit,
  and a gate prompt that really unlocks on yes. Decline once and unlock manually.
- Save at the approach, quit, restart and load that slot. Check your name,
  inventory, coins, equipped coat, opened locks and progress. Finish the boss,
  then load the earlier checkpoint to verify deliberate restoration.
- Test Up/Down history, Left/Right editing and Backspace in the actual terminal.
- Creator follow-up: open a copy, inspect Project Files, export using the matching
  local runtime, and launch the export from another working directory.

RORK remains **7**. Mini-adventure save compatibility is now **2**, because the
map, objects and dialogue nodes changed. Older content-revision-1 saves require
the matching old package; no conversion or automatic overwrite occurs. Use fresh
slot names for this playtest. Exports exclude saves.

## Automated coverage and evidence

The [scenario suite](../adventures/mini_adventure/scenarios.toml) covers full victory,
optional sword, cancellation, wrong trades, locked paths, one-time Light cost,
filtered pickup and the ration economy. `tests/mini_adventure.rs` also checks
turn rollback, prompt validation, repeated conversation/gifts, NPC movement and
endings, key consumption, journal progress, Focus, shop stock, full manual
save/restart and content-revision rejection without changing existing save bytes.

Required checks are cargo test, formatting, strict Clippy, the independent Rust
consumer, documentation links, creator/export acceptance and portable packaging.
Package assembly validates all bundled games and runs their scenarios against
the copied executable. The packaged acceptance script checks full manual
save/restart and exported-launcher victory. Linux CI additionally exercises the
name prompt, persistent conversation and complete route in a real PTY. Exact
results belong in the PR and CI runs.

The creator reported a real dev.11 playthrough; that is baseline evidence.
Human acceptance of these dev.12 changes and native CachyOS testing remain
**NOT VERIFIED** until reported. Windows automation and Ubuntu CI do not replace
those checks. No release or merge is authorized by this candidate document.
