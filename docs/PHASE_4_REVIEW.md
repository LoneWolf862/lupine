# Phase 4 development review

Baseline: fetched origin/main 502c7d12333f0970c16e202b63194b0b86418a53.
Local host: LUPA-2, Windows 10 build 19045, Rust/Cargo 1.100.0.
Development remains 0.10.0-dev.1. No release, merge or save-version change.

## 4A: transaction bindings

Creator T (also R > 8) lists, creates, edits and removes object bindings using
transaction_interactions.toml. Selection filters duplicate bound objects and shops
outside the object's starting room. Offers come from the selected shop. Runtime
validation remains authoritative. Success/failure overrides can be set or removed;
default failure messages explain the actual reason. No offer logic is duplicated.
Exact previews, separately confirmed Save and full staged adventure validation use
the existing Snapshot publication path. Unedited comments and inline lists survive.
Prerequisite I/H managers save separately before starting a draft; an active draft
is retained with guidance to save/discard it before entering those managers.

Tests cover editing, removal, recreation, defaults, comments, duplicate prevention,
declined Save, EOF and no player-save creation. The actual CLI RPG walkthrough now
creates and activates an object binding, preserving wallet/MP and manual-save
assertions. This is scripted automated acceptance, not human beginner testing.

## 4B: internal-ID migration

Creator M supports rooms, items, abilities, enemies, shops, shop-scoped offers,
currencies (including implicit gold IDs), and referenced flags. Weapons have no
separate ID: migrating an item updates its physical weapon association. Interaction
and binding records have no independent IDs. Game identity, filenames/art paths,
runtime instance IDs and RORK migration are explicitly excluded.

The reference inventory follows explicit schema paths, including conditions,
effects, art/prose variants, encounters, loot, rewards, teaching items and offers.
It never substitutes narrative strings or comments. Original filenames remain.
Review lists every changed field and exact file bytes, warns about saves, stages
and validates the complete result, then requires confirmation. Publication rechecks
the snapshot and reuses the existing multi-file rollback path. No save version is
bumped: creators must deliberately choose their game's compatibility policy.
Old flags may remain loadable but change behavior, so compatibility is never promised.

Five focused tests cover all categories, inline bindings/implicit currency IDs,
cross-file puzzles/art, invalid IDs, collisions, cancellation, stale files, unchanged
RORK bytes and reopening. The executable creator acceptance migrates its bound
object before gameplay and verifies the new binding. Shared publication tests cover
invalid dependencies and interrupted multi-file publication.

## Remaining assignment

4C guided recovery, 4D ZIP export and 4E integration review remain
in progress. Cross-platform CI and final release-readiness findings are pending.
