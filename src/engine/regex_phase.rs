//! The pre/post-pattern phase, mirroring upstream's
//! `_process_names_with_regex`: for each pattern, repeatedly search the
//! working text and replace **every occurrence** of the matched capture
//! group with a fresh numeric placeholder, until the pattern no longer
//! matches, then move to the next pattern.
//!
//! The "replace every occurrence of the matched text" behavior is a
//! faithful port of upstream's `fullNameNormalized.replace(group,
//! str(index))` (Python's `str.replace` with no count defaults to
//! replacing all occurrences) — not just the located match. This is a
//! reachable, user-visible quirk for callers supplying custom
//! [`crate::Pattern`]s, so it is preserved rather than "fixed".

use crate::Pattern;
use crate::engine::state::MatchState;

/// Apply each pattern in order, repeatedly matching and placeholder-replacing
/// until it stops matching, before moving to the next pattern.
pub(crate) fn process_with_regex(patterns: &[Pattern], state: &mut MatchState) {
    for pattern in patterns {
        while let Some(caps) = pattern.regex().captures(&state.text) {
            // A capture group that didn't participate (or doesn't exist) is
            // treated the same as "no match", avoiding an infinite loop on
            // a pattern whose top-level match persists but whose group never
            // fills in.
            let Some(matched) = caps.get(pattern.group()) else {
                break;
            };
            let matched = matched.as_str().to_string();
            let id = state.allocate_placeholder(matched.clone());
            state.text = state.text.replace(&matched, &id.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::process_with_regex;
    use crate::Pattern;
    use crate::engine::state::MatchState;

    #[test]
    fn replaces_all_occurrences_of_the_matched_text() {
        let pattern = Pattern::new(r"(foo)", 1).unwrap();
        let mut state = MatchState::new("foo bar foo".to_string());
        process_with_regex(&[pattern], &mut state);
        assert_eq!(state.text, "0 bar 0");
        assert_eq!(state.composing.get(&0), Some(&"foo".to_string()));
    }

    #[test]
    fn applies_patterns_in_order_with_increasing_ids() {
        let p1 = Pattern::new(r"(foo)", 1).unwrap();
        let p2 = Pattern::new(r"(bar)", 1).unwrap();
        let mut state = MatchState::new("foo bar".to_string());
        process_with_regex(&[p1, p2], &mut state);
        assert_eq!(state.text, "0 1");
        assert_eq!(state.composing.get(&0), Some(&"foo".to_string()));
        assert_eq!(state.composing.get(&1), Some(&"bar".to_string()));
    }

    #[test]
    fn loops_a_single_pattern_across_distinct_matches() {
        // Each application of the pattern removes one distinct match from
        // the text (as a placeholder), so a pattern with several distinct
        // matches keeps looping — picking up a new one each time — until no
        // match remains.
        let pattern = Pattern::new(r"(cat.)", 1).unwrap();
        let mut state = MatchState::new("catX dog catY".to_string());
        process_with_regex(&[pattern], &mut state);
        assert_eq!(state.text, "0 dog 1");
        assert_eq!(state.composing.get(&0), Some(&"catX".to_string()));
        assert_eq!(state.composing.get(&1), Some(&"catY".to_string()));
    }

    #[test]
    fn no_match_leaves_state_untouched() {
        let pattern = Pattern::new(r"(zzz)", 1).unwrap();
        let mut state = MatchState::new("foo bar".to_string());
        process_with_regex(&[pattern], &mut state);
        assert_eq!(state.text, "foo bar");
        assert!(state.composing.is_empty());
    }
}
