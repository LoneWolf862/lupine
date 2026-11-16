use crate::world::Room;

pub fn describe_room(room: &Room) -> String {
    let exits = room.exits.iter().map(|(direction, _)| direction.label()).collect::<Vec<_>>();
    format!("{}\n{}\nExits: {}", room.title, room.description, exits.join(", "))
}

pub fn help() -> &'static str {
    "Commands: look (l), north/south/east/west/up/down (n/s/e/w/u/d), go <direction>, help (h), quit (q).\nMore interactions will be added in upcoming milestones."
}
