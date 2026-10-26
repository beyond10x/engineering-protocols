//! Acceptance 4 of `story:takt-protocols-builtin` for `loop.improvement/1`: its fixtures under
//! `fixtures/loop-improvement/` reach every outcome, and show its independence and stale-revision
//! cases as UNKNOWN, not FALSE: a comparison by the run that delivered the loop candidate is
//! excluded as `not_independent`, and a commit to the candidate excludes its definition check as
//! `revision_mismatch`.
//!
//! Every fixture gives case revision `c1`, starts from loop baseline lb1, loop candidate lc1 and
//! loop finding lf1, and lists run-b alone as independent on `different_agent_run`; run-a, the
//! run that delivered the candidate, is listed on no dimension.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each takt protocol's test uses part of it
mod builtin_fixtures;

use std::collections::BTreeMap;

use b10x_canon::model::ExclusionReason;
use builtin_fixtures::{actions, claims, decide, excluded_only_as, fixture, outcomes, reached};

const NAME: &str = "loop-improvement";
const AT: &str = "2026-10-26T12:00:00Z";

/// Every claim it declares, in identifier order.
const CLAIMS: [&str; 4] = [
    "loop.change_merged",
    "loop.definition_honest",
    "loop.metric_below_target",
    "loop.metric_improved",
];

/// Every outcome it declares, in identifier order.
const OUTCOMES: [&str; 3] = ["abandoned", "accepted", "declined"];

/// Every fixture of this protocol, sorted.
const FIXTURES: [&str; 6] = [
    "abandoned",
    "accepted",
    "at-target",
    "declined",
    "definition-moved",
    "not-independent",
];

#[test]
fn loop_improvement_outcomes_are_earned() {
    // 1. The built-in compiles, and declares what this test reads; every loop change is a
    //    judge-surface change, so its one merge needs the owner's capability.
    let compiled = builtin_fixtures::compile(NAME);
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "loop.improvement");
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
    assert_eq!(
        capabilities,
        [("loop.merge", "repository.merge.judge_surface")]
    );

    // 2. Every fixture under fixtures/loop-improvement/ is one of this test's, and each is as it
    //    expects in every state.
    assert_eq!(builtin_fixtures::fixture_ids(NAME), FIXTURES);
    let loaded: Vec<_> = FIXTURES
        .iter()
        .map(|id| fixture(NAME, id, AT, &compiled))
        .collect();
    let by_id: BTreeMap<&str, _> = FIXTURES.iter().copied().zip(loaded.iter()).collect();

    // 3. The fixtures reach every outcome.
    let (legitimate, states) = reached(&loaded, &compiled);
    assert_eq!(legitimate.into_iter().collect::<Vec<_>>(), OUTCOMES);
    assert_eq!(states, 14, "every state of every fixture is evaluated");

    // 4. The comparison by run-a, which delivered the candidate, is excluded as not independent:
    //    `loop.metric_improved` is unknown, not false, and the turn is neither accepted nor
    //    declined.
    let own = decide(by_id["not-independent"], &compiled, "own-comparison");
    assert_eq!(claims(&own)["loop.metric_improved"], "unknown");
    assert!(excluded_only_as(
        &own,
        "loop.metric_improved",
        ExclusionReason::NotIndependent
    ));
    assert!(
        outcomes(&own).values().all(|status| status == "blocked"),
        "4. {:?}",
        outcomes(&own)
    );

    // 5. A commit to the candidate: its definition check about lc1 is excluded as
    //    `revision_mismatch`, `loop.definition_honest` is unknown, not false, and the merge is
    //    blocked; a check at lc2 that finds a silent redefinition makes it false.
    let moved = by_id["definition-moved"];
    let stale = decide(moved, &compiled, "candidate-moved");
    assert_eq!(claims(&stale)["loop.definition_honest"], "unknown");
    assert!(excluded_only_as(
        &stale,
        "loop.definition_honest",
        ExclusionReason::RevisionMismatch
    ));
    assert_eq!(actions(&stale)["loop.merge"], "blocked");
    let redefined = decide(moved, &compiled, "redefined");
    assert_eq!(claims(&redefined)["loop.definition_honest"], "false");
}
