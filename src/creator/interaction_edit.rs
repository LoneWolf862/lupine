//! Existing condition/effect rules with a transactional locked-door recipe.
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
use toml_edit::{value, Array, InlineTable, Item, Table, Value};

fn one(key: &str, text: &str) -> InlineTable {
    let mut t = InlineTable::new();
    t.insert(key, Value::from(text));
    t
}
fn effects<R: BufRead, W: Write>(
    current: &Item,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<Array>, String> {
    let mut result = if current.is_none() {
        Array::new()
    } else {
        current
            .as_array()
            .cloned()
            .ok_or("Cannot safely edit this effect representation; original retained")?
    };
    loop {
        w.say(&format!("Effects run after all requirements pass: {result}\n1. Set flag\n2. Clear flag\n3. Move item to room\nR. Remove an effect\n0. Finish\nback leaves effects unchanged"))?;
        match w.line("Effect:")?.trim().to_ascii_lowercase().as_str(){
            "0"=>return Ok(Some(result)),"back"=>return Ok(None),
            s @ ("1"|"2")=>if let Some(flag)=w.ask("Flag name","","Flags remember story changes; use the same name in exit/description/art conditions",super::validate_id)?{result.push(one(if s=="1"{"set_flag"}else{"clear_flag"},&flag));},
            "3"=>{
                let Some(item)=choose(w,"Item to move:",&entries(snapshot,"items")?)? else {continue;};
                let Some(room)=choose(w,"Destination room:",&entries(snapshot,"rooms")?)? else {continue;};
                let mut fields=one("item",&item);fields.insert("room",Value::from(room));let mut t=InlineTable::new();t.insert("move_item_to_room",Value::InlineTable(fields));result.push(t);
            },
            "r"=>{let list=result.iter().enumerate().map(|(i,v)|(i.to_string(),v.to_string())).collect::<Vec<_>>();if let Some(n)=choose(w,"Remove effect:",&list)?{result.remove(n.parse().unwrap());}},
            _=>w.say("Choose 0-3 or R. Flags can change exits, prose and art without special engine code.")?,
        }
    }
}
fn exit<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<(String, String)>, String> {
    let Some(room) = choose(w, "Room containing the exit:", &entries(snapshot, "rooms")?)? else {
        return Ok(None);
    };
    let doc = snapshot
        .room_sources()
        .find_map(|(_, b)| {
            let v = std::str::from_utf8(b).ok()?.parse::<toml::Value>().ok()?;
            (v["id"].as_str() == Some(&room)).then_some(v)
        })
        .ok_or("Room missing")?;
    let list = doc
        .get("exits")
        .and_then(toml::Value::as_table)
        .map(|t| {
            t.iter()
                .map(|(k, v)| (k.clone(), format!("{k} to {}", v.as_str().unwrap_or(""))))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if list.is_empty() {
        w.say("This room has no exits. Connect rooms with main menu 7 first.")?;
        return Ok(None);
    }
    Ok(choose(w, "Exit direction:", &list)?.map(|dir| (room, dir)))
}
fn interaction<R: BufRead, W: Write>(
    d: &mut Document,
    index: usize,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        let t = rules_ui::entry_mut(&mut d.doc, "interactions", index)?;
        w.heading("INTERACTION")?;
        w.say("Players use <item> on <target>. Both objects must be accessible. Requirements all pass before effects run. Each used-item/target pair has one rule.\n1. Used item\n2. Target object\n3. Requirements\n4. Effects\n5. Success message\n6. Failure message\n0. Finish editing this rule")?;
        match w.line("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0" | "back" => return Ok(()),
            s @ ("1" | "2") => {
                if let Some(id) = choose(w, "Object:", &entries(snapshot, "items")?)? {
                    rules_ui::set(t, if s == "1" { "item" } else { "target" }, value(id));
                }
            }
            "3" => {
                if let Some(a) = rules_ui::edit_conditions(
                    t.get("requires").unwrap_or(&Item::None),
                    snapshot,
                    w,
                )? {
                    rules_ui::set(t, "requires", value(a));
                }
            }
            "4" => {
                if let Some(a) = effects(t.get("effects").unwrap_or(&Item::None), snapshot, w)? {
                    rules_ui::set(t, "effects", value(a));
                }
            }
            s @ ("5" | "6") => {
                let key = if s == "5" {
                    "success_text"
                } else {
                    "failure_text"
                };
                w.say("Write the message players see for this outcome.")?;
                if let Some(v) = w.description(t.get(key).and_then(Item::as_str).unwrap_or(""))? {
                    rules_ui::set(t, key, value(v));
                }
            }
            _ => w.say("Choose 0-6. Return to the rules menu to preview and save all changes.")?,
        }
    }
}
fn exit_rule<R: BufRead, W: Write>(
    d: &mut Document,
    index: usize,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        let t = rules_ui::entry_mut(&mut d.doc, "exit_rules", index)?;
        w.say("CONDITIONAL EXIT\n1. Room and exit\n2. Requirements to allow travel\n3. Message when blocked\n0. Finish editing")?;
        match w.line("Selection:")?.trim(){
            "0"|"back"=>return Ok(()),
            "1"=>if let Some((room,dir))=exit(snapshot,w)?{rules_ui::set(t,"from",value(room));rules_ui::set(t,"direction",value(dir));},
            "2"=>if let Some(a)=rules_ui::edit_conditions(t.get("requires").unwrap_or(&Item::None),snapshot,w)?{rules_ui::set(t,"requires",value(a));},
            "3"=>if let Some(v)=w.description(t.get("blocked_text").and_then(Item::as_str).unwrap_or(""))?{rules_ui::set(t,"blocked_text",value(v));},
            _=>w.say("Choose 0-3. An exit must already connect rooms. All requirements must pass to travel.")?,
        }
    }
}
fn count(doc: &toml_edit::DocumentMut, key: &str) -> usize {
    doc.get(key)
        .map(|i| {
            i.as_array()
                .map(Array::len)
                .or_else(|| i.as_array_of_tables().map(|a| a.len()))
                .unwrap_or(0)
        })
        .unwrap_or(0)
}
fn rules<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    let mut d = Document::load(
        snapshot,
        PathBuf::from("interactions.toml"),
        "# Interaction and exit rules\n",
    )?;
    loop {
        w.heading("INTERACTION AND EXIT RULES")?;
        w.say("1. Add interaction\n2. Edit interaction\n3. Remove interaction\n4. Add conditional exit\n5. Edit conditional exit\n6. Remove conditional exit\n7. Preview all rules\nS. Save\n0. Discard and return")?;
        match w.line("Selection:")?.trim().to_ascii_lowercase().as_str(){
            "0"|"back"=>return Ok(()),
            "1"=>{
                let Some(item)=choose(w,"Used item:",&entries(snapshot,"items")?)? else{continue;};
                let Some(target)=choose(w,"Target object:",&entries(snapshot,"items")?)? else{continue;};
                let mut t=Table::new();t["item"]=value(item);t["target"]=value(target);t["requires"]=value(Array::new());t["effects"]=value(Array::new());t["success_text"]=value("It works.");t["failure_text"]=value("That does not work yet.");
                let i=count(&d.doc,"interactions");rules_ui::append(&mut d.doc,"interactions",t)?;interaction(&mut d,i,snapshot,w)?;
            },
            "4"=>if let Some((room,dir))=exit(snapshot,w)?{
                let mut t=Table::new();t["from"]=value(room);t["direction"]=value(dir);t["requires"]=value(Array::new());t["blocked_text"]=value("The way is blocked.");
                let i=count(&d.doc,"exit_rules");rules_ui::append(&mut d.doc,"exit_rules",t)?;exit_rule(&mut d,i,snapshot,w)?;
            },
            s @ ("2"|"3"|"5"|"6")=>{
                let key=if s=="2"||s=="3"{"interactions"}else{"exit_rules"};
                if let Some(i)=rules_ui::select_entry(&d.doc,key,w)?{
                    if s=="3"||s=="6"{if confirm(w,"Remove this rule?")?{rules_ui::remove(&mut d.doc,key,i)?;}}
                    else if key=="interactions"{interaction(&mut d,i,snapshot,w)?;}else{exit_rule(&mut d,i,snapshot,w)?;}
                }
            },
            "7"=>d.preview(&snapshot.root,w.output)?,
            "s"=>if save(snapshot,&d,w)?{return Ok(());},
            _=>w.say("Choose 0-7 or S. All changes remain drafts until explicit Save. Removing an exit rule makes that route unconditional.")?,
        }
    }
}
fn door_item<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
    room: &str,
    portable: bool,
    changes: &mut BTreeMap<PathBuf, String>,
) -> Result<Option<String>, String> {
    w.say(if portable {
        "Choose a portable key, or N to create one."
    } else {
        "Choose the door object, or N to create a fixed door."
    })?;
    let action = w.line("Enter to select an existing item; N creates; back returns:")?;
    if action.eq_ignore_ascii_case("back") {
        return Ok(None);
    }
    if !action.eq_ignore_ascii_case("n") {
        let id = choose(w, "Item:", &entries(snapshot, "items")?)?;
        if portable {
            if let Some(id) = &id {
                let a = Adventure::load(&snapshot.root)?;
                if !a.game.world.item(id).is_some_and(|i| i.portable) {
                    w.say("A key must be portable to carry. Change it in Item Manager first.")?;
                    return Ok(None);
                }
            }
        }
        return Ok(id);
    }
    let list = entries(snapshot, "items")?;
    let Some(id) = w.ask(
        "New item ID",
        if portable { "key" } else { "door" },
        "Unique stable ID",
        |s| {
            super::validate_id(s)?;
            if list.iter().any(|(id, _)| id == s)
                || changes.contains_key(&PathBuf::from(format!("items/{s}.toml")))
            {
                Err("Choose a unique item ID".into())
            } else {
                Ok(())
            }
        },
    )?
    else {
        return Ok(None);
    };
    let path = PathBuf::from(format!("items/{id}.toml"));
    if snapshot.source(&path).is_some() {
        return Err("That filename already exists; nothing overwritten".into());
    }
    let Some(name) = w.ask(
        "Display name",
        if portable { "Key" } else { "Door" },
        "Name players can type",
        |s| super::validate_text(s, 120),
    )?
    else {
        return Ok(None);
    };
    let Some(description) = w.description(if portable {
        "A key for the locked door."
    } else {
        "A locked door."
    })?
    else {
        return Ok(None);
    };
    let mut d = Document::load(snapshot, path.clone(), "")?;
    d.set("id", value(id.clone()));
    d.set("name", value(name));
    d.set("description", value(description));
    d.set("portable", value(portable));
    d.set("starting_room", value(room));
    d.set("aliases", value(Array::new()));
    changes.insert(path, d.contents());
    Ok(Some(id))
}
fn locked_door<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    w.heading("LOCKED DOOR CREATOR")?;
    w.say("This creates a key-on-door interaction and a conditional exit. A flag remembers when the door is unlocked. The reverse route is unchanged; add its exit rule separately if needed.")?;
    let Some((room, dir)) = exit(snapshot, w)? else {
        return Ok(());
    };
    let mut d = Document::load(snapshot, PathBuf::from("interactions.toml"), "")?;
    let mut changes = BTreeMap::new();
    let Some(key) = door_item(snapshot, w, &room, true, &mut changes)? else {
        return Ok(());
    };
    let Some(door) = door_item(snapshot, w, &room, false, &mut changes)? else {
        return Ok(());
    };
    let Some(flag) = w.ask(
        "Unlocked flag",
        &format!("{door}_open"),
        "Use this same flag for changing descriptions/art. It begins unset.",
        super::validate_id,
    )?
    else {
        return Ok(());
    };
    let Some(success) = w.ask(
        "Success message",
        "The key turns; the door unlocks.",
        "Shown after unlocking",
        |s| super::validate_text(s, 65536),
    )?
    else {
        return Ok(());
    };
    let Some(failure) = w.ask(
        "Failure message",
        "The door cannot be unlocked again.",
        "Shown when the interaction requirements fail",
        |s| super::validate_text(s, 65536),
    )?
    else {
        return Ok(());
    };
    let Some(blocked) = w.ask(
        "Locked exit message",
        "The door is locked.",
        "Shown when trying to travel before unlocking",
        |s| super::validate_text(s, 65536),
    )?
    else {
        return Ok(());
    };
    let mut requires = Array::new();
    requires.push(one("has_item", &key));
    requires.push(one("flag_unset", &flag));
    let mut effects = Array::new();
    effects.push(one("set_flag", &flag));
    let mut t = Table::new();
    t["item"] = value(key);
    t["target"] = value(door);
    t["requires"] = value(requires);
    t["effects"] = value(effects);
    t["success_text"] = value(success);
    t["failure_text"] = value(failure);
    rules_ui::append(&mut d.doc, "interactions", t)?;
    let mut t = Table::new();
    let mut requires = Array::new();
    requires.push(one("flag_set", &flag));
    t["from"] = value(room);
    t["direction"] = value(dir);
    t["requires"] = value(requires);
    t["blocked_text"] = value(blocked);
    rules_ui::append(&mut d.doc, "exit_rules", t)?;
    changes.insert(d.path.clone(), d.contents());
    for (p, s) in &changes {
        DraftFile::text(
            &p.to_string_lossy(),
            s,
            "Locked door: all related files are saved together.",
        )
        .preview(&snapshot.root, w.output)?;
    }
    if confirm(w, "Save this door, items and rules together?")? {
        snapshot.commit(&changes)?;
        w.say("Locked door saved and validated. Playtest the blocked route, take the key, then use key on door. Use the unlocked flag for conditional descriptions or art.")?;
    }
    Ok(())
}
pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    let result = (|| -> Result<(), String> {
        loop {
            let snapshot = Snapshot::capture(root)?;
            Adventure::load(root)?;
            w.heading("PUZZLE MANAGER")?;
            w.say("1. Interaction and conditional exit rules\n2. Guided locked door\n0. Return to creator\nRoom Options > 7 edits conditional descriptions and encounters.")?;
            let result = match w.line("Selection:")?.trim() {
                "0" | "back" => return Ok(()),
                "1" => rules(&snapshot, &mut w),
                "2" => locked_door(&snapshot, &mut w),
                _ => {
                    w.say("Choose 0-2. Puzzles use the existing flags, conditions and effects.")?;
                    continue;
                }
            };
            if let Err(e) = result {
                w.say(&format!(
                    "Puzzle edit stopped: {}",
                    e.replace("Setup canceled", "Draft canceled")
                ))?;
            }
            w.say(&tree::inspect(root).tree)?;
        }
    })();
    if let Err(e) = result {
        w.say(&format!(
            "Puzzle manager stopped: {}",
            e.replace("Setup canceled", "Draft canceled")
        ))
        .map_err(io::Error::other)?;
    }
    Ok(())
}
