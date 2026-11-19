use lupine::{adventure::Adventure, dialogue, state::GameState, state::ItemLocation};
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
            "lupine-dialogue-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        lupine::creator::new_game(&root, "events_test", "Events Test").unwrap();
        let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/dialogue_demo");
        for file in [
            "game.toml",
            "rooms/start.toml",
            "rooms/hall.toml",
            "items/token.toml",
            "rules.toml",
            "NPCs/guide.toml",
            "config/character.fig",
        ] {
            fs::copy(sample.join(file), root.join(file)).unwrap();
        }
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
fn shipped_scenarios_cover_choices_transfers_and_movement() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/dialogue_demo");
    let mut out = Vec::new();
    lupine::scenario::run(&root, &root.join("scenarios.toml"), &mut out).unwrap();
    assert_eq!(String::from_utf8_lossy(&out).matches("PASS:").count(), 3);
}
#[test]
fn opening_cancel_eof_and_missing_npc_never_mutate() {
    let f = Fixture::new();
    for script in [
        "people\ntalk helper\ncancel\n",
        "talk guide\n",
        "talk missing\n",
    ] {
        let (s, t) = f.play(script);
        let a = Adventure::load(&f.0).unwrap();
        assert_eq!(s, GameState::new(&a.game.world), "{t}");
        assert!(!f.0.join("saves").exists());
    }
}
#[test]
fn conditional_responses_reject_unlisted_ids_and_numeric_choice_is_explicit() {
    let f = Fixture::new();
    let (s, t) = f.play("talk guide\nreceive\n1\ntalk guide\n1\n");
    assert_eq!(s.events.turn, 2, "{t}");
    assert_eq!(s.carried_quantity("token"), 2);
    assert_eq!(s.dialogue.npcs["guide"].node, "return");
    assert!(t.contains("Choose a listed number or ID"));
    assert!(t.contains("We have already been introduced."));
}
#[test]
fn ambiguous_npcs_use_read_only_selection_and_hidden_npcs_are_unavailable() {
    let f = Fixture::new();
    let p = f.0.join("NPCs/guide.toml");
    let s = fs::read_to_string(&p).unwrap();
    fs::write(
        f.0.join("NPCs/other.toml"),
        s.replace("id=\"guide\"", "id=\"other\"")
            .replace("npc=\"guide\"", "npc=\"other\""),
    )
    .unwrap();
    let (s, t) = f.play("talk guide\nintroduce\ntalk helper\ncancel\n");
    assert_eq!(s.events.turn, 1, "{t}");
    assert!(t.contains("Which helper"));
    let a = Adventure::load(&f.0).unwrap();
    let mut w = a.game.world;
    w.dialogue.npcs.get_mut("other").unwrap().requires =
        vec![lupine::rules::Condition::FlagUnset("introduced".into())];
    assert_eq!(dialogue::candidates(&w, &s).len(), 1);
}
#[test]
fn failed_transfer_rolls_back_node_effects_and_output() {
    let f = Fixture::new();
    let p = f.0.join("NPCs/guide.toml");
    let t = fs::read_to_string(&p)
        .unwrap()
        .replace("quantity=2}", "quantity=99}");
    fs::write(p, t).unwrap();
    let (s, t) = f.play("talk guide\nintroduce\ntalk guide\nreceive\n");
    assert_eq!(s.events.turn, 1, "{t}");
    assert_eq!(s.dialogue.npcs["guide"].node, "hello");
    assert_eq!(s.carried_quantity("token"), 0);
    assert_eq!(s.count_at("token", &ItemLocation::Npc("guide".into())), 3);
    assert!(!t.contains("Two tokens received."));
}
#[test]
fn save_load_restores_npc_nodes_positions_and_owned_quantities() {
    let f = Fixture::new();
    let (s,t)=f.play("talk guide\nintroduce\ntalk guide\nreceive\nsave before\ntalk guide\nmeet\nload before\nyes\n");
    assert!(t.contains("Loaded: before"), "{t}");
    assert_eq!(s.events.turn, 2);
    assert_eq!(s.dialogue.npcs["guide"].node, "return");
    assert_eq!(s.dialogue.npcs["guide"].room.as_deref(), Some("start"));
    assert_eq!(s.carried_quantity("token"), 2);
    let a = Adventure::load(&f.0).unwrap();
    let encoded = lupine::save::encode_save(&a.game.world, &s, &a.game.id, "roundtrip").unwrap();
    assert_eq!(
        lupine::save::decode_save(&a.game.world, &a.game.id, &encoded).unwrap(),
        s
    );
    for corrupt in [
        encoded.replace("node = \"return\"", "node = \"missing\""),
        encoded.replace("[dialogue.npcs.guide]", "[dialogue.npcs.stranger]"),
    ] {
        assert!(lupine::save::decode_save(&a.game.world, &a.game.id, &corrupt).is_err());
    }
}
#[test]
fn invalid_definitions_locations_and_choice_targets_fail_validation() {
    let f = Fixture::new();
    let p = f.0.join("NPCs/guide.toml");
    let t = fs::read_to_string(&p).unwrap();
    for bad in [
        t.replace("next=\"return\"", "next=\"missing\""),
        t.replace("npc=\"guide\"", "npc=\"missing\""),
        t.replace("room=\"start\"", "room=\"missing\""),
        t.replace("id=\"receive\"", "id=\"introduce\""),
        t.replace(
            "text=\"Choose a demonstration.\"",
            "text=\"Choose a demonstration.\"\nunsupported=true",
        ),
    ] {
        fs::write(&p, bad).unwrap();
        assert!(Adventure::load(&f.0).is_err());
    }
    fs::write(&p, t).unwrap();
    let p = f.0.join("items/token.toml");
    fs::write(
        &p,
        fs::read_to_string(&p)
            .unwrap()
            .replace("id='guide'", "id='missing'"),
    )
    .unwrap();
    assert!(Adventure::load(&f.0).is_err());
}
#[test]
fn shared_npc_effects_support_offstage_and_atomic_transition() {
    use lupine::rules::{self, Condition, Effect};
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    rules::apply(
        w,
        &mut s,
        &[
            Effect::MoveNpc {
                npc: "guide".into(),
                room: None,
            },
            Effect::SetDialogue {
                npc: "guide".into(),
                node: "done".into(),
            },
        ],
    )
    .unwrap();
    assert!(dialogue::candidates(w, &s).is_empty());
    assert!(Condition::DialogueNode {
        npc: "guide".into(),
        node: "done".into()
    }
    .met(&s));
    assert!(!Condition::NpcInRoom {
        npc: "guide".into(),
        room: "start".into()
    }
    .met(&s));
    let encoded = lupine::save::encode_save(w, &s, &a.game.id, "offstage").unwrap();
    assert_eq!(
        lupine::save::decode_save(w, &a.game.id, &encoded).unwrap(),
        s
    );
}
#[test]
fn npc_and_scoped_node_migrations_preserve_typed_refs_and_save_bytes() {
    let f = Fixture::new();
    fs::write(f.0.join("personal.rork"), "untouched").unwrap();
    fs::write(f.0.join("NPCs/other.toml"), "id='other'\nname='Other'\ndescription='Another NPC'\nroom='start'\nstart='hello'\n[nodes.hello]\ntext='Other greeting'\nchoices=[{id='stay',label='Stay',next='hello',requires=[{dialogue_node={npc='guide',node='hello'}}],effects=[{set_dialogue={npc='guide',node='hello'}}]}]\n").unwrap();
    fs::write(f.0.join("events.toml"),"[[events]]\nid='check'\non='turn'\nrequires=[{dialogue_node={npc='guide',node='hello'}}]\neffects=[{set_dialogue={npc='guide',node='hello'}}]\n").unwrap();
    let mut out = Vec::new();
    lupine::creator::session::open(
        &f.0,
        &mut Cursor::new("M\nnodes:guide\nhello\nintro\nyes\nM\nnpcs\nguide\nhelper\nyes\n0\n"),
        &mut out,
        &f.0.join("unused"),
    )
    .unwrap();
    let t = String::from_utf8(out).unwrap();
    assert_eq!(t.matches("Migration saved.").count(), 2, "{t}");
    let a = Adventure::load(&f.0).unwrap();
    assert_eq!(a.game.world.dialogue.npcs["helper"].start, "intro");
    let other = &a.game.world.dialogue.npcs["other"];
    assert_eq!(other.start, "hello");
    assert_eq!(
        other.nodes["hello"].choices[0].next.as_deref(),
        Some("hello")
    );
    let other_file = fs::read_to_string(f.0.join("NPCs/other.toml")).unwrap();
    assert!(other_file.contains("helper") && other_file.contains("intro"));
    let (s, t) = f.play("talk helper\nintroduce\ntalk helper\nreceive\n");
    assert_eq!(s.carried_quantity("token"), 2, "{t}");
    assert_eq!(
        fs::read_to_string(f.0.join("personal.rork")).unwrap(),
        "untouched"
    );
}

#[test]
fn authored_effect_batches_accept_the_documented_limit_and_reject_excess() {
    let effect = "{set_flag='introduced'}";
    for (count, valid) in [(256, true), (257, false)] {
        let text = format!(
            "id='test'\nlabel='Test'\neffects=[{}]",
            vec![effect; count].join(",")
        );
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(
            lupine::rules::decode::<dialogue::Choice>(&value).is_ok(),
            valid
        );
    }
}

#[test]
fn filename_changes_preserve_ids_and_existing_save_bytes() {
    let f = Fixture::new();
    let (state, _) = f.play("talk guide\nintroduce\ntalk guide\nreceive\n");
    let before = Adventure::load(&f.0).unwrap();
    let saved =
        lupine::save::encode_save(&before.game.world, &state, &before.game.id, "same").unwrap();
    fs::write(f.0.join("existing.rork"), &saved).unwrap();
    fs::rename(
        f.0.join("NPCs/guide.toml"),
        f.0.join("NPCs/a_friendly_person.toml"),
    )
    .unwrap();
    let after = Adventure::load(&f.0).unwrap();
    let restored = lupine::save::decode_save(&after.game.world, &after.game.id, &saved).unwrap();
    assert_eq!(restored, state);
    assert_eq!(
        lupine::save::encode_save(&after.game.world, &restored, &after.game.id, "same").unwrap(),
        saved
    );
    assert_eq!(
        fs::read_to_string(f.0.join("existing.rork")).unwrap(),
        saved
    );
}

#[test]
fn npc_files_report_bad_ids_duplicate_origins_and_unknown_fields() {
    let f = Fixture::new();
    let file = f.0.join("NPCs/guide.toml");
    let original = fs::read_to_string(&file).unwrap();
    for bad in [
        original.replacen("id=\"guide\"", "", 1),
        original.replacen("id=\"guide\"", "id='NOT VALID'", 1),
        original.replacen("id=\"guide\"", "id=123", 1),
        format!("unsupported=true\n{original}"),
        "not TOML".into(),
    ] {
        fs::write(&file, bad).unwrap();
        let error = dialogue::Content::load(&f.0).unwrap_err();
        assert!(error.contains("guide.toml"), "{error}");
    }
    fs::write(&file, &original).unwrap();
    fs::write(f.0.join("NPCs/duplicate.toml"), original).unwrap();
    let error = dialogue::Content::load(&f.0).unwrap_err();
    assert!(
        error.contains("duplicate NPC id 'guide'")
            && error.contains("guide.toml")
            && error.contains("duplicate.toml"),
        "{error}"
    );
}

#[test]
fn absent_empty_nested_wrong_case_and_old_layout_are_explicit() {
    let f = Fixture::new();
    fs::remove_file(f.0.join("NPCs/guide.toml")).unwrap();
    assert!(dialogue::Content::load(&f.0).unwrap().npcs.is_empty());
    fs::remove_dir(f.0.join("NPCs")).unwrap();
    assert!(dialogue::Content::load(&f.0).unwrap().npcs.is_empty());
    fs::create_dir(f.0.join("npcs")).unwrap();
    assert!(dialogue::Content::load(&f.0)
        .unwrap_err()
        .contains("exactly NPCs/"));
    fs::remove_dir(f.0.join("npcs")).unwrap();
    fs::create_dir_all(f.0.join("NPCs/nested")).unwrap();
    assert!(dialogue::Content::load(&f.0)
        .unwrap_err()
        .contains("subdirectories"));
    fs::remove_dir(f.0.join("NPCs/nested")).unwrap();
    fs::write(f.0.join("dialogue.toml"), "[npcs]").unwrap();
    assert!(dialogue::Content::load(&f.0)
        .unwrap_err()
        .contains("Move each [npcs.ID]"));
    assert_eq!(
        fs::read_to_string(f.0.join("dialogue.toml")).unwrap(),
        "[npcs]"
    );
}

#[cfg(unix)]
#[test]
fn npc_file_and_directory_symlinks_are_rejected() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let path = f.0.join("NPCs/guide.toml");
    fs::rename(&path, f.0.join("definition.toml")).unwrap();
    symlink(f.0.join("definition.toml"), &path).unwrap();
    assert!(dialogue::Content::load(&f.0).unwrap_err().contains("links"));
    fs::remove_file(path).unwrap();
    fs::remove_dir(f.0.join("NPCs")).unwrap();
    fs::create_dir(f.0.join("elsewhere")).unwrap();
    symlink(f.0.join("elsewhere"), f.0.join("NPCs")).unwrap();
    assert!(dialogue::Content::load(&f.0)
        .unwrap_err()
        .contains("not a link"));
}
