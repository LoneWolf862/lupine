# Lupine v0.8.4-A — Core Command and Display Polish

This incremental update addresses four player-facing issues:

- Living enemies always display at least one filled segment of their approximate health bar. Exact HP stays hidden.
- `spells` aliases `spellbook`.
- Untargeted damage spells display current, actionable syntax rather than an obsolete roadmap message, without spending MP.
- `take all` gathers every portable object in the current room; `drop all` drops carried objects there. Fixed scenery, consumed items, and objects in other rooms are excluded.

## Remaining for v0.8.4-B

- Formal player-death/defeat lifecycle and safe recovery.
- Typo suggestions, unambiguous enemy short-name aliases.
- Story-authoring guidance for conditional room descriptions (static text may mention removed items).
- Expanded regression testing and creator documentation.

No gameplay autosaving. These changes preserve RORK v1 and require no new story fields.
