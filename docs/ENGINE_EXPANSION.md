# Approved engine expansion

Approved by the creator on 2026-11-19. Primary workflow: directly author TOML
content and FIG configuration. Engine capability precedes wizard coverage, which
precedes wizard usability. This is engine work, not HOWLBOUND story development.
The baseline is main 451f409, including merged Phase 4 PRs #26–#30.

## Ordered delivery and status

| Phase | Scope | Status |
|---|---|---|
| 1 | Current format reference, mechanic examples, file-based scenario testing | Candidate; verification recorded below |
| 2 | Shared nested conditions, typed effects, counters, ordered outcomes | Approved, not implemented |
| 3 | Authored actions/aliases, fixed-object use, player feedback | Approved, not implemented |
| 4 | Hidden/starting inventory, containers, quantities, consumables | Approved, not implemented |
| 5 | Events, timers, quests, journals, endings | Approved, not implemented |
| 6 | NPCs, conditional dialogue, choices, transfers and movement | Approved, not implemented |
| 7 | Independent economy, shared rewards, world abilities, equipment/status/combat configuration | Approved, not implemented |
| 8 | Wizard coverage, project properties, repair, duplicate/delete/reference migration | Approved, not implemented |
| 9 | Consistent controls, draft visibility, search and room-centered navigation | Approved, not implemented |

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
worktree. CI is pending; native CachyOS and human beginner acceptance are not claimed.
