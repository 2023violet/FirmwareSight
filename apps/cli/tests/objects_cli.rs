//! `fwsight analyze --objects` and the object section of `fwsight diff` (ADR-0033 A6, A8).
//!
//! The real binary is run, because the contract is what a user's terminal receives: the table is
//! text only, a build without MAP object rows says so instead of printing zeros, the JSON document
//! keeps its shape, and `--objects` cannot be mixed into a `--json` run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BASE: &str = "fixtures/elf/m2-objects/base/firmware.elf";
const BASE_MAP: &str = "fixtures/elf/m2-objects/base/firmware.map";
const TARGET: &str = "fixtures/elf/m2-objects/target/firmware.elf";
const TARGET_MAP: &str = "fixtures/elf/m2-objects/target/firmware.map";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn objects_lists_each_object_and_library_with_the_remainder() {
    let output = run(&["analyze", TARGET, "--map", TARGET_MAP, "--objects"]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);

    assert!(
        text.contains("Objects and libraries (linker MAP rows"),
        "{text}"
    );
    assert!(text.contains("  Objects (4)"), "{text}");
    assert!(text.contains("libdrv.a(spi.o)"), "{text}");
    assert!(text.contains("  Libraries (1)"), "{text}");
    assert!(
        text.contains("Not attributed to an object: 20 bytes Flash, 0 bytes RAM"),
        "{text}"
    );
    assert!(!text.contains("build/app"), "no host or build path: {text}");
}

#[test]
fn objects_without_a_map_says_unavailable_rather_than_printing_zeros() {
    let output = run(&["analyze", TARGET, "--objects"]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    assert!(
        text.contains("unavailable: this build has no linker MAP object rows"),
        "{text}"
    );
    assert!(!text.contains("  Objects ("), "{text}");
}

#[test]
fn objects_cannot_be_mixed_into_a_json_run_and_json_keeps_its_shape() {
    let refused = run(&[
        "analyze",
        TARGET,
        "--map",
        TARGET_MAP,
        "--objects",
        "--json",
    ]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(stdout(&refused).is_empty(), "nothing reaches stdout");

    let json = run(&["analyze", TARGET, "--map", TARGET_MAP, "--json"]);
    assert_eq!(json.status.code(), Some(0));
    let document: serde_json::Value =
        serde_json::from_str(&stdout(&json)).expect("one JSON document");
    assert_eq!(
        document["capabilities"]["objectAttribution"], "available",
        "D7: only the capability value moves"
    );
    assert!(
        !stdout(&json).contains("libdrv.a("),
        "per-object rows stay out of analysis:1"
    );
}

#[test]
fn diff_text_names_object_growth_and_additions() {
    let output = run(&[
        "diff",
        BASE,
        TARGET,
        "--old-map",
        BASE_MAP,
        "--new-map",
        TARGET_MAP,
    ]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    assert!(
        text.contains("Object / library attribution (linker MAP rows)"),
        "{text}"
    );
    assert!(text.contains("Objects    "), "{text}");
    let added = text
        .split("Largest added objects")
        .nth(1)
        .expect("the additions list is printed");
    assert!(added.contains("libdrv.a(spi.o)"), "{text}");
}

const ARM_BASE: &str = "fixtures/elf/d4-armlink/base/firmware.axf";
const ARM_BASE_MAP: &str = "fixtures/elf/d4-armlink/base/firmware.map";
const ARM_TARGET: &str = "fixtures/elf/d4-armlink/target/firmware.axf";
const ARM_TARGET_MAP: &str = "fixtures/elf/d4-armlink/target/firmware.map";

#[test]
fn an_armlink_map_gives_regions_objects_and_libraries_like_a_gnu_one() {
    // ADR-0036: the same report for a Keil / Arm Compiler build as for a GNU ld build.
    let output = run(&["analyze", ARM_TARGET, "--map", ARM_TARGET_MAP, "--objects"]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    assert!(
        text.contains("nonvolatile / load image 192 bytes"),
        "{text}"
    );
    assert!(
        text.contains("ER_IROM1 (nonvolatile / load image)"),
        "{text}"
    );
    assert!(text.contains("Capacity: 65536 bytes"), "{text}");
    assert!(text.contains("libdrv.lib(spi.o)"), "{text}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("map parsed with the armlink adapter"),
        "{stderr}"
    );

    let diff = run(&[
        "diff",
        ARM_BASE,
        ARM_TARGET,
        "--old-map",
        ARM_BASE_MAP,
        "--new-map",
        ARM_TARGET_MAP,
    ]);
    assert_eq!(diff.status.code(), Some(0));
    let text = stdout(&diff);
    assert!(
        text.contains("108 bytes → 192 bytes delta +84 [exact]"),
        "{text}"
    );
    assert!(
        text.contains("Object / library attribution (linker MAP rows)"),
        "{text}"
    );
    assert!(
        text.contains("libdrv.lib(spi.o)"),
        "the added library member: {text}"
    );
    // ADR-0036 A2: the RW data and ZI parts of RW_IRAM1 pair by role, so nothing is left unpaired.
    assert!(
        text.contains("Sections  +0 -0 ~9 (0 rows whose name repeats)"),
        "{text}"
    );
}
