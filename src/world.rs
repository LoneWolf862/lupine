use std::collections::HashSet;

use crate::commands::Direction;
use crate::state::{GameState, ItemLocation};

/// IDs are owned so worlds can eventually be constructed from disk-loaded text.
/// They remain stable identifiers, not player-facing names.
pub type RoomId = String;
pub type ItemId = String;
pub type FlagId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    HasItem(ItemId),
    FlagSet(FlagId),
    FlagUnset(FlagId),
}

impl Condition {
    pub fn met(&self, state: &GameState) -> bool {
        match self {
            Self::HasItem(id) => state.location(id) == Some(ItemLocation::Inventory),
            Self::FlagSet(flag) => state.flags.contains(flag),
            Self::FlagUnset(flag) => !state.flags.contains(flag),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    SetFlag(FlagId),
    ClearFlag(FlagId),
    MoveItemToRoom { item: ItemId, room: RoomId },
}

#[derive(Debug, Clone)]
pub struct DescriptionVariant {
    pub when: Vec<Condition>,
    pub text: String,
}

pub fn select_description<'a>(
    default: &'a str,
    variants: &'a [DescriptionVariant],
    state: &GameState,
) -> &'a str {
    variants
        .iter()
        .find(|variant| variant.when.iter().all(|condition| condition.met(state)))
        .map_or(default, |variant| variant.text.as_str())
}

#[derive(Debug)]
pub struct Room {
    pub id: RoomId,
    pub title: String,
    pub description: String,
    pub descriptions: Vec<DescriptionVariant>,
    pub exits: Vec<(Direction, RoomId)>,
}

#[derive(Debug)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub descriptions: Vec<DescriptionVariant>,
    pub portable: bool,
    pub starting_room: RoomId,
}

#[derive(Debug)]
pub struct ExitRule {
    pub from: RoomId,
    pub direction: Direction,
    pub requires: Vec<Condition>,
    pub blocked_text: String,
}

#[derive(Debug)]
pub struct Interaction {
    pub item: ItemId,
    pub target: ItemId,
    pub requires: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub failure_text: String,
    pub success_text: String,
}

#[derive(Debug)]
pub struct World {
    pub rooms: Vec<Room>,
    pub items: Vec<Item>,
    pub interactions: Vec<Interaction>,
    pub exit_rules: Vec<ExitRule>,
    pub starting_room: RoomId,
}

impl World {
    pub fn room(&self, id: &str) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id == id)
    }

    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Validate an authored world before play; reports all detected problems.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut rooms = HashSet::new();
        let mut items = HashSet::new();
        for room in &self.rooms {
            if room.id.is_empty() || !rooms.insert(room.id.as_str()) {
                errors.push(format!("Duplicate or empty room id: '{}'", room.id));
            }
        }
        for item in &self.items {
            if item.id.is_empty() || !items.insert(item.id.as_str()) {
                errors.push(format!("Duplicate or empty item id: '{}'", item.id));
            }
        }
        if !rooms.contains(self.starting_room.as_str()) {
            errors.push(format!(
                "Starting room '{}' does not exist",
                self.starting_room
            ));
        }
        for room in &self.rooms {
            let mut directions = HashSet::new();
            for (direction, destination) in &room.exits {
                if !directions.insert(direction) {
                    errors.push(format!(
                        "Room '{}' has duplicate {} exits",
                        room.id,
                        direction.label()
                    ));
                }
                if !rooms.contains(destination.as_str()) {
                    errors.push(format!(
                        "Room '{}' points to missing room '{}'",
                        room.id, destination
                    ));
                }
            }
            for variant in &room.descriptions {
                check_conditions(&variant.when, &items, &mut errors);
            }
        }
        for item in &self.items {
            if !rooms.contains(item.starting_room.as_str()) {
                errors.push(format!(
                    "Item '{}' starts in missing room '{}'",
                    item.id, item.starting_room
                ));
            }
            for variant in &item.descriptions {
                check_conditions(&variant.when, &items, &mut errors);
            }
        }
        let mut interactions = HashSet::new();
        for interaction in &self.interactions {
            if !items.contains(interaction.item.as_str())
                || !items.contains(interaction.target.as_str())
            {
                errors.push(format!(
                    "Interaction '{} -> {}' references an unknown item",
                    interaction.item, interaction.target
                ));
            }
            if !interactions.insert((interaction.item.as_str(), interaction.target.as_str())) {
                errors.push(format!(
                    "Duplicate interaction '{} -> {}'",
                    interaction.item, interaction.target
                ));
            }
            check_conditions(&interaction.requires, &items, &mut errors);
            for effect in &interaction.effects {
                if let Effect::MoveItemToRoom { item, room } = effect {
                    if !items.contains(item.as_str()) || !rooms.contains(room.as_str()) {
                        errors.push(format!(
                            "Move effect references missing item '{}' or room '{}'",
                            item, room
                        ));
                    }
                }
            }
        }
        let mut exit_rules = HashSet::new();
        for rule in &self.exit_rules {
            if !rooms.contains(rule.from.as_str())
                || !self
                    .room(&rule.from)
                    .is_some_and(|room| room.exits.iter().any(|(dir, _)| *dir == rule.direction))
            {
                errors.push(format!(
                    "Exit rule references missing exit {} from '{}'",
                    rule.direction.label(),
                    rule.from
                ));
            }
            if !exit_rules.insert((rule.from.as_str(), rule.direction)) {
                errors.push(format!(
                    "Duplicate exit rule {} from '{}'",
                    rule.direction.label(),
                    rule.from
                ));
            }
            check_conditions(&rule.requires, &items, &mut errors);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn check_conditions(conditions: &[Condition], items: &HashSet<&str>, errors: &mut Vec<String>) {
    for condition in conditions {
        if let Condition::HasItem(id) = condition {
            if !items.contains(id.as_str()) {
                errors.push(format!("Condition references missing item '{id}'"));
            }
        }
    }
}
