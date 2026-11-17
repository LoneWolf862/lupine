//! The default fixture remains available; `play PATH` loads TOML story content.
#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_] => lupine::runner::run_with_game_id(
            fixture_world::fixture_world(),
            "LUPINE v0.7.0-alpha.3 — Engine Test Adventure",
            "lupine_fixture",
        ),
        [_, command, path] if command == "play" => {
            let game = lupine::story_loader::load_game(path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            let content = lupine::rpg_content::RpgContent::load(
                std::path::Path::new(path),
                &game.world,
                game.progression.is_some(),
            )
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            lupine::runner::run_with_game_id_compat_progression_content(
                game.world,
                &game.title,
                &game.id,
                game.save_compatibility_version,
                game.progression,
                content,
            )
        }
        [_, command, path] if command == "validate" => {
            let game = lupine::story_loader::load_game(path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            let content = lupine::rpg_content::RpgContent::load(
                std::path::Path::new(path),
                &game.world,
                game.progression.is_some(),
            )
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            println!("Valid story: {} ({})", game.title, game.id);
            println!(
                "{} weapon definitions, {} abilities",
                content.weapons.len(),
                content.abilities.len()
            );
            println!(
                "{} rooms, {} items, {} interactions, {} exit rules",
                game.world.rooms.len(),
                game.world.items.len(),
                game.world.interactions.len(),
                game.world.exit_rules.len()
            );
            Ok(())
        }
        [_, command] if command == "new" => {
            lupine::creator::new_game_wizard(
                &mut std::io::stdin().lock(),
                &mut std::io::stdout().lock(),
            )
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
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
        [_, command, kind, directory, id] if command == "add" && kind == "art" => {
            let paths = lupine::creator::add_art(std::path::Path::new(directory), id, false)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            for path in paths {
                println!("Created artwork: {}", path.display());
            }
            println!("In a room TOML file add: art_file = \"{id}.txt\"");
            Ok(())
        }
        [_, command, kind, directory, id, flag]
            if command == "add" && kind == "art" && flag == "--dynamic" =>
        {
            let paths = lupine::creator::add_art(std::path::Path::new(directory), id, true)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            for path in paths {
                println!("Created artwork: {}", path.display());
            }
            println!("Add these lines ABOVE [exits] in your room TOML file:");
            println!("  art_file = \"{id}_inactive.txt\"");
            println!("  [[art_variants]]");
            println!("  when = [{{ flag_set = \"{id}_active\" }}]");
            println!("  file = \"{id}_active.txt\"");
            println!("Set flag '{id}_active' with an interaction effect to switch the artwork.");
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
            println!("  lupine new                       (guided wizard)");
            println!("  lupine new <directory> <game-id> <title words...>");
            println!("  lupine add room <directory> <room-id>");
            println!("  lupine add item <directory> <item-id>");
            println!("  lupine add art <directory> <art-id> [--dynamic]");
            println!("  lupine validate <directory>");
            println!("  lupine play <directory>");
            println!("Running `lupine` without arguments launches the test adventure.");
            Ok(())
        }
        _ => {
            eprintln!("Usage: lupine [new [<directory> <game-id> <title...>] | add <room|item|art> <directory> <id> [--dynamic for art] | play <directory> | validate <directory> | help]");
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid arguments",
            ))
        }
    }
}
