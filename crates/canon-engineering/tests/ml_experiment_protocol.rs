//! Acceptance for `story:ml-experiment-protocol`: `ml.experiment/1` ships as Canon `protocol/1`
//! data, a fourth built-in, and its outcomes are earned through the fixture harness and Canon.
//!
//! The pressure test is refutation: `refuted` needs `candidate.beats_baseline` FALSE, and UNKNOWN is
//! not FALSE. A comparison that is missing, or about another candidate revision, leaves the claim
//! UNKNOWN and `refuted` blocked; so does a comparison an invalidation rule takes away. A decision
//! taken at another case revision earns nothing.
//!
//! Every fixture is under `fixtures/ml-experiment/`, gives case revision `c1` and starts from the
//! artifact revisions `experiment_design` x1, `dataset_snapshot` d1, `baseline` b1, `candidate` m1,
//! `run_configuration` k1 and `results` s1.

mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, Json};
use canon_engineering::registry;
use canon_engineering::vocabulary::{self, Marking};
use support::{Compiled, Fixture};

/// Every artifact `ml.experiment/1` declares, in identifier order.
const ARTIFACTS: [&str; 6] = [
    "baseline",
    "candidate",
    "dataset_snapshot",
    "experiment_design",
    "results",
    "run_configuration",
];

/// Every evidence kind it declares, in identifier order.
const EVIDENCE_KINDS: [&str; 5] = [
    "comparison",
    "design_review",
    "integrity_check",
    "reproduction",
    "seed_sweep",
];

/// Every claim it declares, in identifier order.
const CLAIMS: [&str; 5] = [
    "candidate.beats_baseline",
    "effect.robust",
    "experiment.valid",
    "hypothesis.supported",
    "result.reproducible",
];

/// Every outcome it declares, in identifier order.
const OUTCOMES: [&str; 5] = [
    "inconclusive",
    "invalid_experiment",
    "refuted",
    "superseded",
    "supported",
];

/// The outcomes that read evidence, of which at most one may be legitimate in any state.
const EXCLUSIVE: [&str; 3] = ["supported", "refuted", "invalid_experiment"];

/// Every fixture of this story, by id.
const FIXTURES: [&str; 7] = [
    "supported",
    "refuted",
    "dataset-moved",
    "baseline-moved",
    "invalid-experiment",
    "inconclusive",
    "superseded",
];

/// The case revision and the artifact revisions every fixture starts from.
const CASE_REVISION: &str = "c1";
const REVISIONS: [(&str, &str); 6] = [
    ("experiment_design", "x1"),
    ("dataset_snapshot", "d1"),
    ("baseline", "b1"),
    ("candidate", "m1"),
    ("run_configuration", "k1"),
    ("results", "s1"),
];

/// The evaluation instant every fixture of this story gives; nothing in them expires.
const AT: &str = "2026-10-12T12:00:00Z";

/// The fixture `fixtures/ml-experiment/<id>.fixture.yaml`, loaded and checked: it evaluates
/// `ml.experiment/1`, starts from [`REVISIONS`] at [`CASE_REVISION`], and every state is as
/// expected.
fn fixture(id: &str, compiled: &Compiled) -> Fixture {
    let path = format!("fixtures/ml-experiment/{id}.fixture.yaml");
    let fixture = Fixture::load(&path).unwrap_or_else(|error| panic!("{path} loads: {error}"));
    assert_eq!(fixture.at(), AT, "{path}: the evaluation instant");
    assert_eq!(
        fixture.protocol_path(),
        support::protocol_path("ml-experiment", 1),
        "{path}"
    );
    let own = fixture
        .compile()
        .unwrap_or_else(|error| panic!("{path}: its protocol compiles: {error}"));
    assert_eq!(
        own.canon_ir(),
        compiled.canon_ir(),
        "{path} evaluates ml.experiment/1"
    );
    let case = fixture.case();
    assert_eq!(
        case.revision.as_ref().map(|revision| revision.as_str()),
        Some(CASE_REVISION),
        "{path}: the case revision"
    );
    let revisions: BTreeMap<&str, &str> = case
        .artifacts
        .iter()
        .map(|(artifact, entry)| (artifact.as_str(), entry.revision.as_str()))
        .collect();
    assert_eq!(
        revisions,
        BTreeMap::from(REVISIONS),
        "{path}: the artifact revisions it starts from"
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

/// The value of every claim in `decision`, keyed by claim id.
fn claims(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .claims
        .iter()
        .map(|(id, entry)| (id.as_str().to_owned(), entry.value.to_string()))
        .collect()
}

/// The status of every outcome in `decision`, keyed by outcome id.
fn statuses(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .outcomes
        .as_ref()
        .and_then(Json::as_object)
        .expect("the decision has an outcomes section")
        .iter()
        .map(|(id, entry)| {
            let status = entry["status"]
                .as_str()
                .unwrap_or_else(|| panic!("outcome `{id}` has no status: {entry}"));
            (id.clone(), status.to_owned())
        })
        .collect()
}

/// Every declared id of `ids`, each with `value`, except those `others` gives.
fn every<const N: usize>(
    ids: [&str; N],
    value: &str,
    others: &[(&str, &str)],
) -> BTreeMap<String, String> {
    ids.iter()
        .map(|id| {
            let value = others
                .iter()
                .find(|(other, _)| other == id)
                .map_or(value, |(_, other)| other);
            ((*id).to_owned(), value.to_owned())
        })
        .collect()
}

/// The ids of a compiled section, in identifier order.
fn ids<'a, K: 'a, V: 'a>(
    section: &'a BTreeMap<K, V>,
    name: impl Fn(&'a K) -> &'a str,
) -> Vec<&'a str> {
    section.keys().map(name).collect()
}

/// Asserts `term` names exactly one vocabulary entry, in `category`, marked `core`.
fn one_core_term(item: &str, term: &str, category: &str) {
    let found = vocabulary::lookup(term).unwrap_or_else(|error| panic!("{item} {error}"));
    let entries = vocabulary::terms()
        .iter()
        .filter(|entry| entry.id == term)
        .count();
    assert_eq!(entries, 1, "{item} `{term}` is declared {entries} times");
    assert_eq!(found.category.as_str(), category, "{item} `{term}`");
    assert_eq!(found.marking, Marking::Core, "{item} `{term}` is core");
}

#[test]
fn ml_experiment_outcomes_are_earned() {
    // 1. The protocol file validates and compiles in Canon, and the registry lists it beside the
    //    other built-ins.
    let path = support::protocol_path("ml-experiment", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("1. {path} validates and compiles: {error}"));
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "ml.experiment", "1. protocol id");
    assert_eq!(ir.protocol.revision, 1, "1. protocol revision");
    assert_eq!(
        ids(&ir.artifacts, |id| id.as_str()),
        ARTIFACTS,
        "1. artifacts"
    );
    assert_eq!(
        ids(&ir.evidence_kinds, |id| id.as_str()),
        EVIDENCE_KINDS,
        "1. evidence kinds"
    );
    assert_eq!(ids(&ir.claims, |id| id.as_str()), CLAIMS, "1. claims");
    assert_eq!(ids(&ir.outcomes, |id| id.as_str()), OUTCOMES, "1. outcomes");
    let decisions: BTreeMap<&str, &str> = ir
        .outcomes
        .iter()
        .filter_map(|(id, outcome)| {
            outcome
                .requires
                .decision()
                .map(|decision| (id.as_str(), decision.as_str()))
        })
        .collect();
    assert_eq!(
        decisions,
        BTreeMap::from([
            ("inconclusive", "explicitly_inconclusive"),
            ("superseded", "explicitly_superseded"),
        ]),
        "1. the outcomes earned on an explicit decision"
    );
    let rules: BTreeMap<&str, Vec<&str>> = ir
        .invalidation
        .values()
        .map(|rule| {
            let claims = rule
                .invalidates
                .iter()
                .map(|claim| claim.as_str())
                .collect();
            (rule.upstream.as_str(), claims)
        })
        .collect();
    assert_eq!(
        rules,
        BTreeMap::from([
            ("baseline", vec!["candidate.beats_baseline"]),
            (
                "dataset_snapshot",
                vec![
                    "candidate.beats_baseline",
                    "effect.robust",
                    "result.reproducible"
                ]
            ),
        ]),
        "1. the invalidation rules, by upstream artifact"
    );
    let listed = registry::list();
    for builtin in [
        ("incident-response", 1),
        ("ml-experiment", 1),
        ("software-change", 1),
        ("support-triage", 1),
    ] {
        assert!(
            listed.contains(&builtin),
            "1. the registry lists {builtin:?}: {listed:?}"
        );
    }
    let builtin = registry::get("ml-experiment", 1)
        .unwrap_or_else(|error| panic!("1. registry::get(ml-experiment, 1): {error}"));
    let on_disk = std::fs::read_to_string(support::repo_root().join(&path))
        .unwrap_or_else(|error| panic!("1. {path}: {error}"));
    assert_eq!(
        builtin.yaml, on_disk,
        "1. the built-in is the file as released"
    );

    // 2. Every artifact, evidence kind, claim and outcome id, and the decision
    //    `explicitly_inconclusive`, resolves to exactly one core vocabulary entry; `superseded` and
    //    `explicitly_superseded`, shared with software.change/1, still resolve to one each.
    for id in ARTIFACTS {
        one_core_term("2.", id, "artifact_kind");
    }
    for id in EVIDENCE_KINDS {
        one_core_term("2.", id, "evidence_kind");
    }
    for id in CLAIMS {
        one_core_term("2.", id, "claim_id");
    }
    for id in OUTCOMES {
        one_core_term("2.", id, "outcome_id");
    }
    for decision in decisions.values() {
        one_core_term("2.", decision, "decision_name");
    }

    // 3. No evidence and no decision: every claim unknown, every outcome blocked.
    let supported = fixture("supported", &compiled);
    let initial = decide(&supported, &compiled, "initial");
    assert_eq!(claims(&initial), every(CLAIMS, "unknown", &[]), "3. claims");
    assert_eq!(
        statuses(&initial),
        every(OUTCOMES, "blocked", &[]),
        "3. outcomes"
    );

    // 4. The full positive evidence: every claim true, `supported` alone legitimate.
    let earned = decide(&supported, &compiled, "supported");
    assert_eq!(claims(&earned), every(CLAIMS, "true", &[]), "4. claims");
    assert_eq!(
        statuses(&earned),
        every(OUTCOMES, "blocked", &[("supported", "legitimate")]),
        "4. outcomes"
    );

    // 5. The same with a comparison that finds the baseline better: the comparison is false and
    //    `refuted` is legitimate.
    let refuted = fixture("refuted", &compiled);
    let against = decide(&refuted, &compiled, "refuted");
    assert_eq!(
        claims(&against)["candidate.beats_baseline"],
        "false",
        "5. candidate.beats_baseline"
    );
    assert_eq!(
        statuses(&against),
        every(OUTCOMES, "blocked", &[("refuted", "legitimate")]),
        "5. outcomes"
    );

    // 6. With the candidate moved to m2 the comparison about m1 no longer counts: UNKNOWN, not
    //    FALSE, so nothing is refuted.
    let moved = decide(&refuted, &compiled, "candidate-moved");
    assert_eq!(
        claims(&moved)["candidate.beats_baseline"],
        "unknown",
        "6. a comparison about another candidate revision is not a refutation"
    );
    assert_eq!(
        statuses(&moved),
        every(OUTCOMES, "blocked", &[]),
        "6. outcomes"
    );

    // 7. Evidence observed against dataset snapshot d1, with the snapshot moved to d2: the result
    //    claims and the hypothesis are unknown, validity stays true, `supported` is blocked.
    let dataset = fixture("dataset-moved", &compiled);
    assert_eq!(
        statuses(&decide(&dataset, &compiled, "current")),
        every(OUTCOMES, "blocked", &[("supported", "legitimate")]),
        "7. before the snapshot moves"
    );
    for record in dataset.evidence("current") {
        let upstream: Vec<(&str, &str)> = record
            .upstream_revisions
            .iter()
            .map(|(artifact, revision)| (artifact.as_str(), revision.as_str()))
            .collect();
        assert_eq!(
            upstream,
            [("dataset_snapshot", "d1")],
            "7. `{}` is observed against d1",
            record.id.as_str()
        );
    }
    let stale = decide(&dataset, &compiled, "dataset-moved");
    assert_eq!(
        claims(&stale),
        every(CLAIMS, "unknown", &[("experiment.valid", "true")]),
        "7. claims"
    );
    assert_eq!(
        statuses(&stale),
        every(OUTCOMES, "blocked", &[]),
        "7. outcomes"
    );

    // 8. The comparison made against baseline b1, with the baseline moved to b2: the comparison and
    //    the hypothesis are unknown, the rest stay true, `supported` is blocked.
    let baseline = fixture("baseline-moved", &compiled);
    assert_eq!(
        statuses(&decide(&baseline, &compiled, "current")),
        every(OUTCOMES, "blocked", &[("supported", "legitimate")]),
        "8. before the baseline moves"
    );
    let stale = decide(&baseline, &compiled, "baseline-moved");
    assert_eq!(
        claims(&stale),
        every(
            CLAIMS,
            "true",
            &[
                ("candidate.beats_baseline", "unknown"),
                ("hypothesis.supported", "unknown"),
            ]
        ),
        "8. claims"
    );
    assert_eq!(
        statuses(&stale),
        every(OUTCOMES, "blocked", &[]),
        "8. outcomes"
    );

    // 9. An integrity violation: the experiment is not valid, `invalid_experiment` is legitimate,
    //    and a comparison that finds the baseline better refutes nothing.
    let invalid = fixture("invalid-experiment", &compiled);
    let violated = decide(&invalid, &compiled, "invalid");
    assert_eq!(
        claims(&violated)["experiment.valid"],
        "false",
        "9. experiment.valid"
    );
    assert_eq!(
        statuses(&violated),
        every(OUTCOMES, "blocked", &[("invalid_experiment", "legitimate")]),
        "9. outcomes"
    );

    // 10. An `explicitly_inconclusive` decision at c0 earns nothing at c1; at c1 it earns
    //     `inconclusive`.
    let inconclusive = fixture("inconclusive", &compiled);
    assert_eq!(
        statuses(&decide(&inconclusive, &compiled, "inconclusive-at-c0")),
        every(OUTCOMES, "blocked", &[]),
        "10. a decision on another case revision"
    );
    assert_eq!(
        statuses(&decide(&inconclusive, &compiled, "inconclusive")),
        every(OUTCOMES, "blocked", &[("inconclusive", "legitimate")]),
        "10. a decision at c1"
    );

    // 11. An `explicitly_superseded` decision at c1 earns `superseded`.
    let superseded = fixture("superseded", &compiled);
    assert_eq!(
        statuses(&decide(&superseded, &compiled, "superseded")),
        every(OUTCOMES, "blocked", &[("superseded", "legitimate")]),
        "11. superseded"
    );

    // 12. In no state of any fixture are two of `supported`, `refuted` and `invalid_experiment`
    //     legitimate together.
    let mut states = 0;
    for id in FIXTURES {
        let loaded = fixture(id, &compiled);
        for state in loaded.states() {
            states += 1;
            let found = statuses(&decide(&loaded, &compiled, state));
            let legitimate: Vec<&str> = EXCLUSIVE
                .into_iter()
                .filter(|outcome| found[*outcome] == "legitimate")
                .collect();
            assert!(
                legitimate.len() <= 1,
                "12. {id} state `{state}` has {legitimate:?} legitimate together"
            );
        }
    }
    assert_eq!(states, 14, "12. every state of every fixture is checked");
}
