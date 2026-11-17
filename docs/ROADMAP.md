# Lupine Development Roadmap

Milestones are aspirational unless explicitly marked verified. HOWLBOUND remains a tentative original game with no prescribed story.

| Milestone | Status | Result / planned objective |
|---|---|---|
| v0.1.0 | Complete, Windows | Basic offline terminal parser and navigation. |
| v0.2.0 | Complete, Windows | Items, inventory, and interactions; 10 tests. |
| v0.3.0 | Complete, Windows | Reusable world model, condition/effect rules, descriptions and tests. |
| v0.4.0 | Complete, Windows | Independent Rust-authored game and shared terminal runner; 17 tests; commit `56db281`. |
| v0.5.0-A | Complete, Windows | Runtime-owned world and state data; 18 tests, strict Clippy and interactive checks; commit `97b9476`. |
| v0.5.0-B | Complete, Windows | TOML story loading, 24 tests and interactive verification; commit `655a5df`. |
| **v0.5.0-C** | **Complete, Windows** | `validate` command, file-aware reference errors, starter template and creator tutorial; 28 tests; commit `77d5bb4`. |
| **v0.6.0-A** | **Candidate / testing** | RORK TOML serialization and full state validation, no terminal commands yet. |
| v0.6.0-B/C | Planned, required | Manual-only `save`, `load`, `saves`, `.rork` TOML save files, safe versioned state persistence. **Never autosave.** |
| v0.7.0+ | Planned | Text-only creator validation, terminal generators/templates and streamlined playtest workflow. |
| v0.9.0+ | Planned | Offline release packaging, creator export workflow, cross-platform verification. |
| v1.0.0 | Long-term goal | Stable usable engine and nonprogrammer-accessible text-based authoring. |

Final creator release requires a person unfamiliar with Rust to author, validate, playtest, and package a complete adventure without modifying Rust or using a GUI. A separate graphical Studio is **not planned**. See `CREATOR_VISION.md`.

Windows development validation is not a claim of native CachyOS/Linux release readiness. The course README template and video remain separate obligations. See `REQUIREMENTS.md`.

## v0.6.0-B implementation candidate (awaiting local verification)

- `save [name]`, `load [name]`, `saves`: reserved manual commands in the terminal runner.
- Per-game user-data save folders and `.rork` slot names.
- Confirm existing save overwrite and loss of unsaved progress.
- v0.6.0-C remains the planned filesystem robustness and game-compatibility stage.
