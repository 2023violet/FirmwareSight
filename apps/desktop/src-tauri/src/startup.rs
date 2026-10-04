//! What the application says when it refused to start, and how it leaves.
//!
//! Prompt §22 draws this module's boundary. Four storage conditions are *expected* results of opening a
//! store — the pre-migration snapshot could not be written, a migration step failed, the file carries a
//! schema this build does not understand, the file could not be opened at all — and until Commit D each of
//! them reached the `.expect(...)` at the end of `run()` and panicked. A panic says "this was not supposed
//! to be possible", and here it was: the storage layer goes out of its way to fail closed in exactly these
//! cases, so the refusal is the product working. This module turns each one into a typed value — a stable
//! code, one sentence about what happened to the person's data, one next step — and `run()` exits with a
//! nonzero status instead of unwinding.
//!
//! What is deliberately *not* here is a window. There is no way to show one on this path:
//! `tauri::Builder::run` is `self.build(context)?.run(...)` (`tauri-2.12.0/src/app.rs:2617`), so a `setup`
//! error returns before the event loop starts, and the dialog plugin's blocking API may not be called from
//! the main thread at all (`tauri-plugin-dialog-2.8.0/src/lib.rs:369-372`). A recovery screen would be a new
//! shell mode, which §22 says stops this subproblem and hands it back to the Architect; that stop is
//! recorded in `P5_VALIDATION/P5_COMMIT_D_DESIGN.md` §9 rather than quietly built around. The stderr line and
//! the exit code are what this round can honestly afford.
//!
//! The module's third job is one the storage layer cannot do for itself. `StorageError::Open`'s `Display`
//! names the database path (`crates/firmwaresight-storage/src/error.rs:9`) because the CLI prints it where
//! that path is the user's own business, while a desktop startup line can end up pasted into a support
//! thread. So no engine text reaches this type without passing through [`without_directories`], which keeps
//! the last component of anything path-shaped and drops the rest. The store is always named by file.

use std::error::Error;
use std::fmt;
use std::path::Path;

use firmwaresight_storage::StorageError;

use crate::service::display_name;
use crate::support;

/// The status this application exits with when it refused to start.
///
/// `1` rather than a panic's `101`, so a launcher, an installer rollback or a CI smoke can tell "the
/// application decided not to open its store" apart from "the application crashed" — and so the difference
/// survives on platforms where the exit status is the only thing anyone sees.
pub(crate) const EXIT_CODE: i32 = 1;

/// How much of an engine message is worth repeating on a startup line.
const MAX_CAUSE_CHARS: usize = 200;

/// A storage failure this application can state without a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StartupFailure {
    /// The stable code from the registry every other FirmwareSight failure quotes, so a person repeating
    /// this line is pointed at the right page.
    code: &'static str,
    /// What happened, in one sentence that names the store by file and never by folder.
    headline: String,
    /// The engine's own reason, with the directories removed, when there is one worth repeating. Separate
    /// from `headline` because it is a mechanism, not a description of what the person should do next.
    cause: Option<String>,
    /// What the person can do now, and what is already safe.
    remediation: &'static str,
}

impl StartupFailure {
    /// The four storage conditions §22 names, each in its own words.
    ///
    /// No arm interpolates `StorageError`'s `Display`, because two of its variants name the database path;
    /// the only engine text that reaches a headline is `Backup`'s detail, which the storage layer already
    /// builds from a file name (`crates/firmwaresight-storage/src/backup.rs`), and even that is filtered.
    #[must_use]
    pub(crate) fn from_storage(err: &StorageError, store: &Path) -> Self {
        let name = display_name(store);
        let (headline, cause, remediation): (String, Option<String>, &'static str) = match err {
            StorageError::Backup { detail } => (
                format!(
                    "FirmwareSight did not change your data. Before upgrading {name} it copies the file \
                     first, and that copy could not be written."
                ),
                Some(without_directories(detail)),
                "Make the folder holding the store writable, with room for a second copy of it, then start \
                 FirmwareSight again. The store is still at the schema it had.",
            ),
            // Version 0 is not a step that failed; it is the bookkeeping table itself refusing to exist,
            // which is what a file that is not a database at all produces. Nothing had been upgraded, so
            // nothing was rolled back either, and saying so would describe an event that did not happen.
            // The engine's own words are worth keeping here — "not a database" and "locked" ask for
            // different repairs — so they are carried, filtered, where every other migration arm stays silent.
            StorageError::Migration { version: 0, source } => (
                format!(
                    "FirmwareSight could not read {name} as its own data store, so no schema step ran and \
                     nothing was changed."
                ),
                Some(without_directories(&source.to_string())),
                "Start FirmwareSight again. If it fails the same way, report this code; no firmware file is \
                 needed.",
            ),
            StorageError::Migration { version, .. } => (
                format!(
                    "FirmwareSight could not finish upgrading {name} to schema version {version}. That one \
                     step was rolled back, and the store keeps the schema it reached."
                ),
                None,
                // Deliberately does not say "nothing was changed": migrations before this one were already
                // committed, and a person told otherwise would go looking for a backup that was never owed.
                "Start FirmwareSight again. If it fails the same way, report this code with the version \
                 number it names; no firmware file is needed.",
            ),
            StorageError::UnsupportedSchemaVersion { found, supported } => (
                format!(
                    "{name} carries schema version {found}, which is newer than the {supported} this \
                     FirmwareSight build understands, so nothing was changed."
                ),
                None,
                err.remediation(),
            ),
            StorageError::Open { .. } => (
                format!(
                    "FirmwareSight could not open its own data store {name}. Nothing was changed, and \
                     nothing was created."
                ),
                None,
                // `StorageError::Open`'s own remediation says "the project directory", which is the CLI's \
                // wording: this store is the application's own, not the project being analyzed.
                "Check that FirmwareSight's application-data folder exists and is writable, and that no \
                 other program is holding the store open, then start again.",
            ),
            StorageError::Configure { .. } => (
                format!(
                    "FirmwareSight opened {name} but could not prepare it for use, so it stopped before \
                     reading or writing anything."
                ),
                None,
                "Start FirmwareSight again. If it fails the same way, report this code; no firmware file \
                 is needed.",
            ),
            // Anything else a store can return at open time is not one of the conditions this round
            // described, so it is reported as itself rather than dressed in one of the sentences above.
            other => (
                format!("FirmwareSight could not start against its data store {name}."),
                Some(without_directories(&other.to_string())),
                "Start FirmwareSight again. If it fails the same way, report this code.",
            ),
        };
        Self {
            code: err.stable_code(),
            headline,
            cause,
            remediation,
        }
    }

    /// The application-data folder could not be resolved, which is the same refusal in a different place.
    #[must_use]
    pub(crate) fn data_directory(err: &tauri::Error) -> Self {
        Self {
            code: "ERR-STORAGE-4001",
            headline: format!(
                "FirmwareSight could not find the folder its own data store lives in, so it did not open \
                 {}.",
                support::STORE_FILE_NAME
            ),
            cause: Some(without_directories(&err.to_string())),
            remediation: "Start FirmwareSight again from a normal user account. If it fails the same way, \
                          report this code.",
        }
    }

    /// The folder for the store could not be created.
    #[must_use]
    pub(crate) fn store_folder(err: &std::io::Error) -> Self {
        Self {
            code: "ERR-STORAGE-4001",
            headline: format!(
                "FirmwareSight could not create the folder for its data store, so it did not open {}.",
                support::STORE_FILE_NAME
            ),
            cause: Some(without_directories(&err.to_string())),
            remediation: "Free space in the account's application-data location, or grant write access to \
                          it, then start FirmwareSight again.",
        }
    }

    /// Say this failure out loud and leave the process.
    ///
    /// The setup hook has to be what exits rather than what answers: Tauri turns a `setup` error into a
    /// `panic!` inside its own event-loop callback (`tauri-2.12.0/src/app.rs:1443-1445`), so a returned
    /// `Err` would surface as a framework panic naming a build-machine path, and exit 101 instead of the
    /// code a launcher can test. Measured on the installed build, not inferred from the source.
    pub(crate) fn abort(&self) -> ! {
        eprintln!("FirmwareSight could not start: {self}");
        std::process::exit(EXIT_CODE);
    }
}

impl fmt::Display for StartupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cause {
            Some(cause) => write!(
                formatter,
                "{} Code {}. Reason: {cause}. {}",
                self.headline, self.code, self.remediation
            ),
            None => write!(
                formatter,
                "{} Code {}. {}",
                self.headline, self.code, self.remediation
            ),
        }
    }
}

impl Error for StartupFailure {}

/// Repeat an engine sentence without repeating where anything lives.
///
/// Every whitespace-separated word containing a path separator is reduced to its final component, so
/// `attempt to open /home/user/.local/share/firmwaresight/p0.sqlite failed` arrives as
/// `attempt to open p0.sqlite failed` — the part that helps still reads, and the part that names a machine
/// and a user is gone. The result is bounded, because a startup line is not a log.
fn without_directories(text: &str) -> String {
    let mut kept = String::new();
    for word in text.split_whitespace() {
        // `rsplit` always yields at least one item; a word ending in a separator reduces to nothing, which
        // is what the empty tail means, so it is skipped rather than kept as a bare `/`.
        let Some(reduced) = word
            .rsplit(['/', '\\'])
            .next()
            .filter(|tail| !tail.is_empty())
        else {
            continue;
        };
        if !kept.is_empty() {
            kept.push(' ');
        }
        kept.push_str(reduced);
        if kept.len() >= MAX_CAUSE_CHARS {
            break;
        }
    }
    kept.truncate(MAX_CAUSE_CHARS);
    kept
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::{Path, PathBuf};

    use firmwaresight_storage::{Database, SCHEMA_VERSION, StorageError};

    use super::{StartupFailure, without_directories};

    /// A folder-shaped string no operating system will resolve, for the cases where the message rather than
    /// the failure is what is under test.
    const FOREIGN_DIRECTORY: &str = "/home/example/.local/share/com.firmwaresight.desktop";

    fn foreign_store() -> PathBuf {
        Path::new(FOREIGN_DIRECTORY).join("firmwaresight-p0.sqlite")
    }

    /// A path whose parent provably does not exist here, so the engine fails for real rather than because a
    /// test said it should.
    fn unreachable_store() -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir()
            .join(format!(
                "firmwaresight-startup-{}-{unique}",
                std::process::id()
            ))
            .join("no-such-folder")
            .join("firmwaresight-p0.sqlite")
    }

    fn unique_temp_store(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        let directory = std::env::temp_dir().join(format!(
            "firmwaresight-startup-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("a temporary folder");
        directory.join("firmwaresight-p0.sqlite")
    }

    /// A store's folder is the one thing this line must never repeat, so every assertion about it uses the
    /// directory the engine itself was given.
    fn directory_of(store: &Path) -> String {
        store
            .parent()
            .expect("a nested temp path")
            .display()
            .to_string()
    }

    fn discard(store: &Path) {
        // A store keeps WAL sidecar files and a removal can fail while a handle is open on Windows, so the
        // folder name carries a pid and a nanosecond stamp: a leftover can never be mistaken for another
        // test's, and a leftover is not a failure of the behaviour under test.
        let _ = std::fs::remove_dir_all(directory_of(store));
    }

    #[test]
    fn a_store_that_cannot_be_opened_is_named_by_file_and_not_by_folder() {
        let store = unreachable_store();
        assert!(
            !Path::new(&directory_of(&store)).exists(),
            "the test needs a folder that does not exist: {}",
            directory_of(&store)
        );
        let err = Database::open(&store)
            .err()
            .unwrap_or_else(|| panic!("opening {} must fail", store.display()));
        // The premise, proven rather than assumed: this is the variant whose own message carries the path.
        let directory = directory_of(&store);
        assert!(
            err.to_string().contains(&directory),
            "expected the engine message to name {directory}, got {err}"
        );
        assert!(matches!(err, StorageError::Open { .. }), "got {err}");

        let failure = StartupFailure::from_storage(&err, &store);
        let text = failure.to_string();
        assert_eq!(failure.code, "ERR-STORAGE-4001");
        assert!(text.contains("firmwaresight-p0.sqlite"), "{text}");
        assert!(text.contains("Nothing was changed"), "{text}");
        assert!(!text.contains(&directory), "{text}");
        assert!(!text.contains("no-such-folder"), "{text}");
        assert!(text.contains("application-data folder"), "{text}");
    }

    #[test]
    fn a_snapshot_that_failed_says_the_upgrade_did_not_start() {
        let snapshot = "firmwaresight-p0.pre-migration-v4-to-v5.sqlite";
        let err = StorageError::Backup {
            detail: format!("could not write {FOREIGN_DIRECTORY}/{snapshot}: disk full"),
        };
        let failure = StartupFailure::from_storage(&err, &foreign_store());
        let text = failure.to_string();

        assert_eq!(failure.code, "ERR-STORAGE-4011");
        assert!(text.contains("did not change your data"), "{text}");
        assert!(text.contains("disk full"), "{text}");
        assert!(
            text.contains(snapshot),
            "the snapshot is named, because a name is what a person can act on: {text}"
        );
        assert!(
            text.contains("still at the schema it had"),
            "the fail-closed promise has to be stated where it is owed: {text}"
        );
        assert!(!text.contains(FOREIGN_DIRECTORY), "{text}");
        assert!(
            !text.contains("example"),
            "the user name must not survive: {text}"
        );
    }

    #[test]
    fn a_failed_step_is_reported_as_one_rolled_back_step() {
        // `StorageError::Migration` carries a `rusqlite::Error` this crate does not name, so the condition
        // is reached through the engine: a store stepped back to v4 with the table 0005 alters removed
        // cannot be upgraded, and the failure the product reports is the one a real machine produces.
        let store = unique_temp_store("rolled-back");
        let current = Database::open(&store).expect("a fresh store opens");
        current
            .connection()
            .execute_batch(
                "DROP INDEX idx_builds_created;
                 ALTER TABLE sections DROP COLUMN file_offset_unknown;
                 ALTER TABLE symbols DROP COLUMN address_unknown;
                 DELETE FROM schema_migrations WHERE version = 5;
                 DROP TABLE sections;",
            )
            .expect("the store is stepped back to a v4 shape with one table missing");
        drop(current);

        let err = Database::open(&store)
            .err()
            .expect("a v4 store with no sections table cannot be upgraded");
        let version = match err {
            StorageError::Migration { version, .. } => version,
            ref other => panic!("expected a migration failure, got {other}"),
        };

        let failure = StartupFailure::from_storage(&err, &store);
        let text = failure.to_string();
        assert_eq!(failure.code, "ERR-STORAGE-4003");
        assert!(text.contains(&format!("version {version}")), "{text}");
        assert!(text.contains("rolled back"), "{text}");
        // The sentence must not claim the whole upgrade undid itself: earlier steps in the same run were
        // already committed, which is exactly why the wording says "that one step".
        assert!(!text.contains("nothing was changed"), "{text}");
        assert!(!text.contains(&directory_of(&store)), "{text}");
        discard(&store);
    }

    #[test]
    fn a_file_that_is_not_a_database_is_reported_as_nothing_having_run() {
        // The condition the installed build actually reached: a file at the store path that is not a
        // database makes the bookkeeping table fail to exist, which arrives as a migration of version 0.
        // Naming that a rolled-back step would invent an upgrade that never started.
        let store = unique_temp_store("not-a-database");
        std::fs::write(&store, b"this is not a database file, and it never was")
            .expect("a file that is not a database");

        let err = Database::open(&store)
            .err()
            .expect("a file that is not a database does not open as a store");
        assert!(
            matches!(err, StorageError::Migration { version: 0, .. }),
            "expected the bookkeeping failure at version 0, got {err}"
        );

        let failure = StartupFailure::from_storage(&err, &store);
        let text = failure.to_string();
        assert_eq!(failure.code, "ERR-STORAGE-4003");
        assert!(text.contains("could not read"), "{text}");
        assert!(text.contains("no schema step ran"), "{text}");
        assert!(text.contains("nothing was changed"), "{text}");
        assert!(
            !text.contains("rolled back"),
            "nothing was rolled back, because nothing was applied: {text}"
        );
        assert!(
            !text.contains("version 0"),
            "the sentence must not name a schema version that no step ever reached: {text}"
        );
        assert!(!text.contains(&directory_of(&store)), "{text}");
        discard(&store);
    }

    #[test]
    fn a_newer_store_is_refused_with_both_version_numbers() {
        let err = StorageError::UnsupportedSchemaVersion {
            found: SCHEMA_VERSION + 4,
            supported: SCHEMA_VERSION,
        };
        let failure = StartupFailure::from_storage(&err, &foreign_store());
        let text = failure.to_string();

        assert_eq!(failure.code, "ERR-STORAGE-4004");
        assert!(
            text.contains(&format!("version {}", SCHEMA_VERSION + 4)),
            "{text}"
        );
        assert!(
            text.contains(&format!("newer than the {SCHEMA_VERSION}")),
            "both numbers have to be on the line, or nobody knows which build to go back to: {text}"
        );
        assert!(
            text.contains("run the version of FirmwareSight that wrote this database"),
            "the storage layer's own next step is the right one here: {text}"
        );
        assert!(!text.contains(FOREIGN_DIRECTORY), "{text}");
    }

    #[test]
    fn no_startup_line_repeats_a_folder() {
        // The two folder-shaped refusals do not come from the store at all, so their messages are built
        // here in the shape a real operating system writes them.
        let io = io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("cannot create {FOREIGN_DIRECTORY}: permission denied"),
        );
        let text = StartupFailure::store_folder(&io).to_string();
        assert!(text.contains("firmwaresight-p0.sqlite"), "{text}");
        assert!(text.contains("permission denied"), "{text}");
        assert!(!text.contains(FOREIGN_DIRECTORY), "{text}");
        assert!(!text.contains("example"), "{text}");

        // The reduction is per word: a Windows path arrives as its final component and a POSIX one as its
        // own, and either way the separators and everything before them are gone.
        assert_eq!(
            without_directories("open C:\\Users\\example\\AppData\\Roaming\\store.sqlite failed"),
            "open store.sqlite failed"
        );
        assert_eq!(
            without_directories(&format!("could not read {FOREIGN_DIRECTORY} at all")),
            "could not read com.firmwaresight.desktop at all"
        );
        assert_eq!(
            without_directories("trailing separator /tmp/ here"),
            "trailing separator here"
        );
        assert_eq!(without_directories(""), "");
        let bounded = without_directories(&"word ".repeat(400));
        assert!(
            bounded.len() <= super::MAX_CAUSE_CHARS,
            "{} chars",
            bounded.len()
        );
    }

    #[test]
    fn an_unexpected_storage_condition_is_stated_rather_than_dressed_up() {
        // The catch-all arm must not borrow one of the four sentences: a write failure is not a snapshot
        // failure, and saying otherwise would send a person down a path that cannot fix it.
        let err = StorageError::Write {
            detail: format!("row under {FOREIGN_DIRECTORY} rejected"),
        };
        let failure = StartupFailure::from_storage(&err, &foreign_store());
        let text = failure.to_string();

        assert_eq!(failure.code, "ERR-STORAGE-4006");
        assert!(
            text.contains("could not start against its data store"),
            "{text}"
        );
        assert!(!text.contains("copies the file first"), "{text}");
        assert!(!text.contains("rolled back"), "{text}");
        assert!(!text.contains(FOREIGN_DIRECTORY), "{text}");
        assert!(text.contains("rejected"), "{text}");
    }
}
