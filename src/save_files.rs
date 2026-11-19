//! Manual-only local RORK save slots; no code path saves without player request.
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::save::{decode_save_with_compat, encode_save_with_compat};
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

/// Returns whether a slot already exists; callers supply explicit overwrite authorization.
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

/// Existing API: version 1 is the default for Rust-authored games.
pub fn write_slot(
    root: &Path,
    world: &World,
    state: &GameState,
    game_id: &str,
    name: &str,
    overwrite_confirmed: bool,
) -> Result<PathBuf, String> {
    write_slot_with_compat(root, world, state, game_id, name, overwrite_confirmed, 1)
}

pub fn write_slot_with_compat(
    root: &Path,
    world: &World,
    state: &GameState,
    game_id: &str,
    name: &str,
    overwrite_confirmed: bool,
    compatibility_version: u32,
) -> Result<PathBuf, String> {
    write_slot_impl(
        root,
        world,
        state,
        game_id,
        name,
        overwrite_confirmed,
        compatibility_version,
        || Ok(()),
    )
}

// A hook at the commit boundary makes the failure path reproducibly testable.
// The public API never uses an injected hook.
// The extra argument is intentional for testing atomic write failures.
#[allow(clippy::too_many_arguments)]
fn write_slot_impl<F: FnOnce() -> Result<(), String>>(
    root: &Path,
    world: &World,
    state: &GameState,
    game_id: &str,
    name: &str,
    overwrite_confirmed: bool,
    compatibility_version: u32,
    before_commit: F,
) -> Result<PathBuf, String> {
    let path = slot_path(root, game_id, name)?;
    let data = encode_save_with_compat(world, state, game_id, name, compatibility_version)?;
    let existing = fs::symlink_metadata(&path).ok();
    if let Some(meta) = &existing {
        if !meta.file_type().is_file() || meta.file_type().is_symlink() {
            return Err(format!(
                "{}: save path is not a regular file",
                path.display()
            ));
        }
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

    // Create a unique sibling in the same filesystem, and never touch the destination
    // until every byte has been written and synced. Drop cleans up failed temp writes.
    let (temporary, mut file) = create_temp(parent, &path)?;
    let pending = PendingTemp(temporary);
    // Close the handle before the cleanup guard on every error path; Windows
    // will not reliably allow deletion of an open temporary save.
    let staged = file
        .write_all(data.as_bytes())
        .and_then(|_| file.sync_all());
    drop(file);
    staged.map_err(|error| format!("Save staging failed: {error}"))?;
    before_commit()?;

    if existing.is_some() {
        // rename replaces the destination on supported Windows/Unix filesystems.
        // NEVER remove the old file first: failure leaves the old slot untouched.
        fs::rename(&pending.0, &path).map_err(|error| {
            format!(
                "Cannot replace {}: {error}; original save was kept",
                path.display()
            )
        })?;
    } else {
        // Creating a new name with hard_link is create-only: a slot added by a
        // second writer cannot be silently overwritten between checks.
        fs::hard_link(&pending.0, &path)
            .map_err(|error| format!("Cannot publish new save {}: {error}", path.display()))?;
    }
    // Dropping pending removes the temp after a hard link, or harmlessly after rename.
    Ok(path)
}

struct PendingTemp(PathBuf);
impl Drop for PendingTemp {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
fn create_temp(dir: &Path, destination: &Path) -> Result<(PathBuf, fs::File), String> {
    let base = destination
        .file_name()
        .ok_or("Invalid save filename")?
        .to_string_lossy();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock error: {error}"))?
        .as_nanos();
    for _ in 0..16 {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!(
            ".{base}.{}.{}.{}.tmp",
            std::process::id(),
            nanos,
            counter
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Cannot create temporary save: {error}")),
        }
    }
    Err("Could not create a unique temporary save file".into())
}

pub fn read_slot(
    root: &Path,
    world: &World,
    game_id: &str,
    name: &str,
) -> Result<GameState, String> {
    read_slot_with_compat(root, world, game_id, name, 1)
}

pub fn read_slot_with_compat(
    root: &Path,
    world: &World,
    game_id: &str,
    name: &str,
    compatibility_version: u32,
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
    decode_save_with_compat(world, game_id, &content, compatibility_version)
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

#[cfg(test)]
mod atomic_tests {
    use super::*;
    use crate::story_loader::load_game;
    use std::time::{SystemTime, UNIX_EPOCH};
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn setup() -> (PathBuf, World, GameState) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "lupine_atomic_{}_{}_{}",
            std::process::id(),
            nanos,
            TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let game = load_game(Path::new(env!("CARGO_MANIFEST_DIR")).join("story_example")).unwrap();
        let state = GameState::new(&game.world);
        (root, game.world, state)
    }

    #[test]
    fn failed_overwrite_keeps_old_file_byte_for_byte() {
        let (root, world, mut state) = setup();
        let path = write_slot(
            &root,
            &world,
            &state,
            "example_adventure",
            "Forest Camp",
            false,
        )
        .unwrap();
        let old = fs::read(&path).unwrap();
        state.current_room = "workshop".into();
        let error = write_slot_impl(
            &root,
            &world,
            &state,
            "example_adventure",
            "Forest Camp",
            true,
            1,
            || Err("simulated failure before commit".into()),
        )
        .unwrap_err();
        assert!(error.contains("simulated failure"));
        assert_eq!(fs::read(&path).unwrap(), old);
        let contents = fs::read_dir(path.parent().unwrap()).unwrap().count();
        assert_eq!(contents, 1, "failed writes should remove temporary files");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_overwrite_replaces_existing_slot_and_cleans_temp() {
        let (root, world, mut state) = setup();
        let path = write_slot(&root, &world, &state, "example_adventure", "Camp", false).unwrap();
        state.current_room = "workshop".into();
        write_slot(&root, &world, &state, "example_adventure", "Camp", true).unwrap();
        let loaded = read_slot(&root, &world, "example_adventure", "Camp").unwrap();
        assert_eq!(loaded.current_room, "workshop");
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failure_creating_new_slot_leaves_no_partial_save() {
        let (root, world, state) = setup();
        let err = write_slot_impl(
            &root,
            &world,
            &state,
            "example_adventure",
            "Camp",
            false,
            1,
            || Err("simulated failure".into()),
        )
        .unwrap_err();
        assert!(err.contains("simulated failure"));
        let path = slot_path(&root, "example_adventure", "Camp").unwrap();
        assert!(!path.exists());
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}
