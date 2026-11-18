//! Shared recoverable content transactions. No gameplay state or player saves are touched.
use super::{files, tree};
use crate::adventure::Adventure;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};
const LOCK: &str = ".lupine-room-edit";
const DIRS: &[&str] = &[
    "rooms", "items", "config", "art", "spells", "weapons", "enemies", "shops",
];
const FILES: &[&str] = &[
    "game.toml",
    "interactions.toml",
    "transaction_interactions.toml",
    "character.toml",
];

#[derive(Clone, PartialEq)]
struct Original {
    bytes: Vec<u8>,
    modified: Option<SystemTime>,
    permissions: fs::Permissions,
}
impl Original {
    fn read(path: &Path) -> Result<Self, String> {
        let m = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if files::is_link(&m) || !m.is_file() {
            return Err(format!(
                "{}: expected a regular file, not a link",
                path.display()
            ));
        }
        if m.len() > 8 * 1024 * 1024 {
            return Err(format!(
                "{} exceeds the editor's 8 MiB per-file limit",
                path.display()
            ));
        }
        Ok(Self {
            bytes: fs::read(path).map_err(|e| e.to_string())?,
            modified: m.modified().ok(),
            permissions: m.permissions(),
        })
    }
}

pub(super) struct Snapshot {
    pub root: PathBuf,
    files: BTreeMap<PathBuf, Original>,
    dirs: Vec<PathBuf>,
}
impl Snapshot {
    pub fn capture(root: &Path) -> Result<Self, String> {
        let root = tree::root(root)?;
        if fs::symlink_metadata(root.join(LOCK)).is_ok() {
            return Err(format!("Recovery folder {} exists. Do not edit further. See its RECOVERY.txt and restore/review the retained originals before removing that folder.", root.join(LOCK).display()));
        }
        let report = tree::inspect(&root);
        if !report.problems.is_empty() {
            return Err(report.problems.join("\n"));
        }
        let mut result = Self {
            root,
            files: BTreeMap::new(),
            dirs: Vec::new(),
        };
        for name in FILES {
            if result.root.join(name).exists() {
                result.files.insert(
                    PathBuf::from(name),
                    Original::read(&result.root.join(name))?,
                );
            }
        }
        for name in DIRS {
            if result.root.join(name).exists() {
                result.collect(Path::new(name))?;
            }
        }
        if result.files.values().map(|o| o.bytes.len()).sum::<usize>() > 64 * 1024 * 1024 {
            return Err("Project exceeds the editor's 64 MiB snapshot limit".into());
        }
        Ok(result)
    }
    fn collect(&mut self, relative: &Path) -> Result<(), String> {
        if relative.components().count() > 9 || self.dirs.len() + self.files.len() >= 2048 {
            return Err("Project changed or exceeds safe snapshot traversal limits".into());
        }
        self.dirs.push(relative.to_owned());
        for entry in fs::read_dir(self.root.join(relative)).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let relative = path.strip_prefix(&self.root).unwrap().to_owned();
            let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if files::is_link(&meta) {
                return Err(format!("{}: links are not editable", path.display()));
            }
            if meta.is_dir() {
                self.collect(&relative)?;
            } else {
                let original = Original::read(&path)?;
                if self.files.len() + self.dirs.len() >= 2048
                    || self.files.values().map(|o| o.bytes.len()).sum::<usize>()
                        + original.bytes.len()
                        > 64 * 1024 * 1024
                {
                    return Err("Project exceeds safe snapshot size/entry limits".into());
                }
                self.files.insert(relative, original);
            }
        }
        Ok(())
    }
    pub fn room_sources(&self) -> impl Iterator<Item = (&PathBuf, &[u8])> {
        self.files
            .iter()
            .filter(|(p, _)| {
                p.parent() == Some(Path::new("rooms")) && p.extension().is_some_and(|e| e == "toml")
            })
            .map(|(p, o)| (p, o.bytes.as_slice()))
    }
    pub fn sources<'a>(
        &'a self,
        category: &'a str,
    ) -> impl Iterator<Item = (&'a PathBuf, &'a [u8])> {
        self.files
            .iter()
            .filter(move |(p, _)| p.parent() == Some(Path::new(category)))
            .map(|(p, o)| (p, o.bytes.as_slice()))
    }
    pub fn art_sources(&self) -> impl Iterator<Item = (&PathBuf, &[u8])> {
        self.files
            .iter()
            .filter(|(p, _)| p.starts_with("art") && p.extension().is_some_and(|e| e == "txt"))
            .map(|(p, o)| (p, o.bytes.as_slice()))
    }
    pub fn source(&self, path: &Path) -> Option<&[u8]> {
        self.files.get(path).map(|o| o.bytes.as_slice())
    }
    fn check(&self, changes: &BTreeMap<PathBuf, String>) -> Result<(), String> {
        // Capture without the transaction directory interfering; it is not content.
        tree::root(&self.root)?;
        let report = tree::inspect(&self.root);
        if !report.problems.is_empty() {
            return Err(report.problems.join("\n"));
        }
        let mut current = Self {
            root: self.root.clone(),
            files: BTreeMap::new(),
            dirs: Vec::new(),
        };
        for name in FILES {
            if self.root.join(name).exists() {
                current
                    .files
                    .insert(PathBuf::from(name), Original::read(&self.root.join(name))?);
            }
        }
        for name in DIRS {
            if self.root.join(name).exists() {
                current.collect(Path::new(name))?;
            }
        }
        let mut old_dirs = self.dirs.clone();
        old_dirs.sort();
        let mut current_dirs = current.dirs.clone();
        current_dirs.sort();
        if current.files != self.files || old_dirs != current_dirs {
            return Err("Project files changed externally since opening this editor. Cancel and reopen; no external edits will be overwritten.".into());
        }
        for path in changes.keys() {
            let safe = path
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)));
            let category = path.parent().and_then(Path::to_str).unwrap_or("");
            let allowed = path == Path::new("interactions.toml")
                || match category {
                    "rooms" | "items" | "spells" => path.extension().is_some_and(|e| e == "toml"),
                    "config" => matches!(
                        path.file_name().and_then(|s| s.to_str()),
                        Some("character.fig" | "terminology.fig")
                    ),
                    "art" => path.extension().is_some_and(|e| e == "txt"),
                    _ => {
                        path.starts_with("art")
                            && self.files.contains_key(path)
                            && path.extension().is_some_and(|e| e == "txt")
                    }
                };
            if !safe || !allowed {
                return Err("This path is not an editable content file".into());
            }
            if let Some(original) = self.files.get(path) {
                if original.permissions.readonly() {
                    return Err(format!("{} is read-only", path.display()));
                }
            } else if fs::symlink_metadata(self.root.join(path)).is_ok() {
                return Err(format!("{} now exists; reopen the editor", path.display()));
            }
        }
        Ok(())
    }
    pub fn commit(&self, changes: &BTreeMap<PathBuf, String>) -> Result<(), String> {
        self.commit_with(changes, |_| Ok(()))
    }
    fn commit_with(
        &self,
        changes: &BTreeMap<PathBuf, String>,
        mut before_publish: impl FnMut(usize) -> Result<(), String>,
    ) -> Result<(), String> {
        self.check(changes)?;
        let lock = self.root.join(LOCK);
        fs::create_dir(&lock).map_err(|e| {
            format!(
                "Cannot reserve {}: {e}. Another edit or recovery may be pending.",
                lock.display()
            )
        })?;
        let mut owned = Owned {
            files: Vec::new(),
            dirs: vec![lock.clone()],
        };
        let mut moved = Vec::new();
        let mut published = Vec::new();
        let mut created_dirs = Vec::new();
        let result = (|| {
            owned.write(&lock.join("RECOVERY.txt"), b"Interrupted content edit: do not run another edit until reviewed.\noriginal/ contains files moved from the adventure, with original bytes/metadata.\nproposed/ contains the validated proposed project; it is not automatically published.\nFor rollback, restore each original/<relative path> to its matching project path; remove newly created content files only after comparing with proposed/.\nCHANGES.txt lists all target paths and whether each existed before. Keep unrelated/external changes.\nAfter recovery run lupine validate, then remove this recovery folder. Never remove recovery data before checking the adventure.\n")?;
            let listing = changes
                .keys()
                .map(|p| {
                    format!(
                        "{} {}\n",
                        if self.files.contains_key(p) {
                            "EXISTING"
                        } else {
                            "NEW"
                        },
                        p.display()
                    )
                })
                .collect::<String>();
            owned.write(&lock.join("CHANGES.txt"), listing.as_bytes())?;
            let staged = lock.join("proposed");
            owned.dir(&staged)?;
            owned.dir(&lock.join("original"))?;
            for category in ["rooms", "items", "spells", "art", "config"] {
                owned.dir(&lock.join("original").join(category))?;
            }
            for dir in &self.dirs {
                owned.dir(&staged.join(dir))?;
            }
            for path in changes.keys() {
                let parent = path.parent().unwrap();
                if !staged.join(parent).exists() {
                    owned.dir(&staged.join(parent))?;
                }
            }
            for (path, original) in &self.files {
                let bytes = changes
                    .get(path)
                    .map(|s| s.as_bytes())
                    .unwrap_or(&original.bytes);
                owned.write(&staged.join(path), bytes)?;
            }
            for path in changes.keys() {
                if let Some(original) = self.files.get(path) {
                    fs::set_permissions(staged.join(path), original.permissions.clone())
                        .map_err(|e| e.to_string())?;
                }
            }
            for path in changes.keys() {
                let mut backup = lock.join("original");
                for component in path.parent().unwrap().components() {
                    backup.push(component);
                    if !backup.exists() {
                        owned.dir(&backup)?;
                    }
                }
            }
            for (path, text) in changes {
                if !self.files.contains_key(path) {
                    owned.write(&staged.join(path), text.as_bytes())?;
                }
            }
            for (path, text) in changes {
                if path.starts_with("art") {
                    crate::story_loader::validate_art(text, &path.display().to_string())?;
                }
            }
            Adventure::load(&staged)
                .map_err(|e| format!("Proposed changes are invalid (nothing published): {e}"))?;
            self.check(changes)?;
            for (index, (relative, text)) in changes.iter().enumerate() {
                before_publish(index)?;
                let target = self.root.join(relative);
                let parent = target.parent().unwrap();
                if !parent.exists() {
                    fs::create_dir(parent).map_err(|e| e.to_string())?;
                    created_dirs.push(parent.to_owned());
                }
                if let Some(original) = self.files.get(relative) {
                    if Original::read(&target)? != *original {
                        return Err(format!("{} changed externally", target.display()));
                    }
                    let backup = lock.join("original").join(relative);
                    fs::rename(&target, &backup)
                        .map_err(|e| format!("Cannot retain original {}: {e}", target.display()))?;
                    moved.push(relative.clone());
                    if Original::read(&backup)? != *original {
                        return Err(format!(
                            "{} changed while publishing; original retained",
                            target.display()
                        ));
                    }
                }
                fs::hard_link(staged.join(relative), &target)
                    .map_err(|e| format!("Cannot publish {}: {e}", target.display()))?;
                published.push((relative.clone(), text.as_bytes()));
            }
            Adventure::load(&self.root)
                .map_err(|e| format!("Post-write validation failed: {e}"))?;
            owned.write(&lock.join("COMMITTED.txt"), b"All proposed content changes were published and the adventure passed validation. If this folder remains, cleanup was interrupted. Review the current adventure before removing recovery files; do not automatically roll back a completed edit.\n")?;
            Ok(())
        })();
        if let Err(error) = result {
            let rollback = (|| -> Result<(), String> {
                for (relative, bytes) in published.iter().rev() {
                    let target = self.root.join(relative);
                    if Original::read(&target)?.bytes != *bytes {
                        return Err(format!(
                            "{} was externally modified; leaving recovery files",
                            target.display()
                        ));
                    }
                    fs::remove_file(&target).map_err(|e| e.to_string())?;
                }
                for relative in moved.iter().rev() {
                    let backup = lock.join("original").join(relative);
                    fs::hard_link(&backup, self.root.join(relative)).map_err(|e| {
                        format!(
                            "Could not restore {} without overwriting: {e}",
                            relative.display()
                        )
                    })?;
                    fs::remove_file(&backup).map_err(|e| e.to_string())?;
                }
                for dir in created_dirs.iter().rev() {
                    fs::remove_dir(dir).map_err(|e| e.to_string())?;
                }
                Ok(())
            })();
            return match rollback {
                Ok(()) => match owned.clean() {
                    Ok(()) => Err(format!(
                        "{error}; original content restored; no changes saved."
                    )),
                    Err(e) => Err(format!("{error}; rollback completed, but {e}")),
                },
                Err(e) => Err(format!(
                    "{error}; rollback needs attention: {e}. Keep {} and follow RECOVERY.txt.",
                    lock.display()
                )),
            };
        }
        for relative in moved {
            owned.files.push(lock.join("original").join(relative));
        }
        // A cleanup failure is not a failed commit: retain recovery and report the actual outcome.
        owned.clean().map_err(|e| {
            format!("Changes were saved and validated, but cleanup needs attention: {e}")
        })
    }
}
struct Owned {
    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
}
impl Owned {
    fn dir(&mut self, path: &Path) -> Result<(), String> {
        fs::create_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
        self.dirs.push(path.to_owned());
        Ok(())
    }
    fn write(&mut self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        self.files.push(path.to_owned());
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| format!("{}: {e}", path.display()))
    }
    fn clean(&self) -> Result<(), String> {
        // Keep recovery instructions/status until all data/subdirectories are cleaned.
        // Unexpected files cause remove_dir to fail; never recursively delete them.
        let is_note = |path: &Path| {
            matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("RECOVERY.txt" | "CHANGES.txt" | "COMMITTED.txt")
            ) && path.parent() == self.dirs.first().map(PathBuf::as_path)
        };
        for path in self.files.iter().rev().filter(|p| !is_note(p)) {
            fs::remove_file(path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        for path in self.dirs.iter().skip(1).rev() {
            fs::remove_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        for path in self.files.iter().rev().filter(|p| is_note(p)) {
            fs::remove_file(path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        if let Some(root) = self.dirs.first() {
            fs::remove_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "lupine-room-tx-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            crate::creator::new_game(&path, "transaction", "Transaction").unwrap();
            Self(path)
        }
        fn changes(&self) -> BTreeMap<PathBuf, String> {
            BTreeMap::from([
                (
                    PathBuf::from("rooms/a_new.toml"),
                    "id='a_new'\ntitle='New'\ndescription='New'\n[exits]\nsouth='start'\n".into(),
                ),
                (
                    PathBuf::from("rooms/start.toml"),
                    "id='start'\ntitle='Changed'\ndescription='Start'\n[exits]\nnorth='a_new'\n"
                        .into(),
                ),
            ])
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn content_transaction_rejects_escape_stale_and_invalid_references() {
        let f = Fixture::new();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        for path in [
            "../items/key.toml",
            "items/../../outside.toml",
            "game.toml",
            "saves/slot.rork",
        ] {
            assert!(snapshot
                .commit(&BTreeMap::from([(PathBuf::from(path), String::new())]))
                .is_err());
        }
        let changes = BTreeMap::from([(
            PathBuf::from("items/key.toml"),
            "id='key'\nname='Key'\ndescription='Key'\nportable=true\nstarting_room='missing'\n"
                .into(),
        )]);
        assert!(snapshot
            .commit(&changes)
            .unwrap_err()
            .contains("Proposed changes"));
        assert!(!f.0.join("items/key.toml").exists());
        fs::write(f.0.join("rooms/start.toml"), "externally changed").unwrap();
        assert!(snapshot
            .commit(&changes)
            .unwrap_err()
            .contains("externally"));
    }
    #[test]
    fn rollback_after_first_publication_removes_only_new_files() {
        let f = Fixture::new();
        let original = fs::read(f.0.join("rooms/start.toml")).unwrap();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        let error = snapshot
            .commit_with(&f.changes(), |n| {
                if n == 1 {
                    Err("injected publish failure".into())
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert!(error.contains("original content restored"));
        assert_eq!(fs::read(f.0.join("rooms/start.toml")).unwrap(), original);
        assert!(!f.0.join("rooms/a_new.toml").exists());
        assert!(!f.0.join(LOCK).exists());
        Adventure::load(&f.0).unwrap();
    }
    #[test]
    fn rollback_restores_moved_existing_file_bytes_and_metadata() {
        let f = Fixture::new();
        crate::creator::add_room(&f.0, "z_existing").unwrap();
        let old = Original::read(&f.0.join("rooms/start.toml")).unwrap();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        let changes = BTreeMap::from([
            (
                PathBuf::from("rooms/start.toml"),
                "id='start'\ntitle='Changed'\ndescription='Text'\n[exits]\nnorth='z_existing'\n"
                    .into(),
            ),
            (
                PathBuf::from("rooms/z_existing.toml"),
                "id='z_existing'\ntitle='Other'\ndescription='Text'\n[exits]\nsouth='start'\n"
                    .into(),
            ),
        ]);
        assert!(snapshot
            .commit_with(&changes, |n| if n == 1 {
                Err("disk failure".into())
            } else {
                Ok(())
            })
            .is_err());
        assert!(Original::read(&f.0.join("rooms/start.toml")).unwrap() == old);
        assert!(!f.0.join(LOCK).exists());
        Adventure::load(&f.0).unwrap();
    }
    #[test]
    fn external_edits_and_new_files_are_not_overwritten() {
        let f = Fixture::new();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        fs::write(f.0.join("rooms/start.toml"), "external edit").unwrap();
        assert!(snapshot
            .commit(&f.changes())
            .unwrap_err()
            .contains("externally"));
        assert_eq!(
            fs::read_to_string(f.0.join("rooms/start.toml")).unwrap(),
            "external edit"
        );
        assert!(!f.0.join(LOCK).exists());
    }
    #[test]
    fn invalid_staged_references_never_publish() {
        let f = Fixture::new();
        let old = fs::read(f.0.join("rooms/start.toml")).unwrap();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        let changes = BTreeMap::from([(
            PathBuf::from("rooms/start.toml"),
            "id='start'\ntitle='Start'\ndescription='Text'\n[exits]\nnorth='missing'\n".into(),
        )]);
        assert!(snapshot
            .commit(&changes)
            .unwrap_err()
            .contains("Proposed changes are invalid"));
        assert_eq!(fs::read(f.0.join("rooms/start.toml")).unwrap(), old);
        assert!(!f.0.join(LOCK).exists());
    }
    #[test]
    fn rollback_conflict_keeps_recovery_and_external_content() {
        let f = Fixture::new();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        let error = snapshot
            .commit_with(&f.changes(), |n| {
                if n == 1 {
                    fs::write(f.0.join("rooms/a_new.toml"), "external content").unwrap();
                    Err("publish failed".into())
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert!(error.contains("rollback needs attention"));
        assert_eq!(
            fs::read_to_string(f.0.join("rooms/a_new.toml")).unwrap(),
            "external content"
        );
        assert!(f.0.join(LOCK).join("RECOVERY.txt").exists());
        assert!(Snapshot::capture(&f.0)
            .err()
            .unwrap()
            .contains("Recovery folder"));
    }
    #[test]
    fn successful_multi_file_commit_validates_and_releases_lock() {
        let f = Fixture::new();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        snapshot.commit(&f.changes()).unwrap();
        let game = Adventure::load(&f.0).unwrap();
        assert_eq!(game.game.world.rooms.len(), 2);
        assert!(!f.0.join(LOCK).exists());
        for (path, text) in f.changes() {
            assert_eq!(fs::read_to_string(f.0.join(path)).unwrap(), text);
        }
    }
    #[test]
    fn invalid_target_path_is_rejected_without_writes() {
        let f = Fixture::new();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        assert!(snapshot
            .commit(&BTreeMap::from([(
                PathBuf::from("../escape.toml"),
                "bad".into()
            )]))
            .is_err());
        assert!(!f.0.join(LOCK).exists());
    }
    #[test]
    fn real_publication_collision_rolls_back_and_preserves_the_collision() {
        let f = Fixture::new();
        let old = fs::read(f.0.join("rooms/start.toml")).unwrap();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        let changes = BTreeMap::from([
            (
                PathBuf::from("rooms/start.toml"),
                "id='start'\ntitle='Changed'\ndescription='Text'\n[exits]\nnorth='z_new'\n".into(),
            ),
            (
                PathBuf::from("rooms/z_new.toml"),
                "id='z_new'\ntitle='New'\ndescription='Text'\n[exits]\nsouth='start'\n".into(),
            ),
        ]);
        let error = snapshot
            .commit_with(&changes, |n| {
                if n == 1 {
                    fs::create_dir(f.0.join("rooms/z_new.toml")).unwrap();
                }
                Ok(())
            })
            .unwrap_err();
        assert!(error.contains("Cannot publish"));
        assert_eq!(fs::read(f.0.join("rooms/start.toml")).unwrap(), old);
        assert!(f.0.join("rooms/z_new.toml").is_dir());
        assert!(!f.0.join(LOCK).exists());
        Adventure::load(&f.0).unwrap();
    }
}
