//! What the fixture tests of `story.delivery/1`, `self.improvement/1` and `loop.improvement/1`
//! share: loading a built-in's fixtures through the harness (`tests/support`), and reading Canon's
//! decision for one state. It evaluates nothing itself.

use std::collections::{BTreeMap, BTreeSet};

use b10x_canon::model::{Decision, ExclusionReason, Json};

use crate::support::{self, Compiled, Fixture};

/// The built-in `protocols/<name>/1.yaml`, validated and compiled by Canon, and the same as
/// `registry::get(name, 1)` returns.
pub fn compile(name: &str) -> Compiled {
    compile_major(name, 1)
}

/// The built-in `protocols/<name>/<major>.yaml`, validated and compiled by Canon, and the same as
/// `registry::get(name, major)` returns.
pub fn compile_major(name: &str, major: u32) -> Compiled {
    let path = support::protocol_path(name, u64::from(major));
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));
    let builtin = canon_engineering::registry::get(name, major)
        .unwrap_or_else(|error| panic!("registry::get({name}, {major}): {error}"));
    let on_disk = std::fs::read_to_string(support::repo_root().join(&path))
        .unwrap_or_else(|error| panic!("{path}: {error}"));
    assert_eq!(builtin.yaml, on_disk, "{path} is the built-in as released");
    compiled
}

/// The fixture `fixtures/<name>/<id>.fixture.yaml`, loaded and checked: it evaluates the built-in
/// `name@1` at `at`, and every state is as it expects.
pub fn fixture(name: &str, id: &str, at: &str, compiled: &Compiled) -> Fixture {
    fixture_of(name, name, 1, id, at, compiled)
}

/// The fixture `fixtures/<directory>/<id>.fixture.yaml`, loaded and checked: it evaluates the
/// built-in `name@major` at `at`, and every state is as it expects.
pub fn fixture_of(
    directory: &str,
    name: &str,
    major: u32,
    id: &str,
    at: &str,
    compiled: &Compiled,
) -> Fixture {
    let path = format!("fixtures/{directory}/{id}.fixture.yaml");
    let fixture = Fixture::load(&path).unwrap_or_else(|error| panic!("{path} loads: {error}"));
    assert_eq!(fixture.at(), at, "{path}: the evaluation instant");
    assert_eq!(
        fixture.protocol_path(),
        support::protocol_path(name, u64::from(major)),
        "{path}"
    );
    let own = fixture
        .compile()
        .unwrap_or_else(|error| panic!("{path}: its protocol compiles: {error}"));
    assert_eq!(
        own.canon_ir(),
        compiled.canon_ir(),
        "{path} evaluates the built-in {name}@{major}"
    );
    if let Err(differences) = fixture.check(compiled) {
        panic!(
            "{path} differs from its expectations:\n{}",
            differences.join("\n")
        );
    }
    fixture
}

/// Every fixture id under `fixtures/<name>/`, sorted.
pub fn fixture_ids(name: &str) -> Vec<String> {
    let directory = support::repo_root().join("fixtures").join(name);
    let mut ids: Vec<String> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", directory.display()));
            entry.file_name().to_string_lossy().into_owned()
        })
        .filter_map(|file| file.strip_suffix(".fixture.yaml").map(str::to_owned))
        .collect();
    ids.sort();
    ids
}

/// Canon's decision for state `state` of `fixture`.
pub fn decide(fixture: &Fixture, compiled: &Compiled, state: &str) -> Decision {
    fixture.evaluate(compiled, state).unwrap_or_else(|refusal| {
        panic!(
            "{} state `{state}` is refused: {}: {refusal}",
            fixture.id(),
            refusal.code()
        )
    })
}

/// The value of every claim in `decision`, keyed by claim id.
pub fn claims(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .claims
        .iter()
        .map(|(id, entry)| (id.as_str().to_owned(), entry.value.to_string()))
        .collect()
}

/// The reasons `decision` gives for each record it excludes from claim `claim`.
pub fn exclusions(decision: &Decision, claim: &str) -> Vec<ExclusionReason> {
    decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == claim)
        .map(|(_, entry)| {
            entry
                .excluded_evidence
                .iter()
                .map(|exclusion| exclusion.reason)
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `decision` excludes at least one record from `claim`, every one for `reason`.
pub fn excluded_only_as(decision: &Decision, claim: &str, reason: ExclusionReason) -> bool {
    let reasons = exclusions(decision, claim);
    !reasons.is_empty() && reasons.iter().all(|found| *found == reason)
}

/// The status of every entry of a decision section, keyed by id.
fn section(section: Option<&Json>, name: &str) -> BTreeMap<String, String> {
    let status = |id: &str, entry: &Json| -> String {
        entry["status"]
            .as_str()
            .unwrap_or_else(|| panic!("{name} `{id}` has no status: {entry}"))
            .to_owned()
    };
    match section {
        Some(Json::Object(entries)) => entries
            .iter()
            .map(|(id, entry)| (id.clone(), status(id, entry)))
            .collect(),
        Some(Json::Array(entries)) => entries
            .iter()
            .map(|entry| {
                let id = entry["id"]
                    .as_str()
                    .unwrap_or_else(|| panic!("an entry of {name} has no id: {entry}"));
                (id.to_owned(), status(id, entry))
            })
            .collect(),
        other => panic!("the decision has no {name} section: {other:?}"),
    }
}

/// The status of every action in `decision`, keyed by action id.
pub fn actions(decision: &Decision) -> BTreeMap<String, String> {
    section(decision.actions.as_ref(), "actions")
}

/// The status of every outcome in `decision`, keyed by outcome id.
pub fn outcomes(decision: &Decision) -> BTreeMap<String, String> {
    section(decision.outcomes.as_ref(), "outcomes")
}

/// Every outcome legitimate in some state of `fixtures`, and the states of all of them counted.
pub fn reached(fixtures: &[Fixture], compiled: &Compiled) -> (BTreeSet<String>, usize) {
    let mut legitimate = BTreeSet::new();
    let mut states = 0;
    for fixture in fixtures {
        for state in fixture.states() {
            states += 1;
            for (outcome, status) in outcomes(&decide(fixture, compiled, state)) {
                if status == "legitimate" {
                    legitimate.insert(outcome);
                }
            }
        }
    }
    (legitimate, states)
}
