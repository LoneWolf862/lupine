# Lupine v0.3.0 architecture update (candidate)

This package is intended to be extracted over v0.2.0 + the documentation update at commit `dfa23f9`.

## Changes

- Story-independent engine remains in `src/world.rs`, `src/state.rs`, `src/engine.rs`, `src/parser.rs` and `src/render.rs`.
- The original test world is **moved** from `src/world.rs` to `examples/fixture_world.rs`. The CLI demo imports it, but the engine library does not.
- Worlds now define conditional interaction requirements and lists of effects, rather than a hardcoded `flag` field.
- Added `ExitRule` for gated movement and `DescriptionVariant` for state-dependent room/item text.
- Added `World::validate()` with multiple clear error messages for common authoring mistakes.
- Added independent-world tests (`tests/reusability.rs`) proving the same core can run unrelated world definitions.
- Added `docs/CREATING_A_GAME.md` for developers.

## Migration / compatibility

`World`, `Room`, `Item`, and `Interaction` gained or changed fields. Any game definitions outside of the included example must be migrated to the new API. This is an intentional pre-1.0 breaking change.

### Important

This is a **source overlay**, not a complete project. Extract at the repository root and keep the existing `.git`, `Cargo.toml`, `Cargo.lock`, docs and parser. No Rust tests were run in the packaging environment because Cargo is not installed there. Run all checks before committing.

## Verification on Windows (Command Prompt)

```cmd
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```

In gameplay: `take key`, `north`, `east`, `north` (blocked), `use key on panel`, `look` (changed), `examine panel` (changed), `north` (vault accessible).

After checks pass, review `git status --short` and `git diff`, then commit/push.
