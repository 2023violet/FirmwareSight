//! The contract test for the portable `analysis:1` document (prompt §15, §53).
//!
//! Like the diff and gate contract tests, this runs through `firmwaresight_report::schema_check` — the
//! dependency-free subset of draft/2020-12 these schemas actually use — because no JSON-Schema validator is
//! admitted (§15, §64) and a contract nobody checks is a promise, not a contract.
//!
//! The rejecting cases are part of the test. A suite that can only pass would not notice the schema
//! drifting away from the projection, which is the one failure mode a portable contract actually has.

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::artifact::{Artifact, ParserId};
use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, SnapshotBuilder};
use firmwaresight_core::domain::capability::{
    Availability, Capabilities, FormatSupport, Provision,
};
use firmwaresight_core::domain::evidence::{Confidence, EvidenceClass, EvidenceItem, SourceType};
use firmwaresight_core::domain::identity::{
    Architecture, ArtifactKind, ArtifactTimes, Bitness, BuildIdentity, Endianness, Fact, Sha256,
};
use firmwaresight_core::domain::memory::{LayoutSource, MemoryFootprint, MemoryLayout};
use firmwaresight_core::domain::release::{RELEASE_ID_PREFIX, ReleaseArtifact};
use firmwaresight_core::domain::section::{Section, SectionFlags, SectionRole};
use firmwaresight_core::domain::symbol::{Symbol, SymbolBinding, SymbolKind, SymbolSectionRef};
use firmwaresight_report::analysis::{
    ANALYSIS_SCHEMA_ID, ANALYSIS_SCHEMA_VERSION, AnalysisDocumentDto,
};
use firmwaresight_report::render::to_json;
use firmwaresight_report::schema_check::{self, Failure};
use serde_json::Value;

fn analysis_schema() -> Value {
    let path = repo_root().join("schemas").join("analysis.schema.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("analysis.schema.json must parse: {error}"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("crates/firmwaresight-report lives at <root>/crates/<name>")
        .to_path_buf()
}

fn validate(doc: &Value) -> Vec<Failure> {
    schema_check::validate(&analysis_schema(), doc)
}

// --------------------------------------------------------------------------- fixtures

const ELF_HEX: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const MAP_HEX: &str = "2222222222222222222222222222222222222222222222222222222222222222";

fn sha(hex: &str) -> Sha256 {
    Sha256::parse(hex).expect("a test digest is 64 lowercase hex characters")
}

fn flags(alloc: bool, execute: bool, write: bool) -> SectionFlags {
    SectionFlags {
        alloc,
        execute,
        write,
        merge: false,
        strings: false,
        tls: false,
        compressed: false,
    }
}

fn section(index: usize, name: Fact<String>, role: SectionRole, file_size: u64) -> Section {
    Section {
        index,
        name,
        role,
        flags: flags(
            true,
            role == SectionRole::Code,
            role == SectionRole::InitializedData,
        ),
        virtual_address: Fact::known(0x0800_0000 + index as u64 * 0x100),
        load_address: Fact::known(0x0800_0000 + index as u64 * 0x100),
        file_offset: Fact::known(0x40 + index as u64 * 0x100),
        file_size,
        memory_size: Fact::known(file_size),
        region: Fact::known("FLASH".to_owned()),
    }
}

/// The two shapes a bundle reader most needs to see stated rather than inferred: a `.bss` that costs RAM
/// but no image bytes, and a section whose name the parser could not read.
fn gap_sections() -> Vec<Section> {
    vec![
        section(1, Fact::known(".text".to_owned()), SectionRole::Code, 1_024),
        Section {
            index: 2,
            name: Fact::known(".bss".to_owned()),
            role: SectionRole::UninitializedData,
            flags: flags(true, false, true),
            virtual_address: Fact::known(0x2000_0000),
            load_address: Fact::unknown("no PT_LOAD covers this address".to_owned()),
            // The one nullable column pair `04_TECH/15` records: absent in the file, and the reason stays.
            file_offset: Fact::unknown("SHT_NOBITS stores no file payload".to_owned()),
            file_size: 0,
            memory_size: Fact::known(512),
            region: Fact::known("SRAM".to_owned()),
        },
        Section {
            index: 3,
            name: Fact::unknown("the section header string table is truncated".to_owned()),
            role: SectionRole::Unknown,
            flags: flags(false, false, false),
            virtual_address: Fact::known(0),
            load_address: Fact::unknown("no load evidence for a non-alloc section".to_owned()),
            file_offset: Fact::known(0x1_000),
            file_size: 64,
            memory_size: Fact::unknown("a non-alloc section occupies no runtime memory".to_owned()),
            region: Fact::unknown("no region claims this address range".to_owned()),
        },
    ]
}

fn symbols() -> Vec<Symbol> {
    vec![
        Symbol {
            name: Fact::known("main".to_owned()),
            address: Fact::known(0x0800_0042),
            size: Fact::known(28),
            kind: SymbolKind::Function,
            binding: SymbolBinding::Global,
            section: SymbolSectionRef::Index(1),
        },
        Symbol {
            name: Fact::known("__bss_start".to_owned()),
            address: Fact::known(0x2000_0000),
            size: Fact::unknown("a symbol addresses a point, not a span".to_owned()),
            kind: SymbolKind::NotType,
            binding: SymbolBinding::Other(17),
            section: SymbolSectionRef::Absolute,
        },
        Symbol {
            name: Fact::unknown("the symbol name is beyond the string table".to_owned()),
            address: Fact::unknown("an undefined symbol has no address".to_owned()),
            size: Fact::known(0),
            kind: SymbolKind::Unknown,
            binding: SymbolBinding::Unknown,
            section: SymbolSectionRef::Undefined,
        },
    ]
}

fn artifact(kind: ArtifactKind, hex: &str, path: &str, parser: ParserId) -> Artifact {
    Artifact {
        id: format!("art-{}", kind_word(kind)),
        path: path.to_owned(),
        kind,
        sha256: sha(hex),
        byte_size: 4_096,
        parser_id: parser,
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

fn kind_word(kind: ArtifactKind) -> &'static str {
    ReleaseArtifact::kind_word(kind)
}

fn evidence() -> Vec<EvidenceItem> {
    vec![
        EvidenceItem {
            id: "ev-1".to_owned(),
            classification: EvidenceClass::Observed,
            source_type: SourceType::ElfSectionHeader,
            source_locator: "elf.section_header[1].sh_size".to_owned(),
            field: "memory.nonvolatile".to_owned(),
            raw_value: "1024".to_owned(),
            rule: "section-role+file-size".to_owned(),
            confidence: Some(Confidence::High),
            notes: None,
        },
        EvidenceItem {
            id: "ev-2".to_owned(),
            classification: EvidenceClass::Unknown,
            source_type: SourceType::RuleEngine,
            source_locator: "evidence:unknown-count=1".to_owned(),
            field: "artifact.build_id".to_owned(),
            raw_value: "unknown".to_owned(),
            rule: "explicit-unknown".to_owned(),
            confidence: None,
            notes: Some("no note section".to_owned()),
        },
    ]
}

fn snapshot() -> BuildSnapshot {
    let sections = gap_sections();
    let layout = MemoryLayout {
        regions: Vec::new(),
        source: LayoutSource::None,
    };
    let footprint = MemoryFootprint::compute(&sections, &layout);
    SnapshotBuilder::new("0.6.0-test")
        .project_id("contract")
        .artifact(artifact(
            ArtifactKind::Elf,
            ELF_HEX,
            "C:\\work\\motor-controller\\build\\firmware.elf",
            ParserId::elf_object("0.36"),
        ))
        .artifact(artifact(
            ArtifactKind::Map,
            MAP_HEX,
            "/home/engineer/build/firmware.map",
            ParserId::gnu_ld_map(),
        ))
        .sections(sections)
        .symbols(symbols())
        .memory(footprint)
        .evidence(evidence())
        .capabilities(Capabilities {
            elf: FormatSupport::Supported,
            sections: Availability::Available,
            symbols: Availability::Partial,
            debug_info: Availability::Unavailable,
            map: Provision::Provided,
            object_attribution: Availability::Unavailable,
            git: Availability::Unknown,
        })
        .seal()
        .expect("a sealed snapshot has at least one artifact")
}

/// The bundle's own artifact rows: file names in the document are these, matched by digest.
fn shipped() -> Vec<ReleaseArtifact> {
    vec![
        ReleaseArtifact {
            kind: ArtifactKind::Elf,
            file_name: "firmware.elf".to_owned(),
            sha256: sha(ELF_HEX),
            byte_size: 4_096,
        },
        ReleaseArtifact {
            kind: ArtifactKind::Map,
            file_name: "firmware.map".to_owned(),
            sha256: sha(MAP_HEX),
            byte_size: 4_096,
        },
    ]
}

fn document() -> Value {
    let projection = AnalysisDocumentDto::from_snapshot(&snapshot(), &shipped());
    serde_json::from_str(&to_json(&projection)).expect("the projection is JSON")
}

// --------------------------------------------------------------------------- positive

#[test]
fn a_full_projection_validates_against_the_published_schema() {
    let doc = document();
    let failures = validate(&doc);
    assert!(failures.is_empty(), "contract violations: {failures:?}");
    assert_eq!(doc["schema"], ANALYSIS_SCHEMA_ID);
    assert_eq!(doc["schemaVersion"], ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn every_normalized_row_is_carried_not_summarised() {
    // §14: the portable document holds the full section and symbol tables. A count would let a truncated
    // file pass as complete, so the arrays are checked at their length rather than against a `counts` block.
    let doc = document();
    assert_eq!(doc["sections"].as_array().expect("array").len(), 3);
    assert_eq!(doc["symbols"].as_array().expect("array").len(), 3);
    assert_eq!(doc["evidence"].as_array().expect("array").len(), 2);
    assert_eq!(doc["artifacts"].as_array().expect("array").len(), 2);
}

#[test]
fn absence_stays_a_value_plus_a_reason_and_is_never_zero() {
    let doc = document();
    let bss = doc["sections"]
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["name"] == ".bss")
        .expect("the .bss row is carried");
    assert_eq!(bss["fileOffset"], Value::Null, "NOBITS has no file offset");
    assert_eq!(
        bss["fileOffsetUnknownReason"], "SHT_NOBITS stores no file payload",
        "the reason survives the write, which is what P1 left open and P4 must not repeat"
    );
    assert_eq!(
        bss["fileSize"], 0,
        "a NOBITS section really is zero bytes in the file"
    );
    assert_eq!(bss["memorySize"], 512, "and really does cost RAM");

    let nameless = doc["sections"]
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["index"] == 3)
        .expect("the unnamed section is carried");
    assert_eq!(nameless["name"], Value::Null);
    assert!(
        nameless["nameUnknownReason"]
            .as_str()
            .is_some_and(|reason| !reason.is_empty()),
        "an absent name states why it is absent"
    );

    let undefined = doc["symbols"]
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["section"] == "undefined")
        .expect("the undefined symbol is carried");
    assert_eq!(undefined["address"], Value::Null);
    assert_eq!(
        undefined["size"], 0,
        "a size of zero is the observed value here, not a stand-in"
    );
}

#[test]
fn addresses_are_hex_and_are_never_unit_converted() {
    let doc = document();
    let main = doc["symbols"]
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["name"] == "main")
        .expect("main is carried");
    assert_eq!(
        main["address"], "0x8000042",
        "lowercase hex, bytes, not KiB"
    );
    assert!(
        !main["address"].as_str().unwrap_or_default().contains('.'),
        "an address is never rendered as a decimal quantity"
    );
    let text = doc["sections"]
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["name"] == ".text")
        .expect("the code section is carried");
    // The fixture places section 1 at 0x0800_0100 with file offset 0x140, and the document says exactly
    // that: the same number the ELF says, in the base the ELF is read in.
    assert_eq!(text["virtualAddress"], "0x8000100");
    assert_eq!(text["loadAddress"], "0x8000100");
    assert_eq!(text["fileOffset"], "0x140");
}

#[test]
fn artifact_rows_are_named_as_the_bundle_carries_them() {
    let doc = document();
    let names: Vec<&str> = doc["artifacts"]
        .as_array()
        .expect("array")
        .iter()
        .map(|row| row["fileName"].as_str().expect("a file name"))
        .collect();
    // The names come from the shipped rows, so the document agrees with `artifacts/` and `SHA256SUMS`.
    assert_eq!(names, ["firmware.elf", "firmware.map"]);

    let kinds: Vec<&str> = doc["artifacts"]
        .as_array()
        .expect("array")
        .iter()
        .map(|row| row["kind"].as_str().expect("a kind"))
        .collect();
    assert_eq!(
        kinds,
        ["elf", "map"],
        "the Gate vocabulary, not the enum's Debug form"
    );
}

#[test]
fn a_host_path_in_a_source_name_never_reaches_the_document() {
    // Both fixtures are seeded with a Windows path and a POSIX path on purpose. A bundle that leaked them
    // would be a portable document that only one machine can read — and a leak of where the user works.
    let doc = document();
    let rendered = doc.to_string();
    assert!(!rendered.contains("C:"), "{rendered}");
    assert!(!rendered.contains('\\'), "{rendered}");
    assert!(!rendered.contains("/home/"), "{rendered}");
    assert!(
        !rendered.contains("work"),
        "the source directory is not a release fact"
    );
}

#[test]
fn the_document_falls_back_to_a_sanitized_leaf_when_no_bundle_row_matches() {
    // A document projected without the bundle's rows still must not carry a path.
    let projection = AnalysisDocumentDto::from_snapshot(&snapshot(), &[]);
    let doc = serde_json::to_value(&projection).expect("serializable");
    let names: Vec<&str> = doc["artifacts"]
        .as_array()
        .expect("array")
        .iter()
        .map(|row| row["fileName"].as_str().expect("a file name"))
        .collect();
    assert_eq!(names, ["firmware.elf", "firmware.map"]);
}

#[test]
fn two_projections_of_one_snapshot_are_byte_identical() {
    let first = to_json(&AnalysisDocumentDto::from_snapshot(&snapshot(), &shipped()));
    let second = to_json(&AnalysisDocumentDto::from_snapshot(&snapshot(), &shipped()));
    assert_eq!(
        first, second,
        "a portable document is a hash target, not a rendering"
    );
    // `render::to_json` emits the compact form; the line terminator belongs to whoever writes the file, so
    // the bytes that land in the bundle are the bytes the manifest hashes.
    assert!(!first.contains('\n'), "the projection is one line: {first}");
}

#[test]
fn the_document_carries_no_wall_clock_field() {
    // §42: no generation timestamp. The audit time lives in the release record, not in the bytes.
    let doc = document();
    let rendered = doc.to_string().to_ascii_lowercase();
    for forbidden in [
        "generatedat",
        "createdat",
        "importedat",
        "timestamp",
        "mtime",
        "absolutepath",
        "sourcepath",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "`{forbidden}` reached a portable document"
        );
    }
}

#[test]
fn the_memory_block_is_the_projection_the_cli_already_emits() {
    // One implementation, two documents: reuse of `dto::memory_dto` is the reason these two blocks cannot
    // disagree about what the same footprint means.
    let doc = document();
    let memory = &doc["memory"];
    assert_eq!(memory["accountingRule"], "adr-0021-dual-budget");
    assert_eq!(memory["layoutSource"], "none");
    assert!(
        memory["nonvolatileImageFootprint"]["state"]
            .as_str()
            .is_some_and(|state| matches!(state, "exact" | "partial" | "unknown")),
        "{memory}"
    );
    assert!(memory["contributions"].is_array());
}

// --------------------------------------------------------------------------- rejecting

#[test]
fn an_unknown_schema_major_is_a_hard_error_not_a_reread() {
    // 04_TECH/26: `unknown input schema major -> hard error, not silent reinterpretation`.
    let mut doc = document();
    doc["schemaVersion"] = serde_json::json!(2);
    let failures = validate(&doc);
    assert!(
        failures
            .iter()
            .any(|failure| failure.path == "$.schemaVersion"),
        "a v2 document must be rejected, not read as v1: {failures:?}"
    );
}

#[test]
fn a_missing_required_property_is_rejected() {
    let mut doc = document();
    doc["snapshot"]
        .as_object_mut()
        .expect("object")
        .remove("snapshotId");
    let failures = validate(&doc);
    assert!(
        failures
            .iter()
            .any(|failure| failure.message.starts_with("missing required")),
        "a snapshot without its id must not validate: {failures:?}"
    );
}

#[test]
fn an_extra_property_is_rejected_because_the_contract_is_closed() {
    // additionalProperties:false is the promise that a reader sees every field the writer promised.
    let mut doc = document();
    doc["snapshot"]["unexpected"] = serde_json::json!("added");
    let failures = validate(&doc);
    assert!(
        failures.iter().any(|failure| failure.path == "$.snapshot"
            && failure.message.contains("not declared")),
        "a closed object must reject an open addition: {failures:?}"
    );
}

#[test]
fn a_vocabulary_value_that_is_not_in_the_enum_is_rejected() {
    let mut doc = document();
    doc["sections"][0]["role"] = serde_json::json!("cod");
    let failures = validate(&doc);
    assert!(
        failures
            .iter()
            .any(|failure| failure.path == "$.sections[0].role"),
        "a near-miss role name must not pass: {failures:?}"
    );
}

#[test]
fn a_digest_that_is_not_lowercase_hex_of_sixty_four_is_rejected() {
    // The uppercase probe needs letters: a digest of nothing but digits would survive `to_uppercase()`
    // unchanged and the case would assert nothing at all.
    const WITH_LETTERS: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    for bad in [
        Value::String("ZZZZ".to_owned()),
        Value::String(WITH_LETTERS.to_uppercase()),
        Value::String(WITH_LETTERS[..63].to_owned()),
        Value::String(WITH_LETTERS[..62].to_owned() + "ZZ"),
        Value::Null,
    ] {
        let mut doc = document();
        doc["artifacts"][0]["sha256"] = bad.clone();
        let failures = validate(&doc);
        assert!(
            failures
                .iter()
                .any(|failure| failure.path == "$.artifacts[0].sha256"),
            "`{bad}` passed as a SHA-256: {failures:?}"
        );
    }
}

#[test]
fn an_address_that_is_not_hex_is_rejected_and_an_unknown_address_may_be_null() {
    let mut doc = document();
    doc["symbols"][0]["address"] = serde_json::json!("8000042");
    assert!(
        validate(&doc)
            .iter()
            .any(|failure| failure.path == "$.symbols[0].address"),
        "the 0x prefix is part of the shape"
    );

    let mut nullable = document();
    nullable["symbols"][0]["address"] = Value::Null;
    let failures = validate(&nullable);
    assert!(
        failures.is_empty(),
        "null is a declared member of that field's type: {failures:?}"
    );
}

#[test]
fn an_empty_artifact_list_is_rejected_because_a_build_always_ships_something() {
    let mut doc = document();
    doc["artifacts"] = Value::Array(Vec::new());
    let failures = validate(&doc);
    assert!(
        failures.iter().any(|failure| failure.path == "$.artifacts"),
        "minItems:1 must bite: {failures:?}"
    );
}

#[test]
fn the_schema_declares_only_keywords_the_validator_implements() {
    // schema_check reports an unimplemented keyword as a failure rather than skipping it, so a construct
    // added to the schema without extending the validator would surface here as a violation instead of as
    // a quietly unchecked contract. The probe below is that check.
    let schema = analysis_schema();
    let probe = serde_json::json!({
        "anyOf": [{ "type": "string" }, { "type": "null" }]
    });
    let mut errors = Vec::new();
    schema_check::Schema::from_root(&probe).check(
        &probe,
        &Value::String("x".to_owned()),
        "$",
        &mut errors,
    );
    assert!(
        !errors.is_empty(),
        "a keyword the subset does not implement must be reported, not ignored"
    );

    // And every `pattern` the analysis schema uses is one the validator actually evaluates.
    let collected = collect_patterns(&schema);
    for pattern in &collected {
        assert!(
            matches!(
                pattern.as_str(),
                "^[0-9a-f]{64}$" | "^0x[0-9a-f]+$" | "^([0-9a-f]{40}|[0-9a-f]{64})$"
            ),
            "`{pattern}` is not in schema_check's evaluated set, so it would panic at the first document"
        );
    }
    assert!(
        !collected.is_empty(),
        "the schema is expected to pin at least the digest and address shapes"
    );
}

fn collect_patterns(value: &Value) -> Vec<String> {
    match value {
        Value::Object(map) => map
            .iter()
            .flat_map(|(key, child)| {
                if key == "pattern" {
                    vec![child.as_str().unwrap_or_default().to_owned()]
                } else {
                    collect_patterns(child)
                }
            })
            .collect(),
        Value::Array(items) => items.iter().flat_map(collect_patterns).collect(),
        _ => Vec::new(),
    }
}

#[test]
fn the_release_id_prefix_is_distinct_from_a_gate_run_id() {
    // Both are `<kind>-<sha256>`, and a bundle that confused the two would bind a release to the wrong
    // verdict. Stated here because the two prefixes live in different modules.
    assert_eq!(RELEASE_ID_PREFIX, "release-");
    assert_ne!(RELEASE_ID_PREFIX, "gate-");
}
