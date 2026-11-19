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
    pub fn status<R: BufRead, W: Write>(&self, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
        w.say(&format!(
            "{} | {}",
            self.path.display(),
            if self.changed() {
                "UNSAVED DRAFT — publication requires confirmation"
            } else {
                "No unsaved changes"
            }
        ))
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
        "yes approves the described action; no/back declines it",
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
    choose_with_actions(w, label, entries, &[])
}

/// Numbers retain their displayed meaning. Exact IDs precede unique names.
pub(super) fn selection_index(answer: &str, entries: &[(String, String)]) -> Result<usize, String> {
    if let Ok(n) = answer.parse::<usize>() {
        return n
            .checked_sub(1)
            .filter(|i| *i < entries.len())
            .ok_or_else(|| {
                "Choose a displayed number, exact ID/name, or 0/back to return.".into()
            });
    }
    let ids = entries
        .iter()
        .enumerate()
        .filter(|(_, (id, _))| id == answer)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let matches = if ids.is_empty() {
        entries
            .iter()
            .enumerate()
            .filter(|(_, (_, name))| name.eq_ignore_ascii_case(answer))
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    } else {
        ids
    };
    match matches.as_slice() {
        [i] => Ok(*i),
        [] => Err("No matching choice. Use a displayed number, exact ID/name, or 0/back to return. Blank input keeps this menu open.".into()),
        _ => Err("That name is ambiguous. Choose its displayed number or exact internal ID.".into()),
    }
}
pub(super) fn choose_with_actions<R: BufRead, W: Write>(
    w: &mut Wizard<'_, R, W>,
    label: &str,
    entries: &[(String, String)],
    actions: &[(&str, &str)],
) -> Result<Option<String>, String> {
    let mut query = String::new();
    let mut page = 0usize;
    loop {
        if !label.is_empty() {
            w.heading(label.trim_end_matches(':'))?;
        }
        let matching = entries
            .iter()
            .enumerate()
            .filter(|(_, (id, name))| {
                id.to_lowercase().contains(&query) || name.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let pages = matching.len().div_ceil(20).max(1);
        page = page.min(pages - 1);
        for &i in matching.iter().skip(page * 20).take(20) {
            let (id, name) = &entries[i];
            w.say(&format!("  {}. {} ({})", i + 1, name, id))?;
        }
        if entries.is_empty() {
            w.say("No eligible definitions yet.")?;
        } else if matching.is_empty() {
            w.say("No matches. / clears the filter.")?;
        }
        w.say(&format!("{} matches; page {}/{}. /text filters ID/name; / clears; > next; < previous. Numbers stay fixed across pages.",matching.len(),page+1,pages))?;
        for (key, description) in actions {
            w.say(&format!("  {key}. {description}"))?;
        }
        w.say("  0. Cancel selection / return\nChoose a number, exact ID or unambiguous name. ?/help explains. Blank input keeps this menu open.")?;
        let answer = w.menu("Selection:")?;
        let answer = answer.trim();
        if answer == "0" || answer.eq_ignore_ascii_case("back") {
            w.say("Selection canceled; this field is unchanged.")?;
            return Ok(None);
        }
        if let Some(text) = answer.strip_prefix('/') {
            query = text.to_lowercase();
            page = 0;
            continue;
        }
        if answer == ">" {
            page = (page + 1).min(pages - 1);
            continue;
        }
        if answer == "<" {
            page = page.saturating_sub(1);
            continue;
        }
        if let Some((key, _)) = actions
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(answer))
        {
            return Ok(Some((*key).into()));
        }
        match selection_index(answer, entries) {
            Ok(i) => {
                let (id, name) = &entries[i];
                w.say(&format!("Selected: {name} ({id})."))?;
                return Ok(Some(id.clone()));
            }
            Err(e) => w.say(&e)?,
        }
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
    let adventure = Adventure::load(&snapshot.root)?;
    let teaching = adventure
        .rpg
        .abilities
        .values()
        .filter(|a| a.learn_from_item.as_deref() == Some(d.text("id")))
        .map(|a| a.name.clone())
        .collect::<Vec<_>>();
    let weapon = adventure.rpg.weapons.contains_key(d.text("id"));
    if weapon {
        w.say("This item has a weapon definition and must stay portable so players can carry it.")?;
    }
    if !teaching.is_empty() {
        w.say(&format!("This item teaches: {}. It must remain portable so players can carry and learn from it.", teaching.join(", ")))?;
    }
    loop {
        w.heading("ITEM OPTIONS")?;
        d.status(w)?;
        w.say(&format!("{} ({})\n1. Display name\n2. Description\n3. Aliases\n4. Portable\n5. Starting room\n6. Inspect / preview TOML\n7. Conditional descriptions\nS. Save\n0. Discard and return", d.text("name"), d.text("id")))?;
        match w.menu("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0" | "back" => { w.say("Draft discarded; no draft changes saved.")?; return Ok(false); },
            "1" => if let Some(s) = w.ask("Display name", d.text("name"), "The name shown to players; the stable ID does not change", |s| super::validate_text(s,120))? { d.set("name", value(s)); },
            "2" => { w.say("Describe what players see when examining this object.")?; if let Some(s) = w.description_labeled("Description", d.text("description"))? { d.set("description", value(s)); } },
            "3" => aliases(d,w)?,
            "4" => {
                w.say("Portable objects can be picked up and carried. A teaching scroll must be carried to learn from it. Fixed objects stay in the room.")?;
                let default = if d.doc["portable"].as_bool() == Some(true) { "yes" } else { "no" };
                if let Some(s) = w.ask("Portable?", default, "yes permits taking; no makes a fixed object", |s| if ["yes","no"].contains(&s) { Ok(()) } else { Err("Enter yes or no".into()) })? { if s=="no" && (!teaching.is_empty() || weapon) { w.say(if weapon {"Cannot make a weapon item fixed. Players must carry it to attack."} else {"Cannot make a teaching item fixed. Remove or change its teaching link in Spell Manager first."})?; } else { d.set("portable", value(s=="yes")); } }
            },
            "5" => { w.say("Starting room affects new play sessions; existing player saves are not rewritten.")?; if let Some(id) = choose(w,"Starting room:",&entries(snapshot,"rooms")?)? { d.doc.remove("initial"); d.set("starting_room",value(id)); } },
            "6" => d.preview(&snapshot.root,w.output)?,
            "7" => super::room_advanced::descriptions(&mut d.doc, snapshot, w)?,
            "s" => if save(snapshot, d, w)? { return Ok(true); },
            "?" | "help" => w.say("Select a field to revise it, 6 to inspect all properties, S to review/save, or 0 to discard. IDs remain stable so weapons, scrolls, shops and puzzles keep their references.")?,
            _ => w.say("Choose 1-7, S, or 0.")?,
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
        w.say("Save declined; draft retained for revision. No files changed.")?;
        return Ok(false);
    }
    if let Err(error) = snapshot.commit(&BTreeMap::from([(d.path.clone(), d.contents())])) {
        w.say(&format!("Save needs attention: {error}\nYour draft remains open. Correct proposed values and retry; for external changes or recovery conflicts, discard and reopen after review."))?;
        return Ok(false);
    }
    w.say(&format!("Saved file: {}", d.path.display()))?;
    w.say("Changes saved. Validation PASSED. Select the content again to review, or return to the creator to playtest.")?;
    Ok(true)
}
pub(super) fn manage<R: BufRead, W: Write>(
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
        let mut actions = vec![("n", "Create new definition")];
        if category == "shops" {
            actions.extend([
                ("i", "Prerequisite items (separate save)"),
                ("t", "Prerequisite spells (separate save)"),
                ("c", "Currencies (separate save)"),
            ]);
        }
        let Some(choice) = choose_with_actions(
            w,
            &format!("{} MANAGER", category.to_ascii_uppercase()),
            &list,
            &actions,
        )?
        else {
            return Ok(());
        };
        if category == "shops" {
            match choice.to_ascii_lowercase().as_str() {
                "i" => {
                    manage(root, "items", w)?;
                    continue;
                }
                "t" => {
                    manage(root, "spells", w)?;
                    continue;
                }
                "c" => {
                    super::currency_edit::run(root, w.input, w.output)
                        .map_err(|e| e.to_string())?;
                    continue;
                }
                _ => (),
            }
        }
        let mut d = if choice.eq_ignore_ascii_case("n") {
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
            if category == "shops" {
                if !super::shop_edit::defaults(&mut d, &snapshot, w)? {
                    continue;
                }
            } else if category == "enemies" {
                super::enemy_edit::defaults(&mut d);
            } else if category == "spells" {
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
        } else if let Some(path) = selection_index(&choice, &list)
            .ok()
            .and_then(|n| paths.get(n))
        {
            let i = selection_index(&choice, &list)?;
            w.say(&format!("Selected: {} ({}).", list[i].1, list[i].0))?;
            Document::load(&snapshot, path.clone(), "")?
        } else {
            w.say(&selection_index(&choice, &list).unwrap_err())?;
            continue;
        };
        let result = if category == "shops" {
            super::shop_edit::form(&mut d, &snapshot, w)
        } else if category == "enemies" {
            super::enemy_edit::form(&mut d, &snapshot, w)
        } else if category == "spells" {
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
    w.say("Defining an ability does not teach it. Link a portable teaching item or use a TeachAbility effect. World abilities with zero MP work without RPG. Use Project Files (F) for world targets, conditions/effects, damage types and on-hit statuses. Other ability kinds and positive MP costs require RPG.").map_err(io::Error::other)?;
    if let Err(e) = manage(root, "spells", &mut w) {
        writeln!(
            w.output,
            "Spell manager stopped: {}",
            e.replace("Setup canceled", "Draft canceled")
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod navigation_tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn search_pages_keep_global_numbers_and_never_auto_select() {
        let entries = (1..=45)
            .map(|i| (format!("id_{i:02}"), format!("Entry {i}")))
            .collect::<Vec<_>>();
        let mut output = Vec::new();
        let mut input = Cursor::new(">\n/id_41\n\n41\n");
        let selected = choose(
            &mut Wizard {
                input: &mut input,
                output: &mut output,
            },
            "Definitions",
            &entries,
        )
        .unwrap();
        assert_eq!(selected.as_deref(), Some("id_41"));
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("page 2/3"));
        assert!(text.contains("1 matches; page 1/1"));
        assert!(text.contains("41. Entry 41 (id_41)"));
        let mut out = Vec::new();
        assert!(choose(
            &mut Wizard {
                input: &mut Cursor::new("/missing\n0\n"),
                output: &mut out
            },
            "Definitions",
            &entries
        )
        .unwrap()
        .is_none());
        assert!(String::from_utf8(out).unwrap().contains("No matches"));
    }
    #[test]
    fn filtered_duplicate_names_still_need_explicit_selection() {
        let entries = vec![("a".into(), "Door".into()), ("b".into(), "Door".into())];
        let mut output = Vec::new();
        let result = choose(
            &mut Wizard {
                input: &mut Cursor::new("/door\nDoor\nb\n"),
                output: &mut output,
            },
            "Select",
            &entries,
        )
        .unwrap();
        assert_eq!(result.as_deref(), Some("b"));
        assert!(String::from_utf8(output).unwrap().contains("ambiguous"));
    }
}
