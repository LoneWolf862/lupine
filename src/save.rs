//! RORK save-file format v6: explicit conversion between mutable GameState and TOML.
//!
//! This module does not trigger any saving. Terminal commands, save directories,
//! confirmations, and crash-safe disk writes are implemented by the terminal layer.

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::state::GameState;
use crate::world::World;

pub const SAVE_FORMAT_VERSION: u32 = 6;

fn default_save_compatibility_version() -> u32 {
    1
}
fn default_next_enemy_id() -> u64 {
    1
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFile {
    systems: crate::systems::Progress,
    dialogue: crate::dialogue::Progress,
    events: crate::events::Progress,
    story: crate::rules::StoryState,
    save_format_version: u32,
    game_id: String,
    #[serde(default = "default_save_compatibility_version")]
    save_compatibility_version: u32,
    save_name: String,
    current_room: String,
    flags: Vec<String>,
    objects: crate::objects::ObjectState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    character: Option<crate::rpg::Character>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    learned_abilities: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    wallet: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    shop_purchases: BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    active_enemies: Vec<crate::encounters::EnemyInstance>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    room_encounters: BTreeMap<String, Vec<crate::encounters::EnemyInstance>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    completed_unique_encounters: Vec<String>,
    #[serde(default = "default_next_enemy_id")]
    next_enemy_id: u64,
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

/// Explicit story compatibility revision, independent of the save serialization version.
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

    let mut flags = state.flags.iter().cloned().collect::<Vec<_>>();
    flags.sort();
    let file = SaveFile {
        systems: state.systems.clone(),
        dialogue: state.dialogue.clone(),
        events: state.events.clone(),
        story: state.story.clone(),
        save_format_version: SAVE_FORMAT_VERSION,
        game_id: game_id.into(),
        save_compatibility_version: compatibility_version,
        save_name: save_name.into(),
        current_room: state.current_room.clone(),
        flags,
        objects: state.objects.clone(),
        character: state.character.clone(),
        learned_abilities: {
            let mut ids = state.learned_abilities.iter().cloned().collect::<Vec<_>>();
            ids.sort();
            ids
        },
        wallet: state.wallet.clone(),
        shop_purchases: state.shop_purchases.clone(),
        active_enemies: state.active_enemies.clone(),
        next_enemy_id: state.next_enemy_id,
        room_encounters: state.room_encounters.clone(),
        completed_unique_encounters: {
            let mut rooms = state
                .completed_unique_encounters
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            rooms.sort();
            rooms
        },
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
    let document: toml::Value =
        toml::from_str(input).map_err(|e| format!("Invalid RORK TOML: {e}"))?;
    let version = document
        .get("save_format_version")
        .and_then(toml::Value::as_integer);
    if version != Some(i64::from(SAVE_FORMAT_VERSION)) {
        return Err(format!("Unsupported RORK save_format_version {version:?} (supported: {SAVE_FORMAT_VERSION}). Use the matching engine/game build; existing saves are never rewritten."));
    }
    let file: SaveFile = document
        .try_into()
        .map_err(|e| format!("Invalid RORK TOML: {e}"))?;
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
    let flags: HashSet<String> = file.flags.iter().cloned().collect();
    if flags.len() != file.flags.len() {
        return Err("RORK save contains duplicate state flags".into());
    }
    let learned_abilities: HashSet<String> = file.learned_abilities.iter().cloned().collect();
    if learned_abilities.len() != file.learned_abilities.len() {
        return Err("RORK save contains duplicate learned abilities".into());
    }
    let completed_unique_encounters: HashSet<String> =
        file.completed_unique_encounters.iter().cloned().collect();
    if completed_unique_encounters.len() != file.completed_unique_encounters.len() {
        return Err("RORK save has duplicate completed unique encounters".into());
    }
    let state = GameState {
        systems: file.systems,
        dialogue: file.dialogue,
        events: file.events,
        story: file.story,
        current_room: file.current_room,
        objects: file.objects,
        flags,
        character: file.character,
        learned_abilities,
        wallet: file.wallet,
        shop_purchases: file.shop_purchases,
        active_enemies: file.active_enemies,
        room_encounters: file.room_encounters,
        completed_unique_encounters,
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
    world.rules.validate_state(&state.story, world)?;
    for id in &state.flags {
        crate::rules::flag(world, id)?;
    }
    if state.shop_purchases.keys().any(|id| !id.contains(':')) {
        return Err("Invalid shop purchase key in RORK save".into());
    }
    if state.next_enemy_id == 0 {
        return Err("RORK next_enemy_id must be positive".into());
    }
    if state.active_enemies.len() > 12 {
        return Err("RORK active encounter exceeds 12 enemies".into());
    }
    if state.character.is_none()
        && (!state.active_enemies.is_empty()
            || !state.room_encounters.is_empty()
            || !state.completed_unique_encounters.is_empty())
    {
        return Err("RORK non-RPG state cannot contain active enemies".into());
    }
    let mut seen = HashSet::new();
    for (room, stored) in &state.room_encounters {
        if room == &state.current_room
            || world.room(room).is_none()
            || stored.is_empty()
            || stored.len() > 12
        {
            return Err(format!(
                "RORK has invalid remembered encounter room '{room}'"
            ));
        }
    }
    for room in &state.completed_unique_encounters {
        if world.room(room).is_none() {
            return Err(format!(
                "RORK has unknown completed encounter room '{room}'"
            ));
        }
    }
    for enemy in state
        .active_enemies
        .iter()
        .chain(state.room_encounters.values().flatten())
    {
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
    crate::objects::validate_state(world, state)?;
    world.systems.validate_state(state)?;
    world.dialogue.validate_state(&state.dialogue, world)?;
    world.events.validate_state(&state.events)?;
    Ok(())
}
