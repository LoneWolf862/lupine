//! Full beginner journey through the real CLI, with isolated authored files/saves.
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn cli_creates_a_complete_playable_adventure_and_reopens_it() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = Scratch(std::env::temp_dir().join(format!(
        "lupine-acceptance-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&root.0).unwrap();
    let game = root.0.join("Demo adventure");
    let data = root.0.join("userdata");
    let script = include_str!("fixtures/creator_acceptance.txt")
        .replace("{PROJECT}", &game.to_string_lossy());
    let execute = |args: &[&str], script: String| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lupine"))
            .args(args)
            .env("APPDATA", &data)
            .env("XDG_DATA_HOME", &data)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        let writer = std::thread::spawn(move || input.write_all(script.as_bytes()).unwrap());
        let output = child.wait_with_output().unwrap();
        writer.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let out = execute(&["new"], script);
    for expected in [
        "ADVENTURE CREATED!",
        "Locked door saved and validated",
        "Validation successful!",
        "You learn Renew",
        "The test door is locked.",
        "The test door unlocks.",
        "The test doorway stands open.",
        "Loaded: before",
        "Loaded: after",
        "You cannot go west",
        "cannot be moved",
        "[OPEN]",
        "[CLOSED]",
    ] {
        assert!(out.contains(expected), "Missing {expected}:\n{out}");
    }
    let play = out.split_once("Powered by Lupine").expect("play session").1;
    for expected in [
        "The test door is locked.",
        "The test door unlocks.",
        "The test doorway stands open.",
        "[OPEN]",
        "[CLOSED]",
        "You learn Renew",
    ] {
        assert!(
            play.contains(expected),
            "Gameplay missing {expected}: {play}"
        );
    }
    assert!(!out.contains("Save needs attention"), "{out}");
    assert!(!out.contains("Proposed changes are invalid"), "{out}");
    let adventure = lupine::adventure::Adventure::load(&game).unwrap();
    assert_eq!(adventure.game.world.rooms.len(), 3);
    assert_eq!(adventure.game.world.items.len(), 4);
    assert_eq!(adventure.rpg.abilities.len(), 1);
    assert_eq!(adventure.game.config.character.unwrap().starting_hp, 120);
    assert!(!game.join("items/discarded.toml").exists());
    let yard: toml::Value = fs::read_to_string(game.join("rooms/yard.toml"))
        .unwrap()
        .parse()
        .unwrap();
    assert!(!yard["exits"].as_table().unwrap().contains_key("west"));
    let saves = data.join(if cfg!(windows) {
        "Lupine/saves"
    } else {
        "lupine/saves"
    });
    let saves = saves.join("creator_acceptance");
    let mut before = fs::read_dir(&saves)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            (p.clone(), fs::read(p).unwrap())
        })
        .collect::<Vec<_>>();
    before.sort();
    assert_eq!(before.len(), 2);
    let out = execute(
        &["creator", game.to_str().unwrap()],
        "2\n1\nload after\nlook\nquit\n0\n".into(),
    );
    assert!(out.contains("The test doorway stands open."), "{out}");
    for (p, bytes) in before {
        assert_eq!(fs::read(p).unwrap(), bytes);
    }
}
