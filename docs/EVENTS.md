# Events, timers and story progress

Optional `events.toml` supplies immutable definitions. No Rust, RPG configuration or
wizard is required. Try `lupine test samples/events_demo samples/events_demo/scenarios.toml`.

```toml
[[events]]
id = "arrival"
on = { enter = "hall" }
once = true # default; false permits repeat firing
requires = [] # shared nested conditions
effects = [{ set_quest_stage = { quest = "delivery", stage = "active" } },
           { add_journal = "instructions" }, { start_timer = "deadline" }]
text = "The demonstration begins."

[[events]]
id = "expired"
on = { timer = "deadline" }
effects = [{ end_game = "timeout" }]

[timers.deadline]
turns = 4
repeat = false # default
start = false # default; true starts on a fresh game

[quests.delivery]
title = "Deliver the token"
# initial = "active" # optional; otherwise absent until an effect activates it
[quests.delivery.stages]
active = "Find the token."
done = "Delivery complete."

[journal.instructions]
title = "Instructions"
text = "Find the token before the timer expires."

[endings.timeout]
kind = "failure" # or "success"
text = "Time ran out."
```

Replace the example room ID with a room in your adventure. Every referenced ID and
quest stage must exist. Definitions allow at most 1024 entries per category, 128
stages per quest, 256 effects per event and timers from 1 to 1,000,000 turns.
Unknown fields, duplicate event IDs and invalid nested rules fail validation.

## Triggers and ordering

`on` accepts `{ enter="room_id" }`, `{ take="item_id" }`,
`{ action="authored_action_id" }`, `{ defeat="enemy_definition_id" }`,
`{ timer="timer_id" }`, or the string `"turn"`.

- Enter fires on fresh startup and successful room entry, including a directional
  exit back into the same room. Effect-driven movement fires when the room changes.
- Take means an increase in carried quantity, including gifts, rewards and purchases.
  Moving contents between carried containers does not count as acquisition.
- Action fires after a successful authored action, even if it changes no state.
- Defeat refers to a defeated enemy definition, not a temporary instance number.
- Turn fires after action hooks and timer expiration hooks.

Matching events run in file order. Requirements are evaluated against the state left
by previous events. A once event is marked complete only after it fires successfully;
unmet requirements do not consume it. Effects can enqueue additional enter/acquisition
events. A complete batch is bounded to 128 event firings; cycles fail with a diagnostic.

An event batch is atomic **separately from the successful player action that triggered
it**. On a hook error, the action remains committed, but all hook/timer/turn changes
and their success output roll back. The runner reports the failure. Author scenarios
should exercise both successful and rejected effect paths.

## Turns and timers

Successful movement, item manipulation, interactions, authored actions, transactions
and spent combat actions advance one turn. `wait` deliberately spends one turn.
Reading, invalid commands, failed actions, canceled clarification, journal viewing
and manual save/load do not advance time. An authored `wait` command may override
the default. This is deterministic action time, never wall-clock time.

After action hooks, timers active before the command tick in sorted ID order.
New or restarted timers retain their full delay until a later successful action.
Cancellation during an earlier hook prevents that timer from ticking. Repeating
timers restart before their expiration hook, which can cancel or restart them.
Fresh startup runs entry hooks without advancing time. Loading restores progress
without replaying startup/entry hooks.

## Shared conditions and effects

These work everywhere [shared rules](RULES.md) are accepted:

| Condition | Meaning |
|---|---|
| `{ quest_stage={quest="delivery",stage="active"} }` | Current stage matches |
| `{ journal_has="instructions" }` | Entry has been acquired |
| `{ timer_running="deadline" }` | Timer is active |
| `{ turn={op="ge",value=4} }` | Compare successful turn count; same comparison operators as counters |

| Effect | Meaning |
|---|---|
| `{ start_timer="deadline" }` | Start or reset its full delay |
| `{ cancel_timer="deadline" }` | Stop it; already stopped is harmless |
| `{ set_quest_stage={quest="delivery",stage="done"} }` | Activate/update a stage |
| `{ add_journal="instructions" }` | Append once, preserving acquisition order |
| `{ end_game="timeout" }` | Show authored ending and stop further gameplay |

`journal` and `quests` display current quest stages and acquired journal entries.
Stages have author-defined meaning; there is no implicit reward or completion rule.
Endings permit reading, help, manual saves, loading and quitting. Loading a prior
unended save resumes gameplay. Endings never save, reset or delete anything.

RORK format **4** persists turn count, completed once events, active timers and their
restart generations, quest stages, journal order and ending ID. Full validation runs
before a load replaces live progress. Older formats require their matching build;
no automatic migration or rewriting occurs. Export includes events.toml; Snapshot
and ID migration cover these definitions and shared references.
