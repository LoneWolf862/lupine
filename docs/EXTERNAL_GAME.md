# Create a separate game using Lupine (v0.4.0)

Lupine is both a Rust library and an example terminal application. Your game's
Cargo package can depend on Lupine without modifying Lupine's source code.

## Run the independent sample

From the **root of the Lupine checkout**, run:

```shell
cargo run --manifest-path samples/independent_game/Cargo.toml
```

Try `north` (blocked), `take token`, `use token on door`, `north`, and `quit`.

## Create your own Cargo application

Create a new binary package with `cargo new my-adventure`. Add a dependency to
its `Cargo.toml` that points at your local Lupine checkout:

```toml
[dependencies]
lupine = { path = "../lupine" }
```

The path shown assumes `my-adventure` and `lupine` are sibling folders;
adjust the relative path for your layout. For a Git dependency instead use:

```toml
[dependencies]
lupine = { git = "https://github.com/LoneWolf862/lupine", rev = "5e878c4" }
```

The revision above is **v0.3.0**, included only to demonstrate the syntax;
replace it with the verified v0.4.0 commit when published, since v0.3.0
**does not provide `lupine::run`**.

In `src/main.rs`, define a `World` using the public `lupine::world` types,
then call:

```rust,ignore
fn main() -> std::io::Result<()> {
    lupine::run(create_world(), "My Adventure")
}
```

See `samples/independent_game/src/main.rs` for a complete working example.
`run` validates the world before starting the terminal loop. For automation,
`runner::run_with_io` accepts custom input/output streams.

**Not implemented yet:** loading TOML/JSON worlds, a visual editor, creator
packaging, and a story-file schema. At v0.4.0 the author must still write Rust.
The sample is an independent *package*, but is included in the Lupine checkout
for convenience; copy it elsewhere to verify true project independence.
