//! Reader/writer adapters over [`crate::engine::NameParser`], driving it
//! from a [`std::io::BufRead`] (feature `sync-io`) or a
//! `tokio::io::AsyncBufRead` (feature `async-io`). Both are thin shells
//! over the same [`crate::engine::NameParser::parse`] hot path — no
//! algorithm logic is duplicated between sync and async.

#[cfg(feature = "sync-io")]
pub mod sync;

#[cfg(feature = "async-io")]
pub mod asynch;
