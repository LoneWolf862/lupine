//! Enemy authoring uses the runtime schema and shared recoverable publication.
use super::rpg_fields::{currencies, numeric, text};
use super::{
    content_edit::{self, choose, confirm, entries, save, Document},
    room_transaction::Snapshot,
    rules_ui,
    wizard::Wizard,
};
use std::{
    io::{self, BufRead, Write},
    path::Path,
};
use toml_edit::{value, Table, TableLike};

pub(super) fn defaults(d: &mut Document) {
    d.set("description", value("An opponent."));
    for (key, n) in [
        ("level", 1),
        ("max_hp", 25),
        ("attack_power", 100),
        ("defense", 0),
        ("experience", 0),
    ] {
        d.set(key, value(n));
    }
}
fn edit_entry<R: BufRead, W: Write>(
    t: &mut dyn TableLike,
    key: &str,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    match key {
        "attacks" => {
            text(t, w, "name", "Name shown during an enemy counterattack")?;
            numeric(
                t,
                w,
                "damage",
                "Base damage before enemy attack-power scaling.",
                1,
                1,
                u32::MAX as i64,
            )?;
            numeric(t,w,"weight","Relative selection weight: weights 1 and 3 mean 25% and 75%. They need not total 100; total must fit u32.",1,1,u32::MAX as i64)?;
        }
        "loot" => {
            w.say("Loot is an existing portable unique item. It starts hidden and can drop on defeat; this does not create item stacks.")?;
            if let Some(id) = choose(w, "Drop item:", &entries(snapshot, "items")?)? {
                rules_ui::set(t, "item", value(id));
            }
            numeric(
                t,
                w,
                "chance",
                "Independent drop chance in percent; 100 always attempts a drop.",
                100,
                0,
                100,
            )?;
        }
        _ => {
            if let Some(id) = choose(w, "Reward currency:", &currencies(snapshot)?)? {
                rules_ui::set(t, "currency", value(id));
            }
            numeric(
                t,
                w,
                "chance",
                "Independent reward chance in percent.",
                100,
                0,
                100,
            )?;
            numeric(
                t,
                w,
                "min",
                "Minimum reward; must be <= maximum.",
                1,
                0,
                i64::MAX,
            )?;
            numeric(t,w,"max","Maximum reward; range width cannot exceed 4294967294. TOML integers are signed 64-bit.",1,0,i64::MAX)?;
        }
    }
    Ok(())
}
fn collection<R: BufRead, W: Write>(
    d: &mut Document,
    key: &str,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        w.say(&format!(
            "{key}: A adds, E edits, R removes, 0 returns. Changes remain in the enemy draft."
        ))?;
        match w.menu("Selection:")?.trim().to_ascii_lowercase().as_str() {
            "0" | "back" => return Ok(()),
            "a" => {
                let mut t = Table::new();
                edit_entry(&mut t, key, snapshot, w)?;
                if confirm(w, "Add this entry to the draft?")? {
                    rules_ui::append(&mut d.doc, key, t)?;
                }
            }
            "e" => {
                if let Some(i) = rules_ui::select_entry(&d.doc, key, w)? {
                    edit_entry(rules_ui::entry_mut(&mut d.doc, key, i)?, key, snapshot, w)?;
                }
            }
            "r" => {
                if let Some(i) = rules_ui::select_entry(&d.doc, key, w)? {
                    if confirm(w, "Remove this entry?")? {
                        rules_ui::remove(&mut d.doc, key, i)?;
                    }
                }
            }
            _ => w.say("Choose A, E, R, or 0.")?,
        }
    }
}
pub(super) fn form<R: BufRead, W: Write>(
    d: &mut Document,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<bool, String> {
    form_inner(d, snapshot, w, false)
}
fn form_inner<R: BufRead, W: Write>(
    d: &mut Document,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
    deferred: bool,
) -> Result<bool, String> {
    w.say("Enemies are reusable combat definitions. Add them to a room through Rooms > Advanced > Encounters. Defining an enemy alone does not spawn it. Existing IDs remain fixed. Stat changes can invalidate saved combat HP; saves are never rewritten. No attacks means no counterattack.")?;
    loop {
        w.heading("ENEMY OPTIONS")?;
        w.say("1. Display name\n2. Description\n3. Level\n4. Maximum HP\n5. Attack power\n6. Defense\n7. Experience reward\n8. Attacks\n9. Item drops\nC. Currency rewards\nP. Exact preview\nS. Save\n0. Discard and return")?;
        let choice = w.menu("Selection:")?.trim().to_ascii_lowercase();
        match choice.as_str() {
            "0" | "back" => {
                w.say("Draft discarded; no draft changes saved.")?;
                return Ok(false);
            }
            "1" => text(
                d.doc.as_table_mut(),
                w,
                "name",
                "Player-facing name; stable ID does not change",
            )?,
            "2" => {
                if let Some(s) = w.description_labeled("Description", d.text("description"))? {
                    d.set("description", value(s));
                }
            }
            "3" | "4" | "5" | "6" | "7" => {
                let (key,help,default,min,max)=match choice.as_str(){
                    "3"=>("level","Enemy level is authored independently; it does not automatically scale other stats.",1,1,u32::MAX as i64),
                    "4"=>("max_hp","Health determines damage the enemy can survive. Gameplay keeps exact enemy HP private.",25,1,u32::MAX as i64),
                    "5"=>("attack_power","100 means 1.00x attack damage; 200 means 2.00x. Damage rounds half up, minimum 1.",100,1,u32::MAX as i64),
                    "6"=>("defense","Subtracts from incoming damage; valid hits still deal at least 1.",0,0,u32::MAX as i64),
                    _=>("experience","XP awarded once on defeat. Player progression is configured separately. TOML supports signed 64-bit integers.",0,0,i64::MAX),
                };
                numeric(d.doc.as_table_mut(), w, key, help, default, min, max)?;
            }
            "8" => collection(d, "attacks", snapshot, w)?,
            "9" => collection(d, "loot", snapshot, w)?,
            "c" => collection(d, "currency_rewards", snapshot, w)?,
            "p" => d.preview(&snapshot.root, w.output)?,
            "s" => {
                if deferred {
                    d.preview(&snapshot.root, w.output)?;
                    if confirm(w,"Keep this enemy in the room draft? Room Save will validate and publish both together.")? { return Ok(true); }
                } else if save(snapshot, d, w)? {
                    return Ok(true);
                }
            }
            _ => w.say("Choose a listed field, P, S, or 0.")?,
        }
    }
}
pub(super) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    let mut w = Wizard { input, output };
    let result = (|| {
        let a = crate::adventure::Adventure::load(root)?;
        if a.game.config.character.is_none() {
            return w.say("Enable RPG explicitly in Configure Adventure Settings first.");
        }
        content_edit::manage(root, "enemies", &mut w)
    })();
    if let Err(e) = result {
        writeln!(w.output, "Enemy manager stopped: {e}")?;
    }
    Ok(())
}

pub(super) fn draft_new<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    extra: &mut std::collections::BTreeMap<std::path::PathBuf, String>,
    w: &mut Wizard<'_, R, W>,
) -> Result<Option<String>, String> {
    let list = entries(snapshot, "enemies")?;
    let Some(id) = w.ask(
        "New enemy ID",
        "",
        "Stable lowercase ID; this enemy joins the room draft and is saved only with the room",
        |s| {
            super::validate_id(s)?;
            if list.iter().any(|(id, _)| id == s)
                || extra.contains_key(&std::path::PathBuf::from(format!("enemies/{s}.toml")))
            {
                Err("Enemy ID exists".into())
            } else {
                Ok(())
            }
        },
    )?
    else {
        return Ok(None);
    };
    let path = std::path::PathBuf::from(format!("enemies/{id}.toml"));
    if snapshot.source(&path).is_some() {
        return Err("Enemy filename exists; retained".into());
    }
    let mut d = Document::load(snapshot, path.clone(), "")?;
    d.set("id", value(id.clone()));
    d.set("name", value(id.replace('_', " ")));
    defaults(&mut d);
    if form_inner(&mut d, snapshot, w, true)? {
        extra.insert(path, d.contents());
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

pub(super) fn edit_pending<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    extra: &mut std::collections::BTreeMap<std::path::PathBuf, String>,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    let list = extra
        .keys()
        .map(|p| (p.to_string_lossy().into_owned(), p.display().to_string()))
        .collect::<Vec<_>>();
    let Some(path) = choose(w, "Pending enemy:", &list)? else {
        return Ok(());
    };
    let path = std::path::PathBuf::from(path);
    let mut d = Document::load(snapshot, path.clone(), &extra[&path])?;
    if form_inner(&mut d, snapshot, w, true)? {
        extra.insert(path, d.contents());
    }
    Ok(())
}
