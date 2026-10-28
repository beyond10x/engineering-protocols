//! Adversarial review of `story:software-change-profiles`, at `51fb753`
//! (`git diff a51a581..wave/2026-10-13-w1`).
//!
//! Each case drives the compiled `protocols/software-change/1.yaml` through Canon directly, with a
//! case and evidence written here, or drives the committed fixtures against a mutated copy of the
//! protocol. Red cases are findings; green ones are attacks that did not land.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;
use std::path::Path;

use b10x_canon::eval;
use b10x_canon::model::{Decision, EvidenceRecord, Json};
use serde_yaml_ng::Value;
use support::{Compiled, Fixture};

const PROTOCOL: &str = "protocols/software-change/1.yaml";

/// Every fixture of `software.change/1`.
const FIXTURES: [&str; 15] = [
    "abandoned",
    "change-kinds",
    "chg-1842",
    "declined",
    "profiles",
    "profiles-discharged",
    "red-first",
    "red-first-edit-first",
    "red-first-green-first",
    "red-first-other-subject",
    "rolled-back-complete",
    "rolled-back",
    "security-boundary",
    "security-boundary-independent",
    "superseded",
];

fn compiled() -> Compiled {
    support::compile_protocol(PROTOCOL).unwrap_or_else(|error| panic!("{PROTOCOL}: {error}"))
}

/// A `software.change` case at implementation `implementation` and release `release`, with
/// `inputs` written as a YAML flow map's body (`risk: standard, change_kind: bugfix`), and
/// `affects_security_boundary: false`.
fn case_value(inputs: &str, implementation: &str, release: &str) -> Value {
    serde_yaml_ng::from_str(&format!(
        "{{format: canon-case/1, id: CHG-9001, protocol: software.change, \
         inputs: {{{inputs}, affects_security_boundary: false}}, \
         artifacts: {{intent: {{revision: i1}}, system_specification: {{revision: s1}}, \
         plan: {{revision: p1}}, implementation: {{revision: {implementation}}}, \
         release: {{revision: {release}}}, deployment: {{revision: d1}}}}}}"
    ))
    .expect("the case is YAML")
}

/// One `canon-evidence/1` record.
fn record(
    id: &str,
    kind: &str,
    result: Option<&str>,
    subject: &str,
    revision: &str,
) -> EvidenceRecord {
    observed(id, kind, result, subject, revision, None)
}

/// One `canon-evidence/1` record, observed at `at` when given.
fn observed(
    id: &str,
    kind: &str,
    result: Option<&str>,
    subject: &str,
    revision: &str,
    at: Option<&str>,
) -> EvidenceRecord {
    let result = result.map_or(String::new(), |result| format!("result: {result}, "));
    let at = at.map_or(String::new(), |at| format!(", observed_at: \"{at}\""));
    eval::evidence_from_value(
        &serde_yaml_ng::from_str(&format!(
            "{{format: canon-evidence/1, id: {id}, kind: {kind}, {result}subject: {subject}, \
             subject_revision: {revision}{at}}}"
        ))
        .expect("the record is YAML"),
    )
    .unwrap_or_else(|refusal| panic!("record `{id}`: {refusal}"))
}

fn decide(
    compiled: &Compiled,
    inputs: &str,
    implementation: &str,
    evidence: &[EvidenceRecord],
) -> Decision {
    let case = eval::case_from_value(&case_value(inputs, implementation, "v1"))
        .unwrap_or_else(|refusal| panic!("the case reads: {refusal}"));
    eval::evaluate(compiled.ir(), &case, evidence)
        .unwrap_or_else(|refusal| panic!("{inputs}: refused: {}: {refusal}", refusal.code()))
}

fn action(decision: &Decision, id: &str) -> Json {
    decision
        .actions
        .as_ref()
        .and_then(|actions| actions.get(id))
        .cloned()
        .unwrap_or(Json::Null)
}

fn obligations(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .obligations
        .as_ref()
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .map(|entry| {
            (
                entry["id"].as_str().unwrap_or_default().to_owned(),
                entry["status"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

fn claim(decision: &Decision, id: &str) -> String {
    decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .map(|(_, entry)| entry.value.to_string())
        .unwrap_or_else(|| "undeclared".to_owned())
}

/// The red-first workflow as a bugfix runs it: the suite is run on R1 and passes, the regression
/// test is written (`tests.write`, which moves no implementation revision), and the regression
/// test is run on R1 and fails. Canon reads a pass and a fail of one kind about one revision as
/// contradicting, and the first record of one kind as the earlier one, so a suite run and a
/// regression test run are told apart by kind: the regression test's first result is a fail, the
/// regression is reproduced, and the edit is admissible on R1. Recorded as suite runs alone, the
/// same green-then-red history still leaves the edit blocked: neither path holds.
#[test]
fn a_bugfix_green_then_red_on_one_revision_may_edit_once_its_regression_test_ran_red() {
    let compiled = compiled();
    let inputs = "risk: standard, change_kind: bugfix";
    let suite_green = observed(
        "suite-r1-green",
        "test_result",
        Some("pass"),
        "implementation",
        "R1",
        Some("2026-10-13T09:00:00Z"),
    );
    let evidence = [
        suite_green.clone(),
        observed(
            "regression-r1-red",
            "regression_test_result",
            Some("fail"),
            "implementation",
            "R1",
            Some("2026-10-13T10:00:00Z"),
        ),
    ];
    let decision = decide(&compiled, inputs, "R1", &evidence);
    assert_eq!(
        (
            claim(&decision, "regression.reproduced"),
            action(&decision, "implementation.edit")["status"].clone()
        ),
        ("true".to_owned(), Json::from("admissible")),
        "a green suite run then a red regression test run about R1: implementation.edit is {}",
        action(&decision, "implementation.edit")
    );
    let suite_only = [
        suite_green,
        observed(
            "suite-r1-red",
            "test_result",
            Some("fail"),
            "implementation",
            "R1",
            Some("2026-10-13T10:00:00Z"),
        ),
    ];
    let decision = decide(&compiled, inputs, "R1", &suite_only);
    assert_eq!(
        (
            claim(&decision, "regression.reproduced"),
            action(&decision, "implementation.edit")["status"].clone()
        ),
        ("unknown".to_owned(), Json::from("blocked")),
        "a green then a red suite run about R1: implementation.edit is {}",
        action(&decision, "implementation.edit")
    );
}

/// No path admits a bugfix's edit without a failing test about the current implementation
/// revision or a regression test whose first result failed: none, a pass, a fail about another
/// revision, a fail about the release, a fail about R1 once the implementation is at R2, or a test
/// result without a result.
#[test]
fn a_bugfix_edit_needs_a_failing_test_about_the_current_implementation_revision() {
    let compiled = compiled();
    let inputs = "risk: critical, change_kind: bugfix";
    for (what, implementation, evidence) in [
        ("no evidence", "R1", vec![]),
        (
            "a pass",
            "R1",
            vec![record(
                "t",
                "test_result",
                Some("pass"),
                "implementation",
                "R1",
            )],
        ),
        (
            "a fail about R0",
            "R1",
            vec![record(
                "t",
                "test_result",
                Some("fail"),
                "implementation",
                "R0",
            )],
        ),
        (
            "a fail about the release",
            "R1",
            vec![record("t", "test_result", Some("fail"), "release", "v1")],
        ),
        (
            "a fail about R1 at R2",
            "R2",
            vec![record(
                "t",
                "test_result",
                Some("fail"),
                "implementation",
                "R1",
            )],
        ),
        (
            "a test result without a result",
            "R1",
            vec![record("t", "test_result", None, "implementation", "R1")],
        ),
        (
            "a failing review",
            "R1",
            vec![record(
                "t",
                "code_review",
                Some("rejected"),
                "implementation",
                "R1",
            )],
        ),
    ] {
        let decision = decide(&compiled, inputs, implementation, &evidence);
        assert_eq!(
            action(&decision, "implementation.edit")["status"],
            Json::from("blocked"),
            "{what}: {}",
            action(&decision, "implementation.edit")
        );
        assert_eq!(
            action(&decision, "tests.write")["status"],
            Json::from("admissible"),
            "{what}"
        );
    }
}

/// An obligation outside its profile is `not_applicable` even when its discharge holds, and one
/// inside it is `discharged` once its claim is TRUE: never `open` or `discharged` outside it.
#[test]
fn obligations_discharge_inside_their_profile_and_are_not_applicable_outside_it() {
    let compiled = compiled();
    let everything = [
        record("tests", "test_result", Some("pass"), "implementation", "R1"),
        record(
            "review",
            "code_review",
            Some("approved"),
            "implementation",
            "R1",
        ),
        record("provenance", "build_provenance", None, "release", "v1"),
        record(
            "health",
            "operational_observation",
            Some("healthy"),
            "deployment",
            "d1",
        ),
    ];
    let order = [
        "tests.required",
        "review.required",
        "provenance.required",
        "health.required",
    ];
    for (index, risk) in ["trivial", "standard", "elevated", "critical"]
        .into_iter()
        .enumerate()
    {
        let decision = decide(
            &compiled,
            &format!("risk: {risk}, change_kind: feature"),
            "R1",
            &everything,
        );
        let mut expected: BTreeMap<String, String> = order
            .iter()
            .enumerate()
            .map(|(position, id)| {
                let status = if position <= index {
                    "discharged"
                } else {
                    "not_applicable"
                };
                ((*id).to_owned(), status.to_owned())
            })
            .collect();
        // A feature carries no red-first obligation at any risk.
        expected.insert("red_first.required".to_owned(), "not_applicable".to_owned());
        // Nor, off a security boundary, a security-review obligation.
        expected.insert(
            "security_review.required".to_owned(),
            "not_applicable".to_owned(),
        );
        assert_eq!(obligations(&decision), expected, "risk `{risk}`");
    }
}

/// `profiles-discharged` holds against the protocol as it ships: each obligation is discharged by
/// its own claim, and `not_applicable` wins over a discharge outside the risk.
#[test]
fn profiles_discharged_holds() {
    let path = "fixtures/software-change/profiles-discharged.fixture.yaml";
    let fixture = Fixture::load(path).unwrap_or_else(|error| panic!("{path}: {error}"));
    if let Err(differences) = fixture.check(&compiled()) {
        panic!("{path} differs:\n{}", differences.join("\n"));
    }
}

/// Each obligation's discharge is a protocol rule, and `AGENTS.md` § Invariants holds that every
/// protocol rule lands with a fixture that exercises it. Point one obligation's
/// `discharged_when` at another claim and run every `software.change/1` fixture against that
/// protocol: some fixture must notice.
#[test]
fn some_fixture_notices_an_obligation_discharged_by_the_wrong_claim() {
    let text = std::fs::read_to_string(support::repo_root().join(PROTOCOL)).expect("reads");
    let fixtures: Vec<Fixture> = FIXTURES
        .iter()
        .map(|id| {
            let path = format!("fixtures/software-change/{id}.fixture.yaml");
            Fixture::load(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
        })
        .collect();
    let mut survivors = Vec::new();
    for (obligation, from, to) in [
        (
            "tests.required",
            "claim: tests.pass\n  review",
            "claim: implementation.reviewed\n  review",
        ),
        (
            "review.required",
            "claim: implementation.reviewed\n  provenance",
            "claim: tests.pass\n  provenance",
        ),
        (
            "provenance.required",
            "claim: release.proven\n  health",
            "claim: deployment.healthy\n  health",
        ),
        (
            "health.required",
            "claim: deployment.healthy\n  red_first",
            "claim: release.proven\n  red_first",
        ),
        (
            "red_first.required",
            "claim: regression.shown_first\n  security_review",
            "claim: regression.reproduced\n  security_review",
        ),
        (
            "security_review.required",
            "claim: security.reviewed\n\nactions",
            "claim: tests.pass\n\nactions",
        ),
    ] {
        assert_eq!(
            text.matches(from).count(),
            1,
            "the mutation site of `{obligation}`"
        );
        let mutant = support::compile_protocol_text(&text.replacen(from, to, 1))
            .unwrap_or_else(|error| panic!("the `{obligation}` mutant compiles: {error}"));
        if fixtures
            .iter()
            .all(|fixture| fixture.check(&mutant).is_ok())
        {
            survivors.push(format!("{obligation}: `{from}` -> `{to}`").replace('\n', " "));
        }
    }
    assert!(
        survivors.is_empty(),
        "every software.change/1 fixture passes against these mutants:\n{}",
        survivors.join("\n")
    );
}

/// A case is refused when it gives a value outside a closed list, leaves out either input, or
/// gives one the protocol does not declare.
#[test]
fn a_case_outside_the_declared_inputs_is_refused() {
    let compiled = compiled();
    for (inputs, code) in [
        ("risk: high, change_kind: bugfix", "undeclared-input-value"),
        (
            "risk: Standard, change_kind: bugfix",
            "undeclared-input-value",
        ),
        (
            "risk: standard, change_kind: hotfix",
            "undeclared-input-value",
        ),
        ("risk: standard", "missing-input"),
        ("change_kind: bugfix", "missing-input"),
        (
            "risk: standard, change_kind: bugfix, size: big",
            "undeclared-input",
        ),
    ] {
        let refused = eval::case_from_value(&case_value(inputs, "R1", "v1"))
            .and_then(|case| eval::evaluate(compiled.ir(), &case, &[]));
        assert_eq!(
            refused.as_ref().err().map(|refusal| refusal.code()),
            Some(code),
            "{inputs}"
        );
    }
}
