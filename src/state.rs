use crate::world::{RoomId, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub current_room: RoomId,
}

impl GameState {
    pub fn new(world: &World) -> Self {
        Self { current_room: world.starting_room }
    }
}
