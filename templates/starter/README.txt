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
