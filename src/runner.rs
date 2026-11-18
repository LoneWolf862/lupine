//! Standard terminal interface with engine-owned manual RORK commands.
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::combat;
use crate::encounters::{EncounterContent, EncounterRng};
use crate::engine::{execute_command, ActionOutcome};
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
        io::stdin().lock(),
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
    mut input: R,
    mut output: W,
) -> io::Result<()> {
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
    writeln!(output, "{title}")?;
    writeln!(output, "Powered by Lupine. Type 'help' for commands.\n")?;
    let mut rng = EncounterRng::from_clock();
    let room = state.current_room.clone();
    encounters.enter_room(&room, &mut state, &mut rng);
    // Starting an encounter is an in-memory state mutation, never an autosave.
    show_room(&world, &state, &mut output)?;
    if !state.active_enemies.is_empty() {
        writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
    }
    loop {
        write!(output, "> ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            writeln!(output, "\nGoodbye!")?;
            break;
        }
        let command = line.trim();
        let (verb, argument) = command
            .split_once(char::is_whitespace)
            .map_or((command, ""), |(verb, rest)| (verb, rest.trim()));
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
        if verb.eq_ignore_ascii_case("enemies") && argument.is_empty() {
            writeln!(output, "{}\n", encounters.describe(&state.active_enemies))?;
            continue;
        }
        // Combat commands are handled before the legacy object parser. Invalid actions
        // are explicitly free; successful attacks trigger one enemy retaliation phase.
        let physical = if verb.eq_ignore_ascii_case("attack") {
            Some(argument.to_owned())
        } else if verb.eq_ignore_ascii_case("use") {
            // Do not hijack ordinary object interactions such as 'use key on door'.
            argument.split_once(" on ").and_then(|(weapon, target)| {
                let known_weapon = content.weapons.values().any(|w| {
                    world.item(&w.item).is_some_and(|item| {
                        item.id.eq_ignore_ascii_case(weapon.trim())
                            || item.name.eq_ignore_ascii_case(weapon.trim())
                            || item
                                .aliases
                                .iter()
                                .any(|a| a.eq_ignore_ascii_case(weapon.trim()))
                    })
                });
                known_weapon.then(|| format!("{} with {}", target.trim(), weapon.trim()))
            })
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
            if let Some(message) = shops.use_transaction_item(&world, &mut state, &wallet, argument)
            {
                writeln!(output, "{message}\n")?;
                continue;
            }
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
        {
            let taking = verb.eq_ignore_ascii_case("take");
            let targets: Vec<_> = world
                .items
                .iter()
                .filter(|item| {
                    if taking {
                        item.portable
                            && state.location(&item.id)
                                == Some(crate::state::ItemLocation::Room(
                                    state.current_room.clone(),
                                ))
                    } else {
                        state.location(&item.id) == Some(crate::state::ItemLocation::Inventory)
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
                    let item = world.item(&id).expect("item ID came from the world");
                    if taking {
                        state
                            .item_locations
                            .insert(id, crate::state::ItemLocation::Inventory);
                        writeln!(output, "You take the {}.", item.name)?;
                    } else {
                        state.item_locations.insert(
                            id,
                            crate::state::ItemLocation::Room(state.current_room.clone()),
                        );
                        writeln!(output, "You drop the {}.", item.name)?;
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
            let name = if argument.is_empty() {
                match prompt(&mut input, &mut output, "Save name: ")? {
                    Some(name) => name,
                    None => break,
                }
            } else {
                argument.to_owned()
            };
            if name.trim().is_empty() {
                writeln!(output, "Save canceled.\n")?;
                continue;
            }
            let overwrite = match slot_exists(root, game_id, &name) {
                Ok(true) => {
                    match confirm(
                        &mut input,
                        &mut output,
                        "Save exists. Overwrite? (yes/no): ",
                    )? {
                        Some(value) => value,
                        None => break,
                    }
                }
                Ok(false) => false,
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
                    writeln!(output, "Loaded: {name}\n")?;
                    show_room(&world, &state, &mut output)?;
                }
                Err(error) => writeln!(output, "Load failed: {error}\n")?,
            }
            continue;
        }
        match parse_command(&line) {
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
            Ok(crate::commands::Command::Help) => {
                writeln!(
                    output,
                    "{}\n",
                    crate::render::help_with_terms(&config.terminology.display)
                )?;
            }
            Ok(command) => {
                let old_room = state.current_room.clone();
                let is_look = matches!(&command, crate::commands::Command::Look);
                match execute_command(&world, &mut state, command) {
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
            Err(ParseError::MissingTarget(action)) => writeln!(
                output,
                "Please specify a target. Try 'help' for the syntax of {action}.\n"
            )?,
            Err(ParseError::Unknown(command)) => {
                writeln!(
                    output,
                    "I don't understand '{command}'. Type 'help' for commands."
                )?;
                if let Some(suggestion) = suggest_verb(&command) {
                    writeln!(output, "Did you mean '{suggestion}'?")?;
                }
                writeln!(output)?;
            }
        }
    }
    Ok(())
}

fn is_defeated(state: &GameState) -> bool {
    state
        .character
        .as_ref()
        .is_some_and(|character| character.hp == 0)
}

// Lightweight command spelling suggestions. Never execute the suggestion.
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
        "north",
        "south",
        "east",
        "west",
        "up",
        "down",
    ];
    let verb = command.split_whitespace().next()?;
    if verb.len() < 3 || !verb.is_ascii() {
        return None;
    }
    let mut matches = VERBS
        .iter()
        .filter_map(|candidate| {
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
    if a.len().abs_diff(b.len()) > 2 {
        return None;
    }
    let mut previous = (0..=b.len()).collect::<Vec<_>>();
    for (row, c) in a.bytes().enumerate() {
        let mut next = vec![row + 1];
        for (col, other) in b.bytes().enumerate() {
            let substitution = usize::from(c != other);
            next.push(
                (previous[col + 1] + 1)
                    .min(next[col] + 1)
                    .min(previous[col] + substitution),
            );
        }
        previous = next;
    }
    (previous[b.len()] <= 2).then_some(previous[b.len()])
}

fn prompt<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    text: &str,
) -> io::Result<Option<String>> {
    write!(output, "{text}")?;
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
