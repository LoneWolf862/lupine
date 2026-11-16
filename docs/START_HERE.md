# Handoff — Start the Rust Terminal Adventure Framework

## Project context

We are building a **new, original terminal-based interactive fiction game engine**, inspired by the command-driven feel of Zork, **not its story or content**. The story, plot, and setting are intentionally undecided and will be supplied later. The most important deliverables are a small, reusable engine and a forgiving deterministic parser.

Read `REQUIREMENTS.md` for mandatory behavior and CSE 310 Rust grading criteria. Read `ARCHITECTURE.md` for recommended structure and development sequence. Treat them as the initial design baseline; make straightforward architecture decisions without expanding the scope.

## Suggested first task for the new chat / coding agent

> Please help me bootstrap this Rust/Cargo project and implement the **first playable framework milestone**. Read `REQUIREMENTS.md` and `ARCHITECTURE.md` first. Keep the story generic, temporary, and completely separable from engine logic; do not decide the final plot, lore, or setting for me. Start with the smallest reasonable project structure, a terminal REPL, typed commands, `help`, `look`, movement between a few fixture rooms, and `quit`. Add automated tests for parser normalization and navigation. Explain briefly which Rust language features demonstrate the CSE 310 rubric. Work in small, verifiable increments. Do not introduce an LLM, web services, graphics, or runtime networking. The resulting application must work fully offline.

## Suggested next task (after milestone 1 works)

> Extend the existing engine with inspect, take, drop, inventory, multiword object names/aliases, forgiving command synonyms and articles, ambiguity handling, and a simple stateful interaction. Keep parser, object resolution, world rules, and story content separate. Add tests and avoid large dependencies.

## Course rubric quick check

Mandatory: mutable and immutable variables; loops; functions showing ownership transfer or borrowing; `Vec`/`vec!`; `match`. Stretch: choose at least one of `struct`, function return type `Option`/`Result`, or `enum`. The design should naturally demonstrate all three stretch options, but the class only requires one.

Course deliverables additionally include public GitHub source, instructor-template README, working demo, video walkthrough, and sprint planning/time documentation. The professor has approved counting eligible work already performed this semester as part of the self-contained two-week sprint.

## What not to do

- Don't recreate Zork's rooms, writing, or puzzles.
- Don't choose the final story setting on my behalf.
- Don't build a giant natural-language processor or install a full AI model.
- Don't require an active network connection during gameplay.
- Don't over-engineer an authoring language before basic gameplay exists.
- Don't trade parser usability for features that only satisfy an assignment checklist.
