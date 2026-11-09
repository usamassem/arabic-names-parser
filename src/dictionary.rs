//! A list of known, normalized names to match against, mirroring upstream's
//! `allNamesNormalized` list sorted longest-first so that
//! [`crate::engine::NameParser::parse`] tries the most specific (longest)
//! known names before shorter ones.

/// Known normalized names, sorted by length descending (ties keep their
/// original relative order, matching Python's stable sort).
#[derive(Debug, Clone, Default)]
pub struct NameDictionary {
    names: Vec<String>,
}

impl NameDictionary {
    /// Build a dictionary from names that are already normalized.
    pub fn from_normalized_names<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut names: Vec<String> = names.into_iter().map(Into::into).collect();
        sort_longest_first(&mut names);
        Self { names }
    }

    /// Build a dictionary from raw names, normalizing each with `normalizer`.
    pub fn from_raw_names<I, S, F>(names: I, normalizer: F) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
        F: Fn(&str) -> String,
    {
        let normalized = names.into_iter().map(|n| normalizer(n.as_ref()));
        Self::from_normalized_names(normalized)
    }

    /// The known names, sorted longest-first.
    pub fn as_slice(&self) -> &[String] {
        &self.names
    }

    /// Number of known names.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether this dictionary has no known names.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

fn sort_longest_first(names: &mut [String]) {
    names.sort_by_key(|n| std::cmp::Reverse(n.chars().count()));
}

#[cfg(test)]
mod tests {
    use super::NameDictionary;

    #[test]
    fn sorts_longest_first_and_is_stable_on_ties() {
        let dict = NameDictionary::from_normalized_names(["ab", "abc", "xy", "abcd"]);
        assert_eq!(dict.as_slice(), &["abcd", "abc", "ab", "xy"]);
    }

    #[test]
    fn from_raw_names_normalizes_each_entry() {
        let dict = NameDictionary::from_raw_names(["Hello World", "Hi"], |s| s.to_lowercase());
        assert_eq!(dict.as_slice(), &["hello world", "hi"]);
    }
}
