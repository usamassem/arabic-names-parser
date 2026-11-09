//! A pluggable extension point for augmenting a [`NameDictionary`] from an
//! external ("online") source, kept deliberately **out of**
//! [`crate::engine::NameParser::parse`]'s synchronous hot path.
//!
//! [`crate::NameParser::parse`] stays pure, synchronous, and side-effect-free —
//! this module instead supports a two-pass pattern the caller drives
//! explicitly:
//!
//! 1. Parse once against the current dictionary.
//! 2. Inspect [`crate::ParsedName::composing_names_normalized`] for tokens
//!    that are plain, unmatched words (present verbatim in
//!    `full_name_normalized`, not a dictionary/pattern-derived composite).
//! 3. Call [`augment_dictionary_with_online_source`] with those leftover
//!    words; if the source confirms any, re-parse with the augmented
//!    dictionary.
//!
//! [`OnlineNameSource::lookup`] returns a boxed future rather than being an
//! `async fn` in a crate with an `async-trait`-style dependency, so this
//! module needs no HTTP client or async runtime of its own — it works
//! under whatever executor the caller already uses.

use std::future::Future;
use std::pin::Pin;

use crate::NameDictionary;
use crate::normalize::normalize_arabic_name;

/// An external source consulted for words not found in a local
/// [`NameDictionary`].
pub trait OnlineNameSource: Send + Sync {
    /// Look up `word` (already normalized). `None` means the source has no
    /// opinion; `Some(true)`/`Some(false)` confirms/rejects it as a known
    /// name.
    fn lookup<'a>(
        &'a self,
        word: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<bool>> + Send + 'a>>;
}

/// Consult `source` for each of `leftover_words` not already present in
/// `base`, returning a new [`NameDictionary`] containing `base`'s names
/// plus any the source confirmed.
pub async fn augment_dictionary_with_online_source(
    base: &NameDictionary,
    leftover_words: impl IntoIterator<Item = String>,
    source: &dyn OnlineNameSource,
) -> NameDictionary {
    let mut names: Vec<String> = base.as_slice().to_vec();

    for word in leftover_words {
        let normalized = normalize_arabic_name(&word);
        if names.iter().any(|n| n == &normalized) {
            continue;
        }
        if source.lookup(&normalized).await == Some(true) {
            names.push(normalized);
        }
    }

    NameDictionary::from_normalized_names(names)
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;

    use super::{OnlineNameSource, augment_dictionary_with_online_source};
    use crate::NameDictionary;

    struct AlwaysConfirms;
    impl OnlineNameSource for AlwaysConfirms {
        fn lookup<'a>(
            &'a self,
            _word: &'a str,
        ) -> Pin<Box<dyn Future<Output = Option<bool>> + Send + 'a>> {
            Box::pin(async { Some(true) })
        }
    }

    struct NeverConfirms;
    impl OnlineNameSource for NeverConfirms {
        fn lookup<'a>(
            &'a self,
            _word: &'a str,
        ) -> Pin<Box<dyn Future<Output = Option<bool>> + Send + 'a>> {
            Box::pin(async { None })
        }
    }

    #[tokio::test]
    async fn confirmed_words_are_added_once_normalized() {
        let base = NameDictionary::from_normalized_names(["محمد"]);
        let augmented =
            augment_dictionary_with_online_source(&base, ["علي".to_string()], &AlwaysConfirms)
                .await;
        assert!(augmented.as_slice().contains(&"على".to_string()));
    }

    #[tokio::test]
    async fn unconfirmed_words_are_not_added() {
        let base = NameDictionary::from_normalized_names(["محمد"]);
        let augmented =
            augment_dictionary_with_online_source(&base, ["فلان".to_string()], &NeverConfirms).await;
        assert_eq!(augmented.len(), 1);
    }

    #[tokio::test]
    async fn already_known_words_are_skipped() {
        let base = NameDictionary::from_normalized_names(["محمد"]);
        let augmented =
            augment_dictionary_with_online_source(&base, ["محمد".to_string()], &NeverConfirms)
                .await;
        assert_eq!(augmented.len(), 1);
    }
}
