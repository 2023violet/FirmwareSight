//! The desktop's local store, named once (ADR-0042 D2).
//!
//! The desktop and `fwsight analyze --record` must write the same file and file builds under the same
//! project, or the CLI's builds would never appear in the desktop's History. These names used to live in
//! the desktop crate alone; they live here so both read one definition.

use std::path::PathBuf;

/// The project the desktop files every analysis under.
pub const LOCAL_PROJECT_ID: &str = "local-desktop";
/// That project's display name.
pub const LOCAL_PROJECT_NAME: &str = "Local analyses";
/// The store's file name inside the application-data folder. Renaming it would be a migration and a
/// compatibility decision, so the historical `p0` stays.
pub const STORE_FILE_NAME: &str = "firmwaresight-p0.sqlite";
/// The desktop application's identifier (`tauri.conf.json`), which names its data folder.
pub const DESKTOP_APP_IDENTIFIER: &str = "com.firmwaresight.desktop";

/// Where the desktop keeps its store on this machine: `<data dir>/<identifier>/<file>`.
///
/// `<data dir>` is the folder Tauri's `app_data_dir()` starts from: `%APPDATA%` on Windows,
/// `~/Library/Application Support` on macOS, and `$XDG_DATA_HOME` or `~/.local/share` elsewhere. `None`
/// when the environment names no such folder.
#[must_use]
pub fn default_store_path() -> Option<PathBuf> {
    store_path_for(std::env::consts::OS, |name| std::env::var_os(name))
}

/// [`default_store_path`] for a named OS and environment, so every platform's rule is testable on any one.
pub fn store_path_for(
    os: &str,
    env: impl Fn(&str) -> Option<std::ffi::OsString>,
) -> Option<PathBuf> {
    // A relative or empty value is not a folder anyone configured; it is ignored, as the XDG rules say.
    let absolute = |name: &str| {
        env(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    let data_dir = match os {
        "windows" => absolute("APPDATA"),
        "macos" => absolute("HOME").map(|home| home.join("Library").join("Application Support")),
        _ => absolute("XDG_DATA_HOME")
            .or_else(|| absolute("HOME").map(|home| home.join(".local").join("share"))),
    }?;
    Some(data_dir.join(DESKTOP_APP_IDENTIFIER).join(STORE_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(*value))
        }
    }

    #[cfg(not(windows))]
    fn tail(path: &std::path::Path) -> Vec<String> {
        path.components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn each_platform_resolves_the_desktop_data_folder() {
        #[cfg(windows)]
        {
            let path = store_path_for(
                "windows",
                env(&[("APPDATA", r"C:\Users\a\AppData\Roaming")]),
            )
            .expect("a path");
            assert!(
                path.ends_with(
                    r"AppData\Roaming\com.firmwaresight.desktop\firmwaresight-p0.sqlite"
                )
            );
        }
        #[cfg(not(windows))]
        {
            let mac = store_path_for("macos", env(&[("HOME", "/Users/a")])).expect("a path");
            assert_eq!(
                tail(&mac),
                [
                    "/",
                    "Users",
                    "a",
                    "Library",
                    "Application Support",
                    DESKTOP_APP_IDENTIFIER,
                    STORE_FILE_NAME
                ]
            );
            let xdg = store_path_for(
                "linux",
                env(&[("XDG_DATA_HOME", "/data"), ("HOME", "/home/a")]),
            )
            .expect("a path");
            assert_eq!(
                tail(&xdg),
                ["/", "data", DESKTOP_APP_IDENTIFIER, STORE_FILE_NAME]
            );
            let fallback = store_path_for(
                "linux",
                env(&[("XDG_DATA_HOME", "relative"), ("HOME", "/home/a")]),
            )
            .expect("a path");
            assert_eq!(
                tail(&fallback),
                [
                    "/",
                    "home",
                    "a",
                    ".local",
                    "share",
                    DESKTOP_APP_IDENTIFIER,
                    STORE_FILE_NAME
                ]
            );
        }
    }

    #[test]
    fn no_folder_in_the_environment_is_no_path() {
        assert_eq!(store_path_for("windows", env(&[])), None);
        assert_eq!(store_path_for("linux", env(&[("HOME", "")])), None);
        assert_eq!(
            store_path_for("macos", env(&[("HOME", "relative/home")])),
            None
        );
    }
}
