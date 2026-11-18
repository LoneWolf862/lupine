# Lupine

**Lupine v0.8.4** is a lightweight, offline, terminal-based interactive fiction engine written in Rust. It is inspired by the *command-driven interface* of classic text adventures; no original Zork story content is included. Game creators define their own adventures through TOML files rather than modifying Rust code. **HOWLBOUND** is only a tentative name for a future original adventure.

> **Status:** Unreleased, pre-1.0 development. Phase 1 introduces FIG configuration; see [configuration contract](docs/CONFIG_FIG_SPEC.md). A manual Windows/Linux packaging workflow exists; this source checkout uses the Rust toolchain. Native Linux and published-release claims require separate verification.

## Start here

Install [Rust](https://www.rust-lang.org/tools/install), clone the repository, and open a terminal in the project directory:

```cmd
cargo run -- help
cargo run -- new my_adventure my_adventure My First Adventure
cargo run -- validate my_adventure
cargo run -- play my_adventure
```

Alternatively, run `cargo run -- new` to use the guided wizard: press Enter for Beginner mode, name your adventure and opening location, then approve the summary. File previews are optional; type `advanced` at the welcome prompt for editable IDs and direct previews. Enter accepts defaults, `?` explains fields, `back` revisits an answer, and `cancel` stops. Completion enters the persistent Adventure Creator, where you can inspect the file tree, validate and play without restarting. Type `quit` in play to return. Use `cargo run -- creator` for a Create/Open menu, or `cargo run -- creator "my_adventure"` to reopen a project. Guided content editors are later phases. Edit generated `game.toml`, room and item TOML files, then validate again; game content does not need to be compiled.

To try prewritten demonstrations:

```cmd
cargo run -- play story_example
cargo run -- play samples/rpg_spellbook
cargo run -- play samples/rpg_rewards
cargo run -- play samples/rpg_transactions
cargo run -- play samples/rpg_persistent
```

### Player commands

Explore with `look`, `north`/`n` (and other directions), `examine <object>`, `take <object>`, `take all`, `drop <object>`, `drop all`, `inventory`, and `use <object> on <target>` (subject to the authored adventure). Optional RPG stories add `stats`, `weapons`, `spellbook`/`spells`, `learn <scroll>`, `attack <enemy> with <weapon>`, `cast <spell> on <enemy>`, `enemies`, `wallet`, `shop`, and `buy <offer>`. Available commands depend on story configuration; type `help` in the game.

**Saving is strictly manual.** Use `save <name>`, `saves`, and `load <name>`. Saves use `.rork` files. There is **no autosave**, no save-on-quit, and no automatic loading after defeat. At zero HP, gameplay locks until a manual load or exit. See [save format](docs/SAVE_FORMAT.md).

### Optional RPG features

- `config/character.fig`: HP, MP, AP, level and XP.
- `weapons/`, `spells/`: weapons, learned abilities and scrolls.
- `enemies/` and room `[encounters]`: weighted combat encounters, XP, loot, hidden HP bars.
- Encounter `persistence`: `temporary` (default), `room` (retains surviving enemy HP after retreat), `unique` (one-time cleared encounter).
- Multi-currency wallet; `shops/` offers for purchases, healing, mana, selling, spells and unlock flags.
- `transaction_interactions.toml`: connect `use <object>` to existing atomic shop offers (for example, `use altar`).
- Dynamic room descriptions may include `{{objects}}`, which reflects currently visible objects.

RPG features are opt-in. The non-RPG interactive fiction engine still works without them.

## Creator references

Start with [Creator Quickstart](docs/CREATOR_QUICKSTART.md) and the updated [RPG Creator Guide](docs/RPG_CREATOR_GUIDE.md). The underlying [TOML story format](docs/STORY_FORMAT_PROPOSAL.md), [room art](docs/ROOM_ART.md), [transactions](docs/TRANSACTIONS_SHOPS.md), [story transactions](docs/STORY_TRANSACTIONS_V0_8_2.md), and [persistent encounters](docs/PERSISTENT_ROOMS_V0_8_3.md) are documented separately. Historical milestone notes describe the state *at the time written* and may mention features now implemented.

### Development checks

```cmd
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

For sample-level verification, `cargo run -- validate samples/rpg_persistent` and the other sample paths above should complete successfully. The regression suite includes save integrity, shop rollback, combat defeat, and sample schema checks.

## Project layout

- `src/` — engine, parsing, terminal runner, progression and persistence.
- `templates/starter/` — minimal creator starter adventure.
- `samples/` — independent sample game and optional RPG examples.
- `tests/` — automated integration and regression tests.
- `docs/` — authoring, architecture, roadmaps and historical stage notes.

**Distribution:** The first standalone builds for Windows and Linux are planned for **v0.9**, not shipped by this source package. The runtime is entirely offline; the Rust/Cargo build process may download crates if not cached.

## License

MIT. Consult the repository's license file for the complete terms.

Optional engine configuration uses TOML syntax in config/character.fig (RPG opt-in) and config/terminology.fig (display labels). No config files are required for a narrative-only game. See docs/CONFIG_FIG_SPEC.md in the source repository. Root character.toml is obsolete; move it into config/character.fig. Player saves remain strictly manual .rork files.
