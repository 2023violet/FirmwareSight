//! `fwsight` — the FirmwareSight command line.
//!
//! P0 implements `analyze` only. `diff`, `gate` and `release prepare` are defined in
//! `04_TECH/07_CLI_SPEC.md` but are *not registered here*, so a user cannot mistake them for
//! working features. Exit codes 4 (gate review) and 5 (gate block) and 6 (export error) are
//! likewise unreachable until the Gate and Export phases exist; nothing pretends otherwise.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use firmwaresight_artifact::ArtifactError;
use firmwaresight_artifact::intake::GuardConfig;
use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_report::{AnalyzeResultDto, render};

/// Exit codes frozen by 04_TECH/07_CLI_SPEC.md. `1` is intentionally not defined.
const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 2;
const EXIT_PARSE_IMPORT: u8 = 3;

#[derive(Debug, Parser)]
#[command(
    name = "fwsight",
    version,
    about = "Know exactly what ships.",
    long_about = "FirmwareSight P0 technical slice.\n\nOnly `analyze` is implemented; Compare, \
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
                let map_state = match (&analysis.map_input, &analysis.map_evidence) {
                    (Some(_), Some(_)) => "parsed with the gnu_ld adapter",
                    (Some(_), None) => "supplied but not parsed",
                    (None, _) => "not provided",
                };
                eprintln!("diagnostics: map {map_state}");
                eprintln!(
                    "diagnostics: hash {} ms, read {} ms, operation {operation_id}",
                    analysis.input.hash_ms, analysis.input.read_ms
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

/// Map an error onto the frozen exit-code table. Every error the CLI can surface in P0 is a
/// parse/import failure, which is 3; there is no gate or export path yet to produce 4, 5 or 6.
#[must_use]
pub fn exit_code_for(_err: &ArtifactError) -> u8 {
    // There is no gate or export path in P0, so every failure that reaches here is import
    // class. Usage mistakes are rejected by the argument parser and exit 2 beforehand.
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
    fn unregistered_future_commands_are_not_accepted() {
        let diff: Vec<OsString> = ["fwsight", "diff", "a", "b"]
            .iter()
            .map(OsString::from)
            .collect();
        let err = parse_from(diff.clone()).expect_err("diff must not be accepted during P0");
        assert_eq!(err, EXIT_USAGE);

        for command in ["gate", "release", "watch", "doctor"] {
            let args: Vec<OsString> = ["fwsight", command].iter().map(OsString::from).collect();
            assert!(
                parse_from(args.clone()).is_err(),
                "`{command}` is not implemented and must not be registered"
            );
        }
    }

    #[test]
    fn analyze_is_the_only_route_that_parses() {
        let args: Vec<OsString> = ["fwsight", "analyze", "x.elf"]
            .iter()
            .map(OsString::from)
            .collect();
        let cli = parse_from(args.clone()).expect("analyze parses");
        assert!(matches!(cli.command, Commands::Analyze(_)));
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
        let all = [EXIT_OK, EXIT_USAGE, EXIT_PARSE_IMPORT];
        assert!(!all.contains(&1));
        assert_eq!(
            [EXIT_OK, EXIT_USAGE, EXIT_PARSE_IMPORT],
            [0, 2, 3],
            "the P0 exit-code surface is exactly 0, 2 and 3"
        );
    }
}
