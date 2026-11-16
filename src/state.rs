use std::collections::{HashMap, HashSet};

use crate::world::{ItemId, RoomId, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemLocation {
    Room(RoomId),
    Inventory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub current_room: RoomId,
    pub item_locations: HashMap<ItemId, ItemLocation>,
    pub flags: HashSet<&'static str>,
}

impl GameState {
    pub fn new(world: &World) -> Self {
        let item_locations = world
            .items
            .iter()
            .map(|item| (item.id, ItemLocation::Room(item.starting_room)))
            .collect();
        Self {
            current_room: world.starting_room,
            item_locations,
            flags: HashSet::new(),
        }
    }

    pub fn location(&self, id: ItemId) -> Option<ItemLocation> {
        self.item_locations.get(id).copied()
    }

    pub fn is_accessible(&self, id: ItemId) -> bool {
        match self.location(id) {
            Some(ItemLocation::Inventory) => true,
            Some(ItemLocation::Room(room)) => room == self.current_room,
            None => false,
        }
    }
}
