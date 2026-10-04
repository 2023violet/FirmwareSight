//! P5 Commit D: the store can say whether it is well, and an upgrade is preceded by a snapshot of
//! what it was.
//!
//! Both halves answer the same product question from two directions. Health: `PRAGMA integrity_check`
//! appeared nowhere in this repository before this file, while `error.rs` has since P3 told a user to
//! "restore a backup" that the product could not make. Backup: `0005` shipped on the sequencing rule
//! recorded in `P5_MIGRATION_DECISION.md` — a schema-5 store cannot be opened by a schema-4 binary, so
//! the only recovery path from an upgrade is a copy taken before it.
//!
//! Both corruption fixtures are measured, not assumed, because the difference between "damaged" and
//! "damaged enough that nothing can open it" is the whole test design. Probed on this crate's own v5
//! store: blasting every page from about page 10 onward (schema pages, root pages and freelist
//! included) makes `Database::open` itself fail with `migration 0 failed`, which is the open path and
//! not the health path. Restricting the damage to b-tree **leaf** pages in the back half leaves the
//! file openable and `PRAGMA integrity_check` still answers — with 73 problem lines before the
//! iteration itself errors out. The single-last-page fixture is the mild case; the deep-leaf fixture
//! is the one that proves the report bound is real rather than convenient. That a failing iteration
//! arrives *after* problems have been named is why `integrity_check()` treats it as evidence of ill
//! health rather than as its own error.
//!
//! `every_older_file_backed_schema_is_snapshotted_at_the_version_it_was_found` is prompt §32's matrix
//! run against the backup rule rather than around it: v1, v2, v3 and v4 file-backed stores each owe a
//! snapshot named for the version they were *found* at. Fresh → v5 and v5 → v5 reopen are the two arms
//! that owe nothing, and they are the two tests next to it.

use std::path::{Path, PathBuf};

use firmwaresight_storage::{
    Database, JournalMode, SCHEMA_VERSION, StoreHealth, pre_migration_backup_path,
};

// ---- fixtures -----------------------------------------------------------------------------------

/// A real directory, so a backup written beside the store is cleaned up with it. The per-file `TempDb`
/// helpers elsewhere in this crate delete only `""`/`-wal`/`-shm`, which would leave every backup this
/// file creates behind.
struct Store {
    dir: PathBuf,
    file: PathBuf,
}

impl Store {
    fn new(label: &str) -> Self {
        let mut dir = std::env::temp_dir();
        let unique = format!(
            "fwsight-commit-d-{label}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        );
        dir.push(unique);
        std::fs::create_dir_all(&dir).expect("test directory");
        let file = dir.join("firmwaresight-p0.sqlite");
        Self { dir, file }
    }

    fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self) -> &Path {
        &self.file
    }

    /// The name the product would use for the snapshot of a `from` → `to` upgrade of this store.
    fn backup_for(&self, from: i64) -> PathBuf {
        pre_migration_backup_path(self.path(), from, SCHEMA_VERSION)
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The schema a `through`-version build left behind: the migrations up to and including it, and the
/// bookkeeping rows that build would have written. Data is added by whichever fixture needs it,
/// because which tables hold rows is part of what is being claimed.
fn apply_migrations_through(store: &Store, through: i64) {
    const NAMES: [&str; 4] = [
        "0001_initial",
        "0002_evidence_keyed_by_build",
        "0003_gate_history",
        "0004_release_records",
    ];
    let conn = rusqlite::Connection::open(store.path()).expect("create the file");
    conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
        .expect("0001");
    if through >= 2 {
        conn.execute_batch(include_str!(
            "../migrations/0002_evidence_keyed_by_build.sql"
        ))
        .expect("0002");
    }
    if through >= 3 {
        conn.execute_batch(include_str!("../migrations/0003_gate_history.sql"))
            .expect("0003");
    }
    if through >= 4 {
        conn.execute_batch(include_str!("../migrations/0004_release_records.sql"))
            .expect("0004");
    }
    let mut bookkeeping = String::from(
        "CREATE TABLE schema_migrations (
             version    INTEGER PRIMARY KEY NOT NULL,
             name       TEXT NOT NULL,
             applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         );",
    );
    for (index, name) in NAMES.iter().take(through as usize).enumerate() {
        bookkeeping.push_str(&format!(
            "INSERT INTO schema_migrations (version, name) VALUES ({}, '{name}');",
            index + 1
        ));
    }
    conn.execute_batch(&bookkeeping)
        .expect("bookkeeping rows the build would have written");
    drop(conn);
}

/// Representative rows, in the shape `0004_release_records.sql` left them: `sections` has no
/// `file_offset_unknown`, `symbols` has no `address_unknown`, and no `idx_builds_created`.
fn build_a_v4_store(store: &Store) {
    apply_migrations_through(store, 4);
    let conn = rusqlite::Connection::open(store.path()).expect("reopen the v4 file");
    conn.execute_batch(
        "INSERT INTO projects (id, name) VALUES ('proj-d', 'Commit D project');
         INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                             created_by_fwsight, state)
             VALUES ('build-d', 'proj-d', 'snap-d', 'p0-normalize-1', '0.6.0', 'COMPLETE');
         INSERT INTO artifacts (id, build_id, path, kind, sha256, byte_size, parser_id,
                                architecture, bitness, endianness)
             VALUES ('art-d', 'build-d',
                     '/home/example/private/secret-project/firmware.elf', 'ELF',
                     '000000000000000000000000000000000000000000000000000000000000000d',
                     4096, 'elf-read', 'arm', '32', 'little');
         INSERT INTO sections (build_id, section_index, name, role, is_alloc, is_write,
                               is_execute, file_offset, file_size)
             VALUES ('build-d', 1, '.text', 'code', 1, 0, 1, 256, 1024),
                    ('build-d', 7, '.bss', 'nobits', 1, 1, 0, NULL, 0);
         INSERT INTO symbols (build_id, ordinal, kind, binding, section_ref, name, address, size)
             VALUES ('build-d', 1, 'OBJECT', 'GLOBAL', 1, 'top_secret_symbol_name', 2048, 16),
                    ('build-d', 2, 'FUNC', 'LOCAL', 1, 'decode_frame', 512, 40);
         INSERT INTO evidence (build_id, id, field, classification, source_type, source_locator,
                               raw_value, rule)
             VALUES ('build-d', 'ev-1', 'file_size', 'observed', 'elf-section',
                     'section:1', '1024', 'read'),
                    ('build-d', 'ev-2', 'file_offset', 'unknown', 'elf-section',
                     'section:7', 'no offset recorded', 'absent-is-unknown');
         INSERT INTO memory_footprints (build_id, layout_source, weakest_basis,
                                        admissible_hard_block, nonvolatile_state, nonvolatile_bytes,
                                        nonvolatile_class, runtime_state, runtime_bytes,
                                        runtime_class, excluded_metadata)
             VALUES ('build-d', 'map-memory-configuration', NULL, 1, 'exact', 1024, 'observed',
                     'exact', 16, 'observed', 0);",
    )
    .expect("v4 database with representative rows");
    // `release_records` and the four Gate tables exist in this file and hold nothing: a release row
    // names a `gate_runs` row by foreign key, and inventing a verdict to fill it would make the
    // fixture claim a Gate history it never ran.
    drop(conn);
}

/// The last page's cell-count field, set to a number the page cannot honour. The store still opens;
/// its b-trees no longer agree with themselves.
fn corrupt_the_last_page(store: &Store) -> Vec<u8> {
    let mut bytes = std::fs::read(store.path()).expect("read the store");
    let page = 4096usize;
    let last_page = bytes.len() / page - 1;
    let header = page * last_page;
    bytes[header + 3] = 0xFF;
    bytes[header + 4] = 0xFF;
    std::fs::write(store.path(), &bytes).expect("write the corruption");
    bytes
}

/// Every table-leaf (0x0d) and index-leaf (0x0a) page in the back half of the file, blasted the same
/// way. The point is that SQLite can name thousands of problems, so the report has to be cut.
///
/// The depth and the page types are both measured, not assumed. The probe this fixture rests on ran
/// on a 111-page store: blasting **every** page from number 10 upward makes the store unopenable
/// (`Database::open` answers `migration 0 failed`, because the engine cannot even read its own schema),
/// which is the open path, not the health path. Restricting the blast to leaf pages past the halfway
/// mark leaves page 1, the schema pages, the root pages and the freelist alone, so the product still
/// opens the file and `PRAGMA integrity_check` still answers — with 73 problem lines measured.
fn corrupt_deep_leaf_pages(store: &Store) -> usize {
    let mut bytes = std::fs::read(store.path()).expect("read the store");
    let page = 4096usize;
    let pages = bytes.len() / page;
    let mut blasted = 0;
    for index in pages / 2..pages {
        let header = page * index;
        if matches!(bytes[header], 0x0d | 0x0a) {
            bytes[header + 3] = 0xFF;
            bytes[header + 4] = 0xFF;
            blasted += 1;
        }
    }
    std::fs::write(store.path(), &bytes).expect("write the corruption");
    blasted
}

/// How many problems the engine names when nobody caps it. The uncapped reader is the non-vacuity
/// control for the bound: a `summary` of three problems only proves something if the store could
/// actually name more than three.
fn uncapped_problem_lines(store: &Store) -> usize {
    let conn = rusqlite::Connection::open(store.path()).expect("open for the uncapped read");
    let mut statement = conn
        .prepare("PRAGMA integrity_check")
        .expect("the uncapped check can be prepared");
    let mut mapped = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("the uncapped check can start");
    let mut lines = 0;
    // Stopping at the first error is the point: the engine names what it can and then refuses to
    // continue, and whatever it named before stopping is what it named.
    while let Some(Ok(value)) = mapped.next() {
        // One row can carry several newline-separated problems, and the first row is the
        // "*** in database main ***" banner rather than a problem.
        for line in value.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed != "ok" && !trimmed.starts_with("*** in database") {
                lines += 1;
            }
        }
    }
    lines
}

fn count(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap_or(-1)
}

fn schema_version_at(path: &Path) -> i64 {
    let conn = rusqlite::Connection::open(path).expect("open for the version read");
    conn.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
        row.get(0)
    })
    .unwrap_or(0)
}

fn has_column(path: &Path, table: &str, column: &str) -> bool {
    let conn = rusqlite::Connection::open(path).expect("open for the column read");
    let mut names = conn
        .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
        .expect("prepare")
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query")
        .collect::<Vec<_>>();
    names.retain(Result::is_ok);
    names.into_iter().flatten().any(|name| name == *column)
}

/// The seven tables a recovery snapshot has to carry, counted before the upgrade so the snapshot can
/// be compared with what the store actually held.
const COUNTED_TABLES: [&str; 7] = [
    "projects",
    "builds",
    "artifacts",
    "sections",
    "symbols",
    "evidence",
    "memory_footprints",
];

fn counts(store: &Store) -> Vec<(&'static str, i64)> {
    let conn = rusqlite::Connection::open(store.path()).expect("open for counting");
    COUNTED_TABLES
        .iter()
        .map(|table| (*table, count(&conn, table)))
        .collect()
}

fn backups_in(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read the test directory") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .expect("file name")
            .to_string_lossy()
            .to_string();
        if name.contains("pre-migration") {
            found.push(name);
        }
    }
    found.sort();
    found
}

// ---- A. health: a well store answers plainly ----------------------------------------------------

#[test]
fn a_store_this_build_migrated_is_reported_healthy() {
    let store = Store::new("healthy");
    let db = Database::open(store.path()).expect("a fresh store opens");

    let health = db.integrity_check().expect("the health check itself runs");

    assert!(
        matches!(health, StoreHealth::Healthy),
        "a store this build created is not the failure case: {health:?}"
    );
}

#[test]
fn an_opened_store_with_rows_is_healthy_and_the_check_writes_nothing() {
    let store = Store::new("healthy-seeded");
    build_a_v4_store(&store);
    let db = Database::open(store.path()).expect("the v4 store upgrades");

    let before = std::fs::read(store.path()).expect("read the store");
    let health = db.integrity_check().expect("the health check itself runs");
    let after = std::fs::read(store.path()).expect("read the store again");

    assert!(matches!(health, StoreHealth::Healthy), "{health:?}");
    assert_eq!(
        before, after,
        "a health check is read-only: it repaired nothing because it changed nothing"
    );
}

// ---- B. health: an unwell store is named, bounded, and left alone -------------------------------

/// Rows in the seven tables a recovery snapshot has to carry, written through the opened store so the
/// fixture never has to guess which columns a schema version added.
///
/// The symbol count is not decoration: the file has to span enough pages that "the back half" is a
/// range of data leaves rather than the schema, and 4 000 rows of `symbols` is what the probe measured
/// a 111-page store at. Inserted in bulk because a page-spanning loop of single-row `INSERT`s would be
/// a page-spanning loop of commits.
fn seed_rows(store: &Store) {
    let db = Database::open(store.path()).expect("open to seed");
    db.connection()
        .execute_batch(
            "INSERT INTO projects (id, name) VALUES ('proj-d', 'Commit D project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                 VALUES ('build-d', 'proj-d', 'snap-d', 'p0-normalize-1', '0.6.0', 'COMPLETE');
             INSERT INTO sections (build_id, section_index, name, role, is_alloc, is_write,
                                   is_execute, file_offset, file_size)
                 VALUES ('build-d', 1, '.text', 'code', 1, 0, 1, 256, 1024);
             INSERT INTO evidence (build_id, id, field, classification, source_type, source_locator,
                                   raw_value, rule)
                 VALUES ('build-d', 'ev-1', 'file_size', 'observed', 'elf-section',
                         'section:1', '1024', 'read');",
        )
        .expect("seed");
    const SYMBOLS: usize = 4_000;
    for start in (1..=SYMBOLS).step_by(500) {
        let end = (start + 499).min(SYMBOLS);
        let mut sql = String::from(
            "INSERT INTO symbols (build_id, ordinal, kind, binding, section_ref, name, address, size) \
             VALUES ",
        );
        for ordinal in start..=end {
            sql.push_str(&format!(
                "('build-d', {ordinal}, 'OBJECT', 'GLOBAL', 1, 'filler_symbol_{ordinal}', {ordinal}, \
                 8), "
            ));
        }
        sql.truncate(sql.len() - 2);
        sql.push(';');
        db.connection().execute_batch(&sql).expect("bulk symbols");
    }
    drop(db);
}

#[test]
fn a_store_whose_pages_disagree_with_themselves_is_reported_unhealthy_and_left_alone() {
    let store = Store::new("corrupt-one-page");
    seed_rows(&store);
    let corrupted = corrupt_the_last_page(&store);

    // The fixture is only testing the health path if the product can still open the file. Blasting a
    // page that holds schema makes the open itself fail, which is a different story; the probe this
    // file rests on found the last data page does the job.
    let reopened = Database::open(store.path());
    assert!(
        reopened.is_ok(),
        "the fixture must leave a store the product can still open, or it tests the open path \
         rather than the health path: {:?}",
        reopened.err().map(|err| err.to_string())
    );
    let db = reopened.expect("opened");

    let health = db.integrity_check().expect("the health check itself runs");
    let StoreHealth::Unhealthy { summary } = &health else {
        panic!("a store whose last page lies about its cells is not Healthy: {health:?}");
    };

    assert!(
        !summary.trim().is_empty(),
        "an unhealthy result has to say something"
    );
    assert!(
        !summary.contains('\\') && !summary.contains('/'),
        "the summary names pages and indexes, never a path: {summary}"
    );
    assert!(
        !summary.contains(&store.dir().to_string_lossy().to_string()),
        "the store's own directory must not travel inside the summary: {summary}"
    );
    assert_eq!(
        corrupted,
        std::fs::read(store.path()).expect("read the store again"),
        "checking health must not repair, truncate or rewrite a damaged store"
    );
    assert!(
        backups_in(store.dir()).is_empty(),
        "a health check is not a migration and writes no backup"
    );
}

#[test]
fn a_store_that_names_many_problems_is_still_bounded() {
    let store = Store::new("corrupt-many-pages");
    seed_rows(&store);
    // Corrupt before reopening, so the engine reads the damaged pages instead of its own cache.
    let blasted = corrupt_deep_leaf_pages(&store);
    assert!(
        blasted >= 5,
        "the fixture has to damage several data pages, and it damaged {blasted}"
    );

    // The control: uncapped, this store names dozens of problems. Without this line the bound below
    // could be satisfied by a store that never had more to say than the summary carried.
    let named = uncapped_problem_lines(&store);
    assert!(
        named > 10,
        "the uncapped report on {blasted} damaged leaf pages should name many problems, and it \
         named {named}"
    );

    let db = Database::open(store.path()).expect("a store with damaged data pages still opens");

    let health = db.integrity_check().expect("the health check itself runs");
    let StoreHealth::Unhealthy { summary } = &health else {
        panic!("many blasted pages cannot read as Healthy: {health:?}");
    };

    let segments = summary.split("; ").count();
    assert!(
        segments <= 4,
        "the summary carries at most the first few problems plus the truncation notice, and it \
         carried {segments}: {summary}"
    );
    assert!(
        summary.chars().count() <= 600,
        "an unbounded SQLite report must not cross an IPC boundary: {} chars",
        summary.chars().count()
    );
    assert!(
        summary.contains("only the first problems are listed"),
        "a cut list has to say it was cut, or {named} problems would read as {segments}: {summary}"
    );
    assert!(
        !summary.contains('\\') && !summary.contains('/'),
        "the summary names pages and indexes, never a path: {summary}"
    );
    assert!(
        !summary.contains(&store.dir().to_string_lossy().to_string()),
        "the store's own directory must not travel inside the summary: {summary}"
    );
}

/// A v4 store built at a path that already exists, for the cases that corrupt after opening.
fn build_a_v4_store_at(store: &Store) {
    let _ = std::fs::remove_file(store.path());
    build_a_v4_store(store);
}

// ---- C. backup: taken before the upgrade reaches the store --------------------------------------

#[test]
fn a_v4_store_is_snapshotted_before_the_upgrade_reaches_it() {
    let store = Store::new("v4-backup");
    build_a_v4_store(&store);
    let before = counts(&store);
    assert!(
        before.iter().all(|(_, rows)| *rows > 0),
        "the fixture has to hold rows in every counted table or the comparison proves nothing: \
         {before:?}"
    );
    assert_eq!(schema_version_at(store.path()), 4, "the store starts at v4");
    assert!(
        !has_column(store.path(), "sections", "file_offset_unknown"),
        "0005's column must be absent before the upgrade, or the ordering claim is empty"
    );

    let db = Database::open(store.path()).expect("the v5 build opens the v4 store");

    // 1 and 2: the snapshot exists, and it opens as valid SQLite.
    let backup = store.backup_for(4);
    assert!(
        backup.exists(),
        "the pre-migration snapshot is missing at {backup:?}"
    );
    let snapshot = rusqlite::Connection::open(&backup).expect("the snapshot is a database");

    // 3: it is the schema the store held, not the schema it became. This is the ordering proof — a
    // snapshot taken after the ALTERs would carry the column this asserts absent.
    assert_eq!(
        schema_version_at(&backup),
        4,
        "the snapshot is the before state"
    );
    assert!(
        !has_column(&backup, "sections", "file_offset_unknown"),
        "a snapshot taken after migration would carry 0005's column"
    );

    // 4: it holds the rows the store held.
    for (table, rows) in &before {
        let in_snapshot = count(&snapshot, table);
        assert_eq!(in_snapshot, *rows, "the snapshot lost rows from {table}");
    }

    // 5 and 6: the live store upgraded and kept its rows.
    assert_eq!(schema_version_at(store.path()), SCHEMA_VERSION);
    assert!(has_column(store.path(), "sections", "file_offset_unknown"));
    let live = counts(&store);
    assert_eq!(live, before, "the upgrade rewrote no history");

    // 7: both copies are well.
    let live_health = db.integrity_check().expect("the live check runs");
    assert!(
        matches!(live_health, StoreHealth::Healthy),
        "{live_health:?}"
    );
    let snapshot_health = snapshot
        .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
        .expect("the snapshot answers");
    assert_eq!(snapshot_health, "ok");

    // 9 and the privacy boundary that is *not* part of a backup: the snapshot keeps the source path
    // the store recorded, because a recovery copy that silently edited stored facts would be a
    // different database than the one it is meant to restore.
    let path_in_snapshot: String = snapshot
        .query_row("SELECT path FROM artifacts WHERE id = 'art-d'", [], |row| {
            row.get(0)
        })
        .expect("the artifact row reads back");
    assert_eq!(
        path_in_snapshot, "/home/example/private/secret-project/firmware.elf",
        "the backup is a local recovery copy, not a redacted export"
    );
}

/// §32's matrix, run against the backup rule instead of around it: every file-backed schema this
/// build can pick up is snapshotted at the version it was *found* at, not at the one it becomes.
///
/// The other two rows of the matrix are separate tests rather than loop arms, because their claim is
/// the opposite one: `a_fresh_store_writes_no_migration_backup` (fresh → v5 owes nothing) and
/// `a_store_already_at_this_version_backs_nothing_up_on_reopen` (v5 reopen owes nothing). In-memory
/// stores are not in this matrix at all — §32 does not require a backup of a database that has no
/// file, and `backup.rs`'s own test asserts that arm directly.
#[test]
fn every_older_file_backed_schema_is_snapshotted_at_the_version_it_was_found() {
    for found in 1..=4 {
        let store = Store::new(&format!("matrix-v{found}"));
        apply_migrations_through(&store, found);
        let conn = rusqlite::Connection::open(store.path()).expect("reopen to seed");
        conn.execute_batch(
            "INSERT INTO projects (id, name) VALUES ('proj-m', 'Matrix project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                 VALUES ('build-m', 'proj-m', 'snap-m', 'p0-normalize-1', '0.6.0', 'COMPLETE');",
        )
        .expect("one build");
        drop(conn);
        assert!(
            backups_in(store.dir()).is_empty(),
            "nothing has upgraded this store yet: {:?}",
            backups_in(store.dir())
        );

        let db = Database::open(store.path()).unwrap_or_else(|err| {
            panic!("a v{found} store must upgrade to v{SCHEMA_VERSION}: {err}")
        });

        // The snapshot is named for the transition it precedes, opens, is sound, and is the old schema.
        let backup = pre_migration_backup_path(store.path(), found, SCHEMA_VERSION);
        assert!(
            backup.exists(),
            "a v{found} store owes a v{found} → v{SCHEMA_VERSION} snapshot"
        );
        assert_eq!(
            schema_version_at(&backup),
            found,
            "the snapshot records the schema the store was found at"
        );
        {
            let snapshot = rusqlite::Connection::open(&backup).expect("the snapshot opens");
            let soundness: String = snapshot
                .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                .expect("the snapshot answers");
            assert_eq!(soundness, "ok", "the snapshot is a sound copy at v{found}");
            let in_snapshot: i64 = snapshot
                .query_row(
                    "SELECT COUNT(*) FROM builds WHERE id = 'build-m'",
                    [],
                    |row| row.get(0),
                )
                .expect("the snapshot reads");
            assert_eq!(
                in_snapshot, 1,
                "the snapshot holds the build the store held"
            );
        }

        // The live store arrived at this build's schema with its row intact and its health intact.
        assert_eq!(
            schema_version_at(store.path()),
            SCHEMA_VERSION,
            "the v{found} store reached v{SCHEMA_VERSION}"
        );
        let live: i64 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM builds WHERE id = 'build-m'",
                [],
                |row| row.get(0),
            )
            .expect("the live store reads");
        assert_eq!(live, 1, "the upgrade kept the build it found");
        let health = db.integrity_check().expect("the check runs");
        assert!(
            matches!(health, StoreHealth::Healthy),
            "a store this build migrated and snapshotted is healthy: {health:?}"
        );

        // Exactly one snapshot file, and no staging debris beside it. A leftover `…staging-<pid>` name
        // also contains `pre-migration`, so this count catches that too. The reader above is closed
        // before the listing on purpose: a snapshot copy inherits the source's WAL header, so an open
        // reader creates `…sqlite-wal` and `…sqlite-shm` next to it, and counting those would be
        // counting this test rather than the product. Closed, SQLite checkpoints and removes them,
        // which is also what makes the copy self-contained enough to restore from.
        assert_eq!(
            backups_in(store.dir()),
            vec![
                backup
                    .file_name()
                    .expect("file name")
                    .to_string_lossy()
                    .into_owned()
            ],
            "exactly one snapshot for the v{found} → v{SCHEMA_VERSION} transition, and nothing else"
        );
    }
}

#[test]
fn a_fresh_store_writes_no_migration_backup() {
    let store = Store::new("fresh-no-backup");

    let db = Database::open(store.path()).expect("a fresh store opens");

    assert_eq!(schema_version_at(store.path()), SCHEMA_VERSION);
    assert!(matches!(
        db.integrity_check().expect("the check runs"),
        StoreHealth::Healthy
    ));
    assert!(
        backups_in(store.dir()).is_empty(),
        "an empty database has nothing worth snapshotting: {:?}",
        backups_in(store.dir())
    );
}

#[test]
fn a_store_already_at_this_version_backs_nothing_up_on_reopen() {
    let store = Store::new("reopen-no-backup");
    let db = Database::open(store.path()).expect("a fresh store opens");
    drop(db);
    let seeded = Database::open(store.path()).expect("the store reopens");
    seeded
        .connection()
        .execute_batch(
            "INSERT INTO projects (id, name) VALUES ('proj-r', 'Reopen project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                 VALUES ('build-r', 'proj-r', 'snap-r', 'p0-normalize-1', '0.6.0', 'COMPLETE');",
        )
        .expect("seed");
    drop(seeded);

    let again = Database::open(store.path()).expect("the store reopens once more");

    assert!(
        backups_in(store.dir()).is_empty(),
        "a store with no pending migration needs no snapshot: {:?}",
        backups_in(store.dir())
    );
    assert!(matches!(
        again.integrity_check().expect("the check runs"),
        StoreHealth::Healthy
    ));
}

// ---- D. backup failure blocks the upgrade -------------------------------------------------------

#[test]
fn a_snapshot_that_cannot_be_written_stops_the_upgrade_and_leaves_the_store_at_v4() {
    let store = Store::new("backup-blocked");
    build_a_v4_store(&store);
    let before = counts(&store);
    let bytes = std::fs::read(store.path()).expect("read the store");

    // The deterministic, cross-platform way to make a destination unusable: occupy it with a
    // directory. `chmod`-style read-only tricks do not mean the same thing on Windows, and SQLite
    // fails here for the same reason on every platform: the output file cannot be created.
    let wanted = store.backup_for(4);
    std::fs::create_dir_all(&wanted).expect("occupy the backup destination");

    let outcome = Database::open(store.path());
    let err = match outcome {
        Ok(_) => panic!("a blocked snapshot must not let the upgrade run"),
        Err(err) => err,
    };

    assert_eq!(err.stable_code(), "ERR-STORAGE-4011", "{err}");
    assert_eq!(schema_version_at(store.path()), 4, "the store stays at v4");
    assert!(
        !has_column(store.path(), "sections", "file_offset_unknown"),
        "no partial migration may reach the schema"
    );
    assert_eq!(counts(&store), before, "every row is still there");

    // The store is *not* byte-for-byte unchanged, and claiming it was would be a lie the next SQLite
    // release could tell on: opening a rollback-journal file switches it to WAL, and the switch writes
    // the database header. Measured on this fixture: four bytes, at 18 and 19 (the file format read and
    // write version, 1 → 2) and 27 and 95 (the change counter and its version-valid-for mirror,
    // 38 → 39). So the honest and still-strong claim is where the bytes are — nothing inside the
    // 100-byte header moved, and no page of stored data did.
    let after = std::fs::read(store.path()).expect("reread");
    assert_eq!(
        after.len(),
        bytes.len(),
        "a refused upgrade neither grew the store nor truncated it"
    );
    let mut moved: Vec<usize> = Vec::new();
    for (offset, (was, now)) in bytes.iter().zip(after.iter()).enumerate() {
        if was != now {
            moved.push(offset);
        }
    }
    assert!(
        moved.iter().all(|offset| *offset < 100),
        "only the database header may differ, and a page byte changed at {moved:?}"
    );

    // Nothing was left half-written: the staging file is gone, and the destination is still the
    // directory that refused it rather than a snapshot.
    assert!(
        wanted.is_dir(),
        "the unusable destination was not overwritten by a partial snapshot"
    );
    let leftovers: Vec<String> = std::fs::read_dir(store.dir())
        .expect("read the test directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains("staging"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "a failed snapshot must clean up its staging file: {leftovers:?}"
    );

    let text = err.to_string();
    assert!(
        !text.contains(&store.dir().to_string_lossy().to_string()),
        "the failure names what went wrong without carrying the store's directory: {text}"
    );
    assert!(
        !text.contains('\\') && !text.contains('/'),
        "the message carries a file name, not a located path: {text}"
    );
    let _ = std::fs::remove_dir_all(&wanted);
}

#[test]
fn a_repeated_transition_keeps_one_snapshot_and_replaces_it_only_with_a_written_one() {
    let store = Store::new("backup-replace");
    build_a_v4_store(&store);
    let first = Database::open(store.path()).expect("the first upgrade runs");
    drop(first);
    assert!(
        store.backup_for(4).exists(),
        "the first transition snapshotted"
    );

    // Same store, same transition, second attempt: the destination is already occupied by a verified
    // snapshot, so the product stages a new one and replaces only then. The store is at v5 now, so
    // this reproduces the situation by rebuilding the v4 file under the same name.
    let backup = store.backup_for(4);
    let kept = std::fs::read(&backup).expect("read the first snapshot");
    build_a_v4_store_at(&store);
    let second = Database::open(store.path()).expect("the second upgrade runs");
    drop(second);

    let listed = backups_in(store.dir());
    assert_eq!(
        listed.len(),
        1,
        "retention is one snapshot per transition, and the directory holds {listed:?}"
    );
    assert_ne!(
        std::fs::read(&backup).expect("reread the snapshot"),
        kept,
        "the replacement is the newer snapshot, not the stale one left standing"
    );
    assert_eq!(schema_version_at(store.path()), SCHEMA_VERSION);
}

// ---- E. counts, versions and journaling: the bounded facts a support surface reads --------------

#[test]
fn an_empty_store_reports_zero_in_every_counted_class() {
    let store = Store::new("counts-empty");
    let db = Database::open(store.path()).expect("a fresh store opens");

    let counts = db.counts();
    // Every class answers `Some(0)` rather than `None`. This is also the check on a mistyped table
    // name inside `counts()`: a count against a table that does not exist does not fail loudly, it
    // fails *silently*, and only a shape assertion catches that a class was never being read.
    assert_eq!(
        [
            counts.projects,
            counts.builds,
            counts.gate_runs,
            counts.accepted_reviews,
            counts.release_records,
        ],
        [Some(0); 5],
        "a store this build created holds nothing, and it answers for all five classes"
    );
}

#[test]
fn each_class_is_counted_on_its_own() {
    let store = Store::new("counts-rows");
    let db = Database::open(store.path()).expect("a fresh store opens");
    db.connection()
        .execute_batch(
            "INSERT INTO projects (id, name) VALUES ('proj-c', 'Counted project'),
                                          ('proj-c2', 'Second project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                 VALUES ('build-c', 'proj-c', 'snap-c', 'p0-normalize-1', '0.6.0', 'COMPLETE'),
                        ('build-c2', 'proj-c', 'snap-c2', 'p0-normalize-1', '0.6.0', 'IMPORTING'),
                        ('build-c3', 'proj-c2', 'snap-c3', 'p0-normalize-1', '0.6.0', 'COMPLETE');",
        )
        .expect("rows");

    let counts = db.counts();
    assert_eq!(counts.projects, Some(2));
    // Three, including the row an interrupted import left behind: the count states what the file
    // holds, and a support number that quietly filtered by state would understate the store a person
    // is being asked about.
    assert_eq!(counts.builds, Some(3));
    assert_eq!(counts.gate_runs, Some(0), "no Gate ran here");
    assert_eq!(counts.accepted_reviews, Some(0));
    assert_eq!(counts.release_records, Some(0));
}

#[test]
fn the_store_states_its_own_schema_version_and_the_journal_it_runs() {
    let store = Store::new("version-and-journal");
    let db = Database::open(store.path()).expect("a fresh store opens");

    assert_eq!(
        db.schema_version(),
        Some(SCHEMA_VERSION),
        "the version the file carries is read from the file, not assumed from the build"
    );
    let mode = db.journal_mode();
    assert!(
        !matches!(mode, JournalMode::Other),
        "SQLite named a journal mode this module could not map: {mode:?}"
    );
    assert!(
        !mode.as_str().is_empty(),
        "every journal mode has a name a support file can state"
    );
}
