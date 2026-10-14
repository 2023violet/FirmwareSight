use std::path::{Path, PathBuf};

use firmwaresight_desktop::{Session, service::FixtureCatalog};

const CONFIG: &str = "schema_version = 1\n[project]\nname = \"bench controller\"\n[artifacts]\nrequired = [\"elf\", \"bin\", \"hex\"]\n[release]\nrelease_notes_path = \"RELEASE_NOTES.md\"\n";

struct TempWorld(PathBuf);

impl TempWorld {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "fwsight-next40-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).expect("create disposable world");
        Self(path)
    }
}

impl Drop for TempWorld {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove disposable world");
    }
}

fn session(world: &TempWorld) -> Session {
    Session::open(
        FixtureCatalog::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../fixtures")),
        world.0.join("store.sqlite"),
    )
    .expect("open isolated desktop session")
}

#[test]
fn readiness_is_none_without_a_loaded_project() {
    let world = TempWorld::new("empty-session");
    let session = session(&world);
    assert_eq!(
        session
            .inspect_project_readiness("op-test")
            .expect("inspect"),
        None
    );
}

#[test]
fn readiness_is_typed_path_free_and_keeps_loaded_gate_policy_unchanged() {
    let world = TempWorld::new("loaded-session");
    let project = world.0.join("project with spaces");
    std::fs::create_dir(&project).expect("create project");
    let config_path = project.join("firmwaresight.toml");
    std::fs::write(&config_path, CONFIG).expect("write project config");
    std::fs::write(project.join("RELEASE_NOTES.md"), "notes\n").expect("write release notes");
    let session = session(&world);
    session
        .load_project_from(Some(config_path.clone()), "op-load")
        .expect("load project");

    let first = session
        .inspect_project_readiness("op-test")
        .expect("inspect loaded project")
        .expect("project remains loaded");
    assert!(first.git_head_commit.is_none());
    assert_eq!(first.git_clean, None);
    assert_eq!(first.release_notes_status, "present");
    assert_eq!(first.declared_artifact_requirements, ["elf", "bin", "hex"]);
    assert!(!first.build_evidence_checked);
    assert!(first.matches_loaded_policy);
    let serialized = serde_json::to_string(&first).expect("serialize typed DTO");
    assert!(serialized.contains("not a Gate result"));
    assert!(!serialized.contains(&world.0.to_string_lossy().to_string()));

    let changed_config = CONFIG.replace("bench controller", "bench controller v2");
    std::fs::write(&config_path, changed_config).expect("externally edit project config");
    let refreshed = session
        .inspect_project_readiness("op-test-2")
        .expect("refresh")
        .expect("project remains loaded");
    assert!(!refreshed.matches_loaded_policy);
    assert_eq!(refreshed.project_name, "bench controller v2");

    std::fs::write(&config_path, "schema_version = 99\n").expect("write unsupported config");
    let refused = session
        .inspect_project_readiness("op-test-3")
        .expect_err("a config changed to an unsupported major is not observed as current");
    let error = serde_json::to_string(&refused).expect("serialize safe error");
    assert!(error.contains("ERR-CONFIG-7003"));
    assert!(!error.contains(&world.0.to_string_lossy().to_string()));
}

#[test]
fn context_and_policy_dtos_redact_untrusted_project_names_before_ipc() {
    let world = TempWorld::new("hidden-project-name");
    let project = world.0.join("project");
    std::fs::create_dir(&project).expect("create project");
    let config_path = project.join("firmwaresight.toml");
    let config = CONFIG.replace(
        "bench controller",
        "sample at C:\\\\Users\\\\canary-private\\\\workspace",
    );
    std::fs::write(&config_path, config).expect("write project config");
    std::fs::write(project.join("RELEASE_NOTES.md"), "notes\n").expect("write notes");
    let session = session(&world);
    let context = session
        .load_project_from(Some(config_path), "op-load-hidden")
        .expect("load project")
        .expect("context");

    assert_eq!(context.project_name, "[project name hidden]");
    assert_eq!(context.policy.project_name, "[project name hidden]");
    let serialized = serde_json::to_string(&context).expect("serialize context");
    assert!(!serialized.contains("canary-private"));

    let readiness = session
        .inspect_project_readiness("op-readiness-hidden")
        .expect("inspect")
        .expect("readiness");
    assert_eq!(readiness.project_name, "[project name hidden]");
    assert!(readiness.matches_loaded_policy);
}
