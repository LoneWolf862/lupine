use lupine::creator::{new_game, session};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
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
            "lupine-room-ui-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let game = root.join("My room's project");
        new_game(&game, "rooms", "Room Tests").unwrap();
        let saves = root.join("saves");
        Self { root, game, saves }
    }
    fn run(&self, script: &str) -> String {
        let mut output = Vec::new();
        session::open(
            &self.game,
            &mut Cursor::new(script),
            &mut output,
            &self.saves,
        )
        .unwrap();
        String::from_utf8(output).unwrap()
    }
    fn room(&self, id: &str) -> String {
        fs::read_to_string(self.game.join(format!("rooms/{id}.toml"))).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn value(text: &str) -> toml::Value {
    text.parse().unwrap()
}

#[test]
fn beginner_creates_connects_previews_and_travels_without_toml_edits() {
    let f = Fixture::new();
    let output = f.run("7\nn\nMountain Pass\n\n/multi\nA narrow trail.\n\nVisible: {{objects}}\n/end\n1\n1\nnorth\n\n0\n5\n6\nyes\n0\n2\n1\nnorth\nsouth\nquit\n0\n");
    assert!(output.contains("Room changes saved"), "{output}");
    let pass = f.room("mountain_pass");
    assert_eq!(value(&pass)["exits"]["south"].as_str(), Some("start"));
    assert_eq!(
        value(&f.room("start"))["exits"]["north"].as_str(),
        Some("mountain_pass")
    );
    assert!(value(&pass)["description"]
        .as_str()
        .unwrap()
        .contains("\n\nVisible: {{objects}}"));
    for id in ["start", "mountain_pass"] {
        let marker = format!(
            "Preview: {}\n",
            f.game.join("rooms").join(format!("{id}.toml")).display()
        );
        let shown = output
            .split_once(&marker)
            .unwrap()
            .1
            .split_once("--- exact contents ---\n")
            .unwrap()
            .1
            .split_once("--- end contents ---")
            .unwrap()
            .0;
        assert_eq!(shown, f.room(id));
    }
    assert!(output.contains("Validation successful!"));
    assert!(output.contains("A narrow trail."));
    assert!(output.contains("Returned to the creator"));
    assert!(!f.saves.exists());
}

#[test]
fn editing_preserves_comments_advanced_fields_and_stable_id() {
    let f = Fixture::new();
    let old = "# authored heading\nid = 'start' # stable\ntitle = 'Old' # title comment\ndescription = 'Old prose'\nart_file = 'view.txt'\n\n[[descriptions]] # conditional text\nwhen = [{ flag_set = 'awake' }]\ntext = 'Conditional prose'\n\n[[art_variants]]\nwhen = [{ flag_set = 'awake' }]\nfile = 'view.txt'\n\n[exits] # exits comment\n";
    fs::write(f.game.join("rooms/start.toml"), old).unwrap();
    fs::write(f.game.join("art/view.txt"), "art").unwrap();
    let output = f.run("7\n1\n1\nRenamed Location\n2\n/multi\nLine one \"quoted\"\nLine two café\n/end\n5\n6\nyes\n0\n0\n");
    assert!(output.contains("Room changes saved"), "{output}");
    let new = f.room("start");
    for text in [
        "# authored heading",
        "# stable",
        "# title comment",
        "# conditional text",
        "# exits comment",
        "art_file = 'view.txt'",
        "text = 'Conditional prose'",
    ] {
        assert!(new.contains(text), "missing {text}: {new}");
    }
    let a = value(old);
    let b = value(&new);
    for key in ["id", "descriptions", "art_file", "art_variants", "exits"] {
        assert_eq!(a[key], b[key]);
    }
    assert_eq!(b["title"].as_str(), Some("Renamed Location"));
    assert!(b["description"].as_str().unwrap().contains("café"));
    let again = f.run("7\n1\n5\n0\n0\n0\n");
    assert!(again.contains(&new));
    assert_eq!(f.room("start"), new);
}

#[test]
fn cancel_eof_and_back_discard_complete_drafts() {
    let f = Fixture::new();
    let original = f.room("start");
    for script in [
        "7\nn\ncancel\n0\n0\n",
        "7\nn\nNew\n\nText\n0\ncancel\n0\n0\n",
        "7\n1\n1\nChanged\n0\n0\n0\n",
        "7\n1\n2\n/multi\nuncommitted",
    ] {
        f.run(script);
        assert_eq!(f.room("start"), original);
        assert_eq!(fs::read_dir(f.game.join("rooms")).unwrap().count(), 1);
        assert!(!f.game.join(".lupine-room-edit").exists());
    }
}

#[test]
fn duplicate_and_invalid_ids_retry_and_one_way_exit_is_valid() {
    let f = Fixture::new();
    let output = f.run("7\nn\nSecond\nstart\n../escape\nBAD\nsecond\nDescription\n1\n1\neast\nno\n0\n6\nyes\n0\n0\n");
    assert!(output.contains("already exists"));
    assert!(output.contains("lowercase"));
    assert_eq!(
        value(&f.room("start"))["exits"]["east"].as_str(),
        Some("second")
    );
    assert!(value(&f.room("second"))["exits"]
        .as_table()
        .unwrap()
        .is_empty());
    lupine::adventure::Adventure::load(&f.game).unwrap();
}

#[test]
fn conflicting_reverse_exit_is_not_partially_changed() {
    let f = Fixture::new();
    lupine::creator::add_room(&f.game, "other").unwrap();
    fs::write(
        f.game.join("rooms/start.toml"),
        "id='start'\ntitle='Start'\ndescription='Text'\n[exits]\nsouth='other'\n",
    )
    .unwrap();
    let start = f.room("start");
    let other = f.room("other");
    // Edit other: connect other north to start; reverse would replace start south -> other (same, no conflict).
    // Instead connect other west to start then add a second forward conflict and decline it.
    let output = f.run("7\n1\n3\n2\n1\nwest\nyes\n0\n6\nyes\n0\n0\n");
    assert!(output.contains("Room changes saved"));
    assert_ne!(f.room("other"), other);
    // Add a third room and request start east -> third; start east is already other.
    let output = f.run("7\nn\nThird\n\nDescription\n1\n2\neast\nyes\nno\n0\n6\nyes\n0\n0\n");
    assert!(output.contains("Conflict:"), "{output}");
    assert!(output.contains("Connection left unchanged"));
    assert_eq!(
        value(&f.room("start"))["exits"]["east"].as_str(),
        Some("other")
    );
    assert!(value(&f.room("third"))["exits"]
        .as_table()
        .unwrap()
        .is_empty());
    assert_eq!(
        value(&f.room("start"))["exits"]["south"],
        value(&start)["exits"]["south"]
    );
}

#[test]
fn preexisting_invalid_content_is_not_silently_repaired() {
    let f = Fixture::new();
    fs::write(
        f.game.join("rooms/start.toml"),
        "id='start'\ntitle='Start'\ndescription='Text'\nunsupported=true\n",
    )
    .unwrap();
    let before = f.room("start");
    let output = f.run("7\n0\n");
    assert!(output.contains("Preexisting project error"));
    assert_eq!(f.room("start"), before);
}

#[test]
fn crlf_and_unmodified_rooms_survive_repeated_edits() {
    let f = Fixture::new();
    let old = "# comment\r\nid = 'start'\r\ntitle = 'Old' # trailing\r\ndescription = 'Text'\r\n[exits]\r\n";
    fs::write(f.game.join("rooms/start.toml"), old).unwrap();
    lupine::creator::add_room(&f.game, "z_other").unwrap();
    let other = f.room("z_other");
    for title in ["First", "Second"] {
        let output = f.run(&format!("7\n1\n1\n{title}\n6\nyes\n0\n0\n"));
        assert!(output.contains("Room changes saved"), "{output}");
        let new = f.room("start");
        assert!(new.contains("# comment\r\n"));
        assert!(new.contains("# trailing\r\n"));
        assert_eq!(f.room("z_other"), other);
    }
}

#[test]
fn existing_encounter_room_keeps_supported_advanced_properties() {
    let f = Fixture::new();
    fn copy(from: &Path, to: &Path) {
        for e in fs::read_dir(from).unwrap() {
            let p = e.unwrap().path();
            let t = to.join(p.file_name().unwrap());
            if p.is_dir() {
                fs::create_dir_all(&t).unwrap();
                copy(&p, &t);
            } else {
                fs::copy(&p, &t).unwrap();
            }
        }
    }
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_persistent"),
        &f.game,
    );
    // Remove the fixture's room, which is unrelated to the copied sample.
    fs::remove_file(f.game.join("rooms/start.toml")).unwrap();
    let mut rooms = fs::read_dir(f.game.join("rooms"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    rooms.sort();
    // Manager sorts by authored ID, not filename.
    let mut ids = rooms
        .iter()
        .map(|p| {
            let text = fs::read_to_string(p).unwrap();
            (
                value(&text)["id"].as_str().unwrap().to_owned(),
                p.clone(),
                text,
            )
        })
        .collect::<Vec<_>>();
    ids.sort();
    let (index, (_, path, old)) = ids
        .iter()
        .enumerate()
        .find(|(_, (_, _, s))| value(s).get("encounters").is_some())
        .unwrap();
    let output = f.run(&format!(
        "7\n{}\n1\nChanged encounter title\n6\nyes\n0\n0\n",
        index + 1
    ));
    assert!(output.contains("Room changes saved"), "{output}");
    let new = fs::read_to_string(path).unwrap();
    assert_eq!(value(old)["encounters"], value(&new)["encounters"]);
    lupine::adventure::Adventure::load(&f.game).unwrap();
}
