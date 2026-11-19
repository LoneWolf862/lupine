# Player feedback follow-up (2026-11-19)

Based on the supplied human transcript after Phase 3 merged to main (2377af1).
The transcript exposed real UI problems, not merely unfamiliarity with parser games.

## Prompt diagnosis and changes

Runner save-name and confirmation questions were flushed but written without a newline.
Rustyline owns/redraws its input line, so its empty prompt erased these questions.
Questions now finish on a separate line, and Rustyline supplies its own visible `> `.
Creator questions already use separate lines and keep their existing flow.

After the owner's follow-up playtest, bare save uses the shorter `Save name:` prompt.
If a slot was successfully saved or loaded in this play session, bare save first asks
`Overwrite current save 'NAME'? (yes/no):`. Yes writes that slot once; no asks for a name.
Enter/cancel/back at the name prompt cancels; EOF never saves. Explicit `save forest camp`
writes directly to that named slot, including replacement of the same name without another
question. A name selected through the prompt still requires confirmation if it exists.
Name validation, collision checks and staged file replacement remain unchanged.
The current slot changes only after successful save/load, is never serialized, and resets
on each fresh play. Failed or declined operations leave it unchanged. No command-name guard
or explanatory paragraph remains at the save-name prompt.

The transcript's first exit was immediately after a successful save: no unsaved-state
confirmation was needed. The second exit had unsaved movement; its question was hidden.
Quitting with unsaved progress still requires yes/no; no keeps the current state.
Saving stays exclusively manual. No save file/compatibility/version change.

## Parser changes

- Movement: travel/run/head, optional to/the, and leave followed by a direction.
  Leave followed by an object still means drop; no facing-based left/right inference.
- Inspection: look OBJECT and look around. Movement toward an object explains exits
  and inspection/use syntax rather than suggesting an unrelated verb.
- Ordered word prefixes: fire scroll can match fireball scroll. Exact IDs/names/authored
  aliases and existing substring matches retain priority. Multiple matches still clarify;
  spelling errors are never automatically accepted.
- Combat: hit/strike/stab/slash; omitted weapon requests explicit selection or cancellation.
  Cast/use accept at as well as on. Explicit attack-with can resolve a learned spell if
  no carried weapon matches. Existing validation, costs, retaliation and item-use priority
  remain authoritative; no damage/progression/encounter changes.
- Typo suggestions cover new verbs, avoid changing recognized verbs, and omit no-argument
  commands when arguments are present. They explain that the player must retype the command.
  Yes/no outside a question gets guidance. Bare object/learned-spell names offer syntax
  guidance, not inferred actions. Missing object arguments show concrete syntax.
- Help groups exploration, movement, objects, combat, spells and manual saves.

The sample explicitly aliases healing_scroll as scroll, explaining why take scroll selected
it despite another scroll being present. This authored priority is preserved. Knife is not
an alias for dagger in that adventure: such vocabulary belongs in content, not engine lore.
Bare fireball goblin gives cast syntax instead of spending MP. No pronouns, chained commands,
automatic typo acceptance or arbitrary natural-language inference were introduced.

## Verification and review

Regression cases extend parser_experience and rork_commands: new grammar, word-prefix
ambiguity, authored alias priority, no mutation on cancellation, current-slot save/save-as,
explicit names, EOF, fresh sessions, failed/declined load/save, visible question lines,
overwrite/load/quit declines and saved bytes.
The existing stable-ID and ordinary item/spell collision tests remain in place.

The Linux PTY script now waits for completed question lines before sending answers and
tests editing, naming confirmation, manual save, overwrite/load/quit declines and EOF.
It isolates user data under target and checks the saved file stays unchanged on declines.
Windows agent-driven terminal checks use both TERM=dumb and TERM=xterm, isolated APPDATA,
and the actual executable. This is not a second human playtest. Test/CI results are recorded
in the review PR; the original Phase 3 verification remains a historical checkpoint.

## Retest

From this feature worktree, run cargo run --release and choose the RPG Persistent sample.
Try take fire scroll, learn fire scroll, take dagger, travel south, attack goblin (cancel,
then select), and cast fireball at goblin. Try walet, then yes, then wallet.

Type save: with no current slot, only Save name appears. Enter camp. Move, then save:
yes overwrites camp; no asks for a new name. Enter forest camp to create another slot.
Try save camp to write directly, then load forest camp and save to verify the current slot.
Move, then exit: no continues,
yes leaves without saving. Use load camp to restore only when explicitly requested.
Retest Up/Down and editing in the friend's normal Windows terminal. Keep any personal
saves; this patch does not rename, rewrite or remove the existing north save.
