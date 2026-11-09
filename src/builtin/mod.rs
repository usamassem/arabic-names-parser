//! An embedded default [`NameDictionary`], built from three reference
//! datasets vendored under `data/` and parsed once, lazily, on first use.

mod data;

use std::sync::LazyLock;

use crate::NameDictionary;
use crate::normalize::normalize_arabic_name;

fn names_from_csv(csv_text: &str) -> impl Iterator<Item = String> + '_ {
    csv::Reader::from_reader(csv_text.as_bytes())
        .into_records()
        .filter_map(|record| record.ok())
        .filter_map(|record| record.get(0).map(str::to_string))
}

/// The embedded default dictionary: the Sultan Qaboos Encyclopedia of
/// Arabic Names, the muslimbabynames.hawramani.com dataset, and a small
/// hand-curated supplement, all vendored from
/// [`anazhmetdin/ArabicNamesParser`](https://github.com/anazhmetdin/ArabicNamesParser)
/// and normalized with [`crate::normalize_arabic_name`].
///
/// # License status
///
/// The upstream repository declares no license, and the two scraped source
/// sites do not publish explicit redistribution terms. This dictionary is
/// embedded as an explicitly accepted risk, not a claim of clear
/// redistribution rights — see `data/ATTRIBUTION.md` in this crate's
/// repository for the full note. If you plan to redistribute a build with
/// `builtin-dict` enabled, verify the licensing situation independently.
pub fn builtin_dictionary() -> &'static NameDictionary {
    static DICT: LazyLock<NameDictionary> = LazyLock::new(|| {
        let names = names_from_csv(data::SULTAN_QABOOS_CSV)
            .chain(names_from_csv(data::MUSLIM_BABY_NAMES_CSV))
            .chain(names_from_csv(data::ADDITIONAL_NAMES_CSV));
        NameDictionary::from_raw_names(names, normalize_arabic_name)
    });
    &DICT
}
