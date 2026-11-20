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
    #[serde(default)]
    pub ask_name: bool,
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
fn linked(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}
impl Content {
    pub fn load(root: &Path) -> Result<Self, String> {
        // This is a content-layout change only. Runtime IDs and saved progress stay unchanged.
        if std::fs::symlink_metadata(root.join("dialogue.toml")).is_ok() {
            return Err(format!("{}: dialogue.toml is no longer supported. Move each [npcs.ID] definition to NPCs/ID.toml, add id = \"ID\", and change [npcs.ID.nodes.*] to [nodes.*]. Remove dialogue.toml after reviewing the split. Do not edit RORK saves.", root.display()));
        }
        for entry in std::fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))? {
            let name = entry.map_err(|e| e.to_string())?.file_name();
            if name
                .to_str()
                .is_some_and(|n| n.eq_ignore_ascii_case("npcs") && n != "NPCs")
            {
                return Err(format!(
                    "{}: name the NPC directory exactly NPCs/ for Windows/Linux portability",
                    root.join(name).display()
                ));
            }
        }
        let dir = root.join("NPCs");
        let metadata = match std::fs::symlink_metadata(&dir) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(format!("{}: {e}", dir.display())),
        };
        if linked(&metadata) || !metadata.is_dir() {
            return Err(format!(
                "{}: expected a regular directory, not a link",
                dir.display()
            ));
        }
        let mut paths = Vec::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let meta =
                std::fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            if linked(&meta) || !meta.is_file() {
                return Err(format!(
                    "{}: NPCs/ accepts regular files directly, without links or subdirectories",
                    path.display()
                ));
            }
            if path.extension().is_some_and(|e| e == "toml") {
                if meta.len() > 8 * 1024 * 1024 {
                    return Err(format!("{}: NPC definition exceeds 8 MiB", path.display()));
                }
                paths.push(path);
                if paths.len() > 1024 {
                    return Err(format!(
                        "{}: at most 1024 NPC definitions are supported",
                        dir.display()
                    ));
                }
            }
        }
        paths.sort();
        let mut content = Self::default();
        let mut origins = BTreeMap::new();
        let mut total = 0usize;
        for path in paths {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            total += text.len();
            if text.len() > 8 * 1024 * 1024 || total > 64 * 1024 * 1024 {
                return Err(format!(
                    "{}: NPC content exceeds size limits",
                    path.display()
                ));
            }
            let parsed = (|| -> Result<(String, Npc), String> {
                let mut value: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
                let id = value
                    .as_table_mut()
                    .and_then(|t| t.remove("id"))
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .ok_or("NPC requires a string id")?;
                crate::rules::valid_id(&id)?;
                let npc = crate::rules::decode(&value)?;
                Ok((id, npc))
            })()
            .map_err(|e| format!("{}: {e}", path.display()))?;
            let (id, npc) = parsed;
            if let Some(previous) = origins.insert(id.clone(), path.clone()) {
                return Err(format!(
                    "{}: duplicate NPC id '{id}' (also defined in {})",
                    path.display(),
                    previous.display()
                ));
            }
            content.npcs.insert(id, npc);
        }
        Ok(content)
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
            return Err("Saved NPC IDs differ from NPCs/ definitions".into());
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
    select_with_name(w, s, npc, choice, None)
}

pub fn select_with_name(
    w: &World,
    s: &mut GameState,
    npc: &str,
    choice: &str,
    player_name: Option<&str>,
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
    if choice.ask_name != player_name.is_some() {
        return Err("This choice requires exactly its authored name input.".into());
    }
    let mut draft = s.clone();
    if let Some(name) = player_name {
        validate_player_name(name)?;
        draft.player_name = Some(name.into());
    }
    if let Some(next) = &choice.next {
        draft.dialogue.npcs.get_mut(npc).unwrap().node = next.clone();
    }
    let mut lines = crate::rules::apply(w, &mut draft, &choice.effects)?;
    w.dialogue.validate_state(&draft.dialogue, w)?;
    if !choice.text.is_empty() {
        lines.insert(0, player_text(&choice.text, &draft));
    }
    *s = draft;
    Ok(lines)
}
/// Viewing and cancellation are read-only; one confirmed choice returns to gameplay.
pub(crate) fn prepare(
    w: &World,
    s: &GameState,
    query: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<Option<PreparedChoice>> {
    let Some(id) = crate::player_input::choose(query, &candidates(w, s), input, output)? else {
        return Ok(None);
    };
    let npc = &w.dialogue.npcs[&id];
    let n = &npc.nodes[&s.dialogue.npcs[&id].node];
    let text = n
        .variants
        .iter()
        .find(|v| v.when.iter().all(|c| c.met(s)))
        .map_or(n.text.as_str(), |v| v.text.as_str());
    writeln!(
        output,
        "{}: {}\n{}",
        npc.name,
        npc.description,
        player_text(text, s)
    )?;
    let choices = n
        .choices
        .iter()
        .filter(|c| c.requires.iter().all(|c| c.met(s)))
        .collect::<Vec<_>>();
    if choices.is_empty() {
        writeln!(output, "No dialogue choices are available.\n")?;
        return Ok(None);
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
            return Ok(None);
        }
        let answer = line.trim();
        if matches!(
            answer.to_ascii_lowercase().as_str(),
            "0" | "back" | "cancel" | "quit"
        ) {
            writeln!(output, "Conversation canceled. Nothing changed.\n")?;
            return Ok(None);
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
            let player_name = if c.ask_name {
                loop {
                    writeln!(
                        output,
                        "What is your name? (1-40 characters; 0/back/cancel cancels):\n> "
                    )?;
                    output.flush()?;
                    let mut name = String::new();
                    if input.read_line(&mut name)? == 0 {
                        return Ok(None);
                    }
                    let name = name.trim_end_matches(['\r', '\n']).trim_matches(' ');
                    if matches!(name.to_ascii_lowercase().as_str(), "0" | "back" | "cancel") {
                        writeln!(output, "Conversation canceled. Nothing changed.\n")?;
                        return Ok(None);
                    }
                    match validate_player_name(name) {
                        Ok(()) => break Some(name.to_owned()),
                        Err(error) => writeln!(output, "{error}")?,
                    }
                }
            } else {
                None
            };
            return Ok(Some(PreparedChoice {
                npc: id.clone(),
                choice: c.id.clone(),
                player_name,
            }));
        }
        writeln!(output, "Choose a listed number or ID, or cancel.")?;
    }
}

pub(crate) struct PreparedChoice {
    pub npc: String,
    pub choice: String,
    pub player_name: Option<String>,
}

pub fn validate_player_name(name: &str) -> Result<(), String> {
    if name.trim() != name
        || name.is_empty()
        || name.chars().count() > 40
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        return Err("Use a name of 1-40 visible characters, without surrounding whitespace or control characters.".into());
    }
    Ok(())
}

pub fn player_text(text: &str, state: &GameState) -> String {
    text.replace(
        "{{player_name}}",
        state.player_name.as_deref().unwrap_or("traveler"),
    )
}
