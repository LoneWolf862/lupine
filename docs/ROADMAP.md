# Lupine development roadmap

> Historical design/stage record: feature-status and compatibility claims below describe earlier development. Current policy and file paths are in [NEXT_PHASE_REQUIREMENTS.md](NEXT_PHASE_REQUIREMENTS.md) and [CONFIG_FIG_SPEC.md](CONFIG_FIG_SPEC.md). Character configuration now belongs in `config/character.fig`.

Milestones are design targets unless marked verified; HOWLBOUND remains a tentative original game without a decided story.

| Milestone | Status | Outcome |
|---|---|---|
| v0.1.0 | Complete, Windows | REPL, terminal parser, navigation. |
| v0.2.0 | Complete, Windows | Items, inventory, interactions. |
| v0.3.0 | Complete, Windows | Generic world model, conditions/effects, stateful descriptions. |
| v0.4.0 | Complete, Windows | Separate independent Rust game, terminal runner; commit `56db281`. |
| v0.5.0-A | Complete, Windows | Runtime-owned world/state; commit `97b9476`. |
| v0.5.0-B | Complete, Windows | External TOML story loading; commit `655a5df`. |
| v0.5.0-C | Complete, Windows | Validator, starter template, creator guide; commit `77d5bb4`. |
| v0.6.0-A | Complete, Windows | Versioned `.rork` serializer and validated state restoration. |
| v0.6.0-B | Complete, Windows | Built-in manual `save`, `load`, `saves`; multiple slots, cross-session gameplay verified. |
| v0.6.0-C | Complete, Windows | Staged write protection, compatibility revisions; 49 tests and gameplay verified; user reported commit. |
| **v0.7.0-A** | **Candidate / local test pending** | Text-only `new` adventure command, repeatable non-overwriting scaffolding. |
| v0.7.0-B | Planned | Terminal `add room`, `add item` generators and help improvements. |
| v0.7.0-C | Planned | Authoring usability tests, example refinement, cross-platform checks. |
| v0.8.0+ | Planned | Packaging and creator exports that do not require Rust for players. |
| v1.0.0 | Long-term goal | Stable engine and nonprogrammer-accessible, text-only authoring. |

**Non-negotiable:** save/load is engine-owned, completely manual. No autosave, checkpoint save, or save-on-quit. RORK `.rork` files remain TOML under the hood.

Windows verification does not establish Linux/CachyOS portability or completed standalone packaging. Keep the course's README template, video, and accurate work log separate.

## v0.7.0-B candidate — room/item tools and dynamic ASCII art

- `add room` and `add item` create exclusive, editable TOML files in an existing adventure.
- Rooms optionally reference `art/*.txt` using `art_file` and select dynamic artwork via `[[art_variants]]` and existing engine conditions.
- Inline artwork via `art` or variant `text` is also supported. No GUI or autosaving.
- `story_example` demonstrates art changing on `door_open`; tests verify look/entry, flag changes, paths, and saved-state rendering.
- This is **not verified on Windows/Linux yet**; run tests and review interactive behavior before committing.


## v0.7.0-C implementation candidate (awaiting local tests)

Interactive text-only `new` wizard, static/dynamic `add art` command, creator quickstart guide, and additional tests. Previous `add room`, `add item`, `play`, `validate`, and manual RORK saves retained. All new creator commands refuse existing files. See `docs/V0_7_0_C_UPDATE.md`.

## v0.8 RPG systems (new)

- v0.8.0-A: optional player progression, stats command and RORK character snapshot (candidate; requires local Windows test).
- v0.8.0-B: weapons, abilities and spellbook.
- v0.8.0-C: enemies, weighted encounters, player/enemy combat, **hidden exact enemy HP and ASCII health bars**.
- v0.8.0-D: persistence and game migration integration.
- v0.9.0: postpone standalone distribution until core combat is tested.

See `docs/COMBAT_RPG_SPEC.md` for decisions and boundaries.

## v0.8.0-B candidate
Optional weapon and spell TOML definitions; learned abilities tracked separately in RORK; healing and mana spells, item-based learning. Enemies and damage combat deferred to Stage C.

## Current creator delivery sequence (2026-11-18)

| Phase | Scope | Status |
|---|---|---|
| 2A�2C.2 | Setup, persistent navigation, rooms, presentation and self-documenting FIGs | Merged through PR #7 |
| 2D-A | Items and shared content transactions | Implemented candidate, PR #8 |
| 2D-B | Spell/ability fields and teaching links | Implemented candidate, PR #9; depends on #8 |
| 2D-C | Multiline artwork and room variants | Implemented candidate, PR #10; depends on #9 |
| 2E-A | Interaction rules, locked doors, conditional prose and encounter settings | Implemented candidate, PR #11; depends on #10 |
| 2E-B | Supported FIG configuration editor | Implemented candidate, PR #12; depends on #11 |
| 2F | Integration, safeguards, scripted acceptance, cross-platform CI and documentation | Implemented candidate, PR #13; depends on #12 |

See [completion review](CREATOR_COMPLETION_REVIEW.md) for exact verification and remaining limits. These PRs remain open for human review. The scripted no-manual-TOML demonstration is implemented; human beginner usability and standalone packaging remain distinct requirements.

<!-- lupine-phase-2c2-spec -->
## Current creator follow-up — Phase 2C.2 (implemented candidate)

Phase 2C.2 implements a self-documenting default directory tree and commented FIG defaults with explicit opt-in for RPG (`config/character.fig`), without generating unsupported FIG categories. See [`SELF_DOCUMENTING_CONFIG_SPEC.md`](SELF_DOCUMENTING_CONFIG_SPEC.md). This is a source/config behavior change, not just a tree-display tweak; use a separate Codex branch and PR. Historical roadmap entries above record earlier development decisions.

Phase 2C.2 now generates commented supported defaults and explicitly disables RPG in new adventures; legacy missing-key behavior remains enabled. Validation results are recorded in CODEX_HANDOFF.md.
