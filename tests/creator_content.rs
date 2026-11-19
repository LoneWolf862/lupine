use lupine::creator::{new_game, session};
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
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
            "lupine-content-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let game = root.join("adventure");
        new_game(&game, "content", "Content").unwrap();
        let saves = root.join("saves");
        Self { root, game, saves }
    }
    fn run(&self, s: &str) -> String {
        let mut out = Vec::new();
        session::open(&self.game, &mut Cursor::new(s), &mut out, &self.saves).unwrap();
        String::from_utf8(out).unwrap()
    }
    fn read(&self, p: &str) -> String {
        fs::read_to_string(self.game.join(p)).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
#[test]
fn items_create_edit_aliases_preview_and_play() {
    let f = Fixture::new();
    let out=f.run("4\nn\nbrass_key\n1\n1\nBrass Key\n2\nA small key.\n3\na\nkey\na\nbrass key\n0\n6\ns\nyes\n0\n2\n1\ntake key\ninventory\nquit\nyes\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    assert!(out.contains("Validation successful"));
    assert!(out.contains("Brass Key"));
    let text = f.read("items/brass_key.toml");
    assert!(out.contains(&text));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(v["aliases"].as_array().unwrap().len(), 2);
    assert!(!f.saves.exists());
    let out = f.run("4\n1\n3\n1\nyes\n0\n4\nno\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let v: toml::Value = f.read("items/brass_key.toml").parse().unwrap();
    assert_eq!(v["portable"].as_bool(), Some(false));
    assert_eq!(v["aliases"].as_array().unwrap().len(), 1);
}
#[test]
fn item_comments_conditional_properties_crlf_and_cancel_survive() {
    let f = Fixture::new();
    let original="# heading\r\nid = 'key' # stable\r\nname = 'Key' # name\r\ndescription = 'A key'\r\naliases = ['key']\r\nportable = true\r\nstarting_room = 'start'\r\n[[descriptions]] # authored\r\nwhen = [{flag_set='lit'}]\r\ntext = 'Shining'\r\n";
    fs::write(f.game.join("items/odd_filename.toml"), original).unwrap();
    f.run("4\n1\n1\nChanged\ncancel\n0\n0\n");
    assert_eq!(f.read("items/odd_filename.toml"), original);
    let out = f.run("4\n1\n1\nNew Key\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let text = f.read("items/odd_filename.toml");
    assert!(text.contains("# name"));
    assert!(text.contains("# authored"));
    assert!(text.contains("\r\n"));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(v["id"].as_str(), Some("key"));
    assert_eq!(v["descriptions"][0]["text"].as_str(), Some("Shining"));
}
#[test]
fn item_creation_cancel_and_eof_write_nothing() {
    for script in ["4\nn\nkey\n1\ncancel\n0\n0\n", "4\nn\nkey\n1\n1\nChanged\n"] {
        let f = Fixture::new();
        f.run(script);
        assert!(!f.game.join("items/key.toml").exists());
    }
}

#[test]
fn spell_enablement_is_explicit_and_all_fields_are_editable() {
    let f = Fixture::new();
    let original = f.read("config/character.fig");
    let output = f.run("5\n0\n");
    assert!(output.contains("Spells require RPG"));
    assert_eq!(original, f.read("config/character.fig"));
    fs::write(
        f.game.join("config/character.fig"),
        original.replace("enabled = false", "enabled = true"),
    )
    .unwrap();
    f.run("4\nn\nscroll\n1\ns\nyes\n0\n0\n");
    let out = f.run("5\nn\nrenew\n1\nRenew\n2\nRestores energy.\n3\n0\n4\n3\n5\n15\n6\n\n1\n7\ns\nno\n7\ns\nyes\n0\n2\n1\ntake scroll\nlearn scroll\nspellbook\nsave trial\nload trial\nquit\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    assert!(out.contains("You learn Renew"), "{out}");
    let text = f.read("spells/renew.toml");
    assert!(out.contains(&text));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(v["kind"].as_str(), Some("restore_mana"));
    assert_eq!(v["power"].as_integer(), Some(15));
    assert_eq!(v["learn_from_item"].as_str(), Some("scroll"));
}
#[test]
fn spell_rejects_bad_numbers_and_preserves_existing_id_and_comments() {
    let f = Fixture::new();
    fs::write(f.game.join("config/character.fig"), "enabled=true\n").unwrap();
    fs::write(
        f.game.join("spells/heal.toml"),
        "# ability\nid='heal' # stable\nname='Heal'\nkind='heal'\npower=4 # amount\nmana_cost=1\n",
    )
    .unwrap();
    let out = f.run("5\n1\n5\n0\n4294967296\n8\n3\n-1\n2\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let text = f.read("spells/heal.toml");
    assert!(text.contains("# amount"));
    assert!(text.contains("# stable"));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(v["id"].as_str(), Some("heal"));
    assert_eq!(v["power"].as_integer(), Some(8));
}

#[test]
fn artwork_preserves_spaces_backslashes_and_supports_dynamic_references() {
    let f = Fixture::new();
    let out=f.run("6\nn\nview\n1\n  /\\  \n\\___/\n\n/end\n2\ns\nyes\nr\n1\n1\n1\n3\n1\n2\nopened\n0\ns\nyes\n0\n2\n1\nlook\nquit\n0\n");
    assert!(out.contains("Artwork saved and validated"), "{out}");
    assert!(out.contains("Changes saved"), "{out}");
    assert_eq!(f.read("art/view.txt"), "  /\\  \n\\___/\n\n");
    let v: toml::Value = f.read("rooms/start.toml").parse().unwrap();
    assert_eq!(v["art_file"].as_str(), Some("view.txt"));
    assert_eq!(
        v["art_variants"][0]["when"][0]["flag_unset"].as_str(),
        Some("opened")
    );
    assert!(out.contains("Validation successful"));
    assert!(!f.saves.exists());
    let out = f.run("6\nr\n1\n2\nyes\n4\n1\nyes\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let v: toml::Value = f.read("rooms/start.toml").parse().unwrap();
    assert!(v.get("art_file").is_none());
    assert!(v
        .get("art_variants")
        .is_none_or(|v| v.as_array().unwrap().is_empty()));
}
#[test]
fn artwork_nested_existing_files_and_cancel_are_safe() {
    let f = Fixture::new();
    fs::create_dir(f.game.join("art/nested")).unwrap();
    fs::write(f.game.join("art/nested/view.txt"), "old\r\n").unwrap();
    f.run("6\n1\n2\n0\n0\n0\n");
    assert_eq!(f.read("art/nested/view.txt"), "old\r\n");
    let out = f.run("6\n1\n1\n new \n/end\ns\nyes\n0\n0\n");
    assert!(out.contains("Artwork saved"), "{out}");
    assert_eq!(f.read("art/nested/view.txt"), " new \n");
    f.run("6\nn\ncanceled\n1\ntext\ncancel\n0\n0\n");
    assert!(!f.game.join("art/canceled.txt").exists());
}
#[test]
fn unreferenced_artwork_control_codes_are_rejected() {
    let f = Fixture::new();
    let out = f.run("6\nn\nunsafe\n1\n\u{1b}[31m\n/end\ns\nyes\n0\n0\n");
    assert!(out.contains("terminal control characters"), "{out}");
    assert!(!f.game.join("art/unsafe.txt").exists());
}

#[test]
fn locked_door_and_conditional_prose_are_created_without_toml_edits() {
    let f = Fixture::new();
    let out=f.run("7\nn\nHall\n\nA hall.\n1\n1\nnorth\n\n0\n6\nyes\n0\n9\n2\n2\n1\nn\nkey\nKey\nA brass key.\nn\ndoor\nDoor\nA fixed door.\n\nUnlocked.\nAlready unlocked.\nLocked.\nyes\n0\n7\n2\n7\n1\n1\n1\ndoor_open\n0\nThe doorway stands open.\n0\n0\n6\nyes\n0\n2\n1\nnorth\ntake key\nuse key on door\nlook\nnorth\nsouth\nsave door_test\nload door_test\nquit\n0\n");
    assert!(out.contains("Locked door saved and validated"), "{out}");
    assert!(out.contains("The doorway stands open."), "{out}");
    assert!(out.contains("> Locked."), "{out}");
    assert!(out.contains("Unlocked."));
    assert!(out.contains("Loaded: door_test"));
    let rules: toml::Value = f.read("interactions.toml").parse().unwrap();
    assert_eq!(
        rules["exit_rules"][0]["requires"][0]["flag_set"].as_str(),
        Some("door_open")
    );
    let door: toml::Value = f.read("items/door.toml").parse().unwrap();
    assert_eq!(door["portable"].as_bool(), Some(false));
}
#[test]
fn canceled_locked_door_publishes_none_of_its_related_files() {
    let f = Fixture::new();
    f.run("7\nn\nHall\n\nA hall.\n1\n1\nnorth\n\n0\n6\nyes\n0\n0\n");
    let old = f.read("interactions.toml");
    f.run("9\n2\n2\n1\nn\nkey\nKey\nKey.\nn\ndoor\nDoor\nDoor.\ncancel\n0\n0\n");
    assert_eq!(f.read("interactions.toml"), old);
    assert!(!f.game.join("items/key.toml").exists());
    assert!(!f.game.join("items/door.toml").exists());
}
#[test]
fn rules_edit_preserves_comments_and_existing_conditions() {
    let f = Fixture::new();
    f.run("4\nn\nkey\n1\ns\nyes\nn\ndoor\n1\ns\nyes\n0\n0\n");
    fs::write(f.game.join("interactions.toml"),"# rules\n[[interactions]] # original\nitem='key'\ntarget='door'\nrequires=[{has_item='key'}, {flag_unset='opened'}] # requirements\neffects=[{set_flag='opened'}]\nsuccess_text='Old' # success\nfailure_text='No'\n").unwrap();
    let out = f.run("9\n1\n2\n1\n5\nChanged.\n4\n2\nother\n3\n2\n1\n0\n0\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let text = f.read("interactions.toml");
    assert!(text.contains("# original"));
    assert!(text.contains("# requirements"));
    assert!(text.contains("# success"));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(
        v["interactions"][0]["requires"].as_array().unwrap().len(),
        2
    );
    assert_eq!(v["interactions"][0]["effects"].as_array().unwrap().len(), 3);
}

#[test]
fn advanced_room_encounters_preserve_existing_weights_and_conditions() {
    let f = Fixture::new();
    fs::write(f.game.join("config/character.fig"), "enabled=true\n").unwrap();
    fs::write(
        f.game.join("enemies/guard.toml"),
        "id='guard'\nname='Guard'\ndescription='A guard'\nlevel=1\nmax_hp=20\n",
    )
    .unwrap();
    let room=f.read("rooms/start.toml")+"\n[encounters] # authored encounter\nchance=40 # probability\npersistence='room'\n[[encounters.enemies]] # keep this\nenemy='guard'\nweight=3\nrequires=[{flag_unset='peace'}]\n";
    fs::write(f.game.join("rooms/start.toml"), room).unwrap();
    let out = f.run("7\n1\n7\n2\n1\n60\n4\n3\n6\n1\n2\n5\n0\n0\n0\n6\nyes\n0\n0\n");
    assert!(out.contains("Room changes saved"), "{out}");
    let text = f.read("rooms/start.toml");
    assert!(text.contains("# keep this"));
    assert!(text.contains("# probability"));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(v["encounters"]["chance"].as_integer(), Some(60));
    assert_eq!(v["encounters"]["persistence"].as_str(), Some("unique"));
    assert_eq!(
        v["encounters"]["enemies"][0]["weight"].as_integer(),
        Some(5)
    );
    assert_eq!(
        v["encounters"]["enemies"][0]["requires"][0]["flag_unset"].as_str(),
        Some("peace")
    );
}
#[test]
fn mixed_case_item_names_match_without_aliases_and_keep_ambiguity() {
    let f = Fixture::new();
    f.run("4\nn\nkey\n1\n1\nBrass Key\ns\nyes\nn\nother\n1\n1\nIron Key\ns\nyes\n0\n0\n");
    let out = f.run("1\ntake key\ntake BRASS KEY\ninventory\nquit\nyes\n0\n");
    assert!(out.contains("Which key do you mean"), "{out}");
    assert!(out.contains("You take the Brass Key"), "{out}");
}

#[test]
fn fig_editor_changes_all_character_values_and_labels_with_exact_preview() {
    let f = Fixture::new();
    let out=f.run("3\n1\n1\ntrue\n2\n2\n3\n10\n4\n80\n5\n40\n6\n110\n7\n12\n8\n6\n9\n11\n10\n200\n11\n150\np\ns\nyes\n2\n1\nGrimoire\n2\nSpells\np\ns\nyes\n0\n2\n1\nstats\nspellbook\nquit\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    assert!(out.contains("Validation successful"));
    let text = f.read("config/character.fig");
    assert!(text.contains("# HP base"));
    assert!(out.contains(&text));
    let v: toml::Value = text.parse().unwrap();
    for (key, n) in [
        ("starting_level", 2),
        ("max_level", 10),
        ("starting_hp", 80),
        ("starting_mp", 40),
        ("starting_ap", 110),
        ("hp_per_level", 12),
        ("mp_per_level", 6),
        ("ap_per_level", 11),
        ("base_requirement", 200),
        ("growth_percent", 150),
    ] {
        assert_eq!(v[key].as_integer(), Some(n));
    }
    assert!(out.contains("Grimoire"));
    assert!(!f.saves.exists());
}
#[test]
fn fig_absence_cancel_legacy_enablement_and_inline_labels_are_preserved() {
    let f = Fixture::new();
    fs::remove_file(f.game.join("config/character.fig")).unwrap();
    fs::remove_file(f.game.join("config/terminology.fig")).unwrap();
    fs::remove_dir(f.game.join("config")).unwrap();
    f.run("3\n1\n0\n2\ncancel\n0\n0\n");
    assert!(!f.game.join("config").exists());
    let out = f.run("3\n2\n1\nJournal\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    assert!(!f.game.join("config/character.fig").exists());
    fs::write(
        f.game.join("config/character.fig"),
        "# legacy\nstarting_hp=70 # keep\n",
    )
    .unwrap();
    fs::write(
        f.game.join("config/terminology.fig"),
        "# custom\ndisplay = { spellbook = 'Book', spells = 'Arts' } # keep\n",
    )
    .unwrap();
    let out = f.run("3\n1\n4\n90\ns\nyes\n2\n2\nSkills\ns\nyes\n0\n0\n");
    assert!(out.contains("true (omitted in existing file)"), "{out}");
    assert!(!f.read("config/character.fig").contains("enabled ="));
    assert!(f.read("config/character.fig").contains("# keep"));
    assert!(f.read("config/terminology.fig").contains("# custom"));
    assert!(lupine::story_loader::load_game(&f.game)
        .unwrap()
        .config
        .character
        .is_some());
}
#[test]
fn fig_invalid_cross_field_values_can_be_corrected_without_losing_draft() {
    let f = Fixture::new();
    let out = f.run("3\n1\n2\n3\n3\n2\ns\nyes\n3\n4\ns\nyes\n0\n0\n");
    assert!(out.contains("Proposed changes are invalid"), "{out}");
    assert!(out.contains("Changes saved"));
    let v: toml::Value = f.read("config/character.fig").parse().unwrap();
    assert_eq!(v["starting_level"].as_integer(), Some(3));
    assert_eq!(v["max_level"].as_integer(), Some(4));
}
#[test]
fn fig_cannot_disable_rpg_with_existing_spells() {
    let f = Fixture::new();
    f.run("3\n1\n1\ntrue\ns\nyes\n0\n5\nn\nheal\ns\nyes\n0\n0\n");
    let old = f.read("config/character.fig");
    let out = f.run("3\n1\n1\nfalse\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("require enabled = true"), "{out}");
    assert_eq!(f.read("config/character.fig"), old);
}

#[test]
fn teaching_item_cannot_be_made_nonportable_by_item_editor() {
    let f = Fixture::new();
    f.run(
        "3\n1\n1\ntrue\ns\nyes\n0\n4\nn\nscroll\n1\ns\nyes\n0\n5\nn\nheal\n6\n\n1\ns\nyes\n0\n0\n",
    );
    let before = f.read("items/scroll.toml");
    let out = f.run("4\n1\n4\nno\ns\n0\n0\n0\n");
    assert!(out.contains("Cannot make a teaching item fixed"), "{out}");
    assert_eq!(f.read("items/scroll.toml"), before);
}
#[test]
fn art_variants_can_be_revised_reordered_and_keep_unedited_conditions() {
    let f = Fixture::new();
    fs::write(f.game.join("art/a.txt"), "A\n").unwrap();
    fs::write(f.game.join("art/b.txt"), "B\n").unwrap();
    let room=f.read("rooms/start.toml")+"\n[[art_variants]] # first\nwhen=[{flag_unset='one'}]\nfile='a.txt'\n[[art_variants]] # second\nwhen=[{flag_unset='two'}]\nfile='b.txt'\n";
    fs::write(f.game.join("rooms/start.toml"), room).unwrap();
    let out = f.run("6\nr\n1\n6\n2\n2\n  TEXT  \n/end\nyes\n7\n2\ns\nyes\n0\n1\nlook\nquit\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    assert!(out.contains("  TEXT  "));
    let text = f.read("rooms/start.toml");
    assert!(text.contains("# first"));
    assert!(text.contains("# second"));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(v["art_variants"][0]["text"].as_str(), Some("  TEXT  \n"));
    assert_eq!(
        v["art_variants"][0]["when"][0]["flag_unset"].as_str(),
        Some("two")
    );
    assert_eq!(v["art_variants"][1]["file"].as_str(), Some("a.txt"));
}

#[test]
fn editing_existing_rpg_items_protects_shop_and_loot_dependencies() {
    fn copy(source: &std::path::Path, target: &std::path::Path) {
        fs::create_dir(target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let p = entry.unwrap().path();
            let dst = target.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &dst);
            } else {
                fs::copy(&p, &dst).unwrap();
            }
        }
    }
    let mut f = Fixture::new();
    f.game = f.root.join("sample");
    copy(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_transactions"),
        &f.game,
    );
    let altar = f.read("items/mana_altar.toml");
    let trophy = f.read("items/goblin_trophy.toml");
    let out = f.run("4\n4\n5\n2\ns\nyes\n0\n2\n4\nno\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("must start in the shop's room"), "{out}");
    assert!(out.contains("must be portable"), "{out}");
    assert_eq!(f.read("items/mana_altar.toml"), altar);
    assert_eq!(f.read("items/goblin_trophy.toml"), trophy);
    let spell = f.read("spells/fireball.toml");
    let weapon = f.read("weapons/rusty_dagger.toml");
    let shop = f.read("shops/healer.toml");
    let out = f.run("4\n1\n1\nFlame Scroll\ns\nyes\n0\n2\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    assert_eq!(f.read("spells/fireball.toml"), spell);
    assert_eq!(f.read("weapons/rusty_dagger.toml"), weapon);
    assert_eq!(f.read("shops/healer.toml"), shop);
}

// The reported Bronze Key workflow uses an existing key, not a newly made key.
fn bronze_fixture() -> Fixture {
    let f = Fixture::new();
    let out = f.run("7\nn\nHall\n\nA hall.\n1\nstart\neast\nyes\n0\n6\nyes\n0\n4\nn\nkey3\nstart\n1\nBronze Key\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    f
}
#[test]
fn existing_bronze_key_selection_unlocks_real_exit_by_id_number_and_name() {
    use lupine::{engine::execute_command, parser::parse_command, state::GameState};
    for selection in ["key3", "1", "Bronze Key", "\nnot_an_item\nhelp\nkey3"] {
        let f = bronze_fixture();
        let out = f.run(&format!("9\n2\nstart\neast\n{selection}\nn\ngate\nBronze Gate\nA fixed gate.\n\nUnlocked.\nAlready open.\nLocked.\nyes\n0\n0\n"));
        assert!(out.contains("Selected: Bronze Key (key3)."), "{out}");
        assert!(out.contains("DOOR CREATED"), "{out}");
        assert!(!out.contains("Enter to select an existing"));
        let game = lupine::story_loader::load_game(&f.game).unwrap();
        let mut state = GameState::new(&game.world);
        execute_command(&game.world, &mut state, parse_command("east").unwrap());
        assert_eq!(state.current_room, "start", "Exit must begin blocked");
        execute_command(
            &game.world,
            &mut state,
            parse_command("take bronze key").unwrap(),
        );
        execute_command(
            &game.world,
            &mut state,
            parse_command("use bronze key on bronze gate").unwrap(),
        );
        assert!(state.flags.contains("gate_open"));
        execute_command(&game.world, &mut state, parse_command("east").unwrap());
        assert_eq!(state.current_room, "hall");
        let before = f.read("interactions.toml");
        assert!(f.run("2\n0\n").contains("Validation successful"));
        assert_eq!(before, f.read("interactions.toml"));
        assert!(!f.saves.exists());
    }
}
#[test]
fn door_cancel_back_and_declined_save_preserve_all_files() {
    for ending in [
        "0\n",
        "back\n",
        "cancel\n",
        "key3\nn\ngate\nGate\nGate.\n\n\n\n\nno\n",
    ] {
        let f = bronze_fixture();
        let before = f.read("interactions.toml");
        let key = f.read("items/key3.toml");
        let room = f.read("rooms/start.toml");
        let out = f.run(&format!("9\n2\nstart\neast\n{ending}0\n0\n"));
        assert!(out.contains("canceled"), "{out}");
        assert!(!out.contains("DOOR CREATED"));
        assert_eq!(before, f.read("interactions.toml"));
        assert_eq!(key, f.read("items/key3.toml"));
        assert_eq!(room, f.read("rooms/start.toml"));
        assert!(!f.game.join("items/gate.toml").exists());
    }
}
#[test]
fn ambiguous_item_names_reprompt_and_exact_id_selects_the_intended_key() {
    let f = bronze_fixture();
    f.run("4\nn\nkey4\nstart\n1\nBronze Key\ns\nyes\n0\n0\n");
    let out =
        f.run("9\n2\nstart\neast\nBronze Key\nkey3\nn\ngate\nGate\nGate.\n\n\n\n\nyes\n0\n0\n");
    assert!(out.contains("ambiguous"), "{out}");
    let rules: toml::Value = f.read("interactions.toml").parse().unwrap();
    assert_eq!(rules["interactions"][0]["item"].as_str(), Some("key3"));
}
#[test]
fn shared_selectors_keep_drafts_and_accept_ids_for_teaching_and_placement() {
    let f = bronze_fixture();
    let out = f.run("4\nkey3\n1\nRevised Key\n5\n\nmissing\nhelp\nback\ns\nno\ns\nyes\n0\n3\n1\nenabled\ntrue\ns\nyes\n0\n5\nn\nrenew\n6\nkey3\ns\nyes\n0\n0\n");
    assert!(out.contains("Save declined; draft retained"), "{out}");
    let item: toml::Value = f.read("items/key3.toml").parse().unwrap();
    assert_eq!(item["name"].as_str(), Some("Revised Key"));
    assert_eq!(item["starting_room"].as_str(), Some("start"));
    let spell: toml::Value = f.read("spells/renew.toml").parse().unwrap();
    assert_eq!(spell["learn_from_item"].as_str(), Some("key3"));
}
#[test]
fn conditional_description_reorder_changes_runtime_priority_and_preserves_comments() {
    let f = Fixture::new();
    let text = f.read("rooms/start.toml") + "\n[[descriptions]] # one\nwhen=[{flag_unset='open'}]\ntext='First.'\n[[descriptions]] # two\nwhen=[{flag_unset='open'}]\ntext='Second.'\n";
    fs::write(f.game.join("rooms/start.toml"), text).unwrap();
    let out = f.run("7\nstart\n7\n1\n4\n2\n0\n0\n6\nyes\n0\n1\nlook\nquit\n0\n");
    assert!(out.contains("Room changes saved"), "{out}");
    let text = f.read("rooms/start.toml");
    assert!(text.contains("# one") && text.contains("# two"));
    let v: toml::Value = text.parse().unwrap();
    assert_eq!(v["descriptions"][0]["text"].as_str(), Some("Second."));
    assert!(out
        .split_once("Powered by Lupine")
        .unwrap()
        .1
        .contains("Second."));
}
