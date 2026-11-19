//! Bounded, read-only project inspection. Never follow symlinks or Windows junctions.
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};
const DIRS: &[&str] = &[
    "config", "rooms", "items", "spells", "weapons", "NPCs", "enemies", "shops", "art",
];
const FILES: &[&str] = &[
    "events.toml",
    "dialogue.toml", // Obsolete content: retained only for explicit repair/deletion.
    "systems.toml",
    "actions.toml",
    "rules.toml",
    "game.toml",
    "interactions.toml",
    "transaction_interactions.toml",
    "character.toml",
    "README.txt",
];
const MAX_ENTRIES: usize = 2048;
const MAX_DEPTH: usize = 8;
const MAX_LINES: usize = 160;

pub(super) fn root(path: &Path) -> Result<PathBuf, String> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("Choose a project path without parent traversal (..).".into());
    }
    let absolute = std::path::absolute(path).map_err(|e| e.to_string())?;
    let mut part = PathBuf::new();
    for component in absolute.components() {
        part.push(component);
        let metadata =
            fs::symlink_metadata(&part).map_err(|e| format!("{}: {e}", part.display()))?;
        if super::files::is_link(&metadata) {
            return Err(format!(
                "{}: linked/junction paths are not opened by the creator",
                part.display()
            ));
        }
    }
    if !absolute.is_dir() {
        return Err("Select an adventure folder, not a file.".into());
    }
    let manifest = absolute.join("game.toml");
    let meta = fs::symlink_metadata(&manifest).map_err(|e| {
        format!(
            "{}: {e}. Choose the folder containing game.toml.",
            manifest.display()
        )
    })?;
    if super::files::is_link(&meta) || !meta.is_file() {
        return Err("game.toml must be a regular file inside the adventure.".into());
    }
    Ok(absolute)
}

pub(super) struct Inspection {
    pub tree: String,
    pub problems: Vec<String>,
}
pub(super) fn inspect(root: &Path) -> Inspection {
    let mut scan = Scan {
        root,
        lines: Vec::new(),
        problems: Vec::new(),
        visited: 0,
        limited: false,
    };
    for name in FILES.iter().chain(DIRS.iter()) {
        let path = root.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => scan.visit(&path, 0, DIRS.contains(name)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => scan.problems.push(format!("{}: {e}", path.display())),
        }
    }
    let mut tree = String::from("Project files (paths relative to the adventure folder):\n./\n");
    for (index, (depth, label)) in scan.lines.iter().take(MAX_LINES).enumerate() {
        for ancestor in 0..*depth {
            let continues = scan.lines[index + 1..]
                .iter()
                .find(|(level, _)| *level <= ancestor)
                .is_some_and(|(level, _)| *level == ancestor);
            tree.push_str(if continues { "|   " } else { "    " });
        }
        tree.push_str("+-- ");
        tree.push_str(label);
        tree.push('\n');
    }
    if scan.lines.len() > MAX_LINES {
        tree.push_str("... display limited to 160 entries; other files remain untouched.\n");
    }
    Inspection {
        tree,
        problems: scan.problems,
    }
}
struct Scan<'a> {
    root: &'a Path,
    lines: Vec<(usize, String)>,
    problems: Vec<String>,
    visited: usize,
    limited: bool,
}
impl Scan<'_> {
    fn limit(&mut self, path: &Path) {
        if !self.limited {
            self.problems.push(format!("{}: project inspection limit reached (2048 entries / 8 directory levels). Reduce nested content before creator validation/play.", path.display()));
            self.lines.push((
                0,
                "... inspection limit reached; some entries omitted".into(),
            ));
            self.limited = true;
        }
    }
    fn visit(&mut self, path: &Path, depth: usize, recognized_dir: bool) {
        if self.visited >= MAX_ENTRIES || depth > MAX_DEPTH {
            self.limit(path);
            return;
        }
        self.visited += 1;
        let relative = path
            .strip_prefix(self.root)
            .expect("paths constructed beneath root");
        let label = path
            .file_name()
            .expect("content has a filename")
            .to_string_lossy()
            .chars()
            .flat_map(|c| {
                if c.is_control() {
                    c.escape_default().collect::<Vec<_>>()
                } else {
                    vec![c]
                }
            })
            .collect::<String>();
        let label = if label.chars().count() > 180 {
            format!(
                "{}... [long path abbreviated]",
                label.chars().take(160).collect::<String>()
            )
        } else {
            label
        };
        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) => {
                self.problems.push(format!("{}: {e}", path.display()));
                return;
            }
        };
        if super::files::is_link(&metadata) {
            self.lines
                .push((depth, format!("{label} [link/junction skipped]")));
            self.problems.push(format!("{label}: links/junctions are not followed. Replace with a regular file/folder inside the project."));
            return;
        }
        if metadata.is_dir() {
            self.lines.push((depth, format!("{label}/")));
            if !recognized_dir {
                self.problems.push(format!("{label}: expected a file"));
                return;
            }
            let entries = match fs::read_dir(path) {
                Ok(e) => e,
                Err(e) => {
                    self.problems.push(format!("{label}: {e}"));
                    return;
                }
            };
            let mut paths = Vec::new();
            for entry in entries {
                if paths.len() + self.visited >= MAX_ENTRIES {
                    self.limit(path);
                    break;
                }
                match entry {
                    Ok(e) => paths.push(e.path()),
                    Err(e) => self.problems.push(format!("{label}: {e}")),
                }
            }
            paths.sort();
            for child in paths {
                // Inspect all entries for links, but display only recognized extensions and directories.
                self.visit(&child, depth + 1, true);
            }
        } else if metadata.is_file() {
            if depth == 0 && DIRS.iter().any(|d| relative == Path::new(d)) {
                self.problems.push(format!("{label}: expected a directory"));
            }
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if depth == 0 || matches!(extension, "toml" | "fig" | "txt") {
                self.lines.push((depth, label));
            }
        } else {
            self.problems
                .push(format!("{label}: not a regular file or directory"));
        }
    }
}
