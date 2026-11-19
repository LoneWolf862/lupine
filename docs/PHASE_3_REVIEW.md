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
