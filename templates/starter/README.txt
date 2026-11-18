LUPINE STARTER ADVENTURE (template)

1. Copy this whole folder to a new location.
2. Edit game.toml (change the id, title, and starting_room).
3. Edit rooms/*.toml to write your rooms; exits refer to other room IDs.
4. Edit items/*.toml to add objects or change starting_room.
5. Edit interactions.toml to define puzzle rules.
6. Run: cargo run -- validate path/to/your-game
7. Run: cargo run -- play path/to/your-game

No Rust editing required. The sample key and door puzzle is replaceable.
This template is an example, not an HOWLBOUND story.

Optional engine configuration uses TOML syntax in config/character.fig (RPG opt-in) and config/terminology.fig (display labels). No config files are required for a narrative-only game. See docs/CONFIG_FIG_SPEC.md in the source repository. Root character.toml is obsolete; move it into config/character.fig. Player saves remain strictly manual .rork files.

Both supported FIG files are included with comments beside every setting.
RPG is disabled by default. Set enabled = true in config/character.fig to opt in,
then validate and reload. Consider game.toml save_compatibility_version before
incompatible changes to games with existing saves. Terminology edits only change labels.
The `new` command creates all eight standard folders, including empty ones. Git does
not store empty directories; when copying this template, create spells/, weapons/,
enemies/, shops/ and art/ as needed. No placeholder content is required.
