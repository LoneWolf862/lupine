//! Optional game-configured RPG progression. No automatic regeneration.
//! Stats use integer tenths (AP of 10 = 1.0x) for deterministic damage scaling.
use serde::{Deserialize, Serialize};

fn one() -> u32 {
    1
}
fn hp_default() -> u32 {
    100
}
fn mp_default() -> u32 {
    30
}
fn ap_default() -> u32 {
    100
}
fn hp_gain() -> u32 {
    10
}
fn mp_gain() -> u32 {
    5
}
fn ap_gain() -> u32 {
    10
}
fn exp_default() -> u64 {
    100
}
fn growth_default() -> u32 {
    120
}

/// Optional `config/character.fig`. Integer AP units are percentage points: 100 = 1.00x.
/// `max_level = 0` means no authored cap, still bounded by numeric limits.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progression {
    #[serde(default = "one")]
    pub starting_level: u32,
    #[serde(default)]
    pub max_level: u32,
    #[serde(default = "hp_default")]
    pub starting_hp: u32,
    #[serde(default = "mp_default")]
    pub starting_mp: u32,
    #[serde(default = "ap_default")]
    pub starting_ap: u32,
    #[serde(default = "hp_gain")]
    pub hp_per_level: u32,
    #[serde(default = "mp_gain")]
    pub mp_per_level: u32,
    #[serde(default = "ap_gain")]
    pub ap_per_level: u32,
    #[serde(default = "exp_default")]
    pub base_requirement: u64,
    /// Experience cost growth in percent, 120 = +20% per level.
    #[serde(default = "growth_default")]
    pub growth_percent: u32,
}

impl Default for Progression {
    fn default() -> Self {
        Self {
            starting_level: 1,
            max_level: 0,
            starting_hp: 100,
            starting_mp: 30,
            starting_ap: 100,
            hp_per_level: 10,
            mp_per_level: 5,
            ap_per_level: 10,
            base_requirement: 100,
            growth_percent: 120,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub level: u32,
    pub experience: u64,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    /// 100 means AP 1.00, 200 means AP 2.00.
    pub ap: u32,
}

impl Progression {
    pub fn validate(&self) -> Result<(), String> {
        if self.starting_level == 0 || (self.max_level != 0 && self.starting_level > self.max_level)
        {
            return Err(
                "starting_level must be >= 1 and <= max_level (unless max_level is 0)".into(),
            );
        }
        if self.starting_hp == 0 || self.starting_ap == 0 || self.base_requirement == 0 {
            return Err("starting_hp, starting_ap and base_requirement must be positive".into());
        }
        if !(100..=1000).contains(&self.growth_percent) {
            return Err("growth_percent must be 100..=1000".into());
        }
        self.stats_at(self.starting_level)?;
        Ok(())
    }

    pub fn stats_at(&self, level: u32) -> Result<(u32, u32, u32), String> {
        if level == 0 {
            return Err("Level cannot be zero".into());
        }
        let steps = level - 1;
        let calc = |start: u32, increment: u32| {
            increment
                .checked_mul(steps)
                .and_then(|g| start.checked_add(g))
                .ok_or_else(|| "Character progression exceeds numeric limits".to_owned())
        };
        Ok((
            calc(self.starting_hp, self.hp_per_level)?,
            calc(self.starting_mp, self.mp_per_level)?,
            calc(self.starting_ap, self.ap_per_level)?,
        ))
    }

    pub fn create_character(&self) -> Result<Character, String> {
        self.validate()?;
        let (max_hp, max_mp, ap) = self.stats_at(self.starting_level)?;
        Ok(Character {
            level: self.starting_level,
            experience: 0,
            hp: max_hp,
            max_hp,
            mp: max_mp,
            max_mp,
            ap,
        })
    }

    /// XP required for the next level; returns None at the level cap.
    pub fn next_requirement(&self, level: u32) -> Option<u64> {
        if level == 0 || level == u32::MAX || (self.max_level != 0 && level >= self.max_level) {
            return None;
        }
        let mut needed = self.base_requirement;
        // Saturation prevents overflow; avoid long loops on extreme levels.
        for _ in 1..level.min(10_000) {
            needed = needed.saturating_mul(u64::from(self.growth_percent)) / 100;
            if needed == u64::MAX {
                break;
            }
        }
        if level > 10_000 && self.growth_percent > 100 {
            needed = u64::MAX;
        }
        Some(needed.max(1))
    }

    /// Award XP, consuming thresholds and restoring HP/MP only on actual level-ups.
    /// If the next level would overflow stats, further leveling stops safely.
    pub fn award_xp(&self, player: &mut Character, amount: u64) -> u32 {
        player.experience = player.experience.saturating_add(amount);
        let mut gained = 0;
        while let Some(required) = self.next_requirement(player.level) {
            // Bound one award operation even for unlimited-level, constant-cost worlds.
            if gained >= 10_000 || player.experience < required {
                break;
            }
            let Some(next_level) = player.level.checked_add(1) else {
                break;
            };
            let Ok((hp, mp, ap)) = self.stats_at(next_level) else {
                break;
            };
            player.experience -= required;
            player.level = next_level;
            player.max_hp = hp;
            player.max_mp = mp;
            player.hp = hp;
            player.mp = mp;
            player.ap = ap;
            gained += 1;
        }
        gained
    }
}

impl Character {
    pub fn validate(&self) -> Result<(), String> {
        if self.level == 0
            || self.max_hp == 0
            || self.ap == 0
            || self.hp > self.max_hp
            || self.mp > self.max_mp
        {
            return Err("RORK character stats are invalid or outside their limits".into());
        }
        Ok(())
    }
    /// Base weapon damage scaled by AP; integer rounding half up.
    pub fn weapon_damage(&self, base: u32) -> u32 {
        let scaled = (u64::from(base) * u64::from(self.ap) + 50) / 100;
        scaled.min(u64::from(u32::MAX)) as u32
    }
    pub fn heal(&mut self, amount: u32) {
        self.hp = self.hp.saturating_add(amount).min(self.max_hp);
    }
    pub fn restore_mana(&mut self, amount: u32) {
        self.mp = self.mp.saturating_add(amount).min(self.max_mp);
    }
    pub fn spend_mana(&mut self, amount: u32) -> bool {
        if self.mp < amount {
            return false;
        }
        self.mp -= amount;
        true
    }
}
