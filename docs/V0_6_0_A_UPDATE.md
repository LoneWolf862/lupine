# Lupine v0.6.0-A — RORK Serialization

Incremental patch over verified v0.5.0-C (commit `77d5bb4`).

- Adds `src/save.rs`: manual, explicit TOML save snapshots via `encode_save` and `decode_save`.
- Adds `tests/rork_format.rs`: seven tests for serialization roundtrip, state restoration, deterministic output, wrong-game and schema rejection, corrupted files, invalid state and no mutation on failed load.
- Updates crate to `0.6.0-alpha.1`, exports save module, and updates RORK specification and roadmap.
- **Not yet implemented:** gameplay save/load commands, save-directory/slot naming, filesystem writes, overwrites, and unsaved-progress prompts. These are next.

After extracting run: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets -- -D warnings`. Check `git diff --stat` before committing.
