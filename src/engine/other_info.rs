//! Extraction of bracketed annotations (`[...]`, `{...}`, `(...)`, `<...>`)
//! from a raw full name, mirroring upstream's `_find_other_info`.
//!
//! Matching is lenient about bracket *kind*, like upstream: any opening
//! bracket character followed by non-bracket content followed by any
//! closing bracket character is treated as an annotation span, regardless
//! of whether the opening/closing kinds actually pair up (e.g. `"[foo)"` is
//! still extracted).

use std::sync::LazyLock;

use regex::Regex;

static OTHER_INFO_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\[{(<]([^\[\]{}()<>]*)[\]})>]").unwrap());

/// Split `full_name` into the text with bracketed spans removed, plus the
/// non-empty bracket contents extracted, in order of appearance.
pub fn find_other_info(full_name: &str) -> (String, Vec<String>) {
    let other_info = OTHER_INFO_PATTERN
        .captures_iter(full_name)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let stripped = OTHER_INFO_PATTERN.replace_all(full_name, "").into_owned();

    (stripped, other_info)
}

#[cfg(test)]
mod tests {
    use super::find_other_info;

    #[test]
    fn extracts_square_bracket_annotation() {
        let (full_name, other_info) = find_other_info("ابن علي [Ibn Ali]");
        assert_eq!(full_name, "ابن علي ");
        assert_eq!(other_info, vec!["Ibn Ali"]);
    }

    #[test]
    fn extracts_multiple_annotations_in_order() {
        let (full_name, other_info) = find_other_info("محمد (note1) علي [note2]");
        assert_eq!(full_name, "محمد  علي ");
        assert_eq!(other_info, vec!["note1", "note2"]);
    }

    #[test]
    fn ignores_empty_brackets() {
        let (full_name, other_info) = find_other_info("محمد []");
        assert_eq!(full_name, "محمد ");
        assert!(other_info.is_empty());
    }

    #[test]
    fn is_lenient_about_mismatched_bracket_kinds() {
        let (full_name, other_info) = find_other_info("محمد [foo) علي");
        assert_eq!(full_name, "محمد  علي");
        assert_eq!(other_info, vec!["foo"]);
    }

    #[test]
    fn passes_through_unannotated_names() {
        let (full_name, other_info) = find_other_info("محمد بن علي");
        assert_eq!(full_name, "محمد بن علي");
        assert!(other_info.is_empty());
    }
}
