//! Guided content forms sharing the room editor's validated transaction boundary.
use super::{
    room_transaction::Snapshot,
    tree,
    wizard::{DraftFile, Wizard},
};
use crate::adventure::Adventure;
use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};
use toml_edit::{value, Array, DocumentMut, Item};

pub(super) struct Document {
    pub path: PathBuf,
    pub doc: DocumentMut,
    original: Option<String>,
    baseline: String,
}
impl Document {
    pub fn load(snapshot: &Snapshot, path: PathBuf, fallback: &str) -> Result<Self, String> {
        let original = snapshot
            .source(&path)
            .map(|b| String::from_utf8(b.to_vec()))
            .transpose()
            .map_err(|e| e.to_string())?;
        let doc = original
            .as_deref()
            .unwrap_or(fallback)
            .parse::<DocumentMut>()
            .map_err(|e| e.to_string())?;
        let baseline = doc.to_string();
        Ok(Self {
            path,
            doc,
            original,
            baseline,
        })
    }
    pub fn text(&self, key: &str) -> &str {
        self.doc.get(key).and_then(Item::as_str).unwrap_or("")
    }
    pub fn set(&mut self, key: &str, mut new: Item) {
        if let (Some(old), Some(v)) = (
            self.doc.get(key).and_then(Item::as_value),
            new.as_value_mut(),
        ) {
            *v.decor_mut() = old.decor().clone();
        }
        self.doc[key] = new;
    }
    pub fn contents(&self) -> String {
        let text = self.doc.to_string();
        if text == self.baseline {
            if let Some(old) = &self.original {
                return old.clone();
            }
        }
        if self.original.as_ref().is_some_and(|s| s.contains("\r\n")) {
            text.replace("\r\n", "\n").replace('\n', "\r\n")
        } else {
            text
        }
    }
    pub fn changed(&self) -> bool {
        self.original.is_none() || self.doc.to_string() != self.baseline
    }
    pub fn preview(&self, root: &Path, out: &mut impl Write) -> Result<(), String> {
        DraftFile::text(&self.path.to_string_lossy(), &self.contents(), "These are the exact proposed file contents. Stable IDs and unedited properties are retained.").preview(root, out)
    }
}

pub(super) fn confirm<R: BufRead, W: Write>(
    w: &mut Wizard<'_, R, W>,
    label: &str,
) -> Result<bool, String> {
    Ok(w.ask(
        label,
        "no",
        "yes saves this draft; no/back returns without saving",
        |s| {
            if ["yes", "no", "y", "n"].contains(&s.to_ascii_lowercase().as_str()) {
                Ok(())
            } else {
                Err("Enter yes or no".into())
            }
        },
    )?
    .is_some_and(|s| s.eq_ignore_ascii_case("yes") || s.eq_ignore_ascii_case("y")))
}
pub(super) fn choose<R: BufRead, W: Write>(
    w: &mut Wizard<'_, R, W>,
    label: &str,
    entries: &[(String, String)],
) -> Result<Option<String>, String> {
    for (n, (id, name)) in entries.iter().enumerate() {
        w.say(&format!("  {}. {} ({})", n + 1, name, id))?;
    }
    loop {
        let Some(s) = w.ask(
            label,
            "0",
            "Choose a listed number. 0/back returns without changing this field.",
            |_| Ok(()),
        )?
        else {
            return Ok(None);
        };
        if s == "0" {
            return Ok(None);
        }
        if let Some((id, _)) = s
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| entries.get(n))
        {
            return Ok(Some(id.clone()));
        }
        w.say("Choose a listed number, or 0 to return.")?;
    }
}
pub(super) fn entries(
    snapshot: &Snapshot,
    category: &str,
) -> Result<Vec<(String, String)>, String> {
    snapshot
        .sources(category)
        .filter(|(p, _)| p.extension().is_some_and(|e| e == "toml"))
        .map(|(_, b)| {
            let doc = std::str::from_utf8(b)
                .map_err(|e| e.to_string())?
                .parse::<DocumentMut>()
                .map_err(|e| e.to_string())?;
            let id = doc["id"].as_str().ok_or("Missing ID")?.to_owned();
            let name = doc
                .get("name")
                .or_else(|| doc.get("title"))
                .and_then(Item::as_str)
                .unwrap_or(&id)
                .to_owned();
            Ok((id, name))
        })
        .collect()
}
fn aliases<R: BufRead, W: Write>(d: &mut Document, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
    w.say("Aliases are alternate names players can type: Brass Key might have key and brass key. Add one name at a time; removing an alias does not change the item ID.")?;
    loop {
        let list = d
            .doc
            .get("aliases")
            .and_then(Item::as_array)
            .cloned()
            .unwrap_or_default();
        for (i, v) in list.iter().enumerate() {
            w.say(&format!("  {}. {}", i + 1, v.as_str().unwrap_or("")))?;
        }
        match w
            .line("A adds an alias; a number removes that alias; 0/back returns:")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "0" | "back" => return Ok(()),
            "a" => {
                if let Some(s) = w.ask("Alias", "", "A name players use for this object", |s| {
                    super::validate_text(s, 120)
                })? {
                    if list
                        .iter()
                        .any(|v| v.as_str().is_some_and(|old| old.eq_ignore_ascii_case(&s)))
                    {
                        w.say("That alias already exists.")?;
                    } else {
                        let mut a = list;
                        a.push(s);
                        d.set("aliases", value(a));
                    }
                }
            }
            "?" | "help" => {
                w.say("A adds a name, a listed number removes it after confirmation. 0 returns.")?
            }
            s => {
                if let Some(i) = s
                    .parse::<usize>()
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .filter(|i| *i < list.len())
                {
                    if confirm(w, "Remove this alias?")? {
                        let mut a = list;
                        a.remove(i);
                        d.set("aliases", value(a));
                    }
                } else {
                    w.say("Choose A, a listed number, or 0.")?;
                }
            }
        }
    }
}
fn item_form<R: BufRead, W: Write>(
    d: &mut Document,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    loop {
        w.heading("ITEM OPTIONS")?;
        w.say(&format!("{} ({})\n1. Display name\n2. Description\n3. Aliases\n4. Portable\n5. Starting room\n6. Inspect / preview TOML\nS. Save\n0. Discard and return", d.text("name"), d.text("id")))?;
        match w.line("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0" | "back" => return Ok(false),
            "1" => if let Some(s) = w.ask("Display name", d.text("name"), "The name shown to players; the stable ID does not change", |s| super::validate_text(s,120))? { d.set("name", value(s)); },
            "2" => { w.say("Describe what players see when examining this object.")?; if let Some(s) = w.description(d.text("description"))? { d.set("description", value(s)); } },
            "3" => aliases(d,w)?,
            "4" => {
                w.say("Portable objects can be picked up and carried. A teaching scroll must be carried to learn from it. Fixed objects stay in the room.")?;
                let default = if d.doc["portable"].as_bool() == Some(true) { "yes" } else { "no" };
                if let Some(s) = w.ask("Portable?", default, "yes permits taking; no makes a fixed object", |s| if ["yes","no"].contains(&s) { Ok(()) } else { Err("Enter yes or no".into()) })? { d.set("portable", value(s=="yes")); }
            },
            "5" => { w.say("Starting room affects new play sessions; existing player saves are not rewritten.")?; if let Some(id) = choose(w,"Starting room:",&entries(snapshot,"rooms")?)? { d.set("starting_room",value(id)); } },
            "6" => d.preview(&snapshot.root,w.output)?,
            "s" => if save(snapshot, d, w)? { return Ok(true); },
            "?" | "help" => w.say("Select a field to revise it, 6 to inspect all properties, S to review/save, or 0 to discard. IDs remain stable so weapons, scrolls, shops and puzzles keep their references.")?,
            _ => w.say("Choose 1-6, S, or 0.")?,
        }
    }
}
pub(super) fn save<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    d: &Document,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    if !d.changed() {
        w.say("No changes to save.")?;
        return Ok(false);
    }
    d.preview(&snapshot.root, w.output)?;
    if !confirm(w, "Save these changes after full adventure validation?")? {
        return Ok(false);
    }
    if let Err(error) = snapshot.commit(&BTreeMap::from([(d.path.clone(), d.contents())])) {
        w.say(&format!("Save needs attention: {error}\nYour draft remains open. Correct proposed values and retry; for external changes or recovery conflicts, discard and reopen after review."))?;
        return Ok(false);
    }
    w.say("Changes saved. Validation PASSED. Select the content again to review, or return to the creator to playtest.")?;
    Ok(true)
}
fn manage<R: BufRead, W: Write>(
    root: &Path,
    category: &str,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        let snapshot = Snapshot::capture(root)?;
        Adventure::load(root)
            .map_err(|e| format!("Correct this existing project error before editing: {e}"))?;
        let paths = snapshot
            .sources(category)
            .filter(|(p, _)| p.extension().is_some_and(|e| e == "toml"))
            .map(|(p, _)| p.clone())
            .collect::<Vec<_>>();
        let list = entries(&snapshot, category)?;
        w.heading(&format!("{} MANAGER", category.to_ascii_uppercase()))?;
        for (i, (id, name)) in list.iter().enumerate() {
            w.say(&format!("{}. {name} ({id})", i + 1))?;
        }
        w.say("N. Create new definition\n0. Return to creator")?;
        let choice = w.line("Selection:")?.trim().to_ascii_lowercase();
        if matches!(choice.as_str(), "0" | "back") {
            return Ok(());
        }
        let mut d = if choice == "n" {
            w.say("Give this definition a stable internal ID; player-facing names can change later. The ID connects authored references and player saves.")?;
            let Some(id) = w.ask(
                "Stable ID",
                "",
                "Lowercase letters, digits, underscores or dashes. For example brass_key",
                |s| {
                    super::validate_id(s)?;
                    if list.iter().any(|(id, _)| id == s) {
                        Err("That ID already exists".into())
                    } else {
                        Ok(())
                    }
                },
            )?
            else {
                continue;
            };
            let path = PathBuf::from(format!("{category}/{id}.toml"));
            if snapshot.source(&path).is_some() {
                w.say("That filename already exists. Choose another ID.")?;
                continue;
            }
            let mut d = Document::load(&snapshot, path, "")?;
            d.set("id", value(id.clone()));
            d.set("name", value(id.replace('_', " ")));
            if category == "spells" {
                d.set("description", value("An ability."));
                d.set("mana_cost", value(0));
                d.set("kind", value("heal"));
                d.set("power", value(1));
            } else {
                d.set("description", value("An object."));
                d.set("aliases", value(Array::new()));
                d.set("portable", value(true));
                let Some(room) = choose(
                    w,
                    "Choose its starting room:",
                    &entries(&snapshot, "rooms")?,
                )?
                else {
                    continue;
                };
                d.set("starting_room", value(room));
            }
            d
        } else if let Some(path) = choice
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| paths.get(n))
        {
            Document::load(&snapshot, path.clone(), "")?
        } else {
            w.say("Choose a listed item, N, or 0. Changes are held until confirmed Save.")?;
            continue;
        };
        let result = if category == "spells" {
            super::spell_edit::form(&mut d, &snapshot, w)
        } else {
            item_form(&mut d, &snapshot, w)
        };
        match result {
            Ok(true) => w.say(&tree::inspect(root).tree)?,
            Ok(false) => (),
            Err(e) if e.starts_with("Setup canceled") => {
                w.say("Draft canceled; no draft changes saved.")?;
            }
            Err(e) => w.say(&format!("Edit stopped: {e}"))?,
        }
    }
}
pub(super) fn run_items<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    if let Err(e) = manage(root, "items", &mut w) {
        writeln!(
            w.output,
            "Item manager stopped: {}",
            e.replace("Setup canceled", "Draft canceled")
        )?;
    }
    Ok(())
}

pub(super) fn run_spells<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    match Adventure::load(root) {
        Ok(a) if a.game.config.character.is_none() => {
            w.say("Spells require RPG. Choose Configure Adventure Settings and explicitly enable character RPG first; no setting has been changed.").map_err(io::Error::other)?;
            return Ok(());
        }
        _ => (),
    }
    w.say("Defining a spell does not teach it. Link a portable teaching item, then take it and use learn <item> during play. The item is consumed. Damage targets an enemy; healing and mana restoration target the player. This schema has no spell damage-type setting.").map_err(io::Error::other)?;
    if let Err(e) = manage(root, "spells", &mut w) {
        writeln!(
            w.output,
            "Spell manager stopped: {}",
            e.replace("Setup canceled", "Draft canceled")
        )?;
    }
    Ok(())
}
