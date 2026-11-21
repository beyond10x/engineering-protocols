//! The engineering vocabulary: the names the engineering protocols use for their terms.
//!
//! The vocabulary is data. Every term a built-in protocol names is declared once in
//! `protocols/vocabulary.yaml`, with its category and its marking, except `support.triage/1`'s,
//! `protocol.adopt/1`'s and the few the spelling rule of `b10x-engineering-vocabulary` refuses
//! (`tests/builtin_vocabulary_coverage.rs` names them, `NOT_SPELLABLE`). The typed reader of that
//! document, [`Vocabulary`] and the types it returns, is the crate `b10x-engineering-vocabulary`
//! (`crates/engineering-vocabulary`) and is re-exported here; the released file is compiled into
//! this library, and [`terms`] and [`lookup`] read it. [`Vocabulary::from_yaml`] reads any
//! vocabulary document given as text, and [`Vocabulary::to_yaml`] writes one.
//!
//! This module holds names and what they mean, nothing more: what a claim, a piece of evidence or
//! an obligation *is*, and how one is decided, is Canon's. The spelling rules are the reader's.

use std::sync::LazyLock;

pub use engineering_vocabulary::{
    Category, FORMAT, Marking, Term, UnknownTerm, Vocabulary, VocabularyError,
};

/// The engineering vocabulary this crate releases, as compiled into this library.
///
/// The path leaves the package on purpose: the released data lives at the repository root,
/// beside the protocols, and this crate embeds it rather than holding a copy.
const RELEASED: &str = include_str!("../../../protocols/vocabulary.yaml");

static VOCABULARY: LazyLock<Vocabulary> = LazyLock::new(|| {
    Vocabulary::from_yaml(RELEASED)
        .unwrap_or_else(|e| panic!("the released protocols/vocabulary.yaml: {e}"))
});

static TERMS: LazyLock<Vec<Term<'static>>> = LazyLock::new(|| VOCABULARY.terms());

/// Every term of the released vocabulary, in file order.
pub fn terms() -> &'static [Term<'static>] {
    &TERMS
}

/// The released vocabulary's entry `id` names, or a refusal naming `id`.
pub fn lookup(id: &str) -> Result<&'static Term<'static>, UnknownTerm> {
    TERMS
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| UnknownTerm {
            term: id.to_owned(),
        })
}
