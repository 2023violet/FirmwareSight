//! `firmwaresight.toml` load, validate, canonicalize and save.
//!
//! The rules are prompt §9 and §10; the two that dominate everything here are that a config error is
//! never quietly replaced by a default, and that a save never quietly deletes a key it does not
//! understand.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use firmwaresight_core::domain::gate::{
    GatePolicy, UnknownDisposition, UnknownPolicy, VersionPolicy,
};
use serde::{Deserialize, Serialize};

use crate::error::ProjectError;
use crate::fingerprint;
use crate::version;

/// The only config major version this build reads.
pub const SUPPORTED_SCHEMA_VERSION: i64 = 1;
/// The file name FirmwareSight looks for, and the name a created config is written under.
pub const CONFIG_FILE_NAME: &str = "firmwaresight.toml";
/// The artifact kinds `[artifacts] required` may name. Not analyzing BIN/HEX does not make a
/// configured requirement pass, so the vocabulary is wider than the parser (prompt §19).
pub const REQUIRED_ARTIFACT_VOCABULARY: [&str; 4] = ["elf", "map", "bin", "hex"];

/// A loaded, validated project config plus everything a caller needs to not re-derive it.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedProject {
    /// The config file itself. Never surfaced to a UI or a portable document (`AGENTS.md` 7).
    pub path: PathBuf,
    /// Its parent directory: the project root every relative path is resolved against.
    pub root: PathBuf,
    pub config: ProjectConfig,
    /// Non-fatal findings, including unknown keys. A warning never blocks a Gate.
    pub warnings: Vec<String>,
    /// Unknown keys in sorted order, so a save can refuse deterministically and a UI can name them.
    pub unknown_keys: Vec<String>,
    /// The semantic policy Core evaluates, and its fingerprint.
    pub policy: GatePolicy,
    pub policy_sha256: String,
}

impl LoadedProject {
    /// Read and validate `<dir>/firmwaresight.toml`.
    pub fn load(dir: &Path) -> Result<Self, ProjectError> {
        Self::load_file(&dir.join(CONFIG_FILE_NAME))
    }

    /// Read and validate one specific config file; its parent directory is the project root.
    pub fn load_file(path: &Path) -> Result<Self, ProjectError> {
        let bytes = std::fs::read(path).map_err(|source| ProjectError::ConfigUnreadable {
            path: path.to_path_buf(),
            source,
        })?;
        let text = String::from_utf8(bytes).map_err(|_| ProjectError::ConfigMalformed {
            path: path.to_path_buf(),
            message: "the file is not valid UTF-8".to_owned(),
        })?;
        Self::parse(path, &text)
    }

    /// Parse a config the caller already read. Public because the desktop loads through a native dialog
    /// and the CLI through a project directory, and neither may re-implement validation.
    pub fn parse(path: &Path, text: &str) -> Result<Self, ProjectError> {
        let table: toml::Table =
            text.parse::<toml::Table>()
                .map_err(|error| ProjectError::ConfigMalformed {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                })?;

        let found = match table.get("schema_version") {
            None => {
                return Err(ProjectError::ConfigValueInvalid {
                    field: "schema_version".to_owned(),
                    detail: "the key is required in major 1".to_owned(),
                });
            }
            Some(toml::Value::Integer(value)) => *value,
            Some(other) => {
                return Err(ProjectError::ConfigValueInvalid {
                    field: "schema_version".to_owned(),
                    detail: format!("expected an integer, found {}", toml_type(other)),
                });
            }
        };
        if found != SUPPORTED_SCHEMA_VERSION {
            return Err(ProjectError::UnsupportedSchemaVersion {
                found,
                expected: SUPPORTED_SCHEMA_VERSION,
            });
        }

        // Unknown keys are warnings in major 1 (`04_TECH/08:52-53`), so they are collected from the raw
        // table instead of being turned into a deserialization failure.
        let unknown_keys = unknown_keys(&table);
        let config: ProjectConfig = toml::Value::Table(table).try_into().map_err(|error| {
            ProjectError::ConfigValueInvalid {
                field: "config".to_owned(),
                detail: error.to_string(),
            }
        })?;

        let warnings = validate(&config)?;
        let policy = to_gate_policy(&config);
        let policy_sha256 = fingerprint::policy_sha256(&policy);
        let root = path
            .parent()
            .map_or_else(|| PathBuf::from("."), |parent| parent.to_path_buf());
        Ok(Self {
            path: path.to_path_buf(),
            root,
            config,
            warnings,
            unknown_keys,
            policy,
            policy_sha256,
        })
    }

    /// Replace the policy fields of this config and write it back atomically.
    ///
    /// A config holding keys this build does not understand is refused rather than rewritten: dropping
    /// them silently would delete data the release owner wrote (prompt §10, §42).
    pub fn save_policy(&self, policy: &GatePolicy) -> Result<(), ProjectError> {
        if !self.unknown_keys.is_empty() {
            return Err(ProjectError::SaveWouldDropKeys {
                path: self.path.clone(),
                keys: self.unknown_keys.join(", "),
            });
        }
        let config = config_from_policy(&self.config, policy);
        validate(&config)?;
        write_config(&self.path, &config)
    }

    /// The project name, which is the only non-policy fact a UI is allowed to see about the config.
    #[must_use]
    pub fn project_name(&self) -> &str {
        &self.config.project.name
    }
}

/// The whole `firmwaresight.toml`, at `schema_version = 1`.
///
/// Every optional field carries `skip_serializing_if = "Option::is_none"`: an unset budget is a key the
/// release owner did not write, not a key holding nothing. `schemas/project-config.schema.json` models
/// it the same way, with no `null` in any type, so a saved config and the published contract cannot
/// disagree about the difference between "absent" and "explicitly nothing".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub schema_version: i64,
    pub project: ProjectSection,
    pub artifacts: ArtifactsSection,
    #[serde(default)]
    pub memory: MemorySection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<VersionSection>,
    #[serde(default)]
    pub release: ReleaseSection,
    #[serde(default)]
    pub diff: DiffSection,
    #[serde(default)]
    pub gate: GateSection,
    #[serde(default)]
    pub sbom: SbomSection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactsSection {
    pub required: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MemorySection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_budget: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram_budget: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionSection {
    pub source: VersionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// MVP has one version source. A config naming anything else is a hard error, not a fallback: the
/// alternative would be evaluating a rule the release owner did not ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionSource {
    GitTag,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReleaseSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_clean_git: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_release_notes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_notes_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DiffSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_growth_review_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram_growth_review_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GateSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unknown_evidence_review_count: Option<u64>,
    #[serde(default)]
    pub on_unknown: OnUnknownSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownWord {
    Review,
    Block,
}

impl UnknownWord {
    #[must_use]
    pub const fn disposition(self) -> UnknownDisposition {
        match self {
            Self::Review => UnknownDisposition::Review,
            Self::Block => UnknownDisposition::Block,
        }
    }
}

/// `[gate.on_unknown]`. Every key is optional, and each defaults to what ADR-0023 and
/// `04_TECH/08:60-67` document for that rule — which is why the struct holds one `Option` per field
/// rather than one default for all seven.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OnUnknownSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_clean: Option<UnknownWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_matches_release: Option<UnknownWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_match: Option<UnknownWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_budget: Option<UnknownWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram_budget: Option<UnknownWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_growth: Option<UnknownWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<UnknownWord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SbomSection {
    #[serde(default)]
    pub enabled: bool,
}

/// Turn the validated config into the semantic policy Core evaluates. This is the only place config
/// text becomes policy, so the CLI and the desktop cannot disagree about what a config means.
#[must_use]
pub fn to_gate_policy(config: &ProjectConfig) -> GatePolicy {
    let defaults = UnknownPolicy::default();
    let on_unknown = &config.gate.on_unknown;
    let disposition = |configured: Option<UnknownWord>, default| {
        configured.map(UnknownWord::disposition).unwrap_or(default)
    };
    GatePolicy {
        require_clean_git: config.release.require_clean_git.unwrap_or(true),
        expected_commit: config.release.expected_commit.clone(),
        version: config.version.as_ref().map(|version| VersionPolicy {
            pattern: version.pattern.clone().unwrap_or_default(),
            expected_version: config.release.expected_version.clone(),
        }),
        required_artifact_kinds: config.artifacts.required.clone(),
        flash_budget: config.memory.flash_budget,
        ram_budget: config.memory.ram_budget,
        flash_growth_review_bytes: config.diff.flash_growth_review_bytes,
        ram_growth_review_bytes: config.diff.ram_growth_review_bytes,
        require_release_notes: config.release.require_release_notes.unwrap_or(true),
        release_notes_path: config
            .release
            .release_notes_path
            .clone()
            .unwrap_or_else(|| "RELEASE_NOTES.md".to_owned()),
        unknown_evidence_review_count: config.gate.unknown_evidence_review_count,
        on_unknown: UnknownPolicy {
            git_clean: disposition(on_unknown.git_clean, defaults.git_clean),
            commit_matches_release: disposition(
                on_unknown.commit_matches_release,
                defaults.commit_matches_release,
            ),
            version_match: disposition(on_unknown.version_match, defaults.version_match),
            flash_budget: disposition(on_unknown.flash_budget, defaults.flash_budget),
            ram_budget: disposition(on_unknown.ram_budget, defaults.ram_budget),
            baseline_growth: disposition(on_unknown.baseline_growth, defaults.baseline_growth),
            release_notes: disposition(on_unknown.release_notes, defaults.release_notes),
        },
    }
}

/// Rebuild a config from an edited policy, keeping the non-policy facts (the project name, and the
/// SBOM switch that P3 records but never acts on).
#[must_use]
pub fn config_from_policy(base: &ProjectConfig, policy: &GatePolicy) -> ProjectConfig {
    let defaults = UnknownPolicy::default();
    // Only a disposition that differs from the documented default is written out: a config that says
    // nothing keeps saying nothing, and reloads to the same policy.
    let word = |disposition: UnknownDisposition, default| -> Option<UnknownWord> {
        if disposition == default {
            return None;
        }
        Some(match disposition {
            UnknownDisposition::Review => UnknownWord::Review,
            UnknownDisposition::Block => UnknownWord::Block,
        })
    };
    ProjectConfig {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        project: base.project.clone(),
        artifacts: ArtifactsSection {
            required: policy.required_artifact_kinds.clone(),
        },
        memory: MemorySection {
            flash_budget: policy.flash_budget,
            ram_budget: policy.ram_budget,
        },
        version: policy.version.as_ref().map(|version| VersionSection {
            source: VersionSource::GitTag,
            pattern: (!version.pattern.is_empty()).then(|| version.pattern.clone()),
        }),
        release: ReleaseSection {
            require_clean_git: Some(policy.require_clean_git),
            require_release_notes: Some(policy.require_release_notes),
            release_notes_path: Some(policy.release_notes_path.clone()),
            expected_commit: policy.expected_commit.clone(),
            expected_version: policy
                .version
                .as_ref()
                .and_then(|version| version.expected_version.clone()),
        },
        diff: DiffSection {
            flash_growth_review_bytes: policy.flash_growth_review_bytes,
            ram_growth_review_bytes: policy.ram_growth_review_bytes,
        },
        gate: GateSection {
            unknown_evidence_review_count: policy.unknown_evidence_review_count,
            on_unknown: OnUnknownSection {
                git_clean: word(policy.on_unknown.git_clean, defaults.git_clean),
                commit_matches_release: word(
                    policy.on_unknown.commit_matches_release,
                    defaults.commit_matches_release,
                ),
                version_match: word(policy.on_unknown.version_match, defaults.version_match),
                flash_budget: word(policy.on_unknown.flash_budget, defaults.flash_budget),
                ram_budget: word(policy.on_unknown.ram_budget, defaults.ram_budget),
                baseline_growth: word(policy.on_unknown.baseline_growth, defaults.baseline_growth),
                release_notes: word(policy.on_unknown.release_notes, defaults.release_notes),
            },
        },
        sbom: base.sbom.clone(),
    }
}

/// A first config for a project that has none yet (prompt §41: the Save dialog creates the file).
#[must_use]
pub fn new_config(name: &str, policy: &GatePolicy) -> ProjectConfig {
    let placeholder = ProjectConfig {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        project: ProjectSection {
            name: name.to_owned(),
        },
        artifacts: ArtifactsSection {
            required: policy.required_artifact_kinds.clone(),
        },
        memory: MemorySection::default(),
        version: None,
        release: ReleaseSection::default(),
        diff: DiffSection::default(),
        gate: GateSection::default(),
        sbom: SbomSection::default(),
    };
    config_from_policy(&placeholder, policy)
}

/// Validate the frozen v1 rules (prompt §10). Returned strings are warnings; a hard rule violation is
/// an `Err`, because a config error must not be quietly overridden by a default.
pub fn validate(config: &ProjectConfig) -> Result<Vec<String>, ProjectError> {
    let mut warnings = Vec::new();

    if config.project.name.trim().is_empty() {
        return Err(ProjectError::ConfigValueInvalid {
            field: "project.name".to_owned(),
            detail: "the project name is empty".to_owned(),
        });
    }

    let outside: Vec<&str> = config
        .artifacts
        .required
        .iter()
        .map(String::as_str)
        .filter(|kind| !REQUIRED_ARTIFACT_VOCABULARY.contains(kind))
        .collect();
    if !outside.is_empty() {
        return Err(ProjectError::ConfigValueInvalid {
            field: "artifacts.required".to_owned(),
            detail: format!(
                "`{}` is not in the v1 vocabulary ({}); a requirement FirmwareSight cannot name would \
                 be evaluated as always-missing",
                outside.join(", "),
                REQUIRED_ARTIFACT_VOCABULARY.join(", ")
            ),
        });
    }
    if !config.artifacts.required.iter().any(|kind| kind == "elf") {
        return Err(ProjectError::ConfigValueInvalid {
            field: "artifacts.required".to_owned(),
            detail:
                "the required artifact list must include `elf`, the build FirmwareSight analyzes"
                    .to_owned(),
        });
    }
    let duplicates = duplicate_words(&config.artifacts.required);
    if !duplicates.is_empty() {
        warnings.push(format!(
            "`artifacts.required` lists {} more than once; the requirement is evaluated once.",
            duplicates.join(", ")
        ));
    }

    if config.memory.flash_budget == Some(0) || config.memory.ram_budget == Some(0) {
        return Err(ProjectError::ConfigValueInvalid {
            field: "memory.flash_budget / memory.ram_budget".to_owned(),
            detail: "a budget must be greater than zero when present; `0` would block every build \
                     including an empty one"
                .to_owned(),
        });
    }

    if let Some(path) = config.release.release_notes_path.as_deref() {
        require_project_relative(path, "release.release_notes_path")?;
    }

    if let Some(commit) = config.release.expected_commit.as_deref() {
        require_git_object_id(commit)?;
    }

    if let Some(section) = config.version.as_ref() {
        if section.source != VersionSource::GitTag {
            return Err(ProjectError::ConfigValueInvalid {
                field: "version.source".to_owned(),
                detail: "MVP supports `git_tag` only".to_owned(),
            });
        }
        let pattern = section.pattern.as_deref().unwrap_or_default();
        if pattern.is_empty() {
            return Err(ProjectError::ConfigValueInvalid {
                field: "version.pattern".to_owned(),
                detail: "a `[version]` policy needs a `pattern` to match the workspace tag against"
                    .to_owned(),
            });
        }
        let compiled = version::compile_pattern(pattern)?;
        let has_capture = compiled
            .capture_names()
            .flatten()
            .any(|name| name == "version");
        if let Some(expected) = config.release.expected_version.as_deref() {
            if expected.trim().is_empty() {
                return Err(ProjectError::ConfigValueInvalid {
                    field: "release.expected_version".to_owned(),
                    detail: "an expected version must be a non-empty string".to_owned(),
                });
            }
            if !has_capture {
                return Err(ProjectError::InvalidVersionPattern {
                    detail:
                        "`release.expected_version` is declared, but `[version] pattern` has no \
                             `(?P<version>...)` capture, so there is nothing to compare it with"
                            .to_owned(),
                });
            }
        }
        if !has_capture {
            warnings.push(
                "`[version] pattern` has no `version` capture, so `release.version_matches_policy` can \
                 only check that the tag matches the pattern."
                    .to_owned(),
            );
        }
    } else if config.release.expected_version.is_some() {
        return Err(ProjectError::ConfigValueInvalid {
            field: "release.expected_version".to_owned(),
            detail: "an expected version needs a `[version]` policy; without one there is no tag pattern \
                     to read a version from"
                .to_owned(),
        });
    }

    if config.sbom.enabled {
        warnings.push(
            "`[sbom] enabled = true` is recorded, but SBOM is not an MVP Gate capability: the Gate \
             evaluates no SBOM rule and claims nothing about one."
                .to_owned(),
        );
    }
    Ok(warnings)
}

/// A configured relative path must resolve inside the project root. Absolute paths and `..` escapes are
/// refused, so a config cannot make the Gate read outside the project (prompt §10, `AGENTS.md` 7).
pub fn require_project_relative(path: &str, field: &str) -> Result<(), ProjectError> {
    let candidate = Path::new(path);
    let reject = |detail: String| {
        Err(ProjectError::UnsafeConfigPath {
            path: path.to_owned(),
            detail: format!("`{field}`: {detail}"),
        })
    };
    if path.is_empty() {
        return reject("the path is empty".to_owned());
    }
    if candidate.is_absolute() || path.starts_with('/') || path.starts_with('\\') {
        return reject("an absolute path cannot be project-relative".to_owned());
    }
    if path.contains(':') {
        return reject("the path carries a drive or scheme prefix".to_owned());
    }
    let mut depth = 0usize;
    for component in candidate.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => {
                let Some(next) = depth.checked_sub(1) else {
                    return reject("`..` escapes the project root".to_owned());
                };
                depth = next;
            }
            other => {
                return reject(format!(
                    "the path carries an unexpected component: {other:?}"
                ));
            }
        }
    }
    if depth == 0 {
        return reject("the path resolves to the project root itself".to_owned());
    }
    Ok(())
}

/// `release.expected_commit` must be a full Git object id: 40 hex (SHA-1) or 64 hex (SHA-256),
/// lowercase. A truncated or non-hex value is a config error, never a silent mismatch.
fn require_git_object_id(commit: &str) -> Result<(), ProjectError> {
    let well_shaped = commit.len() == 40 || commit.len() == 64;
    let hex = commit
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
    if well_shaped && hex {
        Ok(())
    } else {
        Err(ProjectError::ConfigValueInvalid {
            field: "release.expected_commit".to_owned(),
            detail: format!(
                "expected a full 40- or 64-character lowercase hex commit id, not `{commit}`"
            ),
        })
    }
}

/// Write a config atomically. Validation has already happened; the bytes go to a sibling temp file
/// first, are read back and compared, and only then swapped in — so a failed save never leaves a
/// zero-byte or half-written config behind (prompt §42).
pub fn write_config(path: &Path, config: &ProjectConfig) -> Result<(), ProjectError> {
    let text =
        toml::to_string_pretty(config).map_err(|error| ProjectError::ConfigValueInvalid {
            field: "config".to_owned(),
            detail: format!("the policy could not be written as TOML: {error}"),
        })?;
    let temp = temp_path(path);
    let staged = std::fs::write(&temp, text.as_bytes())
        .and_then(|()| std::fs::read(&temp))
        .and_then(|read| {
            if read.as_slice() == text.as_bytes() {
                Ok(())
            } else {
                Err(std::io::Error::other(
                    "the temporary config file did not read back byte-identical",
                ))
            }
        });
    if let Err(source) = staged {
        let _ = std::fs::remove_file(&temp);
        return Err(ProjectError::SaveFailed {
            path: path.to_path_buf(),
            source,
        });
    }
    match std::fs::rename(&temp, path) {
        Ok(()) => Ok(()),
        Err(source) => {
            let _ = std::fs::remove_file(&temp);
            Err(ProjectError::SaveFailed {
                path: path.to_path_buf(),
                source,
            })
        }
    }
}

/// A sibling name that cannot collide with a real config and lives in the same directory, so the swap
/// never crosses a filesystem boundary.
fn temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| CONFIG_FILE_NAME.to_owned());
    let temp = format!("{file_name}.firmwaresight-{}.tmp", std::process::id());
    path.with_file_name(temp)
}

/// The keys this build does not recognize, as dotted paths, sorted. Collected from the raw table
/// because major 1 treats them as warnings — and because a save has to know it cannot preserve them.
fn unknown_keys(table: &toml::Table) -> Vec<String> {
    let mut found = BTreeSet::new();
    let top_level = [
        "schema_version",
        "project",
        "artifacts",
        "memory",
        "version",
        "release",
        "diff",
        "gate",
        "sbom",
    ];
    for key in table.keys() {
        if !top_level.contains(&key.as_str()) {
            found.insert(key.clone());
        }
    }
    for (key, value) in table {
        let allowed: &[&str] = match key.as_str() {
            "project" => &["name"],
            "artifacts" => &["required"],
            "memory" => &["flash_budget", "ram_budget"],
            "version" => &["source", "pattern"],
            "release" => &[
                "require_clean_git",
                "require_release_notes",
                "release_notes_path",
                "expected_commit",
                "expected_version",
            ],
            "diff" => &["flash_growth_review_bytes", "ram_growth_review_bytes"],
            "gate" => &["unknown_evidence_review_count", "on_unknown"],
            "sbom" => &["enabled"],
            _ => &[],
        };
        // A table this build does not know is reported once, by its own name; descending into it would
        // report every key inside it as though the loader had understood the parent.
        if allowed.is_empty() {
            continue;
        }
        let Some(inner) = value.as_table() else {
            continue;
        };
        for name in inner.keys() {
            if !allowed.contains(&name.as_str()) {
                found.insert(format!("{key}.{name}"));
            }
        }
        if key == "gate"
            && let Some(on_unknown) = inner.get("on_unknown").and_then(toml::Value::as_table)
        {
            for name in on_unknown.keys() {
                if !ON_UNKNOWN_KEYS.contains(&name.as_str()) {
                    found.insert(format!("gate.on_unknown.{name}"));
                }
            }
        }
    }
    found.into_iter().collect()
}

const ON_UNKNOWN_KEYS: [&str; 7] = [
    "git_clean",
    "commit_matches_release",
    "version_match",
    "flash_budget",
    "ram_budget",
    "baseline_growth",
    "release_notes",
];

fn duplicate_words(values: &[String]) -> Vec<&str> {
    let mut seen = BTreeSet::new();
    let mut duplicates = Vec::new();
    for value in values {
        if !seen.insert(value.as_str()) && !duplicates.contains(&value.as_str()) {
            duplicates.push(value.as_str());
        }
    }
    duplicates
}

fn toml_type(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::String(_) => "a string",
        toml::Value::Integer(_) => "an integer",
        toml::Value::Float(_) => "a float",
        toml::Value::Boolean(_) => "a boolean",
        toml::Value::Datetime(_) => "a date",
        toml::Value::Array(_) => "an array",
        toml::Value::Table(_) => "a table",
    }
}

#[cfg(test)]
mod tests;
