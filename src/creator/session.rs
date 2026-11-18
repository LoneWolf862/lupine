//! Persistent creator navigation. Existing content is read-only in Phase 2B.
use super::tree;
use crate::adventure::Adventure;
use std::{
    io::{self, BufRead, Write},
    path::Path,
};

fn answer(
    input: &mut impl BufRead,
    output: &mut impl Write,
    prompt: &str,
) -> io::Result<Option<String>> {
    write!(output, "{prompt}")?;
    output.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_owned()))
}
fn error(output: &mut impl Write, detail: &str) -> io::Result<()> {
    writeln!(output, "\nThis adventure is not ready yet:\n{detail}\nNothing was changed. Check the named file and field/reference, correct it\nin your text editor, then choose Validate again. Guided editors arrive in\nlater phases; the creator will reload your files each time.")
}

/// Create/open launcher. The same terminal streams survive setup and gameplay.
/// The save directory is passed to the runner only; this function never writes saves.
pub fn start<R: BufRead, W: Write>(input: &mut R, output: &mut W, saves: &Path) -> io::Result<()> {
    loop {
        writeln!(output, "\nLUPINE ADVENTURE CREATOR\n1. Create a new adventure\n2. Open an existing adventure\n0. Exit\n? or help explains your choices.")?;
        let Some(choice) = answer(input, output, "Selection: ")? else {
            return Ok(());
        };
        match choice.to_ascii_lowercase().as_str() {
            "0" | "back" | "cancel" | "exit" => return Ok(()),
            "1" => {
                match super::new_game_wizard(input, output) {
                    Ok(path) => open(&path, input, output, saves)?,
                    Err(message) => writeln!(output, "{message}")?,
                }
            }
            "2" => loop {
                writeln!(output, "Choose the folder containing game.toml. A relative name is inside\nyour current folder; a full path chooses another location. Do not add\nsurrounding quotes. back/cancel returns to the launcher.")?;
                let Some(path) = answer(input, output, "Adventure folder: ")? else { return Ok(()); };
                match path.to_ascii_lowercase().as_str() {
                    "0" | "back" | "cancel" => break,
                    "?" | "help" => continue,
                    _ => (),
                }
                match tree::root(Path::new(&path)) {
                    Ok(root) => { open(&root, input, output, saves)?; break; }
                    Err(message) => writeln!(output, "Cannot open: {message}")?,
                }
            },
            "?" | "help" => writeln!(output, "Create guides you through a new adventure. Open selects a folder\nyou already made. You can inspect files, validate and play. Existing\nfiles stay unchanged; room/item/configuration editors are later phases.")?,
            _ => writeln!(output, "Choose 1 to create, 2 to open, or 0 to exit.")?,
        }
    }
}

/// Open an existing project, even if its content needs repair. Path/link checks
/// happen before loading; every play or validation reads fresh content.
pub fn open<R: BufRead, W: Write>(
    path: &Path,
    input: &mut R,
    output: &mut W,
    saves: &Path,
) -> io::Result<()> {
    let root = tree::root(path).map_err(io::Error::other)?;
    loop {
        // Recheck the root in case an external process replaced it during this session.
        if let Err(message) = tree::root(&root) {
            error(output, &message)?;
            return Ok(());
        }
        let inspection = tree::inspect(&root);
        writeln!(
            output,
            "\nAdventure folder: {}\n{}",
            root.display(),
            inspection.tree
        )?;
        if !inspection.problems.is_empty() {
            writeln!(
                output,
                "Some paths need attention; choose Validate for details."
            )?;
        }
        writeln!(output, "========================================\n          ADVENTURE CREATOR\n========================================\n0. Exit Creator\n1. Play Your Adventure\n2. Validate Your Adventure\n3. Configure Adventure Settings [Phase 2E - not available yet]\n4. Create or Edit Items [Phase 2D - not available yet]\n5. Create or Edit Spells [Phase 2D - not available yet]\n6. Create or Edit Artwork [Phase 2D - not available yet]\n7. Create or Edit Rooms [Phase 2C - not available yet]\n8. Help\nChoose an option (Enter refreshes the tree):")?;
        let Some(choice) = answer(input, output, "> ")? else {
            writeln!(
                output,
                "End of input. Leaving the creator; no automatic save."
            )?;
            return Ok(());
        };
        match choice.to_ascii_lowercase().as_str() {
            "0" | "back" | "cancel" | "exit" => { writeln!(output, "Leaving this adventure. No automatic save.")?; return Ok(()); }
            "" => (),
            "1" | "2" => {
                if let Err(message) = tree::root(&root) { error(output, &message)?; continue; }
                let inspection = tree::inspect(&root);
                // This inspection is a safety guard, not a replacement for schema validation.
                if !inspection.problems.is_empty() { error(output, &inspection.problems.join("\n"))?; continue; }
                let adventure = match Adventure::load(&root) {
                    Ok(adventure) => adventure,
                    Err(message) => { error(output, &message)?; continue; }
                };
                if choice == "2" {
                    writeln!(output, "\nVALIDATING YOUR ADVENTURE")?;
                    adventure.summary(output)?;
                    writeln!(output, "Validation successful! Your adventure is ready to play.")?;
                } else {
                    writeln!(output, "\nStarting a fresh play session. Type quit to return to the creator.\nUse save/load explicitly if you want to keep or restore player progress.")?;
                    // Borrow the same buffered reader so it cannot swallow menu input.
                    adventure.play(saves, input, output)?;
                    writeln!(output, "Returned to the creator. Player progress was not automatically saved.")?;
                }
            }
            "3" | "4" | "5" | "6" | "7" => writeln!(output, "That guided editor is planned for the phase shown in the menu.\nNo files were changed. Existing noninteractive add commands and text\nediting remain available; see docs/CREATOR_QUICKSTART.md.")?,
            "8" | "?" | "help" => writeln!(output, "You are working in the adventure folder shown above.\nThe tree shows actual files; trailing / means a directory.\nTOML holds story content; optional config/*.fig holds engine settings.\n1 plays here and quit returns here. Each play starts fresh unless you\nexplicitly load a save. 2 checks all supported content and FIG settings.\nEnter refreshes the tree after external edits. 0/back/cancel leaves this\nadventure; no project files or player saves are automatically written.\nRoom editing is Phase 2C; items/spells/art are 2D; interactions/FIG are 2E.")?,
            _ => writeln!(output, "Please choose 0-8, or type help. Your adventure is still selected.")?,
        }
    }
}
