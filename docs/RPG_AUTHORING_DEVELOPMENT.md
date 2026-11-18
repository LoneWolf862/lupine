Phase 2G/2H plan: enemies, weapons, shops, currencies, integration and real CLI acceptance.
Each phase uses an isolated stacked feature branch and draft PR; no self-merge.
Base 417b496. Lupa-2 Windows 10 build 19045, Rust 1.100.0 MSVC.
Reuse Document, Wizard, rules_ui and Snapshot; runtime schemas remain authoritative.
Keep stable IDs, exact previews, explicit confirmation and manual-only RORK.
Run locked tests, fmt and strict Clippy locally; report CI platforms separately.
