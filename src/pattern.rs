//! Pluggable regex patterns used by the pre/post-pattern phases
//! ([`crate::engine`]), mirroring upstream's `Pattern` class: a regex plus
//! the capture group index to extract on each match.

use regex::Regex;

/// A regex paired with the capture group index to extract from each match.
///
/// Construction is the crate's only fallible operation — once a `Pattern` is
/// built, matching against it never fails.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub(crate) regex: Regex,
    pub(crate) group: usize,
}

impl Pattern {
    /// Compile `pattern` and pair it with `group`, the capture group index
    /// to extract on each match (group `0` is the whole match).
    pub fn new(pattern: &str, group: usize) -> Result<Self, regex::Error> {
        Ok(Self {
            regex: Regex::new(pattern)?,
            group,
        })
    }

    /// The compiled regex.
    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    /// The capture group index extracted on each match.
    pub fn group(&self) -> usize {
        self.group
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    #[test]
    fn builds_from_valid_pattern() {
        let pattern = Pattern::new(r"(\d+)", 1).expect("valid regex");
        let caps = pattern.regex().captures("abc123").unwrap();
        assert_eq!(caps.get(pattern.group()).unwrap().as_str(), "123");
    }

    #[test]
    fn rejects_invalid_pattern() {
        assert!(Pattern::new(r"(", 1).is_err());
    }
}
