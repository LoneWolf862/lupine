use lupine::{
    adventure::Adventure,
    creator::{new_game, session},
};
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
            "lupine-rpg-editor-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let game = root.join("game");
        new_game(&game, "rpg_editor", "RPG Editor").unwrap();
        fs::write(game.join("config/character.fig"), "enabled=true\n").unwrap();
        Self {
            saves: root.join("saves"),
            root,
            game,
        }
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
fn enemy_fields_attacks_preview_and_cancel() {
    let f = Fixture::new();
    let out=f.run("r\n1\nn\nguard\n1\nPractice Guard\n3\n0\n2\n4\n30\n5\n125\n6\n2\n7\n100\n8\na\nTap\n3\n1\nyes\na\nPush\n5\n3\nyes\n0\np\ns\nno\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let source = f.read("enemies/guard.toml");
    assert!(out.contains(&source));
    let a = Adventure::load(&f.game).unwrap();
    let e = &a.encounters.enemies["guard"];
    assert_eq!(
        (e.level, e.max_hp, e.attack_power, e.defense, e.experience),
        (2, 30, 125, 2, 100)
    );
    assert_eq!(e.attacks.len(), 2);
    assert_eq!(e.attacks[1].weight, 3);
    assert!(!f.saves.exists());
    f.run("r\n1\n1\n1\nDiscarded\ncancel\n0\n0\n");
    assert_eq!(f.read("enemies/guard.toml"), source);
}
#[test]
fn enemy_preserves_comments_and_rejects_weight_overflow() {
    let f = Fixture::new();
    let original="# authored\nid='guard' # fixed\nname='Guard'\ndescription='Practice'\nlevel=1\nmax_hp=20\nattacks=[{name='Tap',damage=2,weight=4294967295}] # inline\n";
    fs::write(f.game.join("enemies/unusual.toml"), original).unwrap();
    let out = f.run("r\n1\n1\n8\na\nPush\n1\n1\nyes\n0\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("attack weights overflow"), "{out}");
    assert_eq!(f.read("enemies/unusual.toml"), original);
    let out = f.run("r\n1\n1\n1\nRenamed\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let text = f.read("enemies/unusual.toml");
    assert!(text.contains("# authored") && text.contains("# inline") && text.contains("# fixed"));
    assert_eq!(
        Adventure::load(&f.game).unwrap().encounters.enemies["guard"].name,
        "Renamed"
    );
}
#[test]
fn weapon_item_workflow_and_gameplay() {
    let f = Fixture::new();
    let out=f.run("r\n2\ni\nn\nsword\n1\ns\nyes\n0\nn\n1\n1\n0\n9\n2\nslashing\np\ns\nyes\n0\n0\n1\nweapons\ntake sword\nweapons\nsave gear\nload gear\nquit\nyes\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let a = Adventure::load(&f.game).unwrap();
    let w = &a.rpg.weapons["sword"];
    assert_eq!(w.damage, 9);
    assert_eq!(w.damage_type, "slashing");
    assert!(out.contains(&f.read("weapons/sword.toml")));
    assert!(f.saves.exists());
    let source = f.read("weapons/sword.toml");
    f.run("r\n2\n1\n1\n15\ncancel\n0\n0\n0\n");
    assert_eq!(f.read("weapons/sword.toml"), source);
    let out = f.run("r\n2\nn\n0\n0\n0\n");
    assert!(out.contains("No unused portable items"), "{out}");
}
