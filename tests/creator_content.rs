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
