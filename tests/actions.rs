use lupine::{adventure::Adventure, state::GameState};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "lupine-actions-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        lupine::creator::new_game(&p, "actions_test", "Actions Test").unwrap();
        fs::write(
            p.join("rules.toml"),
            "[flags]\nopen=false\n[counters]\npulls=0\n",
        )
        .unwrap();
        for (id, name, portable) in [
            ("lever", "brass lever", false),
            ("door", "door", false),
            ("key", "key", true),
        ] {
            fs::write(p.join(format!("items/{id}.toml")),format!("id='{id}'\nname='{name}'\ndescription='Fixture'\nportable={portable}\nstarting_room='start'\n")).unwrap();
        }
        fs::write(
            p.join("actions.toml"),
            r#"
[[actions]]
id="pull_lever"
commands=["pull {target}","tug {target}","use {target}"]
target="lever"
effects=[{add_counter={id="pulls",amount=1}}]
success_text="Lever pulled."
failure_text="Cannot pull."
[[actions]]
id="unlock_door"
commands=["unlock {target} with {item}"]
target="door"
item="key"
requires=[{not={flag_set="open"}}]
effects=[{set_flag="open"}]
success_text="Door unlocked."
failure_text="Already unlocked."
"#,
        )
        .unwrap();
        Self(p)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn play(path: &Path, commands: &str) -> (GameState, String) {
    let a = Adventure::load(path).unwrap();
    let mut output = Vec::new();
    let state = lupine::runner::run_session_with_io(
        a.game.world,
        &a.game.title,
        &a.game.id,
        &path.join("unused_saves"),
        a.game.save_compatibility_version,
        a.game.config,
        a.rpg,
        a.encounters,
        a.wallet,
        a.shops,
        Cursor::new(commands),
        &mut output,
        Some(1),
    )
    .unwrap();
    (state, String::from_utf8(output).unwrap())
}
#[test]
fn authored_aliases_fixed_use_and_carried_tools_work_through_player_path() {
    let f = Fixture::new();
    let (s,text)=play(&f.0,"tug the brass lever\nuse lever\nunlock door with key\ntake key\nunlock door with key\nunlock door with key\nhelp\n");
    assert_eq!(s.story.counters["pulls"], 2, "{text}");
    assert!(s.flags.contains("open"), "{text}");
    assert!(text.contains("must be carrying"));
    assert!(text.contains("Already unlocked."));
    assert!(text.contains("Actions here:"));
    assert!(!text.contains("Combat:"));
    assert!(!f.0.join("unused_saves").exists());
}
#[test]
fn ambiguous_targets_cancel_without_mutation_then_accept_exact_id() {
    let f = Fixture::new();
    fs::write(f.0.join("items/other.toml"),"id='other'\nname='iron lever'\ndescription='Other'\nportable=false\nstarting_room='start'\n").unwrap();
    let (s, text) = play(&f.0, "pull lever\ncancel\n");
    // Exact ID wins, so use a shared non-ID alias to exercise clarification.
    assert_eq!(s.story.counters["pulls"], 1, "{text}");
    for id in ["lever", "other"] {
        let p = f.0.join(format!("items/{id}.toml"));
        fs::write(
            &p,
            format!("{}\naliases=['handle']\n", fs::read_to_string(&p).unwrap()),
        )
        .unwrap();
    }
    let (s, text) = play(&f.0, "pull handle\ncancel\n");
    assert_eq!(s.story.counters["pulls"], 0, "{text}");
    assert!(text.contains("Which handle"));
    let (s, text) = play(&f.0, "pull handle\nlever\n");
    assert_eq!(s.story.counters["pulls"], 1, "{text}");
}
#[test]
fn failed_effects_inaccessible_targets_and_suggestions_do_not_change_state() {
    let f = Fixture::new();
    let (s, text) = play(&f.0, "pul brass lever\npull missing\n");
    assert_eq!(s.story.counters["pulls"], 0);
    assert!(text.contains("Did you mean"), "{text}");
    let p = f.0.join("rules.toml");
    fs::write(
        p,
        format!("[flags]\nopen=false\n[counters]\npulls={}\n", i64::MAX),
    )
    .unwrap();
    let p = f.0.join("actions.toml");
    let original = fs::read_to_string(&p).unwrap();
    fs::write(
        p,
        original.replace(
            "effects=[{add_counter",
            "effects=[{set_flag='open'}, {show_text='DO NOT PUBLISH'}, {add_counter",
        ),
    )
    .unwrap();
    let (s, text) = play(&f.0, "pull lever\n");
    assert_eq!(s.story.counters["pulls"], i64::MAX);
    assert!(!s.flags.contains("open"));
    assert!(!text.contains("DO NOT PUBLISH"));
    assert!(text.contains("overflow"));
}
#[test]
fn invalid_action_patterns_references_and_conflicts_fail_before_play() {
    let f = Fixture::new();
    let p = f.0.join("actions.toml");
    let original = fs::read_to_string(&p).unwrap();
    for invalid in [
        original.replace("pull {target}", "save {target}"),
        original.replace("pull {target}", "pull {wrong}"),
        original.replace("target=\"lever\"", "target=\"missing\""),
        original.replace("pull {target}", "pull {target} {item}"),
        format!(
            "{original}\n{}",
            original
                .split("[[actions]]")
                .nth(1)
                .map(|a| format!("[[actions]]{a}"))
                .unwrap()
        ),
    ] {
        fs::write(&p, invalid).unwrap();
        assert!(Adventure::load(&f.0).is_err());
    }
}
#[test]
fn migration_updates_action_bindings_and_nested_rules_but_preserves_prose() {
    let f = Fixture::new();
    let mut output = Vec::new();
    lupine::creator::session::open(
        &f.0,
        &mut Cursor::new(
            "M\nitems\nlever\nhandle\nyes\nM\nactions\npull_lever\npull_handle\nyes\n0\n",
        ),
        &mut output,
        &f.0.join("unused"),
    )
    .unwrap();
    let text = String::from_utf8(output).unwrap();
    assert_eq!(text.matches("Migration saved.").count(), 2, "{text}");
    let a = Adventure::load(&f.0).unwrap();
    assert_eq!(a.game.world.actions[0].target.as_deref(), Some("handle"));
    assert_eq!(a.game.world.actions[0].id, "pull_handle");
    assert_eq!(a.game.world.actions[0].success_text, "Lever pulled.");
    let (s, text) = play(&f.0, "pull handle\n");
    assert_eq!(s.story.counters["pulls"], 1, "{text}");
}

#[test]
fn ordinary_use_keeps_priority_and_fixed_sources_can_power_interactions() {
    let f = Fixture::new();
    fs::write(
        f.0.join("interactions.toml"),
        r#"
[[interactions]]
item="lever"
target="door"
effects=[{set_flag="open"}]
success_text="Linked mechanism."
failure_text="Blocked."
"#,
    )
    .unwrap();
    let (s, text) = play(&f.0, "use lever on door\n");
    assert!(s.flags.contains("open"), "{text}");
    assert_eq!(s.story.counters["pulls"], 0);
    assert!(text.contains("Linked mechanism."));
}

#[test]
fn enemy_examination_is_read_only_and_preserves_private_hp() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_persistent");
    let (before, _) = play(&path, "south\n");
    let (after, text) = play(&path, "south\nexamine goblin\n");
    assert_eq!(before, after);
    let a = Adventure::load(&path).unwrap();
    assert!(
        text.contains(&a.encounters.enemies["goblin"].description),
        "{text}"
    );
    assert!(!text.contains("HP:"));
}

#[test]
fn action_movement_runs_encounter_transition_and_records_visited_room() {
    let f = Fixture::new();
    fs::write(
        f.0.join("rooms/other.toml"),
        "id='other'\ntitle='Other room'\ndescription='Destination'\n",
    )
    .unwrap();
    let p = f.0.join("actions.toml");
    fs::write(
        &p,
        fs::read_to_string(&p).unwrap()
            + r#"
[[actions]]
id="enter_portal"
room="start"
commands=["enter portal"]
effects=[{move_player="other"}]
success_text="Travel."
failure_text="Blocked."
"#,
    )
    .unwrap();
    let (s, text) = play(&f.0, "enter portal\nhelp\n");
    assert_eq!(s.current_room, "other", "{text}");
    assert!(s.story.visited.contains("other"));
    assert!(text.contains("Destination"));
    assert!(!text.contains("Actions here:"));
}
