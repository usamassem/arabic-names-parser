//! The compound fix-pattern phase, run after known-name matching, to catch
//! leftover `prefix word`, `word postfix`, and `prefix word postfix`
//! sequences that weren't in the dictionary.
//!
//! **Deviation from upstream, by design** (documented at the crate root):
//! upstream's equivalent (`_replace_all_fixes`) is dead code at runtime —
//! its dynamically built patterns use an f-string `\b` that Python's string
//! literal parser turns into a literal backspace byte rather than a regex
//! word-boundary escape, and its prefix/postfix pair is additionally built
//! in the wrong order (`itertools.product(postfixes, prefixes)`, swapped).
//! This port implements the *intended* behavior instead: real `\b` word
//! boundaries via the `regex` crate, and the documented
//! `prefix + word + postfix` ordering.
//!
//! Patterns are tried in order: every `prefix × postfix` compound pattern
//! first, then every plain `prefix + word` pattern, then every plain
//! `word + postfix` pattern. Each is run through the same
//! search-while-match-plus-replace-all-occurrences machinery as the
//! pre/post-pattern phase ([`crate::engine::regex_phase::process_with_regex`]).

use crate::Pattern;
use crate::engine::regex_phase::process_with_regex;
use crate::engine::state::MatchState;

/// Collapse leftover `prefix word postfix` / `prefix word` / `word postfix`
/// sequences in `state.text` into placeholders.
pub(crate) fn replace_all_fixes(prefixes: &[String], postfixes: &[String], state: &mut MatchState) {
    let mut patterns = Vec::new();

    for prefix in prefixes {
        for postfix in postfixes {
            let pattern = format!(
                r"\b({}\s[^\d\s]+\s{})\b",
                regex::escape(prefix),
                regex::escape(postfix)
            );
            if let Ok(p) = Pattern::new(&pattern, 1) {
                patterns.push(p);
            }
        }
    }

    for prefix in prefixes {
        let pattern = format!(r"\b({}\s[^\d\s]+)", regex::escape(prefix));
        if let Ok(p) = Pattern::new(&pattern, 1) {
            patterns.push(p);
        }
    }

    for postfix in postfixes {
        let pattern = format!(r"([^\d\s]+\s{})\b", regex::escape(postfix));
        if let Ok(p) = Pattern::new(&pattern, 1) {
            patterns.push(p);
        }
    }

    process_with_regex(&patterns, state);
}

#[cfg(test)]
mod tests {
    use super::replace_all_fixes;
    use crate::engine::state::MatchState;

    fn prefixes() -> Vec<String> {
        vec!["ابن".to_string()]
    }

    fn postfixes() -> Vec<String> {
        vec!["الله".to_string()]
    }

    #[test]
    fn compound_prefix_word_postfix_collapses_to_one_placeholder() {
        // Proves the working phase does something upstream's buggy version
        // never can: collapse a full prefix+word+postfix compound.
        let mut state = MatchState::new("ابن فلان الله".to_string());
        replace_all_fixes(&prefixes(), &postfixes(), &mut state);
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"ابن فلان الله".to_string()));
    }

    #[test]
    fn plain_prefix_word_collapses_when_no_postfix_present() {
        let mut state = MatchState::new("ابن فلان".to_string());
        replace_all_fixes(&prefixes(), &postfixes(), &mut state);
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"ابن فلان".to_string()));
    }

    #[test]
    fn plain_word_postfix_collapses_when_no_prefix_present() {
        let mut state = MatchState::new("فلان الله".to_string());
        replace_all_fixes(&prefixes(), &postfixes(), &mut state);
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"فلان الله".to_string()));
    }

    #[test]
    fn unrelated_text_is_left_untouched() {
        let mut state = MatchState::new("محمد وعلى".to_string());
        replace_all_fixes(&prefixes(), &postfixes(), &mut state);
        assert_eq!(state.text, "محمد وعلى");
        assert!(state.composing.is_empty());
    }
}
