//! Demonstration executable, not the engine's required entry point.
#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn main() -> std::io::Result<()> {
    lupine::run(
        fixture_world::fixture_world(),
        "LUPINE v0.5.0-alpha.1 — Engine Test Adventure",
    )
}
