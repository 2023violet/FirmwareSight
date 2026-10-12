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
    prepare_with_attachments, verify_bundle,
};
use firmwaresight_project::evidence::{
    AttachmentError, AttachmentSelection, GateRunRequest, ReleaseAttachment, build_context,
    build_context_with_attachments, growth_facts, observe_attachment, observe_release_notes,
};
use firmwaresight_project::{GitObservation, LoadedProject, SnapshotFacts, policy_sha256, run_id};
use firmwaresight_report::release::ManifestError;

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

/// A release whose policy demands the two attached kinds beside the two analyzed ones. The separation is the
/// whole point of the config: `elf` and `map` can only be satisfied by the build that was analyzed, `bin` and
/// `hex` only by a file the release owner attached (`ADR-0030` D-3), so a bundle for this project cannot be
/// published unless both evidence classes are present and bound.
const ATTACHING_CONFIG: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map", "bin", "hex"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"
"#;

/// The attaching policy **and** a growth threshold the fixture pair is over, so one release carries both a
/// requirement only an attachment can satisfy and a `REVIEW` only an acceptance can dispose of. `C1-U2R1`
/// items 1 and 2 are both about how those two facts interact, which neither single-lesson config can reach.
const ATTACHING_REVIEWING_CONFIG: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map", "bin", "hex"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"

[diff]
flash_growth_review_bytes = 100
"#;

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
    /// The snapshot's Gate facts, kept beside it because `gate_request` borrows them.
    target: SnapshotFacts,
    comparison: Option<DiffResult>,
    acceptances: Vec<BundleAcceptance>,
    selected_run_id: Option<String>,
    elf: PathBuf,
    map: PathBuf,
    notes: PathBuf,
    /// The three files this release may attach, written when the world is built so every test starts from the
    /// same bytes on disk.
    bin: PathBuf,
    hex: PathBuf,
    mystery: PathBuf,
    /// The selection list the release owner has chosen, in the order they chose it.
    attachments: Vec<AttachmentSelection>,
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
        let target = SnapshotFacts::from_snapshot(&snapshot);

        // The attached files are real regular files with distinct bytes, because what is under test is that
        // the bundle ships the bytes a hash was taken of — a mocked observation would prove nothing about that.
        // Under `build/`, beside the real build output, and not in a new top-level entry: the tests that
        // prove a refusal leaves the project folder alone read that folder's listing.
        let attached = dir.join("build/attached");
        std::fs::create_dir_all(&attached).expect("the attachment folder exists");
        let bin = attached.join("app.bin");
        let hex = attached.join("boot.hex");
        let mystery = attached.join("mystery.dat");
        std::fs::write(&bin, b"BIN\xff\x00image bytes").expect("the BIN is written");
        std::fs::write(&hex, b":020000000100FA\r\n:00000001FF\r\n").expect("the HEX is written");
        std::fs::write(&mystery, b"what this is, nobody says")
            .expect("the unknown file is written");

        Self {
            dir,
            loaded,
            git: observation("v1.4.2", false),
            snapshot,
            target,
            comparison: None,
            acceptances: Vec::new(),
            selected_run_id: None,
            elf,
            map,
            notes,
            bin,
            hex,
            mystery,
            attachments: Vec::new(),
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
    ///
    /// Whatever this world has attached is bound into it, so a test that compares a run id against the one the
    /// bundle computes is comparing the same two things the release owner is.
    fn context(&self) -> GateContext {
        build_context_with_attachments(&self.gate_request(), &self.attachment_rows())
    }

    /// The same request the bundle assembles its context from, with this world's attachments in it.
    fn gate_request(&self) -> GateRunRequest<'_> {
        let growth = match self.comparison.as_ref() {
            Some(diff) => growth_facts(diff),
            None => GateGrowthFacts::without_baseline(),
        };
        let notes =
            observe_release_notes(&self.loaded.root, &self.loaded.policy.release_notes_path);
        GateRunRequest {
            target: &self.target,
            growth,
            git: &self.git,
            policy: self.loaded.policy.clone(),
            release_notes: Some(notes),
        }
    }

    /// One attached file, chosen the way a release owner chooses it: a path and a declared kind.
    fn attach(&mut self, path: &Path, kind: ArtifactKind) {
        self.attachments.push(AttachmentSelection {
            path: path.to_path_buf(),
            declared_kind: kind,
        });
    }

    /// This world's attachments as this crate's own observation produces them. The engine calls the same
    /// function, so a run id a test computes is the run id the bundle will recompute rather than an
    /// independently assembled look-alike.
    fn attachment_rows(&self) -> Vec<ReleaseAttachment> {
        self.attachments
            .iter()
            .map(|selection| {
                observe_attachment(&selection.path, selection.declared_kind)
                    .expect("an attached file is a readable regular file")
            })
            .collect()
    }

    /// The run id an attachment-free context would produce, for the tests that prove an attachment moves it.
    fn unattached_run_id(&self) -> String {
        run_id(&build_context(&self.gate_request()))
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

    /// The same two calls with this world's attachment list handed to both, which is what a release owner who
    /// attached files does: one selection, previewed and exported.
    fn publish_attached(&self, label: &str) -> (PathBuf, bundle::BundleOutcome) {
        let parent = self.parent(label);
        std::fs::create_dir_all(&parent).expect("the destination parent exists");
        let plan = prepare_with_attachments(&self.request(), &self.attachments)
            .expect("the attached release is prepared");
        let outcome = plan
            .publish_with_attachments(&self.request(), &self.attachments, &parent, false)
            .expect("the attached bundle is published");
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
        // The two attachment arms, each with the code its own layer registered. A caller that branches on a
        // refusal must be able to tell "this file could not be admitted" from "the set moved after the
        // preview", and both of those from the six codes that predate attachments.
        BundleError::from(AttachmentError::Unreadable {
            name: "app.bin".to_owned(),
            detail: "the chosen path is not a regular file".to_owned(),
        }),
        BundleError::from(AttachmentError::KindNotAttaching {
            kind: ArtifactKind::Elf,
        }),
        BundleError::from(AttachmentError::Empty {
            name: "app.bin".to_owned(),
        }),
        BundleError::AttachmentSetChanged {
            name: "app.bin".to_owned(),
            change: "added",
        },
        // The folded-selection refusal `C1-U2R1` §B separates from the integrity sentence. It keeps `6109`, so
        // a caller that branches on the code sees no change — only the words a person reads do.
        BundleError::DuplicateBundleName {
            name: "artifacts/app.bin".to_owned(),
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
            "ERR-BUNDLE-6115",
            "ERR-BUNDLE-6116",
            "ERR-BUNDLE-6117",
            "ERR-BUNDLE-6118",
            "ERR-BUNDLE-6109",
        ]
    );
    for error in &cases {
        assert!(!error.remediation().is_empty(), "{error}");
        assert!(!error.to_string().is_empty());
    }
}

// ------------------------------------------------------------------ C1-U2: attached bytes in a bundle
//
// Everything above proves a bundle of analyzed artifacts. Everything below proves the second evidence class
// travels through the same engine on the same terms: the file a person attached is hashed by the same
// observation a Gate run performs, bound into the same identity, judged by the same rules, copied into the
// same `artifacts/` folder, indexed by both hash lists, disclosed for what it is, and refused — before a byte
// is written — when any of those stops being true (`04_TECH/28` §3, `ADR-0030` D-3/D-4/D-7/D-8).

/// One release that needs both attached kinds, with both chosen, as a release owner would.
fn attached_world(label: &str) -> World {
    let mut world = World::new(label, ATTACHING_CONFIG);
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world
}

fn read_document(bundle: &Path, name: &str) -> serde_json::Value {
    let text = std::fs::read_to_string(bundle.join(name)).expect("the document is there");
    serde_json::from_str(&text).expect("the document is one JSON document")
}

/// The paths one bundle's manifest indexes.
fn manifest_files(bundle: &Path) -> Vec<String> {
    read_manifest(bundle)["files"]
        .as_array()
        .expect("files is a list")
        .iter()
        .filter_map(|row| row["path"].as_str().map(str::to_owned))
        .collect()
}

/// The `artifacts/` leaves a bundle holds.
fn shipped_artifacts(bundle: &Path) -> Vec<String> {
    std::fs::read_dir(bundle.join("artifacts"))
        .expect("the artifacts folder is there")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect()
}

/// L-1, the link that makes the other five matter: a BIN requirement is satisfied by the attached file's
/// bytes, and the bundle that ships is the one that judgement produced.
#[test]
fn a_release_that_requires_bin_and_hex_passes_only_when_both_are_attached_and_bound() {
    let mut world = attached_world("u2-l1-pass");
    world.select_stored_run();
    let (bundle, outcome) = world.publish_attached("l1");

    let evaluation = world.evaluate();
    let required = evaluation
        .finding(GateRuleId::RequiredArtifacts)
        .expect("the rule always answers");
    assert_eq!(required.state, FindingState::Pass, "{}", required.summary);
    // Each requirement is cited by the class that answers it, in the scheme that names that class.
    let bin_digest = digest_of(&world.bin);
    let hex_digest = digest_of(&world.hex);
    assert!(
        required
            .evidence_refs
            .iter()
            .any(|r| r == &format!("attachment:bin:{}", bin_digest.hex())),
        "{:?}",
        required.evidence_refs
    );
    assert!(
        required
            .evidence_refs
            .iter()
            .any(|r| r == &format!("attachment:hex:{}", hex_digest.hex())),
        "{:?}",
        required.evidence_refs
    );

    // The verdict the document carries is the verdict the run computed, and the shipped folder holds both
    // attached files beside the analyzed pair.
    let gate = read_document(&bundle, "gate-results.json");
    assert_eq!(gate["run_id"].as_str(), Some(outcome.gate_run_id.as_str()));
    let mut names = shipped_artifacts(&bundle);
    names.sort();
    assert_eq!(
        names,
        vec!["app.bin", "boot.hex", "firmware.elf", "firmware.map"]
    );

    // And the shipped bytes are the attached bytes, not a copy of something the engine re-read later.
    for (name, source) in [
        ("app.bin", &world.bin),
        ("boot.hex", &world.hex),
        ("firmware.elf", &world.elf),
    ] {
        assert_eq!(
            digest_of(&bundle.join("artifacts").join(name)),
            digest_of(source),
            "`{name}` is not the file it was judged from"
        );
    }
}

/// L-1's negative half, and the reason the positive one means anything: the same policy with the attached
/// kinds unavailable produces no bundle at all.
#[test]
fn a_required_bin_with_nothing_attached_produces_no_bundle_at_all() {
    let world = World::new("u2-l1-block", ATTACHING_CONFIG);
    let error = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("nothing supplies the BIN or the HEX");
    assert_eq!(error.code(), "ERR-BUNDLE-6101");
    assert!(
        !world.parent("l1block").exists(),
        "a refused release created a destination folder"
    );
}

/// L-2: the attached bytes are part of the run identity, so a release cannot ship a verdict computed over a
/// different set of files.
#[test]
fn attaching_a_file_moves_the_run_id_and_leaves_the_snapshot_alone() {
    let mut world = World::new("u2-l2-identity", ATTACHING_CONFIG);
    let before = world.recomputed_run_id();
    assert_eq!(before, world.unattached_run_id());
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    let after = world.recomputed_run_id();

    assert_ne!(before, after, "an attached file changed no identity");
    assert_eq!(
        world.snapshot.id().as_str(),
        world.target.snapshot_id.as_str(),
        "attaching a file is not re-analyzing a build"
    );
    world.select_stored_run();
    let (_, outcome) = world.publish_attached("l2");
    assert_eq!(outcome.gate_run_id, after);
    assert_ne!(outcome.gate_run_id, before);
}

/// L-2's determinism half: two paths holding the same bytes are one row in the identity and two files in the
/// bundle. The identity binds a `(kind, digest)` pair once; the bundle ships each occurrence it was given,
/// verified on its own path (M2).
#[test]
fn identical_bytes_at_two_paths_are_one_identity_row_and_two_shipped_files() {
    let mut world = World::new("u2-m2-duplicate", ATTACHING_CONFIG);
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    let copy = world.dir.join("build/attached/boot-copy.hex");
    std::fs::copy(&hex, &copy).expect("the second path holds the same bytes");
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world.attach(&copy, ArtifactKind::IntelHex);

    let mut once = World::new("u2-m2-once", ATTACHING_CONFIG);
    let other_bin = once.bin.clone();
    let other_hex = once.hex.clone();
    once.attach(&other_bin, ArtifactKind::Bin);
    once.attach(&other_hex, ArtifactKind::IntelHex);
    assert_eq!(
        world.recomputed_run_id(),
        once.recomputed_run_id(),
        "a second copy of the same bytes moved an identity that binds each (kind, digest) pair once"
    );

    world.select_stored_run();
    let (bundle, _) = world.publish_attached("m2");
    let mut names = shipped_artifacts(&bundle);
    names.sort();
    assert_eq!(
        names,
        vec![
            "app.bin",
            "boot-copy.hex",
            "boot.hex",
            "firmware.elf",
            "firmware.map"
        ],
        "both occurrences of the same bytes ship, each verified on its own path"
    );
}

/// M2's other half: the same file offered twice is the caller repeating themselves, and the engine ships what
/// they meant — once, with the one name it would give that path.
#[test]
fn the_same_path_offered_twice_ships_once() {
    let mut world = World::new("u2-m2-twice-same-path", ATTACHING_CONFIG);
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world.attach(&bin, ArtifactKind::Bin);
    world.select_stored_run();

    let (bundle, _) = world.publish_attached("m2twice");
    let mut names = shipped_artifacts(&bundle);
    names.sort();
    assert_eq!(
        names,
        vec!["app.bin", "boot.hex", "firmware.elf", "firmware.map"],
        "one file offered twice shipped twice, or collided, or was refused — none of which is what the \
         release owner asked for"
    );
}

/// M3: the same kind with different bytes is two rows in the identity and two files in the bundle.
#[test]
fn two_attachments_of_one_kind_with_different_bytes_both_bind_and_both_ship() {
    let mut world = World::new("u2-m3-two-bins", ATTACHING_CONFIG);
    let hex = world.hex.clone();
    let other = world.dir.join("build/attached/second.bin");
    std::fs::write(&other, b"Different bytes entirely").expect("the second BIN is written");
    world.attach(&world.bin.clone(), ArtifactKind::Bin);
    world.attach(&other, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);

    let mut one = World::new("u2-m3-one-bin", ATTACHING_CONFIG);
    let first_bin = one.bin.clone();
    let one_hex = one.hex.clone();
    one.attach(&first_bin, ArtifactKind::Bin);
    one.attach(&one_hex, ArtifactKind::IntelHex);
    assert_ne!(
        world.recomputed_run_id(),
        one.recomputed_run_id(),
        "a second set of bytes under a kind already bound was not bound"
    );

    world.select_stored_run();
    let (bundle, _) = world.publish_attached("m3");
    let mut names = shipped_artifacts(&bundle);
    names.sort();
    assert_eq!(
        names,
        vec![
            "app.bin",
            "boot.hex",
            "firmware.elf",
            "firmware.map",
            "second.bin"
        ]
    );
    assert_eq!(
        digest_of(&bundle.join("artifacts/second.bin")),
        digest_of(&other)
    );
}

/// M4: an attachment named like the analyzed ELF is disambiguated by content, and both files survive. The
/// analyzed artifact keeps its plain name only because the pair is deterministic, not because it arrived
/// first in a directory listing.
#[test]
fn an_attachment_named_like_the_elf_is_disambiguated_and_both_survive() {
    let mut world = World::new("u2-m4-collision", ATTACHING_CONFIG);
    let clone = world.dir.join("build/attached/firmware.elf");
    std::fs::write(&clone, b"not an elf, named like one").expect("the colliding name is written");
    let hex = world.hex.clone();
    world.attach(&clone, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world.select_stored_run();

    let (bundle, _) = world.publish_attached("m4");
    let mut names = shipped_artifacts(&bundle);
    names.sort();
    // Neither file wins by arriving first: `bundle_names` renames *both* halves of a collision to Core's
    // content-keyed form, so the analyzed ELF keeps a name that says it is an ELF and the attached one says
    // it is a BIN, and neither is silently overwritten by the other.
    let attached_leaf = format!("bin-{}-firmware.elf", &digest_of(&clone).hex()[..8]);
    let analyzed_leaf = format!("elf-{}-firmware.elf", &digest_of(&world.elf).hex()[..8]);
    assert_eq!(
        names,
        vec![
            attached_leaf.clone(),
            "boot.hex".to_owned(),
            analyzed_leaf.clone(),
            "firmware.map".to_owned(),
        ],
        "{names:?}"
    );
    assert_eq!(
        digest_of(&bundle.join("artifacts").join(&attached_leaf)),
        digest_of(&clone),
        "the attached bytes were not the attached file"
    );
    assert_eq!(
        digest_of(&bundle.join("artifacts").join(&analyzed_leaf)),
        digest_of(&world.elf),
        "the analyzed bytes were not the analyzed file"
    );
}

/// M5, the portable half: two attached files in two directories whose leaves differ only in case both ship,
/// named apart by content, so a Windows reader that folds the two together still gets two distinct entries.
///
/// `disambiguated_name` keys on `(kind, digest, leaf)`, so the pair cannot fold into one name unless the two
/// files also share their bytes — which is the other half, tested below on the host where it is reachable.
#[test]
fn two_attachment_leaves_differing_only_in_case_are_named_apart_by_content() {
    let mut world = World::new("u2-m5-case", ATTACHING_CONFIG);
    let second = world.dir.join("build/attached/other/APP.BIN");
    std::fs::create_dir_all(second.parent().expect("the folder has a parent"))
        .expect("the second folder exists");
    std::fs::write(&second, b"uppercase path bytes").expect("the second BIN is written");
    let hex = world.hex.clone();
    let first = world.bin.clone();
    world.attach(&first, ArtifactKind::Bin);
    world.attach(&second, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world.select_stored_run();

    assert_ne!(
        digest_of(&first),
        digest_of(&second),
        "the two host files are one file, so this test would prove nothing"
    );
    let (bundle, _) = world.publish_attached("m5");
    let mut names = shipped_artifacts(&bundle);
    names.sort();
    let bins: Vec<&String> = names
        .iter()
        .filter(|name| name.to_lowercase().ends_with(".bin"))
        .collect();
    assert_eq!(bins.len(), 2, "{names:?}");
    // No two entries fold together, and each holds the bytes of the file it was chosen from.
    let folded: std::collections::BTreeSet<String> =
        names.iter().map(|name| name.to_lowercase()).collect();
    assert_eq!(folded.len(), names.len(), "{names:?}");
    for name in &bins {
        let shipped = digest_of(&bundle.join("artifacts").join(name));
        let from = if name.contains("APP.BIN") {
            &second
        } else {
            &first
        };
        assert_eq!(shipped, digest_of(from), "`{name}` is not its own file");
    }
}

/// M5, the Windows half: two selections that differ only in case name **one** file on this host, and the
/// engine says so by refusing rather than by writing the same bytes into the bundle twice under one name.
///
/// This is the case the POSIX half cannot reach: the resolved paths are different strings, so the duplicate
/// selection is not caught as a repeated path, and the two rows carry the same digest. What stops the bundle
/// is the manifest's own case-folded duplicate rule — which refuses before a destination is chosen, so
/// nothing is written and no file is silently dropped. Its wording was recorded as `C1-U2` finding 1 and
/// adjudicated by `C1-U2R1` §B, which is what `the_ntfs_fold_of_one_host_file_names_its_own_cause` now pins.
#[cfg(windows)]
#[test]
fn one_host_file_offered_under_two_cases_is_refused_rather_than_shipped_twice() {
    let mut world = World::new("u2-m5-ntfs", ATTACHING_CONFIG);
    let upper = world.dir.join("build/attached/APP.BIN");
    let hex = world.hex.clone();
    let lower = world.bin.clone();
    world.attach(&lower, ArtifactKind::Bin);
    world.attach(&upper, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);

    assert_eq!(
        digest_of(&lower),
        digest_of(&upper),
        "NTFS no longer folds these two names, so this test's premise has moved"
    );
    let error = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("one bundle cannot hold the same file twice under two spellings");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        !world.parent("m5ntfs").exists(),
        "a refused plan still created a destination"
    );
}

/// M5's wording, on the host where one file really is offered twice. §B kept the code and the refusal and
/// asked only that the sentence a release owner reads name **the selection that collided** rather than a
/// manifest they never wrote, so this asserts the message, the remediation, the absence of a host path, and
/// the absence of any written destination — every one of which the pre-§B wording failed.
#[cfg(windows)]
#[test]
fn the_ntfs_fold_of_one_host_file_names_its_own_cause() {
    let mut world = World::new("u2r1-m5-ntfs-wording", ATTACHING_CONFIG);
    let upper = world.dir.join("build/attached/APP.BIN");
    let lower = world.bin.clone();
    world.attach(&lower, ArtifactKind::Bin);
    world.attach(&upper, ArtifactKind::Bin);
    world.attach(&world.hex.clone(), ArtifactKind::IntelHex);
    world.select_stored_run();

    let error = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("the same host file is still one file under two spellings");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    let message = error.to_string();
    let remediation = error.remediation();
    // Names the attachment collision, not the document the caller never wrote.
    assert!(message.contains("attached"), "{message}");
    assert!(!message.contains("manifest"), "{message}");
    assert!(remediation.contains("case"), "{remediation}");
    assert!(
        !message.contains(&world.dir.display().to_string()),
        "{message}"
    );
    assert!(
        !world.parent("u2r1-m5-ntfs-wording").exists(),
        "a refused plan still created a destination"
    );
}

/// M5, the portable half of the same refusal: two **distinct** host files holding identical bytes under leaves
/// that fold to one name. `bundle_names` renames both halves to `<kind>-<sha8>-<leaf>`, and because the digest
/// is shared, the two renamings still fold together — so the manifest rule that refuses them fires on Linux and
/// macOS exactly as it fires on NTFS, without needing a case-insensitive filesystem to exist.
///
/// This is the test `C1-U2` could not write for the wording finding, because the wording was the finding: its
/// pair, `two_attachment_leaves_differing_only_in_case_are_named_apart_by_content`, covers the *different*-bytes
/// case, where the content-keyed rename saves the pair and both files ship.
#[test]
fn a_folded_duplicate_selection_is_refused_by_naming_the_selection_and_not_a_manifest() {
    let mut world = World::new("u2r1-m5-fold", ATTACHING_CONFIG);
    let upper = world.dir.join("build/attached/other/APP.BIN");
    std::fs::create_dir_all(upper.parent().expect("the folder has a parent"))
        .expect("the second folder exists");
    std::fs::copy(&world.bin, &upper).expect("the same bytes under a second spelling");
    assert_eq!(
        digest_of(&world.bin),
        digest_of(&upper),
        "identical bytes are the premise; without them the pair would ship"
    );

    world.attach(&world.bin.clone(), ArtifactKind::Bin);
    world.attach(&upper, ArtifactKind::Bin);
    world.attach(&world.hex.clone(), ArtifactKind::IntelHex);
    world.select_stored_run();

    let error = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("one bundle cannot ship the same bytes under two spellings");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    let message = error.to_string();
    let remediation = error.remediation();
    assert!(message.contains("attached"), "{message}");
    assert!(!message.contains("manifest"), "{message}");
    assert!(remediation.contains("case"), "{remediation}");
    // What is named is a bundle-relative entry, never the throwaway host directory.
    assert!(message.contains("`artifacts/"), "{message}");
    assert!(
        !message.contains(&world.dir.display().to_string())
            && !remediation.contains(&world.dir.display().to_string()),
        "a host path leaked: {message} / {remediation}"
    );
    assert!(
        !world.parent("u2r1-m5-fold").exists(),
        "a refused plan still created a destination"
    );
}

/// §B's other half: the two `6109` stories stay two stories. A duplicate found while assembling what this
/// release *selects* is the owner's collision and gets the naming sentence; a duplicate found inside a manifest
/// that already exists on disk is a broken bundle, and there the document really is the thing that disagrees
/// with itself, so it keeps the integrity sentence. One stable code, so no caller's branch moves.
#[test]
fn a_selection_duplicate_and_a_written_manifest_duplicate_are_told_apart() {
    let written = BundleError::Index(ManifestError::DuplicatePath {
        value: "artifacts/app.bin".to_owned(),
    });
    let selected = BundleError::DuplicateBundleName {
        name: "artifacts/app.bin".to_owned(),
    };
    assert_eq!(written.code(), selected.code(), "one refusal, one code");
    assert_eq!(selected.code(), "ERR-BUNDLE-6109");
    assert!(
        written
            .remediation()
            .starts_with("keep whatever was written"),
        "{}",
        written.remediation()
    );
    assert!(
        selected.remediation().contains("case"),
        "{}",
        selected.remediation()
    );
    assert!(written.to_string().contains("manifest"), "{written}");
    assert!(!selected.to_string().contains("manifest"), "{selected}");
    // Only the folded-selection case is re-described: every other index refusal still carries the report
    // layer's own words, which is what "do not alter the general self-verification path" requires.
    let untouched = BundleError::Index(ManifestError::NoFiles);
    assert_eq!(untouched.code(), "ERR-BUNDLE-6109");
    assert_eq!(
        untouched.to_string(),
        ManifestError::NoFiles.to_string(),
        "a transparent arm must not gain a re-worded message"
    );
}

/// L-3: a stored run that no longer recomputes because the attached set moved is refused before any file is
/// read for shipping, and the refusal says the context moved rather than naming a file that did not.
#[test]
fn a_stored_run_that_no_longer_recomputes_is_refused_before_any_write() {
    let mut world = World::new("u2-l3-stale", ATTACHING_CONFIG);
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world.select_stored_run();
    let plan = prepare_with_attachments(&world.request(), &world.attachments)
        .expect("the release is prepared");

    // The release owner withdraws one attached file from the selection and exports.
    let without_hex: Vec<AttachmentSelection> = world
        .attachments
        .iter()
        .filter(|selection| selection.path != hex)
        .cloned()
        .collect();
    let error = plan
        .publish_with_attachments(&world.request(), &without_hex, &world.parent("l3"), false)
        .expect_err("the set the verdict was computed over is not the set being shipped");
    assert_eq!(error.code(), "ERR-BUNDLE-6102", "{error}");
    assert!(
        !world
            .parent("l3")
            .join(plan.proposed_directory_name())
            .exists(),
        "a refused plan wrote a bundle"
    );
}

/// `N01-F04`: one withdrawn attachment, two surfaces, two codes, and the reason is the request shape rather
/// than anything about the file. The command line holds no stored run (`selected_run_id: None`), so there is
/// no verdict for the set to disagree with and the change surfaces as the file-level refusal `6118`; the
/// desktop exports against the run a person selected (`Some`), and the recomputed context stops that first,
/// at `6102`, before any file is compared. Both refuse, both leave the destination empty, and neither surface
/// is allowed to be the one that ships.
#[test]
fn a_withdrawn_attachment_is_refused_by_the_surface_that_holds_a_selected_run_and_by_the_one_that_does_not()
 {
    // Both halves start from the same world and the same withdrawal. The policy asks the analyzed build for
    // its own kinds and nothing of the attachments, so the verdict is not what moves: only the set is (`M10`
    // established that reading, and a policy that required the withdrawn kind would answer `6101` here).
    let mut cli = World::new("n01-f04-cli", PASSING_CONFIG);
    let cli_bin = cli.bin.clone();
    let cli_hex = cli.hex.clone();
    cli.attach(&cli_bin, ArtifactKind::Bin);
    cli.attach(&cli_hex, ArtifactKind::IntelHex);
    let cli_plan = prepare_with_attachments(&cli.request(), &cli.attachments)
        .expect("the release is prepared");
    let withdrawn: Vec<AttachmentSelection> = cli
        .attachments
        .iter()
        .filter(|selection| selection.path != cli_hex)
        .cloned()
        .collect();
    let cli_error = cli_plan
        .publish_with_attachments(&cli.request(), &withdrawn, &cli.parent("f04-cli"), false)
        .expect_err("one fewer attached file is a different release");
    assert_eq!(cli_error.code(), "ERR-BUNDLE-6118", "{cli_error}");
    assert!(
        cli_error.to_string().contains("boot.hex"),
        "the file-level refusal lost the name a person can act on: {cli_error}"
    );

    // The desktop's shape: the same world, the same withdrawal, and the run the page is showing named.
    let mut desktop = World::new("n01-f04-desktop", PASSING_CONFIG);
    let desktop_bin = desktop.bin.clone();
    let desktop_hex = desktop.hex.clone();
    desktop.attach(&desktop_bin, ArtifactKind::Bin);
    desktop.attach(&desktop_hex, ArtifactKind::IntelHex);
    desktop.select_stored_run();
    let desktop_plan = prepare_with_attachments(&desktop.request(), &desktop.attachments)
        .expect("the preview is built against the selected run");
    let desktop_withdrawn: Vec<AttachmentSelection> = desktop
        .attachments
        .iter()
        .filter(|selection| selection.path != desktop_hex)
        .cloned()
        .collect();
    let desktop_error = desktop_plan
        .publish_with_attachments(
            &desktop.request(),
            &desktop_withdrawn,
            &desktop.parent("f04-desktop"),
            false,
        )
        .expect_err("the selected run does not recompute from this set");
    assert_eq!(desktop_error.code(), "ERR-BUNDLE-6102", "{desktop_error}");

    // Two codes, one rule: neither refusal wrote anything.
    assert!(
        !cli.parent("f04-cli")
            .join(cli_plan.proposed_directory_name())
            .exists(),
        "the surface with no selected run shipped a bundle"
    );
    assert!(
        !desktop
            .parent("f04-desktop")
            .join(desktop_plan.proposed_directory_name())
            .exists(),
        "the surface with a selected run shipped a bundle"
    );
}

/// L-4: the bytes a bundle ships are the bytes the judged context carries. Every attachment in the folder has
/// its own row in the context the run id was computed over, with the kind it was declared as.
#[test]
fn every_shipped_attachment_is_a_row_the_judged_context_carries() {
    let mut world = World::new("u2-l4-membership", ATTACHING_CONFIG);
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    let mystery = world.mystery.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world.attach(&mystery, ArtifactKind::Unknown);
    world.select_stored_run();
    let (bundle, _) = world.publish_attached("l4");

    let context = world.context();
    for row in &context.attachments {
        let digest = row
            .sha256
            .value()
            .expect("an attachment the engine hashed has a digest");
        let leaf = match row.kind {
            ArtifactKind::Bin => "app.bin",
            ArtifactKind::IntelHex => "boot.hex",
            ArtifactKind::Unknown => "mystery.dat",
            other => panic!("a release attached a kind it cannot attach: {other:?}"),
        };
        assert_eq!(
            digest_of(&bundle.join("artifacts").join(leaf)).hex(),
            digest,
            "`{leaf}` ships as bytes the Gate never bound"
        );
    }
    assert_eq!(context.attachments.len(), 3);
}

/// M11, and L-4 from the other side: a file that cannot be observed is refused by the observation's own code,
/// bridged rather than re-described, and nothing reaches the destination.
#[test]
fn a_directory_or_an_empty_file_offered_as_an_attachment_is_refused_before_any_write() {
    let mut world = World::new("u2-m11-notregular", ATTACHING_CONFIG);
    let folder = world.dir.join("build/attached/package.bin");
    std::fs::create_dir_all(&folder).expect("a directory stands in for the BIN");
    let empty = world.dir.join("build/attached/empty.hex");
    std::fs::write(&empty, b"").expect("an empty file stands in for the HEX");
    world.attach(&folder, ArtifactKind::Bin);
    world.attach(&empty, ArtifactKind::IntelHex);

    let error = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("neither a folder nor an empty file is firmware");
    // The rows are observed in resolved-path order, so the empty file is met first and is refused as E-3.
    assert_eq!(error.code(), "ERR-BUNDLE-6117", "{error}");
    assert!(error.to_string().contains("empty.hex"), "{error}");
    assert!(
        !world.parent("m11").exists(),
        "a release that could not be observed still made a destination"
    );

    // The directory alone is E-1, and it says so in the words that name what is wrong with a folder.
    let mut folder_only = World::new("u2-m11-folder", ATTACHING_CONFIG);
    let other_hex = folder_only.hex.clone();
    let bin = folder_only.bin.clone();
    let package = folder_only.dir.join("build/attached/package.bin");
    std::fs::create_dir_all(&package).expect("a directory stands in for the BIN");
    folder_only.attach(&bin, ArtifactKind::Bin);
    folder_only.attach(&other_hex, ArtifactKind::IntelHex);
    folder_only.attach(&package, ArtifactKind::Bin);
    let error = prepare_with_attachments(&folder_only.request(), &folder_only.attachments)
        .expect_err("a directory is not firmware");
    assert_eq!(error.code(), "ERR-BUNDLE-6115", "{error}");
    assert!(error.to_string().contains("package.bin"), "{error}");
    assert!(error.to_string().contains("not a regular file"), "{error}");
}

/// M11's path half: a selection that tries to step outside the project root is refused by the same
/// observation that refuses a missing file, and no host path of the caller's appears in the message.
#[test]
fn a_selection_that_tries_to_leave_the_project_root_is_refused_without_naming_it() {
    let mut world = World::new("u2-m11-traversal", ATTACHING_CONFIG);
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    let mut selections = world.attachments.clone();
    selections.push(AttachmentSelection {
        path: Path::new("..").join("..").join("outside.bin"),
        declared_kind: ArtifactKind::Bin,
    });

    let error = prepare_with_attachments(&world.request(), &selections)
        .expect_err("a release cannot attach a file outside the project it describes");
    assert_eq!(error.code(), "ERR-BUNDLE-6115", "{error}");
    let message = error.to_string();
    assert!(
        !message.contains(".."),
        "the refusal quoted the traversal back: {message}"
    );
    assert!(
        !message.contains(&world.dir.to_string_lossy().to_string()),
        "the refusal carried a host path: {message}"
    );
    assert!(
        message.contains("outside.bin") || message.contains("firmwaresight"),
        "{message}"
    );
}

/// M11's declared-kind half: an ELF or a MAP cannot be attached, because an unanalyzed file must not stand in
/// for the build a release is about (`ADR-0030` D-3), and the refusal names no file.
#[test]
fn an_elf_offered_as_an_attachment_is_refused_as_an_analysis_input() {
    let mut world = World::new("u2-m11-elf-attached", ATTACHING_CONFIG);
    let elf = world.elf.clone();
    let hex = world.hex.clone();
    world.attach(&elf, ArtifactKind::Elf);
    world.attach(&world.bin.clone(), ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);

    let error = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("the analyzed ELF is not something one attaches");
    assert_eq!(error.code(), "ERR-BUNDLE-6116", "{error}");
    assert!(
        !error.to_string().contains("firmware.elf"),
        "a refusal about a declaration should not read like a refusal about a file: {error}"
    );
}

/// M13: a declared HEX whose bytes are not Intel HEX still answers a `hex` requirement — as raw bytes under a
/// declared kind — and the bundle says so instead of hiding it. Nothing in this unit parses it.
#[test]
fn a_declared_hex_whose_bytes_are_not_hex_is_shipped_and_disclosed_as_declared() {
    let mut world = World::new("u2-m13-declared", ATTACHING_CONFIG);
    let not_hex = world.dir.join("build/attached/not-hex.hex");
    std::fs::write(&not_hex, b"plain text, no records, no checksum").expect("the file is written");
    let bin = world.bin.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&not_hex, ArtifactKind::IntelHex);
    world.select_stored_run();
    let (bundle, _) = world.publish_attached("m13");

    let manifest = read_manifest(&bundle);
    let rows = manifest["extensions"]["attachments"]
        .as_array()
        .expect("a release with attachments discloses them");
    let row = rows
        .iter()
        .find(|row| row["path"].as_str() == Some("artifacts/not-hex.hex"))
        .expect("the file is disclosed");
    assert_eq!(row["kind"].as_str(), Some("hex"));
    assert_eq!(row["kind_basis"].as_str(), Some("declared"));
    assert_eq!(row["provenance"].as_str(), Some("unknown"));
    let shipped_digest = digest_of(&not_hex);
    assert_eq!(
        row["sha256"].as_str(),
        Some(shipped_digest.hex()),
        "the disclosure names bytes other than the ones shipped"
    );

    // And the bundle makes no structural claim: the analysis document describes the files it read as a
    // container, and an attached file was never read that way.
    let analysis = read_document(&bundle, "analysis.json");
    let names: Vec<&str> = analysis["artifacts"]
        .as_array()
        .expect("the analysis lists the artifacts it read")
        .iter()
        .filter_map(|row| row["fileName"].as_str())
        .collect();
    assert!(!names.contains(&"not-hex.hex"), "{names:?}");
}

/// M14: an `unknown` attachment is honest about what it is and satisfies no kind requirement, so a release
/// that needs a BIN cannot close the gap by attaching something it declined to classify.
#[test]
fn an_unknown_attachment_satisfies_no_bin_requirement() {
    let mut world = World::new("u2-m14-unknown", ATTACHING_CONFIG);
    let mystery = world.mystery.clone();
    let hex = world.hex.clone();
    world.attach(&mystery, ArtifactKind::Unknown);
    world.attach(&hex, ArtifactKind::IntelHex);

    let error = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("an unclassified file is not a BIN");
    assert_eq!(error.code(), "ERR-BUNDLE-6101", "{error}");
}

/// M12: an attachment large enough to leave no doubt about how it was read. `fingerprint::file_sha256` streams
/// a 64 KiB buffer, so a 1 MiB file is hashed and shipped without being held in memory. The >512 MiB case the
/// design mentions is not created here, and is reported as not verified rather than assumed.
#[test]
fn a_large_attachment_is_hashed_and_shipped_by_the_same_streaming_read() {
    let mut world = World::new("u2-m12-large", ATTACHING_CONFIG);
    let big = world.dir.join("build/attached/big.bin");
    let bytes: Vec<u8> = (0..1_048_576u64).map(|i| (i % 251) as u8).collect();
    std::fs::write(&big, &bytes).expect("the 1 MiB attachment is written");
    let hex = world.hex.clone();
    let small = world.bin.clone();
    world.attach(&big, ArtifactKind::Bin);
    world.attach(&small, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    world.select_stored_run();

    let (bundle, _) = world.publish_attached("m12");
    assert_eq!(
        digest_of(&bundle.join("artifacts/big.bin")),
        digest_of(&big),
        "a streamed hash of a large file did not match the file"
    );
    assert_eq!(
        std::fs::read(bundle.join("artifacts/big.bin"))
            .expect("the large file is in the bundle")
            .len(),
        1_048_576
    );
    // Two BINs of different sizes both bind: the identity is not a count of files.
    assert_eq!(world.context().attachments.len(), 3);
}

/// M7: the same bytes under a different declared kind move the identity, because what the Gate bound is the
/// pair, not the digest alone — and nothing here re-derives a kind from the content to argue with the owner.
#[test]
fn the_same_bytes_declared_as_a_different_kind_move_the_run_id() {
    let mut as_bin = World::new("u2-m7-bin", ATTACHING_CONFIG);
    let mut as_hex = World::new("u2-m7-hex", ATTACHING_CONFIG);
    let bin = as_bin.bin.clone();
    let other = as_hex.bin.clone();
    as_bin.attach(&bin, ArtifactKind::Bin);
    as_hex.attach(&other, ArtifactKind::IntelHex);

    assert_ne!(
        as_bin.recomputed_run_id(),
        as_hex.recomputed_run_id(),
        "a declared kind is not part of the identity that binds it"
    );
    assert_eq!(
        digest_of(&bin),
        digest_of(&other),
        "the two worlds do not hold the same bytes, so the comparison proved nothing"
    );
}

/// M9: an attachment whose bytes move between the preview and the export is refused by name, with the code that
/// says a file is no longer what it was, and the destination stays empty.
#[test]
fn a_changed_attachment_between_preview_and_publish_is_refused_by_name() {
    let world = attached_world("u2-m9-changed");
    let plan = prepare_with_attachments(&world.request(), &world.attachments)
        .expect("the release is prepared");
    append(&world.bin, b" one more byte in the image");

    let error = plan
        .publish_with_attachments(
            &world.request(),
            &world.attachments,
            &world.parent("m9"),
            false,
        )
        .expect_err("the preview hashed different bytes");
    assert_eq!(error.code(), "ERR-BUNDLE-6103", "{error}");
    assert!(error.to_string().contains("app.bin"), "{error}");
    assert!(
        !world
            .parent("m9")
            .join(plan.proposed_directory_name())
            .exists(),
        "a stale plan wrote a bundle"
    );
}

/// M9's size half: an attachment that grew while the release owner was reading the preview is refused on the
/// same row, whichever fact the filesystem reported first.
#[test]
fn an_attachment_that_changed_size_after_the_preview_is_refused_too() {
    let world = attached_world("u2-m9-size");
    let plan = prepare_with_attachments(&world.request(), &world.attachments)
        .expect("the release is prepared");
    std::fs::write(&world.hex, b":020000000100FA\r\n").expect("the HEX is rewritten shorter");

    let error = plan
        .publish_with_attachments(
            &world.request(),
            &world.attachments,
            &world.parent("m9s"),
            false,
        )
        .expect_err("the attached bytes are not the previewed ones");
    assert_eq!(error.code(), "ERR-BUNDLE-6103", "{error}");
    assert!(error.to_string().contains("boot.hex"), "{error}");
}

/// M8/M10: an attachment withdrawn from the *selection* between preview and export is not a file whose bytes
/// moved. Saying which of the two happened is E-4's whole content, and the refusal names the file that went.
///
/// The world's policy needs an ELF and a MAP, so the withdrawn file is the one whose kind no requirement reads:
/// the verdict is unchanged and the only fact is that the set moved. What a withdrawn file *does* change about a
/// verdict is the next test's subject, and both are refusals.
#[test]
fn an_attachment_removed_between_preview_and_publish_names_itself_as_removed() {
    let mut world = World::new("u2-m10-removed", PASSING_CONFIG);
    let bin = world.bin.clone();
    world.attach(&bin, ArtifactKind::Bin);
    let plan = prepare_with_attachments(&world.request(), &world.attachments)
        .expect("the release is prepared");

    let error = plan
        .publish_with_attachments(&world.request(), &[], &world.parent("m10r"), false)
        .expect_err("one fewer attached file is a different release");
    assert_eq!(error.code(), "ERR-BUNDLE-6118", "{error}");
    assert_eq!(
        error.to_string(),
        "the release attachment set changed after the preview: `app.bin` was removed",
        "{error}"
    );
    assert!(
        !world
            .parent("m10r")
            .join(plan.proposed_directory_name())
            .exists(),
        "a set change still wrote a bundle"
    );
    // The remediation is the owner's next step, stated as theirs, and it names no path.
    let remediation = error.remediation();
    assert!(
        remediation.contains("prepare the bundle again"),
        "{remediation}"
    );
    assert!(remediation.contains("recheck"), "{remediation}");
    assert!(
        !remediation.contains(&world.dir.to_string_lossy().to_string()),
        "the remediation carried a host path: {remediation}"
    );
}

/// M8's other half, and the precedence that follows from recomputing the verdict at export: when the withdrawn
/// attachment is one the policy *requires*, the Gate's own refusal is what answers. Nothing is written either
/// way, and nothing persisted is touched — the stored run keeps its row and its attachment facts.
#[test]
fn a_withdrawn_attachment_the_policy_requires_is_refused_by_the_gate_first() {
    let world = attached_world("u2-m8-required");
    let plan = prepare_with_attachments(&world.request(), &world.attachments)
        .expect("the release is prepared");
    let hex = world.hex.clone();
    let without_hex: Vec<AttachmentSelection> = world
        .attachments
        .iter()
        .filter(|selection| selection.path != hex)
        .cloned()
        .collect();

    let error = plan
        .publish_with_attachments(&world.request(), &without_hex, &world.parent("m8"), false)
        .expect_err("a release that no longer meets its own policy is not a release");
    assert_eq!(error.code(), "ERR-BUNDLE-6101", "{error}");
    assert!(
        !world
            .parent("m8")
            .join(plan.proposed_directory_name())
            .exists(),
        "a refused export left a bundle behind"
    );
}

/// M10's other direction: a file added to the selection after the preview is named as added, not blamed on a
/// file that did not move.
#[test]
fn an_attachment_added_between_preview_and_publish_names_itself_as_added() {
    let mut world = World::new("u2-m10-added", PASSING_CONFIG);
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    let plan = prepare_with_attachments(&world.request(), &world.attachments)
        .expect("the release is prepared");
    let mut longer = world.attachments.clone();
    longer.push(AttachmentSelection {
        path: world.mystery.clone(),
        declared_kind: ArtifactKind::Unknown,
    });

    let error = plan
        .publish_with_attachments(&world.request(), &longer, &world.parent("m10a"), false)
        .expect_err("one more attached file is a different release");
    assert_eq!(error.code(), "ERR-BUNDLE-6118", "{error}");
    assert!(
        error.to_string().contains("`mystery.dat` was added"),
        "{error}"
    );
    assert!(
        !world
            .parent("m10a")
            .join(plan.proposed_directory_name())
            .exists(),
        "an added file still wrote a bundle"
    );
}

/// M6: a rename with the bytes intact moves no identity — the Gate bound the digest, not the name — while the
/// release it identifies is a different one, because a name is release identity. Between those two facts the
/// preview is stale, and E-4 says `renamed` rather than blaming a file that never changed.
#[test]
fn an_attachment_renamed_between_preview_and_publish_names_itself_as_renamed() {
    let world = attached_world("u2-m10-renamed");
    let plan = prepare_with_attachments(&world.request(), &world.attachments)
        .expect("the release is prepared");
    let renamed: Vec<AttachmentSelection> = world
        .attachments
        .iter()
        .map(|selection| {
            if selection.path == world.hex {
                AttachmentSelection {
                    path: world.dir.join("build/attached/flash.hex"),
                    declared_kind: selection.declared_kind,
                }
            } else {
                selection.clone()
            }
        })
        .collect();
    std::fs::copy(&world.hex, &renamed[1].path).expect("the same bytes appear under the new name");

    let error = plan
        .publish_with_attachments(&world.request(), &renamed, &world.parent("m10n"), false)
        .expect_err("a renamed file is not the file the preview named");
    assert_eq!(error.code(), "ERR-BUNDLE-6118", "{error}");
    assert!(error.to_string().contains("was renamed"), "{error}");
    assert!(
        error.to_string().contains("flash.hex") || error.to_string().contains("boot.hex"),
        "the rename named neither side of itself: {error}"
    );
}

/// M6's identity half, run against the same rename: the run id is the bytes, and the release id is the bytes
/// plus the names. One moves and the other does not, which is the distinction `04_TECH/28` §5 rules 4 and 5
/// draw and the reason a renamed attachment is a stale *preview* rather than a stale *verdict*.
#[test]
fn a_renamed_attachment_keeps_the_run_id_and_moves_the_release_id() {
    let before = attached_world("u2-m6-before");
    let (bundle_before, outcome_before) = before.publish_attached("m6a");

    let mut after = World::new("u2-m6-after", ATTACHING_CONFIG);
    let bin = after.bin.clone();
    let hex = after.hex.clone();
    let moved = after.dir.join("build/attached/flash.hex");
    std::fs::copy(&hex, &moved).expect("the bytes are copied to the new name");
    after.attach(&bin, ArtifactKind::Bin);
    after.attach(&moved, ArtifactKind::IntelHex);
    after.select_stored_run();
    let (bundle_after, outcome_after) = after.publish_attached("m6b");

    assert_eq!(
        outcome_after.gate_run_id, outcome_before.gate_run_id,
        "a rename changed a verdict that reads only digests"
    );
    assert_ne!(
        outcome_after.release_id, outcome_before.release_id,
        "a rename left a release record that cannot name its own files"
    );
    assert!(
        shipped_artifacts(&bundle_after).contains(&"flash.hex".to_owned()),
        "the renamed file is not what shipped"
    );
    assert!(shipped_artifacts(&bundle_before).contains(&"boot.hex".to_owned()));
}

/// L-6, the link a reader depends on: the bundle the person ends up with verifies against itself, with both
/// attached files counted as shipped artifacts and both hash layers naming them.
#[test]
fn a_bundle_that_ships_attachments_verifies_and_re_derives_its_release_id() {
    let world = attached_world("u2-l6-verify");
    let (bundle, outcome) = world.publish_attached("l6");

    let verification =
        verify_bundle(&bundle).expect("the attached bundle verifies from its own bytes");
    assert_eq!(verification.release_id, outcome.release_id);
    assert_eq!(
        verification.artifact_count, 4,
        "both classes are shipped artifacts"
    );
    assert_eq!(
        verification.file_count,
        world.tree(&bundle).len(),
        "the verifier counted a different set of files than the folder holds"
    );
    // Every attached file is named by both hash layers, which is what `artifact_count` alone cannot say.
    let sums =
        std::fs::read_to_string(bundle.join("SHA256SUMS")).expect("the checksum index is there");
    for name in ["artifacts/app.bin", "artifacts/boot.hex"] {
        assert!(
            sums.contains(name),
            "`{name}` is not in SHA256SUMS:
{sums}"
        );
        assert!(
            manifest_files(&bundle).iter().any(|row| row == name),
            "`{name}` is not in the manifest"
        );
    }
    // The report the release owner opens is not a summary of the old bundle: it names the attached files too.
    let html =
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is there");
    assert!(
        html.contains("app.bin") && html.contains("boot.hex"),
        "the report named neither attached file"
    );
}

/// L-6 from the reader's side: a relocated bundle is verifiable with the project that produced it gone, which
/// is the property §45 promises a reviewer, and the attachments are part of what survives the move.
#[test]
fn a_relocated_bundle_with_attachments_verifies_with_its_sources_deleted() {
    let world = attached_world("u2-l6-relocated");
    let (bundle, outcome) = world.publish_attached("l6r");

    let relocated = std::env::temp_dir().join(format!(
        "firmwaresight-u2-relocated-{}-{}",
        std::process::id(),
        nanos()
    ));
    std::fs::create_dir_all(&relocated).expect("the second root exists");
    copy_tree(&bundle, &relocated.join("bundle")).expect("the bundle was copied");
    let copy = relocated.join("bundle");

    // Every file the release was assembled from is withdrawn: the analyzed pair, the notes, and both
    // attached files. What verifies now is the folder, not the workspace behind it.
    for source in [&world.elf, &world.map, &world.notes, &world.bin, &world.hex] {
        std::fs::remove_file(source).expect("a source is withdrawn");
    }

    let verification = verify_bundle(&copy).expect("the relocated bundle verifies on its own");
    assert_eq!(verification.release_id, outcome.release_id);
    assert_eq!(
        verification.artifact_count, 4,
        "both classes survive the move"
    );
    assert_eq!(verification.file_count, world.tree(&copy).len());
    let html =
        std::fs::read_to_string(copy.join("release-report.html")).expect("the report is there");
    assert!(
        !html.contains(&world.dir.to_string_lossy().as_ref().to_string()),
        "the report leaked the project path"
    );
    let _ = std::fs::remove_dir_all(&relocated);
}

/// L-6's teeth: one byte changed in a shipped attachment breaks the bundle, because the digest the disclosure,
/// the index and the release fingerprint all carry is a claim about those bytes.
#[test]
fn tampering_with_a_shipped_attachment_breaks_the_bundle_it_sits_in() {
    let world = attached_world("u2-l6-tamper");
    let (bundle, _) = world.publish_attached("l6t");
    append(&bundle.join("artifacts/app.bin"), b"one more byte");

    let error = verify_bundle(&bundle).expect_err("the bytes no longer match their own digest");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
}

/// L-6, the disclosure's own proof: an attached file the manifest does not disclose is refused, because a
/// reader then holds a shipped file whose evidence class nothing in the bundle states.
#[test]
fn an_attached_file_the_manifest_never_disclosed_is_refused() {
    let world = attached_world("u2-l6-undisclosed");
    let (bundle, _) = world.publish_attached("l6d");
    let mut manifest = read_manifest(&bundle);
    let rows = manifest["extensions"]["attachments"]
        .as_array_mut()
        .expect("the release discloses its attachments")
        .iter()
        .filter(|row| row["path"].as_str() != Some("artifacts/app.bin"))
        .cloned()
        .collect::<Vec<_>>();
    manifest["extensions"]["attachments"] = serde_json::Value::Array(rows);
    write_document(&bundle, "release-manifest.json", &manifest);

    let error =
        verify_bundle(&bundle).expect_err("`app.bin` ships and no document says what it is");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error.to_string().contains("no document says what kind"),
        "{error}"
    );
}

/// L-6 in the other direction: a disclosure that names a file the bundle does not ship is a document
/// describing a bundle that is not there.
#[test]
fn a_disclosed_attachment_the_bundle_does_not_ship_is_refused() {
    let world = attached_world("u2-l6-ghost");
    let (bundle, _) = world.publish_attached("l6g");
    let mut manifest = read_manifest(&bundle);
    let ghost = serde_json::json!({
        "path": "artifacts/ghost.bin",
        "kind": "bin",
        "sha256": digest_of(&world.bin).hex(),
        "size": world.bin.metadata().expect("the BIN is there").len(),
        "kind_basis": "declared",
        "provenance": "unknown",
    });
    manifest["extensions"]["attachments"]
        .as_array_mut()
        .expect("the release discloses its attachments")
        .push(ghost);
    write_document(&bundle, "release-manifest.json", &manifest);

    let error = verify_bundle(&bundle).expect_err("nothing named `ghost.bin` was shipped");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(error.to_string().contains("ghost.bin"), "{error}");
}

/// L-6's honesty check: a disclosure that disagrees with the index it sits beside, or that claims more than an
/// attached file can state about itself, is refused rather than read as fact.
#[test]
fn a_disclosure_that_overstates_itself_is_refused() {
    type Overstate = (&'static str, fn(&mut serde_json::Value), &'static str);
    let cases: [Overstate; 4] = [
        (
            "u2-l6-digest",
            |row: &mut serde_json::Value| {
                row["sha256"] = serde_json::Value::String("0".repeat(64));
            },
            "does not carry",
        ),
        (
            "u2-l6-size",
            |row: &mut serde_json::Value| {
                row["size"] = serde_json::Value::from(1u64);
            },
            "does not carry",
        ),
        (
            "u2-l6-basis",
            |row: &mut serde_json::Value| {
                row["kind_basis"] =
                    serde_json::Value::String("derived_from_leading_bytes".to_owned());
            },
            "given its kind",
        ),
        (
            "u2-l6-provenance",
            |row: &mut serde_json::Value| {
                row["provenance"] = serde_json::Value::String("built-by-this-project".to_owned());
            },
            "provenance",
        ),
    ];
    for (name, edit, fragment) in cases {
        let world = attached_world(name);
        let (bundle, _) = world.publish_attached(name);
        let mut manifest = read_manifest(&bundle);
        let rows = manifest["extensions"]["attachments"]
            .as_array_mut()
            .expect("the release discloses its attachments");
        edit(&mut rows[0]);
        write_document(&bundle, "release-manifest.json", &manifest);

        let error =
            verify_bundle(&bundle).expect_err("a disclosure that overstates itself is refused");
        assert_eq!(error.code(), "ERR-BUNDLE-6109", "{name}: {error}");
        assert!(error.to_string().contains(fragment), "{name}: {error}");
    }
}

/// L-6's cross-check, in the direction the two indexes cannot catch: one shipped file claimed by both
/// documents, so a reader would have to guess which evidence class its bytes belong to.
#[test]
fn a_shipped_file_claimed_as_both_analyzed_and_attached_is_refused() {
    let world = attached_world("u2-l6-double");
    let (bundle, _) = world.publish_attached("l6x");
    let mut analysis = read_document(&bundle, "analysis.json");
    let rows = analysis["artifacts"]
        .as_array_mut()
        .expect("the analysis lists the artifacts it read");
    let mut claimed = rows[0].clone();
    claimed["fileName"] = serde_json::Value::String("app.bin".to_owned());
    claimed["kind"] = serde_json::Value::String("bin".to_owned());
    claimed["sha256"] = serde_json::Value::String(digest_of(&world.bin).hex().to_owned());
    rows.push(claimed);
    write_document(&bundle, "analysis.json", &analysis);
    reindex(&bundle, "analysis.json");

    let error = verify_bundle(&bundle).expect_err("one file cannot be both classes at once");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error.to_string().contains("both as an analyzed artifact"),
        "{error}"
    );
}

// ------------------------------------------------------------------- C1-U3 the `gate-results:1` pair, read back

/// Every tamper below re-records **both** indexes with `reindex`, so the bundle is refused because its two
/// documents disagree, not because a digest went stale. A tamper that leaves the indexes stale proves only
/// that hashing works, and `C1-U2` already has tests for that.
#[test]
fn a_bundle_that_ships_attachments_discloses_the_same_bytes_in_both_documents() {
    // `T-C1-04`'s positive direction on a real bundle, and `T-C1-05`'s pairing half: the Gate document names
    // the grammar that judged the attached bytes and lists exactly the rows the manifest discloses.
    let world = attached_world("u3-pair");
    let (bundle, _) = world.publish_attached("pair");
    let gate = read_document(&bundle, "gate-results.json");
    let manifest = read_manifest(&bundle);

    assert_eq!(
        gate["extensions"]["canonical_input_label"], "firmwaresight-gate-input/2",
        "a run that bound attached bytes says which grammar judged them"
    );
    let rows = gate["extensions"]["attachments"]
        .as_array()
        .expect("the disclosure is a list");
    assert_eq!(
        rows.len(),
        2,
        "one row per bound `(kind, digest)`: {rows:?}"
    );
    for row in rows {
        let keys = row
            .as_object()
            .expect("a row is an object")
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec!["byte_size", "kind", "sha256"],
            "the disclosure is the identity's own facts, with no name and no structure claim"
        );
    }
    let kinds = rows
        .iter()
        .map(|row| row["kind"].as_str().expect("kind").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(kinds, vec!["bin".to_owned(), "hex".to_owned()], "{rows:?}");
    let disclosed = manifest["extensions"]["attachments"]
        .as_array()
        .expect("the manifest discloses its attachments");
    for row in rows {
        let found = disclosed
            .iter()
            .find(|entry| entry["sha256"] == row["sha256"] && entry["kind"] == row["kind"])
            .expect("every bound row is a shipped attachment");
        assert_eq!(
            found["size"], row["byte_size"],
            "both documents state the same length for the same bytes"
        );
    }
    assert_eq!(
        gate["run_id"], manifest["extensions"]["gate_run_id"],
        "the bundle gates the run whose verdict it ships"
    );
    verify_bundle(&bundle).expect("a bundle that discloses its pair verifies");

    // And the owner's file names are not the document's order. The publish path hands the Gate its attached
    // rows sorted by path, so a BIN named last and a HEX named first arrive at the context as `hex, bin` —
    // and the disclosure still has to come out in the identity's `(kind, digest)` order, because a run id was
    // taken over the canonicalized set and not over the alphabetical one (`04_TECH/28` §5 rule 1).
    let mut reversed = World::new("u3-pair-order", ATTACHING_CONFIG);
    let late_bin = reversed.dir.join("build/attached/zz-image.bin");
    let early_hex = reversed.dir.join("build/attached/aa-image.hex");
    std::fs::write(&late_bin, b"BIN bytes offered under a name that sorts last")
        .expect("the BIN is written");
    std::fs::write(
        &early_hex,
        b":0000\r\nhex bytes offered under a name that sorts first",
    )
    .expect("the HEX is written");
    reversed.attach(&early_hex, ArtifactKind::IntelHex);
    reversed.attach(&late_bin, ArtifactKind::Bin);
    let (second, _) = reversed.publish_attached("pairorder");
    let second_gate = read_document(&second, "gate-results.json");
    let kinds = second_gate["extensions"]["attachments"]
        .as_array()
        .expect("the pair is present")
        .iter()
        .filter_map(|row| row["kind"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec!["bin", "hex"],
        "the disclosure is canonicalized, not transcribed from the order the paths sort into"
    );
    verify_bundle(&second)
        .expect("a bundle whose owner attached in the other order still verifies the same way");
}

#[test]
fn a_pre_c1_bundle_that_attached_nothing_still_verifies_without_the_pair() {
    // The compatibility direction, on the document side this time: an attachment-free release writes neither
    // entry, and the reader that has learned to demand the pair still accepts it.
    let world = World::new("u3-pair-none", PASSING_CONFIG);
    let (bundle, _) = world.publish("pairnone");
    let gate = read_document(&bundle, "gate-results.json");
    assert!(
        gate["extensions"].get("canonical_input_label").is_none()
            && gate["extensions"].get("attachments").is_none(),
        "a run that bound nothing is documented the way P4 documented one: {}",
        gate["extensions"]
    );
    verify_bundle(&bundle).expect("the pair is conditional in the reader as well as the writer");
}

#[test]
fn a_gate_document_that_discloses_rows_without_a_label_is_refused() {
    let world = attached_world("u3-pair-rows");
    let (bundle, _) = world.publish_attached("rows");
    let mut gate = read_document(&bundle, "gate-results.json");
    gate["extensions"]
        .as_object_mut()
        .expect("extensions is an object")
        .remove("canonical_input_label");
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error = verify_bundle(&bundle).expect_err("half a pair is not a disclosure");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error.to_string().contains("is absent"),
        "the refusal names the missing half: {error}"
    );
}

#[test]
fn a_gate_document_that_names_a_label_with_no_rows_is_refused() {
    let world = attached_world("u3-pair-label");
    let (bundle, _) = world.publish_attached("label");
    let mut gate = read_document(&bundle, "gate-results.json");
    gate["extensions"]["attachments"] = serde_json::Value::Array(Vec::new());
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error = verify_bundle(&bundle).expect_err("a label over an empty block claims nothing");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error
            .to_string()
            .contains("while no attachment is disclosed"),
        "{error}"
    );
}

#[test]
fn a_gate_document_that_names_another_canonical_grammar_is_refused() {
    // Not a coercion: a document that says its run id came from a grammar this build does not write is a
    // disagreement about identity, and reading it as a `/1` run would quietly drop the attached bytes from
    // the record of what was judged.
    for (name, label) in [
        ("u3-grammar-one", "firmwaresight-gate-input/1"),
        ("u3-grammar-three", "firmwaresight-gate-input/3"),
    ] {
        let world = attached_world(name);
        let (bundle, _) = world.publish_attached(name);
        let mut gate = read_document(&bundle, "gate-results.json");
        gate["extensions"]["canonical_input_label"] = serde_json::Value::String(label.to_owned());
        write_document(&bundle, "gate-results.json", &gate);
        reindex(&bundle, "gate-results.json");

        let error = verify_bundle(&bundle).expect_err("a bound run cannot claim another grammar");
        assert_eq!(error.code(), "ERR-BUNDLE-6109", "{name}: {error}");
        assert!(
            error
                .to_string()
                .contains("only grammar a document with bound rows can name"),
            "{name}: {error}"
        );
    }
}

#[test]
fn a_gate_document_whose_rows_are_out_of_the_identitys_order_is_refused() {
    let world = attached_world("u3-order");
    let (bundle, _) = world.publish_attached("order");
    let mut gate = read_document(&bundle, "gate-results.json");
    let rows = gate["extensions"]["attachments"]
        .as_array_mut()
        .expect("the disclosure is a list");
    rows.reverse();
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error =
        verify_bundle(&bundle).expect_err("a reordered block is a different canonical text");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error.to_string().contains("sort before or beside"),
        "{error}"
    );
}

#[test]
fn a_gate_document_that_binds_one_pair_twice_is_refused() {
    let world = attached_world("u3-dupe");
    let (bundle, _) = world.publish_attached("dupe");
    let mut gate = read_document(&bundle, "gate-results.json");
    let rows = gate["extensions"]["attachments"]
        .as_array_mut()
        .expect("the disclosure is a list");
    let again = rows[0].clone();
    rows.push(again);
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error = verify_bundle(&bundle).expect_err("one `(kind, digest)` pair is bound once");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error.to_string().contains("sort before or beside"),
        "{error}"
    );
}

#[test]
fn a_gate_document_that_discloses_bytes_the_manifest_does_not_ship_is_refused() {
    let world = attached_world("u3-sets");
    let (bundle, _) = world.publish_attached("sets");
    let mut gate = read_document(&bundle, "gate-results.json");
    gate["extensions"]["attachments"]
        .as_array_mut()
        .expect("the disclosure is a list")[0]["sha256"] =
        serde_json::Value::String("0".repeat(64));
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error = verify_bundle(&bundle).expect_err("the two documents name different bytes");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error
            .to_string()
            .contains("disclose different attached bytes"),
        "{error}"
    );
}

#[test]
fn an_attachment_release_documented_as_though_it_judged_nothing_is_refused() {
    // The pre-C1 document shape is *not* a legal reading of an attachment release: dropping the pair would
    // turn a run that judged four files into a record that claims it judged two.
    let world = attached_world("u3-nodisclosure");
    let (bundle, _) = world.publish_attached("nodisclosure");
    let mut gate = read_document(&bundle, "gate-results.json");
    let extensions = gate["extensions"]
        .as_object_mut()
        .expect("extensions is an object");
    extensions.remove("attachments");
    extensions.remove("canonical_input_label");
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error = verify_bundle(&bundle).expect_err("an attached byte vanished from the record");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error
            .to_string()
            .contains("disclose different attached bytes"),
        "{error}"
    );
}

#[test]
fn a_gate_document_that_reports_another_run_than_the_bundle_gates_is_refused() {
    // The highest-signal tamper the bundle permits: every digest still matches, both indexes agree with the
    // bytes, the document still validates against `gate-results:1` — and the verdict inside it belongs to a
    // different run than the one the release gates. Nothing here read that back before `C1-U3`.
    let world = attached_world("u3-runid");
    let (bundle, _) = world.publish_attached("runid");
    let mut gate = read_document(&bundle, "gate-results.json");
    let original = gate["run_id"].as_str().expect("a run id").to_owned();
    let other = format!("gate-{}", "f".repeat(64));
    gate["run_id"] = serde_json::Value::String(other.clone());
    for finding in gate["findings"].as_array_mut().expect("findings is a list") {
        let rule = finding["rule_id"].as_str().expect("rule id").to_owned();
        finding["id"] = serde_json::Value::String(format!("{other}#{rule}"));
    }
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error =
        verify_bundle(&bundle).expect_err("the document names a run this bundle does not gate");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error
            .to_string()
            .contains("one run's verdict beside another"),
        "{error}"
    );
    assert_eq!(
        original.len(),
        69,
        "the run id the bundle really gates: `{original}`"
    );
}

#[test]
fn a_gate_document_that_discloses_a_field_beyond_the_identity_is_refused() {
    // A row that carries a file name, a path or a structure claim is asserting something the run id never
    // bound, and §11's excluded conclusions start here.
    for (name, key) in [
        ("u3-extra-name", "file_name"),
        ("u3-extra-span", "address_span"),
    ] {
        let world = attached_world(name);
        let (bundle, _) = world.publish_attached(name);
        let mut gate = read_document(&bundle, "gate-results.json");
        gate["extensions"]["attachments"]
            .as_array_mut()
            .expect("the disclosure is a list")[0][key] = serde_json::Value::String("x".to_owned());
        write_document(&bundle, "gate-results.json", &gate);
        reindex(&bundle, "gate-results.json");

        let error =
            verify_bundle(&bundle).expect_err("a disclosed field nothing judged is refused");
        assert_eq!(error.code(), "ERR-BUNDLE-6109", "{name}: {error}");
        assert!(
            error
                .to_string()
                .contains("enters a run id as its declared kind"),
            "{name}: {error}"
        );
    }
}

#[test]
fn a_gate_document_that_discloses_a_digest_that_is_not_a_digest_is_refused() {
    let world = attached_world("u3-digest");
    let (bundle, _) = world.publish_attached("digest");
    let mut gate = read_document(&bundle, "gate-results.json");
    gate["extensions"]["attachments"]
        .as_array_mut()
        .expect("the disclosure is a list")[0]["sha256"] =
        serde_json::Value::String("ZZ".to_owned());
    write_document(&bundle, "gate-results.json", &gate);
    reindex(&bundle, "gate-results.json");

    let error = verify_bundle(&bundle).expect_err("a malformed digest is not evidence of anything");
    assert_eq!(error.code(), "ERR-BUNDLE-6109", "{error}");
    assert!(
        error.to_string().contains("not the 64 lowercase hex"),
        "{error}"
    );
}

/// The two documents a release with attachments ships about its attached files, read as a stranger reads
/// them: `analysis.json` records what was analyzed, and nothing records what was not.
#[test]
fn an_analysis_document_never_grows_a_row_for_an_attached_file() {
    // `T-C1-11`: `analysis:1` stays the shape of one analyzed build. `C1-U2` proved the negative half (the
    // attached leaf is absent from the artifact rows); this is the exact-row-set half, asserted on the bytes
    // a real publish wrote rather than on a projection.
    let world = attached_world("u3-analysis");
    let (bundle, _) = world.publish_attached("analysis");
    let analysis = read_document(&bundle, "analysis.json");
    let kinds = analysis["artifacts"]
        .as_array()
        .expect("the analysis lists what it read")
        .iter()
        .map(|row| row["kind"].as_str().expect("a kind").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec!["elf".to_owned(), "map".to_owned()],
        "an attached BIN and HEX are not pseudo-analyzed into `analysis.json` (`ADR-0030` D-1)"
    );
    let names = analysis["artifacts"]
        .as_array()
        .expect("the analysis lists what it read")
        .iter()
        .filter_map(|row| row["fileName"].as_str())
        .collect::<Vec<_>>();
    for attached in ["app.bin", "boot.hex"] {
        assert!(
            !names.contains(&attached),
            "`{attached}` ships as bytes and was never analyzed"
        );
    }
    assert!(
        !analysis.to_string().contains("app.bin"),
        "the document does not mention the attached files at all: {}",
        analysis["artifacts"]
    );
}

#[test]
fn bytes_that_begin_with_the_elf_magic_declared_hex_satisfy_hex_and_claim_nothing_more() {
    // `T-C1-17`: the kind an owner declares is what the Gate records, and the bytes are what the identity
    // binds. A file whose first four bytes are the ELF magic, offered as `hex`, satisfies a required `hex`
    // **as raw bytes of a kind the owner declared** (`ADR-0030` D-8) — and nothing this release ships says
    // those bytes are well-formed Intel HEX, because no parser here exists to say it.
    let world = attached_world("u3-elfmagic");
    std::fs::write(
        &world.hex,
        [0x7f_u8, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0],
    )
    .expect("the attached file is rewritten before the release is prepared");
    let (bundle, _) = world.publish_attached("elfmagic");

    let gate = read_document(&bundle, "gate-results.json");
    let required = gate["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|finding| finding["rule_id"] == "artifacts.required")
        .expect("the rule always answers");
    assert_eq!(required["state"], "PASS", "{required}");
    let summary = required["summary"].as_str().expect("a summary");
    assert!(
        summary.contains("bytes verified, provenance not verified"),
        "the sentence that satisfies the requirement also states its limit: {summary}"
    );
    for claim in ["Intel HEX", "hex records", "checksum", "well-formed"] {
        assert!(
            !summary.to_lowercase().contains(&claim.to_lowercase()),
            "the finding asserts `{claim}` about bytes no parser read: {summary}"
        );
    }

    let manifest = read_manifest(&bundle);
    let disclosed = manifest["extensions"]["attachments"]
        .as_array()
        .expect("the release discloses its attachments")
        .iter()
        .filter(|row| row["kind"] == "hex")
        .collect::<Vec<_>>();
    assert_eq!(
        disclosed.len(),
        1,
        "the ELF-shaped bytes are still disclosed as `hex`"
    );
    assert_eq!(disclosed[0]["kind_basis"], "declared", "{:?}", disclosed[0]);
    assert_eq!(disclosed[0]["provenance"], "unknown", "{:?}", disclosed[0]);

    let analysis = read_document(&bundle, "analysis.json");
    assert!(
        !analysis.to_string().contains("boot.hex"),
        "and the analysis document gains no row for bytes it never analyzed"
    );
    verify_bundle(&bundle)
        .expect("a declared kind the bytes disagree with is still a legal release");
}

#[test]
fn no_finding_and_no_report_line_reports_structure_for_an_attached_file() {
    // `T-C1-18` and §11's excluded conclusions: the structural vocabulary belongs to a container format, and
    // an attached image is not one. Every sentence this release writes *about an attached file* is swept —
    // each finding that cites an `attachment:` locator, and each line of the report page that names one.
    let forbidden = [
        "address span",
        "record checksum",
        "well-formed",
        "sections",
        "symbols",
        "entry point",
        "architecture",
        "memory regions",
        "object attribution",
    ];
    let world = attached_world("u3-structure");
    let (bundle, _) = world.publish_attached("structure");

    let gate = read_document(&bundle, "gate-results.json");
    let cited: Vec<&serde_json::Value> = gate["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .filter(|finding| {
            finding["evidence_refs"]
                .as_array()
                .expect("refs")
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|reference| reference.starts_with("attachment:"))
        })
        .collect();
    assert!(
        !cited.is_empty(),
        "a release with attachments cites them, so this sweep is not over an empty set"
    );
    for finding in &cited {
        let text = format!(
            "{} {}",
            finding["summary"].as_str().expect("a summary"),
            finding["remediation"].as_str().unwrap_or_default()
        );
        for word in forbidden {
            assert!(
                !text.to_lowercase().contains(word),
                "the finding `{}` states {} about bytes that were only hashed:\n{text}",
                finding["rule_id"].as_str().expect("a rule id"),
                word
            );
        }
    }

    let report = std::fs::read_to_string(bundle.join("release-report.html"))
        .expect("the report page is in the bundle");
    let mut lines_that_name_one = 0usize;
    for line in report.lines() {
        if !["app.bin", "boot.hex"]
            .iter()
            .any(|leaf| line.contains(leaf))
        {
            continue;
        }
        lines_that_name_one += 1;
        for word in forbidden {
            assert!(
                !line.to_lowercase().contains(word),
                "the report page states {word} about an attached file: {line}"
            );
        }
    }
    assert!(
        lines_that_name_one > 0,
        "the page lists what the bundle ships, so the sweep above is not vacuous"
    );

    // The disclosure rows themselves: two documents, and neither has room for a structure claim.
    for (name, rows) in [
        (
            "gate-results.json",
            gate["extensions"]["attachments"]
                .as_array()
                .expect("the pair is present")
                .clone(),
        ),
        (
            "release-manifest.json",
            read_manifest(&bundle)["extensions"]["attachments"]
                .as_array()
                .expect("the disclosure is present")
                .clone(),
        ),
    ] {
        for row in &rows {
            let text = row.to_string().to_lowercase();
            for word in forbidden {
                assert!(
                    !text.contains(word),
                    "`{name}` carries `{word}` for an attached file: {row}"
                );
            }
        }
    }
}

/// M15 and the compatibility half of every link above: a release that attaches nothing writes no disclosure
/// key at all, so its bundle's bytes are the bytes this engine wrote before `C1-U1` existed.
#[test]
fn an_attachment_free_bundle_carries_no_disclosure_key_at_all() {
    let world = World::new("u2-m15-nokey", PASSING_CONFIG);
    let (bundle, _) = world.publish("m15");
    let manifest = read_manifest(&bundle);
    assert!(
        manifest["extensions"].get("attachments").is_none(),
        "an empty list would still be a change in the bytes every old reader parses"
    );
    verify_bundle(&bundle).expect("the attachment-free bundle still verifies");
}

/// §5.D's one sanctioned path, named from `release_attachments.rs`: the rows a context binds are the facts
/// `observe_attachment` produced, and nothing between the two re-derives a digest.
#[test]
fn the_attachment_rows_a_context_binds_are_the_facts_the_observation_produced() {
    let world = attached_world("u2-sanctioned-path");
    let observed = world.attachment_rows();
    let context = world.context();

    assert_eq!(context.attachments.len(), observed.len());
    for row in &observed {
        let fact = row.as_gate_fact();
        assert!(
            context
                .attachments
                .iter()
                .any(|bound| bound.kind == fact.kind && bound.sha256 == fact.sha256),
            "an observed attachment was not the row the context bound"
        );
    }

    // The same facts, one layer down: what the bundle discloses is the digest the observation wrote, and the
    // bytes on disk inside the bundle still match it.
    let (bundle, _) = world.publish_attached("path");
    let manifest = read_manifest(&bundle);
    for row in observed {
        let disclosed = manifest["extensions"]["attachments"]
            .as_array()
            .expect("the release discloses its attachments")
            .iter()
            .find(|disclosed| {
                disclosed["kind"].as_str() == Some(row.kind.word())
                    && disclosed["sha256"].as_str() == row.sha256.value().map(String::as_str)
            })
            .unwrap_or_else(|| panic!("`{}` is not disclosed", row.file_name));
        let path = disclosed["path"].as_str().expect("a disclosed path");
        assert_eq!(
            digest_of(&bundle.join(path)).hex(),
            disclosed["sha256"].as_str().expect("a disclosed digest"),
            "`{path}` is disclosed with bytes it does not hold"
        );
    }
}

/// `T-C1-13` (`04_TECH/28` §7.2, `C1-U2R1` must-execute item 2): a growth verdict identical with and without
/// attachments. `C1-U2` recorded this exact comparison as NOT_VERIFIED and declined to infer it from the
/// unknown-count half, and the packet asks for the direct one — same snapshot, same baseline, same rule, twice.
#[test]
fn the_growth_rule_answers_identically_with_and_without_release_attachments() {
    let mut world = World::new("u2r1-tc113-growth", ATTACHING_REVIEWING_CONFIG);
    world.with_baseline();

    let without = world.evaluate();
    let base = without
        .finding(GateRuleId::BaselineGrowth)
        .expect("every rule always answers");
    assert_eq!(base.state, FindingState::Review, "{}", base.summary);

    world.attach(&world.bin.clone(), ArtifactKind::Bin);
    world.attach(&world.hex.clone(), ArtifactKind::IntelHex);
    let with = world.evaluate();
    let grown = with
        .finding(GateRuleId::BaselineGrowth)
        .expect("every rule always answers");

    // The verdict, its severity, its prose and its citations are the same facts over the same footprint rows.
    assert_eq!(grown.state, base.state);
    assert_eq!(grown.effective_severity, base.effective_severity);
    assert_eq!(
        grown.summary, base.summary,
        "the growth prose moved with the set"
    );
    assert_eq!(
        grown.evidence_refs, base.evidence_refs,
        "the growth rule cited an attachment, which §7.2 forbids"
    );
    // The two boundaries D-1 and D-7 draw: the build is one build, and an attachment is not unknown evidence.
    assert_eq!(with.snapshot_id, without.snapshot_id, "SnapshotId moved");
    assert_eq!(
        with.counts.unknown, without.counts.unknown,
        "an attachment entered unknown.count, so evidence.unknown_review could move because a file was chosen"
    );
    assert_ne!(
        with.run_id, without.run_id,
        "the two runs must be genuinely different judgements"
    );
    assert_ne!(grown.id, base.id, "a finding id is keyed to its run");
    // And the attachments did the job they are for, out of the other evidence class.
    let required = with
        .finding(GateRuleId::RequiredArtifacts)
        .expect("every rule always answers");
    assert_eq!(required.state, FindingState::Pass, "{}", required.summary);
    assert!(
        required
            .evidence_refs
            .iter()
            .any(|reference| reference.starts_with("artifact:elf:")),
        "{:?}",
        required.evidence_refs
    );
    assert!(
        required
            .evidence_refs
            .iter()
            .any(|reference| reference.starts_with("attachment:bin:"))
            && required
                .evidence_refs
                .iter()
                .any(|reference| reference.starts_with("attachment:hex:")),
        "{:?}",
        required.evidence_refs
    );
}

/// `T-C1-08` at the bundle boundary (`C1-U2R1` must-execute item 1): an acceptance recorded over one attachment
/// set does not make a different set's run releasable, and the refusal arrives before anything is written.
///
/// Two refusals are asserted, because two callers can be here: a library caller carrying run A's acceptance
/// across to run B meets the engine's own invariant (`ERR-INTERNAL-9004`, naming both ids), and the path a
/// person reaches — the Gate re-run over the new set, its review not yet accepted — meets the ordinary
/// `ERR-BUNDLE-6101`. A desktop is stopped one layer earlier still, by the store, which is what
/// `an_acceptance_recorded_over_one_attachment_set_does_not_dispose_of_a_different_one` in
/// `crates/firmwaresight-storage/tests/gate_history.rs` proves against a real SQLite file.
///
/// Both halves are asserted, because a refusal alone could be read as "attachments block releases": the
/// identical-set case ships first, and the changed set ships once the review is accepted **against its own run
/// id**. This is the pairing `C1-U2` named as missing in its own report.
#[test]
fn an_acceptance_recorded_over_one_attachment_set_does_not_make_another_set_release_ready() {
    let mut world = World::new("u2r1-tc108-bundle", ATTACHING_REVIEWING_CONFIG);
    world.with_baseline();
    let bin = world.bin.clone();
    let hex = world.hex.clone();
    world.attach(&bin, ArtifactKind::Bin);
    world.attach(&hex, ArtifactKind::IntelHex);
    let evaluation_a = world.evaluate();
    let run_a = evaluation_a.run_id.clone();
    world.accept(GateRuleId::BaselineGrowth);
    world.select_stored_run();

    let (bundle_a, outcome_a) = world.publish_attached("tc108-a");
    assert_eq!(
        outcome_a.gate_run_id, run_a,
        "the release names the run it was judged over"
    );
    verify_bundle(&bundle_a).expect("the accepted-review release verifies against its own bytes");

    // The release owner replaces one attached file. The analyzed build did not move; the judged bytes did.
    std::fs::write(&bin, b"BIN\xff\x00revised image bytes").expect("the attached bytes move");
    let evaluation_b = world.evaluate();
    let run_b = evaluation_b.run_id.clone();
    assert_ne!(
        run_b, run_a,
        "a different attachment set must be a different run"
    );
    assert_eq!(
        evaluation_b.snapshot_id, evaluation_a.snapshot_id,
        "the attachment moved the release, not the build"
    );
    world.selected_run_id = None;

    // Two refusals, because two different callers can be here. A library caller that hands over run A's
    // acceptance list for run B is handed the engine's own invariant: the id it carries names no finding of the
    // run being judged. Nothing is written either way.
    let stale = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("one run's acceptance is not another run's disposition");
    assert_eq!(stale.code(), "ERR-INTERNAL-9004", "{stale}");
    assert!(stale.to_string().contains(&run_a), "{stale}");
    assert!(stale.to_string().contains(&run_b), "{stale}");

    // And the path a person actually reaches — the Gate was re-run over the new set, and its review has not
    // been accepted yet — is the ordinary Gate-not-ready refusal, because a REVIEW survives an attachment change
    // exactly as it survives any other change of bytes.
    world.acceptances.clear();
    let unready = prepare_with_attachments(&world.request(), &world.attachments)
        .expect_err("the new run's review is still undisposed");
    assert_eq!(unready.code(), "ERR-BUNDLE-6101", "{unready}");
    assert!(
        !world.parent("tc108-b").exists(),
        "a refused plan still created a destination"
    );
    assert_eq!(
        world
            .folder_contents(&world.dir)
            .into_iter()
            .filter(|name| name.starts_with("out-tc108-b"))
            .count(),
        0,
        "a refusal wrote beside the project"
    );

    // Accepting B's own review, under B's id, releases it. The refusal above was about the key.
    world.accept(GateRuleId::BaselineGrowth);
    world.select_stored_run();
    let (bundle_b, outcome_b) = world.publish_attached("tc108-b");
    assert_eq!(outcome_b.gate_run_id, run_b);
    assert_ne!(outcome_b.release_id, outcome_a.release_id);
    verify_bundle(&bundle_b).expect("the second release verifies against its own bytes");

    // And run A's shipped history is untouched: the earlier bundle still stands on its own bytes and still
    // records the run it answered.
    verify_bundle(&bundle_a).expect("the first release still verifies after the second one");
    assert_eq!(
        read_manifest(&bundle_a)["extensions"]["gate_run_id"]
            .as_str()
            .expect("a recorded run id"),
        run_a.as_str(),
        "the first bundle's record of which run it answered moved"
    );
}

// ── C1-U2R1: M11's source-side symlink rule, adjudication C ──────────────────────────────────────────────
//
// The rule as the Architect set it: a link the release owner chose deliberately is followed to its target, and
// what ships is that target's bytes under the name they chose; a broken, directory or inaccessible target fails
// safely; nothing inside a bundle is ever a symlink. `C1-U2` left this unwritten because "writing a test here
// would have fixed one behaviour by inventing a rule" — the rule now exists, so the tests do.
//
// **Platform record, stated before any reader reaches it.** The `#[cfg(unix)]` half below did not compile and
// did not run on the Windows host this round was written on, and cannot: `docker` is absent and `wsl` has no
// installed distribution, so the only POSIX execution this repository offers is a CI job, which this round had
// no authorization to trigger. These assertions are therefore evidence *pending a Linux run*, and the corrective
// report files M11 as `NOT_VERIFIED` on this host. The Windows counterpart self-reports in the same spirit: it
// prints a `SKIPPED` line and returns when the host refuses link creation, rather than passing on an assertion
// it never made.
#[cfg(unix)]
mod source_symlink_posix {
    use super::*;

    /// §C's positive half: an explicitly chosen link ships exactly its target's bytes, under the leaf the owner
    /// named, and the bundle holds a regular file where the link was.
    #[test]
    fn a_symlink_offered_as_a_source_ships_its_target_bytes_and_leaves_no_link_behind() {
        let mut world = World::new("u2r1-m11-follow", ATTACHING_CONFIG);
        let target = world.dir.join("build/attached/real-image.bin");
        std::fs::write(&target, b"BIN\xff\x00behind the link").expect("the target is written");
        let link = world.dir.join("build/attached/linked.bin");
        std::os::unix::fs::symlink(&target, &link)
            .expect("symlink creation is permitted on this runner");

        world.attach(&link, ArtifactKind::Bin);
        world.attach(&world.hex.clone(), ArtifactKind::IntelHex);
        world.select_stored_run();

        let (bundle, _) = world.publish_attached("m11");
        let names = shipped_artifacts(&bundle);
        assert!(
            names.iter().any(|name| name == "linked.bin"),
            "the shipped leaf is not the name the owner chose: {names:?}"
        );
        let shipped = bundle.join("artifacts").join("linked.bin");
        assert_eq!(
            digest_of(&shipped),
            digest_of(&target),
            "the link shipped, not the bytes behind it"
        );
        let metadata = std::fs::symlink_metadata(&shipped).expect("the shipped entry is statable");
        assert!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "a bundle holds a symlink, and a relocated bundle would then hold a broken one"
        );
        for name in &names {
            let entry = std::fs::symlink_metadata(bundle.join("artifacts").join(name))
                .expect("a shipped entry is statable");
            assert!(
                !entry.file_type().is_symlink(),
                "`{name}` is a symlink inside the bundle"
            );
        }
        verify_bundle(&bundle).expect("the bundle verifies against its own bytes");
    }

    /// §C's first refusal: a link to nothing has no bytes, so it is E-1 at the moment of observation — before
    /// any Gate judgement, and it names the leaf the owner chose rather than the host path either end of it.
    #[test]
    fn a_broken_symlink_offered_as_a_source_is_refused_without_naming_its_host_path() {
        let world = World::new("u2r1-m11-broken", ATTACHING_CONFIG);
        let link = world.dir.join("build/attached/dangling.bin");
        std::os::unix::fs::symlink(world.dir.join("build/attached/not-here.bin"), &link)
            .expect("symlink creation is permitted on this runner");

        let error = observe_attachment(&link, ArtifactKind::Bin)
            .expect_err("a link to nothing carries no bytes to ship");
        assert_eq!(error.code(), "ERR-BUNDLE-6115");
        assert!(error.to_string().contains("dangling.bin"), "{error}");
        assert!(
            !error
                .to_string()
                .contains(&world.path().display().to_string()),
            "a host path leaked: {error}"
        );
    }

    /// §C's second refusal, and the one that keeps the rule from becoming a directory walk: a link to a
    /// directory is not a file, and following it must not turn the release into a subtree.
    #[test]
    fn a_symlink_to_a_directory_offered_as_a_source_is_refused_as_not_a_regular_file() {
        let world = World::new("u2r1-m11-dirlink", ATTACHING_CONFIG);
        let directory = world.dir.join("build/attached/sub");
        std::fs::create_dir_all(&directory).expect("the directory is created");
        std::fs::write(directory.join("inner.bin"), b"inner").expect("a file inside it");
        let link = world.dir.join("build/attached/sub.link");
        std::os::unix::fs::symlink(&directory, &link)
            .expect("symlink creation is permitted on this runner");

        let error = observe_attachment(&link, ArtifactKind::Bin)
            .expect_err("a directory is not a file a release ships");
        assert_eq!(error.code(), "ERR-BUNDLE-6115");
        assert!(
            error.to_string().contains("regular file"),
            "the refusal should say what is wrong with the target: {error}"
        );
        assert!(
            !error
                .to_string()
                .contains(&world.path().display().to_string()),
            "a host path leaked: {error}"
        );
    }

    /// §C's timing half, and the reason this is a safety rule rather than a nicety: a link can be re-pointed
    /// after the preview hashed it. What ships then is not what was judged, so the export re-observation —
    /// which follows the link again — must refuse rather than carry the old verdict over.
    #[test]
    fn a_source_symlink_re_pointed_after_the_preview_cannot_ship_under_the_old_verdict() {
        let mut world = World::new("u2r1-m11-repoint", ATTACHING_CONFIG);
        let first = world.dir.join("build/attached/one.bin");
        let second = world.dir.join("build/attached/two.bin");
        std::fs::write(&first, b"first image bytes").expect("the first target is written");
        std::fs::write(&second, b"second image bytes, deliberately longer")
            .expect("the second target is written");
        let link = world.dir.join("build/attached/current.bin");
        std::os::unix::fs::symlink(&first, &link)
            .expect("symlink creation is permitted on this runner");

        world.attach(&link, ArtifactKind::Bin);
        world.attach(&world.hex.clone(), ArtifactKind::IntelHex);
        let plan = prepare_with_attachments(&world.request(), &world.attachments)
            .expect("the release is prepared");

        std::fs::remove_file(&link).expect("the link is removable");
        std::os::unix::fs::symlink(&second, &link).expect("the re-point is permitted");

        let error = plan
            .publish_with_attachments(
                &world.request(),
                &world.attachments,
                &world.parent("m11rp"),
                false,
            )
            .expect_err("the preview hashed the other target");
        assert_eq!(error.code(), "ERR-BUNDLE-6103", "{error}");
        assert!(
            !world
                .parent("m11rp")
                .join(plan.proposed_directory_name())
                .exists(),
            "a stale plan wrote a bundle"
        );
    }
}

/// §C on the host this round actually ran on. This Windows box refuses link creation with
/// `ERROR_PRIVILEGE_NOT_HELD` (WinError 1314), measured before the test was written rather than assumed after
/// it, so the honest outcome here is a loud skip. If a future host permits it, the same three facts are
/// asserted: the target's bytes ship, the shipped entry is a regular file, and the bundle still verifies.
#[cfg(windows)]
#[test]
fn a_source_symlink_on_windows_ships_its_target_bytes_when_the_host_permits_links() {
    let mut world = World::new("u2r1-m11-windows", ATTACHING_CONFIG);
    let target = world.dir.join("build/attached/real-image.bin");
    std::fs::write(&target, b"BIN\xff\x00behind the link").expect("the target is written");
    let link = world.dir.join("build/attached/linked.bin");
    if let Err(error) = std::os::windows::fs::symlink_file(&target, &link) {
        eprintln!(
            "SKIPPED (M11 on Windows): this host refused symlink creation (raw OS error {:?}); the rule is \
             asserted by the POSIX module and stays NOT_VERIFIED here",
            error.raw_os_error()
        );
        return;
    }

    world.attach(&link, ArtifactKind::Bin);
    world.attach(&world.hex.clone(), ArtifactKind::IntelHex);
    world.select_stored_run();
    let (bundle, _) = world.publish_attached("m11w");
    let shipped = bundle.join("artifacts").join("linked.bin");
    assert_eq!(
        digest_of(&shipped),
        digest_of(&target),
        "the link shipped, not the bytes behind it"
    );
    let metadata = std::fs::symlink_metadata(&shipped).expect("the shipped entry is statable");
    assert!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "a bundle holds a symlink"
    );
    verify_bundle(&bundle).expect("the bundle verifies against its own bytes");
}
