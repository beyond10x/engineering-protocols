//! Acceptance for `story:eval-gated-change-protocol`: `eval.gated_change/1` ships as Canon
//! `protocol/1` data, a fifth built-in, in the shape
//! `architecture-decision-record:eval-gated-change-shape` records, and its outcomes are earned
//! through the fixture harness and Canon.
//!
//! The pressure test is the loop's own say-so: a change is kept only on evidence about the current
//! candidate revision from a producer the case lists as another agent run, and UNKNOWN neither
//! keeps nor rejects. Evidence by the run that wrote the candidate is excluded as not independent,
//! and evidence about an earlier candidate revision (a moved basis) as `revision_mismatch`; both
//! leave the claims UNKNOWN, not FALSE.
//!
//! Every fixture is under `fixtures/eval-gated-change/`, gives case revision `c1`, the artifact
//! revisions `eval_design` h1 and `candidate` h1, and lists `run-b` as independent on
//! `different_agent_run`; `run-a`, the run that wrote the candidate, is not listed.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, ExclusionReason, Json};
use canon_engineering::registry;
use canon_engineering::vocabulary::{self, Marking};
use support::{Compiled, Fixture};

/// Every artifact `eval.gated_change/1` declares, in identifier order.
const ARTIFACTS: [&str; 2] = ["candidate", "eval_design"];

/// Every evidence kind it declares, in identifier order.
const EVIDENCE_KINDS: [&str; 4] = [
    "design_review",
    "guard_check",
    "heldout_comparison",
    "tuning_comparison",
];

/// Every claim it declares, in identifier order.
const CLAIMS: [&str; 7] = [
    "change.supported",
    "change.within_surface",
    "content.clean",
    "experiment.valid",
    "heldout.not_regressed",
    "result.reproducible",
    "tuning.improved",
];

/// The keep claims: every claim `change.supported` needs, in identifier order.
const KEEP_CLAIMS: [&str; 6] = [
    "change.within_surface",
    "content.clean",
    "experiment.valid",
    "heldout.not_regressed",
    "result.reproducible",
    "tuning.improved",
];

/// The keep claims about the candidate, which a moved basis takes away.
const CANDIDATE_CLAIMS: [&str; 5] = [
    "change.within_surface",
    "content.clean",
    "heldout.not_regressed",
    "result.reproducible",
    "tuning.improved",
];

/// Every action it declares, in identifier order.
const ACTIONS: [&str; 6] = [
    "candidate.evaluate",
    "change.apply_outside_surface",
    "change.keep",
    "change.propose",
    "change.reject",
    "design.review",
];

/// Every capability an action requires, by action.
const CAPABILITIES: [(&str, &str); 3] = [
    ("change.apply_outside_surface", "loop_change.apply_outside_surface"),
    ("change.keep", "loop_change.keep"),
    ("design.review", "eval_design.review"),
];

/// Every outcome it declares, in identifier order.
const OUTCOMES: [&str; 4] = ["inconclusive", "kept", "proposed", "rejected"];

/// The outcomes that read evidence, of which at most one may be legitimate in any state.
const EXCLUSIVE: [&str; 3] = ["kept", "rejected", "proposed"];

/// The terms reused from `ml.experiment/1` and `software.change/1`, each with its category.
const REUSED: [(&str, &str); 7] = [
    ("candidate", "artifact_kind"),
    ("design_review", "evidence_kind"),
    ("experiment.valid", "claim_id"),
    ("result.reproducible", "claim_id"),
    ("inconclusive", "outcome_id"),
    ("explicitly_inconclusive", "decision_name"),
    ("different_agent_run", "independence_dimension"),
];

/// Every fixture of this story, by id.
const FIXTURES: [&str; 8] = [
    "kept",
    "not-independent",
    "basis-moved",
    "rejected-heldout",
    "rejected-leaked",
    "rejected-not-better",
    "proposed",
    "inconclusive",
];

/// The case revision, the artifact revisions and the independence every fixture starts from.
const CASE_REVISION: &str = "c1";
const REVISIONS: [(&str, &str); 2] = [("candidate", "h1"), ("eval_design", "h1")];
const INDEPENDENT: &str = "run-b";
const DIMENSION: &str = "different_agent_run";

/// The evaluation instant every fixture of this story gives; nothing in them expires.
const AT: &str = "2026-10-17T12:00:00Z";

/// The fixture `fixtures/eval-gated-change/<id>.fixture.yaml`, loaded and checked: it evaluates
/// `eval.gated_change/1`, starts from [`REVISIONS`] at [`CASE_REVISION`], lists [`INDEPENDENT`]
/// alone as independent on [`DIMENSION`], and every state is as expected.
fn fixture(id: &str, compiled: &Compiled) -> Fixture {
    let path = format!("fixtures/eval-gated-change/{id}.fixture.yaml");
    let fixture = Fixture::load(&path).unwrap_or_else(|error| panic!("{path} loads: {error}"));
    assert_eq!(fixture.at(), AT, "{path}: the evaluation instant");
    assert_eq!(
        fixture.protocol_path(),
        support::protocol_path("eval-gated-change", 1),
        "{path}"
    );
    let own = fixture
        .compile()
        .unwrap_or_else(|error| panic!("{path}: its protocol compiles: {error}"));
    assert_eq!(
        own.canon_ir(),
        compiled.canon_ir(),
        "{path} evaluates eval.gated_change/1"
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
    let independence: BTreeMap<&str, Vec<&str>> = case
        .independence
        .iter()
        .map(|(producer, dimensions)| {
            (
                producer.as_str(),
                dimensions.iter().map(|dimension| dimension.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(
        independence,
        BTreeMap::from([(INDEPENDENT, vec![DIMENSION])]),
        "{path}: the independence its case gives"
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

/// The reasons `decision` gives for each record it excludes from claim `claim`.
fn exclusions(decision: &Decision, claim: &str) -> Vec<ExclusionReason> {
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

/// The status of every entry of a decision section, keyed by id; the section is an object of
/// entries or an array of entries that carry their `id`.
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
fn actions(decision: &Decision) -> BTreeMap<String, String> {
    section(decision.actions.as_ref(), "actions")
}

/// The status of every outcome in `decision`, keyed by outcome id.
fn statuses(decision: &Decision) -> BTreeMap<String, String> {
    section(decision.outcomes.as_ref(), "outcomes")
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

/// Asserts `change.keep` and `change.reject` are both not admissible in `decision`.
fn neither_keep_nor_reject(item: &str, decision: &Decision) {
    let found = actions(decision);
    for action in ["change.keep", "change.reject"] {
        assert_ne!(
            found[action], "admissible",
            "{item} `{action}` is not admissible"
        );
    }
}

#[test]
fn eval_gated_change_outcomes_are_earned() {
    // 1. The protocol file validates and compiles in Canon, and the registry lists it beside the
    //    other built-ins.
    let path = support::protocol_path("eval-gated-change", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("1. {path} validates and compiles: {error}"));
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "eval.gated_change", "1. protocol id");
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
    assert_eq!(ids(&ir.actions, |id| id.as_str()), ACTIONS, "1. actions");
    assert_eq!(ids(&ir.outcomes, |id| id.as_str()), OUTCOMES, "1. outcomes");
    let capabilities: Vec<(&str, &str)> = ir
        .actions
        .iter()
        .flat_map(|(id, action)| {
            action
                .requires
                .iter()
                .map(move |capability| (id.as_str(), capability.as_str()))
        })
        .collect();
    assert_eq!(capabilities, CAPABILITIES, "1. capabilities, by action");
    assert!(ir.invalidation.is_empty(), "1. /1 declares no invalidation rule");
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
        BTreeMap::from([("inconclusive", "explicitly_inconclusive")]),
        "1. the outcomes earned on an explicit decision"
    );
    let listed = registry::list();
    for builtin in [
        ("eval-gated-change", 1),
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
    let builtin = registry::get("eval-gated-change", 1)
        .unwrap_or_else(|error| panic!("1. registry::get(eval-gated-change, 1): {error}"));
    let on_disk = std::fs::read_to_string(support::repo_root().join(&path))
        .unwrap_or_else(|error| panic!("1. {path}: {error}"));
    assert_eq!(
        builtin.yaml, on_disk,
        "1. the built-in is the file as released"
    );

    // 2. Every artifact, evidence kind, claim, action and outcome id resolves to exactly one core
    //    vocabulary entry, and the reused terms still resolve to one each. Capabilities are not
    //    vocabulary terms: `vocabulary/1` has no capability category.
    for id in ARTIFACTS {
        one_core_term("2.", id, "artifact_kind");
    }
    for id in EVIDENCE_KINDS {
        one_core_term("2.", id, "evidence_kind");
    }
    for id in CLAIMS {
        one_core_term("2.", id, "claim_id");
    }
    for id in ACTIONS {
        one_core_term("2.", id, "action_id");
    }
    for id in OUTCOMES {
        one_core_term("2.", id, "outcome_id");
    }
    for (id, category) in REUSED {
        one_core_term("2. reused", id, category);
    }

    // 4. No evidence and no decision: every claim unknown, neither keep nor reject admissible,
    //    every outcome blocked.
    let kept = fixture("kept", &compiled);
    let initial = decide(&kept, &compiled, "initial");
    assert_eq!(claims(&initial), every(CLAIMS, "unknown", &[]), "4. claims");
    neither_keep_nor_reject("4.", &initial);
    assert_eq!(
        statuses(&initial),
        every(OUTCOMES, "blocked", &[]),
        "4. outcomes"
    );

    // 5. Every keep claim's evidence passing, by run-b, about h1, with `loop_change.keep` granted:
    //    `change.supported` true, keep admissible, reject not, `kept` legitimate.
    let granted = decide(&kept, &compiled, "keep-granted");
    assert_eq!(claims(&granted), every(CLAIMS, "true", &[]), "5. claims");
    let found = actions(&granted);
    assert_eq!(found["change.keep"], "admissible", "5. change.keep");
    assert_ne!(found["change.reject"], "admissible", "5. change.reject");
    assert_eq!(
        statuses(&granted),
        every(OUTCOMES, "blocked", &[("kept", "legitimate")]),
        "5. outcomes"
    );

    // 6. The same evidence without `loop_change.keep` granted: keep is not admissible.
    let ungranted = decide(&kept, &compiled, "evidence-passing");
    assert_eq!(
        claims(&ungranted)["change.supported"],
        "true",
        "6. change.supported"
    );
    assert_ne!(
        actions(&ungranted)["change.keep"],
        "admissible",
        "6. change.keep without authority"
    );

    // 7. The same evidence by run-a, the run that wrote the candidate: excluded as not
    //    independent, the claims unknown, not false, neither keep nor reject admissible.
    let own = fixture("not-independent", &compiled);
    let said_so = decide(&own, &compiled, "own-evidence");
    assert_eq!(claims(&said_so), every(CLAIMS, "unknown", &[]), "7. claims");
    for claim in KEEP_CLAIMS {
        let reasons = exclusions(&said_so, claim);
        assert!(
            !reasons.is_empty()
                && reasons
                    .iter()
                    .all(|reason| *reason == ExclusionReason::NotIndependent),
            "7. `{claim}` excludes run-a's records as not independent: {reasons:?}"
        );
    }
    neither_keep_nor_reject("7.", &said_so);

    // 8. The same evidence with the candidate moved to h2: excluded as `revision_mismatch`, the
    //    claims on the candidate unknown, neither keep nor reject admissible.
    let moved = fixture("basis-moved", &compiled);
    assert_eq!(
        statuses(&decide(&moved, &compiled, "current")),
        every(OUTCOMES, "blocked", &[("kept", "legitimate")]),
        "8. before the basis moves"
    );
    let stale = decide(&moved, &compiled, "basis-moved");
    assert_eq!(
        claims(&stale),
        every(CLAIMS, "unknown", &[("experiment.valid", "true")]),
        "8. claims"
    );
    for claim in CANDIDATE_CLAIMS {
        let reasons = exclusions(&stale, claim);
        assert!(
            !reasons.is_empty()
                && reasons
                    .iter()
                    .all(|reason| *reason == ExclusionReason::RevisionMismatch),
            "8. `{claim}` excludes the records about h1 as revision_mismatch: {reasons:?}"
        );
    }
    neither_keep_nor_reject("8.", &stale);

    // 9. The evidence of 5 with one keep claim false: reject admissible, keep not, `rejected`
    //    legitimate; with the held-out split regressed, the guard check leaked and the tuning
    //    comparison not better.
    for (id, state, falsified) in [
        ("rejected-heldout", "evidence-complete", "heldout.not_regressed"),
        ("rejected-leaked", "leaked", "content.clean"),
        ("rejected-not-better", "not-better", "tuning.improved"),
    ] {
        let loaded = fixture(id, &compiled);
        let rejected = decide(&loaded, &compiled, state);
        let values = claims(&rejected);
        assert_eq!(values[falsified], "false", "9. {id}: `{falsified}`");
        assert_eq!(
            values["change.supported"], "false",
            "9. {id}: change.supported"
        );
        let found = actions(&rejected);
        assert_eq!(found["change.reject"], "admissible", "9. {id}: change.reject");
        assert_ne!(found["change.keep"], "admissible", "9. {id}: change.keep");
        assert_eq!(
            statuses(&rejected),
            every(OUTCOMES, "blocked", &[("rejected", "legitimate")]),
            "9. {id}: outcomes"
        );
    }

    // 10. No evidence but a held-out regression by run-b: reject is not admissible while the other
    //     keep claims are unknown.
    let heldout = fixture("rejected-heldout", &compiled);
    let partial = decide(&heldout, &compiled, "heldout-regressed");
    assert_eq!(
        claims(&partial),
        every(
            CLAIMS,
            "unknown",
            &[
                ("heldout.not_regressed", "false"),
                ("change.supported", "false"),
            ]
        ),
        "10. claims"
    );
    assert_ne!(
        actions(&partial)["change.reject"],
        "admissible",
        "10. change.reject while other keep claims are unknown"
    );

    // 11. A guard check outside the surface by run-b: `change.within_surface` false, propose
    //     admissible, keep not, `proposed` legitimate.
    let proposed = fixture("proposed", &compiled);
    let outside = decide(&proposed, &compiled, "outside");
    assert_eq!(
        claims(&outside)["change.within_surface"],
        "false",
        "11. change.within_surface"
    );
    let found = actions(&outside);
    assert_eq!(found["change.propose"], "admissible", "11. change.propose");
    assert_ne!(found["change.keep"], "admissible", "11. change.keep");
    assert_eq!(
        statuses(&outside),
        every(OUTCOMES, "blocked", &[("proposed", "legitimate")]),
        "11. outcomes"
    );

    // 12. An `explicitly_inconclusive` decision at c1 earns `inconclusive`; at c0 it earns
    //     nothing.
    let inconclusive = fixture("inconclusive", &compiled);
    assert_eq!(
        statuses(&decide(&inconclusive, &compiled, "inconclusive-at-c0")),
        every(OUTCOMES, "blocked", &[]),
        "12. a decision on another case revision"
    );
    assert_eq!(
        statuses(&decide(&inconclusive, &compiled, "inconclusive")),
        every(OUTCOMES, "blocked", &[("inconclusive", "legitimate")]),
        "12. a decision at c1"
    );

    // 11 and 13. In no state of any fixture is `change.apply_outside_surface` admissible, nor are
    // two of `kept`, `rejected` and `proposed` legitimate together.
    let mut states = 0;
    for id in FIXTURES {
        let loaded = fixture(id, &compiled);
        for state in loaded.states() {
            states += 1;
            let decision = decide(&loaded, &compiled, state);
            assert_ne!(
                actions(&decision)["change.apply_outside_surface"],
                "admissible",
                "11. {id} state `{state}`: change.apply_outside_surface"
            );
            let found = statuses(&decision);
            let legitimate: Vec<&str> = EXCLUSIVE
                .into_iter()
                .filter(|outcome| found[*outcome] == "legitimate")
                .collect();
            assert!(
                legitimate.len() <= 1,
                "13. {id} state `{state}` has {legitimate:?} legitimate together"
            );
        }
    }
    assert_eq!(states, 14, "13. every state of every fixture is checked");
}

#[test]
fn eval_gated_change_check_stays_under_the_bound() {
    // 3. `canon check` over the compiled /1 runs, finds nothing, and its state count is at most
    //    the bound.
    let path = support::protocol_path("eval-gated-change", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("3. {path} validates and compiles: {error}"));
    let report = b10x_canon::check::check(compiled.ir(), None)
        .unwrap_or_else(|refusal| panic!("3. canon check runs: {}: {refusal}", refusal.code()));
    println!(
        "eval.gated_change/1: canon check evaluated {} states (bound {})",
        report.states,
        b10x_canon::check::STATE_BOUND
    );
    assert!(report.is_clean(), "3. canon check found:\n{report}");
    assert!(
        report.states <= b10x_canon::check::STATE_BOUND,
        "3. {} states, more than the bound of {}",
        report.states,
        b10x_canon::check::STATE_BOUND
    );
}
