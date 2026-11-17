//! Manual-only local RORK save slots; no code path saves without player request.
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::save::{decode_save, encode_save};
use crate::state::GameState;
use crate::world::World;

/// Default per-user location. Caller can supply a different root in tests or a host app.
pub fn default_save_root() -> io::Result<PathBuf> {
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "APPDATA is not set"))?;
        Ok(PathBuf::from(appdata).join("Lupine").join("saves"))
    }
    #[cfg(not(windows))]
    {
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            if !xdg.is_empty() {
                return Ok(PathBuf::from(xdg).join("lupine").join("saves"));
            }
        }
        let home = std::env::var_os("HOME")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
        Ok(PathBuf::from(home).join(".local/share/lupine/saves"))
    }
}

/// Normalize a friendly slot name to a conservative cross-platform file stem.
/// Spaces disappear to follow `forestcamp.howlbound.rork` convention.
pub fn slot_slug(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err("Save name must be 1–100 characters long".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-'))
    {
        return Err("Save name may contain only letters, numbers, spaces, '-' and '_'".into());
    }
    let slug = name
        .chars()
        .filter(|c| *c != ' ')
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if slug.is_empty() || slug.len() > 64 {
        return Err("Normalized save name must be 1–64 characters long".into());
    }
    Ok(slug)
}

fn valid_game_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

fn game_dir(root: &Path, game_id: &str) -> Result<PathBuf, String> {
    if !valid_game_id(game_id) {
        return Err("Invalid game ID for RORK save path".into());
    }
    Ok(root.join(game_id))
}

pub fn slot_path(root: &Path, game_id: &str, name: &str) -> Result<PathBuf, String> {
    Ok(game_dir(root, game_id)?.join(format!("{}.{}.rork", slot_slug(name)?, game_id)))
}

/// Returns whether a slot already exists; caller must confirm before overwrite.
pub fn slot_exists(root: &Path, game_id: &str, name: &str) -> Result<bool, String> {
    Ok(slot_path(root, game_id, name)?.exists())
}

fn stored_slot_name(path: &Path) -> Result<String, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let value: toml::Value = toml::from_str(&content)
        .map_err(|error| format!("{}: invalid RORK TOML: {error}", path.display()))?;
    value
        .get("save_name")
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("{}: missing save_name", path.display()))
}

/// Stage B uses create_new for new saves and overwrite only with explicit consent.
/// Stage C will add atomic replacement and compatibility fingerprinting.
pub fn write_slot(
    root: &Path,
    world: &World,
    state: &GameState,
    game_id: &str,
    name: &str,
    overwrite_confirmed: bool,
) -> Result<PathBuf, String> {
    let path = slot_path(root, game_id, name)?;
    let data = encode_save(world, state, game_id, name)?;
    if path.exists() {
        let previous_name = stored_slot_name(&path)?;
        if !previous_name.eq_ignore_ascii_case(name.trim()) {
            return Err(format!(
                "Save name collides with existing slot '{previous_name}'; choose a different name"
            ));
        }
        if !overwrite_confirmed {
            return Err(format!(
                "Save '{previous_name}' already exists; confirm overwrite"
            ));
        }
    }
    let parent = path.parent().ok_or("No save directory")?;
    fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    if overwrite_confirmed {
        // Intentionally not advertised as crash-safe until Stage C.
        fs::write(&path, data).map_err(|error| format!("{}: {error}", path.display()))?;
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        file.write_all(data.as_bytes())
            .map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(path)
}

pub fn read_slot(
    root: &Path,
    world: &World,
    game_id: &str,
    name: &str,
) -> Result<GameState, String> {
    let path = slot_path(root, game_id, name)?;
    let content =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let previous_name = stored_slot_name(&path)?;
    if !previous_name.eq_ignore_ascii_case(name.trim()) {
        return Err(format!(
            "Save name refers to '{previous_name}', not '{name}'"
        ));
    }
    decode_save(world, game_id, &content)
}

pub fn list_slots(root: &Path, game_id: &str) -> Result<Vec<String>, String> {
    let directory = game_dir(root, game_id)?;
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    let suffix = format!(".{game_id}.rork");
    let entries =
        fs::read_dir(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !filename.ends_with(&suffix) {
            continue;
        }
        match stored_slot_name(&path) {
            Ok(name) => names.push(name),
            Err(_) => names.push(format!("[Unreadable save: {filename}]")),
        }
    }
    names.sort_by_key(|a| a.to_lowercase());
    Ok(names)
}
