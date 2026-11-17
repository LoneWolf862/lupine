use std::collections::HashSet;

use crate::commands::Direction;
use crate::state::GameState;

/// Stable identifiers, separate from player-facing names.
/// This first Rust-authored API uses string literals; a file loader is a later milestone.
pub type RoomId = &'static str;
pub type ItemId = &'static str;
pub type FlagId = &'static str;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    HasItem(ItemId),
    FlagSet(FlagId),
    FlagUnset(FlagId),
}

impl Condition {
    pub fn met(&self, state: &GameState) -> bool {
        match self {
            Self::HasItem(id) => state.location(id) == Some(crate::state::ItemLocation::Inventory),
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
    pub text: &'static str,
}

pub fn select_description<'a>(
    default: &'a str,
    variants: &'a [DescriptionVariant],
    state: &GameState,
) -> &'a str {
    variants
        .iter()
        .find(|variant| variant.when.iter().all(|condition| condition.met(state)))
        .map_or(default, |variant| variant.text)
}

#[derive(Debug)]
pub struct Room {
    pub id: RoomId,
    pub title: &'static str,
    pub description: &'static str,
    pub descriptions: Vec<DescriptionVariant>,
    pub exits: Vec<(Direction, RoomId)>,
}

#[derive(Debug)]
pub struct Item {
    pub id: ItemId,
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub description: &'static str,
    pub descriptions: Vec<DescriptionVariant>,
    pub portable: bool,
    pub starting_room: RoomId,
}

#[derive(Debug)]
pub struct ExitRule {
    pub from: RoomId,
    pub direction: Direction,
    pub requires: Vec<Condition>,
    pub blocked_text: &'static str,
}

#[derive(Debug)]
pub struct Interaction {
    pub item: ItemId,
    pub target: ItemId,
    pub requires: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub failure_text: &'static str,
    pub success_text: &'static str,
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
    pub fn room(&self, id: RoomId) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id == id)
    }

    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Check data authored by a game developer before starting a session.
    /// Returns every detected issue, rather than failing at the first one.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut rooms = HashSet::new();
        let mut items = HashSet::new();
        for room in &self.rooms {
            if room.id.is_empty() || !rooms.insert(room.id) {
                errors.push(format!("Duplicate or empty room id: '{}'", room.id));
            }
        }
        for item in &self.items {
            if item.id.is_empty() || !items.insert(item.id) {
                errors.push(format!("Duplicate or empty item id: '{}'", item.id));
            }
        }
        if !rooms.contains(self.starting_room) {
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
                if !rooms.contains(destination) {
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
            if !rooms.contains(item.starting_room) {
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
            if !items.contains(interaction.item) || !items.contains(interaction.target) {
                errors.push(format!(
                    "Interaction '{} -> {}' references an unknown item",
                    interaction.item, interaction.target
                ));
            }
            if !interactions.insert((interaction.item, interaction.target)) {
                errors.push(format!(
                    "Duplicate interaction '{} -> {}'",
                    interaction.item, interaction.target
                ));
            }
            check_conditions(&interaction.requires, &items, &mut errors);
            for effect in &interaction.effects {
                if let Effect::MoveItemToRoom { item, room } = effect {
                    if !items.contains(item) || !rooms.contains(room) {
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
            if !rooms.contains(rule.from)
                || !self
                    .room(rule.from)
                    .is_some_and(|room| room.exits.iter().any(|(dir, _)| *dir == rule.direction))
            {
                errors.push(format!(
                    "Exit rule references missing exit {} from '{}'",
                    rule.direction.label(),
                    rule.from
                ));
            }
            if !exit_rules.insert((rule.from, rule.direction)) {
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

fn check_conditions(conditions: &[Condition], items: &HashSet<ItemId>, errors: &mut Vec<String>) {
    for condition in conditions {
        if let Condition::HasItem(id) = condition {
            if !items.contains(id) {
                errors.push(format!("Condition references missing item '{id}'"));
            }
        }
    }
}
