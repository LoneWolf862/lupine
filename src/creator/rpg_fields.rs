//! Shared RPG prompts and definition choices. Runtime validation stays authoritative.
use super::{room_transaction::Snapshot, rules_ui, spell_edit::number, wizard::Wizard};
use std::io::{BufRead, Write};
use toml_edit::{value, Item, TableLike};
pub(super) fn numeric<R: BufRead, W: Write>(
    t: &mut dyn TableLike,
    w: &mut Wizard<'_, R, W>,
    key: &str,
    help: &str,
    default: i64,
    min: i64,
    max: i64,
) -> Result<(), String> {
    w.say(help)?;
    let current = t.get(key).and_then(Item::as_integer).unwrap_or(default);
    if let Some(n) = number(w, key, current, min, max)? {
        rules_ui::set(t, key, value(n));
    }
    Ok(())
}
pub(super) fn text<R: BufRead, W: Write>(
    t: &mut dyn TableLike,
    w: &mut Wizard<'_, R, W>,
    key: &str,
    help: &str,
) -> Result<(), String> {
    let current = t.get(key).and_then(Item::as_str).unwrap_or("");
    if let Some(s) = w.ask(key, current, help, |s| super::validate_text(s, 120))? {
        rules_ui::set(t, key, value(s));
    }
    Ok(())
}
pub(super) fn currencies(snapshot: &Snapshot) -> Result<Vec<(String, String)>, String> {
    Ok(crate::adventure::Adventure::load(&snapshot.root)?
        .wallet
        .currencies
        .into_iter()
        .map(|(id, c)| (id, c.name))
        .collect())
}
