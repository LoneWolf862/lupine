//! Load an offline, text-authored adventure into the same World used by Rust games.
//! File-format types are deliberately distinct from the engine model.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::commands::Direction;
use crate::world::{
    Condition, DescriptionVariant, Effect, ExitRule, Interaction, Item, Room, World,
};

#[derive(Debug)]
pub struct LoadedGame {
    pub id: String,
    pub title: String,
    pub save_compatibility_version: u32,
    pub world: World,
    pub config: crate::config::GameConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format_version: u32,
    id: String,
    title: String,
    starting_room: String,
    #[serde(default = "default_save_compatibility_version")]
    save_compatibility_version: u32,
    #[serde(default)]
    #[serde(rename = "currency")]
    _currency: Option<toml::Value>,
    #[serde(default, rename = "currencies")]
    _currencies: Vec<toml::Value>,
}

fn default_save_compatibility_version() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomData {
    id: String,
    title: String,
    description: String,
    #[serde(default)]
    descriptions: Vec<DescriptionData>,
    #[serde(default)]
    exits: BTreeMap<String, String>,
    /// Short inline artwork, optional alternative to art_file.
    art: Option<String>,
    /// Relative path under the game's art/ directory.
    art_file: Option<String>,
    #[serde(default)]
    art_variants: Vec<ArtVariantData>,
    #[serde(default)]
    encounters: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtVariantData {
    when: Vec<toml::Value>,
    file: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemData {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    description: String,
    #[serde(default)]
    descriptions: Vec<DescriptionData>,
    portable: bool,
    starting_room: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DescriptionData {
    when: Vec<toml::Value>,
    text: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesData {
    #[serde(default)]
    interactions: Vec<InteractionData>,
    #[serde(default)]
    exit_rules: Vec<ExitRuleData>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InteractionData {
    #[serde(default)]
    outcomes: Vec<toml::Value>,
    item: String,
    target: String,
    #[serde(default)]
    requires: Vec<toml::Value>,
    #[serde(default)]
    effects: Vec<toml::Value>,
    failure_text: String,
    success_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExitRuleData {
    from: String,
    direction: String,
    #[serde(default)]
    requires: Vec<toml::Value>,
    blocked_text: String,
}

fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    toml::from_str(&source).map_err(|error| format!("{}: {error}", path.display()))
}

fn story_files(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for entry in
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("toml")
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn direction(name: &str) -> Result<Direction, String> {
    match name {
        "north" => Ok(Direction::North),
        "south" => Ok(Direction::South),
        "east" => Ok(Direction::East),
        "west" => Ok(Direction::West),
        "up" => Ok(Direction::Up),
        "down" => Ok(Direction::Down),
        _ => Err(format!(
            "unknown direction '{name}'; use north/south/east/west/up/down"
        )),
    }
}

fn condition(value: &toml::Value) -> Result<Condition, String> {
    crate::rules::decode(value)
}
fn effect(value: &toml::Value) -> Result<Effect, String> {
    crate::rules::decode(value)
}

fn conditions(data: &[toml::Value], origin: &str) -> Result<Vec<Condition>, String> {
    data.iter()
        .enumerate()
        .map(|(index, value)| {
            condition(value).map_err(|error| format!("{origin} condition #{}: {error}", index + 1))
        })
        .collect()
}

fn descriptions(
    data: Vec<DescriptionData>,
    origin: &str,
) -> Result<Vec<DescriptionVariant>, String> {
    data.into_iter()
        .enumerate()
        .map(|(index, data)| {
            if data.when.is_empty() {
                return Err(format!(
                    "{origin} description #{}: 'when' must not be empty",
                    index + 1
                ));
            }
            Ok(DescriptionVariant {
                when: conditions(&data.when, origin)?,
                text: data.text,
            })
        })
        .collect()
}

/// Resolve artwork only from <story>/art, including when symlinks exist.
/// Never interpret ANSI control sequences embedded in story files.
const MAX_ART_BYTES: u64 = 64 * 1024;

pub(crate) fn validate_art(text: &str, origin: &str) -> Result<(), String> {
    if text.len() as u64 > MAX_ART_BYTES {
        return Err(format!("{origin}: artwork exceeds 64 KiB"));
    }
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(format!(
            "{origin}: artwork contains terminal control characters"
        ));
    }
    Ok(())
}

fn read_art(root: &Path, file: &str, origin: &str) -> Result<String, String> {
    let relative = Path::new(file);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "{origin}: art file path must be relative with no '..' segments"
        ));
    }
    let story_root = root
        .canonicalize()
        .map_err(|e| format!("{origin}: story directory: {e}"))?;
    let art_directory = root
        .join("art")
        .canonicalize()
        .map_err(|e| format!("{origin}: art/ directory: {e}"))?;
    if !art_directory.starts_with(&story_root) {
        return Err(format!(
            "{origin}: art/ must be inside the adventure directory"
        ));
    }
    let path = root.join("art").join(relative);
    let resolved = path
        .canonicalize()
        .map_err(|e| format!("{origin}: {}: {e}", path.display()))?;
    if !resolved.starts_with(&art_directory) || !resolved.is_file() {
        return Err(format!(
            "{origin}: art file must be a regular file inside art/"
        ));
    }
    let length = fs::metadata(&resolved)
        .map_err(|e| format!("{origin}: {e}"))?
        .len();
    if length > MAX_ART_BYTES {
        return Err(format!("{origin}: artwork exceeds 64 KiB"));
    }
    let text =
        fs::read_to_string(&resolved).map_err(|e| format!("{origin}: {}: {e}", path.display()))?;
    validate_art(&text, origin)?;
    Ok(text)
}

fn resolve_art(
    root: &Path,
    inline: Option<String>,
    file: Option<String>,
    origin: &str,
) -> Result<Option<String>, String> {
    match (inline, file) {
        (Some(_), Some(_)) => Err(format!("{origin}: choose either art or art_file, not both")),
        (Some(text), None) => {
            validate_art(&text, origin)?;
            Ok(Some(text))
        }
        (None, Some(file)) => read_art(root, &file, origin).map(Some),
        (None, None) => Ok(None),
    }
}

fn resolve_art_variants(
    root: &Path,
    variants: Vec<ArtVariantData>,
    origin: &str,
) -> Result<Vec<DescriptionVariant>, String> {
    variants
        .into_iter()
        .enumerate()
        .map(|(i, variant)| {
            let context = format!("{origin} art_variants #{}", i + 1);
            if variant.when.is_empty() {
                return Err(format!("{context}: 'when' must not be empty"));
            }
            let art = resolve_art(root, variant.text, variant.file, &context)?
                .ok_or_else(|| format!("{context}: specify 'file' or 'text'"))?;
            Ok(DescriptionVariant {
                when: conditions(&variant.when, &context)?,
                text: art,
            })
        })
        .collect()
}

/// Load a story directory. No story file is executed as code.
/// The returned World is validated before it can be used for gameplay.
pub fn load_game(directory: impl AsRef<Path>) -> Result<LoadedGame, String> {
    let directory = directory.as_ref();
    let manifest: Manifest = read_toml(&directory.join("game.toml"))?;
    let config = crate::config::GameConfig::load(directory)?;
    if manifest.format_version != 1 {
        return Err(format!(
            "{}: unsupported story format_version {} (expected 1)",
            directory.join("game.toml").display(),
            manifest.format_version
        ));
    }
    if manifest.id.is_empty()
        || !manifest.id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        return Err(format!(
            "{}: game id must contain only lowercase ASCII letters, numbers, '_' or '-'",
            directory.join("game.toml").display()
        ));
    }
    if manifest.save_compatibility_version == 0 {
        return Err(format!(
            "{}: save_compatibility_version must be at least 1",
            directory.join("game.toml").display()
        ));
    }
    if manifest.title.trim().is_empty() {
        return Err(format!(
            "{}: game title must not be empty",
            directory.join("game.toml").display()
        ));
    }
    let mut rooms = Vec::new();
    let mut room_paths = Vec::new();
    for path in story_files(&directory.join("rooms"))? {
        let data: RoomData = read_toml(&path)?;
        let origin = path.display().to_string();
        let mut exits = Vec::new();
        for (name, destination) in data.exits {
            exits.push((
                direction(&name).map_err(|error| format!("{origin}: {error}"))?,
                destination,
            ));
        }
        let _ = &data.encounters; // Parsed and validated by EncounterContent.
        let art = resolve_art(directory, data.art, data.art_file, &origin)?;
        let art_variants = resolve_art_variants(directory, data.art_variants, &origin)?;
        room_paths.push(path.clone());
        rooms.push(Room {
            id: data.id,
            title: data.title,
            description: data.description,
            descriptions: descriptions(data.descriptions, &origin)?,
            art,
            art_variants,
            exits,
        });
    }
    let mut items = Vec::new();
    let mut item_paths = Vec::new();
    for path in story_files(&directory.join("items"))? {
        let data: ItemData = read_toml(&path)?;
        let origin = path.display().to_string();
        item_paths.push(path.clone());
        items.push(Item {
            id: data.id,
            name: data.name,
            aliases: data.aliases,
            description: data.description,
            descriptions: descriptions(data.descriptions, &origin)?,
            portable: data.portable,
            starting_room: data.starting_room,
        });
    }
    let rules_path = directory.join("interactions.toml");
    let rules: RulesData = if rules_path.exists() {
        read_toml(&rules_path)?
    } else {
        RulesData::default()
    };
    let mut interactions = Vec::new();
    for (index, data) in rules.interactions.into_iter().enumerate() {
        let origin = format!("{} interaction #{}", rules_path.display(), index + 1);
        let effects = data
            .effects
            .iter()
            .enumerate()
            .map(|(i, value)| {
                effect(value).map_err(|error| format!("{origin} effect #{}: {error}", i + 1))
            })
            .collect::<Result<Vec<_>, _>>()?;
        interactions.push(Interaction {
            outcomes: data
                .outcomes
                .iter()
                .map(crate::rules::decode)
                .collect::<Result<_, _>>()
                .map_err(|e| format!("{origin}: {e}"))?,
            item: data.item,
            target: data.target,
            requires: conditions(&data.requires, &origin)?,
            effects,
            failure_text: data.failure_text,
            success_text: data.success_text,
        });
    }
    let mut exit_rules = Vec::new();
    for (index, data) in rules.exit_rules.into_iter().enumerate() {
        let origin = format!("{} exit rule #{}", rules_path.display(), index + 1);
        exit_rules.push(ExitRule {
            from: data.from,
            direction: direction(&data.direction).map_err(|error| format!("{origin}: {error}"))?,
            requires: conditions(&data.requires, &origin)?,
            blocked_text: data.blocked_text,
        });
    }
    let world = World {
        rules: crate::rules::StoryRules::load(directory)?,
        rooms,
        items,
        interactions,
        exit_rules,
        starting_room: manifest.starting_room,
    };
    let errors = validate_story_references(&world, &room_paths, &item_paths, directory);
    if !errors.is_empty() {
        return Err(format!("Story validation failed:\n{}", errors.join("\n")));
    }
    world.validate().map_err(|errors| {
        format!(
            "{}: story validation failed:\n{}",
            directory.display(),
            errors.join("\n")
        )
    })?;
    Ok(LoadedGame {
        id: manifest.id,
        title: manifest.title,
        save_compatibility_version: manifest.save_compatibility_version,
        world,
        config,
    })
}

/// Add file-based guidance for common authoring mistakes before general engine validation.
/// The core engine remains the source of truth for rules and state invariants.
fn validate_story_references(
    world: &World,
    room_paths: &[PathBuf],
    item_paths: &[PathBuf],
    root: &Path,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut room_ids: HashMap<&str, &Path> = HashMap::new();
    let mut item_ids: HashMap<&str, &Path> = HashMap::new();
    let mut seen_rooms = HashSet::new();
    let mut seen_items = HashSet::new();
    for (room, path) in world.rooms.iter().zip(room_paths) {
        if room.id.trim().is_empty() {
            errors.push(format!(
                "{}: room 'id' cannot be empty. Add an id such as 'forest'.",
                path.display()
            ));
        } else if !seen_rooms.insert(room.id.as_str()) {
            errors.push(format!(
                "{}: duplicate room id '{}'. Give each room a unique id.",
                path.display(),
                room.id
            ));
        }
        room_ids.entry(room.id.as_str()).or_insert(path);
    }
    for (item, path) in world.items.iter().zip(item_paths) {
        if item.id.trim().is_empty() {
            errors.push(format!(
                "{}: item 'id' cannot be empty. Add an id such as 'lantern'.",
                path.display()
            ));
        } else if !seen_items.insert(item.id.as_str()) {
            errors.push(format!(
                "{}: duplicate item id '{}'. Give each item a unique id.",
                path.display(),
                item.id
            ));
        }
        item_ids.entry(item.id.as_str()).or_insert(path);
    }
    if !room_ids.contains_key(world.starting_room.as_str()) {
        errors.push(format!(
            "{}: starting_room '{}' does not exist. Set it to an id defined in rooms/*.toml.",
            root.join("game.toml").display(),
            world.starting_room
        ));
    }
    for (room, path) in world.rooms.iter().zip(room_paths) {
        for (direction, destination) in &room.exits {
            if !room_ids.contains_key(destination.as_str()) {
                errors.push(format!("{}: room '{}' has a {} exit to unknown room '{}'. Create that room in rooms/ or correct the exit destination.", path.display(), room.id, direction.label(), destination));
            }
        }
    }
    for (item, path) in world.items.iter().zip(item_paths) {
        if !room_ids.contains_key(item.starting_room.as_str()) {
            errors.push(format!("{}: item '{}' starts in unknown room '{}'. Set starting_room to an existing room id.", path.display(), item.id, item.starting_room));
        }
    }
    let rules_path = root.join("interactions.toml");
    for (index, rule) in world.interactions.iter().enumerate() {
        if !item_ids.contains_key(rule.item.as_str())
            || !item_ids.contains_key(rule.target.as_str())
        {
            errors.push(format!("{}: interaction #{} references missing item '{}' or target '{}'. Define both in items/*.toml.", rules_path.display(), index + 1, rule.item, rule.target));
        }
    }
    for (index, rule) in world.exit_rules.iter().enumerate() {
        if !room_ids.contains_key(rule.from.as_str()) {
            errors.push(format!(
                "{}: exit rule #{} references unknown room '{}'.",
                rules_path.display(),
                index + 1,
                rule.from
            ));
        }
    }
    errors
}
