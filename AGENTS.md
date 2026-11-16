# Lupine — contributor / coding-agent instructions

## Identity and product boundaries

- **Lupine** is a reusable, open-source, offline Rust **interactive fiction engine**, not a particular narrative.
- **HOWLBOUND** is the *tentative* first original adventure and a real-world test of Lupine. Do not invent or commit its setting, plot, characters, puzzles, or lore without the creator's direction.
- Preserve the distinction between **engine**, **example/test fixtures**, **story content**, and any **future authoring tools**.
- The long-term final release must enable **nontechnical authors** to create, validate, test, and package games without writing Rust. This is **not** a v0.3.0 or course-MVP gate. Read `docs/CREATOR_VISION.md`.

## Architecture rules

- Deterministic, small, offline terminal *runtime*: no LLM, cloud calls, accounts, web server, or networking dependency at play time.
- Separate normalization/parsing, object resolution, rule validation, state mutation, and output formatting.
- Do not hardcode a particular game's puzzle or prose in the parser or generic engine. Represent eventual story mechanics using reusable data-oriented conditions/effects.
- Prefer stable identifiers and immutable content definitions distinct from mutable `GameState`.
- Preserve Windows 10 and CachyOS/Linux portability; avoid machine-specific paths, commit `Cargo.lock`, and do not commit `target/`.
- Choose the smallest maintainable implementation. Prefer the Rust standard library unless a dependency solves a concrete need.
- Future game content should be loadable without editing engine Rust code. **No story-file schema, editor, or loader is implemented yet**; don't present future examples as supported syntax.

## Development workflow

- Read `README.md`, `docs/REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, and `docs/CREATOR_VISION.md` before architectural changes.
- Work in small, reviewable increments. Add tests for parser ambiguity, missing objects, invalid worlds, and state transitions as features arrive.
- Before a milestone is marked verified, run `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets -- -D warnings` on an available Rust toolchain.
- Update documentation and record what is **implemented**, **proposed**, or **not verified**; do not claim features exist until tested.
- The user prefers collaborative file/patch/ZIP updates and reviews changes before implementation; don't assume Codex automation is the default.
- Protect CSE 310 requirements and the instructor README-template obligation; do not invent time logs or submission evidence.

## Scope control

- v0.3.0 priority: reusable engine API, fixture separation, generic conditions/effects, dynamic descriptions, and tests, without overbuilding an editor or authoring DSL.
- A human-readable content format, validator, guided creation workflow, and possibly Lupine Studio are later-stage goals; see roadmap.
- Do not add combat, NPC systems, networking, mod hosting, save compatibility, or complex parser features without an explicit milestone decision.
