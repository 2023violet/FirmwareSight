//! P0 parity and boundary tests for the Desktop shell.
//!
//! The claim under test is the one the whole slice exists to prove: the CLI and the Desktop
//! report the same Core facts for the same bytes. These tests compare the Desktop's IPC summary
//! against the committed CLI golden, so a divergence fails the build rather than surfacing as a
//! UI difference somebody notices later.

use std::path::{Path, PathBuf};

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
            "fwsight-p0-desktop-{label}-{}-{:?}.sqlite",
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

fn golden(name: &str) -> Value {
    let path = repo_root().join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{} must be committed for the parity check: {err}",
            path.display()
        )
    });
    serde_json::from_str(&text).expect("golden is JSON")
}

fn session(db: &TempDb) -> Session {
    Session::open(FixtureCatalog::from_environment(), db.path())
        .unwrap_or_else(|err| panic!("desktop session opens: {err}"))
}

#[test]
fn desktop_and_cli_report_the_same_core_facts_for_the_same_bytes() {
    let db = TempDb::new("parity");
    let session = session(&db);

    let summary = session
        .analyze_and_store(FixtureKey::P0DualRegion, "op-parity")
        .expect("the dual-region fixture analyzes");
    let cli = golden("golden/cli/p0-dual-region-analyze.json");

    // The field list is the P0 prompt's parity checklist, taken literally.
    assert_eq!(
        summary.artifact.sha256,
        cli["artifact"]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        summary.artifact.byte_size,
        cli["artifact"]["byteSize"].as_u64().unwrap()
    );
    assert_eq!(
        summary.artifact.architecture,
        cli["artifact"]["architecture"].as_str().unwrap()
    );
    assert_eq!(
        summary.artifact.bitness,
        cli["artifact"]["bitness"].as_str().unwrap()
    );
    assert_eq!(
        summary.artifact.endianness,
        cli["artifact"]["endianness"].as_str().unwrap()
    );
    assert_eq!(
        summary.artifact.entry_point.as_deref(),
        Some(cli["artifact"]["entryPoint"].as_str().unwrap())
    );

    assert_eq!(
        summary.memory.nonvolatile_image_footprint.bytes,
        cli["memory"]["nonvolatileImageFootprint"]["bytes"].as_u64()
    );
    assert_eq!(
        summary.memory.runtime_ram_footprint.bytes,
        cli["memory"]["runtimeRamFootprint"]["bytes"].as_u64()
    );
    assert_ne!(
        summary.memory.nonvolatile_image_footprint.bytes,
        summary.memory.runtime_ram_footprint.bytes,
        "the two budgets must not collapse into one number"
    );

    assert_eq!(
        summary.section_count,
        cli["counts"]["sections"].as_u64().expect("section count") as usize
    );
    assert_eq!(
        summary.symbol_count,
        cli["counts"]["symbols"].as_u64().expect("symbol count") as usize
    );

    let cli_caps = &cli["capabilities"];
    assert_eq!(summary.capabilities.elf, cli_caps["elf"].as_str().unwrap());
    assert_eq!(
        summary.capabilities.sections,
        cli_caps["sections"].as_str().unwrap()
    );
    assert_eq!(
        summary.capabilities.symbols,
        cli_caps["symbols"].as_str().unwrap()
    );
    assert_eq!(
        summary.capabilities.debug_info,
        cli_caps["debugInfo"].as_str().unwrap()
    );
    assert_eq!(summary.capabilities.map, cli_caps["map"].as_str().unwrap());
    assert_eq!(
        summary.capabilities.object_attribution,
        cli_caps["objectAttribution"].as_str().unwrap()
    );
    assert_eq!(summary.capabilities.git, cli_caps["git"].as_str().unwrap());
}

#[test]
fn the_desktop_projection_keeps_the_same_evidence_claims_as_the_cli() {
    // Parity is not only numbers: the desktop must not quietly upgrade a name heuristic into
    // observed load evidence, or drop the dual-accounting it cannot support.
    let db = TempDb::new("evidence");
    let session = session(&db);
    let summary = session
        .analyze_and_store(FixtureKey::P0DualRegion, "op-evidence")
        .expect("analyzes");
    let cli = golden("golden/cli/p0-dual-region-analyze.json");

    assert_eq!(
        summary.memory.weakest_evidence_basis.as_deref(),
        Some("map-memory-configuration+elf-load")
    );
    assert_eq!(
        summary.memory.layout_source,
        cli["memory"]["layoutSource"].as_str().unwrap()
    );
    assert!(summary.memory.admissible_for_hard_block);
    assert_eq!(
        summary.memory.dual_accounted_sections.len(),
        cli["memory"]["dualAccountedSections"]
            .as_array()
            .expect("array")
            .len()
    );
    assert_eq!(
        summary.evidence_summary.total,
        cli["counts"]["evidence"].as_u64().expect("evidence count") as usize
    );
    assert_eq!(
        summary.evidence_summary.observed
            + summary.evidence_summary.derived
            + summary.evidence_summary.declared
            + summary.evidence_summary.unknown,
        summary.evidence_summary.total,
        "every evidence item must be classified into exactly one bucket"
    );
}

#[test]
fn without_region_evidence_the_desktop_reports_weaker_evidence_too() {
    // The same rule the CLI follows must hold on the other surface, otherwise the UI would be
    // the place where an unsupported claim becomes possible.
    let db = TempDb::new("basic");
    let session = session(&db);
    let summary = session
        .analyze_and_store(FixtureKey::P0Basic, "op-basic")
        .expect("analyzes");
    let cli = golden("golden/cli/p0-basic-analyze.json");

    assert_eq!(summary.memory.layout_source, "none");
    assert_eq!(
        summary.memory.weakest_evidence_basis.as_deref(),
        Some("elf-address-and-flags"),
        "address and flag inference is rung 3, not linker-declared region evidence"
    );
    assert!(!summary.memory.admissible_for_hard_block);
    assert_eq!(summary.capabilities.map, "not-provided");
    // Dual accounting still appears without a MAP - `.data` has both a load size and a runtime
    // size in the ELF itself. What must change is the strength of the claim, which is exactly
    // what `admissible_for_hard_block` reports.
    assert_eq!(
        summary.memory.dual_accounted_sections,
        cli["memory"]["dualAccountedSections"]
            .as_array()
            .expect("array")
            .iter()
            .map(|v| v.as_str().expect("string locator").to_owned())
            .collect::<Vec<String>>()
    );
    assert_eq!(
        summary.memory.runtime_ram_footprint.state,
        cli["memory"]["runtimeRamFootprint"]["state"]
            .as_str()
            .expect("state")
    );
    assert_eq!(
        summary.memory.runtime_ram_footprint.classification,
        cli["memory"]["runtimeRamFootprint"]["classification"]
            .as_str()
            .expect("classification")
    );
}

#[test]
fn the_ipc_payload_is_bounded_and_carries_no_path() {
    let db = TempDb::new("bounded");
    let session = session(&db);
    let summary = session
        .analyze_and_store(FixtureKey::P0DualRegion, "op-bounded")
        .expect("analyzes");

    let text = serde_json::to_string(&summary).expect("summary serializes");
    let payload: Value = serde_json::from_str(&text).expect("round trip");

    // A symbol table over IPC was explicitly ruled out: 32 symbols here, but the same call on a
    // real firmware would carry tens of thousands.
    assert!(
        payload.get("symbols").is_none(),
        "no symbol list crosses IPC"
    );
    assert!(
        payload.get("sections").is_none(),
        "no section list crosses IPC"
    );
    assert!(
        payload.get("evidence").is_none(),
        "only the evidence summary crosses IPC"
    );
    for forbidden in ["path", "filePath", "directory"] {
        let key = format!("\"{forbidden}\"");
        assert!(
            !text.contains(&key),
            "the IPC payload must name a file, never locate it"
        );
    }
    assert!(
        text.len() < 8_192,
        "summary payload grew to {} bytes; the boundary is supposed to be bounded",
        text.len()
    );
}

#[test]
fn analyzing_the_same_fixture_twice_does_not_duplicate_history() {
    let db = TempDb::new("dedupe");
    let session = session(&db);

    for _ in 0..3 {
        session
            .analyze_and_store(FixtureKey::P0DualRegion, "op-repeat")
            .expect("analyzes");
    }

    let probe = firmwaresight_storage::Database::open(db.path()).expect("reopen to count");
    let builds: i64 = probe
        .connection()
        .query_row("SELECT COUNT(*) FROM builds", [], |row| row.get(0))
        .expect("count builds");
    let complete: i64 = probe
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM builds WHERE state = 'COMPLETE'",
            [],
            |row| row.get(0),
        )
        .expect("count complete");

    assert_eq!(
        builds, 1,
        "three clicks on one fixture is one build in history"
    );
    assert_eq!(complete, 1);
}

#[test]
fn a_missing_analyze_still_produces_a_typed_error_not_a_panic() {
    // Point the catalog at a directory with no fixtures. The failure must arrive as the same
    // envelope shape the UI renders, from the same code the CLI would print.
    let db = TempDb::new("missing");
    let empty = std::env::temp_dir().join(format!("fwsight-p0-empty-{}", std::process::id()));
    let session = Session::open(FixtureCatalog::new(&empty), db.path()).expect("opens");

    match session.analyze_and_store(FixtureKey::P0Basic, "op-missing") {
        Ok(_) => panic!("a catalog with no fixtures must not analyze"),
        Err(envelope) => {
            assert_eq!(envelope.code, "ERR-INPUT-0001");
            assert_eq!(envelope.operation_id, "op-missing");
            assert!(!envelope.message.is_empty());
            assert!(envelope.remediation.is_some());
        }
    }
}
