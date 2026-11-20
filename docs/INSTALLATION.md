# Installing, playing and exporting Lupine

## Pick the right build

The published v0.9.0 release is historical and unchanged. The merged engine expansion
and creator features belong to 0.11.0-dev.10 development builds, available after building this source or downloading
the matching manually dispatched GitHub Actions artifact. A development artifact is
not an official release. Windows x64 and Linux x64 are separate executables.

Windows targets Windows 10 x64 and compatible newer Windows. Linux workflow builds on
Ubuntu 22.04 with glibc; a compatible x86_64 glibc distribution is required. Native
CachyOS acceptance is not claimed. No macOS or ARM distribution is offered by export.

Extract the entire artifact. Windows: run lupine.exe, or Play.cmd in an adventure export.
Linux: open a terminal in the extracted folder, `chmod u+x lupine`, then `./lupine`;
an adventure export also supports `sh Play.sh`, including after ZIP extraction loses
the executable bit. The terminal stays the interface; there is no graphical editor.
Players need no Rust installation, account or internet.

Portable development packages include README.md, docs/, templates/starter/,
story_example/ and playable samples/. Start at docs/README.md for offline player,
creator and developer references. Open Markdown files with a text editor if no
Markdown viewer is installed. Instructions mentioning Cargo or source files apply
to a source checkout; standalone authoring uses the included executable instead.
Development artifacts expire after 14 days on GitHub Actions; keep a local copy
of the matching runtime and its metadata when exporting for another platform.

Maintainers assemble the same verified native package locally with
`python scripts/package_development.py` after `cargo build --locked --release`
(Python 3.12 is used in CI). It writes a new dist/Lupine-VERSION-PLATFORM directory,
checks offline documentation and runs validation on the copied examples. An existing
destination is refused; `--output NEW_PATH` selects another. Only tracked documentation,
templates and playable examples are included. PACKAGE_INCOMPLETE.txt marks an
interrupted assembly and must not be distributed. This command never publishes a release.

## Startup and play

Launch without arguments, choose 1, then an adventure number. Discovery checks the
current directory, executable directory, their immediate children, and adventures/
and samples/. B accepts an adventure folder containing game.toml, or a folder holding
several adventures. Paths with spaces need no surrounding quotes at the interactive
prompt. Advanced shell commands do require normal shell quoting: `lupine play "My Game"`.
0/back/cancel returns from selection. Invalid content reports a filename/diagnostic.

Every play starts fresh. `quit` returns to the launcher; EOF exits. History exists only
during this invocation and is shared across its prompts. Up/Down recall, Left/Right and
Backspace edit; Enter submits. Ctrl-C cancels the editing line. A dumb terminal or
redirected stdin/stdout uses plain line input. The parser never executes a typo suggestion.
Numbered target clarification supports cancellation without spending a turn or resource.

Manual saves: `save name`, `saves`, `load name`. Windows uses `%APPDATA%\Lupine\saves`;
Linux uses `$XDG_DATA_HOME/lupine/saves` or `~/.local/share/lupine/saves`. Names are isolated
by stable game ID. Bare `save` offers to overwrite the last successfully saved/loaded slot
in this play session; no asks for a name. With no current slot, it asks for a name directly.
`save name` writes that named slot without another question, replacing the same name.
Save format and game
compatibility versions are independent of the engine version. No save happens on exit.

## Export an adventure

1. Open it in the Adventure Creator and choose E.
2. Select Windows x64, Linux x64 or Both.
3. Select Ready-to-share ZIP or Uncompressed Folder.
4. For the host OS, Enter uses the running engine. For another runtime, enter its file path
   and keep the matching `lupine.runtime.toml` beside it. Both must come from the same
   trusted build and match the exporting engine version.
5. Enter a new destination folder outside the source adventure. Its parent must exist.
6. Review the platforms, format and file inventory, then type yes to publish.

Folder output contains windows-x64/ and/or linux-x64/. Each has adventure/, the engine,
runtime metadata, Play.cmd or Play.sh, START_HERE.txt, engine license and dependency
notices. ZIP output contains a separate Adventure_Name-PLATFORM.zip for each selected OS.
Extract it before running Play.cmd or `sh Play.sh`. ZIP uses stored entries without
compression, Unicode names and empty required directories. Limits are 256 MiB per entry
and 512 MiB per archive; source content has the existing 8 MiB/file and 64 MiB total limits.
Share the ZIP or whole platform folder. Personal RORK
saves, source build directories, Git files and editor recovery data are not included.
Add your own story license/credits deliberately; arbitrary source-root documents are
not silently distributed. Source adventure files are never rewritten.

Missing binaries produce an explanation; export never downloads or compiles a platform
runtime. Metadata checks version, protocol, target and SHA-256 alongside PE/ELF headers.
It detects mismatched files, not maliciously forged builds. Use trusted artifacts.

For a native build, generate the sidecar with:

```powershell
.\lupine.exe runtime-manifest | Set-Content -Encoding utf8 lupine.runtime.toml
```

```sh
./lupine runtime-manifest > lupine.runtime.toml
```

Use PowerShell 7 for UTF-8 without BOM, as in CI; Windows PowerShell 5 users can write
the captured text with `[IO.File]::WriteAllText` and a UTF8Encoding constructed with false.
The portable-build workflow generates these files automatically. An old v0.9.0 runtime
is not interchangeable with this development export runtime.

Export first builds/validates in a sibling .lupine-export-* workspace. Completed archives
are synced and checked before final publication. An interrupted final publication keeps
EXPORT_INCOMPLETE.txt: do not share that folder; retry into a fresh destination.
Export > C can remove a workspace only with a complete cleanup record and unchanged
inventory. It refuses unknown/modified files and links. Missing records require manual
review. Never blindly delete recovery data. Export requires hard-link support and is not
crash-atomic across multiple files. Avoid concurrent source/output edits. Creator recovery
is a separate guided workflow; see [Phase 4 safety and verification](PHASE_4_REVIEW.md).
