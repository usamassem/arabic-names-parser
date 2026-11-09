#![cfg(feature = "builtin-dict")]

use arabic_names_parser::builtin::builtin_dictionary;

#[test]
fn embeds_a_substantial_default_dictionary() {
    let dict = builtin_dictionary();
    assert!(
        dict.len() > 30_000,
        "expected the embedded dictionary to have more than 30,000 names, got {}",
        dict.len()
    );
}

#[test]
fn contains_a_well_known_name() {
    let dict = builtin_dictionary();
    assert!(dict.as_slice().iter().any(|n| n == "محمد"));
}

#[test]
fn is_sorted_longest_first() {
    let dict = builtin_dictionary();
    let lengths: Vec<usize> = dict.as_slice().iter().map(|n| n.chars().count()).collect();
    for i in 1..lengths.len() {
        assert!(
            lengths[i - 1] >= lengths[i],
            "dictionary is not sorted longest-first at index {i}"
        );
    }
}
