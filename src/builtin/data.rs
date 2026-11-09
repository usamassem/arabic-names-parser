//! Raw CSV text embedded at compile time. See `data/ATTRIBUTION.md` (and
//! [`super::builtin_dictionary`]'s doc comment) for source attribution and
//! the licensing caveat.

pub(crate) const SULTAN_QABOOS_CSV: &str =
    include_str!("../../data/Sultan_Qaboos_Encyclopedia_Names.csv");

pub(crate) const MUSLIM_BABY_NAMES_CSV: &str =
    include_str!("../../data/muslimbabynames_hawramani.csv");

pub(crate) const ADDITIONAL_NAMES_CSV: &str = include_str!("../../data/additional_names.csv");
