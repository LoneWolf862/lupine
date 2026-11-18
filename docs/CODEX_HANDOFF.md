# Lupine — Codex Development Handoff

**Status:** Documentation handoff **only**. Do not start implementation until the user supplies the first Codex prompt.

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

1. Optional, default-compatible, TOML-under-the-hood `.fig` configurations for suitable game-wide settings and flavoring.
2. Expand existing **text-only** creator wizard to build rooms, exits, items, interactions and optional FIG settings; display **actual file previews** to passively teach authoring.
3. Improve deterministic parser/object resolution, ambiguity handling, typo suggestions and errors; never require an LLM.
4. Low-risk cleanup with regression tests and updated current-state documentation.

## Hard constraints

- Offline terminal gameplay; no GUI, internet, accounts or LLM service.
- Authoring is text/TOML/FIG; no story-specific engine code.
- `.rork` retains explicit **manual only** saves; **absolutely no autosave**.
- Do not modify `.rork` format or compatibility versions as a side-effect.
- Existing games without `.fig` must behave the same; protect old samples/CLI/Rust APIs.
- No destructive overwrites or file traversal via creator input.
- Keep tests and changes small, reviewable and accurately reported.
- Prefer architectural proposals before changes where the design is undecided.

## Next action, after the user supplies a separate prompt

Perform a **read-only repository audit**: map existing functionality, identify exact integration points, verify current defaults and progression formulas, propose a minimal FIG schema, analyze wizard and parser gaps (including RPG paths), list risks and regression tests, and recommend the smallest first implementation PR. Make **no code changes** in that first audit. The user will review it before implementation.

## Course obligations

CSE 310 professor allows this semester's preexisting Lupine work to count as a self-contained two-week sprint. Document real Rust features (mutable/immutable variables, loops, borrowing/ownership, `Vec`, `match`, at least one stretch), honest hours, GitHub source, course-template README, and demo/code walkthrough. Don't invent evidence.

## Validation for later implementation

`cargo fmt --all -- --check` · `cargo test` · `cargo clippy --all-targets -- -D warnings`, plus sample play/validation and manual platform checks. Never state these passed without running them.
