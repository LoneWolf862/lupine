# Lupine — Codex Development Handoff

## Current handoff

Development through PR #43 is merged. Continue from current main using an isolated
branch/worktree and a draft PR. [STABILIZATION.md](STABILIZATION.md) records the
merged verification. Current approved work is status-documentation correction and
the [acceptance plan](ACCEPTANCE_PLAN.md), including automated verification and
a handoff for human Windows/CachyOS testing.
Do not resume the retired branches or worktrees named below. No merge or release
is authorized. Published v0.9.0 and manual-only RORK remain unchanged.

## Historical handoffs

The following entries preserve original milestone context, not current instructions.

## Phase 2I creator usability candidate (2026-11-18)

Draft [PR #19](https://github.com/LoneWolf862/lupine/pull/19) targets `codex/phase-2h-rpg-integration`. Existing-definition selectors accept numbers, exact IDs and unambiguous names. Blank selections stay open; 0/back explicitly returns. Locked doors and teaching items use one selection screen. Door review retains answers after declined Save; description ordering and multiline recovery are improved. See the [audit, navigation rules, verification, remaining limits and Windows procedure](CREATOR_USABILITY_AUDIT.md). Nested 0/back follows its displayed finish/discard label; cancel abandons unsaved edits while earlier confirmed saves remain. Exact previews, Snapshot safety and manual-only RORK saves remain required.

## RPG authoring handoff — 2026-11-18

Continue from `codex/phase-2h-rpg-integration` in its isolated managed Windows worktree. Five draft PRs #14-#18 contain enemies, weapons, shops, currencies and integration; none merged. The owner primary checkout and personal adventures remain untouched. Read [complete handoff, tests, limits and human guide](RPG_AUTHORING_DEVELOPMENT.md). No Cargo/RORK versions or gameplay mechanics changed. Earlier records below are historical.

## Current pre-release policy (2026-11-18)

Lupine is unreleased and pre-1.0, with no external users, published games or production deployments requiring development-version compatibility. Justified breaking API and format changes are allowed. Do not add obsolete-format loaders or unnecessary compatibility layers. Update bundled samples, templates, tests and current documentation together; preserve gameplay unless a change is intentional.

File responsibilities:

- `.toml`: adventure content, identity/metadata, rooms/exits, items, spells/abilities, enemies/encounters, interactions/puzzles.
- `.fig`: game-wide engine configuration and presets, using standard TOML syntax. Character settings belong exclusively in `config/character.fig`; terminology belongs in `config/terminology.fig`. Starting stats, progression and XP scaling are configuration. Future gameplay/balancing options require implemented behavior.
- `.rork`: mutable player progress, saved strictly manually. No autosaving, save-on-quit or automatic checkpoints.

Phase 1 relocates existing character fields/defaults/calculations without redesigning progression, adds display terminology, and updates creator guidance. No legacy `character.toml` loader. No save-format change is needed. Full wizard and parser upgrades are later phases.

**Status:** Phase 1 merged. Phase 2A.1 beginner/advanced wizard refinement implemented and automatically verified on Windows on the existing Phase 2A branch and PR #2; awaiting human review. Broader authoring and parser expansion remains future work.

## Project identity and baseline

- Repository: `LoneWolf862/lupine`.
- **Existing** reusable offline terminal interactive fiction engine in Rust. **Not** a new Zork clone or a new scaffold.
- v0.8.4 shown in `README.md`; v0.8.4-C was awaiting final Windows verification in reviewed notes. Verify actual head before work.
- HOWLBOUND is a tentative first original game, not built-in engine content; its story hasn't been specified.
- Core and optional RPG features, save files, TOML authoring commands, and the guided `new` wizard already exist. Do not implement them a second time.

## Read before work

1. `README.md` and `AGENTS.md`.
2. `docs/NEXT_PHASE_REQUIREMENTS.md`.
3. `docs/CONFIG_FIG_SPEC.md`.
4. `docs/CREATOR_WIZARD_SPEC.md`.
5. `docs/PARSER_IMPROVEMENTS_SPEC.md`.
6. Existing `docs/REQUIREMENTS.md`, `docs/CREATOR_VISION.md`, `docs/CREATOR_QUICKSTART.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md` and relevant current source/tests.

**Important:** Some older documentation describes historical milestones as not implemented even though the README and newer source show them implemented. Flag inconsistencies; do not regress behavior based on stale notes.

## Workstreams

1. Per-game, TOML-under-the-hood `.fig` configurations for suitable game-wide settings and flavoring.
2. Expand existing **text-only** creator wizard to build rooms, exits, items, interactions and optional FIG settings; display **actual file previews** to passively teach authoring.
3. Improve deterministic parser/object resolution, ambiguity handling, typo suggestions and errors; never require an LLM.
4. Low-risk cleanup with regression tests and updated current-state documentation.

## Hard constraints

- Offline terminal gameplay; no GUI, internet, accounts or LLM service.
- Authoring is text/TOML/FIG; no story-specific engine code.
- `.rork` retains explicit **manual only** saves; **absolutely no autosave**.
- Do not modify `.rork` format or compatibility versions as a side-effect.
- Migrate bundled RPG games to `config/character.fig`; no duplicate legacy loader. Preserve gameplay, not obsolete development formats.
- No destructive overwrites or file traversal via creator input.
- Keep tests and changes small, reviewable and accurately reported.
- Prefer architectural proposals before changes where the design is undecided.

## Phase 1 implementation and verification (2026-11-18)

Implemented configuration foundation; pending user review. `config.rs` loads immutable per-game GameConfig; LoadedGame carries config instead of a separate progression field. Full runner entry points are run_configured and run_configured_with_io. Existing convenience entry points use default terminology. Spellbook rendering takes explicit display terms.

All five bundled character files moved to config/character.fig with unchanged contents. Root character.toml is rejected with move guidance, not parsed. The RPG transactions sample demonstrates terminology.fig. Generated and starter narrative adventures remain minimal with no unnecessary config files; their instructions explain opt-in.

Windows checks executed successfully: cargo fmt --all -- --check; cargo test (114 tests); cargo clippy --all-targets -- -D warnings. CLI validation passed for story_example, templates/starter and all four RPG samples. Scripted RPG play verified custom empty/populated spellbook labels and unchanged commands. The independent Rust sample compiled after refreshing its stale local Lupine package version in its Cargo.lock; its initial --locked check had correctly refused that update.

No RORK serialization/version or progression arithmetic changes. No dependency additions, full wizard overhaul or parser upgrades. Native Linux, human interactive acceptance and published package execution were not performed. Existing extreme-XP arithmetic limitations identified in the audit are unchanged and remain separate work.

## Course obligations

CSE 310 professor allows this semester's preexisting Lupine work to count as a self-contained two-week sprint. Document real Rust features (mutable/immutable variables, loops, borrowing/ownership, `Vec`, `match`, at least one stretch), honest hours, GitHub source, course-template README, and demo/code walkthrough. Don't invent evidence.

## Validation for later implementation

`cargo fmt --all -- --check` · `cargo test` · `cargo clippy --all-targets -- -D warnings`, plus sample play/validation and manual platform checks. Never state these passed without running them.

## Phase 2A implementation and verification (2026-11-18)

Based on origin/main at 3e8a1e6. The `new` wizard now collects title, stable ID, location, room title and description with defaults, optional help, validation retries, cancellation/EOF, multiline input, exact file previews and explicit confirmation. Shared serialized drafts feed both previews and writes. Noninteractive creation remains available. Creator writes stage and sync bytes before exclusive publication; rollback removes tracked files and empty directories without recursive deletion. New projects are validated through the story loader.

Windows verification passed: `cargo fmt --all -- --check`, `cargo test` (123 tests), `cargo clippy --all-targets -- -D warnings`. Coverage includes exact preview bytes, defaults/help/retries, cancellation and EOF, preexisting destinations, failed writes/project rollback, Windows junction rejection, special characters and multiline text, CLI validation and scripted play without autosaving. Existing FIG, gameplay, sample validation and RORK regressions passed.

No dependencies, player parsing, combat/progression, save behavior or format changes. No native Linux or human interactive usability verification. Filesystem hard-link support is required; interrupted project creation is recoverable but not a crash-atomic multi-file transaction. Additional modes, back/skip, guided linking/items/puzzles and FIG editing remain future phases. See CREATOR_QUICKSTART.md for operational limits.

## Phase 2A.1 refinement and verification (2026-11-18)

Continued on `codex/creator-wizard-foundations` and PR #2. The user's Windows test of Phase 2A successfully created/validated a game but identified jargon-heavy prompts and an unusable PATH-dependent play command. This refinement makes Beginner mode the Enter default, adds four explained steps and automatic IDs, and offers an educational file preview only when requested. Advanced mode retains manual IDs and direct previews. Both modes support help, back navigation, retained answers, review revisions, multiline descriptions, escaped literal controls, cancellation and EOF. CLI cancellation exits cleanly without treating it as a runtime failure.

Folder selection explains relative/full paths and shows the resolved destination. Local sibling-ID collisions are suffixed automatically for beginners and rejected for advanced authors; existing folder suggestions get numbered alternatives. Actual existing destinations are always rejected. A custom advanced ID survives a display-name edit. There is no global registry, and simultaneous creation elsewhere is not coordinated. Generated projects remain validated with the existing loader, with exclusive staged writes and unchanged formats.

Completion provides correctly quoted Cargo and absolute executable commands instead of assuming PATH installation. Windows commands explicitly target PowerShell, with instructions for entering it from Command Prompt. Linux text uses POSIX quoting. Gameplay is a separate action; no automatic launch or save occurs. Generated README guidance now includes the source/Cargo workflow.

Windows verification: `cargo fmt --all -- --check`, `cargo test` (136 tests), and `cargo clippy --all-targets -- -D warnings` all passed. New coverage includes both modes, all defaults, slug/fallback/collision behavior, invalid IDs, relative/absolute folders, explanations/help, back/review revisions, cancellation/EOF, optional preview equality, multiline/literal controls, preexisting content and completion commands. A Windows integration test executed the printed absolute executable command with spaces, an apostrophe and shell metacharacters in the project path. A separate scripted `cargo run -- new` smoke check executed the exact printed Cargo play command and verified the opening room and description. Existing noninteractive creator, FIG, RPG, RORK and sample validation regressions passed.

No native Linux verification or renewed human beginner acceptance yet. The user's untracked `gas/` adventure was left untouched and excluded from commits. Hard-link filesystem support and recoverable/non-crash-atomic project creation remain Phase 2A limitations. Guided room linking, objects, puzzles and FIG editing remain future work. See CREATOR_QUICKSTART.md for the Windows review walkthrough. PR #2 remains open; do not merge automatically.

## Phase 2B implementation and verification (2026-11-18)

PR #2 was still OPEN and unmerged when this phase began. Created isolated branch `codex/creator-session-navigation` from its head `069a524`; Phase 2B is a separate stacked PR targeting `codex/creator-wizard-foundations`. Do not merge either automatically. Retarget/recheck the Phase 2B diff after foundation approval and merge.

Implemented `creator` Create/Open launcher, `creator PATH` reopening, and automatic session entry after interactive CLI `new`. Existing setup-only library and noninteractive commands retain their behavior. `creator/session.rs` keeps the project and borrowed terminal streams; options 1/2/8 support play, validation and help, and 0/back/cancel/EOF leave safely. Future editor options are explicitly unavailable. Gameplay runs in-process through the unchanged configured runner and returns to the creator after quit. Each play starts fresh; load/save remain explicit.

`adventure.rs` shares the existing complete content-validation pipeline between CLI play, CLI validate and creator actions. `creator/tree.rs` refreshes a read-only, recognized-file tree before each menu, with relative paths, empty existing directories, link/junction rejection, long-path abbreviation and traversal/display bounds. Invalid content can be opened for diagnostics; unsafe paths block loads. Existing files are never rewritten in this phase. Later editing safety requirements are documented in ARCHITECTURE.md.

Windows checks passed: `cargo fmt --all --check`, `cargo test` (148 tests), `cargo clippy --all-targets -- -D warnings`. Twelve new session tests cover creation/open/reopen, invalid menus, cancellation/EOF, actual/limited tree rendering, external-edit refresh and reload, bad references/FIG, full RPG validation parity, junction escapes, gameplay return, explicit manual saving and byte-for-byte preservation of content/existing saves. All four bundled RPG games validate and play through the session without automatic save writes. CLI new-to-menu and creator-path reopening are exercised as child processes. No native Linux or human interactive Phase 2B acceptance was performed.

Limits: no guided content editing (2C-2E remain proposed); tree display is 160 entries, inspection is 2048 entries/eight nested levels, and paths over 180 characters are abbreviated. Reaching inspection bounds blocks creator play/validation rather than bypassing checks. Filesystem checks are not a sandbox against concurrent hostile path replacement. Initial creation retains Phase 2A hard-link/crash-recovery limits. No dependencies, runtime networking, parser/RPG behavior changes, RORK format changes or autosaving. User folders `gas/` and `ghoastly_keep/` remain untouched and untracked. See CREATOR_QUICKSTART.md for the Windows walkthrough.


## Phase 2C implementation and verification (2026-11-18)

Current status supersedes the historical PR states above: foundation PR #2 and session PR #3 are merged. Branch `codex/phase-2c-room-editor` starts at main `61bdf1a`, with a separate draft PR targeting main. Do not merge automatically.

Option 7 now lists, creates and edits rooms. Drafts support display names, multiline descriptions, generated/custom unique IDs for new rooms, multiple one-way or reverse connections, explicit conflict replacement, removal of outgoing exits, current properties and exact TOML previews. Existing IDs stay fixed. All affected rooms save together after explicit confirmation; unchanged rooms remain byte-identical. TOML-aware editing preserves comments and supported advanced content (conditional descriptions, artwork variants and encounters). `toml_edit` is now a direct dependency; it was already in the lockfile transitively.

Room persistence snapshots recognized content, rejects unsafe paths/links and stale external changes, validates a private staged project, retains originals, publishes exclusively and validates again. Failure rolls back when safe; conflicts retain `.lupine-room-edit/` with recovery instructions and block further edits. This is recoverable, not crash-atomic. See ARCHITECTURE.md and CREATOR_QUICKSTART.md for the safety model and manual recovery. No gameplay, parser, FIG schema or RORK format changes; player saving stays manual.

Windows checks passed: `cargo fmt --all --check`, `cargo test` (168 tests), and `cargo clippy --all-targets -- -D warnings`. Twenty new tests cover guided creation/editing, multiple/reverse/one-way/conflicting exits, preview equality, multiline text, cancellation/EOF, stable IDs, advanced content/comments/inline tables/CRLF preservation, repeated sessions, invalid references, stale files, junction escapes, actual publication failure, rollback/recovery conflicts and gameplay without save writes. A live agent-driven Windows terminal walkthrough created a project and second room, connected north/south, previewed and saved both files, edited both descriptions, validated, travelled both ways and returned to the creator. This is not human beginner acceptance. Native Linux verification remains pending.

Limits: existing invalid adventures must be corrected before room editing; no ID rename, room deletion, item/spell/art/FIG or conditional-rule editing. Hard-link-capable filesystems are required; crash recovery is manual. Snapshot limits are 8 MiB per file and 64 MiB total, within bounded traversal. Concurrent hostile filesystem replacement is outside this safety model. Untracked user projects `gas/`, `ghoastly_keep/` and `test_2/` were not changed. See the Phase 2C Windows walkthrough in CREATOR_QUICKSTART.md for review.


## Phase 2C.1 interface polish (2026-11-18)

PR #4 is merged. Branch `codex/phase-2c1-creator-ui-polish` starts from latest main `5d2f7e7` and targets main with a separate draft PR. Do not merge automatically.

Implemented an actual hierarchical ASCII project tree with four-column indentation, directory-before-child ordering, stable child sorting and empty-directory display. Recognition filters and traversal safety remain unchanged. Labels now show basenames; names over 180 characters are abbreviated. Shared heading/prompt helpers standardize spacing through setup, creator/room menus, previews and validation. Connection previews explicitly show From/Direction/To/reverse routes; summaries distinguish directionality. Save review avoids repeating long descriptions; successful writes have a room/file/validation screen. Existing menu options, defaults and editing semantics stay unchanged.

Windows verification: 171 tests passed, `cargo fmt --all --check` passed, and strict Clippy passed. Tests cover nested/empty/deep trees, long names, ordering, portable ASCII branch formatting, truncation, junction protection, menu spacing, connection/save presentation and all existing workflows, preview equality, rollback and manual-save regressions. Verification used `CARGO_TARGET_DIR=target/ui-polish` because an existing running `target/debug/lupine.exe` was locked; that process was not interrupted. A scripted Windows CLI walkthrough inspected the RPG sample tree, room properties, return navigation and validation output. Native Linux and human visual acceptance are not verified.

Limits: existing display/inspection bounds remain; no adaptive wrapping or terminal-width detection. Unicode authored names remain Unicode while tree decoration is ASCII. Existing recovery/hard-link restrictions are unchanged. No dependencies or version changes, no gameplay or player-save changes. User folders `adven2/`, `gas/`, `ghoastly_keep/` and `test_2/` remain untouched. Windows review procedure is in CREATOR_QUICKSTART.md.

<!-- lupine-phase-2c2-spec -->
## Next approved work: Phase 2C.2 — self-documenting adventure configuration

After the creator UI-polish PR has been merged, take current `main` and implement [`SELF_DOCUMENTING_CONFIG_SPEC.md`](SELF_DOCUMENTING_CONFIG_SPEC.md) on a dedicated branch. Keep default FIG files human-readable and inline commented, add explicit RPG opt-in without breaking migrated RPG samples, retain current supported FIG schema until intentionally changed, and run full validations. Treat prior handoff notes above as historical; do not rewrite past verification claims as current.


## Phase 2C.2 implementation and verification (2026-11-18)

Documentation PR #6 is merged in main `38527a1`; Phase 2C.1 PR #5 was already merged. Work uses `codex/phase-2c2-self-documenting-config`, with a separate draft PR to main. Do not merge automatically.

Every new adventure now creates config/, rooms/, items/, spells/, weapons/, enemies/, shops/ and art/. Only required initial content and the two supported FIGs are written. Character and terminology templates contain explicit defaults and comments for each field (purpose, accepted values, units, dependencies and save compatibility). Generated FIGs share the starter template bytes and use the normal exact preview/exclusive-write path. Existing adventures are not rewritten.

`enabled = false` keeps generated games narrative-only. True activates the existing progression implementation. Missing enabled in an existing character.fig, including an empty file, retains the legacy enabled behavior; missing character.fig stays narrative-only. All progression values are validated even when disabled. Unknown keys/types remain errors; misplaced root character.fig/terminology.fig now report the correct config/ location. Bundled RPG sample FIGs explicitly use true. No arithmetic, dependencies, Cargo version, RORK format or save behavior changes. Diagnostics now mention enablement when RPG content/saves require it. Blank Open Adventure input gives guidance and retries.

Windows verification passed: `cargo fmt --all --check`, `cargo test` (175 tests), `cargo clippy --all-targets -- -D warnings`, and `git diff --check`. Verification used CARGO_TARGET_DIR=target/config-docs to avoid interfering with running default executables. Added coverage verifies generated directories/empty content folders, explicit/commented defaults, narrative play, manual FIG enablement, invalid disabled settings, misplaced FIGs, legacy missing-key behavior, migrated sample leveling/save data, exact FIG previews in both wizard modes, blank path retry, manual save/load and unchanged save bytes after play. Existing sample play, progression, transactions, creator rollback and manual-only saving regressions pass.

A separate scripted Windows CLI smoke created a disposable new adventure, validated/played stats with enabled false and true, and checked the actual nested FIG tree. Native Linux and human usability verification remain pending. Runtime remains offline. FIG editing is manual until Phase 2E; no unsupported categories are generated. Toggle/stat changes can invalidate saves: guidance requires deliberate game.toml save_compatibility_version management, never automatic version changes. Existing creation hard-link/recovery limits remain. Git cannot retain empty directories in copied templates; `new` creates all eight and the template README explains this distinction.

See CREATOR_QUICKSTART.md for Windows manual steps. User untracked files/folders (including root README.txt, SELF_DOCUMENTING_CONFIG_SPEC.md, apply_docs.py and existing adventures) were left untouched and excluded from commits.


## Adventure Creator completion: Phase 2D-A (2026-11-18)

Started from fetched main 14eefce on codex/phase-2d-items. Item menu supports creation, editing, display names, descriptions, aliases, portability, room selection, property previews and explicitly confirmed saves. Stable IDs and existing filenames are retained. Shared recoverable transactions now permit direct item/spell TOML paths and safely create missing content folders; existing room recovery behavior is retained.

Self-review: preserved scalar decorations, conditional descriptions, CRLF and unchanged bytes; rejected path traversal before staging; tested invalid references and stale snapshots. Windows automated checks: 179 tests passed, formatting and strict Clippy passed (target/creator-complete). Scripted item create/edit/play, alias removal, cancellation and EOF are covered; these are simulated walkthroughs, not human usability testing. No Linux verification. No package/save version changes or autosaving.

Next stacked phases: spells, artwork, interactions/conditional room settings, FIG editor, integration and end-to-end acceptance. Overall creator completion is still pending. Existing untracked user files were excluded.


## Phase 2D-B spell editor candidate

Menu 5 creates/edits every current spell field: stable ID (creation only), display name, description, MP cost, kind, positive power and optional teaching item. Damage targets an enemy; heal/restore_mana affect the player. No spell damage-type field exists. Teaching items are selected from actual items, must be portable in the guided workflow and are consumed by learn. RPG enablement is explicit; disabled projects receive guidance without modification.

Branch codex/phase-2d-spells stacks on codex/phase-2d-items (PR #8). Windows: 181 tests pass, fmt and strict Clippy pass; native Linux/human usability remain pending. Self-review found that declining Save discarded the draft; both item/spell forms now retain it so authors can revise or confirm later. Tests exercise this, learning, manual save/load, field ranges and preserved comments/IDs. Overall completion remains pending artwork, puzzles, FIG and integration.


## Phase 2D-C artwork editor candidate (2026-11-18)

Menu 6 lists existing artwork (including nested files), creates/edits multiline text, previews exact whitespace and attaches/removes room defaults and conditional variants. First-match variants use guided flag-set, flag-unset or carried-item requirements. Existing inline artwork is replaced only after explicit confirmation. Existing variant tables and inline-array representations are preserved. No artwork is stored in RORK.

Branch codex/phase-2d-artwork stacks on codex/phase-2d-spells (#9), following items #8. Windows: 184 tests, fmt and strict Clippy pass. Self-review identified unreferenced art escaping loader validation; the transaction now calls the same runtime artwork validator even before a file is referenced. Tests cover spaces, blank lines, backslashes, nested files, cancellation, terminal controls, dynamic references and removal. Linux/human usability remain pending. Recovery remains manual; no versions or autosaving changed.


## Phase 2E-A interactions and advanced rooms candidate (2026-11-18)

Main menu 9 opens existing interaction/exit-rule forms and a locked-door recipe. Rules expose used item, target, all existing conditions/effects and outcome text; existing lists/comments survive selected edits. The door workflow can create key/door items and publishes them with rules in one validated recoverable transaction. Room Options 7 adds/edits/removes conditional descriptions and edits encounter chance, counts, persistence, weighted existing enemies and flag requirements. It does not create new enemy definitions.

Branch codex/phase-2e-interactions stacks on artwork #10 (after items #8 and spells #9). Windows: 189 tests, fmt and strict Clippy pass. Scripted acceptance creates connected rooms, a door and key, alternate prose, travels after unlocking and explicitly saves/loads. Cancellation publishes none of the related files. Existing conditional lists/comments and encounter choices survive edits.

Self-review uncovered a real authoring blocker: mixed-case display names without aliases could not be resolved against lowercased player input. The minimal engine fix normalizes authored names/aliases for comparison, preserving exact-match priority and ambiguity. This is covered with uppercase input and ambiguous keys; no parser redesign or new mechanic. An existing navigation test was updated for the intentional ninth menu option. Native Linux and human usability remain pending. No package/RORK versions changed, no autosaving.


## Phase 2E-B FIG editor candidate (2026-11-18)

Menu 3 edits character enablement, every current progression field and both implemented display labels. Displayed defaults and explanations come from the same commented templates used for new adventures. Existing omitted-enabled semantics remain true; absent files are only created after confirmed Save. Inline display tables, comments and omitted unrelated defaults survive. Unsupported categories are not generated. Full validation prevents disabling RPG while dependent content remains and checks overflow/cross-field constraints even while disabled.

Branch codex/phase-2e-fig-editor stacks on interactions #11 (after #8-#10). Windows: 193 tests, fmt and strict Clippy pass. Tests cover all fields, exact previews, new missing config directories, legacy semantics, inline labels, cancel, invalid cross-field repair and dependent RPG content. Self-review improved failed-save handling: proposed drafts remain available for correction; stale/recovery conflicts instruct users to discard/reopen after review. No save compatibility or Cargo version is changed automatically. Native Linux/human usability and final integrated acceptance remain pending.


## Phase 2F integration checkpoint (2026-11-18)

Branch codex/phase-2f-creator-integration stacks on FIG #12. Adds item conditional descriptions, teaching-item portability protection, editable/reorderable artwork variants, clearer non-room description prompts and current help. Reordering preserves tables/comments and refreshes TOML source positions so the selected priority is actually serialized. An existing RPG test fixture now uses a process-local atomic counter instead of a timestamp, eliminating the observed parallel temporary-directory collision.

The real CLI acceptance fixture creates an entire three-room/four-item adventure through menus, with a one-way route, locked door, key/aliases, dynamic prose/art, spell/scroll, RPG and FIG settings, cancellation, manual before/after saves and reopening. Gameplay confirms loading restores both flag-driven prose and artwork. A separate actual executable walkthrough also passed, with disposable output under target/creator-complete; bundled story, starter and all four RPG sample CLI validations pass. No personal projects were used.

Windows checkpoint: 198 tests, formatting and strict Clippy passed. Final documentation and current-head verification are in progress. Added a PR/dispatch CI workflow for Windows and Linux; no CI result is claimed yet. Human usability remains pending.


## Creator completion handoff (2026-11-18)

See [CREATOR_COMPLETION_REVIEW.md](CREATOR_COMPLETION_REVIEW.md) for the complete implementation inventory, branch/PR dependency table, actual verification, self-review findings, limitations, Windows review procedure and merge guidance. Current code is on codex/phase-2f-creator-integration. PRs #8–#13 remain open/draft; nothing was merged or pushed to main. Current local suite: 199 tests; formatting and strict Clippy pass. Windows Server 2022 and Ubuntu 22.04 CI passed at f236172 with 198 tests each, before one additional local sample-dependency regression; final PR CI is tracked separately.

Linux CI exposed and verified the fix for unreadable sibling directories in advisory game-ID scanning. Current documentation now distinguishes the delivered menus from still-text-authored RPG definition categories, manual recovery and future packaging. All authored/editor writes remain explicitly confirmed; gameplay saving is still strictly manual. Existing user untracked files remain unchanged.

Final CI also exposed a sibling directory disappearing between enumeration and metadata inspection on Windows. The advisory ID scan now tolerates that concurrent-removal case, without relaxing destination validation; both platform checks are rerun after the fix.


Final code verification: commit `4a5a727` passed 199 tests on local Windows and on each GitHub CI Windows Server 2022 / Ubuntu 22.04 job, with formatting and strict Clippy. Successful run: https://github.com/LoneWolf862/lupine/actions/runs/44416029219. The following documentation-only checkpoint records those outcomes. All six PRs remain draft/open for human review; no main writes or merges occurred.
