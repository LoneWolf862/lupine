use std::collections::{HashMap, HashSet};

use crate::world::{ItemId, RoomId, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemLocation {
    Room(RoomId),
    Inventory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub current_room: RoomId,
    pub item_locations: HashMap<ItemId, ItemLocation>,
    pub flags: HashSet<String>,
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
        }
    }

    pub fn location(&self, id: &str) -> Option<ItemLocation> {
        self.item_locations.get(id).cloned()
    }

    pub fn is_accessible(&self, id: &str) -> bool {
        match self.location(id) {
            Some(ItemLocation::Inventory) => true,
            Some(ItemLocation::Room(room)) => room == self.current_room,
            None => false,
        }
    }
}
