# Adventure Creator completion review

Date: 2026-11-18. Status: implemented candidates in six open draft PRs, awaiting human review. Nothing was merged or pushed directly to main. Work started from fetched main `14eefce` (Phase 2C.2 merged). The latest integration branch contains the whole stack.

## Delivered behavior

| Component | Implemented |
|---|---|
| Items | Create/select/edit, stable IDs, display names, descriptions, aliases, portability, starting room, conditional descriptions, exact preview and confirmed save. Teaching items cannot become fixed through the editor; full validation protects loot/shop/binding relationships. |
| Spells | Every current ability field: ID, name, description, MP cost, kind, power, teaching-item link. Explains targets/learning, selects portable teaching items, never silently enables RPG. |
| Artwork | Direct multiline editing, whitespace/backslash preservation, previews, existing nested files, room attachment/removal, conditional file/inline variants, requirement edits and priority changes. |
| Puzzles | Existing interaction and exit-rule fields, condition/effect list editing, success/failure text, and a locked-door recipe that can create related key/door items in the same transaction. |
| Advanced rooms | Conditional prose editing/reordering and encounter chance, group bounds, persistence, weighted existing enemies and flag requirements. |
| FIG | All implemented character/progression fields and both terminology labels, with template-derived defaults/help, explicit enablement and runtime validation. |
| Integration | Persistent create/open/edit/validate/play/reopen navigation, actual project tree, canceled/declined drafts, shared safe writes and end-to-end CLI acceptance. |

The creator can build the demonstration adventure without manually editing TOML or FIG. Wizard output is the ordinary runtime format. HOWLBOUND content was not designed or modified. No dependencies, package versions, RORK serialization/compatibility versions, progression formulas or automatic player-save behavior changed.

## Audit and implementation choices

The initial audit confirmed working setup, shared Wizard input, room drafts using toml_edit, full Adventure validation, a bounded project tree and recoverable room transactions. Items/spells/art/config menus were placeholders. The transaction allowlist and original-backup directories only supported rooms.

The implementation extended that existing transaction boundary instead of adding independent writers. A shared Document helper preserves unedited fields/comments, filenames and CRLF; conditions and collections use small shared helpers. FIG explanations/defaults come from the bundled self-documenting templates. Runtime validators remain the authority. The historical recovery folder name remains compatible with existing recovery instructions.

All bundled TOML adventures were inspected and validated: story_example, templates/starter and rpg_spellbook, rpg_rewards, rpg_transactions, rpg_persistent. Existing shop, weapon, loot, spell, encounter and binding definitions remain supported. Regression coverage edits a copy of rpg_transactions and verifies unsafe shop-room/loot-portability changes are refused while a valid display-name change preserves related definitions byte-for-byte.

## Branches and review order

| Order | Branch | PR | Base |
|---|---|---|---|
| 1 | `codex/phase-2d-items` | [#8](https://github.com/LoneWolf862/lupine/pull/8) | `main` |
| 2 | `codex/phase-2d-spells` | [#9](https://github.com/LoneWolf862/lupine/pull/9) | `codex/phase-2d-items` |
| 3 | `codex/phase-2d-artwork` | [#10](https://github.com/LoneWolf862/lupine/pull/10) | `codex/phase-2d-spells` |
| 4 | `codex/phase-2e-interactions` | [#11](https://github.com/LoneWolf862/lupine/pull/11) | `codex/phase-2d-artwork` |
| 5 | `codex/phase-2e-fig-editor` | [#12](https://github.com/LoneWolf862/lupine/pull/12) | `codex/phase-2e-interactions` |
| 6 | `codex/phase-2f-creator-integration` | [#13](https://github.com/LoneWolf862/lupine/pull/13) | `codex/phase-2e-fig-editor` |

All branches are pushed. Each PR shows only its changes relative to its immediate predecessor. Review the full experience on the integration branch, then review/merge in the table's order. Prefer merge commits to retain shared ancestry. After each predecessor merges, retarget the next PR to main if GitHub has not done so and confirm its diff contains only the intended next phase. If using squash/rebase merges, rebase descendants deliberately before continuing; do not blindly merge duplicated predecessor history. Re-run required checks after conflict resolution. No PR is authorized to self-merge.

## Verification

- Local Windows: **199 tests passed**; `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` passed using `CARGO_TARGET_DIR=target/creator-complete`.
- GitHub CI at verified code commit `4a5a727`: **199 tests passed on each of Windows Server 2022 and Ubuntu 22.04**, plus formatting and strict Clippy. [Successful final code run](https://github.com/LoneWolf862/lupine/actions/runs/44416029219). Subsequent handoff edits only record these results; they do not change runtime or tests.
- Actual executable CLI validation passed for the six bundled TOML adventures above and the generated demonstration.
- `tests/creator_acceptance.rs` drives the real CLI using `tests/fixtures/creator_acceptance.txt`, with isolated authored files and user-data/save paths. It creates three rooms (reverse and one-way routes), four items (portable/fixed, aliases), a key-based locked door, conditional prose, default/dynamic art, a spell and teaching scroll, explicit RPG/FIG settings, and a canceled draft. It plays blocked/unlocked routes, learns the spell, saves before/after unlocking, loads both states, verifies art/prose restoration, tries the missing reverse route, and reopens without rewriting saves.
- A separate executable walkthrough was run and its actual gameplay output inspected. Disposable files/transcript remain under ignored `target/creator-complete/walkthrough-5asualal/` in the development checkout; they are not committed or part of release content.
- Existing tests cover parser/navigation, inventory, rules, progression, weapons, combat, shops, persistent encounters and manual RORK reliability. New transaction tests cover confined paths, invalid/stale proposals, mixed-category rollback, missing directories and unreferenced oversized/control-character art.

These are automated/scripted results, not genuine human beginner usability testing. CI Ubuntu does not establish CachyOS acceptance; CI Server 2022 and local Windows do not establish a specific Windows 10 installation. Standalone packaging remains separate.

## Findings corrected during self-review

1. Declining Save initially discarded item/spell drafts; forms now retain them. Invalid form saves also retain drafts for correction while stale/recovery errors explain when reopening is required.
2. Unreferenced artwork was not reached by adventure loading; transactions now explicitly apply the runtime art validator to every edited art file.
3. Capitalized object names without aliases could not match the parser's lowercase input. Authored names/aliases now use matching lowercase normalization, retaining exact-match priority and ambiguity. This was the smallest runtime correction required by the creator walkthrough.
4. Reordering toml_edit tables alone retained old source positions and serialized the old variant order. Priority edits now refresh positions while retaining authored values/comments; gameplay and serialized order are tested.
5. Teaching scrolls could be made fixed, preventing learning. The item editor explains the dependency and refuses that change until the spell link changes.
6. An existing parallel RPG test used timestamps for temporary paths and collided on Windows. It and the remaining creator timestamp-only fixtures now use process-local atomic counters.
7. Linux CI revealed that advisory nearby-ID scanning failed on unreadable system siblings under /tmp. The scan now skips unreadable/linked siblings and siblings removed concurrently, with explicit guidance, while destination confinement remains strict. A later Windows CI run exposed the concurrent-removal case. Linux CI passes after the fix.
8. Prompt/help and documentation review removed stale placeholder claims, used generic descriptions outside rooms, and limited encounter count prompts to the runtime's actual 1–12 range.

## Limits and remaining work

- Human beginner testing and product approval remain required. All PRs remain draft/open.
- Enemy, weapon, shop, currency and transaction-binding definition editors are not introduced by these phases. Their existing files are preserved/validated; encounter settings select existing enemies. The no-file-edit demonstration uses optional stats/spells, not a newly authored combat/shop economy.
- Existing IDs cannot be renamed/deleted with automatic reference/save migration. Packaging/export is separate work.
- A preexisting invalid adventure must be repaired before guided editing. Unusual valid TOML representations that the selected editor cannot safely manipulate are refused with an explanation; no destructive fallback serializer exists.
- New artwork files use direct art/ names; existing nested TXT files can be edited. Editing artwork normalizes newly entered line endings while preserving line whitespace; opening/canceling leaves original bytes intact.
- The locked-door recipe is a one-pass draft: cancel/decline writes nothing; later revisions use normal item/rule editors. Other editors permit revisiting fields before Save.
- Shared publication is recoverable, not crash-atomic. Hard links are required. Recovery is manual using `.lupine-room-edit/RECOVERY.txt`, CHANGES.txt, originals and the optional COMMITTED marker. No automatic repair or protection against hostile concurrent path replacement is claimed.
- Destination/snapshot size and traversal bounds remain. Readable-neighbor ID checks are advisory, not a global identity registry.

## Practical Windows human review

1. Check out `codex/phase-2f-creator-integration`. Run the three required Cargo checks. Run `cargo test --test creator_acceptance` for the repeatable scripted demonstration.
2. Run `cargo run -- new`, create a disposable project in a new folder, and inspect the actual tree. Use Enter/defaults, ? help, back and cancel once each.
3. In Rooms (7), create two locations; connect one with a reverse route and one with a one-way route. Preview and explicitly save. Try declining a conflicting exit replacement.
4. In Puzzles (9), create a locked door on an existing route with a new key/door. In Room Options > 7, add prose conditioned on the same unlocked flag.
5. In Artwork (6), type multiline pictures with leading/trailing spaces and backslashes. Attach a default and a flag-controlled variant; edit/reorder the variant and inspect the result.
6. In Items (4), change names/descriptions, add/remove aliases and create a fixed object. Cancel another draft. Confirm unsaved content never appears on disk.
7. In Configuration (3), explicitly enable RPG and change a stat/label. Create a portable scroll in Items and link a new spell in Spells (5). Confirm making that scroll fixed is refused. Try an invalid level/cap proposal, correct it and save.
8. Validate (2), then Play (1). Try the locked exit, take the key, use it on the door, inspect changed prose/art, take/learn the scroll and inspect the spellbook. Save named before/after slots and load them deliberately. Quit back to the creator and reopen the project later.
9. In a disposable copy, edit a file externally after opening an editor but before confirming Save. Confirm the stale snapshot is refused and your external bytes survive. Do not manufacture recovery conflicts in a real authored project.

Use the current [Creator Quickstart](CREATOR_QUICKSTART.md) for exact menu controls and recovery limits. Original untracked files/adventures in the user's checkout were left untouched and excluded from commits. No course time logs, instructor-template completion or submission evidence were fabricated.
