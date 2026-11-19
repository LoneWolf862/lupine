use super::{
    content_edit::{choose, confirm, entries, save, Document},
    room_transaction::Snapshot,
    rules_ui, tree,
    wizard::{DraftFile, Wizard},
};
use crate::adventure::Adventure;
use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};
use toml_edit::{value, Table};

fn art_entries(snapshot: &Snapshot) -> Vec<(String, String)> {
    snapshot
        .art_sources()
        .map(|(p, _)| {
            let s = p
                .strip_prefix("art")
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            (s.clone(), s)
        })
        .collect()
}
fn read_art<R: BufRead, W: Write>(w: &mut Wizard<'_, R, W>) -> Result<Option<String>, String> {
    w.say("ASCII art is a picture made from text. Type/paste lines, including spaces and blank lines. /end finishes; back discards; cancel leaves this draft. Escape a literal control line with one backslash. Other backslashes and whitespace are preserved.")?;
    let mut result = String::new();
    let mut rejected = false;
    loop {
        let line = w.line("... ")?;
        if line == "/end" {
            if rejected && result.is_empty() {
                w.say("No valid artwork entered; previous artwork retained.")?;
                return Ok(None);
            }
            return Ok(Some(result));
        }
        if line.eq_ignore_ascii_case("back") {
            return Ok(None);
        }
        let line = if ["\\/end", "\\back", "\\cancel"].contains(&line.to_ascii_lowercase().as_str())
        {
            &line[1..]
        } else {
            &line
        };
        let previous_len = result.len();
        result.push_str(line);
        result.push('\n');
        if let Err(e) = crate::story_loader::validate_art(&result, "Artwork draft") {
            result.truncate(previous_len);
            rejected = true;
            w.say(&format!("{e}\nThat line was rejected; earlier lines are retained. Enter a corrected line, /end to finish accepted lines, or back to leave the artwork unchanged."))?;
        }
    }
}
fn room<R: BufRead, W: Write>(snapshot: &Snapshot, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
    let Some(id) = choose(w, "Room to illustrate:", &entries(snapshot, "rooms")?)? else {
        return Ok(());
    };
    let path = snapshot
        .room_sources()
        .find_map(|(p, b)| {
            let doc = std::str::from_utf8(b).ok()?.parse::<toml::Value>().ok()?;
            (doc["id"].as_str() == Some(&id)).then(|| p.clone())
        })
        .ok_or("Room disappeared")?;
    let mut d = Document::load(snapshot, path, "")?;
    loop {
        w.heading("ROOM ARTWORK")?;
        w.say("1. Set default artwork\n2. Remove default reference\n3. Add conditional variant\n4. Remove variant\n5. Inspect/preview\n6. Edit a variant\n7. Move a variant earlier\nS. Save\n0. Discard and return\nVariants use first-match order, then fall back to default artwork.")?;
        match w.menu("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0"|"back"=>return Ok(()),
            "1"=>if let Some(file)=choose(w,"Artwork file:",&art_entries(snapshot))? {
                if d.doc.contains_key("art") && !confirm(w,"Replace existing inline artwork with this file reference?")? {continue;}
                d.doc.remove("art"); d.set("art_file",value(file));
            },
            "2"=>if confirm(w,"Remove default artwork (variants remain)?")? {d.doc.remove("art");d.doc.remove("art_file");},
            "3"=>{
                let Some(file)=choose(w,"Artwork for this condition:",&art_entries(snapshot))? else {continue;};
                let Some(when)=rules_ui::conditions(snapshot,w)? else {continue;};
                if when.is_empty(){w.say("A variant needs at least one condition; use default artwork for unconditional art.")?;continue;}
                let mut t=Table::new();t["when"]=value(when);t["file"]=value(file);rules_ui::append(&mut d.doc,"art_variants",t)?;
            },
            "4"=>{
                if let Some(i)=rules_ui::select_entry(&d.doc,"art_variants",w)? {
                    if confirm(w,"Remove this variant?")? {rules_ui::remove(&mut d.doc,"art_variants",i)?;}
                }
            },
            "5"=>d.preview(&snapshot.root,w.output)?,
            "6"=>if let Some(i)=rules_ui::select_entry(&d.doc,"art_variants",w)? {
                let t=rules_ui::entry_mut(&mut d.doc,"art_variants",i)?;
                w.say("1. Choose artwork file\n2. Edit inline artwork\n3. Edit conditions\n0. Return")?;
                match w.menu("Variant field:")?.trim(){
                    "1"=>if let Some(file)=choose(w,"Artwork:",&art_entries(snapshot))? {
                        if t.get("text").is_none() || confirm(w,"Replace this variant's inline text with a file?")? {t.remove("text");rules_ui::set(t,"file",value(file));}
                    },
                    "2"=>if let Some(text)=read_art(w)? {
                        if t.get("file").is_none() || confirm(w,"Replace this variant's file reference with inline artwork?")? {t.remove("file");rules_ui::set(t,"text",value(text));}
                    },
                    "3"=>if let Some(a)=rules_ui::edit_conditions(t.get("when").unwrap_or(&toml_edit::Item::None),snapshot,w)? {
                        if a.is_empty(){w.say("Keep at least one condition for a variant.")?;}else{rules_ui::set(t,"when",value(a));}
                    },
                    _=>(),
                }
            },
            "7"=>if let Some(i)=rules_ui::select_entry(&d.doc,"art_variants",w)? {
                if i==0 {w.say("That variant is already first.")?;}else{rules_ui::move_earlier(&mut d.doc,"art_variants",i)?;}
            },
            "s"=>if save(snapshot,&d,w)? {return Ok(());},
            _=>w.say("Choose 1-7, S, or 0. Conditions change pictures as flags/inventory change; they are not saved as artwork in player saves.")?,
        }
    }
}
fn edit<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    path: PathBuf,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    let original = snapshot
        .source(&path)
        .map(|b| String::from_utf8(b.to_vec()))
        .transpose()
        .map_err(|e| e.to_string())?;
    let mut text = original.clone().unwrap_or_default();
    crate::story_loader::validate_art(&text, "Existing artwork")?;
    loop {
        w.heading("ARTWORK OPTIONS")?;
        w.say("1. Replace text (multiline)\n2. Preview\nS. Save\n0. Discard and return")?;
        match w.menu("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0"|"back"=>return Ok(()),
            "1"=>if let Some(s)=read_art(w)?{text=s;},
            "2"=>DraftFile::text(&path.to_string_lossy(),&text,"Artwork preview; whitespace is significant.").preview(&snapshot.root,w.output)?,
            "s"=>{
                if original.is_none() && text.is_empty() { w.say("Enter artwork before saving a new file; nothing changed.")?; continue; }
                if original.as_ref()==Some(&text){w.say("No changes to save.")?;continue;}
                DraftFile::text(&path.to_string_lossy(),&text,"Saving edits the authored artwork, not player progress.").preview(&snapshot.root,w.output)?;
                if confirm(w,"Save artwork?")? {
                    snapshot.commit(&BTreeMap::from([(path.clone(),text.clone())]))?;
                    w.say("Artwork saved and validated. Choose R in Artwork Manager to attach it to a room.")?; return Ok(());
                }
            },
            _=>w.say("Choose 1, 2, S, or 0. /end finishes multiline entry; spaces and backslashes are kept.")?,
        }
    }
}
fn manage<R: BufRead, W: Write>(root: &Path, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
    loop {
        let snapshot = Snapshot::capture(root)?;
        Adventure::load(root)?;
        w.heading("ARTWORK MANAGER")?;
        let paths = snapshot
            .art_sources()
            .map(|(p, _)| p.clone())
            .collect::<Vec<_>>();
        for (i, p) in paths.iter().enumerate() {
            w.say(&format!("{}. {}", i + 1, p.display()))?;
        }
        w.say(
            "N. Create artwork\nR. Attach/change room artwork and variants\n0. Return to creator",
        )?;
        let choice = w
            .menu("Selection (number, exact artwork filename, N, R, or 0):")?
            .trim()
            .to_owned();
        if matches!(choice.as_str(), "0" | "back") {
            return Ok(());
        }
        let result = if choice.eq_ignore_ascii_case("r") {
            room(&snapshot, w)
        } else {
            let path = if choice.eq_ignore_ascii_case("n") {
                let Some(id) = w.ask(
                    "Artwork name (without .txt)",
                    "",
                    "A simple lowercase ID; the file will be under art/",
                    super::validate_id,
                )?
                else {
                    continue;
                };
                let p = PathBuf::from(format!("art/{id}.txt"));
                if snapshot.source(&p).is_some() {
                    w.say("That artwork exists; select it by number to edit.")?;
                    continue;
                }
                p
            } else if let Some(p) =
                super::content_edit::selection_index(&choice, &art_entries(&snapshot))
                    .ok()
                    .and_then(|n| paths.get(n))
            {
                p.clone()
            } else {
                w.say("Choose a listed artwork, N, R or 0.")?;
                continue;
            };
            edit(&snapshot, path, w)
        };
        if let Err(e) = result {
            w.say(&format!(
                "Artwork operation stopped: {}",
                e.replace("Setup canceled", "Draft canceled")
            ))?;
        }
        w.say(&tree::inspect(root).tree)?;
    }
}
pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    if let Err(e) = manage(root, &mut w) {
        w.say(&format!(
            "Artwork manager stopped: {}",
            e.replace("Setup canceled", "Draft canceled")
        ))
        .map_err(io::Error::other)?;
    }
    Ok(())
}
