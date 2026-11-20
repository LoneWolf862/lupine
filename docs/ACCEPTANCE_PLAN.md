# Complete-adventure and platform acceptance

## Scope and decision

Stabilization is merged through PR #43 at 7bcaa24, version 0.11.0-dev.10.
This plan covers documentation, automated verification and a human test handoff.
It does not authorize merging this follow-up or publishing a release. v0.9.0,
RORK 6 and game save-compatibility revisions remain unchanged.

The next product milestone is one small, complete creator-directed adventure
authored, revised, tested and exported using public Lupine features. Its content
and mechanics need the creator's direction; HOWLBOUND lore is not supplied here.
Existing neutral samples support the checks below but do not count as completing
that original adventure or as human beginner acceptance.

## Evidence and acceptance gates

Record each check as PASS, FAIL, BLOCKED or NOT RUN. Missing evidence is NOT RUN,
not a pass. Record the source commit/artifact run as well as `lupine --version`:
multiple development commits share the same version string.

| Gate | Automated evidence | Human evidence still required |
|---|---|---|
| Engine correctness | Rust tests, formatting, strict Clippy, independent Rust consumer | Unexpected behavior found during real play |
| File-authored behavior | Bundled scenario suites through the real player runner | A complete creator-directed adventure and intended/failed paths |
| Native packaging | Runtime digest/target/version, 10 bundled adventure validations, ZIP extraction and launcher | Actual Windows 10 and CachyOS terminal/desktop environment |
| Persistence | Manual-save/restart checks and no unexpected saves in isolated test data | Visible naming, overwrite and unsaved-progress prompts in a real terminal |
| Authoring | Scripted creator, migration, export and recovery regressions | Beginner comprehension, assistance needed and useful error messages |
| Documentation | UTF-8 and local file links in checkout and package | Instructions understandable without Rust; heading anchors need manual review |

The merged baseline passed [Windows Server 2022 and Ubuntu 22.04 CI](https://github.com/LoneWolf862/lupine/actions/runs/44663495551).
That does not establish Windows 10, CachyOS or human usability acceptance.
Follow-up results belong to its draft PR and exact checked commit; preserve CI
evidence and development packages before their 14-day artifact expiry.

## Automatic verification procedure

From a source checkout, with Rust/rustfmt/Clippy and Python 3.12 available, run
each command and stop on a nonzero exit. These are existing checks, not new gates
that require players to install a toolchain.

```text
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo check --locked --manifest-path samples/independent_game/Cargo.toml --target-dir target/independent-consumer
python scripts/check_docs.py
cargo build --locked --release
python scripts/phase4_acceptance.py
python scripts/package_development.py --output dist/acceptance-plan-native
```

Use a different new package destination if that path exists. Do not overwrite
prior evidence. On native Linux also run `python3 scripts/history_pty_check.py`.
Normal PR CI runs the checks on Windows/Ubuntu. The separate manually dispatched
portable-build workflow can produce Windows/Linux test artifacts from the same
branch; this is not a GitHub Release.

The acceptance script isolates user data, creates a disposable project, exercises
eight explicit save slots and restart/load, validates samples, checks source/save
bytes, independently extracts an exported ZIP and runs its native launcher from
an unrelated working directory. Preserve its `target/phase4-acceptance-*/`
transcripts and summary. Do not upload personal saves.

Also run every bundled `scenarios.toml` using the packaged native executable:
`lupine test ADVENTURE ADVENTURE/scenarios.toml`. Current suites are in
story_example and samples/rules_demo, objects_demo, events_demo, dialogue_demo,
economy_demo. Scenario tests cannot read/write player saves. They assert final
state and transcript substrings, not every intermediate state or output ordering.
Use several short scenarios rather than interpreting one pass as general solvability.

## Human setup: same build, disposable data

Use the Windows x64 and Linux x64 development artifacts from the same successful
workflow run identified on the draft PR. Extract the full package, including
`lupine.runtime.toml`, docs and samples. Do not use the older v0.9.0 release.
No Rust, Python or network access is required for the human checks.

Open a dedicated terminal in the extracted package directory. These commands
create a new isolated save directory for child processes launched from that
terminal. Keep the path with your results, then close the terminal when finished.
Do not launch a separate desktop shortcut for these checks: it would not inherit
the temporary save environment. Preserve your normal saves and authored games.

Windows PowerShell (Windows 10 x64, or record the actual OS if different):

```powershell
$acceptanceData = Join-Path $env:TEMP ('lupine-acceptance-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $acceptanceData | Out-Null
$env:APPDATA = $acceptanceData
Write-Output $acceptanceData
.\lupine.exe --version
.\lupine.exe
```

CachyOS terminal (native x86_64 Linux, not Ubuntu CI or WSL):

```sh
export XDG_DATA_HOME="$(mktemp -d -t lupine-acceptance-XXXXXXXX)"
printf '%s\n' "$XDG_DATA_HOME"
chmod u+x ./lupine
./lupine --version
./lupine
```

After leaving Lupine, list the isolated saves with
`Get-ChildItem -LiteralPath $acceptanceData -Recurse -Filter *.rork` on Windows,
or `find "$XDG_DATA_HOME" -type f -name '*.rork'` on Linux. Quit without saving
before the save tests; the first listing should be empty. An unexpected RORK
file is a failure to investigate, not something to delete to make the check pass.

## W/L: native player checks on both systems

Use the same checklist on Windows and CachyOS, recording results separately.
For reproducible commands, select `samples/rules_demo` via startup Play > B,
or run `lupine play samples/rules_demo` with the platform executable prefix.

1. **W/L-1 Startup and input:** menu, help and room text are readable. Type `loox`,
   use Backspace to replace x with k, submit `look`, then recall/edit commands
   with Up/Down and Left/Right. Resize the terminal and repeat. No missing prompt,
   duplicated input or erased question should prevent use.
2. **W/L-2 Parser and state:** `north` is blocked; `take workshop door` refuses.
   `lok` reports an unknown verb without executing look or lock. `take brass key`,
   `use brass key on workshop door`, then `go to the north` reaches the workshop.
   Record any natural phrase you expected to work and the exact response.
3. **W/L-3 Save prompts:** in a fresh session enter bare `save`, name the slot
   `acceptance one`. Bare `save` now offers overwrite; decline and create
   `acceptance two`. Try bare `save`, decline, then cancel at the name prompt:
   no third file should appear. `save acceptance one` writes directly without
   a second overwrite question. All questions remain visible above input.
4. **W/L-4 Restart and load:** explicitly save after unlocking/reaching the
   workshop. Quit, accepting an unsaved-progress warning only when you intend
   to discard changes. Restart: play begins fresh in the foyer, not from a save.
   `load acceptance one` (confirm discard if prompted) restores the saved state.
   Make another unsaved change and quit/restart; loading the slot should still
   restore the last explicit save, not the discarded progress.
5. **W/L-5 Cancellation and combat:** in a disposable copy of
   `samples/rpg_spellbook`, obtain a weapon and encounter an enemy. Enter
   `attack ENEMY` using its displayed name, then cancel weapon selection.
   No attack, retaliation or resource loss should occur. A confirmed attack
   works and ordinary output does not disclose numeric enemy HP. Retest the
   original player's troublesome phrases and save interactions here too.

Record actual actions/results when the encounter varies; do not claim this is
a deterministic transcript. Automated seeded regressions provide that coverage.

## C: creator and export walkthrough

Have someone unfamiliar with Lupine attempt C-1 through C-5 using the shipped
docs. Record every explanation or intervention. An experienced maintainer's
successful run is useful functional evidence but is not beginner acceptance.

1. **C-1 Create/revise:** create a disposable adventure in a new folder with
   spaces in its name. Add a room, connect it, add a portable item and a simple
   interaction. Use the person's own test text, not prescribed HOWLBOUND content.
   Preview changes, cancel one edit, confirm another, reopen and verify only
   confirmed changes persisted. File-based TOML/FIG authoring is primary;
   guided menus are optional. Record the workflow used.
2. **C-2 Diagnose/repair:** in that disposable project, deliberately change an
   exit to a nonexistent room ID using a text editor. Validate: expect a useful
   error identifying the reference, then restore the correct ID and validate
   successfully. Confirm the creator can find and repair it using the docs.
   Do not inject corruption or interrupted recovery into a personal project.
3. **C-3 Play/test:** walk the intended route and a failed interaction. Create
   at least one scenario using the scenario guide, run it, deliberately make
   an assertion wrong and observe a failure, then correct it. Gameplay changes
   remain in memory until an explicit player save; author files are separate.
4. **C-4 Export:** open Creator > E, choose the native platform and ZIP, accept
   the running engine, select a new destination outside the adventure, inspect
   the inventory and explicitly confirm. Extract into a different path with
   spaces. From the isolated-data terminal run `& 'PATH\Play.cmd'` on Windows
   or `sh 'PATH/Play.sh'` on CachyOS, replacing PATH with the extracted directory.
   Play and save/load there, without Rust or network access. Check that source
   files are unchanged and no RORK files were included in the exported archive.
5. **C-5 Handoff:** give the native export to another player, who extracts and
   launches it using the included instructions. Record assistance, missing files,
   permissions problems or confusing wording. On CachyOS specifically test
   `sh Play.sh` after ordinary ZIP extraction, which may lose executable bits.

For a cross-platform export check, supply the other OS's executable and matching
metadata from the same artifact run; choose Both. Launch each output only on its
actual target OS. Record this check separately as PASS/FAIL/NOT RUN. Export must
never download a runtime or relabel one binary as another platform.

## Complete original adventure: subsequent creator-directed work

Agree on a small playable scope with a clear start and ending, then select only
mechanics that serve it. Include a meaningful blocked/failed path and a revision
after playtesting. Author through public files/tools without story-specific Rust
changes. Add scenarios for intended and failed paths, verify manual save/load at
meaningful points, export and have another person finish it. Keep engine defects,
authoring usability problems and story-design decisions separate in the findings.
This is a future acceptance gate, not content implemented by this documentation PR.

## Report template and follow-up

Copy this into your reply or a local test note; one record per platform is enough:

```text
OS/version and architecture:
Terminal name/version (and shell):
Artifact workflow URL / source commit:
lupine --version:
Tester experience / assistance:
W/L-1 through W/L-5: PASS / FAIL / BLOCKED / NOT RUN, with notes
C-1 through C-5: PASS / FAIL / BLOCKED / NOT RUN, with notes
Cross-platform export: PASS / FAIL / BLOCKED / NOT RUN
For each failure: exact command/steps, expected, actual, relevant output
Evidence paths / screenshots if useful:
```

Do not post personal RORK files or private story content. Fix confirmed failures
in isolated PRs and rerun affected checks. Native and human acceptance stays open
until these reports exist. Choose further guided authoring or scenario-tool work
from the findings. Release approval and CSE 310's instructor README, honest work
log and submission evidence remain separate decisions/deliverables.
