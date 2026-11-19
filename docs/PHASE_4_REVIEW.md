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

## 4E: integration and self-review

Local Windows 10 verification: **270 tests passed**, formatting, strict all-target
Clippy and release build passed. The optimized executable is 4,947,456 bytes. The
full scripted walkthrough artifacts are retained under the integration worktree's
target/phase4-acceptance-ad81a3d9/. Every adventure definition was created through
the real CLI menus, including three connected rooms, key/locked door, conditional
prose/art, enemies/encounters, weapons, spell/progression, two currencies, six offers,
a fixed Mana Altar binding and its ID migration. Tests check wallet/MP arithmetic,
combat, persistent encounters, explicit save/load and exactly eight manual slots.
The script additionally raises/restores an offer's price through menus to verify
bound-object insufficient-funds behavior with no save writes. It restarts, loads,
exports, independently extracts and launches the actual packaged Windows executable
and Play.cmd from an unrelated working directory. Source playable bytes match.
The scaffold creator README is deliberately outside the export content allowlist.

Review fixes: retain a draft after deleting its last binding before prerequisite
navigation; select actual definitions rather than ignored optional references;
show definition names in migration selectors and plain-language offer summaries;
share the editable-path allowlist between publication and recovery; verify a
recovery source before removing its current target. Added direct interrupted-ID-
migration and unsafe-journal regressions. Unchanged read-only snapshot files do
not make disposable baseline cleanup read-only. No parser/combat arithmetic changed.

CI caught a test-environment issue: Ubuntu's default Python lacked tomllib after
Rust tests/checks/build had passed. Phase 4D now explicitly selects Python 3.12 in
both workflows. Python 3.11+ is needed only for the acceptance script, never play.
No Rust dependency or Cargo.lock change; bundled dependency notices remain current.

This is scripted automated acceptance and agent review, **not human beginner
certification or native CachyOS validation**. Human review/approval remains required
before merge or official release. No production-readiness declaration is made.

## Review stack and local testing

All worktrees are under C:\Users\Rex\.codex\worktrees\ and end in \Lupine.
Merge order is 4A -> 4B -> 4C -> 4D -> 4E. Each draft targets its immediate predecessor;
after merging a predecessor, retarget the next to main and inspect its diff/checks.
No branch was force-pushed and no PR was merged or release published.

| Phase | Draft PR | Branch / worktree folder | Checkpoint |
|---|---|---|---|
| 4A | [#26](https://github.com/LoneWolf862/lupine/pull/26) | codex/phase-4a-transaction-bindings / phase-4a-transaction-bindings | 3f26f9f6575152eb2db4ab9d6dbe7bae625a025b |
| 4B | [#27](https://github.com/LoneWolf862/lupine/pull/27) | codex/phase-4b-id-migration / phase-4b-id-migration | 41aac2e76af3991f3c543c80d76a4783cf022bf1 |
| 4C | [#28](https://github.com/LoneWolf862/lupine/pull/28) | codex/phase-4c-guided-recovery / phase-4c-guided-recovery | 510843ebbf6ab1ea15995ec3f79743a1fc682f55 |
| 4D | [#29](https://github.com/LoneWolf862/lupine/pull/29) | codex/phase-4d-zip-export / phase-4d-zip-export | 9e4110e (runtime checkpoint bfc4977e23107d99b3b18a7e9c62017ecdd4053f) |
| 4E | Pending draft | codex/phase-4e-integration-review / phase-4e-integration-review | Final integration checkpoint to be recorded |

Primary checkout remains on main at 2377af13cf7796ae477a1664eed0d6bf7a21c0bb,
with its preexisting personal untracked files untouched. Fetched origin/main was
502c7d12333f0970c16e202b63194b0b86418a53. Published v0.9.0 remains immutable.

In a dedicated PowerShell terminal (Python 3.11+ for scripted acceptance):

```powershell
Set-Location 'C:\Users\Rex\.codex\worktrees\phase-4e-integration-review\Lupine'
$env:RUST_TEST_THREADS = '1'
cargo fmt --all --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
python scripts/phase4_acceptance.py
```

The script prints a new disposable artifact directory each time. To inspect the
retained local walkthrough interactively with isolated saves:

```powershell
$demo = Join-Path (Get-Location) 'target/phase4-acceptance-ad81a3d9'
$env:APPDATA = Join-Path $demo 'userdata'
$env:XDG_DATA_HOME = $env:APPDATA
.\target\release\lupine.exe creator (Join-Path $demo 'Adventure with spaces')
```

Review T, M and E; decline confirmations once and verify drafts/files remain as
described. Play, navigate the locked door, fight/cast, use the altar, save explicitly,
quit/restart and load. Export ZIP to a fresh sibling directory, extract and launch
Play.cmd. For injected recovery fixtures use `cargo test --locked recovery::tests`;
never simulate corruption against personal adventures. Closing this dedicated shell
restores normal save-location environment for other sessions.

Next objectives: human beginner walkthrough; native CachyOS launch/history/export
acceptance; feedback-driven prerequisite navigation; optional ZIP compression if
distribution size warrants a dependency. Game identity, deletion migrations, RORK
migration and unsupported recovery records remain deliberately separate work.

## 4D: staged export and ZIP packaging

Creator E selects Windows x64, Linux x64 or Both, then ZIP or Folder. Both produces
separate platform archives. ZIP32 uses stored entries (no compression), UTF-8 names,
CRC-32, fixed timestamps and Unix executable attributes. The small writer follows
the [PKWARE specification](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)
without adding dependencies. Limits: 256 MiB per ZIP entry, 512 MiB per archive,
8192 package entries; source content retains the existing 8/64 MiB bounds.

Export builds and validates in a unique sibling .lupine-export-* workspace. ZIPs
are synced and read back with SHA-256 before final publication. Final destinations
are exclusively created; publication keeps EXPORT_INCOMPLETE.txt until all files
are linked and checked. Existing destinations are refused. No runtime downloads,
cross-platform relabeling, save packaging or source adventure changes occur.

Export > C removes only a completed workspace's recorded, hash-verified inventory.
Unknown/modified files, links and incomplete/missing records are refused. Crashes
before the cleanup record exists require manual review. Failed final publication
retains its incomplete marker; retry to a new destination. Hard-link support is
required, as for creator transactions. No broad recursive deletion is used.

Independent Python zipfile extraction tests cover CRCs, Unicode names, empty
directories, runtime bytes, exclusions and no overwrites. Unit tests exercise
size limits, write failure and conservative cleanup/retry. Both-platform tests
with synthetic foreign headers establish structure only, not foreign execution.
scripts/phase4_acceptance.py drives the actual native executable through full
menu authoring, binding, migration, gameplay, manual save/restart/load, ZIP export,
independent extraction and native launcher execution. Both CI workflows run it
on Windows Server 2022 and Ubuntu. Results remain pending until recorded below.

## 4C: guided editor recovery

Opening an adventure detects the existing .lupine-room-edit lock before play or
editing. New transactions retain a full baseline and versioned SHA-256 journal
before publishing. Recovery verifies confined regular paths, the whole recognized
content inventory, retained baseline/proposal bytes and full adventure validity.
Creators can inspect, restore the baseline or complete the proposal, with explicit
confirmation. Conflicting external edits, links, unknown recovery files, missing
data and corrupt records cause refusal with preservation/help instructions.

This extends Snapshot rather than creating another publication system. Recovery
can be retried after an interrupted target replacement or verified cleanup. A
resolved marker permits cleanup only when the full current inventory matches the
recorded outcome. Cleanup removes only verified files and empty known directories.
RORK files are never recovery targets. Original target permissions are retained.

Limits: older text-only records and crashes before a complete journal exists need
manual review. An interruption during the final journal/marker removal can also
need manual review. Stop other creator processes before recovery. This is not a
hostile-concurrency or power-loss atomicity guarantee. Baseline plus proposed data
can require roughly twice the 64 MiB snapshot size, in addition to displaced files.
Temporary staging and baseline copies may exist, but no authored file changes
before confirmation. Four failure-simulation tests cover multiple publication
stages, cancellation, external edits, missing/corrupt/unknown data and retry.
