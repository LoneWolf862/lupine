# Creator navigation

Creator controls apply to the current operation. Nothing publishes automatically.

| Control | Result |
|---|---|
| Enter | Uses only a displayed field default; never selects a menu entry or confirms a save |
| `?` / `help` | Explains input without changing the draft |
| `0` / `back` in a menu | Follows the displayed return/discard label; nested editors may return with their parent draft retained |
| `back` at a field | Keeps that field unchanged |
| `cancel` / EOF | Abandons the current operation's unsaved draft; earlier separately confirmed saves remain |
| `/multi`, then `/end` | Enters multiline prose; artwork/raw-file entry accepts `/end` directly |
| Backslash before a control line | Inserts literal control text in multiline input |

Raw file entry also accepts `.end`; `back` discards that replacement while retaining
other files in the batch. `/end` returns to the editor and **does not save**.

## Search and lists

Shared definition selectors, room/item/spell/enemy/weapon/shop/art managers and the
project-file browser support `/text` to filter IDs or names, `/` to clear, `>` for
next page and `<` for previous page. Pages contain at most 20 entries. A filter
never selects its sole result automatically. Numbers retain their original list
positions across pages and filters. Exact IDs/names can select from the whole list;
ambiguous names still require an explicit ID or number. Use **F > L** to search
project paths and display names, including newly created unsaved files.

Document forms show **UNSAVED DRAFT** or **No unsaved changes**. Room drafts show
their changed-file count, including connections and pending enemies. Project Files
shows the whole batch count; **P** previews all changes even when the initial file
list is abbreviated. Saved/declined/discarded messages describe the actual outcome.

## Browse by room

Main menu **B** selects a room using the shared search. The room screen shows saved
prose and files containing typed references to that room: definitions, exits,
placements, actions, events, NPCs and shops. This is a static author view; a reference
does not prove the content is currently visible or reachable during play.

Open the guided room editor, edit a related file directly, follow a connected room,
or open all project files. Each editor retains its own confirmation boundary.
Returning reloads saved content. There is no invisible cross-editor draft and no
autosave. Use Project Files when several related files must publish together.

R now permits shops and currencies in narrative games. Zero-MP world abilities can
be selected in the spell form without enabling RPG. Shared effects/targets, stacks,
equipment, statuses and other advanced fields are accessible through **F**; the
structural editor is the complete-schema interface, not a bespoke novice form for
every field. Dedicated combat FIG settings are under **3 > 3**.

## Local export runtimes

Export continues to offer the running executable for the host platform. It also
checks these locations beside the running executable, for each requested platform:

```text
<platform>/lupine[.exe]
runtimes/<platform>/lupine[.exe]
```

`<platform>` is `windows-x64` or `linux-x64`. Only binaries passing the existing
target/version/header/digest/manifest checks appear; type `@1`, `@2`, etc. to select
one. Explicit executable paths still work. Discovery never executes candidates,
downloads files, scans arbitrary directories or relabels binaries. Export preflight
verifies selected runtimes again before confirmation/publication.

Automated acceptance covers scripted navigation and cancellation. Human beginner
testing and native CachyOS acceptance remain separate, unclaimed checks.
