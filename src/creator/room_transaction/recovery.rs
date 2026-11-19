//! Conservative recovery for Snapshot transactions, using hashed evidence.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, BufRead},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    baseline_valid: bool,
    before: BTreeMap<PathBuf, String>,
    after: BTreeMap<PathBuf, String>,
    targets: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
    notes: BTreeMap<PathBuf, String>,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn prepare(
    s: &Snapshot,
    changes: &BTreeMap<PathBuf, Option<String>>,
    lock: &Path,
    owned: &mut Owned,
) -> Result<(), String> {
    let baseline = lock.join("baseline");
    owned.dir(&baseline)?;
    for dir in &s.dirs {
        owned.dir(&baseline.join(dir))?;
    }
    for (p, o) in &s.files {
        owned.write(&baseline.join(p), &o.bytes)?;
        if changes.contains_key(p) {
            fs::set_permissions(baseline.join(p), o.permissions.clone())
                .map_err(|e| e.to_string())?;
        }
    }
    let before = s
        .files
        .iter()
        .map(|(p, o)| (p.clone(), hash(&o.bytes)))
        .collect::<BTreeMap<_, _>>();
    let mut after = before.clone();
    for (p, text) in changes {
        if let Some(text) = text {
            after.insert(p.clone(), hash(text.as_bytes()));
        } else {
            after.remove(p);
        }
    }
    let notes = ["RECOVERY.txt", "CHANGES.txt"]
        .into_iter()
        .map(|n| {
            Ok((
                PathBuf::from(n),
                hash(&Original::read(&lock.join(n))?.bytes),
            ))
        })
        .collect::<Result<_, String>>()?;
    let journal = Journal {
        version: 2,
        baseline_valid: Adventure::load(&baseline).is_ok(),
        before,
        after,
        targets: changes.keys().cloned().collect(),
        dirs: s.dirs.clone(),
        notes,
    };
    owned.write(
        &lock.join("JOURNAL.toml"),
        toml::to_string(&journal)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
}
fn safe(p: &Path) -> bool {
    !p.as_os_str().is_empty()
        && p.components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}
fn content_path(p: &Path) -> bool {
    safe(p) && (FILES.iter().any(|n| p == Path::new(n)) || DIRS.iter().any(|n| p.starts_with(n)))
}
struct Evidence {
    journal: Journal,
    owned: BTreeMap<PathBuf, String>,
    dirs: Vec<PathBuf>,
    outcome: Option<bool>,
}
fn inspect(root: &Path) -> Result<Evidence, String> {
    tree::root(root)?;
    let lock = root.join(LOCK);
    let meta = fs::symlink_metadata(&lock).map_err(|e| e.to_string())?;
    if files::is_link(&meta) || !meta.is_dir() {
        return Err("Recovery location is not a regular directory; refusing traversal".into());
    }
    let raw=Original::read(&lock.join("JOURNAL.toml")).map_err(|e|format!("No complete supported journal: {e}. Keep this folder and inspect RECOVERY.txt / CHANGES.txt with a developer; automatic recovery is unavailable."))?.bytes;
    let j: Journal = toml::from_str(std::str::from_utf8(&raw).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Corrupt journal: {e}"))?;
    if j.version != 2
        || j.targets.is_empty()
        || j.before.len() > 2048
        || j.after.len() > 2048
        || j.dirs.len() > 2048
        || j.targets
            .iter()
            .any(|p| !editable_path(p, j.before.contains_key(p)))
        || j.before
            .keys()
            .chain(j.after.keys())
            .any(|p| !content_path(p))
        || j.dirs
            .iter()
            .any(|p| !safe(p) || !DIRS.iter().any(|d| p.starts_with(d)))
    {
        return Err("Unsupported or unsafe journal; no files changed".into());
    }
    let targets = j.targets.iter().collect::<BTreeSet<_>>();
    if targets.len() != j.targets.len()
        || j.targets
            .iter()
            .any(|p| !j.after.contains_key(p) && !j.before.contains_key(p))
        || j.before
            .iter()
            .any(|(p, h)| !targets.contains(p) && j.after.get(p) != Some(h))
        || j.after
            .keys()
            .any(|p| !targets.contains(p) && !j.before.contains_key(p))
    {
        return Err("Inconsistent recovery inventory".into());
    }
    let outcome = match fs::symlink_metadata(lock.join("RESOLVED.txt")) {
        Ok(_) => match Original::read(&lock.join("RESOLVED.txt"))?.bytes.as_slice() {
            b"completed\n" => Some(true),
            b"restored\n" => Some(false),
            _ => return Err("Corrupt completion record".into()),
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
    };
    let current = Snapshot::capture_inner(root, true)?;
    for (p, o) in &current.files {
        let h = hash(&o.bytes);
        if j.before.get(p) != Some(&h) && j.after.get(p) != Some(&h) {
            return Err(format!("External edit or unexpected file: {}. Preserve it and the recovery folder; resolve manually.",p.display()));
        }
    }
    for p in j.after.keys() {
        if !current.files.contains_key(p) && !targets.contains(p) {
            return Err(format!("Missing unchanged file: {}", p.display()));
        }
    }
    if let Some(done) = outcome {
        if done || j.baseline_valid {
            Adventure::load(root)?;
        }
        let expected = if done { &j.after } else { &j.before };
        if current.files.len() != expected.len()
            || expected
                .iter()
                .any(|(p, h)| current.files.get(p).is_none_or(|o| hash(&o.bytes) != *h))
        {
            return Err("Adventure changed after recovery; refusing cleanup".into());
        }
    }
    let mut allowed = BTreeMap::from([(PathBuf::from("JOURNAL.toml"), hash(&raw))]);
    for (p, h) in &j.notes {
        if !matches!(p.to_str(), Some("RECOVERY.txt" | "CHANGES.txt")) {
            return Err("Unsafe recovery note".into());
        }
        allowed.insert(p.clone(), h.clone());
    }
    for (p, h) in &j.before {
        allowed.insert(Path::new("baseline").join(p), h.clone());
        if targets.contains(p) {
            allowed.insert(Path::new("original").join(p), h.clone());
        }
    }
    for (p, h) in &j.after {
        allowed.insert(Path::new("proposed").join(p), h.clone());
    }
    if lock.join("COMMITTED.txt").exists() {
        let bytes = Original::read(&lock.join("COMMITTED.txt"))?.bytes;
        if bytes!=b"All proposed content changes were published and the adventure passed validation. If this folder remains, cleanup was interrupted. Review the current adventure before removing recovery files; do not automatically roll back a completed edit.\n"{return Err("Changed completion note".into());}
        allowed.insert("COMMITTED.txt".into(), hash(&bytes));
    }
    if let Some(done) = outcome {
        allowed.insert(
            "RESOLVED.txt".into(),
            hash(if done { b"completed\n" } else { b"restored\n" }),
        );
    }
    let mut expected_dirs = BTreeSet::from([
        PathBuf::from("baseline"),
        PathBuf::from("proposed"),
        PathBuf::from("original"),
    ]);
    for p in allowed.keys() {
        let mut parent = p.parent();
        while let Some(p) = parent.filter(|p| !p.as_os_str().is_empty()) {
            expected_dirs.insert(p.to_owned());
            parent = p.parent();
        }
    }
    for base in ["baseline", "proposed", "original"] {
        for dir in &j.dirs {
            expected_dirs.insert(Path::new(base).join(dir));
        }
    }
    for dir in ["rooms", "items", "spells", "art", "config"] {
        expected_dirs.insert(Path::new("original").join(dir));
    }
    let mut owned = BTreeMap::new();
    let mut dirs = Vec::new();
    scan(
        &lock,
        Path::new(""),
        &allowed,
        &expected_dirs,
        &mut owned,
        &mut dirs,
    )?;
    if outcome.is_none() {
        for p in j.before.keys() {
            if !owned.contains_key(&Path::new("baseline").join(p)) {
                return Err(
                    "Missing baseline data; preserve remaining files for manual review".into(),
                );
            }
        }
        for p in j.after.keys() {
            if !owned.contains_key(&Path::new("proposed").join(p)) {
                return Err("Missing proposed data; automatic recovery refused".into());
            }
        }
        if j.baseline_valid {
            Adventure::load(&lock.join("baseline"))
                .map_err(|e| format!("Baseline validation failed: {e}"))?;
        }
        Adventure::load(&lock.join("proposed"))
            .map_err(|e| format!("Proposed validation failed: {e}"))?;
    }
    Ok(Evidence {
        journal: j,
        owned,
        dirs,
        outcome,
    })
}
fn scan(
    root: &Path,
    rel: &Path,
    allowed: &BTreeMap<PathBuf, String>,
    expected_dirs: &BTreeSet<PathBuf>,
    owned: &mut BTreeMap<PathBuf, String>,
    dirs: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if rel.components().count() > 12 || owned.len() + dirs.len() > 10000 {
        return Err("Recovery traversal limit exceeded".into());
    }
    for e in fs::read_dir(root.join(rel)).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let p = rel.join(e.file_name());
        let m = fs::symlink_metadata(e.path()).map_err(|e| e.to_string())?;
        if files::is_link(&m) {
            return Err(format!("Linked recovery path refused: {}", p.display()));
        }
        if m.is_dir() {
            if !expected_dirs.contains(&p) {
                return Err(format!("Unexpected recovery directory: {}", p.display()));
            }
            dirs.push(p.clone());
            scan(root, &p, allowed, expected_dirs, owned, dirs)?;
        } else {
            let h = hash(&Original::read(&e.path())?.bytes);
            if allowed.get(&p) != Some(&h) {
                return Err(format!(
                    "Changed or unexpected recovery file: {}",
                    p.display()
                ));
            }
            owned.insert(p, h);
        }
    }
    Ok(())
}
fn recover(root: &Path, complete: bool) -> Result<(), String> {
    let evidence = inspect(root)?;
    let j = &evidence.journal;
    let lock = root.join(LOCK);
    if evidence.outcome.is_none() {
        let desired = if complete { &j.after } else { &j.before };
        for p in &j.targets {
            let target = root.join(p);
            let expected = desired.get(p);
            let current = match Original::read(&target) {
                Ok(o) => Some(hash(&o.bytes)),
                Err(e) => {
                    match fs::symlink_metadata(&target) {
                        Err(m) if m.kind() == io::ErrorKind::NotFound => (),
                        _ => return Err(e),
                    }
                    None
                }
            };
            if current.as_ref() == expected {
                continue;
            }
            let source = lock
                .join(if complete { "proposed" } else { "baseline" })
                .join(p);
            if let Some(expected) = expected {
                if hash(&Original::read(&source)?.bytes) != *expected {
                    return Err("Recovery source changed; target retained".into());
                }
            }
            if let Some(h) = &current {
                if j.before.get(p) != Some(h) && j.after.get(p) != Some(h) {
                    return Err("External edit during recovery; evidence retained".into());
                }
                fs::remove_file(&target).map_err(|e| e.to_string())?;
            }
            if expected.is_some() {
                let parent = target.parent().unwrap();
                if !parent.exists() {
                    fs::create_dir(parent).map_err(|e| e.to_string())?;
                }
                fs::hard_link(source, &target).map_err(|e| e.to_string())?;
            }
        }
        if complete || j.baseline_valid {
            Adventure::load(root)?;
        }
        let current = Snapshot::capture_inner(root, true)?;
        if current.files.len() != desired.len()
            || desired
                .iter()
                .any(|(p, h)| current.files.get(p).is_none_or(|o| hash(&o.bytes) != *h))
        {
            return Err("Recovered inventory differs; evidence retained".into());
        }
        let mut owned = Owned {
            files: Vec::new(),
            dirs: Vec::new(),
        };
        owned.write(
            &lock.join("RESOLVED.txt"),
            if complete {
                b"completed\n"
            } else {
                b"restored\n"
            },
        )?;
    }
    let evidence = inspect(root)?;
    for (p, h) in evidence
        .owned
        .iter()
        .filter(|(p, _)| p.components().count() > 1)
    {
        let full = lock.join(p);
        if hash(&Original::read(&full)?.bytes) != *h {
            return Err("Recovery data changed before cleanup".into());
        }
        fs::remove_file(full).map_err(|e| e.to_string())?;
    }
    for p in evidence.dirs.iter().rev() {
        fs::remove_dir(lock.join(p)).map_err(|e| e.to_string())?;
    }
    for p in [
        "COMMITTED.txt",
        "RECOVERY.txt",
        "CHANGES.txt",
        "RESOLVED.txt",
        "JOURNAL.toml",
    ] {
        if let Some(h) = evidence.owned.get(Path::new(p)) {
            if hash(&Original::read(&lock.join(p))?.bytes) != *h {
                return Err("Recovery note changed".into());
            }
            fs::remove_file(lock.join(p)).map_err(|e| e.to_string())?;
        }
    }
    fs::remove_dir(lock).map_err(|e| e.to_string())
}
pub(crate) fn run<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> io::Result<bool> {
    match fs::symlink_metadata(root.join(LOCK)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(true),
        _ => (),
    }
    let mut w = crate::creator::wizard::Wizard { input, output };
    loop {
        w.heading("RECOVERY REQUIRED").map_err(io::Error::other)?;
        let evidence = match inspect(root) {
            Ok(e) => e,
            Err(e) => {
                w.say(&format!("Automatic recovery refused: {e}\nDo not delete or overwrite recovery data. Keep a copy of the entire adventure and recovery folder. Compare retained originals and current files with a developer, then validate. No new edit or play session is allowed here. RORK saves are untouched.")).map_err(io::Error::other)?;
                return Ok(false);
            }
        };
        w.say(&format!("An interrupted content edit requires review. Stop any other creator process first.\nAffected files:\n{}\nNo recovery happens without confirmation.",evidence.journal.targets.iter().map(|p|p.display().to_string()).collect::<Vec<_>>().join("\n"))).map_err(io::Error::other)?;
        w.say(if evidence.outcome.is_some(){"1. Inspect details\n2. Finish verified cleanup\n0. Exit without changes"}else{"1. Inspect details\n2. Restore recorded previous state\n3. Complete validated pending operation\n4. Recovery help\n0. Exit without changes"}).map_err(io::Error::other)?;
        if !evidence.journal.baseline_valid {
            w.say("The recorded baseline was invalid. Restore preserves its exact bytes but it will still need repair; Complete selects a validated proposal.").map_err(io::Error::other)?;
        }
        let choice = match w.menu("Selection:") {
            Ok(c) => c,
            Err(_) => return Ok(false),
        };
        match choice.trim().to_ascii_lowercase().as_str(){
            "0"|"back"=>return Ok(false),
            "1"=>w.say(&format!("Journal version {}; {} original files; {} proposed files. Hashes, confined paths and full adventure validation are checked. Recovery metadata: {}",evidence.journal.version,evidence.journal.before.len(),evidence.journal.after.len(),root.join(LOCK).display())).map_err(io::Error::other)?,
            "2"|"3"=>{let complete=evidence.outcome.unwrap_or(choice.trim()=="3");if evidence.outcome.is_some()&&choice.trim()=="3"{continue;}
                match crate::creator::content_edit::confirm(&mut w,"Confirm this recovery operation? (No player saves will change)"){Ok(true)=>match recover(root,complete){Ok(())=>{w.say("Recovery completed; recorded content restored or validated proposal published. RORK saves unchanged.").map_err(io::Error::other)?;return Ok(true);},Err(e)=>{w.say(&format!("Recovery stopped safely: {e}. Keep the remaining recovery files.")).map_err(io::Error::other)?;return Ok(false);}},Ok(false)=>(),Err(_)=>return Ok(false)}
            },
            _=>w.say("Restore selects the recorded baseline; Complete selects the validated proposal. Neither guesses through external edits. Missing/corrupt or older text-only records need manual review. Never delete unknown files or RORK saves.").map_err(io::Error::other)?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Cursor,
        panic::{catch_unwind, AssertUnwindSafe},
    };
    fn interrupt(at: usize) -> super::super::tests::Fixture {
        let f = super::super::tests::Fixture::new();
        let s = Snapshot::capture(&f.0).unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            s.commit_with(&f.changes(), |n| {
                if n == at {
                    panic!("simulated power loss");
                }
                Ok(())
            })
        }));
        assert!(result.is_err());
        assert!(f.0.join(LOCK).exists());
        f
    }
    #[test]
    fn deletion_and_invalid_baseline_recover_without_guessing() {
        for complete in [false, true] {
            let f = super::super::tests::Fixture::new();
            fs::write(f.0.join("systems.toml"), "invalid syntax [").unwrap();
            fs::write(f.0.join("personal.rork"), "unchanged").unwrap();
            let snapshot = Snapshot::capture(&f.0).unwrap();
            let changes = BTreeMap::from([(PathBuf::from("systems.toml"), None)]);
            assert!(
                catch_unwind(AssertUnwindSafe(|| snapshot.stage(&changes, false, |n| {
                    if n == 1 {
                        panic!("power loss after deletion");
                    }
                    Ok(())
                })))
                .is_err()
            );
            let evidence = inspect(&f.0).unwrap();
            assert!(!evidence.journal.baseline_valid);
            recover(&f.0, complete).unwrap();
            assert_eq!(f.0.join("systems.toml").exists(), !complete);
            assert_eq!(Adventure::load(&f.0).is_ok(), complete);
            assert_eq!(
                fs::read_to_string(f.0.join("personal.rork")).unwrap(),
                "unchanged"
            );
            assert!(!f.0.join(LOCK).exists());
        }
    }
    #[test]
    fn deletion_failure_restores_original_and_external_changes_are_retained() {
        let f = super::super::tests::Fixture::new();
        fs::write(f.0.join("systems.toml"), "# original\n").unwrap();
        let snapshot = Snapshot::capture(&f.0).unwrap();
        let changes = BTreeMap::from([(PathBuf::from("systems.toml"), None)]);
        assert!(snapshot
            .stage(&changes, false, |n| if n == 1 {
                Err("injected".into())
            } else {
                Ok(())
            })
            .is_err());
        assert_eq!(
            fs::read_to_string(f.0.join("systems.toml")).unwrap(),
            "# original\n"
        );
        fs::write(f.0.join("systems.toml"), "# external\n").unwrap();
        assert!(snapshot
            .commit_changes(&changes)
            .unwrap_err()
            .contains("externally"));
        assert_eq!(
            fs::read_to_string(f.0.join("systems.toml")).unwrap(),
            "# external\n"
        );
    }
    #[test]
    fn interrupted_publication_restores_or_completes_at_each_stage() {
        for at in 0..=2 {
            for complete in [false, true] {
                let f = interrupt(at);
                fs::write(f.0.join("personal.rork"), "unchanged").unwrap();
                inspect(&f.0).unwrap();
                recover(&f.0, complete).unwrap();
                let a = Adventure::load(&f.0).unwrap();
                assert_eq!(a.game.world.rooms.len(), if complete { 2 } else { 1 });
                assert!(!f.0.join(LOCK).exists());
                assert_eq!(
                    fs::read_to_string(f.0.join("personal.rork")).unwrap(),
                    "unchanged"
                );
                let mut out = Vec::new();
                assert!(run(&f.0, &mut Cursor::new(""), &mut out).unwrap());
            }
        }
    }
    #[test]
    fn canceled_recovery_and_external_edits_never_overwrite() {
        let f = interrupt(1);
        let before = fs::read(f.0.join("rooms/a_new.toml")).unwrap();
        let mut out = Vec::new();
        assert!(!run(&f.0, &mut Cursor::new("2\nno\n0\n"), &mut out).unwrap());
        assert_eq!(fs::read(f.0.join("rooms/a_new.toml")).unwrap(), before);
        fs::write(f.0.join("rooms/a_new.toml"), "external edit").unwrap();
        assert!(recover(&f.0, false).is_err());
        assert_eq!(
            fs::read_to_string(f.0.join("rooms/a_new.toml")).unwrap(),
            "external edit"
        );
        assert!(f.0.join(LOCK).exists());
    }
    #[test]
    fn missing_corrupt_and_unknown_recovery_files_refuse_all_writes() {
        for variant in 0..4 {
            let f = interrupt(1);
            let lock = f.0.join(LOCK);
            let before = fs::read(f.0.join("rooms/start.toml")).unwrap();
            match variant {
                0 => fs::remove_file(lock.join("JOURNAL.toml")).unwrap(),
                1 => fs::write(lock.join("JOURNAL.toml"), "corrupt").unwrap(),
                2 => fs::remove_file(lock.join("baseline/game.toml")).unwrap(),
                _ => fs::write(lock.join("precious.txt"), "unknown").unwrap(),
            }
            assert!(recover(&f.0, true).is_err());
            assert_eq!(fs::read(f.0.join("rooms/start.toml")).unwrap(), before);
        }
    }
    #[test]
    fn recovery_restarts_after_missing_target_and_cleanup_interruption() {
        let f = interrupt(2);
        fs::remove_file(f.0.join("rooms/start.toml")).unwrap();
        recover(&f.0, false).unwrap();
        Adventure::load(&f.0).unwrap();
        let f = interrupt(2);
        let lock = f.0.join(LOCK);
        fs::write(lock.join("RESOLVED.txt"), "completed\n").unwrap();
        fs::remove_file(lock.join("baseline/game.toml")).unwrap();
        recover(&f.0, true).unwrap();
        Adventure::load(&f.0).unwrap();
        assert!(!lock.exists());
    }
    #[test]
    fn interrupted_id_migration_can_complete_with_all_references() {
        let f = super::super::tests::Fixture::new();
        let s = Snapshot::capture(&f.0).unwrap();
        let (changes, _) = crate::creator::migration::plan(&s, "rooms", "start", "entry").unwrap();
        assert!(
            catch_unwind(AssertUnwindSafe(|| s.commit_with(&changes, |n| {
                if n == 1 {
                    panic!("interrupted migration");
                }
                Ok(())
            })))
            .is_err()
        );
        recover(&f.0, true).unwrap();
        let a = Adventure::load(&f.0).unwrap();
        assert_eq!(a.game.world.starting_room, "entry");
        assert!(a.game.world.room("entry").is_some());
    }
    #[test]
    fn unsafe_journal_paths_are_refused_before_any_recovery_write() {
        let f = interrupt(1);
        let p = f.0.join(LOCK).join("JOURNAL.toml");
        let text = fs::read_to_string(&p).unwrap();
        let mut j: Journal = toml::from_str(&text).unwrap();
        j.targets.push("../escape.toml".into());
        fs::write(p, toml::to_string(&j).unwrap()).unwrap();
        let original = fs::read(f.0.join("rooms/start.toml")).unwrap();
        assert!(recover(&f.0, false).is_err());
        assert_eq!(fs::read(f.0.join("rooms/start.toml")).unwrap(), original);
    }
}
