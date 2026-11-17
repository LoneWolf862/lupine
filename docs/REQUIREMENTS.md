# Lupine Interactive Fiction Engine — Requirements

**Status:** v0.5.0-A verified on Windows; v0.5.0-B TOML loader candidate in testing  
**Language:** Rust (Cargo)  
**Course:** CSE 310 Applied Programming — Rust module  
**First game:** HOWLBOUND (tentative name; story to be designed separately). No setting, plot, characters, or lore are prescribed here.

## 1. Project goal

Build a compact, **offline, terminal-only interactive fiction engine** inspired by the *interaction style* of classic parser adventures such as Zork, but with a more forgiving, discoverable parser. The project is **not** a remake of Zork. Game mechanics and content must be original. A player should be able to run a locally built executable and play without an account, server, internet connection, cloud AI, or installed Rust toolchain.

The primary deliverable is an **open-source reusable framework** that third parties can use to make different adventures without reworking the parser or core world logic. HOWLBOUND will be the first original game used to test the engine, not built-in engine content. A small neutral test world demonstrates behavior in the meantime. **Final-release accessibility goal:** people without a technical/programming background must be able to create, validate, test, and package their own adventures without editing Rust. This is a final-release requirement, **not an MVP or CSE 310 requirement**. See `CREATOR_VISION.md`.

## 2. Non-negotiable constraints

- **TUI / terminal only:** keyboard text input; readable text output; no windowing system or graphical engine.
- **Offline runtime:** after installation/build, gameplay must never require networking. No external API, inference service, or runtime download.
- **Self-contained release:** distribute a platform-specific executable with the story data bundled into it, or with clearly packaged local data files. If story content is in external files, make the release complete and immediately playable.
- **Small footprint:** avoid a large language model, heavyweight runtime, database, web server, or unnecessary dependencies. Prefer standard library and carefully justified small crates.
- **Cross-platform target:** Windows and Linux first; macOS desirable. Build separate binaries per OS if needed.
- **Story-independent engine:** keep story content and world definitions separate from parsing, rules, and rendering. Developers should be able to supply new worlds without rewriting engine internals.
- **Accessible creator path by final release:** a nonprogrammer can author and export a game through approachable tooling; no-Rust creation is not required for the initial framework milestones.
- **Original content:** draw inspiration from the command-driven genre, not from Zork's copyrighted rooms, prose, puzzles, or assets.

## 3. Core player experience (MVP)

### 3.1 Terminal gameplay

- REQ-01: On launch, show a concise title, introductory room description, and a prompt such as `>`.
- REQ-02: Accept repeated commands until the player chooses to quit or input ends.
- REQ-03: Provide `help` describing common commands and illustrative input patterns.
- REQ-04: `look` / `l` displays the current room description, visible exits, and relevant objects.
- REQ-05: Player can move between connected rooms by direction (`north`, `n`, etc.) and a phrase such as `go north`.
- REQ-06: Player can inspect visible objects (`examine lantern`, `look at lantern`, `x lantern`).
- REQ-07: Player can pick up portable objects, list inventory, and drop carried objects (`take`, `inventory`/`i`, `drop`).
- REQ-08: State-changing commands visibly update world/player state; later commands see the change.
- REQ-09: A small test world contains multiple interconnected rooms, portable and fixed objects, and at least one stateful interaction (e.g., opening an object or using one object on another).
- REQ-10: Unsupported or impossible actions return useful feedback and do not crash or silently mutate state.

### 3.2 Easier-than-classic parser

The parser should accept ordinary variations **without pretending to understand arbitrary English**. It is a compact, deterministic, rule-driven parser, not an LLM.

- REQ-11: Ignore leading/trailing whitespace; tolerate extra internal spaces; match verbs and object references case-insensitively.
- REQ-12: Recognize verb synonyms and abbreviations (`get`/`grab`/`take`; `inspect`/`examine`/`x`; `inventory`/`inv`/`i`).
- REQ-13: Understand at least these command forms: `verb`, `verb object`, `verb preposition object`, and `verb object preposition object` for supported actions.
- REQ-14: Ignore common filler words where unambiguous (`the`, `a`, `an`) and recognize simple phrases such as `pick up the brass key`.
- REQ-15: Resolve multiword object names, with explicit aliases in world data (`brass key`, `key`). Prefer visible or carried objects relevant to the action.
- REQ-16: Give actionable errors for unknown verbs, missing targets, inaccessible items, and ambiguous matches (e.g., “Which key: brass key or iron key?”).
- REQ-17: `help` and useful parse errors teach the grammar; a user must not need to memorize exact magic words.
- REQ-18: Keep parsing separate from action validation/execution: a syntactically understood command can still fail based on world rules.

**Optional future parser improvements (not MVP gates):** fuzzy spelling suggestions, pronouns such as `it`, multiple commands joined by `then`, contextual suggestions, command history, and customizable vocabulary. Do not allow fuzzy matching to trigger unintended destructive actions.

### 3.3 World and interaction system

- REQ-19: Rooms have stable IDs, descriptions, and directional exits.
- REQ-20: Objects have stable IDs, display names, aliases, descriptions, and properties (e.g., portable, openable, locked).
- REQ-21: Track object location (room, inventory, container if supported) and mutable state separately from static definitions.
- REQ-22: Implement actions in a reusable rules/command layer rather than hardcoding every player phrase to one story scene.
- REQ-23: Define consistent rules for visibility, portability, movement, and prerequisites; report why an action cannot be completed.
- REQ-24: Story-specific content and puzzle triggers must be added through a world/content layer, not by rewriting core input parsing.

## 4. Rust module grading requirements (from provided course material)

These **five are mandatory** and should be easy to point out during the demo:

| Course requirement | Intended implementation / evidence |
|---|---|
| Mutable **and** immutable variables | Immutable world definitions / parsed command values and mutable `GameState` as the player acts. |
| Loops | Main REPL loop accepting commands; optional loops over inventory/exits. |
| Functions demonstrating ownership transfer **or** reference borrowing | `parse_command(&str, ...)`, `execute_command(&mut GameState, &World, ...)`, or equivalent; explain borrow vs mutable borrow in walkthrough. |
| `vec` list data structure | `Vec` for room objects, inventory, exits, aliases, or candidate matches; use an actual vector in runtime code. |
| `match` keyword | Dispatch normalized command variants and/or action outcomes via `match`. |

**Stretch challenge — select at least one (course options):**
- **Recommended:** use `struct` for `Room`, `Item`, `World`, `GameState`, etc.
- Also natural: use `enum` for typed actions or object locations; use `Option`/`Result` as a function return type for missing targets and errors.

The stretch challenge is distinct from the mandatory five. The above mapping is a **design plan**, not a claim that the implementation already exists.

## 5. Quality and usability requirements

- REQ-25: Invalid input, empty input, an unknown exit, and unexpected end-of-input are handled without a panic.
- REQ-26: Clear, concise output with useful line breaks; avoid excessive repetition.
- REQ-27: Deterministic world updates allow reproducible tests.
- REQ-28: Parser behavior and action rules have unit tests; include at least one automated end-to-end game-flow test.
- REQ-29: `cargo fmt`, `cargo clippy`, and `cargo test` should pass before submission (subject to tool availability).
- REQ-30: Include a public-repo-ready README, build/run steps, command reference, and a short demonstration script.

## 6. Later features (explicitly not needed for first framework pass)

- Manual-only save/load is a **required final-release feature**, specified in `SAVE_FORMAT.md` and planned for v0.6.0; it is not implemented yet.
- A data-driven story format (TOML/JSON/RON or another evaluated format) and bundled example content. **Deferred from MVP but necessary in some suitable form for the final creator release.**
- Conversation systems, NPCs, branching dialogue, combat, quests, or complex puzzles.
- Rich terminal formatting, colors, accessible contrast controls.
- Unicode-friendly matching and typo suggestions.
- User-defined world loading / modding. **User-authored world loading is required eventually for the final creator release, but not for this first pass.**

**Priority:** A robust parser and stateful world matter more than large quantities of story content. Resist scope creep. Creators use TOML templates and terminal commands; no graphical editor is planned.

## 7. Acceptance checks for first playable version

1. Start the game entirely offline in a terminal; receive a room description.
2. Enter `  LOOK  `, `l`, and `look`; get equivalent results.
3. Enter `go north` and `n`; reach the same linked room when appropriate.
4. Enter `take the brass key` and `grab key`; resolve one object consistently when unambiguous.
5. Enter `inventory`; see the key, then `drop key`; see the key in the room.
6. Enter `examine brass key`; receive its description when visible or carried.
7. Try an unknown verb, an absent object, a blocked exit, and an impossible action; receive helpful error messages.
8. Resolve at least one puzzle-like action that changes persistent in-session game state.
9. Unit tests cover commands, synonyms, name resolution, state changes, and invalid inputs.
10. In a code walkthrough, identify all **five Rust course features** and a stretch challenge.

## 8. Long-term reusable and creator-accessible release requirements

The following are **final-release requirements**, not acceptance gates for the current Rust course sprint or v0.3.0:

- **REQ-F01 — Independent games:** a developer can build at least two materially different games with public Lupine capabilities, without editing parser/rules internals.
- **REQ-F02 — No-code story authoring:** a person without Rust or programming knowledge can create rooms, exits, objects, descriptions, and common interactions using an approachable authoring workflow.
- **REQ-F03 — Content validation:** duplicate IDs, broken references, and invalid rules produce creator-facing, actionable diagnostics rather than opaque crashes.
- **REQ-F04 — Rapid playtesting:** creators can run their authored game and immediately test changes in a predictable local workflow.
- **REQ-F05 — Creator packaging:** creators can export a complete offline game via an approachable process. Players must not need a compiler, Cargo, an account, or network access.
- **REQ-F06 — Documentation and usability:** provide a starter game, beginner tutorial, and a practical usability check with someone unfamiliar with programming.
- **REQ-F07 — Game/engine isolation:** HOWLBOUND is created using the same public authoring mechanisms available to other creators; no exclusive story logic in core engine.

Story authoring decisions: TOML files, terminal tools, no GUI. Exact packaging method remains open. See `CREATOR_VISION.md`.

## 9. Scope and story-content boundary

This requirements document intentionally does **not** choose the setting, plot, genre, protagonist, lore, exact items, or puzzle solutions. A generic fixture world is permitted for tests, but should be replaceable later. Story design will have its own document once decided.

## 10. Submission deliverables

- Runnable Rust/Cargo source project in a public GitHub repository.
- Root `README.md` using the instructor-provided CSE 310 template (obtain/fill it rather than assuming an invented template is equivalent).
- Working executable / clear build and run instructions; offline demonstration.
- Video showing the program in use and walking through source code, explicitly identifying the Rust requirements.
- Sprint plan and accurate time log under the professor's approved treatment of this semester's existing work as a self-contained two-week sprint.
- Test results and source/reference links used while learning Rust.

**Instructor-specific note:** The professor has approved counting this semester's existing project work within the self-contained sprint. Document honestly; do not fabricate hours or development dates.

## Implementation status checkpoint (2026-11-16)

- v0.3.0 reusable engine and v0.4.0 independent game support verified on Windows.
- v0.5.0-A runtime-owned data verified: 18 automated tests, Clippy and interactive checks, commit `97b9476` pushed.
- v0.5.0-B TOML loader and CLI story launching are candidates requiring Windows verification before release.
- Not yet implemented: manual save/load and `.rork`, creator generator/export, fully packaged offline player, Linux verification.
- Course-specific README template, time log and demonstration remain separate deliverables.

## Engine-level manual save/load — confirmed post-v0.5.0 requirement

This section supersedes earlier descriptions of save/load as optional for Lupine's final release; it remains **out of scope for v0.5.0-A**.

- Every Lupine game supports the built-in player commands `save`, `load`, and `saves`, including named slots. These are reserved engine commands.
- Saving is exclusively player-initiated: **no autosaving, checkpoint saving, or automatic saving on quit**.
- Saves preserve mutable `GameState` (room, item locations/inventory, flags and future supported state), not static world definitions.
- Native extension: `.rork`, encoded as TOML; proposed filename `<normalized-save-name>.<stable-game-id>.rork`, e.g., `forestcamp.howlbound.rork`.
- Save naming must handle collisions and request confirmation before overwriting an existing slot.
- Load validation checks format/schema version, game identity, references, and compatibility **before mutating current state**.
- Store per-game saves in user-data locations, independent of installation paths. The file contents must be portable across Windows/Linux when the same game version is compatible.
- Protect against partial writes and report corrupt or incompatible saves without panics or accidental overwrites.
