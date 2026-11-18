use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lupine::config::GameConfig;
use lupine::creator::{add_art, add_item, add_room, new_game};
use lupine::rpg_content::RpgContent;
use lupine::runner::run_configured_with_io;
use lupine::save::encode_save;
use lupine::state::GameState;
use lupine::story_loader::load_game;

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "lupine_fig_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        new_game(&path, "fig_test", "FIG test").unwrap();
        Self(path)
    }
    fn write(&self, name: &str, value: &str) {
        fs::create_dir_all(self.0.join("config")).unwrap();
        fs::write(self.0.join("config").join(name), value).unwrap();
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn no_config_is_narrative_and_empty_character_enables_original_defaults() {
    let project = Project::new();
    assert!(!project.0.join("config").exists());
    let game = load_game(&project.0).unwrap();
    assert!(game.config.character.is_none());
    assert_eq!(game.config.terminology.display.spellbook, "spellbook");
    assert_eq!(game.config.terminology.display.spells, "abilities");
    project.write("character.fig", "");
    let config = load_game(&project.0).unwrap().config.character.unwrap();
    assert_eq!((config.starting_level, config.max_level), (1, 0));
    assert_eq!(
        (config.starting_hp, config.starting_mp, config.starting_ap),
        (100, 30, 100)
    );
    assert_eq!(
        (
            config.hp_per_level,
            config.mp_per_level,
            config.ap_per_level
        ),
        (10, 5, 10)
    );
    assert_eq!((config.base_requirement, config.growth_percent), (100, 120));
    assert_eq!(config.next_requirement(3), Some(144));
}

#[test]
fn all_character_fields_keep_their_existing_effects() {
    let project = Project::new();
    project.write("character.fig", "starting_level=2\nmax_level=3\nstarting_hp=40\nstarting_mp=12\nstarting_ap=80\nhp_per_level=4\nmp_per_level=2\nap_per_level=5\nbase_requirement=20\ngrowth_percent=150\n");
    let progression = load_game(&project.0).unwrap().config.character.unwrap();
    let mut player = progression.create_character().unwrap();
    assert_eq!(
        (player.level, player.hp, player.mp, player.ap),
        (2, 44, 14, 85)
    );
    assert_eq!(progression.next_requirement(2), Some(30));
    player.hp = 1;
    player.mp = 0;
    assert_eq!(progression.award_xp(&mut player, 30), 1);
    assert_eq!(
        (player.level, player.hp, player.mp, player.ap),
        (3, 48, 16, 90)
    );
    assert_eq!(progression.next_requirement(3), None);
}

#[test]
fn invalid_settings_report_the_file_and_setting() {
    let cases = [
        ("character.fig", "starting_level=0", "starting_level"),
        ("character.fig", "growth_percent=99", "growth_percent"),
        ("character.fig", "starting_hp=-1", "starting_hp"),
        ("character.fig", "starting_mp='lots'", "starting_mp"),
        ("character.fig", "surprise=1", "surprise"),
        ("terminology.fig", "[display]\nspells=1", "spells"),
        ("terminology.fig", "[display]\nspells=' '", "display.spells"),
        (
            "terminology.fig",
            "[display]\nspellbook=\"bad\\u001b\"",
            "display.spellbook",
        ),
        ("terminology.fig", "[display]\nspellbok='Book'", "spellbok"),
        ("terminology.fig", "[unknown]\nspells='Book'", "unknown"),
        ("terminology.fig", "[display]\nspells=[", "spells"),
    ];
    for (file, source, setting) in cases {
        let project = Project::new();
        project.write(file, source);
        let error = load_game(&project.0).unwrap_err();
        assert!(error.contains(file) && error.contains(setting), "{error}");
    }
    let project = Project::new();
    project.write(
        "terminology.fig",
        &format!("[display]\nspells='{}'", "x".repeat(81)),
    );
    assert!(load_game(&project.0)
        .unwrap_err()
        .contains("display.spells"));
}

#[test]
fn obsolete_unknown_and_unreadable_configuration_are_not_silently_ignored() {
    let project = Project::new();
    fs::write(project.0.join("character.toml"), "starting_level=2").unwrap();
    project.write("character.fig", "");
    let error = load_game(&project.0).unwrap_err();
    assert!(error.contains("character.toml") && error.contains("config/character.fig"));
    let unknown = Project::new();
    unknown.write("charcter.fig", "");
    assert!(load_game(&unknown.0).unwrap_err().contains("charcter.fig"));
    let unreadable = Project::new();
    fs::create_dir_all(unreadable.0.join("config/terminology.fig")).unwrap();
    assert!(load_game(&unreadable.0)
        .unwrap_err()
        .contains("terminology.fig"));
    let bad_directory = Project::new();
    fs::write(bad_directory.0.join("config"), "not a directory").unwrap();
    assert!(load_game(&bad_directory.0).unwrap_err().contains("config"));
}

#[test]
fn configuration_is_per_game_and_partial_terminology_uses_defaults() {
    let first = Project::new();
    first.write("terminology.fig", "[display]\nspellbook='Ability Database'");
    let first_config = GameConfig::load(&first.0).unwrap();
    let second = Project::new();
    second.write("terminology.fig", "");
    let second_config = GameConfig::load(&second.0).unwrap();
    assert_eq!(
        first_config.terminology.display.spellbook,
        "Ability Database"
    );
    assert_eq!(first_config.terminology.display.spells, "abilities");
    assert_eq!(second_config.terminology.display.spellbook, "spellbook");
    assert!(first_config.character.is_none());
    assert!(second_config.character.is_none());
    add_room(&first.0, "tower").unwrap();
    add_item(&first.0, "key").unwrap();
    add_art(&first.0, "symbol", false).unwrap();
    load_game(&first.0).unwrap();
}

#[test]
fn terminology_reaches_runner_without_changing_commands_or_saves() {
    let project = Project::new();
    project.write(
        "terminology.fig",
        "[display]\nspellbook='Ability Database'\nspells='Programs'",
    );
    let terms = GameConfig::load(&project.0).unwrap().terminology;
    let root = std::path::Path::new("samples/rpg_spellbook");
    let mut game = load_game(root).unwrap();
    let content = RpgContent::load(root, &game.world, true).unwrap();
    let mut state = GameState::new(&game.world);
    state.character = Some(
        game.config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    let before = encode_save(&game.world, &state, &game.id, "Test").unwrap();
    game.config.terminology = terms;
    assert_eq!(
        before,
        encode_save(&game.world, &state, &game.id, "Test").unwrap()
    );
    let save_root = project.0.join("saves");
    let mut output = Vec::new();
    run_configured_with_io(
        game.world, &game.title, &game.id, &save_root, game.save_compatibility_version,
        game.config, content, Default::default(), Default::default(), Default::default(),
        Cursor::new("spellbook\nspells\ntake fireball scroll\nlearn fireball scroll\nspellbook\nspells\nhelp\nquit\nyes\n"),
        &mut output,
    ).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert_eq!(output.matches("Your Ability Database is empty.").count(), 2);
    assert_eq!(output.matches("Known Programs:").count(), 2);
    assert!(output.contains("Fireball (10 MP)"), "{output}");
    assert!(output.contains(
        "View your Ability Database with 'spellbook' or 'spells'; lists known Programs."
    ));
    assert!(
        !save_root.exists(),
        "configuration and gameplay must not autosave"
    );
}

#[test]
fn cli_play_validate_and_creator_commands_share_fig_loading() {
    use std::process::{Command, Stdio};
    let project = Project::new();
    project.write("terminology.fig", "[display]\nspellbook='Field Guide'");
    let executable = env!("CARGO_BIN_EXE_lupine");
    let validation = Command::new(executable)
        .arg("validate")
        .arg(&project.0)
        .output()
        .unwrap();
    assert!(validation.status.success(), "{:?}", validation);
    let mut child = Command::new(executable)
        .arg("play")
        .arg(&project.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    std::io::Write::write_all(&mut child.stdin.take().unwrap(), b"spellbook\nquit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Your Field Guide is empty."));
    project.write("terminology.fig", "[display]\nspells=42");
    let invalid = Command::new(executable)
        .arg("validate")
        .arg(&project.0)
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    let error = String::from_utf8(invalid.stderr).unwrap();
    assert!(
        error.contains("terminology.fig") && error.contains("spells"),
        "{error}"
    );
    assert!(add_room(&project.0, "bad").is_err());
    assert!(!project.0.join("rooms/bad.toml").exists());
}
