//! `fwsight` — the FirmwareSight command line.
//!
//! P2 implements `analyze` and `diff`. `gate` and `release prepare` are defined in
//! `04_TECH/07_CLI_SPEC.md` but are *not registered here*, so a user cannot mistake them for
//! working features. Exit codes 4 (gate review) and 5 (gate block) are likewise unreachable until
//! the Gate phase exists; nothing pretends otherwise. Code 6 became reachable with the P2 export.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use firmwaresight_artifact::ArtifactError;
use firmwaresight_artifact::intake::GuardConfig;
use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::diff::{DiffError, DiffSnapshotInput, compare};
use firmwaresight_report::{AnalyzeResultDto, DiffResultDto, diff_render, render};

/// Exit codes frozen by 04_TECH/07_CLI_SPEC.md. `1` is intentionally not defined.
const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 2;
const EXIT_PARSE_IMPORT: u8 = 3;
/// The comparison succeeded but the requested export could not be written.
const EXIT_EXPORT: u8 = 6;

#[derive(Debug, Parser)]
#[command(
    name = "fwsight",
    version,
    about = "Know exactly what ships.",
    long_about = "FirmwareSight P0/P1/P2 technical slice.\n\n`analyze` and `diff` are implemented; \
                  Release Gate and Release Bundle are later phases and are not available."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Analyze one artifact and report the facts that were observed.
    Analyze(AnalyzeArgs),
    /// Compare two artifacts and report what changed between them.
    Diff(DiffArgs),
}

#[derive(Debug, Parser)]
struct AnalyzeArgs {
    /// ELF artifact to analyze.
    path: PathBuf,

    /// GNU ld MAP file for region and load-address evidence.
    #[arg(long, value_name = "FILE")]
    map: Option<PathBuf>,

    /// Emit exactly one machine-readable JSON document on stdout.
    #[arg(long)]
    json: bool,

    /// Full-buffer size ceiling in bytes. Defaults to the baseline 512 MiB.
    #[arg(long, value_name = "BYTES")]
    max_bytes: Option<u64>,
}

#[derive(Debug, Parser)]
struct DiffArgs {
    /// The older ELF artifact. Deltas are measured from this side.
    old: PathBuf,

    /// The newer ELF artifact. Deltas are measured to this side.
    new: PathBuf,

    /// GNU ld MAP file for the older artifact.
    #[arg(long, value_name = "FILE")]
    old_map: Option<PathBuf>,

    /// GNU ld MAP file for the newer artifact.
    #[arg(long, value_name = "FILE")]
    new_map: Option<PathBuf>,

    /// Emit exactly one machine-readable diff document on stdout.
    #[arg(long)]
    json: bool,

    /// Write the deterministic self-contained HTML report to this file.
    #[arg(long, value_name = "FILE")]
    html: Option<PathBuf>,
}

fn main() -> ExitCode {
    // Diagnostics belong on stderr so that `--json` stdout stays a single clean document.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    // Parsing goes through `parse_from` rather than `Cli::parse`, because `parse` exits the
    // process itself: the frozen usage code would be clap's, not the one this table promises.
    let cli = match parse_from(std::env::args_os()) {
        Ok(cli) => cli,
        Err(code) => return ExitCode::from(code),
    };
    let operation_id = next_operation_id();

    match cli.command {
        Commands::Analyze(args) => run_analyze(args, &operation_id),
        Commands::Diff(args) => run_diff(args, &operation_id),
    }
}

/// A per-run identifier for diagnostics. It is deliberately never written into the
/// deterministic JSON payload, whose output must be reproducible across runs.
fn next_operation_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("op-{:x}-{:x}", std::process::id(), nanos)
}

fn run_analyze(args: AnalyzeArgs, operation_id: &str) -> ExitCode {
    let guard = match args.max_bytes {
        Some(limit) => GuardConfig::with_limit(limit),
        None => GuardConfig::default(),
    };

    let mut request = AnalysisRequest::new(args.path.clone()).with_guard(guard);
    if let Some(map_path) = &args.map {
        request = request.with_map(map_path.clone());
    }

    match pipeline::analyze(&request) {
        Ok(analysis) => {
            let dto = AnalyzeResultDto::from_snapshot(&analysis.snapshot, &file_name(&args.path));

            if args.json {
                // stdout carries nothing but the JSON document.
                print!("{}", render::render_json(&dto));
            } else {
                print!("{}", render::render_human(&dto));
                eprintln!("diagnostics: map {}", map_state(&analysis));
                eprintln!(
                    "diagnostics: hash {} ms, read {} ms, parse {} ms, normalize {} ms, operation {operation_id}",
                    analysis.input.hash_ms,
                    analysis.input.read_ms,
                    analysis.parse_ms,
                    analysis.normalize_ms
                );
            }
            ExitCode::from(EXIT_OK)
        }
        Err(err) => {
            let envelope =
                render::ErrorEnvelope::new(err.stable_code(), err.user_message(), operation_id)
                    .with_remediation(err.remediation());

            if args.json {
                // Errors are still machine-readable under --json, and still the only document.
                println!("{}", envelope.to_json());
            }
            eprintln!(
                "error: {} (code {}, operation {operation_id})",
                envelope.message, envelope.code
            );
            tracing::error!(operation_id, code = err.stable_code(), "analysis failed");
            ExitCode::from(exit_code_for(&err))
        }
    }
}

/// Compare two artifacts. Each side is analyzed with the same pipeline `analyze` uses, then handed
/// to Core, which owns every diff rule.
fn run_diff(args: DiffArgs, operation_id: &str) -> ExitCode {
    let base = match analyze_side(&args.old, args.old_map, "old", args.json, operation_id) {
        Ok(analysis) => analysis,
        Err(code) => return ExitCode::from(code),
    };
    let target = match analyze_side(&args.new, args.new_map, "new", args.json, operation_id) {
        Ok(analysis) => analysis,
        Err(code) => return ExitCode::from(code),
    };

    let base_input = DiffSnapshotInput::from_snapshot(&base.snapshot);
    let target_input = DiffSnapshotInput::from_snapshot(&target.snapshot);

    let result = match compare(&base_input, &target_input) {
        Ok(result) => result,
        Err(err) => return report_diff_error(&err, args.json, operation_id),
    };
    let dto = DiffResultDto::from_diff(&result);

    // The export is written before anything is printed: a run that cannot produce the file the user
    // asked for must not already have announced success on stdout.
    if let Some(path) = &args.html {
        let html = diff_render::render_html(&dto);
        if let Err(error) = std::fs::write(path, html) {
            return report_export_error(path, &error, args.json, operation_id);
        }
    }

    if args.json {
        print!("{}", diff_render::render_json(&dto));
    } else {
        print!("{}", diff_render::render_human(&result));
        eprintln!("diagnostics: old map {}", map_state(&base));
        eprintln!("diagnostics: new map {}", map_state(&target));
        if args.html.is_some() {
            eprintln!("diagnostics: html written to the requested file");
        }
        eprintln!("diagnostics: operation {operation_id}");
    }
    ExitCode::from(EXIT_OK)
}

/// Analyze one side of a comparison. A failure here is an import failure, so no partial diff is
/// printed and the process exits 3.
fn analyze_side(
    path: &Path,
    map: Option<PathBuf>,
    side: &str,
    json: bool,
    operation_id: &str,
) -> Result<pipeline::Analysis, u8> {
    let mut request = AnalysisRequest::new(path.to_path_buf()).with_guard(GuardConfig::default());
    if let Some(map_path) = map {
        request = request.with_map(map_path);
    }

    pipeline::analyze(&request).map_err(|err| {
        let envelope =
            render::ErrorEnvelope::new(err.stable_code(), err.user_message(), operation_id)
                .with_details(format!("{side} side: {}", file_name(path)))
                .with_remediation(err.remediation());
        if json {
            println!("{}", envelope.to_json());
        }
        eprintln!(
            "error: {} (code {}, operation {operation_id})",
            envelope.message, envelope.code
        );
        tracing::error!(
            operation_id,
            side,
            code = err.stable_code(),
            "diff input failed"
        );
        exit_code_for(&err)
    })
}

/// Selecting the same build twice is a usage error, not an empty comparison. Answering with empty
/// tables would read as "nothing changed" rather than "nothing compared".
fn report_diff_error(err: &DiffError, json: bool, operation_id: &str) -> ExitCode {
    let (message, remediation) = match err {
        DiffError::SameSnapshot { snapshot_id } => (
            format!(
                "both sides resolve to the same snapshot ({snapshot_id}); there is nothing to compare"
            ),
            "Choose two different builds, or compare the same file against a rebuilt one.",
        ),
    };
    let envelope =
        render::ErrorEnvelope::new(err.code(), message, operation_id).with_remediation(remediation);
    if json {
        println!("{}", envelope.to_json());
    }
    eprintln!(
        "error: {} (code {}, operation {operation_id})",
        envelope.message, envelope.code
    );
    ExitCode::from(EXIT_USAGE)
}

/// The comparison succeeded; writing its export did not. Exit 6, and the host path stays in
/// diagnostics rather than in the machine-readable envelope.
fn report_export_error(
    path: &Path,
    error: &std::io::Error,
    json: bool,
    operation_id: &str,
) -> ExitCode {
    let envelope = render::ErrorEnvelope::new(
        render::EXPORT_FAILED_CODE,
        format!("could not write the HTML export: {error}"),
        operation_id,
    )
    .with_remediation(
        "Check that the target path exists and is writable, then run the command again.",
    );
    if json {
        println!("{}", envelope.to_json());
    }
    eprintln!(
        "error: could not write {} (code {}, operation {operation_id})",
        path.display(),
        envelope.code
    );
    tracing::error!(operation_id, "export failed");
    ExitCode::from(EXIT_EXPORT)
}

fn map_state(analysis: &pipeline::Analysis) -> &'static str {
    match (&analysis.map_input, &analysis.map_evidence) {
        (Some(_), Some(_)) => "parsed with the gnu_ld adapter",
        (Some(_), None) => "supplied but not parsed",
        (None, _) => "not provided",
    }
}

/// Map an error onto the frozen exit-code table. Everything the CLI can hit while reading a build is
/// a parse/import failure, which is 3. Usage mistakes are rejected by the argument parser and exit 2
/// beforehand; the export path reports 6 itself. There is no gate path yet to produce 4 or 5.
#[must_use]
pub fn exit_code_for(_err: &ArtifactError) -> u8 {
    EXIT_PARSE_IMPORT
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Parse the command line and translate a clap failure into the frozen exit code.
///
/// Kept explicit so the usage-error path is testable without spawning a process, and so
/// `main` exits with the code this table promises rather than one chosen by the argument parser.
/// The return value is the numeric code rather than an `ExitCode`, whose value is not readable
/// on stable Rust.
fn parse_from<I, T>(args: I) -> Result<Cli, u8>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => Ok(cli),
        Err(clap_err) => {
            let code = if clap_err.use_stderr() {
                EXIT_USAGE
            } else {
                EXIT_OK
            };
            // Help and version print to stdout; everything else to stderr.
            let _ = clap_err.print();
            Err(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn usage_errors_exit_with_the_frozen_usage_code() {
        // No subcommand at all.
        let err = parse_from(Vec::<OsString>::new()).expect_err("no arguments is a usage error");
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn unregistered_future_commands_are_still_not_accepted() {
        for command in ["gate", "release", "watch", "doctor"] {
            let args: Vec<OsString> = ["fwsight", command].iter().map(OsString::from).collect();
            assert!(
                parse_from(args.clone()).is_err(),
                "`{command}` is not implemented and must not be registered"
            );
        }
    }

    #[test]
    fn diff_is_now_a_registered_command() {
        // P2 authorizes Compare. This replaces the P0 test that asserted `diff` must be rejected,
        // so the change is recorded rather than silent.
        let args: Vec<OsString> = ["fwsight", "diff", "a.elf", "b.elf"]
            .iter()
            .map(OsString::from)
            .collect();
        let cli = parse_from(args).expect("diff parses");
        let Commands::Diff(diff) = cli.command else {
            panic!("diff must route to DiffArgs");
        };
        assert_eq!(diff.old, PathBuf::from("a.elf"));
        assert_eq!(diff.new, PathBuf::from("b.elf"));
        assert!(diff.old_map.is_none() && diff.new_map.is_none());
        assert!(!diff.json);
        assert!(diff.html.is_none());
    }

    #[test]
    fn diff_options_parse_into_their_named_slots() {
        let args: Vec<OsString> = [
            "fwsight",
            "diff",
            "a.elf",
            "b.elf",
            "--old-map",
            "a.map",
            "--new-map",
            "b.map",
            "--json",
            "--html",
            "out.html",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        let cli = parse_from(args).expect("the full option set parses");
        let Commands::Diff(diff) = cli.command else {
            panic!("diff must route to DiffArgs");
        };
        assert_eq!(diff.old_map, Some(PathBuf::from("a.map")));
        assert_eq!(diff.new_map, Some(PathBuf::from("b.map")));
        assert!(diff.json);
        assert_eq!(diff.html, Some(PathBuf::from("out.html")));
    }

    #[test]
    fn a_diff_without_both_paths_is_a_usage_error() {
        let args: Vec<OsString> = ["fwsight", "diff", "a.elf"]
            .iter()
            .map(OsString::from)
            .collect();
        assert_eq!(
            parse_from(args).expect_err("one path is not a comparison"),
            EXIT_USAGE
        );
    }

    #[test]
    fn analyze_and_diff_are_the_two_registered_routes() {
        let args: Vec<OsString> = ["fwsight", "analyze", "x.elf"]
            .iter()
            .map(OsString::from)
            .collect();
        let cli = parse_from(args.clone()).expect("analyze parses");
        assert!(matches!(cli.command, Commands::Analyze(_)));

        let args: Vec<OsString> = ["fwsight", "diff", "a.elf", "b.elf"]
            .iter()
            .map(OsString::from)
            .collect();
        let cli = parse_from(args).expect("diff parses");
        assert!(matches!(cli.command, Commands::Diff(_)));
    }

    #[test]
    fn every_user_facing_error_maps_to_exit_three() {
        let cases = [
            ArtifactError::InputNotFound {
                path: "x".to_owned(),
            },
            ArtifactError::ArtifactTooLarge {
                path: "x".to_owned(),
                byte_size: 10,
                limit_bytes: 1,
            },
            ArtifactError::UnsupportedFormat {
                detected: "pe".to_owned(),
            },
            ArtifactError::InvalidElf {
                detail: "truncated".to_owned(),
            },
            ArtifactError::MapUnsupported {
                detected: "iar".to_owned(),
            },
            ArtifactError::MapParseFailed {
                line: 3,
                detail: "bad".to_owned(),
            },
            ArtifactError::InternalBug {
                detail: "invariant".to_owned(),
            },
        ];

        for case in cases {
            assert_eq!(
                exit_code_for(&case),
                EXIT_PARSE_IMPORT,
                "{case:?} must exit 3"
            );
            assert!(
                !case.user_message().is_empty(),
                "{case:?} must carry a human message"
            );
        }
    }

    #[test]
    fn exit_code_one_is_never_produced() {
        // `1` is not part of the frozen table, so nothing may return it.
        let all = [EXIT_OK, EXIT_USAGE, EXIT_PARSE_IMPORT, EXIT_EXPORT];
        assert!(!all.contains(&1));
        assert_eq!(
            all,
            [0, 2, 3, 6],
            "the P2 exit-code surface is exactly 0, 2, 3 and 6; 4 and 5 belong to the Gate phase \
             and stay unreachable until it exists"
        );
    }
}
