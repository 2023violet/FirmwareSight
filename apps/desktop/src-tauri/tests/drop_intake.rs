//! M1 drag-and-drop intake against a real session: a held drop stages exactly what the dialog would, its
//! path never appears in what the WebView receives, and a token works once.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::Session;
use firmwaresight_desktop::service::FixtureCatalog;

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "fwsight-m1-drop-{label}-{:x}-{:x}",
            std::process::id(),
            nanos
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory creates");
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture_pair(dir: &TempDir) -> (PathBuf, PathBuf) {
    let catalog = FixtureCatalog::from_environment();
    let elf = catalog.artifact_path(firmwaresight_desktop::FixtureKey::P0DualRegion);
    let map = elf
        .parent()
        .expect("fixture directory")
        .join("firmware.map");
    let (elf_copy, map_copy) = (dir.join("app.elf"), dir.join("app.map"));
    std::fs::copy(&elf, &elf_copy).expect("copy elf");
    std::fs::copy(&map, &map_copy).expect("copy map");
    (elf_copy, map_copy)
}

fn session(dir: &TempDir) -> Session {
    Session::open(
        FixtureCatalog::from_environment(),
        dir.join("history.sqlite"),
    )
    .unwrap_or_else(|err| panic!("desktop session opens: {err}"))
}

fn no_path_in(json: &str, dir: &TempDir) {
    let root = dir.0.to_string_lossy();
    assert!(!json.contains(root.as_ref()), "a path crossed: {json}");
    assert!(
        !json.contains(":\\\\") && !json.contains('/'),
        "something path-shaped crossed: {json}"
    );
}

#[test]
fn an_elf_and_map_dropped_together_stage_like_the_dialog_and_cross_as_names() {
    let dir = TempDir::new("pair");
    let (elf, map) = fixture_pair(&dir);
    let session = session(&dir);

    let event = session.hold_drop(&[map, elf]).expect("a drop is described");
    let json = serde_json::to_string(&event).expect("serializes");
    no_path_in(&json, &dir);
    assert_eq!(event.phase, "dropped");
    assert_eq!(event.artifact_name.as_deref(), Some("app.elf"));
    assert_eq!(event.map_name.as_deref(), Some("app.map"));

    let token = event.token.expect("a dropped event carries a token");
    let selection = session
        .accept_drop(&token, None, "op-1")
        .expect("the drop stages");
    assert_eq!(selection.file_name, "app.elf");
    assert!(selection.map_attached);
    let summary = session
        .analyze_selection(&selection.selection_id, "op-2")
        .expect("the staged drop analyzes");
    assert!(
        !summary.memory.region_usage.is_empty(),
        "the MAP reached the analysis"
    );
}

#[test]
fn a_token_is_single_use_and_a_newer_drop_replaces_an_older_one() {
    let dir = TempDir::new("token");
    let (elf, _) = fixture_pair(&dir);
    let session = session(&dir);

    let first = session
        .hold_drop(std::slice::from_ref(&elf))
        .and_then(|e| e.token)
        .expect("token");
    let second = session
        .hold_drop(&[elf])
        .and_then(|e| e.token)
        .expect("token");
    assert_ne!(first, second);
    assert!(
        session.accept_drop(&first, None, "op-a").is_err(),
        "the older drop was replaced"
    );
    assert!(session.accept_drop(&second, None, "op-b").is_ok());
    assert!(
        session.accept_drop(&second, None, "op-c").is_err(),
        "a token works once"
    );
}

#[test]
fn a_map_alone_attaches_to_the_selection_the_ui_names_and_needs_one() {
    let dir = TempDir::new("map");
    let (elf, map) = fixture_pair(&dir);
    let session = session(&dir);
    let staged = session.stage_artifact(&elf, "op-stage").expect("stages");

    let lonely = session
        .hold_drop(std::slice::from_ref(&map))
        .and_then(|e| e.token)
        .expect("token");
    let refused = session
        .accept_drop(&lonely, None, "op-none")
        .expect_err("no selection");
    assert_eq!(refused.code, "ERR-INPUT-0001");

    let token = session
        .hold_drop(&[map])
        .and_then(|e| e.token)
        .expect("token");
    let attached = session
        .accept_drop(&token, Some(&staged.selection_id), "op-map")
        .expect("attaches");
    assert_eq!(attached.selection_id, staged.selection_id);
    assert!(attached.map_attached);
}

#[test]
fn a_refused_drop_holds_nothing_and_says_why() {
    let dir = TempDir::new("refused");
    let (elf, _) = fixture_pair(&dir);
    let session = session(&dir);
    let token = session
        .hold_drop(std::slice::from_ref(&elf))
        .and_then(|e| e.token)
        .expect("token");

    let folder = dir.join("sub");
    std::fs::create_dir_all(&folder).expect("dir");
    let refused = session.hold_drop(&[folder]).expect("described");
    assert_eq!(refused.phase, "refused");
    assert_eq!(refused.refusal.as_deref(), Some("directory"));
    assert!(refused.token.is_none());
    assert!(
        session.accept_drop(&token, None, "op").is_err(),
        "a refusal clears the held drop"
    );

    let too_many = session
        .hold_drop(&[elf.clone(), elf.clone(), Path::new("x.map").to_path_buf()])
        .expect("described");
    assert_eq!(too_many.refusal.as_deref(), Some("too_many"));
}
