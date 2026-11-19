//! Text-only project scaffolding and creator sessions. Player saving remains runner-owned and manual.
use std::fs;
use std::io::{BufRead, Write};
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

mod archive;
mod art_edit;
mod binding_edit;
mod content_edit;
mod currency_edit;
mod enemy_edit;
pub mod export;
mod export_workspace;
mod fig_edit;
mod files;
pub mod inspect;
mod interaction_edit;
mod migration;
mod project_edit;
mod properties;
mod room_advanced;
mod room_edit;
mod room_transaction;
mod rpg_fields;
mod rpg_menu;
mod rules_ui;
pub mod session;
mod setup;
mod shop_edit;
mod spell_edit;
mod tree;
mod ui;
mod weapon_edit;
mod wizard;
use files::{create_file, validate_destination};
use wizard::DraftFile;

const DEFAULT_DESCRIPTION: &str = "You stand at the beginning of your adventure.";

#[derive(Serialize)]
struct StartingRoom<'a> {
    id: &'static str,
    title: &'a str,
    description: &'a str,
    exits: std::collections::BTreeMap<String, String>,
}

fn validate_text(value: &str, max: usize) -> Result<(), String> {
    if value.trim().is_empty()
        || value.chars().count() > max
        || value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(format!(
            "Use 1–{max} characters without terminal control characters"
        ));
    }
    Ok(())
}

fn project_files(
    id: &str,
    title: &str,
    room_title: &str,
    description: &str,
) -> Result<Vec<DraftFile>, String> {
    validate_id(id)?;
    validate_text(title, 120)?;
    validate_text(room_title, 120)?;
    validate_text(description, 65536)?;
    Ok(vec![
        DraftFile::toml("game.toml", &GameManifest {
            format_version: 1, id, title, starting_room: "start", save_compatibility_version: 1,
        }, "id is the stable save identity; title is the display name. starting_room references rooms/start.toml. Version fields describe the content and save compatibility.")?,
        DraftFile::toml("rooms/start.toml", &StartingRoom {
            id: "start", title: room_title, description, exits: Default::default(),
        }, "title and description appear in play. The empty exits table means no connected rooms yet.")?,
        DraftFile::text("interactions.toml", "# Define [[interactions]] and [[exit_rules]] here when needed.\n", "Optional interaction rules; this adventure starts without any."),
        DraftFile::text("config/character.fig", include_str!("../templates/starter/config/character.fig"), "Character settings are explicit but disabled: enabled = false keeps narrative-only play. Comments explain values and save compatibility."),
        DraftFile::text("config/combat.fig", include_str!("../templates/starter/config/combat.fig"), "Combat damage and retaliation defaults; does not enable RPG."),
        DraftFile::text("config/terminology.fig", include_str!("../templates/starter/config/terminology.fig"), "These labels change presentation, not commands. Comments explain each setting."),
        DraftFile::text("README.txt", "Welcome to your Lupine adventure!\n\nEdit game.toml and rooms/start.toml; add TOML files to rooms/ and items/.\nFrom the Lupine source folder: cargo run -- validate <project-folder>\nThen: cargo run -- play <project-folder>\nWith a standalone executable, use its full path followed by play <project-folder>. Quote paths containing spaces.\nEngine settings use TOML syntax. Read the comments in config/character.fig, config/combat.fig and config/terminology.fig for every supported setting. RPG starts disabled; set enabled = true in character.fig to opt in, then validate/reload. Changing RPG availability for existing saves may require increasing game.toml save_compatibility_version. Empty content folders are ready for future content.\nPlayer .rork saves are strictly manual.\n", "Content is editable TOML. Optional FIG files configure the engine; RORK files hold manual player saves."),
    ])
}

/// Create an editable one-room adventure without interactive prompts.
pub fn new_game(destination: &Path, id: &str, title: &str) -> Result<PathBuf, String> {
    let files = project_files(id, title, "Starting Room", DEFAULT_DESCRIPTION)?;
    write_project(destination, &files)
}

fn write_project(destination: &Path, files: &[DraftFile]) -> Result<PathBuf, String> {
    validate_destination(destination)?;
    fs::create_dir(destination).map_err(|e| format!("{}: {e}", destination.display()))?;
    let mut created_files = Vec::new();
    let mut created_dirs = vec![destination.to_path_buf()];
    let result = (|| {
        for folder in [
            "config", "rooms", "items", "spells", "weapons", "enemies", "shops", "art",
        ] {
            let path = destination.join(folder);
            fs::create_dir(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            created_dirs.push(path);
        }
        for file in files {
            let path = destination.join(&file.name);
            create_file(&path, &file.contents)?;
            created_files.push(path);
        }
        crate::story_loader::load_game(destination)?;
        Ok::<_, String>(())
    })();
    if let Err(mut error) = result {
        // Never recursively delete: unexpected or concurrently added content survives.
        for path in created_files.iter().rev() {
            if let Err(e) = fs::remove_file(path) {
                error.push_str(&format!("; cleanup {}: {e}", path.display()));
            }
        }
        for path in created_dirs.iter().rev() {
            if let Err(e) = fs::remove_dir(path) {
                error.push_str(&format!("; cleanup {}: {e}", path.display()));
            }
        }
        return Err(error);
    }
    Ok(destination.to_path_buf())
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
    validate_id(id)?;
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
                    // Publication failed without touching the target. Roll back
                    // only the earlier files created by this call.
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
    if files::reserved_name(id)
        || id.is_empty()
        || id.len() > 100
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    {
        return Err("Use 1-100 lowercase letters, digits, dash or underscore; device names such as con, nul and com1 are reserved".into());
    }
    Ok(())
}

/// Beginner-first project setup. No files are written until review is confirmed.
/// Cancellation and EOF return an explanatory error without creating a project.
pub fn new_game_wizard<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> Result<PathBuf, String> {
    setup::run(input, output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_project_write_rolls_back_its_files() {
        let root =
            std::env::temp_dir().join(format!("lupine-project-rollback-{}", std::process::id()));
        let files = vec![
            DraftFile::text("game.toml", "complete\n", ""),
            DraftFile::text("missing/file.toml", "cannot write\n", ""),
        ];
        assert!(write_project(&root, &files).is_err());
        assert!(!root.exists());
    }
    #[test]
    fn validation_failure_rolls_back_generated_project() {
        let root =
            std::env::temp_dir().join(format!("lupine-project-invalid-{}", std::process::id()));
        assert!(write_project(&root, &[DraftFile::text("game.toml", "invalid TOML", "")]).is_err());
        assert!(!root.exists());
    }
}
