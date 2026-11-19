# Player turn transactions

Implemented in development 0.11.0-dev.10. The normal player runner uses one
in-memory commit boundary for a command and its required game consequences.

For example, a dialogue choice transfers a token, its acquisition hook completes
a quest and unlocks an exit, then a timer expires. If any required effect fails,
the token, dialogue node, quest, exit flag and timer all return to their pre-command
values. The player sees the error, without a false gift/quest success message.

## What commits together

- The player command: movement, inventory/container changes, consumption, dialogue,
  equipment, ability costs, purchases/sales, authored actions and combat.
- Defeat rewards (XP, loot, currencies and authored effects), enemy removal and
  persistent encounter completion, including defeats caused by statuses.
- Status ticks, retaliation, triggered event chains, timers and the turn counter.
- Encounter changes, the session random-number position and buffered success output.

Existing execution order is preserved: command and immediate rewards/encounter
synchronization, status ticks and their rewards, configured retaliation, action
hooks, timer hooks, turn hooks, then final encounter synchronization. A failure
anywhere restores the state before the command. The event chain limit of 128 still
applies. Validation cannot prove that authored conditions always make effects succeed.

The player may retry after the failure's cause changes. There is no automatic
retry or pending-event replay. Content errors can therefore still block an action,
but cannot leave only some of its required consequences committed. Read-only
commands, invalid input, canceled selections and failed actions spend no turn.

## Input, output and saves

Object/NPC selection, dialogue choices and confirmation prompts are displayed live
before mutation. Result messages and room/encounter output produced during mutation
are buffered until the complete turn succeeds. A failed turn discards those bytes
and reports one failure. Already displayed prompts are not erased.

Manual save/load commands remain separate operations, run after any pending turn
has settled. Failed turns never write or rewrite RORK files. No autosaving, automatic
checkpoints or save-on-quit is introduced. The in-memory checkpoint is not a save.
RORK remains version 6; this behavior change adds no serialized fields or game
save-compatibility revision. RNG position is restored for a failed turn within the
session; this does not add RNG persistence to RORK.

Terminal I/O failures terminate the runner with an I/O error. Bytes already sent
to a terminal cannot be recalled, and output publication is not a filesystem
transaction. Creator file publication retains its separate hash-checked recovery
contract; it is not changed by gameplay transactions.

## Rust embedding

`runner::run_session_with_io` provides the complete turn boundary. Low-level
`rules::apply`, shop transactions and dialogue selection retain their local batch
boundaries; calling them directly does not run an entire player turn.

`combat::award_defeat` now returns `Result<String, String>` so callers must handle
reward failure rather than treating a diagnostic as successful reward output.
`events::finish` restores `Step::before` on failure; capture the Step before the
triggering action. Embedders composing their own combat/event loop must also stage
output and checkpoint their RNG and any state outside GameState. The built-in
runner handles those responsibilities.

## Verification

`tests/turn_transactions.rs` exercises real file-authored games through the public
runner: dialogue/gifts/quests/exits, acquisition/timer/turn failures, paid purchases,
full containers, physical/spell/status defeats, equipment, retaliation, random
replay, cyclic events, bulk objects, consumables, world abilities and manual saves.
It compares complete GameState values on failure and checks that failed success
text never reaches the writer. A live-input probe verifies dialogue prompts are
visible when input is requested. Verification results are in ENGINE_EXPANSION.md.
