# v0.8.1 — Creator-controlled transactions and shops (candidate)

Create `shops/<id>.toml` with `id`, `name`, `room`, and `[[offers]]`. Offers support `type` values `item`, `sell_item`, `restore_hp`, `restore_mp`, `learn_ability`, `set_flag`, plus `currency`, `price`, optional `stock`, optional `requires_flags`, and effect-specific `item`, `ability`, `flag`, `amount`. No shop files means existing games behave as before.

Commands: `shop` (one shop in current room), `shop <name>`, `buy <offer>`, `buy <offer> from <shop>`; `sell` also accesses declared offers. Game creators decide narrative context. Offers only work in their configured room; costs/effects are atomic, failed purchase changes nothing. `stock` tracks successful purchases in RORK and does not reset on load.

**Item limitation:** existing Lupine items are unique by ID. Item merchandise is automatically reserved from ordinary room pickups on a fresh session, then moved into inventory when purchased. Items cannot be stocked by multiple offers. Stock quantities count purchases, but do not create duplicate item instances; stackable inventory is a future enhancement. A sell offer consumes the carried item and pays out currency.

Manual `save` only. Old RORK saves default to empty shop-purchase counters. Transactions are restored and validated against present shop IDs and finite stock; changes to the game's shop catalog may require bumping `save_compatibility_version`.

No arbitrary scripts or executable callbacks are accepted.
