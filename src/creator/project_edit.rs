//! File-first, multi-file drafts. The engine loader remains the schema authority.
use super::{
    content_edit::confirm,
    room_transaction::{editable_path, Snapshot},
    wizard::Wizard,
};
use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

struct Draft {
    snapshot: Snapshot,
    changes: BTreeMap<PathBuf, Option<String>>,
}
impl Draft {
    fn text(&self, path: &Path) -> Result<String, String> {
        match self.changes.get(path) {
            Some(Some(s)) => Ok(s.clone()),
            Some(None) => Err("This file is deleted in the draft; Undo restores it".into()),
            None => self
                .snapshot
                .source(path)
                .map(|b| String::from_utf8(b.to_vec()).map_err(|e| e.to_string()))
                .unwrap_or(Ok(String::new())),
        }
    }
    fn put(&mut self, path: PathBuf, text: Option<String>) -> Result<(), String> {
        if !editable_path(&path, self.snapshot.source(&path).is_some()) {
            return Err(
                "Choose a supported content TOML, config FIG or art TXT path inside the adventure"
                    .into(),
            );
        }
        if text.as_ref().is_some_and(|s| s.len() > 8 * 1024 * 1024) {
            return Err("File exceeds 8 MiB".into());
        }
        if text.as_ref().map(|s| s.as_bytes()) == self.snapshot.source(&path) {
            self.changes.remove(&path);
        } else {
            self.changes.insert(path, text);
        }
        Ok(())
    }
    fn preview<R: BufRead, W: Write>(&self, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
        for (p, t) in &self.changes {
            w.say(&format!(
                "\n{} {}",
                if t.is_none() {
                    "DELETE"
                } else if self.snapshot.source(p).is_some() {
                    "REPLACE"
                } else {
                    "CREATE"
                },
                p.display()
            ))?;
            if let Some(old) = self.snapshot.source(p) {
                w.say("--- current bytes ---")?;
                w.output.write_all(old).map_err(|e| e.to_string())?;
            }
            w.say("\n--- exact proposed contents ---")?;
            if let Some(t) = t {
                w.output
                    .write_all(t.as_bytes())
                    .map_err(|e| e.to_string())?;
            } else {
                w.say("(file absent)")?;
            }
            w.say("\n--- end ---")?;
        }
        Ok(())
    }
}

fn parts(path: &str) -> Result<Vec<String>, String> {
    if path.is_empty() || path == "/" {
        return Ok(vec![]);
    }
    if !path.starts_with('/') {
        return Err("Use /key/subkey or /array/0; / selects the whole document".into());
    }
    Ok(path[1..]
        .split('/')
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .collect())
}
fn at_mut<'a>(v: &'a mut toml::Value, path: &[String]) -> Result<&'a mut toml::Value, String> {
    if path.is_empty() {
        return Ok(v);
    }
    let child = match v {
        toml::Value::Table(t) => t.get_mut(&path[0]),
        toml::Value::Array(a) => path[0].parse::<usize>().ok().and_then(|n| a.get_mut(n)),
        _ => None,
    }
    .ok_or_else(|| format!("No field at {}", path[0]))?;
    at_mut(child, &path[1..])
}
fn change(v: &mut toml::Value, path: &[String], value: Option<toml::Value>) -> Result<(), String> {
    if path.is_empty() {
        *v = value
            .filter(toml::Value::is_table)
            .ok_or("The document root must remain a table")?;
        return Ok(());
    }
    let parent = at_mut(v, &path[..path.len() - 1])?;
    let key = path.last().unwrap();
    match parent {
        toml::Value::Table(t) => {
            if let Some(v) = value {
                t.insert(key.clone(), v);
            } else if t.remove(key).is_none() {
                return Err("No such field".into());
            }
        }
        toml::Value::Array(a) => {
            let i = if key == "-" {
                a.len()
            } else {
                key.parse::<usize>()
                    .map_err(|_| "Use a zero-based index, or - to append")?
            };
            if i > a.len() {
                return Err("Array index out of range".into());
            }
            match value {
                Some(v) if i == a.len() => a.push(v),
                Some(v) => a[i] = v,
                None if i < a.len() => {
                    a.remove(i);
                }
                None => return Err("No entry to remove".into()),
            }
        }
        _ => return Err("Parent must be a table or array".into()),
    }
    Ok(())
}
fn fields(v: &toml::Value, p: &str, out: &mut Vec<String>) {
    match v {
        toml::Value::Table(t) => {
            for (k, v) in t {
                let p = format!("{p}/{}", k.replace('~', "~0").replace('/', "~1"));
                out.push(format!(
                    "{p} = {}",
                    if v.is_table() {
                        "(table)".into()
                    } else if v.is_array() {
                        "(array)".into()
                    } else {
                        v.to_string()
                    }
                ));
                fields(v, &p, out);
            }
        }
        toml::Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                let p = format!("{p}/{i}");
                out.push(format!(
                    "{p} = {}",
                    if v.is_table() {
                        "(table)".into()
                    } else {
                        v.to_string()
                    }
                ));
                fields(v, &p, out);
            }
        }
        _ => (),
    }
}
fn raw<R: BufRead, W: Write>(w: &mut Wizard<'_, R, W>) -> Result<Option<String>, String> {
    w.say("Paste exact file text. A line /end (or .end) finishes; back keeps this file unchanged; cancel/EOF discards the entire unsaved project draft. Prefix literal control lines with a backslash. Newlines use LF.")?;
    let mut text = String::new();
    loop {
        let line = w.line("... ")?;
        if matches!(line.as_str(), ".end" | "/end") {
            return Ok(Some(text));
        }
        if line.eq_ignore_ascii_case("back") {
            return Ok(None);
        }
        let unescaped = if ["\\.end", "\\/end", "\\back", "\\cancel"]
            .contains(&line.to_ascii_lowercase().as_str())
        {
            &line[1..]
        } else {
            &line
        };
        text.push_str(unescaped);
        text.push('\n');
        if text.len() > 8 * 1024 * 1024 {
            return Err("File exceeds 8 MiB".into());
        }
    }
}
fn edit<R: BufRead, W: Write>(
    draft: &mut Draft,
    path: PathBuf,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        let text = draft.text(&path)?;
        w.heading(&format!("FILE DRAFT: {}", path.display()))?;
        w.say(&text)?;
        w.say("1. Replace exact file text (also repairs syntax)\n2. Set a field / append an array entry\n3. Remove a field or definition\n4. Copy a field or definition\n5. List field paths\n0. Return with draft retained\nStructured edits reformat this file and remove comments. Exact replacement preserves pasted text. Use M for existing ID/weapon-association migrations; raw edits do not update references.")?;
        let c = w.menu("Operation:")?;
        if matches!(c.as_str(), "0" | "back") {
            return Ok(());
        }
        let result = (|| -> Result<(), String> {
            if c == "1" {
                if let Some(text) = raw(w)? {
                    draft.put(path.clone(), Some(text))?;
                }
                return Ok(());
            }
            let mut value: toml::Value = toml::from_str(&text)
                .map_err(|e| format!("Repair syntax with option 1 first: {e}"))?;
            if c == "5" {
                let mut list = vec![];
                fields(&value, "", &mut list);
                return w.say(&list.join("\n"));
            }
            if !matches!(c.as_str(), "2" | "3" | "4") {
                return Err("Choose 0-5".into());
            }
            let pointer =
                w.line("Field path (/key, /array/0, /array/- to append; back returns):")?;
            if pointer.eq_ignore_ascii_case("back") {
                return Ok(());
            }
            let p = parts(&pointer)?;
            match c.as_str() {
                "2" => {
                    let literal = w.line("TOML value (quote text; arrays/tables accepted):")?;
                    if literal.eq_ignore_ascii_case("back") {
                        return Ok(());
                    }
                    let parsed: toml::Table =
                        toml::from_str(&format!("value = {literal}")).map_err(|e| e.to_string())?;
                    if parsed.len() != 1 {
                        return Err("Enter one TOML value".into());
                    }
                    change(&mut value, &p, parsed.get("value").cloned())?;
                }
                "3" => change(&mut value, &p, None)?,
                "4" => {
                    let mut copy = at_mut(&mut value, &p)?.clone();
                    let dest =
                        parts(&w.line("New destination path (must not exist; - appends):")?)?;
                    if at_mut(&mut value, &dest).is_ok() {
                        return Err("Destination already exists".into());
                    }
                    if let Some(t) = copy.as_table_mut() {
                        if t.contains_key("id") {
                            let id = w.line("New stable ID for the copied definition:")?;
                            crate::creator::validate_id(&id)?;
                            t.insert("id".into(), toml::Value::String(id));
                        }
                    }
                    change(&mut value, &dest, Some(copy))?;
                }
                _ => unreachable!(),
            }
            draft.put(
                path.clone(),
                Some(toml::to_string_pretty(&value).map_err(|e| e.to_string())?),
            )
        })();
        if let Err(e) = result {
            if e.starts_with("Setup canceled") {
                return Err(e);
            }
            w.say(&e)?;
        }
    }
}

pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    run_at(root, None, input, output)
}
pub(super) fn run_at<R: BufRead, W: Write>(
    root: &Path,
    selected: Option<PathBuf>,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    let result = (|| -> Result<(), String> {
        let mut draft = Draft {
            snapshot: Snapshot::capture(root)?,
            changes: BTreeMap::new(),
        };
        if let Some(path) = selected {
            edit(&mut draft, path, &mut w)?;
        }
        loop {
            w.heading(&format!(
                "PROJECT FILES — {} unsaved file(s)",
                draft.changes.len()
            ))?;
            for (p, _) in draft.snapshot.all_sources().take(40) {
                w.say(&format!(
                    "{}{}",
                    if draft.changes.contains_key(p) {
                        "* "
                    } else {
                        "  "
                    },
                    p.display()
                ))?;
            }
            for (p, t) in &draft.changes {
                if draft.snapshot.source(p).is_none() && t.is_some() {
                    w.say(&format!("+ {}", p.display()))?;
                }
            }
            if draft.snapshot.all_sources().count() > 40 {
                w.say("Showing the first 40 paths; L searches every file. P previews all unsaved changes.")?;
            }
            w.say("L. Browse/search files\n1. Edit/create file\n2. Duplicate definition file\n3. Delete file\n4. Undo one file\nP. Preview every change\nV. Validate draft\nS. Validate, preview and publish\nH. Schema help\n0. Discard all unsaved changes and return")?;
            let c = w.menu("Operation:")?.to_ascii_lowercase();
            if matches!(c.as_str(), "0" | "back") {
                return Ok(());
            }
            let result = (|| -> Result<(), String> {
                match c.as_str(){
                "l"=>{
                    let mut paths=draft.snapshot.all_sources().map(|(p,_)|p.clone()).chain(draft.changes.keys().cloned()).collect::<Vec<_>>();paths.sort();paths.dedup();
                    let list=paths.into_iter().filter(|p|!matches!(draft.changes.get(p),Some(None))).map(|p|{let id=p.to_string_lossy().replace('\\', "/");let name=draft.text(&p).ok().and_then(|s|s.parse::<toml::Value>().ok()).and_then(|v|v.get("name").or_else(||v.get("title")).and_then(toml::Value::as_str).map(str::to_owned)).unwrap_or_else(||id.clone());(id,name)}).collect::<Vec<_>>();
                    if let Some(path)=super::content_edit::choose(&mut w,"PROJECT FILES",&list)?{edit(&mut draft,path.into(),&mut w)?;}
                },
                "p"=>draft.preview(&mut w)?,
                "v"=>{draft.snapshot.validate_changes(&draft.changes)?;if draft.changes.is_empty(){crate::adventure::Adventure::load(root)?;}w.say("Draft is valid. Nothing published.")?;},
                "s"=>{if draft.changes.is_empty(){w.say("No unsaved changes.")?;}else{draft.snapshot.validate_changes(&draft.changes)?;draft.preview(&mut w)?;if confirm(&mut w,"Publish all displayed changes, including deletions?")?{draft.snapshot.commit_changes(&draft.changes)?;w.say("Published and validated. Player saves unchanged.")?;draft=Draft{snapshot:Snapshot::capture(root)?,changes:BTreeMap::new()};}}},
                "h"=>w.say("All implemented schemas are editable here, including nested conditions/effects and arrays. game.toml: metadata/currencies; rules.toml: flags/counters; actions.toml: commands; events.toml: hooks/timers/quests/journal/endings; NPCs/*.toml: one NPC with its nodes/choices; systems.toml: equipment/statuses. rooms/items/spells/enemies/weapons/shops hold individual definitions. config/*.fig: character, terminology, combat. See docs/CONTENT_REFERENCE.md and docs/PROJECT_EDITOR.md; bundled samples provide complete examples. Edits may temporarily have broken references. Fix every affected file, then validate/publish as one transaction. RORK files cannot be edited here. Duplicate creates a new ID without redirecting references; M handles deliberate migrations.")?,
                "1"|"2"|"3"|"4"=>{let line=w.line("Relative content file path (back returns):")?;if line.eq_ignore_ascii_case("back"){return Ok(());}let p=PathBuf::from(line);
                    if !editable_path(&p,draft.snapshot.source(&p).is_some()){return Err("Unsupported or unsafe path".into());}
                    match c.as_str(){
                        "1"=>edit(&mut draft,p,&mut w)?,
                        "2"=>{let mut v:toml::Value=toml::from_str(&draft.text(&p)?).map_err(|e|e.to_string())?;if !matches!(p.parent().and_then(Path::to_str),Some("rooms"|"items"|"spells"|"NPCs"|"enemies"|"shops")){return Err("Duplicate an individual definition here; copy nested definitions with Edit > Copy field".into());}
                            let id=w.line("New stable ID:")?;super::validate_id(&id)?;v["id"]=toml::Value::String(id.clone());let dest=p.parent().unwrap().join(format!("{id}.toml"));
                            if draft.snapshot.source(&dest).is_some()||draft.changes.contains_key(&dest){return Err("Destination already exists".into());}
                            draft.put(dest,Some(toml::to_string_pretty(&v).map_err(|e|e.to_string())?))?;w.say("Copy staged. References still point to the original. Review associations and validate.")?;},
                        "3"=>{w.say("Review typed ID occurrences before deleting (definitions and references):")?;
                            match super::migration::references(&draft.snapshot){Ok(lines)=>w.say(&lines.join("\n"))?,Err(e)=>w.say(&format!("Reference inspection unavailable until syntax is repaired: {e}"))?,}
                            draft.put(p,None)?;w.say("Deletion staged. Validation will reject remaining references; edit those files before publication.")?;},
                        "4"=>{draft.changes.remove(&p);},_=>unreachable!(),
                    }
                },
                _=>w.say("Choose a displayed operation.")?,
            }
                Ok(())
            })();
            if let Err(e) = result {
                if e.starts_with("Setup canceled") {
                    return Err(e);
                }
                w.say(&e)?;
            }
        }
    })();
    if let Err(e) = result {
        w.say(&format!("Project editor stopped: {e}"))
            .map_err(io::Error::other)?;
    }
    Ok(())
}
