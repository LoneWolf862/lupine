# File-based playtests

Run `lupine test ADVENTURE SCENARIOS.toml` with a standalone executable, or
`cargo run -- test story_example story_example/scenarios.toml` from source.
Every scenario starts fresh through the actual player parser, resolution, engine,
combat and rendering paths. No Rust or wizard is needed to author tests.

```toml
format_version = 1
seed = 1 # optional; default 1, nonzero, repeats random choices for the same inputs

[[scenario]]
name = "unlock the route"
setup = ["take brass key"] # optional input lines before the commands
commands = ["use brass key on workshop door", "north"]
contains = ["The key turns; the door unlocks."]
excludes = ["I don't understand"]
room = "workshop"
inventory = ["brass_key"]
flags = ["door_open"]
```

Names are unique. `commands` is required; all assertions are optional. `room`
checks final location. `inventory` and `flags`, when supplied, assert the entire
set (an empty list means none). Output assertions are case-sensitive substrings
over the whole session, including setup; they do not imply a particular ordering.
Each array entry is one input line. Include clarification answers as subsequent
entries. EOF ends the session without saving. `save`, `load` and `saves` inputs
are rejected before any scenario runs; scenarios do not access personal saves.

The command reports every assertion failure, includes its transcript and exits
unsuccessfully if any scenario fails. Invalid suite syntax/unknown fields are
errors. Use several small independent scenarios, including failure and cancellation
paths. A passing scenario tests only its authored assertions, not general solvability.

The suite file is an author tool, not adventure content; export excludes it.
Assertions cover prose, room, inventory and flags. Phase 2 additionally supports
`counters = { attempts=1 }` and `visited = ["hall", "workshop"]`, each comparing the
entire final collection when present. Normal play retains clock-derived randomness.
RORK version 2 is introduced by phase 2 for new story state, not by scenario testing.

Object scenarios can assert `hidden = ["note"]` (exact hidden set) and
`quantities = [{ item = "ration", location = { kind = "inventory" }, quantity = 3 }]`.
Each quantity assertion checks one location; unknown item IDs fail even for zero.
See `samples/objects_demo/scenarios.toml` for container and split-stack assertions.

Event scenarios can assert `turn=2`, `timers={deadline=3}` (exact active remaining
turns), `quests={delivery="active"}` (exact stage map), `journal=["instructions"]`
(exact acquisition order), `completed_events=["arrival"]` (exact once-event order),
`ended=false`, or `ending="success"`. See samples/events_demo/scenarios.toml.
