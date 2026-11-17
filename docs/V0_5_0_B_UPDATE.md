# v0.5.0-B — TOML story loader (candidate; testing required)

Introduces `src/story_loader.rs`, a distinct serde/TOML DTO layer mapping disk-authored content into the existing `World` model. Maintains the Rust-defined world API and fixture executable.

## Test on Windows

```cmd
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- play story_example
```

In the external story, run `north`, `take key`, `use key on door`, `look`, `north`, `quit`. Verify locked passage, unlock, state-dependent description, workshop entry.

Also run `cargo run` and `cargo run --manifest-path samples/independent_game/Cargo.toml` to check Rust-defined worlds.

## Limits

- `format_version = 1`, one `.toml` file per room/item; optional `interactions.toml`.
- Six cardinal/vertical direction names; basic condition/effect variants from v0.4.
- Validates world IDs and relationships with existing `World::validate` after loading.
- Does not implement creator commands (`lupine new`), packaging, save/load, or `.rork` serialization.
- Adds small compile-time dependencies `serde` and `toml`; gameplay remains completely offline after build.

No release claim until tests and interactive checks are confirmed.
