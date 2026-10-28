//! Acceptance of `story:story-delivery-2-with-settings`: `story.delivery/2` declares `tests_first`
//! and `gate_on` as case inputs, and the claim `change.complete` that `pr.open` needs. Its fixtures
//! under `fixtures/story-delivery-2/` hold the scenarios, one test per named scenario; the
//! `story.delivery/1` fixtures stay with `story_delivery_protocol.rs`, which evaluates them
//! unchanged (item 7).
//!
//! Every fixture gives case revision `c1`, starts from implementation revision h1 and evaluates
//! the built-in `story-delivery` at major 2.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each built-in protocol's test uses part of it
mod builtin_fixtures;

use b10x_canon::model::ExclusionReason;
use builtin_fixtures::{actions, claims, decide, excluded_only_as, fixture_of};
use support::{Compiled, Fixture};

const NAME: &str = "story-delivery";
const DIRECTORY: &str = "story-delivery-2";
const MAJOR: u32 = 2;
const AT: &str = "2026-10-27T12:00:00Z";

const FIXTURES: [&str; 12] = [
    "a-first-run-pass-is-not-red",
    "a-red-ci-admits-the-edit",
    "a-red-gate-admits-the-edit",
    "a-red-run-about-another-revision-does-not-admit-the-edit",
    "an-old-gate-pass-does-not-hold",
    "baseline-fail-then-red",
    "baseline-pass-then-red",
    "gate-after-handover",
    "gate-each-revision",
    "pr-open-needs-change-complete",
    "red-first-refuses-implementation-first",
    "tests-optional-keeps-the-old-order",
];

fn compiled() -> Compiled {
    builtin_fixtures::compile_major(NAME, MAJOR)
}

fn load(id: &str, compiled: &Compiled) -> Fixture {
    fixture_of(DIRECTORY, NAME, MAJOR, id, AT, compiled)
}

/// The status of `action` in state `state` of `fixture`.
fn action(fixture: &Fixture, compiled: &Compiled, state: &str, action: &str) -> String {
    actions(&decide(fixture, compiled, state))[action].clone()
}

#[test]
fn story_delivery_2_declares_its_settings_and_its_fixtures() {
    let builtin = canon_engineering::registry::get(NAME, MAJOR).expect("story-delivery@2 loads");
    assert_eq!(builtin.model.protocol.id.as_str(), "story.delivery");
    assert_eq!(builtin.model.protocol.revision, 2);
    let inputs: Vec<(&str, Vec<&str>)> = builtin
        .model
        .inputs()
        .map(|(id, input)| {
            (
                id.as_str(),
                input.values.iter().map(|value| value.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(
        inputs,
        [
            ("tests_first", vec!["required", "optional"]),
            ("gate_on", vec!["after_handover", "each_revision"]),
        ]
    );
    assert!(
        canon_engineering::registry::list().contains(&(NAME, 1)),
        "story.delivery/1 stays a built-in beside it"
    );

    let compiled = compiled();
    let claim_ids: Vec<&str> = compiled.ir().claims.keys().map(|id| id.as_str()).collect();
    assert_eq!(
        claim_ids,
        [
            "change.complete",
            "change.handed_over",
            "change.made",
            "ci.green",
            "gate.green",
            "pr.merged",
            "pr.opened",
            "tests.pass",
            "tests.shown_red",
        ]
    );
    assert_eq!(builtin_fixtures::fixture_ids(DIRECTORY), FIXTURES);
}

#[test]
fn red_first_refuses_implementation_first() {
    let compiled = compiled();
    let fixture = load("red-first-refuses-implementation-first", &compiled);
    for state in ["initial", "tests-written"] {
        assert_eq!(
            action(&fixture, &compiled, state, "implementation.edit"),
            "blocked",
            "`{state}`: no recorded fail yet"
        );
        assert_eq!(
            action(&fixture, &compiled, state, "tests.write"),
            "admissible",
            "`{state}`"
        );
    }
    let red = decide(&fixture, &compiled, "red");
    assert_eq!(claims(&red)["tests.shown_red"], "true");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
    // The run about h2 says nothing at h3: the edit waits for a run there, and a failing one
    // admits it again.
    let edited = decide(&fixture, &compiled, "edited");
    assert_eq!(claims(&edited)["tests.pass"], "unknown");
    assert_eq!(claims(&edited)["tests.shown_red"], "unknown");
    assert_eq!(actions(&edited)["implementation.edit"], "blocked");
    let again = decide(&fixture, &compiled, "still-red");
    assert_eq!(claims(&again)["tests.shown_red"], "true");
    assert_eq!(actions(&again)["implementation.edit"], "admissible");
}

#[test]
fn a_baseline_run_at_the_seed_does_not_lock_the_edit_out() {
    let compiled = compiled();
    for id in ["baseline-pass-then-red", "baseline-fail-then-red"] {
        let fixture = load(id, &compiled);
        for state in ["baseline", "tests-written"] {
            assert_eq!(
                action(&fixture, &compiled, state, "implementation.edit"),
                "blocked",
                "{id}: `{state}`"
            );
        }
        let red = decide(&fixture, &compiled, "red");
        assert_eq!(claims(&red)["tests.shown_red"], "true", "{id}");
        assert_eq!(actions(&red)["implementation.edit"], "admissible", "{id}");
    }
}

#[test]
fn a_red_run_about_another_revision_does_not_admit_the_edit() {
    let compiled = compiled();
    let fixture = load(
        "a-red-run-about-another-revision-does-not-admit-the-edit",
        &compiled,
    );
    let late = decide(&fixture, &compiled, "a-seed-run-fails-late");
    assert_eq!(claims(&late)["tests.shown_red"], "unknown");
    assert_eq!(actions(&late)["implementation.edit"], "blocked");
    assert!(excluded_only_as(
        &late,
        "tests.pass",
        ExclusionReason::RevisionMismatch
    ));
}

#[test]
fn a_first_run_pass_is_not_red() {
    let compiled = compiled();
    let fixture = load("a-first-run-pass-is-not-red", &compiled);
    for state in ["tests-written", "first-run-passes"] {
        let decision = decide(&fixture, &compiled, state);
        assert_ne!(claims(&decision)["tests.shown_red"], "true", "`{state}`");
        assert_eq!(
            actions(&decision)["implementation.edit"],
            "blocked",
            "`{state}`"
        );
    }
    let passed = decide(&fixture, &compiled, "first-run-passes");
    assert_eq!(claims(&passed)["tests.pass"], "true");
    assert_eq!(claims(&passed)["tests.shown_red"], "false");
    // Tests rewritten to fail are red at the revision they were written at.
    let rewritten = decide(&fixture, &compiled, "rewritten-tests-red");
    assert_eq!(claims(&rewritten)["tests.shown_red"], "true");
    assert_eq!(actions(&rewritten)["implementation.edit"], "admissible");
}

#[test]
fn tests_optional_keeps_the_old_order() {
    let compiled = compiled();
    let fixture = load("tests-optional-keeps-the-old-order", &compiled);
    for state in ["initial", "first-run-passes"] {
        for write in ["tests.write", "implementation.edit"] {
            assert_eq!(
                action(&fixture, &compiled, state, write),
                "admissible",
                "`{state}`: `{write}`"
            );
        }
    }
}

#[test]
fn gate_after_handover() {
    let compiled = compiled();
    let fixture = load("gate-after-handover", &compiled);
    for state in [
        "initial",
        "red-revision",
        "red-revision-handed-over",
        "green-before-handover",
    ] {
        assert_eq!(
            action(&fixture, &compiled, state, "gate.run"),
            "blocked",
            "`{state}`"
        );
    }
    for state in ["green-and-handed-over", "gate-passed"] {
        assert_eq!(
            action(&fixture, &compiled, state, "gate.run"),
            "admissible",
            "`{state}`"
        );
    }
}

#[test]
fn gate_each_revision() {
    let compiled = compiled();
    let fixture = load("gate-each-revision", &compiled);
    for state in fixture.states() {
        assert_eq!(
            action(&fixture, &compiled, state, "gate.run"),
            "admissible",
            "`{state}`"
        );
    }
}

#[test]
fn an_old_gate_pass_does_not_hold() {
    let compiled = compiled();
    let fixture = load("an-old-gate-pass-does-not-hold", &compiled);
    let passed = decide(&fixture, &compiled, "gate-passed");
    assert_eq!(claims(&passed)["gate.green"], "true");
    let moved = decide(&fixture, &compiled, "new-commit");
    assert_eq!(claims(&moved)["gate.green"], "unknown");
    assert!(excluded_only_as(
        &moved,
        "gate.green",
        ExclusionReason::RevisionMismatch
    ));
}

#[test]
fn pr_open_needs_change_complete() {
    let compiled = compiled();
    let fixture = load("pr-open-needs-change-complete", &compiled);
    for state in ["only-test-paths", "only-implementation-paths"] {
        let decision = decide(&fixture, &compiled, state);
        let found = claims(&decision);
        for green in [
            "tests.pass",
            "gate.green",
            "change.handed_over",
            "change.made",
        ] {
            assert_eq!(found[green], "true", "`{state}`: `{green}`");
        }
        assert_eq!(found["change.complete"], "false", "`{state}`");
        assert_eq!(actions(&decision)["pr.open"], "blocked", "`{state}`");
    }
    let complete = decide(&fixture, &compiled, "both-paths-complete");
    assert_eq!(claims(&complete)["change.complete"], "true");
    assert_eq!(actions(&complete)["pr.open"], "admissible");
    // A gate pass about h3 is no gate at h4: the change is not complete there.
    let moved = decide(&fixture, &compiled, "gate-pass-about-another-revision");
    assert_eq!(claims(&moved)["tests.pass"], "true");
    assert_eq!(claims(&moved)["change.complete"], "unknown");
    assert_eq!(actions(&moved)["pr.open"], "blocked");
}

#[test]
fn a_red_gate_admits_the_edit() {
    let compiled = compiled();
    let fixture = load("a-red-gate-admits-the-edit", &compiled);
    // Green tests, handed over, gate not yet run: nothing is red, so the edit waits for the gate.
    let handed = decide(&fixture, &compiled, "green-and-handed-over");
    assert_eq!(claims(&handed)["tests.shown_red"], "false");
    assert_eq!(actions(&handed)["implementation.edit"], "blocked");
    // The first gate fails at that revision on something no test reads: the edit is admissible,
    // or a format, lint or citation failure under `tests_first=required` could never be fixed.
    let red = decide(&fixture, &compiled, "gate-red");
    assert_eq!(claims(&red)["tests.pass"], "true");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
    assert_eq!(actions(&red)["change.handover"], "blocked");
    // The fix moves the revision; the red gate was about h3 and admits nothing at h4.
    let fixed = decide(&fixture, &compiled, "fixed-not-yet-gated");
    assert_eq!(claims(&fixed)["gate.green"], "unknown");
    assert_eq!(actions(&fixed)["implementation.edit"], "blocked");
    assert_eq!(actions(&fixed)["change.handover"], "admissible");
}

#[test]
fn a_red_ci_admits_the_edit() {
    let compiled = compiled();
    let fixture = load("a-red-ci-admits-the-edit", &compiled);
    // Green tests and gate, handed over, pull request open, CI not yet read: nothing is red.
    let opened = decide(&fixture, &compiled, "green-gated-and-opened");
    assert_eq!(claims(&opened)["gate.green"], "true");
    assert_eq!(actions(&opened)["implementation.edit"], "blocked");
    // CI fails at that revision on something neither the tests nor the gate read: the edit is
    // admissible, or such a failure under `tests_first=required` could never be fixed.
    let red = decide(&fixture, &compiled, "ci-red");
    assert_eq!(claims(&red)["tests.pass"], "true");
    assert_eq!(claims(&red)["gate.green"], "true");
    assert_eq!(claims(&red)["ci.green"], "false");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
    assert_eq!(actions(&red)["repository.merge"], "blocked");
    // The fix moves the revision; the red CI was about h3 and admits nothing at h4.
    let fixed = decide(&fixture, &compiled, "fixed-not-yet-tested");
    assert_eq!(claims(&fixed)["ci.green"], "unknown");
    assert_eq!(actions(&fixed)["implementation.edit"], "blocked");
}
