# Lupine Development Backlog

## Phase 2I creator usability candidate (2026-11-18)

Draft [PR #19](https://github.com/LoneWolf862/lupine/pull/19) targets `codex/phase-2h-rpg-integration`. Existing-definition selectors accept numbers, exact IDs and unambiguous names. Blank selections stay open; 0/back explicitly returns. Locked doors and teaching items use one selection screen. Door review retains answers after declined Save; description ordering and multiline recovery are improved. See the [audit, navigation rules, verification, remaining limits and Windows procedure](CREATOR_USABILITY_AUDIT.md). Nested 0/back follows its displayed finish/discard label; cancel abandons unsaved edits while earlier confirmed saves remain. Exact previews, Snapshot safety and manual-only RORK saves remain required.

## Current RPG authoring status

Enemy, weapon, shop and currency editors are implemented candidates in PRs #14-#18; see [RPG review](RPG_AUTHORING_DEVELOPMENT.md). Remaining work includes human beginner feedback, stable-ID migrations, transaction-object-binding authoring, guided recovery and further packaging/export tooling. Do not interpret the historical editor backlog below as current implementation status.

Deferred until after standalone distribution.

## Interactive Startup Menu

When Lupine launches without arguments, display a terminal menu:

1. Play an Adventure
2. Create an Adventure
3. Help
4. Exit

Keep the existing command-line interface for advanced users.

## Parser and Terminal Improvements

- Support up/down arrow command history.
- Expand aliases and alternate object names where appropriate.
- Improve natural-language command interpretation.
- Handle equivalent phrasing such as using versus casting spells where appropriate.
- Improve typo suggestions for items, spells, and enemies.

## Constraints

- Remain lightweight, offline, and terminal-based.
- Preserve gameplay. Justified pre-1.0 format/API changes are permitted; migrate bundled consumers without obsolete loaders. Avoid unrelated RORK changes.
- Never autosave; players control saving explicitly.
- Implement these improvements in a future Codex development pass.

## Creator follow-ups after Phases 2D–2F

The requested item, spell, art, puzzle, conditional-room and FIG menus are implemented candidates in PRs #8–#13. See CREATOR_COMPLETION_REVIEW.md for verified scope. Remaining work outside this delivered editor scope:

- Human beginner usability testing and accessibility refinements based on observed feedback.
- Stable-ID rename/deletion with cross-file and save-impact migrations.
- Guided recovery for interrupted multi-file edits; current recovery is manual.
- Enemy, weapon, shop, currency and transaction-binding definition editors. Existing definitions remain validated/preserved; room encounter settings select existing enemies.
- Broader TOML representation editing where the safe editor currently reports a limitation.
- Standalone packaging/export and real Windows 10/CachyOS acceptance.

Phase 2C.2 self-documenting FIGs and blank Open-path guidance are already merged. Do not create unsupported configuration categories or add unimplemented runtime knobs to fill a menu.
