//! The default fixture remains available; `play PATH` loads TOML story content.
#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut input = lupine::terminal::stdin()?;
    match args.as_slice() {
        [_] => lupine::startup::run(
            &mut input,
            &mut std::io::stdout().lock(),
            &lupine::save_files::default_save_root()?,
            &lupine::startup::search_roots()?,
        ),
        [_, command] if command == "fixture" => lupine::runner::run_with_io_and_saves(
            fixture_world::fixture_world(),
            "LUPINE v0.8.4 — Engine Test Adventure",
            "lupine_fixture",
            &lupine::save_files::default_save_root()?,
            &mut input,
            &mut std::io::stdout().lock(),
        ),
        [_, command] if command == "creator" => {
            let saves = lupine::save_files::default_save_root()?;
            lupine::creator::session::start(&mut input, &mut std::io::stdout().lock(), &saves)
        }
        [_, command, path] if command == "creator" => {
            let saves = lupine::save_files::default_save_root()?;
            lupine::creator::session::open(
                std::path::Path::new(path),
                &mut input,
                &mut std::io::stdout().lock(),
                &saves,
            )
        }
        [_, command, path] if command == "play" || command == "validate" => {
            let adventure = lupine::adventure::Adventure::load(std::path::Path::new(path))
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            if command == "validate" {
                adventure.summary(&mut std::io::stdout().lock())
            } else {
                let saves = lupine::save_files::default_save_root()?;
                adventure.play(&saves, &mut input, &mut std::io::stdout().lock())
            }
        }
        [_, command] if command == "new" => {
            let result =
                lupine::creator::new_game_wizard(&mut input, &mut std::io::stdout().lock());
            match result {
                Ok(path) => {
                    let saves = lupine::save_files::default_save_root()?;
                    lupine::creator::session::open(
                        &path,
                        &mut input,
                        &mut std::io::stdout().lock(),
                        &saves,
                    )
                }
                Err(message) if message.starts_with("Setup canceled") => {
                    println!("{message}");
                    Ok(())
                }
                Err(error) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)),
            }
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
            println!("  lupine creator [directory]       (create/open, edit content/config, validate and play)");
            println!("  lupine new                       (guided wizard)");
            println!("  lupine new <directory> <game-id> <title words...>");
            println!("  lupine add room <directory> <room-id>");
            println!("  lupine add item <directory> <item-id>");
            println!("  lupine add art <directory> <art-id> [--dynamic]");
            println!("  lupine validate <directory>");
            println!("  lupine play <directory>");
            println!("  lupine fixture                   (development test adventure)");
            println!("Running `lupine` without arguments opens the startup menu.");
            Ok(())
        }
        _ => {
            eprintln!("Usage: lupine [creator [directory] | new [<directory> <game-id> <title...>] | add <room|item|art> <directory> <id> [--dynamic for art] | play <directory> | validate <directory> | help]");
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid arguments",
            ))
        }
    }
}
