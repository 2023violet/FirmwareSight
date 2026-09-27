//! FirmwareSight domain core.
//!
//! Headless and UI-independent: no Tauri, no SQLite, no `object` types, no Tokio.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

pub mod domain;
pub mod error;

pub use error::CoreError;
