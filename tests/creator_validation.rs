use lupine::story_loader::load_game;
use std::fs;
use std::path::{Path, PathBuf};

struct TempStory(PathBuf);

impl TempStory {
    fn new(label: &str) -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "lupine-creator-{label}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("rooms")).unwrap();
        fs::create_dir_all(path.join("items")).unwrap();
        fs::write(
            path.join("game.toml"),
            "format_version=1\nid='testing'\ntitle='Testing'\nstarting_room='foyer'\n",
        )
        .unwrap();
        Self(path)
    }
    fn write(&self, relative: &str, data: &str) {
        fs::write(self.0.join(relative), data).unwrap();
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempStory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn starter_template_is_valid() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/starter");
    let game = load_game(path).unwrap();
    assert_eq!(game.world.rooms.len(), 2);
}

#[test]
fn missing_exit_reports_room_file_and_fix() {
    let tmp = TempStory::new("exit");
    tmp.write(
        "rooms/foyer.toml",
        "id='foyer'\ntitle='Foyer'\ndescription='Foyer'\n[exits]\nnorth='missing'\n",
    );
    let error = load_game(tmp.path()).unwrap_err();
    assert!(
        error.contains("foyer.toml")
            && error.contains("unknown room 'missing'")
            && error.contains("Create"),
        "{error}"
    );
}

#[test]
fn duplicate_room_id_reports_file() {
    let tmp = TempStory::new("duplicates");
    tmp.write("rooms/a.toml", "id='foyer'\ntitle='A'\ndescription='A'\n");
    tmp.write("rooms/b.toml", "id='foyer'\ntitle='B'\ndescription='B'\n");
    let error = load_game(tmp.path()).unwrap_err();
    assert!(
        error.contains("b.toml") && error.contains("duplicate room id"),
        "{error}"
    );
}

#[test]
fn missing_item_starting_room_identifies_item_file() {
    let tmp = TempStory::new("item");
    tmp.write(
        "rooms/foyer.toml",
        "id='foyer'\ntitle='Foyer'\ndescription='Foyer'\n",
    );
    tmp.write(
        "items/torch.toml",
        "id='torch'\nname='torch'\ndescription='Torch'\nportable=true\nstarting_room='unknown'\n",
    );
    let error = load_game(tmp.path()).unwrap_err();
    assert!(
        error.contains("torch.toml") && error.contains("unknown room 'unknown'"),
        "{error}"
    );
}
