//! Schema-aware authored-ID migration. Filenames, prose and RORK files are never renamed.
use super::{
    content_edit::{choose, confirm, Document},
    room_transaction::Snapshot,
    wizard::Wizard,
};
use crate::adventure::Adventure;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};
use toml_edit::{Item, Value};

// Only these schema paths carry references. '*' denotes one array entry or exit key.
fn matches(path: &[String], pattern: &str) -> bool {
    let parts = pattern.split('.').collect::<Vec<_>>();
    parts.len() == path.len() && parts.iter().zip(path).all(|(a, b)| *a == "*" || *a == b)
}
fn category(file: &Path, path: &[String], scope: &str) -> Option<String> {
    let parent = file.parent().and_then(Path::to_str).unwrap_or("");
    let root = file.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let is = |p| matches(path, p);
    let condition_roots = match (parent, root) {
        ("rooms", _) => vec!["descriptions.*.when.*", "art_variants.*.when.*"],
        ("items", _) => vec!["descriptions.*.when.*", "consumable.requires.*"],
        ("", "interactions.toml") => vec![
            "interactions.*.requires.*",
            "exit_rules.*.requires.*",
            "interactions.*.outcomes.*.when.*",
        ],
        ("", "actions.toml") => vec!["actions.*.requires.*", "actions.*.outcomes.*.when.*"],
        ("", "events.toml") => vec!["events.*.requires.*"],
        _ => vec![],
    };
    for prefix in condition_roots {
        let len = prefix.split('.').count();
        if path.len() > len && matches(&path[..len], prefix) {
            return condition_reference(&path[len..], scope).map(str::to_owned);
        }
    }
    if parent.is_empty() && matches!(root, "interactions.toml" | "actions.toml" | "events.toml") {
        for prefix in [
            "interactions.*.effects.*",
            "interactions.*.outcomes.*.effects.*",
            "events.*.effects.*",
            "actions.*.effects.*",
            "actions.*.outcomes.*.effects.*",
        ] {
            let len = prefix.split('.').count();
            if path.len() > len && matches(&path[..len], prefix) {
                return effect_reference(&path[len..], scope).map(str::to_owned);
            }
        }
    }
    if parent == "items" {
        if is("initial.id") {
            return location_category(scope).map(str::to_owned);
        }
        let prefix = "consumable.effects.*";
        if path.len() > 3 && matches(&path[..3], prefix) {
            return effect_reference(&path[3..], scope).map(str::to_owned);
        }
    }
    if is("id") {
        return match parent {
            "rooms" | "items" | "spells" | "enemies" | "shops" => Some(parent.into()),
            _ => None,
        };
    }
    let mut fields: Vec<(&str, &str)> = Vec::new();
    match parent {
        "rooms" => fields.extend([
            ("exits.*", "rooms"),
            ("encounters.enemies.*.enemy", "enemies"),
            ("encounters.enemies.*.requires.*.flag_set", "flags"),
            ("encounters.enemies.*.requires.*.flag_unset", "flags"),
            ("art_variants.*.when.*.has_item", "items"),
            ("art_variants.*.when.*.flag_set", "flags"),
            ("art_variants.*.when.*.flag_unset", "flags"),
        ]),
        "items" => fields.extend([("starting_room", "rooms"), ("container.key", "items")]),
        "spells" => fields.push(("learn_from_item", "items")),
        "weapons" => fields.push(("item", "items")),
        "enemies" => fields.extend([
            ("loot.*.item", "items"),
            ("currency_rewards.*.currency", "currencies"),
        ]),
        "shops" => {
            if is("offers.*.id") {
                return Some(format!("offers:{scope}"));
            }
            fields.extend([
                ("room", "rooms"),
                ("offers.*.currency", "currencies"),
                ("offers.*.item", "items"),
                ("offers.*.ability", "spells"),
                ("offers.*.flag", "flags"),
                ("offers.*.requires_flags.*", "flags"),
            ]);
        }
        "" => match root {
            "events.toml" => fields.extend([
                ("events.*.id", "events"),
                ("events.*.on.enter", "rooms"),
                ("events.*.on.take", "items"),
                ("events.*.on.action", "actions"),
                ("events.*.on.defeat", "enemies"),
                ("events.*.on.timer", "timers"),
            ]),
            "actions.toml" => fields.extend([
                ("actions.*.id", "actions"),
                ("actions.*.room", "rooms"),
                ("actions.*.target", "items"),
                ("actions.*.item", "items"),
            ]),
            "game.toml" => fields.extend([
                ("starting_room", "rooms"),
                ("currency.id", "currencies"),
                ("currencies.*.id", "currencies"),
            ]),
            "transaction_interactions.toml" => {
                if is("bindings.*.offer") {
                    return Some(format!("offers:{scope}"));
                }
                fields.extend([("bindings.*.item", "items"), ("bindings.*.shop", "shops")]);
            }
            "interactions.toml" => fields.extend([
                ("interactions.*.item", "items"),
                ("interactions.*.target", "items"),
                ("interactions.*.requires.*.has_item", "items"),
                ("interactions.*.requires.*.flag_set", "flags"),
                ("interactions.*.requires.*.flag_unset", "flags"),
                ("interactions.*.effects.*.set_flag", "flags"),
                ("interactions.*.effects.*.clear_flag", "flags"),
                ("interactions.*.effects.*.move_item_to_room.item", "items"),
                ("interactions.*.effects.*.move_item_to_room.room", "rooms"),
                ("exit_rules.*.from", "rooms"),
                ("exit_rules.*.requires.*.has_item", "items"),
                ("exit_rules.*.requires.*.flag_set", "flags"),
                ("exit_rules.*.requires.*.flag_unset", "flags"),
            ]),
            _ => (),
        },
        _ => (),
    }
    if parent == "rooms" || parent == "items" {
        fields.extend([
            ("descriptions.*.when.*.has_item", "items"),
            ("descriptions.*.when.*.flag_set", "flags"),
            ("descriptions.*.when.*.flag_unset", "flags"),
        ]);
    }
    fields
        .into_iter()
        .find(|(p, _)| is(p))
        .map(|(_, c)| c.into())
}
fn condition_reference(path: &[String], scope: &str) -> Option<&'static str> {
    let words = path.iter().map(String::as_str).collect::<Vec<_>>();
    match words.as_slice() {
        ["all" | "any", _, ..] => condition_reference(&path[2..], scope),
        ["not", ..] => condition_reference(&path[1..], scope),
        ["has_item" | "item_visible" | "container_open" | "container_locked"]
        | ["item_count", "item"]
        | ["item_in_room", "item"] => Some("items"),
        ["flag_set" | "flag_unset"] => Some("flags"),
        ["in_room" | "visited"] | ["item_in_room", "room"] => Some("rooms"),
        ["quest_stage", "quest"] => Some("quests"),
        ["journal_has"] => Some("journal"),
        ["timer_running"] => Some("timers"),
        ["counter", "id"] => Some("counters"),
        ["item_count", "location", "id"] => location_category(scope),
        _ => None,
    }
}
fn effect_reference(path: &[String], scope: &str) -> Option<&'static str> {
    let words = path.iter().map(String::as_str).collect::<Vec<_>>();
    match words.as_slice() {
        ["start_timer" | "cancel_timer"] => Some("timers"),
        ["set_quest_stage", "quest"] => Some("quests"),
        ["add_journal"] => Some("journal"),
        ["end_game"] => Some("endings"),
        ["set_flag" | "clear_flag"] => Some("flags"),
        ["give_item" | "remove_item" | "reveal_item" | "hide_item"]
        | ["move_item_to_room" | "add_item" | "remove_quantity" | "transfer_item"
        | "set_container", "item"] => Some("items"),
        ["move_player"] | ["move_item_to_room", "room"] => Some("rooms"),
        ["set_counter" | "add_counter", "id"] => Some("counters"),
        ["transfer_item", "from" | "to", "id"] => location_category(scope),
        _ => None,
    }
}
fn location_category(scope: &str) -> Option<&'static str> {
    match scope {
        "location:room" => Some("rooms"),
        "location:container" => Some("items"),
        _ => None,
    }
}
fn walk(
    item: &mut Item,
    path: &mut Vec<String>,
    scope: &str,
    f: &mut impl FnMut(&[String], &str, &mut Value),
) {
    if let Some(t) = item.as_table_like_mut() {
        let scope = t
            .get("shop")
            .and_then(Item::as_str)
            .unwrap_or(scope)
            .to_owned();
        let scope = t
            .get("kind")
            .and_then(Item::as_str)
            .map(|k| format!("location:{k}"))
            .unwrap_or(scope);
        for (k, v) in t.iter_mut() {
            path.push(k.get().to_owned());
            walk(v, path, &scope, f);
            path.pop();
        }
    } else if let Some(a) = item.as_array_of_tables_mut() {
        for (i, t) in a.iter_mut().enumerate() {
            path.push(i.to_string());
            let scope = t
                .get("shop")
                .and_then(Item::as_str)
                .unwrap_or(scope)
                .to_owned();
            for (k, v) in t.iter_mut() {
                path.push(k.get().to_owned());
                walk(v, path, &scope, f);
                path.pop();
            }
            path.pop();
        }
    } else if let Some(v) = item.as_value_mut() {
        walk_value(v, path, scope, f);
    }
}
fn walk_value(
    v: &mut Value,
    path: &mut Vec<String>,
    scope: &str,
    f: &mut impl FnMut(&[String], &str, &mut Value),
) {
    if let Some(t) = v.as_inline_table_mut() {
        let scope = t
            .get("shop")
            .and_then(Value::as_str)
            .unwrap_or(scope)
            .to_owned();
        let scope = t
            .get("kind")
            .and_then(Value::as_str)
            .map(|k| format!("location:{k}"))
            .unwrap_or(scope);
        for (k, v) in t.iter_mut() {
            path.push(k.get().to_owned());
            walk_value(v, path, &scope, f);
            path.pop();
        }
    } else if let Some(a) = v.as_array_mut() {
        for (i, v) in a.iter_mut().enumerate() {
            path.push(i.to_string());
            walk_value(v, path, scope, f);
            path.pop();
        }
    } else {
        f(path, scope, v);
    }
}
fn documents(s: &Snapshot) -> Result<Vec<Document>, String> {
    let mut paths = Vec::new();
    for category in ["rooms", "items", "spells", "weapons", "enemies", "shops"] {
        paths.extend(
            s.sources(category)
                .filter(|(p, _)| p.extension().is_some_and(|e| e == "toml"))
                .map(|(p, _)| p.clone()),
        );
    }
    for name in [
        "events.toml",
        "actions.toml",
        "rules.toml",
        "game.toml",
        "interactions.toml",
        "transaction_interactions.toml",
    ] {
        if s.source(Path::new(name)).is_some() {
            paths.push(name.into());
        }
    }
    paths
        .into_iter()
        .map(|p| Document::load(s, p, ""))
        .collect()
}
fn visit(d: &mut Document, f: &mut impl FnMut(&[String], &str, &mut Value)) {
    let scope = d.text("id").to_owned();
    for (k, v) in d.doc.iter_mut() {
        walk(v, &mut vec![k.get().to_owned()], &scope, f);
    }
}
fn inventory(s: &Snapshot) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let mut all: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for mut d in documents(s)? {
        let file = d.path.clone();
        visit(&mut d, &mut |p, scope, v| {
            if let (Some(c), Some(id)) = (category(&file, p, scope), v.as_str()) {
                if c == "flags" {
                    all.entry(c).or_default().insert(id.into());
                }
            }
        });
    }
    // Select actual definitions, never ignored optional references to nonexistent IDs.
    let a = Adventure::load(&s.root)?;
    all.entry("flags".into())
        .or_default()
        .extend(a.game.world.rules.flags.keys().cloned());
    all.insert(
        "counters".into(),
        a.game.world.rules.counters.keys().cloned().collect(),
    );
    all.insert(
        "actions".into(),
        a.game.world.actions.iter().map(|a| a.id.clone()).collect(),
    );
    all.insert(
        "rooms".into(),
        a.game.world.rooms.iter().map(|r| r.id.clone()).collect(),
    );
    all.insert(
        "items".into(),
        a.game.world.items.iter().map(|i| i.id.clone()).collect(),
    );
    all.insert(
        "events".into(),
        a.game
            .world
            .events
            .events
            .iter()
            .map(|e| e.id.clone())
            .collect(),
    );
    all.insert(
        "timers".into(),
        a.game.world.events.timers.keys().cloned().collect(),
    );
    all.insert(
        "quests".into(),
        a.game.world.events.quests.keys().cloned().collect(),
    );
    all.insert(
        "journal".into(),
        a.game.world.events.journal.keys().cloned().collect(),
    );
    all.insert(
        "endings".into(),
        a.game.world.events.endings.keys().cloned().collect(),
    );
    all.insert("spells".into(), a.rpg.abilities.keys().cloned().collect());
    all.insert(
        "enemies".into(),
        a.encounters.enemies.keys().cloned().collect(),
    );
    all.insert("shops".into(), a.shops.shops.keys().cloned().collect());
    all.insert(
        "currencies".into(),
        a.wallet.currencies.keys().cloned().collect(),
    );
    for shop in a.shops.shops.values() {
        all.insert(
            format!("offers:{}", shop.id),
            shop.offers.iter().map(|o| o.id.clone()).collect(),
        );
    }
    all.retain(|_, ids| !ids.is_empty());
    Ok(all)
}
pub(super) fn plan(
    s: &Snapshot,
    kind: &str,
    old: &str,
    new: &str,
) -> Result<(BTreeMap<PathBuf, String>, Vec<String>), String> {
    super::validate_id(new)?;
    let ids = inventory(s)?;
    let ids = ids
        .get(kind)
        .ok_or("Unsupported category or no definitions")?;
    if !ids.contains(old) {
        return Err("Unknown original ID".into());
    }
    if ids.contains(new) {
        return Err("New ID collides with an existing definition/reference".into());
    }
    let mut changes = BTreeMap::new();
    let mut refs = Vec::new();
    for mut d in documents(s)? {
        if (d.path == Path::new("rules.toml") && matches!(kind, "flags" | "counters"))
            || (d.path == Path::new("events.toml")
                && matches!(kind, "timers" | "quests" | "journal" | "endings"))
        {
            if let Some(entry) = d.doc.get_mut(kind) {
                let changed = if let Some(table) = entry.as_table_mut() {
                    if let Some((key, value)) = table.remove_entry(old) {
                        table.insert_formatted(
                            &toml_edit::Key::new(new).with_leaf_decor(key.leaf_decor().clone()),
                            value,
                        );
                        true
                    } else {
                        false
                    }
                } else if let Some(table) = entry.as_inline_table_mut() {
                    if let Some((key, value)) = table.remove_entry(old) {
                        table.insert_formatted(
                            &toml_edit::Key::new(new).with_leaf_decor(key.leaf_decor().clone()),
                            value,
                        );
                        true
                    } else {
                        false
                    }
                } else {
                    return Err("Cannot safely rename this state declaration representation".into());
                };
                if changed {
                    refs.push(format!(
                        "{}: {kind}.{old} -> {kind}.{new}",
                        d.path.display()
                    ));
                }
            }
        }
        if kind == "currencies" && old == "gold" && d.path == Path::new("game.toml") {
            if let Some(t) = d.doc.get_mut("currency").and_then(Item::as_table_like_mut) {
                if !t.contains_key("id") {
                    t.insert("id", toml_edit::value("gold"));
                }
            }
            let count = d
                .doc
                .get("currencies")
                .and_then(Item::as_array)
                .map(|a| a.len())
                .or_else(|| {
                    d.doc
                        .get("currencies")
                        .and_then(Item::as_array_of_tables)
                        .map(|a| a.len())
                })
                .unwrap_or(0);
            for i in 0..count {
                let t = super::rules_ui::entry_mut(&mut d.doc, "currencies", i)?;
                if !t.contains_key("id") {
                    t.insert("id", toml_edit::value("gold"));
                }
            }
        }
        let file = d.path.clone();
        visit(&mut d, &mut |p, scope, v| {
            if category(&file, p, scope).as_deref() == Some(kind) && v.as_str() == Some(old) {
                let mut replacement = Value::from(new);
                *replacement.decor_mut() = v.decor().clone();
                *v = replacement;
                refs.push(format!(
                    "{}: {}: {old} -> {new}",
                    file.display(),
                    p.join(".")
                ));
            }
        });
        if d.changed() {
            changes.insert(d.path.clone(), d.contents());
        }
    }
    if changes.is_empty() {
        return Err("No supported authored references found".into());
    }
    Ok((changes, refs))
}
fn manage<R: BufRead, W: Write>(root: &Path, w: &mut Wizard<'_, R, W>) -> Result<(), String> {
    loop {
        let s = Snapshot::capture(root)?;
        let adventure = Adventure::load(root)?;
        w.heading("RENAME INTERNAL ID")?;
        w.say("Weapons share their physical item's ID: choose items to update that association. Offers are scoped to their shop. Interactions and bindings have no independent IDs. Game identity, filenames, artwork paths, enemy instance IDs and player saves are not migration targets.")?;
        let all = inventory(&s)?;
        let kinds = all
            .keys()
            .map(|k| (k.clone(), k.clone()))
            .collect::<Vec<_>>();
        let Some(kind) = choose(w, "Definition category", &kinds)? else {
            return Ok(());
        };
        let ids = all[&kind]
            .iter()
            .map(|id| {
                let name = match kind.as_str() {
                    "rooms" => adventure.game.world.room(id).map(|r| r.title.as_str()),
                    "items" => adventure.game.world.item(id).map(|i| i.name.as_str()),
                    "spells" => adventure.rpg.abilities.get(id).map(|a| a.name.as_str()),
                    "enemies" => adventure
                        .encounters
                        .enemies
                        .get(id)
                        .map(|e| e.name.as_str()),
                    "shops" => adventure.shops.shops.get(id).map(|s| s.name.as_str()),
                    "currencies" => adventure.wallet.currencies.get(id).map(|c| c.name.as_str()),
                    k if k.starts_with("offers:") => adventure
                        .shops
                        .shops
                        .get(&k[7..])
                        .and_then(|s| s.offers.iter().find(|o| o.id == *id))
                        .map(|o| o.name.as_str()),
                    _ => None,
                }
                .unwrap_or(id);
                (id.clone(), name.to_owned())
            })
            .collect::<Vec<_>>();
        let Some(old) = choose(w, "Current internal ID", &ids)? else {
            continue;
        };
        let Some(new) = w.ask(
            "New internal ID",
            "",
            "Lowercase letters, digits, underscores or dashes. No existing ID can be replaced.",
            super::validate_id,
        )?
        else {
            continue;
        };
        let (changes, refs) = match plan(&s, &kind, &old, &new) {
            Ok(p) => p,
            Err(e) => {
                w.say(&e)?;
                continue;
            }
        };
        w.say(&format!(
            "Definition: {kind} / {old}\nNew ID: {new}\nAffected authored references:\n{}",
            refs.join("\n")
        ))?;
        for (p, text) in &changes {
            super::wizard::DraftFile::text(
                &p.to_string_lossy(),
                text,
                "Exact proposed migration; filenames and prose stay unchanged.",
            )
            .preview(root, w.output)?;
        }
        w.say("WARNING — PLAYER SAVE COMPATIBILITY\nExisting saves may reference the previous ID and fail validation or retain obsolete flags. Existing RORK saves will NOT be changed. Compatibility revisions remain unchanged; authors must deliberately decide when to reject old saves. No automatic save compatibility is promised. Unsupported targets listed above are excluded, not partially renamed.")?;
        match s.validate(&changes) {
            Ok(()) => {
                w.say("Staged full adventure validation PASSED. No authored files changed.")?
            }
            Err(e) => {
                w.say(&format!("Migration refused: {e}"))?;
                continue;
            }
        }
        if confirm(w, "Confirm migration of authored content only?")? {
            match s.commit(&changes) {
                Ok(()) => {
                    w.say("Migration saved. Full validation PASSED. RORK saves unchanged.")?
                }
                Err(e) => w.say(&format!("Migration needs attention: {e}"))?,
            }
        } else {
            w.say("Migration canceled. No authored files changed.")?;
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
        writeln!(w.output, "Migration stopped: {e}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Cursor,
        sync::atomic::{AtomicU64, Ordering},
    };
    struct Fixture(PathBuf);
    impl Fixture {
        fn new(sample: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "lupine-migrate-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fn copy(a: &Path, b: &Path) {
                fs::create_dir(b).unwrap();
                for e in fs::read_dir(a).unwrap() {
                    let e = e.unwrap();
                    if e.path().is_dir() {
                        copy(&e.path(), &b.join(e.file_name()));
                    } else {
                        fs::copy(e.path(), b.join(e.file_name())).unwrap();
                    }
                }
            }
            copy(Path::new(sample), &p);
            Self(p)
        }
        fn rename(&self, kind: &str, old: &str, new: &str) {
            let s = Snapshot::capture(&self.0).unwrap();
            let (c, r) = plan(&s, kind, old, new).unwrap();
            assert!(!r.is_empty());
            s.validate(&c).unwrap();
            s.commit(&c).unwrap();
            Adventure::load(&self.0).unwrap();
            assert!(!inventory(&Snapshot::capture(&self.0).unwrap()).unwrap()[kind].contains(old));
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn migrates_every_rpg_category_without_rewriting_prose_or_saves() {
        let f = Fixture::new("samples/rpg_transactions");
        fs::write(f.0.join("personal.rork"), "private player data").unwrap();
        let p = f.0.join("items/mana_altar.toml");
        let text = fs::read_to_string(&p).unwrap();
        fs::write(&p, format!("# mana_altar is prose\n{text}")).unwrap();
        for (kind, old, new) in [
            ("items", "mana_altar", "altar"),
            ("items", "rusty_dagger", "blade"),
            ("items", "fireball_scroll", "scroll"),
            ("items", "goblin_trophy", "reward"),
            ("rooms", "camp", "base"),
            ("shops", "healer", "services"),
            ("offers:services", "mana", "restore"),
            ("currencies", "gold", "coins"),
            ("spells", "fireball", "flame"),
            ("enemies", "goblin", "guard"),
            ("flags", "camp_permit", "permitted"),
        ] {
            f.rename(kind, old, new);
        }
        assert!(fs::read_to_string(p)
            .unwrap()
            .contains("# mana_altar is prose"));
        assert_eq!(
            fs::read_to_string(f.0.join("personal.rork")).unwrap(),
            "private player data"
        );
        assert_eq!(
            Adventure::load(&f.0)
                .unwrap()
                .game
                .save_compatibility_version,
            1
        );
    }
    #[test]
    fn migrates_puzzle_conditions_and_art_variants_and_preserves_narrative() {
        let f = Fixture::new("story_example");
        f.rename("items", "brass_key", "bronze_key");
        f.rename("rooms", "foyer", "entry");
        f.rename("flags", "door_open", "unlocked");
    }
    #[test]
    fn migration_rejects_collision_invalid_ids_and_external_changes() {
        let f = Fixture::new("samples/rpg_transactions");
        let s = Snapshot::capture(&f.0).unwrap();
        assert!(plan(&s, "items", "mana_altar", "rusty_dagger").is_err());
        assert!(plan(&s, "items", "mana_altar", "../escape").is_err());
        let (c, _) = plan(&s, "rooms", "camp", "base").unwrap();
        s.validate(&c).unwrap();
        let p = f.0.join("rooms/camp.toml");
        let old = fs::read_to_string(&p).unwrap();
        let external = format!("# external\n{old}");
        fs::write(&p, &external).unwrap();
        assert!(s.commit(&c).unwrap_err().contains("externally"));
        assert_eq!(fs::read_to_string(p).unwrap(), external);
    }
    #[test]
    fn canceled_migration_warns_and_preserves_authored_bytes() {
        let f = Fixture::new("samples/rpg_transactions");
        let before = fs::read(f.0.join("items/mana_altar.toml")).unwrap();
        let mut out = Vec::new();
        run(
            &f.0,
            &mut Cursor::new("items\nmana_altar\naltar\nno\n0\n"),
            &mut out,
        )
        .unwrap();
        assert!(String::from_utf8(out)
            .unwrap()
            .contains("Existing RORK saves will NOT be changed"));
        assert_eq!(fs::read(f.0.join("items/mana_altar.toml")).unwrap(), before);
        assert!(!f.0.join(".lupine-room-edit").exists());
    }
    #[test]
    fn implicit_currency_and_inline_bindings_are_schema_aware() {
        let f = Fixture::new("samples/rpg_transactions");
        let p = f.0.join("game.toml");
        let text = fs::read_to_string(&p)
            .unwrap()
            .replace("id = \"gold\"\n", "");
        fs::write(p, text).unwrap();
        fs::write(
            f.0.join("transaction_interactions.toml"),
            "bindings = [{item='mana_altar', shop='healer', offer='mana', success_text='mana'}]\n",
        )
        .unwrap();
        f.rename("currencies", "gold", "coins");
        f.rename("offers:healer", "mana", "restore");
        let a = Adventure::load(&f.0).unwrap();
        assert_eq!(
            a.shops.transaction_bindings["mana_altar"]
                .success_text
                .as_deref(),
            Some("mana")
        );
    }
    #[test]
    fn migration_selector_excludes_ignored_optional_nondefinitions() {
        let f = Fixture::new("samples/rpg_transactions");
        let p = f.0.join("shops/healer.toml");
        let text = fs::read_to_string(&p).unwrap().replace(
            "type = \"restore_mp\"",
            "type = \"restore_mp\"\nitem = \"not_a_definition\"",
        );
        fs::write(p, text).unwrap();
        let s = Snapshot::capture(&f.0).unwrap();
        let all = inventory(&s).unwrap();
        assert!(!all["items"].contains("not_a_definition"));
        assert!(plan(&s, "items", "not_a_definition", "replacement").is_err());
    }
}
