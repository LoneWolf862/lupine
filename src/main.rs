//! The default fixture remains available; `play PATH` loads TOML story content.
#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_] => lupine::runner::run_with_game_id(
            fixture_world::fixture_world(),
            "LUPINE v0.7.0-alpha.2 — Engine Test Adventure",
            "lupine_fixture",
        ),
        [_, command, path] if command == "play" => {
            let game = lupine::story_loader::load_game(path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            lupine::runner::run_with_game_id_and_compat(
                game.world,
                &game.title,
                &game.id,
                game.save_compatibility_version,
            )
        }
        [_, command, path] if command == "validate" => {
            let game = lupine::story_loader::load_game(path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            println!("Valid story: {} ({})", game.title, game.id);
            println!(
                "{} rooms, {} items, {} interactions, {} exit rules",
                game.world.rooms.len(),
                game.world.items.len(),
                game.world.interactions.len(),
                game.world.exit_rules.len()
            );
            Ok(())
        }
        [_, command, destination, id, title @ ..] if command == "new" && !title.is_empty() => {
            let title = title.join(" ");
            lupine::creator::new_game(std::path::Path::new(destination), id, &title)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
            println!("Created adventure: {destination}");
            println!("Edit {destination}/rooms/start.toml, then run:");
            println!("  lupine validate {destination}");
            println!("  lupine play {destination}");
            Ok(())
        }
        [_, command, kind, directory, id] if command == "add" && kind == "room" => {
            lupine::creator::add_room(std::path::Path::new(directory), id)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            println!("Created room: {directory}/rooms/{id}.toml");
            Ok(())
        }
        [_, command, kind, directory, id] if command == "add" && kind == "item" => {
            lupine::creator::add_item(std::path::Path::new(directory), id)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            println!("Created item: {directory}/items/{id}.toml");
            Ok(())
        }
        [_, command] if command == "help" => {
            println!("Lupine authoring commands:");
            println!("  lupine new <directory> <game-id> <title words...>");
            println!("  lupine add room <directory> <room-id>");
            println!("  lupine add item <directory> <item-id>");
            println!("  lupine validate <directory>");
            println!("  lupine play <directory>");
            println!("Running `lupine` without arguments launches the test adventure.");
            Ok(())
        }
        _ => {
            eprintln!("Usage: lupine [new <directory> <game-id> <title...> | add <room|item> <directory> <id> | play <directory> | validate <directory> | help]");
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid arguments",
            ))
        }
    }
}
