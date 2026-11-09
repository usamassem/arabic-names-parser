//! Low-level, I/O-agnostic normalization and decomposition of Arabic
//! personal names into prefix, known-name, and postfix components.
//!
//! This is a Rust port of the algorithm in
//! [`anazhmetdin/ArabicNamesParser`](https://github.com/anazhmetdin/ArabicNamesParser),
//! restructured as a pure, synchronous engine ([`NameParser::parse`]) that
//! takes a `&str` and a caller-supplied [`NameDictionary`] and returns a
//! [`ParsedName`] — no I/O beyond the algorithm itself. Optional
//! feature-gated modules build on top of that core:
//!
//! - `sync-io`: a [`std::io::BufRead`]-driven iterator and a line writer.
//! - `async-io`: the async equivalents over `tokio::io::AsyncBufRead`.
//! - `builtin-dict`: an embedded default [`NameDictionary`] sourced from
//!   the reference datasets (see [`builtin`] for attribution).
//! - `online-lookup`: a pluggable trait for augmenting the dictionary from
//!   an external source, kept out of the synchronous hot path.
//!
//! # Example
//!
//! ```
//! use arabic_names_parser::{NameDictionary, NameParser, normalize_arabic_name};
//!
//! let dictionary = NameDictionary::from_raw_names(
//!     ["على", "عبد الله", "محمد"],
//!     normalize_arabic_name,
//! );
//! let parser = NameParser::default();
//! let parsed = parser.parse("ابن علي [Ibn Ali]", &dictionary);
//!
//! assert_eq!(parsed.other_info, vec!["Ibn Ali"]);
//! assert_eq!(parsed.full_name_normalized, "ابن على");
//! assert_eq!(parsed.composing_names_normalized, vec!["ابن على"]);
//! ```
//!
//! # Documented deviations from upstream
//!
//! Upstream's own README was found, by tracing the real Python runtime
//! behavior, to describe things the library doesn't actually do:
//!
//! 1. **Prefix/postfix merging.** The README's worked example claims a
//!    swallowed prefix/postfix stays as its own output token. The real
//!    code merges it into the matched name's single token instead. This
//!    crate reproduces the *real* behavior by default
//!    ([`AdjacencyMode::Merge`]) and offers the README's documented
//!    behavior as an opt-in ([`AdjacencyMode::Separate`]) — see
//!    [`NameParserBuilder::adjacency_mode`].
//! 2. **The compound fix-pattern phase.** Upstream's `_replace_all_fixes`
//!    never fires at runtime: its dynamically built regex uses an f-string
//!    `\b` that Python's string-literal parser turns into a literal
//!    backspace byte rather than a word-boundary escape, and its
//!    prefix/postfix pair is additionally built in the wrong order. This
//!    crate implements the *intended, working* version instead — real
//!    `\b` word boundaries, correct ordering — documented in the
//!    `engine::fix_phase` module.

pub mod defaults;
pub mod dictionary;
pub mod engine;
pub mod name;
pub mod normalize;
pub mod pattern;

#[cfg(feature = "builtin-dict")]
pub mod builtin;

#[cfg(any(feature = "sync-io", feature = "async-io"))]
pub mod io;

#[cfg(feature = "online-lookup")]
pub mod online;

pub use dictionary::NameDictionary;
pub use engine::{AdjacencyMode, NameParser, NameParserBuilder};
pub use name::ParsedName;
pub use normalize::normalize_arabic_name;
pub use pattern::Pattern;
