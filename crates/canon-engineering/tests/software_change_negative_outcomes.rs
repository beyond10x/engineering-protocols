//! Acceptance for `story:software-change-negative-outcomes`: `software.change/1` declares the
//! terminal outcomes `declined`, `rolled_back`, `superseded` and `abandoned` beside `accepted`,
//! each earned by its own decision or evidence, through the fixture harness and Canon.
//!
//! Three are earned on an explicit `canon-decisions/1` decision, which applies only when it was
//! taken at the case's current revision: a decision on another revision of the case earns nothing.
//! `rolled_back` is earned only by a `rollback_result` about the current release revision that says
//! the rollback is complete; authority to roll back, and a partial result, earn nothing, and a
//! complete result about another release revision leaves the outcome UNKNOWN, not TRUE.
//!
//! The story's item 7 starts from the state of item 6, partial result included. In Canon 0.2.0 a
//! complete result about v2 added beside the partial one contradicts it and the match is UNKNOWN,
//! so that state cannot earn `rolled_back` by adding a record; the fixture `rolled-back` records
//! that. Item 7 is decided on `rolled-back-complete`, the state of 6 without its partial result,
//! where the result about v1 is the only rollback record and its exclusion alone decides the
//! outcome.

mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, Json};
use canon_engineering::vocabulary;
use support::{Compiled, Fixture};

/// Every outcome `software.change/1` declares, in identifier order.
const OUTCOMES: [&str; 5] = [
    "abandoned",
    "accepted",
    "declined",
    "rolled_back",
    "superseded",
];

/// The evaluation instant every fixture of this story gives; nothing in them expires.
const AT: &str = "2026-10-12T12:00:00Z";

/// The terms this story adds, each with the category it is declared in.
const TERMS: [(&str, &str); 8] = [
    ("declined", "outcome_id"),
    ("rolled_back", "outcome_id"),
    ("superseded", "outcome_id"),
    ("abandoned", "outcome_id"),
    ("rollback_result", "evidence_kind"),
    ("explicitly_declined", "decision_name"),
    ("explicitly_superseded", "decision_name"),
    ("explicitly_abandoned", "decision_name"),
];

/// The fixture `fixtures/software-change/<id>.fixture.yaml`, loaded, with its every state as
/// expected.
fn fixture(id: &str, compiled: &Compiled) -> Fixture {
    let path = format!("fixtures/software-change/{id}.fixture.yaml");
    let fixture = Fixture::load(&path).unwrap_or_else(|error| panic!("{path} loads: {error}"));
    assert_eq!(fixture.at(), AT, "{path}: the evaluation instant");
    assert_eq!(
        fixture.protocol_path(),
        support::protocol_path("software-change", 1),
        "{path}"
    );
    let own = fixture
        .compile()
        .unwrap_or_else(|error| panic!("{path}: its protocol compiles: {error}"));
    assert_eq!(
        own.canon_ir(),
        compiled.canon_ir(),
        "{path} evaluates software.change/1"
    );
    if let Err(differences) = fixture.check(compiled) {
        panic!(
            "{path} differs from its expectations:\n{}",
            differences.join("\n")
        );
    }
    fixture
}

/// Canon's decision for state `state` of `fixture`.
fn decide(fixture: &Fixture, compiled: &Compiled, state: &str) -> Decision {
    fixture.evaluate(compiled, state).unwrap_or_else(|refusal| {
        panic!(
            "{} state `{state}` is refused: {}: {refusal}",
            fixture.id(),
            refusal.code()
        )
    })
}

/// The `outcomes` section of `decision`, keyed by outcome id.
fn outcomes(decision: &Decision) -> &serde_json::Map<String, Json> {
    decision
        .outcomes
        .as_ref()
        .and_then(Json::as_object)
        .expect("the decision has an outcomes section")
}

/// The status of every outcome in `decision`, keyed by outcome id.
fn statuses(decision: &Decision) -> BTreeMap<String, String> {
    outcomes(decision)
        .iter()
        .map(|(id, entry)| {
            let status = entry["status"]
                .as_str()
                .unwrap_or_else(|| panic!("outcome `{id}` has no status: {entry}"));
            (id.clone(), status.to_owned())
        })
        .collect()
}

/// The status of outcome `id` in `decision`.
fn outcome(decision: &Decision, id: &str) -> String {
    statuses(decision)
        .remove(id)
        .unwrap_or_else(|| panic!("the decision has no outcome `{id}`"))
}

/// Every declared outcome, each with `status`, except those `others` gives.
fn every(status: &str, others: &[(&str, &str)]) -> BTreeMap<String, String> {
    OUTCOMES
        .iter()
        .map(|id| {
            let status = others
                .iter()
                .find(|(other, _)| other == id)
                .map_or(status, |(_, other)| other);
            ((*id).to_owned(), status.to_owned())
        })
        .collect()
}

/// The status Canon's decision gives action `id`.
fn action(decision: &Decision, id: &str) -> String {
    let section = decision
        .actions
        .as_ref()
        .expect("the decision has an actions section");
    section[id]["status"]
        .as_str()
        .unwrap_or_else(|| panic!("the decision has no action `{id}`: {section}"))
        .to_owned()
}

/// The reasons Canon's decision gives outcome `id`.
fn reasons(decision: &Decision, id: &str) -> Json {
    outcomes(decision)[id]["reasons"].clone()
}

#[test]
fn negative_outcomes_are_earned() {
    let path = support::protocol_path("software-change", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));
    let declared: Vec<&str> = compiled
        .ir()
        .outcomes
        .keys()
        .map(|id| id.as_str())
        .collect();
    assert_eq!(declared, OUTCOMES, "the outcomes {path} declares");

    // 1. Declined: no outcome without a decision; `declined` names the absent one. With an
    //    `explicitly_declined` decision at c1 it is legitimate, and no evidence is about the
    //    implementation, which stands at a placeholder revision.
    let declined = fixture("declined", &compiled);
    let initial = decide(&declined, &compiled, "initial");
    assert_eq!(
        statuses(&initial),
        every("blocked", &[]),
        "1. declined, initial"
    );
    assert_eq!(
        reasons(&initial, "declined"),
        serde_json::json!([{"decision": "explicitly_declined", "present": false}]),
        "1. declined names the absent decision"
    );
    let decided = decide(&declined, &compiled, "declined");
    assert_eq!(outcome(&decided, "declined"), "legitimate", "1. declined");
    assert_eq!(
        outcomes(&decided)["declined"]["decided_by"],
        serde_json::json!({"decision": "explicitly_declined", "principals": ["change-owner"]}),
        "1. declined records who decided"
    );
    let about_implementation: Vec<String> = declined
        .evidence("declined")
        .iter()
        .filter(|record| record.subject.as_str() == "implementation")
        .map(|record| record.id.as_str().to_owned())
        .collect();
    assert_eq!(
        about_implementation,
        Vec::<String>::new(),
        "1. a declined change carries no evidence about an implementation"
    );

    // 2. Superseded: passing tests at R1 earn no outcome; an `explicitly_superseded` decision at
    //    c1 earns `superseded`.
    let superseded = fixture("superseded", &compiled);
    assert_eq!(
        statuses(&decide(&superseded, &compiled, "initial")),
        every("blocked", &[]),
        "2. superseded, initial"
    );
    assert_eq!(
        statuses(&decide(&superseded, &compiled, "superseded")),
        every("blocked", &[("superseded", "legitimate")]),
        "2. superseded, decided"
    );

    // 3. Abandoned: no evidence and no decision earn no outcome; an `explicitly_abandoned` decision
    //    at c1 earns `abandoned`, and `accepted` stays blocked.
    let abandoned = fixture("abandoned", &compiled);
    assert_eq!(
        abandoned.evidence("abandoned").len(),
        0,
        "3. abandoned has no evidence"
    );
    assert_eq!(
        statuses(&decide(&abandoned, &compiled, "initial")),
        every("blocked", &[]),
        "3. abandoned, initial"
    );
    let decided = decide(&abandoned, &compiled, "abandoned");
    assert_eq!(outcome(&decided, "abandoned"), "legitimate", "3. abandoned");
    assert_eq!(outcome(&decided, "accepted"), "blocked", "3. accepted");

    // 4. The same decision taken at the case's earlier revision c0 earns nothing at c1.
    let stale = decide(&abandoned, &compiled, "abandoned-at-c0");
    assert_eq!(
        outcome(&stale, "abandoned"),
        "blocked",
        "4. abandoned at c0"
    );
    assert_eq!(
        reasons(&stale, "abandoned"),
        serde_json::json!([{"decision": "explicitly_abandoned", "present": false}]),
        "4. a decision on another case revision is no decision"
    );

    // 5. Rolled back, before anything: the rollback needs authority, and `rolled_back` is blocked.
    let rolled_back = fixture("rolled-back", &compiled);
    let initial = decide(&rolled_back, &compiled, "initial");
    assert_eq!(
        action(&initial, "release.rollback"),
        "approval-required",
        "5. release.rollback without authority"
    );
    assert_eq!(
        outcome(&initial, "rolled_back"),
        "blocked",
        "5. rolled_back"
    );

    // 6. With authority and a partial result about v2, the rollback is admissible and
    //    `rolled_back` is still blocked: invoking a rollback is not enough.
    let partial = decide(&rolled_back, &compiled, "rollback-partial");
    assert_eq!(
        action(&partial, "release.rollback"),
        "admissible",
        "6. release.rollback with authority"
    );
    assert_eq!(outcome(&partial, "rolled_back"), "blocked", "6. partial");

    // A complete result about v2 beside the partial one contradicts it: Canon reads two results
    // about one release revision that disagree as UNKNOWN, so the state of 6 cannot be completed
    // by adding a record. Item 7 starts from state 6 without its partial result instead.
    assert_eq!(
        outcome(
            &decide(&rolled_back, &compiled, "complete-after-partial"),
            "rolled_back"
        ),
        "blocked",
        "a complete result about v2 beside a partial one"
    );

    // 7. From the state of 6 without its partial result: a complete result about release v1 is
    //    about another revision of the release, UNKNOWN, not TRUE; one about v2 earns
    //    `rolled_back`.
    let complete = fixture("rolled-back-complete", &compiled);
    let authorized = decide(&complete, &compiled, "authorized");
    assert_eq!(
        action(&authorized, "release.rollback"),
        "admissible",
        "7. release.rollback with authority"
    );
    assert_eq!(
        outcome(&authorized, "rolled_back"),
        "blocked",
        "7. authority alone"
    );
    let other_release = decide(&complete, &compiled, "complete-v1");
    assert_eq!(
        outcome(&other_release, "rolled_back"),
        "blocked",
        "7. a complete result about v1"
    );
    assert_eq!(
        reasons(&other_release, "rolled_back"),
        serde_json::json!([{"evidence": "rollback_result", "present": false, "subject": "release"}]),
        "7. the result about v1 does not apply to release v2"
    );
    assert_eq!(
        outcome(&decide(&complete, &compiled, "complete-v2"), "rolled_back"),
        "legitimate",
        "7. a complete result about v2"
    );

    // 8. Each new term resolves to exactly one vocabulary entry, in its category.
    for (term, category) in TERMS {
        let found = vocabulary::lookup(term).unwrap_or_else(|error| panic!("8. {error}"));
        assert_eq!(found.category.as_str(), category, "8. `{term}`");
        let entries = vocabulary::terms()
            .iter()
            .filter(|entry| entry.id == term)
            .count();
        assert_eq!(entries, 1, "8. `{term}` is declared {entries} times");
    }
}
