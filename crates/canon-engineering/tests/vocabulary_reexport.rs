//! Acceptance for `story:docs-generator-reads-files`: the vocabulary reader lives in
//! `crates/engineering-vocabulary` and is the only definition of `Vocabulary`;
//! `canon_engineering::vocabulary` re-exports it next to the embedded released vocabulary, so
//! every path the module had still resolves to an item of the same shape.

use std::path::{Path, PathBuf};

use canon_engineering::vocabulary::{
    Category, FORMAT, Marking, Term, UnknownTerm, Vocabulary, VocabularyError, lookup, terms,
};

fn same_type<T>(_: &T, _: &T) {}

#[test]
fn the_registry_modules_vocabulary_is_the_readers() {
    let registry = Vocabulary::default();
    let reader = engineering_vocabulary::Vocabulary::default();
    same_type(&registry, &reader);
    let _: engineering_vocabulary::Category = Category::ArtifactKind;
    let _: engineering_vocabulary::Marking = Marking::Core;
    let _: engineering_vocabulary::VocabularyError = VocabularyError::NoMeaning(String::new());
    let _: engineering_vocabulary::UnknownTerm = UnknownTerm {
        term: String::new(),
    };
    let _: Option<engineering_vocabulary::Term<'static>> = terms().first().copied();
    assert_eq!(FORMAT, engineering_vocabulary::FORMAT);
}

#[test]
fn the_embedded_released_vocabulary_is_still_here() {
    let first: Term<'static> = *terms().first().expect("the released vocabulary has terms");
    assert_eq!(lookup(first.id), Ok(&first));
    let read = Vocabulary::from_yaml(include_str!("../../../protocols/vocabulary.yaml"))
        .expect("the released vocabulary reads");
    assert_eq!(read.terms(), terms());
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

#[test]
fn one_file_defines_vocabulary_and_it_is_in_the_reader_crate() {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    rust_files(&crates, &mut files);
    let needle = format!("pub struct {}", "Vocabulary");
    let found: Vec<PathBuf> = files
        .into_iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
                .contains(&needle)
        })
        .collect();
    assert_eq!(found.len(), 1, "`{needle}` is in {found:?}");
    assert!(
        found[0]
            .components()
            .any(|c| c.as_os_str() == "engineering-vocabulary"),
        "`{needle}` is in {:?}, not under crates/engineering-vocabulary/",
        found[0]
    );
}
