# Public releases

## v0.11.0-dev.12 — current-version prerelease

The creator authorized publication on November 21, 2026. This release uses the
current Cargo version unchanged; the `dev.12` suffix and GitHub prerelease label
make its development status explicit. The v0.9.0 tag and assets remain unchanged.

[Public release and downloads](https://github.com/LoneWolf862/lupine/releases/tag/v0.11.0-dev.12)
provide Windows x64 ZIP and Linux x64 tar.gz packages, with SHA-256 checksums.
These assets require no account and do not have Actions artifact expiry.
Extract the whole archive and open the enclosed Lupine folder. Windows launches
with `lupine.exe`; Linux launches with `./lupine`. No Rust installation is needed.

Compared with v0.9.0, this release includes the guided Adventure Creator, offline
export, file-authored rules/actions, containers, dialogue, quests, economy and
the eight-room mini adventure. The latest playthrough follow-up adds player
naming, persistent conversations, clearer navigation and trading, parser
improvements and more readable terminal output. See the [walkthrough](MINI_ADVENTURE.md)
and [playthrough verification](PLAYTHROUGH_REVIEW.md).

RORK remains 7. The mini adventure uses content compatibility revision 2.
Older saves require their matching engine/adventure package; this release does
not migrate or automatically rewrite saves. Saving is always manual.

Windows and Ubuntu native CI cover Rust tests, formatting, strict Clippy,
independent consumers, documentation, creator/export tests and full packaged
mini-adventure play/save/restart/export. Linux also exercises a real terminal.
Native CachyOS and broader human beginner acceptance remain unverified; public
release approval does not imply those tests were performed.

## Maintainer procedure

1. Obtain creator approval for the version and publication. Prepare documentation
   and permanent versioned download URLs on an isolated branch and reviewable PR.
2. Run the checks above. Merge the reviewed release preparation, then run
   **Lupine portable builds** on that exact commit with **release_package=true**.
   The workflow only assembles and tests assets; it cannot publish a release.
3. Both native jobs must pass. Download their release artifacts. Each contains
   a platform archive and `SHA256SUMS-PLATFORM.txt`. Check the archive digest,
   native runtime metadata and the recorded commit. Never relabel an executable
   for a different OS. Package assembly compares every archived file with the
   validated source package and preserves the Linux executable bit.
4. Create a draft GitHub release/tag at the verified commit. Use the exact Cargo
   version prefixed with `v`; mark development versions as prereleases. Upload
   both archives and checksums, and write release notes with compatibility and
   platform limitations. Exclude local saves, transcripts and incomplete packages.
5. After verifying the draft inventory, publish under the creator's authorization.
   Test both versioned download URLs without authentication, verify downloaded
   checksums and check the README links. Do not rewrite a published tag or replace
   old assets. Use a new version for later changes.

Local assembly uses `python scripts/package_development.py --release` after a
native `cargo build --locked --release`. It refuses existing destinations,
validates copied games/scenarios and documentation, and writes archives/checksums
beside the package. This command does not upload or publish anything.
