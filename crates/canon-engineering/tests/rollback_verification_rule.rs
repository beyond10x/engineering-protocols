//! Acceptance for `story:rollback-verification-rule`: `incident.response/1` gains an independent
//! rollback-verification rule. The claim `rollback.verified` holds on a `rollback_result` with
//! result `complete` about the release whose producer the case lists as independent on
//! `different_principal`. A result reported by the actor who performed `release.rollback` is
//! inapplicable, never contrary: Canon excludes it as `not_independent` and the claim stays
//! `UNKNOWN` (CANON-INDEPENDENCE-001).
//!
//! The fixture `rollback-verification` (`fixtures/incident-response/`) holds items 1 and 2, and
//! item 3 with actor A's own result still present; one fixture's states are cumulative, so
//! `rollback-verification-independent` holds item 3 from the state of item 1 with only verifier
//! B's result added. Both give the case `independence: {actor-a: [], verifier-b:
//! [different_principal]}`, read by Canon with the case; actor A performed `release.rollback`. The
//! test checks every acceptance item and reports each one that fails, in item order, rather than
//! stopping at the first.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, ExclusionReason, Json};
use canon_engineering::vocabulary;
use support::{Compiled, Fixture};

/// The evaluation instant every fixture of this story gives.
const AT: &str = "2026-10-13T12:00:00Z";

/// The fixtures of this story.
const FIXTURES: [&str; 2] = ["rollback-verification", "rollback-verification-independent"];

/// The terms the rule names, each with the category it is declared in: the claim this story adds,
/// and the evidence kind and independence dimension it declares in the incident document.
const TERMS: [(&str, &str); 3] = [
    ("rollback.verified", "claim_id"),
    ("rollback_result", "evidence_kind"),
    ("different_principal", "independence_dimension"),
];

/// The states of `inc-492`, which this story keeps.
const INC_492_STATES: [&str; 7] = [
    "initial",
    "cause-identified",
    "rollback-approved",
    "rolled-back",
    "release-observed",
    "service-restored",
    "cause-identified-after-restore",
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
    let path = format!("fixtures/incident-response/{id}.fixture.yaml");
    Fixture::load(&path).unwrap_or_else(|error| panic!("{path} loads: {error}"))
}

/// What one state of a fixture decides: claim values and action statuses, each as text, the
/// evidence each claim excludes, and the decision's explanation.
struct Seen {
    claims: BTreeMap<String, String>,
    excluded: BTreeMap<String, Vec<(String, ExclusionReason)>>,
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

    fn action(&self, id: &str) -> &str {
        self.actions.get(id).map_or("undeclared", String::as_str)
    }
}

fn seen(decision: &Decision) -> Seen {
    let actions = match &decision.actions {
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
        actions,
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

/// Each `rollback_result` of a state's evidence as `<id> <result> <subject> <revision>
/// <producer>`, in the order the fixture gives it.
fn rollback_results(fixture: &Fixture, state: &str) -> Vec<String> {
    fixture
        .evidence(state)
        .iter()
        .filter(|record| record.kind.as_str() == "rollback_result")
        .map(|record| {
            format!(
                "{} {} {} {} {}",
                record.id.as_str(),
                record.result.as_deref().unwrap_or("-"),
                record.subject.as_str(),
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
fn rollback_verified_needs_an_independent_verifier() {
    let mut failures = Failures::default();
    let path = support::protocol_path("incident-response", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));

    // 0. Every fixture of this story holds against the protocol as it ships, and gives the case
    //    the independence the story names, read by Canon with the case.
    let loaded: Vec<Fixture> = FIXTURES.iter().map(|id| fixture(id)).collect();
    for fixture in &loaded {
        failures.equal(
            0,
            fixture.at(),
            AT,
            &format!("the evaluation instant of `{}`", fixture.id()),
        );
        failures.equal(
            0,
            fixture.protocol_path(),
            path.as_str(),
            &format!("the protocol `{}` evaluates", fixture.id()),
        );
        failures.equal(
            0,
            fixture.case().id.as_str(),
            "INC-492",
            &format!("the case of `{}`", fixture.id()),
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
                ("actor-a".to_owned(), Vec::new()),
                (
                    "verifier-b".to_owned(),
                    vec!["different_principal".to_owned()],
                ),
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
    let [rollback, independent] = &loaded[..] else {
        unreachable!("two fixtures")
    };

    // 1. The INC-492 case with `release.rollback` approved and no `rollback_result`: nothing
    //    verified the rollback, so `rollback.verified` is UNKNOWN.
    for fixture in [rollback, independent] {
        let state = "rollback-approved";
        failures.equal(
            1,
            rollback_results(fixture, state),
            Vec::new(),
            &format!("the rollback results of `{}` state `{state}`", fixture.id()),
        );
        if let Some(seen) = decide(&mut failures, 1, fixture, &compiled, state) {
            failures.equal(
                1,
                seen.action("release.rollback"),
                "admissible",
                &format!("release.rollback in `{}`", fixture.id()),
            );
            failures.equal(
                1,
                seen.claim("rollback.verified"),
                "unknown",
                &format!("rollback.verified in `{}`", fixture.id()),
            );
        }
    }

    // 2. Actor A, who rolled the release back, reports the rollback complete: the record is
    //    inapplicable, not contrary. The claim is UNKNOWN, not FALSE, and Canon lists the record
    //    as `not_independent`, both on the claim and in the claim's explanation.
    let state = "self-verified";
    failures.equal(
        2,
        rollback_results(rollback, state),
        vec!["rollback-actor-a complete release r42 actor-a".to_owned()],
        "the rollback results of `self-verified`",
    );
    if let Some(seen) = decide(&mut failures, 2, rollback, &compiled, state) {
        failures.equal(
            2,
            seen.claim("rollback.verified"),
            "unknown",
            "rollback.verified",
        );
        failures.equal(
            2,
            seen.excluded("rollback.verified"),
            &[(
                "rollback-actor-a".to_owned(),
                ExclusionReason::NotIndependent,
            )][..],
            "the evidence excluded from rollback.verified",
        );
        let listed = seen.explanation["claims"]["rollback.verified"]["because"]
            .as_array()
            .is_some_and(|because| {
                because.contains(&serde_json::json!({
                    "evidence": "rollback-actor-a",
                    "reason": "not_independent",
                    "status": "excluded",
                }))
            });
        failures.that(2, listed, || {
            format!(
                "the explanation of rollback.verified lists no `rollback-actor-a` as \
                 not_independent: {}",
                seen.explanation["claims"]["rollback.verified"]
            )
        });
    }

    // 3. Verifier B, independent of actor A on `different_principal`, reports the rollback
    //    complete: verified, with actor A's own result beside it (`rollback-verification`, where
    //    it stays excluded) and without it (`rollback-verification-independent`).
    let state = "independently-verified";
    for (fixture, results) in [
        (
            rollback,
            vec![
                "rollback-actor-a complete release r42 actor-a",
                "rollback-verifier-b complete release r42 verifier-b",
            ],
        ),
        (
            independent,
            vec!["rollback-verifier-b complete release r42 verifier-b"],
        ),
    ] {
        failures.equal(
            3,
            rollback_results(fixture, state),
            results.iter().map(|result| (*result).to_owned()).collect(),
            &format!("the rollback results of `{}` state `{state}`", fixture.id()),
        );
        if let Some(seen) = decide(&mut failures, 3, fixture, &compiled, state) {
            failures.equal(
                3,
                seen.claim("rollback.verified"),
                "true",
                &format!("rollback.verified in `{}`", fixture.id()),
            );
        }
    }

    // 4. Each term the rule names resolves to exactly one vocabulary entry, in its category.
    for (term, category) in TERMS {
        match vocabulary::lookup(term) {
            Ok(found) => failures.equal(
                4,
                found.category.as_str(),
                category,
                &format!("the category of `{term}`"),
            ),
            Err(error) => failures.that(4, false, || error.to_string()),
        }
        failures.equal(
            4,
            vocabulary::terms()
                .iter()
                .filter(|found| found.id == term)
                .count(),
            1,
            &format!("the entries for `{term}`"),
        );
    }

    // 5. `inc-492` keeps its states and every expectation it had, each state's claims now giving
    //    `rollback.verified`, which no state of it establishes.
    let inc_492 = fixture("inc-492");
    failures.equal(
        5,
        inc_492.states(),
        INC_492_STATES.to_vec(),
        "the states of `inc-492`",
    );
    if let Err(differences) = inc_492.check(&compiled) {
        failures.that(5, false, || {
            format!(
                "`inc-492` differs from its expectations:\n{}",
                differences.join("\n")
            )
        });
    }
    for state in INC_492_STATES {
        if let Some(seen) = decide(&mut failures, 5, &inc_492, &compiled, state) {
            failures.equal(
                5,
                seen.claim("rollback.verified"),
                "unknown",
                &format!("rollback.verified in `inc-492` state `{state}`"),
            );
        }
    }

    assert!(
        failures.0.is_empty(),
        "{} acceptance failures:\n{}",
        failures.0.len(),
        failures.0.join("\n")
    );
}
