use lupine::{
    adventure::Adventure,
    creator::{
        export::{ExportPlan, Target},
        new_game,
    },
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture {
    root: PathBuf,
    game: PathBuf,
    binary: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine export {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let game = root.join("My Adventure");
        new_game(&game, "export_test", "Export Test").unwrap();
        let binary = root.join(Target::host().unwrap().executable());
        fs::copy(env!("CARGO_BIN_EXE_lupine"), &binary).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_lupine"))
            .arg("runtime-manifest")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::write(binary.with_extension("runtime.toml"), output.stdout).unwrap();
        Self { root, game, binary }
    }
    fn plan(&self, name: &str) -> Result<ExportPlan, String> {
        ExportPlan::prepare(
            &self.game,
            &self.root.join(name),
            &[(Target::host().unwrap(), self.binary.clone())],
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn snapshot(path: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &Path, path: &Path, result: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(root, &p, result)
            } else {
                result.push((p.strip_prefix(root).unwrap().into(), fs::read(p).unwrap()));
            }
        }
    }
    let mut result = Vec::new();
    walk(path, path, &mut result);
    result.sort();
    result
}

#[test]
fn native_package_is_valid_launchable_and_excludes_saves_and_development_files() {
    let f = Fixture::new();
    fs::write(f.game.join("rules.toml"), "[counters]\nvisits=3\n").unwrap();
    fs::write(f.game.join("events.toml"),"[[events]]\nid='packaged'\non={enter='start'}\nonce=false\ntext='Packaged event executed.'\n").unwrap();
    fs::write(f.game.join("items/tokens.toml"),"id='tokens'\nname='tokens'\ndescription='Counting tokens'\nportable=true\nstackable=true\nquantity=3\ninitial={kind='inventory'}\n").unwrap();
    fs::write(f.game.join("actions.toml"), "[[actions]]\nid='wait_test'\ncommands=['wait']\nsuccess_text='Time passes.'\nfailure_text='Cannot wait.'\n").unwrap();
    for relative in [
        "private.rork",
        "items/private.rork",
        "config/private.rork",
        "NPCs/private.rork",
        "art/private.rork",
        "notes.rs",
    ] {
        fs::write(f.game.join(relative), "personal").unwrap();
    }
    fs::create_dir(f.game.join("target")).unwrap();
    fs::write(f.game.join("target/dev.txt"), "dev").unwrap();
    fs::write(f.game.join("NPCs/guide.toml"),"id='guide'\nname='Guide'\ndescription='Packaged NPC.'\nroom='start'\nstart='hello'\n[nodes.hello]\ntext='Packaged dialogue executed.'\n").unwrap();
    fs::write(f.game.join("config/character.fig"), "enabled=true\n").unwrap();
    fs::write(
        f.game.join("config/combat.fig"),
        "minimum_damage=0\nretaliation='none'\n",
    )
    .unwrap();
    fs::write(f.game.join("items/coat.toml"),"id='coat'\nname='coat'\ndescription='Exported equipment'\nportable=true\ninitial={kind='inventory'}\n").unwrap();
    fs::write(f.game.join("systems.toml"),"[equipment.coat]\nslot='body'\ndefense=3\n[statuses.poison]\nname='Poison'\nturns=3\ndamage=1\n").unwrap();
    let before = snapshot(&f.game);
    let path = f.plan("Export with spaces").unwrap().publish().unwrap();
    assert_eq!(snapshot(&f.game), before);
    assert!(!path.join("EXPORT_INCOMPLETE.txt").exists());
    let package = path.join(Target::host().unwrap().label());
    let packaged = Adventure::load(&package.join("adventure")).unwrap();
    assert_eq!(packaged.game.world.rules.counters["visits"], 3);
    assert_eq!(packaged.game.world.actions[0].id, "wait_test");
    assert_eq!(packaged.game.world.systems.equipment["coat"].defense, 3);
    assert_eq!(packaged.game.config.combat.minimum_damage, 0);
    assert_eq!(
        lupine::state::GameState::new(&packaged.game.world).carried_quantity("tokens"),
        3
    );
    assert_eq!(
        fs::read(package.join("adventure/rules.toml")).unwrap(),
        fs::read(f.game.join("rules.toml")).unwrap()
    );
    assert!(package.join("START_HERE.txt").is_file());
    assert!(package
        .join(if cfg!(windows) { "Play.cmd" } else { "Play.sh" })
        .is_file());
    assert!(snapshot(&package)
        .iter()
        .all(|(p, _)| p.extension().is_none_or(|e| e != "rork")
            && !p.to_string_lossy().contains("notes.rs")
            && !p.starts_with("adventure/target")));
    let unrelated = f.root.join("Empty working directory");
    fs::create_dir(&unrelated).unwrap();
    let mut child = Command::new(package.join(Target::host().unwrap().executable()))
        .current_dir(&unrelated)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"1\n1\nlook\ntalk guide\nquit\n4\n")
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("Welcome to Lupine!"));
    assert!(text.contains("Powered by Lupine"), "{text}");
    assert!(text.contains("Packaged event executed."), "{text}");
    assert!(text.contains("Packaged dialogue executed."), "{text}");
}
#[test]
fn preflight_rejects_missing_wrong_or_mismatched_binaries_before_writing() {
    let f = Fixture::new();
    let output = f.root.join("export");
    assert!(ExportPlan::prepare(
        &f.game,
        &output,
        &[(Target::Windows, f.root.join("missing.exe"))]
    )
    .is_err());
    let wrong = if Target::host() == Some(Target::Windows) {
        Target::Linux
    } else {
        Target::Windows
    };
    assert!(
        ExportPlan::prepare(&f.game, &output, &[(wrong, f.binary.clone())])
            .err()
            .unwrap()
            .contains("not a")
    );
    let metadata = f.binary.with_extension("runtime.toml");
    let original = fs::read_to_string(&metadata).unwrap();
    fs::remove_file(&metadata).unwrap();
    assert!(f.plan("export").err().unwrap().contains("matching"));
    for (key, value) in [
        ("sha256", "bad"),
        ("version", "0.0.0"),
        ("target", "unsupported"),
    ] {
        let mut info: toml::Table = original.parse().unwrap();
        info.insert(key.into(), toml::Value::String(value.into()));
        fs::write(&metadata, toml::to_string(&info).unwrap()).unwrap();
        assert!(f.plan("export").is_err());
    }
    assert!(!output.exists());
}
#[test]
fn invalid_story_unsafe_output_and_existing_paths_preserve_originals() {
    let f = Fixture::new();
    assert!(ExportPlan::prepare(
        &f.game,
        &f.game.join("export"),
        &[(Target::host().unwrap(), f.binary.clone())]
    )
    .is_err());
    assert!(f.plan("../escape").is_err());
    fs::create_dir(f.root.join("existing")).unwrap();
    fs::write(f.root.join("existing/keep"), "keep").unwrap();
    assert!(f.plan("existing").is_err());
    assert_eq!(
        fs::read_to_string(f.root.join("existing/keep")).unwrap(),
        "keep"
    );
    fs::write(f.game.join("rooms/start.toml"), "invalid").unwrap();
    assert!(f.plan("export").is_err());
    assert!(!f.root.join("export").exists());
}
#[test]
fn guided_export_cancel_eof_and_decline_do_not_publish() {
    let f = Fixture::new();
    let target = if cfg!(windows) { "1" } else { "2" };
    for script in [
        "".to_owned(),
        "0\n".into(),
        format!("{target}\ncancel\n"),
        format!(
            "{target}\n2\n{}\n{}\nno\n",
            f.binary.display(),
            f.root.join("export").display()
        ),
    ] {
        lupine::creator::export::run(&f.game, &mut std::io::Cursor::new(script), &mut Vec::new())
            .unwrap();
        assert!(!f.root.join("export").exists());
    }
}

#[test]
fn zip_round_trip_is_independently_readable_and_never_overwrites() {
    use lupine::creator::export::OutputFormat;
    let f = Fixture::new();
    fs::write(f.game.join("art/雪 scene.txt"), "Unicode art\n").unwrap();
    fs::write(f.game.join("private.rork"), "never include").unwrap();
    let before = snapshot(&f.game);
    let path = f
        .plan("ZIP export")
        .unwrap()
        .publish_format(OutputFormat::Zip)
        .unwrap();
    let archive = fs::read_dir(&path).unwrap().next().unwrap().unwrap().path();
    let extracted = f.root.join("extracted");
    let result=Command::new("python").args(["-c", "import sys,zipfile; z=zipfile.ZipFile(sys.argv[1]); assert z.testzip() is None; assert len(z.namelist())==len(set(z.namelist())); assert 'adventure/art/雪 scene.txt' in z.namelist(); assert 'adventure/enemies/' in z.namelist(); assert not any(n.endswith('.rork') for n in z.namelist()); z.extractall(sys.argv[2])"]).arg(&archive).arg(&extracted).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Adventure::load(&extracted.join("adventure")).unwrap();
    assert_eq!(
        fs::read(extracted.join(Target::host().unwrap().executable())).unwrap(),
        fs::read(&f.binary).unwrap()
    );
    assert_eq!(snapshot(&f.game), before);
    assert!(f.plan("ZIP export").is_err());
    assert!(fs::read_dir(&f.root).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".lupine-export-")));
}
#[test]
fn both_platform_preflight_and_package_structure_with_synthetic_foreign_header() {
    // A structural/digest test fixture, NOT evidence that a foreign OS executable ran.
    use sha2::{Digest, Sha256};
    let f = Fixture::new();
    let foreign = if cfg!(windows) {
        Target::Linux
    } else {
        Target::Windows
    };
    let dir = f.root.join("foreign");
    fs::create_dir(&dir).unwrap();
    let binary = dir.join(foreign.executable());
    let mut bytes = vec![0u8; 256];
    match foreign {
        Target::Linux => {
            bytes[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
            bytes[16] = 2;
            bytes[18] = 62;
        }
        Target::Windows => {
            bytes[..2].copy_from_slice(b"MZ");
            bytes[0x3c] = 128;
            bytes[128..134].copy_from_slice(b"PE\0\0\x64\x86");
            bytes[152] = 0x0b;
            bytes[153] = 2;
        }
    }
    fs::write(&binary, &bytes).unwrap();
    let info = format!(
        "engine='lupine'\nversion='{}'\nexport_format=1\ntarget='{}'\nsha256='{:x}'\n",
        env!("CARGO_PKG_VERSION"),
        foreign.label(),
        Sha256::digest(&bytes)
    );
    fs::write(binary.with_extension("runtime.toml"), info).unwrap();
    let output = f.root.join("both");
    let path = ExportPlan::prepare(
        &f.game,
        &output,
        &[
            (Target::host().unwrap(), f.binary.clone()),
            (foreign, binary.clone()),
        ],
    )
    .unwrap()
    .publish()
    .unwrap();
    for target in [Target::Windows, Target::Linux] {
        assert!(path
            .join(target.label())
            .join(target.executable())
            .is_file());
        Adventure::load(&path.join(target.label()).join("adventure")).unwrap();
    }
    let zip_path = ExportPlan::prepare(
        &f.game,
        &f.root.join("both ZIPs"),
        &[
            (Target::host().unwrap(), f.binary.clone()),
            (foreign, binary),
        ],
    )
    .unwrap()
    .publish_format(lupine::creator::export::OutputFormat::Zip)
    .unwrap();
    for target in [Target::Windows, Target::Linux] {
        assert!(zip_path
            .join(format!("Export_Test-{}.zip", target.label()))
            .is_file());
    }
    assert_eq!(fs::read_dir(zip_path).unwrap().count(), 2);
}

#[test]
fn authored_publication_metadata_exports_without_changing_save_compatibility() {
    let f = Fixture::new();
    let p = f.game.join("game.toml");
    let mut text = fs::read_to_string(&p).unwrap();
    text.push_str("\n[metadata]\nauthor='Example Author'\nversion='Preview'\ndescription='Test description'\ncredits='Authored credits'\nlicense='Authored terms'\ninstructions='Authored instructions'\n");
    fs::write(&p, &text).unwrap();
    let before = snapshot(&f.game);
    let out = f.plan("metadata").unwrap().publish().unwrap();
    let package = out.join(Target::host().unwrap().label());
    assert_eq!(
        fs::read_to_string(package.join("GAME_LICENSE.txt")).unwrap(),
        "Authored terms"
    );
    assert_eq!(
        fs::read_to_string(package.join("GAME_CREDITS.txt")).unwrap(),
        "Authored credits"
    );
    assert_eq!(
        fs::read_to_string(package.join("GAME_INSTRUCTIONS.txt")).unwrap(),
        "Authored instructions"
    );
    assert!(fs::read_to_string(package.join("ABOUT_GAME.txt"))
        .unwrap()
        .contains("Example Author"));
    assert_eq!(snapshot(&f.game), before);
    let a = Adventure::load(&package.join("adventure")).unwrap();
    assert_eq!(a.game.save_compatibility_version, 1);
    fs::write(p, text.replace("'Example Author'", "\"\\u001b[31m\"")).unwrap();
    assert!(Adventure::load(&f.game)
        .err()
        .unwrap()
        .contains("metadata.author"));
}
