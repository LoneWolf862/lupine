# Approved engine expansion

## Current status

All nine phases and follow-ups through PR #42 are merged into main at
8f9782febf68bb7374fa66cbee95a5f5a9e70725 (0.11.0-dev.10). Its tree matches the
final PR #42 tree verified by [Windows/Ubuntu CI run 44632403701](https://github.com/LoneWolf862/lupine/actions/runs/44632403701).
Subsequent [stabilization](STABILIZATION.md) merged in PR #43 at 7bcaa24.
The published v0.9.0 release is unchanged. Current work and remaining acceptance
are in [ACCEPTANCE_PLAN.md](ACCEPTANCE_PLAN.md). The old expansion review worktrees are retired.

Approved by the creator on 2026-11-19. Primary workflow: directly author TOML
content and FIG configuration. Engine capability precedes wizard coverage, which
precedes wizard usability. This is engine work, not HOWLBOUND story development.
The original baseline was main 451f409, including merged Phase 4 PRs #26–#30.

## Ordered delivery and status

| Phase | Scope | Status |
|---|---|---|
| 1 | Current format reference, mechanic examples, file-based scenario testing | Merged #31; verified locally and in CI |
| 2 | Shared nested conditions, typed effects, counters, ordered outcomes | Merged #32; verified locally and in CI |
| 3 | Authored actions/aliases, fixed-object use, player feedback | Merged #33; verified locally and in CI |
| 4 | Hidden/starting inventory, containers, quantities, consumables | Merged #34; verified locally and in CI |
| 5 | Events, timers, quests, journals, endings | Merged #35; verified locally and in CI |
| 6 | NPCs, conditional dialogue, choices, transfers and movement | Merged #36; verified locally and in CI |
| 7 | Independent economy, shared rewards, world abilities, equipment/status/combat configuration | Merged #37; verified locally and in CI |
| 8 | Wizard coverage, project properties, repair, duplicate/delete/reference migration | Merged #38; verified locally and in CI |
| 9 | Consistent controls, draft visibility, search and room-centered navigation | Merged #39; verified locally and in CI |

Validation, debug/scenario support, docs, save round trips, references and export
coverage accompanied each phase. Delivery used isolated branches and stacked draft
PRs; the owner subsequently merged them. #40 added per-file NPC definitions, #41
whole-turn rollback and #42 current authoring references. No public release followed.

## Required contracts

- TOML: immutable game content and metadata. FIG: implemented game-wide settings.
  RORK: mutable progress, saved only by explicit player requests.
- Reuse one condition/effect model across features; no arbitrary scripts, network
  calls, LLM, game-specific parser branches or wizard-only data model.
- Failed actions and canceled clarification are atomic and free of mutations.
- Validate referenced IDs, values and bounded nested rules before play.
- New state must be included in explicit saves in the phase that introduces it.
  A deliberate schema revision may accompany structural state changes; no revision
  change for Cargo metadata, no silent save rewriting or compatibility promise.
- Preserve published v0.9.0. Migrate bundled development consumers together without
  obsolete-format loaders. Preserve gameplay unless a change is explicitly described.
- Preserve editor Snapshot confirmation, exact previews, stale-file protection,
  evidence-based recovery, confined export and no saved progress in packages.

## Historical verification

Entries below record verification as it happened; later CI results supersede earlier
pending statements. Draft/merge instructions, old branch/worktree paths and primary
checkout descriptions below refer only to those historical sessions. Do not resume
the retired review stack. See Current status above for today's baseline.

Phase 1 local Windows: 273 tests passed, formatting and strict all-target Clippy
passed, release build passed. `scripts/phase4_acceptance.py` authored the full RPG
fixture through menus, exercised eight explicitly saved slots, exported/extracted a
ZIP independently and launched the native package successfully. No automatic saves;
source content unchanged. Artifacts: target/phase4-acceptance-e488ee8d/ in the phase-1
worktree. Native CachyOS and human beginner acceptance are not claimed.

Phase 1 CI: Windows Server 2022 and Ubuntu 22.04 passed at 87992dc, Actions run
44590136960. [Draft PR #31](https://github.com/LoneWolf862/lupine/pull/31) targets main.
Phase 2 intentionally revises RORK to version 2 for new persistent story state;
no existing save files or game compatibility revisions are changed automatically.

Phase 2 local Windows: 280 tests passed, formatting and strict all-target Clippy
passed, release build and independent Rust consumer check passed. Native creator,
manual save/load and independent ZIP launcher acceptance passed (artifacts under
target/phase4-acceptance-b01994de/). Export regression confirms rules.toml is retained
byte-for-byte and reloads. CI and native CachyOS acceptance are not yet claimed.

Phase 2 Windows/Ubuntu CI passed at 2d02d3d, Actions run 44592663157.
[Draft PR #32](https://github.com/LoneWolf862/lupine/pull/32) targets phase 1.

Phase 3 local Windows: 288 tests passed, formatting and strict all-target Clippy
passed, release build and independent Rust consumer passed. Native creator/manual
save/export/ZIP launcher acceptance passed (target/phase4-acceptance-57fbc4d5/).
New tests cover aliases, carried tools, ambiguity/cancellation, atomic failure,
reserved verbs, migration, fixed source interactions, private enemy inspection and
authored movement. Export reloads authored actions. CI pending; human beginner
and native CachyOS acceptance are not claimed.

Phase 3 Windows/Ubuntu CI passed at 7bec031, Actions run 44594678114.
[Draft PR #33](https://github.com/LoneWolf862/lupine/pull/33) targets phase 2.

Phase 4 local Windows: the full 297-test suite passed. An additional direct-combat
closed-container regression passed with the focused object/combat suites (298 distinct
tests now). Formatting and strict all-target Clippy passed; release build and the
independent Rust consumer passed. Native creator/manual save/load, independent ZIP
extraction and launcher acceptance passed after updating its RORK assertions to the
new schema (target/phase4-acceptance-d7a6758c/). Existing object tests were retained.
RORK version 3 deliberately persists the canonical object ledger; no existing save
files or compatibility revisions are rewritten. CI pending; native CachyOS and
human beginner acceptance are not claimed.

Phase 4 Windows/Ubuntu CI passed at 9ed2971, Actions run 44599414261.
[Draft PR #34](https://github.com/LoneWolf862/lupine/pull/34) targets phase 3.

Phase 5 local Windows: all 310 tests, formatting, strict all-target Clippy, release
build and independent Rust consumer passed. Native creator/manual-save/export/ZIP
launcher acceptance passed (target/phase4-acceptance-577b6485/). Event regressions
cover ordering, timer restart/cancel, cycles, atomic failure, acquisition/defeat,
endings, save/load, migration and native exported hooks. RORK 4 deliberately adds
event progress; existing save bytes and compatibility revisions are untouched.
CI, native CachyOS and human beginner acceptance are not yet claimed.

[Draft PR #35](https://github.com/LoneWolf862/lupine/pull/35) targets phase 4.

Phase 5 Windows/Ubuntu CI passed at 4d7c10a, Actions run 44602595164.

Phase 6 local Windows: all 320 tests passed, including ten dialogue regressions;
formatting, strict all-target Clippy, release build and independent Rust consumer
passed. Native creator/manual save/load/independent ZIP launcher acceptance passed
(target/phase4-acceptance-02da217c/). Exported dialogue executes through the packaged
runtime. RORK 5 deliberately adds persistent NPC locations/nodes; no existing save
bytes or compatibility revisions are rewritten. The shared decoder now accepts the
documented 256 effects while retaining bounded condition/choice arrays.
[Draft PR #36](https://github.com/LoneWolf862/lupine/pull/36) targets phase 5.
CI, native CachyOS and human beginner acceptance are not yet claimed.

Phase 6 Windows/Ubuntu CI passed at 81e5b76, Actions run 44605273082.

Phase 7 local Windows: all 338 tests passed, including 18 system integration tests;
formatting, strict all-target Clippy, release build and independent Rust consumer
passed. Native creator/manual save/load/independent ZIP launcher acceptance passed
(target/phase4-acceptance-89345567/). Tests cover independent economy/world abilities,
quantity-aware shops/loot, atomic rewards, resistance/armor/statuses, configurable
retaliation, status defeat hooks, shared-effect movement, migrations and export.
RORK 6 deliberately persists equipment/status progress; no existing saves or game
compatibility revisions are rewritten. [Draft PR #37](https://github.com/LoneWolf862/lupine/pull/37)
targets phase 6. CI, native CachyOS and human beginner acceptance are not yet claimed.

Phase 7 Windows/Ubuntu CI passed at 0d48cc7, Actions run 44609348060.

Phase 8 local Windows: all 347 tests passed, formatting and strict all-target Clippy passed; release build and native creator/manual-save/independent-ZIP/launcher acceptance passed (target/phase4-acceptance-829a1563/). Nine new regressions cover repair/deletion recovery, publication refusal, cancellation/EOF, combat settings, author trace and metadata export. Complete-schema coverage uses structural/text editing, with dedicated property/combat forms; it does not claim bespoke beginner forms for every feature. Recovery journal is v2, RORK remains 6. [Draft PR #38](https://github.com/LoneWolf862/lupine/pull/38) targets phase 7. CI, native CachyOS and human beginner acceptance are not yet claimed.

Phase 8 Windows/Ubuntu CI passed at a9a7ecc, Actions run 44611857492.

Phase 9 local Windows: the full 355-test suite passed; an additional nested-dialogue
copy/delete/reference-repair regression passed (356 distinct tests). Formatting,
strict all-target Clippy and release build passed. Native creator/manual-save/
independent ZIP extraction/launcher acceptance passed
(target/phase4-acceptance-eb7c9eaa/). New regressions cover paging, ambiguous search,
room navigation, portable file selections, draft retention, narrative economy/world
abilities, raw replacement cancellation and verified local runtime discovery.
RORK remains 6. [Draft PR #39](https://github.com/LoneWolf862/lupine/pull/39) targets
phase 8. Final-head Windows/Ubuntu CI status is tracked on PR #39; native CachyOS and human
beginner acceptance are not claimed.

## Historical review handoff (before the owner's merges)

Review draft PRs #31 through #39 in order. Each targets its immediate dependency;
none is merged or published. The final stack is 0.11.0-dev.8. Engine state additions
intentionally revise development RORK to 6; existing saves require their matching
engine/game build and are never rewritten. The published v0.9.0 release is untouched.
The owner's primary checkout remains at 451f409 with its original untracked files.

Engine schemas, validation, save state, scenario tests and export are implemented.
Wizard coverage for advanced schemas is the multi-file structural/text editor F,
with dedicated properties/combat forms and existing guided managers. This does not
claim bespoke nontechnical forms for every new field. Structural edits reformat the
selected file; exact-text editing preserves pasted text. Both are previewed and
confirmed before publication. Human beginner testing remains a future acceptance
step, consistent with the creator vision.

For a review session, use the final isolated worktree: run the bundled scenario
suites, inspect room/reference output, open F to inspect/repair a disposable game,
then try B room navigation and G publication metadata. Validate and export a new
folder/ZIP; keep production adventures and saves on their matching engine build
until the changes are reviewed. No merge or release action has been taken.

## NPC organization follow-up (0.11.0-dev.9)

Separate review branch on top of phase 9. NPCs/*.toml contains one complete NPC definition per file, with an explicit stable id and local nodes. Loader validation, Snapshot recovery/editing, duplication/deletion, migrations, inspection, room references and exports use the new folder. The bundled dialogue example and current authoring docs use this layout. Old dialogue.toml is rejected with split instructions; the project editor can explicitly repair it in a single confirmed batch. No automatic user-content conversion, new NPC behavior, RORK change or save-compatibility bump.

Local Windows verification (2026-11-19): all 362 tests passed; cargo fmt --all -- --check, cargo clippy --all-targets -- -D warnings and cargo build --locked --release passed. Native scripted creator/manual-save/ZIP launch acceptance passed (target/phase4-acceptance-2236d940). A save created using the prior phase-9 release binary and old dialogue layout loaded into the new release and split sample, restoring the NPC node without rewriting save bytes (target/npc-save-compat-b569b8be). New regressions cover independent files, malformed/duplicate IDs, missing/empty/wrong-case directories, scoped node migrations, explicit old-layout repair, duplication/deletion, file moves and recovery/external edits. Unix symlink rejection is covered by a platform-specific test. Historical phase results above are unchanged.

Review: [draft PR #40](https://github.com/LoneWolf862/lupine/pull/40) targets #39; keep unmerged. Windows/Ubuntu CI results for the final head are recorded on the PR. No claim of human beginner testing.

## Whole-turn transaction follow-up (0.11.0-dev.10)

Player actions, rewards and event/timer consequences now share a rollback boundary,
including GameState, session RNG and staged output. See TURN_TRANSACTIONS.md for
scope, ordering and the deliberate reward API change. RORK remains 6. Integration
coverage exercises cross-system failure paths through the real runner. Local Windows
verification passed: all 374 tests, cargo fmt --all -- --check, strict all-target
Clippy, locked release build and native creator/manual-save/ZIP launcher acceptance
(target/phase4-acceptance-6ddfb4ae). The 12 new integration tests include a live
prompt/output probe. [Draft PR #41](https://github.com/LoneWolf862/lupine/pull/41)
targets #40; final-head Windows/Ubuntu CI is recorded on the PR. Keep unmerged.
No native CachyOS or human beginner acceptance is claimed; no wizard changes.
