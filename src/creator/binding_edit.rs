//! Object triggers reuse shop offers and the shared content publication boundary.
use super::{
    content_edit::{choose, confirm, entries, save, Document},
    room_transaction::Snapshot,
    rules_ui,
    wizard::Wizard,
};
use crate::{adventure::Adventure, shops::TransactionBinding};
use serde::Deserialize;
use std::{
    io::{self, BufRead, Write},
    path::Path,
};
use toml_edit::{value, Item, Table};

#[derive(Deserialize)]
struct Bindings {
    #[serde(default)]
    bindings: Vec<TransactionBinding>,
}

fn references<R: BufRead, W: Write>(
    d: &mut Document,
    index: Option<usize>,
    a: &Adventure,
    s: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    let current: Bindings = toml::from_str(&d.contents()).map_err(|e| e.to_string())?;
    let objects = entries(s, "items")?
        .into_iter()
        .filter(|(id, _)| {
            !current
                .bindings
                .iter()
                .enumerate()
                .any(|(i, b)| Some(i) != index && b.item == *id)
        })
        .collect::<Vec<_>>();
    let Some(item) = choose(w, "Object the player uses", &objects)? else {
        return Ok(());
    };
    let room = &a
        .game
        .world
        .item(&item)
        .ok_or("Missing object")?
        .starting_room;
    let shops = a
        .shops
        .shops
        .values()
        .filter(|shop| shop.room == *room)
        .map(|shop| (shop.id.clone(), shop.name.clone()))
        .collect::<Vec<_>>();
    w.say("The object must start in the shop's room. Players use <object> there. Portable objects also work while accessible in that room. Each object can have only one binding.")?;
    let Some(shop) = choose(w, "Shop in the object's starting room", &shops)? else {
        return Ok(());
    };
    let shop_def = &a.shops.shops[&shop];
    let offers = shop_def
        .offers
        .iter()
        .map(|o| (o.id.clone(), o.name.clone()))
        .collect::<Vec<_>>();
    let Some(offer) = choose(w, "Offer to execute", &offers)? else {
        return Ok(());
    };
    let o = shop_def
        .offers
        .iter()
        .find(|o| o.id == offer)
        .ok_or("Missing offer")?;
    w.say(&format!("Offer: {}\nEffect: {}; amount: {:?}; item: {:?}; ability: {:?}; flag: {:?}\nPrice: {} {} (sell_item pays the player)\nStock: {:?} (None means unlimited)\nRequired flags: {:?}\nThe existing offer checks eligibility, charges resources and applies effects only during play. Failed transactions spend nothing. Creating a binding does not execute it.",o.name,o.kind,o.amount,o.item,o.ability,o.flag,o.price,o.currency,o.stock,o.requires_flags))?;
    if let Some(i) = index {
        let t = rules_ui::entry_mut(&mut d.doc, "bindings", i)?;
        rules_ui::set(t, "item", value(item));
        rules_ui::set(t, "shop", value(shop));
        rules_ui::set(t, "offer", value(offer));
    } else {
        let mut t = Table::new();
        t["item"] = value(item);
        t["shop"] = value(shop);
        t["offer"] = value(offer);
        rules_ui::append(&mut d.doc, "bindings", t)?;
    }
    Ok(())
}
fn edit<R: BufRead, W: Write>(
    d: &mut Document,
    i: usize,
    a: &Adventure,
    s: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        w.say("1. Object / shop / offer\n2. Success text\n3. Failure text\n4. Use default success message\n5. Use default failure message\n0. Keep draft and return")?;
        let choice = w.menu("Binding options:")?;
        match choice.trim().to_ascii_lowercase().as_str() {
            "0" | "back" => return Ok(()),
            "1" => references(d, Some(i), a, s, w)?,
            "2" | "3" => {
                let key = if choice.trim() == "2" {
                    "success_text"
                } else {
                    "failure_text"
                };
                w.say("Custom text replaces the engine's transaction message. Default failure messages explain the actual reason (funds, stock, requirements or no useful effect).")?;
                let t = rules_ui::entry_mut(&mut d.doc, "bindings", i)?;
                let old = t.get(key).and_then(Item::as_str).unwrap_or("");
                if let Some(text) = w.description_labeled(key, old)? {
                    rules_ui::set(t, key, value(text));
                }
            }
            "4" | "5" => {
                rules_ui::entry_mut(&mut d.doc, "bindings", i)?.remove(if choice.trim() == "4" {
                    "success_text"
                } else {
                    "failure_text"
                });
            }
            _ => w.say("Choose 0-5. Changes remain in the draft until Save.")?,
        }
    }
}
fn manage<R: BufRead, W: Write>(root: &Path, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
    loop {
        let s = Snapshot::capture(root)?;
        let a = Adventure::load(root)?;
        let mut d = Document::load(&s, "transaction_interactions.toml".into(), "")?;
        loop {
            w.heading("TRANSACTION BINDINGS")?;
            w.say("Connect a world object to an existing shop offer. Use I / H to save missing prerequisite items / shops separately before starting this draft. Canceling a binding never undoes those confirmed saves.")?;
            let bindings: Bindings = toml::from_str(&d.contents()).map_err(|e| e.to_string())?;
            let list = bindings
                .bindings
                .iter()
                .map(|b| {
                    (
                        b.item.clone(),
                        a.game
                            .world
                            .item(&b.item)
                            .map(|i| i.name.clone())
                            .unwrap_or(b.item.clone()),
                    )
                })
                .collect::<Vec<_>>();
            for (i, b) in bindings.bindings.iter().enumerate() {
                w.say(&format!(
                    "{}. {} ({}) — shop: {}; offer: {}",
                    i + 1,
                    list[i].1,
                    b.item,
                    b.shop,
                    b.offer
                ))?;
            }
            w.say("N. New binding\nD. Remove binding\nI. Prerequisite items\nH. Prerequisite shops\nP. Exact file preview\nS. Validate and save\n0. Discard and return")?;
            let choice = w.menu("Selection:")?;
            match choice.trim().to_ascii_lowercase().as_str() {
                "0" | "back" => return Ok(()),
                "n" => references(&mut d, None, &a, &s, w)?,
                "d" => {
                    if let Some(id) = choose(w, "Remove binding", &list)? {
                        if confirm(w, "Remove this binding from the draft?")? {
                            let i = list.iter().position(|(x, _)| x == &id).unwrap();
                            rules_ui::remove(&mut d.doc, "bindings", i)?;
                        }
                    }
                }
                "p" => d.preview(root, w.output)?,
                "s" => {
                    if save(&s, &d, w)? {
                        break;
                    }
                }
                "i" | "h" => {
                    if d.changed() && !bindings.bindings.is_empty() {
                        w.say("Save or discard the active binding draft before separately saving prerequisites. Your draft is retained.")?;
                        continue;
                    }
                    super::content_edit::manage(
                        root,
                        if choice.trim().eq_ignore_ascii_case("i") {
                            "items"
                        } else {
                            "shops"
                        },
                        w,
                    )?;
                    break;
                }
                _ => match super::content_edit::selection_index(choice.trim(), &list) {
                    Ok(i) => edit(&mut d, i, &a, &s, w)?,
                    Err(e) => w.say(&e)?,
                },
            }
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
        writeln!(w.output, "Binding manager stopped: {e}")?;
    }
    Ok(())
}
