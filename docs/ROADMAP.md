# Lupine development roadmap

Milestones are design targets unless marked verified; HOWLBOUND remains a tentative original game without a decided story.

| Milestone | Status | Outcome |
|---|---|---|
| v0.1.0 | Complete, Windows | REPL, terminal parser, navigation. |
| v0.2.0 | Complete, Windows | Items, inventory, interactions. |
| v0.3.0 | Complete, Windows | Generic world model, conditions/effects, stateful descriptions. |
| v0.4.0 | Complete, Windows | Separate independent Rust game, terminal runner; commit `56db281`. |
| v0.5.0-A | Complete, Windows | Runtime-owned world/state; commit `97b9476`. |
| v0.5.0-B | Complete, Windows | External TOML story loading; commit `655a5df`. |
| v0.5.0-C | Complete, Windows | Validator, starter template, creator guide; commit `77d5bb4`. |
| v0.6.0-A | Complete, Windows | Versioned `.rork` serializer and validated state restoration. |
| v0.6.0-B | Complete, Windows | Built-in manual `save`, `load`, `saves`; multiple slots, cross-session gameplay verified. |
| v0.6.0-C | Complete, Windows | Staged write protection, compatibility revisions; 49 tests and gameplay verified; user reported commit. |
| **v0.7.0-A** | **Candidate / local test pending** | Text-only `new` adventure command, repeatable non-overwriting scaffolding. |
| v0.7.0-B | Planned | Terminal `add room`, `add item` generators and help improvements. |
| v0.7.0-C | Planned | Authoring usability tests, example refinement, cross-platform checks. |
| v0.8.0+ | Planned | Packaging and creator exports that do not require Rust for players. |
| v1.0.0 | Long-term goal | Stable engine and nonprogrammer-accessible, text-only authoring. |

**Non-negotiable:** save/load is engine-owned, completely manual. No autosave, checkpoint save, or save-on-quit. RORK `.rork` files remain TOML under the hood.

Windows verification does not establish Linux/CachyOS portability or completed standalone packaging. Keep the course's README template, video, and accurate work log separate.
