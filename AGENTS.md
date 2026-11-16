# Lupine contribution rules

- This repository is the **Lupine** Rust interactive fiction engine, not the narrative game.
- **HOWLBOUND** is a tentative name for a future game; do not invent its plot, setting, rooms, characters, or lore.
- Keep the engine deterministic, offline, terminal-only, lightweight, and independent of story content.
- Keep parsing separate from object resolution, world rules, and state mutation.
- Develop in small increments, add tests, and preserve Windows/Linux portability.
- Prefer the Rust standard library unless a dependency solves a specific problem.
- Read `docs/REQUIREMENTS.md` and `docs/ARCHITECTURE.md` before significant changes.
- Do not implement optional features ahead of the current milestone without approval.
