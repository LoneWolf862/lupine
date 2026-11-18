# Lupine Creator Quickstart — No Rust Source Editing

Use an existing Lupine binary (`lupine`) or prefix commands with `cargo run --` while developing from source.

1. Run `lupine new` and choose the title, stable game ID, new project location, starting room title, and description; review the exact file previews and confirm creation (or `lupine new new_game new_game My Game` without prompts).
2. Modify `rooms/start.toml`. Add new locations with `lupine add room new_game tower`. **Connect** the location by editing `[exits]` in an existing room: `north = "tower"`. Optionally add a matching `south` exit in `tower.toml`.
3. Add objects with `lupine add item new_game lantern`. Edit its title, aliases, portability, description, and starting room.
4. Add optional ASCII art with `lupine add art new_game symbol` and reference `art_file = "symbol.txt"` in a room. For artwork that changes when a puzzle flag changes, run `lupine add art new_game symbol --dynamic` and copy the printed room TOML example.
5. For puzzle logic, add `[[interactions]]` and `[[exit_rules]]` to `interactions.toml` using the examples from `story_example/`. Conditions: `flag_set`, `flag_unset`, `has_item`; effects: `set_flag`, `clear_flag`, `move_item_to_room`.
6. Run `lupine validate new_game`; correct any reported errors. Then `lupine play new_game` to playtest. During play, `save My Slot`, `load My Slot`, and `saves` work, **only when explicitly invoked**. Lupine never autosaves.

`lupine help` shows the available commands. Story content is ordinary `.toml` and `.txt`; game-wide settings use `.fig`. Room ASCII artwork should be UTF-8 without terminal control sequences, with a maximum size of 64 KiB per artwork file. See `docs/ROOM_ART.md` for full details.

Optional engine configuration uses TOML syntax in config/character.fig (RPG opt-in) and config/terminology.fig (display labels). No config files are required for a narrative-only game. See docs/CONFIG_FIG_SPEC.md in the source repository. Root character.toml is obsolete; move it into config/character.fig. Player saves remain strictly manual .rork files.

## Phase 2A wizard controls and safety

Enter accepts the displayed default; `?` gives field help; `cancel` or end of input stops before creation. Invalid answers explain the problem and repeat the prompt. At final confirmation, Enter means **No**. To enter literal `cancel` or `?`, prefix it with a backslash.

For a multiline room description, enter `/multi`, type the lines (including blank lines), then enter `/end`. Prefix a literal control line with a backslash; double an initial backslash in description text to preserve it. The wizard serializes quotes, Unicode and line breaks as valid TOML. It shows every generated file's destination and exact contents before asking for confirmation.

Choose a new local folder inside an existing parent directory. Existing folders/files, parent traversal, symlink/reparse components, device/network paths and Windows-reserved names are rejected. The noninteractive `new PATH ID TITLE` command remains available with the same default starting room. `add room`, `add item` and `add art` retain their existing interfaces.

Files are staged alongside their destination, synced, and published using exclusive hard links. Filesystems without hard-link support fail safely. Handled project failures remove only files created by that operation and empty directories, reporting cleanup failures. This is a recoverable multi-file sequence, not a crash-atomic project transaction: forced termination may leave complete files or temporary files in a new project folder. Inspect that folder before removing or retrying; the wizard will never overwrite it. Do not edit the project concurrently during creation.

The minimal game is checked through the existing story loader before success is reported. It needs no FIG files and enables no RPG features. Guided room linking, items, puzzles, FIG editing, back/skip navigation and additional interaction modes remain future work.
