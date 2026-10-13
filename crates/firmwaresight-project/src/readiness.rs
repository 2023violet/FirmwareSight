//! Bounded, read-only project observations shared by the CLI and desktop.

use std::path::Path;

use firmwaresight_core::domain::gate::GateFileStatus;
use firmwaresight_core::domain::identity::Fact;

use crate::config::LoadedProject;
use crate::error::ProjectError;
use crate::evidence::observe_release_notes;
use crate::git::GitProbe;

pub const MAX_READINESS_ROWS: usize = 32;
pub const MAX_READINESS_TEXT_CHARS: usize = 160;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectReadiness {
    pub scope: &'static str,
    pub project_name: String,
    pub schema_version: i64,
    pub policy_sha256: String,
    pub warnings: Vec<String>,
    pub unknown_key_count: usize,
    pub git: ReadinessGit,
    pub release_notes: Option<ReadinessNotes>,
    pub required_artifacts: Vec<DeclaredArtifact>,
    pub build_evidence_checked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessGit {
    pub available: bool,
    pub head_commit: Option<String>,
    pub clean: Option<bool>,
    pub unknown_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadinessNotes {
    Present { sha256: Option<String> },
    Missing,
    Unreadable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredArtifact {
    pub kind: String,
    pub satisfaction_checked: bool,
}

/// Load a project's config and observe its workspace facts without changing project state.
pub fn inspect_project(root: &Path) -> Result<ProjectReadiness, ProjectError> {
    inspect_project_with_probe(root, &GitProbe::system())
}

pub fn inspect_project_with_probe(
    root: &Path,
    git_probe: &GitProbe,
) -> Result<ProjectReadiness, ProjectError> {
    let loaded = LoadedProject::load(root)?;
    let git = git_probe.observe(&loaded.root);
    let unknown_reason = git.facts.head_commit.reason_if_unknown().map(safe_text);
    let notes = loaded
        .config
        .release
        .release_notes_path
        .as_deref()
        .map(|relative| observe_release_notes(&loaded.root, relative))
        .map(|fact| match fact.status {
            GateFileStatus::Present { sha256 } => ReadinessNotes::Present { sha256 },
            GateFileStatus::Missing => ReadinessNotes::Missing,
            GateFileStatus::Unreadable { reason } => ReadinessNotes::Unreadable {
                reason: safe_text(reason),
            },
        });
    let mut warnings: Vec<String> = loaded.warnings.iter().map(safe_text).collect();
    if !loaded.unknown_keys.is_empty() {
        warnings.push("Unsupported configuration keys are present.".to_owned());
    }
    warnings.sort();
    warnings.truncate(MAX_READINESS_ROWS);

    Ok(ProjectReadiness {
        scope: "Configuration and workspace observation only; this is not a Gate result.",
        project_name: safe_project_name(loaded.project_name()),
        schema_version: loaded.config.schema_version,
        policy_sha256: loaded.policy_sha256,
        warnings,
        unknown_key_count: loaded.unknown_keys.len(),
        git: ReadinessGit {
            available: git.facts.available,
            head_commit: match git.facts.head_commit {
                Fact::Known(value) => Some(value),
                Fact::Unknown { .. } => None,
            },
            clean: match git.facts.dirty {
                Fact::Known(dirty) => Some(!dirty),
                Fact::Unknown { .. } => None,
            },
            unknown_reason,
        },
        release_notes: notes,
        required_artifacts: loaded
            .config
            .artifacts
            .required
            .iter()
            .take(MAX_READINESS_ROWS)
            .map(|kind| DeclaredArtifact {
                kind: safe_text(kind),
                satisfaction_checked: false,
            })
            .collect(),
        build_evidence_checked: false,
    })
}

fn safe_text(value: impl AsRef<str>) -> String {
    let mut safe = String::new();
    for character in value.as_ref().chars().take(MAX_READINESS_TEXT_CHARS) {
        if character.is_control() {
            safe.push(' ');
        } else {
            safe.push(character);
        }
    }
    safe.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn safe_project_name(value: &str) -> String {
    let text = safe_text(value);
    let bytes = text.as_bytes();
    let windows_absolute = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/');
    if Path::new(&text).is_absolute() || windows_absolute || text.starts_with("\\\\") {
        "Unnamed project".to_owned()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::fixtures::{self, VALID_CONFIG};
    use crate::git::GitProbe;

    use super::{ReadinessNotes, inspect_project_with_probe};

    fn write_config(root: &Path, text: &str) {
        std::fs::write(root.join("firmwaresight.toml"), text).expect("write fixture config");
    }

    fn tree_digest(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn walk(root: &Path, base: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
            for entry in std::fs::read_dir(root).expect("read fixture directory") {
                let entry = entry.expect("read fixture entry");
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, base, files);
                } else {
                    files.push((
                        path.strip_prefix(base)
                            .expect("relative fixture path")
                            .to_path_buf(),
                        std::fs::read(path).expect("read fixture file"),
                    ));
                }
            }
        }
        let mut files = Vec::new();
        walk(root, root, &mut files);
        files.sort_by(|left, right| left.0.cmp(&right.0));
        files
    }

    #[test]
    fn reads_config_git_and_release_notes_without_claiming_build_evidence_or_writing() {
        let root = fixtures::temp_project("readiness-facts");
        write_config(&root, VALID_CONFIG);
        std::fs::write(
            root.join("RELEASE_NOTES.md"),
            "Notes for this test build.\n",
        )
        .expect("write notes");
        let before = tree_digest(&root);

        let first =
            inspect_project_with_probe(&root, &GitProbe::with_program("missing-git-next40"))
                .expect("valid project is inspectable");
        let second =
            inspect_project_with_probe(&root, &GitProbe::with_program("missing-git-next40"))
                .expect("repeated inspection succeeds");

        assert_eq!(first, second);
        assert_eq!(first.project_name, "motor-controller");
        assert_eq!(
            first.scope,
            "Configuration and workspace observation only; this is not a Gate result."
        );
        assert_eq!(first.schema_version, 1);
        assert_eq!(
            first
                .required_artifacts
                .iter()
                .map(|row| row.kind.as_str())
                .collect::<Vec<_>>(),
            ["elf", "map"]
        );
        assert!(
            first
                .required_artifacts
                .iter()
                .all(|row| !row.satisfaction_checked)
        );
        assert!(!first.build_evidence_checked);
        assert!(matches!(
            first.release_notes,
            Some(ReadinessNotes::Present { sha256: Some(_) })
        ));
        assert!(!first.git.available);
        assert_eq!(first.git.clean, None);
        assert!(first.git.head_commit.is_none());
        assert_eq!(before, tree_digest(&root));
        std::fs::remove_dir_all(root).expect("remove disposable fixture");
    }

    #[test]
    fn unknown_keys_and_project_labels_are_bounded_and_do_not_echo_secret_values() {
        let root = fixtures::temp_project("readiness-redaction");
        let text = VALID_CONFIG.replace("motor-controller", "motor\\r\\n$(injected)")
            + "\n[private]\napi_token = \"sentinel-secret-value\"\n";
        write_config(&root, &text);

        let report =
            inspect_project_with_probe(&root, &GitProbe::with_program("missing-git-next40"))
                .expect("unknown keys remain warnings");
        let debug = format!("{report:?}");
        assert_eq!(report.unknown_key_count, 1);
        assert!(!debug.contains("sentinel-secret-value"));
        assert!(!report.project_name.chars().any(char::is_control));
        assert!(
            report.project_name.chars().count()
                <= super::super::readiness::MAX_READINESS_TEXT_CHARS
        );
        assert!(report.warnings.iter().all(|warning| warning.len() <= 1024));
        std::fs::remove_dir_all(root).expect("remove disposable fixture");
    }

    #[test]
    fn absolute_project_name_is_not_projected() {
        let root = fixtures::temp_project("readiness-project-name-path");
        write_config(
            &root,
            &VALID_CONFIG.replace("motor-controller", "C:\\\\Users\\\\private\\\\firmware"),
        );
        let report =
            inspect_project_with_probe(&root, &GitProbe::with_program("missing-git-next40"))
                .expect("valid config is inspectable");
        assert_eq!(report.project_name, "Unnamed project");
        assert!(!format!("{report:?}").contains("private"));
        std::fs::remove_dir_all(root).expect("remove disposable fixture");
    }

    #[test]
    fn malformed_config_keeps_existing_typed_error_behavior() {
        let root = fixtures::temp_project("readiness-invalid");
        write_config(&root, "schema_version = 2\n");
        assert!(
            inspect_project_with_probe(&root, &GitProbe::with_program("missing-git-next40"))
                .is_err()
        );
        std::fs::remove_dir_all(root).expect("remove disposable fixture");
    }

    #[test]
    fn missing_and_outside_release_notes_are_not_reported_as_present() {
        let root = fixtures::temp_project("readiness-notes-boundary");
        let outside = fixtures::temp_project("readiness-notes-outside");
        std::fs::write(outside.join("RELEASE_NOTES.md"), "outside-canary-secret\n")
            .expect("write outside canary");

        write_config(&root, VALID_CONFIG);
        let missing =
            inspect_project_with_probe(&root, &GitProbe::with_program("missing-git-next40"))
                .expect("valid config loads");
        assert_eq!(missing.release_notes, Some(ReadinessNotes::Missing));

        let outside_config = VALID_CONFIG.replace(
            "RELEASE_NOTES.md",
            &outside.join("RELEASE_NOTES.md").to_string_lossy(),
        );
        write_config(&root, &outside_config);
        let rejected =
            inspect_project_with_probe(&root, &GitProbe::with_program("missing-git-next40"));
        assert!(
            rejected.is_err(),
            "absolute notes paths remain config refusals"
        );
        assert!(!format!("{rejected:?}").contains("outside-canary-secret"));

        std::fs::remove_dir_all(root).expect("remove project fixture");
        std::fs::remove_dir_all(outside).expect("remove outside fixture");
    }

    #[test]
    fn git_clean_and_dirty_are_distinct_observations() {
        let root = fixtures::temp_project("readiness-git-state");
        write_config(&root, VALID_CONFIG);
        let git = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .status()
                .expect("git is available for this test");
            assert!(status.success(), "git {args:?} failed");
        };
        git(&["init", "--quiet"]);
        git(&["config", "user.name", "FirmwareSight Test"]);
        git(&["config", "user.email", "test@example.invalid"]);
        git(&["add", "firmwaresight.toml"]);
        git(&["commit", "--quiet", "-m", "fixture"]);

        let clean =
            inspect_project_with_probe(&root, &GitProbe::system()).expect("inspect clean repo");
        assert!(clean.git.available);
        assert_eq!(clean.git.clean, Some(true));
        assert!(
            clean
                .git
                .head_commit
                .as_deref()
                .is_some_and(|head| head.len() >= 40)
        );

        std::fs::write(root.join("untracked.txt"), "changed\n").expect("make repo dirty");
        let dirty =
            inspect_project_with_probe(&root, &GitProbe::system()).expect("inspect dirty repo");
        assert_eq!(dirty.git.clean, Some(false));
        std::fs::remove_dir_all(root).expect("remove disposable fixture");
    }
}
