use lupine::encounters::EncounterContent;
use lupine::rpg_content::RpgContent;
use lupine::runner::run_with_io_and_saves_progression_content_encounters;
use lupine::story_loader::load_game;
use std::io::Cursor;

#[test]
fn take_all_drop_all_and_spellbook_alias() {
    let game = load_game("samples/rpg_spellbook").unwrap();
    let root = std::path::Path::new("samples/rpg_spellbook");
    let content = RpgContent::load(root, &game.world, true).unwrap();
    let ids = game
        .world
        .rooms
        .iter()
        .map(|r| r.id.clone())
        .collect::<Vec<_>>();
    let encounters = EncounterContent::load(root, &ids, true).unwrap();
    let save_root =
        std::env::temp_dir().join(format!("lupine_polish_no_save_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&save_root);
    let mut out = Vec::new();
    run_with_io_and_saves_progression_content_encounters(
        game.world,
        &game.title,
        &game.id,
        &save_root,
        game.save_compatibility_version,
        game.progression,
        content,
        encounters,
        Cursor::new(
            b"take all\ninventory\ntake all\nspells\ndrop all\ninventory\ndrop all\nquit\nyes\n",
        ),
        &mut out,
    )
    .unwrap();
    let result = String::from_utf8(out).unwrap();
    assert!(result.contains("You take the rusty dagger."), "{result}");
    assert!(result.contains("You take the healing scroll."), "{result}");
    assert!(result.contains("You take the fireball scroll."), "{result}");
    assert!(result.contains("Nothing to take."), "{result}");
    assert!(result.contains("Your spellbook is empty."), "{result}");
    assert!(result.contains("You drop the rusty dagger."), "{result}");
    assert!(result.contains("You are carrying nothing."), "{result}");
    assert!(
        result.contains("You are carrying nothing to drop."),
        "{result}"
    );
    assert!(!save_root.exists(), "bulk commands must not autosave");
}
