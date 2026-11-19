use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "lupine-bindings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fn copy(a: &Path, b: &Path) {
            fs::create_dir(b).unwrap();
            for e in fs::read_dir(a).unwrap() {
                let e = e.unwrap();
                if e.path().is_dir() {
                    copy(&e.path(), &b.join(e.file_name()));
                } else {
                    fs::copy(e.path(), b.join(e.file_name())).unwrap();
                }
            }
        }
        copy(Path::new("samples/rpg_transactions"), &p);
        Self(p)
    }
    fn run(&self, script: &str) -> String {
        let mut out = Vec::new();
        lupine::creator::session::open(
            &self.0,
            &mut Cursor::new(script),
            &mut out,
            &self.0.join("isolated-saves"),
        )
        .unwrap();
        String::from_utf8(out).unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn binding_edit_remove_cancel_and_defaults_preserve_unrelated_content() {
    let f = Scratch::new();
    let path = f.0.join("transaction_interactions.toml");
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("# retained comment\n{original}")).unwrap();
    let before = fs::read(&path).unwrap();
    let out = f.run("t\nmana_altar\n2\nCustom success.\n3\nCustom failure.\n0\ns\nno\n0\n0\n");
    assert!(out.contains("draft retained"), "{out}");
    assert_eq!(fs::read(&path).unwrap(), before);
    let out = f.run("t\nmana_altar\n2\nCustom success.\n3\nCustom failure.\n0\ns\nyes\n0\n0\n");
    assert!(out.contains("Validation PASSED"), "{out}");
    let a = lupine::adventure::Adventure::load(&f.0).unwrap();
    assert_eq!(
        a.shops.transaction_bindings["mana_altar"]
            .failure_text
            .as_deref(),
        Some("Custom failure.")
    );
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("# retained comment"));
    f.run("t\nmana_altar\n4\n5\n0\ns\nyes\n0\n0\n");
    assert!(lupine::adventure::Adventure::load(&f.0)
        .unwrap()
        .shops
        .transaction_bindings["mana_altar"]
        .success_text
        .is_none());
    f.run("t\nd\nmana_altar\nyes\ns\nyes\nn\nmana_altar\nhealer\nmana\ns\nyes\n0\n0\n");
    assert_eq!(
        lupine::adventure::Adventure::load(&f.0)
            .unwrap()
            .shops
            .transaction_bindings
            .len(),
        1
    );
    assert!(!f.0.join("isolated-saves").exists());
}
#[test]
fn binding_draft_does_not_offer_duplicate_objects_or_save_on_eof() {
    let f = Scratch::new();
    let p = f.0.join("transaction_interactions.toml");
    let before = fs::read(&p).unwrap();
    let out = f.run("t\nn\nmana_altar\n0\n");
    assert!(out.contains("No matching choice"), "{out}");
    assert_eq!(fs::read(p).unwrap(), before);
}
