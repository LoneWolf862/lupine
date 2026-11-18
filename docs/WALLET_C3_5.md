# Lupine v0.8.0-C3.5 — Optional Currency Wallet

> Historical design/stage record: feature-status and compatibility claims below describe earlier development. Current policy and file paths are in [NEXT_PHASE_REQUIREMENTS.md](NEXT_PHASE_REQUIREMENTS.md) and [CONFIG_FIG_SPEC.md](CONFIG_FIG_SPEC.md). Character configuration now belongs in `config/character.fig`.

Opt-in `[currency]` in `game.toml`; `[[currencies]]` adds additional independent balances.
Each currency has `id` (stable lowercase identifier), `name`, `symbol`, and `starting_amount`.
The primary currency appears first. A story without currency continues unchanged.
`wallet`, `money`, and `balance` are equivalent read-only in-game commands.

Enemy TOML may include `[[currency_rewards]]` entries with `currency`, `chance` (0–100), `min`, `max`. These are rolled independently when an enemy is defeated and removed from an active encounter. Currency goes directly to wallet; it is not an inventory item. A duplicate enemy cannot pay twice after being removed.

Balances are unsigned 64-bit integers, with checked credits and debits. The wallet is saved only on explicit `save` into the normal RORK v1 save, and restored on explicit `load`. Older RORK saves without a wallet field still decode; when loaded by a wallet-enabled game, absent balances begin at zero rather than regranting new-game money. Changed currency IDs in a save are rejected by the story wallet loader. No in-game purchase commands or currency transfers yet; the checked debit API supports future shops.

**Compatibility:** RPG is opt-in via `character.toml`. Wallet additionally opt-in via `game.toml`. No autosaving or network access.

**C3.5 testing:** run `cargo fmt --all && cargo test && cargo clippy --all-targets -- -D warnings`, followed by `cargo run -- play samples/rpg_rewards`; `wallet`; defeat goblin; `wallet`; `save Wallet Test`; restart, `load Wallet Test`, `wallet`.

**Backlog:** static room descriptions referencing removed items, `spells` alias, parser typo suggestions, robust encounter persistence (C4).
