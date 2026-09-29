//! The shipped `fwsight diff` surface: exit codes, stream discipline and deterministic output.
//!
//! These run the real binary rather than the library, because the contract under test is what a
//! user gets from a process: which code it exits with, what lands on stdout versus stderr, and
//! whether the file it wrote is byte-stable. The library path is already covered in
//! `firmwaresight-core` and `firmwaresight-report` tests.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BASE: &str = "fixtures/elf/p2-diff/base/firmware.elf";
const BASE_MAP: &str = "fixtures/elf/p2-diff/base/firmware.map";
const TARGET: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const TARGET_MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("apps/cli lives at <root>/apps/cli")
        .to_path_buf()
}

fn run(args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_fwsight"))
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("the fwsight binary is built for its own tests");
    // A process that panics is a defect regardless of which assertion follows, so every case
    // checks it: no stack trace may reach the user (prompt §46).
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(!stderr.contains("panicked"), "the CLI panicked:\n{stderr}");
    output
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn errors(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn diff_of_two_builds_exits_zero_and_shows_old_new_and_delta() {
    let output = run(&[
        "diff",
        BASE,
        TARGET,
        "--old-map",
        BASE_MAP,
        "--new-map",
        TARGET_MAP,
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));

    let stdout = text(&output);
    assert!(stdout.contains("Base    firmware.elf"), "{stdout}");
    assert!(stdout.contains("Target  firmware.elf"), "{stdout}");
    assert!(
        stdout.contains("256 bytes → 376 bytes   delta +120"),
        "{stdout}"
    );
    assert!(
        stdout.contains("8 bytes → 76 bytes   delta +68"),
        "{stdout}"
    );
    assert!(stdout.contains("Object / module attribution"), "{stdout}");
}

#[test]
fn the_two_maps_are_what_makes_the_memory_deltas_exact() {
    // Without either MAP the same two files still compare, but the memory claim gets weaker, and
    // the report has to say which side degraded (prompt §15).
    let with_maps = run(&[
        "diff",
        BASE,
        TARGET,
        "--json",
        "--old-map",
        BASE_MAP,
        "--new-map",
        TARGET_MAP,
    ]);
    let without = run(&["diff", BASE, TARGET, "--json"]);
    assert_eq!(with_maps.status.code(), Some(0));
    assert_eq!(without.status.code(), Some(0), "{}", errors(&without));

    let strong: serde_json::Value = serde_json::from_str(&text(&with_maps)).expect("json");
    let weak: serde_json::Value = serde_json::from_str(&text(&without)).expect("json");
    assert_eq!(strong["memory"]["comparability"], "exact");
    assert_eq!(weak["memory"]["comparability"], "exact");
    assert_eq!(strong["base"]["memory"]["evidence"]["mapBacked"], true);
    assert_eq!(weak["base"]["memory"]["evidence"]["mapBacked"], false);
    let codes: Vec<&str> = weak["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .map(|w| w["code"].as_str().expect("code"))
        .collect();
    assert!(codes.contains(&"MAP-EVIDENCE"), "{codes:?}");
}

#[test]
fn json_mode_prints_exactly_one_document_to_stdout_and_nothing_else() {
    let output = run(&["diff", BASE, TARGET, "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));
    let stdout = text(&output);
    assert_eq!(
        stdout.matches('\n').count(),
        1,
        "stdout must carry one compact document, got: {stdout}"
    );
    assert!(
        stdout.ends_with("}\n"),
        "a compact document plus exactly one trailing newline: {stdout}"
    );
    let document: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON document");
    assert_eq!(document["schema"], "urn:firmwaresight:schema:diff:1");
    assert_eq!(document["schemaVersion"], 1);
    // The human summary and diagnostics never leak into machine mode.
    assert!(
        !errors(&output).contains("diagnostics:"),
        "{}",
        errors(&output)
    );
}

#[test]
fn human_mode_keeps_diagnostics_on_stderr_and_the_summary_on_stdout() {
    let output = run(&[
        "diff",
        BASE,
        TARGET,
        "--old-map",
        BASE_MAP,
        "--new-map",
        TARGET_MAP,
    ]);
    let stdout = text(&output);
    let stderr = errors(&output);
    assert!(
        stdout.contains("Counts"),
        "summary belongs on stdout: {stdout}"
    );
    assert!(
        !stdout.contains("diagnostics:"),
        "diagnostics leaked to stdout"
    );
    assert!(
        stderr.contains("diagnostics: old map parsed with the gnu_ld adapter"),
        "{stderr}"
    );
    assert!(
        stderr.contains("diagnostics: new map parsed with the gnu_ld adapter"),
        "{stderr}"
    );
    assert!(stderr.contains("operation op-"), "{stderr}");
}

#[test]
fn a_map_that_was_never_supplied_is_reported_as_absent_and_never_inferred() {
    let output = run(&["diff", BASE, TARGET]);
    let stderr = errors(&output);
    assert!(
        stderr.contains("diagnostics: old map not provided"),
        "{stderr}"
    );
    assert!(
        stderr.contains("diagnostics: new map not provided"),
        "{stderr}"
    );

    // Prompt §15: no MAP means no MAP-backed claim in the document either.
    let json = run(&["diff", BASE, TARGET, "--json"]);
    let document: serde_json::Value = serde_json::from_str(&text(&json)).expect("json");
    assert_eq!(document["base"]["memory"]["evidence"]["mapBacked"], false);
    assert_eq!(document["target"]["memory"]["evidence"]["mapBacked"], false);
    assert_eq!(document["memory"]["comparability"], "exact");
}

#[test]
fn both_sides_of_the_document_are_named_and_never_carried_as_host_paths() {
    let output = run(&["diff", BASE, TARGET, "--old-map", BASE_MAP, "--json"]);
    let stdout = text(&output);
    let document: serde_json::Value = serde_json::from_str(&stdout).expect("json");

    assert_eq!(document["base"]["artifact"]["fileName"], "firmware.elf");
    assert_eq!(document["target"]["artifact"]["fileName"], "firmware.elf");
    assert_ne!(
        document["base"]["artifact"]["sha256"], document["target"]["artifact"]["sha256"],
        "the two fixture builds must not collapse onto one identity"
    );
    for marker in [
        BASE, TARGET, BASE_MAP, "C:\\", "D:\\", "/home/", "/Users/", "/tmp/",
    ] {
        assert!(
            !stdout.contains(marker),
            "host path or argument text leaked: {marker}"
        );
    }
}

#[test]
fn the_json_document_is_byte_stable_across_runs() {
    let first = run(&[
        "diff",
        BASE,
        TARGET,
        "--old-map",
        BASE_MAP,
        "--new-map",
        TARGET_MAP,
        "--json",
    ]);
    let second = run(&[
        "diff",
        BASE,
        TARGET,
        "--old-map",
        BASE_MAP,
        "--new-map",
        TARGET_MAP,
        "--json",
    ]);
    assert_eq!(
        text(&first),
        text(&second),
        "the export is not deterministic"
    );
}

#[test]
fn html_mode_writes_one_self_contained_file_and_prints_no_document_body() {
    let dir = temp_dir("html-ok");
    let file = dir.join("diff.html");
    let output = run(&[
        "diff",
        BASE,
        TARGET,
        "--old-map",
        BASE_MAP,
        "--new-map",
        TARGET_MAP,
        "--html",
        file.to_str().expect("utf-8 temp path"),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));

    let html = std::fs::read_to_string(&file).expect("the HTML file exists");
    assert!(html.starts_with("<!doctype html>"), "{html}");
    assert!(html.contains("urn:firmwaresight:schema:diff:1"));
    assert!(html.contains("Base (old)") && html.contains("Target (new)"));
    for forbidden in ["<script", "http://", "https://", "<link", "@import"] {
        assert!(
            !html.to_ascii_lowercase().contains(forbidden),
            "the export references {forbidden}"
        );
    }
    // The path the user chose is a host path, so it stays in diagnostics, never in the file.
    assert!(
        !html.contains(&dir.display().to_string()),
        "host path written into the export"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn json_and_html_together_put_json_on_stdout_and_html_in_the_file() {
    let dir = temp_dir("json-html");
    let file = dir.join("both.html");
    let output = run(&[
        "diff",
        BASE,
        TARGET,
        "--json",
        "--html",
        file.to_str().expect("path"),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));

    let stdout = text(&output);
    let document: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON document");
    assert_eq!(document["schemaVersion"], 1);
    assert!(
        std::fs::read_to_string(&file)
            .expect("html")
            .starts_with("<!doctype html>")
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_rewritten_html_file_is_identical_to_the_first_export() {
    let dir = temp_dir("html-stable");
    let first = dir.join("a.html");
    let second = dir.join("b.html");
    run(&[
        "diff",
        BASE,
        TARGET,
        "--html",
        first.to_str().expect("path"),
    ]);
    run(&[
        "diff",
        BASE,
        TARGET,
        "--html",
        second.to_str().expect("path"),
    ]);

    assert_eq!(
        std::fs::read_to_string(&first).expect("first"),
        std::fs::read_to_string(&second).expect("second"),
        "the same comparison must produce the same bytes"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_missing_input_exits_three_with_a_typed_error() {
    let output = run(&["diff", BASE, "fixtures/elf/p2-diff/nope.elf"]);
    assert_eq!(output.status.code(), Some(3), "{}", errors(&output));
    assert!(
        text(&output).is_empty(),
        "no diff may be printed for a failed import"
    );
    assert!(
        errors(&output).contains("code ERR-INPUT-0001"),
        "{}",
        errors(&output)
    );
}

#[test]
fn a_non_elf_input_exits_three_with_a_typed_error() {
    let dir = temp_dir("bad-elf");
    let junk = dir.join("junk.elf");
    std::fs::write(&junk, b"not an elf at all").expect("temp file writable");

    let output = run(&["diff", BASE, junk.to_str().expect("path")]);
    assert_eq!(output.status.code(), Some(3), "{}", errors(&output));
    assert!(
        errors(&output).contains("code ERR-FORMAT-0001"),
        "{}",
        errors(&output)
    );

    // The same file passed twice is the same snapshot, which is a selection error, not an empty
    // diff: answering with empty tables would read as "nothing changed".
    let same = run(&["diff", BASE, BASE]);
    assert_eq!(same.status.code(), Some(2), "{}", errors(&same));
    assert!(errors(&same).contains("ERR-DIFF-5001"), "{}", errors(&same));
    let envelope: serde_json::Value =
        serde_json::from_str(&text(&run(&["diff", BASE, BASE, "--json"]))).expect("one document");
    assert_eq!(envelope["code"], "ERR-DIFF-5001");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn an_unwritable_export_path_exits_six_after_the_comparison_succeeded() {
    let dir = temp_dir("export-fail");
    let impossible = dir.join("missing-sub").join("out.html");
    let output = run(&[
        "diff",
        BASE,
        TARGET,
        "--html",
        impossible.to_str().expect("path"),
    ]);
    assert_eq!(output.status.code(), Some(6), "{}", errors(&output));
    assert!(
        text(&output).is_empty(),
        "no success document before a failed export"
    );
    assert!(
        errors(&output).contains("ERR-EXPORT-6001"),
        "{}",
        errors(&output)
    );

    // Same run in machine mode: the error is still one machine-readable document.
    let json = run(&[
        "diff",
        BASE,
        TARGET,
        "--json",
        "--html",
        impossible.to_str().expect("path"),
    ]);
    assert_eq!(json.status.code(), Some(6));
    let envelope: serde_json::Value = serde_json::from_str(&text(&json)).expect("one document");
    assert_eq!(envelope["code"], "ERR-EXPORT-6001");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_missing_operand_is_a_usage_error_not_a_crash() {
    let output = run(&["diff", BASE]);
    assert_eq!(output.status.code(), Some(2), "{}", errors(&output));
    assert!(text(&output).is_empty());
}

#[test]
fn diff_writes_no_host_path_into_the_html_it_exports() {
    let dir = temp_dir("html-no-path");
    let file = dir.join("out.html");
    run(&[
        "diff",
        BASE,
        TARGET,
        "--old-map",
        BASE_MAP,
        "--html",
        file.to_str().expect("path"),
    ]);
    let html = std::fs::read_to_string(&file).expect("html");
    for marker in [BASE, BASE_MAP, "C:\\", "D:\\", "/home/", "/Users/"] {
        assert!(!html.contains(marker), "the HTML export carries {marker}");
    }
    std::fs::remove_dir_all(&dir).ok();
}

/// A scratch directory under `target/`, so nothing is written outside the build tree and a failed
/// test leaves no residue in the repository.
fn temp_dir(tag: &str) -> PathBuf {
    let root = repo_root().join("target").join("cli-tests");
    let dir = root.join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}
