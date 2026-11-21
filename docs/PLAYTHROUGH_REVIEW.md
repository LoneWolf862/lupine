# Playthrough follow-up and verification

## Delivered behavior

Development 0.11.0-dev.12, [PR #47](https://github.com/LoneWolf862/lupine/pull/47),
implements the creator-directed follow-up to the dev.11 playthrough. The creator
approved merging this work on **2026-11-21**. This approval is not a record of
native CachyOS testing. The creator subsequently authorized publishing
v0.11.0-dev.12; see [the release record](RELEASES.md).

- Eight rooms: shop west of camp; storeroom north with only a south exit;
  gate east; workshop, peaceful approach and final golem beyond it.
- Immediate cancellable name introduction, one-time shop key and persistent
  guide menus. L/leave closes a conversation; each response settles its own turn.
- Authored entry offers confirm the gate unlock. Declines are free; loading a
  save does not replay entry prompts. Manual unlock commands remain available.
- Article-aware NPC commands, explicit filtered bulk take/drop, plural
  clarification, inactive-number feedback and individual learning suggestions.
- Shop/chest keys remain in their locks only after successful unlocking.
  Light is taught after visiting the dark storeroom. Three goblin coins and
  an eight-coin trophy support the optional ten-coin sword and one-coin Light.
- Readable lists, automatic shop listings, immediate chest contents, current
  journal objectives, authored paragraph breaks and a 72–76-column prose guide.
  Rest restores HP/MP and refreshes Focus before the boss.

RORK remains **7**. This adventure's content compatibility revision is **2**;
older content-revision-1 saves need their original package. Save bytes are never
converted or overwritten automatically. Saving remains strictly manual.

## Recorded automated evidence

Gameplay commit: `4b03217bd43c0564c5c91c6f082fd9241d305420`.
Subsequent documentation/download-link edits do not change engine or content.

- 408 Rust tests, formatting, strict Clippy, independent Rust consumer and
  offline documentation checks passed locally on Windows.
- [Windows/Ubuntu CI](https://github.com/LoneWolf862/lupine/actions/runs/44738370368)
  and [both native portable builds](https://github.com/LoneWolf862/lupine/actions/runs/44738362861)
  passed on that commit.
- Each platform passed the 55-check packaged-adventure suite: validation of all
  11 bundled games, scenarios, save/restart state equality, trading, retreat,
  consumption, victory, creator ZIP export and exported-launcher victory.
- The exact downloaded Windows artifact independently passed the same 55 checks.
  Windows ConPTY play reached victory with Unicode naming, cancellation, menu
  continuation, history/editing, gate decline/acceptance and manual saving.
- Native Ubuntu PTY testing passed history/editing, save prompts, immediate
  Unicode naming, persistent dialogue and the complete mini-adventure route.
- Exported source content stayed unchanged; saves were excluded. No implicit
  saves occurred. Native Linux execution evidence comes from Ubuntu CI.

## Remaining human checks and downloads

The creator supplied a dev.11 playthrough and approved the dev.12 direction and
merge. A separate completed dev.12 human Windows/CachyOS acceptance report has
not been supplied. Broader nontechnical creator acceptance also remains open.
Use the [walkthrough and platform checklist](MINI_ADVENTURE.md) to record those
results without repeating all developer fixtures.

[Front-page downloads](../README.md#downloads) now use permanent public dev.12
release assets, available without GitHub sign-in. The earlier Actions links
required sign-in and expired. Keep a local copy of a runtime and its metadata
when exporting. The older v0.9.0 release remains unchanged.
