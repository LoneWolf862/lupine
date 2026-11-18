//! Shared guided condition lists; ordinary engine TOML, no editor-only schema.
use super::{
    content_edit::{choose, entries},
    room_transaction::Snapshot,
    wizard::Wizard,
};
use std::io::{BufRead, Write};
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

pub(super) fn set(table: &mut dyn toml_edit::TableLike, key: &str, mut new: Item) {
    if let (Some(old), Some(v)) = (table.get(key).and_then(Item::as_value), new.as_value_mut()) {
        *v.decor_mut() = old.decor().clone();
    }
    table.insert(key, new);
}
pub(super) fn entry_mut<'a>(
    doc: &'a mut DocumentMut,
    key: &str,
    index: usize,
) -> Result<&'a mut dyn toml_edit::TableLike, String> {
    let item = doc.get_mut(key).ok_or("No entries")?;
    match item {
        Item::ArrayOfTables(a) => a
            .get_mut(index)
            .map(|t| t as &mut dyn toml_edit::TableLike)
            .ok_or("No such entry".into()),
        Item::Value(Value::Array(a)) => a
            .get_mut(index)
            .and_then(Value::as_inline_table_mut)
            .map(|t| t as &mut dyn toml_edit::TableLike)
            .ok_or("Cannot safely edit this entry".into()),
        _ => Err("Cannot safely edit this representation; original retained".into()),
    }
}
pub(super) fn select_entry<R: BufRead, W: Write>(
    doc: &DocumentMut,
    key: &str,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<usize>, String> {
    let mut list = Vec::new();
    if let Some(item) = doc.get(key) {
        if let Some(a) = item.as_array_of_tables() {
            for (i, t) in a.iter().enumerate() {
                list.push(((i + 1).to_string(), t.to_string()));
            }
        } else if let Some(a) = item.as_array() {
            for (i, t) in a.iter().enumerate() {
                list.push(((i + 1).to_string(), t.to_string()));
            }
        } else {
            return Err(format!("Cannot safely inspect {key}"));
        }
    }
    if list.is_empty() {
        w.say("No entries yet.")?;
        return Ok(None);
    }
    Ok(choose(w, "Entry number:", &list)?.map(|s| s.parse::<usize>().unwrap() - 1))
}

pub(super) fn edit_conditions<R: BufRead, W: Write>(
    current: &Item,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<Array>, String> {
    let mut list = if current.is_none() {
        Array::new()
    } else {
        current
            .as_array()
            .cloned()
            .ok_or("This condition representation cannot be safely edited; it is preserved")?
    };
    loop {
        w.say(&format!("Current conditions (all required): {list}\nA. Add conditions\nR. Remove one\n0. Finish\nback discards these condition edits"))?;
        match w.line("Selection:")?.trim().to_ascii_lowercase().as_str(){
            "0"=>return Ok(Some(list)), "back"=>return Ok(None),
            "a"=>if let Some(extra)=conditions(snapshot,w)? { for v in extra.iter(){list.push(v.clone());} },
            "r"=>{let entries=list.iter().enumerate().map(|(i,v)|(i.to_string(),v.to_string())).collect::<Vec<_>>();if let Some(i)=choose(w,"Condition to remove:",&entries)?{list.remove(i.parse().unwrap());}},
            _=>w.say("A adds requirements, R removes just one, 0 keeps the proposed list; back leaves it unchanged.")?,
        }
    }
}

pub(super) fn conditions<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<Array>, String> {
    let mut result = Array::new();
    w.say("Conditions are requirements that must ALL be true. A flag is a named yes/no memory, initially unset. Interactions can set or clear it. Has item means the player carries that object. Enter 0 when finished; back discards this list.")?;
    loop {
        w.say(&format!("Current requirements: {result}\n1. Flag is set\n2. Flag is unset\n3. Player carries item\n0. Finish"))?;
        let choice = w.line("Condition:")?;
        let key = match choice.trim() {
            "0" => return Ok(Some(result)),
            "back" => return Ok(None),
            "1" => "flag_set",
            "2" => "flag_unset",
            "3" => "has_item",
            _ => {
                w.say("Choose 0-3 or back. Conditions use existing engine rules.")?;
                continue;
            }
        };
        let id = if key == "has_item" {
            choose(w, "Required item:", &entries(snapshot, "items")?)?
        } else {
            w.ask(
                "Flag name",
                "",
                "A shared name such as door_open; use the same name in the interaction",
                super::validate_id,
            )?
        };
        if let Some(id) = id {
            let mut t = InlineTable::new();
            t.insert(key, Value::from(id));
            result.push(t);
        }
    }
}
pub(super) fn append(doc: &mut DocumentMut, key: &str, table: Table) -> Result<(), String> {
    if !doc.contains_key(key) {
        doc[key] = Item::ArrayOfTables(ArrayOfTables::new());
    }
    if let Some(a) = doc[key].as_array_of_tables_mut() {
        a.push(table);
    } else if let Some(a) = doc[key].as_array_mut() {
        a.push(table.into_inline_table());
    } else {
        return Err(format!(
            "Cannot safely edit {key} representation; original is preserved"
        ));
    }
    Ok(())
}
pub(super) fn remove(doc: &mut DocumentMut, key: &str, index: usize) -> Result<(), String> {
    let Some(item) = doc.get_mut(key) else {
        return Err("No entries to remove".into());
    };
    if let Some(a) = item.as_array_of_tables_mut() {
        if index < a.len() {
            a.remove(index);
            return Ok(());
        }
    } else if let Some(a) = item.as_array_mut() {
        if index < a.len() {
            a.remove(index);
            return Ok(());
        }
    }
    Err("Choose an existing entry number".into())
}
