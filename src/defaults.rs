//! Default prefix/postfix word lists, mirroring upstream's `prefixes` and
//! `postfixes` globals. These are raw (un-normalized) forms; [`crate::engine`]
//! normalizes them once at [`crate::engine::NameParser`] construction time,
//! mirroring upstream's `cp_prefixes`/`cp_postfixes`.

/// Default kunya/genealogy prefixes: "son of", "father of", "mother of", "son of".
pub const DEFAULT_PREFIXES: &[&str] = &["ابن", "ابو", "ام", "بن"];

/// Default religious/honorific postfixes.
pub const DEFAULT_POSTFIXES: &[&str] = &["الله", "الدين", "النبي", "الرسول"];
