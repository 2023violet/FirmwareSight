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
    GateArtifactFact, GateAttachmentFact, GateBudgetFact, GateContext, GateFileFact,
    GateFileStatus, GateGrowthFacts, GateMemoryFacts, GatePolicy, GateUnknownEvidence, KindBasis,
    UnknownPolicy,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
use firmwaresight_core::domain::release::sanitize_leaf_name;

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
/// The configured string is placed by the same walk `config::validate` checks it against, so a caller that
/// skipped validation is still handed nothing outside the root, and a string the walk refuses is refused
/// before a `stat`, a read or a hash. Absence and an unreadable file are different facts: only the first is
/// a policy failure. Containment here is **lexical**: a link that sits inside the root and points outside it
/// is read at its target, which is the open question `D-N11-1` records.
#[must_use]
pub fn observe_release_notes(root: &Path, relative_path: &str) -> GateFileFact {
    let missing = |reason: &str| GateFileFact {
        relative_path: relative_path.to_owned(),
        status: GateFileStatus::Unreadable {
            reason: reason.to_owned(),
        },
    };
    let path = match resolve_project_relative(root, relative_path) {
        Ok(path) => path,
        Err(_) => {
            return missing("the configured path is not project-relative, so it was not read");
        }
    };
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

/// One file the release owner chose to ship, as this crate observed it: a leaf name, the kind they
/// declared for it, the digest of its bytes and their length. No structural claim lives here, because
/// nothing read the file as a container (`04_TECH/28` §2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAttachment {
    /// The sanitized leaf name. A name is release identity and never Gate identity (§5 rules 4 and 5),
    /// so this field is what a person recognizes, not what a run id binds.
    pub file_name: String,
    /// The leaf the release owner chose, with only its directory removed (`N01-F02`). [`Self::file_name`]
    /// is the name a portable folder may carry; this is the name the person who picked the file reads back,
    /// so `固件镜像.bin` is not shown to them as `bin`. It is session-local by construction: no bundle
    /// document is built from it, and the directory it came from is dropped here rather than carried.
    pub display_name: String,
    pub kind: ArtifactKind,
    pub sha256: Fact<String>,
    pub byte_size: u64,
    pub kind_basis: KindBasis,
}

impl ReleaseAttachment {
    /// The row the Gate binds: these facts with the name removed. One projection, so no surface can
    /// re-derive a digest on the way into a context.
    #[must_use]
    pub fn as_gate_fact(&self) -> GateAttachmentFact {
        GateAttachmentFact {
            kind: self.kind,
            sha256: self.sha256.clone(),
            byte_size: self.byte_size,
            kind_basis: self.kind_basis.clone(),
        }
    }
}

/// Why one chosen file could not become an attachment. Every arm is a refusal a release owner can act
/// on, so none is filed as an internal error, and none names a host path: the `name` carried is a
/// sanitized leaf (`04_TECH/28` §4.3 E-1, E-2 and E-3). E-4 `AttachmentSetChanged` is the preview-to-export
/// refusal `C1-U2` put on [`crate::bundle::BundleError`], because a set only changes between two bundle calls,
/// so it is not one of these three.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AttachmentError {
    /// E-1: the stat failed, the path is not a regular file, or the streamed read failed.
    #[error("`{name}` is not a readable regular file: {detail}")]
    Unreadable { name: String, detail: String },

    /// E-2: an analyzed kind was offered. `elf` and `map` are produced by the analysis path, and an
    /// unanalyzed file must not stand in for the ELF a release is about (`ADR-0030` D-3).
    #[error(
        "a `{}` file is an analysis input rather than a release attachment",
        kind.word()
    )]
    KindNotAttaching { kind: ArtifactKind },

    /// E-3: the file is 0 bytes. Without this refusal an empty file would be evidence of an image.
    #[error("`{name}` is empty, so it carries no bytes to attach")]
    Empty { name: String },
}

impl AttachmentError {
    /// The stable diagnostics code, continuing the release family the engine registers in
    /// `crates/firmwaresight-project/src/bundle.rs` and `crates/firmwaresight-core/src/domain/release.rs`
    /// (`ERR-BUNDLE-6101`..`6114`, then `6115`..`6117` here and `6118` for the bundle-side attachment-set
    /// refusal). The observation refusal is not a `BundleError` variant of its own: an attachment is observed
    /// long before any bundle is planned, and `C1-U2`'s export path carries it through
    /// [`crate::bundle::BundleError::Attachment`] rather than re-describing it, so one refusal keeps one code
    /// and one next step wherever a person meets it.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unreadable { .. } => "ERR-BUNDLE-6115",
            Self::KindNotAttaching { .. } => "ERR-BUNDLE-6116",
            Self::Empty { .. } => "ERR-BUNDLE-6117",
        }
    }

    /// The next step, stated as the owner's action rather than as this crate's failure.
    #[must_use]
    pub fn remediation(&self) -> &'static str {
        match self {
            Self::Unreadable { .. } => {
                "choose the file again, or close the program that holds it open"
            }
            Self::KindNotAttaching { .. } => {
                "analyze the ELF on the Analyze page and attach the MAP there; a release attachment is a \
                 BIN, an Intel HEX or an unidentified file"
            }
            Self::Empty { .. } => "attach the built image; an empty file ships nothing to verify",
        }
    }

    /// The leaf name the person can act on, where one exists. The kind refusal is about a declaration,
    /// so it names no file.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Unreadable { name, .. } | Self::Empty { name } => Some(name),
            Self::KindNotAttaching { .. } => None,
        }
    }
}

/// The one observation path for a release attachment (`04_TECH/28` §3).
///
/// The order is the design's: refuse a kind that is never an attachment, then `stat`, then regular
/// file, then non-empty, then the streaming SHA-256, then the length. `GuardedInput::load` is not used,
/// because it allocates a whole file for parsing; `fingerprint::file_sha256` streams a 64 KiB buffer and
/// holds nothing, so an attachment of any size is hashed without being read into memory.
///
/// The length recorded is the one the same `stat` reported for the file that was then hashed. A file
/// that moves between the two is caught at export, where the digest and size are re-checked together
/// (§4.2's `SourceArtifactChanged` row); nothing here re-reads the file to reassure itself, and no
/// second observation path exists to disagree with the first.
///
/// # Errors
///
/// [`AttachmentError::KindNotAttaching`] for `elf` or `map`, [`AttachmentError::Unreadable`] when the
/// path is missing, is not a regular file or cannot be read, and [`AttachmentError::Empty`] for 0 bytes.
pub fn observe_attachment(
    path: &Path,
    declared_kind: ArtifactKind,
) -> Result<ReleaseAttachment, AttachmentError> {
    if matches!(declared_kind, ArtifactKind::Elf | ArtifactKind::Map) {
        return Err(AttachmentError::KindNotAttaching {
            kind: declared_kind,
        });
    }
    let name = attachment_leaf_name(path);
    let metadata = std::fs::metadata(path).map_err(|error| AttachmentError::Unreadable {
        name: name.clone(),
        detail: selection_reason(&error),
    })?;
    if !metadata.is_file() {
        return Err(AttachmentError::Unreadable {
            name,
            detail: "the chosen path is not a regular file".to_owned(),
        });
    }
    if metadata.len() == 0 {
        return Err(AttachmentError::Empty { name });
    }
    let digest = fingerprint::file_sha256(path).map_err(|error| AttachmentError::Unreadable {
        name: name.clone(),
        detail: io_reason(&error),
    })?;
    Ok(ReleaseAttachment {
        file_name: name,
        display_name: attachment_display_name(path),
        kind: declared_kind,
        sha256: Fact::known(digest),
        byte_size: metadata.len(),
        // The owner declared this kind by choosing the file; nothing here inferred it from the bytes
        // (`ADR-0030` D-8), so a derived basis is never produced by an observation.
        kind_basis: KindBasis::Declared,
    })
}

/// The leaf name an attachment refusal may quote. The full path stays with the caller: `AGENTS.md` §7 and
/// `04_TECH/28` §4.4 forbid a host path in a message a person copies into a report.
fn attachment_leaf_name(path: &Path) -> String {
    let raw = path
        .file_name()
        .map(|leaf| leaf.to_string_lossy().into_owned())
        .unwrap_or_default();
    sanitize_leaf_name(&raw)
}

/// The leaf the release owner chose, as they chose it (`N01-F02`). Only the directory is dropped, so
/// `固件镜像.bin` keeps the name it has on disk instead of becoming the archive-safe `bin`. A leaf that is
/// not valid UTF-8 arrives lossily: a display name is there to show the file, not to refuse the release,
/// and the bytes that carry identity are the digest and the length, never this string.
fn attachment_display_name(path: &Path) -> String {
    path.file_name()
        .map(|leaf| leaf.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Place a validated relative path onto a project root, or refuse it. One walk, shared with
/// `config::require_project_relative` through [`config::place_project_relative`], so the answer to "is this
/// legal" and the answer to "where does it land" cannot drift apart.
///
/// A refusal is a typed refusal, not a path: it holds a fixed sentence and nothing about it can be opened,
/// hashed or shipped. Before this, a path the walk did not understand resolved onto a fallback *file name*
/// inside the root — and a file name is something a project can own, which is `D-N10-1`.
///
/// The containment this guarantees is lexical. Whether a link that sits inside the root and points outside
/// it should be followed is `D-N11-1`, an open contract question this function deliberately does not decide.
#[must_use = "a refusal is not a path: nothing may be read, hashed or shipped from it"]
pub fn resolve_project_relative(
    root: &Path,
    relative_path: &str,
) -> Result<PathBuf, config::UnplaceableProjectPath> {
    config::place_project_relative(relative_path).map(|placed| root.join(placed))
}

/// A short, path-free description of an I/O failure met *after* the path was already seen. `std::io::Error`'s
/// `Display` can name the file, so only the kind is carried into a fact, a run record or a UI.
fn io_reason(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => "permission denied".to_owned(),
        std::io::ErrorKind::NotFound => "the file disappeared during the read".to_owned(),
        kind => format!("a filesystem error occurred ({kind:?})"),
    }
}

/// Why the *first* `stat` of a file a person chose failed. [`io_reason`] speaks for a failure that follows a
/// successful `stat`, which really is a file that moved; here nothing was ever confirmed to exist, so
/// saying it disappeared would describe an event that did not happen (`N01-F01`). Every other kind keeps the
/// one wording, because a permission denial is the same statement whichever call met it.
fn selection_reason(error: &std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        return "no file exists at the path chosen".to_owned();
    }
    io_reason(error)
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
///
/// This is the attachment-free entry point every surface already uses; the release path that ships files
/// calls [`build_context_with_attachments`] with the rows it observed, so one function assembles one context
/// and no caller can present a set the Gate did not bind.
#[must_use]
pub fn build_context(request: &GateRunRequest) -> GateContext {
    build_context_with_attachments(request, &[])
}

/// The same assembly with the release attachments bound into it (`04_TECH/28` §3's judging row, §5.D of the
/// `C1-U2` authorization). The rows enter as `ReleaseAttachment::as_gate_fact()` and nowhere else: a second
/// projection would be a second place a digest could be re-derived and disagree with the first.
///
/// An empty slice keeps the canonical input byte-identical to the `/1` every stored run already carries,
/// which is what makes an old run id survive this unit (`ADR-0030` D-4).
#[must_use]
pub fn build_context_with_attachments(
    request: &GateRunRequest,
    attachments: &[ReleaseAttachment],
) -> GateContext {
    let version = request.policy.version.as_ref().map(|policy| {
        version::evaluate_tag(
            &policy.pattern,
            request.git.facts.exact_tag.value().map(String::as_str),
        )
    });
    GateContext {
        snapshot_id: request.target.snapshot_id.clone(),
        artifacts: request.target.artifacts.clone(),
        attachments: attachments
            .iter()
            .map(ReleaseAttachment::as_gate_fact)
            .collect(),
        memory: request.target.memory.clone(),
        git: request.git.facts.clone(),
        version,
        release_notes: request.release_notes.clone(),
        growth: request.growth.clone(),
        unknown_evidence: request.target.unknown_evidence.clone(),
        policy: request.policy.clone(),
    }
}

/// One file the release owner chose to ship, as they chose it: a path and the kind they declared for it.
///
/// Deliberately not a `ReleaseAttachment`. The attachment's digest, length and sanitized leaf are what
/// `observe_attachment` establishes, and the bundle re-establishes them at export; a caller that handed the
/// engine a finished fact would have decided in advance what the engine was about to measure. The path is
/// the caller's own string and stays inside this crate's private re-check structures, never a portable
/// document or an error message (`04_TECH/28` §4.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentSelection {
    pub path: PathBuf,
    /// Declared, never inferred from the bytes (`ADR-0030` D-8): an extension does not make a file an
    /// attachment of a kind, and nothing here reads the file as a container.
    pub declared_kind: ArtifactKind,
}

/// The declared kind a file name states, for the one surface that gets a name and no kind: `04_TECH/28` §9
/// freezes the CLI's attach carrier as a repeatable `--attach <FILE>`, so a `.bin` argument has to be recorded
/// as *some* kind and the only fact that edge was given is the extension.
///
/// This is a reading of the release owner's words, not of their bytes. It opens nothing, looks for no record
/// structure, and knows two spellings — `bin` and `hex`, in any case — and nothing else, so `payload.ihex`
/// arrives as `unknown` rather than as a claim this rule cannot support. It can never return `Elf` or `Map`:
/// those are the kinds only an analyzed file may hold, and `observe_attachment` refuses them as declarations
/// (§2.2). A surface that can ask — the desktop — offers the three kinds and uses this only to pre-fill.
#[must_use]
pub fn declared_kind_for_path(path: &Path) -> ArtifactKind {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("bin") => ArtifactKind::Bin,
        Some("hex") => ArtifactKind::IntelHex,
        _ => ArtifactKind::Unknown,
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

    /// Every spelling the config accepts, and the one place the shared walk puts it. A `..` that comes back
    /// inside the root is the legal case `D-N10-1` was refusing to place.
    #[test]
    fn a_project_relative_path_lands_where_its_components_normalize_to() {
        let root = Path::new("/project");
        for (configured, expected) in [
            ("RELEASE_NOTES.md", "/project/RELEASE_NOTES.md"),
            ("docs/NOTES.md", "/project/docs/NOTES.md"),
            ("docs/../RELEASE_NOTES.md", "/project/RELEASE_NOTES.md"),
            ("docs/release/../NOTES.md", "/project/docs/NOTES.md"),
            ("./RELEASE_NOTES.md", "/project/RELEASE_NOTES.md"),
            ("docs/./../build/NOTES.md", "/project/build/NOTES.md"),
        ] {
            let placed = resolve_project_relative(root, configured)
                .unwrap_or_else(|refusal| panic!("`{configured}` is legal: {}", refusal.reason()));
            assert_eq!(
                placed,
                PathBuf::from(expected),
                "`{configured}` places at the file it names"
            );
        }
    }

    /// What the walk refuses, it refuses as a sentence. There is no returned path to open, so a refusal
    /// cannot be occupied by a file somebody else wrote — the property the fallback name did not have.
    #[test]
    fn a_path_that_cannot_be_placed_is_refused_rather_than_resolved() {
        for configured in [
            "../escape.md",
            "docs/../../escape.md",
            "/etc/passwd",
            "C:\\Windows\\win.ini",
            "C:/Windows/win.ini",
            "",
            "docs/..",
            "..",
        ] {
            let outcome = resolve_project_relative(Path::new("/project"), configured);
            let error = match outcome {
                Ok(placed) => panic!(
                    "`{configured}` resolved to {}, which is a place to read from",
                    placed.display()
                ),
                Err(error) => error,
            };
            assert!(
                !error.reason().is_empty(),
                "a refusal still says why nothing was placed"
            );
            assert!(
                config::require_project_relative(configured, "release.release_notes_path").is_err(),
                "`{configured}` is refused by the validator too: the two must not disagree"
            );
        }
    }

    /// The adversarial case, run against a real folder: the project owns a file whose name used to be where
    /// an unplaceable path landed. It is somebody's file, and it is not these notes.
    #[test]
    fn a_file_named_like_the_old_fallback_never_becomes_the_configured_notes() {
        let root = std::env::temp_dir().join(format!(
            "firmwaresight-notes-fallback-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|value| value.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&root).expect("the temporary root exists");
        std::fs::write(root.join("RELEASE_NOTES.md"), "# the owner's notes\n")
            .expect("the configured notes are written");
        std::fs::write(
            root.join("firmwaresight-unresolvable-path"),
            "bytes nobody configured\n",
        )
        .expect("the fallback-named file is written");

        let configured = fingerprint::file_sha256(&root.join("RELEASE_NOTES.md"))
            .expect("the configured notes are readable");
        let fallback = fingerprint::file_sha256(&root.join("firmwaresight-unresolvable-path"))
            .expect("the fallback-named file is readable");
        assert_ne!(
            configured, fallback,
            "the fixture is only a fixture if they differ"
        );

        let fact = observe_release_notes(&root, "docs/../RELEASE_NOTES.md");
        assert!(
            matches!(&fact.status, GateFileStatus::Present { sha256: Some(digest) } if *digest == configured),
            "the notes fact carries the configured file's digest, not the fallback's: {:?}",
            fact.status
        );
        assert_eq!(
            fact.relative_path, "docs/../RELEASE_NOTES.md",
            "the fact quotes the path as configured"
        );

        // The same answer with the fallback name free: only the file that was never chosen is missing.
        std::fs::remove_file(root.join("firmwaresight-unresolvable-path"))
            .expect("the fallback-named file is removed");
        let free = observe_release_notes(&root, "docs/../RELEASE_NOTES.md");
        assert_eq!(
            free, fact,
            "the fallback name is not part of what the Gate reads"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Every way a configured path can be unusable, and the one sentence the Gate records for all of them.
    /// The reason did not change with `D-N10-1`: a refused path was an evidence gap before and is one after,
    /// and in neither case was anything read.
    #[test]
    fn a_refused_notes_path_is_the_same_evidence_gap_it_always_was() {
        for configured in [
            "../escape.md",
            "docs/../../escape.md",
            "/etc/passwd",
            "C:\\Windows\\win.ini",
            "C:/Windows/win.ini",
            "",
            "docs/..",
        ] {
            let fact = observe_release_notes(Path::new("/project"), configured);
            assert_eq!(
                fact.status,
                GateFileStatus::Unreadable {
                    reason: "the configured path is not project-relative, so it was not read"
                        .to_owned()
                },
                "`{configured}` is refused before anything is read"
            );
            assert_eq!(
                fact.relative_path, configured,
                "the fact quotes what was configured, not a normalized stand-in"
            );
        }
    }

    #[test]
    fn the_validator_and_the_resolver_agree_on_every_path_this_round_measured() {
        for configured in [
            "RELEASE_NOTES.md",
            "docs/NOTES.md",
            "docs/../RELEASE_NOTES.md",
            "../escape.md",
            "docs/../../escape.md",
            "C:\\escape.md",
            "",
        ] {
            let legal =
                config::require_project_relative(configured, "release.release_notes_path").is_ok();
            let placed = resolve_project_relative(Path::new("/project"), configured).is_ok();
            assert_eq!(
                legal, placed,
                "`{configured}` cannot be accepted by one and refused by the other"
            );
        }
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

    /// `04_TECH/28` §9's CLI row freezes one carrier — a repeatable `--attach <FILE>` with no kind flag — so a
    /// file name is all that edge gets. These four cases hold the rule that answers for it.
    #[test]
    fn a_bin_name_declares_bin_and_a_hex_name_declares_intel_hex() {
        assert_eq!(
            declared_kind_for_path(Path::new("build/app.bin")),
            ArtifactKind::Bin
        );
        assert_eq!(
            declared_kind_for_path(Path::new("build/boot.hex")),
            ArtifactKind::IntelHex
        );
    }

    #[test]
    fn the_name_rule_reads_the_extension_case_insensitively_and_never_opens_the_file() {
        // `FIRMWARE.BIN` is the same statement as `firmware.bin`. A rule that had to read bytes to know that
        // would be a format check under another name, which `ADR-0030` D-8 forbids.
        for (name, expected) in [
            ("FIRMWARE.BIN", ArtifactKind::Bin),
            ("Mixed.Bin", ArtifactKind::Bin),
            ("Release.HEX", ArtifactKind::IntelHex),
            ("Mixed.Hex", ArtifactKind::IntelHex),
        ] {
            assert_eq!(declared_kind_for_path(Path::new(name)), expected, "{name}");
        }
    }

    #[test]
    fn a_name_that_says_nothing_this_rule_knows_declares_unknown() {
        // Including `.ihex`, a real Intel HEX spelling this rule deliberately does not claim. A person who
        // means it declares so explicitly; the CLI's only carrier cannot ask, so the honest answer is unknown,
        // and `unknown` satisfies no required kind.
        for name in [
            "payload",
            "payload.ihex",
            "image.img",
            "app.bin.tmp",
            "notes.",
        ] {
            assert_eq!(
                declared_kind_for_path(Path::new(name)),
                ArtifactKind::Unknown,
                "{name} names neither of the two spellings"
            );
        }
    }

    #[test]
    fn the_name_rule_can_never_declare_a_kind_that_only_an_analyzed_file_has() {
        // `observe_attachment` refuses `elf` and `map` as declarations (§2.2); a name rule that could produce
        // them would let a file name claim the status of a parsed artifact.
        assert_eq!(
            declared_kind_for_path(Path::new("firmware.elf")),
            ArtifactKind::Unknown
        );
        assert_eq!(
            declared_kind_for_path(Path::new("firmware.map")),
            ArtifactKind::Unknown
        );
    }
}
