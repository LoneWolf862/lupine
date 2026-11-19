//! Data-driven encounter definitions and deterministic sampling. Combat resolution follows in C2.
use crate::state::{GameState, ItemLocation};
use crate::world::World;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enemy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: u32,
    pub max_hp: u32,
    #[serde(default = "default_ap")]
    pub attack_power: u32,
    #[serde(default)]
    pub defense: u32,
    #[serde(default)]
    pub experience: u64,
    #[serde(default)]
    pub loot: Vec<EnemyLoot>,
    #[serde(default)]
    pub currency_rewards: Vec<EnemyCurrencyReward>,
    #[serde(default)]
    pub attacks: Vec<EnemyAttack>,
}
/// Independent percent chance of dropping a unique world item on defeat.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyLoot {
    pub item: String,
    pub chance: u32,
}
/// Independent roll for one configured currency. Awards are stored as balances.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyCurrencyReward {
    pub currency: String,
    pub chance: u32,
    pub min: u64,
    pub max: u64,
}
fn default_ap() -> u32 {
    100
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyAttack {
    pub name: String,
    pub damage: u32,
    pub weight: u32,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterTable {
    /// What happens when a player leaves this room. Defaults to legacy behavior.
    #[serde(default)]
    pub persistence: EncounterPersistence,
    /// Percent chance on every successful room entry (0–100).
    pub chance: u32,
    #[serde(default = "default_one")]
    pub min_enemies: u32,
    #[serde(default = "default_one")]
    pub max_enemies: u32,
    #[serde(default)]
    pub enemies: Vec<EncounterEntry>,
}
/// `temporary` rerolls on entry; `room` retains living enemies between visits;
/// `unique` retains living enemies and permanently clears once defeated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterPersistence {
    #[default]
    Temporary,
    Room,
    Unique,
}
fn default_one() -> u32 {
    1
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterEntry {
    pub enemy: String,
    pub weight: u32,
    #[serde(default)]
    pub requires: Vec<EncounterCondition>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterCondition {
    pub flag_set: Option<String>,
    pub flag_unset: Option<String>,
}
impl EncounterCondition {
    fn validate(&self) -> Result<(), String> {
        match (&self.flag_set, &self.flag_unset) {
            (Some(s), None) | (None, Some(s)) if !s.is_empty() => Ok(()),
            _ => Err(
                "Each encounter condition needs exactly one nonempty flag_set or flag_unset".into(),
            ),
        }
    }
    fn met(&self, state: &GameState) -> bool {
        self.flag_set
            .as_ref()
            .is_none_or(|f| state.flags.contains(f))
            && self
                .flag_unset
                .as_ref()
                .is_none_or(|f| !state.flags.contains(f))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyInstance {
    pub instance_id: u64,
    pub enemy_id: String,
    pub current_hp: u32,
}
#[derive(Debug, Clone, Default)]
pub struct EncounterContent {
    pub enemies: HashMap<String, Enemy>,
    pub tables: HashMap<String, EncounterTable>,
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}
impl EncounterContent {
    /// Loot is authored as ordinary portable game items; reject unknown or fixed items.
    pub fn validate_loot(&self, world: &World) -> Result<(), String> {
        for enemy in self.enemies.values() {
            for drop in &enemy.loot {
                let Some(item) = world.item(&drop.item) else {
                    return Err(format!(
                        "Enemy '{}' loot refers to unknown item '{}'",
                        enemy.id, drop.item
                    ));
                };
                if !item.portable {
                    return Err(format!(
                        "Enemy '{}' loot item '{}' must be portable",
                        enemy.id, drop.item
                    ));
                }
            }
        }
        Ok(())
    }

    /// Hide loot-only items at the start of a new session. They become ordinary
    /// room items when dropped; subsequent saves use standard RORK item locations.
    pub fn prepare_loot(&self, state: &mut GameState) {
        for enemy in self.enemies.values() {
            for drop in &enemy.loot {
                state.set_location(drop.item.clone(), ItemLocation::Consumed);
            }
        }
    }

    pub fn load(root: &Path, room_ids: &[String], rpg_enabled: bool) -> Result<Self, String> {
        let mut result = Self::default();
        let enemy_dir = root.join("enemies");
        if enemy_dir.exists() {
            let mut paths = fs::read_dir(&enemy_dir)
                .map_err(|e| format!("{}: {e}", enemy_dir.display()))?
                .map(|e| e.map(|d| d.path()).map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            paths.sort();
            for path in paths {
                if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }
                if !path.is_file() {
                    return Err(format!("{}: not a regular file", path.display()));
                }
                let src =
                    fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let enemy: Enemy =
                    toml::from_str(&src).map_err(|e| format!("{}: {e}", path.display()))?;
                if !valid_id(&enemy.id)
                    || enemy.max_hp == 0
                    || enemy.level == 0
                    || enemy.attack_power == 0
                {
                    return Err(format!(
                        "{}: enemy needs valid id, positive level/max_hp/attack_power",
                        path.display()
                    ));
                }
                if enemy.name.trim().is_empty() {
                    return Err(format!("{}: enemy name is empty", path.display()));
                }
                if enemy
                    .attacks
                    .iter()
                    .any(|a| a.name.trim().is_empty() || a.damage == 0 || a.weight == 0)
                {
                    return Err(format!(
                        "{}: enemy attacks need name, positive damage and weight",
                        path.display()
                    ));
                }
                if enemy
                    .loot
                    .iter()
                    .any(|drop| !valid_id(&drop.item) || drop.chance > 100)
                {
                    return Err(format!(
                        "{}: loot needs a valid item id and chance 0..=100",
                        path.display()
                    ));
                }
                let total: u64 = enemy.attacks.iter().map(|a| u64::from(a.weight)).sum();
                if total > u64::from(u32::MAX) {
                    return Err(format!("{}: attack weights overflow", path.display()));
                }
                if result.enemies.insert(enemy.id.clone(), enemy).is_some() {
                    return Err(format!("{}: duplicate enemy id", path.display()));
                }
            }
        }
        let room_dir = root.join("rooms");
        let defined: HashSet<_> = room_ids.iter().map(String::as_str).collect();
        for entry in fs::read_dir(&room_dir).map_err(|e| format!("{}: {e}", room_dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") || !path.is_file() {
                continue;
            }
            let raw = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let value: toml::Value =
                toml::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
            let Some(table) = value.get("encounters") else {
                continue;
            };
            let room_id = value
                .get("id")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| format!("{}: missing room id", path.display()))?;
            if !defined.contains(room_id) {
                return Err(format!("{}: unknown room id {room_id}", path.display()));
            }
            if !rpg_enabled {
                return Err(format!(
                    "{}: encounter tables require enabled = true in config/character.fig",
                    path.display()
                ));
            }
            let encounters: EncounterTable = table
                .clone()
                .try_into()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if encounters.chance > 100
                || encounters.min_enemies == 0
                || encounters.max_enemies > 12
                || encounters.max_enemies < encounters.min_enemies
            {
                return Err(format!(
                    "{}: chance must be 0–100 and group size must be 1–12 with min <= max",
                    path.display()
                ));
            }
            if encounters.enemies.is_empty() {
                return Err(format!(
                    "{}: encounter table requires entries",
                    path.display()
                ));
            }
            let mut total: u64 = 0;
            for e in &encounters.enemies {
                if !result.enemies.contains_key(&e.enemy) {
                    return Err(format!("{}: unknown enemy '{}'", path.display(), e.enemy));
                }
                if e.weight == 0 {
                    return Err(format!(
                        "{}: encounter weight must be positive",
                        path.display()
                    ));
                }
                total += u64::from(e.weight);
                for c in &e.requires {
                    c.validate()
                        .map_err(|msg| format!("{}: {msg}", path.display()))?;
                }
            }
            if total > u64::from(u32::MAX) {
                return Err(format!("{}: encounter weights overflow", path.display()));
            }
            if result
                .tables
                .insert(room_id.to_owned(), encounters)
                .is_some()
            {
                return Err(format!("{}: duplicate encounter table", path.display()));
            }
        }
        Ok(result)
    }
    /// Preserve surviving enemies on exit, or discard a temporary encounter.
    /// The engine calls this only after a *successful* room change.
    pub fn leave_room(&self, room_id: &str, state: &mut GameState) {
        let active = std::mem::take(&mut state.active_enemies);
        match self.tables.get(room_id).map(|table| table.persistence) {
            Some(EncounterPersistence::Room | EncounterPersistence::Unique)
                if !active.is_empty() =>
            {
                state.room_encounters.insert(room_id.to_owned(), active);
            }
            _ => {
                state.room_encounters.remove(room_id);
            }
        }
    }

    /// Restore a remembered encounter before considering a new spawn.
    /// No random roll is made for a completed unique encounter.
    pub fn enter_room(&self, room_id: &str, state: &mut GameState, rng: &mut impl RandomSource) {
        if let Some(remembered) = state.room_encounters.remove(room_id) {
            state.active_enemies = remembered;
            return;
        }
        if self.tables.get(room_id).is_some_and(|table| {
            table.persistence == EncounterPersistence::Unique
                && state.completed_unique_encounters.contains(room_id)
        }) {
            state.active_enemies.clear();
            return;
        }
        let mut next_id = state.next_enemy_id;
        let rolled = self.roll(room_id, state, rng, &mut next_id);
        state.next_enemy_id = next_id;
        state.active_enemies = rolled;
    }

    /// Final victory in a unique room permanently prevents respawning.
    /// Only call after a real enemy defeat, never after a failed roll.
    pub fn record_defeat(&self, state: &mut GameState) {
        if state.active_enemies.is_empty()
            && self
                .tables
                .get(&state.current_room)
                .is_some_and(|table| table.persistence == EncounterPersistence::Unique)
        {
            state
                .completed_unique_encounters
                .insert(state.current_room.clone());
        }
    }

    /// Verify story-specific encounters before replacing a live session on load.
    pub fn validate_restored(&self, state: &GameState) -> Result<(), String> {
        let mut seen = HashSet::new();
        let check_enemy = |instance: &EnemyInstance, seen: &mut HashSet<u64>| {
            let enemy = self
                .enemies
                .get(&instance.enemy_id)
                .ok_or_else(|| format!("save refers to unknown enemy '{}'", instance.enemy_id))?;
            if instance.current_hp > enemy.max_hp || instance.current_hp == 0 {
                return Err(format!("save enemy '{}' has invalid HP", instance.enemy_id));
            }
            if !seen.insert(instance.instance_id) {
                return Err("save contains duplicate enemy instance IDs across rooms".into());
            }
            Ok(())
        };
        if !state.active_enemies.is_empty() && !self.tables.contains_key(&state.current_room) {
            return Err("save contains an active encounter without a room table".into());
        }
        for enemy in &state.active_enemies {
            check_enemy(enemy, &mut seen)?;
        }
        for (room, enemies) in &state.room_encounters {
            if room == &state.current_room {
                return Err("save stores the current room encounter twice".into());
            }
            let Some(table) = self.tables.get(room) else {
                return Err(format!("save references unknown encounter room '{room}'"));
            };
            if table.persistence == EncounterPersistence::Temporary
                || enemies.is_empty()
                || enemies.len() > 12
            {
                return Err(format!("save has invalid remembered encounter in '{room}'"));
            }
            for enemy in enemies {
                check_enemy(enemy, &mut seen)?;
            }
        }
        for room in &state.completed_unique_encounters {
            let Some(table) = self.tables.get(room) else {
                return Err(format!(
                    "save completed encounter refers to unknown room '{room}'"
                ));
            };
            if table.persistence != EncounterPersistence::Unique
                || state.room_encounters.contains_key(room)
                || (room == &state.current_room && !state.active_enemies.is_empty())
            {
                return Err(format!(
                    "save has conflicting unique-encounter state for '{room}'"
                ));
            }
        }
        Ok(())
    }

    /// Caller injects RNG; allows deterministic tests and keeps sampling isolated.
    pub fn roll(
        &self,
        room_id: &str,
        state: &GameState,
        rng: &mut impl RandomSource,
        next_id: &mut u64,
    ) -> Vec<EnemyInstance> {
        let Some(table) = self.tables.get(room_id) else {
            return Vec::new();
        };
        if table.chance == 0 || rng.below(100) >= table.chance {
            return Vec::new();
        }
        let choices = table
            .enemies
            .iter()
            .filter(|e| e.requires.iter().all(|c| c.met(state)))
            .collect::<Vec<_>>();
        let total: u64 = choices.iter().map(|e| u64::from(e.weight)).sum();
        if total == 0 {
            return Vec::new();
        }
        let count = table.min_enemies + rng.below(table.max_enemies - table.min_enemies + 1);
        let mut spawned = Vec::new();
        for _ in 0..count {
            let mut ticket = u64::from(rng.below(total as u32));
            let mut chosen = choices[0];
            for candidate in &choices {
                if ticket < u64::from(candidate.weight) {
                    chosen = candidate;
                    break;
                }
                ticket -= u64::from(candidate.weight);
            }
            if let Some(enemy) = self.enemies.get(&chosen.enemy) {
                spawned.push(EnemyInstance {
                    instance_id: *next_id,
                    enemy_id: enemy.id.clone(),
                    current_hp: enemy.max_hp,
                });
                *next_id = next_id.saturating_add(1);
            }
        }
        spawned
    }
    pub fn describe(&self, instances: &[EnemyInstance]) -> String {
        if instances.is_empty() {
            return "No enemies are currently visible.".into();
        }
        let mut result = String::from("Enemies:\n");
        for instance in instances {
            if let Some(enemy) = self.enemies.get(&instance.enemy_id) {
                // Rounded to coarse 10-step display; never print exact HP.
                let full = u64::from(instance.current_hp)
                    .saturating_mul(10)
                    .saturating_add(u64::from(enemy.max_hp) / 2)
                    / u64::from(enemy.max_hp);
                let segments = full.clamp(1, 10) as usize;
                result.push_str(&format!(
                    "  {} #{} [{}{}]\n",
                    enemy.name,
                    instance.instance_id,
                    "-".repeat(segments),
                    " ".repeat(10 - segments)
                ));
            }
        }
        result.trim_end().to_owned()
    }
}
pub trait RandomSource {
    fn below(&mut self, limit: u32) -> u32;
}
/// Small dependency-free PRNG; seed from OS time for play, explicit seed for tests.
#[derive(Debug)]
pub struct EncounterRng(pub u64);
impl EncounterRng {
    pub fn from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        Self(nanos ^ u64::from(std::process::id()))
    }
}
impl RandomSource for EncounterRng {
    fn below(&mut self, limit: u32) -> u32 {
        if limit == 0 {
            return 0;
        }
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % u64::from(limit)) as u32
    }
}

#[cfg(test)]
mod health_bar_regressions {
    use super::*;
    #[test]
    fn living_enemy_always_has_visible_health() {
        let enemy = Enemy {
            id: "test".to_string(),
            name: "Test Enemy".to_string(),
            description: "A test creature".to_string(),
            level: 1,
            max_hp: 100,
            attack_power: 100,
            defense: 0,
            experience: 0,
            loot: vec![],
            currency_rewards: vec![],
            attacks: vec![],
        };
        let mut content = EncounterContent::default();
        content.enemies.insert(enemy.id.clone(), enemy);
        let out = content.describe(&[EnemyInstance {
            instance_id: 1,
            enemy_id: "test".to_string(),
            current_hp: 1,
        }]);
        assert!(out.contains("[-         ]"), "{out}");
        assert!(!out.contains("[          ]"), "{out}");
    }
}
