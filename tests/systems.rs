use lupine::{adventure::Adventure, state::GameState, state::ItemLocation};
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine-systems-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        lupine::creator::new_game(&root, "events_test", "Events Test").unwrap();
        fs::write(root.join("game.toml"),"format_version=1\nid='systems_test'\ntitle='Systems test'\nstarting_room='start'\n[currency]\nid='gold'\nname='Gold'\nstarting_amount=10\n").unwrap();
        fs::write(
            root.join("rooms/start.toml"),
            "id='start'\ntitle='Start'\ndescription='Test room'\n[exits]\nnorth='hall'\n",
        )
        .unwrap();
        fs::write(
            root.join("rooms/hall.toml"),
            "id='hall'\ntitle='Hall'\ndescription='Test hall'\n[exits]\nsouth='start'\n",
        )
        .unwrap();
        fs::write(
            root.join("rules.toml"),
            "[flags]\nlit=false\n[counters]\ndefeats=0\n",
        )
        .unwrap();
        fs::write(root.join("items/door.toml"),"id='door'\nname='door'\ndescription='A test target.'\nportable=false\nstarting_room='start'\n").unwrap();
        fs::write(root.join("items/token.toml"),"id='token'\nname='token'\ndescription='A test stack.'\nportable=true\nstackable=true\nquantity=2\ninitial={kind='inventory'}\n").unwrap();
        fs::write(root.join("spells/light.toml"),"id='light'\nname='Light'\nkind='world'\ntarget='door'\nrequires=[{balance={currency='gold',op='ge',value=1}}]\neffects=[{debit={currency='gold',amount=1}},{set_flag='lit'}]\n").unwrap();
        fs::write(
            root.join("events.toml"),
            "[[events]]\nid='learn'\non={enter='start'}\neffects=[{teach_ability='light'}]\n",
        )
        .unwrap();
        fs::write(root.join("shops/store.toml"),"id='store'\nname='Store'\nroom='start'\n[[offers]]\nid='tokens'\nname='Tokens'\ntype='item'\nitem='token'\nquantity=3\ncurrency='gold'\nprice=2\nstock=2\n[[offers]]\nid='sell'\nname='Sell tokens'\ntype='sell_item'\nitem='token'\nquantity=2\ncurrency='gold'\nprice=1\n").unwrap();
        Self(root)
    }
    fn rpg(&self) {
        fs::write(self.0.join("config/character.fig"), "enabled=true\n").unwrap();
        fs::write(self.0.join("config/combat.fig"), "retaliation='none'\n").unwrap();
        for (id, name) in [("sword", "sword"), ("coat", "coat")] {
            fs::write(self.0.join(format!("items/{id}.toml")),format!("id='{id}'\nname='{name}'\ndescription='Equipment fixture'\nportable=true\ninitial={{kind='inventory'}}\n")).unwrap();
        }
        fs::write(
            self.0.join("weapons/sword.toml"),
            "item='sword'\ndamage=10\n",
        )
        .unwrap();
        fs::write(self.0.join("systems.toml"),"[equipment.coat]\nslot='body'\ndefense=3\n[ equipment.coat.resistances ]\nfire=50\n[statuses.poison]\nname='Poison'\nturns=3\ndamage=2\ndamage_type='poison'\n[statuses.stun]\nname='Stun'\nturns=2\nstun=true\n").unwrap();
        fs::write(self.0.join("spells/fire.toml"),"id='fire'\nname='Fire'\nkind='damage'\npower=10\nmana_cost=2\ndamage_type='fire'\non_hit_status='poison'\n").unwrap();
        fs::write(
            self.0.join("spells/stun.toml"),
            "id='stun'\nname='Stun'\nkind='damage'\npower=1\non_hit_status='stun'\n",
        )
        .unwrap();
        let p = self.0.join("events.toml");
        fs::write(&p,fs::read_to_string(&p).unwrap().replace("{teach_ability='light'}","{teach_ability='light'},{teach_ability='fire'},{teach_ability='stun'}")+"\n[[events]]\nid='defeat'\non={defeat='rat'}\nonce=false\neffects=[{add_counter={id='defeats',amount=1}}]\n").unwrap();
        fs::write(self.0.join("enemies/rat.toml"),"id='rat'\nname='Rat'\ndescription='Test opponent'\nlevel=1\nmax_hp=20\ndefense=2\nexperience=5\nattacks=[{name='Flame',damage=10,weight=1,damage_type='fire'}]\nloot=[{item='token',chance=100,quantity=3}]\ncurrency_rewards=[{currency='gold',chance=100,min=2,max=2}]\n[resistances]\nfire=50\n").unwrap();
        let p = self.0.join("rooms/hall.toml");
        fs::write(&p,fs::read_to_string(&p).unwrap()+"\n[encounters]\npersistence='unique'\nchance=100\nmin_enemies=1\nmax_enemies=1\nenemies=[{enemy='rat',weight=1}]\n").unwrap();
    }
    fn play(&self, input: &str) -> (GameState, String) {
        let a = Adventure::load(&self.0).unwrap();
        let mut output = Vec::new();
        let state = lupine::runner::run_session_with_io(
            a.game.world,
            &a.game.title,
            &a.game.id,
            &self.0.join("saves"),
            a.game.save_compatibility_version,
            a.game.config,
            a.rpg,
            a.encounters,
            a.wallet,
            a.shops,
            Cursor::new(input),
            &mut output,
            Some(1),
        )
        .unwrap();
        (state, String::from_utf8(output).unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn narrative_economy_and_world_abilities_share_the_real_player_path() {
    let f = Fixture::new();
    let (s, t) = f.play("wallet\nbuy tokens\nsell sell\ncast light on door\nsave slot\n");
    assert!(s.character.is_none());
    assert_eq!(s.wallet["gold"], 8, "{t}");
    assert_eq!(s.carried_quantity("token"), 3);
    assert!(s.flags.contains("lit"));
    assert_eq!(s.events.turn, 3);
    assert!(t.contains("Saved:"));
    let a = Adventure::load(&f.0).unwrap();
    let saved = lupine::save::encode_save(&a.game.world, &s, &a.game.id, "test").unwrap();
    assert_eq!(
        lupine::save::decode_save(&a.game.world, &a.game.id, &saved).unwrap(),
        s
    );
}
#[test]
fn shops_preserve_starting_stacks_and_enforce_quantity_stock_and_funds() {
    let f = Fixture::new();
    let (s, t) = f.play("buy tokens\nbuy tokens\nbuy tokens\nsell sell\n");
    assert_eq!(s.carried_quantity("token"), 6, "{t}");
    assert_eq!(s.wallet["gold"], 7);
    assert_eq!(s.shop_purchases["store:tokens"], 2);
    assert_eq!(s.events.turn, 3);
    assert!(t.contains("sold out"));
}
#[test]
fn failed_shared_reward_rolls_back_currency_items_and_success_output() {
    use lupine::rules::{self, Effect};
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    s.wallet = a.wallet.initial_balances();
    let before = s.clone();
    assert!(rules::apply(
        w,
        &mut s,
        &[
            Effect::Credit {
                currency: "gold".into(),
                amount: 5
            },
            Effect::AddItem {
                item: "token".into(),
                quantity: 3
            },
            Effect::ShowText("Success".into()),
            Effect::Debit {
                currency: "gold".into(),
                amount: 99
            }
        ]
    )
    .is_err());
    assert_eq!(s, before);
}
#[test]
fn world_ability_wrong_target_and_unmet_cost_spend_no_turn_or_resources() {
    let f = Fixture::new();
    let (s, t) = f.play("cast light on token\ncast light\n");
    assert_eq!(s.events.turn, 0, "{t}");
    assert_eq!(s.wallet["gold"], 10);
    assert!(!s.flags.contains("lit"));
    let p = f.0.join("game.toml");
    fs::write(
        &p,
        fs::read_to_string(&p)
            .unwrap()
            .replace("starting_amount=10", "starting_amount=0"),
    )
    .unwrap();
    let (s, t) = f.play("cast light on door\n");
    assert_eq!(s.events.turn, 0, "{t}");
    assert!(!s.flags.contains("lit"));
}
#[test]
fn armor_and_damage_types_affect_combat_without_exposing_enemy_hp() {
    let f = Fixture::new();
    f.rpg();
    fs::write(f.0.join("config/combat.fig"), "retaliation='combat'\n").unwrap();
    let (s, t) = f.play("equip coat\nnorth\nattack rat with sword\n");
    assert_eq!(s.character.as_ref().unwrap().hp, 96, "{t}"); // (10 - 3) * 50%, rounded
    assert_eq!(s.active_enemies[0].current_hp, 12);
    assert!(!t.contains("12/20"));
    assert_eq!(s.systems.equipped["body"], "coat");
}
#[test]
fn damage_spell_resistance_and_status_tick_are_deterministic() {
    let f = Fixture::new();
    f.rpg();
    let (s, t) = f.play("north\ncast fire on rat\nlook\nwait\n");
    // Fire: (10-2)*50%=4; poison has minimum 1 after defense and ticks on cast + wait.
    assert_eq!(s.active_enemies[0].current_hp, 14, "{t}");
    assert_eq!(s.active_enemies[0].statuses["poison"], 1);
    assert_eq!(s.character.as_ref().unwrap().mp, 28);
    assert_eq!(s.events.turn, 3);
}
#[test]
fn final_status_turn_still_blocks_enemy_retaliation() {
    let f = Fixture::new();
    f.rpg();
    fs::write(f.0.join("config/combat.fig"), "retaliation='combat'\n").unwrap();
    let (s, t) = f.play("north\ncast stun on rat\nattack rat with sword\n");
    assert_eq!(s.character.as_ref().unwrap().hp, 100, "{t}");
    assert!(s.active_enemies[0].statuses.is_empty());
}
#[test]
fn poison_defeat_awards_stack_loot_currency_xp_and_event_once() {
    let f = Fixture::new();
    f.rpg();
    let p = f.0.join("enemies/rat.toml");
    fs::write(
        &p,
        fs::read_to_string(&p)
            .unwrap()
            .replace("max_hp=20", "max_hp=5"),
    )
    .unwrap();
    let (s, t) = f.play("north\ncast fire on rat\nwait\n");
    assert!(s.active_enemies.is_empty(), "{t}");
    assert_eq!(s.story.counters["defeats"], 1);
    assert_eq!(s.character.as_ref().unwrap().experience, 5);
    assert_eq!(s.wallet["gold"], 12);
    assert_eq!(s.carried_quantity("token"), 2);
    assert_eq!(s.count_at("token", &ItemLocation::Room("hall".into())), 3);
    assert_eq!(t.matches("You collect 2 Gold").count(), 1);
}
#[test]
fn retaliation_policy_preserves_free_commands_and_defaults() {
    let f = Fixture::new();
    f.rpg();
    for (mode, hp) in [("combat", 100), ("none", 100), ("every_action", 80)] {
        fs::write(
            f.0.join("config/combat.fig"),
            format!("retaliation='{mode}'\n"),
        )
        .unwrap();
        let (s, t) = f.play("north\nlook\nhelp\nattack missing with sword\nwait\n");
        assert_eq!(s.character.as_ref().unwrap().hp, hp, "{mode}: {t}");
        assert_eq!(s.events.turn, 2);
    }
}
#[test]
fn dropping_equipment_removes_its_benefit_and_saves_roundtrip_live_statuses() {
    let f = Fixture::new();
    f.rpg();
    let (s, t) =
        f.play("equip coat\nnorth\ncast fire on rat\nsave live\ndrop coat\nload live\nyes\n");
    assert!(t.contains("Loaded: live"), "{t}");
    assert_eq!(s.systems.equipped["body"], "coat");
    assert_eq!(s.active_enemies[0].statuses["poison"], 2);
    let (s, t) = f.play("equip coat\ndrop coat\n");
    assert!(s.systems.equipped.is_empty(), "{t}");
}
#[test]
fn status_effect_and_resource_conditions_work_in_atomic_shared_rules() {
    use lupine::rules::{self, Compare, Condition, Effect};
    let f = Fixture::new();
    f.rpg();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    s.character = Some(a.game.config.character.unwrap().create_character().unwrap());
    s.wallet = a.wallet.initial_balances();
    rules::apply(
        w,
        &mut s,
        &[
            Effect::ApplyStatus("poison".into()),
            Effect::DamagePlayer {
                amount: 10,
                damage_type: "physical".into(),
            },
            Effect::SpendMp(5),
            Effect::RestoreHp(3),
            Effect::RestoreMp(2),
        ],
    )
    .unwrap();
    assert!(Condition::Hp {
        op: Compare::Eq,
        value: 93
    }
    .met(&s));
    assert!(Condition::Mp {
        op: Compare::Eq,
        value: 27
    }
    .met(&s));
    assert!(Condition::StatusActive("poison".into()).met(&s));
    let text = lupine::save::encode_save(w, &s, &a.game.id, "status").unwrap();
    assert_eq!(lupine::save::decode_save(w, &a.game.id, &text).unwrap(), s);
    s.systems.statuses.insert("poison".into(), 99);
    assert!(lupine::save::encode_save(w, &s, &a.game.id, "bad").is_err());
}
#[test]
fn invalid_cross_system_definitions_fail_before_play() {
    let f = Fixture::new();
    f.rpg();
    let p = f.0.join("systems.toml");
    let text = fs::read_to_string(&p).unwrap();
    for bad in [
        text.replace("fire=50", "fire=101"),
        text.replace("turns=3", "turns=0"),
        text.replace("equipment.coat", "equipment.missing"),
    ] {
        fs::write(&p, bad).unwrap();
        assert!(Adventure::load(&f.0).is_err());
    }
    fs::write(&p, text).unwrap();
    fs::write(f.0.join("actions.toml"),"[[actions]]\nid='reward'\ncommands=['reward']\neffects=[{credit={currency='missing',amount=1}}]\nsuccess_text='Done'\nfailure_text='No'\n").unwrap();
    assert!(Adventure::load(&f.0).is_err());
}
#[test]
fn shared_shop_effect_failure_preserves_stock_and_payment() {
    let f = Fixture::new();
    let p = f.0.join("shops/store.toml");
    let text = fs::read_to_string(&p).unwrap().replace(
        "stock=2",
        "stock=2\neffects=[{debit={currency='gold',amount=99}}]",
    );
    fs::write(&p, text).unwrap();
    let (s, t) = f.play("buy tokens\n");
    assert_eq!(s.wallet["gold"], 10, "{t}");
    assert_eq!(s.carried_quantity("token"), 2);
    assert!(s.shop_purchases.is_empty());
    assert_eq!(s.events.turn, 0);
}
#[test]
fn full_immunity_and_zero_minimum_are_implemented_configuration() {
    let f = Fixture::new();
    f.rpg();
    let p = f.0.join("enemies/rat.toml");
    let text = fs::read_to_string(&p)
        .unwrap()
        .replace("fire=50", "fire=100");
    fs::write(&p, text).unwrap();
    fs::write(
        f.0.join("config/combat.fig"),
        "minimum_damage=0\nretaliation='none'\n",
    )
    .unwrap();
    let (s, t) = f.play("north\ncast fire on rat\n");
    assert_eq!(s.active_enemies[0].current_hp, 20, "{t}");
    assert_eq!(s.character.as_ref().unwrap().mp, 28);
    assert_eq!(s.events.turn, 2);
}
#[test]
fn new_system_ids_migrate_across_definitions_and_shared_references() {
    let f = Fixture::new();
    f.rpg();
    fs::write(f.0.join("manual.rork"), "untouched").unwrap();
    let mut out = Vec::new();
    let script="M\nitems\ncoat\nvest\nyes\nM\nstatuses\npoison\nvenom\nyes\nM\ncurrencies\ngold\ncoins\nyes\nM\nspells\nlight\nshine\nyes\n0\n";
    lupine::creator::session::open(
        &f.0,
        &mut Cursor::new(script),
        &mut out,
        &f.0.join("unused"),
    )
    .unwrap();
    let t = String::from_utf8(out).unwrap();
    assert_eq!(t.matches("Migration saved.").count(), 4, "{t}");
    let a = Adventure::load(&f.0).unwrap();
    assert!(a.game.world.systems.equipment.contains_key("vest"));
    assert_eq!(
        a.rpg.abilities["fire"].on_hit_status.as_deref(),
        Some("venom")
    );
    let (s, t) = f.play("cast shine on door\n");
    assert_eq!(s.wallet["coins"], 9, "{t}");
    assert_eq!(
        fs::read_to_string(f.0.join("manual.rork")).unwrap(),
        "untouched"
    );
}

#[test]
fn shipped_narrative_economy_scenarios_pass() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/economy_demo");
    let mut out = Vec::new();
    lupine::scenario::run(&root, &root.join("scenarios.toml"), &mut out).unwrap();
    assert_eq!(String::from_utf8_lossy(&out).matches("PASS:").count(), 3);
}

#[test]
fn saves_reject_unknown_money_and_non_world_abilities_without_character() {
    let f = Fixture::new();
    f.rpg();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    s.wallet.insert("missing".into(), 1);
    assert!(lupine::save::encode_save(w, &s, &a.game.id, "bad").is_err());
    s.wallet.clear();
    s.learned_abilities.insert("fire".into());
    assert!(lupine::save::encode_save(w, &s, &a.game.id, "bad").is_err());
    s.learned_abilities.clear();
    s.learned_abilities.insert("light".into());
    assert!(lupine::save::encode_save(w, &s, &a.game.id, "narrative").is_ok());
}

#[test]
fn shared_transaction_and_defeat_movement_synchronize_encounters() {
    let f = Fixture::new();
    f.rpg();
    let p = f.0.join("shops/store.toml");
    fs::write(&p,fs::read_to_string(&p).unwrap()+"\n[[offers]]\nid='portal'\nname='Portal'\ntype='effects'\ncurrency='gold'\nprice=1\neffects=[{move_player='hall'}]\n").unwrap();
    let (s, t) = f.play("buy portal\n");
    assert_eq!(s.current_room, "hall", "{t}");
    assert_eq!(s.active_enemies.len(), 1);
    assert_eq!(s.next_enemy_id, 2);
    let p = f.0.join("enemies/rat.toml");
    fs::write(
        &p,
        fs::read_to_string(&p)
            .unwrap()
            .replace(
                "experience=5",
                "experience=5\nrewards=[{move_player='start'}]",
            )
            .replace("max_hp=20", "max_hp=1"),
    )
    .unwrap();
    let (s, t) = f.play("north\nattack rat with sword\n");
    assert_eq!(s.current_room, "start", "{t}");
    assert!(s.active_enemies.is_empty());
    assert!(s.completed_unique_encounters.contains("hall"));
}
