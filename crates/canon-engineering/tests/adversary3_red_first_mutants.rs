//! Adversarial review of `story:evidence-order-red-first`, at `cdcffee`: mutants of the new
//! claim forms of `software.change/1` that no fixture under `fixtures/software-change/` kills.
//!
//! AGENTS.md: "Every protocol rule lands with a fixture under `fixtures/` that exercises it". Each
//! case here writes a mutated copy of the protocol under the tree's `target/` (never the protocol
//! itself), compiles it, and runs every software-change fixture against it: a mutant no fixture
//! tells apart is a part of the rule no fixture exercises. A control mutant that the fixtures do
//! kill shows the method can fail.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use support::{Fixture, compile_protocol, repo_root};

const PROTOCOL: &str = "protocols/software-change/1.yaml";

/// The fixtures that fail against the protocol with `from` replaced by `to` once; panics when
/// `from` is not in the protocol exactly once.
fn killed_by(name: &str, from: &str, to: &str) -> Vec<String> {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(PROTOCOL)).expect("protocol reads");
    assert_eq!(text.matches(from).count(), 1, "mutant `{name}`: {from:?}");
    let mutant = text.replacen(from, to, 1);
    let dir = root.join("target/adversary3-mutants");
    std::fs::create_dir_all(&dir).expect("mutant directory");
    let path = format!("target/adversary3-mutants/{name}.yaml");
    std::fs::write(root.join(&path), mutant).expect("mutant writes");
    let compiled = compile_protocol(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
    let mut killed = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(root.join("fixtures/software-change"))
        .expect("fixtures")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("utf-8")
        })
        .filter(|file| file.ends_with(".fixture.yaml"))
        .collect();
    entries.sort();
    for file in entries {
        let fixture = Fixture::load(&format!("fixtures/software-change/{file}"))
            .unwrap_or_else(|error| panic!("{file}: {error}"));
        if fixture.check(&compiled).is_err() {
            killed.push(file);
        }
    }
    killed
}

/// Control: flipping the `first` result to `pass` is killed (by `red-first`, at least).
#[test]
fn control_a_flipped_first_result_is_killed() {
    let killed = killed_by(
        "first-result-pass",
        "        - first:\n            kind: regression_test_result\n            result: fail\n",
        "        - first:\n            kind: regression_test_result\n            result: pass\n",
    );
    assert!(
        !killed.is_empty(),
        "the control mutant survives every fixture"
    );
}

/// `regression.reproduced` without `subject: implementation` on its `first` arm: a regression
/// test result about the release then counts. No fixture holds one.
#[test]
fn a_first_without_its_subject_is_killed_by_a_fixture() {
    let killed = killed_by(
        "first-without-subject",
        "            result: fail\n            subject: implementation\n  regression.shown_first:",
        "            result: fail\n  regression.shown_first:",
    );
    assert!(
        !killed.is_empty(),
        "dropping `subject: implementation` from regression.reproduced's `first` arm survives \
         every software-change fixture"
    );
}

/// `regression.shown_first` without `subject: implementation`: a regression run or a change about
/// another artifact then decides the order. No fixture holds one.
#[test]
fn a_before_without_its_subject_is_killed_by_a_fixture() {
    let killed = killed_by(
        "before-without-subject",
        "        then: implementation_change\n        subject: implementation\n",
        "        then: implementation_change\n",
    );
    assert!(
        !killed.is_empty(),
        "dropping `subject: implementation` from regression.shown_first's `before` survives \
         every software-change fixture"
    );
}
