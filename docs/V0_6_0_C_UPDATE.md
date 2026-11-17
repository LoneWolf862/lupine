# Lupine v0.6.0-C — RORK Reliability Candidate

**Status:** Implementation candidate; test on Windows and Linux before merging. Builds on v0.6.0-B.

## Changes

- Save files are written to uniquely named sibling temporary files using create-new semantics, flushed and synced, and then published with a filesystem operation. Confirmed overwrite uses `rename` rather than truncating the previous slot. A failed staging operation leaves the original file untouched.
- A failed overwrite is intentionally injected during unit testing, verifying the old slot stays byte-for-byte intact and the temporary file is removed. New-save failures and successful replacement are also tested.
- `game.toml` accepts optional `save_compatibility_version = 1` (default 1). Each newly saved `.rork` file records that revision. Game authors increase it **only for incompatible changes**. Loading checks the revision *before* returning a replacement state. Structural validation is still performed on every load.
- Existing RORK v1 saves without a compatibility field continue to load as compatibility revision 1; `save_format_version` remains 1. Versions refer to game-content compatibility, not engine release numbers.
- Rust-authored game APIs remain compatible and default to compatibility revision 1; `run_with_game_id_and_compat` and `run_with_io_and_saves_compat` allow explicit revisions.

## Limits and safety notes

- Filesystem rename replacement is intended for normal local filesystems (Windows NTFS, Linux ext4). Filesystem/platform failure returns an error instead of deleting the old slot first. Do not claim guarantees against all power failures, OS crashes or exotic/network filesystem behavior. A sudden crash can leave an orphan `.tmp` file; Lupine ignores it when listing saves.
- This revision prevents common partially written overwrite scenarios but does not implement a backup/recovery UI, cross-process locking, migrations, or cryptographic corruption checks. Additional safety testing on Windows and CachyOS/Linux is advised.
- Games are saved **only** when the player explicitly executes `save`. No autosaving, periodic writes, checkpoint saving, or save-on-quit.

## Windows verification

1. Run `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets -- -D warnings`.
2. Play `cargo run -- play story_example`, then `save Forest Camp`; change the state and `save Forest Camp` again, confirming overwrite.
3. Exit and restart; `load Forest Camp` must restore the latest saved state.
4. Check the `.rork` file contains `save_compatibility_version = 1`.
5. Previously created v0.6.0-B saves should still load unchanged.

**Do not commit until compiler, Clippy and gameplay checks pass.**
