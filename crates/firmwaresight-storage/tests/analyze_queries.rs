//! The bounded Analyze query layer: P1's inspect-Sections, inspect-Symbols and inspect-Evidence
//! paths, and the payload ceiling that keeps them bounded.
//!
//! `04_TECH/14` 3 forbids sending a whole symbol table across a boundary, and `AGENTS.md` 8 forbids
//! turning an undetermined value into a zero. Both rules are about what a *reader* gets, so they can
//! only be proven at the query layer that builds the page.
//!
//! Three things every test here pins down together:
//! - the page is bounded by a server-side default and a server-side hard maximum, whatever the
//!   caller asked for;
//! - a fact that was stored as unknown comes back unknown, with its reason, and never as `0`;
//! - the rows belong to the snapshot that was named, because every snapshot-scoped query resolves
//!   one snapshot id to one build before it reads.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_core::domain::evidence::EvidenceClass;
use firmwaresight_core::domain::identity::Fact;
use firmwaresight_storage::{
    DEFAULT_QUERY_LIMIT, Database, EvidenceQuery, EvidenceSort, MAX_QUERY_LIMIT, SectionQuery,
    SectionSort, SortDir, StorageError, SymbolQuery, SymbolSort,
};
use rusqlite::params;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

/// The ELF + GNU ld MAP fixture: 17 sections, 32 symbols, 11 evidence items.
fn dual_region_snapshot() -> BuildSnapshot {
    let root = repo_root();
    pipeline::analyze(
        &AnalysisRequest::new(root.join("fixtures/elf/p0-dual-region/firmware.elf"))
            .with_map(root.join("fixtures/elf/p0-dual-region/firmware.map")),
    )
    .expect("the committed fixture must analyze")
    .snapshot
    .clone()
}

/// The ELF-only fixture, which is a different build and therefore a different row set.
fn basic_snapshot() -> BuildSnapshot {
    let root = repo_root();
    pipeline::analyze(&AnalysisRequest::new(
        root.join("fixtures/elf/p0-basic/firmware.elf"),
    ))
    .expect("the second committed fixture must analyze")
    .snapshot
    .clone()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-analyze-queries-{label}-{}-{:x}.sqlite",
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

/// Add symbol rows so a table is larger than one page.
///
/// The committed fixtures hold tens of symbols, which cannot exercise a 100-row default or a
/// 500-row ceiling. These rows are test scaffolding for the boundary, not facts about an artifact:
/// they are attached to a real build id so the read path sees one build, and they are always named
/// `zz-boundary-*` so no filter test can mistake them for fixture content.
fn add_boundary_symbols(db: &Database, build_id: &str, count: i64) {
    let mut stmt = db
        .connection()
        .prepare(
            "INSERT INTO symbols
                 (build_id, ordinal, name, name_unknown, address, size, size_unknown, kind,
                  binding, section_ref)
             VALUES (?1, ?2, ?3, NULL, ?4, NULL, 'harness row carries no size', 'Function',
                     'Global', 'Index(1)')",
        )
        .expect("prepare");
    for index in 0..count {
        stmt.execute(params![
            build_id,
            1000 + index,
            format!("zz-boundary-{index:04}"),
            0x9000_0000i64 + index,
        ])
        .expect("insert boundary row");
    }
}

/// One harness evidence row in a class the committed fixtures never produce.
///
/// Like [`add_boundary_symbols`], this is scaffolding for a boundary rather than a fact about an
/// artifact: it is attached to a real build so the read path sees one build's evidence, and its
/// identifier is prefixed so no product assertion can pick it up.
fn add_boundary_evidence(db: &Database, build_id: &str, id: &str, classification: &str) {
    db.connection()
        .execute(
            "INSERT INTO evidence
                 (build_id, id, field, classification, source_type, source_locator, raw_value,
                  rule, confidence)
             VALUES (?1, ?2, 'harness.field', ?3, 'UserDeclaration', 'harness:classification-order',
                     'harness row', 'test-scaffold/1', NULL)",
            params![build_id, id, classification],
        )
        .expect("insert boundary evidence");
}

fn count_where(db: &Database, sql: &str, build_id: &str) -> i64 {
    db.connection()
        .query_row(sql, [build_id], |row| row.get(0))
        .expect("count")
}

fn section_page(
    db: &Database,
    snapshot: &BuildSnapshot,
    mutate: impl FnOnce(&mut SectionQuery),
) -> firmwaresight_storage::Page<firmwaresight_storage::SectionRow> {
    let mut query = SectionQuery {
        snapshot_id: snapshot.id().as_str().to_owned(),
        ..SectionQuery::default()
    };
    mutate(&mut query);
    db.query_sections(&query).expect("sections page")
}

fn symbol_page(
    db: &Database,
    snapshot: &BuildSnapshot,
    mutate: impl FnOnce(&mut SymbolQuery),
) -> firmwaresight_storage::Page<firmwaresight_storage::SymbolRow> {
    let mut query = SymbolQuery {
        snapshot_id: snapshot.id().as_str().to_owned(),
        ..SymbolQuery::default()
    };
    mutate(&mut query);
    db.query_symbols(&query).expect("symbols page")
}

fn evidence_page(
    db: &Database,
    snapshot: &BuildSnapshot,
    mutate: impl FnOnce(&mut EvidenceQuery),
) -> firmwaresight_storage::Page<firmwaresight_storage::EvidenceRow> {
    let mut query = EvidenceQuery {
        snapshot_id: snapshot.id().as_str().to_owned(),
        ..EvidenceQuery::default()
    };
    mutate(&mut query);
    db.query_evidence(&query).expect("evidence page")
}

#[test]
fn sections_come_back_in_index_order_and_a_whole_table_page_offers_no_next_offset() {
    let db_file = TempDb::new("sections-default");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = section_page(&db, &snapshot, |_| {});

    assert_eq!(page.total, snapshot.section_count() as i64);
    assert_eq!(page.rows.len(), snapshot.section_count());
    assert_eq!(page.offset, 0);
    assert_eq!(page.limit, DEFAULT_QUERY_LIMIT);
    assert_eq!(
        page.next_offset, None,
        "a page that already reached the end must not offer a cursor back to the same row count"
    );
    assert!(
        page.rows
            .windows(2)
            .all(|pair| pair[0].index < pair[1].index),
        "the default order is the section header index, which is the stable locator"
    );
}

#[test]
fn a_first_page_reports_the_next_offset_and_the_second_page_ends_the_table() {
    let db_file = TempDb::new("sections-pages");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");
    let total = snapshot.section_count() as i64;

    let first = section_page(&db, &snapshot, |query| query.limit = 10);
    assert_eq!(first.rows.len(), 10);
    assert_eq!(first.total, total, "total counts the table, not the page");
    assert_eq!(first.next_offset, Some(10));

    let second = section_page(&db, &snapshot, |query| {
        query.offset = 10;
        query.limit = 10;
    });
    assert_eq!(second.rows.len() as i64, total - 10);
    assert_eq!(second.offset, 10);
    assert_eq!(second.next_offset, None);

    let mut paged: Vec<i64> = first.rows.iter().map(|row| row.index).collect();
    paged.extend(second.rows.iter().map(|row| row.index));
    let whole: Vec<i64> = section_page(&db, &snapshot, |_| {})
        .rows
        .iter()
        .map(|row| row.index)
        .collect();
    assert_eq!(
        paged, whole,
        "paging must visit every row exactly once, in the same order as one page"
    );
}

#[test]
fn a_limit_above_the_hard_maximum_is_clamped_in_rust_and_cannot_carry_the_whole_table() {
    // The ceiling has to hold whatever the caller asks for: a WebView that requests 100k rows must
    // get one bounded page, not the symbol table.
    let db_file = TempDb::new("limit-clamp");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    let build_id = db
        .import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");
    let extra = 620;
    add_boundary_symbols(&db, &build_id, extra);
    let total = snapshot.symbol_count() as i64 + extra;

    let page = symbol_page(&db, &snapshot, |query| query.limit = 100_000);

    assert_eq!(
        page.limit, MAX_QUERY_LIMIT,
        "the requested limit is not the answer"
    );
    assert_eq!(page.rows.len() as i64, MAX_QUERY_LIMIT);
    assert_eq!(
        page.total, total,
        "the table is bigger than one page and stays that big"
    );
    assert_eq!(page.next_offset, Some(MAX_QUERY_LIMIT));
}

#[test]
fn a_zero_limit_falls_back_to_the_default_page_size() {
    let db_file = TempDb::new("limit-zero");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = symbol_page(&db, &snapshot, |query| query.limit = 0);

    assert_eq!(page.limit, DEFAULT_QUERY_LIMIT);
    assert_eq!(page.rows.len(), snapshot.symbol_count());
}

#[test]
fn a_negative_offset_is_clamped_to_the_first_page() {
    let db_file = TempDb::new("offset-negative");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = section_page(&db, &snapshot, |query| query.offset = -5);

    assert_eq!(
        page.offset, 0,
        "a negative cursor must not become an SQLite row offset"
    );
    let first = section_page(&db, &snapshot, |_| {});
    assert_eq!(
        page.rows, first.rows,
        "a clamped offset must read the same rows as the first page"
    );
}

#[test]
fn an_offset_past_the_end_is_an_empty_page_and_not_an_error() {
    let db_file = TempDb::new("offset-past");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = section_page(&db, &snapshot, |query| query.offset = 9_000);

    assert!(page.rows.is_empty());
    assert_eq!(page.total, snapshot.section_count() as i64);
    assert_eq!(page.next_offset, None);
}

#[test]
fn a_symbol_filter_matches_a_substring_in_sqlite_rather_than_a_full_name() {
    let db_file = TempDb::new("symbol-filter");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = symbol_page(&db, &snapshot, |query| {
        query.name_contains = Some("MQTT".to_owned());
    });

    assert_eq!(
        page.total, 1,
        "one symbol in this fixture carries that name"
    );
    assert_eq!(
        page.rows[0].name.value().map(String::as_str),
        Some("mqtt_task")
    );

    let broad = symbol_page(&db, &snapshot, |query| {
        query.name_contains = Some("main".to_owned());
    });
    let names: Vec<Option<&str>> = broad
        .rows
        .iter()
        .map(|row| row.name.value().map(String::as_str))
        .collect();
    assert_eq!(
        broad.total, 2,
        "`main` is a substring of `main` and of `main.c`"
    );
    assert!(names.contains(&Some("main")));
    assert!(names.contains(&Some("main.c")));
}

#[test]
fn a_filter_treats_like_wildcards_as_ordinary_characters() {
    // Unescaped, `%` would match every row and `_` would match the `.` in `.text`. A filter is user
    // text, so it has to be read as text.
    let db_file = TempDb::new("filter-wildcards");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let percent = section_page(&db, &snapshot, |query| {
        query.name_contains = Some("%".to_owned());
    });
    assert_eq!(percent.total, 0);
    assert!(percent.rows.is_empty());

    let underscore = section_page(&db, &snapshot, |query| {
        query.name_contains = Some("_text".to_owned());
    });
    assert_eq!(
        underscore.total, 0,
        "no section is literally named with `_text`; `_` must not match the dot of `.text`"
    );

    let literal = section_page(&db, &snapshot, |query| {
        query.name_contains = Some("text".to_owned());
    });
    assert_eq!(
        literal.total, 1,
        "the same filter without a wildcard does match"
    );
}

#[test]
fn sections_without_evidence_come_back_unknown_with_their_reason_instead_of_zero() {
    let db_file = TempDb::new("sections-unknown");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    let build_id = db
        .import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = section_page(&db, &snapshot, |_| {});
    let stored_nulls = count_where(
        &db,
        "SELECT COUNT(*) FROM sections WHERE build_id = ?1 AND load_addr IS NULL",
        &build_id,
    );
    assert!(
        stored_nulls > 0,
        "this fixture must still contain a section with no load evidence, or the test proves nothing"
    );

    let unknown: Vec<_> = page
        .rows
        .iter()
        .filter(|row| row.load_address.value().is_none())
        .collect();
    assert_eq!(
        unknown.len() as i64,
        stored_nulls,
        "an unrecorded load address must read back as unknown, never as a known zero"
    );
    for row in unknown {
        assert!(
            matches!(&row.load_address, Fact::Unknown { reason } if !reason.is_empty()),
            "the reason has to survive the round trip: {:?}",
            row.load_address
        );
    }
}

#[test]
fn symbol_sizes_sort_known_first_and_unknown_last_in_both_directions() {
    let db_file = TempDb::new("symbol-size-sort");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    let build_id = db
        .import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");
    let stored_unknown = count_where(
        &db,
        "SELECT COUNT(*) FROM symbols WHERE build_id = ?1 AND size IS NULL AND size_unknown IS NOT NULL",
        &build_id,
    );
    assert!(
        stored_unknown > 0,
        "ELF entries with no recorded size exist here"
    );

    for direction in [SortDir::Asc, SortDir::Desc] {
        let page = symbol_page(&db, &snapshot, |query| {
            query.sort_by = SymbolSort::Size;
            query.direction = direction;
        });
        assert_eq!(page.rows.len(), snapshot.symbol_count());

        let known: Vec<u64> = page
            .rows
            .iter()
            .filter_map(|row| row.size.value().copied())
            .collect();
        let unknown_block = page
            .rows
            .iter()
            .rev()
            .take_while(|row| row.size.value().is_none())
            .count();
        assert_eq!(
            unknown_block as i64, stored_unknown,
            "unknown sizes must sit at the tail in both directions, not stand in for a value"
        );

        let pairs_ok = match direction {
            SortDir::Asc => known.windows(2).all(|pair| pair[0] <= pair[1]),
            SortDir::Desc => known.windows(2).all(|pair| pair[0] >= pair[1]),
        };
        assert!(
            pairs_ok,
            "{direction:?} order must order the sizes that are known: {known:?}"
        );
    }
}

#[test]
fn symbol_names_sort_in_order_and_equal_names_keep_their_ordinal_order() {
    let db_file = TempDb::new("symbol-name-sort");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    for direction in [SortDir::Asc, SortDir::Desc] {
        let page = symbol_page(&db, &snapshot, |query| {
            query.sort_by = SymbolSort::Name;
            query.direction = direction;
        });

        let names: Vec<&str> = page
            .rows
            .iter()
            .filter_map(|row| row.name.value().map(String::as_str))
            .collect();
        let ordered = match direction {
            SortDir::Asc => names.windows(2).all(|pair| pair[0] <= pair[1]),
            SortDir::Desc => names.windows(2).all(|pair| pair[0] >= pair[1]),
        };
        assert!(ordered, "{direction:?} name order is broken: {names:?}");

        // A storage ordinal is a row position, not identity, but it is the only stable tiebreak:
        // without it two pages could disagree about which `$t` came first.
        for pair in page.rows.windows(2) {
            if pair[0].name.value() == pair[1].name.value() {
                assert!(
                    pair[0].ordinal < pair[1].ordinal,
                    "tied names must fall back to ascending ordinal in both directions"
                );
            }
        }

        let unknown_names = page
            .rows
            .iter()
            .filter(|row| row.name.value().is_none())
            .count();
        let tail_unknown = page
            .rows
            .iter()
            .rev()
            .take_while(|row| row.name.value().is_none())
            .count();
        assert_eq!(
            tail_unknown, unknown_names,
            "an unnamed symbol has no name to sort, so it belongs at the tail in both directions"
        );
    }
}

#[test]
fn symbols_sort_by_address_in_both_directions_over_the_whole_table() {
    let db_file = TempDb::new("symbol-address-sort");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let ascending = symbol_page(&db, &snapshot, |query| query.sort_by = SymbolSort::Address);
    let addresses: Vec<Option<u64>> = ascending.rows.iter().map(|row| row.address).collect();
    assert!(
        addresses.iter().all(Option::is_some),
        "the ELF parser records an address for every symbol in this fixture"
    );
    assert!(addresses.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(ascending.total, snapshot.symbol_count() as i64);

    let descending = symbol_page(&db, &snapshot, |query| {
        query.sort_by = SymbolSort::Address;
        query.direction = SortDir::Desc;
    });
    let reversed: Vec<Option<u64>> = descending.rows.iter().map(|row| row.address).collect();
    assert!(reversed.windows(2).all(|pair| pair[0] >= pair[1]));

    let mut sorted_asc = addresses.clone();
    sorted_asc.sort_unstable();
    let mut sorted_desc = reversed.clone();
    sorted_desc.sort_unstable();
    assert_eq!(
        sorted_asc, sorted_desc,
        "reversing the direction must visit the same rows, not a different set"
    );

    // Several symbols share an address in this fixture, so the tiebreak is what makes a page
    // reproducible: without it two reads of the same order can disagree.
    for (name, rows) in [
        ("ascending", &ascending.rows),
        ("descending", &descending.rows),
    ] {
        for pair in rows.windows(2) {
            if pair[0].address == pair[1].address {
                assert!(
                    pair[0].ordinal < pair[1].ordinal,
                    "{name} ties on an address must fall back to ascending ordinal"
                );
            }
        }
    }
}

#[test]
fn the_largest_section_by_file_size_leads_a_descending_page() {
    // This page is what the top-contributors view reads, so its first row has to be the largest
    // stored payload rather than the first row of the table.
    let db_file = TempDb::new("sections-size-desc");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = section_page(&db, &snapshot, |query| {
        query.sort_by = SectionSort::FileSize;
        query.direction = SortDir::Desc;
        query.limit = 5;
    });

    let sizes: Vec<u64> = page.rows.iter().map(|row| row.file_size).collect();
    assert!(sizes.windows(2).all(|pair| pair[0] >= pair[1]), "{sizes:?}");
    assert_eq!(page.offset, 0);
    assert_eq!(page.next_offset, Some(5));
    assert_eq!(
        page.rows[0].name.value().map(String::as_str),
        Some(".symtab"),
        "528 stored bytes is the largest payload here, and it is host metadata: a largest-stored-\
         payload view must never be labelled as a device-budget view"
    );
}

#[test]
fn evidence_rows_carry_their_provenance_for_the_inspector() {
    let db_file = TempDb::new("evidence-rows");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    db.import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let page = evidence_page(&db, &snapshot, |_| {});
    assert_eq!(page.total, snapshot.evidence().len() as i64);

    let sha = page
        .rows
        .iter()
        .find(|row| row.id == "ev-sha256")
        .expect("the artifact hash is recorded evidence");
    assert_eq!(sha.field, "sha256");
    assert_eq!(sha.classification, "observed");
    assert_eq!(
        sha.raw_value,
        snapshot.primary_artifact().expect("artifact").sha256.hex()
    );
    assert!(
        !sha.source_locator.is_empty(),
        "a reader must be able to re-check the claim"
    );
    assert!(!sha.rule.is_empty());
    assert!(!sha.source_type.is_empty());
}

#[test]
fn evidence_can_be_filtered_to_one_class_and_the_classes_add_up() {
    let db_file = TempDb::new("evidence-classes");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    let build_id = db
        .import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    let mut summed = 0;
    for (class, label) in [
        (EvidenceClass::Observed, "observed"),
        (EvidenceClass::Derived, "derived"),
        (EvidenceClass::Declared, "declared"),
        (EvidenceClass::Unknown, "unknown"),
    ] {
        let page = evidence_page(&db, &snapshot, |query| query.classification = Some(class));
        assert!(
            page.rows.iter().all(|row| row.classification == label),
            "a class filter must not return another class"
        );
        let stored = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM evidence WHERE build_id = ?1 AND classification = ?2",
                params![build_id, label],
                |row| row.get::<_, i64>(0),
            )
            .expect("count");
        assert_eq!(
            page.total, stored,
            "the filter total is the stored count for that class"
        );
        summed += page.total;
    }

    let all = evidence_page(&db, &snapshot, |_| {});
    assert_eq!(
        summed, all.total,
        "the four classes must partition the recorded evidence, with nothing lost between them"
    );
}

#[test]
fn evidence_sorted_by_classification_leads_with_observed_and_ends_with_unknown() {
    // The committed fixtures record observed and derived facts only: nothing in them is declared,
    // and every fact they can determine is determined. A four-way ordering cannot be proven from
    // two populated classes, so one harness row per missing class is attached to a real build.
    let db_file = TempDb::new("evidence-class-sort");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = basic_snapshot();
    let build_id = db
        .import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");
    add_boundary_evidence(&db, &build_id, "ev-harness-declared", "declared");
    add_boundary_evidence(&db, &build_id, "ev-harness-unknown", "unknown");

    let page = evidence_page(&db, &snapshot, |query| {
        query.sort_by = EvidenceSort::Classification;
    });
    assert_eq!(page.total, snapshot.evidence().len() as i64 + 2);

    let rank = |class: &str| match class {
        "observed" => 0,
        "derived" => 1,
        "declared" => 2,
        "unknown" => 3,
        other => panic!("unexpected classification {other}"),
    };
    let ranks: Vec<u8> = page
        .rows
        .iter()
        .map(|row| rank(&row.classification))
        .collect();
    assert!(ranks.windows(2).all(|pair| pair[0] <= pair[1]), "{ranks:?}");
    assert_eq!(*ranks.first().expect("a page"), 0, "observed leads");
    assert_eq!(*ranks.last().expect("a page"), 3, "unknown last");

    // `ranks` is already proven non-decreasing above, so this is the set of classes in order.
    let mut distinct = ranks.clone();
    distinct.dedup();
    assert_eq!(
        distinct,
        vec![0, 1, 2, 3],
        "every class has a rank, so a reader can order evidence quality without knowing the labels"
    );
}

#[test]
fn a_snapshot_that_was_never_stored_is_reported_as_not_found_by_every_query() {
    let db_file = TempDb::new("not-found");
    let db = Database::open(db_file.path()).expect("open");

    let missing = "0".repeat(64);
    let errors = [
        db.query_sections(&SectionQuery {
            snapshot_id: missing.clone(),
            ..SectionQuery::default()
        })
        .err(),
        db.query_symbols(&SymbolQuery {
            snapshot_id: missing.clone(),
            ..SymbolQuery::default()
        })
        .err(),
        db.query_evidence(&EvidenceQuery {
            snapshot_id: missing,
            ..EvidenceQuery::default()
        })
        .err(),
    ];

    for error in errors {
        let error = error.expect("an unknown snapshot id must not read as an empty table");
        assert!(
            matches!(error, StorageError::NotFound { .. }),
            "expected NotFound, got {error:?}"
        );
        assert_eq!(error.stable_code(), "ERR-STORAGE-4005");
    }
}

#[test]
fn a_snapshot_scoped_query_never_returns_another_builds_rows() {
    // Two builds in one database is the case that shipped broken once: evidence identifiers are
    // named within an analysis, so a query keyed by anything other than the build would mix them.
    let db_file = TempDb::new("two-builds");
    let mut db = Database::open(db_file.path()).expect("open");
    let dual = dual_region_snapshot();
    let basic = basic_snapshot();
    db.import_snapshot("proj-1", "P1 project", &dual)
        .expect("import");
    db.import_snapshot("proj-1", "P1 project", &basic)
        .expect("import");

    assert_eq!(
        section_page(&db, &dual, |_| {}).total,
        dual.section_count() as i64
    );
    assert_eq!(
        section_page(&db, &basic, |_| {}).total,
        basic.section_count() as i64
    );
    assert_eq!(
        symbol_page(&db, &dual, |_| {}).total,
        dual.symbol_count() as i64
    );
    assert_eq!(
        evidence_page(&db, &basic, |_| {}).total,
        basic.evidence().len() as i64
    );

    // Both fixtures contain `.data`, and only one of them was analyzed with a MAP, so the region of
    // one section is a fact a mixed-up read would get wrong in a way the summary counts cannot show.
    let dual_sections = section_page(&db, &dual, |_| {});
    let basic_sections = section_page(&db, &basic, |_| {});
    let dual_data = dual_sections
        .rows
        .iter()
        .find(|row| row.name.value().map(String::as_str) == Some(".data"))
        .expect("the MAP-backed fixture carries a .data section");
    let basic_data = basic_sections
        .rows
        .iter()
        .find(|row| row.name.value().map(String::as_str) == Some(".data"))
        .expect("the ELF-only fixture carries a .data section");
    assert_eq!(
        dual_data.region.value().map(String::as_str),
        Some("RAM"),
        "the MAP-backed build keeps its own region fact"
    );
    assert!(
        basic_data.region.value().is_none(),
        "an ELF-only build has no region evidence, and must not borrow the other build's: {:?}",
        basic_data.region
    );
}

#[test]
fn a_snapshot_id_always_resolves_to_the_same_build_row() {
    // A build id is derived from the snapshot id, so two rows for one snapshot cannot be created
    // through the application. The rule asserted here is only that the choice is stable: without an
    // ordering, `LIMIT 1` returns whichever row SQLite visits first, and the same click could then
    // read a different build on a different day.
    let db_file = TempDb::new("deterministic-lookup");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = dual_region_snapshot();
    let real_build = db
        .import_snapshot("proj-1", "P1 project", &snapshot)
        .expect("import");

    db.connection()
        .execute(
            "INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
             VALUES ('a-second-row', 'proj-1', ?1, 'p0-normalize-1', '0.1.0', 'COMPLETE')",
            params![snapshot.id().as_str()],
        )
        .expect("insert a second row for the same snapshot id");

    for _ in 0..3 {
        let found = db
            .build_id_for_snapshot(snapshot.id().as_str())
            .expect("lookup");
        assert_eq!(found.as_deref(), Some("a-second-row"));
        assert_ne!(
            found.as_deref(),
            Some(real_build.as_str()),
            "the answer must not depend on visit order"
        );
    }
}
