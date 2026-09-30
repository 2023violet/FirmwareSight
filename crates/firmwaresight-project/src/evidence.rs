//! Project-local observations, turned into the facts `firmwaresight-core` evaluates.
//!
//! This is the assembly point the CLI and the desktop share (ADR-0027): one place that reads a
//! release-notes file, counts a snapshot's `Unknown` evidence items, projects a memory footprint into
//! budget facts, and puts the result together as a [`GateContext`]. Neither surface may reimplement any
//! of it, and none of it decides what a state *means* — that is Core's job.
//!
//! Nothing here returns a host path. The project root is an input to the read, never part of the
//! observation (`AGENTS.md` 7).

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_core::domain::diff::{BudgetState, DiffResult};
use firmwaresight_core::domain::evidence::{EvidenceClass, EvidenceItem};
use firmwaresight_core::domain::gate::{
    GateArtifactFact, GateBudgetFact, GateContext, GateFileFact, GateFileStatus, GateGrowthFacts,
    GateMemoryFacts, GatePolicy, GateUnknownEvidence, UnknownPolicy,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};

use crate::config;
use crate::fingerprint;
use crate::git::GitObservation;
use crate::version;

/// How many `Unknown` evidence locators a single finding may quote. The count is complete; the sample
/// is bounded so one finding cannot carry a thousand refs (prompt §25).
pub const UNKNOWN_EVIDENCE_SAMPLE: usize = 8;

/// What the Gate needs from one build, whether it came from a fresh analysis or from storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFacts {
    pub snapshot_id: String,
    pub artifacts: Vec<GateArtifactFact>,
    pub memory: Option<GateMemoryFacts>,
    pub unknown_evidence: GateUnknownEvidence,
}

impl SnapshotFacts {
    /// Project a freshly analyzed snapshot. This is the CLI path.
    #[must_use]
    pub fn from_snapshot(snapshot: &BuildSnapshot) -> Self {
        Self {
            snapshot_id: snapshot.id().as_str().to_owned(),
            artifacts: artifact_facts(snapshot),
            memory: memory_facts(snapshot),
            unknown_evidence: unknown_evidence(snapshot, UNKNOWN_EVIDENCE_SAMPLE),
        }
    }

    /// The shape of a snapshot the caller hydrated from storage, where a digest may be absent.
    #[must_use]
    pub fn from_stored(
        snapshot_id: String,
        artifacts: Vec<GateArtifactFact>,
        memory: Option<GateMemoryFacts>,
        unknown: GateUnknownEvidence,
    ) -> Self {
        Self {
            snapshot_id,
            artifacts,
            memory,
            unknown_evidence: unknown,
        }
    }
}

/// Every artifact a snapshot holds, as the Gate reads it: kind, digest, size. No path.
#[must_use]
pub fn artifact_facts(snapshot: &BuildSnapshot) -> Vec<GateArtifactFact> {
    snapshot
        .artifacts()
        .iter()
        .map(|artifact| GateArtifactFact {
            kind: artifact.kind,
            sha256: Fact::known(artifact.sha256.hex().to_owned()),
            byte_size: artifact.byte_size,
        })
        .collect()
}

/// The artifact kinds present, in the v1 requirement vocabulary. Kept here so a UI can show "Required ·
/// Present" without re-deriving the mapping.
#[must_use]
pub fn present_kinds(snapshot: &BuildSnapshot) -> Vec<&'static str> {
    let mut kinds: Vec<&'static str> = snapshot
        .artifacts()
        .iter()
        .map(|artifact| kind_word(artifact.kind))
        .collect();
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}

const fn kind_word(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Elf => "elf",
        ArtifactKind::Map => "map",
        ArtifactKind::Bin => "bin",
        ArtifactKind::IntelHex => "hex",
        ArtifactKind::Unknown => "unknown",
    }
}

/// One stored or computed budget figure, as the Gate reads it.
///
/// `state` is the persisted budget state; `admissible` is whether the evidence basis may support a hard
/// verdict. The mapping from those two to "may this block?" is Core's rule, not a decision made here —
/// this function only carries the facts across the boundary without losing either of them.
#[must_use]
pub fn budget_fact(
    state: BudgetState,
    bytes: Option<u64>,
    admissible: bool,
    basis: Option<String>,
    reason: Option<String>,
    evidence_ref: Option<String>,
) -> GateBudgetFact {
    GateBudgetFact {
        bytes,
        exact: matches!(state, BudgetState::Exact),
        admissible,
        basis,
        reason,
        evidence_ref,
    }
}

/// The evidence field each footprint total is recorded under, in side order.
///
/// A caller hydrating from SQLite needs the same names to look the items up with, because a budget's
/// evidence locator is part of what a Gate run is judged from — and part of the fingerprint that
/// identifies it. The artifact pipeline writes these fields beside the footprint row, so the two
/// surfaces quote one pointer rather than two shapes of the same fact.
pub const FOOTPRINT_EVIDENCE_FIELDS: [&str; 2] = [
    "nonvolatile_image_footprint_bytes",
    "runtime_ram_footprint_bytes",
];

/// The evidence id the pipeline gives one footprint field.
#[must_use]
pub fn footprint_evidence_id(field: &str) -> String {
    format!("ev-memory-{field}")
}

/// The two footprint totals, with the evidence strength each one actually has.
#[must_use]
pub fn memory_facts(snapshot: &BuildSnapshot) -> Option<GateMemoryFacts> {
    let footprint = snapshot.memory()?;
    let admissible = footprint.admissible_for_hard_block();
    let basis = footprint.weakest_basis.map(|value| format!("{value:?}"));
    Some(GateMemoryFacts {
        nonvolatile: Some(total_fact(
            &footprint.nonvolatile,
            admissible,
            basis.clone(),
            &footprint_evidence_id(FOOTPRINT_EVIDENCE_FIELDS[0]),
            snapshot,
        )),
        runtime_ram: Some(total_fact(
            &footprint.runtime_ram,
            admissible,
            basis,
            &footprint_evidence_id(FOOTPRINT_EVIDENCE_FIELDS[1]),
            snapshot,
        )),
    })
}

fn total_fact(
    total: &firmwaresight_core::domain::memory::ByteTotal,
    admissible: bool,
    basis: Option<String>,
    evidence_id: &str,
    snapshot: &BuildSnapshot,
) -> GateBudgetFact {
    let reason = match total {
        firmwaresight_core::domain::memory::ByteTotal::Exact { .. } => None,
        firmwaresight_core::domain::memory::ByteTotal::Partial { reason, .. } => {
            Some(reason.clone())
        }
        firmwaresight_core::domain::memory::ByteTotal::Unknown { reason } => Some(reason.clone()),
    };
    let evidence_ref = snapshot
        .evidence()
        .iter()
        .find(|item| item.id == evidence_id)
        .map(|item| format!("evidence:{}", item.id));
    budget_fact(
        match total {
            firmwaresight_core::domain::memory::ByteTotal::Exact { .. } => BudgetState::Exact,
            firmwaresight_core::domain::memory::ByteTotal::Partial { .. } => BudgetState::Partial,
            firmwaresight_core::domain::memory::ByteTotal::Unknown { .. } => BudgetState::Unknown,
        },
        total.bytes(),
        admissible,
        basis,
        reason,
        evidence_ref,
    )
}

/// How many evidence items a snapshot classifies `Unknown`, with a bounded sample of their locators.
#[must_use]
pub fn unknown_evidence(snapshot: &BuildSnapshot, limit: usize) -> GateUnknownEvidence {
    unknown_evidence_from(snapshot.evidence(), limit)
}

/// The same count over any evidence list, so a caller hydrating from storage can pass its own rows.
#[must_use]
pub fn unknown_evidence_from(items: &[EvidenceItem], limit: usize) -> GateUnknownEvidence {
    let ids: Vec<String> = items
        .iter()
        .filter(|item| item.classification == EvidenceClass::Unknown)
        .map(|item| item.id.clone())
        .collect();
    unknown_evidence_from_ids(&ids, ids.len(), limit)
}

/// The count and the quoted locators from ids alone, which is the shape a storage read returns.
///
/// `total` is separate from `ids` because the read is bounded: a build with nine hundred gaps hands
/// back eight ids and the number nine hundred, and both have to survive into the finding. The
/// `evidence:` locator scheme is written here and nowhere else, so a desktop run and a CLI run quote
/// the same gap the same way.
#[must_use]
pub fn unknown_evidence_from_ids(
    ids: &[String],
    total: usize,
    limit: usize,
) -> GateUnknownEvidence {
    let mut sorted: Vec<&str> = ids.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    let mut sample_refs: Vec<String> = sorted
        .iter()
        .take(limit)
        .map(|id| format!("evidence:{id}"))
        .collect();
    if total > sample_refs.len() {
        // The sample is bounded; the total stays stated, so a reader knows how much was not shown.
        sample_refs.push(format!("evidence:unknown-count={total}"));
    }
    GateUnknownEvidence {
        count: total,
        sample_refs,
    }
}

/// Project P2's diff result into the growth facts the Gate thresholds.
///
/// The deltas are moved, not recomputed: `firmwaresight-core::domain::diff` remains the only place a
/// difference is calculated (prompt §23).
#[must_use]
pub fn growth_facts(diff: &DiffResult) -> GateGrowthFacts {
    GateGrowthFacts {
        baseline_snapshot_id: Some(diff.base_snapshot_id.clone()),
        nonvolatile: Some(diff.memory.nonvolatile.clone()),
        runtime_ram: Some(diff.memory.runtime_ram.clone()),
    }
}

/// Observe the configured Release Notes file inside a project root.
///
/// A path the config declares is validated again here, so no caller can reach outside the root by
/// skipping `config::validate`. Absence and an unreadable file are different facts: only the first is
/// a policy failure.
#[must_use]
pub fn observe_release_notes(root: &Path, relative_path: &str) -> GateFileFact {
    let missing = |reason: &str| GateFileFact {
        relative_path: relative_path.to_owned(),
        status: GateFileStatus::Unreadable {
            reason: reason.to_owned(),
        },
    };
    if config::require_project_relative(relative_path, "release.release_notes_path").is_err() {
        return missing("the configured path is not project-relative, so it was not read");
    }
    let path = join_project_relative(root, relative_path);
    match std::fs::metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => GateFileFact {
            relative_path: relative_path.to_owned(),
            status: GateFileStatus::Missing,
        },
        Err(error) => missing(&io_reason(&error)),
        Ok(metadata) => {
            if !metadata.is_file() {
                return missing("the configured path is not a regular file");
            }
            match fingerprint::file_sha256(&path) {
                Ok(digest) => GateFileFact {
                    relative_path: relative_path.to_owned(),
                    status: GateFileStatus::Present {
                        sha256: Some(digest),
                    },
                },
                Err(error) => missing(&io_reason(&error)),
            }
        }
    }
}

/// Join a validated relative path onto a root, component by component. `..` cannot escape because the
/// path was already refused by `config::require_project_relative`; the loop makes that a property of
/// this function too rather than only of its caller.
#[must_use]
pub fn join_project_relative(root: &Path, relative_path: &str) -> PathBuf {
    let mut joined = root.to_path_buf();
    for component in Path::new(relative_path).components() {
        match component {
            std::path::Component::Normal(name) => joined.push(name),
            std::path::Component::CurDir => {}
            other => {
                // Unreachable for a validated path; if it ever is reached, resolve nothing rather than
                // resolve outside the root.
                let _ = other;
                return root.join("firmwaresight-unresolvable-path");
            }
        }
    }
    joined
}

/// A short, path-free description of an I/O failure. `std::io::Error`'s `Display` can name the file, so
/// only the kind is carried into a fact, a run record or a UI.
fn io_reason(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => "permission denied".to_owned(),
        std::io::ErrorKind::NotFound => "the file disappeared during the read".to_owned(),
        kind => format!("a filesystem error occurred ({kind:?})"),
    }
}

/// Everything needed to assemble one Gate run, in the shape both surfaces can produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateRunRequest<'a> {
    pub target: &'a SnapshotFacts,
    pub growth: GateGrowthFacts,
    pub git: &'a GitObservation,
    pub policy: firmwaresight_core::domain::gate::GatePolicy,
    /// `None` when the policy does not require notes: the rule reports `N/A` and no read happened.
    pub release_notes: Option<GateFileFact>,
}

impl<'a> GateRunRequest<'a> {
    /// A policy default, so a caller that has no config yet still assembles a legal context.
    #[must_use]
    pub fn with_default_policy(target: &'a SnapshotFacts, git: &'a GitObservation) -> Self {
        Self {
            target,
            growth: GateGrowthFacts::without_baseline(),
            git,
            policy: GatePolicy::default(),
            release_notes: None,
        }
    }
}

/// Assemble the context the Gate evaluates: facts in, nothing interpreted.
#[must_use]
pub fn build_context(request: &GateRunRequest) -> GateContext {
    let version = request.policy.version.as_ref().map(|policy| {
        version::evaluate_tag(
            &policy.pattern,
            request.git.facts.exact_tag.value().map(String::as_str),
        )
    });
    GateContext {
        snapshot_id: request.target.snapshot_id.clone(),
        artifacts: request.target.artifacts.clone(),
        memory: request.target.memory.clone(),
        git: request.git.facts.clone(),
        version,
        release_notes: request.release_notes.clone(),
        growth: request.growth.clone(),
        unknown_evidence: request.target.unknown_evidence.clone(),
        policy: request.policy.clone(),
    }
}

/// The disposition table a config omits entirely, exposed so the desktop's "no config yet" screen can
/// show what would be applied.
#[must_use]
pub const fn default_unknown_policy() -> UnknownPolicy {
    UnknownPolicy {
        git_clean: firmwaresight_core::domain::gate::UnknownDisposition::Review,
        commit_matches_release: firmwaresight_core::domain::gate::UnknownDisposition::Review,
        version_match: firmwaresight_core::domain::gate::UnknownDisposition::Review,
        flash_budget: firmwaresight_core::domain::gate::UnknownDisposition::Block,
        ram_budget: firmwaresight_core::domain::gate::UnknownDisposition::Block,
        baseline_growth: firmwaresight_core::domain::gate::UnknownDisposition::Review,
        release_notes: firmwaresight_core::domain::gate::UnknownDisposition::Review,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use firmwaresight_core::domain::evidence::SourceType;

    fn item(id: &str, class: EvidenceClass) -> EvidenceItem {
        EvidenceItem::new(
            id,
            class,
            SourceType::ElfSectionHeader,
            "elf.section_header[1]",
            "size",
            "0",
            "test-rule",
        )
    }

    #[test]
    fn unknown_items_are_counted_and_the_sample_is_bounded() {
        let items: Vec<EvidenceItem> = (0..12)
            .map(|index| item(&format!("ev-{index:02}"), EvidenceClass::Unknown))
            .chain(std::iter::once(item(
                "ev-observed",
                EvidenceClass::Observed,
            )))
            .collect();
        let facts = unknown_evidence_from(&items, 4);
        assert_eq!(facts.count, 12, "the observed item is not counted");
        assert_eq!(facts.sample_refs.len(), 5, "four locators plus one summary");
        assert_eq!(
            facts.sample_refs[0], "evidence:ev-00",
            "sorted, so it is stable"
        );
        assert_eq!(
            *facts.sample_refs.last().expect("the sample is non-empty"),
            "evidence:unknown-count=12"
        );
    }

    #[test]
    fn a_sample_below_the_limit_adds_no_summary_line() {
        let items = vec![item("ev-a", EvidenceClass::Unknown)];
        let facts = unknown_evidence_from(&items, 8);
        assert_eq!(facts.sample_refs, vec!["evidence:ev-a".to_owned()]);
    }

    #[test]
    fn the_sample_is_the_same_whatever_order_the_items_arrive_in() {
        let forward = vec![
            item("ev-a", EvidenceClass::Unknown),
            item("ev-b", EvidenceClass::Unknown),
        ];
        let backward = vec![
            item("ev-b", EvidenceClass::Unknown),
            item("ev-a", EvidenceClass::Unknown),
        ];
        assert_eq!(
            unknown_evidence_from(&forward, 8),
            unknown_evidence_from(&backward, 8),
            "a run fingerprint must not depend on a query's row order"
        );
    }

    #[test]
    fn a_present_file_yields_its_digest_and_no_path() {
        let root = std::env::temp_dir().join(format!("firmwaresight-notes-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("the temporary project directory was created");
        std::fs::write(root.join("RELEASE_NOTES.md"), b"# 1.2.3\n").expect("notes were written");
        let fact = observe_release_notes(&root, "RELEASE_NOTES.md");
        assert_eq!(fact.relative_path, "RELEASE_NOTES.md");
        match &fact.status {
            GateFileStatus::Present { sha256 } => {
                assert_eq!(
                    sha256.as_deref(),
                    Some(fingerprint::sha256_hex(b"# 1.2.3\n").as_str())
                );
            }
            other => panic!("expected a present file, got {other:?}"),
        }
        let text = format!("{fact:?}");
        assert!(
            !text.contains(&root.to_string_lossy().to_string()),
            "the observation carried the project root: {text}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_subdirectory_relative_path_is_read_and_named_relative() {
        let root =
            std::env::temp_dir().join(format!("firmwaresight-notes-sub-{}", std::process::id()));
        std::fs::create_dir_all(root.join("docs")).expect("the docs directory was created");
        std::fs::write(root.join("docs/NOTES.md"), b"notes").expect("notes were written");
        let fact = observe_release_notes(&root, "docs/NOTES.md");
        assert!(matches!(fact.status, GateFileStatus::Present { .. }));
        assert_eq!(fact.relative_path, "docs/NOTES.md");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_absent_file_is_missing_rather_than_unreadable() {
        let root =
            std::env::temp_dir().join(format!("firmwaresight-notes-absent-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("the temporary project directory was created");
        let fact = observe_release_notes(&root, "RELEASE_NOTES.md");
        assert_eq!(fact.status, GateFileStatus::Missing);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_unsafe_configured_path_is_never_read() {
        for path in [
            "/etc/passwd",
            "C:\\Windows\\win.ini",
            "../outside.md",
            "docs/../../outside.md",
            "",
        ] {
            let fact = observe_release_notes(Path::new("."), path);
            assert!(
                matches!(fact.status, GateFileStatus::Unreadable { .. }),
                "{path} was not refused before reading"
            );
            assert!(
                !fact.relative_path.is_empty() || fact.relative_path.is_empty(),
                "the fact still names what was configured: {path}"
            );
        }
    }

    #[test]
    fn a_directory_named_as_the_notes_file_is_an_evidence_failure_not_an_absence() {
        let root =
            std::env::temp_dir().join(format!("firmwaresight-notes-dir-{}", std::process::id()));
        std::fs::create_dir_all(root.join("RELEASE_NOTES.md")).expect("the directory was created");
        let fact = observe_release_notes(&root, "RELEASE_NOTES.md");
        assert!(
            matches!(&fact.status, GateFileStatus::Unreadable { reason } if reason.contains("not a regular file")),
            "{:?}",
            fact.status
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_absolute_root_never_appears_in_a_join_of_a_validated_relative_path() {
        let joined = join_project_relative(Path::new("/project"), "docs/RELEASE_NOTES.md");
        assert_eq!(joined, PathBuf::from("/project/docs/RELEASE_NOTES.md"));
        let escaped = join_project_relative(Path::new("/project"), "../escape.md");
        assert_ne!(
            escaped,
            PathBuf::from("/escape.md"),
            "a path that was not validated resolves to nothing usable rather than outside"
        );
    }

    #[test]
    fn a_budget_state_maps_to_exactness_without_inventing_admissibility() {
        let exact = budget_fact(
            BudgetState::Exact,
            Some(1024),
            true,
            Some("section-headers".to_owned()),
            None,
            None,
        );
        assert!(exact.exact && exact.admissible);
        let partial = budget_fact(
            BudgetState::Partial,
            Some(1024),
            true,
            None,
            Some("2 sections unattributed".to_owned()),
            None,
        );
        assert!(
            !partial.exact,
            "a floor is never presented as a complete total"
        );
        let weak = budget_fact(BudgetState::Exact, Some(1024), false, None, None, None);
        assert!(
            weak.exact && !weak.admissible,
            "the basis, not the number, is what is weak"
        );
    }

    #[test]
    fn the_context_carries_nothing_but_facts_and_never_a_root() {
        let facts = SnapshotFacts::from_stored(
            "snap-1".to_owned(),
            vec![GateArtifactFact {
                kind: ArtifactKind::Elf,
                sha256: Fact::known("a".repeat(64)),
                byte_size: 512,
            }],
            None,
            GateUnknownEvidence::default(),
        );
        let git = GitObservation::unavailable("git is not installed");
        let request = GateRunRequest::with_default_policy(&facts, &git);
        let context = build_context(&request);
        assert_eq!(context.snapshot_id, "snap-1");
        assert_eq!(context.artifacts.len(), 1);
        assert!(!context.git.available);
        assert!(
            context.version.is_none(),
            "no `[version]` policy was configured"
        );
        let text = format!("{context:?}");
        for forbidden in ["C:\\", "/Users/", "/home/", "/tmp"] {
            assert!(
                !text.contains(forbidden),
                "the context carried a host path: {text}"
            );
        }
    }

    #[test]
    fn a_version_policy_is_applied_to_the_tag_the_probe_found() {
        let facts = SnapshotFacts::from_stored(
            "snap-1".to_owned(),
            vec![GateArtifactFact {
                kind: ArtifactKind::Elf,
                sha256: Fact::known("a".repeat(64)),
                byte_size: 512,
            }],
            None,
            GateUnknownEvidence::default(),
        );
        let mut git = GitObservation::unavailable("placeholder");
        git.facts = firmwaresight_core::domain::gate::GateGitFacts {
            available: true,
            head_commit: Fact::known("0123456789abcdef0123456789abcdef01234567".to_owned()),
            exact_tag: Fact::known("v1.2.3".to_owned()),
            dirty: Fact::known(false),
        };
        git.exact_tags = vec!["v1.2.3".to_owned()];
        let mut request = GateRunRequest::with_default_policy(&facts, &git);
        request.policy.version = Some(firmwaresight_core::domain::gate::VersionPolicy {
            pattern: r"^v(?P<version>\d+\.\d+\.\d+)$".to_owned(),
            expected_version: Some("1.2.3".to_owned()),
        });
        let context = build_context(&request);
        let version = context
            .version
            .as_ref()
            .expect("a version policy was configured");
        assert_eq!(version.pattern_matches, Fact::known(true));
        assert_eq!(version.captured_version, Fact::known("1.2.3".to_owned()));
        let evaluation = context.evaluate(&fingerprint::run_id(&context));
        let finding = evaluation
            .finding(firmwaresight_core::domain::gate::GateRuleId::VersionMatchesPolicy)
            .expect("every rule answers");
        assert_eq!(
            finding.state,
            firmwaresight_core::domain::gate::FindingState::Pass,
            "{}",
            finding.summary
        );
    }
}
