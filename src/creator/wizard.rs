use serde::Serialize;
use std::io::{BufRead, Write};
use std::path::Path;

pub(super) struct DraftFile {
    pub name: String,
    pub contents: String,
    explanation: String,
}
impl DraftFile {
    // FIG uses the same TOML serializer; callers choose the destination extension.
    pub fn toml(name: &str, value: &impl Serialize, explanation: &str) -> Result<Self, String> {
        Ok(Self::text(
            name,
            &toml::to_string_pretty(value).map_err(|e| e.to_string())?,
            explanation,
        ))
    }
    pub fn text(name: &str, contents: &str, explanation: &str) -> Self {
        Self {
            name: name.into(),
            contents: contents.into(),
            explanation: explanation.into(),
        }
    }
    pub fn preview(&self, root: &Path, output: &mut impl Write) -> Result<(), String> {
        writeln!(
            output,
            "\nPreview: {}\n\n{}\n\n--- exact contents ---",
            root.join(&self.name).display(),
            self.explanation
        )
        .and_then(|()| output.write_all(self.contents.as_bytes()))
        .and_then(|()| writeln!(output, "--- end contents ---\n"))
        .map_err(|e| e.to_string())
    }
}

pub(super) struct Wizard<'a, R, W> {
    pub input: &'a mut R,
    pub output: &'a mut W,
}
impl<R: BufRead, W: Write> Wizard<'_, R, W> {
    pub fn say(&mut self, message: &str) -> Result<(), String> {
        writeln!(self.output, "{message}").map_err(|e| e.to_string())
    }
    pub fn heading(&mut self, title: &str) -> Result<(), String> {
        super::ui::heading(self.output, title).map_err(|e| e.to_string())
    }
    pub fn line(&mut self, prompt: &str) -> Result<String, String> {
        if prompt == "... " {
            write!(self.output, "{prompt}").and_then(|()| self.output.flush())
        } else {
            super::ui::prompt(self.output, prompt)
        }
        .map_err(|e| e.to_string())?;
        let mut line = String::new();
        if self.input.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            return Err("Setup canceled (end of input); unsaved changes discarded; earlier confirmed saves remain".into());
        }
        let line = line.trim_end_matches(['\n', '\r']).to_owned();
        if line.trim().eq_ignore_ascii_case("cancel") {
            return Err(
                "Setup canceled; unsaved changes discarded; earlier confirmed saves remain".into(),
            );
        }
        Ok(line)
    }
    /// Menu controls do not turn blank input or help into an action.
    pub fn menu(&mut self, prompt: &str) -> Result<String, String> {
        loop {
            let answer = self.line(prompt)?;
            let trimmed = answer.trim();
            match trimmed.to_ascii_lowercase().as_str() {
                "" => self.say("Choose one of the displayed options. Enter has no default here; your draft is unchanged.")?,
                "?" | "help" => self.say("Use a displayed menu option. Definition lists also accept exact IDs and unique names, /text to filter, / to clear, >/< to page. 0/back follows the displayed return/finish/discard action; cancel abandons this operation's unsaved edits. Save always requires confirmation.")?,
                "back" => return Ok("back".into()),
                _ => return Ok(trimmed.to_owned()),
            }
        }
    }
    pub fn ask(
        &mut self,
        label: &str,
        default: &str,
        help: &str,
        validate: impl Fn(&str) -> Result<(), String>,
    ) -> Result<Option<String>, String> {
        loop {
            let line = self.line(&format!("{label} [{default}]: "))?;
            let value = line.trim();
            if value.eq_ignore_ascii_case("back") {
                return Ok(None);
            }
            if value == "?" || value.eq_ignore_ascii_case("help") {
                self.say(help)?;
                continue;
            }
            let value = if value.is_empty() {
                default
            } else {
                if value.starts_with('\\')
                    && matches!(
                        &value[1..].to_ascii_lowercase()[..],
                        "cancel" | "back" | "?" | "help"
                    )
                {
                    &value[1..]
                } else {
                    value
                }
            };
            match validate(value) {
                Ok(()) => return Ok(Some(value.to_owned())),
                Err(error) => self.say(&error)?,
            }
        }
    }
    pub fn description(&mut self, default: &str) -> Result<Option<String>, String> {
        self.description_labeled("Location description", default)
    }
    pub fn description_labeled(
        &mut self,
        label: &str,
        default: &str,
    ) -> Result<Option<String>, String> {
        self.say(
            "Write one line below, or type /multi to write several lines. Instructions will follow.",
        )?;
        let mut current = default.to_owned();
        loop {
            let first = self.line(&format!("{label} [{current}]: "))?;
            if first.eq_ignore_ascii_case("back") {
                return Ok(None);
            }
            if first == "?" || first.eq_ignore_ascii_case("help") {
                self.say("Plain story text, not TOML. Quotes and newlines are encoded for you. In /multi, prefix literal /end or cancel with a backslash.")?;
                continue;
            }
            let value = if first == "/multi" {
                self.say("Write as many lines as you like, including blank lines. Type /end on a line by itself when finished. back discards this edit; cancel abandons the current operation. Prefix literal controls with a backslash.")?;
                let mut lines = Vec::new();
                loop {
                    let line = self.line("... ")?;
                    if line.eq_ignore_ascii_case("back") {
                        return Ok(None);
                    }
                    if line == "?" || line.eq_ignore_ascii_case("help") {
                        self.say("Type /end to finish, back to discard this edit, or cancel to abandon the current operation. Use \\? for a literal question mark.")?;
                        continue;
                    }
                    if line == "/end" {
                        break;
                    }
                    lines.push(line.strip_prefix('\\').unwrap_or(&line).to_owned());
                }
                lines.join("\n")
            } else if first.is_empty() {
                current.clone()
            } else {
                first.strip_prefix('\\').unwrap_or(&first).to_owned()
            };
            match super::validate_text(&value, 65536) {
                Ok(()) => return Ok(Some(value)),
                Err(e) => {
                    self.say(&format!("{e}. Your text is retained below; enter a corrected description or back to leave this field unchanged."))?;
                    current = value;
                }
            }
        }
    }
}
