//! The result of parsing one name, mirroring upstream's `Name` class.

use std::fmt;

/// The result of [`crate::engine::NameParser::parse`]: a raw full name split
/// into its bracketed annotations and decomposed into an ordered list of
/// composing-name tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedName {
    /// The input with any bracketed annotations (`[...]`, `{...}`, `(...)`,
    /// `<...>`) removed.
    pub full_name: String,
    /// The non-empty bracketed annotation contents, in order of appearance.
    pub other_info: Vec<String>,
    /// [`Self::full_name`] run through the configured normalizer.
    pub full_name_normalized: String,
    /// The ordered decomposition of [`Self::full_name_normalized`] into
    /// known-name/prefix/postfix/pattern tokens, with any unmatched plain
    /// word passed through unchanged.
    pub composing_names_normalized: Vec<String>,
}

impl fmt::Display for ParsedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => [{}]",
            self.full_name_normalized,
            self.composing_names_normalized.join(" | ")
        )?;
        if !self.other_info.is_empty() {
            write!(f, " (other: [{}])", self.other_info.join(", "))?;
        }
        Ok(())
    }
}
