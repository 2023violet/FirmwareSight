//! The two things the shell says about itself: the window title, and the identity a person reads on
//! the Help screen.
//!
//! Both are `AGENTS.md` 7 questions wearing different clothes.
//!
//! - **The title.** P1 shipped a window that said "FirmwareSight - Analyze" whatever page was open,
//!   because the title came from `tauri.conf.json` and nothing ever changed it (prompt §14, and
//!   finding 1 of `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md`). The obvious repair is the WebView
//!   calling `getCurrentWindow().setTitle(...)`, and that needs `core:window:allow-set-title` — a new
//!   capability, which `AGENTS.md` 9 says is a human decision, and which would hand arbitrary text to
//!   a window property. So the page name crosses the boundary as one of five closed values and Rust
//!   composes the string: the capability list stays exactly `core:default`, and a file name has no
//!   route into a title bar.
//! - **The identity.** Help has to state a version, and the UI must not guess one. Every field comes
//!   from the running application — the config it was built with, the crate version it was compiled
//!   from, the schema its store carries — so the screen cannot show a number the build did not carry.
//!   The store is named, never located: where it sits is a Diagnostics question with its own privacy
//!   test (prompt §19), not a label on an About page.

use std::env::consts::{ARCH, OS};

use tauri::{AppHandle, Manager};

use crate::ipc::{AppIdentityDto, ErrorEnvelopeDto, MainWindowPage};
use crate::next_operation_id;
use firmwaresight_storage::SCHEMA_VERSION;

/// The file the desktop opens under its own application-data directory.
///
/// A name, not a path: the path is derived at runtime in `run`'s setup and stays on the Rust side.
pub(crate) const STORE_FILE_NAME: &str = "firmwaresight-p0.sqlite";

fn internal(details: String, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-INTERNAL-9001".to_owned(),
        message: "FirmwareSight hit an internal error.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(details),
        remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
    }
}

/// What this running application is: its name, its version, where its data lives by name only.
#[tauri::command]
pub(crate) fn get_app_identity(app: AppHandle) -> AppIdentityDto {
    let config = app.config();
    let package = app.package_info();
    AppIdentityDto {
        // The bundled product name, with the crate name behind it so a screen never shows `None`
        // where a person needs to know what they are running.
        product_name: config
            .product_name
            .clone()
            .unwrap_or_else(|| package.name.clone()),
        binary_name: package.name.clone(),
        // The version this binary was compiled from. The P5 artifact decision made the workspace
        // version the single source, so this is the same string the package and the bundle carry.
        app_version: package.version.to_string(),
        identifier: config.identifier.clone(),
        platform: format!("{OS}-{ARCH}"),
        storage_schema_version: SCHEMA_VERSION,
        store_file_name: STORE_FILE_NAME.to_owned(),
    }
}

/// Name a page so the shell can put its own title on the window.
///
/// This is a synchronous command, which Tauri runs on the main thread: that is where a window
/// property belongs, and it is why no capability had to change to move the title (`AGENTS.md` 9).
#[tauri::command]
pub(crate) fn set_window_title(
    page: MainWindowPage,
    app: AppHandle,
) -> Result<(), ErrorEnvelopeDto> {
    let operation_id = next_operation_id();
    let window = app.get_webview_window("main").ok_or_else(|| {
        internal(
            "the main window was not available to be titled".to_owned(),
            &operation_id,
        )
    })?;
    // A title that could not be set is reported rather than swallowed, and says nothing about the
    // page's data, which has already been read by the time the title moves.
    window.set_title(page.window_title()).map_err(|source| {
        internal(
            format!("the window title was not accepted: {source}"),
            &operation_id,
        )
    })
}
