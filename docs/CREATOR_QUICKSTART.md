# Lupine Creator Quickstart — No Rust Source Editing

Use an existing Lupine binary (`lupine`) or prefix commands with `cargo run --` while developing from source.

1. Run `lupine new` and answer the destination, game ID, and title prompts (or `lupine new new_game new_game My Game` without prompts).
2. Modify `rooms/start.toml`. Add new locations with `lupine add room new_game tower`. **Connect** the location by editing `[exits]` in an existing room: `north = "tower"`. Optionally add a matching `south` exit in `tower.toml`.
3. Add objects with `lupine add item new_game lantern`. Edit its title, aliases, portability, description, and starting room.
4. Add optional ASCII art with `lupine add art new_game symbol` and reference `art_file = "symbol.txt"` in a room. For artwork that changes when a puzzle flag changes, run `lupine add art new_game symbol --dynamic` and copy the printed room TOML example.
5. For puzzle logic, add `[[interactions]]` and `[[exit_rules]]` to `interactions.toml` using the examples from `story_example/`. Conditions: `flag_set`, `flag_unset`, `has_item`; effects: `set_flag`, `clear_flag`, `move_item_to_room`.
6. Run `lupine validate new_game`; correct any reported errors. Then `lupine play new_game` to playtest. During play, `save My Slot`, `load My Slot`, and `saves` work, **only when explicitly invoked**. Lupine never autosaves.

`lupine help` shows the available commands. Story content is ordinary `.toml` and `.txt`; game-wide settings use `.fig`. Room ASCII artwork should be UTF-8 without terminal control sequences, with a maximum size of 64 KiB per artwork file. See `docs/ROOM_ART.md` for full details.

Optional engine configuration uses TOML syntax in config/character.fig (RPG opt-in) and config/terminology.fig (display labels). No config files are required for a narrative-only game. See docs/CONFIG_FIG_SPEC.md in the source repository. Root character.toml is obsolete; move it into config/character.fig. Player saves remain strictly manual .rork files.
