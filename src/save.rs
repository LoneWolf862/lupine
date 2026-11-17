//! RORK save-file format v1: explicit conversion between mutable GameState and TOML.
//!
//! This module does not trigger any saving. Terminal commands, save directories,
//! confirmations, and crash-safe disk writes are subsequent v0.6 milestones.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::state::{GameState, ItemLocation};
use crate::world::World;

pub const SAVE_FORMAT_VERSION: u32 = 1;

fn default_save_compatibility_version() -> u32 {
    1
}
fn default_next_enemy_id() -> u64 {
    1
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFile {
    save_format_version: u32,
    game_id: String,
    #[serde(default = "default_save_compatibility_version")]
    save_compatibility_version: u32,
    save_name: String,
    current_room: String,
    flags: Vec<String>,
    item_locations: BTreeMap<String, SavedLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    character: Option<crate::rpg::Character>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    learned_abilities: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    wallet: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    active_enemies: Vec<crate::encounters::EnemyInstance>,
    #[serde(default = "default_next_enemy_id")]
    next_enemy_id: u64,
}

// An explicit kind prevents a room ID named "inventory" from being confused
// with the player's inventory. The optional room is checked below.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedLocation {
    kind: LocationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    room: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LocationKind {
    Inventory,
    Room,
    Consumed,
}

/// Render a complete state snapshot as human-readable TOML for a future `.rork` file.
/// This function **never** writes to disk and never saves automatically.
/// The game ID must be a stable identifier, not a display title.
pub fn encode_save(
    world: &World,
    state: &GameState,
    game_id: &str,
    save_name: &str,
) -> Result<String, String> {
    encode_save_with_compat(world, state, game_id, save_name, 1)
}

/// Explicit story compatibility revision; 1 preserves legacy RORK v1 saves.
pub fn encode_save_with_compat(
    world: &World,
    state: &GameState,
    game_id: &str,
    save_name: &str,
    compatibility_version: u32,
) -> Result<String, String> {
    validate_compat_version(compatibility_version)?;
    validate_metadata(game_id, save_name)?;
    validate_state(world, state)?;

    let mut item_locations = BTreeMap::new();
    for (item, location) in &state.item_locations {
        let location = match location {
            ItemLocation::Inventory => SavedLocation {
                kind: LocationKind::Inventory,
                room: None,
            },
            ItemLocation::Room(room) => SavedLocation {
                kind: LocationKind::Room,
                room: Some(room.clone()),
            },
            ItemLocation::Consumed => SavedLocation {
                kind: LocationKind::Consumed,
                room: None,
            },
        };
        item_locations.insert(item.clone(), location);
    }
    let mut flags = state.flags.iter().cloned().collect::<Vec<_>>();
    flags.sort();
    let file = SaveFile {
        save_format_version: SAVE_FORMAT_VERSION,
        game_id: game_id.into(),
        save_compatibility_version: compatibility_version,
        save_name: save_name.into(),
        current_room: state.current_room.clone(),
        flags,
        item_locations,
        character: state.character.clone(),
        learned_abilities: {
            let mut ids = state.learned_abilities.iter().cloned().collect::<Vec<_>>();
            ids.sort();
            ids
        },
        wallet: state.wallet.clone(),
        active_enemies: state.active_enemies.clone(),
        next_enemy_id: state.next_enemy_id,
    };
    toml::to_string_pretty(&file).map_err(|e| format!("Cannot encode RORK save: {e}"))
}

/// Decode and validate an entire snapshot before returning it. The caller can
/// replace the active GameState *only after* this returns Ok.
pub fn decode_save(
    world: &World,
    expected_game_id: &str,
    input: &str,
) -> Result<GameState, String> {
    decode_save_with_compat(world, expected_game_id, input, 1)
}

/// Decode using a game's declared compatibility revision. No partial state mutation.
pub fn decode_save_with_compat(
    world: &World,
    expected_game_id: &str,
    input: &str,
    compatibility_version: u32,
) -> Result<GameState, String> {
    validate_compat_version(compatibility_version)?;
    let file: SaveFile = toml::from_str(input).map_err(|e| format!("Invalid RORK TOML: {e}"))?;
    if file.save_format_version != SAVE_FORMAT_VERSION {
        return Err(format!(
            "Unsupported RORK save_format_version {} (supported: {})",
            file.save_format_version, SAVE_FORMAT_VERSION
        ));
    }
    validate_metadata(&file.game_id, &file.save_name)?;
    if file.game_id != expected_game_id {
        return Err(format!(
            "Save belongs to game '{}', not '{}'",
            file.game_id, expected_game_id
        ));
    }
    if file.save_compatibility_version != compatibility_version {
        return Err(format!(
            "RORK save compatibility version {} does not match game compatibility version {}. Use a compatible game revision.",
            file.save_compatibility_version, compatibility_version
        ));
    }
    let mut item_locations = HashMap::new();
    for (item, location) in file.item_locations {
        let value = match (location.kind, location.room) {
            (LocationKind::Inventory, None) => ItemLocation::Inventory,
            (LocationKind::Consumed, None) => ItemLocation::Consumed,
            (LocationKind::Consumed, Some(_)) => {
                return Err(format!(
                    "Item '{item}': consumed location must not specify a room"
                ))
            }
            (LocationKind::Room, Some(room)) => ItemLocation::Room(room),
            (LocationKind::Inventory, Some(_)) => {
                return Err(format!(
                    "Item '{item}': inventory location must not specify a room"
                ));
            }
            (LocationKind::Room, None) => {
                return Err(format!("Item '{item}': room location requires a room ID"));
            }
        };
        item_locations.insert(item, value);
    }
    let flags: HashSet<String> = file.flags.iter().cloned().collect();
    if flags.len() != file.flags.len() {
        return Err("RORK save contains duplicate state flags".into());
    }
    let learned_abilities: HashSet<String> = file.learned_abilities.iter().cloned().collect();
    if learned_abilities.len() != file.learned_abilities.len() {
        return Err("RORK save contains duplicate learned abilities".into());
    }
    let state = GameState {
        current_room: file.current_room,
        item_locations,
        flags,
        character: file.character,
        learned_abilities,
        wallet: file.wallet,
        active_enemies: file.active_enemies,
        next_enemy_id: file.next_enemy_id,
    };
    validate_state(world, &state)?;
    Ok(state)
}

fn validate_compat_version(version: u32) -> Result<(), String> {
    if version == 0 {
        Err("save_compatibility_version must be at least 1".into())
    } else {
        Ok(())
    }
}

fn validate_metadata(game_id: &str, save_name: &str) -> Result<(), String> {
    if game_id.is_empty()
        || !game_id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        return Err(
            "Game ID must contain only lowercase ASCII letters, numbers, '-' or '_'".into(),
        );
    }
    if save_name.trim().is_empty() {
        return Err("Save name cannot be empty".into());
    }
    Ok(())
}

fn validate_state(world: &World, state: &GameState) -> Result<(), String> {
    if state.next_enemy_id == 0 {
        return Err("RORK next_enemy_id must be positive".into());
    }
    if state.active_enemies.len() > 12 {
        return Err("RORK active encounter exceeds 12 enemies".into());
    }
    if state.character.is_none() && !state.active_enemies.is_empty() {
        return Err("RORK non-RPG state cannot contain active enemies".into());
    }
    let mut seen = HashSet::new();
    for enemy in &state.active_enemies {
        if enemy.instance_id == 0
            || enemy.instance_id >= state.next_enemy_id
            || enemy.current_hp == 0
            || !seen.insert(enemy.instance_id)
            || enemy.enemy_id.is_empty()
            || !enemy
                .enemy_id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
        {
            return Err(
                "RORK active encounter contains invalid or duplicate enemy instance".into(),
            );
        }
    }
    if let Some(character) = &state.character {
        character.validate()?;
    }
    if state.wallet.keys().any(|id| {
        id.is_empty()
            || !id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
    }) {
        return Err("RORK wallet has an invalid currency ID".into());
    }
    if state.character.is_none() && !state.wallet.is_empty() {
        return Err("Non-RPG saves cannot contain a wallet".into());
    }
    if state.character.is_none() && !state.learned_abilities.is_empty() {
        return Err("Non-RPG saves cannot contain learned abilities".into());
    }
    if let Err(errors) = world.validate() {
        return Err(format!(
            "Cannot use RORK with invalid world: {}",
            errors.join("; ")
        ));
    }
    if world.room(&state.current_room).is_none() {
        return Err(format!(
            "Save references unknown current room '{}'",
            state.current_room
        ));
    }
    for item in &world.items {
        if !state.item_locations.contains_key(&item.id) {
            return Err(format!("Save is missing location for item '{}'", item.id));
        }
    }
    for (item_id, location) in &state.item_locations {
        if world.item(item_id).is_none() {
            return Err(format!("Save references unknown item '{item_id}'"));
        }
        if let ItemLocation::Room(room) = location {
            if world.room(room).is_none() {
                return Err(format!(
                    "Item '{item_id}' is located in unknown room '{room}'"
                ));
            }
        }
    }
    Ok(())
}
