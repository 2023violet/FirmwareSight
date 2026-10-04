//! The store's own health, reported as a bounded product fact.
//!
//! `PRAGMA integrity_check` is the question, and the shape of its answer is what this module exists
//! for. On a healthy database it returns the single row `ok`. On a damaged one it returns a row per
//! problem — and, measured on this crate's own fixture, it then *fails mid-iteration* with
//! `database disk image is malformed` while still holding the problems it had already named. So a
//! failing step is evidence of ill health, not a separate storage failure, and the only case that
//! deserves `Err` is the one where nothing could be asked at all.
//!
//! The summary is bounded in both directions. A damaged database can name thousands of problems, and
//! this text crosses an IPC boundary into a WebView and into an exported Diagnostics file, so it is
//! cut to the first few problems and every token that carries a directory is reduced to its final
//! component. `04_TECH/05_SECURITY_PRIVACY.md` forbids a path in that payload; a report copied
//! straight out of SQLite is exactly the kind of text that would smuggle one in.

use rusqlite::Connection;

use crate::Database;
use crate::error::StorageError;

/// What the engine said about the file behind a connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreHealth {
    /// `PRAGMA integrity_check` answered `ok` and nothing else.
    Healthy,
    /// The store is damaged. `summary` is the first few problems, with any directory stripped, and
    /// says so when the list was cut. It is a report, not a repair instruction: nothing here is safe
    /// to hand to SQLite as SQL.
    Unhealthy { summary: String },
}

/// Problems named in one report. The cap is proven two ways: `integrity_and_backup.rs::
/// a_store_that_names_many_problems_is_still_bounded` against a damaged store that names 73 lines
/// uncapped, and `a_list_longer_than_the_cap_is_cut_and_says_so` against this constant directly.
const MAX_PROBLEMS: usize = 3;

/// Characters kept per problem.
const MAX_PROBLEM_CHARS: usize = 120;

/// Said out loud when the report was cut, so a truncated list cannot read as the whole list.
const TRUNCATED: &str = "only the first problems are listed";

impl Database {
    /// Ask the engine whether the file behind this connection is structurally sound.
    ///
    /// Read-only, and it repairs nothing: there is no `PRAGMA` here, no rewrite, no rebuild. A
    /// damaged store stays exactly as damaged as it was found, which is the precondition for a user
    /// restoring a backup instead of losing the file twice.
    ///
    /// # Errors
    ///
    /// `StorageError::IntegrityCheck` when the check itself could not be run — a statement SQLite
    /// refused to prepare, or an engine error before it named any problem. Damage it *can* name is
    /// reported as `StoreHealth::Unhealthy`, not as a failure to ask.
    pub fn integrity_check(&self) -> Result<StoreHealth, StorageError> {
        report(self.connection()).map_err(|detail| StorageError::IntegrityCheck { detail })
    }
}

/// Collect what the engine says, and stop at the bound.
fn report(conn: &Connection) -> Result<StoreHealth, String> {
    let mut statement = conn
        .prepare("PRAGMA integrity_check")
        .map_err(|source| describe("could not prepare the health check", &source))?;
    let mut rows = statement
        .query([])
        .map_err(|source| describe("could not start the health check", &source))?;

    let mut problems: Vec<String> = Vec::new();
    let mut cut_short = false;
    loop {
        match rows.next() {
            Ok(Some(row)) => {
                let text: String = match row.get(0) {
                    Ok(text) => text,
                    Err(source) => {
                        return Err(describe("could not read a health answer", &source));
                    }
                };
                problems.push(sanitize(&text));
                if problems.len() == MAX_PROBLEMS {
                    cut_short = true;
                    break;
                }
            }
            Ok(None) => break,
            Err(source) => {
                // The engine stopped answering. If it had already named problems, that naming is the
                // health result; if it stopped before naming any, nothing was observed and the
                // question itself failed.
                if problems.is_empty() {
                    return Err(describe(
                        "the health check could not read the database",
                        &source,
                    ));
                }
                cut_short = true;
                break;
            }
        }
    }

    if problems.len() == 1 && problems[0].eq_ignore_ascii_case("ok") && !cut_short {
        return Ok(StoreHealth::Healthy);
    }
    Ok(StoreHealth::unhealthy(problems, cut_short))
}

fn describe(what: &str, source: &rusqlite::Error) -> String {
    // `rusqlite::Error`'s own text can carry the file it failed on — an I/O failure names it — so the
    // engine's message is not copied here. The result code says as much as a user can act on, and it
    // has no path in it.
    let code = source
        .sqlite_error_code()
        .map_or_else(|| "no engine code".to_owned(), |code| format!("{code:?}"));
    format!("{what} ({code})")
}

/// Keep the first `MAX_PROBLEM_CHARS` characters, and reduce any token that carries a directory to
/// the name it ends with. Integrity reports normally contain no separators at all — they name trees,
/// pages and indexes — so this is a floor under the claim, not the common path.
fn sanitize(text: &str) -> String {
    let cut: String = text.chars().take(MAX_PROBLEM_CHARS).collect();
    cut.split_whitespace()
        .map(|token| {
            if token.contains('/') || token.contains('\\') {
                final_component(token)
            } else {
                token.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn final_component(token: &str) -> String {
    token
        .rsplit(['/', '\\'])
        .next()
        .filter(|tail| !tail.is_empty())
        .unwrap_or(token)
        .to_owned()
}

impl StoreHealth {
    /// The report a damaged store produced: at most `MAX_PROBLEMS` named, and the truncation said out
    /// loud when the list was cut. Public so a caller can rebuild the same shape; the crate's own
    /// tests assert the bound through `integrity_check`.
    #[must_use]
    pub fn unhealthy(problems: Vec<String>, cut_short: bool) -> Self {
        let mut kept = problems;
        // A caller that hands over more than the cap keeps the notice's truthfulness out of its own
        // hands: silently dropping the rest of a damage report is the one failure this type exists to
        // prevent, so an over-long list is cut *and* said to be cut.
        let over_long = kept.len() > MAX_PROBLEMS;
        kept.truncate(MAX_PROBLEMS);
        if cut_short || over_long {
            kept.push(TRUNCATED.to_owned());
        }
        Self::Unhealthy {
            summary: kept.join("; "),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_PROBLEMS, StoreHealth, sanitize};

    /// The privacy floor under the summary. Real `integrity_check` output names trees, pages and
    /// index names and was measured to contain no separator at all, so the reduction that matters for
    /// `04_TECH/05_SECURITY_PRIVACY.md` cannot be provoked through the engine — it has to be proven
    /// against the text that would carry a path, which is what this does.
    #[test]
    fn a_problem_that_carries_a_directory_is_reduced_to_the_name_it_ends_with() {
        let cut = sanitize(
            "corrupt page for /home/example/private/secret-project/firmware.elf and \
                            C:\\Users\\example\\secret-project\\firmware.elf",
        );

        assert!(
            !cut.contains('/') && !cut.contains('\\'),
            "no separator survives the reduction: {cut}"
        );
        assert!(
            !cut.contains("example") && !cut.contains("secret-project") && !cut.contains("Users"),
            "neither the home directory nor the project name travels: {cut}"
        );
        assert!(
            cut.contains("firmware.elf"),
            "the page and file the engine complained about are still recognizable: {cut}"
        );
    }

    #[test]
    fn a_long_problem_is_cut_at_the_character_bound() {
        let cut = sanitize(&"damage ".repeat(200));

        assert!(
            cut.chars().count() <= 120,
            "one problem cannot be longer than the bound, and it was {}",
            cut.chars().count()
        );
    }

    #[test]
    fn a_list_longer_than_the_cap_is_cut_and_says_so() {
        let problems: Vec<String> = (1..=5).map(|index| format!("problem {index}")).collect();

        let health = StoreHealth::unhealthy(problems, false);
        let StoreHealth::Unhealthy { summary } = &health else {
            panic!("a damage list cannot be Healthy");
        };

        let segments: Vec<&str> = summary.split("; ").collect();
        assert_eq!(
            segments.len(),
            MAX_PROBLEMS + 1,
            "the cap plus the notice, and the summary was {summary}"
        );
        assert!(
            !summary.contains("problem 4") && !summary.contains("problem 5"),
            "the cap has to cut: {summary}"
        );
        assert!(
            summary.contains("only the first problems are listed"),
            "a cut list that does not say it was cut reads as the whole damage report: {summary}"
        );
    }

    #[test]
    fn a_short_list_that_was_never_cut_is_not_called_a_cut_list() {
        let health = StoreHealth::unhealthy(vec!["page 9 is bad".to_owned()], false);
        let StoreHealth::Unhealthy { summary } = &health else {
            panic!("a damage list cannot be Healthy");
        };

        assert_eq!(summary, "page 9 is bad");
    }
}
