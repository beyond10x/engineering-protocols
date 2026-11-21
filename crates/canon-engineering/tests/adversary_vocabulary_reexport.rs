//! Adversary pass 1 on `story:docs-generator-reads-files`: every public path of
//! `canon_engineering::vocabulary` at the base (794db23) still resolves to an item of the same
//! shape, written down here as the base declared it, and every one of the six moved types has one
//! definition under `crates/`.
//!
//! `vocabulary_reexport.rs` shows the registry's items are the reader's; it does not say what
//! their shape is, so a change to the reader's signatures would pass it. The signatures below are
//! copied from the base's `crates/canon-engineering/src/vocabulary.rs`, and the definition check
//! matches any visibility and any spacing, not only the one spelling that test searches for.

use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::{Path, PathBuf};

use b10x_canon::model::{ActionId, ClaimId};
use canon_engineering::vocabulary::{
    self, Category, FORMAT, Marking, Term, UnknownTerm, Vocabulary, VocabularyError,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

fn value_type<T: Debug + Clone + Copy + PartialEq + Eq + Hash + Serialize + DeserializeOwned>() {}

fn error_type<T: Debug + Clone + PartialEq + Eq + Display + std::error::Error>() {}

#[test]
fn every_base_path_has_the_base_shape() {
    let _: &'static str = FORMAT;
    assert_eq!(FORMAT, "vocabulary/1");

    value_type::<Category>();
    value_type::<Marking>();
    let _: [Category; 10] = Category::ALL;
    let _: [Marking; 4] = Marking::ALL;
    const CATEGORY: &str = Category::ArtifactKind.as_str();
    const MARKING: &str = Marking::Core.as_str();
    assert_eq!((CATEGORY, MARKING), ("artifact_kind", "core"));

    fn term_type<T: Debug + Clone + Copy + PartialEq + Eq>(_: &T) {}
    let term = Term {
        id: "change.merge",
        category: Category::ActionId,
        marking: Marking::SoftwareChange,
        meaning: "m",
    };
    term_type(&term);
    let _: fn(&Term<'static>) -> Option<ClaimId> = Term::claim_id;
    let _: fn(&Term<'static>) -> Option<ActionId> = Term::action_id;

    error_type::<UnknownTerm>();
    error_type::<VocabularyError>();
    let unknown = UnknownTerm {
        term: String::new(),
    };
    let _: String = unknown.term;
    for error in [
        VocabularyError::Parse(String::new()),
        VocabularyError::Format(String::new()),
        VocabularyError::DuplicateTerm(String::new()),
        VocabularyError::Spelling {
            id: String::new(),
            category: Category::ClaimId,
        },
        VocabularyError::NoMeaning(String::new()),
    ] {
        match error {
            VocabularyError::Parse(_)
            | VocabularyError::Format(_)
            | VocabularyError::DuplicateTerm(_)
            | VocabularyError::Spelling { .. }
            | VocabularyError::NoMeaning(_) => {}
        }
    }

    fn vocabulary_type<T: Debug + Clone + Default + PartialEq + Eq>() {}
    vocabulary_type::<Vocabulary>();
    let _: fn(&str) -> Result<Vocabulary, VocabularyError> = Vocabulary::from_yaml;
    let _: fn(&Vocabulary) -> String = Vocabulary::to_yaml;
    let _: fn(Vocabulary, &str, Category, Marking, &str) -> Vocabulary = Vocabulary::with_term;
    let _: for<'a> fn(&'a Vocabulary) -> Vec<Term<'a>> = Vocabulary::terms;
    let _: for<'a, 'b> fn(&'a Vocabulary, &'b str) -> Result<Term<'a>, UnknownTerm> =
        Vocabulary::lookup;

    let _: fn() -> &'static [Term<'static>] = vocabulary::terms;
    let _: fn(&str) -> Result<&'static Term<'static>, UnknownTerm> = vocabulary::lookup;
    assert_eq!(term.action_id(), Some(ActionId::new("change.merge")));
    assert_eq!(term.claim_id(), None);
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Whether one line of `text` declares `keyword name`, with any visibility (`pub`,
/// `pub(crate)`, none) and any spacing; comments and string literals do not start a line that
/// way.
fn declares(text: &str, keyword: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let mut rest = line.trim_start();
        if let Some(after) = rest.strip_prefix("pub") {
            rest = after.trim_start();
            if rest.starts_with('(') {
                rest = rest.split_once(')').map_or("", |(_, after)| after);
            }
            rest = rest.trim_start();
        }
        let Some(after) = rest.strip_prefix(keyword) else {
            return false;
        };
        if !after.starts_with(char::is_whitespace) {
            return false;
        }
        let after = after.trim_start();
        after.strip_prefix(name).is_some_and(|tail| {
            !tail
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        })
    })
}

#[test]
fn the_declaration_check_reads_declarations_only() {
    // Each literal is split with `concat!` so that this file never holds the contiguous text
    // `vocabulary_reexport.rs` searches every file under `crates/` for.
    assert!(declares(
        concat!("pub ", "struct Vocabulary {"),
        "struct",
        "Vocabulary"
    ));
    assert!(declares("pub  struct  Vocabulary(", "struct", "Vocabulary"));
    assert!(declares(
        "    pub(crate) struct Vocabulary;",
        "struct",
        "Vocabulary"
    ));
    assert!(declares("struct Vocabulary<T> {", "struct", "Vocabulary"));
    assert!(!declares(
        concat!("pub ", "struct VocabularyError {"),
        "struct",
        "Vocabulary"
    ));
    assert!(!declares(
        concat!("// pub ", "struct Vocabulary"),
        "struct",
        "Vocabulary"
    ));
    assert!(!declares(
        concat!("    let needle = \"pub ", "struct Vocabulary\";"),
        "struct",
        "Vocabulary"
    ));
}

#[test]
fn each_moved_type_has_one_definition_in_the_reader_crate() {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    rust_files(&crates, &mut files);
    let texts: Vec<(PathBuf, String)> = files
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            (path, text)
        })
        .collect();
    for (keyword, name) in [
        ("struct", "Vocabulary"),
        ("enum", "Category"),
        ("enum", "Marking"),
        ("struct", "Term"),
        ("struct", "UnknownTerm"),
        ("enum", "VocabularyError"),
    ] {
        let found: Vec<&PathBuf> = texts
            .iter()
            .filter(|(_, text)| declares(text, keyword, name))
            .map(|(path, _)| path)
            .collect();
        assert_eq!(found.len(), 1, "`{keyword} {name}` in {found:?}");
        for path in found {
            assert!(
                path.components()
                    .any(|c| c.as_os_str() == "engineering-vocabulary"),
                "`{keyword} {name}` is in {path:?}, not under crates/engineering-vocabulary/"
            );
        }
    }
}
