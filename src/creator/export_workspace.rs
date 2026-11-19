//! Export staging cleanup is limited to a recorded, hash-verified inventory.
use super::{archive, files};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
const RECORD: &str = "EXPORT_WORKSPACE.toml";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    files: BTreeMap<String, String>,
    dirs: Vec<String>,
}
pub(super) fn reserve(parent: &Path) -> Result<PathBuf, String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    for _ in 0..100 {
        let p = parent.join(format!(
            ".lupine-export-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&p) {
            Ok(()) => return Ok(p),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("Cannot reserve a fresh export workspace".into())
}
pub(super) fn record(root: &Path) -> Result<(), String> {
    let mut record = Record {
        version: 1,
        files: BTreeMap::new(),
        dirs: Vec::new(),
    };
    for (name, p, dir) in archive::inventory(root)? {
        if dir {
            record.dirs.push(name);
        } else {
            record
                .files
                .insert(name, archive::file_hash(&root.join(p))?);
        }
    }
    let text = toml::to_string(&record).map_err(|e| e.to_string())?;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(RECORD))
        .map_err(|e| e.to_string())?;
    f.write_all(text.as_bytes())
        .and_then(|()| f.sync_all())
        .map_err(|e| e.to_string())
}
pub(super) fn cleanup(root: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    if files::is_link(&m)
        || !m.is_dir()
        || !root
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with(".lupine-export-"))
    {
        return Err("Select the exact .lupine-export-* staging directory; final packages and adventures are not cleanup targets".into());
    }
    let path = root.join(RECORD);
    let m=fs::symlink_metadata(&path).map_err(|e|format!("No complete cleanup record: {e}. Preserve this workspace for manual review; do not delete unknown content."))?;
    if files::is_link(&m) || !m.is_file() || m.len() > 4 * 1024 * 1024 {
        return Err("Unsafe cleanup record".into());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let record: Record = toml::from_str(&raw).map_err(|e| e.to_string())?;
    if record.version != 1
        || record.files.len() + record.dirs.len() > 8192
        || record
            .files
            .keys()
            .chain(&record.dirs)
            .any(|n| !archive::safe_name(n))
    {
        return Err("Unsupported cleanup inventory".into());
    }
    let inventory = archive::inventory(root)?;
    for (name, p, dir) in &inventory {
        if name == RECORD {
            continue;
        }
        if *dir {
            if !record.dirs.contains(name) {
                return Err(format!("Unknown directory retained: {name}"));
            }
        } else if record.files.get(name) != Some(&archive::file_hash(&root.join(p))?) {
            return Err(format!("Changed or unknown file retained: {name}"));
        }
    }
    for (name, p, dir) in inventory.iter().rev() {
        if *dir {
            fs::remove_dir(root.join(p)).map_err(|e| e.to_string())?;
        } else if name != RECORD {
            if record.files.get(name) != Some(&archive::file_hash(&root.join(p))?) {
                return Err("File changed during cleanup; retained".into());
            }
            fs::remove_file(root.join(p)).map_err(|e| e.to_string())?;
        }
    }
    if fs::read_to_string(&path).map_err(|e| e.to_string())? != raw {
        return Err("Cleanup record changed".into());
    }
    fs::remove_file(path)
        .and_then(|()| fs::remove_dir(root))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleanup_preserves_unknown_and_modified_files_and_can_resume() {
        let stage = reserve(&std::env::temp_dir()).unwrap();
        fs::create_dir(stage.join("files")).unwrap();
        fs::write(stage.join("files/one"), "one").unwrap();
        fs::write(stage.join("files/two"), "two").unwrap();
        record(&stage).unwrap();
        fs::write(stage.join("precious"), "unknown").unwrap();
        assert!(cleanup(&stage).is_err());
        assert!(stage.join("files/one").exists());
        fs::remove_file(stage.join("precious")).unwrap();
        fs::write(stage.join("files/one"), "modified").unwrap();
        assert!(cleanup(&stage).is_err());
        assert_eq!(
            fs::read_to_string(stage.join("files/one")).unwrap(),
            "modified"
        );
        fs::write(stage.join("files/one"), "one").unwrap();
        fs::remove_file(stage.join("files/two")).unwrap();
        cleanup(&stage).unwrap();
        assert!(!stage.exists());
    }
}
