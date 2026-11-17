# v0.8.4-C release candidate — verification

## Included changes

- Consolidates defeat output into a single runner-owned banner after combat retaliation. The enemy roster is suppressed after lethal retaliation, and gameplay remains locked until a manual load or quit.
- Moves `runner.rs` test module to the end of the file to satisfy strict Clippy.
- Corrects the new v0.8.4-B room-description regression test's `starting_room` field.
- Sets the crate and default test-adventure display version to **0.8.4**. This **does not** modify RORK save file versions, game IDs or story compatibility revisions.
- Updates README and adds a creator-oriented RPG guide.
- Adds a regression covering all bundled RPG sample loaders, plus defeat-output assertions.

## On Windows (CMD), from repository root

```cmd
cargo fmt --all && cargo test && cargo clippy --all-targets -- -D warnings
```

Then validate each sample (one example):

```cmd
cargo run -- validate samples/rpg_persistent
```

Interactive smoke test:

```cmd
cargo run -- play samples/rpg_persistent
```

Try `walet`, `stasts`, `spells`, `take dagger`, `south`, `east`, `attack guardian with dagger`. Use a saved state or fight other enemies until HP reaches zero; the banner should appear **once** and no combat enemy list should follow it. `inventory` and `shop` should be blocked. `saves` and manual `load <name>` should work. No autosave.

Check an older `.rork` save for the same demo with `load boss1` (or the actual name you previously used), and verify stats, inventory and boss state. Do not overwrite the existing save during testing.

## Release decisions

- This is a **candidate** until locally tested; no assertion is made that Cargo was run in the packaging environment.
- Windows and Linux standalone archives are deferred to **v0.9**.
- No new player-facing features beyond earlier v0.8.4 A/B gameplay polish.
