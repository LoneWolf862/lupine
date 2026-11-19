//! Deterministic physical/spell combat, typed damage and atomic shared rewards.
use crate::encounters::{EncounterContent, EnemyInstance, RandomSource};
use crate::rpg::Progression;
use crate::rpg_content::{AbilityKind, RpgContent};
use crate::state::{GameState, ItemLocation};
use crate::world::World;

pub struct CombatAction {
    pub message: String,
    pub turn_spent: bool,
    /// Removed enemy instance, available for a one-time XP/loot payout.
    pub defeated: Option<EnemyInstance>,
}
impl CombatAction {
    fn free(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            turn_spent: false,
            defeated: None,
        }
    }
    fn spent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            turn_spent: true,
            defeated: None,
        }
    }
}

/// Resolve names, ids and optional #instance number. Ambiguous names never pick a target.
fn target_index(
    encounters: &EncounterContent,
    active: &[EnemyInstance],
    target: &str,
) -> Result<usize, String> {
    let query = target.trim();
    if query.is_empty() {
        return Err("Specify an enemy target.".into());
    }
    let (name, number) = match query.rsplit_once('#') {
        Some((name, num)) if num.trim().parse::<u64>().is_ok() => {
            (name.trim(), num.trim().parse::<u64>().ok())
        }
        _ => (query, None),
    };
    let has_exact = active.iter().any(|instance| {
        encounters
            .enemies
            .get(&instance.enemy_id)
            .is_some_and(|d| d.name.eq_ignore_ascii_case(name) || d.id.eq_ignore_ascii_case(name))
    });
    let found: Vec<usize> = active
        .iter()
        .enumerate()
        .filter_map(|(index, instance)| {
            let definition = encounters.enemies.get(&instance.enemy_id)?;
            let matches = (name.is_empty() && number.is_some())
                || definition.name.eq_ignore_ascii_case(name)
                || definition.id.eq_ignore_ascii_case(name)
                || (!has_exact
                    && definition
                        .name
                        .split_whitespace()
                        .any(|part| part.eq_ignore_ascii_case(name)));
            (matches && number.is_none_or(|n| n == instance.instance_id)).then_some(index)
        })
        .collect();
    match found.len() {
        0 => Err(format!("No visible enemy matches '{query}'.")),
        1 => Ok(found[0]),
        _ => Err(format!(
            "Multiple enemies match '{query}'. Use a numbered name, such as '{} #{}'.",
            encounters
                .enemies
                .get(&active[found[0]].enemy_id)
                .map_or("Enemy", |e| e.name.as_str()),
            active[found[0]].instance_id
        )),
    }
}

fn hit(
    world: &World,
    damage_type: &str,
    encounters: &EncounterContent,
    active: &mut Vec<EnemyInstance>,
    index: usize,
    raw: u32,
    label: &str,
) -> CombatAction {
    let Some(definition) = encounters.enemies.get(&active[index].enemy_id) else {
        return CombatAction::free("Unknown enemy definition.");
    };
    let damage = crate::systems::damage(
        raw,
        definition.defense,
        definition
            .resistances
            .get(damage_type)
            .copied()
            .unwrap_or(0),
        world.systems.combat.minimum_damage,
    );
    active[index].current_hp = active[index].current_hp.saturating_sub(damage);
    let name = format!("{} #{}", definition.name, active[index].instance_id);
    let mut msg = format!("{label} {name} for {damage} damage.");
    if active[index].current_hp == 0 {
        let defeated = active.remove(index);
        msg.push_str(&format!(" {name} is defeated."));
        return CombatAction {
            message: msg,
            turn_spent: true,
            defeated: Some(defeated),
        };
    }
    CombatAction::spent(msg)
}

/// `attack <enemy> with <weapon>`; aliases/IDs for the inventory item are supported.
pub fn physical_attack(
    world: &World,
    content: &RpgContent,
    encounters: &EncounterContent,
    state: &mut GameState,
    active: &mut Vec<EnemyInstance>,
    input: &str,
) -> CombatAction {
    let Some((target, weapon_name)) = input.rsplit_once(" with ") else {
        return CombatAction::free("Try 'attack <enemy> with <weapon>'.");
    };
    if state.character.as_ref().is_none_or(|c| c.hp == 0) {
        return CombatAction::free("You cannot fight without a living RPG character.");
    }
    let candidates = crate::resolution::items(world, state)
        .into_iter()
        .filter(|i| content.weapons.contains_key(&i.id))
        .collect::<Vec<_>>();
    let id = match crate::resolution::unique(weapon_name.trim(), &candidates) {
        Ok(id) => id,
        Err(message) => return CombatAction::free(message),
    };
    let weapon = &content.weapons[&id];
    let item = world.item(&id).expect("resolved weapon item");
    if !state.is_carried(&weapon.item) || !state.is_accessible(&weapon.item) {
        return CombatAction::free(format!(
            "You must carry and be able to reach {} to attack with it.",
            item.name
        ));
    }
    let index = match target_index(encounters, active, target) {
        Ok(i) => i,
        Err(m) => return CombatAction::free(m),
    };
    let raw = state
        .character
        .as_ref()
        .map_or(0, |c| c.weapon_damage(weapon.damage));
    if crate::systems::stunned(world, &state.systems.statuses) {
        return CombatAction::spent("You are stunned and cannot attack.");
    }
    let raw = crate::systems::modified_attack(world, &state.systems.statuses, raw);
    hit(
        world,
        &weapon.damage_type,
        encounters,
        active,
        index,
        raw,
        "You strike",
    )
}

/// `cast <damage spell> on <enemy>`; unsuccessful casts consume no mana or turn.
pub fn offensive_spell(
    world: &World,
    content: &RpgContent,
    encounters: &EncounterContent,
    state: &mut GameState,
    active: &mut Vec<EnemyInstance>,
    input: &str,
) -> CombatAction {
    let Some((spell_name, target)) = input.rsplit_once(" on ") else {
        return CombatAction::free("Try 'cast <spell> on <enemy>'.");
    };
    let ability = match content.resolve_ability(spell_name.trim()) {
        Ok(ability) => ability,
        Err(message) => return CombatAction::free(message),
    };
    if !matches!(ability.kind, AbilityKind::Damage) {
        // Non-offensive spells can still use the previous cast command (on self).
        return CombatAction::free("This spell is not an offensive ability. Use 'cast <ability>' for healing or mana restoration.");
    }
    if !state.learned_abilities.contains(&ability.id) {
        return CombatAction::free(format!("You haven't learned {}.", ability.name));
    }
    let Some(character) = state.character.as_ref() else {
        return CombatAction::free("This adventure has no RPG character.");
    };
    if character.hp == 0 {
        return CombatAction::free("You cannot cast while defeated.");
    }
    if character.mp < ability.mana_cost {
        return CombatAction::free(format!("Not enough MP to cast {}.", ability.name));
    }
    let index = match target_index(encounters, active, target) {
        Ok(i) => i,
        Err(m) => return CombatAction::free(m),
    };
    if crate::systems::stunned(world, &state.systems.statuses) {
        return CombatAction::spent("You are stunned and cannot cast.");
    }
    if let Some(id) = &ability.on_hit_status {
        if let Some(status) = world.systems.statuses.get(id) {
            active[index].statuses.insert(id.clone(), status.turns);
        }
    }
    let _ = state
        .character
        .as_mut()
        .is_some_and(|c| c.spend_mana(ability.mana_cost));
    hit(
        world,
        &ability.damage_type,
        encounters,
        active,
        index,
        crate::systems::modified_attack(world, &state.systems.statuses, ability.power),
        &format!("You cast {} on", ability.name),
    )
}

/// Every living enemy gets one weighted attack after a valid offensive player turn.
pub fn retaliate(
    world: &World,
    encounters: &EncounterContent,
    state: &mut GameState,
    active: &[EnemyInstance],
    rng: &mut impl RandomSource,
) -> String {
    let Some(character) = state.character.as_ref() else {
        return String::new();
    };
    if character.hp == 0 {
        return String::new();
    }
    let mut lines = Vec::new();
    for instance in active {
        if state.character.as_ref().is_none_or(|c| c.hp == 0) {
            break;
        }
        if crate::systems::stunned(world, &instance.statuses) {
            continue;
        }
        let Some(enemy) = encounters.enemies.get(&instance.enemy_id) else {
            continue;
        };
        let total: u32 = enemy.attacks.iter().map(|a| a.weight).sum();
        if total == 0 {
            continue;
        }
        let mut ticket = rng.below(total);
        let mut chosen = &enemy.attacks[0];
        for attack in &enemy.attacks {
            if ticket < attack.weight {
                chosen = attack;
                break;
            }
            ticket -= attack.weight;
        }
        let damage = ((u64::from(chosen.damage) * u64::from(enemy.attack_power) + 50) / 100)
            .min(u64::from(u32::MAX)) as u32;
        let raw = crate::systems::modified_attack(world, &instance.statuses, damage);
        let (armor, resistance) = crate::systems::defense(world, state, &chosen.damage_type);
        let damage =
            crate::systems::damage(raw, armor, resistance, world.systems.combat.minimum_damage);
        let character = state.character.as_mut().unwrap();
        character.hp = character.hp.saturating_sub(damage);
        lines.push(format!(
            "{} #{} uses {}. You take {} damage. HP: {}/{}",
            enemy.name, instance.instance_id, chosen.name, damage, character.hp, character.max_hp
        ));
        if character.hp == 0 {
            break;
        } // The runner displays the single defeat screen.
        if let Some(id) = &chosen.status {
            if let Some(status) = world.systems.statuses.get(id) {
                state.systems.statuses.insert(id.clone(), status.turns);
            }
        }
    }
    lines.join("\n")
}

/// Award a defeated enemy exactly once. The caller must supply the removed instance
/// from `CombatAction::defeated`; misses and repeat attacks cannot award rewards.
/// Each TOML item ID is unique: loot drops only while that item is unavailable
/// (`Consumed`), never while it is already on the ground or in inventory.
pub fn award_defeat(
    encounters: &EncounterContent,
    world: &World,
    progression: Option<&Progression>,
    state: &mut GameState,
    defeated: &EnemyInstance,
    rng: &mut impl RandomSource,
) -> Result<String, String> {
    let Some(enemy) = encounters.enemies.get(&defeated.enemy_id) else {
        return Err(format!("Unknown defeated enemy: {}", defeated.enemy_id));
    };
    let mut messages = Vec::new();
    let mut effects = Vec::new();
    if progression.is_some()
        && state.character.as_ref().is_some_and(|c| c.hp > 0)
        && enemy.experience > 0
    {
        effects.push(crate::rules::Effect::AwardXp(enemy.experience));
    }
    for loot in &enemy.loot {
        if rng.below(100) >= loot.chance {
            continue;
        }
        if !crate::objects::definition(world, &loot.item).stackable
            && state.location(&loot.item) != Some(ItemLocation::Consumed)
        {
            continue;
        }
        let Some(item) = world.item(&loot.item) else {
            continue;
        };
        effects.push(crate::rules::Effect::SpawnItem {
            item: loot.item.clone(),
            quantity: loot.quantity,
            location: ItemLocation::Room(state.current_room.clone()),
        });
        messages.push(format!("{} drops {}.", enemy.name, item.name));
    }
    for reward in &enemy.currency_rewards {
        if rng.below(100) >= reward.chance {
            continue;
        }
        let amount = reward.min + u64::from(rng.below((reward.max - reward.min + 1) as u32));
        if amount == 0 {
            continue;
        }
        effects.push(crate::rules::Effect::Credit {
            currency: reward.currency.clone(),
            amount,
        });
        if let Some(c) = world.systems.wallet.currencies.get(&reward.currency) {
            messages.push(format!("You collect {amount} {}.", c.name));
        }
    }
    effects.extend(enemy.rewards.clone());
    let mut lines = crate::rules::apply(world, state, &effects)?;
    lines.extend(messages);
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encounters::{Enemy, EnemyAttack};
    use std::collections::HashMap;
    fn fixture() -> (EncounterContent, Vec<EnemyInstance>) {
        let monster = Enemy {
            resistances: Default::default(),
            id: "goblin".into(),
            name: "Goblin".into(),
            description: String::new(),
            level: 1,
            max_hp: 20,
            attack_power: 100,
            defense: 2,
            experience: 0,
            rewards: vec![],
            loot: vec![],
            currency_rewards: vec![],
            attacks: vec![EnemyAttack {
                damage_type: "physical".into(),
                status: None,
                name: "Stab".into(),
                damage: 4,
                weight: 1,
            }],
        };
        let content = EncounterContent {
            enemies: HashMap::from([("goblin".into(), monster)]),
            tables: HashMap::new(),
        };
        let active = vec![
            EnemyInstance {
                statuses: Default::default(),
                instance_id: 1,
                enemy_id: "goblin".into(),
                current_hp: 20,
            },
            EnemyInstance {
                statuses: Default::default(),
                instance_id: 2,
                enemy_id: "goblin".into(),
                current_hp: 20,
            },
        ];
        (content, active)
    }
    #[test]
    fn duplicate_names_require_explicit_instance() {
        let (content, active) = fixture();
        assert!(target_index(&content, &active, "goblin")
            .unwrap_err()
            .contains("Multiple"));
        assert_eq!(target_index(&content, &active, "Goblin #2"), Ok(1));
        assert!(target_index(&content, &active, "Goblin #99").is_err());
        assert_eq!(target_index(&content, &active, "goblin #1"), Ok(0));
    }
    #[test]
    fn partial_enemy_name_is_allowed_when_unambiguous() {
        let (mut content, mut active) = fixture();
        content.enemies.get_mut("goblin").unwrap().name = "Stone Guardian".into();
        assert!(target_index(&content, &active, "guardian")
            .unwrap_err()
            .contains("Multiple"));
        active.pop();
        assert_eq!(target_index(&content, &active, "guardian"), Ok(0));
        assert_eq!(target_index(&content, &active, "stone guardian"), Ok(0));
        assert_eq!(
            target_index(&content, &active, "ard"),
            Err("No visible enemy matches 'ard'.".into())
        );
    }

    #[test]
    fn damage_respects_defense_and_removes_defeated_enemy() {
        let (content, mut active) = fixture();
        let result = hit(
            &crate::story_loader::load_game(std::path::Path::new("story_example"))
                .unwrap()
                .world,
            "physical",
            &content,
            &mut active,
            0,
            10,
            "You strike",
        );
        assert!(result.turn_spent);
        assert_eq!(active[0].current_hp, 12);
        let result = hit(
            &crate::story_loader::load_game(std::path::Path::new("story_example"))
                .unwrap()
                .world,
            "physical",
            &content,
            &mut active,
            0,
            14,
            "You strike",
        );
        assert!(result.message.contains("defeated"));
        assert_eq!(active.len(), 1);
    }
}
