//! Beginner and advanced flows share the same answers, drafts and writer.
use super::wizard::Wizard;
use super::{
    files, project_files, validate_destination, validate_id, validate_text, write_project,
    DEFAULT_DESCRIPTION,
};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

const CANCEL: &str = "Setup canceled; no files created";

pub(super) fn suggested_id(title: &str) -> String {
    let mut id = title
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    id.truncate(80); // Leave room for a collision suffix.
    if id.is_empty() || files::reserved_name(&id) {
        id = "my_adventure".into();
    }
    id
}

// IDs are checked among adventures immediately inside the selected parent.
// There is no global registry; do not crawl the user's disk or alter existing games.
fn sibling_ids(parent: &Path) -> Result<HashSet<String>, String> {
    let mut ids = HashSet::new();
    for entry in
        fs::read_dir(parent).map_err(|e| format!("Cannot inspect {}: {e}", parent.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let manifest = entry.path().join("game.toml");
        let text = match fs::read_to_string(&manifest) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(format!(
                    "Cannot check adventure ID in {}: {e}",
                    manifest.display()
                ))
            }
        };
        if let Ok(value) = text.parse::<toml::Value>() {
            if let Some(id) = value.get("id").and_then(toml::Value::as_str) {
                ids.insert(id.to_owned());
            }
        }
    }
    Ok(ids)
}

fn unique_id(base: &str, ids: &HashSet<String>) -> String {
    if !ids.contains(base) {
        return base.to_owned();
    }
    for n in 2u64.. {
        let candidate = format!("{base}_{n}");
        if !ids.contains(&candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn suggested_folder(id: &str) -> String {
    if fs::symlink_metadata(id).is_err() {
        return id.into();
    }
    for n in 2u64.. {
        let candidate = format!("{id}_{n}");
        if fs::symlink_metadata(&candidate).is_err() {
            return candidate;
        }
    }
    unreachable!()
}

fn absolute_destination(value: &str) -> Result<PathBuf, String> {
    validate_destination(Path::new(value))?;
    std::path::absolute(value).map_err(|e| e.to_string())
}

fn text_title(value: &str) -> Result<(), String> {
    validate_text(value, 120)?;
    if value.chars().any(char::is_control) {
        return Err("Please use one line for the name".into());
    }
    Ok(())
}

pub(super) fn run<R: BufRead, W: Write>(input: &mut R, output: &mut W) -> Result<PathBuf, String> {
    let mut w = Wizard { input, output };
    w.say("========================================\n        LUPINE ADVENTURE CREATOR\n========================================\nWelcome! Create your own interactive text adventure.\nYou will name your adventure, describe its first location,\nand choose a folder for its files. No programming experience needed.\nYou can edit the story later. Nothing is created until you approve.\n\nEnter accepts a suggestion. ? gives help. back returns to the previous\nquestion; cancel stops. To write a control word as text, put a backslash\nbefore it, for example \\back. Controls only apply to whole answers.")?;
    let advanced = loop {
        let mode = w.ask("Press Enter to begin (or type advanced for shorter prompts)", "beginner", "Beginner explains each step and handles internal names for you. Advanced lets you edit the game ID and shows files directly.", |v| {
            if matches!(v.to_ascii_lowercase().as_str(), "beginner" | "advanced") { Ok(()) } else { Err("Press Enter, or type beginner or advanced".into()) }
        })?;
        if let Some(mode) = mode {
            break mode.eq_ignore_ascii_case("advanced");
        }
        w.say("You are at the beginning. Press Enter to begin, or cancel to stop.")?;
    };
    let mut title = "My Adventure".to_owned();
    let mut id = suggested_id(&title);
    let mut custom_id = false;
    let mut location = String::new();
    let mut room = "Starting Room".to_owned();
    let mut description = DEFAULT_DESCRIPTION.to_owned();
    let mut step = 0usize;
    loop {
        match step {
            0 => {
                if !advanced { w.say("\nSTEP 1 OF 4 - NAME YOUR ADVENTURE\nThis is the name players see when your game begins.\nExamples: The Forgotten Forest; Journey to the Mountains.\nYou can change this display name later.")?; }
                match w.ask("Adventure name", &title, "Choose a name for your story, up to 120 characters. Enter keeps the suggested name.", text_title)? {
                    Some(value) => {
                        if value != title && !custom_id { id = suggested_id(&value); }
                        title = value;
                        step = if advanced { 1 } else { 2 };
                    }
                    None => w.say("This is the first step. Change the name here, or cancel to stop.")?,
                }
            }
            1 => {
                match w.ask("Game ID", &id, "Use lowercase a-z, digits, - or _. Keep this ID stable once players save progress. IDs must differ from other adventures in the selected parent folder.", validate_id)? {
                    Some(value) => { custom_id = value != suggested_id(&title); id = value; step = 2; }
                    None => step = 0,
                }
            }
            2 => {
                if !advanced {
                    w.say("\nSTEP 2 OF 4 - SAVE YOUR ADVENTURE\nA project folder keeps your game's locations, objects and other files\ntogether. Lupine creates this new folder for you.\nExample: my_adventure creates a folder inside your current folder.\nA full (absolute) path chooses a specific place on your computer.\nYou can move the folder later; keep its files together.")?;
                    w.say(&format!("Current folder: {}", std::env::current_dir().map_err(|e| e.to_string())?.display()))?;
                    #[cfg(windows)]
                    w.say("Full path example: C:\\Users\\YourName\\Documents\\my_adventure")?;
                    #[cfg(not(windows))]
                    w.say("Full path example: /home/yourname/my_adventure")?;
                }
                let default = if location.is_empty() { suggested_folder(&id) } else { location.clone() };
                let answer = w.ask("Save adventure in", &default, "Enter a new folder name or a full local path, without surrounding quotes. The parent must already exist. Existing content is never overwritten.", |v| absolute_destination(v).map(|_| ()))?;
                let Some(value) = answer else { step = if advanced { 1 } else { 0 }; continue; };
                let path = absolute_destination(&value)?;
                let ids = sibling_ids(path.parent().ok_or("Missing parent folder")?)?;
                if advanced && ids.contains(&id) {
                    w.say(&format!("Another adventure in this parent folder uses ID '{id}'. Choose a different game ID to keep player saves separate."))?;
                    location = value;
                    step = 1;
                    continue;
                }
                if !advanced { id = unique_id(&suggested_id(&title), &ids); }
                location = value;
                w.say(&format!("Resolved project folder: {}\nInternal adventure name: {id}\nThis identifies player saves. Keep it stable once players begin saving.\nIDs are checked in this parent folder, not across your entire computer.", path.display()))?;
                step = 3;
            }
            3 => {
                if !advanced { w.say("\nSTEP 3 OF 4 - YOUR STARTING LOCATION\nEvery adventure begins somewhere. In Lupine, a place a player can\nvisit is called a room; it does not have to be inside a building.\nExamples: Forest Clearing; Spaceship Bridge; Mountain Village.\nThis is the first location players see. You can rename it later.")?; }
                match w.ask("Starting location name", &room, "The location's name is shown above its description. You can edit it later in rooms/start.toml.", text_title)? {
                    Some(value) => { room = value; step = 4; }
                    None => step = 2,
                }
            }
            4 => {
                if !advanced { w.say("\nWhat does the player see when they arrive?\nThis description helps players imagine their surroundings.\nExample: Tall pine trees surround a quiet clearing.\nYou can change or expand this text later. A single sentence is fine.")?; }
                match w.description(&description)? {
                    Some(value) => { description = value; step = 5; }
                    None => step = 3,
                }
            }
            _ => {
                let path = absolute_destination(&location)?;
                let drafts = project_files(&id, &title, &room, &description)?;
                w.say(&format!("\nSTEP 4 OF 4 - REVIEW YOUR ADVENTURE\nAdventure: {title}\nStarting location: {room}\nDescription:\n{description}\nProject folder: {}\nInternal adventure name: {id}", path.display()))?;
                if advanced { preview(&mut w, &path, &drafts)?; }
                loop {
                    w.say("\n1. Create my adventure\n2. Review the game files\n3. Change an earlier answer\n4. Cancel")?;
                    match w.ask("Selection", "1", "1 creates the adventure. 2 explains and previews the exact files. 3 revisits your answers without losing them. 4 cancels without creating files.", |v| {
                        if matches!(v, "1" | "2" | "3" | "4") { Ok(()) } else { Err("Choose 1, 2, 3 or 4".into()) }
                    })?.as_deref() {
                        Some("1") => {
                            // Recheck local identity before publishing; never silently change a previewed ID.
                            if sibling_ids(path.parent().ok_or("Missing parent folder")?)?.contains(&id) {
                                w.say("Another adventure now uses this ID. Let's check the folder and ID again.")?;
                                step = if advanced { 1 } else { 2 };
                                break;
                            }
                            let path = write_project(&path, &drafts)?;
                            completion(&mut w, &path, &title, &room)?;
                            return Ok(path);
                        }
                        Some("2") => preview(&mut w, &path, &drafts)?,
                        Some("3") => { step = 0; break; }
                        None => { step = 4; break; }
                        Some("4") => return Err(CANCEL.into()),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

fn preview<R: BufRead, W: Write>(
    w: &mut Wizard<'_, R, W>,
    path: &Path,
    drafts: &[super::DraftFile],
) -> Result<(), String> {
    w.say("\nThese are ordinary text files. You do not need to edit them now.\nTOML stores information as named fields: title = a name in quotes.\nOptional FIG files use the same notation for engine settings; this\nsimple adventure needs none. Explanations below are not file contents.\nThe exact contents between the markers are what Lupine will write.")?;
    for draft in drafts {
        draft.preview(path, w.output)?;
    }
    Ok(())
}

fn shell_quote(value: &str) -> String {
    #[cfg(windows)]
    {
        format!("'{}'", value.replace('\'', "''"))
    }
    #[cfg(not(windows))]
    {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

fn completion<R: BufRead, W: Write>(
    w: &mut Wizard<'_, R, W>,
    path: &Path,
    title: &str,
    room: &str,
) -> Result<(), String> {
    w.say(&format!("\n========================================\n          ADVENTURE CREATED!\n========================================\nYour adventure is ready and has passed validation.\nName: {title}\nStarting location: {room}\nSaved in: {}", path.display()))?;
    #[cfg(windows)]
    w.say("\nCopy the following commands into PowerShell. If you use Command Prompt,\ntype powershell first to open PowerShell in the same folder.")?;
    #[cfg(not(windows))]
    w.say("\nCopy the following commands into your shell (sh/bash/zsh).")?;
    let destination = shell_quote(&path.to_string_lossy());
    w.say(&format!("\nIf you started with cargo run, stay in the Lupine source folder and run:\ncargo run -- play {destination}"))?;
    if let Ok(executable) = std::env::current_exe() {
        // Windows current_exe can return a verbatim path; PowerShell accepts it.
        let executable = shell_quote(&executable.to_string_lossy());
        let prefix = if cfg!(windows) { "& " } else { "" };
        w.say(&format!("\nOr run this exact executable directly (no Cargo or PATH setup needed):\n{prefix}{executable} play {destination}"))?;
    }
    w.say("\nIn the game, try look or help. Type quit to leave. Progress is saved\nonly when you explicitly use save; Lupine never autosaves.\n\nNext, you can add locations, connect them with exits, create objects,\nand add puzzles by editing the ordinary text files.\nStart with rooms/start.toml in your project folder. The creator guide\nin the Lupine source repository is docs/CREATOR_QUICKSTART.md.")?;
    w.say(&format!("After editing, check your game from the Lupine source folder:\ncargo run -- validate {destination}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifiers_are_portable_and_bounded() {
        for title in [
            "The Forgotten Forest",
            "!!!",
            "雪",
            "CON",
            "a/b.$!",
            &"long ".repeat(30),
        ] {
            let id = suggested_id(title);
            validate_id(&id).unwrap();
            assert!(id.len() <= 80);
        }
        assert_eq!(suggested_id("The Forgotten Forest"), "the_forgotten_forest");
        assert_eq!(
            unique_id("story", &HashSet::from(["story".into(), "story_2".into()])),
            "story_3"
        );
    }
    #[test]
    fn command_arguments_quote_shell_metacharacters() {
        let quoted = shell_quote("a b'$`&%!雪");
        #[cfg(windows)]
        assert_eq!(quoted, "'a b''$`&%!雪'");
        #[cfg(not(windows))]
        assert_eq!(quoted, "'a b'\"'\"'$`&%!雪'");
    }
}
