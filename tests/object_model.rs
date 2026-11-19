use lupine::{
    adventure::Adventure,
    objects::{self, Stack},
    state::{GameState, ItemLocation as L},
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
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lupine-objects-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        lupine::creator::new_game(&root, "object_test", "Objects Test").unwrap();
        fs::write(root.join("rules.toml"), "[counters]\nmeals=0\n").unwrap();
        for (id,portable,extra) in [
            ("key",true,"starting_room='start'\nhidden=true"),
            ("chest",false,"starting_room='start'\ncontainer={open=false,locked=true,key='key',capacity=3}"),
            ("bag",true,"initial={kind='inventory'}\ncontainer={open=true,capacity=5}"),
            ("gem",true,"initial={kind='container',id='chest'}"),
            ("ration",true,"initial={kind='inventory'}\nstackable=true\nquantity=5\nconsumable={text='A meal.',failure_text='Cannot eat.',effects=[{add_counter={id='meals',amount=1}}]}"),
        ] {
            fs::write(root.join(format!("items/{id}.toml")),format!("id='{id}'\nname='{id}'\ndescription='Fixture {id}'\nportable={portable}\n{extra}\n")).unwrap();
        }
        fs::write(root.join("actions.toml"),"[[actions]]\nid='search'\ncommands=['search']\neffects=[{reveal_item='key'}]\nsuccess_text='You find a key.'\nfailure_text='Nothing.'\n").unwrap();
        Self(root)
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
fn hidden_items_locks_quantity_splits_and_consumption_work_in_player_runner() {
    let f = Fixture::new();
    let (s,text)=f.play("take key\nopen chest\nsearch\ntake key\nunlock chest with key\nopen chest\ntake gem from chest\nput 2 ration in chest\ntake ration from chest\neat ration\ninventory\n");
    assert!(text.contains("It is locked."), "{text}");
    assert!(text.contains("A meal."), "{text}");
    assert_eq!(s.count_at("ration", &L::Inventory), 3, "{text}");
    assert_eq!(s.count_at("ration", &L::Container("chest".into())), 1);
    assert_eq!(s.story.counters["meals"], 1);
    assert!(s.is_carried("gem"));
    assert!(s.objects.containers["chest"].open);
    let a = Adventure::load(&f.0).unwrap();
    let saved = lupine::save::encode_save(&a.game.world, &s, &a.game.id, "test").unwrap();
    assert_eq!(
        lupine::save::decode_save(&a.game.world, &a.game.id, &saved).unwrap(),
        s
    );
    assert!(!f.0.join("saves").exists());
}
#[test]
fn closed_and_hidden_contents_stay_out_of_resolution_and_room_output() {
    let f = Fixture::new();
    let (s, text) = f.play("look\ntake all\nexamine gem\n");
    assert!(!s.is_carried("gem"));
    assert!(!s.is_carried("key"));
    assert!(!text.contains("Fixture gem"));
    assert!(!text.contains("Objects: key"));
    assert_eq!(s.count_at("ration", &L::Inventory), 5);
}
#[test]
fn containers_move_with_contents_and_prevent_cycles_and_overcapacity() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    objects::manipulate(w, &mut s, "put", "ration", Some("bag"), 3).unwrap();
    objects::manipulate(w, &mut s, "drop", "bag", None, 1).unwrap();
    assert_eq!(s.count_at("ration", &L::Container("bag".into())), 3);
    assert_eq!(s.carried_quantity("ration"), 2);
    objects::manipulate(w, &mut s, "take", "bag", None, 1).unwrap();
    let before = s.clone();
    assert!(objects::manipulate(w, &mut s, "put", "bag", Some("bag"), 1).is_err());
    assert_eq!(s, before);
    objects::manipulate(w, &mut s, "put", "ration", Some("bag"), 2).unwrap();
    let before = s.clone();
    assert!(objects::manipulate(w, &mut s, "put", "ration", Some("bag"), 1).is_err());
    assert_eq!(s, before);
    objects::manipulate(w, &mut s, "close", "bag", None, 1).unwrap();
    assert!(!s.is_accessible("ration"));
    assert_eq!(s.carried_quantity("ration"), 5);
}
#[test]
fn failed_consumption_rolls_back_the_item_counter_and_output() {
    let f = Fixture::new();
    fs::write(
        f.0.join("rules.toml"),
        format!("[counters]\nmeals={}\n", i64::MAX),
    )
    .unwrap();
    let (s, text) = f.play("consume ration\n");
    assert_eq!(s.carried_quantity("ration"), 5);
    assert_eq!(s.story.counters["meals"], i64::MAX);
    assert!(text.contains("overflow"));
    assert!(!text.contains("A meal."));
}
#[test]
fn invalid_definitions_and_malformed_saved_ledgers_are_rejected() {
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let original = GameState::new(w);
    let mut invalids = Vec::new();
    let mut s = original.clone();
    s.objects.placements.get_mut("ration").unwrap().push(Stack {
        location: L::Inventory,
        quantity: 1,
    });
    invalids.push(s);
    let mut s = original.clone();
    s.objects.placements.get_mut("ration").unwrap()[0].quantity = 0;
    invalids.push(s);
    let mut s = original.clone();
    s.objects.placements.get_mut("gem").unwrap()[0].location = L::Container("missing".into());
    invalids.push(s);
    let mut s = original.clone();
    s.objects.containers.get_mut("chest").unwrap().open = true;
    invalids.push(s);
    let mut s = original.clone();
    s.objects.hidden.insert("typo".into());
    invalids.push(s);
    for s in invalids {
        assert!(lupine::save::encode_save(w, &s, &a.game.id, "bad").is_err());
    }
    let p = f.0.join("items/bag.toml");
    let original = fs::read_to_string(&p).unwrap();
    for text in [
        original.replace("kind='inventory'", "kind='container',id='bag'"),
        original.replace("capacity=5", "capacity=0"),
        original.replace("portable=true", "portable=false"),
    ] {
        fs::write(&p, text).unwrap();
        assert!(Adventure::load(&f.0).is_err());
    }
}
#[test]
fn explicit_quantity_failures_and_container_cancellation_preserve_state() {
    let f = Fixture::new();
    let (baseline, _) = f.play("");
    let (state, text) = f.play("put 6 ration in bag\ndrop 0 ration\ntake gem from chest\n");
    assert_eq!(baseline, state, "{text}");
}
#[test]
fn migration_follows_typed_placements_keys_and_consumable_effects() {
    let f = Fixture::new();
    let mut output = Vec::new();
    lupine::creator::session::open(&f.0,&mut Cursor::new("M\nitems\nchest\nbox\nyes\nM\nitems\nkey\nsmall_key\nyes\nM\ncounters\nmeals\nfood\nyes\n0\n"),&mut output,&f.0.join("unused")).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert_eq!(text.matches("Migration saved.").count(), 3, "{text}");
    let a = Adventure::load(&f.0).unwrap();
    let s = GameState::new(&a.game.world);
    assert_eq!(s.location("gem"), Some(L::Container("box".into())));
    let (s, text) = f.play("consume ration\n");
    assert_eq!(s.story.counters["food"], 1, "{text}");
}

#[test]
fn object_demo_scenarios_assert_split_locations_and_visibility() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/objects_demo");
    let mut out = Vec::new();
    lupine::scenario::run(&root, &root.join("scenarios.toml"), &mut out).unwrap();
    assert_eq!(String::from_utf8_lossy(&out).matches("PASS:").count(), 3);
}

#[test]
fn quantity_effects_and_conditions_share_atomic_capacity_and_uniqueness_checks() {
    use lupine::rules::{self, Compare, Condition, Effect};
    let f = Fixture::new();
    let a = Adventure::load(&f.0).unwrap();
    let w = &a.game.world;
    let mut s = GameState::new(w);
    rules::apply(
        w,
        &mut s,
        &[
            Effect::AddItem {
                item: "ration".into(),
                quantity: 2,
            },
            Effect::RemoveQuantity {
                item: "ration".into(),
                quantity: 3,
            },
        ],
    )
    .unwrap();
    assert!(Condition::ItemCount {
        item: "ration".into(),
        op: Compare::Eq,
        value: 4,
        location: None
    }
    .met(&s));
    rules::apply(
        w,
        &mut s,
        &[
            Effect::SetContainer {
                item: "chest".into(),
                open: Some(true),
                locked: Some(false),
            },
            Effect::TransferItem {
                item: "ration".into(),
                from: L::Inventory,
                to: L::Container("chest".into()),
                quantity: 2,
            },
        ],
    )
    .unwrap();
    assert!(Condition::ContainerOpen("chest".into()).met(&s));
    assert!(!Condition::ContainerLocked("chest".into()).met(&s));
    let before = s.clone();
    assert!(rules::apply(
        w,
        &mut s,
        &[
            Effect::ShowText("must not appear".into()),
            Effect::TransferItem {
                item: "ration".into(),
                from: L::Inventory,
                to: L::Container("chest".into()),
                quantity: 1
            }
        ]
    )
    .is_err());
    assert_eq!(s, before);
    assert!(rules::apply(
        w,
        &mut s,
        &[Effect::AddItem {
            item: "key".into(),
            quantity: 1
        }]
    )
    .is_err());
    assert_eq!(s, before);
    let encoded = lupine::save::encode_save(w, &s, &a.game.id, "test").unwrap();
    let duplicate = encoded.replace("hidden = [\"key\"]", "hidden = [\"key\", \"key\"]");
    assert_ne!(duplicate, encoded);
    assert!(lupine::save::decode_save(w, &a.game.id, &duplicate).is_err());
}

#[test]
fn closed_bag_weapons_are_unusable_even_through_the_combat_api() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rpg_persistent");
    let mut a = Adventure::load(&root).unwrap();
    a.game.world.items.push(lupine::world::Item {
        id: "bag".into(),
        name: "bag".into(),
        aliases: vec![],
        description: "Bag".into(),
        descriptions: vec![],
        portable: true,
        starting_room: String::new(),
    });
    a.game.world.object_defs.insert(
        "bag".into(),
        objects::Definition {
            initial: Some(L::Inventory),
            container: Some(objects::Container::default()),
            ..Default::default()
        },
    );
    let w = &a.game.world;
    let mut s = GameState::new(w);
    s.character = Some(
        a.game
            .config
            .character
            .as_ref()
            .unwrap()
            .create_character()
            .unwrap(),
    );
    let weapon = a.rpg.weapons.keys().next().unwrap().clone();
    s.set_location(weapon.clone(), L::Container("bag".into()));
    s.current_room = "woods".into();
    a.encounters
        .enter_room("woods", &mut s, &mut lupine::encounters::EncounterRng(1));
    let mut active = std::mem::take(&mut s.active_enemies);
    let before = s.clone();
    let enemies = active.clone();
    let action = lupine::combat::physical_attack(
        w,
        &a.rpg,
        &a.encounters,
        &mut s,
        &mut active,
        &format!("goblin with {weapon}"),
    );
    assert!(!action.turn_spent);
    assert_eq!(s, before);
    assert_eq!(active, enemies);
    assert!(a.rpg.weapons(w, &s).contains("no usable weapons"));
}
