//! P1 Analyze details over IPC: the three bounded query commands, their typed shapes, and the
//! errors around them.
//!
//! The storage layer owns what a page contains; this file owns what a *boundary* may say about it:
//!
//! - an address or a file offset crosses as hexadecimal text and is never unit-converted, while a
//!   byte count crosses as the byte count the artifact carries, because the unit switch is
//!   presentation and presentation lives in the UI (US-001);
//! - a fact that was stored as unknown crosses as a reason, never as a zero and never as an empty
//!   string that a table would render as nothing;
//! - the page ceiling is applied here as well, so a caller cannot ask the shell for more than Rust
//!   will produce;
//! - an unknown snapshot id is a typed envelope, not a panic and not an empty table;
//! - and no detail command names a path: the payload a user's own artifact produced must not
//!   locate it on the machine that produced it.
//!
//! These tests call the same `Session` methods the commands do, with no window and no runtime, the
//! way the parity file does.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::ipc::{
    EvidenceClassDto, EvidenceRequestDto, EvidenceSortDto, SectionRequestDto, SectionSortDto,
    SortDirDto, SymbolRequestDto, SymbolSortDto,
};
use firmwaresight_desktop::{FixtureKey, Session, service::FixtureCatalog};
use serde_json::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // apps/desktop/src-tauri -> apps/desktop
        .and_then(Path::parent) // -> apps
        .and_then(Path::parent) // -> repo root
        .expect("the desktop shell lives at <root>/apps/desktop/src-tauri")
        .to_path_buf()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-p1-details-{label}-{}-{:x}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_file(&path);
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let mut candidate = self.0.as_os_str().to_os_string();
            candidate.push(suffix);
            let _ = std::fs::remove_file(Path::new(&candidate));
        }
    }
}

/// A session with the dual-region fixture already analyzed and stored, plus the snapshot id the UI
/// would hold as its last-good id.
fn session_with_history(
    db: &TempDb,
) -> (
    Session,
    String,
    firmwaresight_desktop::ipc::AnalysisSummaryDto,
) {
    let session = Session::open(FixtureCatalog::from_environment(), db.path())
        .expect("a session opens against a temporary database");
    let summary = session
        .analyze_and_store(FixtureKey::P0DualRegion, "op-details")
        .expect("the committed fixture analyzes and stores");
    let snapshot_id = summary.identity.snapshot_id.clone();
    (session, snapshot_id, summary)
}

fn json(value: &impl serde::Serialize) -> Value {
    serde_json::from_str(&serde_json::to_string(value).expect("serializes")).expect("round trip")
}

#[test]
fn sections_cross_ipc_with_addresses_as_hex_and_sizes_as_untouched_bytes() {
    let db = TempDb::new("sections-shape");
    let (session, snapshot_id, summary) = session_with_history(&db);

    let page = session
        .query_sections(
            &SectionRequestDto {
                snapshot_id,
                ..SectionRequestDto::default()
            },
            "op-sections",
        )
        .expect("the stored snapshot answers");

    assert_eq!(page.total, summary.section_count);
    assert_eq!(page.rows.len(), summary.section_count);

    let text = page
        .rows
        .iter()
        .find(|row| row.name.as_deref() == Some(".text"))
        .expect("the fixture has a .text section");

    // `sh_addr` 0x8000000 and `sh_offset` 0x1000 are locators, so they stay hexadecimal and are
    // never scaled; 60 is the stored payload in bytes, with no KiB arithmetic on the Rust side.
    assert_eq!(text.virtual_address.as_deref(), Some("0x08000000"));
    assert_eq!(text.file_offset.as_deref(), Some("0x00001000"));
    assert_eq!(text.file_size, 60);
    assert_eq!(text.memory_size, Some(60));
    assert_eq!(text.role, "Code");
    assert!(text.alloc && text.execute && !text.write);

    let wire = json(&page);
    assert_eq!(wire["rows"][0]["index"].as_u64(), Some(1));
    assert!(
        wire["rows"][0]
            .get("addressKiB")
            .xor(wire["rows"][0].get("address_kb"))
            .is_none(),
        "a byte-scaled address must not exist at the boundary at all"
    );
}

#[test]
fn a_section_without_load_evidence_crosses_as_a_reason_and_never_as_zero() {
    let db = TempDb::new("sections-unknown");
    let (session, snapshot_id, _) = session_with_history(&db);

    let page = session
        .query_sections(
            &SectionRequestDto {
                snapshot_id,
                ..SectionRequestDto::default()
            },
            "op-sections-unknown",
        )
        .expect("queries");

    let unknown: Vec<_> = page
        .rows
        .iter()
        .filter(|row| row.load_address.is_none())
        .collect();
    assert!(!unknown.is_empty(), "this fixture has unplaced sections");
    for row in unknown {
        assert_eq!(
            row.load_address.as_deref(),
            None,
            "an unplaced section has no load address, so none crosses"
        );
        assert!(
            row.load_address_unknown_reason
                .as_deref()
                .is_some_and(|reason| !reason.is_empty()),
            "the reason has to cross with the absence: {:?}",
            row.load_address_unknown_reason
        );
    }

    // The MAP-backed build does place sections, which is the other half of the claim: unknown is a
    // property of the row, not a default the whole table shares.
    let placed = page
        .rows
        .iter()
        .find(|row| row.name.as_deref() == Some(".data"))
        .expect("the fixture has a .data section");
    assert_eq!(placed.load_address.as_deref(), Some("0x0800005c"));
    assert_eq!(placed.load_address_unknown_reason, None);
    assert_eq!(placed.region.as_deref(), Some("RAM"));
}

#[test]
fn the_detail_commands_apply_the_same_page_bounds_as_the_storage_layer() {
    let db = TempDb::new("page-bounds");
    let (session, snapshot_id, _) = session_with_history(&db);

    let unspecified = session
        .query_sections(
            &SectionRequestDto {
                snapshot_id: snapshot_id.clone(),
                ..SectionRequestDto::default()
            },
            "op-default",
        )
        .expect("default page");
    assert_eq!(
        unspecified.limit, 100,
        "the default page size is the boundary's"
    );

    for command in ["sections", "symbols", "evidence"] {
        let oversized = match command {
            "sections" => session
                .query_sections(
                    &SectionRequestDto {
                        snapshot_id: snapshot_id.clone(),
                        limit: Some(100_000),
                        ..SectionRequestDto::default()
                    },
                    "op-clamp",
                )
                .map(|page| page.limit)
                .expect("query"),
            "symbols" => session
                .query_symbols(
                    &SymbolRequestDto {
                        snapshot_id: snapshot_id.clone(),
                        limit: Some(100_000),
                        ..SymbolRequestDto::default()
                    },
                    "op-clamp",
                )
                .map(|page| page.limit)
                .expect("query"),
            _ => session
                .query_evidence(
                    &EvidenceRequestDto {
                        snapshot_id: snapshot_id.clone(),
                        limit: Some(100_000),
                        ..EvidenceRequestDto::default()
                    },
                    "op-clamp",
                )
                .map(|page| page.limit)
                .expect("query"),
        };
        assert_eq!(oversized, 500, "{command} must clamp to the hard maximum");
    }
}

#[test]
fn a_details_page_names_the_next_offset_the_reader_would_ask_for() {
    let db = TempDb::new("page-cursor");
    let (session, snapshot_id, summary) = session_with_history(&db);

    let first = session
        .query_sections(
            &SectionRequestDto {
                snapshot_id: snapshot_id.clone(),
                limit: Some(10),
                ..SectionRequestDto::default()
            },
            "op-first",
        )
        .expect("first page");
    assert_eq!(first.rows.len(), 10);
    assert_eq!(first.total, summary.section_count);
    assert_eq!(first.next_offset, Some(10));

    let second = session
        .query_sections(
            &SectionRequestDto {
                snapshot_id,
                offset: 10,
                limit: Some(10),
                ..SectionRequestDto::default()
            },
            "op-second",
        )
        .expect("second page");
    assert_eq!(second.offset, 10);
    assert_eq!(second.next_offset, None, "the table ended");
    assert_eq!(first.rows[9].index + 1, second.rows[0].index);
}

#[test]
fn a_symbol_filter_and_sort_travel_as_typed_values() {
    let db = TempDb::new("symbol-query");
    let (session, snapshot_id, _) = session_with_history(&db);

    let page = session
        .query_symbols(
            &SymbolRequestDto {
                snapshot_id,
                filter: Some("mqtt".to_owned()),
                sort: SymbolSortDto::Size,
                direction: SortDirDto::Desc,
                ..SymbolRequestDto::default()
            },
            "op-symbols",
        )
        .expect("symbols");

    assert_eq!(page.total, 1);
    let symbol = &page.rows[0];
    assert_eq!(symbol.name.as_deref(), Some("mqtt_task"));
    assert_eq!(symbol.address.as_deref(), Some("0x08000001"));
    assert_eq!(symbol.size, Some(32), "bytes, unscaled");
    assert_eq!(symbol.kind, "Function");
    assert_eq!(symbol.binding, "Global");

    let wire = json(&page);
    assert_eq!(
        wire["rows"][0]["ordinal"].as_u64(),
        Some(symbol.ordinal as u64)
    );
    assert!(
        wire["filter"].is_null() && wire["sort"].is_null(),
        "the request shape is not part of the response"
    );
}

#[test]
fn the_largest_contributors_page_arrives_ordered_by_stored_payload() {
    // The top-contributors view is one bounded query with one sort, not a second code path that
    // could disagree with the table about what is largest.
    let db = TempDb::new("top-contributors");
    let (session, snapshot_id, _) = session_with_history(&db);

    let page = session
        .query_sections(
            &SectionRequestDto {
                snapshot_id,
                sort: SectionSortDto::FileSize,
                direction: SortDirDto::Desc,
                limit: Some(5),
                ..SectionRequestDto::default()
            },
            "op-top",
        )
        .expect("ordered page");

    let sizes: Vec<u64> = page.rows.iter().map(|row| row.file_size).collect();
    assert!(sizes.windows(2).all(|pair| pair[0] >= pair[1]), "{sizes:?}");
    assert_eq!(page.rows.len(), 5);
    assert_eq!(page.total, 17);
    // The largest stored payload in this fixture is the symbol table: host metadata, which is why
    // the view is labelled by what it measures rather than by what a device budget would charge.
    assert_eq!(page.rows[0].file_size, 528);
    assert_eq!(page.rows[0].name.as_deref(), Some(".symtab"));
}

#[test]
fn evidence_crosses_as_provenance_and_carries_no_host_path() {
    let db = TempDb::new("evidence-shape");
    let (session, snapshot_id, summary) = session_with_history(&db);

    let page = session
        .query_evidence(
            &EvidenceRequestDto {
                snapshot_id,
                classification: Some(EvidenceClassDto::Observed),
                sort: EvidenceSortDto::Classification,
                ..EvidenceRequestDto::default()
            },
            "op-evidence",
        )
        .expect("evidence");

    assert_eq!(page.total, summary.evidence_summary.observed);
    assert!(
        page.rows
            .iter()
            .all(|row| row.classification == "observed" && !row.source_locator.is_empty()),
        "every row must say where it came from"
    );
    let sha = page
        .rows
        .iter()
        .find(|row| row.field == "sha256")
        .expect("the hash is recorded evidence");
    assert_eq!(sha.raw_value, summary.artifact.sha256);

    // A user-selected artifact is analyzed from wherever it lives. Nothing that identifies where
    // may reach the WebView (`AGENTS.md` 7, `05_SECURITY_PRIVACY`).
    let text = serde_json::to_string(&page).expect("serializes");
    for forbidden in [
        repo_root().display().to_string(),
        "C:\\".to_owned(),
        "/home/".to_owned(),
        "/Users/".to_owned(),
    ] {
        assert!(
            !text.contains(&forbidden),
            "the evidence page must not locate a file: {forbidden} appears in it"
        );
    }
}

#[test]
fn an_unknown_snapshot_id_is_a_typed_error_rather_than_a_panic_or_an_empty_table() {
    let db = TempDb::new("not-found");
    let (session, _, _) = session_with_history(&db);
    let missing = "0".repeat(64);

    let error = session
        .query_sections(
            &SectionRequestDto {
                snapshot_id: missing.clone(),
                ..SectionRequestDto::default()
            },
            "op-missing",
        )
        .expect_err("a snapshot that was never stored has no page");

    assert_eq!(error.code, "ERR-STORAGE-4005");
    assert_eq!(error.operation_id, "op-missing");
    assert!(!error.message.is_empty());
    assert!(error.remediation.is_some(), "the user needs a next step");

    let symbol_error = session
        .query_symbols(
            &SymbolRequestDto {
                snapshot_id: missing.clone(),
                ..SymbolRequestDto::default()
            },
            "op-missing-symbols",
        )
        .expect_err("no symbols either");
    assert_eq!(symbol_error.code, "ERR-STORAGE-4005");

    let evidence_error = session
        .query_evidence(
            &EvidenceRequestDto {
                snapshot_id: missing,
                ..EvidenceRequestDto::default()
            },
            "op-missing-evidence",
        )
        .expect_err("no evidence either");
    assert_eq!(evidence_error.code, "ERR-STORAGE-4005");
}

#[test]
fn no_general_purpose_query_surface_was_added_to_the_shell() {
    // The three commands are the whole detail surface. A `run_sql`-shaped escape hatch would make
    // every bound in this file optional, so the absence is asserted rather than assumed.
    let sources = [
        "src/lib.rs",
        "src/details.rs",
        "src/intake.rs",
        "src/service.rs",
    ];
    let forbidden = [
        "run_sql",
        "read_table",
        "query_any",
        "get_database",
        "execute_shell",
    ];
    for file in sources {
        let text = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(file)
                .to_str()
                .expect("utf-8 path"),
        )
        .unwrap_or_else(|err| panic!("read {file}: {err}"));
        for name in forbidden {
            assert!(
                !text.contains(name),
                "{file} mentions {name}; a general-purpose command is not allowed"
            );
        }
    }

    let handler = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        .expect("read lib.rs");
    for command in ["query_sections", "query_symbols", "query_evidence"] {
        assert!(
            handler.contains(command),
            "{command} must be registered with the shell, not just implemented"
        );
    }
}
