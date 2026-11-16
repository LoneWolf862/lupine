# Lupine development roadmap

## v0.1 — Foundation (verified on Windows)
- [x] Cargo package and portable repo files
- [x] CLI read/evaluate/print loop
- [x] Typed commands, directions, help, quit, look, navigation
- [x] Three generic fixture rooms
- [x] Parser unit tests and navigation integration test
- [x] Validate `cargo build` and `cargo test` on Windows (4 passing tests)
- [ ] Validate `cargo fmt --check` and `cargo clippy` on a local Rust toolchain
- [x] Push to public repository under LoneWolf862

## v0.2 — Objects and interactions (implementation candidate; local validation pending)
- [x] Stable item identifiers, portable/fixed properties and object locations
- [x] Examine, take, drop, inventory
- [x] Aliases, articles, synonyms, multiword targets, ambiguity messages
- [x] One generic persistent stateful interaction and corresponding tests
- [ ] Confirm v0.2 builds and all tests pass on Windows and Linux

## v0.3 — Release readiness
- [ ] Windows and CachyOS build/test validation
- [ ] Packaged offline build validation
- [ ] README adjusted to instructor-provided template
- [ ] Demo script and course video preparation

## Later, as needed
- [ ] Original HOWLBOUND story design together
- [ ] Save/load, external world data, typo suggestions, and advanced grammar
