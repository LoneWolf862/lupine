//! Read-only command preparation and clarification before existing rule validation.
use crate::{
    commands::Command,
    encounters::EncounterContent,
    parser::{clean_target, parse_command},
    resolution::{self, Candidate},
    rpg_content::RpgContent,
    state::GameState,
    world::World,
};
use std::io::{self, BufRead, Write};

/// Prepared ordinary targets are stable IDs. Do not strip articles from IDs or
/// reinterpret them as fresh natural-language input after clarification.
pub(crate) fn prepared_command(line: &str) -> Result<Command, crate::parser::ParseError> {
    if let Some((verb, target)) = line.split_once(' ') {
        match verb {
            "take" => return Ok(Command::Take(target.into())),
            "drop" => return Ok(Command::Drop(target.into())),
            "examine" => return Ok(Command::Examine(target.into())),
            "use" => {
                if let Some((item, target)) = target.split_once(" on ") {
                    return Ok(Command::Use {
                        item: item.into(),
                        target: target.into(),
                    });
                }
            }
            _ => (),
        }
    }
    parse_command(line)
}

pub(crate) fn choose(
    query: &str,
    candidates: &[Candidate],
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<Option<String>> {
    let found = resolution::matches(query, candidates);
    choose_matches(query, candidates, found, false, input, output)
}

fn choose_matches(
    query: &str,
    candidates: &[Candidate],
    found: Vec<&Candidate>,
    force_choice: bool,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<Option<String>> {
    match found.as_slice() {
        [one] if !force_choice => Ok(Some(one.id.clone())),
        [] => {
            writeln!(
                output,
                "{}\n",
                resolution::unique(query, candidates).unwrap_err()
            )?;
            Ok(None)
        }
        many => {
            writeln!(output, "Which {query} did you mean?")?;
            for (index, candidate) in many.iter().enumerate() {
                writeln!(
                    output,
                    "  {}. {} [{}]",
                    index + 1,
                    candidate.name,
                    candidate.id
                )?;
            }
            loop {
                write!(
                    output,
                    "Selection (number or exact ID; 0/back/cancel cancels):\n> "
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
                    writeln!(output, "Command canceled. Nothing changed.\n")?;
                    return Ok(None);
                }
                let selected = answer
                    .parse::<usize>()
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .and_then(|i| many.get(i))
                    .copied()
                    .or_else(|| {
                        many.iter()
                            .copied()
                            .find(|c| c.id.eq_ignore_ascii_case(answer))
                    });
                if let Some(c) = selected {
                    return Ok(Some(c.id.clone()));
                }
                writeln!(output, "Choose a listed number or ID, or cancel.")?;
            }
        }
    }
}

pub fn enemies(encounters: &EncounterContent, state: &GameState) -> Vec<Candidate> {
    let mut choices = state
        .active_enemies
        .iter()
        .filter_map(|instance| {
            let enemy = encounters.enemies.get(&instance.enemy_id)?;
            let mut c = Candidate::new(
                format!("{} #{}", enemy.id, instance.instance_id),
                format!("{} #{}", enemy.name, instance.instance_id),
            );
            c.aliases = vec![
                enemy.id.clone(),
                enemy.name.clone(),
                format!("#{}", instance.instance_id),
            ];
            Some(c)
        })
        .collect::<Vec<_>>();
    choices.sort_by(|a, b| a.id.cmp(&b.id));
    choices
}

pub fn abilities(content: &RpgContent, state: &GameState) -> Vec<Candidate> {
    let mut choices = content
        .abilities
        .values()
        .filter(|a| state.learned_abilities.contains(&a.id))
        .map(|a| Candidate::new(&a.id, &a.name))
        .collect::<Vec<_>>();
    choices.sort_by(|a, b| a.id.cmp(&b.id));
    choices
}

/// Resolves all references with immutable state. A canceled/failed preparation never
/// reaches the runner's existing combat, shop, spell or engine mutation paths.
#[allow(clippy::too_many_arguments)]
pub fn prepare(
    line: &str,
    world: &World,
    state: &GameState,
    content: &RpgContent,
    encounters: &EncounterContent,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<Option<String>> {
    let normalized = resolution::normalize(line);
    let (verb, argument) = normalized.split_once(' ').unwrap_or((&normalized, ""));
    let normalized = if matches!(verb, "stab" | "slash" | "hit" | "strike") {
        format!("attack {argument}")
    } else if matches!(verb, "cast" | "use") && !argument.contains(" on ") {
        if let Some((source, target)) = argument.split_once(" at ") {
            format!("{verb} {source} on {target}")
        } else {
            normalized
        }
    } else {
        normalized
    };
    let (verb, argument) = normalized.split_once(' ').unwrap_or((&normalized, ""));
    let visible = resolution::items(world, state);
    let carried = visible
        .iter()
        .filter(|i| state.is_carried(&i.id))
        .cloned()
        .collect::<Vec<_>>();
    let enemy_choices = enemies(encounters, state);
    let spells = abilities(content, state);
    let weapons = carried
        .iter()
        .filter(|i| content.weapons.contains_key(&i.id))
        .cloned()
        .collect::<Vec<_>>();
    let select = |q: &str, choices: &[Candidate], input: &mut _, output: &mut _| {
        choose(&clean_target(q), choices, input, output)
    };

    if verb == "attack" {
        if let Some((enemy, weapon)) = argument.rsplit_once(" with ") {
            // Explicit attack-with may name a learned spell, but carried weapons
            // retain priority and misspellings never select either automatically.
            if resolution::matches(&clean_target(weapon), &weapons).is_empty()
                && !resolution::matches(&clean_target(weapon), &spells).is_empty()
            {
                let Some(spell) = select(weapon, &spells, input, output)? else {
                    return Ok(None);
                };
                let Some(enemy) = select(enemy, &enemy_choices, input, output)? else {
                    return Ok(None);
                };
                return Ok(Some(format!("cast {spell} on {enemy}")));
            }
            let Some(weapon) = select(weapon, &weapons, input, output)? else {
                return Ok(None);
            };
            let Some(enemy) = select(enemy, &enemy_choices, input, output)? else {
                return Ok(None);
            };
            return Ok(Some(format!("attack {enemy} with {weapon}")));
        }
        if !argument.is_empty() {
            let Some(enemy) = select(argument, &enemy_choices, input, output)? else {
                return Ok(None);
            };
            if weapons.is_empty() {
                writeln!(
                    output,
                    "You have no carried weapons. Try 'weapons' or 'cast <spell> on <enemy>'.\n"
                )?;
                return Ok(None);
            }
            let Some(weapon) = choose_matches(
                "weapon",
                &weapons,
                weapons.iter().collect(),
                true,
                input,
                output,
            )?
            else {
                return Ok(None);
            };
            return Ok(Some(format!("attack {enemy} with {weapon}")));
        }
    }
    if verb == "cast" && !argument.is_empty() {
        let (spell, target) = argument
            .split_once(" on ")
            .map_or((argument, None), |(s, t)| (s, Some(t)));
        // Exact unlearned IDs/names retain the established "not learned" diagnostic;
        // hidden/unlearned definitions are never offered as typo suggestions.
        let spell = if let Some(a) = content
            .abilities
            .values()
            .find(|a| a.id == clean_target(spell))
        {
            a.id.clone()
        } else {
            let Some(s) = select(spell, &spells, input, output)? else {
                return Ok(None);
            };
            s
        };
        let target = match target.map(clean_target) {
            Some(t) if t == "self" || t == "me" => format!(" on {t}"),
            Some(t) => {
                let Some(t) = select(&t, &enemy_choices, input, output)? else {
                    return Ok(None);
                };
                format!(" on {t}")
            }
            None => String::new(),
        };
        return Ok(Some(format!("cast {spell}{target}")));
    }
    if verb == "learn" && !argument.is_empty() {
        return Ok(select(argument, &carried, input, output)?.map(|id| format!("learn {id}")));
    }
    if verb == "use" && !argument.is_empty() && !argument.contains(" on ") {
        // Existing transaction-object behavior retains priority over ability aliases.
        return Ok(select(argument, &visible, input, output)?.map(|id| format!("use {id}")));
    }
    match parse_command(&normalized) {
        Ok(Command::Use { item, target }) => {
            let item_matches = resolution::matches(&item, &visible);
            let ordinary_target = !resolution::matches(&target, &visible).is_empty();
            if item_matches.is_empty()
                && !ordinary_target
                && !resolution::matches(&item, &spells).is_empty()
            {
                let Some(spell) = select(&item, &spells, input, output)? else {
                    return Ok(None);
                };
                let target = if target == "self" || target == "me" {
                    target
                } else {
                    let Some(t) = select(&target, &enemy_choices, input, output)? else {
                        return Ok(None);
                    };
                    t
                };
                return Ok(Some(format!("cast {spell} on {target}")));
            }
            let Some(item) = select(&item, &visible, input, output)? else {
                return Ok(None);
            };
            if !ordinary_target
                && content.weapons.contains_key(&item)
                && !resolution::matches(&target, &enemy_choices).is_empty()
            {
                let Some(enemy) = select(&target, &enemy_choices, input, output)? else {
                    return Ok(None);
                };
                return Ok(Some(format!("attack {enemy} with {item}")));
            }
            let Some(target) = select(&target, &visible, input, output)? else {
                return Ok(None);
            };
            Ok(Some(format!("use {item} on {target}")))
        }
        Ok(Command::Take(target)) | Ok(Command::Drop(target)) | Ok(Command::Examine(target)) => {
            let action = match parse_command(&normalized).unwrap() {
                Command::Take(_) => "take",
                Command::Drop(_) => "drop",
                _ => "examine",
            };
            if target == "all" && action != "examine" {
                return Ok(Some(format!("{action} all")));
            }
            let eligible = if action == "drop" {
                carried
            } else if action == "take" {
                visible
                    .iter()
                    .filter(|i| state.available_source(&i.id, false).is_some())
                    .cloned()
                    .collect()
            } else {
                visible
                    .iter()
                    .cloned()
                    .chain(enemy_choices.iter().cloned())
                    .collect()
            };
            // If there is no role-relevant match, preserve useful already-carried / fixed diagnostics.
            let choices = if resolution::matches(&target, &eligible).is_empty() {
                &visible
            } else {
                &eligible
            };
            Ok(select(&target, choices, input, output)?.map(|id| format!("{action} {id}")))
        }
        // Reserved save arguments preserve original spelling and spacing.
        _ if matches!(verb, "save" | "load") => Ok(Some(line.trim().to_owned())),
        _ => {
            if matches!(verb, "yes" | "no") && argument.is_empty() {
                writeln!(output, "There is no question waiting for yes/no. To use a suggestion, type the suggested command.\n")?;
                return Ok(None);
            }
            if matches!(
                parse_command(&normalized),
                Err(crate::parser::ParseError::Unknown(_))
            ) && matches!(verb, "go" | "move" | "walk" | "travel" | "run" | "head")
                && !argument.is_empty()
            {
                writeln!(output, "Move using an exit direction, such as 'go north'. 'look' lists this room's exits. For an object here, try 'examine <object>' or 'use <object>'.\n")?;
                return Ok(None);
            }
            // Bare names are guidance, never an inferred use/attack/cast action.
            // Let all existing verbs reach their established runner handlers.
            if matches!(
                parse_command(&normalized),
                Err(crate::parser::ParseError::Unknown(_))
            ) && !matches!(
                verb,
                "attack"
                    | "cast"
                    | "learn"
                    | "use"
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
                    | "saves"
            ) {
                if let [item] = resolution::matches(&normalized, &visible).as_slice() {
                    writeln!(output, "Try 'examine {}' to inspect this object, or 'use {}' to interact with it.\n", item.id, item.id)?;
                    return Ok(None);
                }
                for spell in &spells {
                    for label in [&spell.id, &spell.name] {
                        let label = resolution::normalize(label);
                        if normalized == label || normalized.starts_with(&format!("{label} ")) {
                            let target = normalized[label.len()..].trim();
                            let target = if target.is_empty() {
                                "<target>"
                            } else {
                                target
                            };
                            writeln!(output, "To cast this spell, type 'cast {} on {target}'. Use 'spellbook' to check its targets. No spell was cast.\n", spell.id)?;
                            return Ok(None);
                        }
                    }
                }
            }
            Ok(Some(normalized))
        }
    }
}
