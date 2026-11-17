# Lupine v0.7.0-C — Creator Experience (Windows test candidate)

## New commands

- `lupine new`: terminal wizard asks for destination, stable game ID, and title; no files created until all answers are collected. Noninteractive `new <folder> <id> <title...>` still works.
- `lupine add art <story> <id>`: generates `art/<id>.txt` with editable ASCII content.
- `lupine add art <story> <id> --dynamic`: generates `art/<id>_inactive.txt` and `art/<id>_active.txt`, displays an example `art_file` and `[[art_variants]]` snippet with flag `<id>_active`.
- `lupine help`: documents these commands.

Generated art files are plain UTF-8 text, and are never added to rooms automatically. Copy the suggested fields into `rooms/*.toml` **above** `[exits]` and define an interaction to set the flag. Generators refuse to replace existing files. There is no autosave.

### Test on Windows

```cmd
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- add art my_first_adventure ancient_symbol --dynamic
cargo run -- validate my_first_adventure
cargo run -- help
```

To test the wizard without accidentally overwriting `my_first_adventure`, use `cargo run -- new` and enter a **new folder name**. Local generated adventure directories should be in `.gitignore`.

## Compatibility

Existing `new`, `add room`, `add item`, `validate`, `play`, and RORK `save`/`load`/`saves` are unchanged. No new dependencies or GUI. Do not claim release readiness until local tests and interactive checks pass.
