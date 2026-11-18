# Lupine Creator Quickstart — No Rust Source Editing

From the Lupine source folder, run `cargo run -- new`. With a standalone executable in your current folder, use `.\lupine.exe new` on Windows or `./lupine new` on Linux. No global PATH installation is required. In the command examples below, replace `lupine` with `cargo run --` when working from source.

1. Run the wizard and press Enter to begin. Beginner mode explains four steps: name your adventure, choose a new folder, describe your starting location, and review. At review, Enter creates the adventure; choose 2 to inspect the files, 3 to revise answers, or 4 to cancel. Type `advanced` at the welcome prompt for shorter prompts, an editable ID and direct previews. Noninteractive `lupine new new_game new_game My Game` remains available.
2. Modify `rooms/start.toml`. Add new locations with `lupine add room new_game tower`. **Connect** the location by editing `[exits]` in an existing room: `north = "tower"`. Optionally add a matching `south` exit in `tower.toml`.
3. Add objects with `lupine add item new_game lantern`. Edit its title, aliases, portability, description, and starting room.
4. Add optional ASCII art with `lupine add art new_game symbol` and reference `art_file = "symbol.txt"` in a room. For artwork that changes when a puzzle flag changes, run `lupine add art new_game symbol --dynamic` and copy the printed room TOML example.
5. For puzzle logic, add `[[interactions]]` and `[[exit_rules]]` to `interactions.toml` using the examples from `story_example/`. Conditions: `flag_set`, `flag_unset`, `has_item`; effects: `set_flag`, `clear_flag`, `move_item_to_room`.
6. Run `lupine validate new_game`; correct any reported errors. Then `lupine play new_game` to playtest. During play, `save My Slot`, `load My Slot`, and `saves` work, **only when explicitly invoked**. Lupine never autosaves.

`lupine help` shows the available commands. Story content is ordinary `.toml` and `.txt`; game-wide settings use `.fig`. Room ASCII artwork should be UTF-8 without terminal control sequences, with a maximum size of 64 KiB per artwork file. See `docs/ROOM_ART.md` for full details.

Optional engine configuration uses TOML syntax in config/character.fig (RPG opt-in) and config/terminology.fig (display labels). No config files are required for a narrative-only game. See docs/CONFIG_FIG_SPEC.md in the source repository. Root character.toml is obsolete; move it into config/character.fig. Player saves remain strictly manual .rork files.

## Beginner and advanced wizard controls

Enter keeps the displayed suggestion; `?` gives field help; `back` returns to the previous question; `cancel` or end of input stops before creation. At the first question, `back` explains that you are already at the beginning. Invalid answers explain the problem and repeat the prompt. Earlier answers are retained as defaults. Review option 3 returns to the adventure name, keeping your other answers. Review option 1 (also Enter) creates the project; it does **not** launch gameplay.

To enter literal `cancel`, `back` or `?`, prefix the whole answer with a backslash, such as `\back`. These controls do not match parts of sentences. For a multiline description, enter `/multi`; the wizard then explains how to enter lines, including blank lines, and finish with `/end`. `back` discards that unfinished edit and retains the previous completed description. Escape literal control lines with a backslash; double an initial backslash in description text to preserve it. Quotes, Unicode and line breaks are serialized safely as TOML.

Beginner mode generates an internal ID from ASCII words in the title, separated by underscores (`The Forgotten Forest` becomes `the_forgotten_forest`). Unsupported characters are separators; titles with no ASCII letters/digits and reserved names fall back to `my_adventure`. IDs are limited to 80 characters before any collision suffix. The wizard checks valid `game.toml` IDs in immediate sibling adventure folders and adds `_2`, `_3`, etc. in Beginner mode. Advanced mode asks for a different ID on a local collision. A custom advanced ID survives display-name edits; automatic IDs follow the title during setup. Once players begin saving, keep the ID stable.

There is no global ID registry: games elsewhere on your computer, copied projects, malformed manifests, and concurrent creators may escape the local collision check. Choose distinct advanced IDs across your games. Noninteractive creation keeps its existing explicit-ID behavior.

A relative folder name is resolved inside the current working folder; a full path specifies another location. The wizard shows the resolved absolute destination before creation. If the suggested folder exists, a numbered suggestion is offered. Explicit existing destinations are rejected and reprompted. Choose a new local folder inside an existing parent. Parent traversal, symlink/reparse components, device/network paths, Windows drive-relative paths and reserved names are rejected.

## Review, play and continue

Beginners see a summary first. Review option 2 explains TOML/FIG and displays the exact serialized contents of every generated file; advanced users see these directly. Explanations sit outside the file-content markers. The preview and disk bytes come from the same serialized drafts, rebuilt after edits. This narrative starter needs no FIG files and enables no RPG features.

Completion shows `cargo run -- play` with your absolute project path for use from the Lupine source folder. It also shows the full path of the running executable, so you can launch it directly without Cargo or PATH setup. Windows commands are explicitly written for **PowerShell** (single-quoted paths and `&` before the executable path); from Command Prompt, type `powershell` first. Linux commands use POSIX shell quoting. Commands handle spaces, apostrophes and shell metacharacters. Launch is deliberately a separate action. Try `look`, `help` and `quit` once playing; saves remain strictly manual.

Files are staged alongside their destination, synced, and published using exclusive hard links. Filesystems without hard-link support fail safely. Handled project failures remove only files created by that operation and empty directories, reporting cleanup failures. This is a recoverable multi-file sequence, not a crash-atomic project transaction: forced termination may leave complete files or temporary files in a new project folder. Inspect that folder before removing or retrying; the wizard never overwrites it. Do not edit the project concurrently during creation.

The minimal game is checked through the existing story loader before success. Existing `add room`, `add item`, `add art` and noninteractive `new` remain available. Guided room linking, items, puzzles, FIG editing and skip navigation remain future work.

## Windows manual walkthrough

1. In the Lupine repository, run `cargo run -- new`. Press Enter at the welcome prompt.
2. Name the adventure, choose a new folder (try one with spaces), and name the starting location. At the description prompt, try `back` and revise the location name. Then write a sentence, or use `/multi` and finish with `/end`.
3. At review, choose 2 to inspect the explained files. Choose 3 to revise a name; keep other answers with Enter. Choose 1 to create.
4. Copy the printed Cargo play command into PowerShell in the repository folder. Try `look`, `help`, then `quit`. Confirm the opening name and description match your answers.
5. Run the wizard again, type `advanced`, and try the existing destination; it should refuse to overwrite it. Use `cancel` to exit. Also try an invalid ID and an ID already used by a sibling adventure.
