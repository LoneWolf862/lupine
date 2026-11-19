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
    fn met<T: Ord>(&self, a: T, b: T) -> bool {
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
    QuestStage {
        quest: String,
        stage: String,
    },
    JournalHas(String),
    TimerRunning(String),
    Turn {
        op: Compare,
        value: u64,
    },
    HasItem(String),
    ItemVisible(String),
    ContainerOpen(String),
    ContainerLocked(String),
    ItemCount {
        item: String,
        op: Compare,
        value: u32,
        #[serde(default)]
        location: Option<ItemLocation>,
    },
    FlagSet(String),
    FlagUnset(String),
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    InRoom(String),
    Visited(String),
    ItemInRoom {
        item: String,
        room: String,
    },
    Counter {
        id: String,
        op: Compare,
        value: i64,
    },
}
impl Condition {
    pub fn met(&self, state: &GameState) -> bool {
        match self {
            Self::QuestStage { quest, stage } => state.events.quests.get(quest) == Some(stage),
            Self::JournalHas(id) => state.events.journal.contains(id),
            Self::TimerRunning(id) => state.events.timers.contains_key(id),
            Self::Turn { op, value } => op.met(state.events.turn, *value),
            Self::HasItem(id) => state.is_carried(id),
            Self::ItemVisible(id) => state.is_accessible(id),
            Self::ContainerOpen(id) => state.objects.containers.get(id).is_some_and(|c| c.open),
            Self::ContainerLocked(id) => state.objects.containers.get(id).is_some_and(|c| c.locked),
            Self::ItemCount {
                item,
                op,
                value,
                location,
            } => op.met(
                i64::from(
                    location
                        .as_ref()
                        .map_or_else(|| state.carried_quantity(item), |p| state.count_at(item, p)),
                ),
                i64::from(*value),
            ),
            Self::FlagSet(id) => state.flags.contains(id),
            Self::FlagUnset(id) => !state.flags.contains(id),
            Self::All(c) => c.iter().all(|c| c.met(state)),
            Self::Any(c) => c.iter().any(|c| c.met(state)),
            Self::Not(c) => !c.met(state),
            Self::InRoom(id) => state.current_room == *id,
            Self::Visited(id) => state.story.visited.contains(id),
            Self::ItemInRoom { item, room } => {
                state.count_at(item, &ItemLocation::Room(room.clone())) > 0
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
            Self::QuestStage { quest, stage } => quest_stage(world, quest, stage),
            Self::JournalHas(id) => known(&world.events.journal, id, "journal entry"),
            Self::TimerRunning(id) => known(&world.events.timers, id, "timer"),
            Self::Turn { .. } => Ok(()),
            Self::HasItem(id) | Self::ItemVisible(id) => item(world, id),
            Self::ContainerOpen(id) | Self::ContainerLocked(id) => {
                crate::objects::validate_location(world, &ItemLocation::Container(id.clone()))
            }
            Self::ItemCount {
                item: id, location, ..
            } => {
                item(world, id)?;
                if let Some(p) = location {
                    crate::objects::validate_location(world, p)?;
                }
                Ok(())
            }
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
    StartTimer(String),
    CancelTimer(String),
    SetQuestStage {
        quest: String,
        stage: String,
    },
    AddJournal(String),
    EndGame(String),
    SetFlag(String),
    ClearFlag(String),
    MoveItemToRoom {
        item: String,
        room: String,
    },
    GiveItem(String),
    RemoveItem(String),
    RevealItem(String),
    HideItem(String),
    AddItem {
        item: String,
        quantity: u32,
    },
    RemoveQuantity {
        item: String,
        quantity: u32,
    },
    TransferItem {
        item: String,
        from: ItemLocation,
        to: ItemLocation,
        quantity: u32,
    },
    SetContainer {
        item: String,
        #[serde(default)]
        open: Option<bool>,
        #[serde(default)]
        locked: Option<bool>,
    },
    MovePlayer(String),
    ShowText(String),
    SetCounter {
        id: String,
        value: i64,
    },
    AddCounter {
        id: String,
        amount: i64,
    },
}
impl Effect {
    pub fn validate(&self, world: &World) -> Result<(), String> {
        match self {
            Self::StartTimer(id) | Self::CancelTimer(id) => {
                known(&world.events.timers, id, "timer")
            }
            Self::SetQuestStage { quest, stage } => quest_stage(world, quest, stage),
            Self::AddJournal(id) => known(&world.events.journal, id, "journal entry"),
            Self::EndGame(id) => known(&world.events.endings, id, "ending"),
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
            Self::RemoveItem(id) | Self::RevealItem(id) | Self::HideItem(id) => item(world, id),
            Self::AddItem { item: id, quantity }
            | Self::RemoveQuantity { item: id, quantity }
            | Self::TransferItem {
                item: id, quantity, ..
            } => {
                item(world, id)?;
                if !world.item(id).unwrap().portable
                    || *quantity == 0
                    || *quantity > crate::objects::MAX_QUANTITY
                    || (!crate::objects::definition(world, id).stackable && *quantity > 1)
                {
                    return Err(format!("Invalid portable item quantity for '{id}'"));
                }
                if let Self::TransferItem { from, to, .. } = self {
                    crate::objects::validate_location(world, from)?;
                    crate::objects::validate_location(world, to)?;
                }
                Ok(())
            }
            Self::SetContainer {
                item: id,
                open,
                locked,
            } => {
                crate::objects::validate_location(world, &ItemLocation::Container(id.clone()))?;
                if (open.is_none() && locked.is_none())
                    || (*open == Some(true) && *locked == Some(true))
                {
                    return Err("SetContainer needs a valid open/locked change".into());
                }
                Ok(())
            }
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
    fn apply(
        &self,
        world: &World,
        state: &mut GameState,
        output: &mut Vec<String>,
    ) -> Result<(), String> {
        match self {
            Self::StartTimer(id) => crate::events::start_timer(world, state, id)?,
            Self::CancelTimer(id) => {
                state.events.timers.remove(id);
            }
            Self::SetQuestStage { quest, stage } => {
                state.events.quests.insert(quest.clone(), stage.clone());
            }
            Self::AddJournal(id) => {
                if !state.events.journal.contains(id) {
                    state.events.journal.push(id.clone());
                }
            }
            Self::EndGame(id) => {
                if state.events.ending.is_some() {
                    return Err("The adventure has already ended".into());
                }
                state.events.ending = Some(id.clone());
                let ending = &world.events.endings[id];
                output.push(format!(
                    "{}\n{}",
                    match ending.kind {
                        crate::events::EndingKind::Success => "ADVENTURE COMPLETE",
                        crate::events::EndingKind::Failure => "ADVENTURE ENDED",
                    },
                    ending.text
                ));
            }
            Self::RevealItem(id) => {
                state.objects.hidden.remove(id);
            }
            Self::HideItem(id) => {
                state.objects.hidden.insert(id.clone());
            }
            Self::SetContainer { item, open, locked } => {
                let c = state
                    .objects
                    .containers
                    .get_mut(item)
                    .ok_or("Missing container state")?;
                if let Some(v) = open {
                    c.open = *v;
                }
                if let Some(v) = locked {
                    c.locked = *v;
                }
            }
            Self::AddItem { item, quantity } => {
                let stacks = state
                    .objects
                    .placements
                    .get_mut(item)
                    .ok_or("Unknown item")?;
                if let Some(s) = stacks
                    .iter_mut()
                    .find(|s| s.location == ItemLocation::Inventory)
                {
                    s.quantity = s
                        .quantity
                        .checked_add(*quantity)
                        .ok_or("Quantity overflow")?;
                } else {
                    stacks.push(crate::objects::Stack {
                        location: ItemLocation::Inventory,
                        quantity: *quantity,
                    });
                }
                stacks.sort_by(|a, b| a.location.cmp(&b.location));
            }
            Self::RemoveQuantity { item, quantity } => {
                let mut remaining = *quantity;
                while remaining > 0 {
                    let from = state
                        .available_source(item, true)
                        .ok_or("Not enough accessible carried items")?;
                    let amount = remaining.min(state.count_at(item, &from));
                    state.transfer(world, item, &from, &ItemLocation::Consumed, amount)?;
                    remaining -= amount;
                }
            }
            Self::TransferItem {
                item,
                from,
                to,
                quantity,
            } => state.transfer(world, item, from, to, *quantity)?,
            Self::SetFlag(id) => {
                state.flags.insert(id.clone());
            }
            Self::ClearFlag(id) => {
                state.flags.remove(id);
            }
            Self::MoveItemToRoom { item, room } => {
                state.set_location(item.clone(), ItemLocation::Room(room.clone()));
            }
            Self::GiveItem(id) => {
                state.set_location(id.clone(), ItemLocation::Inventory);
            }
            Self::RemoveItem(id) => {
                if state.location(id) == Some(ItemLocation::Consumed) {
                    return Err(format!("Item '{id}' is already unavailable"));
                }
                state.set_location(id.clone(), ItemLocation::Consumed);
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
        effect.apply(world, &mut draft, &mut text)?;
    }
    crate::objects::validate_state(world, &draft)?;
    world.events.validate_state(&draft.events)?;
    *state = draft;
    Ok(text)
}

fn known<T>(map: &BTreeMap<String, T>, id: &str, kind: &str) -> Result<(), String> {
    if map.contains_key(id) {
        Ok(())
    } else {
        Err(format!("Unknown {kind} '{id}'"))
    }
}
fn quest_stage(world: &World, quest: &str, stage: &str) -> Result<(), String> {
    if world
        .events
        .quests
        .get(quest)
        .is_some_and(|q| q.stages.contains_key(stage))
    {
        Ok(())
    } else {
        Err(format!("Unknown quest/stage '{quest}:{stage}'"))
    }
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
