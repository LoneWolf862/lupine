# Lupine

Lupine is a lightweight, offline, terminal-based Rust interactive fiction and optional
RPG engine. Authors build adventures with editable TOML content, FIG configuration and
the guided Adventure Creator. HOWLBOUND is only the tentative name of a future original
adventure; no original story is prescribed by this engine.

**Development version: 0.10.0-dev.1.** Creator work through Phase 2I and Phase 3
startup, history, parser and export improvements (#20–#24) are merged.
They are not a published release. The existing
[v0.9.0 standalone release](https://github.com/LoneWolf862/lupine/releases/tag/v0.9.0)
and its tag remain unchanged. See [Phase 3 verification and handoff](docs/PHASE_3_REVIEW.md).

## Players: start here

Extract a package built for your OS. No Rust, Cargo, account or network is needed to play.
Windows adventure packages include **Play.cmd**. On Linux, open a terminal in the package
and run `sh Play.sh`. Or launch `lupine.exe` / `./lupine` without arguments for:

1. Play an Adventure
2. Create or Edit an Adventure
3. Help
4. Exit

Play lists adventures in the current/executable folders and their `adventures/` and
`samples/` collections. Choose a number, or B to enter another adventure/collection folder.
Every selection validates and starts fresh; `quit` returns to startup. Nothing silently
loads a save. See [installation and distribution](docs/INSTALLATION.md).

Explore with `look`, `north`/`n`, `examine <object>`, `take <object>`, `take all`,
`drop <object>`, `drop all`, `inventory`, and `use <item> on <target>`. Names, item aliases,
exact IDs and unambiguous partial names work. `pick up the bronze key` and `grab key`
are accepted when unambiguous. Multiple matches prompt for a number or ID; cancel with
0/back/cancel. Typo suggestions never execute an action.

Movement also accepts `travel south`, `run south`, `head south`, and `go to the south`.
`look <object>` examines it; `look around` shows the room. Ordered word prefixes such as
`fire scroll` can match `fireball scroll`; exact IDs, names and authored aliases still win.
`leave <direction>` moves, while `leave <object>` remains a drop alias. There is no facing
direction: use the listed compass exits rather than left/right. Suggestions explain how
to retry the command; replying yes does not execute them.

In supported interactive terminals, Up/Down recall recent input; Left/Right and Backspace
edit it. History is memory-only, capped at 100 entries across this invocation's prompts,
with empty lines and consecutive duplicates omitted. Redirected input remains scriptable.

Optional RPG adventures add `stats`, `weapons`, `spellbook`/`spells`, `learn <scroll>`,
`attack <enemy> with <weapon>`, `cast <spell> on <enemy>`, `enemies`, `wallet`, `shop`,
and `buy <offer>`. Unambiguous `use <weapon/spell> on <enemy>` reaches the same action;
ordinary visible item targets retain priority. Use an enemy's `#instance` when needed.
Exact enemy HP stays private. `help` explains commands in the game.

`hit`, `strike`, `stab`, and `slash` are attack aliases. `attack <enemy>` asks which carried
weapon to use, even with only one; cancel to spend no turn. `cast fireball at goblin` and
`attack goblin with fireball` work for a learned spell when no carried weapon matches.
Bare spell names provide syntax guidance without casting or choosing a target.

**Saving is strictly manual:** `save <name>`, `saves`, `load <name>`. RORK files live in
your user-data directory. There is no autosave, save-on-quit or automatic recovery after
defeat. Unsaved quit/load confirmations remain in place. See [save format](docs/SAVE_FORMAT.md).
Bare `save` offers to overwrite the last slot successfully saved or loaded in this play
session. Yes updates that slot; no asks `Save name:`. With no current slot, it asks for a
name immediately. Enter/cancel/back at the name prompt cancels. `save forest camp` writes
directly to that named slot, replacing the same name without another question. Selecting
an existing name through the name prompt still asks before overwriting. Save/load/quit
questions appear above the editable input line so terminal redraws cannot erase them.

## Creators

Choose Create or Edit at startup, or `lupine creator [PATH]`. Guided menus cover rooms,
connections, items, spells, artwork, interactions, locked doors, conditional descriptions,
encounters, enemies, weapons, shops, currencies and implemented FIG settings. Changes stay
drafts until explicit confirmation; previews show exact authored bytes. Display-name edits
keep IDs stable. **T** manages transaction-bound objects; **M** deliberately migrates
supported IDs and their authored references, with save-compatibility warnings. Interrupted
edits open guided recovery when the recorded evidence proves restoration/completion safe.

Creator **E** exports Windows x64, Linux x64, or both as separate ZIPs or platform folders
inside a new directory. The native
running engine is available by default; another platform requires its matching local
runtime and build metadata. Export validates, excludes saves/development files and adds
launchers/instructions. It never downloads runtimes or changes source adventure files.
Share the platform ZIP (extract before launching) or the complete platform folder.
ZIPs use stored entries without compression. Export **C** offers verified staging cleanup.
See [Creator Quickstart](docs/CREATOR_QUICKSTART.md) and [RPG Creator Guide](docs/RPG_CREATOR_GUIDE.md).

Content uses `.toml`; game-wide implemented configuration uses TOML-syntax `.fig` files.
New games include commented `config/character.fig` and `config/terminology.fig`; RPG is
disabled until `enabled = true`. Existing FIGs without enabled keep their prior enabled
behavior. Currencies stay in game.toml. Mutable manual player saves use `.rork`.

## Developers and advanced CLI

Install Rust, then from this source checkout:

```text
cargo run
cargo run -- help
cargo run -- creator
cargo run -- new my_adventure my_adventure My First Adventure
cargo run -- validate my_adventure
cargo run -- play my_adventure
cargo run -- fixture
cargo run -- --version
```

Existing `new`, `add room`, `add item`, `add art`, `validate`, `play` and `creator` commands
remain available. Sample adventures are under `story_example/` and `samples/rpg_*`.
The Rust-defined fixture is an explicit development option. Builds may download crates;
the runtime never uses networking.

```text
cargo fmt --all --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
```

CI tests Windows Server 2022 and Ubuntu 22.04. The manually dispatched portable-build
workflow produces versioned development artifacts and runtime metadata; it does not
publish a release. Actual checks, native-platform limits and merge order are recorded
in the [Phase 3 review](docs/PHASE_3_REVIEW.md). Human beginner testing and native CachyOS
testing remain separate from scripted or agent-driven terminal verification.

Architecture, requirements, creator vision and historical milestones are in `docs/`.
Historical stage reports retain their original versions and verification claims.
[Phase 4 review](docs/PHASE_4_REVIEW.md) records the five unmerged development PRs,
verified behavior and conservative migration/recovery limits. Human beginner testing,
native CachyOS testing, pronouns and command chaining remain future work.
The instructor-provided CSE 310 README template, honest time log and submission
evidence remain a separate obligation; this README does not claim that submission is done.

## License

MIT; see LICENSE. THIRD_PARTY_NOTICES.txt contains locked dependency notices, also included
in exported packages. Regenerate with `python scripts/update_notices.py` after dependency
updates. Authors supply their own adventure license and credits.
