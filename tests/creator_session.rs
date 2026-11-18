use lupine::creator::{new_game, session};
use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture {
    root: PathBuf,
    game: PathBuf,
    saves: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine-session-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let game = root.join("My adventure's files");
        new_game(&game, "session_test", "Session Test").unwrap();
        let saves = root.join("manual_saves");
        Self { root, game, saves }
    }
    fn run(&self, script: &str) -> String {
        let mut output = Vec::new();
        session::open(
            &self.game,
            &mut Cursor::new(script),
            &mut output,
            &self.saves,
        )
        .unwrap();
        String::from_utf8(output).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                out.push((
                    path.strip_prefix(root).unwrap().into(),
                    fs::read(path).unwrap(),
                ));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

#[test]
fn navigation_play_validation_and_reopen_preserve_content_and_saves() {
    let f = Fixture::new();
    fs::create_dir(&f.saves).unwrap();
    fs::write(
        f.saves.join("existing.session_test.rork"),
        "existing player save",
    )
    .unwrap();
    let before = snapshot(&f.root);
    let output = f.run("bad\nhelp\n3\n4\n5\n6\n7\n0\n2\n1\nlook\nquit\n2\n1\nquit\n0\n");
    assert!(output.contains("Please choose 0-8"));
    assert!(output.contains("not available yet"));
    assert_eq!(output.matches("Validation successful!").count(), 2);
    assert_eq!(output.matches("Returned to the creator.").count(), 2);
    assert!(output.contains("Starting Room"));
    assert!(output.matches("+-- start.toml").count() >= 5);
    assert_eq!(snapshot(&f.root), before);
    assert!(f.run("2\ncancel\n").contains("Validation successful!"));
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn tree_reflects_actual_recognized_paths_and_empty_directories() {
    let f = Fixture::new();
    fs::create_dir(f.game.join("config")).unwrap();
    fs::write(
        f.game.join("config/terminology.fig"),
        "[display]\nspells = 'Programs'\n",
    )
    .unwrap();
    fs::create_dir(f.game.join("spells")).unwrap();
    fs::create_dir(f.game.join("art/nested")).unwrap();
    fs::write(f.game.join("art/nested/sketch.txt"), " /\\\n/  \\").unwrap();
    fs::write(f.game.join("rooms/draft.tmp"), "ignore me").unwrap();
    fs::create_dir(f.game.join("unrelated")).unwrap();
    fs::write(f.game.join("unrelated/private.txt"), "ignore me too").unwrap();
    let output = f.run("0\n").replace('\\', "/");
    for text in [
        "game.toml",
        "rooms/",
        "items/",
        "art/",
        "spells/",
        "+-- config/\n|   +-- terminology.fig",
        "+-- art/\n    +-- nested/\n        +-- sketch.txt",
        "interactions.toml",
    ] {
        assert!(output.contains(text), "{text}");
    }
    assert!(!output.contains("draft.tmp"));
    assert!(!output.contains("unrelated"));
    assert!(!output.contains("character.fig"));
    assert!(!f.saves.exists());
}

#[test]
fn invalid_schema_and_fig_errors_return_to_menu_without_writes() {
    let f = Fixture::new();
    let room = f.game.join("rooms/start.toml");
    fs::write(
        &room,
        "id='start'\ntitle='Start'\ndescription='Text'\n[exits]\nnorth='missing'\n",
    )
    .unwrap();
    let before = snapshot(&f.game);
    let output = f.run("2\n1\nhelp\n0\n");
    assert!(output.contains("missing"));
    assert!(output.contains("start.toml"));
    assert!(!output.contains("Powered by Lupine"));
    assert!(output.contains("Nothing was changed"));
    assert_eq!(snapshot(&f.game), before);
    fs::write(
        &room,
        "id='start'\ntitle='Start'\ndescription='Text'\n[exits]\n",
    )
    .unwrap();
    fs::create_dir(f.game.join("config")).unwrap();
    fs::write(f.game.join("config/gameplay.fig"), "unsupported=true").unwrap();
    let output = f.run("2\n0\n");
    assert!(output.contains("gameplay.fig"));
    assert!(output.contains("unknown configuration"));
    assert!(!f.saves.exists());
}

#[test]
fn full_rpg_validation_is_shared_with_cli() {
    let f = Fixture::new();
    fs::create_dir(f.game.join("spells")).unwrap();
    fs::write(f.game.join("spells/bad.toml"), "id='bad'\nname='Bad'\nmana_cost=1\nkind='heal'\npower=5\nlearn_from_item='missing_scroll'\n").unwrap();
    let output = f.run("2\n0\n");
    assert!(output.contains("bad.toml"));
    assert!(output.contains("missing_scroll"));
    let cli = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .arg("validate")
        .arg(&f.game)
        .output()
        .unwrap();
    assert!(!cli.status.success());
    assert!(String::from_utf8_lossy(&cli.stderr).contains("missing_scroll"));
}

#[test]
fn eof_in_game_returns_and_exits_without_any_save() {
    let f = Fixture::new();
    let before = snapshot(&f.game);
    let output = f.run("1\nlook\n");
    assert!(output.contains("Returned to the creator"));
    assert!(output.contains("End of input"));
    assert_eq!(snapshot(&f.game), before);
    assert!(!f.saves.exists());
}

#[test]
fn launcher_create_open_cancel_and_invalid_paths() {
    let f = Fixture::new();
    let new = f.root.join("new adventure");
    let script = format!("?\n2\nnot_a_real_project\nback\n1\ncancel\n1\n\nNew Story\n{}\n\n\n1\n2\n0\n2\n{}\n2\n0\n0\n", new.display(), new.display());
    let mut output = Vec::new();
    session::start(&mut Cursor::new(script), &mut output, &f.saves).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Cannot open"));
    assert!(output.contains("Setup canceled"));
    assert_eq!(output.matches("Validation successful!").count(), 2);
    assert!(new.join("rooms/start.toml").is_file());
    assert!(!f.saves.exists());
}

#[test]
fn interactive_new_enters_menu_and_creator_path_reopens() {
    let f = Fixture::new();
    let new = f.root.join("created");
    let mut child = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .arg("new")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("\nTitle\n{}\n\n\n1\n2\n0\n", new.display()).as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Validation successful!"));
    let mut child = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .arg("creator")
        .arg(&new)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"2\n0\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Validation successful!"));
}

#[test]
fn bounded_tree_reports_large_or_deep_content_without_dumping_it() {
    let f = Fixture::new();
    for n in 0..180 {
        fs::write(f.game.join(format!("art/{n:03}.txt")), "art").unwrap();
    }
    let output = f.run("0\n");
    assert!(output.contains("display limited"));
    assert!(output.len() < 20000);
    let mut path = f.game.join("art");
    for _ in 0..10 {
        path.push("nested");
        fs::create_dir(&path).unwrap();
    }
    let output = f.run("2\n0\n");
    assert!(output.contains("inspection limit"));
    assert!(!output.contains("Validation successful"));
}

#[test]
fn junctions_or_symlinks_are_not_followed() {
    let f = Fixture::new();
    let outside = f.root.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("secret.txt"), "outside content").unwrap();
    let link = f.game.join("art/linked");
    #[cfg(windows)]
    {
        let result = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&outside)
            .output()
            .unwrap();
        assert!(result.status.success());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &link).unwrap();
    let output = f.run("2\n1\n0\n");
    assert!(output.contains("link/junction skipped"));
    assert!(!output.contains("secret.txt"));
    assert!(!output.contains("Powered by Lupine"));
    assert_eq!(
        fs::read_to_string(outside.join("secret.txt")).unwrap(),
        "outside content"
    );
    #[cfg(windows)]
    fs::remove_dir(link).unwrap();
    #[cfg(unix)]
    fs::remove_file(link).unwrap();
}

#[test]
fn refreshed_tree_and_validation_read_external_changes() {
    use std::io::{self, BufRead, Read};
    struct ChangingInput {
        cursor: Cursor<Vec<u8>>,
        game: PathBuf,
        changed: bool,
    }
    impl Read for ChangingInput {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.cursor.read(buf)
        }
    }
    impl BufRead for ChangingInput {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            self.cursor.fill_buf()
        }
        fn consume(&mut self, n: usize) {
            self.cursor.consume(n);
        }
        fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
            if !self.changed {
                fs::write(self.game.join("art/new.txt"), "new art\n")?;
                fs::write(
                    self.game.join("rooms/start.toml"),
                    "id='start'\ntitle='Updated Room'\ndescription='Edited externally.'\n[exits]\n",
                )?;
                self.changed = true;
            }
            self.cursor.read_line(buf)
        }
    }
    let f = Fixture::new();
    let mut input = ChangingInput {
        cursor: Cursor::new(b"\n2\n1\nquit\n0\n".to_vec()),
        game: f.game.clone(),
        changed: false,
    };
    let mut output = Vec::new();
    session::open(&f.game, &mut input, &mut output, &f.saves).unwrap();
    let output = String::from_utf8(output).unwrap().replace('\\', "/");
    let sections = output
        .split("Project files (paths relative")
        .collect::<Vec<_>>();
    assert!(!sections[1].contains("new.txt"));
    assert!(sections[2].contains("+-- new.txt"));
    assert!(output.contains("Updated Room"));
    assert!(!f.saves.exists());
}

#[test]
fn explicit_manual_save_still_works_and_is_not_implicitly_rewritten() {
    let f = Fixture::new();
    let output = f.run("1\nsave creator_slot\nquit\n0\n");
    assert!(output.contains("Returned to the creator"));
    let before = snapshot(&f.saves);
    assert_eq!(before.len(), 1);
    assert!(before[0].0.to_string_lossy().ends_with(".rork"));
    f.run("2\n1\nlook\nquit\n0\n");
    assert_eq!(snapshot(&f.saves), before);
}

#[test]
fn bundled_rpg_games_validate_and_play_through_session() {
    let f = Fixture::new();
    for name in [
        "rpg_spellbook",
        "rpg_rewards",
        "rpg_transactions",
        "rpg_persistent",
    ] {
        let game = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(name);
        let before = snapshot(&game);
        let mut output = Vec::new();
        session::open(
            &game,
            &mut Cursor::new("2\n1\nspellbook\nquit\nyes\n0\n"),
            &mut output,
            &f.saves,
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(
            output.contains("Validation successful!"),
            "{name}: {output}"
        );
        assert!(output.contains("Powered by Lupine"));
        assert!(output.contains("Returned to the creator"));
        assert_eq!(snapshot(&game), before);
        assert!(!f.saves.exists());
    }
}

#[test]
fn nested_tree_is_sorted_bounded_and_platform_independent() {
    let f = Fixture::new();
    fs::create_dir_all(f.game.join("art/z_empty")).unwrap();
    fs::create_dir_all(f.game.join("art/a_group/deeper")).unwrap();
    for name in ["z.txt", "a.txt"] {
        fs::write(f.game.join("art/a_group/deeper").join(name), "art").unwrap();
    }
    let output = f.run("0\n");
    let tree = output
        .split("Project files (paths relative to the adventure folder):\n")
        .nth(1)
        .unwrap()
        .split("\n\n")
        .next()
        .unwrap();
    assert!(tree.contains("+-- art/\n    +-- a_group/\n    |   +-- deeper/\n    |       +-- a.txt\n    |       +-- z.txt\n    +-- z_empty/"), "{tree}");
    assert!(!tree.contains('\\'));
    assert!(tree.is_ascii());
    assert_eq!(output, f.run("0\n"));
}

#[test]
fn long_tree_components_are_abbreviated_without_losing_nesting() {
    let f = Fixture::new();
    let long = "a".repeat(185);
    fs::create_dir(f.game.join("art").join(&long)).unwrap();
    fs::write(
        f.game.join("art").join(format!("{}.txt", "z".repeat(185))),
        "art",
    )
    .unwrap();
    let output = f.run("0\n");
    assert!(output.contains("... [long path abbreviated]/"));
    assert_eq!(output.matches("[long path abbreviated]").count(), 2);
    assert!(!output.contains(&long));
    assert!(output
        .lines()
        .filter(|l| l.contains("+--"))
        .all(|l| l.len() < 220));
}

#[test]
fn menus_prompts_and_validation_have_clear_spacing() {
    let f = Fixture::new();
    let output = f.run("2\n7\n0\n0\n");
    assert!(output.contains(
        "========================================\n\n  0. Exit Creator\n  1. Play Your Adventure"
    ));
    assert!(output.contains("\nSelection:\n> "));
    assert!(output.contains("Adventure: Session Test\n\nExisting Rooms:\n\n  1. Starting Room"));
    assert!(output.contains("\n\n  N. Create New Room\n  0. Return to Creator Menu"));
    assert!(output.contains("\nValidation successful! Your adventure is ready to play.\n\n"));
}
