//! Bounded deterministic story events, turn timers, quests, journals and endings.
use crate::{
    rules::{Condition, Effect},
    state::GameState,
    world::World,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
};
const LIMIT: usize = 128;
fn yes() -> bool {
    true
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    Enter(String),
    Take(String),
    Action(String),
    Defeat(String),
    Timer(String),
    Turn,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: String,
    pub on: Trigger,
    #[serde(default = "yes")]
    pub once: bool,
    #[serde(default)]
    pub requires: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub text: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timer {
    pub turns: u32,
    #[serde(default)]
    pub repeat: bool,
    #[serde(default)]
    pub start: bool,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub title: String,
    #[serde(default)]
    pub initial: Option<String>,
    pub stages: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalEntry {
    pub title: String,
    pub text: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndingKind {
    Success,
    Failure,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ending {
    pub kind: EndingKind,
    pub text: String,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Content {
    #[serde(skip)]
    pub events: Vec<Event>,
    pub timers: BTreeMap<String, Timer>,
    pub quests: BTreeMap<String, Quest>,
    pub journal: BTreeMap<String, JournalEntry>,
    pub endings: BTreeMap<String, Ending>,
}
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
#[derive(Default)]
struct File {
    events: Vec<toml::Value>,
    timers: BTreeMap<String, Timer>,
    quests: BTreeMap<String, Quest>,
    journal: BTreeMap<String, JournalEntry>,
    endings: BTreeMap<String, Ending>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveTimer {
    pub remaining: u32,
    pub generation: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub turn: u64,
    pub completed: Vec<String>,
    pub timers: BTreeMap<String, ActiveTimer>,
    pub next_generation: u64,
    pub quests: BTreeMap<String, String>,
    pub journal: Vec<String>,
    pub ending: Option<String>,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            turn: 0,
            completed: vec![],
            timers: BTreeMap::new(),
            next_generation: 1,
            quests: BTreeMap::new(),
            journal: vec![],
            ending: None,
        }
    }
}
impl Progress {
    pub fn new(content: &Content) -> Self {
        let mut state = Self::default();
        for (id, q) in &content.quests {
            if let Some(stage) = &q.initial {
                state.quests.insert(id.clone(), stage.clone());
            }
        }
        for (id, t) in &content.timers {
            if t.start {
                state.timers.insert(
                    id.clone(),
                    ActiveTimer {
                        remaining: t.turns,
                        generation: state.next_generation,
                    },
                );
                state.next_generation += 1;
            }
        }
        state
    }
}
impl Content {
    pub fn load(root: &Path) -> Result<Self, String> {
        let p = root.join("events.toml");
        let text = match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(format!("{}: {e}", p.display())),
        };
        let f: File = toml::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?;
        let events = f
            .events
            .iter()
            .enumerate()
            .map(|(i, v)| {
                crate::rules::decode(v)
                    .map_err(|e| format!("{} event #{}: {e}", p.display(), i + 1))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            events,
            timers: f.timers,
            quests: f.quests,
            journal: f.journal,
            endings: f.endings,
        })
    }
    pub fn validate(&self, w: &World) -> Result<(), String> {
        if self.events.len() > 1024
            || self.timers.len() > 1024
            || self.quests.len() > 1024
            || self.journal.len() > 1024
            || self.endings.len() > 1024
        {
            return Err("events.toml: at most 1024 entries per category".into());
        }
        let mut ids = BTreeSet::new();
        for e in &self.events {
            crate::rules::valid_id(&e.id)?;
            if !ids.insert(&e.id) {
                return Err(format!("Duplicate event '{}'", e.id));
            }
            match &e.on {
                Trigger::Enter(id) => Condition::InRoom(id.clone()).validate(w)?,
                Trigger::Take(id) => Condition::HasItem(id.clone()).validate(w)?,
                Trigger::Action(id) if !w.actions.iter().any(|a| &a.id == id) => {
                    return Err(format!("Event '{}' references unknown action '{id}'", e.id))
                }
                Trigger::Timer(id) if !self.timers.contains_key(id) => {
                    return Err(format!("Event '{}' references unknown timer '{id}'", e.id))
                }
                Trigger::Defeat(id) => crate::rules::valid_id(id)?,
                _ => (),
            }
            if e.effects.len() > 256 {
                return Err(format!("Event '{}' exceeds 256 effects", e.id));
            }
            for c in &e.requires {
                c.validate(w)?;
            }
            for effect in &e.effects {
                effect.validate(w)?;
            }
        }
        for id in self
            .timers
            .keys()
            .chain(self.quests.keys())
            .chain(self.journal.keys())
            .chain(self.endings.keys())
        {
            crate::rules::valid_id(id)?;
        }
        for (id, t) in &self.timers {
            if t.turns == 0 || t.turns > 1_000_000 {
                return Err(format!("Timer '{id}' needs 1..1000000 turns"));
            }
        }
        for (id, q) in &self.quests {
            if q.title.trim().is_empty() || q.stages.is_empty() || q.stages.len() > 128 {
                return Err(format!("Quest '{id}' needs a title and 1..128 stages"));
            }
            for stage in q.stages.keys() {
                crate::rules::valid_id(stage)?;
            }
            if q.initial
                .as_ref()
                .is_some_and(|s| !q.stages.contains_key(s))
            {
                return Err(format!("Quest '{id}' has an unknown initial stage"));
            }
        }
        Ok(())
    }
    pub fn validate_state(&self, s: &Progress) -> Result<(), String> {
        if s.next_generation == 0 {
            return Err("Invalid timer generation counter".into());
        }
        let mut generations = BTreeSet::new();
        for (id, t) in &s.timers {
            let definition = self
                .timers
                .get(id)
                .ok_or_else(|| format!("Unknown saved timer '{id}'"))?;
            if t.remaining == 0
                || t.remaining > definition.turns
                || t.generation == 0
                || t.generation >= s.next_generation
                || !generations.insert(t.generation)
            {
                return Err(format!("Invalid saved timer '{id}'"));
            }
        }
        let mut seen = BTreeSet::new();
        for id in &s.completed {
            if !seen.insert(id) || !self.events.iter().any(|e| &e.id == id && e.once) {
                return Err(format!(
                    "Unknown, repeatable or duplicate completed event '{id}'"
                ));
            }
        }
        for (id, stage) in &s.quests {
            if !self
                .quests
                .get(id)
                .is_some_and(|q| q.stages.contains_key(stage))
            {
                return Err(format!("Unknown saved quest/stage '{id}:{stage}'"));
            }
        }
        let mut seen = BTreeSet::new();
        for id in &s.journal {
            if !seen.insert(id) || !self.journal.contains_key(id) {
                return Err(format!("Unknown or duplicate journal entry '{id}'"));
            }
        }
        if s.ending
            .as_ref()
            .is_some_and(|id| !self.endings.contains_key(id))
        {
            return Err("Unknown saved ending".into());
        }
        Ok(())
    }
}
pub fn start_timer(w: &World, s: &mut GameState, id: &str) -> Result<(), String> {
    let timer = w
        .events
        .timers
        .get(id)
        .ok_or_else(|| format!("Unknown timer '{id}'"))?;
    let generation = s.events.next_generation;
    s.events.next_generation = generation
        .checked_add(1)
        .ok_or("Timer generation overflow")?;
    s.events.timers.insert(
        id.into(),
        ActiveTimer {
            remaining: timer.turns,
            generation,
        },
    );
    Ok(())
}
pub struct Step {
    pub before: GameState,
    pub performed: bool,
    pub combat_turn: bool,
    pub triggers: Vec<Trigger>,
    pub eligible: bool,
}
impl Step {
    pub fn new(state: &GameState, eligible: bool) -> Self {
        Self {
            before: state.clone(),
            performed: false,
            combat_turn: false,
            triggers: vec![],
            eligible,
        }
    }
}
fn changes(before: &GameState, after: &GameState) -> Vec<Trigger> {
    let mut changes = Vec::new();
    if before.current_room != after.current_room {
        changes.push(Trigger::Enter(after.current_room.clone()));
    }
    for id in after.objects.placements.keys() {
        if after.carried_quantity(id) > before.carried_quantity(id) {
            changes.push(Trigger::Take(id.clone()));
        }
    }
    changes
}
fn dispatch(
    w: &World,
    s: &mut GameState,
    queue: &mut VecDeque<Trigger>,
    output: &mut Vec<String>,
    budget: &mut usize,
) -> Result<(), String> {
    while let Some(trigger) = queue.pop_front() {
        if s.events.ending.is_some() {
            break;
        }
        for event in &w.events.events {
            if event.on != trigger
                || (event.once && s.events.completed.contains(&event.id))
                || !event.requires.iter().all(|c| c.met(s))
            {
                continue;
            }
            *budget += 1;
            if *budget > LIMIT {
                return Err(
                    "Event chain exceeded 128 firings; check repeating enter/acquisition effects"
                        .into(),
                );
            }
            let before = s.clone();
            let lines = crate::rules::apply(w, s, &event.effects)
                .map_err(|e| format!("Event '{}': {e}", event.id))?;
            if event.once {
                s.events.completed.push(event.id.clone());
            }
            if !event.text.is_empty() {
                output.push(event.text.clone());
            }
            output.extend(lines);
            queue.extend(changes(&before, s));
            if s.events.ending.is_some() {
                break;
            }
        }
    }
    Ok(())
}
/// Complete a turn; on failure restore the state captured before its player action.
pub fn finish(w: &World, state: &mut GameState, step: Step) -> Result<Vec<String>, String> {
    match finish_inner(w, state, &step) {
        Ok(lines) => Ok(lines),
        Err(error) => {
            *state = step.before;
            Err(error)
        }
    }
}
fn finish_inner(w: &World, state: &mut GameState, step: &Step) -> Result<Vec<String>, String> {
    w.events.validate_state(&state.events)?;
    if !step.eligible
        || (!step.performed && *state == step.before)
        || step.before.events.ending.is_some()
    {
        return Ok(vec![]);
    }
    let mut draft = state.clone();
    let mut output = Vec::new();
    let mut budget = 0;
    draft.events.turn = draft
        .events
        .turn
        .checked_add(1)
        .ok_or("Turn counter overflow")?;
    let mut queue = VecDeque::from(step.triggers.clone());
    for trigger in changes(&step.before, &draft) {
        if !queue.contains(&trigger) {
            queue.push_back(trigger);
        }
    }
    dispatch(w, &mut draft, &mut queue, &mut output, &mut budget)?;
    // Only timers present before the command and not restarted during it advance.
    for (id, old) in &step.before.events.timers {
        if draft.events.ending.is_some() {
            break;
        }
        if draft
            .events
            .timers
            .get(id)
            .is_some_and(|t| t.generation == old.generation)
        {
            let timer = draft.events.timers.get_mut(id).unwrap();
            timer.remaining -= 1;
            if timer.remaining == 0 {
                draft.events.timers.remove(id);
                if w.events.timers[id].repeat {
                    start_timer(w, &mut draft, id)?;
                }
                queue.push_back(Trigger::Timer(id.clone()));
                dispatch(w, &mut draft, &mut queue, &mut output, &mut budget)?;
            }
        }
    }
    queue.push_back(Trigger::Turn);
    dispatch(w, &mut draft, &mut queue, &mut output, &mut budget)?;
    w.events.validate_state(&draft.events)?;
    *state = draft;
    Ok(output)
}
pub fn start(w: &World, state: &mut GameState) -> Result<Vec<String>, String> {
    w.events.validate_state(&state.events)?;
    let mut draft = state.clone();
    let mut output = Vec::new();
    let mut budget = 0;
    let mut queue = VecDeque::from([Trigger::Enter(draft.current_room.clone())]);
    dispatch(w, &mut draft, &mut queue, &mut output, &mut budget)?;
    w.events.validate_state(&draft.events)?;
    *state = draft;
    Ok(output)
}
pub fn journal(w: &World, state: &GameState) -> String {
    let mut lines = Vec::new();
    for (id, stage) in &state.events.quests {
        if let Some(q) = w.events.quests.get(id) {
            lines.push(format!(
                "{}\n{}",
                q.title,
                q.stages.get(stage).map_or("", String::as_str)
            ));
        }
    }
    for id in &state.events.journal {
        if let Some(entry) = w.events.journal.get(id) {
            lines.push(format!("{}\n{}", entry.title, entry.text));
        }
    }
    if lines.is_empty() {
        "Your journal is empty.".into()
    } else {
        lines.join("\n\n")
    }
}
