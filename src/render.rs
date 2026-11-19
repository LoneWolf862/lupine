use crate::state::{GameState, ItemLocation};
use crate::world::{select_description, Room, World};

pub fn describe_room(world: &World, state: &GameState, room: &Room) -> String {
    let exits = room
        .exits
        .iter()
        .map(|(direction, _)| direction.label())
        .collect::<Vec<_>>();
    let objects = world
        .items
        .iter()
        .filter(|item| state.location(&item.id) == Some(ItemLocation::Room(room.id.clone())))
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    let visible = if objects.is_empty() {
        "Objects: none".to_string()
    } else {
        format!("Objects: {}", objects.join(", "))
    };
    // Author-opt-in dynamic placeholder. Never inject stale item names in a
    // description unless an author explicitly writes static item prose.
    let objects_text = if objects.is_empty() {
        "nothing".to_owned()
    } else {
        objects.join(", ")
    };
    let description = select_description(&room.description, &room.descriptions, state)
        .replace("{{objects}}", &objects_text);
    let art = room
        .art_variants
        .iter()
        .find(|variant| variant.when.iter().all(|condition| condition.met(state)))
        .map(|variant| variant.text.as_str())
        .or(room.art.as_deref())
        .filter(|art| !art.is_empty());
    match art {
        Some(art) => format!(
            "{}\n{}\n{}\nExits: {}\n{}",
            room.title,
            description,
            art,
            exits.join(", "),
            visible
        ),
        None => format!(
            "{}\n{}\nExits: {}\n{}",
            room.title,
            description,
            exits.join(", "),
            visible
        ),
    }
}

pub fn help() -> &'static str {
    "Explore: look (l), look <object>, examine <object> (x), inventory (i).\nMove: north/n, south/s, east/e, west/w, up/u, down/d; go/walk/travel/run/head <direction>. 'look' lists exits.\nObjects: take <object|all> (get/grab/pick up), drop <object|all>, use <item> on <target>. Object synonyms depend on the adventure's aliases.\nCombat: attack <enemy> with <weapon> (hit/strike/stab/slash); attack <enemy> asks which weapon.\nSpells: spellbook (spells), learn <scroll>, cast <ability> on/at <enemy|self>, use <ability> on <enemy>.\nOther RPG commands: stats, weapons, enemies, wallet, shop, buy <offer>, sell <offer>.\nManual saves: save camp (a name you choose), saves (list names), load camp. 'save' offers to overwrite the current slot; no asks for a name. With no current slot, it asks for a name. 'save <name>' writes directly, replacing that name. Enter/cancel at the name prompt cancels. No autosaving.\nHelp: help (h). Leave the adventure: quit/exit (q); unsaved progress requires yes/no confirmation.\nSuggestions never execute: type the suggested command to try it. Multiple matches ask you to choose; 0/back/cancel cancels."
}

/// Display terminology never changes the accepted command vocabulary.
pub fn help_with_terms(terms: &crate::config::DisplayTerms) -> String {
    format!(
        "{}\nView your {} with 'spellbook' or 'spells'; lists known {}.",
        help(),
        terms.spellbook,
        terms.spells
    )
}

/// Only advertise optional systems actually available in this adventure.
pub fn help_for_game(config: &crate::config::GameConfig, economy: bool) -> String {
    let base = help()
        .lines()
        .filter(|line| {
            if line.starts_with("Combat:")
                || line.starts_with("Spells:")
                || line.starts_with("Other RPG")
            {
                config.character.is_some()
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if config.character.is_some() {
        format!(
            "{base}\nView your {} with 'spellbook' or 'spells'; lists known {}.",
            config.terminology.display.spellbook, config.terminology.display.spells
        )
    } else if economy {
        format!("{base}\nEconomy: wallet, shop, buy <offer>, sell <offer>.")
    } else {
        base
    }
}
