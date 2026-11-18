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
