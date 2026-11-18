# Lupine — Codex Development Handoff

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
