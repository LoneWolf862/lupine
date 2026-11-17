# Lupine v0.8.2 — Story-triggered transactions (candidate)

## Purpose
Game creators can connect an ordinary nonportable object to any configured shop offer, without implementing new Rust code or requiring the player to type `shop`/`buy`. This stage reuses the already-verified atomic purchase/sell/restore/learn/unlock logic and RORK stock tracking. **Never autosaves.**

## How to author an interaction

1. Make an object in `items/`, with `portable = false` and `starting_room` set to the room of the corresponding shop (e.g. `mana_altar`).
2. Define a normal shop/offer in `shops/` (e.g. healer's `mana` service: `type = "restore_mp"`, `currency = "gold"`, `price = 8`, `amount = 30`).
3. Create `transaction_interactions.toml`:

```toml
[[bindings]]
item = "mana_altar"
shop = "healer"
offer = "mana"
success_text = "The altar hums as it replenishes your mana."
```

The player can enter `use altar` or `use shrine`. The object must be accessible in the current room; the offer must also be available in that room. The engine runs exactly the same transaction as `buy mana from healer` with all-or-nothing state changes and stock accounting. Failed transactions do not charge or consume stock. Without `failure_text`, the original failure reason is shown (e.g. full MP, insufficient Gold). If provided, `failure_text` overrides the error shown to players.

## Scope
- **No changes** to legacy `interactions.toml` syntax or RORK schema. Older stories/saves keep working.
- Bindings currently support `use <object>` only, not `use item on object` or arbitrary verbs.
- Both bound object and its referenced shop offer must be in the same authored room, and the object must start there. The shop need not be displayed by story text.
- A configured shop offer acts as the reusable transaction definition. Later versions can decouple global transactions from shop locations.
- Objects with a binding are not automatically collectible or consumed.
- Existing shop stock is tracked with the same `shop_purchases` data already stored in RORK.
- Story creators can use offer kinds: `item`, `sell_item`, `restore_hp`, `restore_mp`, `learn_ability`, `set_flag`.

## Tests
Run `cargo fmt --all && cargo test && cargo clippy --all-targets -- -D warnings`.
Use `cargo run -- play samples/rpg_transactions`. On a full-mana character, `use altar` must refuse without charge. Learn Fireball, enter the woods, cast Fireball, return to camp, then `use altar`: mana should refill and 8 Gold should be deducted. Check `wallet`, `stats`, and a manual RORK save/load.
