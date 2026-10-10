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

/// The boundary `C1-U1` locked is now the rule it locked against: an attached BIN satisfies a required BIN
/// (`ADR-0030` D-3), and the missing-attachment half of that boundary is kept as its negative rather than
/// deleted with it.
#[test]
fn a_bound_bin_attachment_satisfies_a_required_bin_and_an_unbound_one_still_blocks() {
    let policy = GatePolicy {
        required_artifact_kinds: vec!["elf".to_owned(), "bin".to_owned()],
        ..Default::default()
    };

    let mut bound = context(vec![attachment(ArtifactKind::Bin, &sha('1'), 4_096)]);
    bound.policy = policy.clone();
    let satisfied = bound.evaluate(&run_id(&bound));
    let finding = satisfied
        .finding(GateRuleId::RequiredArtifacts)
        .expect("every rule answers");
    assert_eq!(
        finding.state,
        FindingState::Pass,
        "C1-U2 makes a required BIN satisfiable by the bytes that ship it: {}",
        finding.summary
    );
    assert!(
        finding
            .evidence_refs
            .iter()
            .any(|reference| reference.starts_with("attachment:bin:")),
        "the satisfied kind is cited by the scheme that satisfied it: {:?}",
        finding.evidence_refs
    );

    let mut unbound = context(Vec::new());
    unbound.policy = policy;
    let refused = unbound.evaluate(&run_id(&unbound));
    let finding = refused
        .finding(GateRuleId::RequiredArtifacts)
        .expect("every rule answers");
    assert_eq!(
        finding.state,
        FindingState::Block,
        "the same requirement with nothing attached is still a refusal: {}",
        finding.summary
    );
}

// §5.D's one sanctioned path is proven where a real snapshot lives:
// `bundle_builder.rs::the_attachment_rows_a_context_binds_are_the_facts_the_observation_produced`.

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
        // C1-U2's E-4, taken from the same live `code()` the engine answers with.
        BundleError::AttachmentSetChanged {
            name: "firmware.bin".to_owned(),
            change: "added",
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
        [
            "ERR-BUNDLE-6115",
            "ERR-BUNDLE-6116",
            "ERR-BUNDLE-6117",
            "ERR-BUNDLE-6118"
        ]
    );
}

/// §7: the observation refusals cross into the bundle surface with their own code and next step, and E-4
/// names a membership change instead of blaming a file that did not move.
#[test]
fn the_bundle_surface_reports_each_attachment_refusal_as_its_own_code() {
    let unreadable = BundleError::from(AttachmentError::Unreadable {
        name: "app.bin".to_owned(),
        detail: "the file disappeared during the read".to_owned(),
    });
    let not_attaching = BundleError::from(AttachmentError::KindNotAttaching {
        kind: ArtifactKind::Elf,
    });
    let empty = BundleError::from(AttachmentError::Empty {
        name: "app.bin".to_owned(),
    });
    let bridged: Vec<(&BundleError, &str)> = vec![
        (&unreadable, "ERR-BUNDLE-6115"),
        (&not_attaching, "ERR-BUNDLE-6116"),
        (&empty, "ERR-BUNDLE-6117"),
    ];
    for (error, code) in bridged {
        assert_eq!(error.code(), code, "{error}");
        // E-1..E-3 keep the remediation the observation wrote, not a bundle-side re-wording of it.
        assert!(
            !error.remediation().trim().is_empty(),
            "{error} lost its next step"
        );
        assert!(
            !error.to_string().contains('/') && !error.to_string().contains('\\'),
            "a bridged refusal carried a path: {error}"
        );
    }

    let changed = BundleError::AttachmentSetChanged {
        name: "app.bin".to_owned(),
        change: "added",
    };
    assert_eq!(changed.code(), "ERR-BUNDLE-6118");
    assert_eq!(
        changed.to_string(),
        "the release attachment set changed after the preview: `app.bin` was added"
    );
    assert!(
        changed.remediation().contains("prepare the bundle again"),
        "{}",
        changed.remediation()
    );
    assert!(
        changed.remediation().contains("recheck"),
        "the next step must also say what to look at: {}",
        changed.remediation()
    );
    for form in ["added", "removed", "renamed"] {
        let error = BundleError::AttachmentSetChanged {
            name: "app.bin".to_owned(),
            change: form,
        };
        assert_eq!(error.code(), "ERR-BUNDLE-6118", "{error}");
        assert!(error.to_string().contains(form), "{error}");
    }
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

// ------------------------------------------------------- C1-U3 the pre-C1 corpus, read by today's contracts

/// The repository root, reached the way
/// `the_projects_own_policy_fingerprint_and_the_recorded_run_id_are_untouched` reaches it.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository root")
}

/// The JSON documents the frozen P4 bundle carries, found by listing the directory. `T-C1-06` says *every*
/// pre-C1 document, so the corpus is measured here rather than transcribed: a sixth document dropped into
/// that bundle would be validated by this test instead of silently unlisted.
fn golden_json_documents() -> Vec<(String, PathBuf)> {
    let dir = repo_root().join("golden/reports/p4-release");
    let mut found = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            (
                path.file_name()
                    .expect("a bundle document has a name")
                    .to_string_lossy()
                    .into_owned(),
                path,
            )
        })
        .collect::<Vec<_>>();
    found.sort();
    found
}

/// The contract each document names, by the spelling the bundle itself uses.
fn schema_for(name: &str) -> Option<&'static str> {
    Some(match name {
        "analysis.json" => "analysis.schema.json",
        "diff.json" => "diff.schema.json",
        "gate-results.json" => "gate-results.schema.json",
        "accepted-reviews.json" => "accepted-reviews.schema.json",
        "release-manifest.json" => "release-manifest.schema.json",
        _ => return None,
    })
}

fn schema(name: &str) -> serde_json::Value {
    let path = repo_root().join("schemas").join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{name} must parse: {error}"))
}

#[test]
fn every_pre_c1_bundle_document_still_validates_against_the_schema_it_names() {
    // `T-C1-06`, and the compatibility claim `ADR-0030` D-4 rests on: five portable contracts, all still at
    // major 1, still accept the documents a pre-C1 build wrote. No golden byte is edited to make that true,
    // and no schema file was widened for C1 — the second half of this test asserts the corpus carries no
    // C1 disclosure at all, which is the same fact from the reader's side.
    let documents = golden_json_documents();
    let names = documents
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "accepted-reviews.json",
            "analysis.json",
            "diff.json",
            "gate-results.json",
            "release-manifest.json",
        ],
        "the corpus this test reads is the corpus the bundle holds"
    );

    for (name, path) in &documents {
        let schema_name = schema_for(name).expect("every bundle document names a contract");
        let text = std::fs::read_to_string(path).expect("the golden document reads");
        let document: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{name}: {error}"));
        let contract = schema(schema_name);

        let errors = firmwaresight_report::schema_check::validate(&contract, &document);
        assert!(errors.is_empty(), "{name} must still validate: {errors:?}");

        let declared = document
            .get("schema_version")
            .or(document.get("schemaVersion"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
        assert_eq!(declared, 1, "{name} is a major-1 document");
        assert!(
            contract["$id"]
                .as_str()
                .expect("every published contract carries its URN")
                .ends_with(":1"),
            "{schema_name} is still published at major 1"
        );

        let extensions = document.get("extensions");
        for key in ["attachments", "canonical_input_label"] {
            let present = extensions
                .and_then(|value| value.get(key))
                .is_some_and(|value| !value.is_null());
            assert!(
                !present,
                "{name} predates C1 and must not be read as if it disclosed {key}"
            );
        }
    }
}

#[test]
fn the_frozen_bundle_indexes_the_bytes_it_holds_and_names_the_two_it_does_not() {
    // `T-C1-10`'s golden half: the index a pre-C1 build wrote still matches the bytes on disk, recomputed
    // here rather than trusted. The two entries it cannot check are the two the golden deliberately does not
    // hold — `apps/cli/tests/p4_golden.rs` pins that this corpus is documents without artifact bytes — and
    // naming them is what makes the next test's copy-in an addition rather than a surprise.
    let dir = repo_root().join("golden/reports/p4-release");
    let index = std::fs::read_to_string(dir.join("SHA256SUMS")).expect("the golden index reads");
    let mut absent: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for line in index.lines().filter(|line| !line.trim().is_empty()) {
        let (digest, path) = line
            .split_once("  ")
            .unwrap_or_else(|| panic!("an index line is `<digest>  <path>`: {line}"));
        let file = dir.join(path);
        if file.is_file() {
            let bytes = std::fs::read(&file).expect("the indexed file reads");
            assert_eq!(
                firmwaresight_project::fingerprint::sha256_hex(&bytes),
                digest,
                "`{path}` must hash to the line its own index carries"
            );
            checked += 1;
        } else {
            absent.push(path.to_owned());
        }
    }
    // Measured, not assumed: the index has eight lines (the manifest indexes nine entries, because it also
    // indexes `SHA256SUMS`, which by §20's own rule carries no digest of itself), six of them on disk here.
    assert_eq!(
        checked, 6,
        "six of the eight indexed files are in the corpus"
    );
    assert_eq!(
        absent,
        vec![
            "artifacts/firmware.elf".to_owned(),
            "artifacts/firmware.map".to_owned()
        ],
        "and the two that are not are the artifact bytes the golden deliberately omits"
    );
}

#[test]
fn the_relocated_golden_bundle_verifies_and_re_derives_the_release_id_it_claims() {
    // `T-C1-14`'s read-back on the immutable corpus: the P4 bundle, moved into scratch, given the two
    // committed fixtures its own index already names, and verified by this build's `verify_bundle` — which
    // since `C1-U3` also reads the Gate disclosure pair and ties the run id. A pre-C1 bundle must pass that
    // stricter reader unchanged, or the stricter reader is a breaking change and this test is the STOP.
    let scratch = temp_root("golden-relocated");
    let source = repo_root().join("golden/reports/p4-release");
    copy_dir(&source, &scratch);
    let artifacts = scratch.join("artifacts");
    std::fs::create_dir_all(&artifacts).expect("the artifacts folder is made");
    for name in ["firmware.elf", "firmware.map"] {
        std::fs::copy(
            repo_root().join("fixtures/elf/p2-diff/target").join(name),
            artifacts.join(name),
        )
        .expect("the committed fixture the index names is copied in");
    }

    let verification = firmwaresight_project::bundle::verify_bundle(&scratch)
        .expect("the frozen bundle verifies under this build's verifier");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(scratch.join("release-manifest.json"))
            .expect("the manifest reads"),
    )
    .expect("the manifest is JSON");
    assert_eq!(
        verification.release_id,
        manifest["release"]["id"].as_str().expect("a release id"),
        "the id the bytes describe is the id the bundle claims"
    );
    assert_eq!(
        verification.release_id,
        "release-e400a8ac51a57d34e309d254536f8850044ef4dd53e26245045c24fdc86fdcc5",
        "and that id is the one P4 froze, not one regenerated to suit this code"
    );
    assert_eq!(
        verification.artifact_count, 2,
        "the analyzed ELF and MAP ship"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}

/// The copy the relocated read-back needs: documents as they are, in a directory that is not the repository.
fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("the source reads") {
        let entry = entry.expect("one entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            std::fs::create_dir_all(&target).expect("the directory is made");
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("the file is copied");
        }
    }
}

#[test]
fn the_disclosed_rows_rebuild_the_text_the_run_id_was_taken_over() {
    // `T-C1-05`'s document half: the rows a portable document discloses are enough, beside the saved context
    // the rest of the grammar needs, to re-derive the same canonical text and therefore the same run id. A
    // reader that recomputes from the document detects a moved byte; it does not have to trust the id.
    let rows = vec![
        attachment(ArtifactKind::IntelHex, &sha('7'), 683_214),
        attachment(ArtifactKind::Bin, &sha('1'), 4_096),
    ];
    let judged = context(rows);
    let id = run_id(&judged);
    let text = judged.canonical_input();
    let document =
        serde_json::to_value(firmwaresight_report::gate::GateResultsDto::from_evaluation(
            &judged,
            &judged.evaluate(&id),
            &sha('p'),
            1,
        ))
        .expect("a DTO always serializes");

    let disclosed = document["extensions"]["attachments"]
        .as_array()
        .expect("the pair is present for a bound run");
    assert_eq!(
        disclosed
            .iter()
            .filter_map(|row| row["kind"].as_str())
            .collect::<Vec<_>>(),
        vec!["bin", "hex"],
        "the document arrives in the identity's order, not the order the owner attached in"
    );
    let rebuilt = disclosed
        .iter()
        .map(|row| {
            attachment(
                attachable_kind(row["kind"].as_str().expect("a kind word")),
                row["sha256"].as_str().expect("a digest"),
                row["byte_size"].as_u64().expect("a length"),
            )
        })
        .collect::<Vec<_>>();
    let reread = context(rebuilt);
    assert_eq!(
        reread.canonical_input(),
        text,
        "the document's rows rebuild the identity's own text"
    );
    assert_eq!(run_id(&reread), id, "and therefore the same run id");

    // And the recomputation discriminates: one disclosed digest moved is a different run, not the same run
    // described slightly differently.
    let mut moved = disclosed.clone();
    moved[0]["sha256"] = serde_json::Value::String(sha('2'));
    let drift = moved
        .iter()
        .map(|row| {
            attachment(
                attachable_kind(row["kind"].as_str().expect("a kind word")),
                row["sha256"].as_str().expect("a digest"),
                row["byte_size"].as_u64().expect("a length"),
            )
        })
        .collect::<Vec<_>>();
    assert_ne!(
        run_id(&context(drift)),
        id,
        "a recomputed id must move when a disclosed digest moves"
    );
}

/// The kind words a release may attach, read the way a reader of the disclosure reads them.
fn attachable_kind(word: &str) -> ArtifactKind {
    match word {
        "bin" => ArtifactKind::Bin,
        "hex" => ArtifactKind::IntelHex,
        "unknown" => ArtifactKind::Unknown,
        other => panic!("`{other}` is not a kind a release can attach"),
    }
}
