use std::{
    io::{self, BufRead, Write},
    path::Path,
};
pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    loop {
        let a = match crate::adventure::Adventure::load(root) {
            Ok(a) => a,
            Err(e) => {
                writeln!(output, "Correct existing project error before editing: {e}")?;
                return Ok(());
            }
        };
        super::ui::heading(output, "RPG AND ECONOMY CONTENT")?;
        writeln!(
            output,
            "1. Manage enemies\n2. Manage weapons\n3. Manage shops\n4. Manage currencies\n5. Configure room encounters\n6. Manage prerequisite items\n7. Manage prerequisite spells\n8. Transaction Bindings\n0. Back"
        )?;
        super::ui::prompt(output, "Selection:")?;
        let mut s = String::new();
        if input.read_line(&mut s)? == 0 {
            return Ok(());
        }
        let choice = s.trim().to_ascii_lowercase();
        if a.game.config.character.is_none() && matches!(choice.as_str(), "1" | "2" | "5") {
            writeln!(output,"Enable RPG explicitly in Configure Adventure Settings first. Shops and currencies work without RPG.")?;
            continue;
        }
        match choice.as_str() {
            "0" | "back" | "cancel" => return Ok(()),
            "1" => super::enemy_edit::run(root, input, output)?,
            "5" => super::room_edit::run(root, input, output)?,
            "6" => super::content_edit::run_items(root, input, output)?,
            "7" => super::content_edit::run_spells(root, input, output)?,
            "8" => super::binding_edit::run(root, input, output)?,
            "4" => super::currency_edit::run(root, input, output)?,
            "2" => super::weapon_edit::run(root, input, output)?,
            "3" => {
                let mut w = super::wizard::Wizard { input, output };
                if let Err(e) = super::content_edit::manage(root, "shops", &mut w) {
                    writeln!(w.output, "Shop manager stopped: {e}")?;
                }
            }
            _ => writeln!(output, "Choose a listed option.")?,
        }
    }
}
