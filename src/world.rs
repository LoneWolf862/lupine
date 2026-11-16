use crate::commands::Direction;

pub type RoomId = &'static str;
pub type ItemId = &'static str;

#[derive(Debug)]
pub struct Room {
    pub id: RoomId,
    pub title: &'static str,
    pub description: &'static str,
    pub exits: Vec<(Direction, RoomId)>,
}

#[derive(Debug)]
pub struct Item {
    pub id: ItemId,
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub description: &'static str,
    pub portable: bool,
    pub starting_room: RoomId,
}

#[derive(Debug)]
pub struct Interaction {
    pub item: ItemId,
    pub target: ItemId,
    pub flag: &'static str,
    pub success_text: &'static str,
}

#[derive(Debug)]
pub struct World {
    pub rooms: Vec<Room>,
    pub items: Vec<Item>,
    pub interactions: Vec<Interaction>,
    pub starting_room: RoomId,
}

impl World {
    pub fn room(&self, id: RoomId) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id == id)
    }

    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }
}

/// Temporary testing fixture, not HOWLBOUND story content.
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
                description: "A chamber used to verify a stateful interaction.",
                exits: vec![(Direction::West, "corridor")],
            },
        ],
        items: vec![
            Item {
                id: "brass_key",
                name: "brass key",
                aliases: vec!["key", "brass key"],
                description: "An ordinary brass key for engine testing.",
                portable: true,
                starting_room: "foyer",
            },
            Item {
                id: "stone_table",
                name: "stone table",
                aliases: vec!["table", "stone table"],
                description: "A sturdy table fixed to the floor.",
                portable: false,
                starting_room: "foyer",
            },
            Item {
                id: "test_panel",
                name: "test panel",
                aliases: vec!["panel", "test panel"],
                description: "A stationary panel with a key-shaped slot.",
                portable: false,
                starting_room: "chamber",
            },
        ],
        interactions: vec![Interaction {
            item: "brass_key",
            target: "test_panel",
            flag: "panel_activated",
            success_text: "The key turns in the panel. A test indicator lights up.",
        }],
    }
}
