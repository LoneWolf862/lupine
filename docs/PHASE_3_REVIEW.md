# Phase 3 development review

Base: origin/main e976a88 (Phase 2I merged). Published v0.9.0 is unchanged.

## 3A startup

No arguments opens Play / Create or Edit / Help / Exit. Discovery checks the current
folder and executable folder, their immediate children, and adventures/ and samples/.
It is bounded and deduplicated; B accepts an adventure folder or another collection.
Selection reloads through Adventure::load before each fresh play. Invalid candidates
report loader diagnostics. quit returns to startup; EOF exits. Creator uses its existing
session and buffered streams. Existing CLI automation remains available; `fixture`
explicitly launches the Rust engine test world. No automatic loading or saving.

Tests cover menu navigation, help, creator return, numbered selection, browsing,
invalid directories/content, duplicate roots, cancellation, EOF, real CLI launch and
absence of save-directory creation. Native Linux/human acceptance is recorded separately.

Review finding: bound manifest reads and displayed titles as well as directory scans.
This is local discovery, not a recursive file browser. Missing/unreadable search roots
are skipped; an explicitly selected invalid adventure reports an error.

## 3B terminal editing

The executable wraps stdin with Rustyline 18.0.1 only when stdin/stdout are terminals
and TERM is not dumb. The existing generic BufRead APIs remain scriptable; redirected
input keeps the same buffered reader across menus and gameplay. The CLI shares up to
100 history entries across prompts in the current invocation (including creator/menu
answers). Empty lines and consecutive duplicates are omitted. History disappears on
exit. Up/down recall, left/right and backspace edit; Ctrl-C cancels the current line;
EOF exits. Terminal control bytes arriving through plain gameplay input are rejected.

Rustyline provides Windows/Unix editing and Unicode cursor handling instead of a new
platform-specific raw-key implementation. Default features are disabled: no file
history, directory helper, custom bindings or fuzzy-search dependencies. APIs checked
against https://docs.rs/rustyline/18.0.1/rustyline/ . Runtime stays offline.

Agent-driven Windows PTY check: look, Up recall, Down/Up navigation, replacing recalled
look with inventory using backspace, and quit worked. This is not human testing.
Self-review found and fixed a nested stdin-lock deadlock in fixture launch; fixture
now borrows the CLI reader. Windows locks running executables, so stop walkthroughs
before rebuilding. Generic input and in-memory history policy have regression tests.

## 3C parser and resolution

Normalization and the existing ordinary intent parser feed a pure name matcher, then
a read-only terminal preparation stage. Exact IDs take priority, followed by exact
names/authored item aliases and unambiguous substrings. Take/drop prefer relevant room
or carried candidates. Numbered clarification accepts a number or ID and cancellation;
all targets are resolved before existing gameplay validation/mutation. Enemies retain
instance IDs; explicit attack/cast and unambiguous use weapon/spell forms resolve to
the same existing mechanics. Item matches and ordinary item targets have priority over
spell aliases. Spells/enemies have ID/name matching; their existing schemas do not have
authored alias fields. Unlearned spells and hidden objects are excluded from suggestions.

Bounded edit distance includes adjacent transpositions (`exmain` suggests `examine`).
No suggestion executes. Reserved save/load arguments and confirmations remain separate.
RPG library entry points now reject ambiguous weapon/ability/teaching-item names instead
of selecting whichever HashMap entry happened to appear first. No RORK or gameplay
arithmetic changes. Pronouns and command chaining remain future work.

Tests cover clarification of both use targets, invalid selection/cancel/EOF, exact/partial
names, item/spell collisions, multiword aliases and case, transpositions, visibility,
enemy instances, RPG aliases and immutable preparation. The existing creator ambiguity
test now uses a noncolliding ID and explicit cancel, reflecting the requested new behavior.

## 3D offline export

Creator E opens Windows x64 / Linux x64 / Both. Enter selects the running executable
for its native platform. Other executable paths require an adjacent lupine.runtime.toml
from the same trusted build; `lupine runtime-manifest` emits this metadata on the native
platform. Export checks PE/ELF architecture, engine version, export protocol and SHA-256.
Metadata is an integrity/compatibility check, not a signature or authenticity guarantee.
No executable is silently downloaded or relabeled; foreign binaries are never executed.

Choose a new folder outside the adventure, preview targets and file inventory, then
explicitly confirm yes. Every binary is checked before writes. The immutable plan copies
only recognized content/config/art files, validates the packaged adventure and includes
the runtime, engine license, metadata, START_HERE.txt and Play.cmd/Play.sh. Empty standard
directories are preserved. Saves, target/, Git files, editor recovery and arbitrary root
files are excluded. Authors add their own story credits/license deliberately.

Output is a distribution directory with windows-x64/ and/or linux-x64/ subfolders.
Each is independently shareable; zip it with a normal archive tool if desired. Linux
launch via `sh Play.sh` restores executable permission after a Windows-created ZIP.
This is not cross-compilation. Compatible native runtimes must already be available.
Current official-style Linux builds require compatible x86_64 glibc systems.

Existing output folders, parent traversal, linked source content, recovery-in-progress
and unsafe output paths are refused. Export never edits source files. Handled publication
failure leaves EXPORT_INCOMPLETE.txt and an actionable message; retry into a fresh folder.
No recursive cleanup can delete unexpected files. Publication is not crash-atomic and
does not defend against hostile concurrent directory replacement. Limits: 8 MiB/content
file, 64 MiB/content snapshot, 256 MiB/runtime, bounded inspection/traversal.

Tests cover invalid content, missing/wrong/old/tampered runtime metadata, both-platform
structure, spaces, cancellation/EOF/decline, existing/unsafe destinations, source-byte
preservation, save exclusion and actual native packaged executable launch. Foreign header
fixtures test structure only; they are explicitly not evidence of native execution.
Self-review fixed missing required empty items/ and bounded reads against files growing
after metadata inspection; Windows launchers disable delayed expansion for ! in paths.

## 3E version, distribution and current documentation

Version recommendation implemented: **0.10.0-dev.1**. This substantial post-v0.9 player,
creator and export development warrants the next minor development version, not a claim
that the old release contains new features. Cargo.toml/Cargo.lock, --version, fixture/help
banners and runtime metadata agree. Portable artifact names derive from Cargo metadata.
The published v0.9.0 tag stays at e11cc024fa45029bb0bf145b4e6e7a77812c3005; no tag was rewritten
and no official release was created. RORK serialization remains version 1; no game save
compatibility values were changed. Historical version references remain historical.

README, AGENTS, architecture, roadmap/backlog, creator quickstart and parser status now
distinguish current development from historical records. INSTALLATION.md documents players,
creators, platforms, local runtime metadata, directory export, manual saves and limits.
Some older files contained mixed Windows-1252/UTF-8 punctuation; touched guides normalize
those bytes to UTF-8 without revising the historical milestone claims. CSE 310's instructor
README-template/submission evidence obligation remains explicit and unclaimed.

The portable workflow builds Windows Server 2022 and Ubuntu 22.04 native artifacts with
versioned names, runtime metadata and dependency notices. It runs the scripted native
export/ZIP/extract/play walkthrough. It does not publish a release. Notices are generated
from the locked crate sources by scripts/update_notices.py; identical license texts are
deduplicated. The clipboard-win crate omits its BSL license file, so the standard Boost
license text is retained under scripts/licenses/ with its source URL in the notices.

Final self-review caught a stable-ID reinterpretation hazard: resolving a named item to
ID `all` could become a bulk action, and ID `a` could be stripped as filler. Prepared
ordinary commands now preserve IDs; bulk requires an explicit original bulk intent.
A regression checks both cases and original parse errors remain errors. Launch acceptance
uses an empty unrelated working directory to prove discovery beside the packaged executable,
not accidentally find the source adventure.

### Acceptance procedure

`cargo build --locked --release`, then `python scripts/phase3_acceptance.py`. It creates
a disposable neutral fixture under ignored target/, never alters bundled/owner adventures,
and retains transcripts, source hashes, isolated saves, exported files, ZIP, extracted
package and summary.json. It drives the actual executable through no-argument startup,
three rooms, alternative phrasing, ambiguity cancellation/selection, typo suggestion,
manual save/load, return, creator navigation and export. A separate RPG transcript checks
physical and spell-use aliases and manual saves. The extracted native executable is run
from outside its package and directly against its packaged adventure. Every bundled
adventure is validated. This is scripted agent testing, not a human beginner result.

Live Windows PTY additionally recalled look with Up, navigated history boundaries and
Down/Up, moved Left/Right, deleted inside the recalled line to submit `lok`, received the
look suggestion, then recalled/inserted the missing letter and executed look. Quit returned
normally. No history file or automatic save was created. Native CachyOS/human acceptance
remains outstanding; exact CI and final checkpoint details follow below.

## Verification and handoff (2026-11-19)

Local host: LUPA-2, Windows 10 Pro build 19045, x64 MSVC toolchain, rustc 1.100.0
(0ef1bc386), Cargo 1.100.0 (3acc79c0c). Final runtime code checkpoint: **4537333**.
Checkpoint **455be00** adds only Linux PTY acceptance and its workflow step.

- **241 tests passed locally**, preserving the 219-test baseline and adding 22 tests.
  `cargo test --locked`, `cargo fmt --all --check`, and
  `cargo clippy --locked --all-targets -- -D warnings` passed after the final ID fix.
  RUST_TEST_THREADS=1 reduced filesystem contention on this host; no tests were skipped.
- Final optimized local Windows executable: **4,511,232 bytes**. Ubuntu artifact:
  **5,111,480 bytes**. Runtime has no networking or persistence-history features.
- [Windows Server 2022 and Ubuntu 22.04 checks at 4537333](https://github.com/LoneWolf862/lupine/actions/runs/44476139615)
  passed. [Both native build/package/ZIP-extraction walkthroughs](https://github.com/LoneWolf862/lupine/actions/runs/44476143554)
  passed, producing development artifacts, not an official release.
- At 455be00, [both platform checks](https://github.com/LoneWolf862/lupine/actions/runs/44476482350)
  and [both native portable builds](https://github.com/LoneWolf862/lupine/actions/runs/44476475842)
  passed, including all 241 tests and native exported-package acceptance. The Ubuntu job
  additionally passed Linux PTY acceptance: Up/Down, navigation boundaries, cursor
  movement, backspace, recalled edits and EOF.
- Previous phase CI passed on both platforms: [3A](https://github.com/LoneWolf862/lupine/actions/runs/44471765037),
  [3B](https://github.com/LoneWolf862/lupine/actions/runs/44472422212),
  [3C](https://github.com/LoneWolf862/lupine/actions/runs/44473736553),
  [3D](https://github.com/LoneWolf862/lupine/actions/runs/44474630106).
- Local final walkthrough artifacts are in the 3E worktree under
  `target/phase3-acceptance-1c725631/`: transcripts, summary.json, isolated manual saves,
  Export-Test-windows-x64.zip and Extracted package with spaces/. The extracted Play.cmd
  also ran successfully. Source adventure hashes stayed unchanged, and only the explicitly
  requested checkpoint and combat user-data RORK slots were created.
- A further `Both native platforms/` export used the real local Windows executable and
  the downloaded Ubuntu CI runtime. Export completed; the packaged Linux SHA-256 equals
  `53d74704f367ee4814b730e7e7637d8970f4e4e8db01fbeee662aa98417000d7`, matching the build
  artifact and metadata. Linux execution was verified in Ubuntu CI, not on Windows.

### Branches, worktrees and merge order

All worktrees are under the owner's `.codex/worktrees/` directory, each ending in `/Lupine`.
The primary checkout's tracked files, original HEAD aeb9e30 and preexisting untracked
personal files/adventures were left unchanged. Work started from fetched main e976a88.

| Order | Draft PR | Branch | Worktree folder | Target |
|---|---|---|---|---|
| 1 | [#20](https://github.com/LoneWolf862/lupine/pull/20) | codex/phase-3a-startup-menu | phase-3a-startup-menu | main |
| 2 | [#21](https://github.com/LoneWolf862/lupine/pull/21) | codex/phase-3b-terminal-history | phase-3b-terminal-history | Phase 3A branch |
| 3 | [#22](https://github.com/LoneWolf862/lupine/pull/22) | codex/phase-3c-parser-improvements | phase-3c-parser-improvements | Phase 3B branch |
| 4 | [#23](https://github.com/LoneWolf862/lupine/pull/23) | codex/phase-3d-adventure-export | phase-3d-adventure-export | Phase 3C branch |
| 5 | [#24](https://github.com/LoneWolf862/lupine/pull/24) | codex/phase-3e-release-documentation | phase-3e-release-documentation | Phase 3D branch |

Review the complete stack, then merge in that order. After each predecessor is merged,
retarget the next PR to main and review its diff/checks. The parser's final stable-ID safety
regression is in 3E, so include 3E before treating the combined series as ready to distribute.
All checkpoints were pushed normally; no force-push, self-merge or main push occurred.

### Manual Windows review

Use the 3E worktree and run `cargo run --release` (or its target/release/lupine.exe).

1. Play: try invalid selection, B with an invalid path, back, then open an adventure.
   Quit and confirm return to startup. Open Creator, leave it and confirm return.
2. In a disposable fixture with two visible keys, try `take key`; cancel, retry and choose
   a number. Try `pick up the brass key`, `exmain door` and a missing object. No failed
   command should alter inventory or flags. Test Up/Down, cursor edits and empty history
   after restarting the executable.
3. Open samples/rpg_persistent. Take/learn the fireball scroll, take the dagger, travel
   south and try attack/cast/use forms. Enemy HP stays approximate. Save and load manually;
   quit with unsaved changes and decline once. Verify no implicit save appears.
4. In Creator E, cancel/decline export, try a missing foreign binary, then export Windows
   to a new sibling folder. ZIP, extract elsewhere and double-click Play.cmd. Launch the
   included executable from an unrelated working directory and select its adventure.
5. Inspect source files and exported inventory: no source changes, personal RORK saves,
   Git or target/ files. Existing export destinations must be refused. Both-platform export
   needs the matching local Linux runtime/metadata from the development artifact.

### Remaining limits and next goals

No required Phase 3 feature is intentionally deferred. Export produces directories;
ZIP creation is external. Metadata verifies consistency, not authenticity. Failed export
retains an incomplete folder; editor recovery remains manual. History spans all prompts
in one invocation. Spell/enemy aliases are IDs/names because those schemas have no authored
alias field. Pronouns, chaining, ID migrations and transaction-binding authoring remain
separate work. Runtime files/config/save formats remain unchanged except the requested
player behavior and development package metadata.

Human beginner testing and native CachyOS testing are **not claimed**. Recommended next
steps are human Windows review of this stack, native CachyOS history/export acceptance,
then transaction-binding authoring, deliberate ID migration and guided recovery. An official
release needs separate human approval; the CSE 310 template/time-log/video obligations
remain separate and no submission evidence was invented.
