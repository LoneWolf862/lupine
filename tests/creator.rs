use lupine::creator::new_game;
use lupine::story_loader::load_game;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_project() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("lupine_creator_{}_{}", std::process::id(), nonce))
}

#[test]
fn created_story_validates_and_plays() {
    let path = temp_project();
    new_game(&path, "my_adventure", "My First Adventure").unwrap();
    let story = load_game(&path).unwrap();
    assert_eq!(story.id, "my_adventure");
    assert_eq!(story.title, "My First Adventure");
    assert_eq!(story.world.rooms.len(), 1);
    assert_eq!(story.world.starting_room, "start");
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn project_generator_refuses_existing_directory() {
    let path = temp_project();
    fs::create_dir(&path).unwrap();
    let sentinel = path.join("keep.txt");
    fs::write(&sentinel, "do not overwrite").unwrap();
    assert!(new_game(&path, "test", "Test").is_err());
    assert_eq!(fs::read_to_string(sentinel).unwrap(), "do not overwrite");
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn rejects_invalid_game_ids_without_creating_files() {
    let path = temp_project();
    for id in ["", "../escape", "UpperCase", "contains space"] {
        assert!(new_game(&path, id, "Test").is_err());
        assert!(!path.exists());
    }
}

#[test]
fn game_title_is_safely_encoded_as_toml() {
    let path = temp_project();
    new_game(&path, "quoted_game", "The \"Quoted\" Adventure").unwrap();
    let story = load_game(&path).unwrap();
    assert_eq!(story.title, "The \"Quoted\" Adventure");
    fs::remove_dir_all(path).unwrap();
}
