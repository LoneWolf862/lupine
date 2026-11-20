//! Authored command patterns resolve objects before shared rules mutate state.
use crate::{
    rules::{Condition, Effect, Outcome},
    state::GameState,
    world::World,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, BufRead, Write},
    path::Path,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    /// Literal words plus optional {target} and {item}; each binds a stable ID below.
    pub commands: Vec<String>,
    #[serde(default)]
    pub room: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub item: Option<String>,
    #[serde(default)]
    pub requires: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub outcomes: Vec<Outcome>,
    pub success_text: String,
    pub failure_text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    actions: Vec<toml::Value>,
}
pub fn load(root: &Path) -> Result<Vec<Action>, String> {
    let path = root.join("actions.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let file: File = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if file.actions.len() > 1024 {
        return Err("actions.toml: at most 1024 actions".into());
    }
    file.actions
        .iter()
        .enumerate()
        .map(|(i, v)| {
            crate::rules::decode(v)
                .map_err(|e| format!("{} action #{}: {e}", path.display(), i + 1))
        })
        .collect()
}

fn reserved(verb: &str) -> bool {
    matches!(
        verb,
        "abilities"
            | "skills"
            | "interact"
            | "talk"
            | "people"
            | "journal"
            | "quests"
            | "save"
            | "load"
            | "saves"
            | "quit"
            | "q"
            | "exit"
            | "help"
            | "h"
            | "?"
            | "look"
            | "l"
            | "examine"
            | "inspect"
            | "x"
            | "take"
            | "get"
            | "grab"
            | "drop"
            | "inv"
            | "pick"
            | "inventory"
            | "i"
            | "attack"
            | "stab"
            | "slash"
            | "hit"
            | "strike"
            | "cast"
            | "learn"
            | "stats"
            | "enemies"
            | "weapons"
            | "spells"
            | "spellbook"
            | "wallet"
            | "money"
            | "balance"
            | "shop"
            | "buy"
            | "sell"
            | "north"
            | "south"
            | "east"
            | "west"
            | "up"
            | "down"
            | "n"
            | "s"
            | "e"
            | "w"
            | "u"
            | "d"
            | "go"
            | "move"
            | "walk"
            | "travel"
            | "run"
            | "head"
            | "leave"
            | "yes"
            | "no"
            | "cancel"
            | "back"
    )
}
impl Action {
    pub fn validate(&self, world: &World) -> Result<(), String> {
        crate::rules::valid_id(&self.id)?;
        if self.commands.is_empty() || self.commands.len() > 16 {
            return Err("needs 1..16 command patterns".into());
        }
        if let Some(room) = &self.room {
            Condition::InRoom(room.clone()).validate(world)?;
        }
        for id in [&self.item, &self.target].into_iter().flatten() {
            Condition::HasItem(id.clone()).validate(world)?;
        }
        if let Some(id) = &self.item {
            if !world.item(id).unwrap().portable {
                return Err("{item} must be portable; use {target} for fixed objects".into());
            }
        }
        for pattern in &self.commands {
            let normalized = crate::resolution::normalize(pattern);
            let words = normalized.split_whitespace().collect::<Vec<_>>();
            if normalized != *pattern
                || words.is_empty()
                || words.len() > 16
                || pattern.len() > 240
                || pattern.chars().any(char::is_control)
            {
                return Err(
                    "patterns must be normalized lowercase text (1..16 words, at most 240 bytes)"
                        .into(),
                );
            }
            if reserved(words[0]) || words[0].contains(['{', '}']) {
                return Err(format!("reserved or invalid action verb '{}'", words[0]));
            }
            if words[0] == "use" && pattern != "use {target}" {
                return Err("authored use supports only 'use {target}'; ordinary item/spell use keeps priority".into());
            }
            for (slot, bound) in [("{item}", &self.item), ("{target}", &self.target)] {
                if words.iter().filter(|w| **w == slot).count() != usize::from(bound.is_some()) {
                    return Err(format!(
                        "pattern {pattern:?} must contain {slot} exactly once when its ID is bound"
                    ));
                }
            }
            if words
                .iter()
                .any(|w| w.contains(['{', '}']) && !matches!(*w, "{item}" | "{target}"))
                || words
                    .windows(2)
                    .any(|w| w.iter().all(|v| v.starts_with('{')))
            {
                return Err("unknown or adjacent command placeholders".into());
            }
        }
        if self.effects.len() > 256 || self.outcomes.len() > 128 {
            return Err("too many effects or outcomes".into());
        }
        for c in &self.requires {
            c.validate(world)?;
        }
        for e in &self.effects {
            e.validate(world)?;
        }
        for o in &self.outcomes {
            if o.effects.len() > 256 {
                return Err("too many outcome effects".into());
            }
            for c in &o.when {
                c.validate(world)?;
            }
            for e in &o.effects {
                e.validate(world)?;
            }
        }
        Ok(())
    }
    pub fn in_scope(&self, state: &GameState) -> bool {
        self.room.as_ref().is_none_or(|r| *r == state.current_room)
            && [&self.target, &self.item]
                .into_iter()
                .flatten()
                .all(|id| state.is_accessible(id))
    }
    pub fn execute(&self, world: &World, state: &mut GameState) -> Result<String, String> {
        if !self.in_scope(state) {
            return Err("That action is not available here.".into());
        }
        if let Some(id) = &self.item {
            if !state.is_carried(id) {
                return Err(format!(
                    "You must be carrying the {}.",
                    world.item(id).unwrap().name
                ));
            }
        }
        if !self.requires.iter().all(|c| c.met(state)) {
            return Err(self.failure_text.clone());
        }
        let outcome = self
            .outcomes
            .iter()
            .find(|o| o.when.iter().all(|c| c.met(state)));
        let (effects, text) = outcome.map_or((&self.effects, self.success_text.as_str()), |o| {
            (&o.effects, o.text.as_str())
        });
        let mut lines = crate::rules::apply(world, state, effects)?;
        lines.insert(0, text.to_owned());
        Ok(lines.join("\n"))
    }
    fn example(&self, pattern: &str, world: &World) -> String {
        let mut text = pattern.to_owned();
        for (slot, id) in [("{item}", &self.item), ("{target}", &self.target)] {
            if let Some(item) = id.as_ref().and_then(|id| world.item(id)) {
                text = text.replace(slot, &item.name);
            }
        }
        text
    }
}
pub fn validate(world: &World) -> Vec<String> {
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    for (i, a) in world.actions.iter().enumerate() {
        if !ids.insert(&a.id) {
            errors.push(format!("actions.toml: duplicate action '{}'", a.id));
        }
        if let Err(e) = a.validate(world) {
            errors.push(format!("actions.toml action '{}': {e}", a.id));
        }
        for b in &world.actions[..i] {
            if a.item == b.item
                && a.target == b.target
                && (a.room == b.room || a.room.is_none() || b.room.is_none())
                && a.commands.iter().any(|c| b.commands.contains(c))
            {
                errors.push(format!(
                    "actions.toml: actions '{}' and '{}' overlap; combine them using outcomes",
                    a.id, b.id
                ));
            }
        }
    }
    errors
}

// Match literal separators; names may contain several words. Multiple valid splits
// are refused, never guessed. Maximum input/pattern lengths bound the search.
fn captures(pattern: &str, line: &str) -> Vec<BTreeMap<String, String>> {
    fn visit(
        p: &[&str],
        w: &[&str],
        values: BTreeMap<String, String>,
        out: &mut Vec<BTreeMap<String, String>>,
    ) {
        if out.len() > 1 {
            return;
        }
        if p.is_empty() {
            if w.is_empty() {
                out.push(values);
            }
            return;
        }
        if p[0].starts_with('{') {
            for n in 1..=w.len() {
                let mut v = values.clone();
                v.insert(p[0].into(), w[..n].join(" "));
                visit(&p[1..], &w[n..], v, out);
                if out.len() > 1 {
                    break;
                }
            }
        } else if !w.is_empty() && p[0] == w[0] {
            visit(&p[1..], &w[1..], values, out);
        }
    }
    if line.len() > 1024 {
        return vec![];
    }
    let mut out = Vec::new();
    visit(
        &pattern.split_whitespace().collect::<Vec<_>>(),
        &line.split_whitespace().collect::<Vec<_>>(),
        BTreeMap::new(),
        &mut out,
    );
    out
}
pub enum Prepared {
    Unhandled,
    Canceled,
    Action(usize),
}
pub fn prepare(
    line: &str,
    world: &World,
    state: &GameState,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<Prepared> {
    let line = crate::resolution::normalize(line);
    // Read/search have a non-mutating examination fallback, but authored actions
    // may intentionally attach effects. Preserve their matching/failure semantics.
    let verb = line.split_whitespace().next().unwrap_or("");
    if crate::parser::parse_command(&line).is_ok() && !matches!(verb, "read" | "search") {
        return Ok(Prepared::Unhandled);
    }
    let patterns = world
        .actions
        .iter()
        .filter(|a| a.room.as_ref().is_none_or(|r| *r == state.current_room))
        .flat_map(|a| a.commands.iter())
        .collect::<BTreeSet<_>>();
    let mut matches = Vec::new();
    for pattern in patterns {
        for values in captures(pattern, &line) {
            matches.push((pattern, values));
        }
    }
    if matches.is_empty() {
        return Ok(Prepared::Unhandled);
    }
    if matches.len() != 1 {
        writeln!(output,"That phrasing matches multiple authored patterns. Use an exact command shown by help.\n")?;
        return Ok(Prepared::Canceled);
    }
    let (pattern, queries) = matches.pop().unwrap();
    let candidates = crate::resolution::items(world, state);
    let mut selected = BTreeMap::new();
    for (slot, query) in queries {
        let Some(id) = crate::player_input::choose(
            &crate::parser::clean_target(&query),
            &candidates,
            input,
            output,
        )?
        else {
            return Ok(Prepared::Canceled);
        };
        selected.insert(slot, id);
    }
    let found = world
        .actions
        .iter()
        .enumerate()
        .filter(|(_, a)| {
            a.in_scope(state)
                && a.commands.contains(pattern)
                && a.item.as_ref() == selected.get("{item}")
                && a.target.as_ref() == selected.get("{target}")
        })
        .collect::<Vec<_>>();
    if let [(index, _)] = found.as_slice() {
        Ok(Prepared::Action(*index))
    } else {
        writeln!(
            output,
            "That action is not available for those objects here.\n"
        )?;
        Ok(Prepared::Canceled)
    }
}
pub fn help(world: &World, state: &GameState) -> String {
    let commands = world
        .actions
        .iter()
        .filter(|a| a.in_scope(state))
        .flat_map(|a| a.commands.iter().map(|p| a.example(p, world)))
        .collect::<BTreeSet<_>>();
    if commands.is_empty() {
        String::new()
    } else {
        format!(
            "Actions here: {}",
            commands.into_iter().collect::<Vec<_>>().join("; ")
        )
    }
}
pub fn suggestion(line: &str, world: &World, state: &GameState) -> Option<String> {
    let line = crate::resolution::normalize(line);
    let candidates = world
        .actions
        .iter()
        .filter(|a| a.in_scope(state))
        .flat_map(|a| a.commands.iter().map(|p| a.example(p, world)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|s| crate::resolution::Candidate::new(&s, &s))
        .collect::<Vec<_>>();
    crate::resolution::suggestion(&line, &candidates).map(|c| c.name.clone())
}
