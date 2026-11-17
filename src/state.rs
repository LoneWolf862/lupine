use std::collections::{BTreeMap, HashMap, HashSet};

use crate::world::{ItemId, RoomId, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemLocation {
    Room(RoomId),
    Inventory,
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub current_room: RoomId,
    pub item_locations: HashMap<ItemId, ItemLocation>,
    pub flags: HashSet<String>,
    pub character: Option<crate::rpg::Character>,
    pub learned_abilities: HashSet<String>,
    pub wallet: BTreeMap<String, u64>,
    /// Successful purchases of finite-stock offers, keyed by shop_id:offer_id.
    pub shop_purchases: BTreeMap<String, u32>,
    /// Active encounter only; cleared when moving to another room.
    pub active_enemies: Vec<crate::encounters::EnemyInstance>,
    /// Monotonically increasing ID for unique enemy targets within this session/save.
    pub next_enemy_id: u64,
}

impl GameState {
    pub fn new(world: &World) -> Self {
        let item_locations = world
            .items
            .iter()
            .map(|item| {
                (
                    item.id.clone(),
                    ItemLocation::Room(item.starting_room.clone()),
                )
            })
            .collect();
        Self {
            current_room: world.starting_room.clone(),
            item_locations,
            flags: HashSet::new(),
            character: None,
            learned_abilities: HashSet::new(),
            wallet: BTreeMap::new(),
            shop_purchases: BTreeMap::new(),
            active_enemies: Vec::new(),
            next_enemy_id: 1,
        }
    }

    pub fn location(&self, id: &str) -> Option<ItemLocation> {
        self.item_locations.get(id).cloned()
    }

    pub fn is_accessible(&self, id: &str) -> bool {
        match self.location(id) {
            Some(ItemLocation::Inventory) => true,
            Some(ItemLocation::Room(room)) => room == self.current_room,
            Some(ItemLocation::Consumed) => false,
            None => false,
        }
    }
}
