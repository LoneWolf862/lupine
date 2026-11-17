use crate::state::{GameState, ItemLocation};
use crate::world::{select_description, Room, World};

pub fn describe_room(world: &World, state: &GameState, room: &Room) -> String {
    let exits = room
        .exits
        .iter()
        .map(|(direction, _)| direction.label())
        .collect::<Vec<_>>();
    let objects = world
        .items
        .iter()
        .filter(|item| state.location(&item.id) == Some(ItemLocation::Room(room.id.clone())))
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    let visible = if objects.is_empty() {
        "Objects: none".to_string()
    } else {
        format!("Objects: {}", objects.join(", "))
    };
    format!(
        "{}\n{}\nExits: {}\n{}",
        room.title,
        select_description(&room.description, &room.descriptions, state),
        exits.join(", "),
        visible
    )
}

pub fn help() -> &'static str {
    "Commands: look (l), directions (n/s/e/w/u/d), go <direction>, examine <object> (x), take <object> (get/grab/pick up), drop <object>, inventory (i), use <item> on <target>, help (h), quit (q)."
}
