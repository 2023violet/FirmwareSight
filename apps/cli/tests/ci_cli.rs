//! CI exports (ADR-0040): `gate --junit`, `gate --summary-md` and `diff --summary-md`.
//!
//! This file checks the following contract:
//! - each file is written, and the same run writes the same bytes;
//! - the JUnit suite has one case per Gate rule, and the process exit code still states the verdict;
//! - every summary starts with the marker a CI script searches for, and contains no other HTML;
//! - an existing file is never replaced (exit 6), and two exports of one run may not name the same file.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

const PROJECT: &str = "fixtures/project/p4-release";
const P2_BASE: &str = "fixtures/elf/p2-diff/base/firmware.elf";
const P2_BASE_MAP: &str = "fixtures/elf/p2-diff/base/firmware.map";
const P2_TARGET: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const P2_TARGET_MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";
const D4: &str = "fixtures/elf/d4-armlink";
const MARKER: &str = "<!-- firmwaresight-summary -->";

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
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(!stderr.contains("panicked"), "the CLI panicked:\n{stderr}");
    output
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "fwsight-ci-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn file(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().into_owned()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn read(path: &str) -> String {
    std::fs::read_to_string(path).expect("the export was written")
}

fn gate(extra: &[&str]) -> Output {
    let mut args = vec![
        "gate",
        "--project",
        PROJECT,
        "--artifact",
        P2_TARGET,
        "--map",
        P2_TARGET_MAP,
        "--baseline",
        P2_BASE,
        "--baseline-map",
        P2_BASE_MAP,
    ];
    args.extend_from_slice(extra);
    run(&args)
}

fn no_html_but_the_marker(text: &str) {
    assert!(
        text.starts_with(MARKER),
        "the marker opens the block:\n{text}"
    );
    let rest = &text[MARKER.len()..];
    assert!(!rest.contains('<'), "no HTML after the marker:\n{text}");
}

#[test]
fn gate_junit_has_one_case_per_rule_and_the_exit_code_still_states_the_verdict() {
    let dir = TempDir::new();
    let (first, second) = (dir.file("a.xml"), dir.file("b.xml"));
    let out = gate(&["--junit", &first]);
    let code = out.status.code().expect("exit code");
    assert!(matches!(code, 0 | 4 | 5), "a verdict code, got {code}");
    let doc = read(&first);
    assert!(doc.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites"));
    assert_eq!(doc.matches("<testcase ").count(), 10, "ten Gate rules");
    assert!(doc.contains("name=\"memory.flash_budget\""));
    let verdict = match code {
        0 => "PASS",
        4 => "REVIEW",
        _ => "BLOCK",
    };
    assert!(doc.contains(&format!(
        "name=\"firmwaresight.verdict\" value=\"{verdict}\""
    )));
    assert_eq!(
        code == 5,
        doc.contains("<failure "),
        "a failure exactly when the Gate blocks"
    );

    let again = gate(&["--junit", &second]);
    assert_eq!(again.status.code(), Some(code));
    assert_eq!(doc, read(&second), "the same run writes the same bytes");
}

#[test]
fn gate_summary_names_the_verdict_after_the_marker() {
    let dir = TempDir::new();
    let path = dir.file("gate.md");
    let out = gate(&["--summary-md", &path]);
    let text = read(&path);
    no_html_but_the_marker(&text);
    let verdict = match out.status.code() {
        Some(0) => "PASS",
        Some(4) => "REVIEW",
        _ => "BLOCK",
    };
    assert!(
        text.contains(&format!("release Gate: **{verdict}**")),
        "{text}"
    );
}

#[test]
fn diff_summary_lists_budgets_and_object_movers() {
    let dir = TempDir::new();
    let path = dir.file("diff.md");
    let out = run(&[
        "diff",
        &format!("{D4}/base/firmware.axf"),
        &format!("{D4}/target/firmware.axf"),
        "--old-map",
        &format!("{D4}/base/firmware.map"),
        "--new-map",
        &format!("{D4}/target/firmware.map"),
        "--summary-md",
        &path,
    ]);
    assert_eq!(out.status.code(), Some(0));
    let text = read(&path);
    no_html_but_the_marker(&text);
    assert!(
        text.contains("| Flash (load image) | 108 bytes | 192 bytes | **▲84 bytes** | exact |"),
        "{text}"
    );
    assert!(text.contains("| `imu.o` | changed | ▲54 bytes |"), "{text}");
    assert!(
        text.contains("| `libdrv.lib(spi.o)` | added | ▲24 bytes |"),
        "{text}"
    );
}

#[test]
fn an_existing_file_or_a_shared_destination_is_refused() {
    let dir = TempDir::new();
    let path = dir.file("taken.xml");
    std::fs::write(&path, "keep me").expect("seed");
    let out = gate(&["--junit", &path]);
    assert_eq!(out.status.code(), Some(6));
    assert_eq!(read(&path), "keep me", "an existing file is never replaced");

    let shared = dir.file("shared");
    let out = gate(&["--junit", &shared, "--summary-md", &shared]);
    assert_eq!(out.status.code(), Some(6));
    assert!(!Path::new(&shared).exists(), "refused before any work");
}
