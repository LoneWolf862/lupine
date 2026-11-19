//! Whole-player-turn regressions through the public runner, including real files/saves.
use lupine::{
    adventure::Adventure,
    state::{GameState, ItemLocation},
};
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine-turn-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        lupine::creator::new_game(&root, "atomic", "Atomic fixture").unwrap();
        let f = Self(root);
        f.write("game.toml", "format_version=1\nid='atomic'\ntitle='Atomic fixture'\nstarting_room='start'\n[currency]\nname='Gold'\nstarting_amount=10\n");
        f.write(
            "rooms/start.toml",
            "id='start'\ntitle='Start'\ndescription='Start'\n[exits]\nnorth='hall'\n",
        );
        f.write(
            "rooms/hall.toml",
            "id='hall'\ntitle='Hall'\ndescription='Hall'\n[exits]\nsouth='start'\n",
        );
        f.write(
            "rules.toml",
            &format!(
                "[flags]\nunlocked=false\nfault=true\n[counters]\noverflow={}\n",
                i64::MAX
            ),
        );
        f.write("items/token.toml", "id='token'\nname='token'\ndescription='Token'\nportable=true\nstackable=true\nquantity=2\ninitial={kind='npc',id='guide'}\n");
        f.write("NPCs/guide.toml", "id='guide'\nname='Guide'\ndescription='Guide fixture'\nroom='start'\nstart='hello'\n[nodes.hello]\ntext='Choose a gift.'\nchoices=[{id='gift',label='Receive a gift',next='done',text='Gift committed.',effects=[{transfer_item={item='token',quantity=1,from={kind='npc',id='guide'},to={kind='inventory'}}}]}]\n[nodes.done]\ntext='Gift was delivered.'\n");
        f.write("events.toml", "[timers.clock]\nturns=4\nstart=true\n[quests.task]\ntitle='Delivery'\ninitial='pending'\n[quests.task.stages]\npending='Waiting'\ndone='Delivered'\n[journal.proof]\ntitle='Proof'\ntext='Delivery recorded.'\n[[events]]\nid='progress'\non={take='token'}\neffects=[{set_quest_stage={quest='task',stage='done'}},{set_flag='unlocked'},{add_journal='proof'}]\ntext='Quest committed.'\n");
        f.write("interactions.toml", "[[exit_rules]]\nfrom='start'\ndirection='north'\nrequires=[{flag_set='unlocked'}]\nblocked_text='The route is locked.'\n");
        f.write("actions.toml", "[[actions]]\nid='repair'\ncommands=['repair']\nsuccess_text='Repaired.'\nfailure_text='Cannot repair.'\neffects=[{clear_flag='fault'}]\n");
        f
    }
    fn write(&self, path: &str, text: &str) {
        fs::write(self.0.join(path), text).unwrap();
    }
    fn append(&self, path: &str, text: &str) {
        let p = self.0.join(path);
        fs::write(&p, fs::read_to_string(&p).unwrap() + text).unwrap();
    }
    fn fail_on(&self, trigger: &str) {
        self.append("events.toml", &format!("\n[[events]]\nid='failure'\non={trigger}\nrequires=[{{flag_set='fault'}}]\neffects=[{{add_counter={{id='overflow',amount=1}}}}]\ntext='Failure success must stay hidden.'\n"));
    }
    fn play(&self, commands: &str) -> (GameState, String) {
        self.seeded(commands, 17)
    }
    fn seeded(&self, commands: &str, seed: u64) -> (GameState, String) {
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
            Cursor::new(commands),
            &mut output,
            Some(seed),
        )
        .unwrap();
        (state, String::from_utf8(output).unwrap())
    }
    fn rpg(&self, hp: u32) {
        self.write("interactions.toml", "");
        self.write("config/character.fig", "enabled=true\n");
        self.write("config/combat.fig", "retaliation='every_action'\n");
        self.write("items/sword.toml", "id='sword'\nname='sword'\ndescription='Sword'\nportable=true\ninitial={kind='inventory'}\n");
        self.write("weapons/sword.toml", "item='sword'\ndamage=999\n");
        self.write("items/coat.toml", "id='coat'\nname='coat'\ndescription='Coat'\nportable=true\ninitial={kind='inventory'}\n");
        self.write("systems.toml", "[equipment.coat]\nslot='body'\ndefense=1\n[statuses.poison]\nname='Poison'\nturns=3\ndamage=10\n");
        self.write(
            "spells/fire.toml",
            "id='fire'\nname='Fire'\nkind='damage'\npower=1\nmana_cost=2\non_hit_status='poison'\n",
        );
        self.append(
            "events.toml",
            "\n[[events]]\nid='teach'\non={enter='start'}\neffects=[{teach_ability='fire'}]\n",
        );
        self.write("enemies/rat.toml", &format!("id='rat'\nname='Rat'\ndescription='Opponent'\nlevel=1\nmax_hp={hp}\nexperience=5\nattacks=[{{name='Bite',damage=2,weight=1}},{{name='Claw',damage=5,weight=1}}]\nloot=[{{item='token',chance=100,quantity=2}}]\ncurrency_rewards=[{{currency='gold',chance=100,min=1,max=9}}]\n"));
        self.append("rooms/hall.toml", "\n[encounters]\npersistence='unique'\nchance=100\nmin_enemies=1\nmax_enemies=1\nenemies=[{enemy='rat',weight=1}]\n");
    }
    fn shop(&self, effects: &str) {
        self.write("shops/store.toml", &format!("id='store'\nname='Store'\nroom='start'\n[[offers]]\nid='gift'\nname='Gift'\ntype='item'\nitem='token'\nquantity=1\ncurrency='gold'\nprice=3\nstock=1\neffects={effects}\n"));
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn gift_quest_journal_exit_and_manual_save_commit_together() {
    let f = Fixture::new();
    let (s, text) = f.play("talk guide\ngift\nsave delivered\nnorth\nload delivered\nyes\n");
    assert_eq!(s.current_room, "start");
    assert_eq!(s.dialogue.npcs["guide"].node, "done");
    assert_eq!(s.events.quests["task"], "done");
    assert_eq!(s.events.journal, vec!["proof"]);
    assert_eq!(s.carried_quantity("token"), 1);
    assert!(s.flags.contains("unlocked"));
    assert_eq!(s.events.turn, 1);
    assert_eq!(s.events.timers["clock"].remaining, 3);
    assert!(
        text.contains("Gift committed.")
            && text.contains("Quest committed.")
            && text.contains("Loaded: delivered"),
        "{text}"
    );
    assert_eq!(fs::read_dir(f.0.join("saves/atomic")).unwrap().count(), 1);
    let (_, text) = f.play("load delivered\nyes\nnorth\n");
    assert!(!text.contains("The route is locked."));
}

#[test]
fn gift_failure_in_acquisition_timer_or_turn_restores_every_system_and_output() {
    for trigger in ["{take='token'}", "{timer='clock'}", "'turn'"] {
        let f = Fixture::new();
        if trigger.contains("timer") {
            let p = f.0.join("events.toml");
            f.write(
                "events.toml",
                &fs::read_to_string(p).unwrap().replace("turns=4", "turns=1"),
            );
        }
        f.fail_on(trigger);
        let (before, _) = f.play("");
        let (after, text) = f.play("talk guide\ngift\n");
        assert_eq!(after, before, "{trigger}: {text}");
        assert!(text.contains("all turn changes were rolled back"), "{text}");
        assert!(
            !text.contains("Gift committed.") && !text.contains("Quest committed."),
            "{text}"
        );
        assert!(!f.0.join("saves").exists());
    }
}

#[test]
fn failed_purchase_hook_refunds_payment_stock_inventory_and_quest_progress() {
    let f = Fixture::new();
    f.shop("[]");
    f.fail_on("'turn'");
    let (before, _) = f.play("");
    let (after, text) = f.play("buy gift\n");
    assert_eq!(after, before, "{text}");
    assert!(text.contains("all turn changes were rolled back"));
    assert!(!text.contains("Quest committed."));
}

#[test]
fn purchase_with_full_container_rolls_back_its_payment_and_created_stack() {
    let f = Fixture::new();
    f.write("items/bag.toml", "id='bag'\nname='bag'\ndescription='Bag'\nportable=true\ninitial={kind='inventory'}\n[container]\nopen=true\ncapacity=1\n");
    f.write("items/stone.toml", "id='stone'\nname='stone'\ndescription='Stone'\nportable=true\ninitial={kind='container',id='bag'}\n");
    f.shop("[{transfer_item={item='token',quantity=1,from={kind='inventory'},to={kind='container',id='bag'}}}]");
    let (before, _) = f.play("");
    let (after, text) = f.play("buy gift\n");
    assert_eq!(after, before, "{text}");
    assert!(text.to_lowercase().contains("capacity"), "{text}");
}

#[test]
fn physical_spell_and_status_defeat_reward_failures_restore_the_enemy_and_costs() {
    for (hp, command) in [
        (1, "attack rat with sword"),
        (1, "cast fire on rat"),
        (8, "cast fire on rat"),
    ] {
        let f = Fixture::new();
        f.rpg(hp);
        f.append(
            "enemies/rat.toml",
            "rewards=[{debit={currency='gold',amount=999}}]\n",
        );
        let (before, _) = f.play("north\n");
        let (after, text) = f.play(&format!("north\n{command}\n"));
        assert_eq!(after, before, "{command}: {text}");
        assert!(text.contains("all turn changes were rolled back"), "{text}");
        assert!(
            !text.contains("You strike")
                && !text.contains("You cast")
                && !text.contains("drops token"),
            "{text}"
        );
    }
}

#[test]
fn defeat_hook_failure_undoes_successful_rewards_and_persistent_completion() {
    let f = Fixture::new();
    f.rpg(1);
    f.fail_on("{defeat='rat'}");
    let (before, _) = f.play("north\n");
    let (after, text) = f.play("north\nattack rat with sword\n");
    assert_eq!(after, before, "{text}");
    assert!(text.contains("all turn changes were rolled back"));
    assert!(!text.contains("You collect") && !text.contains("drops token"));
}

#[test]
fn later_failure_restores_equipment_status_damage_and_retaliation() {
    let f = Fixture::new();
    f.rpg(100);
    f.append(
        "events.toml",
        "\n[[events]]\nid='poison_player'\non={enter='hall'}\neffects=[{apply_status='poison'}]\n",
    );
    f.append("events.toml", "\n[[events]]\nid='reject_equipment'\non='turn'\nrequires=[{equipped='coat'}]\neffects=[{add_counter={id='overflow',amount=1}}]\n");
    let (before, _) = f.play("north\n");
    let (after, text) = f.play("north\nequip coat\n");
    assert_eq!(after, before, "{text}");
    assert!(!text.contains("Equipped coat."));
    assert!(text.contains("all turn changes were rolled back"));
}

#[test]
fn failed_entry_restores_random_draws_for_later_encounters_and_rewards() {
    let f = Fixture::new();
    f.rpg(1);
    f.fail_on("{enter='hall'}");
    for seed in [1, 17, 999] {
        let (expected, _) = f.seeded("repair\nnorth\nattack rat with sword\n", seed);
        let (actual, text) = f.seeded("north\nrepair\nnorth\nattack rat with sword\n", seed);
        assert_eq!(actual, expected, "seed {seed}: {text}");
        assert_eq!(text.matches("all turn changes were rolled back").count(), 1);
    }
}

#[test]
fn failed_turn_does_not_touch_manual_save_and_loading_does_not_replay_hooks() {
    let f = Fixture::new();
    f.fail_on("{take='token'}");
    let (saved, _) = f.play("save baseline\n");
    let path = fs::read_dir(f.0.join("saves/atomic"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let bytes = fs::read(&path).unwrap();
    let (actual, text) = f.play("load baseline\nyes\ntalk guide\ngift\nsaves\n");
    assert_eq!(actual, saved, "{text}");
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert_eq!(fs::read_dir(f.0.join("saves/atomic")).unwrap().count(), 1);
}

#[test]
fn cyclic_event_failure_restores_the_action_and_all_earlier_events() {
    let f = Fixture::new();
    f.append("events.toml", "\n[[events]]\nid='begin_cycle'\non={take='token'}\neffects=[{move_player='hall'}]\n[[events]]\nid='cycle_hall'\non={enter='hall'}\nonce=false\neffects=[{move_player='start'}]\n[[events]]\nid='cycle_start'\non={enter='start'}\nonce=false\nrequires=[{flag_set='unlocked'}]\neffects=[{move_player='hall'}]\n");
    let (before, _) = f.play("");
    let (after, text) = f.play("talk guide\ngift\n");
    assert_eq!(after, before, "{text}");
    assert!(text.contains("128 firings"));
    assert!(!text.contains("Gift committed."));
    assert_eq!(
        after.count_at("token", &ItemLocation::Npc("guide".into())),
        2
    );
}

#[test]
fn failed_turn_restores_bulk_objects_consumption_and_world_ability_costs() {
    let f = Fixture::new();
    f.fail_on("'turn'");
    f.write("items/ration.toml", "id='ration'\nname='ration'\ndescription='Ration'\nportable=true\nstackable=true\nquantity=2\ninitial={kind='inventory'}\n[consumable]\neffects=[{credit={currency='gold',amount=3}}]\ntext='Ration consumed.'\nfailure_text='Cannot consume.'\n");
    f.write(
        "items/stone.toml",
        "id='stone'\nname='stone'\ndescription='Stone'\nportable=true\nstarting_room='start'\n",
    );
    f.write("spells/light.toml", "id='light'\nname='Light'\nkind='world'\neffects=[{debit={currency='gold',amount=1}},{set_flag='unlocked'}]\n");
    f.append(
        "events.toml",
        "\n[[events]]\nid='teach'\non={enter='start'}\neffects=[{teach_ability='light'}]\n",
    );
    let (before, _) = f.play("");
    for command in ["take all\n", "drop all\n", "use ration\n", "cast light\n"] {
        let (after, text) = f.play(command);
        assert_eq!(after, before, "{command}: {text}");
        assert!(
            text.contains("all turn changes were rolled back"),
            "{command}: {text}"
        );
        assert!(!text.contains("Ration consumed."));
    }
}

#[test]
fn dialogue_prompts_are_live_but_failed_action_success_text_is_never_written() {
    use std::{
        cell::RefCell,
        io::{self, BufRead, Read, Write},
        rc::Rc,
    };
    #[derive(Clone)]
    struct Shared(Rc<RefCell<Vec<u8>>>);
    impl Write for Shared {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    struct Input {
        inner: Cursor<&'static [u8]>,
        output: Shared,
        reads: usize,
    }
    impl Read for Input {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            self.inner.read(b)
        }
    }
    impl BufRead for Input {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            self.inner.fill_buf()
        }
        fn consume(&mut self, n: usize) {
            self.inner.consume(n)
        }
        fn read_line(&mut self, b: &mut String) -> io::Result<usize> {
            let bytes = self.output.0.borrow();
            let text = String::from_utf8_lossy(&bytes);
            assert!(
                text.ends_with("> "),
                "Input requested without visible prompt: {text}"
            );
            if self.reads == 1 {
                assert!(text.contains("Choose a response"));
            }
            if self.reads == 2 {
                assert!(text.contains("all turn changes were rolled back"));
            }
            assert!(!text.contains("Gift committed."));
            self.reads += 1;
            self.inner.read_line(b)
        }
    }
    let f = Fixture::new();
    f.fail_on("'turn'");
    let a = Adventure::load(&f.0).unwrap();
    let output = Shared(Rc::new(RefCell::new(Vec::new())));
    let input = Input {
        inner: Cursor::new(b"talk guide\ngift\n"),
        output: output.clone(),
        reads: 0,
    };
    lupine::runner::run_session_with_io(
        a.game.world,
        &a.game.title,
        &a.game.id,
        &f.0.join("saves"),
        a.game.save_compatibility_version,
        a.game.config,
        a.rpg,
        a.encounters,
        a.wallet,
        a.shops,
        input,
        output,
        Some(17),
    )
    .unwrap();
}
