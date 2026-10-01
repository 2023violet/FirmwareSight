//! The portable `release-manifest:1` document — the index a reader uses to check a bundle.
//!
//! `schemas/release-manifest.schema.json` was frozen before P4 and is a versioned public contract
//! (`04_TECH/26`), so this module adds no field to it and reinterprets none. Everything P4 needs to say
//! beyond `schema_version / generated_by / release / build / files / extensions` goes in `extensions`,
//! which is the open object the published schema reserved for exactly that (prompt §19). The alternative
//! — a v2 — would break every reader that already validated v1 for a benefit the reserved slot already
//! provides.
//!
//! The document is typed, not a `serde_json::Value` (prompt §40): field declaration order *is* emitted
//! key order, there is no map to iterate, and an omission is a compile error rather than a document that
//! silently lacks a section.
//!
//! What this layer refuses is *content*, never bytes: an unsafe bundle path, a duplicate name, a digest
//! that is not lowercase hex, a `files` list that disagrees with the release fingerprint. Those checks
//! belong here because the manifest and the bundle builder must not have two ideas about what a legal
//! entry is — the second idea is how a bundle ends up shipping bytes its own index does not name. The
//! checks that need the filesystem (does this hash match those bytes?) live with the code that reads it.
//!
//! The self-reference rule (§20) is the reason the manifest is the last file written and the only file
//! with no hash of itself anywhere: `SHA256SUMS` covers every payload file except itself and the manifest,
//! and the manifest covers everything except itself. No fake empty or zeroed self-digest exists here.

use serde::Serialize;

use firmwaresight_core::domain::identity::Sha256;
use firmwaresight_core::domain::release::{
    ACCEPTED_REVIEWS_DOC_NAME, ANALYSIS_DOC_NAME, ARTIFACTS_DIR, DIFF_DOC_NAME,
    GATE_RESULTS_DOC_NAME, MANIFEST_DOC_NAME, RELEASE_NOTES_NAME, REPORT_DOC_NAME, ReleaseArtifact,
    ReleaseError, ReleaseId, ReleaseModel, ReleaseVersionSource, SHA256SUMS_NAME,
    SUMS_EXCLUDED_NAMES, bundle_path_order, is_safe_bundle_relative_path,
};

use crate::analysis::ANALYSIS_SCHEMA_VERSION;
use crate::diff::DIFF_SCHEMA_VERSION;
use crate::gate::{ACCEPTED_REVIEWS_SCHEMA_VERSION, GATE_SCHEMA_VERSION};

/// The URN of the contract this document is an instance of.
pub const RELEASE_MANIFEST_SCHEMA_ID: &str = "urn:firmwaresight:schema:release-manifest:1";

/// The major this build emits. v1, unchanged by P4.
pub const RELEASE_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// The generated documents every bundle carries, whatever the project shipped (§7).
const REQUIRED_ENTRIES: [&str; 5] = [
    ANALYSIS_DOC_NAME,
    GATE_RESULTS_DOC_NAME,
    ACCEPTED_REVIEWS_DOC_NAME,
    REPORT_DOC_NAME,
    SHA256SUMS_NAME,
];

/// One release bundle's index.
#[derive(Debug, Clone, Serialize)]
pub struct ReleaseManifestDto {
    pub schema_version: u32,
    pub generated_by: GeneratedByDto,
    pub release: ReleaseIdentityDto,
    pub build: BuildFactsDto,
    pub files: Vec<ManifestFileDto>,
    pub extensions: ManifestExtensionsDto,
}

/// Who wrote this document. Kept separate from `release` because the FirmwareSight build and the
/// firmware release version are two different facts and the report must not blur them (prompt §43).
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedByDto {
    pub product: &'static str,
    pub version: String,
}

/// The release being shipped.
#[derive(Debug, Clone, Serialize)]
pub struct ReleaseIdentityDto {
    /// `release-<64 lowercase hex>`, the digest of the canonical release input.
    pub id: String,
    /// The project release version, resolved from the context the Gate judged (§11).
    pub version: String,
    /// The canonical bundle-relative name when notes ship, `null` when they legitimately do not (§24).
    pub notes: Option<String>,
}

/// Facts about the build the bundle carries.
#[derive(Debug, Clone, Serialize)]
pub struct BuildFactsDto {
    pub snapshot_id: String,
    /// The primary shipped artifact (the ELF, or the only artifact when there is no ELF). The v1 schema
    /// describes one primary per build, which is why a second hash lives in `files` instead.
    pub artifact_sha256: Option<String>,
    /// Workspace Git facts, present only when the revalidated Gate context observed them (§19).
    pub git_commit: Option<String>,
    pub git_tag: Option<String>,
    pub git_dirty: Option<bool>,
}

/// One file inside the bundle.
#[derive(Debug, Clone, Serialize)]
pub struct ManifestFileDto {
    /// Bundle-relative, `/`-separated, never absolute (§21).
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

/// The stable facts v1's closed property lists have no room for (§19).
#[derive(Debug, Clone, Serialize)]
pub struct ManifestExtensionsDto {
    pub gate_run_id: String,
    /// `null` when this release has no comparison build.
    pub baseline_snapshot_id: Option<String>,
    pub policy_sha256: String,
    /// How the release version was resolved, in the project's own words: `release.expected_version` or
    /// `git_tag.capture`. A reader needs it to know which evidence class the version belongs to.
    pub release_version_source: &'static str,
    pub analysis_schema_version: u32,
    /// `null` when there is no `diff.json` to read, which is the same fact as
    /// `build.baseline_snapshot_id` being `null` and is checked against it.
    pub diff_schema_version: Option<u32>,
    pub gate_schema_version: u32,
    pub accepted_reviews_schema_version: u32,
    pub integrity_model: IntegrityModelDto,
}

/// The exact §20 coverage rule, written into the bundle it describes.
///
/// A reader who finds `SHA256SUMS` with no line for itself needs an explanation that is not "the tool
/// forgot"; the explanation travels in the document rather than in this repository.
#[derive(Debug, Clone, Serialize)]
pub struct IntegrityModelDto {
    pub scheme: &'static str,
    pub digest: &'static str,
    pub sums_file: &'static str,
    pub manifest_file: &'static str,
    /// How a file hashes itself: it cannot, without a fixed point, and a faked digest would claim
    /// coverage that does not exist.
    pub self_digest_written: bool,
    pub sums_covers: &'static str,
    pub sums_excludes: &'static [&'static str],
    pub manifest_covers: &'static str,
    pub manifest_excludes: &'static [&'static str],
    pub rule: &'static str,
}

impl IntegrityModelDto {
    /// The one model P4 emits, spelled from the same constants the writer uses.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            scheme: "sha256-two-layer/1",
            digest: "SHA-256",
            sums_file: SHA256SUMS_NAME,
            manifest_file: MANIFEST_DOC_NAME,
            self_digest_written: false,
            sums_covers: "every bundle file except itself and the manifest",
            sums_excludes: &SUMS_EXCLUDED_NAMES,
            manifest_covers: "every bundle file except itself, SHA256SUMS included",
            manifest_excludes: &[MANIFEST_DOC_NAME],
            rule: "no file carries its own digest; SHA256SUMS hashes the payload, \
                   release-manifest.json hashes the payload plus SHA256SUMS",
        }
    }
}

/// How a project release version was resolved, in the project's own words.
///
/// One spelling shared by the manifest's `extensions` block and the report's identity table, so the two
/// cannot say different things about where the version came from.
#[must_use]
pub const fn version_source_word(source: ReleaseVersionSource) -> &'static str {
    match source {
        ReleaseVersionSource::ExpectedVersion => "release.expected_version",
        ReleaseVersionSource::GitTagCapture => "git_tag.capture",
    }
}

impl ReleaseManifestDto {
    /// Project a validated release model plus the bundle's real file list into the portable document.
    ///
    /// `release_id` is the digest the adapter computed over `model.canonical_release_text()`: Core cannot
    /// hash (ADR-0027), so the id arrives from outside and is checked here for the shape the model
    /// promises. `files` is what was actually staged, with the digests of those bytes.
    ///
    /// The two are cross-checked rather than concatenated, because the release id is a promise about
    /// content and the manifest is the record of that content: a bundle that ships a file the fingerprint
    /// never named, or omits one it did, is a different release wearing the same id.
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError`] when the document would assert something the model does not: an unsafe
    /// or duplicated path, a non-hex digest, a missing required document, notes or a `diff.json` that
    /// disagree with the model, or an `artifacts/` subtree that is not the fingerprinted artifact set.
    pub fn from_parts(
        model: &ReleaseModel,
        release_id: &str,
        files: &[ManifestFileDto],
        policy_sha256: &str,
    ) -> Result<Self, ManifestError> {
        let release_id = ReleaseId::parse(release_id)?;
        Sha256::parse(policy_sha256).map_err(|_| ManifestError::MalformedDigest {
            value: "policy_sha256".to_owned(),
        })?;

        let entries = validate_entries(model, files)?;

        Ok(Self {
            schema_version: RELEASE_MANIFEST_SCHEMA_VERSION,
            generated_by: GeneratedByDto {
                product: "FirmwareSight",
                version: model.fwsight_version.clone(),
            },
            release: ReleaseIdentityDto {
                id: release_id.as_str().to_owned(),
                version: model.version.as_str().to_owned(),
                notes: model
                    .release_notes
                    .as_ref()
                    .map(|notes| notes.bundle_name.clone()),
            },
            build: BuildFactsDto {
                snapshot_id: model.snapshot_id.clone(),
                artifact_sha256: model
                    .primary_artifact()
                    .map(|artifact| artifact.sha256.hex().to_owned()),
                git_commit: model.workspace_git.head_commit.clone(),
                git_tag: model.workspace_git.exact_tag.clone(),
                git_dirty: model.workspace_git.dirty,
            },
            files: entries,
            extensions: ManifestExtensionsDto {
                gate_run_id: model.gate_run_id.clone(),
                baseline_snapshot_id: model.baseline_snapshot_id.clone(),
                policy_sha256: policy_sha256.to_owned(),
                release_version_source: version_source_word(model.version_source),
                analysis_schema_version: ANALYSIS_SCHEMA_VERSION,
                diff_schema_version: model
                    .baseline_snapshot_id
                    .as_ref()
                    .map(|_| DIFF_SCHEMA_VERSION),
                gate_schema_version: GATE_SCHEMA_VERSION,
                accepted_reviews_schema_version: ACCEPTED_REVIEWS_SCHEMA_VERSION,
                integrity_model: IntegrityModelDto::standard(),
            },
        })
    }
}

impl ManifestFileDto {
    /// An entry for a staged file. The digest must be of the bytes as written, which is why this is the
    /// only constructor and it takes the hash rather than computing one.
    #[must_use]
    pub fn new(path: impl Into<String>, sha256: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            sha256: sha256.into(),
            size,
        }
    }

    /// The entry a shipped artifact is expected to have in `files`, so the fingerprint and the index are
    /// compared on one spelling of the path rather than two.
    #[must_use]
    pub fn shipped_artifact(artifact: &ReleaseArtifact) -> Self {
        Self::new(
            format!("{ARTIFACTS_DIR}/{}", artifact.file_name),
            artifact.sha256.hex().to_owned(),
            artifact.byte_size,
        )
    }
}

// --------------------------------------------------------------------------- entry validation

/// Check the list, order it, and return it. Kept separate from `from_parts` so the rejecting tests read
/// as a list of rejections rather than as a document builder with assertions inside.
fn validate_entries(
    model: &ReleaseModel,
    files: &[ManifestFileDto],
) -> Result<Vec<ManifestFileDto>, ManifestError> {
    if files.is_empty() {
        return Err(ManifestError::NoFiles);
    }

    let mut entries = files.to_vec();
    for entry in &entries {
        if !is_safe_bundle_relative_path(&entry.path) {
            return Err(ManifestError::UnsafePath {
                value: entry.path.clone(),
            });
        }
        if entry.path == MANIFEST_DOC_NAME {
            return Err(ManifestError::HashesItself {
                value: entry.path.clone(),
            });
        }
        if entry.sha256.len() != Sha256::HEX_LEN
            || !entry
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            // The path is reported rather than the value: a wrong digest's text is 64 characters of noise,
            // and which file it was attached to is the actionable half.
            return Err(ManifestError::MalformedDigest {
                value: entry.path.clone(),
            });
        }
    }

    entries.sort_by(|left, right| bundle_path_order(&left.path, &right.path));
    for pair in entries.windows(2) {
        // Compared case-insensitively, because a bundle that is legal on a POSIX host and impossible to
        // unpack on a Windows one is not portable (§45): two entries differing only in case name one file
        // to the second reader.
        if pair[0].path.to_lowercase() == pair[1].path.to_lowercase() {
            return Err(ManifestError::DuplicatePath {
                value: pair[1].path.clone(),
            });
        }
    }

    let present: Vec<&str> = entries.iter().map(|entry| entry.path.as_str()).collect();
    for required in REQUIRED_ENTRIES {
        if !present.contains(&required) {
            return Err(ManifestError::MissingRequiredEntry { required });
        }
    }

    // Notes are present in the index if and only if the Gate observed them (§24). Either half of the
    // mismatch is a real failure: a copied note the model never hashed, and a promised note that is not
    // in the bundle.
    if model.release_notes.is_some() != present.contains(&RELEASE_NOTES_NAME) {
        return Err(ManifestError::NotesPresenceDisagrees);
    }

    // §7: `diff.json` exists exactly when there is a baseline to compare against.
    if model.baseline_snapshot_id.is_some() != present.contains(&DIFF_DOC_NAME) {
        return Err(ManifestError::DiffPresenceDisagrees);
    }

    // The `artifacts/` subtree is the fingerprinted set, byte for byte (§23).
    let shipped: Vec<&ManifestFileDto> = entries
        .iter()
        .filter(|entry| entry.path.starts_with(&format!("{ARTIFACTS_DIR}/")))
        .collect();
    let expected: Vec<ManifestFileDto> = model
        .artifacts
        .iter()
        .map(ManifestFileDto::shipped_artifact)
        .collect();
    for want in &expected {
        let found = shipped.iter().find(|entry| entry.path == want.path);
        let Some(found) = found else {
            return Err(ManifestError::ArtifactNotShipped {
                value: want.path.clone(),
            });
        };
        if found.sha256 != want.sha256 || found.size != want.size {
            return Err(ManifestError::ShippedEntryMismatch {
                value: want.path.clone(),
            });
        }
    }
    if shipped.len() != expected.len() {
        return Err(ManifestError::UnfingerprintedEntry {
            value: shipped
                .iter()
                .find(|entry| {
                    !expected
                        .iter()
                        .any(|want| want.path == entry.path && want.sha256 == entry.sha256)
                })
                .map_or_else(
                    || "an unmatched artifacts/ entry".to_owned(),
                    |entry| entry.path.clone(),
                ),
        });
    }

    Ok(entries)
}

/// Why a manifest could not be written.
///
/// These are refusals about *content*. The `ERR-BUNDLE-61xx` family (§51) is about destinations, sources
/// and writes, and belongs to the layer that touches the filesystem; a caller that is staging a bundle
/// maps whichever of these it hits onto `ERR-BUNDLE-6109 VERIFY_FAILED` with this message as the detail.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ManifestError {
    #[error(transparent)]
    Release(#[from] ReleaseError),
    #[error(
        "`{value}` is not a safe bundle-relative path: no absolute form, no backslash, no `..`, no empty segment"
    )]
    UnsafePath { value: String },
    #[error("`{value}` is not a 64-character lowercase SHA-256 digest")]
    MalformedDigest { value: String },
    #[error("two entries name `{value}`, and a manifest cannot hash one file twice")]
    DuplicatePath { value: String },
    #[error("`{value}` is the manifest's own name, and no file carries its own digest")]
    HashesItself { value: String },
    #[error("the manifest lists no files at all")]
    NoFiles,
    #[error("the bundle is missing `{required}`, which every release bundle carries")]
    MissingRequiredEntry { required: &'static str },
    #[error(
        "release notes are in the bundle index but were not observed by the Gate, or the reverse"
    )]
    NotesPresenceDisagrees,
    #[error("`diff.json` is in the bundle index but this release has no baseline, or the reverse")]
    DiffPresenceDisagrees,
    #[error("`{value}` is in the release fingerprint but is not shipped under that path")]
    ArtifactNotShipped { value: String },
    #[error(
        "the bundle ships `{value}` with a digest or size the release fingerprint does not name"
    )]
    ShippedEntryMismatch { value: String },
    #[error("`{value}` sits in `artifacts/` but is not part of the release fingerprint")]
    UnfingerprintedEntry { value: String },
}
