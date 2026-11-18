use lupine::{
    adventure::Adventure,
    creator::{new_game, session},
};
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture {
    root: PathBuf,
    game: PathBuf,
    saves: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine-rpg-editor-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let game = root.join("game");
        new_game(&game, "rpg_editor", "RPG Editor").unwrap();
        fs::write(game.join("config/character.fig"), "enabled=true\n").unwrap();
        Self {
            saves: root.join("saves"),
            root,
            game,
        }
    }
    fn run(&self, s: &str) -> String {
        let mut out = Vec::new();
        session::open(&self.game, &mut Cursor::new(s), &mut out, &self.saves).unwrap();
        String::from_utf8(out).unwrap()
    }
    fn read(&self, p: &str) -> String {
        fs::read_to_string(self.game.join(p)).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
#[test]
fn enemy_fields_attacks_preview_and_cancel() {
    let f = Fixture::new();
    let out=f.run("r\n1\nn\nguard\n1\nPractice Guard\n3\n0\n2\n4\n30\n5\n125\n6\n2\n7\n100\n8\na\nTap\n3\n1\nyes\na\nPush\n5\n3\nyes\n0\np\ns\nno\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let source = f.read("enemies/guard.toml");
    assert!(out.contains(&source));
    let a = Adventure::load(&f.game).unwrap();
    let e = &a.encounters.enemies["guard"];
    assert_eq!(
        (e.level, e.max_hp, e.attack_power, e.defense, e.experience),
        (2, 30, 125, 2, 100)
    );
    assert_eq!(e.attacks.len(), 2);
    assert_eq!(e.attacks[1].weight, 3);
    assert!(!f.saves.exists());
    f.run("r\n1\n1\n1\nDiscarded\ncancel\n0\n0\n");
    assert_eq!(f.read("enemies/guard.toml"), source);
}
#[test]
fn enemy_preserves_comments_and_rejects_weight_overflow() {
    let f = Fixture::new();
    let original="# authored\nid='guard' # fixed\nname='Guard'\ndescription='Practice'\nlevel=1\nmax_hp=20\nattacks=[{name='Tap',damage=2,weight=4294967295}] # inline\n";
    fs::write(f.game.join("enemies/unusual.toml"), original).unwrap();
    let out = f.run("r\n1\n1\n8\na\nPush\n1\n1\nyes\n0\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("attack weights overflow"), "{out}");
    assert_eq!(f.read("enemies/unusual.toml"), original);
    let out = f.run("r\n1\n1\n1\nRenamed\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let text = f.read("enemies/unusual.toml");
    assert!(text.contains("# authored") && text.contains("# inline") && text.contains("# fixed"));
    assert_eq!(
        Adventure::load(&f.game).unwrap().encounters.enemies["guard"].name,
        "Renamed"
    );
}
#[test]
fn weapon_item_workflow_and_gameplay() {
    let f = Fixture::new();
    let out=f.run("r\n2\ni\nn\nsword\n1\ns\nyes\n0\nn\n1\n1\n0\n9\n2\nslashing\np\ns\nyes\n0\n0\n1\nweapons\ntake sword\nweapons\nsave gear\nload gear\nquit\nyes\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let a = Adventure::load(&f.game).unwrap();
    let w = &a.rpg.weapons["sword"];
    assert_eq!(w.damage, 9);
    assert_eq!(w.damage_type, "slashing");
    assert!(out.contains(&f.read("weapons/sword.toml")));
    assert!(f.saves.exists());
    let source = f.read("weapons/sword.toml");
    f.run("r\n2\n1\n1\n15\ncancel\n0\n0\n0\n");
    assert_eq!(f.read("weapons/sword.toml"), source);
    let out = f.run("r\n2\nn\n0\n0\n0\n");
    assert!(out.contains("No unused portable items"), "{out}");
}
#[test]
fn authored_shop_failed_transactions_stock_and_manual_roundtrip() {
    let f = Fixture::new();
    let manifest =
        f.read("game.toml") + "\n[currency]\nid='gold'\nname='Gold'\nstarting_amount=5\n";
    fs::write(f.game.join("game.toml"), manifest).unwrap();
    let out = f.run("r\n3\nn\nclinic\n1\n3\nheal\n1\n4\n3\n6\n2\n0\np\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let a = Adventure::load(&f.game).unwrap();
    let mut state = lupine::state::GameState::new(&a.game.world);
    state.character = Some(
        a.game
            .config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    state.wallet = a.wallet.initial_balances();
    let before = state.clone();
    assert!(a.shops.transact(&mut state, &a.wallet, "heal", "").is_err());
    assert_eq!(state, before);
    state.character.as_mut().unwrap().hp = 50;
    a.shops.transact(&mut state, &a.wallet, "heal", "").unwrap();
    assert_eq!(state.wallet["gold"], 2);
    assert_eq!(state.character.as_ref().unwrap().hp, 60);
    let before = state.clone();
    assert!(a.shops.transact(&mut state, &a.wallet, "heal", "").is_err());
    assert_eq!(state, before);
    let serialized = lupine::save::encode_save(&a.game.world, &state, &a.game.id, "shop").unwrap();
    let restored = lupine::save::decode_save(&a.game.world, &a.game.id, &serialized).unwrap();
    a.shops.validate_saved(&restored).unwrap();
    assert_eq!(restored, state);
    let original = f.read("shops/clinic.toml");
    f.run("r\n3\n1\n4\n1\n6\n1\n0\n0\n0\n0\n0\n");
    assert_eq!(f.read("shops/clinic.toml"), original);
}
#[test]
fn currencies_preserve_manifest_and_feed_rewards() {
    let f = Fixture::new();
    let original = f.read("game.toml");
    fs::write(
        f.game.join("game.toml"),
        format!("# identity stays\n{original}"),
    )
    .unwrap();
    let out=f.run("r\n4\n1\ngold\n1\nGold\n2\nG\n3\n25\n0\n2\ngold\ntokens\n1\nTokens\n2\nT\n3\n2\n0\np\ns\nyes\n1\nn\nguard\nc\na\n2\n100\n3\n3\nyes\n0\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let a = Adventure::load(&f.game).unwrap();
    assert_eq!(a.wallet.primary.as_deref(), Some("gold"));
    assert_eq!(a.wallet.initial_balances()["gold"], 25);
    assert_eq!(a.wallet.initial_balances()["tokens"], 2);
    assert_eq!(
        a.encounters.enemies["guard"].currency_rewards[0].currency,
        "tokens"
    );
    assert!(f.read("game.toml").contains("# identity stays"));
    assert!(out.contains(&f.read("game.toml")));
    assert!(!f.saves.exists());
    let source = f.read("game.toml");
    f.run("r\n4\n1\n1\nDiscarded\ncancel\n0\n0\n");
    assert_eq!(f.read("game.toml"), source);
}
#[test]
fn inline_currency_and_default_id_are_preserved() {
    let f = Fixture::new();
    let manifest = f.read("game.toml")
        + "\ncurrency={ name='Gold', symbol='g', starting_amount=9 } # keep inline\n";
    fs::write(f.game.join("game.toml"), manifest).unwrap();
    let out = f.run("r\n4\n1\n1\nCoins\n0\ns\nyes\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let s = f.read("game.toml");
    assert!(s.contains("# keep inline"));
    let v: toml::Value = s.parse().unwrap();
    assert!(v["currency"].get("id").is_none());
    let a = Adventure::load(&f.game).unwrap();
    assert_eq!(a.wallet.primary.as_deref(), Some("gold"));
    assert_eq!(a.wallet.currencies["gold"].name, "Coins");
}
#[test]
fn encounter_creates_enemy_in_same_transaction_and_discards_both() {
    let f = Fixture::new();
    let script = "7\n1\n7\n2\n1\n100\nn\nguard\n8\na\nTap\n1\n1\nyes\n0\ns\nyes\n0\n0\n";
    f.run(&(script.to_owned() + "0\n0\n0\n"));
    assert!(!f.game.join("enemies/guard.toml").exists());
    assert!(Adventure::load(&f.game)
        .unwrap()
        .encounters
        .tables
        .is_empty());
    let out = f.run(&(script.to_owned() + "6\nyes\n0\n0\n"));
    assert!(out.contains("saved"), "{out}");
    let a = Adventure::load(&f.game).unwrap();
    assert_eq!(a.encounters.tables["start"].enemies[0].enemy, "guard");
    assert_eq!(a.encounters.enemies["guard"].attacks.len(), 1);
}
#[test]
fn rpg_forms_eof_and_missing_references_publish_nothing() {
    for script in [
        "r\n1\nn\nguard\n4\n30\n",
        "r\n3\nn\nshop\n1\n1\nChanged\n",
        "r\n4\n1\ngold\n3\n20\n",
    ] {
        let f = Fixture::new();
        let before = f.read("game.toml");
        f.run(script);
        assert!(Adventure::load(&f.game)
            .unwrap()
            .encounters
            .enemies
            .is_empty());
        assert!(Adventure::load(&f.game).unwrap().shops.shops.is_empty());
        assert_eq!(f.read("game.toml"), before);
        assert!(!f.saves.exists());
    }
    let f = Fixture::new();
    let out =
        f.run("r\n1\nn\nguard\n9\na\n0\n100\nyes\n0\ns\nyes\n9\nr\n1\nyes\n0\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("Save needs attention"), "{out}");
    assert!(out.contains("Changes saved"), "{out}");
    assert!(
        Adventure::load(&f.game).unwrap().encounters.enemies["guard"]
            .loot
            .is_empty()
    );
}
#[test]
fn shop_and_weapon_edit_keep_comments_ids_and_unedited_fields() {
    let f = Fixture::new();
    f.run("4\nn\nsword\n1\ns\nyes\n0\n0\n");
    fs::write(
        f.game.join("weapons/odd.toml"),
        "# weapon\nitem='sword' # fixed\ndamage=5\ndamage_type='piercing' # keep\n",
    )
    .unwrap();
    let manifest = f.read("game.toml") + "\n[currency]\nname='Gold'\n";
    fs::write(f.game.join("game.toml"), manifest).unwrap();
    let shop="# shop\nid='shop'\nname='Shop'\nroom='start'\n[[offers]] # offer comment\nid='heal' # receipt\nname='Healing'\ntype='restore_hp'\ncurrency='gold'\nprice=2 # price\namount=3\nstock=2\nrequires_flags=['open'] # requirements\n";
    fs::write(f.game.join("shops/odd.toml"), shop).unwrap();
    let out = f.run("r\n2\n1\n1\n8\ns\nyes\n0\n3\n1\n4\n1\n4\n4\n0\ns\nyes\n0\n0\n0\n");
    assert!(out.contains("Changes saved"), "{out}");
    let weapon = f.read("weapons/odd.toml");
    for comment in ["# weapon", "# fixed", "# keep"] {
        assert!(weapon.contains(comment));
    }
    let shop = f.read("shops/odd.toml");
    for comment in [
        "# shop",
        "# offer comment",
        "# receipt",
        "# price",
        "# requirements",
    ] {
        assert!(shop.contains(comment), "{shop}");
    }
    let a = Adventure::load(&f.game).unwrap();
    assert_eq!(a.rpg.weapons["sword"].damage, 8);
    let o = &a.shops.shops["shop"].offers[0];
    assert_eq!(o.price, 4);
    assert_eq!(o.stock, Some(2));
    assert_eq!(o.requires_flags, vec!["open"]);
}
