//! Shared typed, bounded story conditions and atomic effects.
use crate::{
    state::{GameState, ItemLocation},
    world::World,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StoryRules {
    pub flags: BTreeMap<String, bool>,
    pub counters: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryState {
    pub counters: BTreeMap<String, i64>,
    #[serde(deserialize_with = "unique_visited")]
    pub visited: BTreeSet<String>,
}
fn unique_visited<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeSet<String>, D::Error> {
    let values = Vec::<String>::deserialize(d)?;
    let set = values.iter().cloned().collect::<BTreeSet<_>>();
    if values.len() != set.len() {
        return Err(serde::de::Error::custom("Duplicate visited room"));
    }
    Ok(set)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Compare {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
impl Compare {
    fn met(&self, a: i64, b: i64) -> bool {
        match self {
            Self::Eq => a == b,
            Self::Ne => a != b,
            Self::Lt => a < b,
            Self::Le => a <= b,
            Self::Gt => a > b,
            Self::Ge => a >= b,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    HasItem(String),
    FlagSet(String),
    FlagUnset(String),
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    InRoom(String),
    Visited(String),
    ItemInRoom { item: String, room: String },
    Counter { id: String, op: Compare, value: i64 },
}
impl Condition {
    pub fn met(&self, state: &GameState) -> bool {
        match self {
            Self::HasItem(id) => state.location(id) == Some(ItemLocation::Inventory),
            Self::FlagSet(id) => state.flags.contains(id),
            Self::FlagUnset(id) => !state.flags.contains(id),
            Self::All(c) => c.iter().all(|c| c.met(state)),
            Self::Any(c) => c.iter().any(|c| c.met(state)),
            Self::Not(c) => !c.met(state),
            Self::InRoom(id) => state.current_room == *id,
            Self::Visited(id) => state.story.visited.contains(id),
            Self::ItemInRoom { item, room } => {
                state.location(item) == Some(ItemLocation::Room(room.clone()))
            }
            Self::Counter { id, op, value } => state
                .story
                .counters
                .get(id)
                .is_some_and(|v| op.met(*v, *value)),
        }
    }
    pub fn validate(&self, world: &World) -> Result<(), String> {
        self.check(world, 0)
    }
    fn check(&self, world: &World, depth: usize) -> Result<(), String> {
        if depth > 16 {
            return Err("Conditions exceed 16 nesting levels".into());
        }
        match self {
            Self::HasItem(id) => item(world, id),
            Self::FlagSet(id) | Self::FlagUnset(id) => flag(world, id),
            Self::InRoom(id) | Self::Visited(id) => room(world, id),
            Self::ItemInRoom { item: id, room: r } => {
                item(world, id)?;
                room(world, r)
            }
            Self::Counter { id, .. } => counter(world, id),
            Self::All(c) | Self::Any(c) => {
                if c.is_empty() || c.len() > 128 {
                    return Err("all/any needs 1..128 conditions".into());
                }
                for c in c {
                    c.check(world, depth + 1)?;
                }
                Ok(())
            }
            Self::Not(c) => c.check(world, depth + 1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    SetFlag(String),
    ClearFlag(String),
    MoveItemToRoom { item: String, room: String },
    GiveItem(String),
    RemoveItem(String),
    MovePlayer(String),
    ShowText(String),
    SetCounter { id: String, value: i64 },
    AddCounter { id: String, amount: i64 },
}
impl Effect {
    pub fn validate(&self, world: &World) -> Result<(), String> {
        match self {
            Self::SetFlag(id) | Self::ClearFlag(id) => flag(world, id),
            Self::MoveItemToRoom { item: id, room: r } => item(world, id)
                .and_then(|()| room(world, r))
                .map_err(|e| format!("Move effect: {e}")),
            Self::GiveItem(id) => {
                item(world, id)?;
                if !world.item(id).is_some_and(|i| i.portable) {
                    return Err(format!("Cannot give fixed item '{id}'"));
                }
                Ok(())
            }
            Self::RemoveItem(id) => item(world, id),
            Self::MovePlayer(id) => room(world, id),
            Self::ShowText(s) => {
                if s.chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\t' | '\r'))
                {
                    Err("Text contains terminal controls".into())
                } else {
                    Ok(())
                }
            }
            Self::SetCounter { id, .. } | Self::AddCounter { id, .. } => counter(world, id),
        }
    }
    fn apply(&self, state: &mut GameState, output: &mut Vec<String>) -> Result<(), String> {
        match self {
            Self::SetFlag(id) => {
                state.flags.insert(id.clone());
            }
            Self::ClearFlag(id) => {
                state.flags.remove(id);
            }
            Self::MoveItemToRoom { item, room } => {
                state
                    .item_locations
                    .insert(item.clone(), ItemLocation::Room(room.clone()));
            }
            Self::GiveItem(id) => {
                state
                    .item_locations
                    .insert(id.clone(), ItemLocation::Inventory);
            }
            Self::RemoveItem(id) => {
                if state.location(id) == Some(ItemLocation::Consumed) {
                    return Err(format!("Item '{id}' is already unavailable"));
                }
                state
                    .item_locations
                    .insert(id.clone(), ItemLocation::Consumed);
            }
            Self::MovePlayer(id) => {
                state.current_room = id.clone();
                state.story.visited.insert(id.clone());
            }
            Self::ShowText(s) => output.push(s.clone()),
            Self::SetCounter { id, value } => {
                *state
                    .story
                    .counters
                    .get_mut(id)
                    .ok_or_else(|| format!("Missing counter '{id}'"))? = *value;
            }
            Self::AddCounter { id, amount } => {
                let current = state
                    .story
                    .counters
                    .get_mut(id)
                    .ok_or_else(|| format!("Missing counter '{id}'"))?;
                *current = current
                    .checked_add(*amount)
                    .ok_or_else(|| format!("Counter '{id}' would overflow"))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    #[serde(default)]
    pub when: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    pub text: String,
}

/// Stage all effects and output, then publish in-memory state only on complete success.
pub fn apply(
    world: &World,
    state: &mut GameState,
    effects: &[Effect],
) -> Result<Vec<String>, String> {
    if effects.len() > 256 {
        return Err("At most 256 effects per outcome".into());
    }
    let mut draft = state.clone();
    let mut text = Vec::new();
    for effect in effects {
        effect.validate(world)?;
        effect.apply(&mut draft, &mut text)?;
    }
    *state = draft;
    Ok(text)
}

pub fn valid_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 80
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    {
        Err(format!(
            "Invalid state ID '{id}'; use lowercase letters, digits, '_' or '-' (1..80 bytes)"
        ))
    } else {
        Ok(())
    }
}
fn item(world: &World, id: &str) -> Result<(), String> {
    if world.item(id).is_some() {
        Ok(())
    } else {
        Err(format!("Unknown item '{id}'"))
    }
}
fn room(world: &World, id: &str) -> Result<(), String> {
    if world.room(id).is_some() {
        Ok(())
    } else {
        Err(format!("Unknown room '{id}'"))
    }
}
fn counter(world: &World, id: &str) -> Result<(), String> {
    if world.rules.counters.contains_key(id) {
        Ok(())
    } else {
        Err(format!(
            "Undeclared counter '{id}' in rules.toml [counters]"
        ))
    }
}
pub(crate) fn flag(world: &World, id: &str) -> Result<(), String> {
    valid_id(id)?;
    if !world.rules.flags.is_empty() && !world.rules.flags.contains_key(id) {
        Err(format!("Undeclared flag '{id}' in rules.toml [flags]"))
    } else {
        Ok(())
    }
}

impl StoryRules {
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join("rules.toml");
        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))
    }
    pub fn validate(&self) -> Result<(), String> {
        for id in self.flags.keys().chain(self.counters.keys()) {
            valid_id(id)?;
        }
        Ok(())
    }
    pub fn validate_state(&self, state: &StoryState, world: &World) -> Result<(), String> {
        if self.counters.keys().collect::<Vec<_>>() != state.counters.keys().collect::<Vec<_>>() {
            return Err("Saved counters differ from rules.toml declarations".into());
        }
        for id in &state.visited {
            room(world, id)?;
        }
        Ok(())
    }
}

/// Bound nested authored data before recursive deserialization.
pub fn decode<T: serde::de::DeserializeOwned>(value: &toml::Value) -> Result<T, String> {
    fn bound(v: &toml::Value, d: usize) -> Result<(), String> {
        if d > 40 {
            return Err("Rule exceeds nesting limit".into());
        }
        match v {
            toml::Value::Array(a) => {
                if a.len() > 128 {
                    return Err("Rule list exceeds 128 entries".into());
                }
                for v in a {
                    bound(v, d + 1)?;
                }
            }
            toml::Value::Table(t) => {
                for v in t.values() {
                    bound(v, d + 1)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    bound(value, 0)?;
    value
        .clone()
        .try_into()
        .map_err(|e| format!("Invalid rule: {e}"))
}
