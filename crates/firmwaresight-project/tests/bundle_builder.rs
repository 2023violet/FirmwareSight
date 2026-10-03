//! The Release Bundle engine, driven the way the CLI and the desktop will drive it (prompt §55).
//!
//! These tests write real files into throwaway directories and read them back, because what is under test is
//! a promise about bytes on a disk: that a bundle contains exactly the layout §7 names, that its two hash
//! lists agree with those bytes and with each other, that a source file which moved is refused rather than
//! shipped, and that an existing directory is never destroyed without a recognizable bundle and an explicit
//! confirmation.
//!
//! FirmwareSight's own repository is never the release subject (§47): every project here is a temporary
//! directory holding a config, a notes file and copies of the committed P2 fixture pair.

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::artifact::{Artifact, ParserId};
use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, SnapshotBuilder};
use firmwaresight_core::domain::capability::{
    Availability, Capabilities, FormatSupport, Provision,
};
use firmwaresight_core::domain::diff::{
    BudgetState, ByteChange, ChangeCounts, Comparability, DiffArtifact, DiffBudget, DiffResult,
    DiffSideMemory, MemoryDiff, ObjectAttribution, SideEvidence, UnchangedCounts,
};
use firmwaresight_core::domain::evidence::{EvidenceClass, EvidenceItem, SourceType};
use firmwaresight_core::domain::gate::{
    FindingState, GateContext, GateEvaluation, GateFileStatus, GateGitFacts, GateGrowthFacts,
    GateRuleId,
};
use firmwaresight_core::domain::identity::{
    Architecture, ArtifactKind, ArtifactTimes, Bitness, BuildIdentity, Endianness, Fact, Sha256,
};
use firmwaresight_core::domain::memory::{MemoryFootprint, MemoryLayout};
use firmwaresight_core::domain::release::is_safe_bundle_relative_path;
use firmwaresight_core::domain::section::{Section, SectionFlags, SectionRole};
use firmwaresight_project::bundle::{
    self, BundleAcceptance, BundleError, BundleRequest, FileRole, is_recognizable_bundle, prepare,
    verify_bundle,
};
use firmwaresight_project::evidence::{GateRunRequest, growth_facts, observe_release_notes};
use firmwaresight_project::{
    GitObservation, LoadedProject, SnapshotFacts, build_context, policy_sha256, run_id,
};

const TARGET_ELF: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const TARGET_MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";
const BASE_ELF: &str = "fixtures/elf/p2-diff/base/firmware.elf";

/// A release the Gate can pass on its own terms: provenance required, notes required, and a version policy
/// whose pattern the workspace tag satisfies. No budget and no growth threshold, so those rules answer `N/A`
/// rather than passing silently — the tests that need a refusal configure one of them and say which.
const PASSING_CONFIG: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"
"#;

/// The same, with a growth threshold the fixture comparison is over: one REVIEW finding to accept (§47).
const REVIEWING_CONFIG: &str = "schema_version = 1\n\n[project]\nname = \"motor-controller\"\n\n\
                                [artifacts]\nrequired = [\"elf\", \"map\"]\n\n[version]\nsource = \"git_tag\"\n\
                                pattern = '^v(?P<version>\\d+\\.\\d+\\.\\d+)$'\n\n[release]\n\
                                require_clean_git = true\nrequire_release_notes = true\n\
                                release_notes_path = \"RELEASE_NOTES.md\"\nexpected_version = \"1.4.2\"\n\n\
                                [diff]\nflash_growth_review_bytes = 100\n";

const NOTES: &str = "# 1.4.2\n\n- the modem driver is now in the image\n";
const HEAD: &str = "8d2f1c4a5b6e7f8091a2b3c4d5e6f708192a3b4c";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/firmwaresight-project lives at <root>/crates/<name>")
        .to_path_buf()
}

// --------------------------------------------------------------------------- the world under test

/// One release subject: a project folder, the build it was analyzed into, and the facts the Gate reads.
///
/// `request()` borrows all of it, which is the shape the engine takes: the caller holds the facts, and the
/// bundle is assembled from them rather than from anything the engine can go and re-read behind its back.
struct World {
    dir: PathBuf,
    loaded: LoadedProject,
    git: GitObservation,
    snapshot: BuildSnapshot,
    comparison: Option<DiffResult>,
    acceptances: Vec<BundleAcceptance>,
    selected_run_id: Option<String>,
    elf: PathBuf,
    map: PathBuf,
    notes: PathBuf,
}

impl World {
    fn new(label: &str, config: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "firmwaresight-p4-{label}-{}-{}",
            std::process::id(),
            nanos()
        ));
        std::fs::create_dir_all(dir.join("build")).expect("the project directory exists");
        std::fs::write(dir.join("firmwaresight.toml"), config).expect("the config is written");
        let notes = dir.join("RELEASE_NOTES.md");
        std::fs::write(&notes, NOTES).expect("the notes are written");

        // The shipped bytes are the committed P2 target fixture pair, copied into a build folder the way real
        // build output sits.
        let root = repo_root();
        let elf = dir.join("build/firmware.elf");
        let map = dir.join("build/firmware.map");
        std::fs::copy(root.join(TARGET_ELF), &elf)
            .expect("the target ELF is copied into the project");
        std::fs::copy(root.join(TARGET_MAP), &map).expect("the MAP is copied into the project");

        let loaded = LoadedProject::load(&dir).expect("the project config loads");
        let snapshot = sealed_snapshot(&dir, &elf, &map);
        Self {
            dir,
            loaded,
            git: observation("v1.4.2", false),
            snapshot,
            comparison: None,
            acceptances: Vec::new(),
            selected_run_id: None,
            elf,
            map,
            notes,
        }
    }

    fn path(&self) -> &Path {
        &self.dir
    }

    /// The workspace facts to re-observe at export: a dirty tree, a moved tag, or no repository at all.
    fn set_git(&mut self, tag: Option<&str>, dirty: bool) {
        self.git = match tag {
            None => GitObservation::unavailable("no git executable was found"),
            Some(name) => observation(name, dirty),
        };
    }

    /// A baseline comparison over the committed base fixture, growing the image by 1 000 bytes.
    fn with_baseline(&mut self) {
        let base = repo_root().join(BASE_ELF);
        let base_metadata = std::fs::metadata(&base).expect("the base fixture is present");
        self.comparison = Some(DiffResult {
            base_snapshot_id: "snap-base-0001".to_owned(),
            target_snapshot_id: self.snapshot.id().as_str().to_owned(),
            base_artifact: DiffArtifact {
                file_name: "firmware.elf".to_owned(),
                sha256: digest_of(&base).hex().to_owned(),
                byte_size: base_metadata.len(),
            },
            target_artifact: DiffArtifact {
                file_name: "firmware.elf".to_owned(),
                sha256: digest_of(&self.elf).hex().to_owned(),
                byte_size: self.elf.metadata().expect("the target is present").len(),
            },
            memory: growth_of(1_000),
            section_changes: Vec::new(),
            symbol_changes: Vec::new(),
            counts: ChangeCounts::default(),
            unchanged: UnchangedCounts::default(),
            attribution: ObjectAttribution {
                available: false,
                reason: "the persisted rows carry no object-file attribution".to_owned(),
            },
            warnings: Vec::new(),
        });
    }

    /// The context the P3 adapter builds from exactly this state — the reuse §9 asks for, in the test too.
    fn context(&self) -> GateContext {
        let target = SnapshotFacts::from_snapshot(&self.snapshot);
        let growth = match self.comparison.as_ref() {
            Some(diff) => growth_facts(diff),
            None => GateGrowthFacts::without_baseline(),
        };
        let notes =
            observe_release_notes(&self.loaded.root, &self.loaded.policy.release_notes_path);
        build_context(&GateRunRequest {
            target: &target,
            growth,
            git: &self.git,
            policy: self.loaded.policy.clone(),
            release_notes: Some(notes),
        })
    }

    /// The run id this state produces, which is what the desktop would have stored a run under.
    fn recomputed_run_id(&self) -> String {
        run_id(&self.context())
    }

    fn evaluate(&self) -> GateEvaluation {
        let generated = self.recomputed_run_id();
        self.context().evaluate(&generated)
    }

    /// Accept one rule's finding, as a release owner would in the desktop.
    fn accept(&mut self, rule: GateRuleId) {
        self.acceptances.push(BundleAcceptance {
            finding_id: format!("{}#{}", self.recomputed_run_id(), rule.as_str()),
            actor: "release-owner".to_owned(),
            accepted_at: "2026-10-01T08:12:44Z".to_owned(),
            reason: "the growth is the new modem driver, reviewed by hand".to_owned(),
        });
    }

    /// Bind the plan to a stored run, which is the desktop's case (§9).
    fn select_stored_run(&mut self) {
        self.selected_run_id = Some(self.recomputed_run_id());
    }

    fn request(&self) -> BundleRequest<'_> {
        BundleRequest {
            project: &self.loaded,
            git: &self.git,
            snapshot: &self.snapshot,
            comparison: self.comparison.as_ref(),
            selected_run_id: self.selected_run_id.as_deref(),
            acceptances: &self.acceptances,
            fwsight_version: "0.6.0-test",
        }
    }

    /// Prepare and publish into a fresh sibling folder, which is what choosing a destination produces.
    fn publish(&self, label: &str) -> (PathBuf, bundle::BundleOutcome) {
        let parent = self.parent(label);
        std::fs::create_dir_all(&parent).expect("the destination parent exists");
        let plan = prepare(&self.request()).expect("the release is prepared");
        let outcome = plan
            .publish(&self.request(), &parent, false)
            .expect("the bundle is published");
        (parent.join(&outcome.directory_name), outcome)
    }

    fn parent(&self, label: &str) -> PathBuf {
        self.dir.join(format!("out-{label}"))
    }

    /// Every file under one root, bundle-relative and sorted.
    fn tree(&self, root: &Path) -> Vec<String> {
        let mut names = Vec::new();
        collect(root, root, &mut names);
        names.sort();
        names
    }

    /// What a destination folder holds: the bundle names, with the engine's own siblings visible.
    fn folder_contents(&self, parent: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(parent)
            .expect("the folder is readable")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn collect(root: &Path, directory: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).expect("the directory is readable") {
        let entry = entry.expect("a readable directory entry");
        let metadata = entry.metadata().expect("an entry has metadata");
        let relative = entry
            .path()
            .strip_prefix(root)
            .expect("the entry is inside the tree")
            .to_string_lossy()
            .replace('\\', "/");
        if metadata.is_dir() {
            collect(root, &entry.path(), out);
        } else {
            out.push(relative);
        }
    }
}

fn nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0)
}

fn observation(tag: &str, dirty: bool) -> GitObservation {
    GitObservation {
        facts: GateGitFacts {
            available: true,
            head_commit: Fact::known(HEAD.to_owned()),
            exact_tag: Fact::known(tag.to_owned()),
            dirty: Fact::known(dirty),
        },
        exact_tags: vec![tag.to_owned()],
    }
}

fn digest_of(path: &Path) -> Sha256 {
    let hex = firmwaresight_project::fingerprint::file_sha256(path).expect("the file is readable");
    Sha256::parse(&hex).expect("a digest reads back as one")
}

fn growth_of(delta: u64) -> MemoryDiff {
    MemoryDiff {
        nonvolatile: ByteChange::between(Some(16_000), Some(16_000 + delta), Comparability::Exact),
        runtime_ram: ByteChange::between(Some(4_096), Some(4_096), Comparability::Exact),
        comparability: Comparability::Exact,
        base: side_memory(16_000),
        target: side_memory(16_000 + delta),
        evidence_warning: None,
    }
}

fn side_memory(total: u64) -> DiffSideMemory {
    DiffSideMemory {
        footprint_row_present: true,
        nonvolatile: DiffBudget {
            state: BudgetState::Exact,
            bytes: Some(total),
        },
        runtime_ram: DiffBudget {
            state: BudgetState::Exact,
            bytes: Some(4_096),
        },
        excluded_metadata_bytes: Some(0),
        evidence: SideEvidence {
            map_backed: true,
            layout_source: "program-headers".to_owned(),
            weakest_basis: Some("ElfProgramHeader".to_owned()),
        },
    }
}

// --------------------------------------------------------------------------- the fixture build

/// One artifact row, holding the bytes' real digest and the path the analysis was pointed at — project
/// relative, which is how a build folder is normally addressed and how §22 resolves it.
fn artifact_row(kind: ArtifactKind, path: &Path, relative: &str) -> Artifact {
    Artifact {
        id: format!("art-{}", kind.word()),
        path: relative.to_owned(),
        kind,
        sha256: digest_of(path),
        byte_size: path.metadata().expect("the artifact is present").len(),
        parser_id: if kind == ArtifactKind::Map {
            ParserId::gnu_ld_map()
        } else {
            ParserId::elf_object("0.36")
        },
        architecture: Architecture::Thumb,
        bitness: Bitness::Bits32,
        endianness: Endianness::Little,
        entry_point: Fact::known(0x0800_0041),
        build_id: Fact::unknown("no .note.gnu.build-id section is present".to_owned()),
        times: ArtifactTimes::default(),
        identity: BuildIdentity::default(),
        evidence: Vec::new(),
    }
}

fn sections() -> Vec<Section> {
    let flags = |execute: bool, write: bool| SectionFlags {
        alloc: true,
        execute,
        write,
        merge: false,
        strings: false,
        tls: false,
        compressed: false,
    };
    vec![
        Section {
            index: 1,
            name: Fact::known(".text".to_owned()),
            role: SectionRole::Code,
            flags: flags(true, false),
            virtual_address: Fact::known(0x0800_0000),
            load_address: Fact::known(0x0800_0000),
            file_offset: Fact::known(0x40),
            file_size: 12_000,
            memory_size: Fact::known(12_000),
            region: Fact::known("FLASH".to_owned()),
        },
        Section {
            index: 2,
            name: Fact::known(".bss".to_owned()),
            role: SectionRole::UninitializedData,
            flags: flags(false, true),
            virtual_address: Fact::known(0x2000_0000),
            load_address: Fact::known(0x2000_0000),
            file_offset: Fact::unknown("SHT_NOBITS stores no file payload".to_owned()),
            file_size: 0,
            memory_size: Fact::known(4_096),
            region: Fact::known("SRAM".to_owned()),
        },
    ]
}

fn sealed_snapshot(dir: &Path, elf: &Path, map: &Path) -> BuildSnapshot {
    let items = sections();
    let footprint = MemoryFootprint::compute(&items, &MemoryLayout::empty());
    SnapshotBuilder::new("0.6.0-test")
        .project_id("motor-controller")
        .artifact(artifact_row(ArtifactKind::Elf, elf, &relative(dir, elf)))
        .artifact(artifact_row(ArtifactKind::Map, map, &relative(dir, map)))
        .sections(items)
        .memory(footprint)
        .evidence(vec![
            EvidenceItem::new(
                "ev-text-size",
                EvidenceClass::Observed,
                SourceType::ElfSectionHeader,
                "elf.section_header[1].sh_size",
                "memory.nonvolatile",
                "12000",
                "section-role+file-size",
            ),
            EvidenceItem::new(
                "ev-build-id",
                EvidenceClass::Unknown,
                SourceType::RuleEngine,
                "elf.note",
                "artifact.build_id",
                "unknown",
                "explicit-unknown",
            ),
        ])
        .capabilities(Capabilities {
            elf: FormatSupport::Supported,
            sections: Availability::Available,
            symbols: Availability::Unavailable,
            debug_info: Availability::Unavailable,
            map: Provision::Provided,
            object_attribution: Availability::Unavailable,
            git: Availability::Available,
        })
        .seal()
        .expect("a snapshot with two artifacts seals")
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("the artifact is inside the project")
        .to_string_lossy()
        .replace('\\', "/")
}

fn read_manifest(bundle: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(bundle.join("release-manifest.json"))
        .expect("the manifest is there");
    serde_json::from_str(&text).expect("the manifest is one JSON document")
}

/// Replace one bundle document with an edited one, in the exact byte shape the engine writes.
fn write_document(bundle: &Path, name: &str, document: &serde_json::Value) {
    std::fs::write(
        bundle.join(name),
        firmwaresight_report::render::document_json(document),
    )
    .expect("the document is rewritten");
}

/// Re-record one document's digest and size in both indexes.
///
/// A tamper that leaves the two lists stale proves only that hashing works. This one proves the bundle is
/// refused when the lists and the bytes agree with each other and nothing else is right. `SHA256SUMS` is
/// re-recorded too, because the manifest indexes it (§20) and rewriting a line in it moves its own digest.
fn reindex(bundle: &Path, name: &str) {
    let bytes = std::fs::read(bundle.join(name)).expect("the document is there");
    let digest = digest_of_bytes(&bytes);

    let sums = bundle.join("SHA256SUMS");
    let rewritten = std::fs::read_to_string(&sums)
        .expect("SHA256SUMS is there")
        .lines()
        .map(|line| {
            if line.ends_with(&format!("  {name}")) {
                format!("{digest}  {name}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&sums, format!("{rewritten}\n")).expect("the checksum index is rewritten");
    let sums_bytes = std::fs::read(&sums).expect("the rewritten index is there");

    let mut manifest = read_manifest(bundle);
    for (entry_name, entry_bytes) in [
        (name, bytes.as_slice()),
        ("SHA256SUMS", sums_bytes.as_slice()),
    ] {
        let entry = manifest["files"]
            .as_array_mut()
            .expect("files is a list")
            .iter_mut()
            .find(|row| row["path"].as_str() == Some(entry_name))
            .expect("the manifest indexes the document");
        entry["sha256"] = serde_json::Value::String(digest_of_bytes(entry_bytes));
        entry["size"] = serde_json::Value::from(entry_bytes.len() as u64);
    }
    write_document(bundle, "release-manifest.json", &manifest);
}

// --------------------------------------------------------------------------- the canonical bundle

#[test]
fn a_passing_release_publishes_exactly_the_canonical_layout() {
    let world = World::new("layout", PASSING_CONFIG);
    let (bundle, outcome) = world.publish("layout");

    assert_eq!(
        world.tree(&bundle),
        vec![
            "SHA256SUMS",
            "accepted-reviews.json",
            "analysis.json",
            "artifacts/firmware.elf",
            "artifacts/firmware.map",
            "gate-results.json",
            "release-manifest.json",
            "release-notes.md",
            "release-report.html",
        ],
        "§7's layout, with no baseline so no `diff.json`, and nothing extra"
    );
    assert_eq!(outcome.release_version, "1.4.2");
    assert!(outcome.release_id.starts_with("release-"));
    assert_eq!(outcome.release_id.len(), "release-".len() + 64);
    assert_eq!(outcome.gate_run_id, world.recomputed_run_id());
    assert_eq!(
        outcome.manifest_sha256.len(),
        64,
        "the release record gets a digest of the manifest as written"
    );
    assert_eq!(
        world.folder_contents(&world.parent("layout")),
        vec![outcome.directory_name.clone()],
        "no staging or backup sibling survives a successful publish"
    );
    verify_bundle(&bundle).expect("the bundle verifies against its own bytes");
}

#[test]
fn the_directory_a_bundle_proposes_is_named_from_the_release_and_carries_no_path() {
    let world = World::new("name", PASSING_CONFIG);
    let plan = prepare(&world.request()).expect("prepared");
    let short: String = plan.preview.release_id["release-".len()..]
        .chars()
        .take(12)
        .collect();
    assert_eq!(
        plan.proposed_directory_name(),
        format!("motor-controller-1.4.2-{short}"),
        "§29's proposed child name, from the project, the version and the release id"
    );
    let text = format!("{:?}", plan.preview);
    assert!(
        !text.contains(&world.dir.to_string_lossy().to_string()),
        "the preview carried the project root"
    );
}

#[test]
fn both_hash_layers_agree_with_the_bytes_and_with_each_other() {
    let world = World::new("hashes", PASSING_CONFIG);
    let (bundle, outcome) = world.publish("hashes");

    let sums = std::fs::read_to_string(bundle.join("SHA256SUMS")).expect("SHA256SUMS is there");
    let listed: Vec<&str> = sums
        .lines()
        .map(|line| line.split_once("  ").expect("a sums line").1)
        .collect();
    assert!(
        !listed.contains(&"SHA256SUMS") && !listed.contains(&"release-manifest.json"),
        "§20: the payload list names neither of the two indexes"
    );
    assert!(listed.contains(&"release-report.html"));
    assert!(listed.contains(&"artifacts/firmware.elf"));
    for line in sums.lines() {
        let (digest, path) = line.split_once("  ").expect("a sums line");
        let bytes = std::fs::read(bundle.join(path)).expect("the listed file is present");
        assert_eq!(
            digest_of_bytes(&bytes),
            digest,
            "`{path}` does not hash to what SHA256SUMS records"
        );
    }

    let manifest = read_manifest(&bundle);
    let indexed: Vec<&str> = manifest["files"]
        .as_array()
        .expect("files[]")
        .iter()
        .map(|entry| entry["path"].as_str().expect("an indexed path"))
        .collect();
    assert!(
        indexed.contains(&"SHA256SUMS"),
        "§20: the manifest hashes the payload list too"
    );
    assert!(
        !indexed.contains(&"release-manifest.json"),
        "the manifest does not index itself"
    );
    let sums_digest = manifest["files"]
        .as_array()
        .expect("files[]")
        .iter()
        .find(|entry| entry["path"] == "SHA256SUMS")
        .expect("SHA256SUMS is indexed")["sha256"]
        .as_str()
        .expect("a digest");
    assert_eq!(
        sums_digest,
        digest_of_bytes(sums.as_bytes()),
        "the digest the manifest records for SHA256SUMS is the digest of its bytes"
    );
    assert_eq!(
        manifest["release"]["id"].as_str().expect("a release id"),
        outcome.release_id
    );
    assert_eq!(
        manifest["extensions"]["release_version_source"]
            .as_str()
            .expect("a version source"),
        "release.expected_version",
        "§11: the version came from the policy the Gate endorsed"
    );
    assert_eq!(
        manifest["extensions"]["integrity_model"]["self_digest_written"].as_bool(),
        Some(false),
        "§20: no file in the bundle carries a faked digest of itself"
    );
}

fn digest_of_bytes(bytes: &[u8]) -> String {
    firmwaresight_project::fingerprint::sha256_hex(bytes)
}

#[test]
fn the_report_precedes_the_two_lists_it_cannot_hash() {
    let world = World::new("cycle", PASSING_CONFIG);
    let (bundle, outcome) = world.publish("cycle");
    let html =
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there");
    let sums = std::fs::read_to_string(bundle.join("SHA256SUMS")).expect("SHA256SUMS is there");
    let manifest =
        std::fs::read_to_string(bundle.join("release-manifest.json")).expect("it is there");

    // §20's cycle, broken rather than closed. The report is payload, so the digest of the file that hashes
    // the report — and of the manifest that hashes that file — cannot appear inside it. Each is hashed here
    // from the bytes on disk, which is what the report could not have known while it was composed.
    let sums_digest = digest_of_bytes(sums.as_bytes());
    let manifest_digest = digest_of_bytes(manifest.as_bytes());
    assert_eq!(
        manifest_digest, outcome.manifest_sha256,
        "the release record binds the manifest as written"
    );
    assert!(
        !html.contains(&sums_digest),
        "the report printed SHA256SUMS's digest"
    );
    assert!(
        !html.contains(&manifest_digest),
        "the report printed the manifest's digest"
    );
    assert!(
        !sums.contains(&manifest_digest),
        "SHA256SUMS hashed the file that hashes it"
    );
    // The payload digests the report *can* name, because those files were written before it.
    let elf = digest_of_bytes(
        &std::fs::read(bundle.join("artifacts/firmware.elf")).expect("the artifact"),
    );
    assert!(html.contains(&elf), "the report names the shipped bytes");
    assert!(sums.contains(&elf));

    for name in ["release-report.html", "SHA256SUMS", "release-manifest.json"] {
        assert!(html.contains(name), "the report does not name `{name}`");
    }
    assert!(
        html.contains("sha256sum -c SHA256SUMS"),
        "§44's steps are printed"
    );
}

#[test]
fn a_bundle_prepared_twice_from_the_same_inputs_writes_the_same_bytes() {
    let world = World::new("determinism", PASSING_CONFIG);
    let (first, first_outcome) = world.publish("one");
    let second_parent = world.parent("two");
    std::fs::create_dir_all(&second_parent).expect("the second destination exists");
    let plan = prepare(&world.request()).expect("prepared again");
    plan.publish(&world.request(), &second_parent, false)
        .expect("published again");
    let second = second_parent.join(plan.proposed_directory_name());

    assert_eq!(first_outcome.release_id, plan.preview.release_id);
    for name in [
        "analysis.json",
        "gate-results.json",
        "accepted-reviews.json",
        "release-report.html",
        "SHA256SUMS",
        "release-manifest.json",
        "release-notes.md",
        "artifacts/firmware.elf",
    ] {
        assert_eq!(
            std::fs::read(first.join(name)).expect("the first bundle holds the file"),
            std::fs::read(second.join(name)).expect("the second bundle holds it too"),
            "`{name}` is not byte-identical across two preparations"
        );
    }
}

#[test]
fn a_released_bundle_verifies_from_its_own_bytes_after_its_project_is_gone() {
    // §45's engine claim in one test: relocate the directory, withdraw everything it was built from, and
    // verify what moved.
    let world = World::new("portable", PASSING_CONFIG);
    let (bundle, outcome) = world.publish("portable");

    let relocated = std::env::temp_dir().join(format!(
        "firmwaresight-p4-relocated-{}-{}",
        std::process::id(),
        nanos()
    ));
    std::fs::create_dir_all(&relocated).expect("the second root exists");
    copy_tree(&bundle, &relocated.join("bundle")).expect("the bundle was copied");
    let copy = relocated.join("bundle");

    std::fs::remove_file(&world.elf).expect("the source ELF is withdrawn");
    std::fs::remove_file(&world.map).expect("the source MAP is withdrawn");
    std::fs::remove_file(&world.notes).expect("the notes are withdrawn");

    let verification = verify_bundle(&copy).expect("the relocated bundle verifies on its own");
    assert_eq!(verification.release_id, outcome.release_id);
    assert_eq!(verification.artifact_count, 2);
    assert!(!verification.carries_comparison);
    assert!(verification.carries_release_notes);
    assert_eq!(
        verification.file_count,
        world.tree(&copy).len(),
        "the verifier counted what is actually there"
    );

    let html =
        std::fs::read_to_string(copy.join("release-report.html")).expect("the report is there");
    for fragment in [
        world.dir.to_string_lossy().as_ref(),
        relocated.to_string_lossy().as_ref(),
        "build/firmware.elf",
    ] {
        assert!(!html.contains(fragment), "the report leaked `{fragment}`");
    }

    let _ = std::fs::remove_dir_all(&relocated);
}

#[test]
fn one_changed_byte_in_a_shipped_file_is_found_by_the_verifier() {
    let world = World::new("tamper", PASSING_CONFIG);
    let (bundle, _) = world.publish("tamper");
    let artifact = bundle.join("artifacts/firmware.elf");
    let mut bytes = std::fs::read(&artifact).expect("the artifact is there");
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    std::fs::write(&artifact, bytes).expect("the artifact was rewritten");

    let error = verify_bundle(&bundle).expect_err("a bundle whose bytes moved must not verify");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    assert!(
        error.to_string().contains("artifacts/firmware.elf"),
        "{error}"
    );
}

#[test]
fn the_preview_lists_every_file_the_bundle_will_hold_and_names_no_host_path() {
    let world = World::new("preview", PASSING_CONFIG);
    let plan = prepare(&world.request()).expect("prepared");
    let preview = plan.preview.clone();
    let rendered = format!("{preview:?}");

    assert_eq!(preview.disposition, "PASS");
    assert_eq!(preview.baseline_snapshot_id, None);
    assert_eq!(preview.snapshot_id, world.snapshot.id().as_str());
    assert_eq!(preview.gate_run_id, world.recomputed_run_id());
    assert!(
        preview
            .plan_id
            .starts_with(&format!("bundle-{}-", std::process::id()))
    );
    assert_eq!(preview.files.len(), 9, "§27's list covers the whole bundle");
    for file in &preview.files {
        assert!(
            is_safe_bundle_relative_path(&file.path),
            "the preview names `{}`",
            file.path
        );
        assert!(file.byte_size > 0, "`{}` is listed as empty", file.path);
        assert_eq!(
            file.sha256.len(),
            64,
            "§27: a planned digest is always known"
        );
    }
    assert_eq!(
        preview
            .files
            .iter()
            .filter(|file| file.role == FileRole::ShippedArtifact)
            .count(),
        2,
        "both current artifacts are listed as shipped"
    );
    assert_eq!(
        preview
            .files
            .iter()
            .filter(|file| file.role == FileRole::ChecksumIndex || file.role == FileRole::Manifest)
            .count(),
        2,
        "the two indexes are listed as themselves"
    );
    assert!(
        !rendered.contains(&world.dir.to_string_lossy().to_string()),
        "the preview carried the project root"
    );
    assert!(
        !rendered.contains("build/firmware"),
        "the preview carried a source path"
    );

    // §34: the document the CLI prints is the document the bundle carries.
    let (bundle, _) = world.publish("manifest-equals-stdout");
    let written =
        std::fs::read_to_string(bundle.join("release-manifest.json")).expect("the manifest");
    assert_eq!(plan.manifest_document(), written);
}

#[test]
fn a_configuration_that_found_something_to_say_reaches_the_preview() {
    // §27: the preview is where a release owner learns what the preparation noticed. The SBOM switch is the
    // one recorded-but-not-evaluated capability, so it is the cleanest warning to check the channel with.
    let world = World::new(
        "warnings",
        &format!("{PASSING_CONFIG}\n[sbom]\nenabled = true\n"),
    );
    let plan = prepare(&world.request()).expect("prepared");
    let warnings = plan.preview.warnings.join("\n");
    assert!(warnings.contains("SBOM"), "{warnings}");
    assert!(
        !warnings.contains(&world.dir.to_string_lossy().to_string()),
        "a warning carried a host path: {warnings}"
    );
    // And the bundle itself is unaffected: a warning is shown, not written into the portable documents.
    let (bundle, _) = world.publish("warnings");
    let html =
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there");
    assert!(
        !html.contains("SBOM is not an MVP Gate capability"),
        "a config warning was embedded in the report"
    );
}

// ------------------------------------------------- identity is the bytes on disk (P5 §32, L22)

/// What the release-notes digest actually measures, pinned as a test rather than as an argument.
///
/// `observe_release_notes` hashes the file it finds, so a checkout that rewrites LF to CRLF has not
/// changed the release's words — it has changed the release. Both identities move: the notes digest is a
/// line of the Gate's canonical input, and the Gate run id and the release id are digests of inputs that
/// contain it. §32 requires this to be documented *and tested*; the prose, the two options that would
/// change the semantics, and the STOP that keeps this round from choosing either are in
/// `P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md`.
#[test]
fn a_notes_file_that_differs_only_in_line_endings_is_a_different_release() {
    let lf = World::new("eol-lf", PASSING_CONFIG);
    let crlf = World::new("eol-crlf", PASSING_CONFIG);
    let crlf_bytes = NOTES.replace('\n', "\r\n");
    assert!(
        crlf_bytes != NOTES,
        "the fixture must actually differ, or this test proves nothing"
    );
    std::fs::write(&crlf.notes, crlf_bytes.as_bytes()).expect("the same words, in CRLF");

    let observed = |world: &World| match observe_release_notes(
        &world.loaded.root,
        &world.loaded.policy.release_notes_path,
    )
    .status
    {
        GateFileStatus::Present {
            sha256: Some(digest),
        } => digest,
        other => panic!("the notes file was not observed as present: {other:?}"),
    };
    let (lf_digest, crlf_digest) = (observed(&lf), observed(&crlf));

    // Bytes, not words: each recorded digest is the digest of exactly what is on disk, with no
    // normalization anywhere between the file and the hash.
    assert_eq!(lf_digest, digest_of_bytes(NOTES.as_bytes()));
    assert_eq!(crlf_digest, digest_of_bytes(crlf_bytes.as_bytes()));
    assert_ne!(
        lf_digest, crlf_digest,
        "identical words in a different line ending are the same evidence"
    );

    // Which moves both identities the product mints.
    assert_ne!(
        lf.recomputed_run_id(),
        crlf.recomputed_run_id(),
        "the notes digest is inside the Gate's canonical input, so the run id moves with it"
    );
    let lf_release = prepare(&lf.request())
        .expect("the LF release is offerable")
        .preview
        .release_id;
    let crlf_release = prepare(&crlf.request())
        .expect("the CRLF release is offerable")
        .preview
        .release_id;
    assert_ne!(
        lf_release, crlf_release,
        "and so the release this bundle names is a different release"
    );

    // What does *not* move is the verdict: same rules, same answers, different identity. Treating the
    // two as one question is what would let a normalization slip in as a "presentation fix".
    assert_eq!(
        lf.evaluate().overall_effective_severity,
        crlf.evaluate().overall_effective_severity
    );
    assert_eq!(lf.evaluate().counts, crlf.evaluate().counts);

    // And the bundle ships the bytes it hashed, so the artifact matches its own manifest rather than
    // being quietly rewritten on the way out.
    let (bundle, outcome) = crlf.publish("eol-crlf");
    assert_eq!(outcome.release_id, crlf_release);
    let shipped = std::fs::read(bundle.join("release-notes.md")).expect("the notes ship");
    assert_eq!(
        shipped,
        crlf_bytes.as_bytes(),
        "the bundle normalized the bytes it was built from"
    );
    verify_bundle(&bundle).expect("a CRLF bundle verifies against its own bytes");
}

// --------------------------------------------------------------------------- refusals (§8, §9, §10, §11)

#[test]
fn a_review_disposition_produces_no_bundle_at_all() {
    let mut world = World::new("review", REVIEWING_CONFIG);
    world.with_baseline();
    world.select_stored_run();
    assert_eq!(
        world.evaluate().aggregate_with_acceptances(&[]),
        firmwaresight_core::domain::gate::EffectiveSeverity::Review,
        "the fixture review is the thing being refused"
    );

    let error = prepare(&world.request()).expect_err("a REVIEW is not a release");
    assert_eq!(error.code(), "ERR-BUNDLE-6101");
    assert!(error.to_string().contains("REVIEW"), "{error}");
    assert_eq!(
        world.folder_contents(world.path()),
        vec!["RELEASE_NOTES.md", "build", "firmwaresight.toml"],
        "a refusal writes no directory into the project"
    );
}

#[test]
fn an_ordinary_block_disposition_produces_no_bundle() {
    let world = World::new(
        "block",
        &format!("{PASSING_CONFIG}\n[memory]\nflash_budget = 8\n"),
    );
    let error = prepare(&world.request()).expect_err("a BLOCK is not a release");
    assert_eq!(error.code(), "ERR-BUNDLE-6101");
    assert!(error.to_string().contains("BLOCK"), "{error}");
}

#[test]
fn an_accepted_unknown_is_refused_because_unknown_is_never_acceptable() {
    let mut world = World::new("unknown-acceptance", PASSING_CONFIG);
    world.set_git(None, false);
    // With no Git the version rule loses its evidence, so the run holds an UNKNOWN to try to accept.
    let version = world
        .evaluate()
        .finding(GateRuleId::VersionMatchesPolicy)
        .expect("every rule answers")
        .clone();
    assert_eq!(
        version.state,
        FindingState::Unknown,
        "the fixture yields an UNKNOWN"
    );
    world.acceptances.push(BundleAcceptance {
        finding_id: version.id.clone(),
        actor: "release-owner".to_owned(),
        accepted_at: "2026-10-01T08:12:44Z".to_owned(),
        reason: "we accept the missing evidence".to_owned(),
    });

    let error = prepare(&world.request()).expect_err("an evidence gap cannot be signed away");
    assert_eq!(error.code(), "ERR-INTERNAL-9004");
    assert!(error.to_string().contains("UNKNOWN"), "{error}");
}

#[test]
fn a_stale_gate_context_is_refused_with_6102_and_the_old_id_named() {
    let mut world = World::new("stale-context", PASSING_CONFIG);
    world.select_stored_run();
    let selected = world.selected_run_id.clone().expect("a run was selected");
    let plan = prepare(&world.request()).expect("prepared against the selected run");

    // The workspace moves after the preview: a file is edited, so the tree is dirty.
    world.set_git(Some("v1.4.2"), true);
    let error = plan
        .publish(&world.request(), &world.parent("stale"), false)
        .expect_err("a dirty workspace after the preview invalidates the plan");
    assert_eq!(error.code(), "ERR-BUNDLE-6102");
    let message = error.to_string();
    assert!(message.contains(&selected), "{message}");
    assert!(
        !message.contains(&world.dir.to_string_lossy().to_string()),
        "the host path reached a message: {message}"
    );
    assert!(
        !world
            .parent("stale")
            .join(plan.proposed_directory_name())
            .exists(),
        "a refused export publishes nothing"
    );
}

#[test]
fn a_release_with_nothing_to_resolve_its_version_fails_closed_with_6110() {
    let world = World::new(
        "no-version",
        r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf"]

[release]
require_clean_git = false
require_release_notes = false
"#,
    );
    let error = prepare(&world.request()).expect_err("a bundle needs a project release version");
    assert_eq!(error.code(), "ERR-BUNDLE-6110");
    assert!(
        error.remediation().contains("version"),
        "{}",
        error.remediation()
    );
}

#[test]
fn a_tag_capture_is_the_version_a_release_uses_when_no_expected_version_is_configured() {
    let world = World::new(
        "tag-capture",
        &PASSING_CONFIG.replace("expected_version = \"1.4.2\"\n", ""),
    );
    let plan = prepare(&world.request()).expect("the tag yields the capture");
    assert_eq!(plan.preview.release_version, "1.4.2");
    let document: serde_json::Value =
        serde_json::from_str(plan.manifest_document()).expect("the manifest is one document");
    assert_eq!(
        document["extensions"]["release_version_source"]
            .as_str()
            .expect("a version source"),
        "git_tag.capture"
    );
    let html = {
        let (bundle, _) = world.publish("capture");
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there")
    };
    assert!(
        html.contains("git_tag.capture"),
        "§43 names where the version came from"
    );
}

// --------------------------------------------------------------------------- source safety (§22, §24)

#[test]
fn a_changed_source_elf_is_refused_rather_than_shipped_under_the_old_digest() {
    let mut world = World::new("elf-changed", PASSING_CONFIG);
    world.select_stored_run();
    let plan = prepare(&world.request()).expect("prepared");
    append(&world.elf, b"one more byte in the image");

    let error = plan
        .publish(&world.request(), &world.parent("elf"), false)
        .expect_err("the bytes are no longer what the snapshot hashed");
    assert_eq!(error.code(), "ERR-BUNDLE-6103");
    assert!(error.to_string().contains("firmware.elf"), "{error}");
    assert!(
        error.remediation().contains("re-analyze"),
        "{}",
        error.remediation()
    );
    assert!(
        !error
            .to_string()
            .contains(&world.dir.to_string_lossy().to_string()),
        "the host path reached a message: {error}"
    );
    assert!(
        !world
            .parent("elf")
            .join(plan.proposed_directory_name())
            .exists()
    );
}

#[test]
fn a_changed_source_map_is_refused_too() {
    let world = World::new("map-changed", PASSING_CONFIG);
    let plan = prepare(&world.request()).expect("prepared");
    append(&world.map, b"x");
    let error = plan
        .publish(&world.request(), &world.parent("map"), false)
        .expect_err("a companion artifact is verified like any other");
    assert_eq!(error.code(), "ERR-BUNDLE-6103");
    assert!(error.to_string().contains("firmware.map"), "{error}");
}

#[test]
fn a_missing_source_artifact_is_refused_and_names_no_path() {
    let world = World::new("elf-missing", PASSING_CONFIG);
    let plan = prepare(&world.request()).expect("prepared");
    std::fs::remove_file(&world.elf).expect("the source ELF is withdrawn");
    let error = plan
        .publish(&world.request(), &world.parent("missing"), false)
        .expect_err("a withdrawn file cannot be shipped");
    assert_eq!(error.code(), "ERR-BUNDLE-6103");
    assert!(error.to_string().contains("is not there"), "{error}");
    assert!(!error.to_string().contains("build/firmware.elf"), "{error}");
}

#[test]
fn a_directory_in_place_of_an_artifact_is_refused() {
    let world = World::new("not-regular", PASSING_CONFIG);
    let plan = prepare(&world.request()).expect("prepared");
    std::fs::remove_file(&world.elf).expect("the source ELF is withdrawn");
    std::fs::create_dir_all(world.dir.join("build/firmware.elf"))
        .expect("a directory takes its place");
    let error = plan
        .publish(&world.request(), &world.parent("dir"), false)
        .expect_err("a directory is not the firmware");
    assert_eq!(error.code(), "ERR-BUNDLE-6103");
    assert!(error.to_string().contains("not a regular file"), "{error}");
}

#[test]
fn notes_that_move_after_the_preview_are_refused_with_6104() {
    let world = World::new("notes-moved", PASSING_CONFIG);
    let plan = prepare(&world.request()).expect("prepared");
    std::fs::write(
        &world.notes,
        "# 1.4.2\n\n- a different sentence about the same release\n",
    )
    .expect("the notes are rewritten");

    // No stored run was selected, so the recomputed context still passes on its own terms: the refusal is the
    // plan-level one, and it names the notes rather than the run.
    let error = plan
        .publish(&world.request(), &world.parent("notes"), false)
        .expect_err("the shipped notes must be the notes the Gate read");
    assert_eq!(error.code(), "ERR-BUNDLE-6104");
    assert!(error.to_string().contains("Release Notes"), "{error}");
}

#[test]
fn notes_that_move_under_a_selected_run_are_refused_as_a_changed_context() {
    let mut world = World::new("notes-context", PASSING_CONFIG);
    world.select_stored_run();
    let plan = prepare(&world.request()).expect("prepared");
    std::fs::write(&world.notes, "# 1.4.2\n\n- a different sentence\n")
        .expect("the notes are rewritten");
    let error = plan
        .publish(&world.request(), &world.parent("notes2"), false)
        .expect_err("a notes change moves the run id, and a stored run no longer matches");
    assert_eq!(error.code(), "ERR-BUNDLE-6102");
}

#[test]
fn the_shipped_notes_are_the_source_bytes_and_nothing_else() {
    let world = World::new("notes-exact", PASSING_CONFIG);
    let (bundle, _) = world.publish("notes-exact");
    assert_eq!(
        std::fs::read(bundle.join("release-notes.md")).expect("the notes shipped"),
        std::fs::read(&world.notes).expect("the source notes are still there"),
        "§24: a byte-for-byte copy, not a rendered or trimmed one"
    );
    let html =
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there");
    assert!(
        !html.contains("the modem driver is now in the image"),
        "the notes were embedded, which §24 and §25 both refuse"
    );
}

#[test]
fn a_release_the_policy_does_not_require_notes_for_omits_the_file_and_says_so() {
    let world = World::new(
        "no-notes",
        &PASSING_CONFIG.replace(
            "require_release_notes = true",
            "require_release_notes = false",
        ),
    );
    std::fs::remove_file(&world.notes).expect("the notes are withdrawn");
    let (bundle, _) = world.publish("no-notes");
    assert!(!world.tree(&bundle).contains(&"release-notes.md".to_owned()));
    let manifest = read_manifest(&bundle);
    assert!(
        manifest["release"]["notes"].is_null(),
        "§24: an absent requirement is null, not an empty string"
    );
    let sums = std::fs::read_to_string(bundle.join("SHA256SUMS")).expect("SHA256SUMS is there");
    assert!(!sums.contains("release-notes.md"));
}

// --------------------------------------------------------------------------- reviews (§10)

#[test]
fn an_accepted_review_ships_the_review_its_acceptance_and_the_comparison() {
    let mut world = World::new("accepted", REVIEWING_CONFIG);
    world.with_baseline();
    world.accept(GateRuleId::BaselineGrowth);
    world.select_stored_run();

    let (bundle, outcome) = world.publish("accepted");
    assert!(!outcome.replaced);
    assert_eq!(outcome.release_version, "1.4.2");
    assert_eq!(
        world
            .tree(&bundle)
            .iter()
            .filter(|name| name.starts_with("artifacts/"))
            .count(),
        2
    );

    let reviews: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(bundle.join("accepted-reviews.json")).expect("reviews are there"),
    )
    .expect("one document");
    let rows = reviews["acceptances"].as_array().expect("acceptances");
    assert_eq!(
        rows.len(),
        1,
        "§10: the acceptance that caused the PASS is in the bundle"
    );
    assert_eq!(
        rows[0]["actor"].as_str().expect("an actor"),
        "release-owner"
    );
    assert_eq!(
        rows[0]["original_state"].as_str().expect("a state"),
        "REVIEW",
        "the record names what was accepted"
    );
    assert_eq!(
        rows[0]["accepted_at"].as_str().expect("a time"),
        "2026-10-01T08:12:44Z",
        "an audit time already in the record travels with it (§25)"
    );
    assert_eq!(
        reviews["extensions"]["overall_effective_severity"]
            .as_str()
            .expect("an aggregate"),
        "PASS"
    );

    let gate: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(bundle.join("gate-results.json")).expect("gate results are there"),
    )
    .expect("one document");
    let growth = gate["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|finding| finding["rule_id"] == "diff.growth")
        .expect("the growth rule answered");
    assert_eq!(
        growth["state"].as_str().expect("a state"),
        "REVIEW",
        "§10: the finding keeps its own state; only the aggregate moved"
    );
    assert_eq!(
        growth["id"].as_str().expect("an id"),
        rows[0]["finding_id"].as_str().expect("the accepted id")
    );
    assert!(world.tree(&bundle).contains(&"diff.json".to_owned()));
    let html =
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there");
    assert!(
        html.contains("release-owner"),
        "the review is visible in the report"
    );
    assert!(
        html.contains("modem driver"),
        "the reviewer's reason is visible"
    );
    verify_bundle(&bundle).expect("the accepted-review bundle verifies");
}

#[test]
fn an_edited_acceptance_after_the_preview_invalidates_the_plan() {
    let mut world = World::new("acceptance-stale", REVIEWING_CONFIG);
    world.with_baseline();
    world.accept(GateRuleId::BaselineGrowth);
    world.select_stored_run();
    let plan = prepare(&world.request()).expect("prepared");

    world.acceptances[0].reason = "the growth is the new modem driver, reviewed twice".to_owned();
    let error = plan
        .publish(&world.request(), &world.parent("accept"), false)
        .expect_err("the acceptance set is part of what the preview promised");
    assert_eq!(error.code(), "ERR-BUNDLE-6105");
    assert!(error.to_string().contains("acceptance set"), "{error}");
}

#[test]
fn an_empty_review_list_is_a_fact_the_bundle_states_rather_than_omits() {
    let world = World::new("no-reviews", PASSING_CONFIG);
    let (bundle, _) = world.publish("no-reviews");
    let reviews: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(bundle.join("accepted-reviews.json"))
            .expect("§10: the document is always included"),
    )
    .expect("one document");
    assert_eq!(
        reviews["acceptances"]
            .as_array()
            .expect("acceptances")
            .len(),
        0
    );
    assert_eq!(
        reviews["run_id"].as_str().expect("a run"),
        world.recomputed_run_id()
    );
    let html =
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there");
    assert!(
        html.contains("No review was accepted"),
        "an empty list is stated as a fact, not shown as an empty table"
    );
}

// --------------------------------------------------------------------------- destinations (§30, §31, §32)

#[test]
fn an_existing_destination_is_refused_until_the_release_owner_confirms() {
    let world = World::new("exists", PASSING_CONFIG);
    let (bundle, _) = world.publish("exists");
    let before = std::fs::read(bundle.join("release-manifest.json")).expect("the first manifest");

    let plan = prepare(&world.request()).expect("prepared again");
    let error = plan
        .publish(&world.request(), &world.parent("exists"), false)
        .expect_err("a second export over the same name needs confirmation");
    assert_eq!(error.code(), "ERR-BUNDLE-6106");
    assert!(
        error.to_string().contains(plan.proposed_directory_name()),
        "{error}"
    );
    assert_eq!(
        std::fs::read(bundle.join("release-manifest.json")).expect("the first manifest"),
        before,
        "a refused export leaves the bundle that is there exactly as it was"
    );
    assert_eq!(
        world.folder_contents(&world.parent("exists")),
        vec![plan.proposed_directory_name()],
        "a refused export leaves no staging sibling either"
    );
}

#[test]
fn an_arbitrary_directory_is_never_replaced_even_when_overwrite_is_true() {
    let world = World::new("not-a-bundle", PASSING_CONFIG);
    let parent = world.parent("arbitrary");
    let plan = prepare(&world.request()).expect("prepared");
    let proposed = plan.proposed_directory_name().to_owned();
    std::fs::create_dir_all(parent.join(&proposed)).expect("somebody else's directory exists");
    std::fs::write(
        parent.join(&proposed).join("notes.txt"),
        b"do not delete this",
    )
    .expect("and it holds somebody else's file");

    let error = plan.publish(&world.request(), &parent, true).expect_err(
        "`overwrite` is not permission to destroy a directory this engine did not write",
    );
    assert_eq!(error.code(), "ERR-BUNDLE-6107");
    assert_eq!(
        std::fs::read(parent.join(&proposed).join("notes.txt")).expect("the file is untouched"),
        b"do not delete this"
    );
    assert_eq!(
        world.folder_contents(&parent),
        vec![proposed],
        "no staging sibling was created for a refused replacement"
    );
}

#[test]
fn a_recognized_bundle_is_replaced_and_its_predecessor_discarded_only_after_success() {
    let world = World::new("replace", PASSING_CONFIG);
    let (bundle, first) = world.publish("replace");
    let parent = world.parent("replace");

    // Re-exporting the same release: the destination name is derived from the release id, so this is the
    // case a release owner actually meets — "I already exported this, export it again".
    let plan = prepare(&world.request()).expect("prepared again");
    assert_eq!(plan.proposed_directory_name(), first.directory_name);
    let outcome = plan
        .publish(&world.request(), &parent, true)
        .expect("a recognized bundle is replaced under explicit authorization");

    assert!(outcome.replaced, "the engine says what it did");
    assert_eq!(outcome.release_id, first.release_id);
    assert_eq!(
        world.folder_contents(&parent),
        vec![outcome.directory_name.clone()],
        "no backup or staging sibling is left once the new bundle verified"
    );
    verify_bundle(&bundle).expect("the replacement verifies");
    assert_eq!(
        world.tree(&bundle),
        vec![
            "SHA256SUMS",
            "accepted-reviews.json",
            "analysis.json",
            "artifacts/firmware.elf",
            "artifacts/firmware.map",
            "gate-results.json",
            "release-manifest.json",
            "release-notes.md",
            "release-report.html",
        ]
    );
}

#[test]
fn a_destination_that_cannot_be_written_publishes_nothing() {
    let world = World::new("write-failed", PASSING_CONFIG);
    let blocker = world.dir.join("blocker");
    std::fs::write(&blocker, b"not a directory")
        .expect("a file stands where a folder was asked for");

    let plan = prepare(&world.request()).expect("prepared");
    let error = plan
        .publish(&world.request(), &blocker.join("out"), false)
        .expect_err("a path through a file cannot hold a bundle");
    assert_eq!(error.code(), "ERR-BUNDLE-6108");
    assert!(!blocker.join("out").exists());
    assert!(
        !error
            .to_string()
            .contains(&world.dir.to_string_lossy().to_string()),
        "the host path reached a message: {error}"
    );
}

#[test]
fn a_bundle_directory_is_recognizable_and_anything_else_is_not() {
    let world = World::new("recognize", PASSING_CONFIG);
    let (bundle, _) = world.publish("recognize");
    assert!(
        is_recognizable_bundle(&bundle).expect("the directory is readable"),
        "§32: the bundle a previous publish wrote is recognizable"
    );

    let other = world.dir.join("not-a-bundle");
    std::fs::create_dir_all(&other).expect("the directory exists");
    assert!(!is_recognizable_bundle(&other).expect("the directory is readable"));
    std::fs::write(other.join("release-manifest.json"), b"{ not json").expect("a broken manifest");
    assert!(!is_recognizable_bundle(&other).expect("the file is readable"));
    std::fs::write(
        other.join("release-manifest.json"),
        r#"{"schema_version":1,"generated_by":{"product":"Something Else","version":"9"},"release":{"id":"release-1111111111111111111111111111111111111111111111111111111111111111"}}"#,
    )
    .expect("a manifest from another producer");
    assert!(
        !is_recognizable_bundle(&other).expect("the file is readable"),
        "a manifest naming a different producer is not this product's bundle"
    );
}

// --------------------------------------------------------------------------- verification (§44)

#[test]
fn a_stray_file_in_the_bundle_is_refused_rather_than_indexed() {
    let world = World::new("stray", PASSING_CONFIG);
    let (bundle, _) = world.publish("stray");
    std::fs::write(bundle.join("build.log"), b"something happened").expect("a stray file appears");

    let error = verify_bundle(&bundle).expect_err("a bundle carries only its own layout");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    assert!(error.to_string().contains("build.log"), "{error}");
}

#[test]
fn a_sum_line_that_is_not_the_recorded_form_is_refused() {
    let world = World::new("bad-sums", PASSING_CONFIG);
    let (bundle, _) = world.publish("bad-sums");
    let sums = bundle.join("SHA256SUMS");
    let text = std::fs::read_to_string(&sums).expect("SHA256SUMS is there");
    std::fs::write(&sums, text.replace("  analysis.json", " analysis.json"))
        .expect("the line is rewritten in the one-space form");

    let error = verify_bundle(&bundle).expect_err("§21's format is what makes the file parseable");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
}

#[test]
fn a_file_that_no_index_lists_is_refused() {
    let world = World::new("partial", PASSING_CONFIG);
    let (bundle, _) = world.publish("partial");
    std::fs::write(bundle.join("extra.elf"), b"an artifact nobody indexed")
        .expect("an extra artifact");
    let sums = bundle.join("SHA256SUMS");
    std::fs::write(
        &sums,
        format!(
            "{}{}  extra.elf\n",
            std::fs::read_to_string(&sums).expect("the sums file"),
            "0".repeat(64)
        ),
    )
    .expect("and SHA256SUMS claims it");

    let error = verify_bundle(&bundle).expect_err("the two lists have to cover what is there");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    assert!(error.to_string().contains("extra.elf"), "{error}");
}

#[test]
fn a_bundle_claiming_a_release_id_its_documents_do_not_add_up_to_is_refused() {
    // The strongest check in §44: the release id is re-derived from the shipped bytes. A manifest whose id
    // was swapped is refused even though no list hashes the manifest itself (§20).
    let world = World::new("identity", PASSING_CONFIG);
    let (bundle, _) = world.publish("identity");
    let manifest_path = bundle.join("release-manifest.json");
    let mut document = read_manifest(&bundle);
    let other = format!("release-{}", "9".repeat(64));
    document["release"]["id"] = serde_json::Value::String(other.clone());
    std::fs::write(
        &manifest_path,
        format!("{}\n", firmwaresight_report::render::to_json(&document)),
    )
    .expect("the manifest is rewritten");

    let error =
        verify_bundle(&bundle).expect_err("a swapped release id must not read as a release");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    assert!(error.to_string().contains(&other), "{error}");
}

#[test]
fn a_manifest_carrying_a_key_its_contract_never_declared_is_refused() {
    // §44's "manifest schema valid". Nothing hashes the manifest (§20), and its own entry list still agrees
    // with the bytes, so the contract is the only thing in the verifier that can notice this.
    let world = World::new("manifest-contract", PASSING_CONFIG);
    let (bundle, _) = world.publish("manifest-contract");
    let mut document = read_manifest(&bundle);
    document["unexpected"] = serde_json::Value::Bool(true);
    write_document(&bundle, "release-manifest.json", &document);

    let error = verify_bundle(&bundle).expect_err("a manifest is valid, not merely parseable");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    let detail = error.to_string();
    assert!(detail.contains("release-manifest.json"), "{detail}");
    assert!(detail.contains("unexpected"), "{detail}");
}

#[test]
fn a_document_under_an_unreadable_major_is_refused_by_version_not_by_a_page_of_failures() {
    // A bundle written under another major is one clear answer, not a recital of every key the new contract
    // moved. The identity check runs before the contract for exactly this reason.
    let world = World::new("major", PASSING_CONFIG);
    let (bundle, _) = world.publish("major");
    let mut document = read_manifest(&bundle);
    document["schema_version"] = serde_json::Value::from(2);
    write_document(&bundle, "release-manifest.json", &document);

    let error = verify_bundle(&bundle).expect_err("this build reads major 1");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    let detail = error.to_string();
    assert!(detail.contains("major 2"), "{detail}");
    assert!(!detail.contains("must equal"), "{detail}");
}

#[test]
fn a_document_that_matches_both_indexes_yet_breaks_its_contract_is_still_refused() {
    // The honest version of tampering: a reader who edits `analysis.json` and re-records its digest in
    // `SHA256SUMS` and in the manifest defeats both hash layers. What is left is the contract, and §44 makes
    // it a check rather than a document someone is free to read later.
    let world = World::new("reindexed", PASSING_CONFIG);
    let (bundle, _) = world.publish("reindexed");
    let path = bundle.join("analysis.json");
    let mut document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the analysis is there"))
            .expect("one document");
    document["memory"] = serde_json::Value::Number(7.into());
    std::fs::write(
        &path,
        firmwaresight_report::render::document_json(&document),
    )
    .expect("the analysis is rewritten");
    reindex(&bundle, "analysis.json");

    let error = verify_bundle(&bundle).expect_err("agreement between the lists is not validity");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    let detail = error.to_string();
    assert!(detail.contains("analysis.json"), "{detail}");
    assert!(!detail.contains("hashes to"), "{detail}");
}

#[test]
fn a_bundle_that_no_longer_aggregates_to_pass_is_refused() {
    // §8 read from the reader's side: `accepted-reviews.json` must still say PASS.
    let mut world = World::new("aggregate", REVIEWING_CONFIG);
    world.with_baseline();
    world.accept(GateRuleId::BaselineGrowth);
    let (bundle, _) = world.publish("aggregate");

    let path = bundle.join("accepted-reviews.json");
    let mut document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the reviews are there"))
            .expect("one document");
    document["extensions"]["overall_effective_severity"] =
        serde_json::Value::String("REVIEW".to_owned());
    std::fs::write(
        &path,
        format!("{}\n", firmwaresight_report::render::to_json(&document)),
    )
    .expect("the aggregate is rewritten");

    // The lists are inconsistent with the bytes as well, so any of §44's checks may speak first. What matters
    // is that the bundle is refused rather than trusted.
    let error = verify_bundle(&bundle).expect_err("a bundle is verified, not assumed");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
}

#[test]
fn a_bundle_with_an_empty_artifacts_folder_is_refused() {
    let world = World::new("empty-artifacts", PASSING_CONFIG);
    let (bundle, _) = world.publish("empty-artifacts");
    std::fs::remove_dir_all(bundle.join("artifacts")).expect("the artifact folder is removed");
    let error = verify_bundle(&bundle).expect_err("a release ships a firmware");
    assert_eq!(error.code(), "ERR-BUNDLE-6109");
    assert!(error.to_string().contains("artifacts/"), "{error}");
}

// --------------------------------------------------------------------------- identity (§12)

#[test]
fn the_release_id_moves_when_a_review_is_accepted_and_not_when_only_the_folder_changes() {
    let mut world = World::new("identity-inputs", REVIEWING_CONFIG);
    world.with_baseline();
    let error =
        prepare(&world.request()).expect_err("a REVIEW with no acceptance is not a release");
    assert_eq!(error.code(), "ERR-BUNDLE-6101");

    world.accept(GateRuleId::BaselineGrowth);
    let first = prepare(&world.request()).expect("the same state with an acceptance");
    let first_id = first.preview.release_id.clone();
    let second_parent = world.parent("identity");
    std::fs::create_dir_all(&second_parent).expect("the destination parent exists");
    let again = prepare(&world.request()).expect("prepared once more");
    assert_eq!(
        again.preview.release_id, first_id,
        "the same inputs name the same release"
    );

    world.acceptances[0].actor = "someone-else".to_owned();
    let third = prepare(&world.request()).expect("a different actor is still a review");
    assert_ne!(
        third.preview.release_id, first_id,
        "§12: a changed acceptance set is a different release"
    );
}

#[test]
fn the_gate_facts_a_bundle_ships_are_the_ones_the_adapter_computed() {
    // §9's reuse promise, checked rather than assumed: the engine's recomputed run id equals the P3
    // adapter's, so a stored run the desktop selected is comparable with what preparation produces.
    let world = World::new("reuse", PASSING_CONFIG);
    let plan = prepare(&world.request()).expect("prepared");
    assert_eq!(plan.preview.gate_run_id, world.recomputed_run_id());
    assert_eq!(
        policy_sha256(&world.loaded.policy),
        world.loaded.policy_sha256,
        "the loaded project's own fingerprint is the one the adapter derives"
    );
    let document: serde_json::Value =
        serde_json::from_str(plan.manifest_document()).expect("the manifest is one document");
    assert_eq!(
        document["extensions"]["policy_sha256"]
            .as_str()
            .expect("a policy digest"),
        world.loaded.policy_sha256
    );
    assert_eq!(
        document["build"]["git_commit"].as_str().expect("a head"),
        HEAD,
        "a workspace fact, labelled as one (§43)"
    );
    assert_eq!(
        document["build"]["artifact_sha256"]
            .as_str()
            .expect("a primary artifact digest"),
        digest_of(&world.elf).hex()
    );
    assert_eq!(
        document["build"]["git_dirty"].as_bool(),
        Some(false),
        "the workspace was observed clean"
    );
    assert_eq!(document["schema_version"].as_u64().expect("a major"), 1);

    let html = {
        let (bundle, _) = world.publish("reuse");
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there")
    };
    assert!(
        html.contains("Workspace HEAD observed by the Release Gate"),
        "§43's wording, not a provenance claim"
    );
    for banned in ["trusted", "authentic", "signed", "tamper"] {
        assert!(
            !html.to_lowercase().contains(banned),
            "§62: the report says `{banned}`"
        );
    }
}

// --------------------------------------------------------------------------- helpers

fn append(path: &Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .expect("the source file opens");
    file.write_all(bytes).expect("the file is appended to");
    file.sync_all().expect("the append is on the disk");
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.metadata()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// The engine's own error type is what a caller maps onto exit codes and UI cards, so the variants a test
/// asserts on have to keep compiling as the type changes.
#[test]
fn every_refusal_a_caller_can_branch_on_has_a_code_and_a_remediation() {
    let cases = [
        BundleError::GateContextChanged {
            selected: "gate-1".to_owned(),
            recomputed: "gate-2".to_owned(),
        },
        BundleError::SourceArtifactChanged {
            name: "firmware.elf".to_owned(),
            detail: "its SHA-256 is not the digest the snapshot holds".to_owned(),
        },
        BundleError::ReleaseNotesChanged {
            detail: "the bytes moved".to_owned(),
        },
        BundleError::PlanStale {
            detail: "the acceptance set changed".to_owned(),
        },
        BundleError::DestinationExists {
            display: "motor-controller-1.4.2-abcdef012345".to_owned(),
        },
        BundleError::UnsafeDestination {
            display: "my-notes".to_owned(),
            detail: "no manifest".to_owned(),
        },
        BundleError::WriteFailed {
            name: "SHA256SUMS".to_owned(),
            detail: "no space".to_owned(),
        },
        BundleError::VerificationFailed {
            detail: "a digest disagreed".to_owned(),
        },
        BundleError::Internal {
            detail: "a row was not in the context".to_owned(),
        },
    ];
    let codes: Vec<&str> = cases.iter().map(BundleError::code).collect();
    assert_eq!(
        codes,
        vec![
            "ERR-BUNDLE-6102",
            "ERR-BUNDLE-6103",
            "ERR-BUNDLE-6104",
            "ERR-BUNDLE-6105",
            "ERR-BUNDLE-6106",
            "ERR-BUNDLE-6107",
            "ERR-BUNDLE-6108",
            "ERR-BUNDLE-6109",
            "ERR-INTERNAL-9004",
        ]
    );
    for error in &cases {
        assert!(!error.remediation().is_empty(), "{error}");
        assert!(!error.to_string().is_empty());
    }
}
