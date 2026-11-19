# RPG authoring review and handoff

Date: 2026-11-18. Implemented development candidate, pending human review. The five PRs remain draft; none was merged. The published v0.9.0 release is separate from this development checkout, whose Cargo version remains 0.8.4.

## Delivered workflows

Main menu **R** opens enemies (1), weapons (2), shops (3), currencies (4), room encounters (5), prerequisite items (6) and spells (7). Existing main menu numbers are unchanged. Enable RPG explicitly in main menu 3 first.

- **Enemies:** create/select/edit stable definitions; display name, description, level, maximum HP, attack power, defense, XP; add/edit/remove weighted attacks, unique-item drops and currency rewards. AP 100 means 1x; enemy level does not automatically scale stats. Attack weights are relative, need not sum to 100, and their sum must fit u32. Empty attacks means no counterattack. Loot chance and reward chance are independent percentages. Reward min must not exceed max; range width is at most 4294967294.
- **Weapons:** select an existing unused portable item, or enter Item Manager with I first. Edit damage and the free-text damage-type label. There is no damage-type resistance mechanic. The associated item ID stays fixed; players must possess it. Item Manager saves are separately confirmed and remain if a subsequent weapon draft is canceled.
- **Shops:** create/select/edit name and room; add/edit offers for item purchase, item sale, HP restoration, MP restoration, learning an ability and setting a flag. Configure currency, price, target, amount, optional stock and all-required flags. Existing shop/offer IDs remain fixed. Items are unique, not stacks; one item cannot be stocked by multiple offers. I/T/C open prerequisite item/spell/currency managers before a shop draft, with separately confirmed saves. Changing offer meaning or reducing stock can invalidate existing saved purchases; saves are never rewritten.
- **Currencies:** primary `[currency]` and additional `[[currencies]]` remain in `game.toml`. Edit name, symbol and starting balance; add definitions without renaming/removing existing IDs. Implicit `gold` IDs and inline tables are preserved. Without an explicit primary table, the runtime displays the alphabetically first additional currency first. Starting balances affect fresh games only. TOML authoring integers are signed 64-bit even where runtime storage uses u64.
- **Encounters:** Room Options 7 > 2 keeps the existing chance/group/persistence editor. N creates an enemy in the same room draft and selects it; E revises a pending enemy. Room Save validates and publishes all affected rooms and pending enemies together. Back from encounter settings discards those encounter edits and newly pending definitions; discarding the room writes neither. Pending definitions can be reused by another choice in that room draft.

Use P for exact previews (Room Options uses 5), S for Save (Room Options uses 6), and explicit yes to publish. No edit changes player saves. No autosave, runtime mechanics, dependency additions, package bumps or RORK version changes were introduced.

## Architecture and self-review

All publication uses the existing Snapshot: confined targets, staged full Adventure validation, stale bytes/metadata checks, permissions, exclusive publication and recoverable rollback. The target allowlist now includes enemies, weapons, shops and game.toml. Document/rules_ui retain filenames, stable IDs, unedited fields, comments, inline/table collections and CRLF behavior. Unsupported representations are refused. Shared RPG prompts live in rpg_fields.rs; enemy forms are reused for standalone and deferred encounter creation.

Review corrections and evidence:

1. Creating an enemy through an independent save while a room snapshot was open would make the room stale. Pending enemy documents now join the room transaction instead. Joint cancellation/publication is tested.
2. Pending enemies need correction before publication. E reopens them, and room validation failures retain the draft. Stale/recovery failures still require review and reopening.
3. The first acceptance script failed to reach the second enemy despite an initially passing shallow assertion. The route and assertions were strengthened: saved HP now proves both encounters retain wounds, and successful healing/mana transactions are required in actual output.
4. Expected wallet arithmetic was corrected in the test: 25 minus 5, 1, 2 and 3, plus a 5 reward is 19. Runtime arithmetic was unchanged.
5. Shared field prompts were moved out of the enemy module to avoid coupling other managers to enemy authoring.
6. Item Manager now refuses making a weapon item fixed, matching its existing teaching-item safeguard; this is an authoring guard, not a runtime schema change.
7. Tests cover inline primary currencies with omitted default IDs, table/inline offer preservation, invalid weights/missing references, duplicate currency IDs, declined saves, EOF, cancellation, external edits and mixed-category rollback.

## Environment and branches

Primary checkout: `C:\Users\Rex\Code\Lupine`, main at 417b496 when work began. Its branch/files and personal untracked projects were untouched. Local host **Lupa-2**, Windows 10 Pro 22H2 build **19045.6466**, Rust **1.100.0**, Cargo **1.100.0**, stable x86_64-pc-windows-msvc. GitHub authentication works as LoneWolf862 using the local keyring; restricted sandbox commands initially reported a misleading authentication failure.

Managed worktrees use `C:\Users\Rex\.codex\worktrees\<name>\Lupine`, outside the primary checkout.

| Order | Worktree name | Branch | PR | Target |
|---|---|---|---|---|
| 1 | phase-2g-enemies | codex/phase-2g-enemies | [14](https://github.com/LoneWolf862/lupine/pull/14) | main |
| 2 | phase-2g-weapons | codex/phase-2g-weapons | [15](https://github.com/LoneWolf862/lupine/pull/15) | codex/phase-2g-enemies |
| 3 | phase-2g-shops | codex/phase-2g-shops | [16](https://github.com/LoneWolf862/lupine/pull/16) | codex/phase-2g-weapons |
| 4 | phase-2g-currencies | codex/phase-2g-currencies | [17](https://github.com/LoneWolf862/lupine/pull/17) | codex/phase-2g-shops |
| 5 | phase-2h-rpg-integration | codex/phase-2h-rpg-integration | [18](https://github.com/LoneWolf862/lupine/pull/18) | codex/phase-2g-currencies |

Review the complete experience in the integration worktree. Merge in the listed order, preferably preserving ancestry with merge commits; retarget descendants to main when appropriate. Squash/rebase merging requires deliberate descendant rebasing and diff review. No self-merge is authorized.

## Verification

Local final code commit: `a937b65`. **211 tests passed**, `cargo fmt --all -- --check` passed and `cargo clippy --locked --all-targets -- -D warnings` passed in the integration worktree. All six bundled adventures also passed direct executable validation. **CI at `a937b65`: 211 tests passed on each of Windows Server 2022 and Ubuntu 22.04, with formatting and strict Clippy passing.** [Verified run](https://github.com/LoneWolf862/lupine/actions/runs/44425466352). Later documentation-only commits record these results without changing code. Scripted executable testing is not human beginner usability certification. Windows Server CI is distinct from local Windows 10; Ubuntu CI is not native CachyOS testing.

`tests/fixtures/rpg_creator_acceptance.txt` authors the whole demonstration through real CLI menus: three connected rooms, locked door and key, conditional prose/art, portable/fixed items, teaching scroll/spell, two weapons, two enemies with different attacks, XP, item/currency rewards, two currencies and all six shop offer types. No TOML/FIG construction is used by that acceptance test. Gameplay checks ownership, exact configured weapon damage through saved HP, room/unique encounter persistence, XP/level-up, loot collection, currency rewards, shop charges, teaching, healing/mana, and explicit named save/load. Existing bundled samples remain in the full regression suite.

## Windows human testing

In PowerShell:

```powershell
Set-Location 'C:\Users\Rex\.codex\worktrees\phase-2h-rpg-integration\Lupine'
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test rpg_creator_acceptance
cargo run -- new
```

Create a disposable adventure. Enable RPG in 3 > Character > enabled. Use R > 4 for Gold and Tokens, then items/spells, weapons, enemies and shops. In Rooms > 7 > 2, try N to create an enemy and then cancel the room; confirm neither file appeared. Repeat and Save, then edit a pending enemy with E. Preview each category and decline Save once before revising/confirming. Configure a 100-percent unique encounter and a room-persistent encounter in connected rooms. Validate, play, collect/buy weapons, fight, retreat/re-enter, learn a spell, buy healing and explicitly save/load named slots. Reopen the creator and verify no automatic player-save write occurs. In a disposable copy only, change an authored file externally before Save and verify the editor refuses to overwrite it.

## Retained local demonstration

A separately executed walkthrough retained its generated adventure at `target/rpg-review-190e6599/Adventure` in the integration worktree, alongside `walkthrough.txt` and isolated `userdata`. It was created entirely through the real CLI menus. Reopening and validating preserved all authored/save bytes (SHA-256 comparison). These disposable artifacts are ignored, not committed. To open it from that worktree:

```powershell
$env:APPDATA = Join-Path (Get-Location) 'target/rpg-review-190e6599/userdata'
$env:XDG_DATA_HOME = $env:APPDATA
cargo run -- creator 'target/rpg-review-190e6599/Adventure'
```

Run this in a dedicated PowerShell session; closing it restores the normal environment for other sessions. Existing personal adventures were not used. The automated regression cleans up its own temporary adventure; the retained walkthrough remains available for review.

## Updated documentation

README.md, AGENTS.md, CREATOR_QUICKSTART.md, RPG_CREATOR_GUIDE.md, CREATOR_WIZARD_SPEC.md, CONFIG_FIG_SPEC.md, ARCHITECTURE.md, ROADMAP.md, BACKLOG.md, CODEX_HANDOFF.md, CREATOR_COMPLETION_REVIEW.md and this report. Older milestone sections remain historical; the current sections/report supersede their feature-status claims. The Cargo source version remains 0.8.4 despite the separately published v0.9.0 release.

## Limits

No stable-ID rename/removal migrations, transaction-object-binding editor, automatic recovery or new gameplay mechanics. Weapon-to-item associations stay fixed; author a new weapon for a different unused item. Existing invalid projects must be repaired before guided editing. Recovery remains manual in `.lupine-room-edit`; publication is recoverable rather than crash-atomic, requires hard links, and retains the prior size/traversal bounds. Human beginner testing and native CachyOS testing remain unperformed. No course time logs or submission evidence were invented.
