//! Adversary pass 1 on `story:docs-generator-reads-files`: the generator does not depend on
//! `b10x-canon-engineering` by any route cargo honours, not only by the three top-level tables
//! under the package's own name.
//!
//! `no_registry_dependency.rs` reads the keys of `[dependencies]`, `[dev-dependencies]` and
//! `[build-dependencies]`. A dependency under a target table (`[target.'cfg(unix)'.dependencies]`)
//! or under another key (`registry = { package = "b10x-canon-engineering", ... }`) passes it, and
//! a source file can then name the library as `use canon_engineering as registry;` or
//! `extern crate canon_engineering;`, which the text `canon_engineering::` does not match. This
//! file asks cargo instead: `cargo metadata --no-deps` reports every dependency of every
//! workspace package with its real package name, its kind and its target, and the walk follows
//! path dependencies through the workspace, as `cargo tree -p canon-engineering-docs` would.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

const DOCS_PACKAGE: &str = "canon-engineering-docs";
const REGISTRY_PACKAGE: &str = "b10x-canon-engineering";
const REGISTRY_LIBRARY: &str = "canon_engineering";
const READER_PACKAGE: &str = "b10x-engineering-vocabulary";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every workspace package's dependencies, by package name, as cargo resolves the manifests:
/// (dependency package name, kind, target, rename).
fn workspace_dependencies() -> BTreeMap<String, Vec<(String, String, String, String)>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(manifest_dir())
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
        ])
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("cargo metadata JSON");
    let text = |value: &Value| value.as_str().unwrap_or("").to_owned();
    metadata["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .map(|package| {
            let dependencies = package["dependencies"]
                .as_array()
                .expect("dependencies")
                .iter()
                .map(|dependency| {
                    (
                        text(&dependency["name"]),
                        text(&dependency["kind"]),
                        text(&dependency["target"]),
                        text(&dependency["rename"]),
                    )
                })
                .collect();
            (text(&package["name"]), dependencies)
        })
        .collect()
}

#[test]
fn no_route_through_the_workspace_reaches_the_registry_crate() {
    let packages = workspace_dependencies();
    assert!(
        packages.contains_key(DOCS_PACKAGE),
        "{DOCS_PACKAGE} is not a workspace package: {:?}",
        packages.keys().collect::<Vec<_>>()
    );
    let mut reached = BTreeSet::new();
    let mut routes = Vec::new();
    let mut queue = vec![(DOCS_PACKAGE.to_owned(), DOCS_PACKAGE.to_owned())];
    while let Some((package, route)) = queue.pop() {
        if !reached.insert(package.clone()) {
            continue;
        }
        for (name, kind, target, rename) in packages.get(&package).into_iter().flatten() {
            let is_root = package == DOCS_PACKAGE;
            if kind == "dev" && !is_root {
                continue;
            }
            let step =
                format!("{route} -> {name} (kind {kind:?}, target {target:?}, key {rename:?})");
            if name == REGISTRY_PACKAGE {
                routes.push(step.clone());
            }
            if packages.contains_key(name) {
                queue.push((name.clone(), step));
            }
        }
    }
    assert!(
        routes.is_empty(),
        "{DOCS_PACKAGE} reaches {REGISTRY_PACKAGE}: {routes:#?}"
    );
    let direct = &packages[DOCS_PACKAGE];
    assert!(
        direct
            .iter()
            .any(|(name, kind, target, _)| name == READER_PACKAGE
                && kind.is_empty()
                && target.is_empty()),
        "{DOCS_PACKAGE} has no unconditional normal dependency on {READER_PACKAGE}: {direct:?}"
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
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `canon_engineering` as a whole identifier: not `canon_engineering_assertions`, and found
/// whether it is followed by `::`, ` as`, `;` or anything else.
fn names_the_library(text: &str) -> bool {
    text.match_indices(REGISTRY_LIBRARY).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + REGISTRY_LIBRARY.len()..].chars().next();
        !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
    })
}

#[test]
fn the_identifier_check_tells_the_library_from_the_assertions_library() {
    assert!(names_the_library("use canon_engineering as registry;"));
    assert!(names_the_library("extern crate canon_engineering;"));
    assert!(names_the_library(
        "::canon_engineering::vocabulary::Vocabulary"
    ));
    assert!(!names_the_library(
        "use canon_engineering_assertions::documentation;"
    ));
    assert!(!names_the_library("b10x_canon_engineering"));
}

#[test]
fn no_source_or_build_script_names_the_registry_library() {
    let mut files = Vec::new();
    rust_files(&manifest_dir().join("src"), &mut files);
    let build = manifest_dir().join("build.rs");
    if build.exists() {
        files.push(build);
    }
    assert!(!files.is_empty(), "no Rust file under src/");
    let offenders: Vec<String> = files
        .iter()
        .filter(|path| {
            names_the_library(
                &std::fs::read_to_string(path)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display())),
            )
        })
        .map(|path| path.display().to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "files name the library `{REGISTRY_LIBRARY}`: {offenders:?}"
    );
}
