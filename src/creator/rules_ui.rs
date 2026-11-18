//! Shared guided condition lists; ordinary engine TOML, no editor-only schema.
use super::{
    content_edit::{choose, entries},
    room_transaction::Snapshot,
    wizard::Wizard,
};
use std::io::{BufRead, Write};
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

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
