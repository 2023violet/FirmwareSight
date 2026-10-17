//! The terminal edition of the report grammar (R6, `03_DESIGN/09` §8).
//!
//! Every human CLI report reads in one order: a masthead naming the product and the report, the answer,
//! the facts, the evidence and the limits. This module holds the pieces they share so the three reports
//! cannot drift apart:
//!
//! - a masthead and section headings underlined with `─`, so a section is found by eye in a scrollback;
//! - an 80-column wrap with a hanging indent, the width a terminal and a code review both assume;
//! - a state tag that is always a word in brackets (`[BLOCK]`), never a colour, so the text survives a
//!   pipe, a log file and a monochrome terminal.

/// The column every human report wraps at.
pub const WIDTH: usize = 80;

/// `FirmwareSight · <report>`, ruled to the full width.
#[must_use]
pub fn masthead(report: &str) -> String {
    format!("FirmwareSight · {report}\n{}\n", "─".repeat(WIDTH))
}

/// A section title, ruled to its own length, preceded by one blank line.
#[must_use]
pub fn heading(title: &str) -> String {
    format!("\n{title}\n{}\n", "─".repeat(title.chars().count()))
}

/// A state word in brackets, padded so the text after it lines up across rows.
#[must_use]
pub fn state_tag(word: &str) -> String {
    format!("{:<10}", format!("[{word}]"))
}

/// Wrap `text` at [`WIDTH`] columns. The first line starts with `first`, the rest with `rest`.
#[must_use]
pub fn wrap(text: &str, first: &str, rest: &str) -> String {
    let mut out = String::new();
    let mut line = String::new();
    let mut indent = first;
    for word in text.split_whitespace() {
        let used = indent.chars().count() + line.chars().count();
        if !line.is_empty() && used + 1 + word.chars().count() > WIDTH {
            out.push_str(indent);
            out.push_str(&line);
            out.push('\n');
            line.clear();
            indent = rest;
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push_str(indent);
        out.push_str(&line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wrapped_paragraph_never_passes_the_width_and_keeps_its_indent() {
        let text = "word ".repeat(60);
        let wrapped = wrap(&text, "  [BLOCK]  ", "           ");
        for line in wrapped.lines() {
            assert!(line.chars().count() <= WIDTH, "{line}");
        }
        assert!(
            wrapped
                .lines()
                .skip(1)
                .all(|line| line.starts_with("           word"))
        );
    }

    #[test]
    fn a_heading_is_ruled_to_its_own_length() {
        assert_eq!(heading("Memory"), "\nMemory\n──────\n");
        assert_eq!(state_tag("N/A"), "[N/A]     ");
    }
}
