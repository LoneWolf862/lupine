use lupine::{
    adventure::Adventure,
    events::{self, Step, Trigger},
    state::GameState,
};
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine-events-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        lupine::creator::new_game(&root, "events_test", "Events Test").unwrap();
        fs::write(
            root.join("rooms/start.toml"),
            "id='start'\ntitle='Start'\ndescription='Start'\n[exits]\nnorth='hall'\n",
        )
        .unwrap();
        fs::write(
            root.join("rooms/hall.toml"),
            "id='hall'\ntitle='Hall'\ndescription='Hall'\n[exits]\nsouth='start'\n",
        )
        .unwrap();
        fs::write(
            root.join("items/key.toml"),
            "id='key'\nname='key'\ndescription='Key'\nportable=true\nstarting_room='start'\n",
        )
        .unwrap();
        fs::write(
            root.join("rules.toml"),
            "[counters]\npings=0\ntaken=0\narrivals=0\n",
        )
        .unwrap();
        fs::write(
            root.join("actions.toml"),
            r#"
[[actions]]
id="signal"
commands=["signal"]
success_text="Signal sent."
failure_text="No signal."
[[actions]]
id="reset"
commands=["reset alarm"]
effects=[{start_timer="alarm"}]
success_text="Reset."
failure_text="No reset."
[[actions]]
id="cancel_alarm"
commands=["stop alarm"]
effects=[{cancel_timer="alarm"}]
success_text="Canceled."
failure_text="Cannot cancel."
[[actions]]
id="finish"
commands=["finish"]
effects=[{set_quest_stage={quest="task",stage="done"}},{end_game="win"}]
success_text="Finished."
failure_text="Cannot finish."
"#,
        )
        .unwrap();
        fs::write(
            root.join("events.toml"),
            r#"
[[events]]
id="welcome"
on={enter="start"}
effects=[{set_quest_stage={quest="task",stage="active"}},{add_journal="welcome"}]
text="Welcome once."
[[events]]
id="arrival"
on={enter="hall"}
once=false
effects=[{add_counter={id="arrivals",amount=1}}]
[[events]]
id="acquired"
on={take="key"}
effects=[{add_counter={id="taken",amount=1}}]
text="Acquisition reward."
[[events]]
id="signal_received"
on={action="signal"}
once=false
effects=[{add_counter={id="pings",amount=1}}]
[[events]]
id="alarm_fired"
on={timer="alarm"}
once=false
effects=[{add_counter={id="pings",amount=10}}]
text="Alarm rings."
[timers.alarm]
turns=2
repeat=true
start=true
[quests.task]
title="Sample task"
[quests.task.stages]
active="Find the key."
done="Task completed."
[journal.welcome]
title="Welcome"
text="Read-only journal entry."
[endings.win]
kind="success"
text="A successful ending."
"#,
        )
        .unwrap();
        Self(root)
    }
    fn play(&self, input: &str) -> (GameState, String) {
        let a = Adventure::load(&self.0).unwrap();
        let mut output = Vec::new();
        let state = lupine::runner::run_session_with_io(
            a.game.world,
            &a.game.title,
            &a.game.id,
            &self.0.join("saves"),
            a.game.save_compatibility_version,
            a.game.config,
            a.rpg,
            a.encounters,
            a.wallet,
            a.shops,
            Cursor::new(input),
            &mut output,
            Some(1),
        )
        .unwrap();
        (state, String::from_utf8(output).unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn timers_advance_only_successful_actions_and_quests_journal_are_read_only() {
    let f = Fixture::new();
    let (s, text) = f.play("signal\nlook\nhelp\njournal\ntake missing\nxyzzy\ntake key\n");
    assert_eq!(s.events.turn, 2, "{text}");
    assert_eq!(s.story.counters["pings"], 11, "{text}");
    assert_eq!(s.story.counters["taken"], 1);
    assert_eq!(s.events.timers["alarm"].remaining, 2);
    assert_eq!(s.events.quests["task"], "active");
    assert_eq!(s.events.journal, vec!["welcome"]);
    assert_eq!(text.matches("Welcome once.").count(), 1);
    assert!(text.contains("Read-only journal entry."));
    assert!(!f.0.join("saves").exists());
}
#[test]
fn timer_restart_gets_a_full_delay_and_cancellation_stops_firing() {
    let f = Fixture::new();
    let (s, text) = f.play("wait\nreset alarm\nlook\n");
    assert_eq!(s.events.turn, 2, "{text}");
    assert_eq!(s.events.timers["alarm"].remaining, 2);
    assert_eq!(s.story.counters["pings"], 0);
    let (s, text) = f.play("wait\nreset alarm\nwait\nstop alarm\nwait\nwait\n");
    assert!(s.events.timers.is_empty(), "{text}");
    assert_eq!(s.story.counters["pings"], 0);
}
#[test]
fn repeat_enter_events_and_once_acquisition_survive_manual_save_restore() {
    let f = Fixture::new();
    let (s,text)=f.play("take key\nsave slot\nnorth\nsouth\nload slot\nyes\ndrop key\ntake key\nnorth\nsouth\nnorth\n");
    assert!(text.contains("Loaded: slot"), "{text}");
    assert_eq!(s.story.counters["taken"], 1);
    assert_eq!(s.story.counters["arrivals"], 2);
    assert_eq!(s.events.turn, 6);
    assert_eq!(text.matches("Welcome once.").count(), 1);
    assert_eq!(fs::read_dir(f.0.join("saves")).unwrap().count(), 1);
}
#[test]
fn endings_block_further_gameplay_but_allow_manual_save_and_journal() {
    let f = Fixture::new();
    let (s, text) = f.play("finish\nwait\ntake key\njournal\nsave ending\n");
    assert_eq!(s.events.ending.as_deref(), Some("win"), "{text}");
    assert_eq!(s.events.turn, 1);
    assert!(!s.is_carried("key"));
    assert_eq!(s.events.quests["task"], "done");
    assert!(text.contains("ADVENTURE COMPLETE"));
    assert!(text.contains("This adventure has ended"));
    assert!(text.contains("Saved:"));
    let a = Adventure::load(&f.0).unwrap();
    let encoded = lupine::save::encode_save(&a.game.world, &s, &a.game.id, "ending").unwrap();
    assert_eq!(
        lupine::save::decode_save(&a.game.world, &a.game.id, &encoded).unwrap(),
        s
    );
}
#[test]
fn failed_event_batch_rolls_back_the_entire_triggering_action() {
    let f = Fixture::new();
    fs::write(
        f.0.join("rules.toml"),
        format!("[counters]\npings=0\ntaken={}\narrivals=0\n", i64::MAX),
    )
    .unwrap();
    let (before, _) = f.play("");
    let (s, text) = f.play("take key\n");
    assert_eq!(s, before);
    assert!(!s.is_carried("key"));
    assert_eq!(s.events.turn, 0);
    assert!(!s.events.completed.contains(&"acquired".into()));
    assert_eq!(s.events.timers["alarm"].remaining, 2);
    assert!(!text.contains("Acquisition reward."));
    assert!(text.contains("all turn changes were rolled back"));
}
#[test]
fn cyclic_enter_events_are_bounded_and_startup_batch_is_atomic() {
    let f = Fixture::new();
    let p = f.0.join("events.toml");
    let mut text = fs::read_to_string(&p).unwrap();
    text.push_str("\n[[events]]\nid='cycle_a'\non={enter='start'}\nonce=false\neffects=[{move_player='hall'}]\n[[events]]\nid='cycle_b'\non={enter='hall'}\nonce=false\neffects=[{move_player='start'}]\n");
    fs::write(p, text).unwrap();
    let a = Adventure::load(&f.0).unwrap();
    let mut s = GameState::new(&a.game.world);
    let before = s.clone();
    let error = events::start(&a.game.world, &mut s).unwrap_err();
    assert!(error.contains("128 firings"), "{error}");
    assert_eq!(s, before);
}
#[test]
fn invalid_event_references_and_saved_progress_fail_validation() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let original = GameState::new(w);
    let mut variants = Vec::new();
    let mut s = original.clone();
    s.events.timers.get_mut("alarm").unwrap().remaining = 0;
    variants.push(s);
    let mut s = original.clone();
    s.events.completed = vec!["welcome".into(), "welcome".into()];
    variants.push(s);
    let mut s = original.clone();
    s.events.quests.insert("task".into(), "typo".into());
    variants.push(s);
    let mut s = original.clone();
    s.events.ending = Some("typo".into());
    variants.push(s);
    for s in variants {
        assert!(lupine::save::encode_save(w, &s, &a.game.id, "bad").is_err());
    }
    let p = f.0.join("events.toml");
    let text = fs::read_to_string(&p).unwrap();
    for invalid in [
        text.replace("turns=2", "turns=0"),
        text.replace("on={action=\"signal\"}", "on={action=\"missing\"}"),
        text.replace("on={timer=\"alarm\"}", "on={defeat=\"missing\"}"),
        text.replace("stage=\"active\"", "stage=\"missing\""),
    ] {
        fs::write(&p, invalid).unwrap();
        assert!(Adventure::load(&f.0).is_err());
    }
}
#[test]
fn event_ids_declarations_and_references_migrate_without_touching_saves() {
    let f = Fixture::new();
    let sentinel = f.0.join("personal.rork");
    fs::write(&sentinel, "unchanged").unwrap();
    let mut out = Vec::new();
    let script="M\ntimers\nalarm\nbell\nyes\nM\nquests\ntask\ngoal\nyes\nM\njournal\nwelcome\nintro\nyes\nM\nendings\nwin\nvictory\nyes\nM\nevents\nwelcome\narrival_once\nyes\n0\n";
    lupine::creator::session::open(
        &f.0,
        &mut Cursor::new(script),
        &mut out,
        &f.0.join("unused"),
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    assert_eq!(text.matches("Migration saved.").count(), 5, "{text}");
    let (s, text) = f.play("wait\nwait\nfinish\n");
    assert_eq!(s.events.ending.as_deref(), Some("victory"), "{text}");
    assert_eq!(s.events.quests["goal"], "done");
    assert_eq!(s.events.journal, vec!["intro"]);
    assert_eq!(fs::read_to_string(sentinel).unwrap(), "unchanged");
}
#[test]
fn explicit_event_api_ticks_a_successful_state_neutral_action() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    events::start(w, &mut s).unwrap();
    let mut step = Step::new(&s, true);
    step.performed = true;
    step.triggers.push(Trigger::Action("signal".into()));
    events::finish(w, &mut s, step).unwrap();
    assert_eq!(s.events.turn, 1);
    assert_eq!(s.story.counters["pings"], 1);
}

#[test]
fn enemy_defeat_triggers_the_authored_hook_through_combat() {
    let f = Fixture::new();
    fs::write(f.0.join("config/character.fig"), "enabled=true\n").unwrap();
    fs::create_dir(f.0.join("weapons")).ok();
    fs::create_dir(f.0.join("enemies")).ok();
    fs::write(f.0.join("weapons/key.toml"), "item='key'\ndamage=999\n").unwrap();
    fs::write(f.0.join("enemies/rat.toml"),"id='rat'\nname='Rat'\ndescription='Fixture opponent'\nlevel=1\nmax_hp=1\nattacks=[{name='Bite',damage=1,weight=1}]\n").unwrap();
    let p = f.0.join("rooms/hall.toml");
    fs::write(&p,fs::read_to_string(&p).unwrap()+"\n[encounters]\nchance=100\nmin_enemies=1\nmax_enemies=1\nenemies=[{enemy='rat',weight=1}]\n").unwrap();
    let p = f.0.join("events.toml");
    fs::write(&p,fs::read_to_string(&p).unwrap()+"\n[[events]]\nid='defeated_rat'\non={defeat='rat'}\neffects=[{set_counter={id='taken',value=99}}]\ntext='Defeat hook.'\n").unwrap();
    let (s, text) = f.play("take key\nnorth\nattack rat with key\n");
    assert_eq!(s.story.counters["taken"], 99, "{text}");
    assert_eq!(s.events.turn, 3);
    assert!(s.active_enemies.is_empty());
    assert!(text.contains("Defeat hook."));
}

#[test]
fn successful_no_change_interactions_spend_turns_but_canceled_clarification_does_not() {
    let f = Fixture::new();
    for id in ["door_a", "door_b"] {
        fs::write(f.0.join(format!("items/{id}.toml")),format!("id='{id}'\nname='{id}'\naliases=['door']\ndescription='Door'\nportable=false\nstarting_room='start'\n")).unwrap();
    }
    fs::write(f.0.join("interactions.toml"),"[[interactions]]\nitem='key'\ntarget='door_a'\nsuccess_text='No change, successful action.'\nfailure_text='Cannot.'\n").unwrap();
    let (s, text) = f.play("take key\nuse key on door\ncancel\nuse key on door_a\n");
    assert_eq!(s.events.turn, 2, "{text}");
    assert_eq!(s.story.counters["pings"], 10);
    assert!(text.contains("Nothing changed"));
}

#[test]
fn shipped_event_scenarios_cover_success_failure_and_free_commands() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/events_demo");
    let mut out = Vec::new();
    lupine::scenario::run(&root, &root.join("scenarios.toml"), &mut out).unwrap();
    assert_eq!(String::from_utf8_lossy(&out).matches("PASS:").count(), 3);
}
