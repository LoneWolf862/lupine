//! Assemble existing content validators for CLI and creator play/validation.
use crate::{
    encounters::EncounterContent,
    rpg_content::RpgContent,
    shops::ShopContent,
    story_loader::{self, LoadedGame},
    wallet::WalletContent,
};
use std::{
    io::{self, BufRead, Write},
    path::Path,
};

pub struct Adventure {
    pub game: LoadedGame,
    pub rpg: RpgContent,
    pub encounters: EncounterContent,
    pub wallet: WalletContent,
    pub shops: ShopContent,
}
impl Adventure {
    pub fn load(path: &Path) -> Result<Self, String> {
        let game = story_loader::load_game(path)?;
        let enabled = game.config.character.is_some();
        let rpg = RpgContent::load(path, &game.world, enabled)?;
        let rooms = game
            .world
            .rooms
            .iter()
            .map(|r| r.id.clone())
            .collect::<Vec<_>>();
        let encounters = EncounterContent::load(path, &rooms, enabled)?;
        encounters.validate_loot(&game.world)?;
        let wallet = WalletContent::load(path, enabled)?;
        wallet.validate_encounters(&encounters)?;
        let shops = ShopContent::load(path, &game.world, &wallet, &rpg)?;
        Ok(Self {
            game,
            rpg,
            encounters,
            wallet,
            shops,
        })
    }
    pub fn summary(&self, output: &mut impl Write) -> io::Result<()> {
        writeln!(output, "Valid story: {} ({})\n{} rooms, {} items, {} interactions, {} exit rules\n{} weapon definitions, {} abilities\n{} enemy definitions, {} encounter tables\n{} configured currencies, {} configured shops",
            self.game.title, self.game.id, self.game.world.rooms.len(), self.game.world.items.len(), self.game.world.interactions.len(), self.game.world.exit_rules.len(), self.rpg.weapons.len(), self.rpg.abilities.len(), self.encounters.enemies.len(), self.encounters.tables.len(), self.wallet.currencies.len(), self.shops.shops.len())
    }
    pub fn play<R: BufRead, W: Write>(
        self,
        saves: &Path,
        input: &mut R,
        output: &mut W,
    ) -> io::Result<()> {
        crate::runner::run_configured_with_io(
            self.game.world,
            &self.game.title,
            &self.game.id,
            saves,
            self.game.save_compatibility_version,
            self.game.config,
            self.rpg,
            self.encounters,
            self.wallet,
            self.shops,
            input,
            output,
        )
    }
}
