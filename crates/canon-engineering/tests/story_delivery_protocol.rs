//! Acceptance 4 of `story:takt-protocols-builtin` for `story.delivery/1`: its fixtures under
//! `fixtures/story-delivery/` reach every outcome, and a new commit returns every claim to
//! UNKNOWN, not FALSE (the protocol's stale-revision case; it declares no independence).
//!
//! Every fixture gives case revision `c1` and starts from implementation revision h1.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each takt protocol's test uses part of it
mod builtin_fixtures;

use std::collections::BTreeMap;

use b10x_canon::model::ExclusionReason;
use builtin_fixtures::{actions, claims, decide, excluded_only_as, fixture, outcomes, reached};

const NAME: &str = "story-delivery";
const AT: &str = "2026-10-26T12:00:00Z";

/// Every claim it declares, in identifier order.
const CLAIMS: [&str; 7] = [
    "change.handed_over",
    "change.made",
    "ci.green",
    "gate.green",
    "pr.merged",
    "pr.opened",
    "tests.pass",
];

/// Every action it declares, in identifier order.
const ACTIONS: [&str; 10] = [
    "change.handover",
    "change.read",
    "ci.read",
    "gate.run",
    "implementation.edit",
    "pr.open",
    "repository.inspect",
    "repository.merge",
    "tests.run",
    "tests.write",
];

/// Every outcome it declares, in identifier order.
const OUTCOMES: [&str; 2] = ["abandoned", "accepted"];

/// Every fixture of this protocol, sorted.
const FIXTURES: [&str; 7] = [
    "abandoned",
    "accepted",
    "checks-failed",
    "merge-denied",
    "new-commit",
    "nothing-to-publish",
    "precondition-gaps",
];

#[test]
fn story_delivery_outcomes_are_earned() {
    // 1. The built-in compiles, and declares what this test reads.
    let compiled = builtin_fixtures::compile(NAME);
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "story.delivery");
    let claim_ids: Vec<&str> = ir.claims.keys().map(|id| id.as_str()).collect();
    assert_eq!(claim_ids, CLAIMS);
    let action_ids: Vec<&str> = ir.actions.keys().map(|id| id.as_str()).collect();
    assert_eq!(action_ids, ACTIONS);
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
    assert_eq!(capabilities, [("repository.merge", "repository.merge")]);

    // 2. Every fixture under fixtures/story-delivery/ is one of this test's, and each is as it
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
    assert_eq!(states, 21, "every state of every fixture is evaluated");

    // 4. `accepted` is earned only once the pull request for the current revision is merged, and
    //    the merge is admissible only with authority granted.
    let accepted = by_id["accepted"];
    let ci_green = decide(accepted, &compiled, "ci-green");
    assert_eq!(actions(&ci_green)["repository.merge"], "approval-required");
    assert_eq!(outcomes(&ci_green)["accepted"], "blocked");
    let merged = decide(accepted, &compiled, "merged");
    assert_eq!(outcomes(&merged)["accepted"], "legitimate");
    let denied = decide(by_id["merge-denied"], &compiled, "denied");
    assert_eq!(actions(&denied)["repository.merge"], "blocked");

    // 5. A new commit: every record about h1 is excluded as `revision_mismatch`, every claim is
    //    unknown, not false, and neither the pull request nor the merge is admissible.
    let moved = by_id["new-commit"];
    let stale = decide(moved, &compiled, "new-commit");
    for claim in CLAIMS {
        assert_eq!(
            claims(&stale)[claim],
            "unknown",
            "5. `{claim}` after a new commit"
        );
        if claim != "pr.merged" {
            assert!(
                excluded_only_as(&stale, claim, ExclusionReason::RevisionMismatch),
                "5. `{claim}` excludes the records about h1 as revision_mismatch"
            );
        }
    }
    for action in ["pr.open", "ci.read", "repository.merge"] {
        assert_eq!(actions(&stale)[action], "blocked", "5. `{action}`");
    }
    let old_merge = decide(moved, &compiled, "merged-old-revision");
    assert_eq!(claims(&old_merge)["pr.merged"], "unknown");
    assert!(excluded_only_as(
        &old_merge,
        "pr.merged",
        ExclusionReason::RevisionMismatch
    ));
    assert_eq!(outcomes(&old_merge)["accepted"], "blocked");
    // A result about the current revision decides the claim: failing tests at h2 are false.
    let failing = decide(moved, &compiled, "tests-fail");
    assert_eq!(claims(&failing)["tests.pass"], "false");
}
