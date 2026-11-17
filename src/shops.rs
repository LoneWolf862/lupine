//! Creator-authored shops, service stations, and generic atomic exchanges.
//! No external dependencies, network, UI requirement, or automatic saving.
use crate::rpg_content::RpgContent;
use crate::state::{GameState, ItemLocation};
use crate::wallet::WalletContent;
use crate::world::World;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    pub id: String,
    pub name: String,
    /// item, sell_item, restore_hp, restore_mp, learn_ability, or set_flag.
    #[serde(rename = "type")]
    pub kind: String,
    pub currency: String,
    pub price: u64,
    #[serde(default)]
    pub item: Option<String>,
    #[serde(default)]
    pub ability: Option<String>,
    #[serde(default)]
    pub flag: Option<String>,
    #[serde(default)]
    pub amount: Option<u32>,
    /// Absent = unlimited purchases. A finite stock counts successful purchases.
    #[serde(default)]
    pub stock: Option<u32>,
    #[serde(default)]
    pub requires_flags: Vec<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shop {
    pub id: String,
    pub name: String,
    pub room: String,
    pub offers: Vec<Offer>,
}
#[derive(Debug, Default, Clone)]
pub struct ShopContent {
    pub shops: BTreeMap<String, Shop>,
}

fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}
impl ShopContent {
    pub fn load(
        root: &Path,
        world: &World,
        wallet: &WalletContent,
        rpg: &RpgContent,
    ) -> Result<Self, String> {
        let dir = root.join("shops");
        if !dir.exists() {
            return Ok(Self::default());
        }
        let mut result = Self::default();
        let mut paths = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().is_some_and(|e| e == "toml") {
                paths.push(entry.path());
            }
        }
        paths.sort();
        for path in paths {
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let shop: Shop =
                toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            if !valid_id(&shop.id)
                || shop.name.trim().is_empty()
                || world.room(&shop.room).is_none()
            {
                return Err(format!("{}: invalid shop id/name/room", path.display()));
            }
            let mut ids = HashSet::new();
            for offer in &shop.offers {
                if !valid_id(&offer.id) || offer.name.trim().is_empty() || !ids.insert(&offer.id) {
                    return Err(format!(
                        "{}: duplicate or invalid offer id/name",
                        path.display()
                    ));
                }
                if !wallet.currencies.contains_key(&offer.currency) {
                    return Err(format!(
                        "{}: offer '{}' uses unknown currency",
                        path.display(),
                        offer.id
                    ));
                }
                if offer.requires_flags.iter().any(|f| !valid_id(f)) {
                    return Err(format!("{}: invalid requirement flag", path.display()));
                }
                if offer.stock == Some(0) {
                    return Err(format!(
                        "{}: stock must be positive or omitted",
                        path.display()
                    ));
                }
                match offer.kind.as_str() {
                    "item" | "sell_item" => {
                        if !offer
                            .item
                            .as_ref()
                            .is_some_and(|id| world.item(id).is_some_and(|i| i.portable))
                        {
                            return Err(format!(
                                "{}: offer '{}' must reference a portable item",
                                path.display(),
                                offer.id
                            ));
                        }
                    }
                    "restore_hp" | "restore_mp" => {
                        if offer.amount.is_none_or(|v| v == 0) {
                            return Err(format!(
                                "{}: restoration amount must be positive",
                                path.display()
                            ));
                        }
                    }
                    "learn_ability" => {
                        if !offer
                            .ability
                            .as_ref()
                            .is_some_and(|id| rpg.abilities.contains_key(id))
                        {
                            return Err(format!("{}: unknown ability", path.display()));
                        }
                    }
                    "set_flag" => {
                        if !offer.flag.as_ref().is_some_and(|v| valid_id(v)) {
                            return Err(format!("{}: invalid flag", path.display()));
                        }
                    }
                    _ => {
                        return Err(format!(
                            "{}: unknown offer type '{}'",
                            path.display(),
                            offer.kind
                        ))
                    }
                }
            }
            if result.shops.insert(shop.id.clone(), shop).is_some() {
                return Err(format!("{}: duplicate shop id", path.display()));
            }
        }
        // An item ID is unique in Lupine. It cannot be stocked at two shops.
        let mut reserved = HashSet::new();
        for shop in result.shops.values() {
            for offer in &shop.offers {
                if offer.kind == "item"
                    && !reserved.insert(offer.item.as_ref().expect("validated item"))
                {
                    return Err(format!(
                        "Item is stocked by multiple shop offers: {}",
                        offer.name
                    ));
                }
            }
        }
        Ok(result)
    }
    /// Reserve stock merchandise: not freely obtainable as a normal room pickup.
    /// Call only for a brand-new session, never after loading a RORK save.
    pub fn prepare_inventory(&self, state: &mut GameState) {
        for shop in self.shops.values() {
            for offer in &shop.offers {
                if offer.kind == "item" {
                    state.item_locations.insert(
                        offer.item.as_ref().expect("validated item").clone(),
                        ItemLocation::Consumed,
                    );
                }
            }
        }
    }
    pub fn describe(&self, state: &GameState, wallet: &WalletContent, name: &str) -> String {
        let Some(shop) = self.resolve_shop(state, name) else {
            return "No such shop is available here.".into();
        };
        let mut lines = vec![format!("{}:", shop.name)];
        for offer in &shop.offers {
            let currency = &wallet.currencies[&offer.currency];
            let key = format!("{}:{}", shop.id, offer.id);
            let count = state.shop_purchases.get(&key).copied().unwrap_or(0);
            let remaining = offer.stock.map(|stock| stock.saturating_sub(count));
            let stock = match remaining {
                Some(0) => " (sold out)".to_owned(),
                Some(n) => format!(" ({n} left)"),
                None => String::new(),
            };
            lines.push(format!(
                "  {} — {} {}{}",
                offer.name, offer.price, currency.name, stock
            ));
        }
        lines.join("\n")
    }
    fn resolve_shop<'a>(&'a self, state: &GameState, name: &str) -> Option<&'a Shop> {
        let here = self
            .shops
            .values()
            .filter(|s| s.room == state.current_room)
            .collect::<Vec<_>>();
        if name.is_empty() {
            return (here.len() == 1).then(|| here[0]);
        }
        here.into_iter()
            .find(|s| s.id.eq_ignore_ascii_case(name) || s.name.eq_ignore_ascii_case(name))
    }
    pub fn transact(
        &self,
        state: &mut GameState,
        wallet: &WalletContent,
        offer_query: &str,
        shop_query: &str,
    ) -> Result<String, String> {
        let shop = self
            .resolve_shop(state, shop_query)
            .ok_or("No such shop is available here.")?;
        let offer = shop
            .offers
            .iter()
            .find(|o| {
                o.id.eq_ignore_ascii_case(offer_query) || o.name.eq_ignore_ascii_case(offer_query)
            })
            .ok_or_else(|| format!("No offer '{offer_query}' at {}.", shop.name))?;
        let key = format!("{}:{}", shop.id, offer.id);
        if offer
            .stock
            .is_some_and(|v| state.shop_purchases.get(&key).copied().unwrap_or(0) >= v)
        {
            return Err("That offer is sold out.".into());
        }
        if offer
            .requires_flags
            .iter()
            .any(|flag| !state.flags.contains(flag))
        {
            return Err("You do not meet the requirements.".into());
        }
        // Transactional isolation: all checks and effects occur on a clone.
        // Nothing is deducted, awarded, or marked purchased until every step succeeds.
        let mut draft = state.clone();
        if offer.kind == "sell_item" {
            let id = offer.item.as_ref().expect("validated item offer");
            if draft.location(id) != Some(ItemLocation::Inventory) {
                return Err("You do not carry the required item.".into());
            }
            draft
                .item_locations
                .insert(id.clone(), ItemLocation::Consumed);
            wallet.credit(&mut draft, &offer.currency, offer.price)?;
        } else {
            wallet.debit(&mut draft, &offer.currency, offer.price)?;
            match offer.kind.as_str() {
                "item" => {
                    let id = offer.item.as_ref().expect("validated item offer");
                    if draft.location(id) == Some(ItemLocation::Inventory) {
                        return Err("You already own that item.".into());
                    }
                    // Items are reserved by ShopContent::prepare_inventory; only these
                    // (or previously consumed items) can be purchased.
                    if draft.location(id) != Some(ItemLocation::Consumed) {
                        return Err("That item is not available for purchase.".into());
                    }
                    draft
                        .item_locations
                        .insert(id.clone(), ItemLocation::Inventory);
                }
                "restore_hp" => {
                    let player = draft
                        .character
                        .as_mut()
                        .ok_or("This adventure has no character.")?;
                    if player.hp == player.max_hp {
                        return Err("Your HP is already full; no charge.".into());
                    }
                    player.hp = player
                        .hp
                        .saturating_add(offer.amount.unwrap_or(0))
                        .min(player.max_hp);
                }
                "restore_mp" => {
                    let player = draft
                        .character
                        .as_mut()
                        .ok_or("This adventure has no character.")?;
                    if player.mp == player.max_mp {
                        return Err("Your MP is already full; no charge.".into());
                    }
                    player.mp = player
                        .mp
                        .saturating_add(offer.amount.unwrap_or(0))
                        .min(player.max_mp);
                }
                "learn_ability" => {
                    if !draft
                        .learned_abilities
                        .insert(offer.ability.as_ref().expect("validated ability").clone())
                    {
                        return Err("You already know that ability.".into());
                    }
                }
                "set_flag" => {
                    if !draft
                        .flags
                        .insert(offer.flag.as_ref().expect("validated flag").clone())
                    {
                        return Err("That is already unlocked.".into());
                    }
                }
                _ => return Err("Unsupported offer.".into()),
            }
        }
        if offer.stock.is_some() {
            let next = draft
                .shop_purchases
                .get(&key)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("Stock counter overflow")?;
            draft.shop_purchases.insert(key, next);
        }
        *state = draft;
        Ok(format!(
            "Transaction complete: {} ({} {}).",
            offer.name, offer.price, wallet.currencies[&offer.currency].name
        ))
    }
    pub fn validate_saved(&self, state: &GameState) -> Result<(), String> {
        for (key, count) in &state.shop_purchases {
            let Some((shop_id, offer_id)) = key.split_once(':') else {
                return Err("Invalid saved shop purchase key".into());
            };
            let offer = self
                .shops
                .get(shop_id)
                .and_then(|s| s.offers.iter().find(|o| o.id == offer_id))
                .ok_or("Save references unknown shop offer")?;
            if offer.stock.is_none_or(|n| *count > n) {
                return Err("Save contains invalid shop stock usage".into());
            }
        }
        Ok(())
    }
}
