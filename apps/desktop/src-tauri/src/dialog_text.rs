//! The words a native dialog shows, chosen by the UI in the reader's language and only displayed here.
//!
//! `ADR-0032` keeps Rust independent of the UI language: the shell never reads the language preference
//! and never holds a translation. Native dialogs (file pickers, Save, the replace confirmation) are drawn
//! by the operating system from strings this process passes, so before M1 they were English literals on a
//! Chinese window (NEXT45-AC20). The UI now sends the sentences it already renders, and this module's job
//! is to make that safe: every string is display-only, trimmed, stripped of control characters and bounded
//! in length, and anything missing or out of bounds falls back to the English literal the call site has
//! always used. A replace message must carry exactly one `{file}` placeholder, which is filled with the
//! file name Rust resolved, so the WebView can never make the dialog name a different file than the one
//! about to be replaced.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Longest title, filter or button label accepted, in characters.
pub const MAX_LINE_CHARS: usize = 160;
/// Longest replace message accepted, in characters, before the file name is substituted.
pub const MAX_MESSAGE_CHARS: usize = 400;
const FILE_TOKEN: &str = "{file}";
const FORMAT_TOKEN: &str = "{format}";

/// Optional, display-only dialog words. Every field falls back independently.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct DialogTextDto {
    /// The dialog's title bar.
    pub title: Option<String>,
    /// The Save dialog's file-type label; may contain `{format}`, which becomes `PNG`, `SVG`, `JSON`...
    pub filter: Option<String>,
    pub replace_title: Option<String>,
    /// Must contain `{file}` exactly once.
    pub replace_message: Option<String>,
    pub replace_confirm: Option<String>,
    pub replace_keep: Option<String>,
}

/// The four strings of a replace confirmation, resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceText {
    pub title: String,
    pub message: String,
    pub confirm: String,
    pub keep: String,
}

fn clean(value: Option<&String>, max: usize) -> Option<String> {
    let text: String = value?.chars().filter(|c| !c.is_control()).collect();
    let text = text.trim();
    if text.is_empty() || text.chars().count() > max {
        return None;
    }
    Some(text.to_owned())
}

impl DialogTextDto {
    /// The title to show, or `fallback` when the UI sent none or an unusable one.
    #[must_use]
    pub fn title_or(&self, fallback: &str) -> String {
        clean(self.title.as_ref(), MAX_LINE_CHARS).unwrap_or_else(|| fallback.to_owned())
    }

    /// The Save dialog's filter label, with `{format}` filled in. `fallback` is used as is.
    #[must_use]
    pub fn filter_or(&self, format_label: &str, fallback: String) -> String {
        match clean(self.filter.as_ref(), MAX_LINE_CHARS) {
            Some(label) if label.matches(FORMAT_TOKEN).count() <= 1 => {
                label.replace(FORMAT_TOKEN, format_label)
            }
            _ => fallback,
        }
    }

    /// The replace confirmation for `file_name`. `fallback_message` must contain `{file}` once.
    #[must_use]
    pub fn replace_for(&self, file_name: &str, fallback_message: &str) -> ReplaceText {
        let message = clean(self.replace_message.as_ref(), MAX_MESSAGE_CHARS)
            .filter(|message| message.matches(FILE_TOKEN).count() == 1)
            .unwrap_or_else(|| fallback_message.to_owned());
        let confirm = clean(self.replace_confirm.as_ref(), MAX_LINE_CHARS);
        let keep = clean(self.replace_keep.as_ref(), MAX_LINE_CHARS);
        // `tauri-plugin-dialog` decides "confirmed" by comparing the pressed button's label with the
        // confirm label (`show`, lib.rs:350). Two equal labels would turn Keep into Replace, so a pair
        // that is not clearly two different buttons falls back to the English pair as a whole.
        let (confirm, keep) = match (confirm, keep) {
            (Some(confirm), Some(keep)) if confirm.to_lowercase() != keep.to_lowercase() => {
                (confirm, keep)
            }
            _ => ("Replace it".to_owned(), "Keep mine".to_owned()),
        };
        ReplaceText {
            title: clean(self.replace_title.as_ref(), MAX_LINE_CHARS)
                .unwrap_or_else(|| "Replace existing file?".to_owned()),
            message: message.replace(FILE_TOKEN, file_name),
            confirm,
            keep,
        }
    }
}

/// The English replace message every export used before M1.
pub const REPLACE_MESSAGE_EN: &str =
    "{file} already exists. Replacing it overwrites the file you have.";

#[cfg(test)]
mod tests {
    use super::*;

    fn text() -> DialogTextDto {
        DialogTextDto {
            title: Some("选择要分析的固件文件".to_owned()),
            filter: Some("FirmwareSight {format} 报告".to_owned()),
            replace_title: Some("替换已有文件？".to_owned()),
            replace_message: Some("{file} 已存在。替换会覆盖你现有的文件。".to_owned()),
            replace_confirm: Some("替换".to_owned()),
            replace_keep: Some("保留原文件".to_owned()),
        }
    }

    #[test]
    fn localized_words_are_used_as_sent() {
        let text = text();
        assert_eq!(text.title_or("Choose"), "选择要分析的固件文件");
        assert_eq!(
            text.filter_or("PNG", "x".to_owned()),
            "FirmwareSight PNG 报告"
        );
        let replace = text.replace_for("report.png", REPLACE_MESSAGE_EN);
        assert_eq!(replace.title, "替换已有文件？");
        assert_eq!(
            replace.message,
            "report.png 已存在。替换会覆盖你现有的文件。"
        );
        assert_eq!(replace.confirm, "替换");
        assert_eq!(replace.keep, "保留原文件");
    }

    #[test]
    fn missing_words_fall_back_to_english() {
        let text = DialogTextDto::default();
        assert_eq!(text.title_or("Choose the firmware"), "Choose the firmware");
        assert_eq!(
            text.filter_or("PNG", "FirmwareSight PNG report".to_owned()),
            "FirmwareSight PNG report"
        );
        let replace = text.replace_for("a.json", REPLACE_MESSAGE_EN);
        assert_eq!(replace.title, "Replace existing file?");
        assert_eq!(
            replace.message,
            "a.json already exists. Replacing it overwrites the file you have."
        );
        assert_eq!(replace.confirm, "Replace it");
        assert_eq!(replace.keep, "Keep mine");
    }

    #[test]
    fn a_message_must_name_the_file_exactly_once() {
        for bad in ["no placeholder at all", "{file} and {file}"] {
            let text = DialogTextDto {
                replace_message: Some(bad.to_owned()),
                ..DialogTextDto::default()
            };
            assert_eq!(
                text.replace_for("x.svg", REPLACE_MESSAGE_EN).message,
                "x.svg already exists. Replacing it overwrites the file you have."
            );
        }
    }

    #[test]
    fn control_characters_are_stripped_and_blank_is_absent() {
        let text = DialogTextDto {
            title: Some("  Pick\u{0}\n a file \u{7}".to_owned()),
            ..DialogTextDto::default()
        };
        assert_eq!(text.title_or("fallback"), "Pick a file");
        let blank = DialogTextDto {
            title: Some(" \t\r\n ".to_owned()),
            ..DialogTextDto::default()
        };
        assert_eq!(blank.title_or("fallback"), "fallback");
    }

    #[test]
    fn oversized_words_are_refused_not_truncated() {
        let long = "x".repeat(MAX_LINE_CHARS + 1);
        let text = DialogTextDto {
            title: Some(long),
            ..DialogTextDto::default()
        };
        assert_eq!(text.title_or("fallback"), "fallback");
        let exact = "字".repeat(MAX_LINE_CHARS);
        let text = DialogTextDto {
            title: Some(exact.clone()),
            ..DialogTextDto::default()
        };
        assert_eq!(text.title_or("fallback"), exact);
    }

    #[test]
    fn equal_button_labels_can_never_turn_keep_into_replace() {
        for (confirm, keep) in [("替换", "替换"), ("Replace", "replace"), ("OK", "")] {
            let text = DialogTextDto {
                replace_confirm: Some(confirm.to_owned()),
                replace_keep: Some(keep.to_owned()),
                ..DialogTextDto::default()
            };
            let words = text.replace_for("a.png", REPLACE_MESSAGE_EN);
            assert_eq!(
                (words.confirm.as_str(), words.keep.as_str()),
                ("Replace it", "Keep mine")
            );
        }
        let half = DialogTextDto {
            replace_confirm: Some("替换".to_owned()),
            ..DialogTextDto::default()
        };
        let words = half.replace_for("a.png", REPLACE_MESSAGE_EN);
        assert_eq!(
            (words.confirm.as_str(), words.keep.as_str()),
            ("Replace it", "Keep mine")
        );
    }

    #[test]
    fn a_filter_with_two_format_tokens_falls_back() {
        let text = DialogTextDto {
            filter: Some("{format} {format}".to_owned()),
            ..DialogTextDto::default()
        };
        assert_eq!(text.filter_or("SVG", "fallback".to_owned()), "fallback");
    }
}
