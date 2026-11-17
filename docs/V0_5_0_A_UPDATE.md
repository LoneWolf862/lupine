# Lupine v0.5.0-A — Owned World Data

**Status:** development candidate; requires local Cargo verification before commit.

## Why this update exists

In v0.4.0, world IDs, text, and flags used `&'static str`. That works for compiled-in Rust stories, but does not support room names or descriptions read from files at runtime without undesirable lifetime workarounds. This update makes world definitions and state own their strings (`String`).

## Changes

- Room, item and flag IDs now use owned `String` aliases rather than `&'static str`.
- Room/item descriptions, names, aliases, success/failure messages and interaction text own their strings.
- GameState owns the current room ID, item-location IDs and world flags.
- Lookup APIs accept `&str` to avoid forcing a new allocation on each lookup.
- Existing Rust-authored games use `.into()` for string literals in their definitions.
- One added integration test constructs every world field from runtime-allocated strings and exercises an unlock puzzle.

**Breaking Rust API change:** `ItemLocation` is no longer `Copy`, and `GameState::location()` returns a cloned `Option<ItemLocation>`. Users of Lupine's early Rust API may need to use `.into()` and borrow IDs with `&`.

## Not yet implemented

- TOML story loading, format validation, or CLI authoring commands (v0.5.0-B/C).
- `save`, `load`, `saves`, or `.rork` persistence (planned after v0.5.0).
- Autosaves are intentionally *not* planned.

## Validation on Windows

In the repository root:

```cmd
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
cargo run --manifest-path samples/independent_game/Cargo.toml
```

The nested independent sample is a separate package. The root `cargo test` does not test it; run the sample explicitly. Cargo will regenerate the root `Cargo.lock` for the pre-release version; commit that change.

## Engine-level save policy (design, not implemented)

Saving belongs to Lupine rather than each game. Players deliberately use `save`, `load`, and `saves`; there will be **no autosave, checkpoint save, or save-on-quit**. Named saves use a portable, human-readable TOML-based `.rork` format, with a filename like `forestcamp.howlbound.rork`, derived from a stable game ID and collision-safe save slot. Files include schema version and game identity; loading must validate before applying changes. Writes should protect existing saves against partial writes. Local, offline and per-game isolated storage is required. Details will be finalized during the save-system milestone.
