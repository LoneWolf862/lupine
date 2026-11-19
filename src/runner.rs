//! Standard terminal interface with engine-owned manual RORK commands.
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::combat;
use crate::encounters::{EncounterContent, EncounterRng};
use crate::engine::ActionOutcome;
use crate::parser::{parse_command, ParseError};
use crate::render::describe_room;
use crate::save_files::{
    default_save_root, list_slots, read_slot_with_compat, slot_exists, write_slot_with_compat,
};
use crate::shops::ShopContent;
use crate::state::GameState;
use crate::wallet::WalletContent;
use crate::world::World;

/// Launch a Rust-authored game, using a stable ID derived from its title.
/// For durable save compatibility, prefer [`run_with_game_id`] with an explicit ID.
pub fn run(world: World, title: &str) -> io::Result<()> {
    let id = title
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '-')
        .collect::<String>();
    run_with_game_id(world, title, &id)
}

/// Launch with an explicit stable game ID used to isolate saves.
pub fn run_with_game_id(world: World, title: &str, game_id: &str) -> io::Result<()> {
    run_with_game_id_and_compat(world, title, game_id, 1)
}

/// Run with a stable game ID and explicit save compatibility revision.
pub fn run_with_game_id_and_compat(
    world: World,
    title: &str,
    game_id: &str,
    compatibility_version: u32,
) -> io::Result<()> {
    run_with_game_id_compat_and_progression(world, title, game_id, compatibility_version, None)
}

/// Launch a data-driven adventure with optional RPG character progression.
pub fn run_with_game_id_compat_and_progression(
    world: World,
    title: &str,
    game_id: &str,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
) -> io::Result<()> {
    run_with_game_id_compat_progression_content(
        world,
        title,
        game_id,
        compatibility_version,
        progression,
        crate::rpg_content::RpgContent::default(),
    )
}

/// Scriptable runner, preserving the original API. Its game ID is based on the title.
pub fn run_with_io<R: BufRead, W: Write>(
    world: World,
    title: &str,
    input: R,
    output: W,
) -> io::Result<()> {
    let id = title
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '-')
        .collect::<String>();
    // The default directory is only touched when a save-related command runs.
    let root = default_save_root()?;
    run_with_io_and_saves(world, title, &id, &root, input, output)
}

/// Scriptable and isolated save-root API; suitable for host apps and unit tests.
pub fn run_with_io_and_saves<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    input: R,
    output: W,
) -> io::Result<()> {
    run_with_io_and_saves_compat(world, title, game_id, root, 1, input, output)
}

/// Testable terminal runner that also enforces content compatibility versions.
pub fn run_with_io_and_saves_compat<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    compatibility_version: u32,
    input: R,
    output: W,
) -> io::Result<()> {
    run_with_io_and_saves_progression(
        world,
        title,
        game_id,
        root,
        compatibility_version,
        None,
        input,
        output,
    )
}

/// Optional RPG progression is activated only by a game's `config/character.fig`.
#[allow(clippy::too_many_arguments)]
pub fn run_with_io_and_saves_progression<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
    input: R,
    output: W,
) -> io::Result<()> {
    run_with_io_and_saves_progression_content(
        world,
        title,
        game_id,
        root,
        compatibility_version,
        progression,
        crate::rpg_content::RpgContent::default(),
        input,
        output,
    )
}

/// Launch a TOML RPG adventure with optional weapon and ability definitions.
pub fn run_with_game_id_compat_progression_content(
    world: World,
    title: &str,
    game_id: &str,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
    content: crate::rpg_content::RpgContent,
) -> io::Result<()> {
    run_with_game_id_compat_progression_content_encounters(
        world,
        title,
        game_id,
        compatibility_version,
        progression,
        content,
        EncounterContent::default(),
    )
}

/// Run a TOML story with validated encounter definitions (C1 preview).
pub fn run_with_game_id_compat_progression_content_encounters(
    world: World,
    title: &str,
    game_id: &str,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
    content: crate::rpg_content::RpgContent,
    encounters: EncounterContent,
) -> io::Result<()> {
    run_with_game_id_compat_progression_content_encounters_wallet(
        world,
        title,
        game_id,
        compatibility_version,
        progression,
        content,
        encounters,
        WalletContent::default(),
    )
}

/// Launch an RPG adventure with its optional story-configured wallet.
#[allow(clippy::too_many_arguments)]
pub fn run_with_game_id_compat_progression_content_encounters_wallet(
    world: World,
    title: &str,
    game_id: &str,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
    content: crate::rpg_content::RpgContent,
    encounters: EncounterContent,
    wallet: WalletContent,
) -> io::Result<()> {
    run_configured(
        world,
        title,
        game_id,
        compatibility_version,
        crate::config::GameConfig {
            character: progression,
            ..Default::default()
        },
        content,
        encounters,
        wallet,
        ShopContent::default(),
    )
}

/// Run with creator-authored shop definitions (opt-in).
#[allow(clippy::too_many_arguments)]
pub fn run_configured(
    world: World,
    title: &str,
    game_id: &str,
    compatibility_version: u32,
    config: crate::config::GameConfig,
    content: crate::rpg_content::RpgContent,
    encounters: EncounterContent,
    wallet: WalletContent,
    shops: ShopContent,
) -> io::Result<()> {
    let root = default_save_root()?;
    run_configured_with_io(
        world,
        title,
        game_id,
        &root,
        compatibility_version,
        config,
        content,
        encounters,
        wallet,
        shops,
        crate::terminal::stdin()?,
        io::stdout().lock(),
    )
}

/// Testable RPG runner with spellbook and weapon definitions.
#[allow(clippy::too_many_arguments)]
pub fn run_with_io_and_saves_progression_content<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
    content: crate::rpg_content::RpgContent,
    input: R,
    output: W,
) -> io::Result<()> {
    run_with_io_and_saves_progression_content_encounters(
        world,
        title,
        game_id,
        root,
        compatibility_version,
        progression,
        content,
        EncounterContent::default(),
        input,
        output,
    )
}

/// Isolated runner with optional encounter tables; C1 enemies are preview-only.
#[allow(clippy::too_many_arguments)]
pub fn run_with_io_and_saves_progression_content_encounters<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
    content: crate::rpg_content::RpgContent,
    encounters: EncounterContent,
    input: R,
    output: W,
) -> io::Result<()> {
    run_with_io_and_saves_progression_content_encounters_wallet(
        world,
        title,
        game_id,
        root,
        compatibility_version,
        progression,
        content,
        encounters,
        WalletContent::default(),
        input,
        output,
    )
}

/// Isolated testable runner that manages optional currency balances.
#[allow(clippy::too_many_arguments)]
pub fn run_with_io_and_saves_progression_content_encounters_wallet<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    compatibility_version: u32,
    progression: Option<crate::rpg::Progression>,
    content: crate::rpg_content::RpgContent,
    encounters: EncounterContent,
    wallet: WalletContent,
    input: R,
    output: W,
) -> io::Result<()> {
    run_configured_with_io(
        world,
        title,
        game_id,
        root,
        compatibility_version,
        crate::config::GameConfig {
            character: progression,
            ..Default::default()
        },
        content,
        encounters,
        wallet,
        ShopContent::default(),
        input,
        output,
    )
}

/// Testable runner with creator-configured shops.
#[allow(clippy::too_many_arguments)]
pub fn run_configured_with_io<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    compatibility_version: u32,
    config: crate::config::GameConfig,
    content: crate::rpg_content::RpgContent,
    encounters: EncounterContent,
    wallet: WalletContent,
    shops: ShopContent,
    input: R,
    output: W,
) -> io::Result<()> {
    run_session_with_io(
        world,
        title,
        game_id,
        root,
        compatibility_version,
        config,
        content,
        encounters,
        wallet,
        shops,
        input,
        output,
        None,
    )
    .map(|_| ())
}

/// The normal player command path with an optional repeatable random seed and
/// final in-memory state for author scenario assertions. This never saves implicitly.
#[allow(clippy::too_many_arguments)]
pub fn run_session_with_io<R: BufRead, W: Write>(
    world: World,
    title: &str,
    game_id: &str,
    root: &Path,
    compatibility_version: u32,
    config: crate::config::GameConfig,
    content: crate::rpg_content::RpgContent,
    encounters: EncounterContent,
    wallet: WalletContent,
    shops: ShopContent,
    mut input: R,
    mut output: W,
    seed: Option<u64>,
) -> io::Result<GameState> {
    let progression = &config.character;
    if compatibility_version == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "save compatibility version must be at least 1",
        ));
    }
    if let Err(errors) = world.validate() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            errors.join("\n"),
        ));
    }
    encounters
        .validate_loot(&world)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    wallet
        .validate_encounters(&encounters)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if progression.is_none() && !wallet.currencies.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Wallet requires RPG progression",
        ));
    }
    let mut state = GameState::new(&world);
    state.wallet = wallet.initial_balances();
    encounters.prepare_loot(&mut state);
    shops.prepare_inventory(&mut state);
    if let Some(config) = &progression {
        state.character = Some(config.create_character().map_err(io::Error::other)?);
    }
    let mut last_saved = state.clone();
    // Session-only convenience: selecting a slot never triggers a save itself.
    let mut current_save: Option<String> = None;
    writeln!(output, "{title}")?;
    writeln!(output, "Powered by Lupine. Type 'help' for commands.\n")?;
    let mut rng = seed.map_or_else(EncounterRng::from_clock, EncounterRng);
    for text in crate::events::start(&world, &mut state).map_err(io::Error::other)? {
        writeln!(output, "{text}\n")?;
    }
    let room = state.current_room.clone();
    if state.events.ending.is_none() {
        encounters.enter_room(&room, &mut state, &mut rng);
    }
    // Starting an encounter is an in-memory state mutation, never an autosave.
    show_room(&world, &state, &mut output)?;
    if !state.active_enemies.is_empty() {
        writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
    }
    let mut pending = None::<crate::events::Step>;
    loop {
        if let Some(step) = pending.take() {
            let old_room = state.current_room.clone();
            match crate::events::finish(&world,&mut state,step){
                Ok(lines)=>for text in lines{writeln!(output,"{text}\n")?;},
                Err(e)=>writeln!(output,"Story event could not complete: {e}. The triggering action remains applied; event changes were rolled back.\n")?,
            }
            sync_object_room(
                &old_room,
                &world,
                &mut state,
                &encounters,
                &mut rng,
                &mut output,
            )?;
        }
        write!(output, "> ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            writeln!(output, "\nGoodbye!")?;
            break;
        }
        if line
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            writeln!(
                output,
                "Input contains terminal control characters. Please type a command.\n"
            )?;
            continue;
        }
        let command = line.trim();
        let verb = command.split_whitespace().next().unwrap_or("");
        // Defeat is terminal for this in-memory run. Only manual load or quit can
        // recover; do not silently heal, autosave or permit paid services.
        if is_defeated(&state)
            && !["load", "saves", "quit", "q", "exit", "help", "h", "?"]
                .iter()
                .any(|allowed| verb.eq_ignore_ascii_case(allowed))
        {
            writeln!(
                output,
                "You have been defeated. Use 'saves', 'load <name>', or 'quit'.\n"
            )?;
            continue;
        }
        if state.events.ending.is_some()
            && ![
                "look",
                "l",
                "examine",
                "inspect",
                "x",
                "inventory",
                "i",
                "inv",
                "journal",
                "quests",
                "save",
                "saves",
                "load",
                "quit",
                "q",
                "exit",
                "help",
                "h",
                "?",
            ]
            .iter()
            .any(|v| verb.eq_ignore_ascii_case(v))
        {
            writeln!(output,"This adventure has ended. You may inspect, read your journal, manually save/load, or quit.\n")?;
            continue;
        }
        pending = Some(crate::events::Step::new(
            &state,
            !["save", "load", "saves"]
                .iter()
                .any(|v| verb.eq_ignore_ascii_case(v)),
        ));
        if verb.eq_ignore_ascii_case("people") && command.split_whitespace().count() == 1 {
            writeln!(output, "{}\n", crate::dialogue::people(&world, &state))?;
            continue;
        }
        if verb.eq_ignore_ascii_case("talk") {
            let query = crate::resolution::normalize(command[verb.len()..].trim());
            let query = query.strip_prefix("to ").unwrap_or(&query);
            if query.is_empty() {
                writeln!(output, "Talk to whom? Use talk <name>.\n")?;
                continue;
            }
            let before = state.current_room.clone();
            if crate::dialogue::talk(&world, &mut state, query, &mut input, &mut output)? {
                if let Some(step) = pending.as_mut() {
                    step.performed = true;
                }
                sync_object_room(
                    &before,
                    &world,
                    &mut state,
                    &encounters,
                    &mut rng,
                    &mut output,
                )?;
            }
            continue;
        }
        if ["journal", "quests"]
            .iter()
            .any(|v| verb.eq_ignore_ascii_case(v))
            && command.split_whitespace().count() == 1
        {
            writeln!(output, "{}\n", crate::events::journal(&world, &state))?;
            continue;
        }
        if !verb.eq_ignore_ascii_case("use") {
            match crate::actions::prepare(&line, &world, &state, &mut input, &mut output)? {
                crate::actions::Prepared::Canceled => continue,
                crate::actions::Prepared::Action(index) => {
                    if run_authored(
                        index,
                        &world,
                        &mut state,
                        &encounters,
                        &mut rng,
                        &mut output,
                    )? {
                        if let Some(step) = pending.as_mut() {
                            step.performed = true;
                            step.triggers.push(crate::events::Trigger::Action(
                                world.actions[index].id.clone(),
                            ));
                        }
                    }
                    continue;
                }
                crate::actions::Prepared::Unhandled => (),
            }
        }
        if verb.eq_ignore_ascii_case("wait") && command.split_whitespace().count() == 1 {
            if let Some(step) = pending.as_mut() {
                step.performed = true;
            }
            writeln!(output, "Time passes.\n")?;
            continue;
        }
        match crate::objects::prepare(&line, &world, &state, &mut input, &mut output)? {
            crate::objects::Prepared::Unhandled => (),
            crate::objects::Prepared::Canceled => continue,
            crate::objects::Prepared::Action {
                verb,
                item,
                target,
                quantity,
            } => {
                let old_room = state.current_room.clone();
                let result = crate::objects::manipulate(
                    &world,
                    &mut state,
                    &verb,
                    &item,
                    target.as_deref(),
                    quantity,
                );
                if result.is_ok() {
                    if let Some(step) = pending.as_mut() {
                        step.performed = true;
                    }
                }
                let text = result.unwrap_or_else(|e| e);
                writeln!(output, "{text}\n")?;
                sync_object_room(
                    &old_room,
                    &world,
                    &mut state,
                    &encounters,
                    &mut rng,
                    &mut output,
                )?;
                continue;
            }
        }
        let original_intent = parse_command(&line);
        let requested_bulk = matches!(&original_intent, Ok(crate::commands::Command::Take(target) | crate::commands::Command::Drop(target)) if target == "all");
        let Some(line) = crate::player_input::prepare(
            &line,
            &world,
            &state,
            &content,
            &encounters,
            &mut input,
            &mut output,
        )?
        else {
            continue;
        };
        let command = line.trim();
        let (verb, argument) = command
            .split_once(char::is_whitespace)
            .map_or((command, ""), |(verb, rest)| (verb, rest.trim()));
        if verb.eq_ignore_ascii_case("enemies") && argument.is_empty() {
            writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
            continue;
        }
        // Combat commands are handled before the legacy object parser. Invalid actions
        // are explicitly free; successful attacks trigger one enemy retaliation phase.
        let physical = if verb.eq_ignore_ascii_case("attack") {
            Some(argument.to_owned())
        } else {
            None
        };
        if let Some(physical) = physical {
            let mut current = std::mem::take(&mut state.active_enemies);
            let action = combat::physical_attack(
                &world,
                &content,
                &encounters,
                &mut state,
                &mut current,
                &physical,
            );
            state.active_enemies = current;
            writeln!(output, "{}\n", action.message)?;
            if let Some(defeated) = &action.defeated {
                if let Some(step) = pending.as_mut() {
                    step.triggers
                        .push(crate::events::Trigger::Defeat(defeated.enemy_id.clone()));
                }
                encounters.record_defeat(&mut state);
                let reward = combat::award_defeat(
                    &encounters,
                    &world,
                    progression.as_ref(),
                    &mut state,
                    defeated,
                    &mut rng,
                );
                if !reward.is_empty() {
                    writeln!(output, "{reward}\n")?;
                }
                let coins = wallet.award(&encounters, &defeated.enemy_id, &mut state, &mut rng);
                if !coins.is_empty() {
                    writeln!(output, "{coins}\n")?;
                }
            }
            if action.turn_spent {
                if let Some(step) = pending.as_mut() {
                    step.performed = true;
                }
                let surviving = state.active_enemies.clone();
                let retaliation = combat::retaliate(&encounters, &mut state, &surviving, &mut rng);
                if !retaliation.is_empty() {
                    writeln!(output, "{retaliation}\n")?;
                }
                if is_defeated(&state) {
                    writeln!(
                        output,
                        "\nYOU HAVE BEEN DEFEATED\nAvailable commands: saves, load <name>, quit.\n"
                    )?;
                }
                if !is_defeated(&state) && !state.active_enemies.is_empty() {
                    writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
                }
            }
            continue;
        }
        if verb.eq_ignore_ascii_case("cast")
            && argument.to_ascii_lowercase().contains(" on ")
            && !argument.to_ascii_lowercase().ends_with(" on self")
            && !argument.to_ascii_lowercase().ends_with(" on me")
        {
            let mut current = std::mem::take(&mut state.active_enemies);
            let action =
                combat::offensive_spell(&content, &encounters, &mut state, &mut current, argument);
            state.active_enemies = current;
            writeln!(output, "{}\n", action.message)?;
            if let Some(defeated) = &action.defeated {
                if let Some(step) = pending.as_mut() {
                    step.triggers
                        .push(crate::events::Trigger::Defeat(defeated.enemy_id.clone()));
                }
                encounters.record_defeat(&mut state);
                let reward = combat::award_defeat(
                    &encounters,
                    &world,
                    progression.as_ref(),
                    &mut state,
                    defeated,
                    &mut rng,
                );
                if !reward.is_empty() {
                    writeln!(output, "{reward}\n")?;
                }
                let coins = wallet.award(&encounters, &defeated.enemy_id, &mut state, &mut rng);
                if !coins.is_empty() {
                    writeln!(output, "{coins}\n")?;
                }
            }
            if action.turn_spent {
                if let Some(step) = pending.as_mut() {
                    step.performed = true;
                }
                let surviving = state.active_enemies.clone();
                let retaliation = combat::retaliate(&encounters, &mut state, &surviving, &mut rng);
                if !retaliation.is_empty() {
                    writeln!(output, "{retaliation}\n")?;
                }
                if is_defeated(&state) {
                    writeln!(
                        output,
                        "\nYOU HAVE BEEN DEFEATED\nAvailable commands: saves, load <name>, quit.\n"
                    )?;
                }
                if !is_defeated(&state) && !state.active_enemies.is_empty() {
                    writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
                }
            }
            continue;
        }
        if ["wallet", "money", "balance"]
            .iter()
            .any(|v| verb.eq_ignore_ascii_case(v))
            && argument.is_empty()
        {
            writeln!(output, "{}\n", wallet.describe(&state))?;
            continue;
        }
        // Opt-in creator-authored object interactions backed by atomic shop offers.
        // These run before the legacy parser, which otherwise requires `use X on Y`.
        if verb.eq_ignore_ascii_case("use")
            && !argument.is_empty()
            && !argument.to_ascii_lowercase().contains(" on ")
        {
            if let Some(result) =
                shops.use_transaction_item_result(&world, &mut state, &wallet, argument)
            {
                if result.is_ok() {
                    if let Some(step) = pending.as_mut() {
                        step.performed = true;
                    }
                }
                let message = result.unwrap_or_else(|e| e);
                writeln!(output, "{message}\n")?;
                continue;
            }
            if let Some(index) = world.actions.iter().position(|a| {
                a.in_scope(&state)
                    && a.target.as_deref() == Some(argument)
                    && a.commands.iter().any(|p| p == "use {target}")
            }) {
                if run_authored(
                    index,
                    &world,
                    &mut state,
                    &encounters,
                    &mut rng,
                    &mut output,
                )? {
                    if let Some(step) = pending.as_mut() {
                        step.performed = true;
                        step.triggers.push(crate::events::Trigger::Action(
                            world.actions[index].id.clone(),
                        ));
                    }
                }
                continue;
            }
        }
        if verb.eq_ignore_ascii_case("use")
            && world
                .object_defs
                .get(argument)
                .is_some_and(|d| d.consumable.is_some())
        {
            let old_room = state.current_room.clone();
            let result =
                crate::objects::manipulate(&world, &mut state, "consume", argument, None, 1);
            if result.is_ok() {
                if let Some(step) = pending.as_mut() {
                    step.performed = true;
                }
            }
            let text = result.unwrap_or_else(|e| e);
            writeln!(output, "{text}\n")?;
            sync_object_room(
                &old_room,
                &world,
                &mut state,
                &encounters,
                &mut rng,
                &mut output,
            )?;
            continue;
        }
        if verb.eq_ignore_ascii_case("shop") {
            writeln!(output, "{}\n", shops.describe(&state, &wallet, argument))?;
            continue;
        }
        if verb.eq_ignore_ascii_case("buy") || verb.eq_ignore_ascii_case("sell") {
            let (offer, shop) = argument
                .split_once(" from ")
                .map_or((argument, ""), |(o, s)| (o.trim(), s.trim()));
            if offer.is_empty() {
                writeln!(output, "Specify an offer, e.g. 'buy healing service'.\n")?;
            } else {
                // Sell offers are creator-defined; the verb is convenience syntax.
                let result = shops.transact(&mut state, &wallet, offer, shop);
                if result.is_ok() {
                    if let Some(step) = pending.as_mut() {
                        step.performed = true;
                    }
                }
                writeln!(
                    output,
                    "{}\n",
                    match result {
                        Ok(s) => s,
                        Err(e) => e,
                    }
                )?;
            }
            continue;
        }
        // Bulk inventory operations are explicit: only items actually in this room
        // can be collected, and only carried items can be dropped. Never touch fixed
        // scenery, consumed items or another room's objects.
        if (verb.eq_ignore_ascii_case("take") || verb.eq_ignore_ascii_case("drop"))
            && argument.eq_ignore_ascii_case("all")
            && requested_bulk
        {
            let taking = verb.eq_ignore_ascii_case("take");
            let targets: Vec<_> = world
                .items
                .iter()
                .filter(|item| {
                    item.portable
                        && state.is_accessible(&item.id)
                        && if taking {
                            state.available_source(&item.id, false).is_some()
                        } else {
                            state.count_at(&item.id, &crate::state::ItemLocation::Inventory) > 0
                        }
                })
                .map(|item| item.id.clone())
                .collect();
            if targets.is_empty() {
                writeln!(
                    output,
                    "{}\n",
                    if taking {
                        "Nothing to take."
                    } else {
                        "You are carrying nothing to drop."
                    }
                )?;
            } else {
                for id in targets {
                    while let Some(from) = state.available_source(&id, !taking) {
                        let quantity = state.count_at(&id, &from);
                        let to = if taking {
                            crate::state::ItemLocation::Inventory
                        } else {
                            crate::state::ItemLocation::Room(state.current_room.clone())
                        };
                        match state.transfer(&world, &id, &from, &to, quantity) {
                            Ok(()) => writeln!(
                                output,
                                "You {} the {}{}.",
                                if taking { "take" } else { "drop" },
                                world.item(&id).unwrap().name,
                                if crate::objects::definition(&world, &id).stackable {
                                    format!(" ({quantity})")
                                } else {
                                    String::new()
                                }
                            )?,
                            Err(e) => {
                                writeln!(output, "{e}")?;
                                break;
                            }
                        }
                    }
                }
                writeln!(output)?;
            }
            continue;
        }
        if verb.eq_ignore_ascii_case("stats") && argument.is_empty() {
            if let Some(character) = &state.character {
                writeln!(
                    output,
                    "Level {} | XP {} | HP {}/{} | MP {}/{} | AP {:.2}\n",
                    character.level,
                    character.experience,
                    character.hp,
                    character.max_hp,
                    character.mp,
                    character.max_mp,
                    f64::from(character.ap) / 100.0
                )?;
            } else {
                writeln!(output, "This adventure has no character progression.\n")?;
            }
            continue;
        }
        if (verb.eq_ignore_ascii_case("spellbook") || verb.eq_ignore_ascii_case("spells"))
            && argument.is_empty()
        {
            writeln!(
                output,
                "{}\n",
                content.spellbook(&state, &config.terminology.display)
            )?;
            continue;
        }
        if verb.eq_ignore_ascii_case("weapons") && argument.is_empty() {
            writeln!(output, "{}\n", content.weapons(&world, &state))?;
            continue;
        }
        if verb.eq_ignore_ascii_case("learn") {
            if argument.is_empty() {
                writeln!(output, "Learn what? Try 'learn <scroll>'.\n")?;
            } else {
                writeln!(output, "{}\n", content.learn(&world, &mut state, argument))?;
            }
            continue;
        }
        if verb.eq_ignore_ascii_case("cast") {
            if argument.is_empty() {
                writeln!(output, "Cast what? Try 'cast <ability> [on self]'.\n")?;
            } else {
                writeln!(output, "{}\n", content.cast(&mut state, argument))?;
            }
            continue;
        }
        if verb.eq_ignore_ascii_case("save") {
            // An explicit name authorizes writing that slot. Bare save asks once
            // before reusing the last successfully saved/loaded slot.
            let mut overwrite_authorized = !argument.is_empty();
            let name = if argument.is_empty() {
                if let Some(name) = &current_save {
                    match confirm(
                        &mut input,
                        &mut output,
                        &format!("Overwrite current save '{name}'? (yes/no):"),
                    )? {
                        Some(true) => overwrite_authorized = true,
                        Some(false) => (),
                        None => break,
                    }
                }
                if overwrite_authorized {
                    current_save.clone().expect("confirmed current save")
                } else {
                    match prompt(&mut input, &mut output, "Save name:")? {
                        Some(name) => name,
                        None => break,
                    }
                }
            } else {
                argument.to_owned()
            };
            if !overwrite_authorized
                && matches!(name.to_ascii_lowercase().as_str(), "" | "cancel" | "back")
            {
                writeln!(output, "Save canceled.\n")?;
                continue;
            }
            let overwrite = match slot_exists(root, game_id, &name) {
                Ok(true) if !overwrite_authorized => {
                    match confirm(
                        &mut input,
                        &mut output,
                        "Save exists. Overwrite? (yes/no): ",
                    )? {
                        Some(value) => value,
                        None => break,
                    }
                }
                Ok(exists) => exists && overwrite_authorized,
                Err(error) => {
                    writeln!(output, "Save failed: {error}\n")?;
                    continue;
                }
            };
            if !overwrite && slot_exists(root, game_id, &name).unwrap_or(false) {
                writeln!(output, "Save canceled.\n")?;
                continue;
            }
            match write_slot_with_compat(
                root,
                &world,
                &state,
                game_id,
                &name,
                overwrite,
                compatibility_version,
            ) {
                Ok(path) => {
                    last_saved = state.clone();
                    current_save = Some(name.clone());
                    writeln!(
                        output,
                        "Saved: {}\n",
                        path.file_name().map_or_else(
                            || "unknown".into(),
                            |name| name.to_string_lossy().into_owned()
                        )
                    )?;
                }
                Err(error) => writeln!(output, "Save failed: {error}\n")?,
            }
            continue;
        }
        if verb.eq_ignore_ascii_case("saves") && argument.is_empty() {
            match list_slots(root, game_id) {
                Ok(names) if names.is_empty() => writeln!(output, "No saved games.\n")?,
                Ok(names) => {
                    writeln!(output, "Saved games:")?;
                    for name in names {
                        writeln!(output, "  {name}")?;
                    }
                    writeln!(output)?;
                }
                Err(error) => writeln!(output, "Could not list saves: {error}\n")?,
            }
            continue;
        }
        if verb.eq_ignore_ascii_case("load") {
            let name = if argument.is_empty() {
                match prompt(&mut input, &mut output, "Load save name: ")? {
                    Some(name) => name,
                    None => break,
                }
            } else {
                argument.to_owned()
            };
            if name.trim().is_empty() {
                writeln!(output, "Load canceled.\n")?;
                continue;
            }
            // Validate the entire file before offering to discard the current session.
            match read_slot_with_compat(root, &world, game_id, &name, compatibility_version) {
                Ok(restored) => {
                    // Validate all story-specific encounter data before any confirmation or mutation.
                    if let Err(e) = encounters.validate_restored(&restored) {
                        writeln!(output, "Load failed: {e}\n")?;
                        continue;
                    }
                    if state != last_saved {
                        match confirm(
                            &mut input,
                            &mut output,
                            "Discard unsaved progress and load? (yes/no): ",
                        )? {
                            Some(true) => {}
                            Some(false) => {
                                writeln!(output, "Load canceled.\n")?;
                                continue;
                            }
                            None => break,
                        }
                    }
                    if progression.is_some() && restored.character.is_none() {
                        writeln!(output, "Load failed: this legacy save predates character progression. Keep the old game revision to load it, or migrate this save.\n")?;
                        continue;
                    }
                    if progression.is_none() && restored.character.is_some() {
                        writeln!(output, "Load failed: this save contains RPG character data but RPG is disabled or config/character.fig is absent. Check enabled and the game save_compatibility_version.\n")?;
                        continue;
                    }
                    if restored
                        .learned_abilities
                        .iter()
                        .any(|id| !content.abilities.contains_key(id))
                    {
                        writeln!(
                            output,
                            "Load failed: save references unknown learned ability.\n"
                        )?;
                        continue;
                    }
                    if let Err(e) = shops.validate_saved(&restored) {
                        writeln!(output, "Load failed: {e}\n")?;
                        continue;
                    }
                    if let Err(e) = wallet.validate_balances(&restored) {
                        writeln!(output, "Load failed: {e}\n")?;
                        continue;
                    }
                    state = restored;
                    // Pre-wallet RORK saves have no wallet field: missing balances default to zero,
                    // preventing a configured starting grant from being silently replayed.
                    for id in wallet.currencies.keys() {
                        state.wallet.entry(id.clone()).or_insert(0);
                    }
                    // C4: the validated encounter is restored directly from RORK;
                    // never reroll on load, since that could regenerate defeated enemies.
                    last_saved = state.clone();
                    current_save = Some(name.clone());
                    writeln!(output, "Loaded: {name}\n")?;
                    show_room(&world, &state, &mut output)?;
                }
                Err(error) => writeln!(output, "Load failed: {error}\n")?,
            }
            continue;
        }
        let parsed = if original_intent.is_ok() {
            crate::player_input::prepared_command(&line)
        } else {
            original_intent
        };
        match parsed {
            Ok(crate::commands::Command::Quit) if state != last_saved => {
                match confirm(
                    &mut input,
                    &mut output,
                    "Unsaved progress. Quit without saving? (yes/no): ",
                )? {
                    Some(true) => {
                        writeln!(output, "Goodbye!")?;
                        break;
                    }
                    Some(false) => continue,
                    None => {
                        writeln!(output, "Goodbye!")?;
                        break;
                    }
                }
            }
            Ok(crate::commands::Command::Examine(ref target))
                if crate::player_input::enemies(&encounters, &state)
                    .iter()
                    .any(|e| &e.id == target) =>
            {
                if let Some(enemy) = state
                    .active_enemies
                    .iter()
                    .find(|e| format!("{} #{}", e.enemy_id, e.instance_id) == *target)
                {
                    if let Some(definition) = encounters.enemies.get(&enemy.enemy_id) {
                        writeln!(
                            output,
                            "{}\n{}\n",
                            definition.description,
                            encounters.describe(std::slice::from_ref(enemy))
                        )?;
                    }
                }
            }
            Ok(crate::commands::Command::Help) => {
                writeln!(
                    output,
                    "{}\n{}\n",
                    crate::render::help_for_game(&config, !wallet.currencies.is_empty()),
                    crate::actions::help(&world, &state)
                )?;
            }
            Ok(command) => {
                let old_room = state.current_room.clone();
                let is_look = matches!(&command, crate::commands::Command::Look);
                let entering = matches!(&command, crate::commands::Command::Move(_));
                let (result, performed) =
                    crate::engine::execute_command_report(&world, &mut state, command);
                if let Some(step) = pending.as_mut() {
                    step.performed |= performed;
                    if entering && performed {
                        step.triggers
                            .push(crate::events::Trigger::Enter(state.current_room.clone()));
                    }
                }
                match result {
                    ActionOutcome::Message(message) => {
                        writeln!(output, "{message}\n")?;
                        if state.current_room != old_room {
                            // Temporary encounters disperse. Room/unique encounters survive retreat.
                            encounters.leave_room(&old_room, &mut state);
                            let room = state.current_room.clone();
                            encounters.enter_room(&room, &mut state, &mut rng);
                        }
                        if (state.current_room != old_room || is_look)
                            && !state.active_enemies.is_empty()
                        {
                            writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
                        }
                    }
                    ActionOutcome::Quit(message) => {
                        writeln!(output, "{message}")?;
                        break;
                    }
                }
            }
            Err(ParseError::Empty) => continue,
            Err(ParseError::MissingDirection) => {
                writeln!(output, "Where would you like to go? Try 'go north'.\n")?
            }
            Err(ParseError::MissingTarget(action)) => {
                let example = match action {
                    "take" => "take <object> or take all",
                    "drop" => "drop <object> or drop all",
                    "examine" => "examine <object>",
                    _ => "use <item> on <target>",
                };
                writeln!(output, "Please specify a target. Try '{example}'. 'look' shows nearby objects; 'inventory' shows what you carry.\n")?;
            }
            Err(ParseError::Unknown(command)) => {
                writeln!(
                    output,
                    "I don't understand '{command}'. Type 'help' for commands."
                )?;
                if let Some(suggestion) = crate::actions::suggestion(&command, &world, &state)
                    .or_else(|| suggest_verb(&command))
                {
                    writeln!(output, "Did you mean '{suggestion}'? Type '{suggestion}' to try it; suggestions do not run automatically.")?;
                }
                writeln!(output)?;
            }
        }
    }
    Ok(state)
}

fn is_defeated(state: &GameState) -> bool {
    state
        .character
        .as_ref()
        .is_some_and(|character| character.hp == 0)
}

// Lightweight command spelling suggestions. Never execute the suggestion.
fn sync_object_room(
    old_room: &str,
    world: &World,
    state: &mut GameState,
    encounters: &EncounterContent,
    rng: &mut EncounterRng,
    output: &mut impl Write,
) -> io::Result<()> {
    if state.current_room != old_room {
        encounters.leave_room(old_room, state);
        let room = state.current_room.clone();
        if state.events.ending.is_none() {
            encounters.enter_room(&room, state, rng);
        }
        show_room(world, state, output)?;
        if !state.active_enemies.is_empty() {
            writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
        }
    }
    Ok(())
}
fn run_authored(
    index: usize,
    world: &World,
    state: &mut GameState,
    encounters: &EncounterContent,
    rng: &mut EncounterRng,
    output: &mut impl Write,
) -> io::Result<bool> {
    let old_room = state.current_room.clone();
    let result = world.actions[index].execute(world, state);
    let performed = result.is_ok();
    let text = result.unwrap_or_else(|e| e);
    writeln!(output, "{text}\n")?;
    if state.current_room != old_room {
        encounters.leave_room(&old_room, state);
        let room = state.current_room.clone();
        if state.events.ending.is_none() {
            encounters.enter_room(&room, state, rng);
        }
        show_room(world, state, output)?;
        if !state.active_enemies.is_empty() {
            writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
        }
    }
    Ok(performed)
}

fn suggest_verb(command: &str) -> Option<String> {
    const VERBS: &[&str] = &[
        "look",
        "inventory",
        "examine",
        "take",
        "drop",
        "use",
        "attack",
        "cast",
        "learn",
        "spellbook",
        "spells",
        "weapons",
        "stats",
        "enemies",
        "wallet",
        "balance",
        "money",
        "shop",
        "buy",
        "sell",
        "save",
        "saves",
        "load",
        "quit",
        "help",
        "wait",
        "journal",
        "quests",
        "north",
        "south",
        "east",
        "west",
        "up",
        "down",
        "go",
        "move",
        "walk",
        "travel",
        "run",
        "head",
        "leave",
        "stab",
        "slash",
        "hit",
        "strike",
        "inspect",
        "open",
        "close",
        "unlock",
        "lock",
        "put",
        "consume",
        "eat",
        "drink",
        "grab",
        "get",
    ];
    let verb = command.split_whitespace().next()?;
    // A recognized verb with unsupported arguments needs syntax guidance, not
    // a different action (e.g. "look X" must never suggest "load X").
    if VERBS.contains(&verb) {
        return None;
    }
    if verb.len() < 3 || !verb.is_ascii() {
        return None;
    }
    let mut matches = VERBS
        .iter()
        .filter_map(|candidate| {
            let remainder = command.strip_prefix(verb).unwrap_or("").trim();
            if !remainder.is_empty()
                && matches!(
                    *candidate,
                    "inventory"
                        | "spellbook"
                        | "spells"
                        | "weapons"
                        | "stats"
                        | "enemies"
                        | "wallet"
                        | "balance"
                        | "money"
                        | "saves"
                        | "quit"
                        | "help"
                        | "north"
                        | "south"
                        | "east"
                        | "west"
                        | "up"
                        | "down"
                )
            {
                return None;
            }
            let distance = edit_distance_at_most_two(verb, candidate)?;
            (distance > 0).then_some((distance, *candidate))
        })
        .collect::<Vec<_>>();
    matches.sort_unstable();
    let (distance, best) = *matches.first()?;
    // A tie means there is no clear recommendation.
    if matches.get(1).is_some_and(|next| next.0 == distance) {
        return None;
    }
    let remainder = command.strip_prefix(verb).unwrap_or("");
    Some(format!("{best}{remainder}"))
}

fn edit_distance_at_most_two(a: &str, b: &str) -> Option<usize> {
    crate::resolution::distance(a, b)
}

fn prompt<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    text: &str,
) -> io::Result<Option<String>> {
    // Rustyline redraws the input line; keep the question above that line.
    writeln!(output, "{}", text.trim_end())?;
    output.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_owned()))
}

fn confirm<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    text: &str,
) -> io::Result<Option<bool>> {
    loop {
        let Some(answer) = prompt(input, output, text)? else {
            return Ok(None);
        };
        match answer.to_ascii_lowercase().as_str() {
            "yes" | "y" => return Ok(Some(true)),
            "no" | "n" => return Ok(Some(false)),
            _ => writeln!(output, "Please answer yes or no.")?,
        }
    }
}

fn show_room<W: Write>(world: &World, state: &GameState, output: &mut W) -> io::Result<()> {
    if let Some(room) = world.room(&state.current_room) {
        writeln!(output, "{}\n", describe_room(world, state, room))?;
    }
    Ok(())
}

#[cfg(test)]
mod polish_b_tests {
    use super::*;

    #[test]
    fn suggests_only_unambiguous_commands_without_executing() {
        assert_eq!(suggest_verb("walet"), Some("wallet".into()));
        assert_eq!(
            suggest_verb("attak goblin with dagger"),
            Some("attack goblin with dagger".into())
        );
        assert_eq!(suggest_verb("stasts"), Some("stats".into()));
        assert_eq!(suggest_verb("xyzzy"), None);
    }
}
