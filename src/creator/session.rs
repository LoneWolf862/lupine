//! Persistent creator navigation. Authored edits use explicit, validated transactions.
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
    super::ui::prompt(output, prompt)?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_owned()))
}
fn error(output: &mut impl Write, detail: &str) -> io::Result<()> {
    writeln!(output, "\nThis adventure is not ready yet:\n{detail}\nNothing was changed. Check the named file and field/reference, correct it\nin your text editor, then choose Validate again. Guided editing requires a valid\nadventure; the creator will reload your files each time.\n")
}

/// Create/open launcher. The same terminal streams survive setup and gameplay.
/// The save directory is passed to the runner only; this function never writes saves.
pub fn start<R: BufRead, W: Write>(input: &mut R, output: &mut W, saves: &Path) -> io::Result<()> {
    loop {
        super::ui::heading(output, "LUPINE ADVENTURE CREATOR")?;
        writeln!(output, "  1. Create a new adventure\n  2. Open an existing adventure\n  0. Exit\n\n? or help explains your choices.")?;
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
                    "" => { writeln!(output, "\nEnter the folder where you saved your adventure (for example, my_adventure),\nor type back to return. Look for the folder containing game.toml.\n")?; continue; }
                    "0" | "back" | "cancel" => break,
                    "?" | "help" => continue,
                    _ => (),
                }
                match tree::root(Path::new(&path)) {
                    Ok(root) => { open(&root, input, output, saves)?; break; }
                    Err(message) => writeln!(output, "\nCannot open: {message}\n")?,
                }
            },
            "?" | "help" => writeln!(output, "Create guides you through a new adventure. Open selects a folder\nyou already made. You can inspect files, validate and play. Choose 7\nto create or edit rooms. Items, spells, artwork, configuration and puzzle editors are available.")?,
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
    writeln!(output, "Creator controls: Enter accepts only a displayed default. Definition lists accept numbers, exact IDs and unique names. ?/help explains. Read each 0/back label: some finish a nested edit; others discard its draft. cancel abandons the current operation and unsaved edits; earlier confirmed saves remain.")?;
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
        super::ui::heading(output, "ADVENTURE CREATOR")?;
        writeln!(output, "  0. Exit Creator\n  1. Play Your Adventure\n  2. Validate Your Adventure\n  3. Configure Adventure Settings\n  4. Create or Edit Items\n  5. Create or Edit Spells\n  6. Create or Edit Artwork\n  7. Create or Edit Rooms\n  8. Help\n  9. Interactions and puzzles\n  R. RPG Content\n  E. Export Your Adventure\n  T. Transaction Bindings\n  M. Rename Internal ID\n\nEnter refreshes the tree; ? gives help.")?;
        let Some(choice) = answer(input, output, "Selection:")? else {
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
                    super::ui::heading(output, "VALIDATING YOUR ADVENTURE")?;
                    adventure.summary(output)?;
                    writeln!(output, "\nValidation successful! Your adventure is ready to play.\n")?;
                } else {
                    writeln!(output, "\nStarting a fresh play session. Type quit to return to the creator.\nUse save/load explicitly if you want to keep or restore player progress.")?;
                    // Borrow the same buffered reader so it cannot swallow menu input.
                    adventure.play(saves, input, output)?;
                    writeln!(output, "Returned to the creator. Player progress was not automatically saved.")?;
                }
            }
            "e" => super::export::run(&root, input, output)?,
            "t" => super::binding_edit::run(&root, input, output)?,
            "m" => super::migration::run(&root, input, output)?,
            "r" => super::rpg_menu::run(&root, input, output)?,
            "9" => super::interaction_edit::run(&root, input, output)?,
            "7" => super::room_edit::run(&root, input, output)?,
            "4" => super::content_edit::run_items(&root, input, output)?,
            "5" => super::content_edit::run_spells(&root, input, output)?,
            "6" => super::art_edit::run(&root, input, output)?,
            "3" => super::fig_edit::run(&root, input, output)?,
            "8" | "?" | "help" => writeln!(output, "You are working in the adventure folder shown above.\nThe tree shows actual files; trailing / means a directory.\nTOML holds story content; optional config/*.fig holds engine settings.\n1 plays here and quit returns here. Each play starts fresh unless you\nexplicitly load a save. 2 checks all supported content and FIG settings.\nEnter refreshes the tree after external edits. 0/back/cancel leaves this\nadventure. Room edits require explicit Save confirmation; player saves remain manual.\n7 opens the Room Manager for guided names, descriptions and connections. 3 configures FIG settings; 4 edits items; 5 edits spells; 6 edits art. 9 opens puzzles and locked doors. Room Options > 7 edits conditional prose/encounters. R opens enemies, weapons, shops, currencies and encounters. Changes remain drafts until explicit confirmation.")?,
            _ => writeln!(output, "\nPlease choose 0-9, R, E, or type help. Your adventure is still selected.\n")?,
        }
    }
}
