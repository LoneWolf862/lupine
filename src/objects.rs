//! Object definitions and one canonical quantity/location ledger.
use crate::{
    rules::{Condition, Effect},
    state::{GameState, ItemLocation},
    world::World,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_QUANTITY: u32 = 1_000_000;

pub fn describe_container(world: &World, state: &GameState, id: &str) -> String {
    let Some(container) = state.objects.containers.get(id) else {
        return String::new();
    };
    if container.locked {
        return "\nIt is locked.".into();
    }
    if !container.open {
        return "\nIt is closed.".into();
    }
    let contents = world
        .items
        .iter()
        .filter(|i| !state.objects.hidden.contains(&i.id))
        .filter_map(|i| {
            let count = state.count_at(&i.id, &ItemLocation::Container(id.into()));
            (count > 0).then(|| {
                if definition(world, &i.id).stackable {
                    format!("{} ({count})", i.name)
                } else {
                    i.name.clone()
                }
            })
        })
        .collect::<Vec<_>>();
    format!(
        "\nIt is open. Contents:\n  {}",
        if contents.is_empty() {
            "nothing".into()
        } else {
            contents.join("\n  ")
        }
    )
}

pub enum Prepared {
    Unhandled,
    Canceled,
    Action {
        verb: String,
        item: String,
        target: Option<String>,
        quantity: u32,
    },
}
pub fn prepare(
    line: &str,
    world: &World,
    state: &GameState,
    input: &mut impl std::io::BufRead,
    output: &mut impl std::io::Write,
) -> std::io::Result<Prepared> {
    let line = crate::resolution::normalize(line);
    let (verb, argument) = line.split_once(' ').unwrap_or((&line, ""));
    let verb = match verb {
        "eat" | "drink" => "consume",
        v => v,
    };
    if !matches!(
        verb,
        "open" | "close" | "unlock" | "lock" | "put" | "consume" | "take" | "drop"
    ) {
        return Ok(Prepared::Unhandled);
    }
    // Preserve the existing take/drop aliases and explicit bulk-command route.
    let quantified = argument
        .split_whitespace()
        .next()
        .is_some_and(|s| s.parse::<i64>().is_ok());
    if matches!(verb, "take" | "drop") && !quantified && !argument.contains(" from ") {
        return Ok(Prepared::Unhandled);
    }
    if argument.is_empty() {
        writeln!(output, "Specify an object for '{verb}'.\n")?;
        return Ok(Prepared::Canceled);
    }
    if quantified && !matches!(verb, "take" | "drop" | "put") {
        writeln!(
            output,
            "Use one object per '{verb}' command; omit the quantity.\n"
        )?;
        return Ok(Prepared::Canceled);
    }
    let (quantity, argument) = if quantified {
        let (n, rest) = argument.split_once(' ').unwrap_or((argument, ""));
        let Ok(n) = n.parse::<u32>() else {
            writeln!(output, "Quantity must be 1..{MAX_QUANTITY}.\n")?;
            return Ok(Prepared::Canceled);
        };
        if n == 0 || n > MAX_QUANTITY {
            writeln!(output, "Quantity must be 1..{MAX_QUANTITY}.\n")?;
            return Ok(Prepared::Canceled);
        }
        (n, rest)
    } else {
        (1, argument)
    };
    let separator = match verb {
        "put" => " in ",
        "take" => " from ",
        "lock" | "unlock" => " with ",
        _ => "\0",
    };
    let (query, target) = argument
        .split_once(separator)
        .map_or((argument, None), |(a, b)| (a, Some(b)));
    let choices = crate::resolution::items(world, state);
    let target = if let Some(query) = target {
        let Some(id) = crate::player_input::choose(
            &crate::parser::clean_target(query),
            &choices,
            input,
            output,
        )?
        else {
            return Ok(Prepared::Canceled);
        };
        Some(id)
    } else {
        None
    };
    if verb == "take"
        && target.as_ref().is_some_and(|id| {
            !state
                .objects
                .containers
                .get(id)
                .is_some_and(|c| c.open && !c.locked)
        })
    {
        writeln!(output, "Open the source container first.\n")?;
        return Ok(Prepared::Canceled);
    }
    let Some(id) =
        crate::player_input::choose(&crate::parser::clean_target(query), &choices, input, output)?
    else {
        return Ok(Prepared::Canceled);
    };
    Ok(Prepared::Action {
        verb: verb.into(),
        item: id,
        target,
        quantity,
    })
}
fn one() -> u32 {
    1
}
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Definition {
    pub initial: Option<ItemLocation>,
    pub hidden: bool,
    pub stackable: bool,
    pub quantity: u32,
    pub container: Option<Container>,
    pub consumable: Option<Consumable>,
}
impl Default for Definition {
    fn default() -> Self {
        Self {
            initial: None,
            hidden: false,
            stackable: false,
            quantity: 1,
            container: None,
            consumable: None,
        }
    }
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Container {
    pub open: bool,
    pub locked: bool,
    pub key: Option<String>,
    pub capacity: Option<u32>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consumable {
    #[serde(default)]
    pub requires: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    pub text: String,
    pub failure_text: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    pub location: ItemLocation,
    pub quantity: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerState {
    pub open: bool,
    pub locked: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectState {
    pub placements: BTreeMap<String, Vec<Stack>>,
    #[serde(deserialize_with = "unique_hidden")]
    pub hidden: BTreeSet<String>,
    pub containers: BTreeMap<String, ContainerState>,
}
fn unique_hidden<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeSet<String>, D::Error> {
    let list = Vec::<String>::deserialize(d)?;
    let set = list.iter().cloned().collect::<BTreeSet<_>>();
    if set.len() != list.len() {
        return Err(serde::de::Error::custom("Duplicate hidden item"));
    }
    Ok(set)
}
pub fn initial_location(world: &World, id: &str) -> Option<ItemLocation> {
    let item = world.item(id)?;
    Some(
        definition(world, id)
            .initial
            .unwrap_or_else(|| ItemLocation::Room(item.starting_room.clone())),
    )
}
impl ObjectState {
    pub fn new(world: &World) -> Self {
        let mut result = Self::default();
        for item in &world.items {
            let d = world.object_defs.get(&item.id).cloned().unwrap_or_default();
            let location = d
                .initial
                .unwrap_or_else(|| ItemLocation::Room(item.starting_room.clone()));
            result.placements.insert(
                item.id.clone(),
                if location == ItemLocation::Consumed {
                    vec![]
                } else {
                    vec![Stack {
                        location,
                        quantity: d.quantity,
                    }]
                },
            );
            if d.hidden {
                result.hidden.insert(item.id.clone());
            }
            if let Some(c) = d.container {
                result.containers.insert(
                    item.id.clone(),
                    ContainerState {
                        open: c.open,
                        locked: c.locked,
                    },
                );
            }
        }
        result
    }
}
pub fn definition(world: &World, id: &str) -> Definition {
    world.object_defs.get(id).cloned().unwrap_or_default()
}
pub fn validate(world: &World) -> Result<(), String> {
    for (id, d) in &world.object_defs {
        let item = world
            .item(id)
            .ok_or_else(|| format!("Unknown object definition '{id}'"))?;
        if d.quantity == 0 || d.quantity > MAX_QUANTITY || (!d.stackable && d.quantity != 1) {
            return Err(format!(
                "Item '{id}': unique items need quantity 1; stacks need 1..{MAX_QUANTITY}"
            ));
        }
        if d.stackable && (!item.portable || d.container.is_some()) {
            return Err(format!(
                "Item '{id}': only portable non-container items can stack"
            ));
        }
        if let Some(initial) = &d.initial {
            validate_location(world, initial)?;
            if !item.portable
                && matches!(
                    initial,
                    ItemLocation::Inventory | ItemLocation::Container(_) | ItemLocation::Npc(_)
                )
            {
                return Err(format!(
                    "Fixed item '{id}' cannot start carried or contained"
                ));
            }
        }
        if let Some(c) = &d.container {
            if c.locked && c.open {
                return Err(format!("Container '{id}' cannot start open and locked"));
            }
            if c.capacity.is_some_and(|n| n == 0 || n > MAX_QUANTITY) {
                return Err(format!(
                    "Container '{id}' capacity must be 1..{MAX_QUANTITY}"
                ));
            }
            if let Some(key) = &c.key {
                if !world.item(key).is_some_and(|i| i.portable) {
                    return Err(format!("Container '{id}' has unknown or fixed key '{key}'"));
                }
            }
        }
        if let Some(c) = &d.consumable {
            if !item.portable || d.container.is_some() {
                return Err(format!(
                    "Consumable '{id}' must be portable and not a container"
                ));
            }
            if c.effects.len() > 256 {
                return Err(format!("Consumable '{id}' exceeds 256 effects"));
            }
            for c in &c.requires {
                c.validate(world)?;
            }
            for e in &c.effects {
                e.validate(world)?;
            }
        }
    }
    validate_state(world, &GameState::new(world))
}
pub(crate) fn validate_location(world: &World, location: &ItemLocation) -> Result<(), String> {
    match location {
        ItemLocation::Npc(id) if !world.dialogue.npcs.contains_key(id) => {
            Err(format!("Unknown NPC location '{id}'"))
        }
        ItemLocation::Room(id) if world.room(id).is_none() => {
            Err(format!("Unknown item room '{id}'"))
        }
        ItemLocation::Container(id)
            if !world
                .object_defs
                .get(id)
                .is_some_and(|d| d.container.is_some()) =>
        {
            Err(format!("Unknown container '{id}'"))
        }
        _ => Ok(()),
    }
}
pub fn validate_state(world: &World, state: &GameState) -> Result<(), String> {
    for item in &world.items {
        if !state.objects.placements.contains_key(&item.id) {
            return Err(format!("Save is missing location for item '{}'", item.id));
        }
    }
    for (id, stacks) in &state.objects.placements {
        let item = world
            .item(id)
            .ok_or_else(|| format!("Unknown saved item '{id}'"))?;
        let d = definition(world, id);
        let mut places = BTreeSet::new();
        let mut total = 0u32;
        for stack in stacks {
            validate_location(world, &stack.location)?;
            if stack.quantity == 0
                || stack.location == ItemLocation::Consumed
                || !places.insert(&stack.location)
            {
                return Err(format!(
                    "Item '{id}' has empty, duplicate or consumed stack"
                ));
            }
            total = total
                .checked_add(stack.quantity)
                .ok_or("Item quantity overflow")?;
            if !item.portable
                && matches!(
                    stack.location,
                    ItemLocation::Inventory | ItemLocation::Container(_) | ItemLocation::Npc(_)
                )
            {
                return Err(format!("Fixed item '{id}' cannot be carried or contained"));
            }
            let mut parent = &stack.location;
            let mut seen = BTreeSet::from([id.as_str()]);
            while let ItemLocation::Container(container) = parent {
                if !seen.insert(container.as_str()) || seen.len() > 16 {
                    return Err(format!("Container cycle or nesting past 16 at '{id}'"));
                }
                let contents = state
                    .objects
                    .placements
                    .get(container)
                    .ok_or("Missing container location")?;
                if contents.len() > 1 {
                    return Err("Containers cannot occupy multiple locations".into());
                }
                let Some(p) = contents.first() else {
                    break;
                };
                parent = &p.location;
            }
        }
        if total > MAX_QUANTITY || (!d.stackable && total > 1) {
            return Err(format!("Item '{id}' exceeds its quantity limit"));
        }
    }
    for id in &state.objects.hidden {
        if world.item(id).is_none() {
            return Err(format!("Unknown hidden item '{id}'"));
        }
    }
    let expected = world
        .object_defs
        .iter()
        .filter(|(_, d)| d.container.is_some())
        .map(|(id, _)| id)
        .collect::<BTreeSet<_>>();
    if expected != state.objects.containers.keys().collect() {
        return Err("Saved container IDs differ from definitions".into());
    }
    for (id, c) in &state.objects.containers {
        if c.open && c.locked {
            return Err(format!("Container '{id}' is both open and locked"));
        }
        if let Some(capacity) = definition(world, id).container.and_then(|c| c.capacity) {
            let count = state
                .objects
                .placements
                .values()
                .flatten()
                .filter(|s| s.location == ItemLocation::Container(id.clone()))
                .try_fold(0u32, |sum, s| sum.checked_add(s.quantity))
                .ok_or("Container quantity overflow")?;
            if count > capacity {
                return Err(format!("Container '{id}' capacity exceeded"));
            }
        }
    }
    Ok(())
}
impl GameState {
    pub fn count_at(&self, id: &str, place: &ItemLocation) -> u32 {
        self.objects
            .placements
            .get(id)
            .into_iter()
            .flatten()
            .filter(|s| &s.location == place)
            .map(|s| s.quantity)
            .sum()
    }
    pub fn is_carried(&self, id: &str) -> bool {
        self.carried_quantity(id) > 0
    }
    pub fn carried_quantity(&self, id: &str) -> u32 {
        self.objects
            .placements
            .get(id)
            .into_iter()
            .flatten()
            .filter(|s| self.place_carried(&s.location, 0))
            .map(|s| s.quantity)
            .sum()
    }
    fn place_carried(&self, place: &ItemLocation, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        match place {
            ItemLocation::Inventory => true,
            ItemLocation::Container(id) => self
                .objects
                .placements
                .get(id)
                .into_iter()
                .flatten()
                .any(|s| self.place_carried(&s.location, depth + 1)),
            _ => false,
        }
    }
    pub fn place_accessible(&self, place: &ItemLocation, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        match place {
            ItemLocation::Inventory => true,
            ItemLocation::Room(room) => room == &self.current_room,
            ItemLocation::Container(id) => {
                !self.objects.hidden.contains(id)
                    && self
                        .objects
                        .containers
                        .get(id)
                        .is_some_and(|c| c.open && !c.locked)
                    && self
                        .objects
                        .placements
                        .get(id)
                        .into_iter()
                        .flatten()
                        .any(|s| self.place_accessible(&s.location, depth + 1))
            }
            ItemLocation::Consumed | ItemLocation::Npc(_) => false,
        }
    }
    /// Explicit whole-object relocation; authored give/remove retain their original semantics.
    pub fn set_location(&mut self, id: String, location: ItemLocation) {
        let quantity = self
            .objects
            .placements
            .get(&id)
            .into_iter()
            .flatten()
            .map(|s| s.quantity)
            .sum::<u32>()
            .max(1);
        self.objects.placements.insert(
            id,
            if location == ItemLocation::Consumed {
                vec![]
            } else {
                vec![Stack { location, quantity }]
            },
        );
    }
    pub fn transfer(
        &mut self,
        world: &World,
        id: &str,
        from: &ItemLocation,
        to: &ItemLocation,
        quantity: u32,
    ) -> Result<(), String> {
        if quantity == 0 || self.count_at(id, from) < quantity {
            return Err("Not enough items at that location.".into());
        }
        if from == to {
            return Err("The item is already there.".into());
        }
        let mut draft = self.clone();
        let stacks = draft.objects.placements.get_mut(id).ok_or("Unknown item")?;
        let source = stacks
            .iter_mut()
            .find(|s| &s.location == from)
            .ok_or("Missing source")?;
        source.quantity -= quantity;
        stacks.retain(|s| s.quantity > 0);
        if *to != ItemLocation::Consumed {
            if let Some(stack) = stacks.iter_mut().find(|s| &s.location == to) {
                stack.quantity = stack
                    .quantity
                    .checked_add(quantity)
                    .ok_or("Quantity overflow")?;
            } else {
                stacks.push(Stack {
                    location: to.clone(),
                    quantity,
                });
            }
        }
        stacks.sort_by(|a, b| a.location.cmp(&b.location));
        crate::systems::prune_equipment(&mut draft);
        validate_state(world, &draft)?;
        *self = draft;
        Ok(())
    }
    pub fn consume_one(&mut self, id: &str) -> Result<(), String> {
        if !self.is_accessible(id) {
            return Err("Item is inaccessible".into());
        }
        let from = self
            .available_source(id, true)
            .ok_or("You must carry the item")?;
        let stacks = self.objects.placements.get_mut(id).ok_or("Unknown item")?;
        let stack = stacks
            .iter_mut()
            .find(|s| s.location == from)
            .ok_or("Missing item")?;
        stack.quantity -= 1;
        stacks.retain(|s| s.quantity > 0);
        Ok(())
    }
    pub fn available_source(&self, id: &str, carried: bool) -> Option<ItemLocation> {
        if self.objects.hidden.contains(id) {
            return None;
        }
        self.objects
            .placements
            .get(id)?
            .iter()
            .filter(|s| {
                self.place_accessible(&s.location, 0)
                    && if carried {
                        self.place_carried(&s.location, 0)
                    } else {
                        s.location != ItemLocation::Inventory
                    }
            })
            .min_by_key(|s| {
                if carried {
                    usize::from(s.location != ItemLocation::Inventory)
                } else {
                    usize::from(!matches!(s.location, ItemLocation::Room(_)))
                }
            })
            .map(|s| s.location.clone())
    }
}

/// Apply one built-in object action atomically. Resolution happens in the input layer.
pub fn manipulate(
    world: &World,
    state: &mut GameState,
    verb: &str,
    id: &str,
    target: Option<&str>,
    quantity: u32,
) -> Result<String, String> {
    if !state.is_accessible(id) {
        return Err("That object is not accessible.".into());
    }
    let item = world.item(id).ok_or("Unknown object")?;
    match verb {
        "take" | "drop" | "put" => {
            if !item.portable {
                return Err(format!("The {} cannot be moved.", item.name));
            }
            let from = if verb == "take" {
                if let Some(container) = target {
                    ItemLocation::Container(container.into())
                } else {
                    state
                        .available_source(id, false)
                        .ok_or_else(|| format!("You are already carrying the {}.", item.name))?
                }
            } else {
                state
                    .available_source(id, true)
                    .ok_or_else(|| format!("You are not carrying the {}.", item.name))?
            };
            if !state.place_accessible(&from, 0) {
                return Err("The source is closed, locked or inaccessible.".into());
            }
            let to = match verb {
                "take" => ItemLocation::Inventory,
                "put" => {
                    let container = target.ok_or("Specify a container")?;
                    if !state.is_accessible(container)
                        || !state
                            .objects
                            .containers
                            .get(container)
                            .is_some_and(|c| c.open && !c.locked)
                    {
                        return Err("The destination must be an accessible open container.".into());
                    }
                    ItemLocation::Container(container.into())
                }
                _ => ItemLocation::Room(state.current_room.clone()),
            };
            state.transfer(world, id, &from, &to, quantity)?;
            let suffix = if definition(world, id).stackable {
                format!(" ({quantity})")
            } else {
                String::new()
            };
            Ok(format!("You {verb} the {}{suffix}.", item.name))
        }
        "open" | "close" | "unlock" | "lock" => {
            let c = state
                .objects
                .containers
                .get(id)
                .ok_or("That object is not a container.")?;
            let mut next = c.clone();
            match verb {
                "open" if c.locked => return Err("It is locked.".into()),
                "open" => next.open = true,
                "close" => next.open = false,
                _ => {
                    if c.open {
                        return Err("Close it first.".into());
                    }
                    let key = definition(world, id).container.and_then(|c| c.key).ok_or(
                        "This container has no authored key; an action may control its lock.",
                    )?;
                    if target.is_none() {
                        return Err(format!(
                            "Specify which key to use. Try '{verb} {} with <key>'.",
                            item.id
                        ));
                    }
                    if target != Some(key.as_str())
                        || !state.is_carried(&key)
                        || !state.is_accessible(&key)
                    {
                        return Err("You need the correct carried key.".into());
                    }
                    next.locked = verb == "lock";
                }
            }
            if next == *c {
                return Err(format!(
                    "It is already {}.",
                    if c.open {
                        "open"
                    } else if c.locked {
                        "locked"
                    } else {
                        "closed and unlocked"
                    }
                ));
            }
            state.objects.containers.insert(id.into(), next);
            Ok(format!(
                "You {verb} the {}.{}",
                item.name,
                if verb == "open" {
                    describe_container(world, state, id)
                } else {
                    String::new()
                }
            ))
        }
        "consume" => {
            let d = definition(world, id)
                .consumable
                .ok_or("That item is not consumable.")?;
            if !state.is_carried(id) {
                return Err("You must be carrying it.".into());
            }
            if !d.requires.iter().all(|c| c.met(state)) {
                return Err(d.failure_text);
            }
            let from = state
                .available_source(id, true)
                .ok_or("Item is inaccessible")?;
            let mut draft = state.clone();
            draft.transfer(world, id, &from, &ItemLocation::Consumed, one())?;
            let mut output = crate::rules::apply(world, &mut draft, &d.effects)?;
            output.insert(0, d.text);
            *state = draft;
            Ok(output.join("\n"))
        }
        _ => Err("Unknown object action".into()),
    }
}

/// Create units at one typed location, preserving existing stacks and validating atomically.
pub fn add(
    world: &World,
    state: &mut GameState,
    id: &str,
    location: &ItemLocation,
    quantity: u32,
) -> Result<(), String> {
    if quantity == 0 || *location == ItemLocation::Consumed {
        return Err("Use a positive quantity and an available destination".into());
    }
    let mut draft = state.clone();
    let stacks = draft.objects.placements.get_mut(id).ok_or("Unknown item")?;
    if let Some(s) = stacks.iter_mut().find(|s| &s.location == location) {
        s.quantity = s
            .quantity
            .checked_add(quantity)
            .ok_or("Quantity overflow")?;
    } else {
        stacks.push(Stack {
            location: location.clone(),
            quantity,
        });
    }
    stacks.sort_by(|a, b| a.location.cmp(&b.location));
    validate_state(world, &draft)?;
    *state = draft;
    Ok(())
}
