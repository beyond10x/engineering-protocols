//! Acceptance for `story:evidence-order-red-first`: `software.change/1` reads a bugfix's
//! red-first rule from evidence order, on Canon 0.3.0's order forms. A bugfix whose suite ran green
//! on R1 and whose regression test then ran red on R1 may edit the implementation, and the red run
//! keeps counting once the edit moves the implementation to R2; `red_first.required` is discharged
//! when the first regression test result came before the first implementation change, and applies
//! to a bugfix only.
//!
//! The fixtures `red-first`, `red-first-edit-first` and `red-first-green-first`
//! (`fixtures/software-change/`) hold the five cases: one fixture's states are cumulative, and a
//! history in which the regression test passed first, or the implementation changed first, cannot
//! later become one in which neither happened. The test checks every acceptance item and reports
//! each one that fails, in item order, rather than stopping at the first.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, Json};
use canon_engineering::vocabulary;
use support::{Compiled, Fixture};

/// The evaluation instant every fixture of this story gives.
const AT: &str = "2026-10-13T12:00:00Z";

/// The fixtures of this story.
const FIXTURES: [&str; 3] = ["red-first", "red-first-edit-first", "red-first-green-first"];

/// The terms this story adds, each with the category it is declared in.
const TERMS: [(&str, &str); 4] = [
    ("regression_test_result", "evidence_kind"),
    ("implementation_change", "evidence_kind"),
    ("regression.shown_first", "claim_id"),
    ("red_first.required", "obligation_id"),
];

/// The failures found so far, each prefixed with its acceptance item (0 for the fixtures as a
/// whole, 6 for the vocabulary).
#[derive(Default)]
struct Failures(Vec<String>);

impl Failures {
    fn that(&mut self, item: u8, holds: bool, what: impl FnOnce() -> String) {
        if !holds {
            self.0.push(format!("item {item}: {}", what()));
        }
    }

    fn equal<T: PartialEq + std::fmt::Debug>(
        &mut self,
        item: u8,
        found: T,
        expected: T,
        what: &str,
    ) {
        if found != expected {
            self.0.push(format!(
                "item {item}: {what}: found {found:?}, expected {expected:?}"
            ));
        }
    }
}

fn fixture(id: &str) -> Fixture {
    let path = format!("fixtures/software-change/{id}.fixture.yaml");
    let fixture = Fixture::load(&path).unwrap_or_else(|error| panic!("{path} loads: {error}"));
    assert_eq!(fixture.at(), AT, "{path}: the evaluation instant");
    fixture
}

/// What one state of a fixture decides: claim values, obligation and action statuses, each as
/// text, and the decision's explanation.
struct Seen {
    claims: BTreeMap<String, String>,
    obligations: BTreeMap<String, String>,
    actions: BTreeMap<String, String>,
    explanation: Json,
}

impl Seen {
    fn claim(&self, id: &str) -> &str {
        self.claims.get(id).map_or("undeclared", String::as_str)
    }

    fn obligation(&self, id: &str) -> &str {
        self.obligations
            .get(id)
            .map_or("undeclared", String::as_str)
    }

    fn action(&self, id: &str) -> &str {
        self.actions.get(id).map_or("undeclared", String::as_str)
    }
}

fn seen(decision: &Decision) -> Seen {
    let entries = |section: &Option<Json>, key: &str| -> BTreeMap<String, String> {
        match section {
            Some(Json::Array(entries)) => entries
                .iter()
                .map(|entry| {
                    (
                        entry[key].as_str().unwrap_or_default().to_owned(),
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
    };
    Seen {
        claims: decision
            .claims
            .iter()
            .map(|(id, entry)| (id.as_str().to_owned(), entry.value.to_string()))
            .collect(),
        obligations: entries(&decision.obligations, "id"),
        actions: entries(&decision.actions, "id"),
        explanation: decision.explanation.clone().unwrap_or(Json::Null),
    }
}

/// What state `state` of `fixture` decides; `None`, with the refusal recorded under `item`, when
/// Canon refuses it.
fn decide(
    failures: &mut Failures,
    item: u8,
    fixture: &Fixture,
    compiled: &Compiled,
    state: &str,
) -> Option<Seen> {
    match fixture.evaluate(compiled, state) {
        Ok(decision) => Some(seen(&decision)),
        Err(refusal) => {
            failures.that(item, false, || {
                format!(
                    "`{}` state `{state}` is refused: {}: {refusal}",
                    fixture.id(),
                    refusal.code()
                )
            });
            None
        }
    }
}

/// Whether some object anywhere in `value` has all of `fields`.
fn contains(value: &Json, fields: &Json) -> bool {
    let wanted = fields.as_object().expect("the fields are an object");
    match value {
        Json::Object(object) => {
            wanted
                .iter()
                .all(|(key, field)| object.get(key) == Some(field))
                || object.values().any(|inner| contains(inner, fields))
        }
        Json::Array(members) => members.iter().any(|inner| contains(inner, fields)),
        _ => false,
    }
}

/// Each record of a state's evidence as `<kind> <result> <revision> <observed_at>`, in the order
/// the fixture gives it.
fn records(fixture: &Fixture, state: &str) -> Vec<String> {
    fixture
        .evidence(state)
        .iter()
        .map(|record| {
            format!(
                "{} {} {} {}",
                record.kind.as_str(),
                record.result.as_deref().unwrap_or("-"),
                record.subject_revision.as_str(),
                record
                    .observed_at
                    .as_ref()
                    .map_or("unplaced", |at| at.as_str())
            )
        })
        .collect()
}

#[test]
fn a_bugfix_reads_red_first_from_evidence_order() {
    let mut failures = Failures::default();
    let path = support::protocol_path("software-change", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));

    // 0. Every fixture of this story holds against the protocol as it ships.
    let loaded: Vec<Fixture> = FIXTURES.iter().map(|id| fixture(id)).collect();
    for fixture in &loaded {
        if let Err(differences) = fixture.check(&compiled) {
            failures.that(0, false, || {
                format!(
                    "`{}` differs from its expectations:\n{}",
                    fixture.id(),
                    differences.join("\n")
                )
            });
        }
    }
    let [red_first, edit_first, green_first] = &loaded[..] else {
        unreachable!("three fixtures")
    };

    // 1. A standard bugfix at R1: the suite passes at 09:00 and the regression test fails at
    //    10:00, both about R1, and both reach Canon with their observation times. The regression
    //    is reproduced, and the implementation may be edited.
    failures.equal(
        1,
        records(red_first, "regression-red"),
        vec![
            "test_result pass R1 2026-10-13T09:00:00Z".to_owned(),
            "regression_test_result fail R1 2026-10-13T10:00:00Z".to_owned(),
        ],
        "the evidence of `regression-red`, as passed to Canon",
    );
    if let Some(seen) = decide(&mut failures, 1, red_first, &compiled, "regression-red") {
        failures.equal(
            1,
            seen.claim("regression.reproduced"),
            "true",
            "regression.reproduced",
        );
        failures.equal(
            1,
            seen.action("implementation.edit"),
            "admissible",
            "implementation.edit",
        );
    }

    // 2. The edit moves the implementation to R2 and records an implementation change at 11:00:
    //    the red run still counts, the regression test ran first, and the obligation is
    //    discharged. A second edit, to R3, still may be made.
    for state in ["edited", "edited-again"] {
        if let Some(seen) = decide(&mut failures, 2, red_first, &compiled, state) {
            for (claim, expected) in [
                ("regression.reproduced", "true"),
                ("regression.shown_first", "true"),
                ("tests.pass", "unknown"),
            ] {
                failures.equal(
                    2,
                    seen.claim(claim),
                    expected,
                    &format!("{claim} at `{state}`"),
                );
            }
            failures.equal(
                2,
                seen.obligation("red_first.required"),
                "discharged",
                &format!("red_first.required at `{state}`"),
            );
            failures.equal(
                2,
                seen.action("implementation.edit"),
                "admissible",
                &format!("implementation.edit at `{state}`"),
            );
        }
    }

    // 3. An implementation change and no regression test result: not shown first, the obligation
    //    open, the edit blocked. A regression test that fails after the change reproduces the
    //    regression but does not discharge the obligation.
    if let Some(seen) = decide(
        &mut failures,
        3,
        edit_first,
        &compiled,
        "edited-unreproduced",
    ) {
        failures.equal(
            3,
            seen.claim("regression.shown_first"),
            "false",
            "regression.shown_first",
        );
        failures.equal(
            3,
            seen.obligation("red_first.required"),
            "open",
            "red_first.required",
        );
        failures.equal(
            3,
            seen.action("implementation.edit"),
            "blocked",
            "implementation.edit",
        );
    }
    if let Some(seen) = decide(&mut failures, 3, edit_first, &compiled, "red-after-edit") {
        failures.equal(
            3,
            seen.claim("regression.shown_first"),
            "false",
            "regression.shown_first after a later red",
        );
        failures.equal(
            3,
            seen.obligation("red_first.required"),
            "open",
            "red_first.required after a later red",
        );
    }

    // 4. The regression test passes first, then fails: `first` is FALSE, and the edit is blocked
    //    until a test result fails about the current revision.
    for state in ["regression-green", "regression-red-later"] {
        if let Some(seen) = decide(&mut failures, 4, green_first, &compiled, state) {
            failures.equal(
                4,
                seen.action("implementation.edit"),
                "blocked",
                &format!("implementation.edit at `{state}`"),
            );
            let form = serde_json::json!({
                "first": "regression_test_result",
                "result": "fail",
                "subject": "implementation",
                "value": "false",
            });
            failures.that(4, contains(&seen.explanation, &form), || {
                format!(
                    "the explanation at `{state}` names no {form}: {}",
                    seen.explanation
                )
            });
        }
    }
    if let Some(seen) = decide(&mut failures, 4, green_first, &compiled, "suite-red") {
        failures.equal(
            4,
            seen.action("implementation.edit"),
            "admissible",
            "implementation.edit once a test result fails about R1",
        );
    }

    // 5. A feature and a refactor: `red_first.required` does not apply.
    for state in ["feature", "refactor"] {
        if let Some(seen) = decide(&mut failures, 5, red_first, &compiled, state) {
            failures.equal(
                5,
                seen.obligation("red_first.required"),
                "not_applicable",
                &format!("red_first.required at `{state}`"),
            );
        }
    }

    // 6. Each term this story adds resolves to exactly one vocabulary entry, in its category.
    for (term, category) in TERMS {
        match vocabulary::lookup(term) {
            Ok(found) => failures.equal(
                6,
                found.category.as_str(),
                category,
                &format!("the category of `{term}`"),
            ),
            Err(error) => failures.that(6, false, || error.to_string()),
        }
        failures.equal(
            6,
            vocabulary::terms()
                .iter()
                .filter(|entry| entry.id == term)
                .count(),
            1,
            &format!("entries declaring `{term}`"),
        );
    }

    assert!(
        failures.0.is_empty(),
        "{} acceptance failures:\n{}",
        failures.0.len(),
        failures.0.join("\n")
    );
}
