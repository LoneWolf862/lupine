//! Bounded ZIP32 stored-entry writer; no compression or extraction implementation.
//! Format reference: https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT
use super::files;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
const FILE_LIMIT: u64 = 256 * 1024 * 1024;
const ARCHIVE_LIMIT: u64 = 512 * 1024 * 1024;
pub(super) fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains(['\\', ':'])
        && !name.chars().any(char::is_control)
        && name
            .trim_end_matches('/')
            .split('/')
            .all(|s| !s.is_empty() && s != "." && s != ".." && !s.ends_with(['.', ' ']))
}
pub(super) fn inventory(root: &Path) -> Result<Vec<(String, PathBuf, bool)>, String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf, bool)>) -> Result<(), String> {
        if dir.components().count() > 12 || out.len() > 8192 {
            return Err("Package traversal limit exceeded".into());
        }
        for e in fs::read_dir(root.join(dir)).map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            let rel = dir.join(e.file_name());
            let m = fs::symlink_metadata(e.path()).map_err(|e| e.to_string())?;
            if files::is_link(&m) || (!m.is_file() && !m.is_dir()) {
                return Err("Package contains an unsafe link or special file".into());
            }
            let raw = rel.to_str().ok_or("Package filenames must be Unicode")?;
            if !cfg!(windows) && raw.contains('\\') {
                return Err("Backslash in archive filename is unsafe".into());
            }
            let mut name = rel
                .to_str()
                .ok_or("Package filenames must be Unicode")?
                .replace('\\', "/");
            if m.is_dir() {
                name.push('/');
            }
            if !safe_name(&name) {
                return Err(format!("Unsafe archive path: {name}"));
            }
            out.push((name, rel.clone(), m.is_dir()));
            if m.is_dir() {
                walk(root, &rel, out)?;
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, Path::new(""), &mut out)?;
    out.sort();
    let mut names = BTreeSet::new();
    for (n, _, _) in &out {
        if !names.insert(n.trim_end_matches('/').to_lowercase()) {
            return Err(format!("Duplicate/case-conflicting archive path: {n}"));
        }
    }
    if out.len() > 8192 {
        return Err("Too many package entries".into());
    }
    Ok(out)
}
pub(super) fn file_hash(path: &Path) -> Result<String, String> {
    let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if files::is_link(&m) || !m.is_file() || m.len() > ARCHIVE_LIMIT {
        return Err("Unsafe or oversized package file".into());
    }
    let mut reader = fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(ARCHIVE_LIMIT + 1);
    let mut h = Sha256::new();
    let mut bytes = [0; 65536];
    let mut total = 0;
    loop {
        let n = reader.read(&mut bytes).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > ARCHIVE_LIMIT {
            return Err("File exceeds size limit".into());
        }
        h.update(&bytes[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn crc(bytes: &[u8]) -> u32 {
    const fn table() -> [u32; 256] {
        let mut t = [0; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut n = 0;
            while n < 8 {
                c = if c & 1 != 0 {
                    0xedb88320 ^ (c >> 1)
                } else {
                    c >> 1
                };
                n += 1;
            }
            t[i] = c;
            i += 1;
        }
        t
    }
    const TABLE: [u32; 256] = table();
    let mut c = u32::MAX;
    for b in bytes {
        c = TABLE[((c ^ u32::from(*b)) & 255) as usize] ^ (c >> 8);
    }
    !c
}
fn u16le(v: &mut Vec<u8>, n: u16) {
    v.extend(n.to_le_bytes());
}
fn u32le(v: &mut Vec<u8>, n: u32) {
    v.extend(n.to_le_bytes());
}
struct Writer {
    file: fs::File,
    hash: Sha256,
    size: u64,
}
impl Writer {
    fn write(&mut self, b: &[u8]) -> Result<(), String> {
        self.size = self
            .size
            .checked_add(b.len() as u64)
            .ok_or("Archive size overflow")?;
        if self.size > ARCHIVE_LIMIT {
            return Err("Archive exceeds 512 MiB limit".into());
        }
        self.file.write_all(b).map_err(|e| e.to_string())?;
        self.hash.update(b);
        Ok(())
    }
}
pub(super) fn create(root: &Path, destination: &Path) -> Result<(), String> {
    let entries = inventory(root)?;
    let mut writer = Writer {
        file: fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|e| e.to_string())?,
        hash: Sha256::new(),
        size: 0,
    };
    let mut central = Vec::new();
    for (name, path, dir) in &entries {
        let bytes = if *dir {
            Vec::new()
        } else {
            let m = fs::symlink_metadata(root.join(path)).map_err(|e| e.to_string())?;
            if files::is_link(&m) || !m.is_file() || m.len() > FILE_LIMIT {
                return Err("ZIP entry exceeds 256 MiB or is not a regular file".into());
            }
            let mut bytes = Vec::new();
            fs::File::open(root.join(path))
                .map_err(|e| e.to_string())?
                .take(FILE_LIMIT + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() as u64 > FILE_LIMIT {
                return Err("ZIP entry grew beyond 256 MiB".into());
            }
            bytes
        };
        let length = u16::try_from(name.len()).map_err(|_| "Archive filename too long")?;
        let size = bytes.len() as u32;
        let crc = crc(&bytes);
        let offset = writer.size as u32;
        let mut local = Vec::new();
        u32le(&mut local, 0x04034b50);
        for n in [20, 0x800, 0, 0, 33] {
            u16le(&mut local, n);
        }
        u32le(&mut local, crc);
        u32le(&mut local, size);
        u32le(&mut local, size);
        u16le(&mut local, length);
        u16le(&mut local, 0);
        local.extend(name.as_bytes());
        writer.write(&local)?;
        writer.write(&bytes)?;
        u32le(&mut central, 0x02014b50);
        for n in [0x314, 20, 0x800, 0, 0, 33] {
            u16le(&mut central, n);
        }
        u32le(&mut central, crc);
        u32le(&mut central, size);
        u32le(&mut central, size);
        for n in [length, 0, 0, 0, 0] {
            u16le(&mut central, n);
        }
        let mode = if *dir {
            0o40755
        } else if name == "lupine" || name == "Play.sh" {
            0o100755
        } else {
            0o100644
        };
        u32le(&mut central, (mode << 16) | if *dir { 0x10 } else { 0 });
        u32le(&mut central, offset);
        central.extend(name.as_bytes());
    }
    let offset = writer.size as u32;
    writer.write(&central)?;
    let mut end = Vec::new();
    u32le(&mut end, 0x06054b50);
    for n in [0, 0, entries.len() as u16, entries.len() as u16] {
        u16le(&mut end, n);
    }
    u32le(&mut end, central.len() as u32);
    u32le(&mut end, offset);
    u16le(&mut end, 0);
    writer.write(&end)?;
    writer.file.sync_all().map_err(|e| e.to_string())?;
    let expected = format!("{:x}", writer.hash.finalize());
    drop(writer.file);
    if file_hash(destination)? != expected {
        return Err("Archive read-back digest mismatch".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crc_and_path_safety() {
        assert_eq!(crc(b"123456789"), 0xcbf43926);
        for n in [
            "../escape",
            "/abs",
            "a/../b",
            "a\\b",
            "C:/x",
            "a//b",
            "a:stream",
            "a/./b",
        ] {
            assert!(!safe_name(n));
        }
        assert!(safe_name("adventure/art/雪 scene.txt"));
    }
    #[test]
    fn archive_limits_and_write_failures_leave_no_final_package() {
        let stage = super::super::export_workspace::reserve(&std::env::temp_dir()).unwrap();
        let input = stage.join("input");
        fs::create_dir(&input).unwrap();
        let big = input.join("large");
        fs::File::create(&big)
            .unwrap()
            .set_len(FILE_LIMIT + 1)
            .unwrap();
        let part = stage.join("archive.part");
        assert!(create(&input, &part).is_err());
        assert!(!stage.join("archive.zip").exists());
        fs::remove_file(big).unwrap();
        fs::remove_file(&part).unwrap();
        fs::remove_dir(input).unwrap();
        fs::write(&part, "existing").unwrap();
        assert!(fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&part)
            .is_err());
        let mut writer = Writer {
            file: fs::File::open(&part).unwrap(),
            hash: Sha256::new(),
            size: 0,
        };
        assert!(writer.write(b"no").is_err());
        writer.size = ARCHIVE_LIMIT;
        assert!(writer.write(b"x").is_err());
        drop(writer);
        assert_eq!(fs::read_to_string(&part).unwrap(), "existing");
        fs::remove_file(part).unwrap();
        fs::remove_dir(stage).unwrap();
    }
}
