//! Shared resource definitions, equipment, damage modifiers and bounded statuses.
use crate::{state::GameState, world::World};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
fn hundred() -> u32 {
    100
}
fn one() -> u32 {
    1
}
pub fn physical() -> String {
    "physical".into()
}
#[derive(Debug, Clone, Default)]
pub struct Content {
    pub wallet: crate::wallet::WalletContent,
    pub rpg: crate::rpg_content::RpgContent,
    pub progression: Option<crate::rpg::Progression>,
    pub combat: CombatConfig,
    pub equipment: BTreeMap<String, Equipment>,
    pub statuses: BTreeMap<String, Status>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct File {
    equipment: BTreeMap<String, Equipment>,
    statuses: BTreeMap<String, Status>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Equipment {
    pub slot: String,
    #[serde(default)]
    pub defense: u32,
    #[serde(default)]
    pub resistances: BTreeMap<String, i16>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub name: String,
    pub turns: u32,
    #[serde(default)]
    pub damage: u32,
    #[serde(default = "physical")]
    pub damage_type: String,
    #[serde(default = "hundred")]
    pub attack_percent: u32,
    #[serde(default)]
    pub stun: bool,
}
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Retaliation {
    #[default]
    Combat,
    EveryAction,
    None,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CombatConfig {
    pub minimum_damage: u32,
    pub retaliation: Retaliation,
}
impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            minimum_damage: one(),
            retaliation: Retaliation::Combat,
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub equipped: BTreeMap<String, String>,
    pub statuses: BTreeMap<String, u32>,
}
impl Content {
    pub fn load(root: &Path, w: &World, c: &crate::config::GameConfig) -> Result<Self, String> {
        let path = root.join("systems.toml");
        let f: File = match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => File::default(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        Ok(Self {
            wallet: crate::wallet::WalletContent::load(root, c.character.is_some())?,
            rpg: crate::rpg_content::RpgContent::load(root, w, c.character.is_some())?,
            progression: c.character.clone(),
            combat: c.combat.clone(),
            equipment: f.equipment,
            statuses: f.statuses,
        })
    }
    pub fn validate(&self, w: &World) -> Result<(), String> {
        if self.equipment.len() > 1024 || self.statuses.len() > 128 {
            return Err("systems.toml exceeds 1024 equipment / 128 status definitions".into());
        }
        if self.combat.minimum_damage > 1_000_000 {
            return Err("combat.fig minimum_damage must be 0..1000000".into());
        }
        if self.progression.is_none() && (!self.equipment.is_empty() || !self.statuses.is_empty()) {
            return Err("Equipment and statuses require an enabled character".into());
        }
        for (id, e) in &self.equipment {
            crate::rules::valid_id(&e.slot)?;
            if !w.item(id).is_some_and(|i| i.portable)
                || crate::objects::definition(w, id).stackable
            {
                return Err(format!("Equipment '{id}' must be a unique portable item"));
            }
            validate_resistances(&e.resistances)?;
        }
        for (id, s) in &self.statuses {
            crate::rules::valid_id(id)?;
            crate::rules::valid_id(&s.damage_type)?;
            if s.name.trim().is_empty()
                || s.turns == 0
                || s.turns > 1_000_000
                || s.attack_percent > 1000
            {
                return Err(format!(
                    "Invalid status '{id}': require name, turns 1..1000000, attack_percent 0..1000"
                ));
            }
        }
        for a in self.rpg.abilities.values() {
            if let Some(id) = &a.on_hit_status {
                self.status(id)?;
            }
            if let Some(id) = &a.target {
                if w.item(id).is_none() {
                    return Err(format!("Ability '{}' has unknown target '{id}'", a.id));
                }
            }
            for c in &a.requires {
                c.validate(w)?;
            }
            for e in &a.effects {
                e.validate(w)?;
            }
        }
        Ok(())
    }
    pub fn status(&self, id: &str) -> Result<(), String> {
        if self.statuses.contains_key(id) {
            Ok(())
        } else {
            Err(format!("Unknown status '{id}'"))
        }
    }
    pub fn validate_statuses(&self, s: &BTreeMap<String, u32>) -> Result<(), String> {
        for (id, turns) in s {
            if !self
                .statuses
                .get(id)
                .is_some_and(|d| *turns > 0 && *turns <= d.turns)
            {
                return Err(format!("Invalid saved status '{id}' duration"));
            }
        }
        Ok(())
    }
    pub fn validate_state(&self, s: &GameState) -> Result<(), String> {
        self.validate_statuses(&s.systems.statuses)?;
        if s.character.is_none()
            && (!s.systems.equipped.is_empty() || !s.systems.statuses.is_empty())
        {
            return Err("Equipment/status progress requires a character".into());
        }
        for (slot, id) in &s.systems.equipped {
            if !self.equipment.get(id).is_some_and(|e| &e.slot == slot)
                || !s.is_carried(id)
                || !s.is_accessible(id)
            {
                return Err(format!("Invalid saved equipment '{slot}:{id}'"));
            }
        }
        for enemy in s
            .active_enemies
            .iter()
            .chain(s.room_encounters.values().flatten())
        {
            self.validate_statuses(&enemy.statuses)?;
        }
        self.wallet.validate_balances(s)?;
        for id in &s.learned_abilities {
            let a = self
                .rpg
                .abilities
                .get(id)
                .ok_or_else(|| format!("Unknown learned ability '{id}'"))?;
            if s.character.is_none()
                && (!matches!(a.kind, crate::rpg_content::AbilityKind::World) || a.mana_cost > 0)
            {
                return Err(format!("Learned ability '{id}' requires a character"));
            }
        }
        Ok(())
    }
}
pub fn validate_resistances(r: &BTreeMap<String, i16>) -> Result<(), String> {
    if r.len() > 128 {
        return Err("At most 128 damage resistances".into());
    }
    for (id, v) in r {
        crate::rules::valid_id(id)?;
        if !(-100..=100).contains(v) {
            return Err(format!("Resistance '{id}' must be -100..100 percent"));
        }
    }
    Ok(())
}
pub fn damage(raw: u32, defense: u32, resistance: i16, minimum: u32) -> u32 {
    if resistance >= 100 {
        return 0;
    }
    let raw = raw.saturating_sub(defense);
    (((u64::from(raw) * u64::try_from(100 - i32::from(resistance)).unwrap_or(0) + 50) / 100)
        .min(u64::from(u32::MAX)) as u32)
        .max(minimum)
}
pub fn defense(w: &World, s: &GameState, kind: &str) -> (u32, i16) {
    let mut defense = 0u32;
    let mut resistance = 0i32;
    for id in s.systems.equipped.values() {
        if s.is_carried(id) && s.is_accessible(id) {
            if let Some(e) = w.systems.equipment.get(id) {
                defense = defense.saturating_add(e.defense);
                resistance += i32::from(e.resistances.get(kind).copied().unwrap_or(0));
            }
        }
    }
    (defense, resistance.clamp(-100, 100) as i16)
}
pub fn modified_attack(w: &World, statuses: &BTreeMap<String, u32>, raw: u32) -> u32 {
    statuses
        .keys()
        .filter_map(|id| w.systems.statuses.get(id))
        .fold(raw, |n, s| {
            ((u64::from(n) * u64::from(s.attack_percent) + 50) / 100).min(u64::from(u32::MAX))
                as u32
        })
}
pub fn stunned(w: &World, statuses: &BTreeMap<String, u32>) -> bool {
    statuses
        .keys()
        .any(|id| w.systems.statuses.get(id).is_some_and(|s| s.stun))
}
pub fn prune_equipment(s: &mut GameState) {
    let invalid = s
        .systems
        .equipped
        .iter()
        .filter(|(_, id)| !s.is_carried(id) || !s.is_accessible(id))
        .map(|(slot, _)| slot.clone())
        .collect::<Vec<_>>();
    for slot in invalid {
        s.systems.equipped.remove(&slot);
    }
}
pub fn equip(w: &World, s: &mut GameState, id: &str) -> Result<(), String> {
    if s.character.is_none() {
        return Err("This adventure has no character equipment.".into());
    }
    let e = w
        .systems
        .equipment
        .get(id)
        .ok_or("That item is not equipment.")?;
    if !s.is_carried(id) || !s.is_accessible(id) {
        return Err("Carry and open access to the item before equipping it.".into());
    }
    if s.systems.equipped.get(&e.slot).is_some_and(|v| v == id) {
        return Err("That item is already equipped.".into());
    }
    s.systems.equipped.insert(e.slot.clone(), id.into());
    Ok(())
}
pub fn describe(w: &World, s: &GameState) -> String {
    let mut lines = vec!["Equipment:".into()];
    for (slot, id) in &s.systems.equipped {
        lines.push(format!(
            "  {slot}: {}",
            w.item(id).map_or(id.as_str(), |i| i.name.as_str())
        ));
    }
    if s.systems.equipped.is_empty() {
        lines.push("  none".into());
    }
    lines.push("Statuses:".into());
    for (id, turns) in &s.systems.statuses {
        lines.push(format!(
            "  {}: {turns} turns",
            w.systems
                .statuses
                .get(id)
                .map_or(id.as_str(), |d| d.name.as_str())
        ));
    }
    if s.systems.statuses.is_empty() {
        lines.push("  none".into());
    }
    lines.join("\n")
}
/// One deterministic status tick per successful action; application turns count.
pub fn tick(
    w: &World,
    s: &mut GameState,
    encounters: &crate::encounters::EncounterContent,
) -> (Vec<String>, Vec<crate::encounters::EnemyInstance>) {
    let mut lines = Vec::new();
    for (id, remaining) in s.systems.statuses.clone() {
        if let Some(d) = w.systems.statuses.get(&id) {
            let (armor, resist) = defense(w, s, &d.damage_type);
            let damage = if d.damage == 0 {
                0
            } else {
                damage(d.damage, armor, resist, w.systems.combat.minimum_damage)
            };
            if let Some(c) = &mut s.character {
                c.hp = c.hp.saturating_sub(damage);
            }
            if damage > 0 {
                lines.push(format!("{} deals {damage} damage to you.", d.name));
            }
            if remaining <= 1 {
                s.systems.statuses.remove(&id);
                lines.push(format!("{} wears off.", d.name));
            } else {
                s.systems.statuses.insert(id, remaining - 1);
            }
        }
    }
    for enemy in &mut s.active_enemies {
        for (id, remaining) in enemy.statuses.clone() {
            if let (Some(d), Some(def)) = (
                w.systems.statuses.get(&id),
                encounters.enemies.get(&enemy.enemy_id),
            ) {
                if d.damage > 0 {
                    enemy.current_hp = enemy.current_hp.saturating_sub(damage(
                        d.damage,
                        def.defense,
                        def.resistances.get(&d.damage_type).copied().unwrap_or(0),
                        w.systems.combat.minimum_damage,
                    ));
                    lines.push(format!(
                        "{} affects {} #{}.",
                        d.name, def.name, enemy.instance_id
                    ));
                }
                if remaining <= 1 {
                    enemy.statuses.remove(&id);
                } else {
                    enemy.statuses.insert(id, remaining - 1);
                }
            }
        }
    }
    let mut defeated = Vec::new();
    s.active_enemies.retain(|e| {
        if e.current_hp == 0 {
            defeated.push(e.clone());
            false
        } else {
            true
        }
    });
    (lines, defeated)
}
