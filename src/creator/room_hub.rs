//! Room-centered navigation. Each editor owns and confirms its own transaction.
use super::{
    content_edit::{choose, entries},
    room_transaction::Snapshot,
    wizard::Wizard,
};
use std::{
    io::{self, BufRead, Write},
    path::Path,
};
pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    let result = (|| -> Result<(), String> {
        loop {
            let snapshot = Snapshot::capture(root)?;
            let Some(mut id) = choose(&mut w, "BROWSE BY ROOM", &entries(&snapshot, "rooms")?)?
            else {
                return Ok(());
            };
            loop {
                let snapshot = Snapshot::capture(root)?;
                let a = crate::adventure::Adventure::load(root)?;
                let Some(room) = a.game.world.room(&id) else {
                    w.say("Room no longer exists; choose another.")?;
                    break;
                };
                w.heading(&format!("ROOM: {} ({id})", room.title))?;
                w.say(&room.description)?;
                let mut exits = room
                    .exits
                    .iter()
                    .map(|(d, id)| {
                        (
                            id.clone(),
                            format!(
                                "{d:?}: {}",
                                a.game
                                    .world
                                    .room(id)
                                    .map(|r| r.title.as_str())
                                    .unwrap_or(id)
                            ),
                        )
                    })
                    .collect::<Vec<_>>();
                exits.sort();
                // An exit destination can occur in multiple directions. Keep one navigation entry.
                exits.dedup_by(|a, b| a.0 == b.0);
                w.say("Saved-content overview; no active draft. Related files below contain typed references to this room (including exits, placements, actions, events, NPCs and shops). Conditional content may differ during play.")?;
                let related = super::migration::referenced_files(&snapshot, "rooms", &id)?;
                for p in &related {
                    w.say(&format!("  {}", p.display()))?;
                }
                w.say("1. Edit this room (guided)\n2. Browse/edit related files\n3. Follow an exit\n4. All project files / new systems\n0. Back to room list\nEditors confirm separately. Returning here reloads saved files; no hidden shared draft or automatic save.")?;
                match w.menu("Selection:")?.to_ascii_lowercase().as_str() {
                    "0" | "back" => break,
                    "1" => super::room_edit::edit_selected(root, &id, w.input, w.output)
                        .map_err(|e| e.to_string())?,
                    "2" => {
                        let list = related
                            .iter()
                            .map(|p| {
                                let s = p.to_string_lossy().replace('\\', "/");
                                (s.clone(), s)
                            })
                            .collect::<Vec<_>>();
                        if let Some(p) = choose(&mut w, "RELATED FILES", &list)? {
                            super::project_edit::run_at(root, Some(p.into()), w.input, w.output)
                                .map_err(|e| e.to_string())?;
                        }
                    }
                    "3" => {
                        if let Some(next) = choose(&mut w, "EXIT DESTINATION", &exits)? {
                            id = next;
                        }
                    }
                    "4" => super::project_edit::run(root, w.input, w.output)
                        .map_err(|e| e.to_string())?,
                    _ => w.say("Choose 0-4. Back keeps earlier confirmed saves.")?,
                }
            }
        }
    })();
    if let Err(e) = result {
        w.say(&format!(
            "Room navigation stopped: {e}. Use F for project repair if needed."
        ))
        .map_err(io::Error::other)?;
    }
    Ok(())
}
