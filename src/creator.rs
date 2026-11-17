//! Text-only project scaffolding. Nothing here runs a game or creates saves.
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Serialize)]
struct GameManifest<'a> {
    format_version: u32,
    id: &'a str,
    title: &'a str,
    starting_room: &'static str,
    save_compatibility_version: u32,
}

/// Create an editable one-room TOML adventure. Refuses to touch an existing project.
/// On failure, removes only a directory created by this call.
pub fn new_game(destination: &Path, id: &str, title: &str) -> Result<PathBuf, String> {
    if id.is_empty()
        || id.len() > 100
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    {
        return Err("Game ID must be 1–100 characters: lowercase a–z, 0–9, '-' or '_'".into());
    }
    if title.trim().is_empty() || title.chars().count() > 120 {
        return Err("Game title must be between 1 and 120 characters".into());
    }
    // create_dir is exclusive, preventing accidental overwrites even if the path
    // appears between checking the command and creating it.
    fs::create_dir(destination)
        .map_err(|e| format!("Cannot create {}: {e}", destination.display()))?;
    let result = (|| -> Result<(), String> {
        let rooms = destination.join("rooms");
        let items = destination.join("items");
        fs::create_dir(&rooms).map_err(|e| e.to_string())?;
        fs::create_dir(&items).map_err(|e| e.to_string())?;
        fs::create_dir(destination.join("art")).map_err(|e| e.to_string())?;
        let manifest = GameManifest {
            format_version: 1,
            id,
            title,
            starting_room: "start",
            save_compatibility_version: 1,
        };
        let manifest = toml::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        create_file(&destination.join("game.toml"), &manifest)?;
        create_file(&rooms.join("start.toml"), "id = \"start\"\ntitle = \"Starting Room\"\ndescription = \"You stand at the beginning of your adventure.\"\n# Optional ASCII art: create art/symbol.txt and set art_file = \"symbol.txt\"\n\n[exits]\n")?;
        create_file(
            &destination.join("interactions.toml"),
            "# Define [[interactions]] and [[exit_rules]] here when needed.\n",
        )?;
        create_file(&destination.join("README.txt"), "Welcome to your Lupine adventure!\n\nEdit game.toml, rooms/start.toml, and add TOML files to rooms/ and items/.\nRun: lupine validate <project-folder>\nThen: lupine play <project-folder>\nFor examples see docs/CREATING_A_GAME.md in the Lupine source repository.\n")?;
        Ok(())
    })();
    if let Err(error) = result {
        // This directory did not exist before create_dir succeeded. Never remove
        // anything outside it. A failed cleanup is reported rather than hidden.
        return match fs::remove_dir_all(destination) {
            Ok(()) => Err(format!("Could not initialize adventure: {error}")),
            Err(cleanup) => Err(format!(
                "Could not initialize adventure: {error}; cleanup failed: {cleanup}"
            )),
        };
    }
    Ok(destination.to_path_buf())
}

fn create_file(path: &Path, content: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(content.as_bytes())
        .map_err(|e: io::Error| format!("{}: {e}", path.display()))
}

/// Add a room definition in an existing adventure; never overwrite or create
/// content outside the validated game directory.
pub fn add_room(directory: &Path, id: &str) -> Result<PathBuf, String> {
    let path = entity_path(directory, "rooms", id)?;
    let content = format!("id = {id:?}\ntitle = {id:?}\ndescription = \"Describe this room.\"\n# Optional: art_file = \"symbol.txt\"  (place file in art/)\n# Dynamic art example:\n# [[art_variants]]\n# when = [{{ flag_set = \"switch_on\" }}]\n# file = \"symbol_active.txt\"\n\n[exits]\n");
    create_file(&path, &content)?;
    Ok(path)
}

/// Add a portable item template to an existing game.
pub fn add_item(directory: &Path, id: &str) -> Result<PathBuf, String> {
    let path = entity_path(directory, "items", id)?;
    let start = crate::story_loader::load_game(directory)?
        .world
        .starting_room;
    let content = format!("id = {id:?}\nname = {id:?}\naliases = []\ndescription = \"Describe this item.\"\nportable = true\nstarting_room = {start:?}\n");
    create_file(&path, &content)?;
    Ok(path)
}

fn entity_path(directory: &Path, folder: &str, id: &str) -> Result<PathBuf, String> {
    if id.is_empty()
        || id.len() > 100
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    {
        return Err(
            "Entity ID must be 1–100 characters, lowercase ASCII a-z, 0-9, '-' or '_'".into(),
        );
    }
    // Ensure the project is valid before adding a file. Creating a template is
    // not a replacement for authoring the exits/interactions it needs.
    crate::story_loader::load_game(directory)?;
    let root = directory.canonicalize().map_err(|e| e.to_string())?;
    let target_dir = directory
        .join(folder)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !target_dir.starts_with(&root) {
        return Err(format!("{folder}/ must be inside the adventure"));
    }
    Ok(target_dir.join(format!("{id}.toml")))
}

/// Generate static or flag-controlled UTF-8 ASCII art in the game's art/ folder.
/// The templates contain ordinary text; no terminal formatting is required.
/// Existing files are never overwritten. If writing a later file fails, any
/// files created by this call are removed, preserving existing content.
pub fn add_art(directory: &Path, id: &str, dynamic: bool) -> Result<Vec<PathBuf>, String> {
    validate_id(id)?;
    crate::story_loader::load_game(directory)?;
    let root = directory
        .canonicalize()
        .map_err(|e| format!("{}: {e}", directory.display()))?;
    let art_dir = directory.join("art");
    let created_dir = if !art_dir.exists() {
        fs::create_dir(&art_dir).map_err(|e| format!("{}: {e}", art_dir.display()))?;
        true
    } else {
        false
    };
    let result = (|| -> Result<Vec<PathBuf>, String> {
        let actual = art_dir.canonicalize().map_err(|e| e.to_string())?;
        if !actual.starts_with(&root) {
            return Err("art/ must be inside the adventure directory".into());
        }
        let names = if dynamic {
            vec![format!("{id}_inactive.txt"), format!("{id}_active.txt")]
        } else {
            vec![format!("{id}.txt")]
        };
        let paths = names
            .iter()
            .map(|name| art_dir.join(name))
            .collect::<Vec<_>>();
        // Preflight is advisory only. create_new protects against races.
        for path in &paths {
            if path.exists() {
                return Err(format!(
                    "{} already exists; nothing was overwritten",
                    path.display()
                ));
            }
        }
        let mut created = Vec::new();
        for (index, path) in paths.iter().enumerate() {
            let art = if dynamic {
                if index == 0 {
                    "  +----------+\n  | INACTIVE |\n  +----------+\n"
                } else {
                    "  +----------+\n  |  ACTIVE  |\n  +----------+\n"
                }
            } else {
                "  /\\\n /  \\\n/____\\\n"
            };
            match create_file(path, art) {
                Ok(()) => created.push(path.clone()),
                Err(error) => {
                    // create_file might have created a partial target; remove only
                    // this call's files, not unrelated or existing user files.
                    for file in &created {
                        let _ = fs::remove_file(file);
                    }
                    return Err(error);
                }
            }
        }
        Ok(paths)
    })();
    if created_dir && result.is_err() {
        let _ = fs::remove_dir(&art_dir);
    }
    result
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 100
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    {
        return Err("ID must be 1–100 characters, lowercase ASCII a-z, 0-9, '-' or '_'".into());
    }
    Ok(())
}

/// Interactive wizard for the existing `new_game` generator. No files are
/// created until every prompt has been answered and validated.
pub fn new_game_wizard<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> Result<PathBuf, String> {
    fn ask<R: BufRead, W: Write>(
        input: &mut R,
        output: &mut W,
        label: &str,
    ) -> Result<String, String> {
        write!(output, "{label}").map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
        let mut answer = String::new();
        if input.read_line(&mut answer).map_err(|e| e.to_string())? == 0 {
            return Err("Setup canceled (end of input); no files created".into());
        }
        Ok(answer.trim().to_owned())
    }
    writeln!(output, "Lupine adventure setup — no Rust required.").map_err(|e| e.to_string())?;
    let dest = ask(input, output, "New project folder: ")?;
    if dest.is_empty() {
        return Err("Project folder cannot be empty".into());
    }
    let id = ask(
        input,
        output,
        "Game ID (lowercase letters, numbers, - or _): ",
    )?;
    validate_id(&id)?;
    let title = ask(input, output, "Adventure title: ")?;
    if title.trim().is_empty() {
        return Err("Adventure title cannot be empty".into());
    }
    let path = PathBuf::from(dest);
    new_game(&path, &id, &title)?;
    writeln!(output, "Created: {}", path.display()).map_err(|e| e.to_string())?;
    writeln!(output, "Next: lupine validate {}", path.display()).map_err(|e| e.to_string())?;
    writeln!(output, "Then: lupine play {}", path.display()).map_err(|e| e.to_string())?;
    Ok(path)
}
