# Lupine Room Art (v0.7.0-B)

Room art is **optional**. It is displayed on entering a room and on `look`, after the description and before exits. It is part of world content, **not** RORK save state. No GUI or extra rendering dependency is necessary.

## Recommended: external text files

Create `art/symbol.txt` in your game directory, with the literal text drawing. In a room such as `rooms/chamber.toml`:

```toml
id = "chamber"
title = "Ancient Chamber"
description = "A symbol is carved into the floor."
art_file = "symbol.txt"

[exits]
south = "forest"
```

`art_file` is resolved relative to the game's **`art/` folder**, *not* relative to the room file. Both `art_file` and variant `file` use this rule. Nested relative files such as `symbols/door.txt` are supported, but `..`, absolute paths, and symbolic links escaping `art/` are rejected. Art files are plain UTF-8 text, up to 64 KiB, with no terminal control sequences. Whitespace and line breaks are preserved.

## Dynamic artwork using flags

The initial art might show a sealed door. An interaction sets the flag `door_open`; then the art changes:

```toml
art_file = "door_locked.txt"

[[art_variants]]
when = [{ flag_set = "door_open" }]
file = "door_open.txt"
```

`when` accepts existing conditions, such as `flag_set`, `flag_unset`, and `has_item`, using the same syntax as interaction rules. **The first matching variant wins**. If no variants match, default `art_file` (or inline `art`) is displayed; if there is no default, no art is shown. The artwork updates whenever the room description is rendered (`look`, entering a room, or loading a save into it). Because flags and item positions are stored in RORK, the correct art returns when loading a save.

`art_variants` can also contain `text = '''...'''` instead of `file`. A variant requires one non-empty `when` list and exactly one of `file` or `text`.

## Optional embedded artwork

```toml
art = '''
   /\
  /__\
'''
```

Use TOML triple **single** quotes for literal artwork with backslashes. Use either `art` or `art_file` on a room, never both.

## Creating content

```sh
lupine add room MyAdventure chamber
lupine add item MyAdventure lantern
lupine validate MyAdventure
lupine play MyAdventure
```

The generator creates editable files and refuses to overwrite files that already exist. A generated room is not automatically connected to the map—edit an existing room's `[exits]` and optionally your new room's exits. An item starts in the adventure's starting room; edit `starting_room` as desired.

See `story_example/` for a fully functional room that changes its artwork when a door unlocks.

## Generate artwork from the terminal (v0.7.0-C)

```sh
lupine add art MyAdventure sigil
lupine add art MyAdventure door --dynamic
```

The first command creates `art/sigil.txt`; add `art_file = "sigil.txt"` near the top of your room TOML (before `[exits]`). The second creates `art/door_inactive.txt` and `art/door_active.txt`. Configure your room with:

```toml
art_file = "door_inactive.txt"
[[art_variants]]
when = [{ flag_set = "door_active" }]
file = "door_active.txt"
```

Add `{ set_flag = "door_active" }` to the relevant interaction's `effects` array. The new files are templates; edit them with your preferred text editor. An existing file is never overwritten, and the generator does not edit your room TOML automatically.
