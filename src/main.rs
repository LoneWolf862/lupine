//! The default fixture remains available; `play PATH` loads TOML story content.
#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_] => lupine::run(
            fixture_world::fixture_world(),
            "LUPINE v0.5.0-alpha.2 — Engine Test Adventure",
        ),
        [_, command, path] if command == "play" => {
            let game = lupine::story_loader::load_game(path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            lupine::run(game.world, &game.title)
        }
        _ => {
            eprintln!("Usage: lupine [play <story-directory>]");
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid arguments",
            ))
        }
    }
}
