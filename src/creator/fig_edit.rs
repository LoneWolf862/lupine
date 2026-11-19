//! Field help and displayed defaults come from the same FIG templates as scaffolding.
use super::{
    content_edit::{save, Document},
    room_transaction::Snapshot,
    rules_ui,
    spell_edit::number,
    wizard::Wizard,
};
use crate::adventure::Adventure;
use std::{
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};
use toml_edit::{value, DocumentMut, Item, Table};
const CHARACTER: &str = include_str!("../../templates/starter/config/character.fig");
const TERMS: &str = include_str!("../../templates/starter/config/terminology.fig");

fn explanation(template: &str, key: &str) -> String {
    let mut comments = Vec::new();
    for line in template.lines() {
        let line = line.trim();
        if let Some(text) = line.strip_prefix('#') {
            comments.push(text.trim());
        } else if line
            .split_once('=')
            .is_some_and(|(name, _)| name.trim() == key)
        {
            return comments.join("\n");
        } else if !line.is_empty() {
            comments.clear();
        }
    }
    String::new()
}
fn character<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    let path = PathBuf::from("config/character.fig");
    let existed = snapshot.source(&path).is_some();
    let mut d = Document::load(snapshot, path, CHARACTER)?;
    let defaults = CHARACTER
        .parse::<DocumentMut>()
        .map_err(|e| e.to_string())?;
    let fields = defaults
        .iter()
        .map(|(k, _)| k.to_owned())
        .collect::<Vec<_>>();
    w.say("Character settings control optional RPG stats and progression. New projects default to disabled. An existing FIG with no enabled key is enabled. Changes apply on reload. Existing player saves are not rewritten; incompatible settings may require a deliberate game.toml save_compatibility_version change.")?;
    loop {
        w.heading("CHARACTER CONFIGURATION")?;
        for (i, key) in fields.iter().enumerate() {
            let current = if key == "enabled" && existed && !d.doc.contains_key(key) {
                "true (omitted in existing file)".into()
            } else {
                d.doc.get(key).unwrap_or(&defaults[key]).to_string()
            };
            w.say(&format!(
                "{}. {key}: {} (new-project default: {})",
                i + 1,
                current.trim(),
                defaults[key].to_string().trim()
            ))?;
        }
        w.say("P. Preview exact FIG\nS. Save\n0. Discard and return")?;
        let choice = w
            .menu("Setting (number or exact field name):")?
            .trim()
            .to_ascii_lowercase();
        match choice.as_str() {
            "0" | "back" => return Ok(()),
            "p" => d.preview(&snapshot.root, w.output)?,
            "s" => {
                if save(snapshot, &d, w)? {
                    return Ok(());
                }
            }
            _ => {
                let list = fields
                    .iter()
                    .map(|k| (k.clone(), k.clone()))
                    .collect::<Vec<_>>();
                let Some(key) = super::content_edit::selection_index(&choice, &list)
                    .ok()
                    .and_then(|n| fields.get(n))
                else {
                    w.say("Select a listed setting, P, S or 0. ? at a field shows help; Enter keeps its current value.")?;
                    continue;
                };
                w.say(&explanation(CHARACTER, key))?;
                if key == "enabled" {
                    let current = d.doc.get(key).and_then(Item::as_bool).unwrap_or(existed);
                    if let Some(s)=w.ask("Enable RPG?",if current{"true"}else{"false"},"true enables RPG; false disables it. Existing RPG content prevents disabling until removed. This is never changed implicitly.",|s|if ["true","false"].contains(&s){Ok(())}else{Err("Enter true or false".into())})?{d.set(key,value(s=="true"));}
                } else {
                    let current = d
                        .doc
                        .get(key)
                        .unwrap_or(&defaults[key])
                        .as_integer()
                        .ok_or("Expected integer setting")?;
                    let (min, max) = match key.as_str() {
                        "starting_level" | "starting_hp" | "starting_ap" => (1, u32::MAX as i64),
                        "base_requirement" => (1, i64::MAX),
                        "growth_percent" => (100, 1000),
                        _ => (0, u32::MAX as i64),
                    };
                    if let Some(n) = number(w, key, current, min, max)? {
                        d.set(key, value(n));
                    }
                }
            }
        }
    }
}
fn terminology<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    let mut d = Document::load(snapshot, PathBuf::from("config/terminology.fig"), TERMS)?;
    let defaults = TERMS.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    w.say("Terminology changes displayed labels, not player commands. spellbook and spells remain commands. No mana/currency renaming fields are implemented.")?;
    loop {
        w.heading("TERMINOLOGY CONFIGURATION")?;
        for (i, key) in ["spellbook", "spells"].iter().enumerate() {
            let current = d
                .doc
                .get("display")
                .and_then(Item::as_table_like)
                .and_then(|t| t.get(key))
                .and_then(Item::as_str)
                .unwrap_or(defaults["display"][key].as_str().unwrap());
            w.say(&format!(
                "{}. {key}: {current} (default: {})",
                i + 1,
                defaults["display"][key].as_str().unwrap()
            ))?;
        }
        w.say("P. Preview\nS. Save\n0. Discard and return")?;
        match w
            .menu("Setting (number or exact field name):")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "0" | "back" => return Ok(()),
            "p" => d.preview(&snapshot.root, w.output)?,
            "s" => {
                if save(snapshot, &d, w)? {
                    return Ok(());
                }
            }
            s @ ("1" | "2" | "spellbook" | "spells") => {
                let key = if s == "1" || s == "spellbook" {
                    "spellbook"
                } else {
                    "spells"
                };
                w.say(&explanation(TERMS, key))?;
                let current = d
                    .doc
                    .get("display")
                    .and_then(Item::as_table_like)
                    .and_then(|t| t.get(key))
                    .and_then(Item::as_str)
                    .unwrap_or(defaults["display"][key].as_str().unwrap());
                if let Some(s) = w.ask(
                    "Displayed label",
                    current,
                    "1-80 characters, nonblank, no control characters. This changes labels only.",
                    |s| {
                        if s.trim().is_empty()
                            || s.chars().count() > 80
                            || s.chars().any(char::is_control)
                        {
                            Err("Use 1-80 nonblank characters without controls".into())
                        } else {
                            Ok(())
                        }
                    },
                )? {
                    if !d.doc.contains_key("display") {
                        d.doc["display"] = Item::Table(Table::new());
                    }
                    let t = d.doc["display"]
                        .as_table_like_mut()
                        .ok_or("Cannot safely edit this display representation")?;
                    rules_ui::set(t, key, value(s));
                }
            }
            _ => w.say("Choose 1, 2, P, S or 0.")?,
        }
    }
}
fn combat<R: BufRead, W: Write>(
    snapshot: &Snapshot,
    w: &mut Wizard<'_, R, W>,
) -> Result<(), String> {
    let template = include_str!("../../templates/starter/config/combat.fig");
    let mut d = Document::load(snapshot, PathBuf::from("config/combat.fig"), template)?;
    loop {
        w.heading("COMBAT CONFIGURATION")?;
        w.say(&format!(
            "{}\n1. Minimum damage\n2. Retaliation\nP. Preview\nS. Save\n0. Discard and return",
            d.doc
        ))?;
        match w.menu("Setting:")?.to_ascii_lowercase().as_str() {
            "0" | "back" => return Ok(()),
            "1" => {
                w.say(&explanation(template, "minimum_damage"))?;
                if let Some(n) = number(
                    w,
                    "Minimum damage",
                    d.doc
                        .get("minimum_damage")
                        .and_then(Item::as_integer)
                        .unwrap_or(1),
                    0,
                    1_000_000,
                )? {
                    d.set("minimum_damage", value(n));
                }
            }
            "2" => {
                w.say(&explanation(template, "retaliation"))?;
                let current = d
                    .doc
                    .get("retaliation")
                    .and_then(Item::as_str)
                    .unwrap_or("combat");
                if let Some(s) = w.ask(
                    "Retaliation",
                    current,
                    "combat: attacks only; every_action: any successful turn; none: never",
                    |s| {
                        if ["combat", "every_action", "none"].contains(&s) {
                            Ok(())
                        } else {
                            Err("Choose combat, every_action or none".into())
                        }
                    },
                )? {
                    d.set("retaliation", value(s));
                }
            }
            "p" => d.preview(&snapshot.root, w.output)?,
            "s" => {
                if save(snapshot, &d, w)? {
                    return Ok(());
                }
            }
            _ => w.say("Choose 0-2, P or S.")?,
        }
    }
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
            w.heading("ADVENTURE CONFIGURATION")?;
            w.say("1. Character: RPG enablement, starting stats and progression\n2. Terminology: displayed collection labels\n3. Combat: damage and retaliation\n0. Return to creator\nThese three FIG categories are implemented. Opening a category writes nothing.")?;
            let result = match w.menu("Category:")?.trim() {
                "0" | "back" => return Ok(()),
                "1" => character(&snapshot, &mut w),
                "2" => terminology(&snapshot, &mut w),
                "3" => combat(&snapshot, &mut w),
                _ => {
                    w.say("Choose 0-3.")?;
                    continue;
                }
            };
            if let Err(e) = result {
                w.say(&format!(
                    "Configuration edit stopped: {}",
                    e.replace("Setup canceled", "Draft canceled")
                ))?;
            }
        }
    })();
    if let Err(e) = result {
        w.say(&format!(
            "Configuration manager stopped: {}",
            e.replace("Setup canceled", "Draft canceled")
        ))
        .map_err(io::Error::other)?;
    }
    Ok(())
}
