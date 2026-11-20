# Lupine Mini Adventure

Development 0.11.0-dev.11 candidate. This is a neutral, seven-room adventure
combining the previously separate human playtests. It is not HOWLBOUND and does
not establish that game's setting or story. The smaller samples remain developer
regression fixtures; players do not need to repeat every sample.

## Start and goal

Extract the complete development package. Launch `lupine.exe` on Windows, or
`chmod u+x lupine` then `./lupine` on Linux. Choose **Play an Adventure**, then
**Lupine Mini Adventure — Start Here**. From a source checkout:
`cargo run -- play adventures/mini_adventure`.

Your goal is to defeat the stone golem. Start with `talk guide`, choose
`introduce`, and enter your character's name. The guide teaches Light. At a name
or choice prompt, `0`, `back` or `cancel` leaves progress unchanged. Each valid
dialogue choice takes one action turn. `stats` displays your name.

Use `journal` for the route and current quest, `abilities` for spell syntax and
costs, `shop` for trades, `look` for exits and `help` for commands. There is no
deadline; the short timer only delivers a saving reminder. Reading and failed
commands do not spend turns. Nothing saves automatically.

## Optional walkthrough

1. Camp: `take dagger`, `take fireball scroll`, `learn fireball scroll`,
   `take healing scroll`, `learn healing scroll`.
2. Go `south`. `cast fireball on goblin` defeats this first opponent for 10 MP.
   Alternatively, repeat `attack goblin with dagger` until defeated.
   `take trophy`, `search stump`, then `take chest key`.
3. Go `north`, then `east` to the market. `shop`, `sell trophy`, `buy sword`.
   You start with 2 coins, gain 2 from the goblin, receive 8 for its trophy and
   spend 10 on the sword. Buying the sword is optional; the dagger and spells work.
4. Go `west`, then `north`. `cast light on lamp` spends 1 coin and reveals the
   chest. Repeating it does not charge again. `unlock chest with chest key`,
   `open chest`, `look in chest`, `take brass key from chest`,
   `take padded coat from chest`, `equip padded coat`, `take 2 ration from chest`.
   `eat ration` consumes one and restores up to 15 HP.
5. Go `east`. `unlock gate with brass key` (or `use brass key on gate`), then
   `east` into the workshop. `use mana altar` restores HP and MP for free;
   `interact with mana altar` is equivalent. `status` shows temporary Focus.
6. Optionally `save before-boss`. Go `east`, then repeat
   `attack stone golem with steel sword` until defeated. Fireball also works.
   Retreat `west` and use the altar if necessary; a living golem retains its
   damage when you return. Neither enemy respawns after defeat.
7. After victory, `read journal`, `stats`, `abilities`, `read coat` and inventory
   remain available. Gameplay is stopped. `save victory` is optional. `quit`
   returns to startup; confirm if you have unsaved progress.

## What to test on Windows and CachyOS

One complete playthrough on each OS is enough for the combined player check.
Record the package run/commit, `lupine --version`, OS and terminal. These builds
are development artifacts, not a replacement release for v0.9.0.

- Start from an extracted package without Cargo. Check that the adventure is
  easy to find and the room/journal instructions tell you what to do.
- Cancel the guide's name prompt once, then introduce yourself. Try your preferred
  name, including spaces or accented letters. Check that the question stays
  above the editable input and the name displays correctly in `stats`.
- Complete the route. Try a typo, `learn all`, `sell sword` and `buy trophy`;
  these must not accidentally consume a scroll or execute the opposite trade.
- Explicitly save before the boss, quit, restart the executable and load that
  slot. Check your name, coins, inventory, open gate and chest progress. Finish
  the boss, then load the earlier checkpoint to verify intentional restoration.
- In the actual terminal, check Up/Down history, Left/Right editing, Backspace
  and wrapping for long commands. Report confusing prose or unexpected prompts
  along with the commands immediately before them.
- Creator/platform follow-up: open a copy of the adventure, inspect it in Project
  Files, export with the matching local runtime, and run the exported launcher
  from another directory. Preserve exact previews and explicit confirmations.

RORK 7 stores the new character name. Earlier RORK 6 saves need the older matching
build; no saves are converted or overwritten automatically. Use fresh named slots
for this playtest. Export intentionally excludes player saves.

## Automated coverage and remaining evidence

The [scenario suite](../adventures/mini_adventure/scenarios.toml) exercises a complete
winning route, an optional-sword route, canceled introduction, wrong-direction
trades, locked paths and the one-time Light cost. `tests/mini_adventure.rs` covers
name validation, cancellation/EOF, whole-turn rollback, explicit save/restart,
old-format rejection, shop ambiguity, parser non-mutation, post-ending inspection
and fresh-session quit behavior. Package assembly validates every bundled game
and runs every bundled scenario suite using the copied native executable.

The standard Rust, independent-consumer, documentation, export and packaging
checks remain required; see [acceptance procedure](ACCEPTANCE_PLAN.md). Exact
results belong in the PR and CI run. Automated Windows/Ubuntu results do not count
as native CachyOS acceptance or human usability approval. This new adventure's
human playtest remains **NOT RUN** until the creator reports results.
