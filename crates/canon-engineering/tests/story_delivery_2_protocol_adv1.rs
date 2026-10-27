//! Adversarial cases for `story:story-delivery-2-with-settings`: `story.delivery/2` driven with
//! histories its seven fixtures do not hold.
//!
//! The story's outcome: under `tests_first: required`, `implementation.edit` is admissible only
//! after a `tests.run` recorded `fail` on a revision `tests.write` produced; the protocol's own
//! claim text says the failing run "is about the revision the tests were written at".

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, Json};
use support::Fixture;

/// A `story.delivery/2` fixture at implementation revision h1 with the given case inputs (YAML
/// mapping lines, indented under `inputs:`) and states.
fn story(id: &str, inputs: &str, states: &str) -> Fixture {
    let text = format!(
        "format: els-fixture/1
id: {id}
protocol: protocols/story-delivery/2.yaml
at: \"2026-10-27T12:00:00Z\"
case:
  format: canon-case/1
  id: SD-ADV1
  protocol: story.delivery
  revision: c1
  artifacts:
    intent: {{revision: i1}}
    plan: {{revision: p1}}
    implementation: {{revision: h1}}
  inputs:
{inputs}
states:
{states}"
    );
    Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{id} loads: {error}"))
}

const REQUIRED: &str = "    tests_first: required\n    gate_on: each_revision";

fn entry(at: &str, id: &str, kind: &str, result: Option<&str>, rev: &str) -> String {
    let result = result.map_or(String::new(), |result| format!(", result: {result}"));
    format!(
        "      - observed_at: \"{at}\"
        record: {{format: canon-evidence/1, id: {id}, kind: {kind}{result}, subject: implementation, subject_revision: {rev}}}
"
    )
}

fn state(id: &str, set_implementation: Option<&str>, evidence: &[String]) -> String {
    let set = set_implementation.map_or(String::new(), |rev| {
        format!("    set_revisions:\n      implementation: {rev}\n")
    });
    let evidence = if evidence.is_empty() {
        "    add_evidence: []\n".to_owned()
    } else {
        format!("    add_evidence:\n{}", evidence.concat())
    };
    format!("  - id: {id}\n{set}{evidence}    expect:\n      claims: {{}}\n")
}

fn statuses(section: &Option<Json>) -> BTreeMap<String, String> {
    match section {
        Some(Json::Array(entries)) => entries
            .iter()
            .map(|entry| {
                (
                    entry["id"].as_str().unwrap_or_default().to_owned(),
                    entry["status"].as_str().unwrap_or_default().to_owned(),
                )
            })
            .collect(),
        Some(Json::Object(entries)) => entries
            .iter()
            .map(|(id, entry)| {
                (
                    id.clone(),
                    entry["status"].as_str().unwrap_or_default().to_owned(),
                )
            })
            .collect(),
        _ => BTreeMap::new(),
    }
}

struct Seen {
    claims: BTreeMap<String, String>,
    actions: BTreeMap<String, String>,
}

fn seen(decision: &Decision) -> Seen {
    Seen {
        claims: decision
            .claims
            .iter()
            .map(|(id, entry)| (id.as_str().to_owned(), entry.value.to_string()))
            .collect(),
        actions: statuses(&decision.actions),
    }
}

fn decide(fixture: &Fixture, state: &str) -> Seen {
    let compiled = fixture.compile().expect("story.delivery/2 compiles");
    let decision = fixture
        .evaluate(&compiled, state)
        .unwrap_or_else(|refusal| panic!("state `{state}` refused: {refusal}"));
    seen(&decision)
}

const T08: &str = "2026-10-27T08:00:00Z";
const T09: &str = "2026-10-27T09:00:00Z";
const T10: &str = "2026-10-27T10:00:00Z";
const T11: &str = "2026-10-27T11:00:00Z";

/// The tests are written at h2, and the only failing run recorded afterwards is about h1, the
/// seed, which carries none of the story's tests (a late result of a run started before the
/// write). No run of the tests the story wrote has failed, so the edit stays refused.
#[test]
fn a_red_run_about_another_revision_does_not_admit_the_edit() {
    let fixture = story(
        "adv1-red-about-the-seed",
        REQUIRED,
        &[
            state(
                "tests-written",
                Some("h2"),
                &[entry(T09, "tests-written-h2", "tests_change", None, "h2")],
            ),
            state(
                "a-seed-run-fails-late",
                None,
                &[entry(
                    T10,
                    "tests-h1-red",
                    "test_result",
                    Some("fail"),
                    "h1",
                )],
            ),
        ]
        .concat(),
    );
    let found = decide(&fixture, "a-seed-run-fails-late");
    assert_ne!(
        found.actions["implementation.edit"], "admissible",
        "implementation.edit admitted on a fail about h1 while the tests were written at h2 \
         (tests.shown_red = {}): the order forms do not read the revision",
        found.claims["tests.shown_red"]
    );
}

/// A run of the tests at the seed, before the story writes any (a baseline, which `tests.run`
/// admits at the seed), then the tests written at h2 and their run there failing. The story's
/// condition holds: a `tests.run` recorded `fail` on the revision `tests.write` produced. The edit
/// is admissible, or the case can never reach an implementation and only `abandoned` is left.
#[test]
fn a_baseline_run_at_the_seed_does_not_lock_the_edit_out() {
    for baseline in ["pass", "fail"] {
        let fixture = story(
            &format!("adv1-baseline-{baseline}"),
            REQUIRED,
            &[
                state(
                    "baseline",
                    None,
                    &[entry(T08, "tests-h1", "test_result", Some(baseline), "h1")],
                ),
                state(
                    "tests-written",
                    Some("h2"),
                    &[entry(T09, "tests-written-h2", "tests_change", None, "h2")],
                ),
                state(
                    "red",
                    None,
                    &[entry(
                        T10,
                        "tests-h2-red",
                        "test_result",
                        Some("fail"),
                        "h2",
                    )],
                ),
            ]
            .concat(),
        );
        let base = decide(&fixture, "baseline");
        assert_eq!(base.actions["tests.run"], "admissible");
        let found = decide(&fixture, "red");
        assert_eq!(
            found.actions["implementation.edit"], "admissible",
            "baseline {baseline} at h1, tests written at h2, red at h2: the edit stays refused \
             for good (tests.shown_red = {})",
            found.claims["tests.shown_red"]
        );
    }
}

/// `gate_on: after_handover`: a handover at an earlier revision does not admit the gate at the
/// current one, even with the tests passing there.
#[test]
fn a_handover_at_an_earlier_revision_does_not_admit_the_gate() {
    let fixture = story(
        "adv1-old-handover",
        "    tests_first: optional\n    gate_on: after_handover",
        &[
            state(
                "handed-over",
                Some("h2"),
                &[
                    entry(T09, "tests-h2-pass", "test_result", Some("pass"), "h2"),
                    entry(T10, "handover-h2", "handover", None, "h2"),
                ],
            ),
            state(
                "edited-and-green",
                Some("h3"),
                &[entry(
                    T11,
                    "tests-h3-pass",
                    "test_result",
                    Some("pass"),
                    "h3",
                )],
            ),
        ]
        .concat(),
    );
    assert_eq!(
        decide(&fixture, "handed-over").actions["gate.run"],
        "admissible"
    );
    let moved = decide(&fixture, "edited-and-green");
    assert_eq!(moved.claims["tests.pass"], "true");
    assert_eq!(moved.claims["change.handed_over"], "unknown");
    assert_eq!(moved.actions["gate.run"], "blocked");
}

/// Under `tests_first: required` with no run at all, rewriting the tests twice never admits the
/// edit; neither does a pass followed by a tie with a fail at the same instant.
#[test]
fn required_without_a_red_first_run_never_admits_the_edit() {
    let fixture = story(
        "adv1-no-red",
        REQUIRED,
        &[
            state(
                "written-twice",
                Some("h3"),
                &[
                    entry(T09, "tests-written-h2", "tests_change", None, "h2"),
                    entry(T10, "tests-written-h3", "tests_change", None, "h3"),
                ],
            ),
            state(
                "tie",
                None,
                &[
                    entry(T11, "tests-h3-pass", "test_result", Some("pass"), "h3"),
                    entry(T11, "tests-h3-fail", "test_result", Some("fail"), "h3"),
                ],
            ),
        ]
        .concat(),
    );
    for state in ["written-twice", "tie"] {
        assert_eq!(
            decide(&fixture, state).actions["implementation.edit"],
            "blocked",
            "`{state}`"
        );
    }
}

/// A value outside the declared set, and a missing input, are refused when the case is
/// evaluated, not read as some other value.
#[test]
fn an_input_outside_its_declared_values_is_refused() {
    for inputs in [
        "    tests_first: sometimes\n    gate_on: each_revision",
        "    tests_first: required\n    gate_on: never",
        "    tests_first: required",
    ] {
        let fixture = story("adv1-bad-input", inputs, &state("initial", None, &[]));
        let compiled = fixture.compile().expect("compiles");
        assert!(
            fixture.evaluate(&compiled, "initial").is_err(),
            "inputs {inputs:?} were accepted"
        );
    }
}
