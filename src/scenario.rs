//! Read-only, file-authored acceptance scenarios through the real player runner.
use crate::adventure::Adventure;
use serde::Deserialize;
use std::{
    collections::{BTreeSet, HashSet},
    io::{self, Cursor, Write},
    path::Path,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    format_version: u32,
    #[serde(default = "default_seed")]
    seed: u64,
    scenario: Vec<Scenario>,
}
fn default_seed() -> u64 {
    1
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    name: String,
    #[serde(default)]
    setup: Vec<String>,
    commands: Vec<String>,
    #[serde(default)]
    contains: Vec<String>,
    #[serde(default)]
    excludes: Vec<String>,
    room: Option<String>,
    /// When present, assert the entire inventory, not merely a subset.
    inventory: Option<Vec<String>>,
    flags: Option<Vec<String>>,
    counters: Option<std::collections::BTreeMap<String, i64>>,
    visited: Option<Vec<String>>,
    hidden: Option<Vec<String>>,
    #[serde(default)]
    quantities: Vec<QuantityAssertion>,
    turn: Option<u64>,
    timers: Option<std::collections::BTreeMap<String, u32>>,
    quests: Option<std::collections::BTreeMap<String, String>>,
    journal: Option<Vec<String>>,
    ending: Option<String>,
    ended: Option<bool>,
    completed_events: Option<Vec<String>>,
    npcs: Option<std::collections::BTreeMap<String, crate::dialogue::NpcState>>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct QuantityAssertion {
    item: String,
    location: crate::state::ItemLocation,
    quantity: u32,
}

fn validate_lines(lines: &[String]) -> Result<(), String> {
    for line in lines {
        if line.chars().any(char::is_control) {
            return Err("Each input must be one line without control characters".into());
        }
        let verb = line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if ["save", "load", "saves"].contains(&verb.as_str()) {
            return Err(
                "Scenario commands cannot access player saves; use fresh setup commands".into(),
            );
        }
    }
    Ok(())
}

/// Execute independent fresh sessions. No author files or player saves are written.
/// All failures are reported; a failing assertion makes the CLI exit unsuccessfully.
pub fn run(root: &Path, path: &Path, output: &mut impl Write) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let suite: Suite = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if suite.format_version != 1 || suite.scenario.is_empty() || suite.seed == 0 {
        return Err(
            "Scenario suite needs format_version = 1, a nonzero seed and at least one scenario"
                .into(),
        );
    }
    let mut names = HashSet::new();
    // Validate the complete suite before running any scenario.
    for case in &suite.scenario {
        if case.name.trim().is_empty() || !names.insert(&case.name) {
            return Err("Scenario names must be nonempty and unique".into());
        }
        validate_lines(&case.setup)?;
        validate_lines(&case.commands)?;
    }
    let mut failures = Vec::new();
    for case in suite.scenario {
        let a = Adventure::load(root)?;
        let commands = case
            .setup
            .iter()
            .chain(&case.commands)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let mut transcript = Vec::new();
        let state = crate::runner::run_session_with_io(
            a.game.world,
            &a.game.title,
            &a.game.id,
            Path::new(""),
            a.game.save_compatibility_version,
            a.game.config,
            a.rpg,
            a.encounters,
            a.wallet,
            a.shops,
            Cursor::new(commands),
            &mut transcript,
            Some(suite.seed),
        )
        .map_err(|e| e.to_string())?;
        let transcript = String::from_utf8(transcript).map_err(|e| e.to_string())?;
        let mut errors = Vec::new();
        for expected in &case.contains {
            if !transcript.contains(expected) {
                errors.push(format!("missing output {expected:?}"));
            }
        }
        for forbidden in &case.excludes {
            if transcript.contains(forbidden) {
                errors.push(format!("unexpected output {forbidden:?}"));
            }
        }
        if let Some(room) = &case.room {
            if state.current_room != *room {
                errors.push(format!(
                    "expected room {room:?}, got {:?}",
                    state.current_room
                ));
            }
        }
        if let Some(inventory) = &case.inventory {
            let actual = state
                .objects
                .placements
                .keys()
                .filter(|id| state.is_carried(id))
                .cloned()
                .collect::<BTreeSet<_>>();
            if actual != inventory.iter().cloned().collect() {
                errors.push(format!("expected inventory {inventory:?}, got {actual:?}"));
            }
        }
        if let Some(flags) = &case.flags {
            if state.flags != flags.iter().cloned().collect() {
                errors.push(format!("expected flags {flags:?}, got {:?}", state.flags));
            }
        }
        if let Some(counters) = &case.counters {
            if state.story.counters != *counters {
                errors.push(format!(
                    "expected counters {counters:?}, got {:?}",
                    state.story.counters
                ));
            }
        }
        if let Some(visited) = &case.visited {
            if state.story.visited != visited.iter().cloned().collect() {
                errors.push(format!(
                    "expected visited rooms {visited:?}, got {:?}",
                    state.story.visited
                ));
            }
        }
        if let Some(hidden) = &case.hidden {
            if state.objects.hidden != hidden.iter().cloned().collect() {
                errors.push(format!(
                    "expected hidden items {hidden:?}, got {:?}",
                    state.objects.hidden
                ));
            }
        }
        for assertion in &case.quantities {
            if !state.objects.placements.contains_key(&assertion.item)
                || state.count_at(&assertion.item, &assertion.location) != assertion.quantity
            {
                errors.push(format!(
                    "expected {} of '{}' at {:?}, got {}",
                    assertion.quantity,
                    assertion.item,
                    assertion.location,
                    state.count_at(&assertion.item, &assertion.location)
                ));
            }
        }
        if let Some(expected) = &case.npcs {
            if expected != &state.dialogue.npcs {
                errors.push(format!(
                    "NPC states: expected {expected:?}, got {:?}",
                    state.dialogue.npcs
                ));
            }
        }
        if case.turn.is_some_and(|n| n != state.events.turn) {
            errors.push(format!(
                "expected turn {:?}, got {}",
                case.turn, state.events.turn
            ));
        }
        if let Some(expected) = &case.timers {
            let actual = state
                .events
                .timers
                .iter()
                .map(|(id, t)| (id.clone(), t.remaining))
                .collect::<std::collections::BTreeMap<_, _>>();
            if expected != &actual {
                errors.push(format!("expected timers {expected:?}, got {actual:?}"));
            }
        }
        if case
            .quests
            .as_ref()
            .is_some_and(|q| q != &state.events.quests)
        {
            errors.push(format!(
                "expected quests {:?}, got {:?}",
                case.quests, state.events.quests
            ));
        }
        if case
            .journal
            .as_ref()
            .is_some_and(|q| q != &state.events.journal)
        {
            errors.push(format!(
                "expected journal {:?}, got {:?}",
                case.journal, state.events.journal
            ));
        }
        if case
            .completed_events
            .as_ref()
            .is_some_and(|q| q != &state.events.completed)
        {
            errors.push(format!(
                "expected completed events {:?}, got {:?}",
                case.completed_events, state.events.completed
            ));
        }
        if case
            .ended
            .is_some_and(|v| v != state.events.ending.is_some())
            || case
                .ending
                .as_ref()
                .is_some_and(|e| Some(e) != state.events.ending.as_ref())
        {
            errors.push(format!(
                "expected ending {:?} / ended {:?}, got {:?}",
                case.ending, case.ended, state.events.ending
            ));
        }
        if errors.is_empty() {
            writeln!(output, "PASS: {}", case.name).map_err(io_error)?;
        } else {
            let detail = format!("FAIL: {}: {}\n{transcript}", case.name, errors.join("; "));
            writeln!(output, "{detail}").map_err(io_error)?;
            failures.push(case.name);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} scenario(s) failed: {}",
            failures.len(),
            failures.join(", ")
        ))
    }
}
fn io_error(e: io::Error) -> String {
    e.to_string()
}
