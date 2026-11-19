# Installing, playing and exporting Lupine

## Pick the right build

The published v0.9.0 release is historical and unchanged. Phase 3 features belong to
0.10.0-dev.1 development builds, available after building this source or downloading
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
by stable game ID. Existing slots require overwrite confirmation. Save format and game
compatibility versions are independent of the engine version. No save happens on exit.

## Export an adventure

1. Open it in the Adventure Creator and choose E.
2. Select Windows x64, Linux x64 or Both.
3. For the host OS, Enter uses the running engine. For another runtime, enter its file path
   and keep the matching `lupine.runtime.toml` beside it. Both must come from the same
   trusted build and match the exporting engine version.
4. Enter a new destination folder outside the source adventure. Its parent must exist.
5. Review the platforms and file inventory, then type yes to publish.

The result contains windows-x64/ and/or linux-x64/. Each has adventure/, the engine,
runtime metadata, Play.cmd or Play.sh, START_HERE.txt, engine license and dependency
notices. Share the whole platform folder. You may ZIP it with Explorer/Compress-Archive
or your preferred archive tool; archive creation is not built into Lupine. Personal RORK
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

An interrupted/failed export may retain EXPORT_INCOMPLETE.txt. Do not share that folder;
retry into a fresh destination and remove the incomplete folder only after reviewing it.
Export is not a crash-atomic multi-file operation. Do not modify source/output paths
concurrently while exporting. Editor recovery remains a separate manual workflow.
