//! Guided shops expose existing atomic offers, not a second transaction format.
use super::{
    content_edit::{choose, entries, save, Document},
    room_transaction::Snapshot,
    rpg_fields::{currencies, numeric, text},
    rules_ui,
    wizard::Wizard,
};
use std::io::{BufRead, Write};
use toml_edit::{value, Array, DocumentMut, Item, Table};
fn offer<R: BufRead, W: Write>(
    d: &mut DocumentMut,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        w.say(&format!("OFFER {}\n1. Name\n2. Type\n3. Currency\n4. Price\n5. Item / ability / flag / restoration amount\n6. Stock\n7. Required flags\nP. Preview offer\n0. Keep draft and return",d["id"].as_str().unwrap_or("")))?;
        match w.line("Selection:")?.trim().to_ascii_lowercase().as_str(){
   "0"|"back"=>return Ok(()),
   "1"=>text(d.as_table_mut(),w,"name","Name players can use with buy <offer>")?,
   "2"=>{
    w.say("Item sells a unique object to the player; sell_item buys their carried object. Restoration only charges when useful. Learning teaches an existing ability; set_flag unlocks a named story state. Changing an existing offer's meaning may affect saved purchases; no saves are rewritten.")?;
    let choices=[("item","Player buys item"),("sell_item","Player sells item"),("restore_hp","Restore HP"),("restore_mp","Restore MP"),("learn_ability","Teach ability"),("set_flag","Set story flag")].map(|(id,n)|(id.into(),n.into()));
    if let Some(s)=choose(w,"Offer type:",&choices)?{rules_ui::set(d.as_table_mut(),"type",value(s));}
   },
   "3"=>if let Some(id)=choose(w,"Currency:",&currencies(snapshot)?)?{rules_ui::set(d.as_table_mut(),"currency",value(id));},
   "4"=>numeric(d.as_table_mut(),w,"price","Amount charged (or paid to the player for sell_item). Zero means free. TOML integers are signed 64-bit.",0,0,i64::MAX)?,
   "5"=>{
    let kind=d.get("type").and_then(Item::as_str).unwrap_or("").to_owned();
    match kind.as_str(){
     "item"|"sell_item"=>{let a=crate::adventure::Adventure::load(&snapshot.root)?;let list=entries(snapshot,"items")?.into_iter().filter(|(id,_)|a.game.world.item(id).is_some_and(|i|i.portable)).collect::<Vec<_>>();if let Some(id)=choose(w,"Portable item:",&list)?{rules_ui::set(d.as_table_mut(),"item",value(id));}},
     "learn_ability"=>if let Some(id)=choose(w,"Ability:",&entries(snapshot,"spells")?)?{rules_ui::set(d.as_table_mut(),"ability",value(id));},
     "set_flag"=>{let current=d.get("flag").and_then(Item::as_str).unwrap_or("");if let Some(s)=w.ask("Flag",current,"A named yes/no memory, initially unset. Use the same name in puzzle/exit requirements.",super::validate_id)?{rules_ui::set(d.as_table_mut(),"flag",value(s));}},
     "restore_hp"|"restore_mp"=>numeric(d.as_table_mut(),w,"amount","Positive number of HP or MP restored, capped at the player's maximum.",10,1,u32::MAX as i64)?,
     _=>w.say("Choose a type first.")?,
    }
   },
   "6"=>{
    w.say("Stock counts successful transactions and persists in manual RORK saves. Failed transactions consume none. Omitted means unlimited. Reducing stock below saved purchase counts can invalidate saves.")?;
    let current=d.get("stock").and_then(Item::as_integer).map(|n|n.to_string()).unwrap_or("unlimited".into());
    if let Some(s)=w.ask("Stock",&current,"Enter unlimited or a positive whole number up to 4294967295",|s|if s=="unlimited"||s.parse::<u32>().is_ok_and(|v|v>0){Ok(())}else{Err("Use unlimited or a positive u32".into())})?{if s=="unlimited"{d.remove("stock");}else{rules_ui::set(d.as_table_mut(),"stock",value(s.parse::<i64>().unwrap()));}}
   },
   "7"=>{
    let mut list=d.get("requires_flags").and_then(Item::as_array).cloned().unwrap_or_default();
    loop {w.say(&format!("All these flags must be set: {list}\nA. Add flag\nR. Remove flag\n0. Return"))?;match w.line("Selection:")?.trim().to_ascii_lowercase().as_str(){
     "0"|"back"=>break,
     "a"=>if let Some(s)=w.ask("Required flag","","A flag set by an interaction or another offer",super::validate_id)?{if !list.iter().any(|v|v.as_str()==Some(&s)){list.push(s);}},
     "r"=>{let choices=list.iter().enumerate().map(|(i,v)|(i.to_string(),v.to_string())).collect::<Vec<_>>();if let Some(i)=choose(w,"Remove:",&choices)?{list.remove(i.parse().unwrap());}},
     _=>w.say("Choose A, R, or 0.")?,
    }}rules_ui::set(d.as_table_mut(),"requires_flags",value(list));
   },
   "p"=>w.say(&d.to_string())?,
   _=>w.say("Choose 1-7, P, or 0. IDs are stable; use the shop Save to validate and publish.")?,
  }
    }
}
pub(super) fn defaults<R: BufRead, W: Write>(
    d: &mut Document,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    let Some(room) = choose(w, "Shop location:", &entries(snapshot, "rooms")?)? else {
        return Ok(false);
    };
    d.set("room", value(room));
    d.set("offers", value(Array::new()));
    Ok(true)
}
pub(super) fn form<R: BufRead, W: Write>(
    d: &mut Document,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    w.say("A shop belongs to one room. Offers exchange currencies for goods/services, or pay players for items. Inventory items are unique, not stacks; one item cannot be stocked in multiple offers. Create prerequisite items, spells and currencies before opening this draft. Existing shop/offer IDs remain stable for saved stock and object bindings.")?;
    loop {
        w.heading("SHOP OPTIONS")?;
        w.say("1. Display name\n2. Room\n3. Add offer\n4. Edit offer\nP. Exact preview\nS. Save\n0. Discard")?;
        match w.line("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0" | "back" => return Ok(false),
            "1" => text(d.doc.as_table_mut(), w, "name", "Shop display name")?,
            "2" => {
                if let Some(id) = choose(w, "Shop room:", &entries(snapshot, "rooms")?)? {
                    d.set("room", value(id));
                }
            }
            "3" => {
                let existing: crate::shops::Shop = toml::from_str(&d.contents()).map_err(|e| {
                    format!("Finish existing offer fields before adding another: {e}")
                })?;
                let Some(id)=w.ask("Stable offer ID","","Lowercase letters, numbers, underscores or dashes; remains fixed for saved purchases",|s|{super::validate_id(s)?;if existing.offers.iter().any(|o|o.id==s){Err("Offer ID already exists".into())}else{Ok(())}})?else{continue;};
                let mut t = Table::new();
                t["id"] = value(id.clone());
                t["name"] = value(id.replace('_', " "));
                t["type"] = value("restore_hp");
                t["price"] = value(0);
                t["amount"] = value(10);
                let Some(currency) = choose(w, "Currency:", &currencies(snapshot)?)? else {
                    continue;
                };
                t["currency"] = value(currency);
                let mut ed = DocumentMut::from(t);
                offer(&mut ed, snapshot, w)?;
                rules_ui::append(&mut d.doc, "offers", ed.into_table())?;
            }
            "4" => {
                if let Some(i) = rules_ui::select_entry(&d.doc, "offers", w)? {
                    let t = rules_ui::entry_mut(&mut d.doc, "offers", i)?;
                    let mut table = Table::new();
                    for (k, v) in t.iter() {
                        table.insert(k, v.clone());
                    }
                    let mut ed = DocumentMut::from(table);
                    offer(&mut ed, snapshot, w)?;
                    let target = rules_ui::entry_mut(&mut d.doc, "offers", i)?;
                    // Retain representation and surrounding table comments; update only changed keys.
                    let removed = target
                        .iter()
                        .filter(|(k, _)| !ed.contains_key(k))
                        .map(|(k, _)| k.to_owned())
                        .collect::<Vec<_>>();
                    for k in removed {
                        target.remove(&k);
                    }
                    for (k, v) in ed.iter() {
                        if target.get(k).map(ToString::to_string) != Some(v.to_string()) {
                            rules_ui::set(target, k, v.clone());
                        }
                    }
                }
            }
            "p" => d.preview(&snapshot.root, w.output)?,
            "s" => {
                if save(snapshot, d, w)? {
                    return Ok(true);
                }
            }
            _ => w.say("Choose 1-4, P, S, or 0.")?,
        }
    }
}
