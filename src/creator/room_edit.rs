//! Guided room drafts. TOML edits preserve unknown-to-the-editor valid properties and comments.
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
use toml_edit::{value, DocumentMut, Item};
const DIRECTIONS: [&str; 6] = ["north", "south", "east", "west", "up", "down"];
const REVERSE: [&str; 6] = ["south", "north", "west", "east", "down", "up"];

struct Room {
    path: PathBuf,
    original: Option<String>,
    baseline: String,
    doc: DocumentMut,
}
impl Room {
    fn text(&self, key: &str) -> &str {
        self.doc.get(key).and_then(Item::as_str).unwrap_or("")
    }
    fn set(&mut self, key: &str, text: &str) {
        let mut new = value(text);
        if let (Some(old), Some(new)) = (
            self.doc.get(key).and_then(Item::as_value),
            new.as_value_mut(),
        ) {
            *new.decor_mut() = old.decor().clone();
        }
        self.doc[key] = new;
    }
    fn exit(&self, direction: &str) -> Option<&str> {
        self.doc
            .get("exits")
            .and_then(Item::as_table_like)
            .and_then(|t| t.get(direction))
            .and_then(Item::as_str)
    }
    fn set_exit(&mut self, direction: &str, target: &str) -> Result<(), String> {
        if self.doc.get("exits").is_none() {
            self.doc["exits"] = Item::Table(toml_edit::Table::new());
        }
        let table = self.doc["exits"]
            .as_table_like_mut()
            .ok_or("Cannot safely edit this exits representation; no changes saved")?;
        let mut new = value(target);
        if let (Some(old), Some(new)) = (
            table.get(direction).and_then(Item::as_value),
            new.as_value_mut(),
        ) {
            *new.decor_mut() = old.decor().clone();
        }
        table.insert(direction, new);
        Ok(())
    }
}
struct Draft {
    snapshot: Snapshot,
    rooms: BTreeMap<String, Room>,
}
impl Draft {
    fn load(root: &Path) -> Result<Self, String> {
        let snapshot = Snapshot::capture(root)?;
        Adventure::load(root).map_err(|e| format!("Preexisting project error; room editing has not changed anything. Fix this before editing:\n{e}"))?;
        let mut rooms = BTreeMap::new();
        for (path, bytes) in snapshot.room_sources() {
            let text = std::str::from_utf8(bytes)
                .map_err(|e| e.to_string())?
                .to_owned();
            let doc = text
                .parse::<DocumentMut>()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            let id = doc
                .get("id")
                .and_then(Item::as_str)
                .ok_or("Room has no string ID")?
                .to_owned();
            rooms.insert(
                id,
                Room {
                    path: path.clone(),
                    original: Some(text),
                    baseline: doc.to_string(),
                    doc,
                },
            );
        }
        Ok(Self { snapshot, rooms })
    }
    fn changes(&self) -> BTreeMap<PathBuf, String> {
        self.rooms
            .values()
            .filter_map(|r| {
                let text = r.doc.to_string();
                if r.original.is_some() && text == r.baseline {
                    None
                } else {
                    Some((
                        r.path.clone(),
                        if r.original.as_ref().is_some_and(|s| s.contains("\r\n")) {
                            text.replace("\r\n", "\n").replace("\n", "\r\n")
                        } else {
                            text
                        },
                    ))
                }
            })
            .collect()
    }
    fn preview(&self, selected: &str, output: &mut impl Write) -> Result<(), String> {
        let mut changes = self.changes();
        if changes.is_empty() {
            let room = &self.rooms[selected];
            changes.insert(
                room.path.clone(),
                room.original
                    .clone()
                    .unwrap_or_else(|| room.doc.to_string()),
            );
        }
        for (path, text) in changes {
            DraftFile::text(&path.to_string_lossy(), &text, "id connects locations internally; title is the player-facing name; description paints the scene; [exits] maps directions to room IDs. Other authored properties are preserved.").preview(&self.snapshot.root, output)?;
        }
        Ok(())
    }
}

fn menu<R: BufRead, W: Write>(w: &mut Wizard<'_, R, W>, prompt: &str) -> Result<String, String> {
    let line = w.line(prompt)?;
    if matches!(line.trim().to_ascii_lowercase().as_str(), "?" | "help") {
        w.say("Changes stay in memory until Save and explicit confirmation. 0/back returns, cancel abandons this draft. ? explains prompts. Existing IDs cannot be renamed here because exits and saved games refer to them.")?;
        return Ok("help".into());
    }
    Ok(line.trim().to_ascii_lowercase())
}
fn confirm<R: BufRead, W: Write>(w: &mut Wizard<'_, R, W>, prompt: &str) -> Result<bool, String> {
    Ok(w.ask(
        prompt,
        "no",
        "yes approves this change; no/back keeps the current draft unchanged",
        |v| {
            if matches!(v.to_ascii_lowercase().as_str(), "yes" | "y" | "no" | "n") {
                Ok(())
            } else {
                Err("Enter yes or no".into())
            }
        },
    )?
    .is_some_and(|s| s.eq_ignore_ascii_case("yes") || s.eq_ignore_ascii_case("y")))
}
fn choose_room<R: BufRead, W: Write>(
    draft: &Draft,
    w: &mut Wizard<'_, R, W>,
    except: Option<&str>,
) -> Result<Option<String>, String> {
    let ids = draft
        .rooms
        .keys()
        .filter(|id| Some(id.as_str()) != except)
        .cloned()
        .collect::<Vec<_>>();
    for (i, id) in ids.iter().enumerate() {
        w.say(&format!(
            "  {}. {} ({id})",
            i + 1,
            draft.rooms[id].text("title")
        ))?;
    }
    if ids.is_empty() {
        w.say("No other locations yet. Create a second room first.")?;
        return Ok(None);
    }
    loop {
        let choice = menu(w, "Select a location number (0/back cancels connection): ")?;
        if matches!(choice.as_str(), "0" | "back") {
            return Ok(None);
        }
        if let Ok(n) = choice.parse::<usize>() {
            if n > 0 && n <= ids.len() {
                return Ok(Some(ids[n - 1].clone()));
            }
        }
        if choice != "help" {
            w.say("Choose a listed room number.")?;
        }
    }
}
fn direction<R: BufRead, W: Write>(w: &mut Wizard<'_, R, W>) -> Result<Option<usize>, String> {
    w.say("1 north, 2 south, 3 east, 4 west, 5 up, 6 down")?;
    loop {
        let answer = menu(w, "Direction (number or word; 0/back returns): ")?;
        if matches!(answer.as_str(), "0" | "back") {
            return Ok(None);
        }
        if let Some(index) = DIRECTIONS.iter().position(|d| *d == answer) {
            return Ok(Some(index));
        }
        if let Ok(n) = answer.parse::<usize>() {
            if (1..=6).contains(&n) {
                return Ok(Some(n - 1));
            }
        }
        if answer != "help" {
            w.say("Use one of the six listed directions.")?;
        }
    }
}
fn connections<R: BufRead, W: Write>(
    draft: &mut Draft,
    id: &str,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    w.heading("STEP 3 - CONNECT YOUR LOCATION")?;
    w.say("Exits let players travel by typing north, south, east, west, up or down.\nA reverse exit lets them return. One-way exits allow travel only in the shown direction.\nA room without a route from the starting location may be unreachable.")?;
    loop {
        w.say("\n  1. Connect an existing location TO this room\n  2. Connect this room TO another location\n  3. Remove an outgoing connection\n  0. Done / leave connections as they are")?;
        let choice = menu(w, "Connection choice: ")?;
        if matches!(choice.as_str(), "0" | "back") {
            return Ok(());
        }
        if choice == "3" {
            let Some(dir) = direction(w)? else {
                continue;
            };
            let Some(old) = draft.rooms[id].exit(DIRECTIONS[dir]).map(str::to_owned) else {
                w.say("There is no exit in that direction.")?;
                continue;
            };
            if confirm(
                w,
                &format!(
                    "Remove {id} --{}--> {old}? Other exits and conditional rules stay unchanged",
                    DIRECTIONS[dir]
                ),
            )? {
                draft.rooms.get_mut(id).unwrap().doc["exits"]
                    .as_table_like_mut()
                    .unwrap()
                    .remove(DIRECTIONS[dir]);
            }
            continue;
        }
        if !matches!(choice.as_str(), "1" | "2") {
            if choice != "help" {
                w.say("Choose 0, 1, 2 or 3.")?;
            }
            continue;
        }
        let Some(other) = choose_room(draft, w, Some(id))? else {
            continue;
        };
        let (from, to) = if choice == "1" {
            (other.as_str(), id)
        } else {
            (id, other.as_str())
        };
        w.say(&format!(
            "Choose the direction leading FROM {} TO {}.",
            draft.rooms[from].text("title"),
            draft.rooms[to].text("title")
        ))?;
        let Some(dir) = direction(w)? else {
            continue;
        };
        w.heading("CONNECTION PREVIEW")?;
        w.say(&format!(
            "From: {} ({from})\nDirection: {}\nTo:   {} ({to})",
            draft.rooms[from].text("title"),
            DIRECTIONS[dir],
            draft.rooms[to].text("title")
        ))?;
        w.say(&format!(
            "\nReverse connection:\n  {} --{}--> {}\n",
            draft.rooms[to].text("title"),
            REVERSE[dir],
            draft.rooms[from].text("title")
        ))?;
        let reverse = w.ask(
            &format!(
                "Add {} --{}--> {} so players can return?",
                draft.rooms[to].text("title"),
                REVERSE[dir],
                draft.rooms[from].text("title")
            ),
            "yes",
            "no makes this a one-way connection; existing exits are not removed",
            |v| {
                if matches!(v.to_ascii_lowercase().as_str(), "yes" | "y" | "no" | "n") {
                    Ok(())
                } else {
                    Err("Enter yes or no".into())
                }
            },
        )?;
        let Some(reverse) = reverse else {
            continue;
        };
        let reverse = matches!(reverse.to_ascii_lowercase().as_str(), "yes" | "y");
        let mut links = vec![(from, DIRECTIONS[dir], to)];
        if reverse {
            links.push((to, REVERSE[dir], from));
        }
        let mut approved = true;
        for (source, direction, target) in &links {
            if let Some(old) = draft.rooms[*source].exit(direction) {
                if old != *target && !confirm(w, &format!("Conflict: {source} already goes {direction} to {old}. Replace that exit with {target}?"))? { approved = false; break; }
            }
        }
        if approved {
            for (source, direction, target) in links {
                draft
                    .rooms
                    .get_mut(source)
                    .unwrap()
                    .set_exit(direction, target)?;
            }
            w.say(if reverse {
                "\nBidirectional connection drafted (outgoing and reverse exits)."
            } else {
                "\nOne-way connection drafted. Existing reverse exits, if any, stay unchanged."
            })?;
            w.say("Connection added to the draft. Add more, or choose 0 to review. Existing conditional exit rules still apply; they are not edited here.")?;
        } else {
            w.say("Connection left unchanged.")?;
        }
    }
}
fn properties<R: BufRead, W: Write>(
    draft: &Draft,
    id: &str,
    w: &mut Wizard<'_, R, W>,
    detailed: bool,
) -> Result<(), String> {
    let room = &draft.rooms[id];
    w.heading("ROOM SUMMARY")?;
    w.say(&format!("Name: {}\nStable ID: {id}", room.text("title")))?;
    if detailed {
        w.say(&format!("\nDescription:\n{}", room.text("description")))?;
    }
    w.say("\nOutgoing connections:")?;
    let mut outgoing = false;
    for (index, dir) in DIRECTIONS.iter().enumerate() {
        if let Some(target) = room.exit(dir) {
            outgoing = true;
            let kind = if draft
                .rooms
                .get(target)
                .is_some_and(|r| r.exit(REVERSE[index]) == Some(id))
            {
                "bidirectional"
            } else {
                "one-way"
            };
            w.say(&format!(
                "  {dir} -> {} ({target}) [{kind}]",
                draft
                    .rooms
                    .get(target)
                    .map(|r| r.text("title"))
                    .unwrap_or("missing")
            ))?;
        }
    }
    if !outgoing {
        w.say("  None - this room has no outgoing exits.")?;
    }
    w.say("\nIncoming connections:")?;
    let mut incoming = false;
    for (source, r) in &draft.rooms {
        for dir in DIRECTIONS {
            if r.exit(dir) == Some(id) {
                incoming = true;
                w.say(&format!(
                    "  {} --{dir}--> this location ({source})",
                    r.text("title")
                ))?;
            }
        }
    }
    if !incoming {
        w.say("  None - players may not be able to reach this room.")?;
    }
    if !detailed {
        return Ok(());
    }
    for key in [
        "descriptions",
        "art",
        "art_file",
        "art_variants",
        "encounters",
    ] {
        if let Some(value) = room.doc.get(key) {
            w.say(&format!("\nPreserved {key}: {value}"))?;
        }
    }
    if let Ok(game) = crate::story_loader::load_game(&draft.snapshot.root) {
        w.say("\nItems starting here (authored in item files):")?;
        for item in game
            .world
            .items
            .iter()
            .filter(|item| item.starting_room == id)
        {
            w.say(&format!("  {} ({})", item.name, item.id))?;
        }
        w.say("\nConditional exits (preserved in interactions.toml):")?;
        for rule in game.world.exit_rules.iter().filter(|rule| rule.from == id) {
            w.say(&format!(
                "  {}: {:?}; blocked message: {}",
                rule.direction.label(),
                rule.requires,
                rule.blocked_text
            ))?;
        }
    }
    w.say("\nConditional descriptions choose text based on flags or inventory. ASCII artwork\nand art variants decorate a room. Items are placed by their own starting_room field;\nconditional exits live in interactions.toml. Encounters are optional RPG content.\nThese advanced properties are preserved; their guided editors come later.")
}
fn create<R: BufRead, W: Write>(
    draft: &mut Draft,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<String>, String> {
    w.heading("STEP 1 - NAME YOUR LOCATION")?;
    w.say("A room is any place a player visits, indoors or outdoors.\nExamples: Forest Clearing, Mountain Pass, Spaceship Bridge. You can rename its display title later.")?;
    let mut title = "New Location".to_owned();
    let mut id = String::new();
    let mut step = 0;
    loop {
        match step {
            0 => match w.ask(
                "Location name",
                &title,
                "A player-facing name, up to 120 characters",
                |v| super::validate_text(v, 120),
            )? {
                Some(v) => {
                    title = v;
                    id = super::setup::suggested_id(&title);
                    step = 1;
                }
                None => return Ok(None),
            },
            1 => {
                w.say("This internal ID connects rooms. Enter accepts the suggestion; advanced authors may type another. Keep it stable once players save progress.")?;
                match w.ask("Internal room ID", &id, "Lowercase letters, digits, - or _. It must be unique. Existing room IDs are never renamed by this editor.", |v| {
                    super::validate_id(v)?;
                    if draft.rooms.contains_key(v) || std::fs::symlink_metadata(draft.snapshot.root.join("rooms").join(format!("{v}.toml"))).is_ok() { Err("That room ID or filename already exists. Choose another.".into()) } else { Ok(()) }
                })? { Some(v) => { id = v; step = 2; } None => step = 0 }
            }
            _ => {
                w.heading("STEP 2 - DESCRIBE YOUR LOCATION")?;
                w.say("What does the player see? Example: A narrow trail winds into the mountains.\n{{objects}} inserts the names of currently visible objects during play.\nYou can revise this description in Room Options before saving.")?;
                let Some(description) = w.description("Describe this location.")? else {
                    step = 1;
                    continue;
                };
                let mut doc = DocumentMut::new();
                doc["id"] = value(&id);
                doc["title"] = value(&title);
                doc["description"] = value(description);
                doc["exits"] = Item::Table(toml_edit::Table::new());
                draft.rooms.insert(
                    id.clone(),
                    Room {
                        path: PathBuf::from("rooms").join(format!("{id}.toml")),
                        original: None,
                        baseline: String::new(),
                        doc,
                    },
                );
                connections(draft, &id, w)?;
                return Ok(Some(id));
            }
        }
    }
}
fn edit<R: BufRead, W: Write>(
    draft: &mut Draft,
    id: &str,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    loop {
        w.heading("ROOM OPTIONS")?;
        w.say(&format!("Room: {} ({id})\n\n  1. Change name\n  2. Edit description\n  3. Manage connections\n  4. Review properties\n  5. Preview exact TOML changes\n  6. Save room and connections\n  7. Conditional descriptions and encounters\n  0. Back without saving (discard draft)", draft.rooms[id].text("title")))?;
        match menu(w, "Selection: ")?.as_str() {
            "0" | "back" => return Ok(false),
            "1" => {
                if let Some(v) = w.ask(
                    "Location name",
                    draft.rooms[id].text("title"),
                    "Display name only; the stable ID stays unchanged",
                    |v| super::validate_text(v, 120),
                )? {
                    draft.rooms.get_mut(id).unwrap().set("title", &v);
                }
            }
            "2" => {
                w.say("Describe what players see. {{objects}} lists visible objects; conditional descriptions, if present, can take precedence.")?;
                if let Some(v) = w.description(draft.rooms[id].text("description"))? {
                    draft.rooms.get_mut(id).unwrap().set("description", &v);
                }
            }
            "3" => connections(draft, id, w)?,
            "4" => properties(draft, id, w, true)?,
            "7" => super::room_advanced::edit(
                &mut draft.rooms.get_mut(id).unwrap().doc,
                &draft.snapshot,
                w,
            )?,
            "5" => draft.preview(id, w.output)?,
            "6" => {
                properties(draft, id, w, false)?;
                let changes = draft.changes();
                if changes.is_empty() {
                    w.say("No changes to save.")?;
                    return Ok(false);
                }
                w.say("\nFiles to create/update (only these files will change):")?;
                for path in changes.keys() {
                    w.say(&format!("  {}", path.display()))?;
                }
                w.say("Choose 5 from Room Options to inspect exact TOML. Saving keeps existing IDs. Changes to connections can affect a saved game's routes; no save files are changed.")?;
                if confirm(w, "Save all these room changes?")? {
                    draft.snapshot.commit(&changes)?;
                    w.heading("ROOM SAVED!")?;
                    w.say(&format!(
                        "Room:\n  {}\n\nFiles saved:",
                        draft.rooms[id].text("title")
                    ))?;
                    for path in changes.keys() {
                        w.say(&format!("  {}", path.display()))?;
                    }
                    w.say("\nValidation:\n  PASSED\n\nRoom changes saved. Returning to Room Manager; select a room again to inspect its result.\n")?;
                    return Ok(true);
                }
            }
            "help" => (),
            _ => w.say("Choose 0-7 or help.")?,
        }
    }
}

pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    loop {
        let mut draft = match Draft::load(root) {
            Ok(d) => d,
            Err(e) => {
                writeln!(output, "\nCannot edit rooms: {e}\n")?;
                return Ok(());
            }
        };
        let title = Adventure::load(root)
            .map(|a| a.game.title)
            .unwrap_or_default();
        let mut w = Wizard { input, output };
        w.heading("ROOM MANAGER").map_err(io::Error::other)?;
        w.say(&format!("Adventure: {title}\n\nExisting Rooms:\n"))
            .map_err(io::Error::other)?;
        let ids = draft.rooms.keys().cloned().collect::<Vec<_>>();
        for (i, id) in ids.iter().enumerate() {
            w.say(&format!(
                "  {}. {} ({id})",
                i + 1,
                draft.rooms[id].text("title")
            ))
            .map_err(io::Error::other)?;
        }
        w.say("\n----------------------------------------\n\n  N. Create New Room\n  0. Return to Creator Menu")
            .map_err(io::Error::other)?;
        let choice = match menu(&mut w, "Select a room or N: ") {
            Ok(c) => c,
            Err(e) if e.starts_with("Setup canceled") => return Ok(()),
            Err(e) => return Err(io::Error::other(e)),
        };
        if matches!(choice.as_str(), "0" | "back") {
            return Ok(());
        }
        let operation = (|| -> Result<bool, String> {
            let id = if choice == "n" {
                create(&mut draft, &mut w)?
            } else {
                choice
                    .parse::<usize>()
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .and_then(|n| ids.get(n))
                    .cloned()
            };
            if let Some(id) = id {
                edit(&mut draft, &id, &mut w)
            } else {
                if choice != "help" {
                    w.say("Choose a listed room number, N, or 0.")?;
                }
                Ok(false)
            }
        })();
        match operation {
            Ok(true) => writeln!(w.output, "{}", tree::inspect(root).tree)?,
            Ok(false) => (),
            Err(e) => writeln!(
                w.output,
                "\nRoom operation stopped: {}\n",
                e.replace("Setup canceled", "Draft canceled")
            )?,
        }
    }
}
