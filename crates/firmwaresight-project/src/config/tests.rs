use super::*;
use crate::fixtures::{VALID_CONFIG, temp_project};
use firmwaresight_core::domain::gate::UnknownDisposition;
use std::path::Path;

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn parse(text: &str) -> Result<LoadedProject, ProjectError> {
    LoadedProject::parse(Path::new("/project/firmwaresight.toml"), text)
}

fn loaded(text: &str) -> LoadedProject {
    parse(text).unwrap_or_else(|error| panic!("the config should load: {error}"))
}

#[test]
fn the_reference_shape_loads_with_no_warnings() {
    let project = loaded(VALID_CONFIG);
    assert_eq!(project.config.project.name, "motor-controller");
    assert!(
        project.warnings.is_empty(),
        "a valid config must not warn: {:?}",
        project.warnings
    );
    assert!(project.unknown_keys.is_empty());
    assert_eq!(project.root, PathBuf::from("/project"));
    assert_eq!(project.policy.flash_budget, Some(262_144));
    assert_eq!(project.policy.ram_budget, Some(131_072));
    assert_eq!(project.policy.required_artifact_kinds, vec!["elf", "map"]);
    assert_eq!(project.policy.unknown_evidence_review_count, Some(1));
    assert_eq!(
        project.policy.version.as_ref().map(|v| v.pattern.as_str()),
        Some(r"^v(?P<version>\d+\.\d+\.\d+)$")
    );
    assert_eq!(
        project.policy.on_unknown.git_clean,
        UnknownDisposition::Review
    );
    assert_eq!(
        project.policy.on_unknown.flash_budget,
        UnknownDisposition::Block
    );
}

#[test]
fn the_shipped_example_is_a_config_this_loader_accepts() {
    let example = include_str!("../../../../examples/firmwaresight.toml");
    let project = loaded(example);
    assert!(
        project.warnings.is_empty(),
        "the published example must not warn: {:?}",
        project.warnings
    );
    assert!(
        project.policy.require_release_notes,
        "the example documents the release-notes rule it satisfies"
    );
}

#[test]
fn a_minimal_config_gets_the_documented_defaults() {
    let project = loaded(
        "schema_version = 1\n\n[project]\nname = \"p\"\n\n[artifacts]\nrequired = [\"elf\"]\n",
    );
    assert!(project.policy.require_clean_git);
    assert!(project.policy.require_release_notes);
    assert_eq!(project.policy.release_notes_path, "RELEASE_NOTES.md");
    assert_eq!(project.policy.flash_budget, None);
    assert_eq!(project.policy.version, None);
    assert_eq!(project.policy.unknown_evidence_review_count, None);
    // The ADR-0023 defaults: budgets block, everything else asks a human.
    assert_eq!(
        project.policy.on_unknown.git_clean,
        UnknownDisposition::Review
    );
    assert_eq!(
        project.policy.on_unknown.ram_budget,
        UnknownDisposition::Block
    );
}

#[test]
fn an_unsupported_schema_version_is_a_hard_error_not_a_warning() {
    for version in [0, 2, 99] {
        let error = parse(&format!("schema_version = {version}\n")).expect_err("must fail");
        assert!(
            matches!(error, ProjectError::UnsupportedSchemaVersion { found, expected: 1 } if found == version),
            "{error}"
        );
        assert_eq!(error.code(), "ERR-CONFIG-7003");
    }
}

#[test]
fn a_missing_schema_version_is_refused_rather_than_assumed() {
    let error = parse("[project]\nname = \"p\"\n\n[artifacts]\nrequired = [\"elf\"]\n")
        .expect_err("must fail");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { field, .. } if field == "schema_version"),
        "{error}"
    );
}

#[test]
fn malformed_toml_and_wrong_types_are_hard_errors() {
    let malformed = parse("schema_version = 1\n[project\n").expect_err("must fail");
    assert!(
        matches!(&malformed, ProjectError::ConfigMalformed { .. }),
        "{malformed}"
    );
    assert_eq!(malformed.code(), "ERR-CONFIG-7002");

    let wrong_type =
        parse(&VALID_CONFIG.replace("flash_budget = 262144", "flash_budget = \"262144\""))
            .expect_err("a string budget must not be coerced");
    assert!(
        matches!(&wrong_type, ProjectError::ConfigValueInvalid { .. }),
        "{wrong_type}"
    );

    let wrong_enum =
        parse(&VALID_CONFIG.replace("git_clean = \"review\"", "git_clean = \"ignore\""))
            .expect_err("an unknown disposition must not default");
    assert!(
        matches!(&wrong_enum, ProjectError::ConfigValueInvalid { .. }),
        "{wrong_enum}"
    );
}

#[test]
fn unknown_keys_warn_and_are_listed_rather_than_dropped_silently() {
    let project = loaded(&VALID_CONFIG.replace(
        "[sbom]\nenabled = false\n",
        "[sbom]\nenabled = false\n\n[future]\nsigned_bundles = true\n",
    ));
    assert_eq!(
        project.unknown_keys,
        vec!["future"],
        "the key is reported by name so the owner can decide"
    );
    assert!(
        project.warnings.is_empty(),
        "an unknown key is not an error: {:?}",
        project.warnings
    );

    let nested = loaded(&VALID_CONFIG.replace("[memory]\n", "[memory]\ntcm_budget = 4096\n"));
    assert_eq!(nested.unknown_keys, vec!["memory.tcm_budget"]);

    let deep = loaded(&VALID_CONFIG.replace(
        "git_clean = \"review\"",
        "git_clean = \"review\"\nquantum_check = \"block\"",
    ));
    assert_eq!(deep.unknown_keys, vec!["gate.on_unknown.quantum_check"]);
}

#[test]
fn a_config_holding_unknown_keys_refuses_a_destructive_save() {
    let dir = temp_project("refuse-save");
    let path = dir.join(CONFIG_FILE_NAME);
    std::fs::write(
        &path,
        VALID_CONFIG.replace("[memory]\n", "[memory]\ntcm_budget = 4096\n"),
    )
    .expect("the config was written");
    let project = LoadedProject::load(&dir).expect("the config loads with a warning");
    let error = project
        .save_policy(&project.policy)
        .expect_err("saving would drop `memory.tcm_budget`");
    assert!(
        matches!(&error, ProjectError::SaveWouldDropKeys { keys, .. } if keys == "memory.tcm_budget"),
        "{error}"
    );
    assert_eq!(error.code(), "ERR-CONFIG-7007");
    // The file is untouched: still parseable, still carrying the key this build cannot name, and
    // certainly not a zero-byte stub.
    let on_disk = std::fs::read_to_string(&path).expect("the config is still readable");
    assert!(on_disk.contains("tcm_budget"), "{on_disk}");
    assert!(!on_disk.is_empty());
    assert!(
        LoadedProject::load(&dir).is_ok(),
        "a refused save left the config unreadable"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_saved_policy_round_trips_to_the_same_semantics() {
    let dir = temp_project("round-trip");
    let path = dir.join(CONFIG_FILE_NAME);
    std::fs::write(&path, VALID_CONFIG).expect("the config was written");
    let project = LoadedProject::load(&dir).expect("the config loads");

    let mut edited = project.policy.clone();
    edited.flash_budget = Some(131_072);
    edited.required_artifact_kinds = vec!["elf".to_owned()];
    edited.on_unknown.git_clean = UnknownDisposition::Block;
    edited.expected_commit = Some(SHA.to_owned());
    project.save_policy(&edited).expect("the save succeeded");

    let reloaded = LoadedProject::load(&dir).expect("the saved config loads again");
    assert_eq!(reloaded.policy, edited);
    assert_eq!(reloaded.policy_sha256, fingerprint::policy_sha256(&edited));
    assert_eq!(
        reloaded.project_name(),
        "motor-controller",
        "the name survives"
    );
    assert!(reloaded.unknown_keys.is_empty() && reloaded.warnings.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_save_leaves_no_temporary_file_and_never_a_zero_byte_config() {
    let dir = temp_project("atomic");
    let path = dir.join(CONFIG_FILE_NAME);
    std::fs::write(&path, VALID_CONFIG).expect("the config was written");
    let project = LoadedProject::load(&dir).expect("the config loads");
    project
        .save_policy(&project.policy)
        .expect("the save succeeded");
    let leftovers: Vec<String> = std::fs::read_dir(&dir)
        .expect("the directory reads")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temporary files survived: {leftovers:?}"
    );
    assert!(
        std::fs::metadata(&path).expect("the config exists").len() > 40,
        "a saved config is never a stub"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_edit_that_would_produce_an_invalid_config_is_refused_before_any_write() {
    let dir = temp_project("refuse-invalid");
    let path = dir.join(CONFIG_FILE_NAME);
    std::fs::write(&path, VALID_CONFIG).expect("the config was written");
    let before = std::fs::read_to_string(&path).expect("the config reads back");
    let project = LoadedProject::load(&dir).expect("the config loads");

    let mut broken = project.policy.clone();
    broken.required_artifact_kinds = vec!["map".to_owned()];
    let error = project
        .save_policy(&broken)
        .expect_err("a policy without `elf` is not savable");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { .. }),
        "{error}"
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("the config still reads"),
        before,
        "a refused save must not touch the file"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn paths_outside_the_project_are_refused() {
    for path in [
        "/etc/passwd",
        "\\shared\\notes.md",
        "C:\\Windows\\win.ini",
        "docs\\notes.md",
        "../RELEASE_NOTES.md",
        "docs/../../escape.md",
        "",
        ".",
    ] {
        let error = require_project_relative(path, "release.release_notes_path")
            .expect_err("the path must be refused");
        assert!(
            matches!(&error, ProjectError::UnsafeConfigPath { .. }),
            "{path} accepted: {error}"
        );
        assert_eq!(error.code(), "ERR-CONFIG-7005");
    }
    for path in [
        "RELEASE_NOTES.md",
        "docs/RELEASE_NOTES.md",
        "./RELEASE_NOTES.md",
    ] {
        require_project_relative(path, "release.release_notes_path")
            .unwrap_or_else(|error| panic!("{path} should be accepted: {error}"));
    }
}

#[test]
fn an_absolute_or_escaping_notes_path_in_a_config_is_a_hard_error() {
    for replacement in [
        "release_notes_path = \"/etc/passwd\"",
        "release_notes_path = \"../notes.md\"",
        "release_notes_path = \"C:\\\\notes.md\"",
    ] {
        let text = VALID_CONFIG.replace("release_notes_path = \"RELEASE_NOTES.md\"", replacement);
        let error = parse(&text).expect_err("the path must be refused");
        assert!(
            matches!(&error, ProjectError::UnsafeConfigPath { .. }),
            "{error}"
        );
    }
}

#[test]
fn an_uncompilable_pattern_is_reported_with_the_regex_reason() {
    let text = VALID_CONFIG.replace(
        "pattern = '^v(?P<version>\\d+\\.\\d+\\.\\d+)$'",
        "pattern = '^v(?P<version>'",
    );
    let error = parse(&text).expect_err("the pattern must be refused");
    assert!(
        matches!(&error, ProjectError::InvalidVersionPattern { .. }),
        "{error}"
    );
    assert_eq!(error.code(), "ERR-CONFIG-7006");
}

/// Put a line into `[release]` of the fixture config.
fn with_release_line(line: &str) -> String {
    VALID_CONFIG.replace(
        "[release]\nrequire_clean_git = true",
        &format!("[release]\n{line}\nrequire_clean_git = true"),
    )
}

#[test]
fn an_expected_version_requires_a_pattern_that_can_produce_one() {
    let no_capture = VALID_CONFIG.replace(
        "pattern = '^v(?P<version>\\d+\\.\\d+\\.\\d+)$'",
        "pattern = '^v\\d+\\.\\d+\\.\\d+$'",
    );
    let error = parse(&with_release_line("expected_version = \"1.2.3\"").replace(
        "pattern = '^v(?P<version>\\d+\\.\\d+\\.\\d+)$'",
        "pattern = '^v\\d+\\.\\d+\\.\\d+$'",
    ))
    .expect_err("an expected version with no `version` capture cannot be compared");
    assert!(
        matches!(&error, ProjectError::InvalidVersionPattern { .. }),
        "{error}"
    );

    // The same pattern without an expected version is legal, and warns about what it cannot check.
    let project = loaded(&no_capture);
    assert!(
        project
            .warnings
            .iter()
            .any(|warning| warning.contains("no `version` capture")),
        "{:?}",
        project.warnings
    );
}

#[test]
fn an_expected_version_without_a_version_section_is_refused() {
    let text = with_release_line("expected_version = \"1.2.3\"").replace(
        "[version]\nsource = \"git_tag\"\npattern = '^v(?P<version>\\d+\\.\\d+\\.\\d+)$'\n",
        "",
    );
    let error = parse(&text).expect_err("nothing supplies a workspace version to compare");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { field, .. } if field == "release.expected_version"),
        "{error}"
    );
}

#[test]
fn an_empty_expected_version_is_refused() {
    let error = parse(&with_release_line("expected_version = \"  \""))
        .expect_err("a blank version is not a version");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { .. }),
        "{error}"
    );
}

#[test]
fn a_non_git_tag_version_source_is_refused() {
    // Serde rejects the unknown variant before validation sees it, so the message is still typed.
    let text = VALID_CONFIG.replace("source = \"git_tag\"", "source = \"elf_note\"");
    let error = parse(&text).expect_err("MVP has one version source");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { .. }),
        "{error}"
    );
}

#[test]
fn a_zero_budget_is_refused_because_it_would_block_everything() {
    for key in ["flash_budget = 262144", "ram_budget = 131072"] {
        let zeroed = key.replace("262144", "0").replace("131072", "0");
        let text = VALID_CONFIG.replace(key, &zeroed);
        let error = parse(&text).expect_err("a zero budget is not a budget");
        assert!(
            matches!(&error, ProjectError::ConfigValueInvalid { field, .. } if field.contains("budget")),
            "{error}"
        );
    }
}

#[test]
fn the_required_list_must_name_elf_and_only_v1_words() {
    let no_elf = VALID_CONFIG.replace("required = [\"elf\", \"map\"]", "required = [\"map\"]");
    let error = parse(&no_elf).expect_err("an analyze-less release is not a release");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { field, .. } if field == "artifacts.required"),
        "{error}"
    );

    let invented = VALID_CONFIG.replace(
        "required = [\"elf\", \"map\"]",
        "required = [\"elf\", \"dts\"]",
    );
    let error = parse(&invented).expect_err("a word outside the vocabulary cannot be evaluated");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { .. }),
        "{error}"
    );

    let repeated = VALID_CONFIG.replace(
        "required = [\"elf\", \"map\"]",
        "required = [\"elf\", \"elf\"]",
    );
    let project = loaded(&repeated);
    assert!(
        project
            .warnings
            .iter()
            .any(|warning| warning.contains("more than once")),
        "{:?}",
        project.warnings
    );
}

#[test]
fn an_expected_commit_must_be_a_full_object_id() {
    let error = parse(&with_release_line("expected_commit = \"abc123\""))
        .expect_err("a short id would silently never match");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { .. }),
        "{error}"
    );

    assert!(
        parse(&with_release_line(&format!(
            "expected_commit = \"{}\"",
            SHA.to_uppercase()
        )))
        .is_err(),
        "git object ids are lowercase hex"
    );

    let project = loaded(&with_release_line(&format!("expected_commit = \"{SHA}\"")));
    assert_eq!(project.policy.expected_commit.as_deref(), Some(SHA));

    assert!(
        parse(&with_release_line(&format!(
            "expected_commit = \"{}\"",
            "f".repeat(64)
        )))
        .is_ok(),
        "a SHA-256 repository is legal"
    );
}

#[test]
fn an_empty_project_name_is_refused() {
    let text = VALID_CONFIG.replace("name = \"motor-controller\"", "name = \"   \"");
    let error = parse(&text).expect_err("a project has to be named");
    assert!(
        matches!(&error, ProjectError::ConfigValueInvalid { field, .. } if field == "project.name"),
        "{error}"
    );
}

#[test]
fn enabling_sbom_warns_and_evaluates_nothing() {
    let text = VALID_CONFIG.replace("[sbom]\nenabled = false", "[sbom]\nenabled = true");
    let project = loaded(&text);
    assert!(
        project
            .warnings
            .iter()
            .any(|warning| warning.contains("SBOM is not an MVP Gate capability")),
        "{:?}",
        project.warnings
    );
    assert_eq!(
        project.policy,
        loaded(VALID_CONFIG).policy,
        "the switch must not add a rule or a PASS"
    );
}

#[test]
fn the_policy_hash_ignores_comments_blank_lines_and_key_order() {
    let commented = format!(
        "# release policy\n\n{}\n\n# trailing note\n",
        VALID_CONFIG.replace("[memory]\nflash_budget", "[memory]\n\nflash_budget")
    );
    let reordered = VALID_CONFIG.replace(
        "[release]\nrequire_clean_git = true\nrequire_release_notes = true\nrelease_notes_path = \"RELEASE_NOTES.md\"\n",
        "[release]\nrelease_notes_path = \"RELEASE_NOTES.md\"\nrequire_release_notes = true\nrequire_clean_git = true\n",
    );
    let base = loaded(VALID_CONFIG);
    let with_comments = loaded(&commented);
    let reordered = loaded(&reordered);
    assert_eq!(
        base.policy_sha256, with_comments.policy_sha256,
        "formatting is not policy"
    );
    assert_eq!(
        base.policy_sha256, reordered.policy_sha256,
        "key order is not policy"
    );
    assert_eq!(base.policy, reordered.policy);
}

#[test]
fn every_semantic_field_changes_the_policy_hash() {
    let base = loaded(VALID_CONFIG);
    let changes = [
        (
            "flash budget",
            VALID_CONFIG.replace("flash_budget = 262144", "flash_budget = 262145"),
        ),
        (
            "required artifacts",
            VALID_CONFIG.replace("required = [\"elf\", \"map\"]", "required = [\"elf\"]"),
        ),
        (
            "clean git",
            VALID_CONFIG.replace("require_clean_git = true", "require_clean_git = false"),
        ),
        (
            "unknown disposition",
            VALID_CONFIG.replace("git_clean = \"review\"", "git_clean = \"block\""),
        ),
        (
            "notes path",
            VALID_CONFIG.replace("RELEASE_NOTES.md", "docs/NOTES.md"),
        ),
        (
            "version pattern",
            VALID_CONFIG.replace("\\d+\\.\\d+\\.\\d+", "\\d+\\.\\d+"),
        ),
        (
            "review threshold",
            VALID_CONFIG.replace(
                "flash_growth_review_bytes = 4096",
                "flash_growth_review_bytes = 8192",
            ),
        ),
    ];
    for (label, text) in changes {
        let other = loaded(&text);
        assert_ne!(
            base.policy_sha256, other.policy_sha256,
            "changing {label} must change the policy fingerprint"
        );
        assert_ne!(
            base.policy, other.policy,
            "changing {label} must change the policy"
        );
    }
}

#[test]
fn a_warning_never_becomes_an_error_and_an_error_never_becomes_a_warning() {
    // Unknown key: warning, config loads.
    let warned = loaded(&VALID_CONFIG.replace("[memory]\n", "[memory]\nextra = 1\n"));
    assert_eq!(warned.unknown_keys, vec!["memory.extra"]);
    // Bad budget: error, config does not load.
    assert!(parse(&VALID_CONFIG.replace("ram_budget = 131072", "ram_budget = -1")).is_err());
}

#[test]
fn a_missing_config_file_is_reported_with_its_own_code() {
    let error = LoadedProject::load(Path::new("/does/not/exist")).expect_err("no config there");
    assert!(
        matches!(&error, ProjectError::ConfigUnreadable { .. }),
        "{error}"
    );
    assert_eq!(error.code(), "ERR-CONFIG-7001");
}

#[test]
fn writing_a_fresh_config_from_a_policy_loads_back_identically() {
    let policy = GatePolicy {
        flash_budget: Some(65_536),
        required_artifact_kinds: vec!["elf".to_owned(), "map".to_owned()],
        unknown_evidence_review_count: Some(3),
        ..GatePolicy::default()
    };
    let dir = temp_project("create");
    let path = dir.join(CONFIG_FILE_NAME);
    write_config(&path, &new_config("created-project", &policy)).expect("the config was written");
    let project = LoadedProject::load(&dir).expect("the created config loads");
    assert_eq!(project.policy, policy);
    assert_eq!(project.project_name(), "created-project");
    assert!(project.warnings.is_empty() && project.unknown_keys.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_saved_config_never_leaks_an_absolute_path_into_its_own_text() {
    let dir = temp_project("no-root");
    let path = dir.join(CONFIG_FILE_NAME);
    std::fs::write(&path, VALID_CONFIG).expect("the config was written");
    let project = LoadedProject::load(&dir).expect("the config loads");
    project
        .save_policy(&project.policy)
        .expect("the save succeeded");
    let text = std::fs::read_to_string(&path).expect("the config reads back");
    let root = dir.to_string_lossy().into_owned();
    assert!(
        !text.contains(&root),
        "the project root was written into the config"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_vocabulary_is_the_one_the_gate_rule_reads() {
    assert_eq!(REQUIRED_ARTIFACT_VOCABULARY, ["elf", "map", "bin", "hex"]);
    assert_eq!(SUPPORTED_SCHEMA_VERSION, 1);
    assert_eq!(CONFIG_FILE_NAME, "firmwaresight.toml");
}
