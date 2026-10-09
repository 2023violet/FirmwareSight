//! C1-U1 release-attachment identity contract (`04_TECH/28` §5, §7.3; `ADR-0030` D-4 and D-5).
//!
//! Written before the implementation, per the round's §六 Step 2: every test here names a property the
//! design froze and the code did not yet have.

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::gate::{
    EffectiveSeverity, FindingState, GateArtifactFact, GateAttachmentFact, GateContext,
    GateGitFacts, GateGrowthFacts, GatePolicy, GateRuleId, GateUnknownEvidence, KindBasis,
    UnknownPolicy,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
use firmwaresight_core::domain::release::ReleaseError;
use firmwaresight_project::{
    AttachmentError, BundleError, LoadedProject, observe_attachment, run_id,
};

fn sha(letter: char) -> String {
    letter.to_string().repeat(64)
}

fn attachment(kind: ArtifactKind, digest: &str, byte_size: u64) -> GateAttachmentFact {
    GateAttachmentFact {
        kind,
        sha256: Fact::known(digest.to_owned()),
        byte_size,
        kind_basis: KindBasis::Declared,
    }
}

/// The same context the pre-C1 fixture describes: one analyzed ELF, clean tagged workspace, nothing
/// configured. `attachments` is the only field this unit adds to it.
fn context(attachments: Vec<GateAttachmentFact>) -> GateContext {
    GateContext {
        snapshot_id: "snap-1".to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known(sha('a')),
            byte_size: 1_024,
        }],
        attachments,
        memory: None,
        git: GateGitFacts {
            available: true,
            head_commit: Fact::known("0123456789abcdef0123456789abcdef01234567".to_owned()),
            exact_tag: Fact::known("v1.2.3".to_owned()),
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

#[test]
fn an_empty_attachment_set_keeps_the_label_at_one_and_writes_no_block() {
    let text = context(Vec::new()).canonical_input();
    assert!(
        text.starts_with("firmwaresight-gate-input/1\n"),
        "the empty set must not move the label: {}",
        text.lines().next().unwrap_or("")
    );
    assert!(
        !text.contains("attachments["),
        "an empty set must not gain a block or an empty field: {text}"
    );
}

#[test]
fn a_bound_bin_attachment_moves_the_label_to_two_and_binds_its_digest() {
    let digest = sha('1');
    let text = context(vec![attachment(ArtifactKind::Bin, &digest, 245_760)]).canonical_input();
    assert!(
        text.starts_with("firmwaresight-gate-input/2\n"),
        "a non-empty set is canonicalized under /2: {}",
        text.lines().next().unwrap_or("")
    );
    assert!(
        text.contains(&format!("  bin known:{digest}\n")),
        "the digest is the bound fact: {text}"
    );
}

#[test]
fn changing_only_the_attached_bytes_changes_the_run_id_and_not_the_snapshot() {
    let first = context(vec![attachment(ArtifactKind::Bin, &sha('1'), 16)]);
    let second = context(vec![attachment(ArtifactKind::Bin, &sha('2'), 16)]);
    assert_ne!(
        run_id(&first),
        run_id(&second),
        "different bytes, different run"
    );
    assert_eq!(
        first.snapshot_id, second.snapshot_id,
        "an attachment is never part of the analyzed build's identity"
    );
}

#[test]
fn redeclaring_the_kind_of_identical_bytes_changes_the_run_id() {
    let digest = sha('1');
    let as_bin = context(vec![attachment(ArtifactKind::Bin, &digest, 64)]);
    let as_hex = context(vec![attachment(ArtifactKind::IntelHex, &digest, 64)]);
    assert_ne!(
        run_id(&as_bin),
        run_id(&as_hex),
        "the block prints the kind word, so a re-declaration is a different judgement"
    );
}

#[test]
fn the_canonical_block_orders_by_kind_word_then_digest_and_ignores_how_they_arrive() {
    // The digests are chosen so the two keys disagree: `hex` with the smallest digest and `bin` with
    // the largest. Ordering by the digest alone puts the hex row first, so this fixture is what makes
    // "kind word first" a checked rule rather than a comment.
    let arriving = context(vec![
        attachment(ArtifactKind::IntelHex, &sha('0'), 10),
        attachment(ArtifactKind::IntelHex, &sha('3'), 9),
        attachment(ArtifactKind::Bin, &sha('9'), 8),
        attachment(ArtifactKind::Bin, &sha('1'), 7),
    ]);
    let enumerating = context(vec![
        attachment(ArtifactKind::Bin, &sha('1'), 7),
        attachment(ArtifactKind::Bin, &sha('9'), 8),
        attachment(ArtifactKind::IntelHex, &sha('0'), 10),
        attachment(ArtifactKind::IntelHex, &sha('3'), 9),
    ]);
    assert_eq!(
        arriving.canonical_attachments(),
        enumerating.canonical_attachments(),
        "the canonical order is one order per set"
    );
    assert_eq!(
        arriving.canonical_input(),
        enumerating.canonical_input(),
        "and the canonical text is one text per set"
    );
    let words: Vec<String> = arriving
        .canonical_attachments()
        .iter()
        .map(|row| {
            format!(
                "{} {}",
                row.kind.word(),
                row.sha256.value().cloned().unwrap_or_default()
            )
        })
        .collect();
    assert_eq!(
        words,
        [
            format!("bin {}", sha('1')),
            format!("bin {}", sha('9')),
            format!("hex {}", sha('0')),
            format!("hex {}", sha('3')),
        ],
        "the kind word is the primary key and the digest only breaks a tie within one kind"
    );
}

#[test]
fn an_exact_kind_and_digest_pair_is_bound_once_however_many_times_it_arrives() {
    let once = context(vec![attachment(ArtifactKind::Bin, &sha('1'), 4_096)]);
    let twice = context(vec![
        attachment(ArtifactKind::Bin, &sha('1'), 4_096),
        attachment(ArtifactKind::Bin, &sha('1'), 4_096),
    ]);
    assert_eq!(
        twice.canonical_attachments().len(),
        1,
        "two rows of identical bytes say one thing twice"
    );
    assert_eq!(
        run_id(&once),
        run_id(&twice),
        "and a set is the honest model of what a digest binds"
    );
}

#[test]
fn two_files_of_one_kind_with_different_bytes_are_both_bound() {
    let text = context(vec![
        attachment(ArtifactKind::Bin, &sha('1'), 7),
        attachment(ArtifactKind::Bin, &sha('2'), 8),
    ])
    .canonical_input();
    assert!(
        text.contains(&format!("  bin known:{}\n", sha('1'))),
        "{text}"
    );
    assert!(
        text.contains(&format!("  bin known:{}\n", sha('2'))),
        "{text}"
    );
}

#[test]
fn the_attachment_block_carries_no_name_no_path_and_no_size() {
    let text = context(vec![attachment(ArtifactKind::Bin, &sha('1'), 245_760)]).canonical_input();
    let block = text
        .split("attachments[\n")
        .nth(1)
        .and_then(|rest| rest.split("]\n").next())
        .expect("the block is present for a non-empty set");
    assert_eq!(
        block.lines().filter(|line| !line.trim().is_empty()).count(),
        1,
        "one row per bound attachment: {block:?}"
    );
    for forbidden in ["245760", "bin.bin", "\\", "C:", "/", "name"] {
        assert!(
            !block.contains(forbidden),
            "the block named {forbidden}, which is not Gate identity: {block:?}"
        );
    }
}

#[test]
fn a_size_claim_difference_alone_is_not_a_gate_identity_input() {
    let smaller = context(vec![attachment(ArtifactKind::Bin, &sha('1'), 1_024)]);
    let claimed = context(vec![attachment(ArtifactKind::Bin, &sha('1'), 2_048)]);
    assert_eq!(
        run_id(&smaller),
        run_id(&claimed),
        "size is a stored fact checked at packaging, not a Gate identity input (`04_TECH/28` §5 rule 1)"
    );
    assert_ne!(
        smaller.attachments[0].byte_size, claimed.attachments[0].byte_size,
        "while the two rows do differ in the fact they carry"
    );
}

#[test]
fn the_projects_own_policy_fingerprint_and_the_recorded_run_id_are_untouched() {
    // §四 E, T-C1-16: the before-and-after comparison is made against the repository's own project and the
    // frozen golden document rather than a number copied into this file. The manifest's
    // `extensions.policy_sha256` was written by the pre-C1 build, so recomputing it from the same
    // configuration is the measurement. The second half asserts the golden run id was not regenerated to
    // match new code.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository root");
    let loaded =
        LoadedProject::load_file(&root.join("fixtures/project/p4-release/firmwaresight.toml"))
            .expect("the repository's own fixture project loads");

    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("golden/reports/p4-release/release-manifest.json"))
            .expect("the frozen golden manifest reads"),
    )
    .expect("the golden manifest is JSON");
    assert_eq!(
        loaded.policy_sha256,
        manifest["extensions"]["policy_sha256"]
            .as_str()
            .expect("the golden records a policy fingerprint"),
        "C1-U1 added a fact set to the Gate context and changed no policy vocabulary"
    );

    let gate: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("golden/reports/p4-release/gate-results.json"))
            .expect("the frozen golden gate result reads"),
    )
    .expect("the golden gate document is JSON");
    assert_eq!(
        gate["run_id"].as_str(),
        Some("gate-a17e9861ce643da5e7061ac349793f80d1be87736ad52b2daf3433dc8cb8bc26"),
        "the run id the pre-C1 build recorded is still the one the golden carries"
    );
}

#[test]
fn an_attachment_still_satisfies_nothing_because_the_rules_do_not_read_it_yet() {
    // C1-U1's boundary as a test: `artifacts.required` is `C1-U2`'s work. A required BIN with only an
    // attachment bound must still be refused, so this unit cannot be mistaken for a shipped capability.
    let policy = GatePolicy {
        required_artifact_kinds: vec!["elf".to_owned(), "bin".to_owned()],
        ..Default::default()
    };
    let mut bound = context(vec![attachment(ArtifactKind::Bin, &sha('1'), 4_096)]);
    bound.policy = policy;
    let evaluation = bound.evaluate(&run_id(&bound));
    let finding = evaluation
        .finding(GateRuleId::RequiredArtifacts)
        .expect("every rule answers");
    assert_eq!(
        finding.state,
        FindingState::Block,
        "an attached BIN must not yet satisfy the rule: {}",
        finding.summary
    );
}

// ── observation and its typed refusals: U1-01 and U1-02, `04_TECH/28` §3 and §4.3 E-1/E-2/E-3 ──

/// A scratch root per test, named by label so two tests never share a folder. The bytes inside are
/// this file's own test material, created and removed here.
fn temp_root(label: &str) -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!("firmwaresight-c1u1-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the scratch attachment root was created");
    root
}

fn write_scratch(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = root.join(name);
    std::fs::write(&path, bytes).expect("the scratch attachment was written");
    path
}

/// The published SHA-256 of the three bytes `abc`, quoted so the digest is checked against an outside
/// authority rather than against this workspace's own hasher.
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn digest(fact: &Fact<String>) -> String {
    fact.value().cloned().expect("an observed digest")
}

#[test]
fn an_observed_bin_carries_a_digest_a_length_and_a_declared_basis_and_nothing_else() {
    let root = temp_root("observe-bin");
    let path = write_scratch(&root, "firmware.bin", b"abc");
    let observed = observe_attachment(&path, ArtifactKind::Bin)
        .expect("a regular, non-empty file is observed");

    assert_eq!(observed.file_name, "firmware.bin");
    assert_eq!(observed.kind, ArtifactKind::Bin);
    assert_eq!(
        observed.byte_size, 3,
        "the length of the bytes actually read"
    );
    assert_eq!(digest(&observed.sha256), ABC_SHA256);
    assert_eq!(observed.kind_basis, KindBasis::Declared);
    // The Gate's own row is the same facts with the name removed: a name is release identity, not
    // Gate identity (`04_TECH/28` §5, rules 4 and 5).
    assert_eq!(
        observed.as_gate_fact(),
        attachment(ArtifactKind::Bin, ABC_SHA256, 3)
    );
    // Nothing here read a header, so no analyzed-kind vocabulary may appear in the fact.
    let shown = format!("{observed:?}").to_lowercase();
    for forbidden in ["section", "symbol", "entry point", "machine"] {
        assert!(
            !shown.contains(forbidden),
            "the observation claimed structure it never read: {shown}"
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_elf_or_map_offered_as_an_attachment_is_refused_by_kind_before_any_read() {
    // A path that does not exist proves the order: the kind is refused without touching the
    // filesystem, so an analyzed kind cannot be smuggled in by naming a file.
    for kind in [ArtifactKind::Elf, ArtifactKind::Map] {
        let error = observe_attachment(Path::new("firmwaresight-c1u1-never-read.bin"), kind)
            .expect_err("an analyzed kind is never an attachment");
        assert_eq!(error, AttachmentError::KindNotAttaching { kind });
        assert_eq!(error.code(), "ERR-BUNDLE-6116");
    }
}

#[test]
fn an_intel_hex_or_unknown_kind_is_observed_because_the_design_allows_it() {
    let root = temp_root("observe-other-kinds");
    let hex = write_scratch(&root, "image.hex", b"abc");
    let unknown = write_scratch(&root, "payload.dat", b"abc");
    for (kind, path) in [
        (ArtifactKind::IntelHex, hex.clone()),
        (ArtifactKind::Unknown, unknown.clone()),
    ] {
        let observed = observe_attachment(&path, kind).expect("an attachable kind is observed");
        assert_eq!(observed.kind, kind);
        assert_eq!(digest(&observed.sha256), ABC_SHA256);
        assert_eq!(observed.kind_basis, KindBasis::Declared);
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_zero_byte_attachment_is_refused_before_any_digest_is_recorded() {
    let root = temp_root("observe-empty");
    let path = write_scratch(&root, "nothing.bin", b"");
    let error =
        observe_attachment(&path, ArtifactKind::Bin).expect_err("an empty file is not a BIN image");
    assert!(
        matches!(&error, AttachmentError::Empty { name } if name == "nothing.bin"),
        "{error:?}"
    );
    assert_eq!(error.code(), "ERR-BUNDLE-6117");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_directory_or_a_missing_file_is_refused_as_unreadable_without_naming_its_host_path() {
    let root = temp_root("observe-unreadable");
    let directory = root.join("not-a-file.bin");
    std::fs::create_dir_all(&directory).expect("the scratch directory was created");
    let missing = root.join("gone.bin");
    let root_text = root.to_string_lossy().to_string();

    for (path, leaf) in [(&directory, "not-a-file.bin"), (&missing, "gone.bin")] {
        let error = observe_attachment(path, ArtifactKind::Bin)
            .expect_err("neither path is a readable regular file");
        assert_eq!(error.code(), "ERR-BUNDLE-6115", "{error:?}");
        let shown = format!(
            "{error} {} {}",
            error.remediation(),
            error.name().unwrap_or("<no name>")
        );
        assert!(
            !shown.contains(&root_text),
            "a refusal carried the host directory it was refused in: {shown}"
        );
        assert!(
            shown.contains(leaf),
            "the refusal lost the name a person can act on: {shown}"
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_three_refusals_are_three_distinct_codes_each_with_its_own_next_step() {
    let refusals = [
        AttachmentError::Unreadable {
            name: "firmware.bin".to_owned(),
            detail: "permission denied".to_owned(),
        },
        AttachmentError::KindNotAttaching {
            kind: ArtifactKind::Elf,
        },
        AttachmentError::Empty {
            name: "firmware.bin".to_owned(),
        },
    ];
    let codes: Vec<&str> = refusals
        .iter()
        .map(|error| -> &str { error.code() })
        .collect();
    assert_eq!(
        codes,
        ["ERR-BUNDLE-6115", "ERR-BUNDLE-6116", "ERR-BUNDLE-6117"]
    );
    let remedies: Vec<&str> = refusals
        .iter()
        .map(|error| -> &str { error.remediation() })
        .collect();
    let mut distinct = remedies.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        3,
        "two different refusals tell the owner to do the same thing: {remedies:?}"
    );
    for (error, code) in refusals.iter().zip(&codes) {
        assert!(!error.remediation().trim().is_empty(), "{error:?}");
        // A user-fixable input refusal must not be filed as a software bug.
        assert!(
            code.starts_with("ERR-BUNDLE-"),
            "{code} is not a release-family code"
        );
    }
}

#[test]
fn the_new_codes_take_no_number_that_an_existing_refusal_already_uses() {
    // The registry is these `code()` matches, measured from
    // `crates/firmwaresight-core/src/domain/release.rs`,
    // `crates/firmwaresight-project/src/bundle.rs` and the two Desktop files that own 6111-6114.
    const RELEASE_FAMILY: [&str; 14] = [
        "ERR-BUNDLE-6101",
        "ERR-BUNDLE-6102",
        "ERR-BUNDLE-6103",
        "ERR-BUNDLE-6104",
        "ERR-BUNDLE-6105",
        "ERR-BUNDLE-6106",
        "ERR-BUNDLE-6107",
        "ERR-BUNDLE-6108",
        "ERR-BUNDLE-6109",
        "ERR-BUNDLE-6110",
        "ERR-BUNDLE-6111",
        "ERR-BUNDLE-6112",
        "ERR-BUNDLE-6113",
        "ERR-BUNDLE-6114",
    ];
    // The engine's answers are called live rather than copied, so a renumbering fails this test.
    let engine: Vec<&str> = [
        BundleError::Release(ReleaseError::GateNotReady {
            disposition: EffectiveSeverity::Review,
        }),
        BundleError::Release(ReleaseError::NoReleaseArtifacts),
        BundleError::GateContextChanged {
            selected: "gate-1".to_owned(),
            recomputed: "gate-2".to_owned(),
        },
        BundleError::SourceArtifactChanged {
            name: "firmware.elf".to_owned(),
            detail: "the digest differs".to_owned(),
        },
        BundleError::ReleaseNotesChanged {
            detail: "the digest differs".to_owned(),
        },
        BundleError::PlanStale {
            detail: "the run id moved".to_owned(),
        },
        BundleError::DestinationExists {
            display: "brake-node-1.2.3-de87846cad6b".to_owned(),
        },
        BundleError::UnsafeDestination {
            display: "somewhere".to_owned(),
            detail: "not a FirmwareSight bundle".to_owned(),
        },
        BundleError::WriteFailed {
            name: "release-manifest.json".to_owned(),
            detail: "the disk was full".to_owned(),
        },
        BundleError::VerificationFailed {
            detail: "SHA256SUMS disagrees".to_owned(),
        },
        BundleError::Release(ReleaseError::VersionUnavailable {
            reason: "no tag matched".to_owned(),
        }),
    ]
    .iter()
    .map(|error| error.code())
    .collect();
    for code in &engine {
        assert!(
            RELEASE_FAMILY.contains(code),
            "an existing refusal left the 6101-6114 family: {code}"
        );
    }
    let added = [
        AttachmentError::Unreadable {
            name: "firmware.bin".to_owned(),
            detail: "permission denied".to_owned(),
        }
        .code(),
        AttachmentError::KindNotAttaching {
            kind: ArtifactKind::Elf,
        }
        .code(),
        AttachmentError::Empty {
            name: "firmware.bin".to_owned(),
        }
        .code(),
    ];
    for code in added {
        assert!(
            !RELEASE_FAMILY.contains(&code),
            "{code} was already owned by another refusal"
        );
        assert!(
            !engine.contains(&code),
            "{code} collides with a live engine code"
        );
    }
    assert_eq!(
        added,
        ["ERR-BUNDLE-6115", "ERR-BUNDLE-6116", "ERR-BUNDLE-6117"]
    );
}

#[test]
fn an_observed_attachment_bound_to_a_context_is_the_same_fact_the_identity_reads() {
    // The two authorities agree: the row the observation produces and the row the Gate canonicalizes
    // are one value, so nothing re-derives a digest on the way into the context.
    let root = temp_root("observe-into-context");
    let path = write_scratch(&root, "firmware.bin", b"abc");
    let observed = observe_attachment(&path, ArtifactKind::Bin)
        .expect("a regular, non-empty file is observed");
    let bound = context(vec![observed.as_gate_fact()]);
    assert!(
        bound
            .canonical_input()
            .contains(&format!("bin known:{ABC_SHA256}")),
        "{}",
        bound.canonical_input()
    );
    assert_eq!(bound.attachments.len(), 1);
    let _ = std::fs::remove_dir_all(&root);
}
