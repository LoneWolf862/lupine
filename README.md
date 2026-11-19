# Lupine

## RPG authoring development candidate

Main menu **R** now exposes guided enemies, physical weapons, shops and currencies; room encounters can create an enemy in the same confirmed transaction. See [RPG authoring review](docs/RPG_AUTHORING_DEVELOPMENT.md) for schemas, safety, tests, Windows instructions and draft PRs #14-#18. Earlier creator PRs #8-#13 are merged. These new changes are pending review.

**Lupine** is a lightweight, offline, terminal-based interactive fiction engine written in Rust. It is inspired by the *command-driven interface* of classic text adventures; no original Zork story content is included. Game creators define their own adventures through TOML files rather than modifying Rust code. **HOWLBOUND** is only a tentative name for a future original adventure.

> **Status:** The [v0.9.0 standalone release](https://github.com/LoneWolf862/lupine/releases/tag/v0.9.0) is published. This creator development checkout still declares Cargo version 0.8.4; the draft RPG authoring changes are not part of that release.

## Start here

Install [Rust](https://www.rust-lang.org/tools/install), clone the repository, and open a terminal in the project directory:

```cmd
cargo run -- help
cargo run -- new my_adventure my_adventure My First Adventure
cargo run -- validate my_adventure
cargo run -- play my_adventure
```

Run `cargo run -- new` for beginner setup, then continue in the persistent **Adventure Creator**. Reopen with `cargo run -- creator "my_adventure"`. Its guided editors cover rooms/connections, items/aliases, spells/teaching items, multiline artwork/variants, interaction puzzles/locked doors, conditional descriptions, encounter tables and enemy/weapon/shop/currency definitions, and supported FIG settings. Review previews and explicitly confirm Save; validate and play from the same menu. Type `quit` in play to return. Content remains ordinary editable TOML, FIG and text files.

**Creator status:** Core guided authoring is merged. RPG definition editors are implemented candidates in draft PRs #14-#18. See the [current RPG review](docs/RPG_AUTHORING_DEVELOPMENT.md) for exact verification and limitations.

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

**Distribution:** Standalone builds are published separately in **v0.9.0**. These draft creator changes require this source checkout and are not claimed to be in those binaries. The runtime is entirely offline; the Rust/Cargo build process may download crates if not cached.

## License

MIT. Consult the repository's license file for the complete terms.

Optional engine configuration uses TOML syntax in config/character.fig (RPG opt-in) and config/terminology.fig (display labels). No config files are required for a narrative-only game. See docs/CONFIG_FIG_SPEC.md in the source repository. Root character.toml is obsolete; move it into config/character.fig. Player saves remain strictly manual .rork files.

<!-- lupine-phase-2c2-spec -->
## Generated creator configuration

New adventures include all standard content folders and self-documenting `config/character.fig` and `config/terminology.fig`. RPG is disabled by default (`enabled = false`); set it to true and validate/reload to opt in. Existing FIGs without the key retain their prior enabled behavior. See [self-documenting configuration](docs/SELF_DOCUMENTING_CONFIG_SPEC.md) and [FIG settings](docs/CONFIG_FIG_SPEC.md).
