//! Test-only fixtures shared across this crate's modules.
//!
//! Kept in one place so two modules' tests cannot disagree about what "a clean workspace with a tagged
//! HEAD" means.

use firmwaresight_core::domain::gate::{
    GateArtifactFact, GateContext, GateGitFacts, GatePolicy, GateUnknownEvidence,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};

/// The commit id a fixture workspace is "checked out" at.
pub const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

/// A Gate context over a clean, tagged workspace with one ELF and nothing configured.
pub fn gate_context() -> GateContext {
    GateContext {
        snapshot_id: "snap-1".to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known("b".repeat(64)),
            byte_size: 1_024,
        }],
        memory: None,
        git: GateGitFacts {
            available: true,
            head_commit: Fact::known(HEAD.to_owned()),
            exact_tag: Fact::known("v1.2.3".to_owned()),
            dirty: Fact::known(false),
        },
        version: None,
        release_notes: None,
        growth: firmwaresight_core::domain::gate::GateGrowthFacts::without_baseline(),
        unknown_evidence: GateUnknownEvidence::default(),
        policy: GatePolicy {
            require_clean_git: false,
            expected_commit: None,
            version: None,
            required_artifact_kinds: vec!["elf".to_owned()],
            flash_budget: None,
            ram_budget: None,
            flash_growth_review_bytes: None,
            ram_growth_review_bytes: None,
            require_release_notes: false,
            release_notes_path: "RELEASE_NOTES.md".to_owned(),
            unknown_evidence_review_count: None,
            on_unknown: firmwaresight_core::domain::gate::UnknownPolicy::default(),
        },
    }
}

/// The smallest config that satisfies every frozen v1 rule.
pub const VALID_CONFIG: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map"]

[memory]
flash_budget = 262144
ram_budget = 131072

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"

[diff]
flash_growth_review_bytes = 4096
ram_growth_review_bytes = 2048

[gate]
unknown_evidence_review_count = 1

[gate.on_unknown]
git_clean = "review"
commit_matches_release = "review"
version_match = "review"
flash_budget = "block"
ram_budget = "block"
baseline_growth = "review"
release_notes = "review"

[sbom]
enabled = false
"#;

/// A unique directory under the system temp directory. The caller removes it; `Drop`-style cleanup is
/// deliberately not hidden here, so a failing test leaves the tree for inspection.
pub fn temp_project(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "firmwaresight-project-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap_or_else(|error| {
        panic!("could not create the temporary project for {label}: {error}")
    });
    dir
}
