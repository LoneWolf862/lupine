use super::{
    content_edit::{choose, entries, save, Document},
    room_transaction::Snapshot,
    wizard::Wizard,
};
use std::io::{BufRead, Write};
use toml_edit::{value, Item};

pub(super) fn number<R: BufRead, W: Write>(
    w: &mut Wizard<'_, R, W>,
    label: &str,
    current: i64,
    min: i64,
    max: i64,
) -> Result<Option<i64>, String> {
    w.say(&format!(
        "Accepted range: {min}..{max}. Enter keeps {current}."
    ))?;
    Ok(w.ask(
        label,
        &current.to_string(),
        "Use a whole number in the displayed range; back keeps this field",
        |s| {
            if s.parse::<i64>().is_ok_and(|v| (min..=max).contains(&v)) {
                Ok(())
            } else {
                Err(format!("Enter a whole number from {min} to {max}"))
            }
        },
    )?
    .map(|s| s.parse().unwrap()))
}
pub(super) fn form<R: BufRead, W: Write>(
    d: &mut Document,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    loop {
        w.heading("SPELL OPTIONS")?;
        w.say(&format!("{} ({})\n1. Display name\n2. Description\n3. MP cost\n4. Effect / target\n5. Power\n6. Teaching item\n7. Inspect / preview\nS. Save\n0. Discard and return",d.text("name"),d.text("id")))?;
        match w.menu("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0" | "back" => { w.say("Draft discarded; no draft changes saved.")?; return Ok(false); },
            "1" => if let Some(s)=w.ask("Spell name",d.text("name"),"Displayed name; ID remains stable",|s| super::validate_text(s,120))? {d.set("name",value(s));},
            "2" => if let Some(s)=w.description_labeled("Description", d.text("description"))? {d.set("description",value(s));},
            s @ ("3" | "5") => {
                let (key, label, min) = if s=="3" { ("mana_cost", "MP spent per cast", 0) } else { ("power", "Damage or restoration amount", 1) };
                let current=d.doc.get(key).and_then(Item::as_integer).unwrap_or(min);
                if let Some(n)=number(w,label,current,min,u32::MAX as i64)? {d.set(key,value(n));}
            },
            "4" => {
                w.say("Damage requires a living enemy target (cast <spell> on <enemy>). Heal restores your HP. Restore mana restores your MP; MP cost is still paid. Power must be positive.")?;
                if let Some(s)=choose(w,"Effect:",&[("damage".into(),"Damage an enemy".into()),("heal".into(),"Heal the player".into()),("restore_mana".into(),"Restore player MP".into())])? {d.set("kind",value(s));}
            },
            "6" => {
                w.say("A teaching item is consumed by learn <item>. It must be portable and available to carry. Create items with main menu 4. Choose 0/back to keep the link, or type remove to clear it.")?;
                let a = crate::adventure::Adventure::load(&snapshot.root)?;
                let list = entries(snapshot,"items")?.into_iter().filter(|(id,_)| a.game.world.item(id).is_some_and(|i|i.portable)).collect::<Vec<_>>();
                if let Some(id) = super::content_edit::choose_with_actions(w, "Teaching item:", &list, &[("remove", "Clear the teaching link")])? {
                    if id == "remove" { d.doc.remove("learn_from_item"); }
                    else { d.set("learn_from_item",value(id)); }
                }
            },
            "7" => d.preview(&snapshot.root,w.output)?,
            "s" => if save(snapshot,d,w)? { return Ok(true); },
            "?" | "help" => w.say("Change any field, inspect with 7, then S to save. Defining a spell never teaches it automatically. 0/back discards this draft.")?,
            _ => w.say("Choose 1-7, S, or 0.")?,
        }
    }
}
