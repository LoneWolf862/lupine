use lupine::{
    adventure::Adventure,
    commands::Command,
    engine::{execute_command, ActionOutcome},
    rules::{self, Compare, Condition, Effect, Outcome},
    state::{GameState, ItemLocation},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
#[test]
fn shipped_rules_scenarios_pass_through_the_player_runner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rules_demo");
    let mut output = Vec::new();
    lupine::scenario::run(&root, &root.join("scenarios.toml"), &mut output).unwrap();
    assert_eq!(String::from_utf8_lossy(&output).matches("PASS:").count(), 3);
}
impl Fixture {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "lupine-rules-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        lupine::creator::new_game(&p, "rules_test", "Rules Test").unwrap();
        fs::write(
            p.join("rules.toml"),
            "[flags]\nopen=false\nblocked=false\n[counters]\nuses=0\n",
        )
        .unwrap();
        for (id, portable) in [("key", true), ("door", false)] {
            fs::write(p.join(format!("items/{id}.toml")),format!("id='{id}'\nname='{id}'\ndescription='Fixture'\nportable={portable}\nstarting_room='start'\n")).unwrap();
        }
        fs::write(
            p.join("interactions.toml"),
            r#"
[[interactions]]
item="key"
target="door"
requires=[{all=[{has_item="key"},{not={flag_set="blocked"}}]}]
effects=[]
success_text="Fallback"
failure_text="Blocked"
[[interactions.outcomes]]
when=[{counter={id="uses",op="eq",value=0}}]
effects=[{set_flag="open"},{add_counter={id="uses",amount=1}}]
text="First use"
[[interactions.outcomes]]
when=[{any=[{flag_set="open"},{in_room="start"}]}]
effects=[{add_counter={id="uses",amount=1}}]
text="Later use"
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

#[test]
fn migrations_follow_nested_rules_and_declarations_without_rewriting_saves() {
    let f = Fixture::new();
    let untouched = f.0.join("personal.rork");
    fs::write(&untouched, "unchanged evidence").unwrap();
    let mut output = Vec::new();
    lupine::creator::session::open(
        &f.0,
        &mut std::io::Cursor::new(
            "M\nflags\nopen\nopened\nyes\nM\ncounters\nuses\nattempts\nyes\n0\n",
        ),
        &mut output,
        &f.0.join("userdata"),
    )
    .unwrap();
    let log = String::from_utf8(output).unwrap();
    assert_eq!(log.matches("Migration saved.").count(), 2, "{log}");
    let a = Adventure::load(&f.0).unwrap();
    assert!(a.game.world.rules.flags.contains_key("opened"));
    assert!(a.game.world.rules.counters.contains_key("attempts"));
    let text = fs::read_to_string(f.0.join("interactions.toml")).unwrap();
    assert!(text.contains("opened"));
    assert!(text.contains("attempts"));
    assert_eq!(
        fs::read_to_string(&untouched).unwrap(),
        "unchanged evidence"
    );
}

#[test]
fn nested_toml_conditions_and_ordered_outcomes_execute_and_roundtrip() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    execute_command(w, &mut s, Command::Take("key".into()));
    let cmd = || Command::Use {
        item: "key".into(),
        target: "door".into(),
    };
    assert_eq!(
        execute_command(w, &mut s, cmd()),
        ActionOutcome::Message("First use".into())
    );
    assert_eq!(
        execute_command(w, &mut s, cmd()),
        ActionOutcome::Message("Later use".into())
    );
    assert_eq!(s.story.counters["uses"], 2);
    assert!(s.flags.contains("open"));
    let encoded = lupine::save::encode_save(w, &s, &a.game.id, "test").unwrap();
    assert_eq!(
        lupine::save::decode_save(w, &a.game.id, &encoded).unwrap(),
        s
    );
    assert!(lupine::save::decode_save(
        w,
        &a.game.id,
        &encoded.replace("save_format_version = 2", "save_format_version = 1")
    )
    .is_err());
    s.story.counters.insert("unknown".into(), 1);
    assert!(lupine::save::encode_save(w, &s, &a.game.id, "test").is_err());
}
#[test]
fn effects_are_atomic_when_counter_overflows_or_item_is_missing() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    s.story.counters.insert("uses".into(), i64::MAX);
    let before = s.clone();
    assert!(rules::apply(
        w,
        &mut s,
        &[
            Effect::SetFlag("open".into()),
            Effect::GiveItem("key".into()),
            Effect::ShowText("Never published".into()),
            Effect::AddCounter {
                id: "uses".into(),
                amount: 1
            }
        ]
    )
    .is_err());
    assert_eq!(s, before);
    assert!(rules::apply(
        w,
        &mut s,
        &[
            Effect::SetFlag("open".into()),
            Effect::GiveItem("missing".into())
        ]
    )
    .is_err());
    assert_eq!(s, before);
}
#[test]
fn invalid_nested_references_unknown_fields_and_depth_are_rejected() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    for c in [
        Condition::Any(vec![Condition::HasItem("missing".into())]),
        Condition::Counter {
            id: "typo".into(),
            op: Compare::Eq,
            value: 0,
        },
        Condition::FlagSet("opne".into()),
        Condition::All(vec![]),
    ] {
        assert!(c.validate(w).is_err());
    }
    let mut c = Condition::FlagSet("open".into());
    for _ in 0..18 {
        c = Condition::Not(Box::new(c));
    }
    assert!(c.validate(w).is_err());
    let bad: toml::Value = toml::from_str("counter={id='uses',op='eq',value=0,typo=true}").unwrap();
    assert!(rules::decode::<Condition>(&bad).is_err());
}
#[test]
fn movement_giving_removal_and_failed_conditions_preserve_expected_state() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    rules::apply(
        w,
        &mut s,
        &[
            Effect::GiveItem("key".into()),
            Effect::SetCounter {
                id: "uses".into(),
                value: 4,
            },
            Effect::MovePlayer("start".into()),
        ],
    )
    .unwrap();
    assert_eq!(s.location("key"), Some(ItemLocation::Inventory));
    assert!(Condition::Visited("start".into()).met(&s));
    s.flags.insert("blocked".into());
    let before = s.clone();
    assert_eq!(
        execute_command(
            w,
            &mut s,
            Command::Use {
                item: "key".into(),
                target: "door".into()
            }
        ),
        ActionOutcome::Message("Blocked".into())
    );
    assert_eq!(s, before);
    rules::apply(w, &mut s, &[Effect::RemoveItem("key".into())]).unwrap();
    assert_eq!(s.location("key"), Some(ItemLocation::Consumed));
}
#[test]
fn diagnostics_warn_without_rejecting_intentional_storage_and_shadowed_branches() {
    let f = Fixture::new();
    fs::write(
        f.0.join("rooms/storage.toml"),
        "id='storage'\ntitle='Storage'\ndescription='Offstage'\n",
    )
    .unwrap();
    let mut a = Adventure::load(&f.0).unwrap();
    a.game.world.interactions[0].outcomes.insert(
        0,
        Outcome {
            when: vec![],
            effects: vec![],
            text: "Always".into(),
        },
    );
    let warnings = lupine::diagnostics::warnings(&a).join("\n");
    assert!(warnings.contains("storage"));
    assert!(warnings.contains("shadowed"));
    assert!(warnings.contains("blocked"));
}
