//! Demonstration executable, not the engine's required entry point.
#[path = "../examples/fixture_world.rs"]
mod fixture_world;

fn main() -> std::io::Result<()> {
    lupine::run(
        fixture_world::fixture_world(),
        "LUPINE v0.4.0 — Engine Test Adventure",
    )
}
