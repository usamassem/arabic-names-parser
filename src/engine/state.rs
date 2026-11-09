//! Shared working state threaded through the matching phases: the mutating
//! normalized text plus the placeholder bookkeeping, mirroring upstream's
//! `fullNameNormalized` string and `composingNames` dict.

use std::collections::HashMap;

/// Working state for one `parse()` call: the text being progressively
/// replaced with numeric placeholders, and the placeholder -> matched-text
/// map used to resolve them back at the end.
pub(crate) struct MatchState {
    pub(crate) text: String,
    pub(crate) composing: HashMap<u32, String>,
    pub(crate) next_id: u32,
}

impl MatchState {
    pub(crate) fn new(text: String) -> Self {
        Self {
            text,
            composing: HashMap::new(),
            next_id: 0,
        }
    }

    /// Record `matched` under a fresh placeholder id and return it.
    pub(crate) fn allocate_placeholder(&mut self, matched: String) -> u32 {
        let id = self.next_id;
        self.composing.insert(id, matched);
        self.next_id += 1;
        id
    }
}
