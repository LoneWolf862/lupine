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

Enter keeps the displayed suggestion; `?` gives field help; `back` returns to the previous question; `cancel` or end of input stops before creation. At the first question, `back` explains that you are already at the beginning. Invalid answers explain the problem and repeat the prompt. Earlier answers are retained as defaults. Review option 3 returns to the adventure name, keeping your other answers. Review option 1 (also Enter) creates the project; it enters the Adventure Creator in the CLI; it does **not** automatically launch gameplay.

To enter literal `cancel`, `back` or `?`, prefix the whole answer with a backslash, such as `\back`. These controls do not match parts of sentences. For a multiline description, enter `/multi`; the wizard then explains how to enter lines, including blank lines, and finish with `/end`. `back` discards that unfinished edit and retains the previous completed description. Escape literal control lines with a backslash; double an initial backslash in description text to preserve it. Quotes, Unicode and line breaks are serialized safely as TOML.

Beginner mode generates an internal ID from ASCII words in the title, separated by underscores (`The Forgotten Forest` becomes `the_forgotten_forest`). Unsupported characters are separators; titles with no ASCII letters/digits and reserved names fall back to `my_adventure`. IDs are limited to 80 characters before any collision suffix. The wizard checks valid `game.toml` IDs in immediate sibling adventure folders and adds `_2`, `_3`, etc. in Beginner mode. Advanced mode asks for a different ID on a local collision. A custom advanced ID survives display-name edits; automatic IDs follow the title during setup. Once players begin saving, keep the ID stable.

There is no global ID registry: games elsewhere on your computer, copied projects, malformed manifests, and concurrent creators may escape the local collision check. Choose distinct advanced IDs across your games. Noninteractive creation keeps its existing explicit-ID behavior.

A relative folder name is resolved inside the current working folder; a full path specifies another location. The wizard shows the resolved absolute destination before creation. If the suggested folder exists, a numbered suggestion is offered. Explicit existing destinations are rejected and reprompted. Choose a new local folder inside an existing parent. Parent traversal, symlink/reparse components, device/network paths, Windows drive-relative paths and reserved names are rejected.

## Review, play and continue

Beginners see a summary first. Review option 2 explains TOML/FIG and displays the exact serialized contents of every generated file; advanced users see these directly. Explanations sit outside the file-content markers. The preview and disk bytes come from the same serialized drafts, rebuilt after edits. This narrative starter needs no FIG files and enables no RPG features.

Completion shows `cargo run -- play` with your absolute project path for use from the Lupine source folder. It also shows the full path of the running executable, so you can launch it directly without Cargo or PATH setup. Windows commands are explicitly written for **PowerShell** (single-quoted paths and `&` before the executable path); from Command Prompt, type `powershell` first. Linux commands use POSIX shell quoting. Commands handle spaces, apostrophes and shell metacharacters. Choose Play (1) in the creator to launch directly and type `quit` to return, or use the printed command separately. Try `look`, `help` and `quit` once playing; saves remain strictly manual.

Files are staged alongside their destination, synced, and published using exclusive hard links. Filesystems without hard-link support fail safely. Handled project failures remove only files created by that operation and empty directories, reporting cleanup failures. This is a recoverable multi-file sequence, not a crash-atomic project transaction: forced termination may leave complete files or temporary files in a new project folder. Inspect that folder before removing or retrying; the wizard never overwrites it. Do not edit the project concurrently during creation.

The minimal game is checked through the existing story loader before success. Existing `add room`, `add item`, `add art` and noninteractive `new` remain available. Guided room linking, items, puzzles, FIG editing and skip navigation remain future work.

## Windows manual walkthrough

1. In the Lupine repository, run `cargo run -- new`. Press Enter at the welcome prompt.
2. Name the adventure, choose a new folder (try one with spaces), and name the starting location. At the description prompt, try `back` and revise the location name. Then write a sentence, or use `/multi` and finish with `/end`.
3. At review, choose 2 to inspect the explained files. Choose 3 to revise a name; keep other answers with Enter. Choose 1 to create.
4. Copy the printed Cargo play command into PowerShell in the repository folder. Try `look`, `help`, then `quit`. Confirm the opening name and description match your answers.
5. Run the wizard again, type `advanced`, and try the existing destination; it should refuse to overwrite it. Use `cancel` to exit. Also try an invalid ID and an ID already used by a sibling adventure.

## Continue in the Adventure Creator (Phase 2B)

From the repository, run `cargo run -- creator` to create/open an adventure, or `cargo run -- creator "my_adventure"` to reopen it. Interactive `cargo run -- new` also enters the creator after setup. Noninteractive `new PATH ID TITLE` still generates and exits; no-argument Lupine still runs the engine test adventure.

The project remains selected while you use the menu:

- **1 Play:** starts the selected adventure directly. `quit` returns to the menu; the existing unsaved-progress confirmation still applies. Each play starts fresh unless you explicitly load a save. No autosaving occurs.
- **2 Validate:** reloads story, supported FIG, RPG and related definitions through the same validators as CLI `validate`. Counts and original file/key/reference diagnostics are displayed. Errors return to the menu without changing files.
- **7 Rooms:** create/edit room names, descriptions and connections with exact previews and confirmed saves.
- **3-6:** clearly labeled future editors; they do not change files yet.
- **8 / help / ?:** explains the current interface.
- **Enter:** refreshes the actual tree after external changes.
- **0 / back / cancel:** leaves this project. If you used the Create/Open launcher, it returns there; use 0 again to exit. EOF also exits cleanly.

The tree includes existing game/interactions/transaction files, README.txt, and recognized `config/`, `rooms/`, `items/`, `spells/`, `weapons/`, `enemies/`, `shops/` and `art/` folders. Empty existing folders are shown; absent optional folders are not invented. Paths are relative to the project root and directories end in `/`. Nested text/content files appear, although their presence in the tree does not imply the runtime recursively loads every content category. Unrelated root files and temporary extensions are omitted. Unsupported FIG files are visible so validation can identify them.

The tree displays at most 160 entries and abbreviates paths over 180 characters. Inspection is limited to 2048 entries and 8 nested directory levels; reaching that limit blocks creator validation/play with an explanation rather than silently skipping safety checks. Linked files, symlinks, junctions and other Windows reparse points are not followed. Regular projects with invalid TOML can open for diagnosis; a regular game.toml and a safe existing root are required. These are read-time checks, not protection against a hostile process concurrently swapping filesystem objects. Avoid modifying the project while a load is in progress.

Navigation, play and validation do not rewrite content. Room editing now uses the separately tested Phase 2C persistence model below. Other guided content editors remain future work. Unsupported schema fields still fail normal validation.

### Phase 2B Windows review

1. Run `cargo run -- creator`; create a new adventure in a folder with spaces. Confirm the real tree and menu appear after setup.
2. Choose 2 to validate; choose 1 to play, then `look` and `quit`. Confirm you return to the same selected adventure.
3. Try an invalid menu number, help, and a future-editor option. Each should remain in the creator without changing files.
4. Exit with 0, then 0 at the launcher. Reopen with `cargo run -- creator "your project folder"` and validate again.
5. Optionally edit a disposable room file in a text editor while at the menu. Press Enter to refresh and validate. A bad reference should report an error and keep the menu usable; restore the file afterward.

## Guided Room Manager (Phase 2C)

Choose **7** in the Adventure Creator. Select **N** to add a location, or select an existing room by number. A room can be any place: a forest, cave, village or spaceship. Give it a display name, accept the suggested internal ID (or customize it), and describe what players see. IDs must be unique lowercase letters/digits/dashes/underscores; existing IDs cannot be renamed here. Display names can always be changed. `/multi` starts a multiline description; `/end` finishes. `{{objects}}` inserts the names of currently visible objects during play.

The connection menu explains travel direction. Choose **1** to connect another existing room **to** the selected room; choose **2** to connect the selected room **to** another. Select the other room and a direction, then decide whether to add the reverse direction. The reverse prompt shows the exact route. Answering no makes only the requested edge; it does not delete existing edges. Conflicts require a separate yes to replace an exit. Additional connections can be added before saving. Option 3 removes a selected outgoing exit after confirmation; it does not automatically remove the reverse exit or puzzle rules. Leaving a room disconnected may make it unreachable.

Room Options:

1. Change display name (stable ID remains fixed).
2. Edit description, including multiline text.
3. Manage connections.
4. Review current properties, incoming/outgoing exits, items and preserved advanced settings.
5. Preview exact proposed TOML for all changed rooms (or the selected room's saved file when unchanged).
6. Save the room and all drafted connections, after a summary and explicit **yes**. Enter at confirmation means **no**.
0. Return to Room Manager, discarding this unsaved draft.

`?` gives help; `back` returns from a question/menu; `cancel` abandons the entire current draft. Prefix literal control words with a backslash. EOF abandons unsaved work and exits through the existing creator behavior. At creation's name/ID/description steps, back revisits the earlier question. After creation, use Room Options to revise any supported answer. A failed save stops the operation and reloads actual files; reopen the room to retry. Successful saving validates the whole adventure, refreshes the real file tree, and returns to Room Manager. Choose a saved room again to inspect its result.

Existing conditional descriptions, art references/variants and encounter tables survive ordinary edits. Comments and formatting are preserved through TOML-aware editing; modified scalar quoting may change, while CRLF file style is retained. Item/ability/art/FIG/conditional-rule editors remain future phases. Unsupported schema fields cause normal validation errors rather than being silently removed. Preexisting invalid projects must be corrected before room editing.

### Room-write safety and recovery

All proposed room content is held in memory until explicit Save confirmation. The editor snapshots recognized content and metadata, validates a private staged project with the real validator, rechecks for external edits, and publishes complete files without truncation. Multi-file failures trigger rollback. If an external change prevents safe rollback, recovery files remain and the error identifies them. Do not edit or play the same project concurrently with publication. Player saves remain completely separate and manual.

The private `.lupine-room-edit/` folder is an edit lock and recovery folder. It normally disappears after success or rollback. If it remains, further room edits are refused. **Do not delete it blindly.** Read RECOVERY.txt and CHANGES.txt. Retained `original/rooms/...` files are the originals; `proposed/` is the proposed project. A COMMITTED.txt marker means publication and validation finished, but cleanup may not have. Inspect the current project and preserve any external edits before deliberately restoring originals/removing only newly created files. Validate after recovery, then remove the reviewed recovery folder. There is no automatic recovery command yet.

Hard-link support is required. This protects against handled write failures but does not make multiple-file changes atomic across crashes or power loss. The editor refuses projects exceeding its safe snapshot limits (8 MiB per file, 64 MiB total, existing 2048-entry/eight-level traversal bounds) rather than silently skipping content.

### Phase 2C Windows walkthrough

1. Run `cargo run -- new`, create a disposable adventure, then choose **7**.
2. Choose **N**, name a second location, accept its ID, and enter its description.
3. Choose connection option **1**, select the starting room, choose **north**, and accept the **south** reverse connection. Choose **0** when done connecting.
4. Choose **5** to inspect both proposed files, then **6**, **yes** to save. The new room should appear in the refreshed tree.
5. Select each room in turn, use **2** to revise its description, and save with **6**, **yes**.
6. Return with **0**, validate with **2**, then play with **1**. Type `north`, `south`, then `quit` to return to the creator. No TOML editing is needed for this workflow.
7. Reopen the project with `cargo run -- creator "your project folder"`. Try canceling a draft and declining an exit conflict; the existing files should remain unchanged.


## Creator presentation (Phase 2C.1)

The project tree now nests filenames beneath directories using ASCII `+--` branches and four-column indentation. `./` is the selected adventure root; a trailing `/` identifies a directory, including an empty directory. Root files and content categories retain their fixed order; entries within each directory are sorted by name. Only recognized content is displayed; unrelated root folders and temporary file extensions stay hidden. Links/junctions are marked and never followed. The tree refreshes after saves and on returning to the creator.

The display remains limited to 160 entries, with an omission message. Inspection still stops at 2048 entries/eight nested levels and blocks unsafe validation/play. Individual names longer than 180 characters are abbreviated in the tree; filenames on disk are unchanged. Tree branches use the same separators on Windows and Linux. Authored names retain their original characters.

Creator screens share ASCII headings, aligned options, blank lines around sections, and a prompt label followed by `>`. Multiline input retains its compact `...` prompt. Menu numbers, defaults, back/cancel behavior and command meanings have not changed. Connection previews show From, Direction, To and the proposed reverse route. Properties distinguish incoming/outgoing routes and label outgoing routes as bidirectional or one-way. Choosing no reverse never removes a previously existing reverse route.

Save review summarizes the room and routes without repeating long descriptions or advanced properties; option 4 still shows the detailed properties. A ROOM SAVED screen lists affected files and reports PASSED only after persistence and validation succeed. Exact preview bytes remain between the existing content markers.

### Windows presentation review

1. Run `cargo run -- creator "your project folder"` in PowerShell or Command Prompt. Check that rooms, FIG files and artwork appear beneath their actual folders.
2. Choose **7**, select a room and **4**. Check headings, incoming/outgoing routes and separation between sections.
3. In a disposable adventure, create a second room and connect it with a reverse exit. Inspect From/To and reverse prompts. Repeat with a one-way route and decline a conflicting replacement.
4. Preview with **5**, then save with **6**, **yes**. Check the exact preview, file list, validation result and refreshed tree. Cancel a separate draft and confirm that no success screen appears.
5. Return, validate, play, then quit back to the creator. Reopen the project to confirm navigation still works.

If a running executable locks the normal debug build on Windows, leave that session open and use `cargo run --target-dir target/ui-polish -- creator "your project folder"` for a separate build.


## Self-documenting configuration (Phase 2C.2)

Every newly generated adventure includes config/, rooms/, items/, spells/, weapons/, enemies/, shops/ and art/. Empty content directories are intentional: add only the content you need. The nested tree shows both supported FIGs beneath config/.

Open config/character.fig in any text editor. Each key has comments explaining its values and dependencies. `enabled = false` keeps exploration/narrative-only play; changing it to `true` activates the existing character system on reload. Invalid stats remain errors while disabled. Existing adventures without FIGs still work; older character.fig files without enabled keep RPG enabled. Existing RPG samples explicitly opt in.

config/terminology.fig explains the two supported display labels. These do not rename commands. No gameplay.fig, progression.fig, combat.fig or parser.fig settings are supported. TOML holds content, FIG holds configuration, and RORK holds manually saved progress. Before incompatible enablement/stat changes, deliberately manage game.toml save_compatibility_version; configuration editing does not automatically migrate saves or change version numbers.

### Windows review

1. Run `cargo run -- new` and create a disposable adventure. Use Review game files to inspect commented FIG previews before creating it.
2. Check the actual tree for both FIGs and all eight directories. Play and type `stats`: it should say there is no character progression. Quit back to the creator.
3. Open config/character.fig in Notepad; change enabled to true. Validate and play again; `stats` should show the default character.
4. Explicitly `save config_test`, then `load config_test`. Quit/reopen; progress should only return when explicitly loaded.
5. Try enabled = "yes" or an invalid stat, then validate. Confirm the file/setting diagnostic and restore valid values.
6. Run `cargo run -- creator`, choose Open, and press Enter without a path. Read the friendly guidance, then enter the adventure folder.

A separate Cargo target directory (`--target-dir target/config-docs`) can avoid a locked running Windows debug executable. Linux execution has not been verified for this phase.
