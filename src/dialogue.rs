//! File-authored NPCs and atomic, conditional conversation choices.
use crate::{
    resolution::Candidate,
    rules::{Condition, Effect},
    state::GameState,
    world::World,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
    path::Path,
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Content {
    pub npcs: BTreeMap<String, Npc>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Npc {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub room: Option<String>,
    #[serde(default)]
    pub requires: Vec<Condition>,
    pub start: String,
    pub nodes: BTreeMap<String, Node>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub text: String,
    #[serde(default)]
    pub variants: Vec<Variant>,
    #[serde(default)]
    pub choices: Vec<Choice>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub when: Vec<Condition>,
    pub text: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub requires: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub text: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub npcs: BTreeMap<String, NpcState>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcState {
    pub room: Option<String>,
    pub node: String,
}
impl Progress {
    pub fn new(content: &Content) -> Self {
        Self {
            npcs: content
                .npcs
                .iter()
                .map(|(id, n)| {
                    (
                        id.clone(),
                        NpcState {
                            room: n.room.clone(),
                            node: n.start.clone(),
                        },
                    )
                })
                .collect(),
        }
    }
}
pub fn node(w: &World, npc: &str, id: &str) -> Result<(), String> {
    if w.dialogue
        .npcs
        .get(npc)
        .is_some_and(|n| n.nodes.contains_key(id))
    {
        Ok(())
    } else {
        Err(format!("Unknown dialogue node '{npc}:{id}'"))
    }
}
impl Content {
    pub fn load(root: &Path) -> Result<Self, String> {
        let p = root.join("dialogue.toml");
        let text = match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(format!("{}: {e}", p.display())),
        };
        let value: toml::Value =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?;
        crate::rules::decode(&value).map_err(|e| format!("{}: {e}", p.display()))
    }
    pub fn validate(&self, w: &World) -> Result<(), String> {
        if self.npcs.len() > 1024 {
            return Err("At most 1024 NPCs are supported".into());
        }
        for (id, n) in &self.npcs {
            crate::rules::valid_id(id)?;
            if n.name.trim().is_empty()
                || n.nodes.is_empty()
                || n.nodes.len() > 128
                || n.aliases.len() > 128
            {
                return Err(format!(
                    "NPC '{id}' requires a name, 1..128 nodes and at most 128 aliases"
                ));
            }
            if let Some(room) = &n.room {
                Condition::InRoom(room.clone()).validate(w)?;
            }
            node(w, id, &n.start)?;
            for c in &n.requires {
                c.validate(w)?;
            }
            for (nid, v) in &n.nodes {
                crate::rules::valid_id(nid)?;
                if v.choices.len() > 128 || v.variants.len() > 128 {
                    return Err(format!(
                        "NPC '{id}' node '{nid}' exceeds 128 choices/variants"
                    ));
                }
                for variant in &v.variants {
                    for c in &variant.when {
                        c.validate(w)?;
                    }
                }
                let mut ids = std::collections::BTreeSet::new();
                for choice in &v.choices {
                    crate::rules::valid_id(&choice.id)?;
                    if !ids.insert(&choice.id)
                        || choice.label.trim().is_empty()
                        || choice.effects.len() > 256
                    {
                        return Err(format!(
                            "Invalid/duplicate choice '{}' in '{id}:{nid}'",
                            choice.id
                        ));
                    }
                    if let Some(next) = &choice.next {
                        node(w, id, next)?;
                    }
                    for c in &choice.requires {
                        c.validate(w)?;
                    }
                    for e in &choice.effects {
                        e.validate(w)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn validate_state(&self, s: &Progress, w: &World) -> Result<(), String> {
        if s.npcs.keys().ne(self.npcs.keys()) {
            return Err("Saved NPC IDs differ from dialogue.toml".into());
        }
        for (id, n) in &s.npcs {
            node(w, id, &n.node)?;
            if let Some(room) = &n.room {
                Condition::InRoom(room.clone()).validate(w)?;
            }
        }
        Ok(())
    }
}
pub fn candidates(w: &World, s: &GameState) -> Vec<Candidate> {
    w.dialogue
        .npcs
        .iter()
        .filter(|(id, n)| {
            s.dialogue
                .npcs
                .get(*id)
                .is_some_and(|p| p.room.as_ref() == Some(&s.current_room))
                && n.requires.iter().all(|c| c.met(s))
        })
        .map(|(id, n)| Candidate {
            id: id.clone(),
            name: n.name.clone(),
            aliases: n.aliases.clone(),
        })
        .collect()
}
pub fn people(w: &World, s: &GameState) -> String {
    let people = candidates(w, s);
    if people.is_empty() {
        "People: none".into()
    } else {
        format!(
            "People: {}. Use 'talk <name>'.",
            people
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
/// Apply one currently available choice. State and success text publish together.
pub fn select(
    w: &World,
    s: &mut GameState,
    npc: &str,
    choice: &str,
) -> Result<Vec<String>, String> {
    w.dialogue.validate_state(&s.dialogue, w)?;
    if !candidates(w, s).iter().any(|c| c.id == npc) {
        return Err("That person is not available here.".into());
    }
    let n = &w.dialogue.npcs[npc];
    let current = &n.nodes[&s.dialogue.npcs[npc].node];
    let choice = current
        .choices
        .iter()
        .find(|c| c.id == choice && c.requires.iter().all(|c| c.met(s)))
        .ok_or("That dialogue choice is not available.")?;
    let mut draft = s.clone();
    if let Some(next) = &choice.next {
        draft.dialogue.npcs.get_mut(npc).unwrap().node = next.clone();
    }
    let mut lines = crate::rules::apply(w, &mut draft, &choice.effects)?;
    w.dialogue.validate_state(&draft.dialogue, w)?;
    if !choice.text.is_empty() {
        lines.insert(0, choice.text.clone());
    }
    *s = draft;
    Ok(lines)
}
/// Viewing and cancellation are read-only; one confirmed choice returns to gameplay.
pub(crate) fn talk(
    w: &World,
    s: &mut GameState,
    query: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<bool> {
    let Some(id) = crate::player_input::choose(query, &candidates(w, s), input, output)? else {
        return Ok(false);
    };
    let npc = &w.dialogue.npcs[&id];
    let n = &npc.nodes[&s.dialogue.npcs[&id].node];
    let text = n
        .variants
        .iter()
        .find(|v| v.when.iter().all(|c| c.met(s)))
        .map_or(n.text.as_str(), |v| v.text.as_str());
    writeln!(output, "{}: {}\n{}", npc.name, npc.description, text)?;
    let choices = n
        .choices
        .iter()
        .filter(|c| c.requires.iter().all(|c| c.met(s)))
        .collect::<Vec<_>>();
    if choices.is_empty() {
        writeln!(output, "No dialogue choices are available.\n")?;
        return Ok(false);
    }
    for (i, c) in choices.iter().enumerate() {
        writeln!(output, "  {}. {} [{}]", i + 1, c.label, c.id)?;
    }
    loop {
        write!(
            output,
            "Choose a response (number or ID; 0/back/cancel cancels):\n> "
        )?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(false);
        }
        let answer = line.trim();
        if matches!(
            answer.to_ascii_lowercase().as_str(),
            "0" | "back" | "cancel" | "quit"
        ) {
            writeln!(output, "Conversation canceled. Nothing changed.\n")?;
            return Ok(false);
        }
        let selected = answer
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|i| choices.get(i))
            .copied()
            .or_else(|| {
                choices
                    .iter()
                    .copied()
                    .find(|c| c.id.eq_ignore_ascii_case(answer))
            });
        if let Some(c) = selected {
            return match select(w, s, &id, &c.id) {
                Ok(lines) => {
                    for line in lines {
                        writeln!(output, "{line}")?;
                    }
                    writeln!(output)?;
                    Ok(true)
                }
                Err(e) => {
                    writeln!(output, "{e}\n")?;
                    Ok(false)
                }
            };
        }
        writeln!(output, "Choose a listed number or ID, or cancel.")?;
    }
}
