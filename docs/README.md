# Lupine documentation

These entry points describe development 0.11.0-dev.10, merged through PR #42.
The published v0.9.0 executable has fewer features. [Stabilization](STABILIZATION.md)
records current work and remaining acceptance; [engine expansion](ENGINE_EXPANSION.md)
preserves the implementation and verification history. Portable development packages
include this directory for offline reading in any text/Markdown editor.

| Audience | Start here | Next reference |
|---|---|---|
| Player | [Player instructions](../README.md#players-start-here) | [Installation and packages](INSTALLATION.md), [manual saves](SAVE_FORMAT.md) |
| Creator | [File-first quickstart and optional creator menus](CREATOR_QUICKSTART.md) | [Current content fields](CONTENT_REFERENCE.md), [scenario testing](SCENARIO_TESTING.md) |
| Engine developer | [Current architecture](ARCHITECTURE.md) | [Turn transactions and embedding](TURN_TRANSACTIONS.md), [requirements](REQUIREMENTS.md) |

## Which file does what?

| File type | Purpose | Who changes it? |
|---|---|---|
| `.toml` | Adventure definitions: rooms, items, NPCs, dialogue, abilities, rules, events, quests, shops and metadata | Creator |
| `.fig` | Game-wide settings using TOML syntax: character/progression, terminology and combat | Creator |
| `.rork` | Mutable player progress, stored outside the authored adventure | Engine, only when the player explicitly saves |

Each NPC has its own `NPCs/*.toml`; items and shared rules retain their own definitions.
IDs connect definitions and saves; changing filenames does not rename IDs. Never
edit RORK files to change authored content. An in-memory turn rollback is not an autosave.

## Authoring references

- [Shared conditions/effects](RULES.md) and [custom actions](ACTIONS.md)
- [Objects, containers and quantities](OBJECTS.md)
- [NPCs and dialogue](DIALOGUE.md)
- [Events, timers, quests and endings](EVENTS.md)
- [Economy, abilities, equipment and statuses](SYSTEMS.md)
- [All three FIG files](CONFIG_FIG_SPEC.md)
- [Project editing and recovery](PROJECT_EDITOR.md), [navigation](CREATOR_NAVIGATION.md)
- [Whole-turn failure behavior](TURN_TRANSACTIONS.md)

## Historical records and verification

Milestone reports, old handoffs and historical sections of requirements/roadmaps
record earlier states. Use the references above to determine current capabilities.
The [creator vision](CREATOR_VISION.md) retains the final accessibility goal;
complete-schema editing is not a claim of human beginner acceptance.

Windows and Ubuntu automated results belong to their recorded commit/PR. Native
CachyOS verification, human beginner testing and public release approval are separate
checks. Development CI packages are not a new public release.
