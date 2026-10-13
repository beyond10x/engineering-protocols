//! Acceptance for `story:security-independence-rules`: `software.change/1` gains an independent
//! security-review rule. When the case input `affects_security_boundary` is `true`, the obligation
//! `security_review.required` applies, and `implementation.verified` needs `security.reviewed`: an
//! approving `security_review` bound to the current implementation revision whose producer the
//! case lists as independent on `different_principal`. A review by the implementing principal is
//! inapplicable, never contrary: Canon excludes it as `not_independent` and the claim stays
//! `UNKNOWN` (CANON-INDEPENDENCE-001).
//!
//! The fixture `security-boundary` (`fixtures/software-change/`) holds items 1, 2 and 4, and item 3
//! with the implementer's own review still present; one fixture's states are cumulative, so
//! `security-boundary-independent` holds item 3 from the state of item 1 with only the independent
//! review added. Both give the case `independence: {p1: [], p2: [different_principal]}`, read by
//! Canon with the case; P1 is the implementing principal. The test checks every acceptance item
//! and reports each one that fails, in item order, rather than stopping at the first.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;
use std::fs;

use b10x_canon::model::{Decision, ExclusionReason, InputId, Json};
use canon_engineering::vocabulary;
use support::{Compiled, Fixture};

/// The evaluation instant every fixture of this story gives.
const AT: &str = "2026-10-13T12:00:00Z";

/// The fixtures of this story. `security-boundary-rejected` holds an independent rejection and
/// `security-boundary-other-subject` an independent approval about the plan, so that a rule
/// without `result: approved` or `subject: implementation` differs from the protocol on some
/// fixture.
const FIXTURES: [&str; 4] = [
    "security-boundary",
    "security-boundary-independent",
    "security-boundary-rejected",
    "security-boundary-other-subject",
];

/// The terms this story adds, each with the category it is declared in.
const TERMS: [(&str, &str); 5] = [
    ("security_review", "evidence_kind"),
    ("security.reviewed", "claim_id"),
    ("affects_security_boundary", "case_input"),
    ("different_principal", "independence_dimension"),
    ("different_agent_run", "independence_dimension"),
];

/// The failures found so far, each prefixed with its acceptance item (0 for the fixtures of this
/// story as a whole).
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
/// text, the evidence each claim excludes, and the decision's explanation.
struct Seen {
    claims: BTreeMap<String, String>,
    excluded: BTreeMap<String, Vec<(String, ExclusionReason)>>,
    obligations: BTreeMap<String, String>,
    actions: BTreeMap<String, String>,
    explanation: Json,
}

impl Seen {
    fn claim(&self, id: &str) -> &str {
        self.claims.get(id).map_or("undeclared", String::as_str)
    }

    fn excluded(&self, id: &str) -> &[(String, ExclusionReason)] {
        self.excluded.get(id).map_or(&[], Vec::as_slice)
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
    let entries = |section: &Option<Json>| -> BTreeMap<String, String> {
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
    };
    Seen {
        claims: decision
            .claims
            .iter()
            .map(|(id, entry)| (id.as_str().to_owned(), entry.value.to_string()))
            .collect(),
        excluded: decision
            .claims
            .iter()
            .map(|(id, entry)| {
                (
                    id.as_str().to_owned(),
                    entry
                        .excluded_evidence
                        .iter()
                        .map(|exclusion| (exclusion.evidence.as_str().to_owned(), exclusion.reason))
                        .collect(),
                )
            })
            .collect(),
        obligations: entries(&decision.obligations),
        actions: entries(&decision.actions),
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

/// The value the case of `fixture`, as Canon read it, gives `affects_security_boundary`.
fn boundary_input(fixture: &Fixture) -> Option<String> {
    fixture
        .case()
        .inputs
        .get(&InputId::new("affects_security_boundary"))
        .map(|value| value.as_str().to_owned())
}

/// Each record of a state's evidence as `<kind> <result> <revision> <producer>`, in the order the
/// fixture gives it.
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
                    .producer
                    .as_ref()
                    .map_or("-", |producer| producer.as_str())
            )
        })
        .collect()
}

#[test]
fn independent_security_review_gates_implementation_verified() {
    let mut failures = Failures::default();
    let path = support::protocol_path("software-change", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));

    // 0. Every fixture of this story holds against the protocol as it ships, and gives the case
    //    the independence the story names, read by Canon with the case.
    let loaded: Vec<Fixture> = FIXTURES.iter().map(|id| fixture(id)).collect();
    for fixture in &loaded {
        failures.equal(
            0,
            fixture.protocol_path(),
            path.as_str(),
            &format!("the protocol `{}` evaluates", fixture.id()),
        );
        let independence: BTreeMap<String, Vec<String>> = fixture
            .case()
            .independence
            .iter()
            .map(|(producer, dimensions)| {
                (
                    producer.as_str().to_owned(),
                    dimensions
                        .iter()
                        .map(|dimension| dimension.as_str().to_owned())
                        .collect(),
                )
            })
            .collect();
        failures.equal(
            0,
            independence,
            BTreeMap::from([
                ("p1".to_owned(), Vec::new()),
                ("p2".to_owned(), vec!["different_principal".to_owned()]),
            ]),
            &format!("the independence `{}` gives its case", fixture.id()),
        );
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
    let [boundary, independent, ..] = &loaded[..] else {
        unreachable!("four fixtures")
    };

    // 1. Implementation revision R1 on a security boundary, a passing test result about R1 by P1,
    //    no security review: nothing reviewed it, so it is not verified, the obligation is open
    //    and merging is blocked.
    for (fixture, state) in [(boundary, "initial"), (independent, "initial")] {
        failures.equal(
            1,
            boundary_input(fixture).as_deref(),
            Some("true"),
            &format!("affects_security_boundary in `{}`", fixture.id()),
        );
        failures.equal(
            1,
            records(fixture, state),
            vec!["test_result pass R1 p1".to_owned()],
            &format!("the evidence of `{}` state `{state}`", fixture.id()),
        );
        if let Some(seen) = decide(&mut failures, 1, fixture, &compiled, state) {
            for (claim, expected) in [
                ("tests.pass", "true"),
                ("security.reviewed", "unknown"),
                ("implementation.verified", "unknown"),
            ] {
                failures.equal(
                    1,
                    seen.claim(claim),
                    expected,
                    &format!("{claim} in `{}`", fixture.id()),
                );
            }
            failures.equal(
                1,
                seen.obligation("security_review.required"),
                "open",
                &format!("security_review.required in `{}`", fixture.id()),
            );
            failures.equal(
                1,
                seen.action("repository.merge"),
                "blocked",
                &format!("repository.merge in `{}`", fixture.id()),
            );
        }
    }

    // 2. P1, the implementer, approves its own change: the record is inapplicable, not contrary.
    //    The claim is UNKNOWN, not FALSE, and Canon lists the record as `not_independent`, both on
    //    the claim and in the claim's explanation.
    if let Some(seen) = decide(&mut failures, 2, boundary, &compiled, "self-reviewed") {
        failures.equal(
            2,
            records(boundary, "self-reviewed")
                .last()
                .map(String::as_str),
            Some("security_review approved R1 p1"),
            "the record `self-reviewed` adds",
        );
        for (claim, expected) in [
            ("security.reviewed", "unknown"),
            ("implementation.verified", "unknown"),
        ] {
            failures.equal(2, seen.claim(claim), expected, claim);
        }
        failures.equal(
            2,
            seen.excluded("security.reviewed"),
            &[(
                "security-review-p1".to_owned(),
                ExclusionReason::NotIndependent,
            )][..],
            "the evidence excluded from security.reviewed",
        );
        let listed = seen.explanation["claims"]["security.reviewed"]["because"]
            .as_array()
            .is_some_and(|because| {
                because.contains(&serde_json::json!({
                    "evidence": "security-review-p1",
                    "reason": "not_independent",
                    "status": "excluded",
                }))
            });
        failures.that(2, listed, || {
            format!(
                "the explanation of security.reviewed lists no `security-review-p1` as \
                 not_independent: {}",
                seen.explanation["claims"]["security.reviewed"]
            )
        });
        failures.equal(
            2,
            seen.obligation("security_review.required"),
            "open",
            "security_review.required",
        );
        failures.equal(
            2,
            seen.action("repository.merge"),
            "blocked",
            "repository.merge",
        );
    }

    // 3. P2, independent of the implementer on `different_principal`, approves: reviewed,
    //    verified, the obligation discharged and merging waits only for authority; with the
    //    implementer's own review beside it (`security-boundary`) and without it
    //    (`security-boundary-independent`).
    for fixture in [boundary, independent] {
        let state = "independently-reviewed";
        failures.equal(
            3,
            records(fixture, state).last().map(String::as_str),
            Some("security_review approved R1 p2"),
            &format!("the record `{}` state `{state}` adds", fixture.id()),
        );
        if let Some(seen) = decide(&mut failures, 3, fixture, &compiled, state) {
            for (claim, expected) in [
                ("security.reviewed", "true"),
                ("implementation.verified", "true"),
            ] {
                failures.equal(
                    3,
                    seen.claim(claim),
                    expected,
                    &format!("{claim} in `{}`", fixture.id()),
                );
            }
            failures.equal(
                3,
                seen.obligation("security_review.required"),
                "discharged",
                &format!("security_review.required in `{}`", fixture.id()),
            );
            failures.equal(
                3,
                seen.action("repository.merge"),
                "approval-required",
                &format!("repository.merge in `{}`", fixture.id()),
            );
        }
    }
    failures.equal(
        3,
        records(independent, "independently-reviewed").len(),
        2,
        "the records of `security-boundary-independent`: the test result and P2's review only",
    );

    // 4. The state of 1 with `affects_security_boundary: false`: verified on the passing test
    //    result alone, with no security review, and the obligation does not apply.
    failures.equal(
        4,
        records(boundary, "outside-boundary"),
        vec!["test_result pass R1 p1".to_owned()],
        "the evidence of `outside-boundary`",
    );
    if let Some(seen) = decide(&mut failures, 4, boundary, &compiled, "outside-boundary") {
        failures.equal(
            4,
            seen.claim("implementation.verified"),
            "true",
            "implementation.verified",
        );
        failures.equal(
            4,
            seen.obligation("security_review.required"),
            "not_applicable",
            "security_review.required",
        );
    }

    // 5. Each term this story adds resolves to exactly one vocabulary entry, in its category.
    for (term, category) in TERMS {
        match vocabulary::lookup(term) {
            Ok(found) => failures.equal(
                5,
                found.category.as_str(),
                category,
                &format!("the category of `{term}`"),
            ),
            Err(error) => failures.that(5, false, || error.to_string()),
        }
        failures.equal(
            5,
            vocabulary::terms()
                .iter()
                .filter(|found| found.id == term)
                .count(),
            1,
            &format!("the entries for `{term}`"),
        );
    }

    // 6. Every other fixture of software.change/1 gives `affects_security_boundary: false` and
    //    still holds against the protocol.
    let directory = support::repo_root().join("fixtures/software-change");
    let mut others: Vec<String> = fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| entry.expect("a directory entry").file_name())
        .filter_map(|name| {
            name.to_str()?
                .strip_suffix(".fixture.yaml")
                .map(str::to_owned)
        })
        .filter(|id| !FIXTURES.contains(&id.as_str()))
        .collect();
    others.sort();
    failures.that(6, !others.is_empty(), || {
        format!("{} holds no other fixture", directory.display())
    });
    for id in &others {
        let path = format!("fixtures/software-change/{id}.fixture.yaml");
        let other = match Fixture::load(&path) {
            Ok(other) => other,
            Err(error) => {
                failures.that(6, false, || format!("{path} does not load: {error}"));
                continue;
            }
        };
        failures.equal(
            6,
            boundary_input(&other).as_deref(),
            Some("false"),
            &format!("affects_security_boundary in `{id}`"),
        );
        if let Err(differences) = other.check(&compiled) {
            failures.that(6, false, || {
                format!(
                    "`{id}` differs from its expectations:\n{}",
                    differences.join("\n")
                )
            });
        }
    }

    assert!(
        failures.0.is_empty(),
        "{} acceptance failures:\n{}",
        failures.0.len(),
        failures.0.join("\n")
    );
}
