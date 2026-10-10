//! The private halves of publication, tested where an outside test cannot drive them.
//!
//! End-to-end behaviour lives in `tests/bundle_builder.rs`. What is here is the code that needs either the
//! crate's own types or a filesystem state no public call can produce without racing: the swap and its
//! rollback, the two name-safety rules, and the shape of an I/O description that must never quote a path.

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::gate::{GateGitFacts, GateUnknownEvidence};
use firmwaresight_core::domain::identity::Fact;

use super::*;

/// A throwaway directory, removed when the test ends.
struct Temp(PathBuf);

impl Temp {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "firmwaresight-bundle-{label}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("the temporary directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn child(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    fn leaf_names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.path())
            .expect("the directory is readable")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn row(kind: ArtifactKind, raw_path: &str, marker: char) -> VerifiedRow {
    VerifiedRow {
        class: SourceClass::Snapshot,
        kind,
        sha256: Sha256::parse(&format!("{marker}{}", "f".repeat(63))).expect("a test digest"),
        byte_size: 4_096,
        raw_path: raw_path.to_owned(),
        source: PathBuf::from(raw_path),
    }
}

// --------------------------------------------------------------------------- names (§23)

#[test]
fn a_source_path_contributes_only_its_leaf_to_a_bundle_name() {
    for raw in [
        "..\\..\\Windows\\System32\\cmd.exe",
        "/etc/passwd",
        "build/../escape.elf",
        "",
        "firmware.elf ",
    ] {
        let names = bundle_names(&[row(ArtifactKind::Elf, raw, '1')]);
        assert_eq!(names.len(), 1);
        let name = &names[0];
        assert!(
            is_safe_bundle_relative_path(&format!("{ARTIFACTS_DIR}/{name}")),
            "`{raw}` became `{name}`, which is not a safe bundle leaf"
        );
        assert!(!name.is_empty(), "`{raw}` became an empty leaf");
    }
}

#[test]
fn two_files_that_would_land_on_one_leaf_are_both_renamed_by_content() {
    // A case-insensitive filesystem reads these two as the same file, so neither name may win by order.
    let names = bundle_names(&[
        row(ArtifactKind::Elf, "build/firmware.elf", '1'),
        row(ArtifactKind::Elf, "build/FIRMWARE.elf", '2'),
    ]);
    assert_eq!(
        names,
        vec!["elf-1fffffff-firmware.elf", "elf-2fffffff-FIRMWARE.elf"],
        "both collide, so both are disambiguated; the recognizable name is kept inside each"
    );
}

#[test]
fn a_file_that_keeps_its_own_leaf_is_not_renamed_for_a_collision_that_has_none() {
    let names = bundle_names(&[
        row(ArtifactKind::Elf, "build/firmware.elf", '1'),
        row(ArtifactKind::Map, "build/firmware.map", '2'),
    ]);
    assert_eq!(names, vec!["firmware.elf", "firmware.map"]);
}

#[test]
fn a_windows_reserved_leaf_is_made_writable_rather_than_dropped() {
    let names = bundle_names(&[row(ArtifactKind::Elf, "build/CON.elf", '1')]);
    assert_eq!(names, vec!["_CON.elf".to_owned()]);
}

// --------------------------------------------------------------------------- swap (§31)

#[test]
fn an_absent_destination_is_one_rename_and_leaves_no_sibling_behind() {
    let temp = Temp::new("fresh-publish");
    let staging = temp.child(".firmwaresight-staging");
    std::fs::create_dir_all(staging.join(ARTIFACTS_DIR)).expect("the staging tree exists");
    std::fs::write(staging.join(SHA256SUMS_NAME), b"sums").expect("the staged payload exists");
    let destination = temp.child("motor-controller-1.4.2-abcdef012345");

    let backup = replace_directory(
        &staging,
        &destination,
        false,
        "motor-controller-1.4.2-abcdef012345",
    )
    .expect("a directory that is not there is a rename");

    assert!(backup.is_none(), "nothing was moved aside");
    assert!(destination.join(ARTIFACTS_DIR).is_dir());
    assert!(destination.join(SHA256SUMS_NAME).is_file());
    assert!(
        !staging.exists(),
        "the staging path was consumed by the rename"
    );
    assert_eq!(
        temp.leaf_names(),
        vec!["motor-controller-1.4.2-abcdef012345".to_owned()],
        "no staging or backup sibling is left in the folder"
    );
}

#[test]
fn an_occupied_destination_without_authorization_is_refused_and_untouched() {
    let temp = Temp::new("unauthorized");
    let destination = temp.child("bundle");
    std::fs::create_dir_all(&destination).expect("the existing bundle directory");
    write_at(&destination.join(MANIFEST_DOC_NAME), b"{}");
    let staging = temp.child("staging");
    std::fs::create_dir_all(&staging).expect("the staging directory");

    let error = replace_directory(&staging, &destination, false, "bundle")
        .expect_err("no rename happens without the release owner's authorization");

    assert!(matches!(error, BundleError::DestinationExists { .. }));
    assert_eq!(error.code(), "ERR-BUNDLE-6106");
    assert_eq!(read(&destination.join(MANIFEST_DOC_NAME)), b"{}");
    assert!(
        staging.exists(),
        "the staged tree is still there for the caller to discard"
    );
}

#[test]
fn a_swap_that_cannot_finish_puts_the_previous_bundle_back() {
    // The failure is provoked honestly: the staged tree is gone by the time the second rename runs, which is
    // what a full disk or an interrupted write looks like from here.
    let temp = Temp::new("rollback");
    let destination = temp.child("bundle");
    std::fs::create_dir_all(&destination).expect("the existing bundle directory");
    write_at(&destination.join(MANIFEST_DOC_NAME), b"the old bundle");
    let staging = temp.child("staging");
    std::fs::create_dir_all(&staging).expect("the staging directory");
    std::fs::remove_dir_all(&staging).expect("the staged tree is gone before the swap");

    let error = replace_directory(&staging, &destination, true, "bundle")
        .expect_err("a staged tree that is not there cannot be published");

    assert!(matches!(error, BundleError::WriteFailed { .. }), "{error}");
    assert_eq!(error.code(), "ERR-BUNDLE-6108");
    assert_eq!(
        read(&destination.join(MANIFEST_DOC_NAME)),
        b"the old bundle",
        "a failed replacement leaves the release owner exactly the bundle they had"
    );
    assert_eq!(
        temp.leaf_names(),
        vec!["bundle".to_owned()],
        "the backup was renamed back, not left beside it"
    );
}

#[test]
fn an_authorized_swap_keeps_the_predecessor_until_the_new_bundle_is_in_place() {
    let temp = Temp::new("replace");
    let destination = temp.child("bundle");
    std::fs::create_dir_all(&destination).expect("the existing bundle directory");
    write_at(&destination.join(MANIFEST_DOC_NAME), b"the old bundle");
    let staging = temp.child("staging");
    std::fs::create_dir_all(staging.join(ARTIFACTS_DIR)).expect("the staged tree");
    write_at(&staging.join(MANIFEST_DOC_NAME), b"the new bundle");

    let backup = replace_directory(&staging, &destination, true, "bundle")
        .expect("a staged tree publishes over a recognized bundle");

    let backup = backup.expect("the predecessor was moved aside");
    assert!(
        backup.exists(),
        "the predecessor is kept until the new bundle verifies"
    );
    assert_eq!(
        read(&destination.join(MANIFEST_DOC_NAME)),
        b"the new bundle"
    );
    assert!(!staging.exists());
}

// --------------------------------------------------------------------------- descriptions

fn write_at(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).expect("the file is written");
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).expect("the file is readable")
}

#[test]
fn an_io_failure_is_described_by_its_kind_and_never_by_the_path_it_tried() {
    let missing = Temp::new("io-detail").child("does-not-exist");
    let detail = io_detail(&std::fs::File::open(&missing).expect_err("the file is not there"));
    assert!(detail.contains("NotFound"), "{detail}");
    assert!(
        !detail.contains(&missing.to_string_lossy().to_string()),
        "the host path reached a message: {detail}"
    );
}

#[test]
fn the_host_path_scan_reads_a_drive_prefix_and_ignores_a_schema_urn() {
    assert_eq!(
        host_path_in(r#""path":"C:\work\build\firmware.elf""#),
        Some("a Windows drive path")
    );
    assert_eq!(
        host_path_in(r#""path":"/home/engineer/build/firmware.map""#),
        Some("an absolute POSIX path")
    );
    assert_eq!(host_path_in("D:/one/more"), Some("a Windows drive path"));
    // The two shapes that must not be mistaken for one: a scheme, and a digest label.
    assert_eq!(host_path_in("urn:firmwaresight:schema:analysis:1"), None);
    assert_eq!(
        host_path_in(
            "\"sha256\":\"1111111111111111111111111111111111111111111111111111111111111111\""
        ),
        None
    );
    assert_eq!(host_path_in("artifacts/firmware.elf"), None);
    // A short input has no three-byte window to inspect, and must not panic.
    assert_eq!(host_path_in("ab"), None);
    assert_eq!(host_path_in(""), None);
}

#[test]
fn a_staled_plan_names_what_moved_and_never_a_source_path() {
    let planned = InputChecks {
        run_id: "gate-1".to_owned(),
        policy_sha256: "policy".to_owned(),
        acceptances_sha256: "acceptances".to_owned(),
        sources: vec![snapshot_row(r"C:\work\build\firmware.elf", "1", 1)],
        notes: None,
    };

    // A source whose bytes moved keeps its own code, because the remediation is about the file.
    let moved = InputChecks {
        sources: vec![snapshot_row(r"C:\work\build\firmware.elf", "2", 2)],
        ..planned.clone()
    };
    let error = planned
        .compare_with(&moved)
        .expect_err("a changed source invalidates the plan");
    assert!(
        matches!(&error, BundleError::SourceArtifactChanged { name, .. } if name == "firmware.elf")
    );
    assert_eq!(error.code(), "ERR-BUNDLE-6103");
    assert!(
        !error.to_string().contains(r"C:\work"),
        "the host path reached a message: {error}"
    );

    // The three things §28 lists that are not files all reach the same refusal, naming which one moved.
    for (label, changed) in [
        (
            "Gate run id",
            InputChecks {
                run_id: "gate-2".to_owned(),
                ..planned.clone()
            },
        ),
        (
            "policy fingerprint",
            InputChecks {
                policy_sha256: "other".to_owned(),
                ..planned.clone()
            },
        ),
        (
            "acceptance set",
            InputChecks {
                acceptances_sha256: "other".to_owned(),
                ..planned.clone()
            },
        ),
    ] {
        let error = planned
            .compare_with(&changed)
            .expect_err("a changed input invalidates the plan");
        assert!(
            matches!(error, BundleError::PlanStale { .. }),
            "{label}: {error}"
        );
        assert_eq!(error.code(), "ERR-BUNDLE-6105");
        assert!(error.to_string().contains(label), "{error}");
    }

    // And the notes file, which is neither a shipped artifact nor a policy value.
    let with_notes = InputChecks {
        notes: Some(("RELEASE_NOTES.md".to_owned(), "3".repeat(64), 40)),
        ..planned.clone()
    };
    let edited = InputChecks {
        notes: Some(("RELEASE_NOTES.md".to_owned(), "4".repeat(64), 41)),
        ..planned.clone()
    };
    let error = with_notes
        .compare_with(&edited)
        .expect_err("notes that moved invalidate the plan");
    assert!(
        matches!(error, BundleError::ReleaseNotesChanged { .. }),
        "{error}"
    );
    assert_eq!(error.code(), "ERR-BUNDLE-6104");
}

/// One staleness row, spelled the way the engine builds them.
fn snapshot_row(path: &str, digest: &str, size: u64) -> SourceRow {
    source_row(SourceClass::Snapshot, path, digest, size)
}

fn source_row(class: SourceClass, path: &str, digest: &str, size: u64) -> SourceRow {
    SourceRow {
        class,
        path: path.to_owned(),
        digest: digest.repeat(8),
        size,
    }
}

#[test]
fn a_set_change_is_named_as_one_and_never_as_a_file_whose_bytes_moved() {
    let preview = vec![
        source_row(SourceClass::Snapshot, "build/firmware.elf", "1", 1),
        source_row(SourceClass::Attachment, "build/attached/app.bin", "2", 2),
        source_row(SourceClass::Attachment, "build/attached/boot.hex", "3", 3),
    ];

    // One attached file withdrawn: every remaining row is byte-for-byte the same, so the only fact is that the
    // set lost a member. The reading this replaced compared rows by position and named a file that had not
    // moved.
    let export: Vec<SourceRow> = preview
        .iter()
        .filter(|r| r.path != "build/attached/boot.hex")
        .cloned()
        .collect();
    assert_eq!(
        attachment_set_change(&preview, &export),
        Some(("removed", "boot.hex".to_owned()))
    );

    // One added, in the order the engine lists it.
    let mut longer = preview.clone();
    longer.push(source_row(
        SourceClass::Attachment,
        "build/attached/x.bin",
        "4",
        4,
    ));
    assert_eq!(
        attachment_set_change(&preview, &longer),
        Some(("added", "x.bin".to_owned()))
    );

    // Same bytes, new path: a rename, which moves no verdict.
    let renamed: Vec<SourceRow> = preview
        .iter()
        .map(|r| {
            if r.path == "build/attached/app.bin" {
                source_row(
                    SourceClass::Attachment,
                    "build/attached/renamed.bin",
                    "2",
                    2,
                )
            } else {
                r.clone()
            }
        })
        .collect();
    assert_eq!(
        attachment_set_change(&preview, &renamed),
        Some(("renamed", "renamed.bin".to_owned()))
    );

    // Nothing came or went: the set rule has no story to tell, and the bytes rule keeps its own.
    let rewritten: Vec<SourceRow> = preview
        .iter()
        .map(|r| {
            if r.class == SourceClass::Attachment {
                SourceRow {
                    digest: "9".repeat(64),
                    ..r.clone()
                }
            } else {
                r.clone()
            }
        })
        .collect();
    assert_eq!(attachment_set_change(&preview, &rewritten), None);
    assert_eq!(
        changed_leaf(&preview, &rewritten),
        "app.bin".to_owned(),
        "the first file whose bytes moved is the one to name"
    );
}

#[test]
fn a_snapshot_row_is_never_read_as_a_set_change() {
    // A run's snapshot is fixed, so its rows cannot come or go. Counted as set members, one rewritten artifact
    // would look like the release owner changing their mind about what ships.
    let preview = vec![
        source_row(SourceClass::Snapshot, "build/firmware.elf", "1", 1),
        source_row(SourceClass::Attachment, "build/attached/app.bin", "2", 2),
    ];
    let withdrawn = vec![source_row(
        SourceClass::Snapshot,
        "build/firmware.elf",
        "1",
        1,
    )];
    assert_eq!(
        attachment_set_change(&preview, &withdrawn),
        Some(("removed", "app.bin".to_owned())),
        "the attachment that went is the one to name"
    );
    // Two lists whose only difference is that a snapshot row is gone or rewritten: no attachment came or went,
    // so the set rule has nothing to say and the bytes rule keeps its own code.
    let rewritten_snapshot = vec![
        source_row(SourceClass::Snapshot, "build/firmware.elf", "5", 1),
        source_row(SourceClass::Attachment, "build/attached/app.bin", "2", 2),
    ];
    assert_eq!(
        attachment_set_change(&preview, &rewritten_snapshot),
        None,
        "one rewritten artifact is not the release owner changing what ships"
    );
    // The renamed pair is keyed by path, so a file that came or went cannot shift the blame onto its neighbour.
    let shifted: Vec<SourceRow> = preview.iter().skip(1).cloned().collect();
    assert_eq!(changed_leaf(&preview, &shifted), "a shipped artifact");
}

#[test]
fn an_attachment_the_judged_context_does_not_carry_is_refused() {
    // The three guards in `verify_attachments` are structural: no public call can hand it an attached row the
    // context never bound, a kind a release cannot attach, or a basis no observation writes. They are tested
    // here because the refusal they carry is what proves L-2 and L-4 still hold if a later caller wires the two
    // halves together in a different order.
    let temp = Temp::new("verify-attachments");
    let path = temp.child("app.bin");
    std::fs::write(&path, b"attached bytes").expect("the file is written");
    let digest = fingerprint::file_sha256(&path).expect("the file is hashed");
    let attachment = ReleaseAttachment {
        file_name: "app.bin".to_owned(),
        kind: ArtifactKind::Bin,
        sha256: Fact::known(digest.clone()),
        byte_size: 12,
        kind_basis: KindBasis::Declared,
    };
    let attached = vec![AttachedSource {
        path: path.clone(),
        attachment: attachment.clone(),
    }];
    let bound = GateContext {
        attachments: vec![attachment.as_gate_fact()],
        ..empty_context()
    };
    let rows =
        verify_attachments(&attached, &bound).expect("the row the context carries is verified");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].class, SourceClass::Attachment);
    assert_eq!(rows[0].sha256.hex(), digest);
    assert_eq!(rows[0].source, path);

    // The same row against a context that never saw it.
    let error = verify_attachments(&attached, &empty_context())
        .expect_err("an unbound attachment is not evidence");
    assert_eq!(error.code(), "ERR-INTERNAL-9004");
    assert!(error.to_string().contains("no such row"), "{error}");
    assert!(
        !error.to_string().contains(path.to_string_lossy().as_ref()),
        "the refusal carried a host path: {error}"
    );

    // A kind a release cannot attach, and a basis nothing here writes, are refused the same way.
    let elf_kind = ReleaseAttachment {
        kind: ArtifactKind::Elf,
        ..attachment.clone()
    };
    let derived = ReleaseAttachment {
        kind_basis: KindBasis::DerivedFromLeadingBytes("MZ".to_owned()),
        ..attachment.clone()
    };
    for altered in [&elf_kind, &derived] {
        let rows = vec![AttachedSource {
            path: path.clone(),
            attachment: altered.clone(),
        }];
        let context = GateContext {
            attachments: vec![altered.as_gate_fact()],
            ..empty_context()
        };
        let error = verify_attachments(&rows, &context)
            .expect_err("a fact no observation can produce is refused");
        assert_eq!(error.code(), "ERR-INTERNAL-9004", "{error}");
    }
}

/// The smallest context this module accepts, with no attachment rows in it.
fn empty_context() -> GateContext {
    GateContext {
        snapshot_id: "snap-test".to_owned(),
        artifacts: Vec::new(),
        attachments: Vec::new(),
        memory: None,
        git: GateGitFacts::unavailable("no repository was read for this test"),
        version: None,
        release_notes: None,
        growth: GateGrowthFacts::without_baseline(),
        unknown_evidence: GateUnknownEvidence::default(),
        policy: GatePolicy::default(),
    }
}
