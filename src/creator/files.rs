//! Exclusive publication of complete, synced files. No overwrite fallback.
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn validate_destination(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() || path.file_name().is_none() {
        return Err("Choose a new project folder".into());
    }
    if path.components().any(|c| matches!(c, Component::Prefix(prefix) if !matches!(prefix.kind(), std::path::Prefix::Disk(_)))) {
        return Err("Use a local folder path, not a device or network path".into());
    }
    if path.components().any(|c| matches!(c, Component::Prefix(_))) && !path.is_absolute() {
        return Err(
            "Use a full path such as C:\\Games\\my_adventure, not a drive-relative path".into(),
        );
    }
    let mut current = std::path::PathBuf::new();
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err("Parent traversal (..) is not allowed".into());
        }
        if let Component::Normal(name) = component {
            let name = name.to_string_lossy();
            if reserved_name(&name)
                || name
                    .chars()
                    .any(|c| c.is_control() || "<>:\"|?*".contains(c))
                || name.ends_with(['.', ' '])
            {
                return Err(
                    "Folder names must be portable and contain no reserved characters".into(),
                );
            }
        }
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if is_link(&metadata) => {
                return Err(format!(
                    "Symlink/reparse path is not allowed: {}",
                    current.display()
                ))
            }
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(format!("{}: {e}", current.display())),
        }
    }
    if fs::symlink_metadata(path).is_ok() {
        return Err(format!(
            "{} already exists; choose a new folder",
            path.display()
        ));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !parent.is_dir() {
        return Err(format!(
            "Parent directory does not exist: {}",
            parent.display()
        ));
    }
    Ok(())
}
pub(super) fn reserved_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_lowercase();
    matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        || ((stem.starts_with("com") || stem.starts_with("lpt"))
            && stem.len() == 4
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

pub(super) fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

pub(super) fn create_file(path: &Path, content: &str) -> Result<(), String> {
    publish_file(path, |file| file.write_all(content.as_bytes()))
}
fn publish_file(
    path: &Path,
    write: impl FnOnce(&mut fs::File) -> io::Result<()>,
) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().ok_or("Missing parent directory")?;
    let (staged, mut file) = loop {
        let staged = parent.join(format!(
            ".lupine-{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
        {
            Ok(file) => break (staged, file),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("{}: {e}", staged.display())),
        }
    };
    let result = write(&mut file).and_then(|()| file.sync_all());
    drop(file);
    // hard_link atomically creates a new name and fails if any target exists.
    // The sibling staging file ensures the same filesystem; unsupported filesystems fail safely.
    let result = result.and_then(|()| fs::hard_link(&staged, path));
    let cleanup = fs::remove_file(&staged);
    match result {
        Err(e) => Err(format!(
            "{}: {e}{}",
            path.display(),
            cleanup
                .err()
                .map(|e| format!("; staging cleanup {}: {e}", staged.display()))
                .unwrap_or_default()
        )),
        // A complete published target is success, even if its temporary link remains.
        Ok(()) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_write_never_publishes_partial_content() {
        let root =
            std::env::temp_dir().join(format!("lupine-write-failure-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let target = root.join("test.toml");
        let result = publish_file(&target, |file| {
            file.write_all(b"partial")?;
            Err(io::Error::other("injected disk failure"))
        });
        assert!(result.unwrap_err().contains("injected disk failure"));
        assert!(!target.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(root).unwrap();
    }
}
