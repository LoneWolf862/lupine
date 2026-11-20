# Post-expansion stabilization

## Baseline and scope

All development through PR #42 is merged at main 8f9782f, development version
0.11.0-dev.10. Its source tree is identical to final PR #42, which passed Windows
and Ubuntu checks, including the 374-test suite, strict Clippy, formatting, release
builds and native package acceptance. [Original CI evidence](https://github.com/LoneWolf862/lupine/actions/runs/44632403701).
The [engine expansion record](ENGINE_EXPANSION.md) preserves the original milestones.

This stabilization candidate updates current status and UTF-8 documentation, adds
offline documentation/templates to native development packages, strengthens routine
CI and corrects the consumable movement reachability warning. It does not change
gameplay, RORK 6, game compatibility revisions or the published v0.9.0 release.
Changes are reviewed in an isolated branch and draft PR; no automatic merge/release.

## Implemented in this candidate

- Author diagnostics recognize MovePlayer effects on consumables and continue
  following exits from those destinations. Truly disconnected rooms still warn.
  This is advisory graph analysis, not proof that a puzzle is solvable.
- PR and main-push CI covers Windows/Ubuntu tests, formatting, strict Clippy,
  independent Rust consumer compilation, documentation, release builds and native
  creator/manual-save/export acceptance. Linux also runs the PTY history/prompt test.
- The shared package script assembles a new native package with a matching runtime
  manifest, README, licenses, offline docs, starter templates and playable examples.
  Only tracked content is copied. It checks copied bytes, the runtime digest,
  Markdown encoding/local file links and packaged adventure validation. Existing
  destinations are refused; interrupted assembly keeps PACKAGE_INCOMPLETE.txt.
- Verification workflows retain acceptance transcripts/summaries and the Linux PTY
  transcript for 14 days, including after a later step fails. Player saves are not
  uploaded as evidence. Development packages remain a separate manually requested
  artifact; normal PR verification does not publish a release.

## Verification

Local Windows verification passed: all 375 tests (including the new consumable
route regression), formatting, strict all-target Clippy, the locked independent
Rust consumer and a locked release build. Native creator/manual-save/ZIP-launch
acceptance passed with eight explicitly saved slots and unchanged source content
(target/phase4-acceptance-466d29b5). No automatic saves occurred.

The native development package passed runtime digest/version checks, all 10 bundled
adventure validations and UTF-8/local-file-link checks across 66 Markdown documents.
Additional probes confirmed that an untracked synthetic RORK file is excluded,
existing destinations are refused without changed bytes, and missing/non-UTF-8
documentation is detected (target/package-probes-f352e0ab/summary.json). These are
scripted checks, not human beginner acceptance. Markdown anchors are not checked.

Final-head Windows/Ubuntu CI results are recorded on [draft PR #43](https://github.com/LoneWolf862/lupine/pull/43).
Native CachyOS and human beginner acceptance remain unverified. No public release
or merge is part of this work.

Maintainer commands (Python 3.12; Rust toolchain with rustfmt and Clippy):

```text
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo check --locked --manifest-path samples/independent_game/Cargo.toml --target-dir target/independent-consumer
python scripts/check_docs.py
cargo build --locked --release
python scripts/phase4_acceptance.py
python scripts/package_development.py
```

On native Linux also run `python scripts/history_pty_check.py`. Packaging requires
a new output directory; use `--output NEW_PATH` for a second build. Never delete
unreviewed acceptance evidence or personal saves just to rerun a command.

## Remaining acceptance and next decisions

1. Have a beginner create/revise a small adventure, diagnose an error, playtest,
   export and hand the package to another player. Record assistance and obstacles.
2. Retest the original player's parser/save interactions on Windows 10 and perform
   native CachyOS terminal/export acceptance. CI on Ubuntu does not establish that.
3. Exercise a complete small creator-directed adventure and its scenario suite.
   HOWLBOUND content remains the creator's decision; no setting or plot is assumed.
4. Use those findings to prioritize guided authoring. Structural/text access to
   every schema is not evidence of beginner accessibility.
5. Separately approve a prerelease, its platform scope and save expectations.
   Keep v0.9.0 unchanged. CSE 310's instructor README template, honest work log and
   submission evidence remain separate obligations; no completion is claimed.

Optional later work: pronouns/chained commands, ZIP compression, scoped history,
per-step scenario assertions, larger-world measurements and a simpler embedding
API. None is implemented by this patch or required merely to declare it verified.
