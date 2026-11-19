# Authored actions

Implemented in expansion phase 3. `actions.toml` defines commands without Rust or
parser edits. Actions use the same [conditions, effects and ordered outcomes](RULES.md)
as interactions. They belong to game content, not FIG settings.

```toml
[[actions]]
id = "unlock_door"
commands = ["unlock {target} with {item}", "turn {item} in {target}"]
room = "foyer"                 # optional; absent means wherever objects are accessible
target = "workshop_door"        # optional stable object ID
item = "brass_key"              # optional portable tool ID; must be carried
requires = [{ not = { flag_set = "door_open" } }]
effects = [{ set_flag = "door_open" }]
success_text = "The door unlocks."
failure_text = "The door is already unlocked."

[[actions.outcomes]]
when = [{ counter = { id = "unlocks", op = "eq", value = 0 } }]
effects = [{ set_flag = "door_open" }, { add_counter = { id = "unlocks", amount = 1 } }]
text = "The mechanism turns for the first time."
```

Define the rooms, items, flag and counter referenced here. Each action requires
`id`, `commands`, `success_text` and `failure_text`. `requires`, `effects` and
`outcomes` default to empty arrays. The first matching outcome replaces the outer
effects/text; otherwise the outer fallback runs.

Patterns are lowercase normalized text with literal words and zero, one or two
placeholders: `{target}` and `{item}`. Each bound ID must appear exactly once in
every pattern. Separate two placeholders with literal words. Examples:

- `pull {target}`, `read {target}`, `use {target}` for fixed or portable objects.
- `unlock {target} with {item}` for a carried tool and accessible target.
- `give {item} to {target}` or `put {item} in {target}` for explicitly authored
  exchanges using effects. NPCs and actual containers arrive in later phases;
  phase 3 targets are ordinary world objects.
- `wait` or `enter portal` with no placeholders, optionally restricted to a room.

This is a deterministic pattern grammar, not natural-language scripting. Names,
aliases, exact IDs and word-prefix matching use the standard resolver. Ambiguity
asks for a selection; cancellation, missing targets and failed requirements leave
state unchanged. An effect error rolls back the entire sequence and its output.
Multiple matching pattern layouts are refused rather than guessed; use distinct
literal verbs/separators for different layouts.

`{target}` may be fixed scenery and only needs to be accessible. `{item}` must be
portable and carried. Normal `use X on Y` also now permits a fixed accessible X;
portable X still must be carried. Ordinary item use and spell aliases keep their
existing precedence. `use {target}` is the only authored `use` pattern; an object
cannot have both that action and a transaction binding. Built-in commands and
their aliases, including manual saves, cannot be replaced by authored actions.

Limits: 1,024 actions per file, 1–16 patterns per action, 16 words/240 bytes per
pattern. Shared rule depth/list limits still apply. Unknown fields, missing IDs,
invalid placeholders and duplicate/overlapping bindings fail validation.

Help lists applicable action examples for accessible objects in the current room,
including actions whose requirements are currently unmet. Do not encode secret
phrases as public action patterns if revealing them in help would spoil a puzzle.
Typo suggestions are advisory. Enemy examination shares object clarification and
shows the authored description and approximate health, never exact HP. Games with
RPG disabled no longer advertise combat/spell commands in player help.

The creator's Snapshot, ID migration, recovery and export inventories include
`actions.toml`; a guided action editor is phase 8. The scenarios in
`samples/rules_demo` exercise both interaction and action commands. Actions use
existing mutable story state, so this phase does not change RORK version 2.
