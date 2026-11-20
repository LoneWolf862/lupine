# Container and inspection parser feedback

## Report and scope

The creator reported that the Windows development executable launched and played
successfully, then supplied an Objects Demonstration transcript showing parser
friction. This is useful player feedback, not completion of every Windows,
CachyOS or beginner-authoring acceptance check. No original HOWLBOUND content
is introduced. The published v0.9.0 release remains unchanged.

## Changes in this candidate

| Reported input | Candidate behavior |
|---|---|
| `look in chest` | Inspect the chest, including locked/closed state; open containers list visible contents. Also accepts inside/into. |
| `read note` | Inspect an accessible note unless an authored read action handles it; no movement suggestion. |
| `search chest` | Inspect the chest unless an authored search action handles it; generic inspection reveals nothing hidden. |
| `use key on chest` | Unlock using the container's authored key if no item interaction handles that pair. Authored failures retain priority. Opening remains explicit. |
| `unlock chest` | Explain `unlock chest with <key>` without guessing or revealing the required key. |
| `serch room` / `serch chest` | Suggest the supported search phrase, without executing it. |
| `search room` | The objects demo now authors this alias, plus `search the room` and `search around`, for its existing discovery action. |

Movement spelling suggestions with arguments are offered only when the suggested
phrase parses as a direction. Object names, aliases, exact IDs and ambiguity
handling retain the existing resolver. Inspection, failed commands, clarification
and cancellation spend no turn. Authored read/search actions can deliberately
change state and spend a turn. A successful key-use unlock participates in the
existing whole-turn rollback, including event failures and staged success output.
RORK and game save-compatibility versions are unchanged; saves remain manual.

## Verification and remaining human check

`tests/parser_container_feedback.rs` exercises the reported phrases through the
real runner, inspection privacy, typo suggestions, authored precedence, wrong or
inaccessible keys, ambiguity/cancellation, and whole-turn failure rollback.
Full automated results and the exact tested commit are recorded on this
candidate's draft PR. Automated checks do not claim native CachyOS or human
terminal verification. No merge or release is authorized by this follow-up.

Retest the Objects Demonstration with the candidate executable:

```text
look in chest
unlock chest
use key on chest
look inside chest
open chest
look into chest
serch chest
search chest
serch room
search room
read note
```

The first inspection reports a lock; bare unlock gives syntax help. Explicit
key use unlocks but leaves the chest closed. Opening then inspecting shows the
gem. Misspellings only suggest a retry. Searching the room reveals the note,
which can then be read. Report any phrase you expected to work, its exact output,
the artifact run/commit and your actual OS/terminal. This candidate remains a
development build rather than a published release.
