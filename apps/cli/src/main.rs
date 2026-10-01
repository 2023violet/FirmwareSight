//! `fwsight` — the FirmwareSight command line.
//!
//! P3 implemented `analyze`, `diff` and `gate`; P4 added `release prepare`, which assembles the portable
//! Release Bundle. `project doctor` is still defined in `04_TECH/07_CLI_SPEC.md` and still *not registered
//! here*, so a user cannot mistake it for a working feature. Exit codes 4 (gate review) and 5 (gate block)
//! became reachable with `gate`; code 6 became reachable with the P2 export, and is what a bundle that could
//! not be written or could not be verified reports.
//!
//! `gate` is the whole CLI Gate path in one sentence: it reads `firmwaresight.toml` from the project
//! directory, analyzes the artifact and an optional baseline in the same process, asks the workspace
//! Git what it knows, and prints Core's answer. It creates no database and stores nothing — a Gate
//! record becomes reviewable when the desktop persists it, and a command line that quietly left a
//! project database behind would be a side effect nobody asked for (prompt §38).
//!
//! `release prepare` reads the same four sources and then publishes through the same staging and swap engine
//! the desktop exports with (§35: one bundle semantics, not two). It has no review history to draw on, so a
//! Gate aggregating to REVIEW or BLOCK ends the run at 4 or 5 instead of bundling around a finding nobody
//! accepted: headless releases are the clean-pass ones, and accepting a review stays a desktop audit action
//! (§33). Like `gate`, it writes no database.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use firmwaresight_artifact::ArtifactError;
use firmwaresight_artifact::intake::GuardConfig;
use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::diff::{DiffError, DiffSnapshotInput, compare};
use firmwaresight_core::domain::gate::{EffectiveSeverity, GateGrowthFacts};
use firmwaresight_core::domain::release::ReleaseError;
use firmwaresight_project::evidence::{SnapshotFacts, growth_facts, observe_release_notes};
use firmwaresight_project::{
    BundleError, BundleRequest, GateRunRequest, GitObservation, GitProbe, LoadedProject,
    ProjectError, build_context, prepare, run_id,
};
use firmwaresight_report::{
    AnalyzeResultDto, DiffResultDto, GateResultsDto, diff_render, gate_render, render,
};

/// Exit codes frozen by 04_TECH/07_CLI_SPEC.md. `1` is intentionally not defined.
const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 2;
const EXIT_PARSE_IMPORT: u8 = 3;
/// The Gate aggregate is REVIEW: the release may go out once a named person has accepted each review.
const EXIT_GATE_REVIEW: u8 = 4;
/// The Gate aggregate is BLOCK: something failed, or a budget lost the evidence a hard verdict needs.
const EXIT_GATE_BLOCK: u8 = 5;
/// The requested export or bundle could not be written, or a bundle that was written did not verify against
/// its own bytes. §34 puts every destination and source-file problem behind this one code.
const EXIT_EXPORT: u8 = 6;

#[derive(Debug, Parser)]
#[command(
    name = "fwsight",
    version,
    about = "Know exactly what ships.",
    long_about = "FirmwareSight P0/P1/P2/P3/P4 technical slice.\n\n`analyze`, `diff`, `gate` and \
                  `release prepare` are implemented; `project doctor` is a later phase and is not \
                  available."
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
    /// Evaluate the release Gate for one artifact against a project's policy.
    Gate(GateArgs),
    /// Publish the portable Release Bundle for a Gate that passed.
    Release(ReleaseArgs),
}

#[derive(Debug, Subcommand)]
enum ReleaseCommands {
    /// Assemble, verify and write one Release Bundle.
    Prepare(ReleasePrepareArgs),
}

/// `fwsight release prepare` (§33).
///
/// The four artifact arguments are the frozen spec's `release prepare --project . --out dist/release`
/// resolved with the paths the project config deliberately does not name: `firmwaresight.toml` describes a
/// *policy*, not which build is being released, and only the release owner knows which bytes are shipping.
/// This is an implementation clarification of that example, not a new product verb.
#[derive(Debug, Parser)]
struct ReleasePrepareArgs {
    /// Directory holding `firmwaresight.toml`. Its policy is what the Gate was evaluated against.
    #[arg(long, value_name = "DIR", default_value = ".")]
    project: PathBuf,

    /// The artifact this release ships.
    #[arg(long, value_name = "FILE")]
    artifact: PathBuf,

    /// GNU ld MAP file for the shipped artifact.
    #[arg(long, value_name = "FILE")]
    map: Option<PathBuf>,

    /// The baseline build the comparison ships with.
    #[arg(long, value_name = "FILE")]
    baseline: Option<PathBuf>,

    /// GNU ld MAP file for the baseline artifact.
    #[arg(long, value_name = "FILE")]
    baseline_map: Option<PathBuf>,

    /// The folder the bundle directory is created in.
    #[arg(long, value_name = "DIR")]
    out: PathBuf,

    /// Authorize replacing an existing FirmwareSight bundle in that folder.
    #[arg(long)]
    force: bool,

    /// Emit exactly the `release-manifest.json` document the bundle carries on stdout.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Parser)]
struct ReleaseArgs {
    #[command(subcommand)]
    command: ReleaseCommands,
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

#[derive(Debug, Parser)]
struct GateArgs {
    /// Directory holding `firmwaresight.toml`. Its policy is what the Gate evaluates against.
    #[arg(long, value_name = "DIR", default_value = ".")]
    project: PathBuf,

    /// The artifact this release ships.
    #[arg(long, value_name = "FILE")]
    artifact: PathBuf,

    /// GNU ld MAP file for the shipped artifact.
    #[arg(long, value_name = "FILE")]
    map: Option<PathBuf>,

    /// The baseline build to measure growth against, when `[diff]` names a threshold.
    #[arg(long, value_name = "FILE")]
    baseline: Option<PathBuf>,

    /// GNU ld MAP file for the baseline artifact.
    #[arg(long, value_name = "FILE")]
    baseline_map: Option<PathBuf>,

    /// Emit exactly one gate-results JSON document on stdout.
    #[arg(long)]
    json: bool,
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
        Commands::Gate(args) => run_gate(args, &operation_id),
        Commands::Release(args) => match args.command {
            ReleaseCommands::Prepare(prepare) => run_release_prepare(prepare, &operation_id),
        },
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

/// Evaluate the release Gate for one artifact against one project's policy.
///
/// The four fact sources are the ones prompt §38 names: `firmwaresight.toml` in the project directory,
/// the artifact (and optional baseline) analyzed in this process, the workspace Git consulted read-only,
/// and the project-relative Release Notes file. Core decides what each of them means; this function only
/// assembles facts and prints the answer, and it stores nothing.
fn run_gate(args: GateArgs, operation_id: &str) -> ExitCode {
    let project = match LoadedProject::load(&args.project) {
        Ok(loaded) => loaded,
        Err(err) => return report_config_error(&err, args.json, operation_id),
    };

    let target = match analyze_side(
        &args.artifact,
        args.map.clone(),
        "artifact",
        args.json,
        operation_id,
    ) {
        Ok(analysis) => analysis,
        Err(code) => return ExitCode::from(code),
    };

    // A baseline is compared with the same Core diff `fwsight diff` runs, so the growth a Gate blocks
    // on and the growth a release owner reads in Compare are one calculation, not two (§23).
    let growth = match &args.baseline {
        None => GateGrowthFacts::without_baseline(),
        Some(path) => {
            let base = match analyze_side(
                path,
                args.baseline_map.clone(),
                "baseline",
                args.json,
                operation_id,
            ) {
                Ok(analysis) => analysis,
                Err(code) => return ExitCode::from(code),
            };
            let base_input = DiffSnapshotInput::from_snapshot(&base.snapshot);
            let target_input = DiffSnapshotInput::from_snapshot(&target.snapshot);
            match compare(&base_input, &target_input) {
                Ok(diff) => growth_facts(&diff),
                Err(err) => return report_diff_error(&err, args.json, operation_id),
            }
        }
    };

    let git = GitProbe::system().observe(&project.root);
    let release_notes = project
        .policy
        .require_release_notes
        .then(|| observe_release_notes(&project.root, &project.policy.release_notes_path));
    let facts = SnapshotFacts::from_snapshot(&target.snapshot);
    let context = build_context(&GateRunRequest {
        target: &facts,
        growth,
        git: &git,
        policy: project.policy.clone(),
        release_notes,
    });

    let run = run_id(&context);
    let evaluation = context.evaluate(&run);
    let dto = GateResultsDto::from_evaluation(
        &evaluation,
        &project.policy_sha256,
        project.config.schema_version,
    );

    if args.json {
        // stdout carries nothing but the Gate document.
        print!("{}", gate_render::render_json(&dto));
    } else {
        print!(
            "{}",
            gate_render::render_human(&evaluation, &project.policy_sha256)
        );
    }
    // These four lines are diagnostics in both modes: they say what the run read and that the CLI
    // stored nothing, which is exactly the stderr half of the `--json` contract (§39).
    eprintln!("diagnostics: map {}", map_state(&target));
    if let Some(baseline) = &args.baseline {
        eprintln!("diagnostics: baseline {}", file_name(baseline));
    }
    eprintln!("diagnostics: git {}", git_state(&git));
    eprintln!(
        "diagnostics: this run is not stored; the desktop Release page writes the reviewable record"
    );
    for warning in &project.warnings {
        eprintln!("warning: {warning}");
    }
    eprintln!("diagnostics: operation {operation_id}");

    ExitCode::from(gate_exit_code(evaluation.overall_effective_severity))
}

/// Which process code a verdict produces. `UNKNOWN` never arrives as its own answer here: Core carried
/// the rule's disposition into the aggregate's effective severity, so a gap mapped to review exits 4
/// and a gap mapped to block exits 5 (prompt §39).
#[must_use]
pub const fn gate_exit_code(severity: EffectiveSeverity) -> u8 {
    match severity {
        EffectiveSeverity::Pass => EXIT_OK,
        EffectiveSeverity::Review => EXIT_GATE_REVIEW,
        EffectiveSeverity::Block => EXIT_GATE_BLOCK,
    }
}

/// Publish one Release Bundle (§33) and report which code the run earns (§34).
///
/// The Gate is run by the same engine the desktop exports through — `prepare` recomputes the run from the
/// project, Git and artifact facts and refuses anything that does not aggregate to `PASS` — so the command
/// line neither re-derives a verdict nor keeps a second copy of the rule that requires one (§35). It has no
/// review history and prompts for nothing, so a REVIEW here means a finding nobody accepted yet, which ends
/// the run at 4 with nothing written rather than bundling around it (§33).
fn run_release_prepare(args: ReleasePrepareArgs, operation_id: &str) -> ExitCode {
    let project = match LoadedProject::load(&args.project) {
        Ok(loaded) => loaded,
        Err(err) => return report_config_error(&err, args.json, operation_id),
    };

    let target = match analyze_side(
        &args.artifact,
        args.map.clone(),
        "artifact",
        args.json,
        operation_id,
    ) {
        Ok(analysis) => analysis,
        Err(code) => return ExitCode::from(code),
    };

    // One comparison, shipped as `diff.json` and read by the Gate's growth rule from the same bytes: the
    // delta a release owner counts in the bundle is the delta that was scored (§23).
    let comparison = match &args.baseline {
        None => None,
        Some(path) => {
            let base = match analyze_side(
                path,
                args.baseline_map.clone(),
                "baseline",
                args.json,
                operation_id,
            ) {
                Ok(analysis) => analysis,
                Err(code) => return ExitCode::from(code),
            };
            let base_input = DiffSnapshotInput::from_snapshot(&base.snapshot);
            let target_input = DiffSnapshotInput::from_snapshot(&target.snapshot);
            match compare(&base_input, &target_input) {
                Ok(diff) => Some(diff),
                Err(err) => return report_diff_error(&err, args.json, operation_id),
            }
        }
    };

    let git = GitProbe::system().observe(&project.root);
    // `acceptances` is empty and `selected_run_id` is None because the command line holds neither: review
    // acceptance is a Desktop audit action stored against a recorded run (§33).
    let request = BundleRequest {
        project: &project,
        git: &git,
        snapshot: &target.snapshot,
        comparison: comparison.as_ref(),
        selected_run_id: None,
        acceptances: &[],
        fwsight_version: pipeline::FWSIGHT_VERSION,
    };

    let plan = match prepare(&request) {
        Ok(plan) => plan,
        Err(error) => {
            return report_bundle_error(&error, release_exit_code(&error), args.json, operation_id);
        }
    };
    // §28 re-checks every input against what the preview was built from, so the second `publish` argument is
    // not a formality: a file that moved between the two calls stops the write.
    let outcome = match plan.publish(&request, &args.out, args.force) {
        Ok(outcome) => outcome,
        Err(error) => {
            return report_bundle_error(&error, release_exit_code(&error), args.json, operation_id);
        }
    };

    if args.json {
        // The stdout copy is the bundle copy: one string, read from the plan that wrote those bytes (§34).
        print!("{}", plan.manifest_document());
    } else {
        print!(
            "Release bundle written\n\
             \x20 directory   {}\n\
             \x20 release     {}\n\
             \x20 version     {}\n\
             \x20 gate run    {}\n\
             \x20 files       {} ({} bytes)\n\
             \x20 artifacts   {}\n\
             \x20 comparison  {}\n\
             \x20 notes       {}\n",
            outcome.directory_name,
            outcome.release_id,
            outcome.release_version,
            outcome.gate_run_id,
            outcome.verification.file_count,
            outcome.verification.total_bytes,
            outcome.verification.artifact_count,
            yes_if(
                outcome.verification.carries_comparison,
                "shipped",
                "not shipped"
            ),
            yes_if(
                outcome.verification.carries_release_notes,
                "shipped",
                "not shipped"
            ),
        );
        if outcome.replaced {
            eprintln!("diagnostics: the recognized bundle that was there has been replaced");
        }
    }
    eprintln!("diagnostics: map {}", map_state(&target));
    if let Some(baseline) = &args.baseline {
        eprintln!("diagnostics: baseline {}", file_name(baseline));
    }
    eprintln!("diagnostics: git {}", git_state(&git));
    eprintln!("diagnostics: integrity verified against the bundle's own SHA256SUMS and manifest");
    eprintln!(
        "diagnostics: this run is not stored; the desktop Release page writes the release record"
    );
    for warning in &project.warnings {
        eprintln!("warning: {warning}");
    }
    eprintln!("diagnostics: operation {operation_id}");
    ExitCode::from(EXIT_OK)
}

/// One of two words, for the two lines of the human summary that report a conditional file.
#[must_use]
const fn yes_if(held: bool, present: &'static str, absent: &'static str) -> &'static str {
    if held { present } else { absent }
}

/// Which of §34's codes one refusal earns.
///
/// Core's aggregate decides 4 against 5, and it does so in the refusal itself: `GateNotReady` carries the
/// effective severity the aggregate produced, so the CLI reads the disposition rather than computing a second
/// one. A version that cannot be resolved is the §34 "version resolution error" and exits 2; everything about
/// the bundle — its sources, its destination, its own verification — is 6.
#[must_use]
fn release_exit_code(error: &BundleError) -> u8 {
    match error {
        BundleError::Release(ReleaseError::GateNotReady { disposition }) => {
            gate_exit_code(*disposition)
        }
        BundleError::Release(ReleaseError::VersionUnavailable { .. }) => EXIT_USAGE,
        _ => EXIT_EXPORT,
    }
}

/// Report a bundle refusal with the engine's own code and remediation, rather than a CLI re-wording of a
/// decision the engine made. Nothing was written, and the message carries no host path: every name in a
/// `BundleError` is bundle-relative or a display name (§34).
fn report_bundle_error(error: &BundleError, code: u8, json: bool, operation_id: &str) -> ExitCode {
    let envelope = render::ErrorEnvelope::new(error.code(), error.to_string(), operation_id)
        .with_remediation(error.remediation());
    if json {
        println!("{}", envelope.to_json());
    }
    eprintln!(
        "error: {} (code {}, operation {operation_id})",
        envelope.message, envelope.code
    );
    tracing::error!(operation_id, code = error.code(), "release bundle refused");
    ExitCode::from(code)
}

/// A config problem is a usage error (exit 2), never a Gate verdict: there is no run to score until the
/// policy can be read.
fn report_config_error(err: &ProjectError, json: bool, operation_id: &str) -> ExitCode {
    let envelope =
        render::ErrorEnvelope::new(err.code(), err.to_string(), operation_id).with_remediation(
            "Correct `firmwaresight.toml` in the project directory. FirmwareSight refuses a policy it \
             cannot read fully rather than filling the gap with a default.",
        );
    if json {
        println!("{}", envelope.to_json());
    }
    eprintln!(
        "error: {} (code {}, operation {operation_id})",
        envelope.message, envelope.code
    );
    tracing::error!(operation_id, code = err.code(), "project config rejected");
    ExitCode::from(EXIT_USAGE)
}

/// One line about what the workspace Git said. It describes the *directory the release owner pointed
/// at*, and never claims the artifact was built from it (`04_TECH/24`, prompt §50).
fn git_state(git: &GitObservation) -> String {
    if !git.facts.available {
        return "not consulted".to_owned();
    }
    let dirty = match git.facts.dirty.value() {
        Some(true) => "dirty",
        Some(false) => "clean",
        None => "state unknown",
    };
    let tag = match git.facts.exact_tag.value() {
        Some(tag) => format!("tag {tag}"),
        None => "no exact tag".to_owned(),
    };
    let ambiguous = if git.ambiguous_tags() {
        " (several tags on HEAD)"
    } else {
        ""
    };
    format!("{dirty}, {tag}{ambiguous}")
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
/// a parse/import failure, which is 3. Usage mistakes — including a `firmwaresight.toml` this build
/// cannot read — are rejected before any verdict exists and exit 2; the export path reports 6 itself;
/// a completed Gate reports 4 or 5 from its own aggregate through `gate_exit_code`.
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
        for command in ["watch", "doctor"] {
            let args: Vec<OsString> = ["fwsight", command].iter().map(OsString::from).collect();
            assert!(
                parse_from(args.clone()).is_err(),
                "`{command}` is not implemented and must not be registered"
            );
        }
    }

    /// The one verb of the `release` family, unwrapped. A `let … else` would be an irrefutable pattern here,
    /// because §33 froze exactly one verb, so the extraction is a match rather than a panic branch nobody can
    /// reach — the closed family is asserted by the process-level test instead.
    fn prepared(cli: Cli) -> ReleasePrepareArgs {
        let Commands::Release(release) = cli.command else {
            panic!("release must route to its own subcommand family");
        };
        match release.command {
            ReleaseCommands::Prepare(prepare) => prepare,
        }
    }

    #[test]
    fn release_is_now_a_registered_command_family() {
        // P4 authorizes the Release Bundle. This replaces the P0 test that asserted `release` must be
        // rejected, so the change is recorded rather than silent. `project doctor` stays unregistered above.
        let args: Vec<OsString> = ["fwsight", "release", "prepare"]
            .iter()
            .map(OsString::from)
            .collect();
        // The two required paths are missing, which is a usage error — but the *route* exists, so clap
        // complains about the arguments rather than about an unknown subcommand.
        let err = parse_from(args).expect_err("prepare needs an artifact and a destination");
        assert_eq!(err, EXIT_USAGE);

        let args: Vec<OsString> = [
            "fwsight",
            "release",
            "prepare",
            "--artifact",
            "firmware.elf",
            "--out",
            "dist/release",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        let prepare = prepared(parse_from(args).expect("release prepare parses"));
        assert_eq!(
            prepare.project,
            PathBuf::from("."),
            "--project defaults to here"
        );
        assert_eq!(prepare.artifact, PathBuf::from("firmware.elf"));
        assert_eq!(prepare.out, PathBuf::from("dist/release"));
        assert!(prepare.map.is_none());
        assert!(prepare.baseline.is_none() && prepare.baseline_map.is_none());
        assert!(!prepare.force, "overwriting is never the default (§30)");
        assert!(!prepare.json);
    }

    #[test]
    fn release_prepare_options_parse_into_their_named_slots() {
        let args: Vec<OsString> = [
            "fwsight",
            "release",
            "prepare",
            "--project",
            "release/brake-node",
            "--artifact",
            "out/firmware.elf",
            "--map",
            "out/firmware.map",
            "--baseline",
            "out/previous.elf",
            "--baseline-map",
            "out/previous.map",
            "--out",
            "dist/release",
            "--force",
            "--json",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        let prepare = prepared(parse_from(args).expect("the full release option set parses"));
        assert_eq!(prepare.project, PathBuf::from("release/brake-node"));
        assert_eq!(prepare.artifact, PathBuf::from("out/firmware.elf"));
        assert_eq!(prepare.map, Some(PathBuf::from("out/firmware.map")));
        assert_eq!(prepare.baseline, Some(PathBuf::from("out/previous.elf")));
        assert_eq!(
            prepare.baseline_map,
            Some(PathBuf::from("out/previous.map"))
        );
        assert_eq!(prepare.out, PathBuf::from("dist/release"));
        assert!(prepare.force);
        assert!(prepare.json);
    }

    #[test]
    fn a_release_without_an_artifact_or_a_destination_is_a_usage_error() {
        // Neither of these has any answer to bundle. Writing a folder of documents about an unspecified
        // build would read as a release of something the release owner never chose (§33).
        let without_artifact: Vec<OsString> = ["fwsight", "release", "prepare", "--out", "dist"]
            .iter()
            .map(OsString::from)
            .collect();
        assert_eq!(
            parse_from(without_artifact).expect_err("no artifact"),
            EXIT_USAGE
        );

        let without_out: Vec<OsString> = [
            "fwsight",
            "release",
            "prepare",
            "--artifact",
            "firmware.elf",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        assert_eq!(
            parse_from(without_out).expect_err("no destination"),
            EXIT_USAGE
        );
    }

    #[test]
    fn a_refusal_maps_to_the_code_its_own_words_describe() {
        use firmwaresight_core::domain::gate::EffectiveSeverity::{Block, Review};
        // §34's split: the Gate's verdict is 4 or 5, a version that cannot be resolved is a config error at
        // 2, and anything about the bundle itself is 6.
        for (error, expected) in [
            (
                BundleError::Release(ReleaseError::GateNotReady {
                    disposition: Review,
                }),
                EXIT_GATE_REVIEW,
            ),
            (
                BundleError::Release(ReleaseError::GateNotReady { disposition: Block }),
                EXIT_GATE_BLOCK,
            ),
            (
                BundleError::Release(ReleaseError::VersionUnavailable {
                    reason: "no tag".to_owned(),
                }),
                EXIT_USAGE,
            ),
            (
                BundleError::DestinationExists {
                    display: "motor-controller-1.4.2-release-abc".to_owned(),
                },
                EXIT_EXPORT,
            ),
            (
                BundleError::UnsafeDestination {
                    display: "notes".to_owned(),
                    detail: "it is not a bundle".to_owned(),
                },
                EXIT_EXPORT,
            ),
            (
                BundleError::Release(ReleaseError::NoReleaseArtifacts),
                EXIT_EXPORT,
            ),
        ] {
            assert_eq!(release_exit_code(&error), expected, "{error}");
        }
    }

    #[test]
    fn gate_is_now_a_registered_command() {
        // P3 authorizes the Release Gate. This replaces the P0 test that asserted `gate` must be
        // rejected, so the change is recorded rather than silent; `release` stays rejected above.
        let args: Vec<OsString> = ["fwsight", "gate", "--artifact", "firmware.elf"]
            .iter()
            .map(OsString::from)
            .collect();
        let cli = parse_from(args).expect("gate parses");
        let Commands::Gate(gate) = cli.command else {
            panic!("gate must route to GateArgs");
        };
        assert_eq!(
            gate.project,
            PathBuf::from("."),
            "--project defaults to here"
        );
        assert_eq!(gate.artifact, PathBuf::from("firmware.elf"));
        assert!(gate.map.is_none());
        assert!(gate.baseline.is_none() && gate.baseline_map.is_none());
        assert!(!gate.json);
    }

    #[test]
    fn gate_options_parse_into_their_named_slots() {
        let args: Vec<OsString> = [
            "fwsight",
            "gate",
            "--project",
            "release/brake-node",
            "--artifact",
            "out/firmware.elf",
            "--map",
            "out/firmware.map",
            "--baseline",
            "out/previous.elf",
            "--baseline-map",
            "out/previous.map",
            "--json",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        let cli = parse_from(args).expect("the full gate option set parses");
        let Commands::Gate(gate) = cli.command else {
            panic!("gate must route to GateArgs");
        };
        assert_eq!(gate.project, PathBuf::from("release/brake-node"));
        assert_eq!(gate.artifact, PathBuf::from("out/firmware.elf"));
        assert_eq!(gate.map, Some(PathBuf::from("out/firmware.map")));
        assert_eq!(gate.baseline, Some(PathBuf::from("out/previous.elf")));
        assert_eq!(gate.baseline_map, Some(PathBuf::from("out/previous.map")));
        assert!(gate.json);
    }

    #[test]
    fn a_gate_without_an_artifact_is_a_usage_error() {
        // A Gate with no artifact has nothing to score. Answering with an all-N/A verdict would read
        // as a release that passed ten rules.
        let args: Vec<OsString> = ["fwsight", "gate", "--project", "."]
            .iter()
            .map(OsString::from)
            .collect();
        assert_eq!(
            parse_from(args).expect_err("no artifact is no gate"),
            EXIT_USAGE
        );
    }

    #[test]
    fn every_aggregate_maps_to_its_own_exit_code_and_none_of_them_is_one() {
        use firmwaresight_core::domain::gate::EffectiveSeverity;
        assert_eq!(gate_exit_code(EffectiveSeverity::Pass), EXIT_OK);
        assert_eq!(gate_exit_code(EffectiveSeverity::Review), EXIT_GATE_REVIEW);
        assert_eq!(gate_exit_code(EffectiveSeverity::Block), EXIT_GATE_BLOCK);
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
    fn analyze_diff_and_gate_are_the_three_registered_routes() {
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

        let args: Vec<OsString> = ["fwsight", "gate", "--artifact", "x.elf"]
            .iter()
            .map(OsString::from)
            .collect();
        let cli = parse_from(args).expect("gate parses");
        assert!(matches!(cli.command, Commands::Gate(_)));
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
        let all = [
            EXIT_OK,
            EXIT_USAGE,
            EXIT_PARSE_IMPORT,
            EXIT_GATE_REVIEW,
            EXIT_GATE_BLOCK,
            EXIT_EXPORT,
        ];
        assert!(!all.contains(&1));
        assert_eq!(
            all,
            [0, 2, 3, 4, 5, 6],
            "the P3 exit-code surface is 0, 2, 3, 4, 5 and 6; 4 and 5 arrived with the Gate, and 1 \
             stays undefined"
        );
    }
}
