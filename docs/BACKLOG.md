# Lupine Development Backlog

Deferred until after standalone distribution.

## Interactive Startup Menu

When Lupine launches without arguments, display a terminal menu:

1. Play an Adventure
2. Create an Adventure
3. Help
4. Exit

Keep the existing command-line interface for advanced users.

## Parser and Terminal Improvements

- Support up/down arrow command history.
- Expand aliases and alternate object names where appropriate.
- Improve natural-language command interpretation.
- Handle equivalent phrasing such as using versus casting spells where appropriate.
- Improve typo suggestions for items, spells, and enemies.

## Constraints

- Remain lightweight, offline, and terminal-based.
- Preserve TOML adventure and RORK save compatibility.
- Never autosave; players control saving explicitly.
- Implement these improvements in a future Codex development pass.
