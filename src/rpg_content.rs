//! Optional, data-driven combat equipment and learned abilities.
//! Stage B does not spawn enemies or inflict combat damage: that is Stage C.
use crate::state::{GameState, ItemLocation};
use crate::world::World;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weapon {
    pub item: String,
    pub damage: u32,
    #[serde(default = "default_damage_type")]
    pub damage_type: String,
}
fn default_damage_type() -> String {
    "physical".into()
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityKind {
    Damage,
    Heal,
    RestoreMana,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub mana_cost: u32,
    pub kind: AbilityKind,
    pub power: u32,
    /// Optional physical scroll/item that teaches this ability on `learn <item>`.
    pub learn_from_item: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RpgContent {
    pub weapons: HashMap<String, Weapon>,
    pub abilities: HashMap<String, Ability>,
}

fn definition_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    if !dir.is_dir() {
        return Err(format!("{} must be a directory", dir.display()));
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") && path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

impl RpgContent {
    /// Load optional definitions. Empty directories and absent directories are valid.
    pub fn load(directory: &Path, world: &World, has_character: bool) -> Result<Self, String> {
        let mut result = Self::default();
        let mut scrolls = HashSet::new();
        for path in definition_files(&directory.join("weapons"))? {
            let input =
                fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let weapon: Weapon =
                toml::from_str(&input).map_err(|e| format!("{}: {e}", path.display()))?;
            if !valid_id(&weapon.item) || world.item(&weapon.item).is_none() {
                return Err(format!(
                    "{}: weapon item '{}' is not a defined item",
                    path.display(),
                    weapon.item
                ));
            }
            if weapon.damage == 0 {
                return Err(format!(
                    "{}: weapon damage must be positive",
                    path.display()
                ));
            }
            if weapon.damage_type.trim().is_empty() {
                return Err(format!("{}: damage_type is empty", path.display()));
            }
            if result.weapons.insert(weapon.item.clone(), weapon).is_some() {
                return Err(format!("{}: duplicate weapon item", path.display()));
            }
        }
        for path in definition_files(&directory.join("spells"))? {
            let input =
                fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let ability: Ability =
                toml::from_str(&input).map_err(|e| format!("{}: {e}", path.display()))?;
            if !valid_id(&ability.id) {
                return Err(format!(
                    "{}: ability id must be lowercase ASCII",
                    path.display()
                ));
            }
            if ability.name.trim().is_empty() || ability.power == 0 {
                return Err(format!(
                    "{}: ability must have a name and positive power",
                    path.display()
                ));
            }
            if let Some(scroll) = &ability.learn_from_item {
                if world.item(scroll).is_none() {
                    return Err(format!(
                        "{}: learn_from_item '{}' is not an item",
                        path.display(),
                        scroll
                    ));
                }
                if !scrolls.insert(scroll.clone()) {
                    return Err(format!(
                        "{}: an item cannot teach multiple abilities",
                        path.display()
                    ));
                }
            }
            if result
                .abilities
                .insert(ability.id.clone(), ability)
                .is_some()
            {
                return Err(format!("{}: duplicate ability id", path.display()));
            }
        }
        if !has_character && (!result.weapons.is_empty() || !result.abilities.is_empty()) {
            return Err(
                "Weapons and spells require enabled = true in config/character.fig for HP/MP/AP"
                    .into(),
            );
        }
        Ok(result)
    }

    pub fn learn(&self, world: &World, state: &mut GameState, text: &str) -> String {
        let Some(ability) = self.abilities.values().find(|a| {
            a.learn_from_item
                .as_ref()
                .and_then(|id| world.item(id))
                .is_some_and(|item| {
                    item.id.eq_ignore_ascii_case(text)
                        || item.name.eq_ignore_ascii_case(text)
                        || item
                            .aliases
                            .iter()
                            .any(|alias| alias.eq_ignore_ascii_case(text))
                })
        }) else {
            return format!("No ability can be learned from '{text}'.");
        };
        if state.learned_abilities.contains(&ability.id) {
            return format!("You already know {}.", ability.name);
        }
        let Some(item_id) = &ability.learn_from_item else {
            return "This ability cannot be learned from an item.".into();
        };
        if state.location(item_id) != Some(ItemLocation::Inventory) {
            return format!(
                "You must carry the teaching item to learn {}.",
                ability.name
            );
        }
        state.learned_abilities.insert(ability.id.clone());
        // Scroll consumed permanently, including after manual save/load.
        state
            .item_locations
            .insert(item_id.clone(), ItemLocation::Consumed);
        format!("You learn {}. The teaching item is consumed.", ability.name)
    }

    pub fn spellbook(&self, state: &GameState, terms: &crate::config::DisplayTerms) -> String {
        let mut spells = state
            .learned_abilities
            .iter()
            .filter_map(|id| self.abilities.get(id))
            .collect::<Vec<_>>();
        spells.sort_by(|a, b| a.name.cmp(&b.name));
        if spells.is_empty() {
            return format!("Your {} is empty.", terms.spellbook);
        }
        let mut output = format!("Known {}:\n", terms.spells);
        for spell in spells {
            output.push_str(&format!(
                "  {} ({} MP) — {}\n",
                spell.name, spell.mana_cost, spell.description
            ));
        }
        output.trim_end().to_owned()
    }

    pub fn weapons(&self, world: &World, state: &GameState) -> String {
        let Some(character) = &state.character else {
            return "This adventure has no character combat statistics.".into();
        };
        let mut weapons = self
            .weapons
            .values()
            .filter(|w| state.location(&w.item) == Some(ItemLocation::Inventory))
            .collect::<Vec<_>>();
        weapons.sort_by(|a, b| a.item.cmp(&b.item));
        if weapons.is_empty() {
            return "You carry no usable weapons.".into();
        }
        let mut output = String::from("Carried weapons:\n");
        for weapon in weapons {
            let name = world
                .item(&weapon.item)
                .map_or(weapon.item.as_str(), |item| item.name.as_str());
            output.push_str(&format!(
                "  {}: {} {} damage at AP {:.2}\n",
                name,
                character.weapon_damage(weapon.damage),
                weapon.damage_type,
                f64::from(character.ap) / 100.0
            ));
        }
        output.trim_end().to_owned()
    }

    pub fn cast(&self, state: &mut GameState, text: &str) -> String {
        let (name, target) = text
            .split_once(" on ")
            .map_or((text.trim(), None), |(name, target)| {
                (name.trim(), Some(target.trim()))
            });
        let Some(ability) = self
            .abilities
            .values()
            .find(|a| a.id.eq_ignore_ascii_case(name) || a.name.eq_ignore_ascii_case(name))
        else {
            return format!("Unknown ability '{name}'.");
        };
        if !state.learned_abilities.contains(&ability.id) {
            return format!("You haven't learned {}.", ability.name);
        }
        let Some(character) = state.character.as_mut() else {
            return "This adventure has no RPG character.".into();
        };
        match ability.kind {
            AbilityKind::Damage => return "Damage spells require an enemy target. Try 'cast <spell> on <enemy>'; no MP spent.".into(),
            AbilityKind::Heal => if target.is_some_and(|t| !t.eq_ignore_ascii_case("self") && !t.eq_ignore_ascii_case("me")) {
                return "Healing currently targets yourself only.".into();
            },
            AbilityKind::RestoreMana => if target.is_some_and(|t| !t.eq_ignore_ascii_case("self") && !t.eq_ignore_ascii_case("me")) {
                return "Mana restoration currently targets yourself only.".into();
            },
        }
        if character.mp < ability.mana_cost {
            return format!("Not enough MP to cast {}.", ability.name);
        }
        if matches!(ability.kind, AbilityKind::Heal) && character.hp == character.max_hp {
            return "Your HP is already full; no MP spent.".into();
        }
        if matches!(ability.kind, AbilityKind::RestoreMana) && character.mp == character.max_mp {
            return "Your MP is already full; no MP spent.".into();
        }
        character.spend_mana(ability.mana_cost);
        match ability.kind {
            AbilityKind::Heal => {
                character.heal(ability.power);
                format!(
                    "You cast {} and recover HP. HP: {}/{} | MP: {}/{}",
                    ability.name, character.hp, character.max_hp, character.mp, character.max_mp
                )
            }
            AbilityKind::RestoreMana => {
                character.restore_mana(ability.power);
                format!(
                    "You use {} to restore mana. MP: {}/{}",
                    ability.name, character.mp, character.max_mp
                )
            }
            AbilityKind::Damage => unreachable!(),
        }
    }
}
