use crate::commands::Direction;

pub type RoomId = &'static str;

#[derive(Debug)]
pub struct Room {
    pub id: RoomId,
    pub title: &'static str,
    pub description: &'static str,
    pub exits: Vec<(Direction, RoomId)>,
}

#[derive(Debug)]
pub struct World {
    pub rooms: Vec<Room>,
    pub starting_room: RoomId,
}

impl World {
    pub fn room(&self, id: RoomId) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id == id)
    }
}

/// Temporary test fixture; this is not HOWLBOUND story content.
pub fn fixture_world() -> World {
    World {
        starting_room: "foyer",
        rooms: vec![
            Room {
                id: "foyer",
                title: "Test Foyer",
                description: "A plain room used to test navigation. A corridor leads north.",
                exits: vec![(Direction::North, "corridor")],
            },
            Room {
                id: "corridor",
                title: "Test Corridor",
                description: "A short corridor. The foyer is south; a test chamber is east.",
                exits: vec![(Direction::South, "foyer"), (Direction::East, "chamber")],
            },
            Room {
                id: "chamber",
                title: "Test Chamber",
                description: "An empty chamber reserved for later interaction tests.",
                exits: vec![(Direction::West, "corridor")],
            },
        ],
    }
}
