//! Optional, data-driven combat equipment and learned abilities.
//! World abilities share typed rules and may be used without RPG character stats.
use crate::state::GameState;
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
    World,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub mana_cost: u32,
    pub kind: AbilityKind,
    #[serde(default)]
    pub power: u32,
    #[serde(default = "crate::systems::physical")]
    pub damage_type: String,
    #[serde(default)]
    pub on_hit_status: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub requires: Vec<crate::rules::Condition>,
    #[serde(default)]
    pub effects: Vec<crate::rules::Effect>,
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
    pub(crate) fn resolve_ability(&self, query: &str) -> Result<&Ability, String> {
        let mut candidates = self
            .abilities
            .values()
            .map(|a| crate::resolution::Candidate::new(&a.id, &a.name))
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.id.cmp(&b.id));
        let id = crate::resolution::unique(query, &candidates)?;
        self.abilities
            .get(&id)
            .ok_or_else(|| "Unknown ability.".into())
    }
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
            if crate::rules::valid_id(&weapon.damage_type).is_err() {
                return Err(format!("{}: damage_type is empty", path.display()));
            }
            if result.weapons.insert(weapon.item.clone(), weapon).is_some() {
                return Err(format!("{}: duplicate weapon item", path.display()));
            }
        }
        for path in definition_files(&directory.join("spells"))? {
            let input =
                fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let value: toml::Value =
                toml::from_str(&input).map_err(|e| format!("{}: {e}", path.display()))?;
            let ability: Ability =
                crate::rules::decode(&value).map_err(|e| format!("{}: {e}", path.display()))?;
            if !valid_id(&ability.id) {
                return Err(format!(
                    "{}: ability id must be lowercase ASCII",
                    path.display()
                ));
            }
            if ability.name.trim().is_empty()
                || (!matches!(ability.kind, AbilityKind::World) && ability.power == 0)
            {
                return Err(format!(
                    "{}: ability must have a name and positive power",
                    path.display()
                ));
            }
            crate::rules::valid_id(&ability.damage_type)?;
            if !matches!(ability.kind, AbilityKind::World)
                && (ability.target.is_some()
                    || !ability.requires.is_empty()
                    || !ability.effects.is_empty())
            {
                return Err(format!(
                    "{}: target/requires/effects are for kind=world",
                    path.display()
                ));
            }
            if !matches!(ability.kind, AbilityKind::Damage) && ability.on_hit_status.is_some() {
                return Err(format!(
                    "{}: on_hit_status requires kind=damage",
                    path.display()
                ));
            }
            if let Some(scroll) = &ability.learn_from_item {
                if !world.item(scroll).is_some_and(|i| i.portable) {
                    return Err(format!(
                        "{}: learn_from_item '{}' is not a portable item",
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
        if !has_character
            && (!result.weapons.is_empty()
                || result
                    .abilities
                    .values()
                    .any(|a| !matches!(a.kind, AbilityKind::World) || a.mana_cost > 0))
        {
            return Err(
                "Weapons and spells require enabled = true in config/character.fig for HP/MP/AP"
                    .into(),
            );
        }
        Ok(result)
    }

    pub fn learn(&self, world: &World, state: &mut GameState, text: &str) -> String {
        let candidates = crate::resolution::items(world, state)
            .into_iter()
            .filter(|i| {
                self.abilities
                    .values()
                    .any(|a| a.learn_from_item.as_deref() == Some(i.id.as_str()))
            })
            .collect::<Vec<_>>();
        let id = match crate::resolution::unique(text, &candidates) {
            Ok(id) => id,
            Err(message) => return message,
        };
        let ability = self
            .abilities
            .values()
            .find(|a| a.learn_from_item.as_deref() == Some(id.as_str()))
            .expect("resolved teaching item");
        if state.learned_abilities.contains(&ability.id) {
            return format!("You already know {}.", ability.name);
        }
        let Some(item_id) = &ability.learn_from_item else {
            return "This ability cannot be learned from an item.".into();
        };
        if !state.is_carried(item_id) {
            return format!(
                "You must carry the teaching item to learn {}.",
                ability.name
            );
        }
        if let Err(e) = state.consume_one(item_id) {
            return e;
        }
        state.learned_abilities.insert(ability.id.clone());
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
            .filter(|w| state.is_carried(&w.item) && state.is_accessible(&w.item))
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

    pub fn cast_world(
        &self,
        world: &World,
        state: &mut GameState,
        text: &str,
    ) -> Option<Result<String, String>> {
        let (id, target) = text
            .split_once(" on ")
            .map_or((text, None), |(a, b)| (a, Some(b)));
        let ability = self.abilities.get(id)?;
        if !matches!(ability.kind, AbilityKind::World) {
            return None;
        }
        Some((|| {
            if !state.learned_abilities.contains(id) {
                return Err(format!("You haven't learned {}.", ability.name));
            }
            if ability.target.as_deref() != target
                || target.is_some_and(|t| !state.is_accessible(t))
            {
                return Err("This ability requires its authored accessible target.".into());
            }
            if !ability.requires.iter().all(|c| c.met(state)) {
                return Err("The ability's requirements are not met.".into());
            }
            let mut effects = Vec::new();
            if ability.mana_cost > 0 {
                effects.push(crate::rules::Effect::SpendMp(ability.mana_cost));
            }
            effects.extend(ability.effects.clone());
            let lines = crate::rules::apply(world, state, &effects)?;
            Ok(format!(
                "You use {}.{}",
                ability.name,
                if lines.is_empty() {
                    String::new()
                } else {
                    format!("\n{}", lines.join("\n"))
                }
            ))
        })())
    }
    pub fn cast(&self, state: &mut GameState, text: &str) -> String {
        let (name, target) = text
            .split_once(" on ")
            .map_or((text.trim(), None), |(name, target)| {
                (name.trim(), Some(target.trim()))
            });
        let ability = match self.resolve_ability(name) {
            Ok(ability) => ability,
            Err(message) => return message,
        };
        if !state.learned_abilities.contains(&ability.id) {
            return format!("You haven't learned {}.", ability.name);
        }
        let Some(character) = state.character.as_mut() else {
            return "This adventure has no RPG character.".into();
        };
        match ability.kind {
            AbilityKind::World => return "World abilities use the shared world cast path.".into(),
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
            AbilityKind::Damage | AbilityKind::World => unreachable!(),
        }
    }
}
