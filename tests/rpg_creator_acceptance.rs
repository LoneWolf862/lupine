//! Real CLI acceptance: every adventure definition is authored through menus.
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn wizard_authors_combat_economy_and_manual_saves() {
    let root =
        Scratch(std::env::temp_dir().join(format!("lupine-rpg-acceptance-{}", std::process::id())));
    fs::create_dir(&root.0).unwrap();
    let game = root.0.join("RPG demo");
    let data = root.0.join("data");
    let script = include_str!("fixtures/rpg_creator_acceptance.txt")
        .replace("{PROJECT}", &game.to_string_lossy());
    let mut child = Command::new(env!("CARGO_BIN_EXE_lupine"))
        .arg("new")
        .env("APPDATA", &data)
        .env("XDG_DATA_HOME", &data)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || input.write_all(script.as_bytes()).unwrap());
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(output.status.success());
    let out = String::from_utf8(output.stdout).unwrap();
    // Retain a readable transcript when explicitly requested for local review.
    if let Ok(path) = std::env::var("LUPINE_RPG_TRANSCRIPT") {
        fs::write(path, &out).unwrap();
    }
    assert!(!out.contains("Save needs attention"), "{out}");
    assert!(!out.contains("operation stopped"), "{out}");
    assert!(!out.contains("manager stopped"), "{out}");
    for expected in [
        "Validation successful",
        "Loaded: wounded",
        "Loaded: victory",
        "Loaded: final",
    ] {
        assert!(out.contains(expected), "Missing {expected}: {out}");
    }
    let a = lupine::adventure::Adventure::load(&game).unwrap();
    assert_eq!(a.encounters.enemies.len(), 2);
    assert_eq!(a.rpg.weapons.len(), 2);
    assert_eq!(a.wallet.currencies.len(), 2);
    assert_eq!(a.shops.shops["supplies"].offers.len(), 6);
    let saves = data
        .join(if cfg!(windows) {
            "Lupine/saves"
        } else {
            "lupine/saves"
        })
        .join("creator_acceptance");
    let read = |slot: &str| {
        let path = fs::read_dir(&saves)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(&format!("{slot}."))
            })
            .unwrap();
        fs::read_to_string(path).unwrap()
    };
    assert_eq!(
        fs::read_dir(&saves).unwrap().count(),
        8,
        "Only the eight explicitly named save slots should exist"
    );
    let wounded = read("wounded");
    let retained = read("retained");
    let victory = read("victory");
    let final_save = read("final");
    let decode = |s: &str| lupine::save::decode_save(&a.game.world, &a.game.id, s).unwrap();
    let w = decode(&wounded);
    let r = decode(&retained);
    let v = decode(&victory);
    let last = decode(&final_save);
    let unowned = decode(&read("unowned"));
    assert_eq!(unowned.active_enemies[0].current_hp, 25);
    assert_eq!(unowned.character.as_ref().unwrap().hp, 120);
    assert_eq!(
        v.location("trophy"),
        Some(lupine::state::ItemLocation::Inventory)
    );
    assert_eq!(w.active_enemies[0].current_hp, 15);
    assert_eq!(r.active_enemies, w.active_enemies);
    assert!(v.active_enemies.is_empty());
    assert_eq!(v.character.as_ref().unwrap().level, 2);
    assert_eq!(v.wallet["gold"], 19);
    assert_eq!(v.wallet["tokens"], 2);
    assert!(v.learned_abilities.contains("renew"));
    assert_eq!(v.shop_purchases["supplies:axe_sale"], 1);
    assert_eq!(v.shop_purchases["supplies:healing"], 1);
    assert_eq!(v.shop_purchases["supplies:lesson"], 1);
    assert_eq!(decode(&read("economy")).wallet["gold"], 23);
    assert!(last.active_enemies.is_empty());
    assert_eq!(last.current_room, "yard");
    assert_eq!(last.character.as_ref().unwrap().experience, 20);
    let sw = decode(&read("scout_wounded"));
    let sr = decode(&read("scout_retained"));
    assert_eq!(sw.active_enemies[0].current_hp, 4);
    assert_eq!(sw.active_enemies, sr.active_enemies);
    let gameplay = out.split_once("Powered by Lupine").unwrap().1;
    assert!(!gameplay.contains("No visible enemy"));
    assert!(!gameplay.contains("You cannot go"));
    assert!(gameplay.contains("Transaction complete: healing"));
    assert!(gameplay.contains("Transaction complete: mana"));
    a.shops.validate_saved(&last).unwrap();
}
