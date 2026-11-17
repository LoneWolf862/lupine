# Lupine v0.7.0-B candidate — Room/Item Generators and Dynamic Text Art

## Changes

- Added terminal `add room` and `add item` commands, with stable IDs, input validation, and exclusive file creation.
- Added optional room `art`/`art_file` and dynamic `[[art_variants]]`, resolved through existing `Condition` types.
- Plain UTF-8 drawings live under the story `art/` directory; relative path confinement, file-size and control-character checks apply.
- Updated all Rust-authored worlds to include empty optional art fields. This changes the pre-v1 `Room` struct literal API.
- Added a working dynamic-art sample with locked and open door drawings and new integration tests.
- No changes to RORK format; flags already persist across saves. No autosaving.

## Validation (run locally before commit)

```cmd
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- play story_example
cargo run -- add room my_first_adventure cabin
cargo run -- add item my_first_adventure lantern
cargo run -- validate my_first_adventure
```

`add room` creates a room but does not automatically connect exits; adjust room files manually. Test `look` before unlocking and after `use key on door`, then `load` a previously saved state.
