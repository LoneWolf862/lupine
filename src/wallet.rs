//! Optional story-configured currencies; independent of physical inventory.
use crate::encounters::{EncounterContent, RandomSource};
use crate::state::GameState;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Currency {
    #[serde(default = "default_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub starting_amount: u64,
}
fn default_id() -> String {
    "gold".into()
}

#[derive(Debug, Clone, Default)]
pub struct WalletContent {
    pub currencies: BTreeMap<String, Currency>,
    pub primary: Option<String>,
}
impl WalletContent {
    /// Read [currency] for the primary currency and optional [[currencies]] for extras.
    /// Absent entries mean a non-currency game. No file writes occur.
    pub fn load(root: &Path, rpg_enabled: bool) -> Result<Self, String> {
        let path = root.join("game.toml");
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let manifest: toml::Value =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let primary: Option<Currency> = manifest
            .get("currency")
            .map(|v| v.clone().try_into())
            .transpose()
            .map_err(|e| format!("{} [currency]: {e}", path.display()))?;
        let extras: Vec<Currency> = manifest
            .get("currencies")
            .map(|v| v.clone().try_into())
            .transpose()
            .map_err(|e| format!("{} [[currencies]]: {e}", path.display()))?
            .unwrap_or_default();
        if (!extras.is_empty() || primary.is_some()) && !rpg_enabled {
            return Err(
                "Currency configuration requires enabled = true in config/character.fig".into(),
            );
        }
        let mut result = Self::default();
        if let Some(currency) = primary {
            result.primary = Some(currency.id.clone());
            result.insert(currency)?;
        }
        for currency in extras {
            result.insert(currency)?;
        }
        if result.primary.is_none() {
            result.primary = result.currencies.keys().next().cloned();
        }
        Ok(result)
    }
    fn insert(&mut self, currency: Currency) -> Result<(), String> {
        if currency.id.is_empty()
            || currency.id.len() > 80
            || !currency
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
        {
            return Err(format!(
                "Invalid currency id '{}' (use lowercase letters, digits, '_' or '-')",
                currency.id
            ));
        }
        if currency.name.trim().is_empty() || currency.name.len() > 80 || currency.symbol.len() > 24
        {
            return Err(format!(
                "Currency '{}' must have a short nonempty name and symbol <= 24 bytes",
                currency.id
            ));
        }
        if self
            .currencies
            .insert(currency.id.clone(), currency)
            .is_some()
        {
            return Err("Duplicate currency id".into());
        }
        Ok(())
    }
    pub fn initial_balances(&self) -> BTreeMap<String, u64> {
        self.currencies
            .iter()
            .map(|(id, currency)| (id.clone(), currency.starting_amount))
            .collect()
    }
    pub fn validate_encounters(&self, encounters: &EncounterContent) -> Result<(), String> {
        for enemy in encounters.enemies.values() {
            let mut seen = HashSet::new();
            for reward in &enemy.currency_rewards {
                if !self.currencies.contains_key(&reward.currency) {
                    return Err(format!(
                        "Enemy '{}' references undefined currency '{}'",
                        enemy.id, reward.currency
                    ));
                }
                if reward.chance > 100 || reward.min > reward.max {
                    return Err(format!(
                        "Enemy '{}' currency reward must have chance 0..100 and min <= max",
                        enemy.id
                    ));
                }
                if reward.max - reward.min > u64::from(u32::MAX) - 1 {
                    return Err(format!(
                        "Enemy '{}' currency reward range is too large",
                        enemy.id
                    ));
                }
                if !seen.insert(reward.currency.as_str()) {
                    return Err(format!(
                        "Enemy '{}' repeats currency reward '{}'",
                        enemy.id, reward.currency
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn validate_balances(&self, state: &GameState) -> Result<(), String> {
        if state
            .wallet
            .keys()
            .any(|id| !self.currencies.contains_key(id))
        {
            return Err("Save contains currency not defined by this game".into());
        }
        Ok(())
    }
    pub fn describe(&self, state: &GameState) -> String {
        if self.currencies.is_empty() {
            return "This adventure has no currency wallet.".into();
        }
        let mut lines = vec!["Wallet:".to_string()];
        if let Some(id) = &self.primary {
            if let Some(c) = self.currencies.get(id) {
                lines.push(Self::line(state, c));
            }
        }
        for (id, c) in &self.currencies {
            if Some(id) != self.primary.as_ref() {
                lines.push(Self::line(state, c));
            }
        }
        lines.join("\n")
    }
    fn line(state: &GameState, currency: &Currency) -> String {
        let amount = state.wallet.get(&currency.id).copied().unwrap_or(0);
        if currency.symbol.is_empty() {
            format!("  {}: {}", currency.name, amount)
        } else {
            format!("  {}: {} {}", currency.name, amount, currency.symbol)
        }
    }
    /// Checked debit for future stores and scripted purchases. No negative balances.
    pub fn debit(&self, state: &mut GameState, id: &str, amount: u64) -> Result<(), String> {
        if !self.currencies.contains_key(id) {
            return Err(format!("Unknown currency '{id}'"));
        }
        let balance = state.wallet.entry(id.to_owned()).or_default();
        *balance = balance
            .checked_sub(amount)
            .ok_or_else(|| format!("Insufficient {id}"))?;
        Ok(())
    }
    pub fn credit(&self, state: &mut GameState, id: &str, amount: u64) -> Result<(), String> {
        if !self.currencies.contains_key(id) {
            return Err(format!("Unknown currency '{id}'"));
        }
        let balance = state.wallet.entry(id.to_owned()).or_default();
        *balance = balance
            .checked_add(amount)
            .ok_or_else(|| format!("Currency '{id}' balance overflow"))?;
        Ok(())
    }
    /// Called only after combat has removed a defeated enemy instance.
    pub fn award(
        &self,
        encounters: &EncounterContent,
        enemy_id: &str,
        state: &mut GameState,
        rng: &mut impl RandomSource,
    ) -> String {
        let Some(enemy) = encounters.enemies.get(enemy_id) else {
            return String::new();
        };
        let mut lines = Vec::new();
        for reward in &enemy.currency_rewards {
            if rng.below(100) >= reward.chance {
                continue;
            }
            let range = (reward.max - reward.min + 1) as u32;
            let amount = reward.min + u64::from(rng.below(range));
            if amount == 0 {
                continue;
            }
            match self.credit(state, &reward.currency, amount) {
                Ok(()) => {
                    let currency = &self.currencies[&reward.currency];
                    lines.push(format!("You collect {amount} {}.", currency.name));
                }
                Err(e) => lines.push(format!("Currency reward unavailable: {e}.")),
            }
        }
        lines.join("\n")
    }
}
