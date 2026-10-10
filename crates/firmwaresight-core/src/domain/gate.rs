//! Release Gate domain: five factual states, a policy-driven disposition for missing evidence,
//! and the aggregate a release owner acts on.
//!
//! ADR-0023 and `04_TECH/27_GATE_STATE_SEMANTICS.md` fix the vocabulary. `04_TECH/08_CONFIG_SPEC.md:60-69`
//! fixes the rule that a rule may never answer "PASS" because a fact was missing.
//!
//! Core decides what facts *mean* and nothing else. It does not read `firmwaresight.toml`, does not run
//! Git, does not open a database and does not hash a fingerprint, because `AGENTS.md` 3 keeps this crate
//! headless and dependency-free. The facts arrive in [`GateContext`], assembled by
//! `firmwaresight-project` (ADR-0027), and the run id arrives as an argument to
//! [`GateContext::evaluate`] for the same reason: the digest is a hashing concern, the semantics are not.

use std::fmt;

use crate::domain::diff::{ByteChange, Comparability};
use crate::domain::identity::{ArtifactKind, Fact};

/// The factual state of one rule evaluation. Five members, frozen by ADR-0023: an answer, an answer
/// that needs a human, a blocking answer, "not determinable from the evidence", and "does not apply" —
/// the last two of which are not the same thing and never merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FindingState {
    /// The rule evaluated deterministically and is satisfied.
    Pass,
    /// The rule evaluated deterministically, but a policy or a person has to dispose of it.
    Review,
    /// The rule evaluated deterministically and failed in a blocking way.
    Block,
    /// Evidence is missing or unverifiable, so the rule could not be evaluated.
    Unknown,
    /// The rule does not apply to this configuration or this build.
    NotApplicable,
}

impl FindingState {
    /// The portable spelling from `schemas/gate-results.schema.json`, which is a compatibility promise.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Review => "REVIEW",
            Self::Block => "BLOCK",
            Self::Unknown => "UNKNOWN",
            Self::NotApplicable => "N/A",
        }
    }

    /// Reads back [`Self::as_str`]. Returns `None` instead of guessing: a stored row that spells the
    /// state neither way is a corruption the reader must surface, and mapping it to `UNKNOWN` would
    /// invent the very state ADR-0023 says is only for missing evidence.
    #[must_use]
    pub fn parse(stored: &str) -> Option<Self> {
        match stored {
            "PASS" => Some(Self::Pass),
            "REVIEW" => Some(Self::Review),
            "BLOCK" => Some(Self::Block),
            "UNKNOWN" => Some(Self::Unknown),
            "N/A" => Some(Self::NotApplicable),
            _ => None,
        }
    }
}

impl fmt::Display for FindingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How much a finding escalates the release — a *separate* field from the factual state (ADR-0023),
/// so `UNKNOWN` keeps saying "evidence is missing" while `on_unknown` decides whether that blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EffectiveSeverity {
    /// Satisfied, or neutral. `N/A` findings carry this value because the severity vocabulary has
    /// exactly three members and `schemas/gate-results.schema.json` requires one of them; the factual
    /// state stays `N/A` and the UI shows both.
    Pass,
    Review,
    Block,
}

impl EffectiveSeverity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Review => "REVIEW",
            Self::Block => "BLOCK",
        }
    }

    /// Reads back [`Self::as_str`], `None` for anything else. `UNKNOWN` is deliberately not accepted:
    /// the factual state carries that word, and a severity that could be either would let a reader
    /// confuse the two columns ADR-0023 keeps apart.
    #[must_use]
    pub fn parse(stored: &str) -> Option<Self> {
        match stored {
            "PASS" => Some(Self::Pass),
            "REVIEW" => Some(Self::Review),
            "BLOCK" => Some(Self::Block),
            _ => None,
        }
    }
}

impl fmt::Display for EffectiveSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a rule does when its evidence is missing (`04_TECH/08:67`). Two members, because "silently
/// pass" is not one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnknownDisposition {
    Review,
    Block,
}

impl UnknownDisposition {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Block => "block",
        }
    }

    #[must_use]
    pub const fn severity(self) -> EffectiveSeverity {
        match self {
            Self::Review => EffectiveSeverity::Review,
            Self::Block => EffectiveSeverity::Block,
        }
    }
}

/// The ten frozen MVP rule identities. Display text is presentation and may change; these ids are what
/// storage, portable output and review acceptances point at (prompt §15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GateRuleId {
    GitClean,
    CommitMatchesExpected,
    VersionMatchesPolicy,
    RequiredArtifacts,
    ArtifactHashes,
    FlashBudget,
    RamBudget,
    BaselineGrowth,
    ReleaseNotes,
    UnknownEvidenceReview,
}

impl GateRuleId {
    /// Every rule in canonical evaluation order, which is the order findings are recorded in.
    pub const ALL: [Self; 10] = [
        Self::GitClean,
        Self::CommitMatchesExpected,
        Self::VersionMatchesPolicy,
        Self::RequiredArtifacts,
        Self::ArtifactHashes,
        Self::FlashBudget,
        Self::RamBudget,
        Self::BaselineGrowth,
        Self::ReleaseNotes,
        Self::UnknownEvidenceReview,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GitClean => "git.clean",
            Self::CommitMatchesExpected => "release.commit_matches_expected",
            Self::VersionMatchesPolicy => "release.version_matches_policy",
            Self::RequiredArtifacts => "artifacts.required",
            Self::ArtifactHashes => "artifacts.hashes",
            Self::FlashBudget => "memory.flash_budget",
            Self::RamBudget => "memory.ram_budget",
            Self::BaselineGrowth => "diff.growth",
            Self::ReleaseNotes => "release.notes",
            Self::UnknownEvidenceReview => "evidence.unknown_review",
        }
    }

    /// Reads back [`Self::as_str`]. `None` means "a rule this build does not know", which is how a row
    /// written by a newer FirmwareSight stays readable instead of failing the whole query.
    #[must_use]
    pub fn parse(stored: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|rule| rule.as_str() == stored)
    }
}

impl fmt::Display for GateRuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The `[gate.on_unknown]` table: one disposition per rule that can lose its evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPolicy {
    pub git_clean: UnknownDisposition,
    pub commit_matches_release: UnknownDisposition,
    pub version_match: UnknownDisposition,
    pub flash_budget: UnknownDisposition,
    pub ram_budget: UnknownDisposition,
    pub baseline_growth: UnknownDisposition,
    pub release_notes: UnknownDisposition,
}

impl Default for UnknownPolicy {
    /// The shape `04_TECH/08_CONFIG_SPEC.md` documents and `04_TECH/08:60-62` requires: a budget with
    /// missing evidence blocks, everything else asks a human.
    fn default() -> Self {
        Self {
            git_clean: UnknownDisposition::Review,
            commit_matches_release: UnknownDisposition::Review,
            version_match: UnknownDisposition::Review,
            flash_budget: UnknownDisposition::Block,
            ram_budget: UnknownDisposition::Block,
            baseline_growth: UnknownDisposition::Review,
            release_notes: UnknownDisposition::Review,
        }
    }
}

/// `[version]` as the Gate needs it. The pattern is *matched by the project adapter*, which owns the
/// regex crate, so this crate stays dependency-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionPolicy {
    /// The declared pattern text, carried for evidence refs and for the UI to show what was applied.
    pub pattern: String,
    pub expected_version: Option<String>,
}

/// The Gate-relevant semantic values of `firmwaresight.toml`. Comments, key order and formatting are
/// not represented here, which is why [`canonical_policy_text`] can be hashed to a fingerprint that
/// survives a reformat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatePolicy {
    pub require_clean_git: bool,
    pub expected_commit: Option<String>,
    /// `None` when no `[version]` policy is configured, so the rule is `N/A` rather than failing.
    pub version: Option<VersionPolicy>,
    /// The v1 requirement vocabulary: `elf`, `map`, `bin`, `hex`.
    pub required_artifact_kinds: Vec<String>,
    pub flash_budget: Option<u64>,
    pub ram_budget: Option<u64>,
    pub flash_growth_review_bytes: Option<u64>,
    pub ram_growth_review_bytes: Option<u64>,
    pub require_release_notes: bool,
    /// Path as declared; the observation of the file lives in [`GateContext::release_notes`].
    pub release_notes_path: String,
    pub unknown_evidence_review_count: Option<u64>,
    pub on_unknown: UnknownPolicy,
}

impl Default for GatePolicy {
    fn default() -> Self {
        Self {
            require_clean_git: true,
            expected_commit: None,
            version: None,
            required_artifact_kinds: vec!["elf".to_owned()],
            flash_budget: None,
            ram_budget: None,
            flash_growth_review_bytes: None,
            ram_growth_review_bytes: None,
            require_release_notes: true,
            release_notes_path: "RELEASE_NOTES.md".to_owned(),
            unknown_evidence_review_count: None,
            on_unknown: UnknownPolicy::default(),
        }
    }
}

/// Workspace Git facts, deliberately without a repository root: the root is a host path, and a host
/// path must not reach a Gate run record, a portable document, a UI surface or a run fingerprint
/// (`AGENTS.md` 7, prompt §28).
///
/// `available` is the difference between "Git answered" and "Git could be consulted at all". A known
/// HEAD with an unknown tag means the commit carries no tag; an unavailable repository means nothing
/// about tags or dirtiness is knowable — those are `BLOCK` and `UNKNOWN` respectively.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateGitFacts {
    pub available: bool,
    pub head_commit: Fact<String>,
    pub exact_tag: Fact<String>,
    pub dirty: Fact<bool>,
}

impl GateGitFacts {
    /// The normalized shape of "no Git": every fact unknown with one reason.
    #[must_use]
    pub fn unavailable(reason: &str) -> Self {
        Self {
            available: false,
            head_commit: Fact::unknown(reason),
            exact_tag: Fact::unknown(reason),
            dirty: Fact::unknown(reason),
        }
    }
}

/// One version-policy outcome, as decided by the adapter that owns regex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateVersionFacts {
    /// Whether the workspace tag satisfies `[version] pattern`.
    pub pattern_matches: Fact<bool>,
    /// The `version` named capture, when the pattern produced one.
    pub captured_version: Fact<String>,
}

/// How much of one memory budget is actually known, and how strongly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateBudgetFact {
    /// The accounted total, if any. A partial total is still a floor, and `exact` says which.
    pub bytes: Option<u64>,
    /// Every allocatable section attributed.
    pub exact: bool,
    /// Whether the accounting basis may support a hard verdict at all
    /// (`MemoryFootprint::admissible_for_hard_block`). Weak evidence never blocks.
    pub admissible: bool,
    pub basis: Option<String>,
    pub reason: Option<String>,
    /// A stable `evidence:ev-*` locator for the accounting that produced the figure.
    pub evidence_ref: Option<String>,
}

impl GateBudgetFact {
    /// The shape of "the snapshot supports no figure for this side".
    #[must_use]
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self {
            bytes: None,
            exact: false,
            admissible: false,
            basis: None,
            reason: Some(reason.into()),
            evidence_ref: None,
        }
    }

    /// An exact, admissible total — the only shape allowed to block.
    #[must_use]
    pub fn exact(bytes: u64, basis: impl Into<String>, evidence_ref: impl Into<String>) -> Self {
        Self {
            bytes: Some(bytes),
            exact: true,
            admissible: true,
            basis: Some(basis.into()),
            reason: None,
            evidence_ref: Some(evidence_ref.into()),
        }
    }
}

/// The two budget figures a snapshot supports, if it supports any.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GateMemoryFacts {
    pub nonvolatile: Option<GateBudgetFact>,
    pub runtime_ram: Option<GateBudgetFact>,
}

/// One release-notes file, as observed relative to the project root. No absolute path exists in this
/// type, so none can leak into a ref.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFileFact {
    pub relative_path: String,
    pub status: GateFileStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateFileStatus {
    /// The file is there; `sha256` is its digest when the read succeeded.
    Present { sha256: Option<String> },
    /// The path resolved inside the project root and nothing is there.
    Missing,
    /// The file is there but could not be stat'd or read — an evidence failure, not an absence.
    Unreadable { reason: String },
}

/// Growth against a baseline, reading P2's Core diff rather than computing a delta here (prompt §23).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateGrowthFacts {
    /// `None` when the Gate ran without a baseline snapshot.
    pub baseline_snapshot_id: Option<String>,
    /// The nonvolatile delta from `diff::MemoryDiff::nonvolatile`.
    pub nonvolatile: Option<ByteChange>,
    /// The runtime RAM delta from `diff::MemoryDiff::runtime_ram`.
    pub runtime_ram: Option<ByteChange>,
}

impl GateGrowthFacts {
    #[must_use]
    pub fn without_baseline() -> Self {
        Self {
            baseline_snapshot_id: None,
            nonvolatile: None,
            runtime_ram: None,
        }
    }
}

/// One artifact the snapshot holds, as the Gate needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateArtifactFact {
    pub kind: ArtifactKind,
    /// The snapshot's recorded digest. `Unknown` means the snapshot holds no valid digest, which the
    /// hashes rule blocks on — the Gate never re-reads the original file to recompute it (prompt §20).
    pub sha256: Fact<String>,
    pub byte_size: u64,
}

/// How an attachment came to be read as the kind it carries (`ADR-0030` D-8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindBasis {
    /// The release owner chose the kind when selecting the file. This is the only basis an observation
    /// can produce for an attached file, because nothing reads the file as a container.
    Declared,
    /// The kind came from the bytes at the start of the file, with what was seen named. Reserved for a
    /// path that does not exist yet: an attached file's structure is never analyzed, so a derived kind
    /// may describe a shape and may not state validity, records or an address span.
    DerivedFromLeadingBytes(String),
}

impl KindBasis {
    /// The stable word migration `0006` stores. The derived variant's detail is not a column, which is
    /// why [`Self::from_word`] refuses to invent one.
    #[must_use]
    pub fn word(&self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::DerivedFromLeadingBytes(_) => "derived_from_leading_bytes",
        }
    }

    /// The basis a stored word names, or `None` when the word carries a detail this schema does not
    /// persist. A reconstruction never fabricates a byte sample it does not have.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "declared" => Some(Self::Declared),
            _ => None,
        }
    }
}

/// One file the release owner attached to the release, as the Gate needs it: raw bytes, their length and
/// their digest, plus the basis of the kind claimed for them. It carries no section, symbol, memory,
/// entry point or object claim, because nothing read the file as a container (`ADR-0030` D-1, D-8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateAttachmentFact {
    /// `Bin`, `IntelHex` or `Unknown`. `Elf` and `Map` are refused before a fact exists: they come from
    /// the analysis path, and letting an unanalyzed ELF stand in for the ELF a release is about would
    /// decouple the verdict from the build it claims to gate (`04_TECH/28` 2.2).
    pub kind: ArtifactKind,
    /// Observed: the digest of the bytes actually read. An attachment that could not be hashed never
    /// reaches a context, so a canonical block has no `unknown:` row.
    pub sha256: Fact<String>,
    /// Observed: the length of those same bytes. Stored and re-checked at packaging, never hashed into
    /// the identity (`04_TECH/28` §5 rule 1).
    pub byte_size: u64,
    pub kind_basis: KindBasis,
}

impl GateAttachmentFact {
    /// The identity's own ordering of a set of attachment rows: sorted by `(kind word, digest)`, with an
    /// exact `(kind, digest)` pair kept once (`04_TECH/28` §5 rules 1 and 2).
    ///
    /// Exposed as a function over rows rather than only as a method on a context, because storage has to
    /// check the same rule against a draft it never assembled into a context — and a second, slightly
    /// different copy of that rule is how a stored ordinal stops matching the text it came from.
    #[must_use]
    pub fn canonicalized(rows: &[Self]) -> Vec<Self> {
        let mut sorted = rows.to_vec();
        sorted.sort_by(|left, right| {
            artifact_kind_label(left.kind)
                .cmp(artifact_kind_label(right.kind))
                .then_with(|| fact_text(&left.sha256).cmp(&fact_text(&right.sha256)))
        });
        sorted.dedup_by(|left, right| left.kind == right.kind && left.sha256 == right.sha256);
        sorted
    }
}

/// The count of snapshot evidence items classified `Unknown`, plus a bounded deterministic sample of
/// their locators (prompt §25). The items themselves are never reclassified here.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GateUnknownEvidence {
    pub count: usize,
    pub sample_refs: Vec<String>,
}

/// Everything a Gate evaluation reads. Built by `firmwaresight-project`, fingerprinted there, and
/// evaluated here without mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateContext {
    pub snapshot_id: String,
    pub artifacts: Vec<GateArtifactFact>,
    /// The files the release owner attached, as observed bytes. A separate set from `artifacts` on
    /// purpose: `artifacts` answers what the analyzed snapshot holds, and an attachment answers what the
    /// release ships. Merging them would put attachment prose into snapshot findings, which is the
    /// defect `ADR-0030` D-4 refuses (`04_TECH/28` §2).
    pub attachments: Vec<GateAttachmentFact>,
    pub memory: Option<GateMemoryFacts>,
    pub git: GateGitFacts,
    /// `None` when no `[version]` policy is configured, or when no tag needs matching.
    pub version: Option<GateVersionFacts>,
    pub release_notes: Option<GateFileFact>,
    pub growth: GateGrowthFacts,
    pub unknown_evidence: GateUnknownEvidence,
    pub policy: GatePolicy,
}

/// One rule's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFinding {
    /// Deterministic for a run and a rule: `<run id>#<rule id>` (prompt §15).
    pub id: String,
    pub rule_id: GateRuleId,
    pub state: FindingState,
    pub effective_severity: EffectiveSeverity,
    pub summary: String,
    pub evidence_refs: Vec<String>,
    pub remediation: Option<String>,
}

impl GateFinding {
    /// Whether a release owner may dispose of this finding by accepting it. Only `REVIEW` may be
    /// accepted: accepting an `UNKNOWN` would record a judgement about evidence that never arrived.
    #[must_use]
    pub fn acceptable_as_review(&self) -> bool {
        self.state == FindingState::Review
    }
}

/// How many findings fell into each factual state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GateStateCounts {
    pub pass: usize,
    pub review: usize,
    pub block: usize,
    pub unknown: usize,
    pub not_applicable: usize,
}

impl GateStateCounts {
    #[must_use]
    pub fn of(findings: &[GateFinding]) -> Self {
        let mut counts = Self::default();
        for finding in findings {
            match finding.state {
                FindingState::Pass => counts.pass += 1,
                FindingState::Review => counts.review += 1,
                FindingState::Block => counts.block += 1,
                FindingState::Unknown => counts.unknown += 1,
                FindingState::NotApplicable => counts.not_applicable += 1,
            }
        }
        counts
    }
}

/// The whole answer for one run: findings in canonical rule order, plus the aggregate computed with no
/// acceptances in play. Accepted reviews are a separate audit fact and never rewrite a finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateEvaluation {
    pub run_id: String,
    pub snapshot_id: String,
    pub baseline_snapshot_id: Option<String>,
    pub findings: Vec<GateFinding>,
    pub overall_effective_severity: EffectiveSeverity,
    pub counts: GateStateCounts,
}

impl GateEvaluation {
    /// The aggregate once the run's accepted reviews are taken into account. A finding keeps its
    /// `REVIEW` state and severity in the immutable record; only the aggregate moves, and only for
    /// findings the owner actually accepted (`04_TECH/27:43-48`).
    #[must_use]
    pub fn aggregate_with_acceptances(&self, accepted_finding_ids: &[String]) -> EffectiveSeverity {
        aggregate(&self.findings, accepted_finding_ids)
    }

    /// The finding for one rule. Every rule always produces exactly one, so this is for callers
    /// holding a rule id from elsewhere, such as a re-read run record.
    #[must_use]
    pub fn finding(&self, rule_id: GateRuleId) -> Option<&GateFinding> {
        self.findings.iter().find(|f| f.rule_id == rule_id)
    }
}

/// Aggregate precedence, frozen by prompt §14: any effective block wins, then any review that is
/// either an unaccepted `REVIEW` finding or an `UNKNOWN` mapped to review, then pass. `N/A` is
/// neutral, and an `UNKNOWN` can never be dispositioned away by an acceptance.
#[must_use]
pub fn aggregate(findings: &[GateFinding], accepted_finding_ids: &[String]) -> EffectiveSeverity {
    let mut reviewed = false;
    for finding in findings {
        match finding.effective_severity {
            EffectiveSeverity::Block => return EffectiveSeverity::Block,
            EffectiveSeverity::Review => {
                let dispositioned = finding.state == FindingState::Review
                    && accepted_finding_ids.contains(&finding.id);
                if !dispositioned {
                    reviewed = true;
                }
            }
            EffectiveSeverity::Pass => {}
        }
    }
    if reviewed {
        EffectiveSeverity::Review
    } else {
        EffectiveSeverity::Pass
    }
}

/// The canonical semantic text of a policy. Two configs that differ only in comments, blank lines or
/// key order produce the same text, and so the same `policy_sha256`; any change that alters an
/// evaluation changes it. The project crate hashes this, because hashing is not Core's job.
#[must_use]
pub fn canonical_policy_text(policy: &GatePolicy) -> String {
    let mut text = String::from("firmwaresight-gate-policy/1\n");
    text.push_str(&format!(
        "release.require_clean_git={}\n",
        policy.require_clean_git
    ));
    text.push_str(&format!(
        "release.expected_commit={}\n",
        canonical_optional(policy.expected_commit.as_deref())
    ));
    match policy.version.as_ref() {
        Some(version) => {
            text.push_str("version.source=git_tag\n");
            text.push_str(&format!("version.pattern={}\n", version.pattern));
            text.push_str(&format!(
                "version.expected={}\n",
                canonical_optional(version.expected_version.as_deref())
            ));
        }
        None => {
            text.push_str("version.source=-\n");
            text.push_str("version.pattern=-\n");
            text.push_str("version.expected=-\n");
        }
    }
    text.push_str(&format!(
        "artifacts.required={}\n",
        canonical_list(&policy.required_artifact_kinds)
    ));
    text.push_str(&format!(
        "memory.flash_budget={}\n",
        canonical_number(policy.flash_budget)
    ));
    text.push_str(&format!(
        "memory.ram_budget={}\n",
        canonical_number(policy.ram_budget)
    ));
    text.push_str(&format!(
        "diff.flash_growth_review_bytes={}\n",
        canonical_number(policy.flash_growth_review_bytes)
    ));
    text.push_str(&format!(
        "diff.ram_growth_review_bytes={}\n",
        canonical_number(policy.ram_growth_review_bytes)
    ));
    text.push_str(&format!(
        "release.require_release_notes={}\n",
        policy.require_release_notes
    ));
    text.push_str(&format!(
        "release.release_notes_path={}\n",
        policy.release_notes_path
    ));
    text.push_str(&format!(
        "gate.unknown_evidence_review_count={}\n",
        canonical_number(policy.unknown_evidence_review_count)
    ));
    text.push_str(&format!(
        "gate.on_unknown.git_clean={}\n",
        policy.on_unknown.git_clean.as_str()
    ));
    text.push_str(&format!(
        "gate.on_unknown.commit_matches_release={}\n",
        policy.on_unknown.commit_matches_release.as_str()
    ));
    text.push_str(&format!(
        "gate.on_unknown.version_match={}\n",
        policy.on_unknown.version_match.as_str()
    ));
    text.push_str(&format!(
        "gate.on_unknown.flash_budget={}\n",
        policy.on_unknown.flash_budget.as_str()
    ));
    text.push_str(&format!(
        "gate.on_unknown.ram_budget={}\n",
        policy.on_unknown.ram_budget.as_str()
    ));
    text.push_str(&format!(
        "gate.on_unknown.baseline_growth={}\n",
        policy.on_unknown.baseline_growth.as_str()
    ));
    text.push_str(&format!(
        "gate.on_unknown.release_notes={}\n",
        policy.on_unknown.release_notes.as_str()
    ));
    text
}

impl GateContext {
    /// Evaluate every MVP rule once, in canonical order. `run_id` comes from the caller because hashing
    /// is the project adapter's job; [`Self::canonical_input`] makes it checkable that the id covers
    /// exactly the facts that decide the verdict.
    #[must_use]
    pub fn evaluate(&self, run_id: &str) -> GateEvaluation {
        let findings: Vec<GateFinding> = GateRuleId::ALL
            .iter()
            .map(|rule| match rule {
                GateRuleId::GitClean => self.rule_git_clean(),
                GateRuleId::CommitMatchesExpected => self.rule_expected_commit(),
                GateRuleId::VersionMatchesPolicy => self.rule_version(),
                GateRuleId::RequiredArtifacts => self.rule_required_artifacts(),
                GateRuleId::ArtifactHashes => self.rule_artifact_hashes(),
                GateRuleId::FlashBudget => self.rule_budget(true),
                GateRuleId::RamBudget => self.rule_budget(false),
                GateRuleId::BaselineGrowth => self.rule_growth(),
                GateRuleId::ReleaseNotes => self.rule_release_notes(),
                GateRuleId::UnknownEvidenceReview => self.rule_unknown_evidence(),
            })
            .map(|finding| finding.into_finding(run_id))
            .collect();
        let counts = GateStateCounts::of(&findings);
        let overall_effective_severity = aggregate(&findings, &[]);
        GateEvaluation {
            run_id: run_id.to_owned(),
            snapshot_id: self.snapshot_id.clone(),
            baseline_snapshot_id: self.growth.baseline_snapshot_id.clone(),
            findings,
            overall_effective_severity,
            counts,
        }
    }

    /// The attachment rows in the order and multiplicity the identity binds: sorted by
    /// `(kind word, digest)`, with an exact `(kind, digest)` pair bound once. `04_TECH/28` §5 rules 1
    /// and 2. Storage writes its `ordinal` from this order, so a re-read restores the same text without
    /// sorting again.
    #[must_use]
    pub fn canonical_attachments(&self) -> Vec<GateAttachmentFact> {
        GateAttachmentFact::canonicalized(&self.attachments)
    }

    /// A canonical, self-labelled rendering of everything that affects the verdict. The same facts
    /// produce the same text and therefore the same run id; a dirty workspace, a policy edit, a
    /// different baseline or different Release Notes bytes change it.
    ///
    /// Absolute paths, host names, import times and process ids are absent from these types by
    /// construction, so they cannot be included by accident.
    #[must_use]
    pub fn canonical_input(&self) -> String {
        let attachments = self.canonical_attachments();
        let label = if attachments.is_empty() {
            "firmwaresight-gate-input/1\n"
        } else {
            "firmwaresight-gate-input/2\n"
        };
        let mut text = String::from(label);
        text.push_str(&format!("snapshot={}\n", self.snapshot_id));
        text.push_str(&format!(
            "baseline={}\n",
            canonical_optional(self.growth.baseline_snapshot_id.as_deref())
        ));
        text.push_str("artifacts[\n");
        for artifact in &self.artifacts {
            text.push_str(&format!(
                "  {} {}\n",
                artifact_kind_label(artifact.kind),
                fact_text(&artifact.sha256)
            ));
        }
        text.push_str("]\n");
        // Emitted only when a row exists: with no attachment the text stays byte-identical to the
        // `/1` a reviewer already accepted, so no historical run id moves.
        if !attachments.is_empty() {
            text.push_str("attachments[\n");
            for row in &attachments {
                text.push_str(&format!(
                    "  {} {}\n",
                    artifact_kind_label(row.kind),
                    fact_text(&row.sha256)
                ));
            }
            text.push_str("]\n");
        }
        text.push_str(&format!("git.available={}\n", self.git.available));
        text.push_str(&format!("git.head={}\n", fact_text(&self.git.head_commit)));
        text.push_str(&format!("git.tag={}\n", fact_text(&self.git.exact_tag)));
        text.push_str(&format!("git.dirty={}\n", fact_text(&self.git.dirty)));
        match &self.version {
            Some(version) => {
                text.push_str(&format!(
                    "version.matches={}\n",
                    fact_text(&version.pattern_matches)
                ));
                text.push_str(&format!(
                    "version.captured={}\n",
                    fact_text(&version.captured_version)
                ));
            }
            None => {
                text.push_str("version.matches=-\n");
                text.push_str("version.captured=-\n");
            }
        }
        text.push_str(&format!(
            "notes={}\n",
            self.release_notes
                .as_ref()
                .map(canonical_file_fact)
                .unwrap_or_else(|| "-".to_owned())
        ));
        text.push_str(&canonical_memory(self.memory.as_ref()));
        text.push_str(&canonical_growth(&self.growth));
        text.push_str(&format!("unknown.count={}\n", self.unknown_evidence.count));
        text.push_str("unknown.refs[\n");
        for reference in &self.unknown_evidence.sample_refs {
            text.push_str(&format!("  {reference}\n"));
        }
        text.push_str("]\n");
        text.push_str(&canonical_policy_text(&self.policy));
        text
    }

    /// `git.clean` — the workspace dirty-state rule. A policy that does not require cleanliness makes
    /// the rule inapplicable rather than silently satisfied.
    fn rule_git_clean(&self) -> RawFinding {
        let rule = GateRuleId::GitClean;
        if !self.policy.require_clean_git {
            return RawFinding::deterministic(
                rule,
                FindingState::NotApplicable,
                "`release.require_clean_git` is false, so a clean workspace is not required.",
                &["policy:release.require_clean_git"],
                None,
            );
        }
        match self.git.dirty.value() {
            Some(false) => RawFinding::deterministic(
                rule,
                FindingState::Pass,
                "Workspace has no uncommitted changes at Gate evaluation time.",
                &["git:status", "git:head"],
                None,
            ),
            Some(true) => RawFinding::deterministic(
                rule,
                FindingState::Block,
                "Workspace has uncommitted changes at Gate evaluation time.",
                &["git:status", "git:head"],
                Some(
                    "Commit or discard the pending changes, then rerun the Gate against the resulting \
                     HEAD.",
                ),
            ),
            None => RawFinding::unknown(
                rule,
                self.policy.on_unknown.git_clean,
                format!(
                    "Workspace dirty state is unavailable: {}.",
                    unknown_reason(&self.git.dirty)
                ),
                &["git:status"],
                Some(
                    "Run the Gate inside a Git working tree the release owner can consult, or set \
                     `release.require_clean_git = false` if a clean tree is not part of this policy.",
                ),
            ),
        }
    }

    /// `release.commit_matches_expected` — workspace provenance against the declared release commit. It
    /// never claims the artifact was built from that commit (prompt §17, §50).
    fn rule_expected_commit(&self) -> RawFinding {
        let rule = GateRuleId::CommitMatchesExpected;
        let policy_ref = "policy:release.expected_commit";
        let Some(expected) = self.policy.expected_commit.as_deref() else {
            return RawFinding::deterministic(
                rule,
                FindingState::NotApplicable,
                "No `release.expected_commit` is declared for this release.",
                &[policy_ref],
                None,
            );
        };
        let refs = [policy_ref, "git:head"];
        match self.git.head_commit.value() {
            Some(head) if head == expected => RawFinding::deterministic(
                rule,
                FindingState::Pass,
                "Workspace HEAD matches the declared release commit.",
                refs.as_slice(),
                None,
            ),
            Some(head) => RawFinding::deterministic(
                rule,
                FindingState::Block,
                format!("Workspace HEAD `{head}` is not the declared release commit `{expected}`."),
                refs.as_slice(),
                Some(
                    "Check out the declared release commit, or update `release.expected_commit` to the \
                     HEAD this release ships from.",
                ),
            ),
            None => RawFinding::unknown(
                rule,
                self.policy.on_unknown.commit_matches_release,
                format!(
                    "Workspace HEAD is unavailable: {}, so the declared release commit cannot be checked.",
                    unknown_reason(&self.git.head_commit)
                ),
                refs.as_slice(),
                Some(
                    "Run the Gate where Git can report HEAD, or remove `release.expected_commit` if this \
                     release does not pin a commit.",
                ),
            ),
        }
    }

    /// `release.version_matches_policy` — the exact tag on workspace HEAD against `[version] pattern`
    /// and, when declared, against `release.expected_version`. No version is ever inferred from the
    /// artifact (prompt §18).
    fn rule_version(&self) -> RawFinding {
        let rule = GateRuleId::VersionMatchesPolicy;
        let Some(policy) = self.policy.version.as_ref() else {
            return RawFinding::deterministic(
                rule,
                FindingState::NotApplicable,
                "No `[version]` policy is configured, so no tag has to match.",
                &["policy:version.source"],
                None,
            );
        };
        let mut refs = vec![
            "policy:version.source".to_owned(),
            // The pattern text is *not* part of this locator. It is the one policy value that is
            // arbitrary user text, and a regex carries `\` — the separator that marks a host path,
            // which no locator may contain (`04_TECH/27`, `AGENTS.md` 7). The pattern the rule was
            // judged with is stated in full in the summary below, and the run fingerprint still
            // covers it, so nothing is hidden: only the pointer changes shape.
            "policy:version.pattern".to_owned(),
        ];
        if !self.git.available {
            refs.push("git:exact-tag".to_owned());
            return RawFinding::unknown(
                rule,
                self.policy.on_unknown.version_match,
                format!(
                    "Git workspace facts are unavailable: {}, so no tag can be checked.",
                    unknown_reason(&self.git.exact_tag)
                ),
                refs.as_slice(),
                Some(
                    "Run the Gate where Git can report the workspace, or drop the `[version]` policy.",
                ),
            );
        }
        let Some(tag) = self.git.exact_tag.value().cloned() else {
            refs.push("git:exact-tag".to_owned());
            return RawFinding::deterministic(
                rule,
                FindingState::Block,
                "No exact tag points at workspace HEAD, so the version policy cannot be met.",
                refs.as_slice(),
                Some(
                    "Tag HEAD for this release with a name that matches `[version] pattern`, then rerun \
                     the Gate.",
                ),
            );
        };
        refs.push(format!("git:exact-tag={tag}"));
        let Some(version_facts) = self.version.as_ref() else {
            refs.push("git:head".to_owned());
            return RawFinding::unknown(
                rule,
                self.policy.on_unknown.version_match,
                format!(
                    "Workspace tag `{tag}` exists but was not matched against `[version] pattern`, \
                     because no match result reached the Gate."
                ),
                refs.as_slice(),
                Some("Rerun the Gate; the version pattern was not evaluated."),
            );
        };
        if version_facts.pattern_matches.value() != Some(&true) {
            return RawFinding::deterministic(
                rule,
                FindingState::Block,
                format!(
                    "Workspace tag `{tag}` does not match the configured version pattern `{}`.",
                    policy.pattern
                ),
                refs.as_slice(),
                Some(
                    "Retag HEAD with a name the `[version] pattern` accepts, or change the pattern to the \
                     naming scheme this project actually releases under.",
                ),
            );
        }
        let Some(expected) = policy.expected_version.as_deref() else {
            return RawFinding::deterministic(
                rule,
                FindingState::Pass,
                format!("Workspace tag `{tag}` matches the configured version pattern."),
                refs.as_slice(),
                None,
            );
        };
        // The declared version is quoted in the summaries below and never inside a locator, for the
        // same reason the pattern is not: it is arbitrary text, and a locator may not carry the
        // separator a host path always has.
        refs.push("policy:release.expected_version".to_owned());
        match version_facts.captured_version.value() {
            Some(captured) if captured == expected => RawFinding::deterministic(
                rule,
                FindingState::Pass,
                format!(
                    "Workspace tag `{tag}` matches the version pattern and its version `{captured}` is \
                     the declared release version."
                ),
                refs.as_slice(),
                None,
            ),
            Some(captured) => RawFinding::deterministic(
                rule,
                FindingState::Block,
                format!(
                    "Workspace tag `{tag}` yields version `{captured}`, not the declared release version \
                     `{expected}`."
                ),
                refs.as_slice(),
                Some("Retag HEAD for the declared version, or update `release.expected_version`."),
            ),
            None => RawFinding::unknown(
                rule,
                self.policy.on_unknown.version_match,
                format!(
                    "Workspace tag `{tag}` matched the pattern, but the pattern yields no `version` \
                     capture, so the declared release version cannot be compared."
                ),
                refs.as_slice(),
                Some(
                    "Add a `(?P<version>...)` capture to `[version] pattern`, or remove \
                     `release.expected_version`.",
                ),
            ),
        }
    }

    /// `artifacts.required` — every configured requirement present in the evidence that carries its kind.
    /// `elf` and `map` are satisfied only by an analyzed snapshot row and `bin` and `hex` only by a release
    /// attachment row, because an attachment is bytes nobody parsed (`ADR-0030` D-3). A required BIN that
    /// was attached is therefore a real answer; a required BIN nobody attached is still a `BLOCK`.
    fn rule_required_artifacts(&self) -> RawFinding {
        let rule = GateRuleId::RequiredArtifacts;
        let attachments = self.canonical_attachments();
        let mut refs = vec![format!(
            "policy:artifacts.required={}",
            canonical_list(&self.policy.required_artifact_kinds)
        )];
        for artifact in &self.artifacts {
            refs.push(artifact_ref(artifact));
        }
        let mut from_build: Vec<&str> = Vec::new();
        let mut from_attachments: Vec<&str> = Vec::new();
        let mut missing: Vec<&str> = Vec::new();
        let mut cited: Vec<GateAttachmentFact> = Vec::new();
        for word in &self.policy.required_artifact_kinds {
            match requirement_kind(word) {
                Some(kind @ (ArtifactKind::Elf | ArtifactKind::Map)) => {
                    if self.artifacts.iter().any(|a| a.kind == kind) {
                        from_build.push(word);
                    } else {
                        missing.push(word);
                    }
                }
                Some(kind @ (ArtifactKind::Bin | ArtifactKind::IntelHex)) => {
                    let rows: Vec<&GateAttachmentFact> =
                        attachments.iter().filter(|row| row.kind == kind).collect();
                    if rows.is_empty() {
                        missing.push(word);
                    } else {
                        from_attachments.push(word);
                        cited.extend(rows.into_iter().cloned());
                    }
                }
                // `unknown` attaches but satisfies nothing (D-3), and a word outside the requirement
                // vocabulary matches no kind at all, which is what this rule already reported.
                Some(ArtifactKind::Unknown) | None => missing.push(word),
            }
        }
        // One order for the cited rows regardless of the order the requirements listed them in.
        for row in GateAttachmentFact::canonicalized(&cited) {
            refs.push(attachment_ref(&row));
        }
        if missing.is_empty() {
            let summary = if from_attachments.is_empty() {
                format!(
                    "Every required artifact kind is present in the snapshot ({}).",
                    self.policy.required_artifact_kinds.join(", ")
                )
            } else if from_build.is_empty() {
                format!(
                    "Every required artifact kind is present: {} from release attachments (bytes verified, \
                     provenance not verified).",
                    from_attachments.join(" and ")
                )
            } else {
                format!(
                    "Every required artifact kind is present: {} from the analyzed build; {} from release \
                     attachments (bytes verified, provenance not verified).",
                    from_build.join(" and "),
                    from_attachments.join(" and ")
                )
            };
            RawFinding::deterministic(rule, FindingState::Pass, summary, refs.as_slice(), None)
        } else {
            let (summary, remediation) = if attachments.is_empty() {
                (
                    format!(
                        "Required artifact kind(s) missing from the snapshot: {}.",
                        missing.join(", ")
                    ),
                    "Add the missing artifact to this build, or remove it from `[artifacts] required` if \
                     this release does not ship it.",
                )
            } else {
                (
                    format!(
                        "Required artifact kind(s) are in neither the analyzed build nor the release \
                         attachments: {}.",
                        missing.join(", ")
                    ),
                    "Add the missing artifact to this build, or attach a file of that kind, or remove it \
                     from `[artifacts] required` if this release does not ship it.",
                )
            };
            RawFinding::deterministic(
                rule,
                FindingState::Block,
                summary,
                refs.as_slice(),
                Some(remediation),
            )
        }
    }

    /// `artifacts.hashes` — the digest invariant over both evidence classes. A snapshot row is read from the
    /// immutable snapshot rather than recomputed from the original file (prompt §20), and an attachment row
    /// carries the digest of the bytes `observe_attachment` actually streamed; what neither class ever
    /// establishes is where those bytes came from.
    fn rule_artifact_hashes(&self) -> RawFinding {
        let rule = GateRuleId::ArtifactHashes;
        let attachments = self.canonical_attachments();
        let mut unhashed: Vec<String> = self
            .artifacts
            .iter()
            .filter(|artifact| artifact.sha256.value().is_none())
            .map(|artifact| artifact_kind_label(artifact.kind).to_owned())
            .collect();
        for row in &attachments {
            if row.sha256.value().is_none() {
                unhashed.push(format!("attachment {}", artifact_kind_label(row.kind)));
            }
        }
        let mut refs: Vec<String> = self.artifacts.iter().map(artifact_ref).collect();
        for row in &attachments {
            refs.push(attachment_ref(row));
        }
        refs.push("policy:artifacts.hashes".to_owned());
        if unhashed.is_empty() {
            let summary = if attachments.is_empty() {
                format!(
                    "All {} snapshot artifact(s) carry a SHA-256 digest.",
                    self.artifacts.len()
                )
            } else {
                format!(
                    "All {} input(s) carry a SHA-256 digest: {} analyzed artifact(s) plus {} release \
                     attachment(s) whose bytes were hashed and whose origin or build provenance was not \
                     verified.",
                    self.artifacts.len() + attachments.len(),
                    self.artifacts.len(),
                    attachments.len()
                )
            };
            RawFinding::deterministic(rule, FindingState::Pass, summary, refs.as_slice(), None)
        } else {
            let summary = if attachments.is_empty() {
                format!(
                    "Snapshot artifact(s) without a valid SHA-256 digest: {}.",
                    unhashed.join(", ")
                )
            } else {
                format!(
                    "Artifact or attachment(s) without a valid SHA-256 digest: {}.",
                    unhashed.join(", ")
                )
            };
            RawFinding::deterministic(
                rule,
                FindingState::Block,
                summary,
                refs.as_slice(),
                Some(if attachments.is_empty() {
                    "Re-analyze the build so the snapshot records a digest for every artifact it holds."
                } else {
                    "Re-analyze the build, or attach the file again, so every input this run binds carries \
                     the digest of bytes that were actually read."
                }),
            )
        }
    }

    /// `memory.flash_budget` and `memory.ram_budget`: one implementation, two rules, because both
    /// budgets answer the same question about different sides of the footprint. A hard verdict requires
    /// an exact *and* admissible total — a floor never blocks (prompt §21-§22).
    fn rule_budget(&self, nonvolatile: bool) -> RawFinding {
        let rule = if nonvolatile {
            GateRuleId::FlashBudget
        } else {
            GateRuleId::RamBudget
        };
        let (budget, disposition, policy_ref, side) = if nonvolatile {
            (
                self.policy.flash_budget,
                self.policy.on_unknown.flash_budget,
                "policy:memory.flash_budget",
                "Nonvolatile",
            )
        } else {
            (
                self.policy.ram_budget,
                self.policy.on_unknown.ram_budget,
                "policy:memory.ram_budget",
                "Runtime RAM",
            )
        };
        let Some(budget) = budget else {
            return RawFinding::deterministic(
                rule,
                FindingState::NotApplicable,
                format!("No `{policy_ref}` budget is configured."),
                &[policy_ref],
                None,
            );
        };
        let fact = self.memory.as_ref().and_then(|memory| {
            if nonvolatile {
                memory.nonvolatile.as_ref()
            } else {
                memory.runtime_ram.as_ref()
            }
        });
        let mut refs = vec![format!("{policy_ref}={budget}")];
        if let Some(evidence_ref) = fact.and_then(|fact| fact.evidence_ref.as_ref()) {
            refs.push(evidence_ref.clone());
        }
        let reason = match fact {
            None => Some("the snapshot carries no footprint for this side".to_owned()),
            Some(fact) => match fact.bytes {
                None => Some(
                    fact.reason
                        .clone()
                        .unwrap_or_else(|| "no bytes were attributed".to_owned()),
                ),
                Some(bytes) if !fact.exact => Some(format!(
                    "the accounted total of {bytes} B is a floor, not a complete attribution ({})",
                    fact.reason
                        .clone()
                        .unwrap_or_else(|| "unattributed sections remain".to_owned())
                )),
                Some(bytes) if !fact.admissible => Some(format!(
                    "the evidence basis is too weak for a hard verdict ({bytes} B, basis: {})",
                    fact.basis.clone().unwrap_or_else(|| "unknown".to_owned())
                )),
                Some(_) => None,
            },
        };
        if let Some(reason) = reason {
            return RawFinding::unknown(
                rule,
                disposition,
                format!(
                    "{side} footprint cannot be compared with the {budget} B budget: {reason}."
                ),
                refs.as_slice(),
                Some(
                    "Supply the evidence this budget needs — a complete section attribution, and for a \
                     runtime budget a layout the accounting can admit — then rerun the Gate.",
                ),
            );
        }
        let bytes = fact
            .and_then(|fact| fact.bytes)
            .expect("a budget verdict with a reason already returned");
        if bytes > budget {
            RawFinding::deterministic(
                rule,
                FindingState::Block,
                format!("{side} footprint {bytes} B exceeds the {budget} B budget."),
                refs.as_slice(),
                Some(
                    "Reduce the footprint, or raise the budget deliberately: a budget raised to pass a \
                     Gate is a release decision, not a formatting change.",
                ),
            )
        } else {
            RawFinding::deterministic(
                rule,
                FindingState::Pass,
                format!(
                    "{side} footprint {bytes} B is within the {budget} B budget ({} B headroom).",
                    budget - bytes
                ),
                refs.as_slice(),
                None,
            )
        }
    }

    /// `diff.growth` — threshold review over a baseline comparison, reading P2's `ByteChange` values
    /// instead of computing a delta again (prompt §23).
    fn rule_growth(&self) -> RawFinding {
        let rule = GateRuleId::BaselineGrowth;
        let flash_threshold = self.policy.flash_growth_review_bytes;
        let ram_threshold = self.policy.ram_growth_review_bytes;
        if flash_threshold.is_none() && ram_threshold.is_none() {
            return RawFinding::deterministic(
                rule,
                FindingState::NotApplicable,
                "No `[diff]` growth threshold is configured.",
                &[
                    "policy:diff.flash_growth_review_bytes",
                    "policy:diff.ram_growth_review_bytes",
                ],
                None,
            );
        }
        let mut refs = Vec::new();
        if let Some(threshold) = flash_threshold {
            refs.push(format!("policy:diff.flash_growth_review_bytes={threshold}"));
        }
        if let Some(threshold) = ram_threshold {
            refs.push(format!("policy:diff.ram_growth_review_bytes={threshold}"));
        }
        let Some(baseline) = self.growth.baseline_snapshot_id.clone() else {
            return RawFinding::unknown(
                rule,
                self.policy.on_unknown.baseline_growth,
                "Growth against the baseline cannot be thresholded: no baseline snapshot was supplied \
                 for this run.",
                refs.as_slice(),
                Some(
                    "Select a baseline build for this release, or clear the `[diff]` thresholds if this \
                     release is not compared against one.",
                ),
            );
        };
        let mut over: Vec<String> = Vec::new();
        let mut gap: Option<String> = None;
        for (side, threshold, change) in [
            (
                "nonvolatile",
                flash_threshold,
                self.growth.nonvolatile.as_ref(),
            ),
            (
                "runtime_ram",
                ram_threshold,
                self.growth.runtime_ram.as_ref(),
            ),
        ] {
            let Some(threshold) = threshold else { continue };
            refs.push(format!("diff:{baseline}:{}:{side}", self.snapshot_id));
            let Some(change) = change else {
                gap = Some(format!("the {side} comparison produced no delta"));
                break;
            };
            match change.comparability {
                Comparability::Unknown => {
                    gap = Some(format!(
                        "the {side} delta is unknown ({})",
                        change
                            .reason
                            .clone()
                            .unwrap_or_else(|| "one side carries no number".to_owned())
                    ))
                }
                // A floor is not a measurement of the whole, so a threshold verdict on partial evidence
                // stays an evidence gap rather than becoming a review the owner would have to accept.
                Comparability::Partial => {
                    gap = Some(format!(
                        "the {side} delta rests on partial evidence, so a threshold verdict needs complete \
                     accounting"
                    ))
                }
                Comparability::Exact => match change.delta {
                    None => gap = Some(format!("the {side} delta is unavailable")),
                    Some(delta) => {
                        // Negative growth is not a failure: a smaller build is what the threshold wants.
                        if u64::try_from(delta).is_ok_and(|growth| growth > threshold) {
                            over.push(format!(
                                "{side} +{delta} B over the {threshold} B threshold"
                            ));
                        }
                    }
                },
            }
            if gap.is_some() {
                break;
            }
        }
        if let Some(gap) = gap {
            return RawFinding::unknown(
                rule,
                self.policy.on_unknown.baseline_growth,
                format!("Growth against the baseline cannot be thresholded: {gap}."),
                refs.as_slice(),
                Some(
                    "Give the baseline build the same evidence class as this one — a MAP on both sides \
                     usually does it — or clear the threshold this side cannot answer.",
                ),
            );
        }
        if over.is_empty() {
            RawFinding::deterministic(
                rule,
                FindingState::Pass,
                "Growth against the baseline stays within every configured threshold.",
                refs.as_slice(),
                None,
            )
        } else {
            RawFinding::deterministic(
                rule,
                FindingState::Review,
                format!(
                    "Growth over the configured review threshold: {}.",
                    over.join("; ")
                ),
                refs.as_slice(),
                Some(
                    "Review the change in Compare, then accept the review with a reason if the growth is \
                     intended, or reduce the footprint.",
                ),
            )
        }
    }

    /// `release.notes` — the declared Release Notes file exists. P3 does not parse its Markdown
    /// (prompt §24), and only the project-relative path is ever named.
    fn rule_release_notes(&self) -> RawFinding {
        let rule = GateRuleId::ReleaseNotes;
        let policy_ref = "policy:release.require_release_notes";
        if !self.policy.require_release_notes {
            return RawFinding::deterministic(
                rule,
                FindingState::NotApplicable,
                "`release.require_release_notes` is false, so no notes file is required.",
                &[policy_ref],
                None,
            );
        }
        let Some(fact) = self.release_notes.as_ref() else {
            return RawFinding::unknown(
                rule,
                self.policy.on_unknown.release_notes,
                format!(
                    "No observation was recorded for `release.release_notes_path` ({}).",
                    self.policy.release_notes_path
                ),
                &[policy_ref, "file:-"],
                Some("Rerun the Gate; the configured notes path was not observed."),
            );
        };
        let path_ref = format!("file:{}", fact.relative_path);
        let refs = vec![policy_ref.to_owned(), path_ref.clone()];
        match &fact.status {
            GateFileStatus::Present { sha256 } => {
                let mut refs = refs;
                let summary = match sha256 {
                    Some(digest) => {
                        refs.push(format!("file:{}#sha256={digest}", fact.relative_path));
                        format!("Release Notes `{}` is present.", fact.relative_path)
                    }
                    None => format!(
                        "Release Notes `{}` is present but was not digested.",
                        fact.relative_path
                    ),
                };
                RawFinding::deterministic(rule, FindingState::Pass, summary, &refs, None)
            }
            GateFileStatus::Missing => RawFinding::deterministic(
                rule,
                FindingState::Block,
                format!(
                    "Required Release Notes `{}` does not exist in the project.",
                    fact.relative_path
                ),
                refs.as_slice(),
                Some(
                    "Write the release notes at that project-relative path, or point \
                     `release.release_notes_path` at where they actually live.",
                ),
            ),
            GateFileStatus::Unreadable { reason } => RawFinding::unknown(
                rule,
                self.policy.on_unknown.release_notes,
                format!(
                    "Release Notes `{}` could not be read: {reason}.",
                    fact.relative_path
                ),
                refs.as_slice(),
                Some(
                    "Fix the file's permissions or lock and rerun. A read failure is an evidence gap, not \
                     a missing file.",
                ),
            ),
        }
    }

    /// `evidence.unknown_review` — how much of the snapshot's own evidence is `Unknown`. The count comes
    /// from the snapshot; the items are never reclassified here (prompt §25).
    fn rule_unknown_evidence(&self) -> RawFinding {
        let rule = GateRuleId::UnknownEvidenceReview;
        let Some(threshold) = self.policy.unknown_evidence_review_count else {
            return RawFinding::deterministic(
                rule,
                FindingState::NotApplicable,
                "`gate.unknown_evidence_review_count` is not configured.",
                &["policy:gate.unknown_evidence_review_count"],
                None,
            );
        };
        let mut refs = vec![format!(
            "policy:gate.unknown_evidence_review_count={threshold}"
        )];
        refs.extend(self.unknown_evidence.sample_refs.iter().cloned());
        if self.unknown_evidence.count >= usize::try_from(threshold).unwrap_or(usize::MAX) {
            RawFinding::deterministic(
                rule,
                FindingState::Review,
                format!(
                    "{} snapshot evidence item(s) are Unknown, at or above the review threshold of \
                     {threshold}. The items stay Unknown; this finding asks a person to look.",
                    self.unknown_evidence.count
                ),
                refs.as_slice(),
                Some(
                    "Supply the missing evidence — a MAP, a complete section attribution, a declared \
                     version — or accept the review with a reason once the gaps are understood.",
                ),
            )
        } else {
            RawFinding::deterministic(
                rule,
                FindingState::Pass,
                format!(
                    "{} snapshot evidence item(s) are Unknown, below the review threshold of {threshold}.",
                    self.unknown_evidence.count
                ),
                refs.as_slice(),
                None,
            )
        }
    }
}

/// A rule's answer before the run id is folded into its finding id.
#[derive(Debug)]
struct RawFinding {
    rule: GateRuleId,
    state: FindingState,
    effective: EffectiveSeverity,
    summary: String,
    refs: Vec<String>,
    remediation: Option<String>,
}

impl RawFinding {
    /// A deterministically evaluated rule: severity follows state exactly (ADR-0023).
    fn deterministic(
        rule: GateRuleId,
        state: FindingState,
        summary: impl Into<String>,
        refs: &[impl AsRef<str>],
        remediation: Option<&str>,
    ) -> Self {
        let effective = match state {
            FindingState::Pass | FindingState::NotApplicable => EffectiveSeverity::Pass,
            FindingState::Review => EffectiveSeverity::Review,
            FindingState::Block => EffectiveSeverity::Block,
            // An `UNKNOWN` finding is always built through `unknown`, which carries its disposition.
            FindingState::Unknown => EffectiveSeverity::Review,
        };
        Self {
            rule,
            state,
            effective,
            summary: summary.into(),
            refs: refs.iter().map(|r| r.as_ref().to_owned()).collect(),
            remediation: remediation.map(str::to_owned),
        }
    }

    /// The evidence is missing. The state says so and stays `UNKNOWN`; only the disposition decides how
    /// hard that counts (ADR-0023 — an unknown is never rewritten into a review or a block).
    fn unknown(
        rule: GateRuleId,
        disposition: UnknownDisposition,
        summary: impl Into<String>,
        refs: &[impl AsRef<str>],
        remediation: Option<&str>,
    ) -> Self {
        Self {
            rule,
            state: FindingState::Unknown,
            effective: disposition.severity(),
            summary: summary.into(),
            refs: refs.iter().map(|r| r.as_ref().to_owned()).collect(),
            remediation: remediation.map(str::to_owned),
        }
    }

    fn into_finding(self, run_id: &str) -> GateFinding {
        GateFinding {
            id: format!("{run_id}#{}", self.rule.as_str()),
            rule_id: self.rule,
            state: self.state,
            effective_severity: self.effective,
            summary: self.summary,
            evidence_refs: self.refs,
            remediation: self.remediation,
        }
    }
}

/// The stable `artifact:<kind>:<sha256>` locator (prompt §30). No host path: the kind and the digest
/// identify the input, and the digest is the snapshot's own.
fn artifact_ref(artifact: &GateArtifactFact) -> String {
    format!(
        "artifact:{}:{}",
        artifact_kind_label(artifact.kind),
        artifact.sha256.value().map(String::as_str).unwrap_or("-")
    )
}

/// The stable `attachment:<kind>:<sha256>` locator (`04_TECH/28` §5 rule 7). It is deliberately a different
/// scheme from `artifact:`: an attachment is a file nobody analyzed, and citing it with the locator that
/// means "a row of the analyzed snapshot" would be the prose form of the defect `ADR-0030` D-4 refuses.
/// The same rule keeps the host path out — a kind and a digest name the bytes, and nothing else does.
fn attachment_ref(attachment: &GateAttachmentFact) -> String {
    format!(
        "attachment:{}:{}",
        artifact_kind_label(attachment.kind),
        attachment.sha256.value().map(String::as_str).unwrap_or("-")
    )
}

/// The v1 requirement vocabulary maps onto artifact kinds. A word outside the vocabulary matches no
/// artifact, which the required-artifacts rule reports as missing; the config loader rejects it long
/// before a Gate sees it (prompt §10, §19).
fn requirement_kind(word: &str) -> Option<ArtifactKind> {
    match word {
        "elf" => Some(ArtifactKind::Elf),
        "map" => Some(ArtifactKind::Map),
        "bin" => Some(ArtifactKind::Bin),
        "hex" => Some(ArtifactKind::IntelHex),
        _ => None,
    }
}

const fn artifact_kind_label(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Elf => "elf",
        ArtifactKind::Map => "map",
        ArtifactKind::Bin => "bin",
        ArtifactKind::IntelHex => "hex",
        ArtifactKind::Unknown => "unknown",
    }
}

fn unknown_reason<T>(fact: &Fact<T>) -> String {
    fact.reason_if_unknown()
        .map(str::to_owned)
        .unwrap_or_else(|| "no fact was supplied".to_owned())
}

fn fact_text<T: fmt::Display>(fact: &Fact<T>) -> String {
    match fact {
        Fact::Known(value) => format!("known:{value}"),
        Fact::Unknown { reason } => format!("unknown:{reason}"),
    }
}

fn canonical_optional(value: Option<&str>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| format!("known:{v}"))
}

fn canonical_number(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| v.to_string())
}

fn canonical_list(values: &[String]) -> String {
    if values.is_empty() {
        return "-".to_owned();
    }
    values.join(",")
}

fn canonical_file_fact(fact: &GateFileFact) -> String {
    let status = match &fact.status {
        GateFileStatus::Present { sha256 } => format!(
            "present:{}",
            sha256.clone().unwrap_or_else(|| "no-digest".to_owned())
        ),
        GateFileStatus::Missing => "missing".to_owned(),
        GateFileStatus::Unreadable { reason } => format!("unreadable:{reason}"),
    };
    format!("{}#{status}", fact.relative_path)
}

fn canonical_memory(memory: Option<&GateMemoryFacts>) -> String {
    let mut text = String::from("memory[\n");
    match memory {
        None => text.push_str("  none\n"),
        Some(memory) => {
            for (side, fact) in [
                ("nonvolatile", memory.nonvolatile.as_ref()),
                ("runtime_ram", memory.runtime_ram.as_ref()),
            ] {
                match fact {
                    None => text.push_str(&format!("  {side}=-\n")),
                    // The reason text is deliberately absent. It is prose that *explains* a gap, and
                    // the deciding facts are already here: the figure, whether it is complete, whether
                    // its basis may block, and what that basis was. A desktop run hydrates the same
                    // totals from SQLite, which stores no reason, so carrying the words here would
                    // give one judgement two run ids depending on which surface read it.
                    Some(fact) => text.push_str(&format!(
                        "  {side} {} {} {} {} {}\n",
                        fact.bytes
                            .map(|b| b.to_string())
                            .unwrap_or_else(|| "-".to_owned()),
                        fact.exact,
                        fact.admissible,
                        fact.basis.as_deref().unwrap_or("-"),
                        fact.evidence_ref.as_deref().unwrap_or("-"),
                    )),
                }
            }
        }
    }
    text.push_str("]\n");
    text
}

fn canonical_growth(growth: &GateGrowthFacts) -> String {
    let mut text = String::from("growth[\n");
    for (side, change) in [
        ("nonvolatile", growth.nonvolatile.as_ref()),
        ("runtime_ram", growth.runtime_ram.as_ref()),
    ] {
        match change {
            None => text.push_str(&format!("  {side}=-\n")),
            Some(change) => text.push_str(&format!(
                "  {side} {} {} {} {}\n",
                change
                    .base
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "-".to_owned()),
                change
                    .target
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "-".to_owned()),
                change
                    .delta
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "-".to_owned()),
                change.comparability.label(),
            )),
        }
    }
    text.push_str("]\n");
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(known: bool) -> Fact<String> {
        if known {
            Fact::known("a".repeat(40))
        } else {
            Fact::unknown("git is not installed")
        }
    }

    /// A context where nothing is required, nothing is configured and Git can answer: the baseline for
    /// per-rule tests, which then change exactly one thing.
    fn context() -> GateContext {
        GateContext {
            snapshot_id: "snap-1".to_owned(),
            artifacts: vec![GateArtifactFact {
                kind: ArtifactKind::Elf,
                sha256: Fact::known("b".repeat(64)),
                byte_size: 1_024,
            }],
            attachments: Vec::new(),
            memory: None,
            git: GateGitFacts {
                available: true,
                head_commit: head(true),
                exact_tag: Fact::unknown("no tag points at HEAD"),
                dirty: Fact::known(false),
            },
            version: None,
            release_notes: None,
            growth: GateGrowthFacts::without_baseline(),
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
                on_unknown: UnknownPolicy::default(),
            },
        }
    }

    fn evaluate(context: &GateContext) -> GateEvaluation {
        context.evaluate("gate-test")
    }

    fn finding(evaluation: &GateEvaluation, rule: GateRuleId) -> &GateFinding {
        evaluation
            .finding(rule)
            .unwrap_or_else(|| panic!("rule {rule} produced no finding"))
    }

    /// Evaluate and take one finding by value, so a test can assert on several fields of it without
    /// holding a borrow into a temporary evaluation.
    fn rule_finding(context: &GateContext, rule: GateRuleId) -> GateFinding {
        evaluate(context)
            .finding(rule)
            .cloned()
            .unwrap_or_else(|| panic!("rule {rule} produced no finding"))
    }

    /// An evidence ref is a stable locator, never a host path (prompt §30).
    fn assert_locator_not_host_path(reference: &str) {
        for forbidden in ["\\", ":/", "://"] {
            assert!(
                !reference.contains(forbidden),
                "ref {reference} carries the host-path marker {forbidden:?}"
            );
        }
        let tail = reference.rsplit(':').next().unwrap_or(reference);
        assert!(
            !tail.starts_with('/'),
            "ref {reference} carries an absolute path"
        );
    }

    /// One bound attachment, in the only shape `C1-U1`'s observation can produce: observed bytes with a
    /// kind the release owner declared (`04_TECH/28` §2, §5 rule 3).
    fn attached(kind: ArtifactKind, digest: &str, byte_size: u64) -> GateAttachmentFact {
        GateAttachmentFact {
            kind,
            sha256: Fact::known(digest.to_owned()),
            byte_size,
            kind_basis: KindBasis::Declared,
        }
    }

    #[test]
    fn every_rule_answers_exactly_once_in_canonical_order() {
        let evaluation = evaluate(&context());
        let ids: Vec<&str> = evaluation
            .findings
            .iter()
            .map(|f| f.rule_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "git.clean",
                "release.commit_matches_expected",
                "release.version_matches_policy",
                "artifacts.required",
                "artifacts.hashes",
                "memory.flash_budget",
                "memory.ram_budget",
                "diff.growth",
                "release.notes",
                "evidence.unknown_review",
            ]
        );
        assert_eq!(evaluation.findings.len(), 10);
    }

    #[test]
    fn an_unconfigured_policy_is_all_not_applicable_and_passes() {
        let evaluation = evaluate(&context());
        // Only the two artifact rules always answer: an ELF is required, and a snapshot digest is an
        // invariant. The other eight report that nothing was configured, which is not a pass.
        assert_eq!(evaluation.counts.not_applicable, 8);
        assert_eq!(evaluation.counts.pass, 2);
        assert_eq!(
            evaluation.overall_effective_severity,
            EffectiveSeverity::Pass
        );
    }

    #[test]
    fn evaluation_never_mutates_the_context_it_reads() {
        let context = context();
        let before = context.clone();
        let _ = evaluate(&context);
        assert_eq!(
            context, before,
            "Gate evaluation must be pure over its input"
        );
    }

    #[test]
    fn identical_input_evaluates_identically_including_finding_ids() {
        let context = context();
        assert_eq!(
            context.evaluate("gate-abc"),
            context.evaluate("gate-abc"),
            "the same run input must produce the same findings, ids included"
        );
        assert_eq!(
            finding(&context.evaluate("gate-abc"), GateRuleId::GitClean).id,
            "gate-abc#git.clean"
        );
    }

    #[test]
    fn unknown_state_is_never_rewritten_and_never_counts_as_pass() {
        let mut context = context();
        context.git = GateGitFacts::unavailable("git is not installed");
        context.policy.require_clean_git = true;
        let evaluation = evaluate(&context);
        let git = finding(&evaluation, GateRuleId::GitClean);
        assert_eq!(git.state, FindingState::Unknown);
        assert_eq!(git.effective_severity, EffectiveSeverity::Review);
        assert_eq!(
            evaluation.overall_effective_severity,
            EffectiveSeverity::Review,
            "an unknown mapped to review cannot let the run pass"
        );
    }

    #[test]
    fn unknown_mapped_to_block_aggregates_to_block() {
        let mut context = context();
        context.git = GateGitFacts::unavailable("this directory is not a git repository");
        context.policy.require_clean_git = true;
        context.policy.on_unknown.git_clean = UnknownDisposition::Block;
        let evaluation = evaluate(&context);
        assert_eq!(
            finding(&evaluation, GateRuleId::GitClean).state,
            FindingState::Unknown,
            "the disposition changes severity, never the factual state"
        );
        assert_eq!(
            evaluation.overall_effective_severity,
            EffectiveSeverity::Block
        );
    }

    #[test]
    fn block_outranks_every_other_effective_severity() {
        let mut context = context();
        context.policy.require_clean_git = true;
        context.git.dirty = Fact::known(true);
        context.policy.flash_budget = Some(512);
        context.memory = Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact::exact(
                4_096,
                "section-headers",
                "evidence:ev-memory-flash",
            )),
            runtime_ram: None,
        });
        let evaluation = evaluate(&context);
        assert_eq!(evaluation.counts.block, 2);
        assert_eq!(
            evaluation.overall_effective_severity,
            EffectiveSeverity::Block
        );
    }

    #[test]
    fn git_clean_covers_na_pass_block_and_unknown() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::GitClean).state,
            FindingState::NotApplicable
        );
        context.policy.require_clean_git = true;
        assert_eq!(
            rule_finding(&context, GateRuleId::GitClean).state,
            FindingState::Pass
        );
        context.git.dirty = Fact::known(true);
        let dirty = rule_finding(&context, GateRuleId::GitClean);
        assert_eq!(dirty.state, FindingState::Block);
        assert!(dirty.remediation.is_some());
        context.git.dirty = Fact::unknown("git timed out");
        assert_eq!(
            rule_finding(&context, GateRuleId::GitClean).state,
            FindingState::Unknown
        );
    }

    #[test]
    fn expected_commit_matches_only_the_workspace_head_and_never_claims_the_artifact() {
        let mut context = context();
        context.policy.expected_commit = Some("a".repeat(40));
        let matched = rule_finding(&context, GateRuleId::CommitMatchesExpected);
        assert_eq!(matched.state, FindingState::Pass);
        assert_eq!(
            matched.summary,
            "Workspace HEAD matches the declared release commit."
        );
        for banned in ["built from", "artifact was built"] {
            assert!(
                !matched.summary.contains(banned),
                "workspace provenance must not be worded as artifact build proof: {}",
                matched.summary
            );
        }
        context.policy.expected_commit = Some("f".repeat(40));
        assert_eq!(
            rule_finding(&context, GateRuleId::CommitMatchesExpected).state,
            FindingState::Block
        );
        context.git.head_commit = head(false);
        assert_eq!(
            rule_finding(&context, GateRuleId::CommitMatchesExpected).state,
            FindingState::Unknown
        );
    }

    #[test]
    fn version_rule_covers_na_no_tag_bad_tag_good_tag_and_expected_mismatch() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::NotApplicable
        );
        context.policy.version = Some(VersionPolicy {
            pattern: r"^v(?P<version>\d+\.\d+\.\d+)$".to_owned(),
            expected_version: None,
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::Block,
            "a reachable Git with no exact tag is a determinate failure, not an evidence gap"
        );
        context.git = GateGitFacts::unavailable("git is not installed");
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::Unknown
        );
        context.git = GateGitFacts {
            available: true,
            head_commit: head(true),
            exact_tag: Fact::known("v1.2.3".to_owned()),
            dirty: Fact::known(false),
        };
        context.version = Some(GateVersionFacts {
            pattern_matches: Fact::known(false),
            captured_version: Fact::unknown("the tag did not match the pattern"),
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::Block,
            "a tag that does not match the pattern blocks"
        );
        context.version = Some(GateVersionFacts {
            pattern_matches: Fact::known(true),
            captured_version: Fact::known("1.2.3".to_owned()),
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::Pass
        );
        context.policy.version = Some(VersionPolicy {
            pattern: r"^v(?P<version>\d+\.\d+\.\d+)$".to_owned(),
            expected_version: Some("1.2.4".to_owned()),
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::Block
        );
        context.policy.version = Some(VersionPolicy {
            pattern: r"^v(?P<version>\d+\.\d+\.\d+)$".to_owned(),
            expected_version: Some("1.2.3".to_owned()),
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::Pass
        );
    }

    #[test]
    fn a_version_pattern_never_puts_a_separator_into_an_evidence_locator() {
        // A regex is the ordinary shape for `[version] pattern`, and `\` is the one character that
        // marks a host path: the desktop persists every locator, and the storage layer refuses a ref
        // carrying a separator. So the pattern is quoted in the words a reader sees, never in the
        // pointer (prompt §46, `AGENTS.md` 7).
        let mut context = context();
        context.git.exact_tag = Fact::known("v1.2.3".to_owned());
        context.version = Some(GateVersionFacts {
            pattern_matches: Fact::known(true),
            captured_version: Fact::known("1.2.3".to_owned()),
        });
        context.policy.version = Some(VersionPolicy {
            pattern: r"^v(?P<version>\d+\.\d+\.\d+)$".to_owned(),
            expected_version: Some("1.2.3".to_owned()),
        });
        let evaluation = context.evaluate("gate-locator-check");
        for finding in &evaluation.findings {
            for reference in &finding.evidence_refs {
                assert!(
                    !reference.contains('\\'),
                    "a locator carried a Windows separator: {reference}"
                );
            }
        }
        let version = rule_finding(&context, GateRuleId::VersionMatchesPolicy);
        assert_eq!(
            version.evidence_refs,
            vec![
                "policy:version.source".to_owned(),
                "policy:version.pattern".to_owned(),
                "git:exact-tag=v1.2.3".to_owned(),
                "policy:release.expected_version".to_owned(),
            ],
            "the version rule points at policy fields, not at their text"
        );
        assert!(
            context.canonical_input().contains(r"(?P<version>\d+"),
            "leaving the pattern out of the locator must not leave it out of the run fingerprint"
        );
    }

    #[test]
    fn a_pattern_without_a_version_capture_cannot_check_the_expected_version() {
        let mut context = context();
        context.git.exact_tag = Fact::known("v1.2.3".to_owned());
        context.policy.version = Some(VersionPolicy {
            pattern: r"^v\d+\.\d+\.\d+$".to_owned(),
            expected_version: Some("1.2.3".to_owned()),
        });
        context.version = Some(GateVersionFacts {
            pattern_matches: Fact::known(true),
            captured_version: Fact::unknown("pattern has no `version` capture"),
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::VersionMatchesPolicy).state,
            FindingState::Unknown
        );
    }

    #[test]
    fn required_artifacts_blocks_on_a_kind_the_snapshot_does_not_hold() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::RequiredArtifacts).state,
            FindingState::Pass
        );
        context.policy.required_artifact_kinds = vec!["elf".to_owned(), "map".to_owned()];
        let blocked = rule_finding(&context, GateRuleId::RequiredArtifacts);
        assert_eq!(blocked.state, FindingState::Block);
        assert!(blocked.summary.contains("map"));
        for reference in &blocked.evidence_refs {
            assert_locator_not_host_path(reference);
        }
    }

    #[test]
    fn an_unconfigured_requirement_word_matches_no_artifact() {
        let mut context = context();
        context.policy.required_artifact_kinds = vec!["elf".to_owned(), "hex".to_owned()];
        assert_eq!(
            rule_finding(&context, GateRuleId::RequiredArtifacts).state,
            FindingState::Block,
            "not analyzing BIN/HEX does not make a configured requirement pass"
        );
    }

    /// C1-U2's two-source rule (`04_TECH/28` §5 rule 7, `ADR-0030` D-3): the attachment classes are
    /// satisfiable, and the finding says which set satisfied what.
    #[test]
    fn a_bound_bin_and_hex_attachment_pair_satisfies_their_own_requirements() {
        let mut context = context();
        context.policy.required_artifact_kinds =
            vec!["elf".to_owned(), "bin".to_owned(), "hex".to_owned()];
        context.attachments = vec![
            attached(ArtifactKind::IntelHex, &"d".repeat(64), 683_214),
            attached(ArtifactKind::Bin, &"c".repeat(64), 245_760),
        ];
        let finding = rule_finding(&context, GateRuleId::RequiredArtifacts);
        assert_eq!(
            finding.state,
            FindingState::Pass,
            "a bound BIN and HEX must satisfy their kinds: {}",
            finding.summary
        );
        assert_eq!(finding.effective_severity, EffectiveSeverity::Pass);
        assert_eq!(
            finding.summary,
            "Every required artifact kind is present: elf from the analyzed build; bin and hex from \
             release attachments (bytes verified, provenance not verified).",
            "the summary names which set satisfied each kind, and never implies the bytes were analyzed"
        );
        // Canonical order is `(kind word, digest)`, so the BIN row is cited before the HEX row even though
        // the context received them the other way round.
        assert_eq!(
            finding.evidence_refs,
            [
                "policy:artifacts.required=elf,bin,hex".to_owned(),
                format!("artifact:elf:{}", "b".repeat(64)),
                format!("attachment:bin:{}", "c".repeat(64)),
                format!("attachment:hex:{}", "d".repeat(64)),
            ],
            "{finding:?}"
        );
        for reference in &finding.evidence_refs {
            assert_locator_not_host_path(reference);
        }
    }

    /// The retained negative half of the boundary `C1-U1` locked: a required BIN with nothing attached is
    /// still a `BLOCK`, so making the kind satisfiable did not make it optional.
    #[test]
    fn a_required_bin_with_no_attachment_still_blocks() {
        let mut context = context();
        context.policy.required_artifact_kinds = vec!["elf".to_owned(), "bin".to_owned()];
        let finding = rule_finding(&context, GateRuleId::RequiredArtifacts);
        assert_eq!(finding.state, FindingState::Block, "{finding:?}");
        assert_eq!(
            finding.summary, "Required artifact kind(s) missing from the snapshot: bin.",
            "with no attachment bound the rule says what it said before this unit"
        );
        assert_eq!(
            finding.remediation.as_deref(),
            Some(
                "Add the missing artifact to this build, or remove it from `[artifacts] required` if this \
                 release does not ship it."
            ),
            "{finding:?}"
        );
    }

    /// `ADR-0030` D-3 in the direction a person would try first: an attached ELF is not the analyzed ELF.
    #[test]
    fn an_attachment_never_satisfies_a_snapshot_only_kind() {
        let mut context = context();
        context.policy.required_artifact_kinds = vec!["map".to_owned()];
        // Hand-built on purpose: `observe_attachment` refuses an `elf` or `map` before a fact exists, so the
        // only way this row can reach a context is a caller that assembled one without observing it. The
        // rule must still refuse it.
        context.attachments = vec![attached(ArtifactKind::Map, &"e".repeat(64), 4_096)];
        let finding = rule_finding(&context, GateRuleId::RequiredArtifacts);
        assert_eq!(
            finding.state,
            FindingState::Block,
            "a MAP-shaped attachment is not the MAP a build was analyzed with: {}",
            finding.summary
        );
        assert!(
            !finding
                .evidence_refs
                .iter()
                .any(|reference| reference.starts_with("attachment:")),
            "an unsatisfying attachment was cited as if it did: {:?}",
            finding.evidence_refs
        );
        assert!(
            finding
                .remediation
                .as_deref()
                .unwrap_or_default()
                .contains("attach a file of that kind"),
            "a block with an attachment bound still tells the owner about the analyzed build only: {:?}",
            finding.remediation
        );
    }

    /// `ADR-0030` D-3 and §8's M14: an `unknown` row satisfies neither kind.
    #[test]
    fn an_unknown_attachment_satisfies_no_required_kind() {
        for word in ["bin", "hex"] {
            let mut context = context();
            context.policy.required_artifact_kinds = vec![word.to_owned(), "elf".to_owned()];
            context.attachments = vec![attached(ArtifactKind::Unknown, &"f".repeat(64), 512)];
            let finding = rule_finding(&context, GateRuleId::RequiredArtifacts);
            assert_eq!(
                finding.state,
                FindingState::Block,
                "an unidentified file cannot stand in for a required `{word}`: {}",
                finding.summary
            );
        }
        // And it is not cited when a real attachment does the satisfying.
        let mut context = context();
        context.policy.required_artifact_kinds = vec!["bin".to_owned()];
        context.attachments = vec![
            attached(ArtifactKind::Unknown, &"f".repeat(64), 512),
            attached(ArtifactKind::Bin, &"c".repeat(64), 512),
        ];
        let finding = rule_finding(&context, GateRuleId::RequiredArtifacts);
        assert_eq!(finding.state, FindingState::Pass, "{finding:?}");
        assert!(
            !finding
                .evidence_refs
                .iter()
                .any(|reference| reference.starts_with("attachment:unknown:")),
            "the unknown row was cited as evidence of a satisfied requirement: {:?}",
            finding.evidence_refs
        );
    }

    /// §5.B: both classes are counted and located, and the attachment half of the sentence says what was
    /// verified and what was not.
    #[test]
    fn the_hashes_rule_counts_both_classes_and_cites_each_by_its_own_scheme() {
        let mut context = context();
        context.attachments = vec![
            attached(ArtifactKind::Bin, &"c".repeat(64), 245_760),
            attached(ArtifactKind::IntelHex, &"d".repeat(64), 683_214),
        ];
        let finding = rule_finding(&context, GateRuleId::ArtifactHashes);
        assert_eq!(finding.state, FindingState::Pass, "{finding:?}");
        assert_eq!(
            finding.summary,
            "All 3 input(s) carry a SHA-256 digest: 1 analyzed artifact(s) plus 2 release attachment(s) \
             whose bytes were hashed and whose origin or build provenance was not verified.",
            "the counts are per class and the limitation travels with the count"
        );
        assert_eq!(
            finding.evidence_refs,
            [
                format!("artifact:elf:{}", "b".repeat(64)),
                format!("attachment:bin:{}", "c".repeat(64)),
                format!("attachment:hex:{}", "d".repeat(64)),
                "policy:artifacts.hashes".to_owned(),
            ],
            "{finding:?}"
        );
        for reference in &finding.evidence_refs {
            assert_locator_not_host_path(reference);
        }
    }

    /// An attachment row with no digest is unreachable through observation — an attachment that could not be
    /// hashed stopped at E-1 — so a context that carries one blocks rather than counting it as hashed.
    #[test]
    fn an_attachment_row_without_a_digest_blocks_the_hashes_rule() {
        let mut context = context();
        context.attachments = vec![GateAttachmentFact {
            kind: ArtifactKind::Bin,
            sha256: Fact::unknown("no read happened"),
            byte_size: 1,
            kind_basis: KindBasis::Declared,
        }];
        let finding = rule_finding(&context, GateRuleId::ArtifactHashes);
        assert_eq!(finding.state, FindingState::Block, "{finding:?}");
        assert_eq!(
            finding.summary,
            "Artifact or attachment(s) without a valid SHA-256 digest: attachment bin.",
            "{finding:?}"
        );
    }

    /// I1, AC-01 and AC-11 in one place: with nothing attached, both rules produce the words and the refs
    /// they produced before this unit existed. Summaries are not hashed into a run id, but they are what a
    /// reviewer reads, and a frozen golden and a stored record must keep them.
    #[test]
    fn the_two_artifact_rules_say_exactly_what_they_said_when_nothing_is_attached() {
        let context = context();
        let required = rule_finding(&context, GateRuleId::RequiredArtifacts);
        assert_eq!(
            required.summary,
            "Every required artifact kind is present in the snapshot (elf)."
        );
        assert_eq!(
            required.evidence_refs,
            [
                "policy:artifacts.required=elf".to_owned(),
                format!("artifact:elf:{}", "b".repeat(64)),
            ]
        );
        let hashes = rule_finding(&context, GateRuleId::ArtifactHashes);
        assert_eq!(
            hashes.summary,
            "All 1 snapshot artifact(s) carry a SHA-256 digest."
        );
        assert_eq!(
            hashes.evidence_refs,
            [
                format!("artifact:elf:{}", "b".repeat(64)),
                "policy:artifacts.hashes".to_owned(),
            ]
        );
    }

    /// `ADR-0030` D-7 and I5: the Unknown this unit introduces is a permanent statement about provenance,
    /// not a snapshot evidence gap, so it must not move `unknown.count` or its rule.
    #[test]
    fn attaching_a_file_never_moves_the_unknown_evidence_rule() {
        let mut base = context();
        base.unknown_evidence = GateUnknownEvidence {
            count: 3,
            sample_refs: vec!["evidence:section-count".to_owned()],
        };
        base.policy.unknown_evidence_review_count = Some(2);
        let without = rule_finding(&base, GateRuleId::UnknownEvidenceReview);
        base.attachments = vec![attached(ArtifactKind::Bin, &"c".repeat(64), 245_760)];
        let with = rule_finding(&base, GateRuleId::UnknownEvidenceReview);
        assert_eq!(without.state, with.state, "{without:?} vs {with:?}");
        assert_eq!(without.summary, with.summary);
        assert_eq!(without.evidence_refs, with.evidence_refs);
        assert_eq!(with.effective_severity, without.effective_severity);
    }

    #[test]
    fn artifact_hashes_block_when_a_snapshot_digest_is_missing() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::ArtifactHashes).state,
            FindingState::Pass
        );
        context.artifacts.push(GateArtifactFact {
            kind: ArtifactKind::Map,
            sha256: Fact::unknown("the companion row carries no digest"),
            byte_size: 4_096,
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::ArtifactHashes).state,
            FindingState::Block
        );
    }

    #[test]
    fn budgets_cover_na_pass_block_and_unknown() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::FlashBudget).state,
            FindingState::NotApplicable
        );
        context.policy.flash_budget = Some(4_096);
        context.policy.ram_budget = Some(2_048);
        context.memory = Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact::exact(
                4_096,
                "section-headers-and-regions",
                "evidence:ev-memory-flash",
            )),
            runtime_ram: Some(GateBudgetFact::exact(
                3_000,
                "ram-region-attribution",
                "evidence:ev-memory-runtime",
            )),
        });
        let evaluation = evaluate(&context);
        assert_eq!(
            finding(&evaluation, GateRuleId::FlashBudget).state,
            FindingState::Pass
        );
        assert_eq!(
            finding(&evaluation, GateRuleId::RamBudget).state,
            FindingState::Block
        );
        context.memory = Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact::unknown("no section could be attributed")),
            runtime_ram: None,
        });
        let evaluation = evaluate(&context);
        assert_eq!(
            finding(&evaluation, GateRuleId::FlashBudget).state,
            FindingState::Unknown
        );
        assert_eq!(
            finding(&evaluation, GateRuleId::FlashBudget).effective_severity,
            EffectiveSeverity::Block,
            "the default disposition for a missing budget figure is block"
        );
    }

    #[test]
    fn a_floor_never_hard_blocks_however_far_over_budget_it_sits() {
        let mut context = context();
        context.policy.flash_budget = Some(1_024);
        context.memory = Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact {
                bytes: Some(900_000),
                exact: false,
                admissible: true,
                basis: Some("section-headers".to_owned()),
                reason: Some("2 sections unattributed".to_owned()),
                evidence_ref: Some("evidence:ev-memory-flash".to_owned()),
            }),
            runtime_ram: None,
        });
        let budget = rule_finding(&context, GateRuleId::FlashBudget);
        assert_eq!(budget.state, FindingState::Unknown);
        assert!(
            !budget.summary.contains("exceeds"),
            "a partial total must not be reported as an overrun: {}",
            budget.summary
        );
    }

    #[test]
    fn weak_evidence_is_not_admissible_even_when_every_number_is_exact() {
        let mut context = context();
        context.policy.flash_budget = Some(1_024);
        context.memory = Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact {
                bytes: Some(900_000),
                exact: true,
                admissible: false,
                basis: Some("heuristic-file-size".to_owned()),
                reason: None,
                evidence_ref: None,
            }),
            runtime_ram: None,
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::FlashBudget).state,
            FindingState::Unknown
        );
    }

    #[test]
    fn growth_covers_na_unknown_without_baseline_review_and_pass() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::BaselineGrowth).state,
            FindingState::NotApplicable
        );
        context.policy.flash_growth_review_bytes = Some(4_096);
        let no_baseline = rule_finding(&context, GateRuleId::BaselineGrowth);
        assert_eq!(no_baseline.state, FindingState::Unknown);
        assert_eq!(no_baseline.effective_severity, EffectiveSeverity::Review);

        // The Gate reads P2's diff output; it never recomputes a delta.
        context.growth = GateGrowthFacts {
            baseline_snapshot_id: Some("snap-0".to_owned()),
            nonvolatile: Some(ByteChange::between(
                Some(100_000),
                Some(120_000),
                Comparability::Exact,
            )),
            runtime_ram: None,
        };
        let review = rule_finding(&context, GateRuleId::BaselineGrowth);
        assert_eq!(review.state, FindingState::Review);
        assert!(review.summary.contains("+20000 B"));

        context.growth.nonvolatile = Some(ByteChange::between(
            Some(100_000),
            Some(101_000),
            Comparability::Exact,
        ));
        assert_eq!(
            rule_finding(&context, GateRuleId::BaselineGrowth).state,
            FindingState::Pass
        );
        context.growth.nonvolatile = Some(ByteChange::between(
            Some(100_000),
            Some(90_000),
            Comparability::Exact,
        ));
        assert_eq!(
            rule_finding(&context, GateRuleId::BaselineGrowth).state,
            FindingState::Pass,
            "negative growth is not a failure"
        );
    }

    #[test]
    fn growth_on_partial_or_unknown_evidence_is_an_evidence_gap_not_a_review() {
        let mut context = context();
        context.policy.flash_growth_review_bytes = Some(1);
        context.growth = GateGrowthFacts {
            baseline_snapshot_id: Some("snap-0".to_owned()),
            nonvolatile: Some(ByteChange::unknown(Some(1), Some(2), "one side is partial")),
            runtime_ram: None,
        };
        assert_eq!(
            rule_finding(&context, GateRuleId::BaselineGrowth).state,
            FindingState::Unknown
        );
        context.growth.nonvolatile = Some(ByteChange::between(
            Some(1),
            Some(500),
            Comparability::Partial,
        ));
        assert_eq!(
            rule_finding(&context, GateRuleId::BaselineGrowth).state,
            FindingState::Unknown,
            "prompt §23: a Partial diff is UNKNOWN for Gate thresholding"
        );
    }

    #[test]
    fn release_notes_covers_na_present_missing_and_unreadable() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::ReleaseNotes).state,
            FindingState::NotApplicable
        );
        context.policy.require_release_notes = true;
        context.release_notes = Some(GateFileFact {
            relative_path: "RELEASE_NOTES.md".to_owned(),
            status: GateFileStatus::Present {
                sha256: Some("c".repeat(64)),
            },
        });
        let present = rule_finding(&context, GateRuleId::ReleaseNotes);
        assert_eq!(present.state, FindingState::Pass);
        assert!(
            present
                .evidence_refs
                .iter()
                .any(|r| r == "file:RELEASE_NOTES.md"),
            "{:?}",
            present.evidence_refs
        );
        context.release_notes = Some(GateFileFact {
            relative_path: "RELEASE_NOTES.md".to_owned(),
            status: GateFileStatus::Missing,
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::ReleaseNotes).state,
            FindingState::Block
        );
        context.release_notes = Some(GateFileFact {
            relative_path: "RELEASE_NOTES.md".to_owned(),
            status: GateFileStatus::Unreadable {
                reason: "permission denied".to_owned(),
            },
        });
        assert_eq!(
            rule_finding(&context, GateRuleId::ReleaseNotes).state,
            FindingState::Unknown,
            "a read failure is an evidence gap, not an absence"
        );
    }

    #[test]
    fn a_release_notes_path_never_carries_a_host_path_into_an_evidence_ref() {
        let mut context = context();
        context.policy.require_release_notes = true;
        context.release_notes = Some(GateFileFact {
            relative_path: "docs/RELEASE_NOTES.md".to_owned(),
            status: GateFileStatus::Present { sha256: None },
        });
        for reference in &rule_finding(&context, GateRuleId::ReleaseNotes).evidence_refs {
            assert_locator_not_host_path(reference);
        }
    }

    #[test]
    fn unknown_evidence_count_reviews_only_at_its_threshold() {
        let mut context = context();
        assert_eq!(
            rule_finding(&context, GateRuleId::UnknownEvidenceReview).state,
            FindingState::NotApplicable
        );
        context.policy.unknown_evidence_review_count = Some(2);
        context.unknown_evidence = GateUnknownEvidence {
            count: 1,
            sample_refs: vec!["evidence:ev-symbol-address".to_owned()],
        };
        assert_eq!(
            rule_finding(&context, GateRuleId::UnknownEvidenceReview).state,
            FindingState::Pass
        );
        context.unknown_evidence.count = 2;
        let review = rule_finding(&context, GateRuleId::UnknownEvidenceReview);
        assert_eq!(review.state, FindingState::Review);
        assert_eq!(review.effective_severity, EffectiveSeverity::Review);
        assert!(
            review.summary.contains("The items stay Unknown"),
            "the underlying evidence is never rewritten: {}",
            review.summary
        );
        context.unknown_evidence.count = 40;
        assert_eq!(
            rule_finding(&context, GateRuleId::UnknownEvidenceReview).state,
            FindingState::Review,
            "one finding covers a large count, with a bounded sample of refs"
        );
    }

    #[test]
    fn accepting_a_review_dispositions_the_aggregate_without_touching_the_finding() {
        let mut context = context();
        context.policy.flash_growth_review_bytes = Some(1);
        context.growth = GateGrowthFacts {
            baseline_snapshot_id: Some("snap-0".to_owned()),
            nonvolatile: Some(ByteChange::between(
                Some(1),
                Some(500),
                Comparability::Exact,
            )),
            runtime_ram: None,
        };
        let evaluation = evaluate(&context);
        let review = finding(&evaluation, GateRuleId::BaselineGrowth);
        assert_eq!(review.state, FindingState::Review);
        assert!(review.acceptable_as_review());
        assert_eq!(
            evaluation.overall_effective_severity,
            EffectiveSeverity::Review
        );
        let accepted = vec![review.id.clone()];
        assert_eq!(
            evaluation.aggregate_with_acceptances(&accepted),
            EffectiveSeverity::Pass
        );
        // The immutable record still says REVIEW with REVIEW severity after acceptance.
        let after = finding(&evaluation, GateRuleId::BaselineGrowth);
        assert_eq!(after.state, FindingState::Review);
        assert_eq!(after.effective_severity, EffectiveSeverity::Review);
    }

    #[test]
    fn only_review_findings_are_acceptable() {
        let mut context = context();
        context.policy.require_clean_git = true;
        context.git.dirty = Fact::known(true);
        context.policy.expected_commit = Some("f".repeat(40));
        context.git.head_commit = head(false);
        let evaluation = evaluate(&context);
        for rule in [
            GateRuleId::GitClean,
            GateRuleId::CommitMatchesExpected,
            GateRuleId::ArtifactHashes,
        ] {
            let finding = finding(&evaluation, rule);
            let acceptable = finding.acceptable_as_review();
            match finding.state {
                FindingState::Review => assert!(acceptable, "{rule} must be acceptable"),
                FindingState::Block | FindingState::Unknown => {
                    assert!(
                        !acceptable,
                        "{rule} must not be acceptable: {}",
                        finding.state
                    )
                }
                _ => {}
            }
        }
        assert!(
            !finding(&evaluation, GateRuleId::GitClean).acceptable_as_review(),
            "a block is not a review"
        );
    }

    #[test]
    fn an_unknown_mapped_to_review_survives_every_acceptance() {
        let mut context = context();
        context.git = GateGitFacts::unavailable("git is not installed");
        context.policy.require_clean_git = true;
        let evaluation = evaluate(&context);
        let ids: Vec<String> = evaluation
            .findings
            .iter()
            .filter(|f| f.state == FindingState::Unknown)
            .map(|f| f.id.clone())
            .collect();
        assert_eq!(
            evaluation.aggregate_with_acceptances(&ids),
            EffectiveSeverity::Review,
            "accepting an unknown's id must not disposition it: UNKNOWN clears only with evidence"
        );
    }

    #[test]
    fn canonical_input_is_stable_over_identical_facts() {
        let context = context();
        assert_eq!(context.canonical_input(), context.clone().canonical_input());
    }

    #[test]
    fn canonical_input_tracks_every_fact_that_changes_a_verdict() {
        let base = context();
        let base_text = base.canonical_input();
        let mut dirty = base.clone();
        dirty.git.dirty = Fact::known(true);
        assert_ne!(
            base_text,
            dirty.canonical_input(),
            "a dirty workspace is a new run"
        );

        let mut policy = base.clone();
        policy.policy.flash_budget = Some(4_096);
        assert_ne!(
            base_text,
            policy.canonical_input(),
            "a policy change is a new run"
        );

        let mut baseline = base.clone();
        baseline.growth.baseline_snapshot_id = Some("snap-0".to_owned());
        assert_ne!(
            base_text,
            baseline.canonical_input(),
            "a baseline change is a new run"
        );

        let mut notes = base.clone();
        notes.release_notes = Some(GateFileFact {
            relative_path: "RELEASE_NOTES.md".to_owned(),
            status: GateFileStatus::Present {
                sha256: Some("c".repeat(64)),
            },
        });
        assert_ne!(base_text, notes.canonical_input());
        let mut notes_bytes = notes.clone();
        notes_bytes.release_notes = Some(GateFileFact {
            relative_path: "RELEASE_NOTES.md".to_owned(),
            status: GateFileStatus::Present {
                sha256: Some("d".repeat(64)),
            },
        });
        assert_ne!(
            notes.canonical_input(),
            notes_bytes.canonical_input(),
            "different notes bytes are a different run"
        );

        let mut head = base.clone();
        head.git.head_commit = Fact::known("f".repeat(40));
        assert_ne!(base_text, head.canonical_input());
    }

    #[test]
    fn canonical_input_carries_no_absolute_path_or_wall_clock() {
        let mut context = context();
        context.release_notes = Some(GateFileFact {
            relative_path: "RELEASE_NOTES.md".to_owned(),
            status: GateFileStatus::Present { sha256: None },
        });
        let text = context.canonical_input();
        for forbidden in ["\\", "C:", "/home/", "/Users/", "imported_at", "snap-1\\"] {
            assert!(
                !text.contains(forbidden),
                "the run fingerprint must not carry {forbidden}: {text}"
            );
        }
    }

    #[test]
    fn policy_text_is_stable_and_changes_only_with_semantics() {
        let base = context();
        let text = canonical_policy_text(&base.policy);
        assert_eq!(text, canonical_policy_text(&base.clone().policy));
        for field in [
            "release.require_release_notes=",
            "artifacts.required=",
            "memory.flash_budget=",
            "gate.on_unknown.flash_budget=",
            "release.expected_commit=",
            "version.pattern=",
        ] {
            assert!(text.contains(field), "the policy text omits {field}");
        }
        let mut changed = base.clone();
        changed.policy.release_notes_path = "CHANGELOG.md".to_owned();
        assert_ne!(text, canonical_policy_text(&changed.policy));
        changed.policy.release_notes_path = base.policy.release_notes_path.clone();
        changed.policy.on_unknown.git_clean = UnknownDisposition::Block;
        assert_ne!(
            text,
            canonical_policy_text(&changed.policy),
            "a disposition change is a policy change, so it must change the hash input"
        );
    }

    #[test]
    fn policy_text_names_the_version_pattern_and_expected_version_separately() {
        let mut context = context();
        context.policy.version = Some(VersionPolicy {
            pattern: r"^v(?P<version>\d+\.\d+\.\d+)$".to_owned(),
            expected_version: Some("1.2.3".to_owned()),
        });
        let text = canonical_policy_text(&context.policy);
        assert!(text.contains("version.pattern="), "{text}");
        assert!(text.contains("version.expected=known:1.2.3"), "{text}");
        assert!(
            text.contains("gate.on_unknown.flash_budget=block"),
            "{text}"
        );
    }

    #[test]
    fn state_names_are_the_portable_spellings() {
        assert_eq!(FindingState::NotApplicable.as_str(), "N/A");
        assert_eq!(FindingState::Unknown.as_str(), "UNKNOWN");
        assert_eq!(EffectiveSeverity::Pass.as_str(), "PASS");
        assert_eq!(UnknownDisposition::Review.as_str(), "review");
    }
}
