//! Portable release semantics: what a Release Bundle asserts, and how that assertion is identified.
//!
//! A bundle is not an export of whatever happens to be on screen. It is the packaged answer of a Gate
//! that passed, so the model below is a *verdict* plus the facts that verdict was reached over. The rule
//! that gives a bundle its meaning is that the verdict must be `PASS` after accepted Reviews are applied,
//! and that `UNKNOWN` is never acceptable as a Review — an evidence gap therefore cannot be packaged as a
//! release, which is the same ADR-0023 property the Gate enforces, seen from the other side.
//!
//! Like `GateContext`, this module builds the canonical *input text* a fingerprint is taken over and never
//! the digest itself: `firmwaresight-core` declares no external dependencies (ADR-0027), so the SHA-256
//! happens in `firmwaresight-project`, which already owns `run_id` and `policy_sha256` for the same reason.
//! The pair `canonical_release_text` / `release-<sha256>` mirrors `canonical_input` / `gate-<sha256>`
//! exactly, and for the same reason: a release id must answer "would re-assembling this produce the same
//! thing?", which is a question about content, never about a clock or a filesystem.
//!
//! Nothing here reads, writes, copies or hashes a file, and no field is a path. `04_TECH/05` names bundle
//! filename normalization and path-traversal prevention as requirements of an untrusted-input product, so
//! the naming rules live in the deterministic layer that can be tested on every platform rather than in
//! whichever crate happens to be holding the directory handle.

use crate::domain::gate::EffectiveSeverity;
use crate::domain::identity::{ArtifactKind, Sha256};

/// The prefix that marks an id as a release id in storage, portable output and the UI.
pub const RELEASE_ID_PREFIX: &str = "release-";

/// How many hex characters follow the prefix.
const RELEASE_ID_HEX_LEN: usize = 64;

/// The label every release fingerprint starts with, so a digest over a different kind of input can never
/// collide with one over a release.
pub const RELEASE_INPUT_LABEL: &str = "firmwaresight-release-input/1";

/// A deterministic release identifier: `release-<64 lowercase hex>`.
///
/// Construction is restricted to [`ReleaseModel::release_id_from`], so an id that exists has been derived
/// from a canonical release input. There is no parse-from-arbitrary-string constructor that yields this
/// type for a caller-invented value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReleaseId(String);

impl ReleaseId {
    /// Validate a full id. Kept public because storage and the portable document read ids back from disk
    /// or from JSON, and a stored `release-` row whose tail is not hex is a corrupted record, not a name.
    pub fn parse(input: &str) -> Result<Self, ReleaseError> {
        let Some(hex) = input.strip_prefix(RELEASE_ID_PREFIX) else {
            return Err(ReleaseError::MalformedReleaseId {
                value: input.to_owned(),
            });
        };
        if hex.len() != RELEASE_ID_HEX_LEN
            || !hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ReleaseError::MalformedReleaseId {
                value: input.to_owned(),
            });
        }
        Ok(Self(input.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The leading hex, for a display name that must stay short and must not wrap.
    ///
    /// Twelve characters is 2^48 of the digest: enough that two bundles of the same release collide by
    /// accident no more often than a released firmware version number repeats, and short enough to read.
    #[must_use]
    pub fn short_hex(&self) -> &str {
        &self.0[RELEASE_ID_PREFIX.len()..RELEASE_ID_PREFIX.len() + 12]
    }
}

/// Where the project release version came from.
///
/// `04_TECH/08` gives the MVP exactly one `[version] source`, and a release version may only be read from
/// the same project and Git context the Gate evaluated. The absence of an app-version, artifact-filename,
/// import-timestamp or user-typed variant is the point of this type: an export-time version string the
/// Gate never judged cannot be expressed here, so it cannot reach a manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReleaseVersionSource {
    /// `release.expected_version`, when configured and the version rule passed.
    ExpectedVersion,
    /// The `version` capture of the exact Git tag validated by `[version].pattern`.
    GitTagCapture,
}

/// A project release version that is safe to put in a filename, a manifest and a report.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReleaseVersion(String);

impl ReleaseVersion {
    /// Reject empty, whitespace-only, over-long, control-character-bearing and separator-bearing values.
    ///
    /// The separator rule is not decoration: a version reaching `release-manifest.json` also reaches the
    /// proposed bundle directory name, so `../` in a version would be a path-traversal vector with a
    /// plausible-looking source.
    pub fn parse(input: &str) -> Result<Self, ReleaseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed != input {
            return Err(ReleaseError::VersionUnavailable {
                reason: "the resolved version is empty or padded with whitespace".to_owned(),
            });
        }
        if trimmed.len() > 64 {
            return Err(ReleaseError::VersionUnavailable {
                reason: "the resolved version is longer than 64 characters".to_owned(),
            });
        }
        if trimmed
            .chars()
            .any(|ch| ch == '/' || ch == '\\' || ch.is_control())
        {
            return Err(ReleaseError::VersionUnavailable {
                reason: "the resolved version contains a path separator or a control character"
                    .to_owned(),
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One file the bundle ships as release content.
///
/// There is deliberately no source path: `file_name` is a sanitized bundle-relative leaf, and the bytes it
/// names are proven elsewhere by re-hashing the source before the copy (prompt §22).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReleaseArtifact {
    pub kind: ArtifactKind,
    pub file_name: String,
    pub sha256: Sha256,
    pub byte_size: u64,
}

impl ReleaseArtifact {
    /// The stable word the Gate and the manifest both use for a kind.
    ///
    /// [`ArtifactKind::word`] holds the vocabulary; this is the spelling the release documents have always
    /// used, kept as the name callers know and as the one place the two could drift apart.
    #[must_use]
    pub fn kind_word(kind: ArtifactKind) -> &'static str {
        kind.word()
    }

    #[must_use]
    pub fn kind_word_for(&self) -> &'static str {
        Self::kind_word(self.kind)
    }
}

/// The Release Notes the Gate observed, identified by digest rather than by path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReleaseNotesDigest {
    /// The canonical bundle-relative name, always `release-notes.md`.
    pub bundle_name: String,
    pub sha256: Sha256,
}

/// The major versions of the portable contracts a bundle writes.
///
/// A bundle whose documents are at different majors is a different release object, so these belong in the
/// fingerprint. They are also what a reader needs to interpret the files without asking FirmwareSight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaMajors {
    pub analysis: u32,
    pub diff: u32,
    pub gate_results: u32,
    pub accepted_reviews: u32,
    pub release_manifest: u32,
}

impl SchemaMajors {
    /// The set this build emits. `diff` is `None` in [`ReleaseModel`] when no baseline exists, which is a
    /// present-or-absent fact about the bundle rather than a version, so it still carries its major.
    pub const CURRENT: Self = Self {
        analysis: 1,
        diff: 1,
        gate_results: 1,
        accepted_reviews: 1,
        release_manifest: 1,
    };
}

/// Workspace Git facts, kept as what the Gate observed about the workspace.
///
/// `04_TECH/24` is explicit that a workspace HEAD is not proof the artifact was built from it, and this
/// type exists to keep that distinction structural: there is no field here that could be read as artifact
/// build provenance, and the report wording follows the same rule (prompt §43).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceGit {
    pub head_commit: Option<String>,
    pub exact_tag: Option<String>,
    pub dirty: Option<bool>,
}

impl WorkspaceGit {
    #[must_use]
    pub fn observed_any(&self) -> bool {
        self.head_commit.is_some() || self.exact_tag.is_some() || self.dirty.is_some()
    }
}

/// The semantic inputs of one release, before any filesystem is involved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseModel {
    /// The build that ships.
    pub snapshot_id: String,
    /// The build the diff explains growth against, if any. Never shipped (§23).
    pub baseline_snapshot_id: Option<String>,
    /// The Gate run this bundle is the output of. Revalidated at bundle time (§9).
    pub gate_run_id: String,
    /// The disposition of that run after accepted Reviews. Must be `PASS` (§8).
    pub disposition: EffectiveSeverity,
    pub version: ReleaseVersion,
    pub version_source: ReleaseVersionSource,
    /// Digest of the canonical `accepted-reviews` document that produced the `PASS`. Its content, so a
    /// changed acceptance changes the release id (§12).
    pub acceptances_sha256: Sha256,
    /// The current artifacts only, in canonical order (§23).
    pub artifacts: Vec<ReleaseArtifact>,
    pub release_notes: Option<ReleaseNotesDigest>,
    pub workspace_git: WorkspaceGit,
    pub schema_majors: SchemaMajors,
    /// The FirmwareSight build that assembled the bundle. Kept distinct from `version` in every surface.
    pub fwsight_version: String,
}

/// §8 as one rule with two callers: a REVIEW is not "almost a PASS" and a BLOCK is not a warning, and neither
/// may be packaged as a release.
///
/// [`ReleaseModel::validated`] applies it to the model that was built; the bundle engine applies it to the
/// aggregate it computed one step earlier, because a verdict that has already refused a release must be the
/// answer the release owner gets rather than whatever a file read says next. Two call sites, one statement of
/// the rule, so the engine cannot develop a disposition of its own (`AGENTS.md` 3).
pub fn require_packageable(disposition: EffectiveSeverity) -> Result<(), ReleaseError> {
    if disposition == EffectiveSeverity::Pass {
        Ok(())
    } else {
        Err(ReleaseError::GateNotReady { disposition })
    }
}

impl ReleaseModel {
    /// Refuse anything that would make the bundle a claim the evidence does not support, then put the
    /// artifact list in canonical order.
    ///
    /// Construction is separate from validation because the refusal rules *are* the product's core promise
    /// (§8), while a half-gathered set of facts is a normal intermediate state on the way to them.
    pub fn validated(mut self) -> Result<Self, ReleaseError> {
        require_packageable(self.disposition)?;

        // §17: a bundle is bound to one Gate run, so the id it names must be shaped like one.
        if !self.gate_run_id.starts_with("gate-")
            || self.gate_run_id.len() != "gate-".len() + RELEASE_ID_HEX_LEN
        {
            return Err(ReleaseError::MalformedGateRunId {
                value: self.gate_run_id.clone(),
            });
        }

        if self.snapshot_id.trim().is_empty() {
            return Err(ReleaseError::InternalInvariant {
                detail: "a release needs the snapshot it ships".to_owned(),
            });
        }
        if self.baseline_snapshot_id.as_deref().map(str::trim) == Some(self.snapshot_id.trim()) {
            return Err(ReleaseError::SelfComparison {
                snapshot_id: self.snapshot_id.clone(),
            });
        }

        if self.artifacts.is_empty() {
            return Err(ReleaseError::NoReleaseArtifacts);
        }
        for artifact in &self.artifacts {
            if artifact.file_name.is_empty()
                || artifact.file_name.contains('/')
                || artifact.file_name.contains('\\')
                || artifact.file_name == "."
                || artifact.file_name == ".."
            {
                return Err(ReleaseError::MalformedArtifactName {
                    value: artifact.file_name.clone(),
                });
            }
        }
        if let Some(notes) = &self.release_notes
            && notes.bundle_name != RELEASE_NOTES_NAME
        {
            return Err(ReleaseError::MalformedNotesName {
                value: notes.bundle_name.clone(),
            });
        }

        if self.fwsight_version.trim().is_empty() {
            return Err(ReleaseError::InternalInvariant {
                detail: "a bundle must name the build that produced it".to_owned(),
            });
        }

        // Canonical order is a property of the model, not of the caller's directory listing (§52.11).
        // Kind, then content, then size, then name: a total order, so no two inputs can sort differently by
        // accident of the order they arrived in.
        self.artifacts.sort_by(|left, right| {
            left.kind_word_for()
                .cmp(right.kind_word_for())
                .then_with(|| left.sha256.hex().cmp(right.sha256.hex()))
                .then_with(|| left.byte_size.cmp(&right.byte_size))
                .then_with(|| left.file_name.cmp(&right.file_name))
        });
        Ok(self)
    }

    /// The canonical text a release id is the digest of.
    ///
    /// Included: the snapshot, the optional baseline, the Gate run, the resolved version and how it was
    /// resolved, the acceptance set, the shipped artifact digests in canonical order, the notes digest,
    /// the schema majors and the FirmwareSight version.
    ///
    /// Excluded by construction, because no field of this type can hold them: an output directory, a
    /// project root, a source path, a filesystem timestamp, a process id, a temporary directory and a
    /// SQLite row id. That list is the promise §12 makes, and it is enforced by the absence of a place to
    /// put those values rather than by a check that they were left out.
    #[must_use]
    pub fn canonical_release_text(&self) -> String {
        let mut text = String::new();
        text.push_str(RELEASE_INPUT_LABEL);
        text.push('\n');
        text.push_str(&format!("snapshot={}\n", self.snapshot_id));
        text.push_str(&format!(
            "baseline={}\n",
            self.baseline_snapshot_id.as_deref().unwrap_or("none")
        ));
        text.push_str(&format!("gate_run={}\n", self.gate_run_id));
        text.push_str(&format!("disposition={}\n", self.disposition.as_str()));
        text.push_str(&format!("version={}\n", self.version.as_str()));
        text.push_str(&format!(
            "version_source={}\n",
            match self.version_source {
                ReleaseVersionSource::ExpectedVersion => "release.expected_version",
                ReleaseVersionSource::GitTagCapture => "git_tag.capture",
            }
        ));
        text.push_str(&format!("acceptances={}\n", self.acceptances_sha256.hex()));
        text.push_str(&format!("artifacts={}\n", self.artifacts.len()));
        for artifact in &self.artifacts {
            text.push_str(&format!(
                "artifact={}:{}:{}:{}\n",
                artifact.kind_word_for(),
                artifact.sha256.hex(),
                artifact.byte_size,
                artifact.file_name
            ));
        }
        text.push_str(&format!(
            "notes={}\n",
            self.release_notes
                .as_ref()
                .map_or_else(|| "none".to_owned(), |notes| notes.sha256.hex().to_owned())
        ));
        // Workspace Git facts enter the release id, because a bundle over the same build at a different
        // HEAD is a different release. They are labelled as workspace facts in the same breath.
        text.push_str(&format!(
            "workspace.head={}\n",
            self.workspace_git
                .head_commit
                .as_deref()
                .unwrap_or("unknown")
        ));
        text.push_str(&format!(
            "workspace.tag={}\n",
            self.workspace_git.exact_tag.as_deref().unwrap_or("unknown")
        ));
        text.push_str(&format!(
            "workspace.dirty={}\n",
            match self.workspace_git.dirty {
                Some(true) => "true",
                Some(false) => "false",
                None => "unknown",
            }
        ));
        text.push_str(&format!(
            "schemas=analysis:{},diff:{},gate-results:{},accepted-reviews:{},release-manifest:{}\n",
            self.schema_majors.analysis,
            self.schema_majors.diff,
            self.schema_majors.gate_results,
            self.schema_majors.accepted_reviews,
            self.schema_majors.release_manifest
        ));
        text.push_str(&format!("app={}\n", self.fwsight_version));
        text
    }

    /// Bind the digest the adapter computed to the prefix, so the id is assembled in exactly one way.
    #[must_use]
    pub fn release_id_from(digest_hex: &str) -> String {
        format!("{RELEASE_ID_PREFIX}{digest_hex}")
    }

    /// The shipped artifact the manifest names as `build.artifact_sha256`.
    ///
    /// `release-manifest` v1 describes one primary artifact per build, so the ELF is primary and a supplied
    /// MAP is covered by `files[]` rather than by pretending the schema has a second slot.
    #[must_use]
    pub fn primary_artifact(&self) -> Option<&ReleaseArtifact> {
        self.artifacts
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::Elf)
            .or_else(|| self.artifacts.first())
    }
}

/// The canonical bundle-relative name of the Release Notes copy.
pub const RELEASE_NOTES_NAME: &str = "release-notes.md";

/// The bundle-relative directory the shipped artifacts sit in.
pub const ARTIFACTS_DIR: &str = "artifacts";

/// The rest of the canonical layout names (§7). They live here, next to the two that were already
/// Core's, so the crate that stages files and the crate that writes document content cannot each
/// spell `SHA256SUMS` slightly differently — the mismatch that would make a bundle half-verifiable.
pub const ANALYSIS_DOC_NAME: &str = "analysis.json";
pub const DIFF_DOC_NAME: &str = "diff.json";
pub const GATE_RESULTS_DOC_NAME: &str = "gate-results.json";
pub const ACCEPTED_REVIEWS_DOC_NAME: &str = "accepted-reviews.json";
pub const REPORT_DOC_NAME: &str = "release-report.html";
pub const SHA256SUMS_NAME: &str = "SHA256SUMS";
pub const MANIFEST_DOC_NAME: &str = "release-manifest.json";

/// The two names `SHA256SUMS` never carries (§20). A file that hashed itself would need a fixed
/// point, and a faked empty or zeroed self-hash is a lie about coverage.
pub const SUMS_EXCLUDED_NAMES: [&str; 2] = [SHA256SUMS_NAME, MANIFEST_DOC_NAME];

/// The one name `release-manifest.json` never carries in its `files` array (§20).
pub const MANIFEST_EXCLUDED_NAMES: [&str; 1] = [MANIFEST_DOC_NAME];

// --------------------------------------------------------------------------- filename safety

/// Windows refuses these leaf names regardless of case or extension (`CON`, `CON.txt`, `con.tar`).
const WINDOWS_RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// True when a leaf name would be refused or silently mangled on a supported platform.
#[must_use]
pub fn is_reserved_leaf_name(name: &str) -> bool {
    let head = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    WINDOWS_RESERVED.contains(&head.as_str())
}

/// Reduce a source file name to a safe bundle-relative leaf, keeping the recognizable name where it is safe.
///
/// Untrusted-input rule, not a nicety: `04_TECH/05` lists path-traversal prevention and bundle filename
/// normalization among the requirements for a product that reads files it did not produce. The source
/// artifact path is chosen by a person in a dialog, so this is where a name like
/// `../../Windows/System32/cmd.exe` or `firmware.elf\n` stops being a path.
#[must_use]
pub fn sanitize_leaf_name(raw: &str) -> String {
    // Only the final component survives, and it is taken on both separators so a Windows path and a POSIX
    // path are handled by one rule rather than by whichever one the host happens to use.
    let leaf = raw
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(raw)
        .trim_matches('\0');

    let mut out = String::with_capacity(leaf.len());
    let mut previous_was_separator = false;
    for ch in leaf.chars() {
        let keep = matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_');
        if keep {
            out.push(ch);
            previous_was_separator = false;
        } else if !previous_was_separator {
            out.push('-');
            previous_was_separator = true;
        }
    }

    // A leading or trailing dot or space is what Windows drops from a name it accepts.
    let mut trimmed = out.trim_matches(|ch| ch == '.' || ch == '-').to_owned();
    if trimmed.is_empty() {
        trimmed = "artifact".to_owned();
    }
    if is_reserved_leaf_name(&trimmed) {
        trimmed = format!("_{trimmed}");
    }

    // A leaf that would overflow a MAX_PATH-length destination is truncated rather than rejected, and the
    // truncation is deterministic: it keeps the tail extension so the file still says what it is.
    const MAX_LEAF: usize = 96;
    if trimmed.len() > MAX_LEAF {
        let extension = match trimmed.rfind('.') {
            Some(dot) if dot > 0 && dot + 1 < trimmed.len() => trimmed[dot..].to_owned(),
            _ => String::new(),
        };
        let extension = if extension.len() >= MAX_LEAF {
            String::new()
        } else {
            extension
        };
        let stem: String = trimmed.chars().take(MAX_LEAF - extension.len()).collect();
        trimmed = format!("{}{}", stem.trim_end_matches('.'), extension);
    }
    trimmed
}

/// The deterministic second name for a leaf that collides with another artifact's.
///
/// `<kind>-<sha8>-<sanitized-name>`, keyed on content rather than on an index, so the same two files
/// collide the same way in every bundle and a reader can tell which bytes each name stands for.
#[must_use]
pub fn disambiguated_name(kind: ArtifactKind, sha256: &Sha256, name: &str) -> String {
    format!(
        "{}-{}-{}",
        ReleaseArtifact::kind_word(kind),
        &sha256.hex()[..8],
        sanitize_leaf_name(name)
    )
}

/// Whether a string may name a file inside a bundle.
///
/// This is the whole path-escape rule, stated once: `/` separators only, no absolute form, no drive
/// letter, no `.` or `..` segment, no empty or control-bearing segment, and at least one segment.
/// `SHA256SUMS` and `release-manifest.json` are both read back by a consumer that was not present
/// when they were written (prompt §44, §45), so the check that guards what a bundle *names* cannot
/// live in the code that walks a filesystem — it has to be here, where a test can run it on any host.
#[must_use]
pub fn is_safe_bundle_relative_path(raw: &str) -> bool {
    if raw.is_empty() || raw.starts_with('/') || raw.contains('\\') || raw.contains(':') {
        return false;
    }
    if raw.chars().any(char::is_control) {
        return false;
    }
    raw.split('/').all(|segment| {
        !segment.is_empty() && segment != "." && segment != ".." && !segment.ends_with('.')
    })
}

/// The `SHA256SUMS` and `files[]` order (§21): case-insensitive lexicographic by normalized
/// bundle-relative path, with the exact bytes as tie-break so two paths that differ only in case
/// cannot swap places between runs.
///
/// Sorting is exposed rather than buried in a renderer because the manifest, the sums text and the
/// report all list the same set, and a reader who compares them must see the same order three times.
#[must_use]
pub fn bundle_path_order(left: &str, right: &str) -> std::cmp::Ordering {
    left.to_lowercase()
        .cmp(&right.to_lowercase())
        .then_with(|| left.cmp(right))
}

// --------------------------------------------------------------------------- errors

/// Why a release could not be assembled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseError {
    /// The Gate disposition is not `PASS`. §8: REVIEW and BLOCK both produce no bundle, and an unaccepted
    /// Review or an UNKNOWN-mapped severity reaches this arm rather than a separate one, because the
    /// disposition the aggregate returns is already the answer.
    GateNotReady { disposition: EffectiveSeverity },
    /// No version could be resolved from the context the Gate judged (§11).
    VersionUnavailable { reason: String },
    /// The run id a bundle was asked to bind is not shaped like a Gate run id.
    MalformedGateRunId { value: String },
    /// A release id does not read as `release-<64 hex>`.
    MalformedReleaseId { value: String },
    /// An artifact leaf name carries a separator, a traversal segment or nothing at all.
    MalformedArtifactName { value: String },
    /// Notes named something other than the canonical bundle-relative name.
    MalformedNotesName { value: String },
    /// Nothing was selected to ship. An empty bundle is not a release of an empty firmware.
    NoReleaseArtifacts,
    /// A build was asked to be its own baseline. Core refuses that comparison for the Gate too, and the
    /// stored verdict that records it says so.
    SelfComparison { snapshot_id: String },
    /// A structurally impossible state, reported rather than worked around.
    InternalInvariant { detail: String },
}

impl ReleaseError {
    /// Stable code, continuing the `ERR-BUNDLE-61xx` family §51 opens. Two of the ten codes in that family
    /// are properties of the release *model* and belong here; the rest are about files and destinations and
    /// live with the code that touches them.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::GateNotReady { .. } | Self::NoReleaseArtifacts => "ERR-BUNDLE-6101",
            Self::VersionUnavailable { .. } => "ERR-BUNDLE-6110",
            Self::MalformedGateRunId { .. }
            | Self::MalformedReleaseId { .. }
            | Self::MalformedArtifactName { .. }
            | Self::MalformedNotesName { .. }
            | Self::SelfComparison { .. }
            | Self::InternalInvariant { .. } => "ERR-INTERNAL-9002",
        }
    }
}

impl std::fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GateNotReady { disposition } => write!(
                f,
                "the Gate disposition is {}, and only a PASS disposition may be packaged as a release",
                disposition.as_str()
            ),
            Self::VersionUnavailable { reason } => {
                write!(f, "no project release version could be resolved: {reason}")
            }
            Self::MalformedGateRunId { value } => {
                write!(f, "`{value}` is not shaped like a Gate run id")
            }
            Self::MalformedReleaseId { value } => {
                write!(f, "`{value}` is not shaped like a release id")
            }
            Self::MalformedArtifactName { value } => {
                write!(f, "`{value}` cannot be a bundle-relative artifact name")
            }
            Self::MalformedNotesName { value } => write!(
                f,
                "release notes must be carried as `{RELEASE_NOTES_NAME}`, not `{value}`"
            ),
            Self::NoReleaseArtifacts => write!(f, "no current artifact was selected to ship"),
            Self::SelfComparison { snapshot_id } => write!(
                f,
                "build `{snapshot_id}` cannot be its own baseline; there is no diff of a release against itself to package"
            ),
            Self::InternalInvariant { detail } => write!(f, "release invariant violated: {detail}"),
        }
    }
}

impl std::error::Error for ReleaseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::identity::Sha256;

    const HEX_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HEX_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HEX_GATE: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

    fn digest(hex: &str) -> Sha256 {
        Sha256::parse(hex).expect("a test digest is 64 lowercase hex characters")
    }

    fn artifact(kind: ArtifactKind, hex: &str, name: &str, size: u64) -> ReleaseArtifact {
        ReleaseArtifact {
            kind,
            file_name: name.to_owned(),
            sha256: digest(hex),
            byte_size: size,
        }
    }

    fn elf() -> ReleaseArtifact {
        artifact(ArtifactKind::Elf, HEX_A, "firmware.elf", 4096)
    }

    fn map() -> ReleaseArtifact {
        artifact(ArtifactKind::Map, HEX_B, "firmware.map", 8192)
    }

    /// The facts of a release that passes, before validation.
    fn draft(artifacts: Vec<ReleaseArtifact>) -> ReleaseModel {
        ReleaseModel {
            snapshot_id: "snap-1".to_owned(),
            baseline_snapshot_id: None,
            gate_run_id: format!("gate-{HEX_GATE}"),
            disposition: EffectiveSeverity::Pass,
            version: ReleaseVersion::parse("1.2.3").expect("a plain version parses"),
            version_source: ReleaseVersionSource::GitTagCapture,
            acceptances_sha256: digest(HEX_B),
            artifacts,
            release_notes: None,
            workspace_git: WorkspaceGit::default(),
            schema_majors: SchemaMajors::CURRENT,
            fwsight_version: "0.6.0".to_owned(),
        }
    }

    fn model(artifacts: Vec<ReleaseArtifact>) -> ReleaseModel {
        draft(artifacts)
            .validated()
            .expect("a PASS disposition over one artifact is a release")
    }

    // --------------------------------------------------------------------- identity

    #[test]
    fn the_same_semantic_input_always_produces_the_same_release_text() {
        let first = model(vec![elf()]);
        let second = model(vec![elf()]);
        assert_eq!(
            first.canonical_release_text(),
            second.canonical_release_text()
        );
        assert_eq!(
            ReleaseModel::release_id_from(digest(HEX_A).hex()),
            ReleaseModel::release_id_from(digest(HEX_A).hex())
        );
    }

    #[test]
    fn a_changed_snapshot_changes_the_release() {
        let mut other = model(vec![elf()]);
        other.snapshot_id = "snap-2".to_owned();
        assert_ne!(
            other.canonical_release_text(),
            model(vec![elf()]).canonical_release_text()
        );
    }

    #[test]
    fn a_changed_or_added_baseline_changes_the_release() {
        let without = model(vec![elf()]);
        let mut with_other = model(vec![elf()]);
        with_other.baseline_snapshot_id = Some("snap-0".to_owned());
        assert_ne!(
            without.canonical_release_text(),
            with_other.canonical_release_text()
        );

        let mut with_second_baseline = with_other.clone();
        with_second_baseline.baseline_snapshot_id = Some("snap-0b".to_owned());
        assert_ne!(
            with_other.canonical_release_text(),
            with_second_baseline.canonical_release_text()
        );
    }

    #[test]
    fn a_changed_gate_run_changes_the_release() {
        let mut other = model(vec![elf()]);
        other.gate_run_id = format!("gate-{HEX_A}");
        assert_ne!(
            other.canonical_release_text(),
            model(vec![elf()]).canonical_release_text()
        );
    }

    #[test]
    fn a_changed_version_or_the_source_it_came_from_changes_the_release() {
        let base = model(vec![elf()]);
        let mut other = base.clone();
        other.version = ReleaseVersion::parse("1.2.4").expect("a plain version parses");
        assert_ne!(
            base.canonical_release_text(),
            other.canonical_release_text()
        );

        let mut by_expected = base.clone();
        by_expected.version_source = ReleaseVersionSource::ExpectedVersion;
        assert_ne!(
            base.canonical_release_text(),
            by_expected.canonical_release_text()
        );
    }

    #[test]
    fn a_changed_acceptance_set_changes_the_release() {
        // Review acceptance changes the release id: the digest of the acceptance document is in the input,
        // so two acceptances of the same run over the same bytes are two different releases.
        let mut other = model(vec![elf()]);
        other.acceptances_sha256 = digest(HEX_A);
        assert_ne!(
            other.canonical_release_text(),
            model(vec![elf()]).canonical_release_text()
        );
    }

    #[test]
    fn a_changed_release_notes_digest_changes_the_release_and_absence_is_its_own_value() {
        let without = model(vec![elf()]);
        let with_empty = ReleaseModel {
            release_notes: Some(ReleaseNotesDigest {
                bundle_name: RELEASE_NOTES_NAME.to_owned(),
                sha256: digest(HEX_A),
            }),
            ..without.clone()
        };
        assert_ne!(
            without.canonical_release_text(),
            with_empty.canonical_release_text()
        );

        let with_other_notes = ReleaseModel {
            release_notes: Some(ReleaseNotesDigest {
                bundle_name: RELEASE_NOTES_NAME.to_owned(),
                sha256: digest(HEX_B),
            }),
            ..without
        };
        assert_ne!(
            with_empty.canonical_release_text(),
            with_other_notes.canonical_release_text()
        );
    }

    #[test]
    fn an_output_path_cannot_change_the_release_because_no_field_can_hold_one() {
        // §12 excludes the output directory. The exclusion is structural: the type has no path field, so
        // two models of the same release are equal however differently their bundle lands on disk. What can
        // be checked is that no *value* in the canonical text carries a path shape. The label is
        // `firmwaresight-release-input/1`, so one slash is expected and it is the version marker, exactly
        // as `firmwaresight-gate-input/1` is for a Gate run.
        let text = model(vec![elf(), map()]).canonical_release_text();
        assert_eq!(
            text.lines()
                .next()
                .expect("the text carries its label")
                .matches('/')
                .count(),
            1,
            "the only slash in the release input is the label's version separator"
        );
        for line in text.lines().skip(1) {
            let value = line.split('=').nth(1).unwrap_or_default();
            assert!(
                !value.contains('/') && !value.contains('\\'),
                "a path shape reached a release input value: {line}"
            );
            // A `:` on its own is legitimate - `artifact=elf:<hex>:<size>:<name>` and the schema list use
            // it as a field separator - so a drive prefix is matched as the pair it always appears in.
            assert!(
                !value.contains(":\\") && !value.contains(":/"),
                "a drive prefix reached a release input value: {line}"
            );
        }
    }

    #[test]
    fn no_wall_clock_or_process_identity_releases_into_the_text() {
        let text = model(vec![elf()]).canonical_release_text();
        for line in text.lines() {
            let key = line.split('=').next().unwrap_or(line);
            for forbidden in [
                "created_at",
                "generated_at",
                "imported_at",
                "timestamp",
                "mtime",
                "pid",
                "temp",
                "rowid",
            ] {
                assert_ne!(key, forbidden, "release input must not carry {forbidden}");
            }
            assert!(
                !looks_like_utc_stamp(line.split('=').nth(1).unwrap_or_default()),
                "a clock value reached the release input: {line}"
            );
        }
        // The stored release record's created_at is an audit time and is never in the portable bytes (§42).
    }

    /// The exact shape `04_TECH/15` writes audit times in: `YYYY-MM-DDTHH:MM:SSZ`.
    fn looks_like_utc_stamp(value: &str) -> bool {
        let bytes = value.as_bytes();
        bytes.len() == 20
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b'T'
            && bytes[19] == b'Z'
    }

    #[test]
    fn artifact_ordering_is_canonical_whatever_order_the_caller_handed_over() {
        let one_way = model(vec![map(), elf()]);
        let other_way = model(vec![elf(), map()]);
        assert_eq!(
            one_way.canonical_release_text(),
            other_way.canonical_release_text()
        );
        // ELF sorts before MAP by kind word, and the model exposes that order rather than the caller's.
        let words: Vec<&str> = one_way
            .artifacts
            .iter()
            .map(ReleaseArtifact::kind_word_for)
            .collect();
        assert_eq!(words, ["elf", "map"]);
    }

    #[test]
    fn a_release_id_is_prefixed_and_only_a_digest_tail_is_accepted() {
        let id = ReleaseModel::release_id_from(HEX_A);
        let parsed = ReleaseId::parse(&id).expect("a composed release id parses");
        assert_eq!(parsed.as_str(), id);
        assert_eq!(parsed.short_hex(), &HEX_A[..12]);

        for bad in [
            String::from("1.2.3"),
            format!("gate-{HEX_GATE}"),
            format!("release-{}", "a".repeat(63)),
            format!("release-{}", "A".repeat(64)),
            format!("release-{}z", "a".repeat(63)),
        ] {
            assert!(
                ReleaseId::parse(&bad).is_err(),
                "`{bad}` is not a release id"
            );
        }
    }

    #[test]
    fn the_current_schema_majors_are_recorded_in_the_input() {
        let text = model(vec![elf()]).canonical_release_text();
        assert!(
            text.contains(
                "schemas=analysis:1,diff:1,gate-results:1,accepted-reviews:1,release-manifest:1"
            ),
            "{text}"
        );
    }

    // --------------------------------------------------------------------- what ships

    #[test]
    fn the_current_artifact_list_is_ordered_and_the_primary_is_the_elf() {
        let with_map = model(vec![map(), elf()]);
        assert_eq!(with_map.artifacts.len(), 2);
        assert_eq!(
            with_map.primary_artifact().expect("one exists").kind,
            ArtifactKind::Elf
        );

        // A build with no ELF falls back to the first row rather than claiming nothing ships.
        let map_only = ReleaseModel {
            artifacts: vec![map()],
            ..model(vec![elf()])
        };
        assert_eq!(
            map_only.primary_artifact().expect("one exists").kind,
            ArtifactKind::Map
        );
    }

    #[test]
    fn a_baseline_is_never_a_shipped_artifact() {
        // §23: the baseline exists only to explain the diff. The model carries its snapshot id, and there
        // is no collection of baseline artifacts to ship, so the only way to reach a bundle with baseline
        // bytes in it is to have analyzed them as current.
        let with_baseline = ReleaseModel {
            baseline_snapshot_id: Some("snap-0".to_owned()),
            ..model(vec![elf()])
        };
        assert_eq!(with_baseline.artifacts.len(), 1);
        assert_eq!(
            with_baseline.artifacts[0].sha256.hex(),
            HEX_A,
            "the shipped row is the current build's, not the baseline's"
        );
        assert!(
            with_baseline
                .canonical_release_text()
                .contains("baseline=snap-0\n"),
            "the baseline is named as a comparison, once"
        );
    }

    #[test]
    fn the_model_has_no_field_that_could_carry_a_host_path() {
        // §19 and §52.13. Debug is the shape a test can read without a filesystem: no value may look like
        // a path on either supported platform.
        let rendered = format!("{:?}", model(vec![elf(), map()]));
        assert!(!rendered.contains('\\'), "{rendered}");
        assert!(
            !rendered.contains("C:"),
            "no drive-letter path in a release model"
        );
        assert!(
            !rendered.contains("repo_root"),
            "workspace root is not a release fact"
        );
    }

    #[test]
    fn workspace_git_facts_are_labelled_as_workspace_and_not_as_build_provenance() {
        // §43: never "firmware built from commit X" unless artifact evidence proves it. The type name is
        // where that distinction is kept; the report renders the same words.
        let with_git = ReleaseModel {
            workspace_git: WorkspaceGit {
                head_commit: Some(HEX_A.to_owned()),
                exact_tag: Some("v1.2.3".to_owned()),
                dirty: Some(false),
            },
            ..model(vec![elf()])
        };
        let text = with_git.canonical_release_text();
        assert!(text.contains("workspace.head="), "{text}");
        assert!(
            !text.contains("built_from"),
            "a workspace HEAD is not artifact build provenance"
        );
        assert!(with_git.workspace_git.observed_any());
        assert!(!WorkspaceGit::default().observed_any());
    }

    // --------------------------------------------------------------------- disposition

    #[test]
    fn a_review_disposition_is_refused_and_a_block_disposition_is_not_downgraded() {
        // §8: an unaccepted REVIEW produces no bundle, and a BLOCK produces no bundle. This is the whole
        // product rule of the stage, stated once in the layer that owns the semantics, so neither surface
        // can quietly relax it and neither can read a BLOCK as a warning.
        for refused in [EffectiveSeverity::Review, EffectiveSeverity::Block] {
            let outcome = ReleaseModel {
                disposition: refused,
                ..draft(vec![elf()])
            }
            .validated();
            let error = match outcome {
                Err(error) => error,
                Ok(_) => panic!("a {refused} disposition must not yield a bundle"),
            };
            assert_eq!(
                error,
                ReleaseError::GateNotReady {
                    disposition: refused
                }
            );
            assert_eq!(error.code(), "ERR-BUNDLE-6101");
        }
    }

    #[test]
    fn a_pass_disposition_assembles() {
        // The accepted case, so the two refusals above mean something.
        assert_eq!(
            model(vec![elf()]).disposition,
            EffectiveSeverity::Pass,
            "a PASS disposition over one artifact validates into a release"
        );
    }

    #[test]
    fn the_early_refusal_and_the_model_refusal_are_one_rule() {
        // The bundle engine asks §8 as soon as the aggregate exists, and the model asks it again when it is
        // complete. Two call sites are only honest if they cannot disagree, so they call one function.
        for severity in [
            EffectiveSeverity::Pass,
            EffectiveSeverity::Review,
            EffectiveSeverity::Block,
        ] {
            let outcome = require_packageable(severity);
            assert_eq!(
                outcome.is_ok(),
                severity == EffectiveSeverity::Pass,
                "a {severity} disposition"
            );
            if severity != EffectiveSeverity::Pass {
                assert_eq!(
                    outcome.expect_err("a refused disposition"),
                    ReleaseError::GateNotReady {
                        disposition: severity
                    }
                );
            }
        }
    }

    #[test]
    fn an_unknown_never_reaches_the_release_because_the_severity_vocabulary_has_no_such_member() {
        // ADR-0023 keeps UNKNOWN a finding state and maps it to REVIEW or BLOCK severity. ReleaseModel
        // consumes the severity, so an evidence gap arrives as one of those two and is refused above; that
        // is why "UNKNOWN cannot be accepted as a Review" needs no third arm here.
        assert!(EffectiveSeverity::parse("UNKNOWN").is_none());
        let refused = ReleaseError::GateNotReady {
            disposition: EffectiveSeverity::Review,
        };
        assert!(
            refused.to_string().contains("only a PASS disposition"),
            "{refused}"
        );
    }

    #[test]
    fn a_version_that_cannot_be_resolved_is_reported_with_its_own_code() {
        let too_long = "a".repeat(65);
        for bad in [
            "",
            "  ",
            "1.2.3 ",
            "../1.2.3",
            "1.2.3\\x",
            too_long.as_str(),
        ] {
            let error = ReleaseVersion::parse(bad)
                .expect_err("an unresolvable version is not a release version");
            assert_eq!(error.code(), "ERR-BUNDLE-6110", "`{bad}` was refused");
        }
        assert_eq!(
            ReleaseVersion::parse("1.2.3-rc1")
                .expect("a release version parses")
                .as_str(),
            "1.2.3-rc1"
        );
    }

    #[test]
    fn a_build_cannot_be_its_own_baseline() {
        let outcome = ReleaseModel {
            baseline_snapshot_id: Some("snap-1".to_owned()),
            ..draft(vec![elf()])
        }
        .validated();
        assert!(matches!(outcome, Err(ReleaseError::SelfComparison { .. })));
    }

    #[test]
    fn an_escapable_artifact_name_is_refused_at_validation() {
        // A name reaching this type has already been through sanitize_leaf_name, so reaching it with a
        // separator means a caller bypassed the rule. Refusing here is the second lock, not a duplicate.
        for name in ["", "..", "../evil.elf", "dir\\firmware.elf", "."] {
            let outcome = ReleaseModel {
                artifacts: vec![ReleaseArtifact {
                    kind: ArtifactKind::Elf,
                    file_name: name.to_owned(),
                    sha256: digest(HEX_A),
                    byte_size: 1,
                }],
                ..draft(vec![elf()])
            }
            .validated();
            assert!(
                matches!(outcome, Err(ReleaseError::MalformedArtifactName { .. })),
                "`{name}` was accepted"
            );
        }
    }

    #[test]
    fn notes_must_arrive_under_the_canonical_bundle_name() {
        let outcome = ReleaseModel {
            release_notes: Some(ReleaseNotesDigest {
                bundle_name: "RELEASE_NOTES.md".to_owned(),
                sha256: digest(HEX_A),
            }),
            ..draft(vec![elf()])
        }
        .validated();
        assert!(matches!(
            outcome,
            Err(ReleaseError::MalformedNotesName { .. })
        ));
    }

    #[test]
    fn nothing_selected_means_nothing_ships() {
        let error = draft(Vec::new())
            .validated()
            .expect_err("an empty artifact list is not a release");
        assert_eq!(error, ReleaseError::NoReleaseArtifacts);
        assert_eq!(error.code(), "ERR-BUNDLE-6101");
    }

    #[test]
    fn a_gate_run_id_that_is_not_one_is_refused_before_any_work_is_done() {
        let outcome = ReleaseModel {
            gate_run_id: "latest".to_owned(),
            ..draft(vec![elf()])
        }
        .validated();
        assert!(matches!(
            outcome,
            Err(ReleaseError::MalformedGateRunId { .. })
        ));
    }

    // --------------------------------------------------------------------- names

    #[test]
    fn a_source_name_never_escapes_the_bundle() {
        // Only the final component survives, and the component split happens on both separators, so a
        // traversal written POSIX-style and one written Windows-style lose the same way.
        for (raw, expected) in [
            ("../../Windows/System32/cmd.exe", "cmd.exe"),
            ("/etc/passwd", "passwd"),
            ("C:\\firm\\build.elf", "build.elf"),
            ("dir/../firmware.elf", "firmware.elf"),
            (".", "artifact"),
            ("..", "artifact"),
            ("", "artifact"),
        ] {
            let leaf = sanitize_leaf_name(raw);
            assert_eq!(leaf, expected, "`{raw}` normalizes unexpectedly");
            assert!(
                !leaf.contains('/') && !leaf.contains('\\') && leaf != "..",
                "`{raw}` produced an escapable name `{leaf}`"
            );
        }
    }

    #[test]
    fn a_recognizable_name_survives_untouched() {
        for (raw, expected) in [
            ("firmware.elf", "firmware.elf"),
            ("app-v2.3.1.map", "app-v2.3.1.map"),
            ("motor_controller.bin", "motor_controller.bin"),
            ("RELEASE_NOTES.md", "RELEASE_NOTES.md"),
            // One space is the only thing that moves, and it moves to a separator rather than vanishing.
            ("release notes.md", "release-notes.md"),
        ] {
            assert_eq!(sanitize_leaf_name(raw), expected, "`{raw}` was rewritten");
        }
    }

    #[test]
    fn control_and_trailing_characters_that_platforms_mangle_are_removed() {
        assert_eq!(sanitize_leaf_name("firmware.elf\n"), "firmware.elf");
        assert_eq!(sanitize_leaf_name("firmware.elf."), "firmware.elf");
        assert_eq!(sanitize_leaf_name("firmware.elf "), "firmware.elf");
        assert_eq!(sanitize_leaf_name("..firmware"), "firmware");
        assert_eq!(sanitize_leaf_name("***"), "artifact");
    }

    #[test]
    fn windows_reserved_names_are_never_emitted_as_written() {
        for reserved in ["CON", "nul", "COM1", "LPT9.txt", "aux.map"] {
            let leaf = sanitize_leaf_name(reserved);
            assert!(
                !is_reserved_leaf_name(&leaf),
                "`{reserved}` stayed reserved as `{leaf}`"
            );
            assert!(leaf.starts_with('_'), "`{reserved}` became `{leaf}`");
        }
        assert!(is_reserved_leaf_name("CON.txt"));
        assert!(!is_reserved_leaf_name("CONSOLE.txt"));
    }

    #[test]
    fn an_overlong_leaf_is_truncated_deterministically_and_keeps_its_extension() {
        let long = format!("{}{}", "x".repeat(200), ".elf");
        let leaf = sanitize_leaf_name(&long);
        assert!(leaf.len() <= 96, "{} characters is too many", leaf.len());
        assert!(leaf.ends_with(".elf"), "{leaf}");

        let again = sanitize_leaf_name(&long);
        assert_eq!(leaf, again, "truncation is a rule, not a race");
    }

    #[test]
    fn colliding_leaves_are_disambiguated_by_content_not_by_index() {
        let first = elf();
        let second = ReleaseArtifact {
            sha256: digest(HEX_B),
            ..elf()
        };
        let a = disambiguated_name(first.kind, &first.sha256, "firmware.elf");
        let b = disambiguated_name(second.kind, &second.sha256, "firmware.elf");
        assert_eq!(a, format!("elf-{}-firmware.elf", &HEX_A[..8]));
        assert_ne!(a, b, "two different bytes must not keep one name");
        assert!(!b.contains('/') && !b.contains('\\'));
    }

    #[test]
    fn the_canonical_bundle_names_are_the_ones_the_layout_uses() {
        assert_eq!(RELEASE_NOTES_NAME, "release-notes.md");
        assert_eq!(ARTIFACTS_DIR, "artifacts");
        assert_eq!(RELEASE_ID_PREFIX, "release-");
        assert_eq!(ANALYSIS_DOC_NAME, "analysis.json");
        assert_eq!(DIFF_DOC_NAME, "diff.json");
        assert_eq!(GATE_RESULTS_DOC_NAME, "gate-results.json");
        assert_eq!(ACCEPTED_REVIEWS_DOC_NAME, "accepted-reviews.json");
        assert_eq!(REPORT_DOC_NAME, "release-report.html");
        assert_eq!(SHA256SUMS_NAME, "SHA256SUMS");
        assert_eq!(MANIFEST_DOC_NAME, "release-manifest.json");
        assert_eq!(
            ReleaseId::parse(&ReleaseModel::release_id_from(HEX_A))
                .expect("composed")
                .as_str()
                .len(),
            RELEASE_ID_PREFIX.len() + RELEASE_ID_HEX_LEN
        );
    }

    #[test]
    fn the_two_exclusion_sets_are_the_self_reference_rule_and_nothing_more() {
        // §20 is the whole reason the bundle has no fixed-point problem, so the sets it names are
        // asserted here rather than inferred from a renderer that omitted two files by accident.
        assert_eq!(SUMS_EXCLUDED_NAMES, [SHA256SUMS_NAME, MANIFEST_DOC_NAME]);
        assert_eq!(MANIFEST_EXCLUDED_NAMES, [MANIFEST_DOC_NAME]);
        assert!(
            !SUMS_EXCLUDED_NAMES.contains(&REPORT_DOC_NAME),
            "the report is payload, so the sums must cover it"
        );
        assert!(
            !MANIFEST_EXCLUDED_NAMES.contains(&SHA256SUMS_NAME),
            "the manifest hashes SHA256SUMS, which is what closes the cycle"
        );
    }

    #[test]
    fn a_bundle_relative_path_that_could_escape_or_confuse_a_reader_is_refused() {
        // §44 lists these as verification requirements; they are refused at the naming rule so the
        // generator and the verifier cannot disagree about what a legal entry is.
        for refused in [
            "",
            "/",
            "/absolute/analysis.json",
            "artifacts\\firmware.elf",
            "C:/work/firmware.elf",
            "../release-manifest.json",
            "artifacts/../../outside.elf",
            "./analysis.json",
            "artifacts//firmware.elf",
            "release-notes.",
            "notes\n.json",
        ] {
            assert!(
                !is_safe_bundle_relative_path(refused),
                "`{refused}` must not be accepted as a bundle-relative path"
            );
        }
        for accepted in [
            "analysis.json",
            "artifacts/firmware.elf",
            "artifacts/build.map",
            "release-notes.md",
            "SHA256SUMS",
        ] {
            assert!(
                is_safe_bundle_relative_path(accepted),
                "`{accepted}` is ordinary bundle content"
            );
        }
    }

    #[test]
    fn bundle_paths_sort_case_insensitively_with_an_exact_tie_break() {
        // §21's rule has two halves, and the second one is what makes the order total: `SHA256SUMS` and
        // `sha256sums` differ only in case and must still have a fixed relative order.
        let mut paths = vec![
            "release-report.html",
            "artifacts/firmware.elf",
            "analysis.json",
            "SHA256SUMS",
            "accepted-reviews.json",
        ];
        paths.sort_by(|left, right| bundle_path_order(left, right));
        assert_eq!(
            paths,
            vec![
                "accepted-reviews.json",
                "analysis.json",
                "artifacts/firmware.elf",
                "release-report.html",
                "SHA256SUMS",
            ]
        );
        assert_eq!(bundle_path_order("ABC", "abc"), std::cmp::Ordering::Less);
        assert_eq!(bundle_path_order("abc", "ABC"), std::cmp::Ordering::Greater);
    }
}
