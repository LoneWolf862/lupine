//! Read-only author inspection; never a player command or a save writer.
use std::{io::Write, path::Path};
pub fn run(root: &Path, out: &mut impl Write) -> Result<(), String> {
    let snapshot = super::room_transaction::Snapshot::capture(root)?;
    let adventure = crate::adventure::Adventure::load(root)?;
    adventure.summary(out).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "\nAUTHORED ROOM MAP (conditional exits may be blocked in play)"
    )
    .map_err(|e| e.to_string())?;
    let mut rooms = adventure.game.world.rooms.iter().collect::<Vec<_>>();
    rooms.sort_by(|a, b| a.id.cmp(&b.id));
    for room in rooms {
        writeln!(out, "{}: {}", room.id, room.title).map_err(|e| e.to_string())?;
        let mut exits = room
            .exits
            .iter()
            .map(|(d, r)| format!("  {d:?} -> {r}"))
            .collect::<Vec<_>>();
        exits.sort();
        for exit in exits {
            writeln!(out, "{exit}").map_err(|e| e.to_string())?;
        }
    }
    writeln!(
        out,
        "\nTYPED ID OCCURRENCES (definitions and references; prose excluded)"
    )
    .map_err(|e| e.to_string())?;
    for line in super::migration::references(&snapshot)? {
        writeln!(out, "{line}").map_err(|e| e.to_string())?;
    }
    Ok(())
}
