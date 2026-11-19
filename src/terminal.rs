//! CLI-only terminal editing adapter. Scriptable library APIs still accept BufRead.
use rustyline::{error::ReadlineError, DefaultEditor};
use std::io::{self, BufRead, IsTerminal, Read};

pub fn supports_editing(stdin: bool, stdout: bool, term: Option<&str>) -> bool {
    stdin && stdout && !term.is_some_and(|t| t.eq_ignore_ascii_case("dumb"))
}

/// Memory-only history shared by prompts during this CLI invocation. No file API
/// is enabled. Redirected streams use the original buffered input unchanged.
pub struct TerminalInput<R> {
    plain: R,
    editor: Option<DefaultEditor>,
    buffer: Vec<u8>,
    offset: usize,
}
impl<R: BufRead> TerminalInput<R> {
    pub fn new(plain: R, interactive: bool) -> io::Result<Self> {
        let editor = if interactive {
            let config = rustyline::Config::builder()
                .max_history_size(100)
                .map_err(io::Error::other)?
                .history_ignore_dups(true)
                .map_err(io::Error::other)?
                .build();
            Some(DefaultEditor::with_config(config).map_err(io::Error::other)?)
        } else {
            None
        };
        Ok(Self {
            plain,
            editor,
            buffer: Vec::new(),
            offset: 0,
        })
    }
}

pub fn stdin() -> io::Result<TerminalInput<io::StdinLock<'static>>> {
    let enabled = supports_editing(
        io::stdin().is_terminal(),
        io::stdout().is_terminal(),
        std::env::var("TERM").ok().as_deref(),
    );
    TerminalInput::new(io::stdin().lock(), enabled)
}

impl<R: BufRead> BufRead for TerminalInput<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let Some(editor) = &mut self.editor else {
            return self.plain.fill_buf();
        };
        if self.offset == self.buffer.len() {
            self.buffer.clear();
            self.offset = 0;
            loop {
                match editor.readline("") {
                    Ok(line) => {
                        if !line.trim().is_empty() {
                            editor
                                .add_history_entry(line.trim())
                                .map_err(io::Error::other)?;
                        }
                        self.buffer.extend_from_slice(line.as_bytes());
                        self.buffer.push(b'\n');
                        break;
                    }
                    Err(ReadlineError::Interrupted) => continue, // cancel this line, never submit it
                    Err(ReadlineError::Eof) => break,
                    Err(error) => return Err(io::Error::other(error)),
                }
            }
        }
        Ok(&self.buffer[self.offset..])
    }
    fn consume(&mut self, amount: usize) {
        if self.editor.is_some() {
            self.offset = (self.offset + amount).min(self.buffer.len());
        } else {
            self.plain.consume(amount);
        }
    }
}
impl<R: BufRead> Read for TerminalInput<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let available = self.fill_buf()?;
        let count = buffer.len().min(available.len());
        buffer[..count].copy_from_slice(&available[..count]);
        self.consume(count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redirected_and_dumb_terminals_keep_plain_input() {
        for (input, output, term) in [
            (false, true, None),
            (true, false, None),
            (true, true, Some("dumb")),
        ] {
            assert!(!supports_editing(input, output, term));
        }
        assert!(supports_editing(true, true, None));
        let script = "look\n\n\u{1b}[A\nquit\n";
        let mut reader = TerminalInput::new(io::Cursor::new(script), false).unwrap();
        let mut result = String::new();
        reader.read_to_string(&mut result).unwrap();
        assert_eq!(result, script);
        assert_eq!(reader.read_line(&mut result).unwrap(), 0);
    }
    #[test]
    fn history_is_bounded_memory_only_and_ignores_duplicates() {
        use rustyline::history::{History, MemHistory};
        let config = rustyline::Config::builder()
            .max_history_size(2)
            .unwrap()
            .build();
        let mut history = MemHistory::with_config(&config);
        for line in ["look", "look", "", "north", "inventory"] {
            history.add(line).unwrap();
        }
        assert_eq!(history.len(), 2);
        assert_eq!(
            history
                .get(0, rustyline::history::SearchDirection::Forward)
                .unwrap()
                .unwrap()
                .entry,
            "north"
        );
        assert_eq!(
            history
                .get(1, rustyline::history::SearchDirection::Forward)
                .unwrap()
                .unwrap()
                .entry,
            "inventory"
        );
    }
}
