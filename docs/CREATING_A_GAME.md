# Creating your first Lupine adventure (TOML format 1)

**This guide uses v0.5.0-C candidate features.** No graphical interface or Rust game code is required.

## 1. Copy the starter project

Copy `templates/starter/` into a folder you own, for example `my_adventure/`. Keep its `rooms/` and `items/` subfolders.

## 2. Set your game's identity

Open `game.toml` and change `id` to a **stable**, lowercase identifier, such as `my_adventure`; change `title` to the player-facing title. `starting_room` must equal one room's `id`. Keep `format_version = 1`.

## 3. Write a room

In `rooms/foyer.toml`, edit the `title` and `description`. To add another room, make `rooms/garden.toml`:

```toml
id = "garden"
title = "Garden"
description = "An old garden runs along a stone wall."

[exits]
south = "foyer"
```

In `rooms/foyer.toml`, add `north = "garden"` beneath `[exits]`. Both directions are optional; rooms need not have reciprocal exits. Supported directions are `north`, `south`, `east`, `west`, `up`, `down`.

## 4. Define objects

Each item goes in `items/<name>.toml`. The `starting_room` matches a room ID, not its title. `portable = true` enables taking and dropping; `portable = false` makes an object stationary. See `items/brass_key.toml` for an example.

## 5. Add puzzles

The starter's `interactions.toml` demonstrates a key, a door, an unlock flag, and a conditional exit. Use the same structure when changing the puzzle. Conditions supported now: `has_item`, `flag_set`, `flag_unset`. Effects: `set_flag`, `clear_flag`, `move_item_to_room`. See `STORY_FORMAT_PROPOSAL.md` for syntax.

## 6. Validate before play

From the Lupine source directory:

```sh
cargo run -- validate my_adventure
cargo run -- play my_adventure
```

Replace `my_adventure` with the actual relative or absolute folder path. When using a prebuilt Lupine executable, omit `cargo run --` and use `lupine validate ...` or `lupine play ...`. Validation does **not** start a game or edit story files.

If validation reports a filename and an unknown room/item, check that the referenced ID is spelled exactly as defined in another file. The starter is deliberately small: duplicate it and make one change at a time.

## Configuration and manual saves

Game content remains TOML. Optional engine settings belong in config/character.fig and config/terminology.fig; see CONFIG_FIG_SPEC.md for supported fields and defaults. The former root character.toml file is obsolete and must be moved, not duplicated.

`new`, `add room`, `add item`, `add art` and manual `save`, `load`, `saves` are implemented. Never autosave. Full guided wizard editing is future work. Portable build automation exists; published releases and native platform verification must be reported separately.
