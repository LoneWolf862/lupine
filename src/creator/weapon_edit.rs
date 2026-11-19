//! Physical weapon definitions, keyed by a stable inventory item.
use super::{
    content_edit::{self, choose, save, Document},
    room_transaction::Snapshot,
    rpg_fields::{numeric, text},
    wizard::Wizard,
};
use std::{
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};
use toml_edit::{value, DocumentMut, Item};

fn form<R: BufRead, W: Write>(
    d: &mut Document,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    loop {
        w.heading("WEAPON OPTIONS")?;
        w.say(&format!("Physical item: {} (stable association)\n1. Damage\n2. Damage type\nP. Exact preview\nS. Save\n0. Discard",d.text("item")))?;
        match w.menu("Selection:")?.trim().to_ascii_lowercase().as_str(){
   "0"|"back"=>{w.say("Draft discarded; no draft changes saved.")?;return Ok(false);},
   "1"=>numeric(d.doc.as_table_mut(),w,"damage","Base damage at AP 100 (1.00x). AP scales damage, then enemy defense subtracts; a valid hit deals at least 1.",5,1,u32::MAX as i64)?,
   "2"=>text(d.doc.as_table_mut(),w,"damage_type","A nonempty descriptive label, such as physical, piercing or slashing. The engine has no damage-type resistance mechanic.")?,
   "p"=>d.preview(&snapshot.root,w.output)?,
   "s"=>if save(snapshot,d,w)?{return Ok(true);},
   _=>w.say("Choose 1, 2, P, S, or 0. To change item names or descriptions use Item Manager. Weapon item IDs remain stable.")?,
  }
    }
}
fn manage<R: BufRead, W: Write>(root: &Path, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
    w.say("Weapons are physical items: players must carry the associated item before attacking with it. Each item has at most one weapon definition. Create prerequisite items with I; their separately confirmed saves remain even if you later cancel a weapon. No player saves are rewritten.")?;
    loop {
        let snapshot = Snapshot::capture(root)?;
        let a = crate::adventure::Adventure::load(root)?;
        let mut paths = Vec::new();
        let mut list = Vec::new();
        w.heading("WEAPON MANAGER")?;
        for (p, b) in snapshot.sources("weapons") {
            if p.extension().is_none_or(|e| e != "toml") {
                continue;
            }
            let d = std::str::from_utf8(b)
                .map_err(|e| e.to_string())?
                .parse::<DocumentMut>()
                .map_err(|e| e.to_string())?;
            let id = d
                .get("item")
                .and_then(Item::as_str)
                .ok_or("Weapon missing item")?;
            paths.push(p.clone());
            list.push((
                id.to_owned(),
                a.game
                    .world
                    .item(id)
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| id.to_owned()),
            ));
            w.say(&format!(
                "{}. {} ({id})",
                paths.len(),
                a.game.world.item(id).map(|i| i.name.as_str()).unwrap_or(id)
            ))?;
        }
        w.say(
            "N. New weapon\nI. Create/edit prerequisite items (separate confirmed save)\n0. Return",
        )?;
        let choice = w
            .line("Selection (number, exact item ID/name, N, I, or 0):")?
            .trim()
            .to_owned();
        let mut d = match choice.to_ascii_lowercase().as_str() {
            "0" | "back" => return Ok(()),
            "i" => {
                content_edit::manage(root, "items", w)?;
                continue;
            }
            "n" => {
                let list = content_edit::entries(&snapshot, "items")?
                    .into_iter()
                    .filter(|(id, _)| {
                        !a.rpg.weapons.contains_key(id)
                            && a.game.world.item(id).is_some_and(|i| i.portable)
                    })
                    .collect::<Vec<_>>();
                if list.is_empty() {
                    w.say("No unused portable items. Choose I to create one or make an existing item portable, then return here.")?;
                    continue;
                }
                let Some(id) = choose(w, "Associated portable item:", &list)? else {
                    continue;
                };
                let path = PathBuf::from(format!("weapons/{id}.toml"));
                if snapshot.source(&path).is_some() {
                    w.say("That filename already exists; original retained.")?;
                    continue;
                }
                let mut d = Document::load(&snapshot, path, "")?;
                d.set("item", value(id));
                d.set("damage", value(5));
                d.set("damage_type", value("physical"));
                d
            }
            _ => {
                let Some(p) = super::content_edit::selection_index(&choice, &list)
                    .ok()
                    .and_then(|n| paths.get(n))
                else {
                    w.say(&super::content_edit::selection_index(&choice, &list).unwrap_err())?;
                    continue;
                };
                Document::load(&snapshot, p.clone(), "")?
            }
        };
        match form(&mut d, &snapshot, w) {
            Ok(_) => (),
            Err(e) if e.starts_with("Setup canceled") => {
                w.say("Weapon draft canceled; no weapon changes saved.")?
            }
            Err(e) => w.say(&e)?,
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
        writeln!(w.output, "Weapon manager stopped: {e}")?;
    }
    Ok(())
}
