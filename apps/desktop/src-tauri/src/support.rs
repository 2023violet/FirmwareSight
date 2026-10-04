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
//!   The store is named, never located: no surface of this product reports the directory it sits in,
//!   and Diagnostics is one of them, because prompt §19 puts an absolute path outside that payload.
//!   `product_facts` is the half Diagnostics reads from here, so the two surfaces cannot state
//!   different versions of the same product.

use std::env::consts::{ARCH, OS};
use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::ipc::{AppIdentityDto, ErrorEnvelopeDto, MainWindowPage};
use crate::{Session, next_operation_id};
use firmwaresight_storage::SCHEMA_VERSION;

/// The file the desktop opens under its own application-data directory.
///
/// A name, not a path: the path is derived at runtime in `run`'s setup and stays on the Rust side.
/// Prompt §8 keeps this name through Commit D — renaming it would be a migration and a compatibility
/// decision, and the wording a person reads for it is "local FirmwareSight data store", not the
/// historical `p0` label the file carries.
pub(crate) const STORE_FILE_NAME: &str = "firmwaresight-p0.sqlite";

/// What the binary knows about itself, gathered where a `tauri::AppHandle` is in reach.
///
/// Shared by the About page and by Diagnostics, because two copies of these four reads would be two
/// sources of truth — the exact failure the P5 artifact decision (D1: the workspace version is the
/// single source) was written against.
pub(crate) fn product_facts(app: &AppHandle) -> ProductFacts {
    let config = app.config();
    let package = app.package_info();
    ProductFacts {
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
    }
}

/// The four identity facts that come from the bundle rather than from the store.
///
/// `pub` so the Diagnostics test drives the same builder the command does, the way `Session::open` is
/// `pub` for the parity tests: the payload's shape is only worth claiming if a test can assemble one.
#[derive(Debug, Clone)]
pub struct ProductFacts {
    pub product_name: String,
    pub binary_name: String,
    pub app_version: String,
    pub identifier: String,
}

fn internal(details: String, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-INTERNAL-9001".to_owned(),
        message: "FirmwareSight hit an internal error.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(details),
        remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
    }
}

/// What this running application is: its name, its version, and where its data lives by name only.
#[tauri::command]
pub(crate) fn get_app_identity(app: AppHandle, state: State<'_, Arc<Session>>) -> AppIdentityDto {
    let facts = product_facts(&app);
    AppIdentityDto {
        product_name: facts.product_name,
        binary_name: facts.binary_name,
        app_version: facts.app_version,
        identifier: facts.identifier,
        platform: format!("{OS}-{ARCH}"),
        storage_schema_version: SCHEMA_VERSION,
        // Named by the session that opened it, so the screen states the file this run actually holds
        // rather than a constant that would still say `p0` after a rename.
        store_file_name: state.store_file_name(),
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
