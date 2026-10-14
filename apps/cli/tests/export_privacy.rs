//! Real-process output boundary regressions. Every path/name here is synthetic.
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .into()
}

#[test]
fn export_refusals_never_echo_destination_or_config_in_either_stream() {
    let scratch = root()
        .join("target/cli-tests")
        .join(format!("privacy-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let existing = scratch.join("private-canary-existing.svg");
    std::fs::write(&existing, b"KEEP ORIGINAL").unwrap();
    let directory = scratch.join("private-canary-directory");
    std::fs::create_dir_all(&directory).unwrap();
    let blocked_parent = scratch.join("private-canary-parent-file");
    std::fs::write(&blocked_parent, b"KEEP PARENT").unwrap();
    let destinations = [
        existing.clone(),
        directory,
        blocked_parent.join("report.svg"),
        scratch.join("private-canary-missing/nested/report.svg"),
        scratch
            .join("missing")
            .join("DRIVE_TEXT=C:\\private-canary\\config=PRIVATE_CONFIG_CANARY.svg"),
        scratch
            .join("missing")
            .join("UNC_TEXT=\\\\server\\private-canary\\report.svg"),
        scratch.join("missing/home/private-canary/report.svg"),
        scratch.join("missing/固件\u{202e}PRIVATE_CONFIG_CANARY.svg"),
    ];
    for verb in ["analyze", "diff"] {
        for flag in ["--svg", "--png", "--report-html", "--html"] {
            if verb == "analyze" && flag == "--html" {
                continue;
            }
            for json in [false, true] {
                for destination in &destinations {
                    // Legacy detailed HTML permits file replacement; use a directory instead.
                    if flag == "--html" && destination == &existing {
                        continue;
                    }
                    let mut command = Command::new(env!("CARGO_BIN_EXE_fwsight"));
                    command
                        .current_dir(root())
                        .env("RUST_LOG", "trace,fontdb=trace");
                    command.args([verb, "fixtures/elf/p2-diff/base/firmware.elf"]);
                    if verb == "diff" {
                        command.arg("fixtures/elf/p2-diff/target/firmware.elf");
                    }
                    command.arg(flag).arg(destination);
                    if json {
                        command.arg("--json");
                    }
                    let output = command.output().unwrap();
                    assert_eq!(output.status.code(), Some(6), "{verb} {flag} must refuse");
                    for stream in [&output.stdout, &output.stderr] {
                        let text = String::from_utf8_lossy(stream);
                        for forbidden in [
                            "private-canary",
                            "PRIVATE_CONFIG_CANARY",
                            "C:\\",
                            "\\\\server",
                            "/home/",
                            "\\WINDOWS\\Fonts\\",
                            "\u{202e}",
                        ] {
                            assert!(
                                !text.contains(forbidden),
                                "{verb} {flag} disclosed synthetic input marker"
                            );
                        }
                        assert!(
                            !text.contains(&scratch.to_string_lossy().to_string()),
                            "destination parent disclosed"
                        );
                    }
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    assert!(stderr.contains("ERR-EXPORT-6001"));
                    assert!(stderr.contains("operation op-"));
                    assert!(stderr.contains("retry"));
                    if json {
                        let error: serde_json::Value =
                            serde_json::from_slice(&output.stdout).unwrap();
                        assert_eq!(error["code"], "ERR-EXPORT-6001");
                        assert!(error["operationId"].as_str().unwrap().starts_with("op-"));
                        assert!(error["remediation"].as_str().unwrap().contains("retry"));
                    } else {
                        assert!(output.stdout.is_empty());
                    }
                }
            }
        }
    }
    assert_eq!(std::fs::read(existing).unwrap(), b"KEEP ORIGINAL");
    assert_eq!(std::fs::read(blocked_parent).unwrap(), b"KEEP PARENT");
    std::fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn explicit_foreign_map_is_refused_and_elf_only_recovery_is_actionable() {
    let scratch = root()
        .join("target/cli-tests")
        .join(format!("map-privacy-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let map = scratch.join("private-canary-固件.map");
    for content in [
        "IAR ELF Linker synthetic",
        "Component: ARM Compiler Tool: armlink [synthetic]",
    ] {
        std::fs::write(&map, content).unwrap();
        for json in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_fwsight"));
            command
                .current_dir(root())
                .args([
                    "analyze",
                    "fixtures/elf/p5-compat/clang-arm/firmware.elf",
                    "--map",
                ])
                .arg(&map);
            if json {
                command.arg("--json");
            }
            let output = command.output().unwrap();
            assert_eq!(output.status.code(), Some(3));
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!combined.contains("private-canary"));
            assert!(combined.contains("ERR-MAP-3001"));
            assert!(combined.contains("without a MAP"));
        }
    }
    let alone = Command::new(env!("CARGO_BIN_EXE_fwsight"))
        .current_dir(root())
        .args([
            "analyze",
            "fixtures/elf/p5-compat/clang-arm/firmware.elf",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(alone.status.success());
    let value: serde_json::Value = serde_json::from_slice(&alone.stdout).unwrap();
    assert_eq!(value["capabilities"]["map"], "not-provided");
    let help = Command::new(env!("CARGO_BIN_EXE_fwsight"))
        .args(["--help"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&help.stdout)
            .contains("Keil/IAR compiler cohorts remain unverified")
    );
    std::fs::remove_dir_all(scratch).unwrap();
}
