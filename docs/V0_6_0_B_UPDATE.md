# Lupine v0.6.0-B — Interactive RORK commands

**Status:** Candidate for testing; not yet verified on Windows/Linux.

## New functionality

- Engine-owned `save`, `load`, and `saves` commands in the terminal runner.
- `save Forest Camp` writes `forestcamp.<game-id>.rork` in a per-game user-data directory.
- Bare `save` and `load` prompt for a name. `saves` lists saved slots.
- Existing slot overwrites prompt for **yes**. Colliding normalized names never silently overwrite.
- Loading restores inventory, item placement, room and flags, then renders the restored room.
- Uncommitted changes prompt before `load` replaces current state or `quit` discards it.
- No save occurs on quit, EOF, movement, interaction, loading, or validation. **No autosaving.**

## Save location

- Windows: `%APPDATA%\\Lupine\\saves\\<game-id>\\`
- Linux: `$XDG_DATA_HOME/lupine/saves/<game-id>/` or `~/.local/share/lupine/saves/<game-id>/`.

The TOML adventure's manifest `id` is authoritative; Rust-authored games should call `runner::run_with_game_id(world, title, stable_id)`. The compatibility `run(world, title)` API derives an ID from the title, so renaming a game can rename its save namespace.

## Stage C intentionally outstanding

- Crash-safe atomic replacement and safe rollback on interrupted overwrite.
- Game content-version fingerprints and compatibility policy.
- More comprehensive filesystem hardening, limits, permissions and cross-platform verification.

## Test checklist

1. Run `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets -- -D warnings`.
2. Run `cargo run -- play story_example`.
3. `take key`, `use key on door`, `save Forest Camp`, `saves`, `north`, `load Forest Camp`, `inventory`.
4. Confirm that `forestcamp.example_adventure.rork` exists under your user save directory.
5. Exit the game, launch it again, `load Forest Camp`, and verify unlocked door and inventory.
6. Test `save Forest Camp` again and answer `no` to the overwrite prompt.
7. Verify unsaved-progress warning on `quit` and load; saving is never automatic.

**Do not commit until local tests and interactive gameplay pass.**
