//! The name-parsing engine: normalization, bracketed-annotation extraction,
//! pluggable pre/post regex patterns, longest-known-name-first matching with
//! prefix/postfix adjacency handling, and the compound fix-pattern phase.
//!
//! See the crate-level docs for the two documented deviations from upstream
//! (`AdjacencyMode` and the working fix-pattern phase).

pub(crate) mod fix_phase;
pub(crate) mod name_phase;
pub(crate) mod other_info;
pub(crate) mod regex_phase;
pub(crate) mod state;

/// How a matched known name absorbs an adjacent prefix/postfix word.
///
/// Tracing upstream's actual runtime behavior (as opposed to its README,
/// which documents something it doesn't do) shows that a prefix/postfix
/// word immediately adjacent to a matched known name gets merged into that
/// name's single output token — e.g. `"ابن علي"` against the known name
/// `"على"` produces one composing-name entry `"ابن على"`, not two separate
/// entries `"ابن"` and `"على"`. [`AdjacencyMode::Merge`] reproduces that
/// real behavior (and is the default); [`AdjacencyMode::Separate`]
/// reproduces the README's documented-but-not-actual behavior instead, for
/// callers who want the prefix/postfix word to remain its own token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdjacencyMode {
    /// Swallow an adjacent prefix/postfix word into the matched name's
    /// placeholder (upstream's real, verified runtime behavior).
    #[default]
    Merge,
    /// Leave an adjacent prefix/postfix word as its own token, only
    /// replacing the bare matched name.
    Separate,
}

use std::sync::Arc;

use crate::defaults::{DEFAULT_POSTFIXES, DEFAULT_PREFIXES};
use crate::name::ParsedName;
use crate::normalize::normalize_arabic_name;
use crate::{NameDictionary, Pattern};
use fix_phase::replace_all_fixes;
use name_phase::match_known_names;
use other_info::find_other_info;
use regex_phase::process_with_regex;
use state::MatchState;

type Normalizer = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// A configured name parser: normalizer, default-or-custom prefix/postfix
/// word lists, pluggable pre/post regex patterns, and an [`AdjacencyMode`].
///
/// Construction (via [`NameParserBuilder`]) is infallible given already-
/// valid [`Pattern`]s — [`Pattern::new`] is the crate's one fallible
/// operation, surfacing invalid caller-supplied regex at the point it's
/// compiled. [`NameParser::parse`] itself never fails.
pub struct NameParser {
    prefixes: Vec<String>,
    postfixes: Vec<String>,
    pre_patterns: Vec<Pattern>,
    post_patterns: Vec<Pattern>,
    adjacency_mode: AdjacencyMode,
    normalizer: Normalizer,
}

impl Default for NameParser {
    fn default() -> Self {
        NameParserBuilder::new().build()
    }
}

impl NameParser {
    /// Start building a [`NameParser`] with custom configuration.
    pub fn builder() -> NameParserBuilder {
        NameParserBuilder::new()
    }

    /// Parse `full_name` against `dictionary`, following upstream's
    /// `Name.__init__` phases: extract bracketed annotations, normalize,
    /// apply pre-patterns, match known names longest-first (per
    /// [`AdjacencyMode`]), apply the fix-pattern phase, apply post-patterns,
    /// then resolve the remaining placeholder/plain-word text into the
    /// final composing-name list.
    pub fn parse(&self, full_name: &str, dictionary: &NameDictionary) -> ParsedName {
        let (full_name_stripped, other_info) = find_other_info(full_name);
        let full_name_normalized = (self.normalizer)(&full_name_stripped);

        let mut state = MatchState::new(full_name_normalized.clone());

        process_with_regex(&self.pre_patterns, &mut state);
        match_known_names(
            dictionary,
            &self.prefixes,
            &self.postfixes,
            self.adjacency_mode,
            &mut state,
        );
        replace_all_fixes(&self.prefixes, &self.postfixes, &mut state);
        process_with_regex(&self.post_patterns, &mut state);

        let composing_names_normalized = state
            .text
            .split(' ')
            .map(|token| {
                token
                    .parse::<u32>()
                    .ok()
                    .and_then(|id| state.composing.get(&id).cloned())
                    .unwrap_or_else(|| token.to_string())
            })
            .collect();

        ParsedName {
            full_name: full_name_stripped,
            other_info,
            full_name_normalized,
            composing_names_normalized,
        }
    }
}

/// Fluent builder for [`NameParser`].
pub struct NameParserBuilder {
    prefixes: Option<Vec<String>>,
    postfixes: Option<Vec<String>>,
    pre_patterns: Vec<Pattern>,
    post_patterns: Vec<Pattern>,
    adjacency_mode: AdjacencyMode,
    normalizer: Normalizer,
}

impl Default for NameParserBuilder {
    fn default() -> Self {
        Self {
            prefixes: None,
            postfixes: None,
            pre_patterns: Vec::new(),
            post_patterns: Vec::new(),
            adjacency_mode: AdjacencyMode::default(),
            normalizer: Arc::new(normalize_arabic_name),
        }
    }
}

impl NameParserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the default prefix word list (un-normalized; normalized
    /// once at [`Self::build`] time).
    pub fn prefixes<I, S>(mut self, prefixes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.prefixes = Some(prefixes.into_iter().map(Into::into).collect());
        self
    }

    /// Override the default postfix word list (un-normalized; normalized
    /// once at [`Self::build`] time).
    pub fn postfixes<I, S>(mut self, postfixes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.postfixes = Some(postfixes.into_iter().map(Into::into).collect());
        self
    }

    /// Patterns applied, in order, before known-name matching.
    pub fn pre_patterns(mut self, patterns: Vec<Pattern>) -> Self {
        self.pre_patterns = patterns;
        self
    }

    /// Patterns applied, in order, after the fix-pattern phase.
    pub fn post_patterns(mut self, patterns: Vec<Pattern>) -> Self {
        self.post_patterns = patterns;
        self
    }

    /// Override the normalizer (default: [`crate::normalize_arabic_name`]).
    pub fn normalizer<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.normalizer = Arc::new(f);
        self
    }

    /// Set how a matched known name absorbs an adjacent prefix/postfix word
    /// (default: [`AdjacencyMode::Merge`]).
    pub fn adjacency_mode(mut self, mode: AdjacencyMode) -> Self {
        self.adjacency_mode = mode;
        self
    }

    /// Build the [`NameParser`]. Infallible: any fallible step
    /// ([`Pattern::new`]) must already have happened to produce the
    /// `Vec<Pattern>` passed to [`Self::pre_patterns`]/[`Self::post_patterns`].
    pub fn build(self) -> NameParser {
        let normalizer = self.normalizer;
        let prefixes = self
            .prefixes
            .unwrap_or_else(|| DEFAULT_PREFIXES.iter().map(|s| s.to_string()).collect());
        let postfixes = self
            .postfixes
            .unwrap_or_else(|| DEFAULT_POSTFIXES.iter().map(|s| s.to_string()).collect());

        NameParser {
            prefixes: prefixes.iter().map(|p| normalizer(p)).collect(),
            postfixes: postfixes.iter().map(|p| normalizer(p)).collect(),
            pre_patterns: self.pre_patterns,
            post_patterns: self.post_patterns,
            adjacency_mode: self.adjacency_mode,
            normalizer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AdjacencyMode, NameParser};
    use crate::NameDictionary;

    /// The upstream README's worked example, asserted against the real
    /// (verified) runtime behavior under the default `AdjacencyMode::Merge`
    /// — upstream's README claims `["ابن", "على"]`, but the actual library
    /// merges the swallowed prefix into the matched name's token. See
    /// [`AdjacencyMode`] for why both are supported.
    #[test]
    fn readme_example_merge_mode_matches_verified_upstream_behavior() {
        let dictionary = NameDictionary::from_raw_names(
            ["على", "عبد الله", "محمد"],
            crate::normalize_arabic_name,
        );
        let parser = NameParser::default();
        let parsed = parser.parse("ابن علي [Ibn Ali]", &dictionary);

        assert_eq!(parsed.other_info, vec!["Ibn Ali"]);
        assert_eq!(parsed.full_name_normalized, "ابن على");
        assert_eq!(parsed.composing_names_normalized, vec!["ابن على"]);
    }

    /// The same example under `AdjacencyMode::Separate`, matching the
    /// README's documented (but not actually implemented upstream) output.
    #[test]
    fn readme_example_separate_mode_matches_documented_behavior() {
        let dictionary = NameDictionary::from_raw_names(
            ["على", "عبد الله", "محمد"],
            crate::normalize_arabic_name,
        );
        let parser = NameParser::builder()
            .adjacency_mode(AdjacencyMode::Separate)
            .build();
        let parsed = parser.parse("ابن علي [Ibn Ali]", &dictionary);

        assert_eq!(parsed.other_info, vec!["Ibn Ali"]);
        assert_eq!(parsed.full_name_normalized, "ابن على");
        assert_eq!(parsed.composing_names_normalized, vec!["ابن", "على"]);
    }

    #[test]
    fn unmatched_plain_words_pass_through_unchanged() {
        let dictionary = NameDictionary::from_normalized_names(["محمد"]);
        let parser = NameParser::default();
        let parsed = parser.parse("محمد الغريب", &dictionary);
        // "ي" normalizes to "ى", same as everywhere else.
        assert_eq!(parsed.composing_names_normalized, vec!["محمد", "الغرىب"]);
    }
}
