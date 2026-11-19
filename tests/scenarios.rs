use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "lupine-scenarios-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn authored_scenarios_use_real_player_path_repeatably_without_saves() {
    let f = Scratch::new();
    let mut outputs = Vec::new();
    for _ in 0..2 {
        let out = Command::new(env!("CARGO_BIN_EXE_lupine"))
            .args(["test", "story_example", "story_example/scenarios.toml"])
            .env("APPDATA", f.0.join("userdata"))
            .env("XDG_DATA_HOME", f.0.join("userdata"))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout)
                .matches("PASS:")
                .count(),
            3
        );
        outputs.push(out.stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
    assert!(!f.0.join("userdata").exists());
}

#[test]
fn assertion_failures_fail_cli_and_scenarios_are_independent() {
    let f = Scratch::new();
    let file = f.0.join("cases.toml");
    fs::write(&file, "format_version=1\n[[scenario]]\nname='wrong room'\ncommands=['north']\nroom='workshop'\n[[scenario]]\nname='fresh'\ncommands=['look']\nroom='foyer'\nflags=[]\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .args(["test", "story_example", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("FAIL: wrong room"));
    assert!(text.contains("PASS: fresh"));
}

#[test]
fn suite_rejects_unknown_fields_multiline_input_and_save_access_before_execution() {
    let f = Scratch::new();
    let file = f.0.join("cases.toml");
    for body in [
        "format_version=1\n[[scenario]]\nname='a'\ncommands=['SAVE forbidden']",
        "format_version=1\n[[scenario]]\nname='a'\ncommands=[\"look\\nsave forbidden\"]",
        "format_version=1\n[[scenario]]\nname='a'\ncommands=['look']\nunknown=true",
        "format_version=1\nseed=0\n[[scenario]]\nname='a'\ncommands=['look']",
    ] {
        fs::write(&file, body).unwrap();
        let mut out = Vec::new();
        assert!(
            lupine::scenario::run(std::path::Path::new("story_example"), &file, &mut out).is_err()
        );
        assert!(out.is_empty());
    }
}
