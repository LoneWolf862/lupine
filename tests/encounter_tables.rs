use lupine::encounters::{EncounterContent, RandomSource};
use lupine::state::GameState;
use lupine::story_loader::load_game;
use std::path::Path;

struct FixedRng(u32);
impl RandomSource for FixedRng {
    fn below(&mut self, upper: u32) -> u32 {
        self.0.min(upper.saturating_sub(1))
    }
}
fn example() -> (lupine::story_loader::LoadedGame, EncounterContent) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_spellbook");
    let game = load_game(&root).unwrap();
    let ids = game
        .world
        .rooms
        .iter()
        .map(|r| r.id.clone())
        .collect::<Vec<_>>();
    let content = EncounterContent::load(&root, &ids, true).unwrap();
    (game, content)
}
#[test]
fn definitions_and_tables_load() {
    let (_, c) = example();
    assert_eq!(c.enemies.len(), 2);
    assert_eq!(c.tables["woods"].chance, 25);
    assert_eq!(c.tables["woods"].max_enemies, 2);
}
#[test]
fn zero_or_failed_roll_does_not_spawn() {
    let (game, c) = example();
    let state = GameState::new(&game.world);
    let mut next = 1;
    assert!(c
        .roll("camp", &state, &mut FixedRng(0), &mut next)
        .is_empty());
    assert!(c
        .roll("woods", &state, &mut FixedRng(99), &mut next)
        .is_empty());
    assert_eq!(next, 1);
}
#[test]
fn success_roll_has_unique_instances_and_hides_hp() {
    let (game, c) = example();
    let state = GameState::new(&game.world);
    let mut next = 1;
    let group = c.roll("woods", &state, &mut FixedRng(0), &mut next);
    assert_eq!(group.len(), 1);
    assert_eq!(group[0].instance_id, 1);
    let out = c.describe(&group);
    assert!(out.contains("[----------]"), "{out}");
    assert!(!out.contains("25/25"));
    assert!(!out.contains("30/30"));
    assert_eq!(next, 2);
}
#[test]
fn room_entry_and_look_show_visible_enemies() {
    let (game, c) = example();
    let mut output = Vec::new();
    // Deterministic rolling tested separately; runner uses time-based seed.
    lupine::runner::run_with_io_and_saves_progression_content_encounters(
        game.world,
        &game.title,
        &game.id,
        &std::env::temp_dir().join("lupine_no_write_encounter_test"),
        game.save_compatibility_version,
        game.progression,
        lupine::rpg_content::RpgContent::default(),
        c,
        std::io::Cursor::new("look\nenemies\nquit\n"),
        &mut output,
    )
    .unwrap();
    assert!(String::from_utf8(output)
        .unwrap()
        .contains("No enemies are currently visible."));
}
