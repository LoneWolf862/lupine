//! Offline startup navigation; each play reloads and validates content.
use crate::adventure::Adventure;
use std::{
    fs,
    io::{self, BufRead, Read, Write},
    path::{Path, PathBuf},
};

/// Search only conventional local locations, never recursively scan a drive.
pub fn search_roots() -> io::Result<Vec<PathBuf>> {
    let mut roots = vec![std::env::current_dir()?];
    if let Some(parent) = std::env::current_exe()?.parent() {
        roots.push(parent.to_owned());
    }
    Ok(roots)
}

#[derive(Debug)]
pub struct Choice {
    pub path: PathBuf,
    pub title: String,
}

/// Invalid candidates stay visible so selecting them can explain validation errors.
pub fn discover(roots: &[PathBuf]) -> Vec<Choice> {
    let mut paths = std::collections::BTreeSet::new();
    for root in roots {
        for directory in [root.clone(), root.join("adventures"), root.join("samples")] {
            if directory.join("game.toml").is_file() {
                paths.insert(directory.clone());
            }
            if let Ok(entries) = fs::read_dir(&directory) {
                for entry in entries.take(2048).flatten() {
                    if entry.file_type().is_ok_and(|t| t.is_dir())
                        && entry.path().join("game.toml").is_file()
                    {
                        paths.insert(entry.path());
                    }
                }
            }
        }
    }
    // Packaged adventures form the player menu; fixtures remain available through Browse.
    let preferred = paths
        .iter()
        .filter(|p| {
            roots
                .iter()
                .any(|r| p.parent() == Some(r.join("adventures").as_path()))
        })
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    if !preferred.is_empty() {
        paths.retain(|p| {
            roots.contains(p)
                || !roots
                    .iter()
                    .any(|r| p.starts_with(r.join("samples")) || *p == r.join("story_example"))
        });
    }
    let mut unique = std::collections::BTreeSet::new();
    paths
        .into_iter()
        .filter_map(|path| {
            let path = path.canonicalize().ok()?;
            if !unique.insert(path.clone()) {
                return None;
            }
            let title = fs::File::open(path.join("game.toml"))
                .ok()
                .and_then(|file| {
                    let mut text = String::new();
                    file.take(65536)
                        .read_to_string(&mut text)
                        .ok()
                        .map(|_| text)
                })
                .and_then(|s| s.parse::<toml::Table>().ok())
                .and_then(|t| t.get("title").and_then(|v| v.as_str()).map(str::to_owned))
                .unwrap_or_else(|| {
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                });
            Some(Choice {
                path,
                title: title
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(160)
                    .collect(),
            })
        })
        .collect()
}

fn answer(input: &mut impl BufRead, output: &mut impl Write) -> io::Result<Option<String>> {
    write!(output, "Selection:\n> ")?;
    output.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_owned()))
}

pub fn run(
    input: &mut impl BufRead,
    output: &mut impl Write,
    saves: &Path,
    roots: &[PathBuf],
) -> io::Result<()> {
    loop {
        writeln!(output, "\n========================================\n                LUPINE\n         Interactive Fiction\n========================================\n\nWelcome to Lupine!\n\n  1. Play an Adventure\n  2. Create or Edit an Adventure\n  3. Help\n  4. Exit\n")?;
        let Some(choice) = answer(input, output)? else {
            return Ok(());
        };
        match choice.to_ascii_lowercase().as_str() {
            "1" => play_menu(input, output, saves, roots)?,
            "2" => crate::creator::session::start(input, output, saves)?,
            "3" | "?" | "help" => writeln!(output, "Play explores an existing adventure. Create or Edit opens the Adventure Creator.\nChoose a number; B in Play browses another folder. 0/back cancels selection.\nIn play: look, north, take <item>, inventory, help. quit returns here.\nProgress is saved only with save <name>; load <name> explicitly restores it.\nAdvanced commands: lupine play PATH, validate PATH, creator [PATH], new.\nDevelopers: lupine fixture opens the engine test adventure.")?,
            "4" | "0" | "quit" | "exit" => return Ok(()),
            _ => writeln!(output, "Please choose 1, 2, 3, or 4.")?,
        }
    }
}

fn play_menu(
    input: &mut impl BufRead,
    output: &mut impl Write,
    saves: &Path,
    roots: &[PathBuf],
) -> io::Result<()> {
    let mut roots = roots.to_vec();
    loop {
        let choices = discover(&roots);
        writeln!(
            output,
            "\nPLAY AN ADVENTURE\nChoose its number, or B to browse another folder.\n  0. Back"
        )?;
        for (index, choice) in choices.iter().enumerate() {
            writeln!(
                output,
                "  {}. {}\n     {}",
                index + 1,
                choice.title,
                display_path(&choice.path)
            )?;
        }
        if choices.is_empty() {
            writeln!(
                output,
                "No adventures found here. Use B to enter a folder path."
            )?;
        }
        writeln!(
            output,
            "  B. Browse to another adventure or developer fixtures in samples/"
        )?;
        let Some(choice) = answer(input, output)? else {
            return Ok(());
        };
        if matches!(
            choice.to_ascii_lowercase().as_str(),
            "0" | "back" | "cancel"
        ) {
            return Ok(());
        }
        let path = if choice.eq_ignore_ascii_case("b") {
            writeln!(
                output,
                "Enter a folder path (spaces are allowed; no surrounding quotes), or back:"
            )?;
            let Some(path) = answer(input, output)? else {
                return Ok(());
            };
            if matches!(
                path.to_ascii_lowercase().as_str(),
                "0" | "back" | "cancel" | ""
            ) {
                continue;
            }
            let path = PathBuf::from(path);
            if !path.is_dir() {
                writeln!(output, "Cannot open folder: {}", path.display())?;
                continue;
            }
            if !path.join("game.toml").exists() {
                roots = vec![path];
                continue;
            }
            path
        } else if let Some(entry) = choice
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|i| choices.get(i))
        {
            entry.path.clone()
        } else {
            writeln!(output, "Choose a listed number, B, or 0.")?;
            continue;
        };
        match Adventure::load(&path) {
            Ok(adventure) => {
                writeln!(output, "Starting fresh. Use load explicitly to restore a save; quit returns to startup.")?;
                adventure.play(saves, input, output)?;
                return Ok(());
            }
            Err(message) => writeln!(
                output,
                "Cannot play {}:\n{message}\nCorrect the adventure files or choose another folder.",
                path.display()
            )?,
        }
    }
}

fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned()
    }
}
