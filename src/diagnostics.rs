//! Advisory author checks: structural validity is separate from solvability.
use crate::{
    adventure::Adventure,
    rules::{Condition, Effect},
};
use std::collections::BTreeSet;

pub fn warnings(a: &Adventure) -> Vec<String> {
    let w = &a.game.world;
    let mut out = Vec::new();
    let mut reachable = BTreeSet::from([w.starting_room.clone()]);
    loop {
        let before = reachable.len();
        for r in &w.rooms {
            if reachable.contains(&r.id) {
                for (_, to) in &r.exits {
                    reachable.insert(to.clone());
                }
            }
        }
        for i in &w.interactions {
            for e in i
                .effects
                .iter()
                .chain(i.outcomes.iter().flat_map(|o| &o.effects))
            {
                if let Effect::MovePlayer(to) = e {
                    reachable.insert(to.clone());
                }
            }
        }
        for action in &w.actions {
            for effect in action
                .effects
                .iter()
                .chain(action.outcomes.iter().flat_map(|o| &o.effects))
            {
                if let Effect::MovePlayer(to) = effect {
                    reachable.insert(to.clone());
                }
            }
        }
        for event in &w.events.events {
            for effect in &event.effects {
                if let Effect::MovePlayer(to) = effect {
                    reachable.insert(to.clone());
                }
            }
        }
        for n in w.dialogue.npcs.values().flat_map(|n| n.nodes.values()) {
            for e in n.choices.iter().flat_map(|c| &c.effects) {
                if let Effect::MovePlayer(to) = e {
                    reachable.insert(to.clone());
                }
            }
        }
        if reachable.len() == before {
            break;
        }
    }
    for room in &w.rooms {
        if !reachable.contains(&room.id) {
            out.push(format!("Room '{}' has no known route from the starting room (intentional offstage storage is allowed)",room.id));
        }
    }
    let mut read = BTreeSet::new();
    let mut written = BTreeSet::new();
    fn condition(c: &Condition, read: &mut BTreeSet<String>) {
        match c {
            Condition::FlagSet(id) | Condition::FlagUnset(id) => {
                read.insert(id.clone());
            }
            Condition::All(v) | Condition::Any(v) => {
                for c in v {
                    condition(c, read);
                }
            }
            Condition::Not(c) => condition(c, read),
            _ => (),
        }
    }
    fn contradictory(c: &[Condition]) -> bool {
        c.iter().any(|a| {
            if let Condition::FlagSet(id) = a {
                c.contains(&Condition::FlagUnset(id.clone()))
            } else {
                false
            }
        })
    }
    for room in &w.rooms {
        for v in room.descriptions.iter().chain(&room.art_variants) {
            for c in &v.when {
                condition(c, &mut read);
            }
        }
    }
    for item in &w.items {
        for v in &item.descriptions {
            for c in &v.when {
                condition(c, &mut read);
            }
        }
    }
    for rule in &w.exit_rules {
        for c in &rule.requires {
            condition(c, &mut read);
        }
        if contradictory(&rule.requires) {
            out.push(format!(
                "Exit {} from '{}' has contradictory flag requirements",
                rule.direction.label(),
                rule.from
            ));
        }
    }
    for rule in &w.interactions {
        if contradictory(&rule.requires) {
            out.push(format!(
                "Interaction '{} -> {}' has contradictory flag requirements",
                rule.item, rule.target
            ));
        }
        for c in rule
            .requires
            .iter()
            .chain(rule.outcomes.iter().flat_map(|o| &o.when))
        {
            condition(c, &mut read);
        }
        for e in rule
            .effects
            .iter()
            .chain(rule.outcomes.iter().flat_map(|o| &o.effects))
        {
            if let Effect::SetFlag(id) = e {
                written.insert(id.clone());
            }
        }
        if rule
            .outcomes
            .iter()
            .take(rule.outcomes.len().saturating_sub(1))
            .any(|o| o.when.is_empty())
        {
            out.push(format!("Interaction '{} -> {}' has an unconditional outcome before later outcomes; those later outcomes are shadowed",rule.item,rule.target));
        }
    }
    for action in &w.actions {
        for c in action
            .requires
            .iter()
            .chain(action.outcomes.iter().flat_map(|o| &o.when))
        {
            condition(c, &mut read);
        }
        for e in action
            .effects
            .iter()
            .chain(action.outcomes.iter().flat_map(|o| &o.effects))
        {
            if let Effect::SetFlag(id) = e {
                written.insert(id.clone());
            }
        }
        if action
            .outcomes
            .iter()
            .take(action.outcomes.len().saturating_sub(1))
            .any(|o| o.when.is_empty())
        {
            out.push(format!("Action '{}' has shadowed outcomes", action.id));
        }
    }
    for d in w.object_defs.values() {
        if let Some(consumable) = &d.consumable {
            for c in &consumable.requires {
                condition(c, &mut read);
            }
            for e in &consumable.effects {
                if let Effect::SetFlag(id) = e {
                    written.insert(id.clone());
                }
            }
        }
    }
    for npc in w.dialogue.npcs.values() {
        for c in &npc.requires {
            condition(c, &mut read);
        }
        for n in npc.nodes.values() {
            for v in &n.variants {
                for c in &v.when {
                    condition(c, &mut read);
                }
            }
            for choice in &n.choices {
                for c in &choice.requires {
                    condition(c, &mut read);
                }
                for e in &choice.effects {
                    if let Effect::SetFlag(id) = e {
                        written.insert(id.clone());
                    }
                }
            }
        }
    }
    for event in &w.events.events {
        for c in &event.requires {
            condition(c, &mut read);
        }
        for effect in &event.effects {
            if let Effect::SetFlag(id) = effect {
                written.insert(id.clone());
            }
        }
    }
    for table in a.encounters.tables.values() {
        for entry in &table.enemies {
            for c in &entry.requires {
                read.extend(c.flag_set.iter().chain(&c.flag_unset).cloned());
            }
        }
    }
    for shop in a.shops.shops.values() {
        for offer in &shop.offers {
            read.extend(offer.requires_flags.iter().cloned());
            if offer.kind == "set_flag" {
                written.extend(offer.flag.iter().cloned());
            }
        }
    }
    written.extend(
        w.rules
            .flags
            .iter()
            .filter(|(_, value)| **value)
            .map(|(id, _)| id.clone()),
    );
    for flag in read.difference(&written) {
        out.push(format!("Flag '{flag}' is checked but no authored source sets it; check its spelling or document the intended always-unset state"));
    }
    out
}
