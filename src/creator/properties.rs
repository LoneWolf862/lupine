use super::{
    content_edit::{save, Document},
    room_transaction::Snapshot,
    wizard::Wizard,
};
use std::{
    io::{self, BufRead, Write},
    path::Path,
};
use toml_edit::{value, Item, Table};
pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    let result = (|| -> Result<(), String> {
        let snapshot = Snapshot::capture(root)?;
        let mut d = Document::load(&snapshot, "game.toml".into(), "")?;
        loop {
            w.heading("PROJECT PROPERTIES")?;
            w.say(&format!("Game ID: {} (stable)\nTitle: {}\n1. Title\n2. Author\n3. Description\n4. Game version label\n5. Credits\n6. Game license text\n7. Player instructions\nP. Preview\nS. Save\n0. Discard and return",d.text("id"),d.text("title")))?;
            w.say("Publication metadata does not change save compatibility. No license is assumed. Export includes your exact credits, license and instructions when supplied.")?;
            match w.menu("Property:")?.to_ascii_lowercase().as_str() {
                "0" | "back" => return Ok(()),
                "p" => d.preview(root, w.output)?,
                "s" => {
                    if save(&snapshot, &d, &mut w)? {
                        return Ok(());
                    }
                }
                "1" => {
                    if let Some(s) = w.ask(
                        "Title",
                        d.text("title"),
                        "Displayed game title; keep the internal ID stable",
                        |s| super::validate_text(s, 160),
                    )? {
                        d.set("title", value(s));
                    }
                }
                c => {
                    let key = match c {
                        "2" => "author",
                        "3" => "description",
                        "4" => "version",
                        "5" => "credits",
                        "6" => "license",
                        "7" => "instructions",
                        _ => {
                            w.say("Choose 0-7, P or S.")?;
                            continue;
                        }
                    };
                    let old = d
                        .doc
                        .get("metadata")
                        .and_then(Item::as_table_like)
                        .and_then(|t| t.get(key))
                        .and_then(Item::as_str)
                        .unwrap_or("");
                    if let Some(text) = w.description_labeled(key, old)? {
                        if !d.doc.contains_key("metadata") {
                            d.doc["metadata"] = Item::Table(Table::new());
                        }
                        let table = d.doc["metadata"]
                            .as_table_like_mut()
                            .ok_or("Repair metadata as a TOML table with F first")?;
                        super::rules_ui::set(table, key, value(text));
                    }
                }
            }
        }
    })();
    if let Err(e) = result {
        w.say(&format!("Properties stopped: {e}"))
            .map_err(io::Error::other)?;
    }
    Ok(())
}
