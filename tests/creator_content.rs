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
