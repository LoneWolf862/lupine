//! Immutable, per-game engine configuration. FIG uses standard TOML syntax.
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::rpg::Progression;

#[derive(Debug, Clone, Default)]
pub struct GameConfig {
    pub character: Option<Progression>,
    pub terminology: Terminology,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Terminology {
    pub display: DisplayTerms,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DisplayTerms {
    pub spellbook: String,
    pub spells: String,
}

impl Default for DisplayTerms {
    fn default() -> Self {
        Self {
            spellbook: "spellbook".into(),
            spells: "abilities".into(),
        }
    }
}

impl GameConfig {
    pub fn load(root: &Path) -> Result<Self, String> {
        for name in ["character.fig", "terminology.fig"] {
            if fs::symlink_metadata(root.join(name)).is_ok() {
                return Err(format!(
                    "{}: misplaced configuration; move it to config/{name}",
                    root.join(name).display()
                ));
            }
        }
        let obsolete = root.join("character.toml");
        match fs::symlink_metadata(&obsolete) {
            Ok(_) => {
                return Err(format!(
                "{}: obsolete configuration; move it to config/character.fig (no legacy loader)",
                obsolete.display()
            ))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(format!("{}: {e}", obsolete.display())),
        }
        let directory = root.join("config");
        match fs::read_dir(&directory) {
            Ok(entries) => {
                let canonical_root = root
                    .canonicalize()
                    .map_err(|e| format!("{}: {e}", root.display()))?;
                let actual = directory
                    .canonicalize()
                    .map_err(|e| format!("{}: {e}", directory.display()))?;
                if !actual.starts_with(&canonical_root) {
                    return Err(format!(
                        "{}: configuration must stay inside the game directory",
                        directory.display()
                    ));
                }
                let mut paths = entries
                    .map(|entry| entry.map(|e| e.path()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("{}: {e}", directory.display()))?;
                paths.sort();
                for path in paths {
                    if path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("fig"))
                        && !matches!(
                            path.file_name().and_then(|s| s.to_str()),
                            Some("character.fig" | "terminology.fig")
                        )
                    {
                        return Err(format!("{}: unknown configuration file; expected character.fig or terminology.fig", path.display()));
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // A dangling config symlink is an error, not an absent configuration.
                if fs::symlink_metadata(&directory).is_ok() {
                    return Err(format!(
                        "{}: configuration directory is unavailable",
                        directory.display()
                    ));
                }
                return Ok(Self::default());
            }
            Err(e) => return Err(format!("{}: {e}", directory.display())),
        }
        let character_path = directory.join("character.fig");
        let source: Option<toml::Table> = read_optional(root, &character_path)?;
        let character = source
            .map(|mut table| -> Result<_, String> {
                // Missing enabled preserves the original FIG contract for authored games.
                let enabled = match table.remove("enabled") {
                    None => true,
                    Some(toml::Value::Boolean(value)) => value,
                    Some(_) => {
                        return Err(format!(
                            "{}: enabled must be a boolean (true or false)",
                            character_path.display()
                        ))
                    }
                };
                let progression: Progression = toml::Value::Table(table)
                    .try_into()
                    .map_err(|e| format!("{}: {e}", character_path.display()))?;
                Ok((enabled, progression))
            })
            .transpose()?;
        if let Some((_, character)) = &character {
            character
                .validate()
                .map_err(|e| format!("{}: character settings (starting_level, starting_hp/mp/ap, hp/mp/ap_per_level): {e}", character_path.display()))?;
        }
        let terminology_path = directory.join("terminology.fig");
        let terminology: Terminology = read_optional(root, &terminology_path)?.unwrap_or_default();
        for (key, value) in [
            ("display.spellbook", &terminology.display.spellbook),
            ("display.spells", &terminology.display.spells),
        ] {
            if value.trim().is_empty()
                || value.chars().count() > 80
                || value.chars().any(char::is_control)
            {
                return Err(format!("{}: {key} must contain 1–80 characters, not be blank, and contain no control characters", terminology_path.display()));
            }
        }
        Ok(Self {
            character: character.and_then(|(enabled, progression)| enabled.then_some(progression)),
            terminology,
        })
    }
}

fn read_optional<T: DeserializeOwned>(root: &Path, path: &Path) -> Result<Option<T>, String> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", path.display())),
        Ok(_) => {}
    }
    let actual = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let root = root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", root.display()))?;
    if !actual.starts_with(&root) {
        return Err(format!(
            "{}: configuration must stay inside the game directory",
            path.display()
        ));
    }
    let source = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&source)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}
