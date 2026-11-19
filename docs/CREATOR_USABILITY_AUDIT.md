# Adventure Creator usability audit — Phase 2I

Date: 2026-11-18. Development candidate; review PR #19 targets `codex/phase-2h-rpg-integration`. No merge or release/version change.

## Changes and findings

1. **Locked-door key selection:** removed the preliminary Enter/N question that swallowed `key3`. Portable keys appear immediately with numbers, names and IDs, alongside N and explicit 0. The door-object selector shows fixed objects in the selected exit's room. Both selectors confirm the chosen object. Ineligible objects are excluded with a portability/location explanation.
2. **Shared selection:** existing definitions accept displayed numbers, exact case-sensitive IDs and case-insensitive whole display names. Exact IDs take precedence over names. Duplicate names produce an ambiguity message and retain the prompt. No fuzzy matching. Number inputs retain their displayed menu meaning; action/control words remain reserved and the displayed number selects a definition with a reserved name.
3. **Blank input and help:** shared selection menus no longer default to 0. Blank/invalid input keeps the prompt and draft open. Menu help explains controls without navigating away; field prompts accept both ? and help. Enter at a field keeps its displayed default. Numeric zero remains a legitimate field value, not a global cancellation command. Main-menu Enter still refreshes the project tree.
4. **Manager coverage:** items, spells, enemies and shops share number/ID/name resolution; rooms and room connections, weapon associations, artwork filenames, FIG field names, offer/currency/table-entry selectors use the same resolution rules. Lists without authored IDs remain positional. Definition selections report the chosen value; existing form headers identify the object being edited.
5. **Teaching items:** removed the second Enter-to-choose prompt. Portable teaching items and the explicit remove action appear together. Teaching remains an intentional gameplay action; definition does not grant a spell.
6. **Door completion and revision:** exact previews precede explicit Save. Summary includes door/key names and IDs, source/destination rooms, direction, initial locked state, unlock flag and player commands using display names. It explains that the reverse route is unchanged. Success appears only after Snapshot publication and validation. Declining Save retains the door draft and offers message/flag revisions or discard; failed publication retains the same review. Back among message questions revisits the previous answer without replacing it with a default. Retrying rebuilds rules from the original snapshot, avoiding duplicate append operations.
7. **Description ordering:** the advertised “move earlier” option had no handler. It now changes serialized priority through the existing comment-preserving reorder helper, with runtime regression coverage.
8. **Encounter revision:** editing an existing encounter choice can now select a pending enemy from the same room draft, just like adding a choice. Publication remains joint with Room Save.
9. **Draft outcomes:** shared Save identifies its file; declined Save explicitly retains the draft. Item/spell/enemy/weapon discard reports no draft changes saved. Cancellation/EOF wording no longer implies that earlier separately confirmed prerequisite saves never happened. Entry guidance warns that cancel abandons the current unsaved operation.
10. **Data entry:** field help uses action-neutral confirmation wording instead of saying every yes “saves”; removal confirmations still default to no. Rejected description text remains visible for correction. Artwork rejects a bad line while preserving accepted lines, and does not create an empty new file following entirely rejected input. Artwork removal now lists actual variants rather than asking for an unchecked number.

## Menu audit inventory

| Area | Reviewed paths and result |
|---|---|
| Setup/launcher | Name, folder, IDs, first location, description, preview/create/revisit, open path; existing displayed defaults and back steps retained. No implicit publication. |
| Main creator/RPG hub | Play, validate, managers, help, return and EOF; existing numbered shortcuts and R preserved. |
| Rooms/connections | Manager, new room, title/description, source/destination, directions, reverse/conflicting exits, removal, summary/preview/save; room lookup now accepts IDs/names. Existing overwrite confirmation remains. |
| Items | Manager, placement, name/description, aliases, portability, conditional prose, preview/save; selection retry and selected-value feedback shared. Weapon/teaching portability protections retained. |
| Spells | Manager, cost/power bounds, effect/target, teaching link/remove, preview/save; one-screen portable teaching selector. |
| Artwork | Manager/file names, multiline text, room attachment, inline/file replacement, default removal, conditional variants/edit/remove/reorder; improved recovery and actual variant list. |
| Puzzles | Interaction used item/target, condition/effect lists, move-item destination, conditional exit and locked-door recipe; all references use shared selectors. |
| Conditional prose | Conditions, text, edit/remove and order; advertised reorder repaired. |
| Encounters | Chance, min/max, persistence, add/edit/remove choices, weights, requirements, pending enemy create/edit; pending references included during revision. |
| FIG | Character enablement/stats/progression and terminology; numbered or exact field-name selection, template explanations, safe defaults and staged validation retained. |
| Enemies | Manager, stats, attacks, weights, item drops/currency rewards and their table selectors; shared references and ambiguity handling. |
| Weapons | Existing manager, prerequisite Item Manager, unused portable association, damage/type; exact item IDs/names supported. Associations remain fixed. |
| Shops/offers | Manager, prerequisite managers, location, offers/type/currency/price/target/stock/flags, table entry edit; shared selectors, actual offer IDs/names. |
| Currencies | Primary/additional definitions, name/symbol/balance, additional-currency entry selection, manifest preview/save; authored IDs and names supported. |

## Navigation contract

- Number selects the displayed option. Definition lists additionally accept exact ID or unambiguous whole name; filename lists accept their listed filename. FIG menus accept the listed field name.
- `0` and `back` follow the screen's stated action. A reference selector leaves its field unchanged; an options screen labeled discard abandons its draft. Nested rule/offer/encounter screens explicitly say when 0 keeps proposed edits.
- `cancel` abandons the current operation, including unsaved edits. Previously confirmed saves stay saved. EOF abandons unsaved edits; it never saves player progress.
- `?` and `help` explain menu/field controls. Escape control words with a leading backslash when authoring literal text. Multiline text ends with `/end`.
- Enter accepts a **displayed field default**. Selection menus have no default and stay open on Enter. The main menu explicitly uses Enter to refresh.
- Save requires a separate yes; the confirmation defaults to no. Exact previews remain available. Validation failure is not success.

## Safety and self-review

Snapshot, confined paths, stale detection, exclusive publication, staged/full validation, rollback/recovery, comment preservation and exact preview/write contents are unchanged. No runtime/parser/RPG mechanics, Cargo version, dependency, save format or compatibility versions changed. No player autosave was added.

Self-review corrections beyond the initial key fix: teaching selector numbering changed when fixed items were filtered, so acceptance scripts now use the teaching item's exact ID; restored missing description reorder; included pending enemies in revised encounter choices; corrected door command summary to use player-facing names; retained door answers/review after declined Save; retained multiline artwork after invalid lines; kept cancellation wording honest about earlier saves. The first artwork-recovery regression exposed creation of an empty file after wholly rejected input; the save guard and regression now prevent that.

## Verification

Local environment: Lupa-2, Windows 10 Pro build 19045, Rust/Cargo 1.100.0, MSVC toolchain. **219 tests passed** with `cargo test --locked`; `cargo fmt --all --check` and `cargo clippy --locked --all-targets -- -D warnings` passed. Final code checkpoint: `8175484`. **CI at `8175484`: Windows Server 2022 and Ubuntu 22.04 both passed all 219 tests, formatting and strict Clippy.** [Verified Actions run](https://github.com/LoneWolf862/lupine/actions/runs/44452065317). Later documentation-only commits record these results without changing code.

Regression coverage includes existing Bronze Key selection by number, exact ID and display name; ambiguous duplicate names; invalid/empty/help input; explicit 0/back/cancel; unchanged files after cancellation; declined save and retained revision; reopening; initially blocked exit, taking the key, using it and successful travel; shared placement and teaching selection; description-order runtime priority; artwork line recovery. Existing tests exercise snapshot failures, stale files, rollback, RPG and manual save behavior.

A separate actual executable walkthrough (outside the test harness) consumed `tests/fixtures/rpg_creator_acceptance.txt` and retained the resulting adventure at `target/usability-final/Adventure`, output at `target/usability-final/walkthrough.txt` and isolated saves at `target/usability-final/userdata`. It covers new setup, connected rooms, items, a locked door, artwork/variants, FIG enablement, teaching item/spell, two enemies and weapons, encounter configuration, two currencies, shop offers, validation, gameplay and explicitly named saves. Reviewed output contained the expected saved/validated and gameplay messages and no save/manager errors. An additional reopen selected enemy, weapon, shop offer and currencies by ID and discarded all drafts; SHA-256 comparison confirmed every authored file and player save remained byte-identical. A live PTY session additionally reopened the earlier walkthrough and exercised exact-ID item/teaching selection, blank/invalid/help room selection, discard and validation.

## Remaining limits

This is agent-driven terminal testing, not certification by a human beginner. Native CachyOS testing was not performed. Ubuntu CI and Windows Server CI are separate from local Windows 10.

Navigation is explicit but not a universal undo stack: 0/back finish some nested lists and discard some whole drafts as labeled. Early new-definition identity/placement and key/door-object setup are sequential; cancel/back can abandon those new-operation answers. Door message questions and post-preview revisions retain answers, but switching its room/key/door after that stage requires restarting the recipe. Advanced condition/effect/offer collection details still include TOML in previews; no new visual editor or arbitrary scripting language was added. Ambiguous names must be resolved by a number or unique ID. Player-parser name ambiguity is unchanged. Recovery remains manual; invalid preexisting projects and stale snapshots may require repair/reopening. No ID migrations or transaction-object-binding editor.

## Windows human testing

Use the isolated worktree, not your primary checkout:

```powershell
Set-Location 'C:\Users\Rex\.codex\worktrees\creator-usability\Lupine'
cargo fmt --all --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run -- new
```

1. Create a disposable adventure, add Hall and connect Start east to Hall. Create a portable item ID `key3`, name `Bronze Key`, starting in Start.
2. Open 9 > 2. Select Start and east, then enter `key3` directly on the key list. Create a fixed gate. Try blank, help and a wrong ID first; ensure selection stays open. Repeat in separate disposable adventures using the displayed number and `Bronze Key`.
3. Review exact files and summary. Decline Save, revise a message and then confirm. Play: east must be blocked; take bronze key; use bronze key on gate; east must work. Reopen/validate. No RORK file should appear unless you explicitly save in gameplay.
4. Repeat and cancel at key selection, during new-door entry and at review; inspect files for no new gate/rules. Create a second Bronze Key name and verify the ambiguous name is rejected until you choose an ID/number.
5. Enable RPG in 3 > Character > enabled. Create a teaching item and spell, link by ID, create a portable weapon, enemy and encounter. Test N/E pending enemies and room cancellation. Create currencies and a shop with an item offer, selecting currency/target by ID. Preview and decline Save, then revise and save.
6. Create and attach artwork. Enter a bad control character in a disposable draft; verify accepted lines remain. Add/reorder description and artwork variants. Validate and play. Try help, blank and back in each manager and report any unclear discard boundary.

Review the open draft PR; do not merge automatically.

## Git handoff

- Worktree: `C:\Users\Rex\.codex\worktrees\creator-usability\Lupine`.
- Branch: `codex/phase-2i-creator-usability`.
- Base fetched at `8455c8c885c9191fabac2d06a81e8f70b6edb366` from `origin/codex/phase-2h-rpg-integration`.
- [Draft PR #19](https://github.com/LoneWolf862/lupine/pull/19) targets that integration branch. No merge.
- Checkpoints: `44bf163` shared selection; `2a7d75c` review/recovery and audit; `8175484` remaining guidance and pending-enemy regression. All pushed.
- Owner's primary checkout tracked files and preexisting untracked projects remain untouched. Disposable walkthroughs/logs are under ignored `target/` in this worktree.
