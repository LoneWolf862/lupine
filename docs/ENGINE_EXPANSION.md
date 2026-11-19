# Approved engine expansion

Approved by the creator on 2026-11-19. Primary workflow: directly author TOML
content and FIG configuration. Engine capability precedes wizard coverage, which
precedes wizard usability. This is engine work, not HOWLBOUND story development.
The baseline is main 451f409, including merged Phase 4 PRs #26â€“#30.

## Ordered delivery and status

| Phase | Scope | Status |
|---|---|---|
| 1 | Current format reference, mechanic examples, file-based scenario testing | Verified locally and in CI; draft PR #31 |
| 2 | Shared nested conditions, typed effects, counters, ordered outcomes | Verified locally; draft PR #32 |
| 3 | Authored actions/aliases, fixed-object use, player feedback | Verified locally; draft PR #33 |
| 4 | Hidden/starting inventory, containers, quantities, consumables | Verified locally and in CI; draft PR #34 |
| 5 | Events, timers, quests, journals, endings | Verified locally; draft PR #35 |
| 6 | NPCs, conditional dialogue, choices, transfers and movement | Verified locally; draft PR #36 |
| 7 | Independent economy, shared rewards, world abilities, equipment/status/combat configuration | Verified locally; draft PR #37 |
| 8 | Wizard coverage, project properties, repair, duplicate/delete/reference migration | Verified locally; draft PR #38 |
| 9 | Consistent controls, draft visibility, search and room-centered navigation | Implemented candidate; verification in progress |

Validation, debug/scenario support, docs, save round trips, references and export
coverage accompany each phase. Each major phase uses an isolated feature branch,
checkpoint commits/pushes and a draft PR targeting its immediate dependency.
Nothing is merged or released by the agent. The owner's primary checkout and
personal untracked adventures remain untouched.

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

## Verification

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
