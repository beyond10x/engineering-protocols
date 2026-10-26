//! Acceptance 4 of `story:takt-protocols-builtin` for `self.improvement/1`: its fixtures under
//! `fixtures/self-improvement/` reach every outcome, and show its independence and stale-revision
//! cases as UNKNOWN, not FALSE: a verification by the run that delivered the candidate is excluded
//! as `not_independent`, and a commit to the candidate excludes every record about the earlier
//! revision as `revision_mismatch` while the baseline's claims stand.
//!
//! Every fixture gives case revision `c1`, starts from finding f1, baseline b1, candidate h1 and
//! promotion p1, and lists run-b alone as independent on `different_agent_run`; run-a, the run
//! that delivered the candidate, is listed on no dimension.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each takt protocol's test uses part of it
mod builtin_fixtures;

use std::collections::BTreeMap;

use b10x_canon::model::ExclusionReason;
use builtin_fixtures::{actions, claims, decide, excluded_only_as, fixture, outcomes, reached};

const NAME: &str = "self-improvement";
const AT: &str = "2026-10-26T12:00:00Z";

/// Every claim it declares, in identifier order.
const CLAIMS: [&str; 6] = [
    "finding.cited",
    "fix.merged",
    "promoted",
    "scenario.passes_after",
    "scenario.reproduced_before",
    "scorecard.no_regression",
];

/// The claims about the candidate, which a commit to it takes away.
const CANDIDATE_CLAIMS: [&str; 3] = [
    "fix.merged",
    "scenario.passes_after",
    "scorecard.no_regression",
];

/// Every outcome it declares, in identifier order.
const OUTCOMES: [&str; 4] = ["abandoned", "accepted", "declined", "unreproducible"];

/// Every capability an action requires, by action.
const CAPABILITIES: [(&str, &str); 3] = [
    ("binary.promote", "takt.promote"),
    ("candidate.merge", "repository.merge"),
    (
        "candidate.merge_judge_surface",
        "repository.merge.judge_surface",
    ),
];

/// Every fixture of this protocol, sorted.
const FIXTURES: [&str; 14] = [
    "abandoned",
    "accepted",
    "candidate-moved",
    "declined",
    "judge-surface",
    "judge-surface-unverified",
    "malformed-finding",
    "not-independent",
    "promoted-unverified",
    "promoted-with-regression",
    "regression",
    "scenario-error",
    "scored-unmerged",
    "unreproducible",
];

#[test]
fn self_improvement_outcomes_are_earned() {
    // 1. The built-in compiles, and declares what this test reads.
    let compiled = builtin_fixtures::compile(NAME);
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "self.improvement");
    let claim_ids: Vec<&str> = ir.claims.keys().map(|id| id.as_str()).collect();
    assert_eq!(claim_ids, CLAIMS);
    let outcome_ids: Vec<&str> = ir.outcomes.keys().map(|id| id.as_str()).collect();
    assert_eq!(outcome_ids, OUTCOMES);
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
    assert_eq!(capabilities, CAPABILITIES);

    // 2. Every fixture under fixtures/self-improvement/ is one of this test's, and each is as it
    //    expects in every state.
    assert_eq!(builtin_fixtures::fixture_ids(NAME), FIXTURES);
    let loaded: Vec<_> = FIXTURES
        .iter()
        .map(|id| fixture(NAME, id, AT, &compiled))
        .collect();
    let by_id: BTreeMap<&str, _> = FIXTURES.iter().copied().zip(loaded.iter()).collect();

    // 3. The fixtures reach every outcome, and no state has two legitimate at once.
    let (legitimate, states) = reached(&loaded, &compiled);
    assert_eq!(legitimate.into_iter().collect::<Vec<_>>(), OUTCOMES);
    assert_eq!(states, 28, "every state of every fixture is evaluated");
    for loaded in &loaded {
        for state in loaded.states() {
            let found = outcomes(&decide(loaded, &compiled, state));
            let together = found
                .values()
                .filter(|status| *status == "legitimate")
                .count();
            assert!(
                together <= 1,
                "3. {} state `{state}`: {found:?}",
                loaded.id()
            );
        }
    }

    // 4. The verification by run-a, which delivered the candidate, is excluded as not
    //    independent: `scenario.passes_after` is unknown, not false, and no merge is admissible.
    let own = decide(by_id["not-independent"], &compiled, "own-verification");
    assert_eq!(claims(&own)["scenario.passes_after"], "unknown");
    assert!(excluded_only_as(
        &own,
        "scenario.passes_after",
        ExclusionReason::NotIndependent
    ));
    for action in ["candidate.merge", "candidate.merge_judge_surface"] {
        assert_eq!(actions(&own)[action], "blocked", "4. `{action}`");
    }

    // 5. A commit to the candidate: every claim about it is unknown, not false, its records
    //    excluded as `revision_mismatch`; the finding's and the baseline's claims stand.
    let moved = decide(by_id["candidate-moved"], &compiled, "candidate-moved");
    let values = claims(&moved);
    for claim in CANDIDATE_CLAIMS {
        assert_eq!(values[claim], "unknown", "5. `{claim}`");
        assert!(
            excluded_only_as(&moved, claim, ExclusionReason::RevisionMismatch),
            "5. `{claim}` excludes the records about h1 as revision_mismatch"
        );
    }
    assert_eq!(values["finding.cited"], "true");
    assert_eq!(values["scenario.reproduced_before"], "true");
    assert_eq!(actions(&moved)["binary.promote"], "blocked");

    // 6. The case input decides which merge applies: on a judge surface only the owner's.
    let judge = decide(by_id["judge-surface"], &compiled, "judge-granted");
    assert_eq!(actions(&judge)["candidate.merge"], "blocked");
    assert_eq!(
        actions(&judge)["candidate.merge_judge_surface"],
        "admissible"
    );
}
