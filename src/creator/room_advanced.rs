use super::{
    content_edit::{choose, confirm, entries},
    room_transaction::Snapshot,
    rules_ui,
    spell_edit::number,
    wizard::Wizard,
};
use std::io::{BufRead, Write};
use toml_edit::{value, Array, DocumentMut, InlineTable, Item, Table, Value};

pub(super) fn descriptions<R: BufRead, W: Write>(
    doc: &mut DocumentMut,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        w.say("CONDITIONAL DESCRIPTIONS\nThe first matching description replaces the default. All its conditions must pass. {{objects}} lists visible objects.\n1. Add variant\n2. Edit variant\n3. Remove variant\n4. Move a variant earlier\n0. Return")?;
        match w.line("Selection:")?.trim() {
            "0" | "back" => return Ok(()),
            "1" => {
                let Some(when) = rules_ui::conditions(snapshot, w)? else {
                    continue;
                };
                if when.is_empty() {
                    w.say("At least one condition is needed; edit the default description for unconditional text.")?;
                    continue;
                }
                let Some(text) = w.description_labeled("Description", "")? else {
                    continue;
                };
                let mut t = Table::new();
                t["when"] = value(when);
                t["text"] = value(text);
                rules_ui::append(doc, "descriptions", t)?;
            }
            "2" => {
                if let Some(i) = rules_ui::select_entry(doc, "descriptions", w)? {
                    let t = rules_ui::entry_mut(doc, "descriptions", i)?;
                    w.say("1. Alternate text\n2. Conditions\n0. Return")?;
                    match w.line("Field:")?.trim() {
                        "1" => {
                            if let Some(s) = w.description_labeled(
                                "Description",
                                t.get("text").and_then(Item::as_str).unwrap_or(""),
                            )? {
                                rules_ui::set(t, "text", value(s));
                            }
                        }
                        "2" => {
                            if let Some(a) = rules_ui::edit_conditions(
                                t.get("when").unwrap_or(&Item::None),
                                snapshot,
                                w,
                            )? {
                                if a.is_empty() {
                                    w.say("A variant must keep at least one condition.")?;
                                } else {
                                    rules_ui::set(t, "when", value(a));
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            "3" => {
                if let Some(i) = rules_ui::select_entry(doc, "descriptions", w)? {
                    if confirm(w, "Remove this description variant?")? {
                        rules_ui::remove(doc, "descriptions", i)?;
                    }
                }
            }
            _ => w.say(
                "Choose 0-3. Existing variants stay in order and remain unchanged unless selected.",
            )?,
        }
    }
}
fn encounter_entry<R: BufRead, W: Write>(
    doc: &mut DocumentMut,
    i: usize,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        let t = rules_ui::entry_mut(doc, "enemies", i)?;
        w.say("ENCOUNTER CHOICE\n1. Enemy\n2. Relative weight\n3. Flag requirements\n0. Return\nWeights are relative chances among eligible enemies, not percentages.")?;
        match w.line("Selection:")?.trim() {
            "0" | "back" => return Ok(()),
            "1" => {
                if let Some(id) = choose(w, "Enemy:", &entries(snapshot, "enemies")?)? {
                    rules_ui::set(t, "enemy", value(id));
                }
            }
            "2" => {
                if let Some(n) = number(
                    w,
                    "Weight",
                    t.get("weight").and_then(Item::as_integer).unwrap_or(1),
                    1,
                    u32::MAX as i64,
                )? {
                    rules_ui::set(t, "weight", value(n));
                }
            }
            "3" => {
                let current = t
                    .get("requires")
                    .cloned()
                    .unwrap_or_else(|| value(Array::new()));
                let mut list = current
                    .as_array()
                    .cloned()
                    .ok_or("Cannot safely edit this requirement representation")?;
                w.say(&format!("Current requirements: {list}\n1. Add flag set\n2. Add flag unset\n3. Remove a requirement\n0. Return"))?;
                match w.line("Requirement:")?.trim() {
                    s @ ("1" | "2") => {
                        if let Some(flag) = w.ask(
                            "Flag name",
                            "",
                            "Encounter selection supports flag conditions only",
                            super::validate_id,
                        )? {
                            let mut t = InlineTable::new();
                            t.insert(
                                if s == "1" { "flag_set" } else { "flag_unset" },
                                Value::from(flag),
                            );
                            list.push(t);
                        }
                    }
                    "3" => {
                        let choices = list
                            .iter()
                            .enumerate()
                            .map(|(i, v)| (i.to_string(), v.to_string()))
                            .collect::<Vec<_>>();
                        if let Some(i) = choose(w, "Requirement to remove:", &choices)? {
                            list.remove(i.parse().unwrap());
                        }
                    }
                    _ => (),
                }
                rules_ui::set(t, "requires", value(list));
            }
            _ => w.say("Choose 0-3.")?,
        }
    }
}
fn encounters<R: BufRead, W: Write>(
    doc: &mut DocumentMut,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    if crate::adventure::Adventure::load(&snapshot.root)?
        .game
        .config
        .character
        .is_none()
    {
        w.say(
            "Encounters require RPG. Enable it explicitly in Configure Adventure Settings first.",
        )?;
        return Ok(());
    }
    let original = doc.get("encounters").cloned();
    let inline = original.as_ref().is_some_and(Item::is_inline_table);
    let table = match original.clone() {
        Some(Item::Table(t)) => t,
        Some(Item::Value(Value::InlineTable(t))) => t.into_table(),
        None => {
            let mut t = Table::new();
            t["chance"] = value(0);
            t["min_enemies"] = value(1);
            t["max_enemies"] = value(1);
            t["persistence"] = value("temporary");
            t
        }
        _ => return Err("Cannot safely edit this encounter representation".into()),
    };
    let mut ed = DocumentMut::from(table);
    let baseline = ed.to_string();
    loop {
        w.say(&format!("ROOM ENCOUNTERS\n{ed}\n1. Encounter chance (percent)\n2. Minimum enemies\n3. Maximum enemies\n4. Persistence\n5. Add enemy choice\n6. Edit enemy choice\n7. Remove enemy choice\n8. Remove entire encounter table\n0. Keep draft and return\nback discards these encounter edits"))?;
        match w.line("Selection:")?.trim(){
            "back"=>return Ok(()),
            "0"=>{if ed.to_string()!=baseline||original.is_none(){let table=ed.into_table();doc["encounters"]=if inline{value(table.into_inline_table())}else{Item::Table(table)};}return Ok(());},
            s @ ("1"|"2"|"3")=>{let(key,min,max)=match s{"1"=>("chance",0,100),"2"=>("min_enemies",1,12),_=>("max_enemies",1,12)};if let Some(n)=number(w,key,ed.get(key).and_then(Item::as_integer).unwrap_or(min),min,max)?{rules_ui::set(ed.as_table_mut(),key,value(n));}},
            "4"=>if let Some(s)=choose(w,"Persistence:",&[("temporary".into(),"Reroll on re-entry".into()),("room".into(),"Keep living enemy wounds after retreat".into()),("unique".into(),"Stay cleared after defeat".into())])?{rules_ui::set(ed.as_table_mut(),"persistence",value(s));},
            "5"=>if let Some(id)=choose(w,"Enemy definition:",&entries(snapshot,"enemies")?)?{let mut t=Table::new();t["enemy"]=value(id);t["weight"]=value(1);rules_ui::append(&mut ed,"enemies",t)?;},
            "6"=>if let Some(i)=rules_ui::select_entry(&ed,"enemies",w)?{encounter_entry(&mut ed,i,snapshot,w)?;},
            "7"=>if let Some(i)=rules_ui::select_entry(&ed,"enemies",w)?{if confirm(w,"Remove this enemy choice?")?{rules_ui::remove(&mut ed,"enemies",i)?;}},
            "8"=>if confirm(w,"Remove this room's entire encounter table?")?{doc.remove("encounters");return Ok(());},
            _=>w.say("Choose 0-8 or back. Full validation checks min/max bounds, weights and references at Save.")?,
        }
    }
}
pub(super) fn edit<R: BufRead, W: Write>(
    doc: &mut DocumentMut,
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    loop {
        w.heading("ADVANCED ROOM SETTINGS")?;
        w.say("1. Conditional descriptions\n2. RPG encounter settings\n0. Return to Room Options\nThese changes join the room draft. Use Room Options > Save to publish.")?;
        match w.line("Selection:")?.trim() {
            "0" | "back" => return Ok(()),
            "1" => descriptions(doc, snapshot, w)?,
            "2" => encounters(doc, snapshot, w)?,
            _ => w.say("Choose 0-2. Advanced settings use the existing room schema.")?,
        }
    }
}
