//! Application preferences: the settings a person chooses on this machine in the Settings page.
//!
//! Three scopes are kept apart on purpose (`FirmwareSight … 决策 v0.2` §3, and `NEXT-02` §1):
//! a **preference** changes how this application looks and starts on this machine; a **project
//! policy** (`firmwaresight.toml`) changes what the Gate judges; **session state** is what page a
//! reader is standing on right now. Only the first lives here.
//!
//! The boundary is a hard one, and it is what makes this module safe to exist at all: nothing here
//! reaches `firmwaresight-core`, the Gate canonical input, a run id, a policy fingerprint, a
//! `SnapshotId`, a release id, a portable document or a bundle. A preference is not evidence, so it
//! is never bound into an identity, and it is never written beside a project's policy. The
//! regression that proves it is not a promise is this round's untouched `p4_golden` byte comparison,
//! the frozen goldens, and the `drift/goldens unchanged` gate step — a change in the size unit must
//! leave every one of those bytes where it is.
//!
//! Storage is one versioned JSON document in the platform's application-config directory
//! (`app_config_dir`, resolved by Rust, never supplied by the WebView). It is `config`, which
//! `AGENTS.md` §6 allows JSON for; it is not structured history, so it does not go into SQLite, and
//! it is not a second project policy.

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::ipc::{ErrorEnvelopeDto, PreferenceRecoveryDto, PreferencesDto};
use crate::startup;
use crate::{Session, next_operation_id};

/// The file's name. One constant, so Diagnostics and the Settings page cannot speak of a file this
/// build does not open.
pub const SETTINGS_FILE_NAME: &str = "settings.json";
/// Where bytes this build could not read are kept before a save replaces them.
pub const PRESERVED_FILE_NAME: &str = "settings.json.unreadable";
/// The only schema version this build writes.
///
/// It went 1 → 2 with `ADR-0031` to add `theme`, and 2 → 3 with `ADR-0032` to add `language`. The
/// version moves rather than a field being slipped into an older document, because the document is
/// `deny_unknown_fields`: an older build meeting a newer key would have called it damage, and "this
/// file is corrupt" is both false and the prelude to overwriting it. `read_document` accepts every
/// version from 1 up to this one, so the floor is two versions down and the ceiling is still
/// "anything above this build is not mine to interpret" — which is the order `ADR-0031` D2 fixes and
/// `ADR-0032` D3 repeats, because raising the number before widening the reader turns every existing
/// person's settings into a recovery story that ends in `.unreadable`.
pub const SCHEMA_VERSION: u32 = 3;

/// How sizes are written out. Reuses the `bytes` / `kib` pair the existing switch and `format.ts`
/// already speak; it relabels text and never re-derives a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizeUnitPref {
    Bytes,
    Kib,
}

/// Which page a start opens on. A static choice, deliberately not "the last page I was on": a
/// restored page must not become a route by which an earlier Gate run reads as current.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupPagePref {
    Analyze,
    History,
}

/// How the window is coloured. `ADR-0031` D1: three values, one default, no theme editor.
///
/// `system` is not a third palette — it is the instruction to ask the platform, which is why the
/// resolved palette is never stored. Writing the resolved value would turn every OS change into a
/// preference write, and would make a person who chose "follow the system" lose that choice the
/// first time their OS flips after dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePref {
    /// Follow `prefers-color-scheme`, and follow it while the window is open.
    #[default]
    System,
    Light,
    Dark,
}

/// Which words the interface is written in. `ADR-0032` D1: three values, one default, no language
/// editor and no fourth option.
///
/// `system` is the instruction to ask the platform, and the resolved language is never stored —
/// storing it would turn every desktop language change into a preference write and would silently
/// replace a choice the person made. The document's word for Simplified Chinese is `zh-CN`, spelled
/// exactly as the Settings page sends it: `ADR-0032` D1 refuses to claim Traditional support, so
/// `zh-TW` and `zh-HK` are not offered here and the UI resolver maps them to this catalog rather
/// than inventing a fourth value that would promise a second one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LanguagePref {
    #[default]
    System,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

/// The preferences a person holds right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationPreferences {
    pub size_unit: SizeUnitPref,
    pub startup_page: StartupPagePref,
    pub theme: ThemePref,
    pub language: LanguagePref,
}

impl Default for ApplicationPreferences {
    fn default() -> Self {
        Self {
            size_unit: SizeUnitPref::Bytes,
            startup_page: StartupPagePref::Analyze,
            theme: ThemePref::System,
            language: LanguagePref::System,
        }
    }
}

impl SizeUnitPref {
    /// The closed set, in the order the Settings page lists it.
    pub const WORDS: &'static [(&'static str, Self)] =
        &[("bytes", Self::Bytes), ("kib", Self::Kib)];

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Kib => "kib",
        }
    }

    /// `None` for any word outside the closed set, so a refusal is possible rather than a silent
    /// default.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::WORDS
            .iter()
            .find(|(w, _)| *w == word)
            .map(|(_, v)| *v)
    }
}

impl StartupPagePref {
    pub const WORDS: &'static [(&'static str, Self)] =
        &[("analyze", Self::Analyze), ("history", Self::History)];

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Analyze => "analyze",
            Self::History => "history",
        }
    }

    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::WORDS
            .iter()
            .find(|(w, _)| *w == word)
            .map(|(_, v)| *v)
    }
}

impl ThemePref {
    /// The closed set, in the order the Settings page lists it: the default first.
    pub const WORDS: &'static [(&'static str, Self)] = &[
        ("system", Self::System),
        ("light", Self::Light),
        ("dark", Self::Dark),
    ];

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::WORDS
            .iter()
            .find(|(w, _)| *w == word)
            .map(|(_, v)| *v)
    }
}

impl LanguagePref {
    /// The closed set, in the order the Settings page lists it: the default first.
    pub const WORDS: &'static [(&'static str, Self)] = &[
        ("system", Self::System),
        ("zh-CN", Self::ZhCn),
        ("en", Self::En),
    ];

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::WORDS
            .iter()
            .find(|(w, _)| *w == word)
            .map(|(_, v)| *v)
    }
}

/// The document on disk at this build's own version: four values plus the version that says how to
/// read them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFile {
    schema_version: u32,
    size_unit: SizeUnitPref,
    startup_page: StartupPagePref,
    theme: ThemePref,
    language: LanguagePref,
}

/// The document as the `ADR-0031` round wrote it: the same object with no language.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFileV2 {
    schema_version: u32,
    size_unit: SizeUnitPref,
    startup_page: StartupPagePref,
    theme: ThemePref,
}

/// The document as `B1-U1` wrote it: the same object with no theme.
///
/// It is a separate shape rather than `theme: Option<ThemePref>` on the one struct because the
/// version decides which keys are legal. A v1 document that carried a `theme` key is a disagreement
/// about which build wrote it, and `deny_unknown_fields` on this shape is what refuses to guess.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFileV1 {
    schema_version: u32,
    size_unit: SizeUnitPref,
    startup_page: StartupPagePref,
}

/// Why the loaded values are the defaults rather than what a person last chose. Said out loud to the
/// Settings page, because a recovered default that is not named is indistinguishable from a setting
/// that was never offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recovery {
    /// No file yet — the first start, not an error.
    Missing,
    /// The bytes could not be read as this version's document.
    Corrupted {
        /// Whether a copy of those bytes is already on hand under [`PRESERVED_FILE_NAME`], which is
        /// what lets the page promise recovery instead of implying the settings were thrown away.
        /// A load never creates that copy — only a save that replaces the bytes does.
        preserved: bool,
    },
    /// A newer build wrote this file; its values are not interpreted here.
    FutureVersion { seen: u32 },
}

/// What a load produced: the values to use, and the reason when they are the defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    pub preferences: ApplicationPreferences,
    pub recovery: Option<Recovery>,
}

/// A preference that could not be stored. Every arm is a refusal the person can act on and carries
/// its own code; none reports success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreferenceError {
    /// The config folder could not be reached or created.
    FolderUnreachable { detail: String },

    /// A fresh document could not be written.
    WriteFailed { detail: String },

    /// The completed document could not be put in place.
    ReplaceFailed { detail: String },

    /// A newer file is on disk and a save would silently downgrade it.
    FutureVersionOnDisk { seen: u32, current: u32 },

    /// A word outside a closed set arrived from a caller.
    InvalidValue { key: &'static str, value: String },
}

impl fmt::Display for PreferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FolderUnreachable { detail } => {
                write!(
                    formatter,
                    "the preferences folder could not be reached: {detail}"
                )
            }
            Self::WriteFailed { detail } => {
                write!(formatter, "the preferences could not be written: {detail}")
            }
            Self::ReplaceFailed { detail } => {
                write!(
                    formatter,
                    "the written preferences could not be put in place: {detail}"
                )
            }
            Self::FutureVersionOnDisk { seen, current } => write!(
                formatter,
                "the stored preferences were written by schema version {seen}, newer than this \
                 build's {current}"
            ),
            Self::InvalidValue { key, value } => {
                write!(formatter, "`{value}` is not a {key} this build offers")
            }
        }
    }
}

impl std::error::Error for PreferenceError {}

impl PreferenceError {
    /// The diagnostics family for preferences. `8001`–`8005` are the whole set this build can emit
    /// (`the_preference_family_is_five_distinct_codes_and_nothing_else`), and `ERR-PREFS` named no
    /// other refusal anywhere in the tracked tree at the handover head.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::FolderUnreachable { .. } => "ERR-PREFS-8001",
            Self::WriteFailed { .. } => "ERR-PREFS-8002",
            Self::ReplaceFailed { .. } => "ERR-PREFS-8003",
            Self::FutureVersionOnDisk { .. } => "ERR-PREFS-8004",
            Self::InvalidValue { .. } => "ERR-PREFS-8005",
        }
    }

    #[must_use]
    pub fn remediation(&self) -> &'static str {
        match self {
            Self::FolderUnreachable { .. } => {
                "check that your user profile is writable, then save again; the values on this screen \
                 are still what this window will use"
            }
            Self::WriteFailed { .. } | Self::ReplaceFailed { .. } => {
                "close whatever program holds the preferences file, then save again; the previously \
                 saved preferences are still in place"
            }
            Self::FutureVersionOnDisk { .. } => {
                "these preferences came from a newer version, so they are not re-written here; use \
                 Reset application preferences to replace them deliberately"
            }
            Self::InvalidValue { .. } => {
                "choose one of the options this screen offers; nothing was saved"
            }
        }
    }
}

/// Where the document lives for this process, and the lock that gives two saves in one window a
/// determined order.
#[derive(Debug, Default)]
pub struct Store {
    dir: Option<PathBuf>,
    write_lock: Mutex<()>,
}

/// The only field a probe reads, and read loosely on purpose: a document from a newer build may carry
/// keys this one has never heard of, and their presence must not be mistaken for damage. The strict
/// read comes after the version is known to be this build's.
#[derive(Debug, Deserialize)]
struct VersionProbe {
    #[serde(default)]
    schema_version: Option<u32>,
}

/// Why a document's bytes are not this build's document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unreadable {
    /// A newer version wrote it.
    Future { seen: u32 },
    /// Nothing about these bytes can be trusted: not JSON, not this version, or a value outside a
    /// closed set.
    Damaged,
}

/// The versioned read. One path is used by the load and by the save's check, so the two can never
/// disagree about whether a file is readable.
///
/// Three shapes are understood. The version picks which keys are legal, so a v1 document is read
/// whole and gains this build's default theme and language, a v2 document gains only the language,
/// and a v1 document carrying a v2 key is still a disagreement rather than a hint. Above this
/// build's own version nothing is interpreted at all.
fn read_document(raw: &[u8]) -> Result<ApplicationPreferences, Unreadable> {
    let Ok(probe) = serde_json::from_slice::<VersionProbe>(raw) else {
        return Err(Unreadable::Damaged);
    };
    match probe.schema_version {
        None => Err(Unreadable::Damaged),
        Some(seen) if seen > SCHEMA_VERSION => Err(Unreadable::Future { seen }),
        Some(1) => serde_json::from_slice::<SettingsFileV1>(raw)
            .map(|document| ApplicationPreferences {
                size_unit: document.size_unit,
                startup_page: document.startup_page,
                theme: ThemePref::default(),
                language: LanguagePref::default(),
            })
            .map_err(|_| Unreadable::Damaged),
        Some(2) => serde_json::from_slice::<SettingsFileV2>(raw)
            .map(|document| ApplicationPreferences {
                size_unit: document.size_unit,
                startup_page: document.startup_page,
                theme: document.theme,
                language: LanguagePref::default(),
            })
            .map_err(|_| Unreadable::Damaged),
        Some(3) => serde_json::from_slice::<SettingsFile>(raw)
            .map(|document| ApplicationPreferences {
                size_unit: document.size_unit,
                startup_page: document.startup_page,
                theme: document.theme,
                language: document.language,
            })
            .map_err(|_| Unreadable::Damaged),
        // Version 0 and anything else below the oldest readable document: no contract to honour.
        Some(_) => Err(Unreadable::Damaged),
    }
}

impl Store {
    /// A store over the platform's application-config directory. `None` says this host gave the
    /// application no such directory: the product then runs on defaults and says so instead of
    /// pretending a save landed somewhere.
    #[must_use]
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self {
            dir,
            write_lock: Mutex::new(()),
        }
    }

    /// The full path, when there is one. It never crosses to the WebView; only the file's name does.
    #[must_use]
    pub fn settings_path(&self) -> Option<PathBuf> {
        self.dir.as_ref().map(|dir| dir.join(SETTINGS_FILE_NAME))
    }

    /// Read the document. Writes nothing, ever: a file this build cannot read is evidence about the
    /// past, and destroying it during a load would be a side effect nobody asked for.
    #[must_use]
    pub fn load(&self) -> Loaded {
        let defaults = ApplicationPreferences::default();
        let Some(path) = self.settings_path() else {
            // Nowhere to read from is not a damaged file, and it is not a first start either.
            return Loaded {
                preferences: defaults,
                recovery: None,
            };
        };
        match std::fs::read(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Loaded {
                preferences: defaults,
                recovery: Some(Recovery::Missing),
            },
            // Unreadable for a reason other than absence — the path is a directory, the profile is
            // locked. The bytes are left exactly where they are.
            Err(_) => Loaded {
                preferences: defaults,
                recovery: Some(Recovery::Corrupted {
                    preserved: self.has_preserved_copy(),
                }),
            },
            Ok(raw) => match read_document(&raw) {
                Ok(preferences) => Loaded {
                    preferences,
                    recovery: None,
                },
                Err(Unreadable::Future { seen }) => Loaded {
                    preferences: defaults,
                    recovery: Some(Recovery::FutureVersion { seen }),
                },
                Err(Unreadable::Damaged) => Loaded {
                    preferences: defaults,
                    recovery: Some(Recovery::Corrupted {
                        preserved: self.has_preserved_copy(),
                    }),
                },
            },
        }
    }

    /// Store the document, replacing any previous one only after its bytes are no longer needed.
    ///
    /// # Errors
    ///
    /// [`PreferenceError`] when the folder, the write or the replace fails, or when a newer document
    /// is on disk.
    pub fn save(&self, preferences: &ApplicationPreferences) -> Result<Loaded, PreferenceError> {
        self.write_document(preferences, false)
    }

    /// Replace whatever is stored with the defaults, deliberately. This is the one path allowed to
    /// put a document of this build over a newer version's, because a person asked for it by name.
    ///
    /// # Errors
    ///
    /// As [`Store::save`].
    pub fn reset(&self) -> Result<Loaded, PreferenceError> {
        self.write_document(&ApplicationPreferences::default(), true)
    }

    /// The whole write: check what is there, keep what cannot be read, land the new bytes, then say
    /// what is now on disk.
    ///
    /// The lock gives two saves issued from one window a determined order — each runs this whole
    /// sequence, so the last one to take the lock is the last one whose bytes are on disk.
    fn write_document(
        &self,
        preferences: &ApplicationPreferences,
        force: bool,
    ) -> Result<Loaded, PreferenceError> {
        let guard = self
            .write_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = self
            .dir
            .as_ref()
            .ok_or_else(|| PreferenceError::FolderUnreachable {
                detail: "this host gave the application no configuration folder".to_owned(),
            })?;
        let settings = dir.join(SETTINGS_FILE_NAME);
        if let Err(error) = std::fs::create_dir_all(dir) {
            return Err(PreferenceError::FolderUnreachable {
                detail: detail_for(&error),
            });
        }
        let existing = std::fs::read(&settings);
        // One parse decides all three questions this write faces: whether a newer document makes the
        // save a downgrade, whether the bytes need keeping before they are replaced, and whether an
        // ordinary replace is all that is owed. A file that could not be read at all leaves nothing
        // to keep, and is not treated as damage.
        if let Ok(raw) = existing.as_deref() {
            match read_document(raw) {
                Ok(_) => {}
                Err(Unreadable::Future { seen }) if !force => {
                    return Err(PreferenceError::FutureVersionOnDisk {
                        seen,
                        current: SCHEMA_VERSION,
                    });
                }
                Err(_) => keep_unreadable(dir, raw)?,
            }
        }
        let temp = dir.join(format!("{SETTINGS_FILE_NAME}.tmp-{}", std::process::id()));
        if let Err(error) = std::fs::File::create(&temp).and_then(|mut file| {
            file.write_all(render(preferences).as_bytes())?;
            file.flush()
        }) {
            let _ = std::fs::remove_file(&temp);
            return Err(PreferenceError::WriteFailed {
                detail: detail_for(&error),
            });
        }
        // The replace is what has to work on this host, not on a UNIX assumption: on Windows
        // `MoveFileExW` with replace-existing is what Rust's `rename` issues, and it fails while
        // another program holds the file. Both outcomes are tested here rather than assumed.
        if let Err(error) = std::fs::rename(&temp, &settings) {
            let _ = std::fs::remove_file(&temp);
            return Err(PreferenceError::ReplaceFailed {
                detail: detail_for(&error),
            });
        }
        drop(guard);
        // Say what is on disk now, read back through the same parser that will read it at the next
        // start — a save that "succeeded" into bytes no load can read is not a save.
        Ok(self.load())
    }

    fn has_preserved_copy(&self) -> bool {
        self.dir
            .as_ref()
            .is_some_and(|dir| dir.join(PRESERVED_FILE_NAME).is_file())
    }

    /// What the Settings page is allowed to know: the values, why they are what they are, and the
    /// document's name. Not where it lives.
    pub(crate) fn dto(&self, loaded: &Loaded) -> PreferencesDto {
        let recovery = match (&self.dir, loaded.recovery) {
            (None, _) => PreferenceRecoveryDto::NowhereToStore,
            (Some(_), None) => PreferenceRecoveryDto::Read,
            (Some(_), Some(Recovery::Missing)) => PreferenceRecoveryDto::Missing,
            (Some(_), Some(Recovery::Corrupted { .. })) => PreferenceRecoveryDto::Corrupted,
            (Some(_), Some(Recovery::FutureVersion { .. })) => PreferenceRecoveryDto::FutureVersion,
        };
        PreferencesDto {
            size_unit: loaded.preferences.size_unit.word().to_owned(),
            startup_page: loaded.preferences.startup_page.word().to_owned(),
            theme: loaded.preferences.theme.word().to_owned(),
            language: loaded.preferences.language.word().to_owned(),
            recovery,
            kept_unreadable_copy: self.has_preserved_copy(),
            file_name: SETTINGS_FILE_NAME.to_owned(),
        }
    }
}

/// The preferences this window is running with, and why.
///
/// A read never fails and never writes: with no folder, no file, or unreadable bytes the answer is
/// the defaults plus the reason, which is what lets the page open instead of refusing to start
/// (`NEXT-02` T01, T04, T05).
#[tauri::command]
pub(crate) fn get_application_preferences(state: State<'_, Arc<Session>>) -> PreferencesDto {
    let store = state.preferences();
    let loaded = store.load();
    store.dto(&loaded)
}

/// Store what the person chose.
///
/// The words arrive as text and are matched against the closed sets here, so an option the page does
/// not offer is refused with a code and a next step rather than reaching the document — the same
/// shape `set_release_attachment_kind` uses for its kind words. A refusal leaves the stored document
/// untouched, and the page shows the words it was refused instead of a saved confirmation.
#[tauri::command]
pub(crate) fn update_application_preferences(
    size_unit: String,
    startup_page: String,
    theme: String,
    language: String,
    state: State<'_, Arc<Session>>,
) -> Result<PreferencesDto, ErrorEnvelopeDto> {
    let store = state.preferences();
    let operation_id = next_operation_id();
    let preferences = ApplicationPreferences {
        size_unit: SizeUnitPref::from_word(&size_unit).ok_or_else(|| {
            preference_envelope(
                &PreferenceError::InvalidValue {
                    key: "size unit",
                    value: size_unit.clone(),
                },
                &operation_id,
            )
        })?,
        startup_page: StartupPagePref::from_word(&startup_page).ok_or_else(|| {
            preference_envelope(
                &PreferenceError::InvalidValue {
                    key: "start page",
                    value: startup_page.clone(),
                },
                &operation_id,
            )
        })?,
        theme: ThemePref::from_word(&theme).ok_or_else(|| {
            preference_envelope(
                &PreferenceError::InvalidValue {
                    key: "theme",
                    value: theme.clone(),
                },
                &operation_id,
            )
        })?,
        language: LanguagePref::from_word(&language).ok_or_else(|| {
            preference_envelope(
                &PreferenceError::InvalidValue {
                    key: "language",
                    value: language.clone(),
                },
                &operation_id,
            )
        })?,
    };
    store
        .save(&preferences)
        .map(|loaded| store.dto(&loaded))
        .map_err(|error| preference_envelope(&error, &operation_id))
}

/// Put the defaults back, and only the defaults.
///
/// The store's own folder is the whole scope: no build, Gate run or release record is opened by this
/// command, and no file beside the document is touched (`NEXT-02` T06).
#[tauri::command]
pub(crate) fn reset_application_preferences(
    state: State<'_, Arc<Session>>,
) -> Result<PreferencesDto, ErrorEnvelopeDto> {
    let store = state.preferences();
    let operation_id = next_operation_id();
    store
        .reset()
        .map(|loaded| store.dto(&loaded))
        .map_err(|error| preference_envelope(&error, &operation_id))
}

/// A preference refusal, in the envelope every command in this shell answers with.
///
/// The colon, not a full stop, joins the two halves: `PreferenceError` renders the way Rust errors
/// do, from a lowercase clause, and a sentence boundary before it put a lowercase word in the middle
/// of a person's message.
fn preference_envelope(error: &PreferenceError, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: error.code().to_owned(),
        message: format!("Your preferences were not saved: {error}"),
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some(error.remediation().to_owned()),
    }
}

/// The document's text. Serialization of an integer and three closed-set words cannot fail; `expect`
/// states that instead of inventing a fifth refusal for it. Writing through `SettingsFile` rather
/// than a format string means a field added to the document cannot be silently left out of the text
/// it declares — which is the trap this function exists to close, since a document that writes two
/// of its three fields still parses.
fn render(preferences: &ApplicationPreferences) -> String {
    let document = SettingsFile {
        schema_version: SCHEMA_VERSION,
        size_unit: preferences.size_unit,
        startup_page: preferences.startup_page,
        theme: preferences.theme,
        language: preferences.language,
    };
    let mut text =
        serde_json::to_string(&document).expect("the settings document always serializes");
    text.push('\n');
    text
}

/// Keep bytes this build cannot interpret before anything replaces them.
///
/// A keep that fails stops the save: destroying the only copy of what a person last chose is not a
/// price worth paying for a preference, and a save that carries on regardless would report a success
/// it earned by discarding their file.
fn keep_unreadable(dir: &Path, raw: &[u8]) -> Result<(), PreferenceError> {
    std::fs::write(dir.join(PRESERVED_FILE_NAME), raw).map_err(|error| {
        PreferenceError::ReplaceFailed {
            detail: format!(
                "the unreadable file could not be kept before replacement: {}",
                detail_for(&error)
            ),
        }
    })
}

/// An operating-system failure, said without saying where anything lives.
///
/// `std::io::Error`'s own message is allowed to quote the path it failed on, and a preference refusal
/// is shown to the WebView. The reduction is the same one startup already uses, so there is one rule
/// for it in this crate rather than two that can drift.
fn detail_for(error: &std::io::Error) -> String {
    startup::without_directories(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A detail never carries a directory, whatever the host's separator is, and never loses the
    /// reason or the file the person can act on.
    #[test]
    fn an_operating_system_failure_is_said_without_a_path() {
        for (message, reason) in [
            (
                "permission denied opening /home/example/.config/com.firmwaresight.desktop/settings.json",
                "permission",
            ),
            (
                "cannot create a file when that file already exists \\\\Users\\example\\AppData\\Roaming\\settings.json",
                "already",
            ),
        ] {
            let detail = detail_for(&std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                message,
            ));
            assert!(
                !detail.contains('/') && !detail.contains('\\'),
                "the detail leaks where the file lives: {detail}"
            );
            assert!(detail.contains(reason), "the reason is gone: {detail}");
            assert!(
                detail.contains("settings.json"),
                "the file is gone: {detail}"
            );
        }
    }

    /// The render and the strict read are the same contract seen from both ends, and the text names
    /// all four choices: a document that wrote three of them would still parse into this build's
    /// struct, which is exactly how a field goes silently missing.
    #[test]
    fn a_rendered_document_is_exactly_what_the_reader_accepts() {
        let text = render(&ApplicationPreferences {
            size_unit: SizeUnitPref::Kib,
            startup_page: StartupPagePref::History,
            theme: ThemePref::Dark,
            language: LanguagePref::ZhCn,
        });
        assert_eq!(
            text,
            "{\"schema_version\":3,\"size_unit\":\"kib\",\"startup_page\":\"history\",\"theme\":\"dark\",\"language\":\"zh-CN\"}\n"
        );
        assert_eq!(
            read_document(text.as_bytes()),
            Ok(ApplicationPreferences {
                size_unit: SizeUnitPref::Kib,
                startup_page: StartupPagePref::History,
                theme: ThemePref::Dark,
                language: LanguagePref::ZhCn,
            })
        );
    }

    /// The language set is three words, the default asks the platform, and the hyphenated word is
    /// the document's text exactly as the Settings page sends it — `zh-CN`, not a lowercased or
    /// underscored variant that the UI would then have to know a second spelling of.
    #[test]
    fn the_language_offers_three_words_and_defaults_to_following_the_platform() {
        assert_eq!(LanguagePref::default(), LanguagePref::System);
        assert_eq!(
            LanguagePref::from_word("system"),
            Some(LanguagePref::System)
        );
        assert_eq!(LanguagePref::from_word("zh-CN"), Some(LanguagePref::ZhCn));
        assert_eq!(LanguagePref::from_word("en"), Some(LanguagePref::En));
        for word in ["zh", "zh-cn", "ZH-CN", "en-US", "fr", "auto", "", "en "] {
            assert_eq!(
                LanguagePref::from_word(word),
                None,
                "`{word}` is not offered"
            );
        }
        for (word, value) in LanguagePref::WORDS {
            assert_eq!(
                &value.word(),
                word,
                "a word that does not round trip is a second name"
            );
        }
    }

    /// The theme set is three words and nothing else, and the default is the one that asks the
    /// platform rather than picking a palette.
    #[test]
    fn the_theme_offers_three_words_and_defaults_to_following_the_platform() {
        assert_eq!(ThemePref::default(), ThemePref::System);
        assert_eq!(ThemePref::from_word("system"), Some(ThemePref::System));
        assert_eq!(ThemePref::from_word("light"), Some(ThemePref::Light));
        assert_eq!(ThemePref::from_word("dark"), Some(ThemePref::Dark));
        for word in ["System", "DARK", "auto", "follow", "", " system", "sepia"] {
            assert_eq!(ThemePref::from_word(word), None, "`{word}` is not offered");
        }
        for (word, value) in ThemePref::WORDS {
            assert_eq!(
                &value.word(),
                word,
                "a word that does not round trip is a second name"
            );
        }
    }
}
