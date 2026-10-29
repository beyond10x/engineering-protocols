//! Adversary pass 1 on `story:story-delivery-2-edit-after-a-red-ci`: `implementation.edit` under
//! `tests_first=required` is admissible while `ci.green` is false at the current revision and the
//! story's tests are written and pass there.
//!
//! These cases drive the new branch from histories the story's fixture does not hold: a red CI
//! about an earlier revision, at a seed with no test written, while the tests fail, with records
//! that disagree, under `tests_first=optional`, together with a red gate under
//! `gate_on=each_revision`, and through the loop to a merge.
//!
//! The fixtures live in `fixtures/story-delivery-2/red-ci-adv1/`, a subdirectory, because
//! `story_delivery_2_protocol.rs` asserts the exact fixture list of `fixtures/story-delivery-2/`
//! and `fixture_ids` reads only that directory's own files. Each fixture's claims are checked by
//! `fixture_of`; the actions are asserted here. Tests that passed on their first run were never
//! red, and a red CI does not admit the edit for them
//! (story:story-delivery-2-red-gate-needs-red-tests).

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each built-in protocol's test uses part of it
mod builtin_fixtures;

use b10x_canon::model::ExclusionReason;
use builtin_fixtures::{actions, claims, decide, excluded_only_as, fixture_of, outcomes};
use support::{Compiled, Fixture};

const NAME: &str = "story-delivery";
const DIRECTORY: &str = "story-delivery-2/red-ci-adv1";
const MAJOR: u32 = 2;
const AT: &str = "2026-10-27T12:00:00Z";

fn compiled() -> Compiled {
    builtin_fixtures::compile_major(NAME, MAJOR)
}

fn load(id: &str, compiled: &Compiled) -> Fixture {
    fixture_of(DIRECTORY, NAME, MAJOR, id, AT, compiled)
}

/// A CI failure about h3 arriving after the tests were written again at h4 says nothing at h4.
#[test]
fn a_late_red_ci_about_another_revision_does_not_admit_the_edit() {
    let compiled = compiled();
    let fixture = load("adv1-a-late-red-ci-about-another-revision", &compiled);
    let rewritten = decide(&fixture, &compiled, "tests-rewritten-and-passing");
    assert_eq!(actions(&rewritten)["implementation.edit"], "blocked");
    let late = decide(&fixture, &compiled, "late-ci-red-about-h3");
    assert_eq!(claims(&late)["ci.green"], "unknown");
    assert!(excluded_only_as(
        &late,
        "ci.green",
        ExclusionReason::RevisionMismatch
    ));
    assert_eq!(
        actions(&late)["implementation.edit"],
        "blocked",
        "a ci_result fail about h3 admits implementation.edit at h4"
    );
    assert_eq!(actions(&late)["change.handover"], "admissible");
    assert_eq!(actions(&late)["ci.read"], "blocked");
}

/// No test written: a red CI at the seed, and a red gate beside it, admit no edit.
#[test]
fn a_red_ci_at_the_seed_does_not_admit_the_edit_before_any_test() {
    let compiled = compiled();
    let fixture = load("adv1-a-red-ci-at-the-seed-before-any-test", &compiled);
    for state in ["seed-opened-ci-red", "seed-gate-red-too"] {
        let decision = decide(&fixture, &compiled, state);
        assert_eq!(claims(&decision)["ci.green"], "false", "`{state}`");
        assert_eq!(
            actions(&decision)["implementation.edit"],
            "blocked",
            "`{state}`: tests_first=required and no tests_change recorded, yet a red CI \
             (gate.green = {}) admits implementation.edit before any test exists",
            claims(&decision)["gate.green"]
        );
        assert_eq!(
            actions(&decision)["repository.merge"],
            "blocked",
            "`{state}`"
        );
    }
}

/// A red CI while the tests fail: at the seed with no test written the edit stays refused; with
/// the tests written and red, it is admissible, and the red CI changes nothing about that.
#[test]
fn a_red_ci_while_the_tests_fail() {
    let compiled = compiled();
    let fixture = load("adv1-a-red-ci-while-the-tests-fail", &compiled);
    let seed = decide(&fixture, &compiled, "seed-tests-fail-ci-red");
    assert_eq!(
        actions(&seed)["implementation.edit"],
        "blocked",
        "failing seed tests and a red CI, no tests_change: the edit is admitted"
    );
    let red = decide(&fixture, &compiled, "tests-written-red");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
    let both = decide(&fixture, &compiled, "tests-red-and-ci-red");
    assert_eq!(actions(&both)["implementation.edit"], "admissible");
    assert_eq!(actions(&both)["repository.merge"], "blocked");
}

/// A red CI and a green re-run at one revision disagree: `ci.green` is unknown, and neither the
/// edit nor the merge is admissible; only a new revision moves the story on.
#[test]
fn ci_records_that_disagree_admit_neither_the_edit_nor_the_merge() {
    let compiled = compiled();
    let fixture = load("adv1-ci-records-that-disagree", &compiled);
    let red = decide(&fixture, &compiled, "ci-red");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
    assert_eq!(actions(&red)["ci.read"], "admissible");
    let rerun = decide(&fixture, &compiled, "ci-green-on-a-rerun");
    assert_eq!(claims(&rerun)["ci.green"], "unknown");
    assert_eq!(actions(&rerun)["implementation.edit"], "blocked");
    assert_eq!(actions(&rerun)["repository.merge"], "blocked");
    assert_eq!(actions(&rerun)["change.handover"], "blocked");
    assert_eq!(actions(&rerun)["tests.write"], "admissible");
}

/// `gate_on=each_revision`: a red CI, then a red gate re-run beside it, admit the edit; at the
/// next revision, with the tests and the gate green, it is refused although h3's red records
/// stand; the loop reaches CI green, the merge and `accepted`.
#[test]
fn the_fix_loop_at_each_revision() {
    let compiled = compiled();
    let fixture = load("adv1-the-fix-loop-at-each-revision", &compiled);
    let opened = decide(&fixture, &compiled, "green-gated-and-opened");
    assert_eq!(actions(&opened)["implementation.edit"], "blocked");
    let red = decide(&fixture, &compiled, "ci-red");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
    assert_eq!(actions(&red)["pr.open"], "admissible");
    assert_eq!(actions(&red)["repository.merge"], "blocked");
    let both = decide(&fixture, &compiled, "ci-red-and-gate-rerun-red");
    assert_eq!(actions(&both)["implementation.edit"], "admissible");
    assert_eq!(actions(&both)["pr.open"], "blocked");
    let fixed = decide(&fixture, &compiled, "fixed-tests-and-gate-green");
    assert_eq!(
        actions(&fixed)["implementation.edit"],
        "blocked",
        "h4 has its tests and gate green and no CI read; h3's red CI admits the edit at h4"
    );
    assert_eq!(actions(&fixed)["gate.run"], "admissible");
    assert_eq!(actions(&fixed)["change.handover"], "admissible");
    assert_eq!(actions(&fixed)["pr.open"], "blocked");
    assert_eq!(actions(&fixed)["ci.read"], "blocked");
    assert_eq!(actions(&fixed)["repository.merge"], "blocked");
    let green = decide(&fixture, &compiled, "opened-and-ci-green");
    assert_eq!(actions(&green)["implementation.edit"], "blocked");
    assert_eq!(actions(&green)["repository.merge"], "approval-required");
    let merged = decide(&fixture, &compiled, "merged");
    assert_eq!(actions(&merged)["implementation.edit"], "blocked");
    assert_eq!(outcomes(&merged)["accepted"], "legitimate");
}

/// `tests_first=optional`: the edit is admissible before and after the red CI.
#[test]
fn a_red_ci_under_tests_optional_changes_nothing() {
    let compiled = compiled();
    let fixture = load("adv1-a-red-ci-under-tests-optional", &compiled);
    for state in ["seed-gated-and-opened", "ci-red"] {
        let decision = decide(&fixture, &compiled, state);
        assert_eq!(
            actions(&decision)["implementation.edit"],
            "admissible",
            "`{state}`"
        );
    }
}

/// Tests written and passing on their first run (never red), handed over, gated green and opened,
/// then CI red on another platform: the red CI does not admit the edit, as the red gate does not.
#[test]
fn a_red_ci_does_not_admit_the_edit_when_the_tests_were_never_red() {
    let compiled = compiled();
    let fixture = load("adv1-a-red-ci-before-the-tests-were-red", &compiled);
    let opened = decide(&fixture, &compiled, "first-run-passes-gated-and-opened");
    assert_eq!(actions(&opened)["implementation.edit"], "blocked");
    let red = decide(&fixture, &compiled, "ci-red-on-another-platform");
    assert_eq!(claims(&red)["ci.green"], "false");
    assert_eq!(claims(&red)["tests.shown_red"], "false");
    assert_eq!(
        actions(&red)["implementation.edit"],
        "blocked",
        "the tests were written and passed on their first run, so they were never red; a CI \
         failure admits implementation.edit anyway"
    );
}
