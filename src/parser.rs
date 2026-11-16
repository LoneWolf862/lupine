use crate::commands::{Command, Direction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    MissingDirection,
    Unknown(String),
}

pub fn parse_command(input: &str) -> Result<Command, ParseError> {
    let normalized = input.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    if normalized.is_empty() {
        return Err(ParseError::Empty);
    }

    match normalized.as_str() {
        "l" | "look" => return Ok(Command::Look),
        "h" | "help" | "?" => return Ok(Command::Help),
        "q" | "quit" | "exit" => return Ok(Command::Quit),
        "go" | "move" | "walk" => return Err(ParseError::MissingDirection),
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
    direction.map(Command::Move).ok_or(ParseError::Unknown(normalized))
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
        assert_eq!(parse_command(" go   north "), Ok(Command::Move(Direction::North)));
        assert_eq!(parse_command("walk west"), Ok(Command::Move(Direction::West)));
    }

    #[test]
    fn rejects_unknown_and_incomplete_commands() {
        assert_eq!(parse_command("   "), Err(ParseError::Empty));
        assert_eq!(parse_command("go"), Err(ParseError::MissingDirection));
        assert_eq!(parse_command("fly"), Err(ParseError::Unknown("fly".into())));
    }
}
