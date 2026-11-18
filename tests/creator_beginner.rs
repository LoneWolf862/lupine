use lupine::creator::{new_game, new_game_wizard};
use lupine::story_loader::load_game;
use std::{
    fs,
    io::{Cursor, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine-beginner-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn path(&self) -> PathBuf {
        self.0.join("adventure")
    }
    fn run(&self, script: &str) -> String {
        let mut output = Vec::new();
        assert_eq!(
            new_game_wizard(&mut Cursor::new(script), &mut output).unwrap(),
            self.path()
        );
        String::from_utf8(output).unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn beginner_defaults_explain_fields_and_skip_technical_previews() {
    let p = Project::new();
    let output = p.run(&format!("\n\n{}\n\n\n\n", p.path().display()));
    let game = load_game(p.path()).unwrap();
    assert_eq!(game.title, "My Adventure");
    assert_eq!(game.id, "my_adventure");
    assert_eq!(game.world.rooms[0].title, "Starting Room");
    for expected in [
        "No programming experience",
        "STEP 1 OF 4",
        "STEP 2 OF 4",
        "STEP 3 OF 4",
        "STEP 4 OF 4",
        "does not have to be inside",
        "You can change",
        "Resolved project folder:",
        "cargo run -- play",
        "cargo run -- validate",
        "ADVENTURE CREATED!",
    ] {
        assert!(output.contains(expected), "Missing: {expected}");
    }
    assert!(!output.contains("--- exact contents ---"));
    assert!(!output.contains("Game ID ["));
}

#[test]
fn beginner_optional_preview_is_exact_and_educational() {
    let p = Project::new();
    let output = p.run(&format!(
        "\nThe Forgotten Forest\n{}\nClearing\n/multi\nA \"quote\".\n\n雪 and café\n/end\n2\n1\n",
        p.path().display()
    ));
    assert_eq!(load_game(p.path()).unwrap().id, "the_forgotten_forest");
    assert!(output.contains("TOML stores information"));
    assert!(output.contains("title is the display name"));
    for name in [
        "game.toml",
        "rooms/start.toml",
        "interactions.toml",
        "README.txt",
        "config/character.fig",
        "config/terminology.fig",
    ] {
        let marker = format!("Preview: {}\n", p.path().join(name).display());
        let content = output
            .split_once(&marker)
            .unwrap()
            .1
            .split_once("--- exact contents ---\n")
            .unwrap()
            .1
            .split_once("--- end contents ---")
            .unwrap()
            .0;
        assert_eq!(content.as_bytes(), fs::read(p.path().join(name)).unwrap());
    }
}

#[test]
fn back_preserves_answers_and_review_changes_rebuild_the_preview() {
    let p = Project::new();
    let script = format!("\nFirst Title\n{}\nFirst Room\nback\nNew Room\nNew description\n2\n3\nFinal Title\n\n\n\n2\n1\n", p.path().display());
    let output = p.run(&script);
    let game = load_game(p.path()).unwrap();
    assert_eq!(game.title, "Final Title");
    assert_eq!(game.id, "final_title");
    assert_eq!(game.world.rooms[0].title, "New Room");
    assert_eq!(game.world.rooms[0].description, "New description");
    let final_preview = output.rsplit_once("--- exact contents ---\n");
    assert!(final_preview.is_some());
    let manifest = fs::read_to_string(p.path().join("game.toml")).unwrap();
    assert!(output.contains(&manifest));
    assert!(!manifest.contains("First Title"));
    assert!(output.contains("Adventure name [First Title]"));
    assert!(output.contains("Starting location name [New Room]"));
}

#[test]
fn help_back_and_literal_controls_are_distinct_from_story_text() {
    let p = Project::new();
    let script = format!("?\n\nback\n?\n\\cancel\n?\n{}\n\\back\n?\n/multi\n?\n\\back\n\\cancel\n\\?\n\\/end\nplease go back\n/end\n?\nback\n\n1\n", p.path().display());
    let output = p.run(&script);
    let game = load_game(p.path()).unwrap();
    assert_eq!(game.title, "cancel");
    assert_eq!(game.world.rooms[0].title, "back");
    assert_eq!(
        game.world.rooms[0].description,
        "back\ncancel\n?\n/end\nplease go back"
    );
    assert!(output.contains("This is the first step"));
    assert!(output.contains("1 creates the adventure"));
}

#[test]
fn beginner_disambiguates_local_ids_and_advanced_rejects_duplicates() {
    let p = Project::new();
    new_game(&p.0.join("existing"), "same_title", "Existing").unwrap();
    new_game(&p.0.join("second"), "same_title_2", "Existing 2").unwrap();
    let output = p.run(&format!("\nSame Title\n{}\n\n\n1\n", p.path().display()));
    assert_eq!(load_game(p.path()).unwrap().id, "same_title_3");
    assert!(output.contains("same_title_3"));
    let other = p.0.join("advanced");
    let mut output = Vec::new();
    let script = format!(
        "advanced\nSame Title\nBAD ID\ncon\nsame_title\n{}\nunique_id\n\n\n\n1\n",
        other.display()
    );
    new_game_wizard(&mut Cursor::new(script), &mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Another adventure"));
    assert!(output.contains("--- exact contents ---"));
    assert_eq!(load_game(&other).unwrap().id, "unique_id");
    assert_eq!(load_game(p.0.join("existing")).unwrap().title, "Existing");
}

#[test]
fn beginner_cancellation_eof_and_multiline_back_do_not_create_files() {
    let p = Project::new();
    let path = p.path();
    let answers = ["", "Story", path.to_str().unwrap(), "Room", "Description"];
    for step in 0..=answers.len() {
        let prefix = answers[..step]
            .iter()
            .map(|s| format!("{s}\n"))
            .collect::<String>();
        for ending in ["", "cancel\n"] {
            let err = new_game_wizard(
                &mut Cursor::new(format!("{prefix}{ending}")),
                &mut Vec::new(),
            )
            .unwrap_err();
            assert!(err.contains("canceled"));
            assert!(!p.path().exists());
        }
    }
    for ending in ["", "cancel\n", "back\ncancel\n"] {
        let script = format!(
            "\nStory\n{}\nRoom\n/multi\nunfinished\n{ending}",
            p.path().display()
        );
        assert!(new_game_wizard(&mut Cursor::new(script), &mut Vec::new()).is_err());
        assert!(!p.path().exists());
    }
}

fn run_cli(p: &Project, input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .arg("new")
        .current_dir(&p.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn relative_default_folder_collisions_and_completion_commands() {
    let p = Project::new();
    new_game(&p.0.join("my_adventure"), "my_adventure", "Keep me").unwrap();
    let output = run_cli(&p, "\n\n\n\n\n\n");
    let path = p.0.join("my_adventure_2");
    assert_eq!(load_game(&path).unwrap().id, "my_adventure_2");
    assert!(output.contains(&format!("Resolved project folder: {}", path.display())));
    assert!(output.contains("cargo run -- play"));
    assert_eq!(
        load_game(p.0.join("my_adventure")).unwrap().title,
        "Keep me"
    );
    assert!(!p.0.join("saves").exists());
}

#[test]
fn existing_folder_reprompts_and_does_not_touch_user_content() {
    let p = Project::new();
    fs::write(p.0.join("notes.txt"), "user content").unwrap();
    let output = p.run(&format!(
        "\nStory\n{}\n{}\n\n\n1\n",
        p.0.display(),
        p.path().display()
    ));
    assert!(output.contains("already exists"));
    assert_eq!(
        fs::read_to_string(p.0.join("notes.txt")).unwrap(),
        "user content"
    );
}

#[cfg(windows)]
#[test]
fn printed_executable_command_plays_from_powershell_with_special_path() {
    let p = Project::new();
    let folder = "A space's $cash `tick & %value%!";
    let output = run_cli(&p, &format!("\nStory\n{folder}\n\n\n1\n"));
    let command = output.lines().find(|line| line.starts_with("& '")).unwrap();
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("'quit' | {command}"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Starting Room"));
}

#[test]
fn cli_cancel_and_eof_are_friendly_successful_exits() {
    let p = Project::new();
    for script in ["", "cancel\n", "\nStory\nadventure\n\n\n4\n"] {
        let output = run_cli(&p, script);
        assert!(output.contains("Setup canceled"));
        assert!(!p.path().exists());
    }
}

#[test]
fn advanced_back_keeps_custom_id_and_other_answers() {
    let p = Project::new();
    let script = format!("advanced\nOriginal\ncustom_id\nback\n\n{}\nback\n\nRoom\nDescription\n3\nRenamed\n\n\n\n\n1\n", p.path().display());
    let output = p.run(&script);
    let game = load_game(p.path()).unwrap();
    assert_eq!(game.title, "Renamed");
    assert_eq!(game.id, "custom_id");
    assert_eq!(game.world.rooms[0].title, "Room");
    assert!(output.contains("Game ID [custom_id]"));
}

#[cfg(unix)]
#[test]
fn unreadable_sibling_does_not_block_safe_project_creation() {
    use std::os::unix::fs::PermissionsExt;
    let p = Project::new();
    let blocked = p.0.join("private");
    fs::create_dir(&blocked).unwrap();
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).unwrap();
    let mut output = Vec::new();
    let result = new_game_wizard(
        &mut Cursor::new(format!("\nSafe Adventure\n{}\n\n\n1\n", p.path().display())),
        &mut output,
    );
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(result.unwrap(), p.path());
    assert!(load_game(p.path()).is_ok());
}
