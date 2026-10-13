//! Adversarial review of wave 2026-10-13-w3 at `f695bbf`: `story:security-independence-rules`,
//! `story:rollback-verification-rule` and `story:stale-evidence-fixtures`.
//!
//! The behavioural cases build fixtures in memory and evaluate them through the harness, so every
//! state reaches Canon at its own instant exactly as a fixture file's does. The mutant cases write a
//! mutated copy of a protocol under the tree's `target/` (never the protocol itself), compile it and
//! run every fixture of that protocol against it: a mutant no fixture tells apart is a part of a
//! rule no fixture exercises (AGENTS.md: "Every protocol rule lands with a fixture under
//! `fixtures/` that exercises it").

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, ExclusionReason, Json};
use support::{Compiled, Fixture, compile_protocol, repo_root};

const SOFTWARE_CHANGE: &str = "protocols/software-change/1.yaml";
const INCIDENT: &str = "protocols/incident-response/1.yaml";

/// One evidence entry of a fixture state, written as the fixture format writes it.
fn entry(
    observed_at: &str,
    id: &str,
    kind: &str,
    result: &str,
    subject: &str,
    revision: &str,
    producer: Option<&str>,
) -> String {
    let producer = producer.map_or(String::new(), |producer| format!(", producer: {producer}"));
    format!(
        "      - observed_at: \"{observed_at}\"\n        record: {{format: canon-evidence/1, \
         id: {id}, kind: {kind}, result: {result}, subject: {subject}, \
         subject_revision: {revision}{producer}}}\n"
    )
}

/// One state: its id, the extra keys it gives (already indented), and its evidence entries.
fn state(id: &str, extra: &str, evidence: &[String]) -> String {
    let evidence = if evidence.is_empty() {
        "    add_evidence: []\n".to_owned()
    } else {
        format!("    add_evidence:\n{}", evidence.concat())
    };
    format!("  - id: {id}\n{extra}{evidence}    expect: {{claims: {{}}}}\n")
}

/// A `software.change/1` fixture at `at` with case inputs `risk` and `affects_security_boundary`
/// `boundary`, the independence map `independence`, and `states`.
fn software_change(
    at: &str,
    risk: &str,
    boundary: &str,
    independence: &str,
    states: &[String],
) -> Fixture {
    let text = format!(
        "format: els-fixture/1\nid: adversary4\nprotocol: {SOFTWARE_CHANGE}\nat: \"{at}\"\n\
         case:\n  format: canon-case/1\n  id: CHG-9001\n  protocol: software.change\n  inputs:\n    \
         risk: {risk}\n    change_kind: feature\n    affects_security_boundary: {boundary}\n  \
         independence: {independence}\n  artifacts:\n    intent: {{revision: i1}}\n    \
         system_specification: {{revision: s1}}\n    plan: {{revision: p1}}\n    \
         implementation: {{revision: R1}}\n    release: {{revision: v0}}\n    \
         deployment: {{revision: d0}}\nstates:\n{}",
        states.concat()
    );
    Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}\n{text}"))
}

/// An `incident.response/1` fixture of the INC-492 case at `at`, with `release.rollback` granted in
/// its first state, the independence map `independence`, and `states`.
fn incident(at: &str, independence: &str, states: &[String]) -> Fixture {
    let text = format!(
        "format: els-fixture/1\nid: adversary4\nprotocol: {INCIDENT}\nat: \"{at}\"\n\
         case:\n  format: canon-case/1\n  id: INC-492\n  protocol: incident.response\n  \
         independence: {independence}\n  artifacts:\n    service: {{revision: s1}}\n    \
         release: {{revision: r42}}\nstates:\n{}",
        states.concat()
    );
    Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn compiled(path: &str) -> Compiled {
    compile_protocol(path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn decide(fixture: &Fixture, compiled: &Compiled, state: &str) -> Decision {
    fixture
        .evaluate(compiled, state)
        .unwrap_or_else(|refusal| panic!("state `{state}` refused: {}: {refusal}", refusal.code()))
}

fn claim(decision: &Decision, id: &str) -> String {
    decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .map_or("undeclared".to_owned(), |(_, entry)| {
            entry.value.to_string()
        })
}

fn excluded(decision: &Decision, id: &str) -> Vec<(String, ExclusionReason)> {
    decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .map(|(_, entry)| {
            entry
                .excluded_evidence
                .iter()
                .map(|exclusion| (exclusion.evidence.as_str().to_owned(), exclusion.reason))
                .collect()
        })
        .unwrap_or_default()
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

fn obligation(decision: &Decision, id: &str) -> String {
    statuses(&decision.obligations)
        .get(id)
        .cloned()
        .unwrap_or_else(|| "undeclared".to_owned())
}

fn action(decision: &Decision, id: &str) -> String {
    statuses(&decision.actions)
        .get(id)
        .cloned()
        .unwrap_or_else(|| "undeclared".to_owned())
}

const TWO: &str = "{p1: [], p2: [different_principal], p3: [different_agent_run]}";

fn tests_pass_r1() -> String {
    entry(
        "2026-10-13T09:00:00Z",
        "tests-r1",
        "test_result",
        "pass",
        "implementation",
        "R1",
        Some("p1"),
    )
}

fn security(at: &str, id: &str, result: &str, revision: &str, producer: &str) -> String {
    entry(
        at,
        id,
        "security_review",
        result,
        "implementation",
        revision,
        Some(producer),
    )
}

// ---------------------------------------------------------------------------------------------
// story:security-independence-rules
// ---------------------------------------------------------------------------------------------

/// An independent rejection beside an independent approval: the reviews disagree, so
/// `security.reviewed` is UNKNOWN, not TRUE, and merging stays blocked.
#[test]
fn independent_rejection_beside_independent_approval_is_unknown() {
    let compiled = compiled(SOFTWARE_CHANGE);
    let fixture = software_change(
        "2026-10-13T12:00:00Z",
        "standard",
        "true",
        "{p1: [], p2: [different_principal], p4: [different_principal]}",
        &[state(
            "disagree",
            "",
            &[
                tests_pass_r1(),
                security("2026-10-13T10:00:00Z", "sr-p2", "approved", "R1", "p2"),
                security("2026-10-13T10:05:00Z", "sr-p4", "rejected", "R1", "p4"),
            ],
        )],
    );
    let decision = decide(&fixture, &compiled, "disagree");
    assert_eq!(claim(&decision, "security.reviewed"), "unknown");
    assert_eq!(claim(&decision, "implementation.verified"), "unknown");
    assert_eq!(obligation(&decision, "security_review.required"), "open");
    assert_eq!(action(&decision, "repository.merge"), "blocked");
}

/// An independent rejection alone makes the claim FALSE and keeps the implementation unverified.
#[test]
fn independent_rejection_alone_is_false() {
    let compiled = compiled(SOFTWARE_CHANGE);
    let fixture = software_change(
        "2026-10-13T12:00:00Z",
        "standard",
        "true",
        TWO,
        &[state(
            "rejected",
            "",
            &[
                tests_pass_r1(),
                security("2026-10-13T10:00:00Z", "sr-p2", "rejected", "R1", "p2"),
            ],
        )],
    );
    let decision = decide(&fixture, &compiled, "rejected");
    assert_eq!(claim(&decision, "security.reviewed"), "false");
    assert_eq!(claim(&decision, "implementation.verified"), "false");
    assert_eq!(obligation(&decision, "security_review.required"), "open");
    assert_eq!(action(&decision, "repository.merge"), "blocked");
}

/// An independent approval of a revision the implementation no longer has is stale: excluded as
/// `revision_mismatch`, the claim UNKNOWN even with current tests passing.
#[test]
fn independent_approval_of_a_stale_revision_is_unknown() {
    let compiled = compiled(SOFTWARE_CHANGE);
    let fixture = software_change(
        "2026-10-13T12:00:00Z",
        "standard",
        "true",
        TWO,
        &[
            state(
                "reviewed-r1",
                "",
                &[
                    tests_pass_r1(),
                    security("2026-10-13T10:00:00Z", "sr-p2", "approved", "R1", "p2"),
                ],
            ),
            state(
                "moved-r2",
                "    set_revisions: {implementation: R2}\n",
                &[entry(
                    "2026-10-13T11:00:00Z",
                    "tests-r2",
                    "test_result",
                    "pass",
                    "implementation",
                    "R2",
                    Some("p1"),
                )],
            ),
        ],
    );
    let before = decide(&fixture, &compiled, "reviewed-r1");
    assert_eq!(claim(&before, "security.reviewed"), "true");
    let after = decide(&fixture, &compiled, "moved-r2");
    assert_eq!(claim(&after, "tests.pass"), "true");
    assert_eq!(claim(&after, "security.reviewed"), "unknown");
    assert_eq!(claim(&after, "implementation.verified"), "unknown");
    assert_eq!(
        excluded(&after, "security.reviewed"),
        [("sr-p2".to_owned(), ExclusionReason::RevisionMismatch)]
    );
    assert_eq!(action(&after, "repository.merge"), "blocked");
}

/// A producer the case lists independent only on `different_agent_run` is not independent on
/// `different_principal`: its approval is `not_independent`; a record naming no producer too.
#[test]
fn agent_run_independence_or_no_producer_does_not_review() {
    let compiled = compiled(SOFTWARE_CHANGE);
    let fixture = software_change(
        "2026-10-13T12:00:00Z",
        "standard",
        "true",
        TWO,
        &[state(
            "other-run",
            "",
            &[
                tests_pass_r1(),
                security("2026-10-13T10:00:00Z", "sr-p3", "approved", "R1", "p3"),
                entry(
                    "2026-10-13T10:01:00Z",
                    "sr-anon",
                    "security_review",
                    "approved",
                    "implementation",
                    "R1",
                    None,
                ),
                security(
                    "2026-10-13T10:02:00Z",
                    "sr-unlisted",
                    "approved",
                    "R1",
                    "p9",
                ),
            ],
        )],
    );
    let decision = decide(&fixture, &compiled, "other-run");
    assert_eq!(claim(&decision, "security.reviewed"), "unknown");
    assert_eq!(claim(&decision, "implementation.verified"), "unknown");
    let mut found = excluded(&decision, "security.reviewed");
    found.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        found,
        [
            ("sr-anon".to_owned(), ExclusionReason::NotIndependent),
            ("sr-p3".to_owned(), ExclusionReason::NotIndependent),
            ("sr-unlisted".to_owned(), ExclusionReason::NotIndependent),
        ]
    );
}

/// The implementer's own rejection does not count against an independent approval: the claim is
/// TRUE (the record is not of the match, CANON-INDEPENDENCE-001). Recorded behaviour.
#[test]
fn implementer_rejection_beside_independent_approval_is_true() {
    let compiled = compiled(SOFTWARE_CHANGE);
    let fixture = software_change(
        "2026-10-13T12:00:00Z",
        "standard",
        "true",
        TWO,
        &[state(
            "mixed",
            "",
            &[
                tests_pass_r1(),
                security("2026-10-13T10:00:00Z", "sr-p1", "rejected", "R1", "p1"),
                security("2026-10-13T10:05:00Z", "sr-p2", "approved", "R1", "p2"),
            ],
        )],
    );
    let decision = decide(&fixture, &compiled, "mixed");
    assert_eq!(claim(&decision, "security.reviewed"), "true");
    assert_eq!(claim(&decision, "implementation.verified"), "true");
}

/// On a security boundary, at every risk: the obligation applies, merging is blocked without an
/// independent approval and waits only for authority with one.
#[test]
fn merge_on_a_security_boundary_at_every_risk() {
    let compiled = compiled(SOFTWARE_CHANGE);
    for risk in ["trivial", "standard", "elevated", "critical"] {
        let fixture = software_change(
            "2026-10-13T12:00:00Z",
            risk,
            "true",
            TWO,
            &[
                state("unreviewed", "", &[tests_pass_r1()]),
                state(
                    "self-reviewed",
                    "",
                    &[security(
                        "2026-10-13T10:00:00Z",
                        "sr-p1",
                        "approved",
                        "R1",
                        "p1",
                    )],
                ),
                state(
                    "reviewed",
                    "",
                    &[security(
                        "2026-10-13T10:05:00Z",
                        "sr-p2",
                        "approved",
                        "R1",
                        "p2",
                    )],
                ),
            ],
        );
        for (state, merge, required) in [
            ("unreviewed", "blocked", "open"),
            ("self-reviewed", "blocked", "open"),
            ("reviewed", "approval-required", "discharged"),
        ] {
            let decision = decide(&fixture, &compiled, state);
            assert_eq!(
                action(&decision, "repository.merge"),
                merge,
                "{risk} {state}"
            );
            assert_eq!(
                obligation(&decision, "security_review.required"),
                required,
                "{risk} {state}"
            );
            assert_eq!(
                obligation(&decision, "tests.required"),
                "discharged",
                "{risk} {state}"
            );
        }
    }
}

/// `affects_security_boundary` flipped to `false` after an independent rejection verifies the
/// implementation: the input is a snapshot value, and nothing holds it to the value the case
/// opened with. Recorded behaviour; the vocabulary calls the input "fixed when its case opens".
#[test]
fn flipping_the_boundary_input_after_a_rejection_verifies() {
    let compiled = compiled(SOFTWARE_CHANGE);
    let fixture = software_change(
        "2026-10-13T12:00:00Z",
        "standard",
        "true",
        TWO,
        &[
            state(
                "rejected",
                "",
                &[
                    tests_pass_r1(),
                    security("2026-10-13T10:00:00Z", "sr-p2", "rejected", "R1", "p2"),
                ],
            ),
            state(
                "flipped",
                "    set_inputs: {affects_security_boundary: false}\n",
                &[],
            ),
        ],
    );
    assert_eq!(
        action(&decide(&fixture, &compiled, "rejected"), "repository.merge"),
        "blocked"
    );
    let flipped = decide(&fixture, &compiled, "flipped");
    assert_eq!(claim(&flipped, "security.reviewed"), "false");
    assert_eq!(claim(&flipped, "implementation.verified"), "true");
    assert_eq!(action(&flipped, "repository.merge"), "approval-required");
}

// ---------------------------------------------------------------------------------------------
// story:rollback-verification-rule
// ---------------------------------------------------------------------------------------------

const VERIFIERS: &str = "{actor-a: [], verifier-b: [different_principal]}";

fn rollback(
    at: &str,
    id: &str,
    result: &str,
    subject: &str,
    revision: &str,
    producer: &str,
) -> String {
    entry(
        at,
        id,
        "rollback_result",
        result,
        subject,
        revision,
        Some(producer),
    )
}

/// A partial rollback reported by an independent verifier contradicts the claim: FALSE.
#[test]
fn independent_partial_rollback_is_false() {
    let compiled = compiled(INCIDENT);
    let fixture = incident(
        "2026-10-13T12:00:00Z",
        VERIFIERS,
        &[state(
            "partial",
            "    add_authority:\n      - {capability: release.rollback, decision: granted}\n",
            &[rollback(
                "2026-10-13T11:45:00Z",
                "rb-b",
                "partial",
                "release",
                "r42",
                "verifier-b",
            )],
        )],
    );
    assert_eq!(
        claim(&decide(&fixture, &compiled, "partial"), "rollback.verified"),
        "false"
    );
}

/// A complete rollback about the wrong subject, or about a release revision that is no longer
/// current, verifies nothing: UNKNOWN.
#[test]
fn independent_complete_rollback_of_the_wrong_subject_or_revision_is_unknown() {
    let compiled = compiled(INCIDENT);
    let fixture = incident(
        "2026-10-13T12:00:00Z",
        VERIFIERS,
        &[
            state(
                "about-service",
                "",
                &[rollback(
                    "2026-10-13T11:45:00Z",
                    "rb-service",
                    "complete",
                    "service",
                    "s1",
                    "verifier-b",
                )],
            ),
            state(
                "about-old-release",
                "",
                &[rollback(
                    "2026-10-13T11:46:00Z",
                    "rb-r41",
                    "complete",
                    "release",
                    "r41",
                    "verifier-b",
                )],
            ),
        ],
    );
    assert_eq!(
        claim(
            &decide(&fixture, &compiled, "about-service"),
            "rollback.verified"
        ),
        "unknown"
    );
    let old = decide(&fixture, &compiled, "about-old-release");
    assert_eq!(claim(&old, "rollback.verified"), "unknown");
    assert_eq!(
        excluded(&old, "rollback.verified"),
        [("rb-r41".to_owned(), ExclusionReason::RevisionMismatch)]
    );
}

// ---------------------------------------------------------------------------------------------
// story:stale-evidence-fixtures
// ---------------------------------------------------------------------------------------------

/// An operational observation exactly 30 minutes old still applies; one second later it has
/// expired, and a later state without its own `at` keeps the later instant. Both protocols.
#[test]
fn observation_horizon_is_exactly_thirty_minutes() {
    let software = compiled(SOFTWARE_CHANGE);
    let fixture = software_change(
        "2026-10-13T11:30:00Z",
        "critical",
        "false",
        "{}",
        &[
            state(
                "exactly-30m",
                "",
                &[entry(
                    "2026-10-13T11:00:00Z",
                    "health-d0",
                    "operational_observation",
                    "healthy",
                    "deployment",
                    "d0",
                    None,
                )],
            ),
            state("30m-1s", "    at: \"2026-10-13T11:30:01Z\"\n", &[]),
            state("inherits", "", &[]),
        ],
    );
    assert_eq!(fixture.instant("inherits"), "2026-10-13T11:30:01Z");
    let exact = decide(&fixture, &software, "exactly-30m");
    assert_eq!(claim(&exact, "deployment.healthy"), "true");
    assert_eq!(obligation(&exact, "health.required"), "discharged");
    for state in ["30m-1s", "inherits"] {
        let late = decide(&fixture, &software, state);
        assert_eq!(claim(&late, "deployment.healthy"), "unknown", "{state}");
        assert_eq!(
            excluded(&late, "deployment.healthy"),
            [("health-d0".to_owned(), ExclusionReason::Expired)],
            "{state}"
        );
        assert_eq!(obligation(&late, "health.required"), "open", "{state}");
    }

    let incident_ir = compiled(INCIDENT);
    let fixture = incident(
        "2026-10-13T11:30:00Z",
        "{}",
        &[
            state(
                "exactly-30m",
                "",
                &[
                    entry(
                        "2026-10-13T10:00:00Z",
                        "impact-1",
                        "impact_assessment",
                        "bounded",
                        "service",
                        "s1",
                        None,
                    ),
                    entry(
                        "2026-10-13T11:00:00Z",
                        "health-1",
                        "operational_observation",
                        "healthy",
                        "service",
                        "s1",
                        None,
                    ),
                ],
            ),
            state("30m-1s", "    at: \"2026-10-13T11:30:01Z\"\n", &[]),
        ],
    );
    let exact = decide(&fixture, &incident_ir, "exactly-30m");
    assert_eq!(claim(&exact, "service.healthy"), "true");
    assert_eq!(action(&exact, "emergency.leave"), "admissible");
    let late = decide(&fixture, &incident_ir, "30m-1s");
    assert_eq!(claim(&late, "service.healthy"), "unknown");
    assert_eq!(action(&late, "emergency.leave"), "blocked");
}

/// An expired healthy observation no longer outvotes a fresh unhealthy one, and an expired
/// unhealthy one no longer contradicts a fresh healthy one.
#[test]
fn expiry_drops_old_contrary_observations() {
    let compiled = compiled(INCIDENT);
    let fixture = incident(
        "2026-10-13T11:20:00Z",
        "{}",
        &[
            state(
                "both-fresh",
                "",
                &[
                    entry(
                        "2026-10-13T11:00:00Z",
                        "unhealthy-1",
                        "operational_observation",
                        "unhealthy",
                        "service",
                        "s1",
                        None,
                    ),
                    entry(
                        "2026-10-13T11:15:00Z",
                        "healthy-2",
                        "operational_observation",
                        "healthy",
                        "service",
                        "s1",
                        None,
                    ),
                ],
            ),
            state("old-one-expired", "    at: \"2026-10-13T11:31:00Z\"\n", &[]),
        ],
    );
    assert_eq!(
        claim(
            &decide(&fixture, &compiled, "both-fresh"),
            "service.healthy"
        ),
        "unknown"
    );
    assert_eq!(
        claim(
            &decide(&fixture, &compiled, "old-one-expired"),
            "service.healthy"
        ),
        "true"
    );
}

// ---------------------------------------------------------------------------------------------
// Mutants
// ---------------------------------------------------------------------------------------------

/// The fixtures under `fixtures/<directory>/` that fail against `protocol` with `from` replaced by
/// `to` once; panics when `from` is not in the protocol exactly once.
fn killed_by(protocol: &str, directory: &str, name: &str, from: &str, to: &str) -> Vec<String> {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(protocol)).expect("protocol reads");
    assert_eq!(text.matches(from).count(), 1, "mutant `{name}`: {from:?}");
    let mutant = text.replacen(from, to, 1);
    std::fs::create_dir_all(root.join("target/adversary4-mutants")).expect("mutant directory");
    let path = format!("target/adversary4-mutants/{name}.yaml");
    std::fs::write(root.join(&path), mutant).expect("mutant writes");
    let compiled = compile_protocol(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
    let mut files: Vec<String> = std::fs::read_dir(root.join("fixtures").join(directory))
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
    files.sort();
    files
        .into_iter()
        .filter(|file| {
            let fixture = Fixture::load(&format!("fixtures/{directory}/{file}"))
                .unwrap_or_else(|error| panic!("{file}: {error}"));
            fixture.check(&compiled).is_err()
        })
        .collect()
}

/// Mutants of `software.change/1`'s new rules each killed by some software-change fixture.
#[test]
fn software_change_mutants_are_killed_by_a_fixture() {
    let security_reviewed = "        kind: security_review\n        result: approved\n        \
                             subject: implementation\n        independent_on: [different_principal]\n";
    let mutants: [(&str, &str, String); 7] = [
        (
            "sc-drop-independent-on",
            "        independent_on: [different_principal]\n",
            String::new(),
        ),
        (
            "sc-flip-boundary-false",
            "            - input: affects_security_boundary\n              in: [false]\n",
            "            - input: affects_security_boundary\n              in: [true]\n".to_owned(),
        ),
        ("sc-drop-max-age", "    max_age: 30m\n", String::new()),
        (
            "sc-drop-approved",
            security_reviewed,
            "        kind: security_review\n        subject: implementation\n        \
             independent_on: [different_principal]\n"
                .to_owned(),
        ),
        (
            "sc-agent-run",
            "        independent_on: [different_principal]\n",
            "        independent_on: [different_agent_run]\n".to_owned(),
        ),
        (
            "sc-any-to-all",
            "        - any:\n            - input: affects_security_boundary\n",
            "        - all:\n            - input: affects_security_boundary\n".to_owned(),
        ),
        (
            "sc-obligation-flip",
            "      input: affects_security_boundary\n      in: [true]\n",
            "      input: affects_security_boundary\n      in: [false]\n".to_owned(),
        ),
    ];
    let mut survivors = Vec::new();
    for (name, from, to) in &mutants {
        let killed = killed_by(SOFTWARE_CHANGE, "software-change", name, from, to);
        if killed.is_empty() {
            survivors.push(*name);
        }
    }
    assert!(
        survivors.is_empty(),
        "mutants no software-change fixture kills: {survivors:?}"
    );
}

/// Mutants of `incident.response/1`'s new rules each killed by some incident-response fixture.
#[test]
fn incident_mutants_are_killed_by_a_fixture() {
    let mutants: [(&str, &str, &str); 3] = [
        (
            "ir-drop-independent-on",
            "        independent_on: [different_principal]\n",
            "",
        ),
        ("ir-drop-max-age", "    max_age: 30m\n", ""),
        (
            "ir-drop-complete",
            "        kind: rollback_result\n        result: complete\n",
            "        kind: rollback_result\n",
        ),
    ];
    let mut survivors = Vec::new();
    for (name, from, to) in mutants {
        let killed = killed_by(INCIDENT, "incident-response", name, from, to);
        if killed.is_empty() {
            survivors.push(name);
        }
    }
    assert!(
        survivors.is_empty(),
        "mutants no incident-response fixture kills: {survivors:?}"
    );
}

/// `rollback.verified` without its `subject: release`: no incident-response fixture holds a
/// rollback result about another artifact. `incident_response_protocol.rs` item 5 pins the
/// subject structurally, so the suite does catch this mutant; no fixture exercises it.
#[test]
fn incident_rollback_subject_mutant_is_killed_by_a_fixture() {
    let killed = killed_by(
        INCIDENT,
        "incident-response",
        "ir-drop-subject",
        "        result: complete\n        subject: release\n",
        "        result: complete\n",
    );
    assert!(
        !killed.is_empty(),
        "no incident-response fixture kills ir-drop-subject"
    );
}
