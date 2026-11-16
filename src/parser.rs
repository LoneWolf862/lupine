use crate::commands::{Command, Direction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    MissingDirection,
    MissingTarget(&'static str),
    Unknown(String),
}

fn clean_target(text: &str) -> String {
    text.split_whitespace()
        .filter(|word| !matches!(*word, "the" | "a" | "an"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn required_target(text: &str, verb: &'static str) -> Result<String, ParseError> {
    let target = clean_target(text);
    if target.is_empty() {
        Err(ParseError::MissingTarget(verb))
    } else {
        Ok(target)
    }
}

pub fn parse_command(input: &str) -> Result<Command, ParseError> {
    let normalized = input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if normalized.is_empty() {
        return Err(ParseError::Empty);
    }

    match normalized.as_str() {
        "l" | "look" => return Ok(Command::Look),
        "i" | "inv" | "inventory" => return Ok(Command::Inventory),
        "h" | "help" | "?" => return Ok(Command::Help),
        "q" | "quit" | "exit" => return Ok(Command::Quit),
        "go" | "move" | "walk" => return Err(ParseError::MissingDirection),
        "x" | "examine" | "inspect" | "look at" => {
            return Err(ParseError::MissingTarget("examine"))
        }
        "take" | "get" | "grab" | "pick up" => return Err(ParseError::MissingTarget("take")),
        "drop" | "leave" => return Err(ParseError::MissingTarget("drop")),
        "use" => return Err(ParseError::MissingTarget("use")),
        _ => {}
    }

    let direction_text = normalized
        .strip_prefix("go ")
        .or_else(|| normalized.strip_prefix("move "))
        .or_else(|| normalized.strip_prefix("walk "))
        .unwrap_or(&normalized);
    let direction = match direction_text {
        "n" | "north" => Some(Direction::North),
        "s" | "south" => Some(Direction::South),
        "e" | "east" => Some(Direction::East),
        "w" | "west" => Some(Direction::West),
        "u" | "up" => Some(Direction::Up),
        "d" | "down" => Some(Direction::Down),
        _ => None,
    };
    if let Some(direction) = direction {
        return Ok(Command::Move(direction));
    }

    for prefix in ["look at ", "examine ", "inspect ", "x "] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            return Ok(Command::Examine(required_target(rest, "examine")?));
        }
    }
    for prefix in ["pick up ", "take ", "get ", "grab "] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            return Ok(Command::Take(required_target(rest, "take")?));
        }
    }
    for prefix in ["drop ", "leave "] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            return Ok(Command::Drop(required_target(rest, "drop")?));
        }
    }
    if let Some(rest) = normalized.strip_prefix("use ") {
        let rest = rest.trim();
        if let Some((item, target)) = rest.split_once(" on ") {
            return Ok(Command::Use {
                item: required_target(item, "use")?,
                target: required_target(target, "use")?,
            });
        }
        return Err(ParseError::MissingTarget("use <item> on <target>"));
    }

    Err(ParseError::Unknown(normalized))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_look_aliases_and_normalization() {
        assert_eq!(parse_command("  LOOK  "), Ok(Command::Look));
        assert_eq!(parse_command("l"), Ok(Command::Look));
    }

    #[test]
    fn accepts_directions_and_phrases() {
        assert_eq!(parse_command("N"), Ok(Command::Move(Direction::North)));
        assert_eq!(
            parse_command(" go   north "),
            Ok(Command::Move(Direction::North))
        );
        assert_eq!(
            parse_command("walk west"),
            Ok(Command::Move(Direction::West))
        );
    }

    #[test]
    fn rejects_unknown_and_incomplete_commands() {
        assert_eq!(parse_command("   "), Err(ParseError::Empty));
        assert_eq!(parse_command("go"), Err(ParseError::MissingDirection));
        assert_eq!(parse_command("fly"), Err(ParseError::Unknown("fly".into())));
        assert_eq!(
            parse_command("take"),
            Err(ParseError::MissingTarget("take"))
        );
    }

    #[test]
    fn parses_objects_and_synonyms() {
        assert_eq!(
            parse_command(" PICK   UP the  BRASS KEY "),
            Ok(Command::Take("brass key".into()))
        );
        assert_eq!(parse_command("grab a key"), Ok(Command::Take("key".into())));
        assert_eq!(
            parse_command("look at the stone table"),
            Ok(Command::Examine("stone table".into()))
        );
        assert_eq!(parse_command("inv"), Ok(Command::Inventory));
        assert_eq!(parse_command("leave key"), Ok(Command::Drop("key".into())));
        assert_eq!(
            parse_command("use the key on the panel"),
            Ok(Command::Use {
                item: "key".into(),
                target: "panel".into()
            })
        );
    }
}
