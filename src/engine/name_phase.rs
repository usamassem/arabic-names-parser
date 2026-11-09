//! Longest-known-name-first matching with prefix/postfix adjacency,
//! mirroring upstream's `_parse_names_from_list` known-name loop
//! (`_add_prefix_to_name`/`_add_postfix_to_name`/`_replace_name`).
//!
//! Upstream locates each occurrence of a known name via plain substring
//! search, then looks at the word immediately before/after that occurrence
//! to see whether it's a configured prefix/postfix, extending the match if
//! so, then splices the (possibly extended) span out of the text and
//! records it under a fresh placeholder — repeating per name until no more
//! occurrences remain, trying names longest-first.
//!
//! This reimplements that index arithmetic via straightforward word-
//! boundary lookups on the (always single-space-normalized) working text,
//! rather than porting upstream's position-offset formulas line-for-line —
//! verified equivalent for the normalized-text case these phases always
//! operate on. It also uses a direct byte-range splice instead of
//! upstream's "pad with spaces then replace first occurrence of the padded
//! string" trick: since the span to replace was just located via a
//! leftmost `find`, splicing by position produces an identical result and
//! needs no padding.
//!
//! See [`crate::engine::AdjacencyMode`] for the two supported ways to
//! record a prefix/postfix adjacency.

use crate::NameDictionary;
use crate::engine::AdjacencyMode;
use crate::engine::state::MatchState;

/// Match every known name in `dictionary` (already longest-first) against
/// `state.text`, swallowing or skipping adjacent prefix/postfix words per
/// `adjacency_mode`, until each name is exhausted.
pub(crate) fn match_known_names(
    dictionary: &NameDictionary,
    prefixes: &[String],
    postfixes: &[String],
    adjacency_mode: AdjacencyMode,
    state: &mut MatchState,
) {
    for name in dictionary.as_slice() {
        if name.is_empty() {
            continue;
        }
        while let Some(start) = state.text.find(name.as_str()) {
            let end = start + name.len();

            let (replace_start, replace_end, matched) = match adjacency_mode {
                AdjacencyMode::Merge => extend_match(&state.text, start, end, prefixes, postfixes),
                AdjacencyMode::Separate => (start, end, name.clone()),
            };

            let id = state.allocate_placeholder(matched);
            state
                .text
                .replace_range(replace_start..replace_end, &id.to_string());
        }
    }
}

/// Extend `[start, end)` to swallow an adjacent prefix word before it and/or
/// an adjacent postfix word after it, returning the extended span and its
/// text.
fn extend_match(
    text: &str,
    start: usize,
    end: usize,
    prefixes: &[String],
    postfixes: &[String],
) -> (usize, usize, String) {
    let bare = &text[start..end];

    let prefix = find_adjacent_prefix(text, start, prefixes);
    let postfix = find_adjacent_postfix(text, end, postfixes);

    let new_start = match prefix {
        Some(p) => start - (p.len() + 1),
        None => start,
    };
    let new_end = match postfix {
        Some(p) => end + (p.len() + 1),
        None => end,
    };

    let mut matched = String::new();
    if let Some(p) = prefix {
        matched.push_str(p);
        matched.push(' ');
    }
    matched.push_str(bare);
    if let Some(p) = postfix {
        matched.push(' ');
        matched.push_str(p);
    }

    (new_start, new_end, matched)
}

/// The word immediately before byte offset `position` in `text`, if it is
/// one of `prefixes`. Requires a single space directly before `position`
/// (i.e. the match starts a new word) — a match that starts mid-word has no
/// adjacent prefix by definition.
fn find_adjacent_prefix<'a>(
    text: &str,
    position: usize,
    prefixes: &'a [String],
) -> Option<&'a str> {
    if position == 0 {
        return None;
    }
    let before = text[..position].strip_suffix(' ')?;
    let word_start = before.rfind(' ').map(|i| i + 1).unwrap_or(0);
    let word = &before[word_start..];
    prefixes
        .iter()
        .find(|p| p.as_str() == word)
        .map(String::as_str)
}

/// The word immediately after byte offset `position` in `text`, if it is
/// one of `postfixes`. The word may run to a following space or to the end
/// of the text (both count as a complete word boundary).
fn find_adjacent_postfix<'a>(
    text: &str,
    position: usize,
    postfixes: &'a [String],
) -> Option<&'a str> {
    let after = text.get(position..)?;
    let after = after.strip_prefix(' ')?;
    let word_end = after.find(' ').unwrap_or(after.len());
    let word = &after[..word_end];
    postfixes
        .iter()
        .find(|p| p.as_str() == word)
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::match_known_names;
    use crate::NameDictionary;
    use crate::engine::AdjacencyMode;
    use crate::engine::state::MatchState;

    fn prefixes() -> Vec<String> {
        vec!["ابن".to_string(), "ابو".to_string()]
    }

    fn postfixes() -> Vec<String> {
        vec!["الله".to_string()]
    }

    #[test]
    fn merge_mode_swallows_leading_prefix_into_one_placeholder() {
        let dict = NameDictionary::from_normalized_names(["على"]);
        let mut state = MatchState::new("ابن على".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Merge,
            &mut state,
        );
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"ابن على".to_string()));
    }

    #[test]
    fn separate_mode_leaves_prefix_as_its_own_token() {
        let dict = NameDictionary::from_normalized_names(["على"]);
        let mut state = MatchState::new("ابن على".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Separate,
            &mut state,
        );
        assert_eq!(state.text, "ابن 0");
        assert_eq!(state.composing.get(&0), Some(&"على".to_string()));
    }

    #[test]
    fn merge_mode_swallows_trailing_postfix() {
        let dict = NameDictionary::from_normalized_names(["عبد"]);
        let mut state = MatchState::new("عبد الله".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Merge,
            &mut state,
        );
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"عبد الله".to_string()));
    }

    #[test]
    fn merge_mode_swallows_both_prefix_and_postfix() {
        let dict = NameDictionary::from_normalized_names(["عبد"]);
        let mut state = MatchState::new("ابن عبد الله".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Merge,
            &mut state,
        );
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"ابن عبد الله".to_string()));
    }

    #[test]
    fn no_adjacent_fix_words_matches_bare_name_only() {
        let dict = NameDictionary::from_normalized_names(["محمد"]);
        let mut state = MatchState::new("محمد".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Merge,
            &mut state,
        );
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"محمد".to_string()));
    }

    #[test]
    fn match_at_string_end_has_no_false_postfix() {
        let dict = NameDictionary::from_normalized_names(["محمد"]);
        let mut state = MatchState::new("ابن محمد".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Merge,
            &mut state,
        );
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"ابن محمد".to_string()));
    }

    #[test]
    fn repeats_per_name_until_no_more_occurrences() {
        let dict = NameDictionary::from_normalized_names(["محمد"]);
        let mut state = MatchState::new("محمد و محمد".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Merge,
            &mut state,
        );
        assert_eq!(state.text, "0 و 1");
        assert_eq!(state.composing.get(&0), Some(&"محمد".to_string()));
        assert_eq!(state.composing.get(&1), Some(&"محمد".to_string()));
    }

    #[test]
    fn longest_names_are_tried_first() {
        let dict = NameDictionary::from_normalized_names(["عبد", "عبد الله"]);
        let mut state = MatchState::new("عبد الله".to_string());
        match_known_names(
            &dict,
            &prefixes(),
            &postfixes(),
            AdjacencyMode::Merge,
            &mut state,
        );
        // "عبد الله" (longer) is tried first and consumes the whole string,
        // leaving nothing for the shorter "عبد" to match afterwards.
        assert_eq!(state.text, "0");
        assert_eq!(state.composing.get(&0), Some(&"عبد الله".to_string()));
    }
}
