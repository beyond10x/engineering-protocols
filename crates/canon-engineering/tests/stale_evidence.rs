//! Acceptance for `story:stale-evidence-fixtures`: stale evidence is inapplicable, never contrary.
//! A record bound to a revision its subject no longer has is excluded as `revision_mismatch`
//! (C-004), and an operational observation older than its kind's `max_age` of 30 minutes at the
//! evaluation instant is excluded as `expired` (C-008); either way the claim it would have decided
//! is `UNKNOWN`, not `FALSE`. Canon decides both; the fixtures exercise them through the two
//! protocols that declare them, and the harness passes each state's instant to Canon.
//!
//! The fixture `stale-review` (`fixtures/software-change/`) holds items 1 and 2: a review of R1
//! while the implementation is at R2, then a review of R2 added; item 2 is also checked on the
//! fixture with the review of R1 bound to R2 instead. `stale-observation` holds items 3 and 4 and
//! `stale-health` (`fixtures/incident-response/`) items 5 and 6, each as two states over the same
//! single observation at T0: `fresh` at T0 + 29 minutes, then `stale` at T0 + 31 minutes. The
//! test checks every acceptance item and reports each one that fails, in item order, rather than
//! stopping at the first.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, ExclusionReason, Json};
use serde_yaml_ng::Value;
use support::{Compiled, Fixture};

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

    /// Whether the explanation of `claim` lists `evidence` as excluded for `reason`.
    fn explains(&self, claim: &str, evidence: &str, reason: &str) -> bool {
        self.explanation["claims"][claim]["because"]
            .as_array()
            .is_some_and(|because| {
                because.contains(&serde_json::json!({
                    "evidence": evidence,
                    "reason": reason,
                    "status": "excluded",
                }))
            })
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

/// The text of the fixture `<directory>/<id>.fixture.yaml`.
fn text(directory: &str, id: &str) -> String {
    let path = format!("fixtures/{directory}/{id}.fixture.yaml");
    std::fs::read_to_string(support::repo_root().join(&path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The fixture `<directory>/<id>.fixture.yaml`, or `None` with the reason recorded under each of
/// `items` when the harness does not load it.
fn load(failures: &mut Failures, items: &[u8], directory: &str, id: &str) -> Option<Fixture> {
    let path = format!("fixtures/{directory}/{id}.fixture.yaml");
    match Fixture::load(&path) {
        Ok(fixture) => Some(fixture),
        Err(error) => {
            for &item in items {
                failures.that(item, false, || format!("{path} does not load: {error}"));
            }
            None
        }
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

/// Each record of a state's evidence as `<id> <kind> <result> <revision> <observed_at>`, in the
/// order the fixture gives it.
fn records(fixture: &Fixture, state: &str) -> Vec<String> {
    fixture
        .evidence(state)
        .iter()
        .map(|record| {
            format!(
                "{} {} {} {} {}",
                record.id.as_str(),
                record.kind.as_str(),
                record.result.as_deref().unwrap_or("-"),
                record.subject_revision.as_str(),
                record.observed_at.as_ref().map_or("-", |at| at.as_str())
            )
        })
        .collect()
}

/// The instant state `state` of the fixture written as `text` is evaluated at, read from the
/// fixture as written: the state's own `at`, or else that of the state before it, or else the
/// fixture's.
fn instant(text: &str, state: &str) -> String {
    let document: Value = serde_yaml_ng::from_str(text).expect("the fixture is YAML");
    let mut at = document["at"]
        .as_str()
        .expect("the fixture's at")
        .to_owned();
    for written in document["states"].as_sequence().expect("states") {
        if let Some(own) = written.get("at").and_then(Value::as_str) {
            own.clone_into(&mut at);
        }
        if written["id"].as_str() == Some(state) {
            return at;
        }
    }
    panic!("the fixture has no state `{state}`")
}

/// Minutes from `earlier` to `later`, both written `YYYY-MM-DDTHH:MM:SSZ` on the same day.
fn minutes(earlier: &str, later: &str) -> i64 {
    assert_eq!(earlier[..10], later[..10], "{earlier} and {later}: one day");
    let seconds = |at: &str| {
        let field = |from: usize| at[from..from + 2].parse::<i64>().expect("a number");
        field(11) * 3600 + field(14) * 60 + field(17)
    };
    let (from, to) = (seconds(earlier), seconds(later));
    assert_eq!((to - from) % 60, 0, "{earlier} to {later}: whole minutes");
    (to - from) / 60
}

/// Items 3 to 6: the one operational observation of `fixture`, `observation`, evaluated at
/// T0 + 31 minutes (`stale`, item `stale_item`) and at T0 + 29 minutes (`fresh`, item
/// `fresh_item`). Returns what each state decides.
fn freshness_states(
    failures: &mut Failures,
    (fresh_item, stale_item): (u8, u8),
    fixture: &Fixture,
    written: &str,
    compiled: &Compiled,
    observation: &str,
) -> (Option<Seen>, Option<Seen>) {
    failures.equal(
        fresh_item,
        fixture.states(),
        vec!["fresh", "stale"],
        &format!("the states of `{}`", fixture.id()),
    );
    let observed = fixture
        .evidence("fresh")
        .iter()
        .find(|record| record.id.as_str() == observation)
        .and_then(|record| record.observed_at.as_ref())
        .map(|at| at.as_str().to_owned());
    let Some(observed) = observed else {
        failures.that(fresh_item, false, || {
            format!(
                "`{}` state `fresh` holds no `{observation}` with its observation time",
                fixture.id()
            )
        });
        return (None, None);
    };
    failures.equal(
        stale_item,
        records(fixture, "stale"),
        records(fixture, "fresh"),
        &format!(
            "the evidence of `{}` state `stale`: that of `fresh`, nothing added",
            fixture.id()
        ),
    );
    for (item, state, age) in [(fresh_item, "fresh", 29), (stale_item, "stale", 31)] {
        failures.equal(
            item,
            minutes(&observed, &instant(written, state)),
            age,
            &format!(
                "minutes from `{observation}` to the instant of `{}` state `{state}`",
                fixture.id()
            ),
        );
    }
    (
        decide(failures, fresh_item, fixture, compiled, "fresh"),
        decide(failures, stale_item, fixture, compiled, "stale"),
    )
}

#[test]
fn stale_evidence_is_unknown_and_fresh_evidence_holds() {
    let mut failures = Failures::default();
    let compile = |name: &str| {
        let path = support::protocol_path(name, 1);
        support::compile_protocol(&path)
            .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"))
    };
    let change = compile("software-change");
    let incident = compile("incident-response");

    // 0. Every fixture of this story loads, names the protocol it evaluates and holds against
    //    that protocol as it ships, and every operational observation kind has its 30-minute
    //    horizon.
    for (compiled, kind) in [
        (&change, "operational_observation"),
        (&incident, "operational_observation"),
    ] {
        let max_age = compiled
            .ir()
            .evidence_kinds
            .iter()
            .find(|(id, _)| id.as_str() == kind)
            .and_then(|(_, declared)| declared.max_age.as_ref())
            .map(|age| age.as_str().to_owned());
        failures.equal(
            0,
            max_age.as_deref(),
            Some("30m"),
            &format!("the max_age of `{kind}`"),
        );
    }
    let review = load(&mut failures, &[1, 2], "software-change", "stale-review");
    let observation = load(
        &mut failures,
        &[3, 4],
        "software-change",
        "stale-observation",
    );
    let health = load(&mut failures, &[5, 6], "incident-response", "stale-health");
    for (fixture, compiled, path) in [
        (&review, &change, "protocols/software-change/1.yaml"),
        (&observation, &change, "protocols/software-change/1.yaml"),
        (&health, &incident, "protocols/incident-response/1.yaml"),
    ] {
        let Some(fixture) = fixture else { continue };
        failures.equal(
            0,
            fixture.protocol_path(),
            path,
            &format!("the protocol `{}` evaluates", fixture.id()),
        );
        if let Err(differences) = fixture.check(compiled) {
            failures.that(0, false, || {
                format!(
                    "`{}` differs from its expectations:\n{}",
                    fixture.id(),
                    differences.join("\n")
                )
            });
        }
    }

    if let Some(review) = &review {
        // 1. Implementation R2, a passing test result about R2 and one approving review, of R1:
        //    the review is bound to a revision the implementation no longer has. Canon excludes
        //    it as `revision_mismatch`; `implementation.reviewed` is UNKNOWN, not FALSE.
        failures.equal(
            1,
            review
                .case()
                .artifacts
                .iter()
                .find(|(id, _)| id.as_str() == "implementation")
                .map(|(_, artifact)| artifact.revision.as_str().to_owned()),
            Some("R2".to_owned()),
            "the case's implementation revision",
        );
        failures.equal(
            1,
            records(review, "reviewed-r1"),
            vec![
                "review-r1 code_review approved R1 2026-10-13T10:00:00Z".to_owned(),
                "tests-r2 test_result pass R2 2026-10-13T10:30:00Z".to_owned(),
            ],
            "the evidence of `reviewed-r1`",
        );
        if let Some(seen) = decide(&mut failures, 1, review, &change, "reviewed-r1") {
            failures.equal(
                1,
                seen.claim("implementation.reviewed"),
                "unknown",
                "implementation.reviewed",
            );
            failures.equal(
                1,
                seen.excluded("implementation.reviewed"),
                &[("review-r1".to_owned(), ExclusionReason::RevisionMismatch)][..],
                "the evidence excluded from implementation.reviewed",
            );
            failures.that(
                1,
                seen.explains("implementation.reviewed", "review-r1", "revision_mismatch"),
                || {
                    format!(
                        "the explanation of implementation.reviewed lists no `review-r1` as \
                         revision_mismatch: {}",
                        seen.explanation["claims"]["implementation.reviewed"]
                    )
                },
            );
            failures.equal(
                1,
                seen.obligation("review.required"),
                "open",
                "review.required",
            );
        }

        // 2. The same fixture with an approving review of R2: `implementation.reviewed` is TRUE,
        //    with the review of R1 added beside it (state `reviewed-r2`) and with it bound to R2
        //    instead (the fixture's text changed, state `reviewed-r1`).
        failures.equal(
            2,
            records(review, "reviewed-r2").last().map(String::as_str),
            Some("review-r2 code_review approved R2 2026-10-13T11:00:00Z"),
            "the record `reviewed-r2` adds",
        );
        if let Some(seen) = decide(&mut failures, 2, review, &change, "reviewed-r2") {
            failures.equal(
                2,
                seen.claim("implementation.reviewed"),
                "true",
                "implementation.reviewed in `reviewed-r2`",
            );
            failures.equal(
                2,
                seen.obligation("review.required"),
                "discharged",
                "review.required in `reviewed-r2`",
            );
        }
        let written = text("software-change", "stale-review");
        let of_r1 = "id: review-r1, kind: code_review, result: approved,\n                 \
                     subject: implementation, subject_revision: R1}";
        failures.equal(
            2,
            written.matches(of_r1).count(),
            1,
            "the review of R1 as `stale-review` writes it",
        );
        let instead = written.replacen(
            of_r1,
            &of_r1.replace("subject_revision: R1", "subject_revision: R2"),
            1,
        );
        match Fixture::from_yaml(&instead) {
            Ok(instead) => {
                if let Some(seen) = decide(&mut failures, 2, &instead, &change, "reviewed-r1") {
                    failures.equal(
                        2,
                        seen.claim("implementation.reviewed"),
                        "true",
                        "implementation.reviewed with the review bound to R2 instead",
                    );
                    failures.equal(
                        2,
                        seen.excluded("implementation.reviewed"),
                        &[][..],
                        "the evidence excluded with the review bound to R2 instead",
                    );
                }
            }
            Err(error) => failures.that(2, false, || {
                format!("`stale-review` with the review bound to R2 does not load: {error}")
            }),
        }
    }

    if let Some(observation) = &observation {
        // 3 and 4. A deployment observed healthy once, at T0: at T0 + 31 minutes the observation
        //    is excluded as `expired` and `deployment.healthy` is UNKNOWN, not FALSE; at T0 + 29
        //    minutes it holds and `deployment.healthy` is TRUE.
        let written = text("software-change", "stale-observation");
        let (fresh, stale) = freshness_states(
            &mut failures,
            (4, 3),
            observation,
            &written,
            &change,
            "health-d0",
        );
        if let Some(seen) = stale {
            failures.equal(
                3,
                seen.claim("deployment.healthy"),
                "unknown",
                "deployment.healthy at T0 + 31",
            );
            failures.equal(
                3,
                seen.excluded("deployment.healthy"),
                &[("health-d0".to_owned(), ExclusionReason::Expired)][..],
                "the evidence excluded from deployment.healthy at T0 + 31",
            );
            failures.that(
                3,
                seen.explains("deployment.healthy", "health-d0", "expired"),
                || {
                    format!(
                        "the explanation of deployment.healthy at T0 + 31 lists no `health-d0` as \
                         expired: {}",
                        seen.explanation["claims"]["deployment.healthy"]
                    )
                },
            );
            failures.equal(
                3,
                seen.obligation("health.required"),
                "open",
                "health.required at T0 + 31",
            );
        }
        if let Some(seen) = fresh {
            failures.equal(
                4,
                seen.claim("deployment.healthy"),
                "true",
                "deployment.healthy at T0 + 29",
            );
            failures.equal(
                4,
                seen.excluded("deployment.healthy"),
                &[][..],
                "the evidence excluded from deployment.healthy at T0 + 29",
            );
            failures.equal(
                4,
                seen.obligation("health.required"),
                "discharged",
                "health.required at T0 + 29",
            );
        }
    }

    if let Some(health) = &health {
        // 5 and 6. INC-492 with the rollback approved, its impact assessed bounded and the service
        //    observed healthy once, at T0: at T0 + 31 minutes `service.healthy` is UNKNOWN, not
        //    FALSE, `restore_service` is open and `emergency.leave` blocked; at T0 + 29 minutes the
        //    service is healthy, `restore_service` discharged and `emergency.leave` admissible.
        let written = text("incident-response", "stale-health");
        failures.equal(
            5,
            health.case().id.as_str(),
            "INC-492",
            "the case of `stale-health`",
        );
        let (fresh, stale) = freshness_states(
            &mut failures,
            (6, 5),
            health,
            &written,
            &incident,
            "health-1",
        );
        for (item, seen, healthy, excluded, restore, leave) in [
            (
                5,
                &stale,
                "unknown",
                &[("health-1".to_owned(), ExclusionReason::Expired)][..],
                "open",
                "blocked",
            ),
            (6, &fresh, "true", &[][..], "discharged", "admissible"),
        ] {
            let Some(seen) = seen else { continue };
            let when = if item == 5 { "T0 + 31" } else { "T0 + 29" };
            failures.equal(
                item,
                seen.claim("service.healthy"),
                healthy,
                &format!("service.healthy at {when}"),
            );
            failures.equal(
                item,
                seen.excluded("service.healthy"),
                excluded,
                &format!("the evidence excluded from service.healthy at {when}"),
            );
            failures.equal(
                item,
                seen.claim("impact.bounded"),
                "true",
                &format!("impact.bounded at {when}"),
            );
            failures.equal(
                item,
                seen.obligation("restore_service"),
                restore,
                &format!("restore_service at {when}"),
            );
            failures.equal(
                item,
                seen.action("release.rollback"),
                "admissible",
                &format!("release.rollback at {when}"),
            );
            failures.equal(
                item,
                seen.action("emergency.leave"),
                leave,
                &format!("emergency.leave at {when}"),
            );
        }
        if let Some(seen) = &stale {
            failures.that(
                5,
                seen.explains("service.healthy", "health-1", "expired"),
                || {
                    format!(
                        "the explanation of service.healthy at T0 + 31 lists no `health-1` as \
                         expired: {}",
                        seen.explanation["claims"]["service.healthy"]
                    )
                },
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
