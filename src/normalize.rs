//! Arabic name normalization, mirroring the upstream `normalize_arabic_name`:
//! unify common letter-shape variants, strip diacritics (tashkeel), drop
//! punctuation while keeping bracket characters (used by [`crate::engine`]'s
//! `OtherInfo` extraction), and collapse whitespace.

const LETTER_VARIANTS: &[(char, char)] = &[
    ('أ', 'ا'),
    ('إ', 'ا'),
    ('آ', 'ا'),
    ('ئ', 'ى'),
    ('ي', 'ى'),
    ('ؤ', 'و'),
    ('ة', 'ه'),
];

const TASHKEEL: &[char] = &[
    'ً', // tanween fatha
    'ٌ', // tanween damma
    'ٍ', // tanween kasra
    'َ', // fatha
    'ُ', // damma
    'ِ', // kasra
    'ّ', // shadda
    'ْ', // sokon
];

const BRACKET_CHARS: &str = "[]{}()<>";

/// Normalize an Arabic name: unify letter-shape variants, strip tashkeel,
/// drop punctuation (keeping bracket characters and `_`), and collapse
/// whitespace to single spaces, trimmed.
pub fn normalize_arabic_name(name: &str) -> String {
    let trimmed = name.trim();

    let unified: String = trimmed
        .chars()
        .map(|c| {
            LETTER_VARIANTS
                .iter()
                .find(|(from, _)| *from == c)
                .map(|(_, to)| *to)
                .unwrap_or(c)
        })
        .filter(|c| !TASHKEEL.contains(c))
        .collect();

    let cleaned: String = unified
        .chars()
        .filter(|c| {
            c.is_alphanumeric() || c.is_whitespace() || *c == '_' || BRACKET_CHARS.contains(*c)
        })
        .collect();

    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::normalize_arabic_name;

    #[test]
    fn unifies_alef_variants() {
        assert_eq!(normalize_arabic_name("أحمد"), "احمد");
        assert_eq!(normalize_arabic_name("إحمد"), "احمد");
        assert_eq!(normalize_arabic_name("آحمد"), "احمد");
    }

    #[test]
    fn unifies_hamza_variants() {
        assert_eq!(normalize_arabic_name("مسئول"), "مسىول");
        assert_eq!(normalize_arabic_name("مؤمن"), "مومن");
    }

    #[test]
    fn unifies_ya_and_alef_maksura() {
        assert_eq!(normalize_arabic_name("علي"), "على");
        assert_eq!(normalize_arabic_name("على"), "على");
    }

    #[test]
    fn unifies_ta_marbuta() {
        assert_eq!(normalize_arabic_name("فاطمة"), "فاطمه");
    }

    #[test]
    fn strips_each_tashkeel_mark() {
        assert_eq!(normalize_arabic_name("مُحَمَّدٌ"), "محمد");
        assert_eq!(normalize_arabic_name("كِتَابٍ"), "كتاب");
        assert_eq!(normalize_arabic_name("رَجًلاً"), "رجلا");
        assert_eq!(normalize_arabic_name("سُكُوْن"), "سكون");
    }

    #[test]
    fn strips_punctuation_but_keeps_brackets() {
        // Comma and dash are punctuation and get stripped; brackets/parens
        // are explicitly exempt and survive untouched (letters inside are
        // still normalized, same as anywhere else).
        assert_eq!(
            normalize_arabic_name("محمد، [ملاحظة] (ابن-علي)"),
            "محمد [ملاحظه] (ابنعلى)"
        );
    }

    #[test]
    fn collapses_whitespace_and_trims() {
        assert_eq!(normalize_arabic_name("  محمد   بن    علي  "), "محمد بن على");
    }
}
