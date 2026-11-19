use std::collections::{BTreeMap, HashSet};

use crate::world::{ItemId, RoomId, World};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ItemLocation {
    Room(RoomId),
    Container(ItemId),
    Inventory,
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub story: crate::rules::StoryState,
    pub current_room: RoomId,
    pub objects: crate::objects::ObjectState,
    pub flags: HashSet<String>,
    pub character: Option<crate::rpg::Character>,
    pub learned_abilities: HashSet<String>,
    pub wallet: BTreeMap<String, u64>,
    /// Successful purchases of finite-stock offers, keyed by shop_id:offer_id.
    pub shop_purchases: BTreeMap<String, u32>,
    /// Active encounter only; cleared when moving to another room.
    pub active_enemies: Vec<crate::encounters::EnemyInstance>,
    /// Surviving enemies retained in rooms configured with room/unique persistence.
    pub room_encounters: BTreeMap<String, Vec<crate::encounters::EnemyInstance>>,
    /// Unique rooms whose encounters have been defeated permanently.
    pub completed_unique_encounters: HashSet<String>,
    /// Monotonically increasing ID for unique enemy targets within this session/save.
    pub next_enemy_id: u64,
}

impl GameState {
    pub fn new(world: &World) -> Self {
        Self {
            story: crate::rules::StoryState {
                counters: world.rules.counters.clone(),
                visited: [world.starting_room.clone()].into_iter().collect(),
            },
            current_room: world.starting_room.clone(),
            objects: crate::objects::ObjectState::new(world),
            flags: world
                .rules
                .flags
                .iter()
                .filter(|(_, v)| **v)
                .map(|(id, _)| id.clone())
                .collect(),
            character: None,
            learned_abilities: HashSet::new(),
            wallet: BTreeMap::new(),
            shop_purchases: BTreeMap::new(),
            active_enemies: Vec::new(),
            next_enemy_id: 1,
            room_encounters: BTreeMap::new(),
            completed_unique_encounters: HashSet::new(),
        }
    }

    /// Summary for callers needing one location. Use count_at for split stacks.
    pub fn location(&self, id: &str) -> Option<ItemLocation> {
        let stacks = self.objects.placements.get(id)?;
        if stacks.iter().any(|s| s.location == ItemLocation::Inventory) {
            return Some(ItemLocation::Inventory);
        }
        Some(
            stacks
                .first()
                .map_or(ItemLocation::Consumed, |s| s.location.clone()),
        )
    }
    pub fn is_accessible(&self, id: &str) -> bool {
        !self.objects.hidden.contains(id)
            && self
                .objects
                .placements
                .get(id)
                .into_iter()
                .flatten()
                .any(|s| s.quantity > 0 && self.place_accessible(&s.location, 0))
    }
}
