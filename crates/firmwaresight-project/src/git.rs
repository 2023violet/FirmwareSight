//! The read-only system Git adapter.
//!
//! `04_TECH/22_GIT_PROVENANCE_ADAPTER.md` decided the mechanism — the installed `git` CLI, not a
//! library parser — and this module is the only place that decision becomes code. Everything here is
//! read-only: `rev-parse`, `tag --points-at` and `status --porcelain`. No `fetch`, no `checkout`, no
//! `reset`, no mutation, and no network.
//!
//! Three properties are structural rather than promised:
//! - the executable and its arguments reach the OS as an argv vector, never through a shell;
//! - every command carries a timeout and an output cap, so a hung or enormous repository cannot hang a
//!   Gate that a release owner is waiting on in a terminal;
//! - the repository root is read to aim the commands and then dropped. It is a host path, and a host
//!   path must not reach a Gate run record, a portable document, a UI surface or a run fingerprint
//!   (`AGENTS.md` 7, prompt §26, §28).

use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use firmwaresight_core::domain::gate::GateGitFacts;
use firmwaresight_core::domain::identity::Fact;

/// How long one Git command may run before it is killed. A Gate must answer, not wait.
pub const GIT_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
/// How often a running child is polled: small enough that a fast command is not delayed, large enough
/// that polling is not a busy loop.
pub const GIT_POLL_INTERVAL: Duration = Duration::from_millis(20);
/// How long a reader thread is given to finish after the child exits or is killed.
pub const GIT_DRAIN_TIMEOUT: Duration = Duration::from_millis(250);
/// Per-command output cap. `git status --porcelain` in a monorepo can be far larger than a Gate needs.
pub const GIT_MAX_OUTPUT_BYTES: usize = 1024 * 1024;

/// What one bounded command produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    /// The executable could not be started at all — usually Git is not installed.
    Unavailable,
    /// The command ran and exited 0 with output inside the cap.
    Success { output: String },
    /// The command ran and exited non-zero. Its output is not read: a failed Git command's message can
    /// name the repository path, and no part of it may become a fact.
    Failed { code: Option<i32> },
    /// The command exceeded its timeout and was killed.
    TimedOut,
    /// Output passed the cap, so the fact cannot be trusted even though the command succeeded.
    Overlong,
}

impl RunOutcome {
    /// The stdout of a command that succeeded within its bounds.
    #[must_use]
    pub fn output(&self) -> Option<&str> {
        match self {
            Self::Success { output } => Some(output),
            _ => None,
        }
    }
}

/// A Git probe: which executable to run, and the bounds it runs under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitProbe {
    pub program: OsString,
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

impl Default for GitProbe {
    fn default() -> Self {
        Self::system()
    }
}

impl GitProbe {
    /// The installed `git`, with the shipped bounds.
    #[must_use]
    pub fn system() -> Self {
        Self {
            program: OsString::from("git"),
            timeout: GIT_COMMAND_TIMEOUT,
            max_output_bytes: GIT_MAX_OUTPUT_BYTES,
        }
    }

    /// A probe pointed at a specific executable, for tests that must not depend on `PATH`.
    #[must_use]
    pub fn with_program(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            ..Self::system()
        }
    }

    /// Read every workspace fact the Gate needs. It never fails: what cannot be learned becomes an
    /// `Unknown` fact with the reason it is unknown.
    #[must_use]
    pub fn observe(&self, root: &Path) -> GitObservation {
        let toplevel = self.git(root, &["rev-parse", "--show-toplevel"]);
        if !matches!(toplevel, RunOutcome::Success { .. }) {
            return GitObservation::unavailable(absent_reason(&toplevel, self.timeout));
        }
        let head = self.git(root, &["rev-parse", "HEAD"]);
        let tags = self.git(root, &["tag", "--points-at", "HEAD"]);
        let status = self.git(root, &["status", "--porcelain"]);
        normalize(&head, &tags, &status, self.timeout)
    }

    /// One read-only Git subcommand, aimed at `root` by `-C`, with nothing else on the command line.
    fn git(&self, root: &Path, subcommand: &[&str]) -> RunOutcome {
        let argv = argv_for(root, subcommand);
        run_bounded(&self.program, &argv, self.timeout, self.max_output_bytes)
    }
}

/// `-C <root> <subcommand…>` as an argument vector. Public because the temporary repository a test
/// builds has to run the same read-only-shaped commands through the same bounded path.
#[must_use]
pub fn argv_for(root: &Path, subcommand: &[&str]) -> Vec<OsString> {
    let mut argv: Vec<OsString> = Vec::with_capacity(subcommand.len() + 2);
    argv.push(OsString::from("-C"));
    argv.push(root.as_os_str().to_os_string());
    argv.extend(subcommand.iter().map(|arg| OsString::from(*arg)));
    argv
}

/// The workspace facts, plus every tag that points at HEAD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitObservation {
    pub facts: GateGitFacts,
    /// Sorted, so a repository carrying two tags on one commit yields the same facts on every run.
    pub exact_tags: Vec<String>,
}

impl GitObservation {
    /// An observation with no facts in it, for a probe that could not run and for a caller that has no
    /// workspace to point at. Every field is `Unknown` with the same reason, because the rule that reads
    /// this gets to say *why* nothing was learned rather than only that nothing was learned.
    #[must_use]
    pub fn unavailable(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            facts: GateGitFacts::unavailable(&reason),
            exact_tags: Vec::new(),
        }
    }

    /// More than one tag on HEAD is worth stating, because the version rule applies the pattern to one
    /// of them.
    #[must_use]
    pub fn ambiguous_tags(&self) -> bool {
        self.exact_tags.len() > 1
    }
}

/// Turn three raw command outcomes into the facts the Gate reads. Pure, so the mapping is testable
/// without a Git installation.
fn normalize(
    head: &RunOutcome,
    tags: &RunOutcome,
    status: &RunOutcome,
    timeout: Duration,
) -> GitObservation {
    let Some(raw_head) = head.output().map(str::trim) else {
        return GitObservation::unavailable(command_reason("rev-parse HEAD", head, timeout));
    };
    if !looks_like_object_id(raw_head) {
        return GitObservation::unavailable(
            "`git rev-parse HEAD` did not return a commit id, so workspace provenance is unknown",
        );
    }

    let Some(raw_tags) = tags.output() else {
        return GitObservation::unavailable(format!(
            "workspace HEAD is known but its tags are not: {}",
            command_reason("tag --points-at HEAD", tags, timeout)
        ));
    };
    let mut exact_tags: Vec<String> = raw_tags
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    exact_tags.sort();

    let dirty = match status.output() {
        // A porcelain entry per changed path is the dirty signal; empty output means clean.
        Some(output) => Fact::known(output.lines().any(|line| !line.trim().is_empty())),
        None => Fact::unknown(command_reason("status --porcelain", status, timeout)),
    };

    GitObservation {
        facts: GateGitFacts {
            available: true,
            head_commit: Fact::known(raw_head.to_owned()),
            exact_tag: match exact_tags.first() {
                Some(tag) => Fact::known(tag.clone()),
                None => Fact::unknown("no exact tag points at workspace HEAD"),
            },
            dirty,
        },
        exact_tags,
    }
}

/// The one shape `git rev-parse HEAD` must answer with: 40 or 64 lowercase hex characters.
fn looks_like_object_id(value: &str) -> bool {
    (value.len() == 40 || value.len() == 64)
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// Why the repository itself could not be consulted.
fn absent_reason(toplevel: &RunOutcome, timeout: Duration) -> String {
    match toplevel {
        RunOutcome::Unavailable => {
            "git is not installed, so workspace provenance is unavailable".to_owned()
        }
        RunOutcome::TimedOut => format!(
            "git did not answer within {}s, so workspace provenance is unavailable",
            timeout.as_secs()
        ),
        RunOutcome::Overlong => {
            "git returned more output than the Gate reads, so workspace provenance is unavailable"
                .to_owned()
        }
        RunOutcome::Failed { .. } => {
            "this directory is not a git repository, so workspace provenance is unavailable"
                .to_owned()
        }
        RunOutcome::Success { .. } => "the git repository root could not be resolved".to_owned(),
    }
}

fn command_reason(what: &str, outcome: &RunOutcome, timeout: Duration) -> String {
    match outcome {
        RunOutcome::Unavailable => "git is not installed".to_owned(),
        RunOutcome::TimedOut => {
            format!("`git {what}` did not finish within {}s", timeout.as_secs())
        }
        RunOutcome::Overlong => format!("`git {what}` produced more output than the Gate reads"),
        RunOutcome::Failed { code } => format!("`git {what}` exited {code:?}"),
        RunOutcome::Success { .. } => format!("`git {what}` returned nothing usable"),
    }
}

/// Run one program with a bounded output read and a hard timeout. No shell is involved: `argv` reaches
/// the OS as an argument vector, which is what makes a path containing a space or a `;` inert.
#[must_use]
pub fn run_bounded(
    program: &OsStr,
    argv: &[OsString],
    timeout: Duration,
    max_output_bytes: usize,
) -> RunOutcome {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(argv);
    let Ok(mut child) = command.spawn() else {
        return RunOutcome::Unavailable;
    };
    let stdout = child.stdout.take();
    // stderr is drained so a child cannot block on a full pipe; its bytes are then dropped, because a
    // Git error message can name the repository path.
    let stderr = child.stderr.take();
    let stdout_reader = spawn_drain(stdout, max_output_bytes);
    let stderr_reader = spawn_drain(stderr, max_output_bytes);

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_reader.recv_timeout(GIT_DRAIN_TIMEOUT);
                    let _ = stderr_reader.recv_timeout(GIT_DRAIN_TIMEOUT);
                    return RunOutcome::TimedOut;
                }
                std::thread::sleep(GIT_POLL_INTERVAL);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return RunOutcome::Unavailable;
            }
        }
    };

    match stdout_reader.recv_timeout(GIT_DRAIN_TIMEOUT) {
        Err(RecvTimeoutError::Timeout) => RunOutcome::Overlong,
        Err(RecvTimeoutError::Disconnected) => RunOutcome::Unavailable,
        Ok(Err(_)) => RunOutcome::Unavailable,
        Ok(Ok((bytes, truncated))) => {
            // The stderr pipe is closed by now; receiving it keeps the thread from lingering.
            let _ = stderr_reader.recv_timeout(GIT_DRAIN_TIMEOUT);
            if truncated {
                return RunOutcome::Overlong;
            }
            let output = String::from_utf8_lossy(&bytes).into_owned();
            match status.code() {
                Some(0) => RunOutcome::Success { output },
                code => RunOutcome::Failed { code },
            }
        }
    }
}

/// Read a pipe to completion on its own thread, stopping after `limit + 1` bytes so enormous output
/// cannot exhaust memory. The flag says whether the cap was passed.
fn spawn_drain(
    pipe: Option<impl Read + Send + 'static>,
    limit: usize,
) -> mpsc::Receiver<io::Result<(Vec<u8>, bool)>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let result = match pipe {
            None => Ok((Vec::new(), false)),
            Some(pipe) => read_bounded(pipe, limit),
        };
        let _ = sender.send(result);
    });
    receiver
}

/// The bounded read itself, separate so the cap is testable without starting a process.
fn read_bounded(pipe: impl Read, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut buffer = Vec::new();
    pipe.take(limit as u64 + 1).read_to_end(&mut buffer)?;
    let truncated = buffer.len() > limit;
    Ok((buffer, truncated))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
    static PING_ARGS: [&str; 3] = ["-n", "5", "127.0.0.1"];
    static SLEEP_ARGS: [&str; 1] = ["5"];

    fn success(output: &str) -> RunOutcome {
        RunOutcome::Success {
            output: output.to_owned(),
        }
    }

    fn head_outcome() -> RunOutcome {
        success(&format!("{HEAD}\n"))
    }

    #[test]
    fn a_clean_tagged_repository_yields_known_facts() {
        let observation = normalize(
            &head_outcome(),
            &success("v1.2.3\n"),
            &success(""),
            GIT_COMMAND_TIMEOUT,
        );
        assert!(observation.facts.available);
        assert_eq!(
            observation.facts.head_commit.value(),
            Some(&HEAD.to_owned())
        );
        assert_eq!(
            observation.facts.exact_tag.value(),
            Some(&"v1.2.3".to_owned())
        );
        assert_eq!(observation.facts.dirty, Fact::known(false));
        assert!(!observation.ambiguous_tags());
    }

    #[test]
    fn any_status_line_makes_the_workspace_dirty() {
        let observation = normalize(
            &head_outcome(),
            &success("v1.2.3\n"),
            &success(" M firmware.c\n"),
            GIT_COMMAND_TIMEOUT,
        );
        assert_eq!(observation.facts.dirty, Fact::known(true));
    }

    #[test]
    fn a_repository_without_a_tag_is_a_known_absence_not_a_missing_fact() {
        let observation = normalize(
            &head_outcome(),
            &success(""),
            &success(""),
            GIT_COMMAND_TIMEOUT,
        );
        assert!(observation.facts.available);
        assert!(!observation.facts.exact_tag.is_known());
        assert_eq!(
            observation.facts.exact_tag.reason_if_unknown(),
            Some("no exact tag points at workspace HEAD")
        );
    }

    #[test]
    fn several_tags_on_one_commit_are_sorted_and_reported() {
        let observation = normalize(
            &head_outcome(),
            &success("v9.9.9\nv1.0.0\n"),
            &success(""),
            GIT_COMMAND_TIMEOUT,
        );
        assert_eq!(observation.exact_tags, vec!["v1.0.0", "v9.9.9"]);
        assert!(observation.ambiguous_tags());
        assert_eq!(
            observation.facts.exact_tag.value(),
            Some(&"v1.0.0".to_owned())
        );
    }

    #[test]
    fn a_failed_head_probe_leaves_every_fact_unknown() {
        let observation = normalize(
            &RunOutcome::Failed { code: Some(128) },
            &success(""),
            &success(""),
            GIT_COMMAND_TIMEOUT,
        );
        assert!(!observation.facts.available);
        assert!(matches!(
            &observation.facts.head_commit,
            Fact::Unknown { reason } if reason.contains("exited")
        ));
        assert!(matches!(
            &observation.facts.dirty,
            Fact::Unknown { reason } if reason.contains("not installed") || !reason.is_empty()
        ));
    }

    #[test]
    fn a_timeout_on_status_makes_dirtiness_unknown_without_losing_head() {
        let observation = normalize(
            &head_outcome(),
            &success("v1.2.3\n"),
            &RunOutcome::TimedOut,
            GIT_COMMAND_TIMEOUT,
        );
        assert!(observation.facts.available);
        assert_eq!(
            observation.facts.head_commit.value(),
            Some(&HEAD.to_owned())
        );
        assert!(matches!(
            &observation.facts.dirty,
            Fact::Unknown { reason } if reason.contains("did not finish")
        ));
    }

    #[test]
    fn a_directory_that_is_not_a_repository_is_reported_as_such() {
        assert_eq!(
            absent_reason(&RunOutcome::Failed { code: Some(128) }, GIT_COMMAND_TIMEOUT),
            "this directory is not a git repository, so workspace provenance is unavailable"
        );
        assert_eq!(
            absent_reason(&RunOutcome::Unavailable, GIT_COMMAND_TIMEOUT),
            "git is not installed, so workspace provenance is unavailable"
        );
        assert!(
            absent_reason(&RunOutcome::TimedOut, Duration::from_secs(5))
                .contains("did not answer within 5s")
        );
    }

    #[test]
    fn normalized_facts_never_carry_a_repository_path() {
        let observation = normalize(
            &head_outcome(),
            &success("v1.2.3\n"),
            &success(""),
            GIT_COMMAND_TIMEOUT,
        );
        let text = format!("{observation:?}");
        for forbidden in ["C:\\", "\\repos\\", "/home/", "/Users/", ":/"] {
            assert!(!text.contains(forbidden), "leaked a host path: {text}");
        }
    }

    #[test]
    fn a_missing_executable_is_unavailable_rather_than_a_crash() {
        let probe = GitProbe::with_program("firmwaresight-definitely-not-git");
        let observation = probe.observe(Path::new("."));
        assert!(!observation.facts.available);
        assert!(matches!(
            &observation.facts.head_commit,
            Fact::Unknown { reason } if reason == "git is not installed, so workspace provenance is unavailable"
        ));
    }

    #[test]
    fn a_command_that_outlives_its_timeout_is_killed_and_reported() {
        // A real long-running process: `sleep` on unix, `ping`'s retry delay on Windows. The budget is
        // 150 ms against a program that cannot finish that fast, so no scheduling race flips this.
        let (program, args) = sleeper();
        let argv: Vec<OsString> = args.iter().map(|arg| OsString::from(*arg)).collect();
        let outcome = run_bounded(
            OsStr::new(program),
            &argv,
            Duration::from_millis(150),
            4_096,
        );
        assert_eq!(
            outcome,
            RunOutcome::TimedOut,
            "`{program} {args:?}` was expected to exceed the timeout"
        );
    }

    #[test]
    fn output_beyond_the_cap_is_flagged_instead_of_silently_cut() {
        let (bytes, truncated) = read_bounded(io::Cursor::new(vec![b'x'; 5_000]), 1_024).unwrap();
        assert!(truncated, "5000 bytes past a 1024 cap must be flagged");
        assert_eq!(bytes.len(), 1_025, "the read stops one byte past the cap");
        let (bytes, truncated) = read_bounded(io::Cursor::new(vec![b'x'; 100]), 1_024).unwrap();
        assert!(!truncated);
        assert_eq!(bytes.len(), 100);
    }

    #[test]
    fn argv_is_built_so_a_path_is_one_opaque_argument() {
        let argv = argv_for(Path::new(".;echo-pwned"), &["rev-parse", "HEAD"]);
        assert_eq!(argv.len(), 4);
        assert_eq!(argv[0], OsString::from("-C"));
        assert_eq!(argv[1], OsString::from(".;echo-pwned"));
        assert_eq!(argv[2], OsString::from("rev-parse"));
        assert_eq!(argv[3], OsString::from("HEAD"));
    }

    #[test]
    fn a_path_shaped_like_a_shell_command_is_not_executed() {
        // A directory name that would be a separator inside a shell string. The probe hands it over as
        // one argument, so the worst outcome is "not a git repository" — never a command.
        let probe = GitProbe::system();
        let observation = probe.observe(Path::new(".;echo-pwned"));
        assert!(!observation.facts.available);
        let text = format!("{observation:?}");
        assert!(
            !text.contains("pwned"),
            "a fragment of the path became part of a fact: {text}"
        );
    }

    #[test]
    fn a_real_repository_reports_its_own_head_tag_and_dirtiness() {
        let workspace = TemporaryRepository::create();
        let probe = GitProbe::system();
        let clean = probe.observe(&workspace.root);
        assert!(
            clean.facts.available,
            "a fresh repository must be readable: {:?}",
            clean.facts
        );
        let head = clean
            .facts
            .head_commit
            .value()
            .expect("git reports HEAD in a real repository");
        assert!(looks_like_object_id(head), "{head}");
        assert_eq!(clean.facts.exact_tag.value(), Some(&"v1.2.3".to_owned()));
        assert_eq!(clean.facts.dirty, Fact::known(false));

        workspace.edit_tracked_file();
        let dirty = probe.observe(&workspace.root);
        assert_eq!(dirty.facts.dirty, Fact::known(true));
        assert_eq!(
            dirty.facts.head_commit, clean.facts.head_commit,
            "editing a tracked file changes dirtiness, not HEAD"
        );

        workspace.remove_tag();
        let untagged = probe.observe(&workspace.root);
        assert!(
            !untagged.facts.exact_tag.is_known(),
            "a repository with no tag at HEAD reports the absence, not a guess"
        );
        assert_eq!(untagged.facts.dirty, Fact::known(true));
    }

    fn sleeper() -> (&'static str, &'static [&'static str]) {
        if cfg!(windows) {
            ("ping", &PING_ARGS)
        } else {
            ("sleep", &SLEEP_ARGS)
        }
    }

    /// A throwaway repository under the system temp directory, removed when dropped. It exists so the
    /// adapter is proven against real Git output and not only against synthesized outcomes.
    struct TemporaryRepository {
        root: std::path::PathBuf,
    }

    impl TemporaryRepository {
        fn create() -> Self {
            let root = std::env::temp_dir()
                .join(format!("firmwaresight-git-probe-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            assert!(
                std::fs::create_dir_all(&root).is_ok(),
                "could not create the temporary probe directory"
            );
            let workspace = Self { root };
            assert!(
                std::fs::write(workspace.file(), "int main(void){return 0;}\n").is_ok(),
                "could not write the file the probe repository tracks"
            );
            for step in workspace.steps() {
                let outcome = run_bounded(
                    OsStr::new("git"),
                    &argv_for(&workspace.root, &step),
                    Duration::from_secs(30),
                    1 << 20,
                );
                assert!(
                    matches!(outcome, RunOutcome::Success { .. }),
                    "`git {step:?}` failed in the temporary repository: {outcome:?}. Is git installed, \
                     and is `git config --global --get user.name` unset so the -c overrides below hold?"
                );
            }
            workspace
        }

        fn file(&self) -> std::path::PathBuf {
            self.root.join("firmware.c")
        }

        /// Every step is a command line the adapter itself would never run: `init`, local config, one
        /// commit and one tag, each with the repository-local hooks and signing disabled so a global
        /// developer config cannot reach into the test.
        fn steps(&self) -> Vec<Vec<&'static str>> {
            vec![
                vec!["init", "-q", "."],
                vec![
                    "-c",
                    "core.hooksPath=.",
                    "config",
                    "user.email",
                    "gate@test.invalid",
                ],
                vec![
                    "-c",
                    "core.hooksPath=.",
                    "config",
                    "user.name",
                    "Gate Probe",
                ],
                vec!["-c", "core.hooksPath=.", "add", "firmware.c"],
                vec![
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    "core.hooksPath=.",
                    "commit",
                    "-m",
                    "probe commit",
                ],
                vec!["-c", "core.hooksPath=.", "tag", "v1.2.3"],
            ]
        }

        fn edit_tracked_file(&self) {
            let mut text = std::fs::read_to_string(self.file()).unwrap_or_default();
            text.push_str("/* edited after the commit */\n");
            assert!(std::fs::write(self.file(), text).is_ok());
        }

        fn remove_tag(&self) {
            let outcome = run_bounded(
                OsStr::new("git"),
                &argv_for(&self.root, &["tag", "-d", "v1.2.3"]),
                Duration::from_secs(30),
                1 << 20,
            );
            assert!(
                matches!(outcome, RunOutcome::Success { .. }),
                "the probe could not remove its own tag: {outcome:?}"
            );
        }
    }

    impl Drop for TemporaryRepository {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}
