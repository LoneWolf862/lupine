//! Plain ASCII presentation shared by creator screens; no terminal state or colors.
use std::io::{self, Write};

pub(super) fn heading(output: &mut impl Write, title: &str) -> io::Result<()> {
    writeln!(output, "\n========================================\n{title:^40}\n========================================\n")
}

pub(super) fn prompt(output: &mut impl Write, label: &str) -> io::Result<()> {
    write!(output, "\n{}\n> ", label.trim())?;
    output.flush()
}
