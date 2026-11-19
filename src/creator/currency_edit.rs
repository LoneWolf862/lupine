//! Currency drafts preserve the game manifest and stable wallet references.
use super::{
    content_edit::{save, Document},
    room_transaction::Snapshot,
    rpg_fields::numeric,
    rules_ui,
    wizard::Wizard,
};
use std::{
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};
use toml_edit::{value, Item, Table, TableLike};
fn edit<R: BufRead, W: Write>(
    t: &mut dyn TableLike,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        let id = t.get("id").and_then(Item::as_str).unwrap_or("gold");
        w.say(&format!("Currency ID: {id} (fixed)\n1. Display name\n2. Symbol\n3. Starting balance\n0. Keep draft and return"))?;
        match w.line("Selection:")?.trim(){
   "0"|"back"=>return Ok(()),
   s @ ("1"|"2")=>field(t,w,if s=="1"{"name"}else{"symbol"})?,
   "3"=>numeric(t,w,"starting_amount","Starting balances apply only to fresh games. Manual saves retain their balances. Shops spend this currency and enemies can reward it; changing a label does not change references.",0,0,i64::MAX)?,
   _=>w.say("Choose 1-3 or 0.")?,
  }
    }
}
fn field<R: BufRead, W: Write>(
    t: &mut dyn TableLike,
    w: &mut Wizard<'_, R, W>,
    key: &str,
) -> Result<(), String> {
    let current = t.get(key).and_then(Item::as_str).unwrap_or("");
    if let Some(s) = w.ask(
        key,
        current,
        "Name: 1-80 bytes. Symbol: up to 24 bytes; remove clears it. IDs remain fixed.",
        |s| {
            let max = if key == "name" { 80 } else { 24 };
            if s.len() > max
                || s.chars().any(char::is_control)
                || (key == "name" && s.trim().is_empty())
            {
                Err("Use a nonempty name up to 80 bytes or a symbol up to 24 bytes".into())
            } else {
                Ok(())
            }
        },
    )? {
        rules_ui::set(
            t,
            key,
            value(if key == "symbol" && s == "remove" {
                String::new()
            } else {
                s
            }),
        );
    }
    Ok(())
}
fn create<R: BufRead, W: Write>(
    d: &Document,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<Table>, String> {
    let v: toml::Value = d
        .contents()
        .parse::<toml::Value>()
        .map_err(|e| e.to_string())?;
    let mut ids = Vec::new();
    if let Some(t) = v.get("currency") {
        ids.push(
            t.get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or("gold")
                .to_owned(),
        );
    }
    if let Some(list) = v.get("currencies").and_then(toml::Value::as_array) {
        for t in list {
            ids.push(
                t.get("id")
                    .and_then(toml::Value::as_str)
                    .unwrap_or("gold")
                    .to_owned(),
            );
        }
    }
    let Some(id)=w.ask("Stable currency ID","","Lowercase letters, digits, underscores/dashes; at most 80 bytes. Never rename IDs used by shops, rewards or saves.",|s|{super::validate_id(s)?;if s.len()>80||ids.iter().any(|id|id==s){Err("Use an unused ID up to 80 bytes".into())}else{Ok(())}})?else{return Ok(None);};
    let mut t = Table::new();
    t["id"] = value(id.clone());
    t["name"] = value(id.replace('_', " "));
    t["symbol"] = value("");
    t["starting_amount"] = value(0);
    edit(&mut t, w)?;
    Ok(Some(t))
}
fn manage<R: BufRead, W: Write>(root: &Path, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
    let snapshot = Snapshot::capture(root)?;
    let a = crate::adventure::Adventure::load(root)?;
    let mut d = Document::load(&snapshot, PathBuf::from("game.toml"), "")?;
    w.say("Currencies live in game.toml alongside adventure metadata. Shops and enemy rewards refer to fixed IDs. Primary currency is shown first in the wallet. Saves retain balances; this editor never renames IDs, removes currencies, rewrites saves, or changes compatibility versions.")?;
    loop {
        w.heading("CURRENCY SETTINGS")?;
        w.say(&format!("Current saved primary: {}\n{}\n1. Configure primary currency\n2. Add another currency\n3. Edit additional currency\nP. Exact manifest preview\nS. Save\n0. Discard",a.wallet.primary.as_deref().unwrap_or("none"),d.doc.get("currency").map(ToString::to_string).unwrap_or_default()))?;
        match w.line("Selection:")?.trim().to_ascii_lowercase().as_str(){
   "0"|"back"=>return Ok(()),
   "1"=>{if !d.doc.contains_key("currency"){if let Some(t)=create(&d,w)?{d.doc["currency"]=Item::Table(t);}}else{let t=d.doc.get_mut("currency").and_then(Item::as_table_like_mut).ok_or("Cannot safely edit primary currency representation")?;edit(t,w)?;}},
   "2"=>if let Some(t)=create(&d,w)?{rules_ui::append(&mut d.doc,"currencies",t)?;},
   "3"=>if let Some(i)=rules_ui::select_entry(&d.doc,"currencies",w)?{edit(rules_ui::entry_mut(&mut d.doc,"currencies",i)?,w)?;},
   "p"=>d.preview(root,w.output)?,
   "s"=>if save(&snapshot,&d,w)?{return Ok(());},
   _=>w.say("Choose 1-3, P, S, or 0. Without [currency], the runtime uses the alphabetically first additional ID as primary.")?,
  }
    }
}
pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    if let Err(e) = manage(root, &mut w) {
        writeln!(w.output, "Currency editor stopped: {e}")?;
    }
    Ok(())
}
