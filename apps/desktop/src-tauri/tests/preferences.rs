//! Application preferences: what is stored, where, and what happens when it cannot be.
//!
//! These cases are the storage half of `NEXT-02` §3's item 2 and its T01–T07: a preference is a small
//! local document, and the interesting failures are all about *this* host — a file that is a
//! directory, a document from a newer build, a torn write, two saves in a row, a folder whose name
//! is not ASCII. Each case reads real bytes from a real temporary folder, and none of them trusts
//! `std::fs::rename` to behave the same on every platform: the replace path is executed here, on the
//! host the round is being proved on.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use firmwaresight_desktop::preferences::{
    ApplicationPreferences, Loaded, PRESERVED_FILE_NAME, PreferenceError, Recovery, SCHEMA_VERSION,
    SETTINGS_FILE_NAME, SizeUnitPref, StartupPagePref, Store,
};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A throwaway config directory, including one whose name carries non-ASCII text and a space.
fn temp_dir(label: &str, fancy: bool) -> PathBuf {
    let leaf = if fancy {
        format!(
            "偏好 设置 {label}-{}",
            COUNTER.fetch_add(1, Ordering::SeqCst)
        )
    } else {
        format!("{label}-{}", COUNTER.fetch_add(1, Ordering::SeqCst))
    };
    let mut dir = std::env::temp_dir();
    dir.push(format!("fwsight-prefs-{leaf}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a temporary config folder");
    dir
}

fn settings(dir: &Path) -> PathBuf {
    dir.join(SETTINGS_FILE_NAME)
}

fn bytes(dir: &Path) -> Option<Vec<u8>> {
    std::fs::read(settings(dir)).ok()
}

/// The kept copy's bytes, read directly: `bytes` is for the live document and would join a second
/// name onto this path.
fn kept_bytes(dir: &Path) -> Option<Vec<u8>> {
    std::fs::read(dir.join(PRESERVED_FILE_NAME)).ok()
}

fn prefs(size_unit: SizeUnitPref, startup_page: StartupPagePref) -> ApplicationPreferences {
    ApplicationPreferences {
        size_unit,
        startup_page,
    }
}

fn listing(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("the folder is there")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// T01: nothing stored yet is not an error, and it is the safe pair of defaults.
#[test]
fn a_missing_file_loads_the_defaults_and_creates_nothing() {
    let dir = temp_dir("missing", false);
    let loaded = Store::new(Some(dir.clone())).load();
    assert_eq!(
        loaded,
        Loaded {
            preferences: ApplicationPreferences::default(),
            recovery: Some(Recovery::Missing),
        },
        "a first start has to say it is a first start"
    );
    assert_eq!(
        SizeUnitPref::Bytes,
        ApplicationPreferences::default().size_unit
    );
    assert_eq!(
        StartupPagePref::Analyze,
        ApplicationPreferences::default().startup_page
    );
    assert!(
        listing(&dir).is_empty(),
        "a load wrote into the folder: {loaded:?}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// T02: the value a person chose comes back through a new store over the same folder — which is what
/// a restart is to this document.
#[test]
fn a_saved_unit_comes_back_to_a_fresh_store_over_the_same_folder() {
    let dir = temp_dir("restart", false);
    Store::new(Some(dir.clone()))
        .save(&prefs(SizeUnitPref::Kib, StartupPagePref::History))
        .expect("the first save lands");
    let reopened = Store::new(Some(dir.clone())).load();
    assert_eq!(
        reopened.preferences,
        prefs(SizeUnitPref::Kib, StartupPagePref::History)
    );
    assert_eq!(reopened.recovery, None, "a readable file is not a recovery");
    std::fs::remove_dir_all(&dir).ok();
}

/// The replace path, on this host. A second save must overwrite rather than refuse, and no
/// temporary may survive either attempt.
#[test]
fn saving_twice_replaces_the_document_and_leaves_no_temporary_behind() {
    let dir = temp_dir("replace", false);
    let store = Store::new(Some(dir.clone()));
    store
        .save(&prefs(SizeUnitPref::Bytes, StartupPagePref::Analyze))
        .ok();
    store
        .save(&prefs(SizeUnitPref::Kib, StartupPagePref::Analyze))
        .expect("replacing a stored preference is a normal save");
    assert_eq!(
        listing(&dir),
        vec![SETTINGS_FILE_NAME.to_owned()],
        "a save left a stray file"
    );
    let text = String::from_utf8(bytes(&dir).expect("the document")).expect("utf-8");
    assert!(text.contains("\"kib\""), "{text}");
    assert!(
        text.contains(&format!("\"schema_version\":{SCHEMA_VERSION}")),
        "the document does not say which version it is: {text}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// §3's closed sets are closed: no producer may hand the store a word it invented.
#[test]
fn the_closed_sets_refuse_every_other_word() {
    assert_eq!(SizeUnitPref::from_word("bytes"), Some(SizeUnitPref::Bytes));
    assert_eq!(SizeUnitPref::from_word("kib"), Some(SizeUnitPref::Kib));
    for word in ["KiB", "KIB", "kibs", "mb", "", "bytes "] {
        assert_eq!(
            SizeUnitPref::from_word(word),
            None,
            "`{word}` is not offered"
        );
    }
    for word in ["kib", "last_page", "overview", "Analyze"] {
        assert_eq!(
            StartupPagePref::from_word(word),
            None,
            "`{word}` is not an offered start page"
        );
    }
}

/// T04: a document whose value is outside the closed set loads as the safe default, and the person
/// is told instead of the file being quietly rewritten.
#[test]
fn an_unreadable_value_loads_the_default_and_says_so_without_touching_the_file() {
    let dir = temp_dir("bad-enum", false);
    let written = b"{\"schema_version\":1,\"size_unit\":\"kibs\",\"startup_page\":\"analyze\"}";
    std::fs::write(settings(&dir), written).expect("the document is placed");
    let loaded = Store::new(Some(dir.clone())).load();
    assert_eq!(loaded.preferences.size_unit, SizeUnitPref::Bytes);
    assert_eq!(
        loaded.recovery,
        Some(Recovery::Corrupted { preserved: false }),
        "a load must not have preserved anything yet: {loaded:?}"
    );
    assert_eq!(bytes(&dir).as_deref(), Some(written.as_slice()));
    std::fs::remove_dir_all(&dir).ok();
}

/// T04 + §3's quality gate: the bytes a build could not read are kept before they are replaced.
#[test]
fn corrupt_bytes_survive_until_a_save_takes_them_over_and_then_are_kept() {
    let dir = temp_dir("corrupt", false);
    let written = b"{ not json at all ";
    std::fs::write(settings(&dir), written).expect("the document is placed");
    let before = Store::new(Some(dir.clone())).load();
    assert_eq!(
        before.recovery,
        Some(Recovery::Corrupted { preserved: false }),
        "a load copied what it could not read: {before:?}"
    );
    assert_eq!(
        listing(&dir),
        vec![SETTINGS_FILE_NAME.to_owned()],
        "the load already wrote a copy"
    );
    Store::new(Some(dir.clone()))
        .save(&prefs(SizeUnitPref::Kib, StartupPagePref::Analyze))
        .expect("a save over unreadable bytes is allowed, after preserving them");
    assert_eq!(
        kept_bytes(&dir).as_deref(),
        Some(written.as_slice()),
        "the unreadable original was destroyed rather than preserved"
    );
    assert_eq!(
        Store::new(Some(dir.clone())).load().preferences.size_unit,
        SizeUnitPref::Kib
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// T04: a newer document is not re-interpreted, and a save refuses rather than downgrade it.
#[test]
fn a_future_version_loads_defaults_and_a_save_refuses_to_downgrade_it() {
    let dir = temp_dir("future", false);
    let written = b"{\"schema_version\":7,\"size_unit\":\"kib\",\"startup_page\":\"history\"}";
    std::fs::write(settings(&dir), written).expect("the document is placed");
    let loaded = Store::new(Some(dir.clone())).load();
    assert_eq!(
        loaded.recovery,
        Some(Recovery::FutureVersion { seen: 7 }),
        "{loaded:?}"
    );
    assert_eq!(loaded.preferences, ApplicationPreferences::default());
    let error = Store::new(Some(dir.clone()))
        .save(&prefs(SizeUnitPref::Kib, StartupPagePref::Analyze))
        .expect_err("a newer document must not be overwritten by an older build");
    assert_eq!(error.code(), "ERR-PREFS-8004", "{error}");
    assert_eq!(bytes(&dir).as_deref(), Some(written.as_slice()));
    std::fs::remove_dir_all(&dir).ok();
}

/// Reset is the one deliberate way to replace a newer document, and it still preserves the bytes.
#[test]
fn reset_replaces_a_future_version_after_keeping_its_bytes() {
    let dir = temp_dir("reset-future", false);
    let written = b"{\"schema_version\":7,\"size_unit\":\"kib\",\"startup_page\":\"history\"}";
    std::fs::write(settings(&dir), written).expect("the document is placed");
    let reset = Store::new(Some(dir.clone()))
        .reset()
        .expect("reset is the escape hatch this refusal names");
    assert_eq!(reset.preferences, ApplicationPreferences::default());
    assert_eq!(kept_bytes(&dir).as_deref(), Some(written.as_slice()));
    assert_eq!(
        Store::new(Some(dir.clone())).load().recovery,
        None,
        "the reset document is readable, not a recovery"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// T06: resetting preferences touches nothing else in the folder — not the store, not any other file.
#[test]
fn a_reset_restores_defaults_and_leaves_every_sibling_file_untouched() {
    let dir = temp_dir("reset-siblings", false);
    let other = dir.join("firmwaresight.sqlite");
    std::fs::write(&other, b"NOT A REAL STORE BUT ITS BYTES MATTER")
        .expect("the sibling is placed");
    let before = std::fs::read(&other).expect("readable");
    Store::new(Some(dir.clone()))
        .save(&prefs(SizeUnitPref::Kib, StartupPagePref::History))
        .expect("a save lands");
    Store::new(Some(dir.clone())).reset().expect("reset lands");
    assert_eq!(
        Store::new(Some(dir.clone())).load().preferences,
        ApplicationPreferences::default()
    );
    assert_eq!(
        std::fs::read(&other).expect("the sibling still exists"),
        before,
        "a preference reset moved a file it does not own"
    );
    assert_eq!(
        listing(&dir),
        vec![
            "firmwaresight.sqlite".to_owned(),
            SETTINGS_FILE_NAME.to_owned()
        ]
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// T05: an un-storable location is a visible refusal, never a saved success.
#[test]
fn a_place_that_cannot_hold_the_document_refuses_loudly() {
    let dir = temp_dir("nowrite", false);
    // `settings.json` itself is a directory: neither the read nor the replace can succeed.
    std::fs::create_dir(settings(&dir)).expect("the obstructing folder");
    let loaded = Store::new(Some(dir.clone())).load();
    assert_eq!(
        loaded.recovery,
        Some(Recovery::Corrupted { preserved: false }),
        "{loaded:?}"
    );
    let error = Store::new(Some(dir.clone()))
        .save(&prefs(SizeUnitPref::Kib, StartupPagePref::Analyze))
        .expect_err("a directory cannot be replaced by a document");
    assert!(
        matches!(
            error.code(),
            "ERR-PREFS-8002" | "ERR-PREFS-8003" | "ERR-PREFS-8001"
        ),
        "an I/O refusal stays in the preference family: {}",
        error.code()
    );
    assert!(!error.remediation().trim().is_empty());
    std::fs::remove_dir_all(&dir).ok();
}

/// With no config directory at all, the product still answers, and a save says it did not land.
#[test]
fn a_host_that_offers_no_config_folder_runs_on_defaults_and_refuses_to_claim_a_save() {
    let store = Store::new(None);
    assert_eq!(
        store.load().preferences,
        ApplicationPreferences::default(),
        "defaults are the only honest answer with nowhere to read from"
    );
    assert_eq!(
        store
            .save(&prefs(SizeUnitPref::Kib, StartupPagePref::Analyze))
            .expect_err("nothing was written")
            .code(),
        "ERR-PREFS-8001"
    );
}

/// T07: two saves in one process leave exactly one whole document, and the last one wins.
#[test]
fn two_saves_in_a_row_leave_one_readable_document() {
    let dir = temp_dir("race", false);
    let store = Store::new(Some(dir.clone()));
    for (index, unit) in [SizeUnitPref::Bytes, SizeUnitPref::Kib].iter().enumerate() {
        store
            .save(&prefs(*unit, StartupPagePref::Analyze))
            .unwrap_or_else(|error| panic!("save {index} failed: {error}"));
    }
    assert_eq!(
        Store::new(Some(dir.clone())).load().preferences.size_unit,
        SizeUnitPref::Kib
    );
    assert_eq!(listing(&dir), vec![SETTINGS_FILE_NAME.to_owned()]);
    std::fs::remove_dir_all(&dir).ok();
}

/// §9's item 7: a folder whose name is not ASCII and carries a space is a normal case, not a crash.
#[test]
fn a_non_ascii_and_spaced_config_folder_round_trips_the_preferences() {
    let dir = temp_dir("cjk", true);
    Store::new(Some(dir.clone()))
        .save(&prefs(SizeUnitPref::Kib, StartupPagePref::History))
        .expect("the document lands in a non-ASCII folder");
    assert_eq!(
        Store::new(Some(dir.clone())).load().preferences,
        prefs(SizeUnitPref::Kib, StartupPagePref::History)
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// A zero-byte document is not "no preferences"; it is a file whose bytes must be kept.
#[test]
fn an_empty_document_is_reported_as_corruption_rather_than_as_defaults() {
    let dir = temp_dir("empty", false);
    std::fs::write(settings(&dir), b"").expect("the empty file is placed");
    assert_eq!(
        Store::new(Some(dir.clone())).load().recovery,
        Some(Recovery::Corrupted { preserved: false })
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// A key this build does not know is a disagreement about the document, not a hint to guess.
#[test]
fn a_document_carrying_an_unknown_key_is_not_partially_understood() {
    let dir = temp_dir("unknown-key", false);
    std::fs::write(
        settings(&dir),
        br#"{"schema_version":1,"size_unit":"kib","startup_page":"analyze","theme":"dark"}"#,
    )
    .expect("the document is placed");
    let loaded = Store::new(Some(dir.clone())).load();
    assert_eq!(loaded.preferences.size_unit, SizeUnitPref::Bytes);
    assert!(
        matches!(loaded.recovery, Some(Recovery::Corrupted { .. })),
        "{loaded:?}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// `preserved` is the difference between "these bytes are gone" and "these bytes are still yours".
/// It reports what is on hand, not what this load did — a load never copies anything.
#[test]
fn unreadable_bytes_already_kept_are_reported_as_kept() {
    let dir = temp_dir("kept", false);
    std::fs::write(
        settings(&dir),
        b"{\"schema_version\":1,\"size_unit\":\"sideways\"}",
    )
    .unwrap();
    std::fs::write(
        dir.join(PRESERVED_FILE_NAME),
        b"the settings from before the damage",
    )
    .unwrap();
    assert_eq!(
        Store::new(Some(dir.clone())).load().recovery,
        Some(Recovery::Corrupted { preserved: true }),
        "the page has to be able to promise recovery"
    );
    assert_eq!(
        listing(&dir),
        vec![
            SETTINGS_FILE_NAME.to_owned(),
            PRESERVED_FILE_NAME.to_owned()
        ],
        "a load copied or removed something"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// Every refusal a caller can branch on carries a code and a next step.
#[test]
fn every_preference_refusal_carries_a_code_and_a_next_step() {
    let cases = [
        PreferenceError::FolderUnreachable {
            detail: "denied".to_owned(),
        },
        PreferenceError::WriteFailed {
            detail: "disk full".to_owned(),
        },
        PreferenceError::ReplaceFailed {
            detail: "in use".to_owned(),
        },
        PreferenceError::FutureVersionOnDisk {
            seen: 7,
            current: SCHEMA_VERSION,
        },
        PreferenceError::InvalidValue {
            key: "size_unit",
            value: "kibs".to_owned(),
        },
    ];
    let codes: Vec<&str> = cases.iter().map(|error| error.code()).collect();
    assert_eq!(
        codes,
        vec![
            "ERR-PREFS-8001",
            "ERR-PREFS-8002",
            "ERR-PREFS-8003",
            "ERR-PREFS-8004",
            "ERR-PREFS-8005"
        ],
        "a code moved, and a caller that branches on it would now read the wrong answer"
    );
    for (index, error) in cases.iter().enumerate() {
        assert!(!error.remediation().trim().is_empty(), "{error}");
        let spoken = error.to_string();
        assert!(!spoken.is_empty(), "refusal {index} says nothing");
        // Each refusal has to name the thing the person can act on, not just a category.
        let marker = match error {
            PreferenceError::FolderUnreachable { detail }
            | PreferenceError::WriteFailed { detail }
            | PreferenceError::ReplaceFailed { detail } => detail.as_str(),
            PreferenceError::FutureVersionOnDisk { seen, .. } => {
                assert!(spoken.contains(&seen.to_string()), "{spoken}");
                "schema version"
            }
            PreferenceError::InvalidValue { value, .. } => value.as_str(),
        };
        assert!(
            spoken.contains(marker),
            "{spoken} does not carry `{marker}`"
        );
    }
}

/// The family is contiguous, five arms deep, and no arm shares another's number: the codes are what
/// a caller branches on, so `8001`–`8005` are the whole set this build can emit. Vacancy of the
/// family against every other family was measured before it was chosen — `ERR-PREFS` appears
/// nowhere in the tracked tree at the handover head.
#[test]
fn the_preference_family_is_five_distinct_codes_and_nothing_else() {
    let mine = [
        PreferenceError::FolderUnreachable {
            detail: String::new(),
        },
        PreferenceError::WriteFailed {
            detail: String::new(),
        },
        PreferenceError::ReplaceFailed {
            detail: String::new(),
        },
        PreferenceError::FutureVersionOnDisk {
            seen: 2,
            current: SCHEMA_VERSION,
        },
        PreferenceError::InvalidValue {
            key: "startup_page",
            value: "last_page".to_owned(),
        },
    ];
    let codes: Vec<&str> = mine.iter().map(|error| error.code()).collect();
    let unique: std::collections::BTreeSet<&str> = codes.iter().copied().collect();
    assert_eq!(
        unique.len(),
        codes.len(),
        "two refusals share a code: {codes:?}"
    );
    assert!(
        codes.iter().all(|code| code.starts_with("ERR-PREFS-800")),
        "{codes:?}"
    );
    for other in [
        "ERR-BUNDLE-6115",
        "ERR-STORAGE-4007",
        "ERR-CONFIG-7001",
        "ERR-EXPORT-6001",
        "ERR-INTERNAL-9004",
    ] {
        assert!(
            !codes.contains(&other),
            "`{other}` belongs to another family"
        );
    }
}
