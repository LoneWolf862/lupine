use lupine::startup::{discover, run};
use std::{io::Cursor, path::PathBuf};

#[test]
fn actual_cli_without_arguments_and_explicit_fixture() {
    use std::process::{Command, Stdio};
    let output = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Welcome to Lupine!"));
    let output = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .arg("fixture")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Engine Test Adventure"));
}

fn session(script: &str, roots: &[PathBuf]) -> String {
    let saves =
        std::env::temp_dir().join(format!("lupine-startup-no-saves-{}", std::process::id()));
    let mut output = Vec::new();
    run(&mut Cursor::new(script), &mut output, &saves, roots).unwrap();
    assert!(!saves.exists());
    String::from_utf8(output).unwrap()
}

#[test]
fn navigation_help_creator_cancellation_and_eof() {
    let text = session("wrong\n3\n2\n0\n1\nnope\n0\n4\n", &[]);
    assert!(text.contains("Please choose"));
    assert!(text.contains("Play explores"));
    assert!(text.contains("LUPINE ADVENTURE CREATOR"));
    assert!(text.contains("No adventures found"));
    for script in ["", "1\n", "1\nb\n", "2\n"] {
        session(script, &[]);
    }
}

#[test]
fn browse_invalid_path_then_valid_adventure_and_return() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("story_example");
    let text = session(
        &format!(
            "1\nb\nmissing-startup-adventure\nb\n{}\nlook\nquit\n4\n",
            path.display()
        ),
        &[],
    );
    assert!(text.contains("Cannot open folder"));
    assert!(text.contains("Powered by Lupine"));
    assert_eq!(text.matches("Welcome to Lupine!").count(), 2);
}

#[test]
fn discovery_deduplicates_and_number_starts_a_game() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("story_example");
    let choices = discover(&[root.clone(), root.clone()]);
    assert_eq!(choices.len(), 1);
    assert!(!choices[0].title.is_empty());
    let text = session("1\n1\nquit\n4\n", &[root]);
    assert!(text.contains("Powered by Lupine"));
}

#[test]
fn invalid_adventure_reports_validation_without_starting() {
    let root = std::env::temp_dir().join(format!("lupine-startup-invalid-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("game.toml"), "not valid toml").unwrap();
    let text = session("1\n1\n0\n4\n", std::slice::from_ref(&root));
    assert!(text.contains("Cannot play"));
    assert!(!text.contains("Powered by Lupine"));
    std::fs::remove_file(root.join("game.toml")).unwrap();
    std::fs::remove_dir(root).unwrap();
}
