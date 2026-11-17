# Lupine v0.5.0-C — candidate update

Building on v0.5.0-B (commit `655a5df`):
- `lupine validate <story-directory>` validation command, no gameplay launched.
- Source-file-aware diagnostics for common missing/duplicate room and item references.
- Copyable `templates/starter/` with a short starter README.
- Step-by-step text authoring guide in `docs/CREATING_A_GAME.md`.
- Additional integration tests for template and common creator errors.

**Verification required on user machine:** `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo run -- validate story_example`, and `cargo run -- play templates/starter`. Do not commit until these pass.

Still future: manual `.rork` saving in v0.6.0; never autosave. No GUI.
