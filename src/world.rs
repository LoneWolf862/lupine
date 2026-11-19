use std::collections::HashSet;

use crate::commands::Direction;
use crate::state::GameState;

/// IDs are owned so worlds can eventually be constructed from disk-loaded text.
/// They remain stable identifiers, not player-facing names.
pub type RoomId = String;
pub type ItemId = String;
pub type FlagId = String;

pub use crate::rules::{Condition, Effect};

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
    /// Optional default text art for this room.
    pub art: Option<String>,
    /// First variant whose conditions match takes precedence over default art.
    pub art_variants: Vec<DescriptionVariant>,
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
    pub outcomes: Vec<crate::rules::Outcome>,
    pub item: ItemId,
    pub target: ItemId,
    pub requires: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub failure_text: String,
    pub success_text: String,
}

#[derive(Debug)]
pub struct World {
    pub dialogue: crate::dialogue::Content,
    pub events: crate::events::Content,
    pub object_defs: std::collections::BTreeMap<String, crate::objects::Definition>,
    pub actions: Vec<crate::actions::Action>,
    pub rules: crate::rules::StoryRules,
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
        let mut errors = crate::actions::validate(self);
        if let Err(e) = self.dialogue.validate(self) {
            errors.push(format!("dialogue.toml: {e}"));
        }
        if let Err(e) = self.events.validate(self) {
            errors.push(format!("events.toml: {e}"));
        }
        if let Err(e) = crate::objects::validate(self) {
            errors.push(e);
        }
        if let Err(e) = self.rules.validate() {
            errors.push(e);
        }
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
            for variant in room.descriptions.iter().chain(&room.art_variants) {
                check_conditions(&variant.when, self, &mut errors);
            }
        }
        for item in &self.items {
            if self
                .object_defs
                .get(&item.id)
                .is_none_or(|d| d.initial.is_none())
                && !rooms.contains(item.starting_room.as_str())
            {
                errors.push(format!(
                    "Item '{}' starts in missing room '{}'",
                    item.id, item.starting_room
                ));
            }
            for variant in &item.descriptions {
                check_conditions(&variant.when, self, &mut errors);
            }
        }
        let mut interactions = HashSet::new();
        for interaction in &self.interactions {
            if interaction.effects.len() > 256 || interaction.outcomes.len() > 128 {
                errors.push("Interaction exceeds 256 effects or 128 outcomes".into());
            }
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
            check_conditions(&interaction.requires, self, &mut errors);
            for effect in &interaction.effects {
                if let Err(e) = effect.validate(self) {
                    errors.push(format!(
                        "Interaction '{} -> {}': {e}",
                        interaction.item, interaction.target
                    ));
                }
            }
            for outcome in &interaction.outcomes {
                if outcome.effects.len() > 256 {
                    errors.push("Outcome exceeds 256 effects".into());
                }
                check_conditions(&outcome.when, self, &mut errors);
                for effect in &outcome.effects {
                    if let Err(e) = effect.validate(self) {
                        errors.push(e);
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
            check_conditions(&rule.requires, self, &mut errors);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn check_conditions(conditions: &[Condition], world: &World, errors: &mut Vec<String>) {
    for condition in conditions {
        if let Err(e) = condition.validate(world) {
            errors.push(e);
        }
    }
}
