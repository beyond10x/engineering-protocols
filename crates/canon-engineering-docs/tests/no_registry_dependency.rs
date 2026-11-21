//! Acceptance for `story:docs-generator-reads-files`: the docs generator reads `protocols/` and
//! `protocols/vocabulary.yaml` as files and links Canon; it does not depend on
//! `b10x-canon-engineering`. The vocabulary reader it uses is the small crate
//! `b10x-engineering-vocabulary`.

use std::path::{Path, PathBuf};

const REGISTRY_PACKAGE: &str = "b10x-canon-engineering";
const READER_PACKAGE: &str = "b10x-engineering-vocabulary";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn manifest() -> toml::Table {
    let path = manifest_dir().join("Cargo.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    toml::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn dependency_names(manifest: &toml::Table, section: &str) -> Vec<String> {
    manifest
        .get(section)
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

#[test]
fn the_manifest_lists_no_registry_crate_in_any_dependency_table() {
    let manifest = manifest();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let names = dependency_names(&manifest, section);
        assert!(
            !names.iter().any(|name| name == REGISTRY_PACKAGE),
            "[{section}] lists {REGISTRY_PACKAGE}: {names:?}"
        );
    }
}

#[test]
fn the_manifest_lists_the_vocabulary_reader() {
    let names = dependency_names(&manifest(), "dependencies");
    assert!(
        names.iter().any(|name| name == READER_PACKAGE),
        "[dependencies] does not list {READER_PACKAGE}: {names:?}"
    );
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn no_source_file_names_the_registry_library() {
    let mut files = Vec::new();
    rust_files(&manifest_dir().join("src"), &mut files);
    assert!(!files.is_empty(), "no file under src/");
    let needle = format!("{}::", "canon_engineering");
    let offenders: Vec<String> = files
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
                .contains(&needle)
        })
        .map(|path| path.display().to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "files under src/ name `{needle}`: {offenders:?}"
    );
}
