# Lupine v0.7.0-A — text-only project creation (candidate)

Adds `lupine new <directory> <game-id> <title words...>` and `lupine help`.

Example on Windows in the source repository:

```cmd
cargo run -- new my_adventure my_adventure My First Adventure
cargo run -- validate my_adventure
cargo run -- play my_adventure
```

`new` creates a minimal valid one-room game using versioned TOML, with a README and empty `items/` folder and `interactions.toml`. The adventure can be modified without Rust. The command refuses to overwrite an existing directory and checks game IDs before writing. It does not create saves or autosave; in-game manual `.rork` commands are unchanged.

This update does **not** yet implement `add room` or `add item`. Those are candidates for v0.7.0-B. The generator is deliberately noninteractive for simple repeatable use; a guided prompt may be added later.

Verify `cargo fmt --all`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`. Verify project generation from a fresh folder and check that the generated story passes `validate` and `play` on Windows and eventually CachyOS.
