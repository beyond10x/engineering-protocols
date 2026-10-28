//! Adversary pass 1 on `story:story-delivery-2-edit-after-a-red-gate`: `implementation.edit`
//! under `tests_first=required` is admissible while `gate.green` is false at the current revision.
//!
//! The story's outcome is that a gate failing on something no test reads can be fixed after the
//! tests went red and then green. The `tests_first` input still says the tests must fail before the
//! implementation is edited. These cases drive the new branch from histories in which no test was
//! ever red: the red gate must not admit the edit there.
//!
//! The fixtures live in `fixtures/story-delivery-2/red-gate-adv1/`, a subdirectory, because
//! `story_delivery_2_protocol.rs` asserts the exact fixture list of `fixtures/story-delivery-2/`
//! and `fixture_ids` reads only that directory's own files. Each fixture's claims are checked by
//! `fixture_of`; the actions are asserted here. The case of tests that passed on their first run
//! is parked in `tests/pending/` (story:story-delivery-2-red-gate-needs-red-tests).

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each built-in protocol's test uses part of it
mod builtin_fixtures;

use b10x_canon::model::ExclusionReason;
use builtin_fixtures::{actions, claims, decide, excluded_only_as, fixture_of};
use support::{Compiled, Fixture};

const NAME: &str = "story-delivery";
const DIRECTORY: &str = "story-delivery-2/red-gate-adv1";
const MAJOR: u32 = 2;
const AT: &str = "2026-10-27T12:00:00Z";

fn compiled() -> Compiled {
    builtin_fixtures::compile_major(NAME, MAJOR)
}

fn load(id: &str, compiled: &Compiled) -> Fixture {
    fixture_of(DIRECTORY, NAME, MAJOR, id, AT, compiled)
}

/// No test written, the gate red at the seed and again after an edit: the implementation was
/// edited before any test existed.
#[test]
fn a_red_gate_at_the_seed_does_not_admit_the_edit_before_any_test() {
    let compiled = compiled();
    let fixture = load("adv1-a-red-gate-at-the-seed-before-any-test", &compiled);
    for state in ["seed-gate-red", "implementation-edited-gate-red-again"] {
        let decision = decide(&fixture, &compiled, state);
        assert_eq!(actions(&decision)["gate.run"], "admissible", "`{state}`");
        assert_eq!(
            actions(&decision)["implementation.edit"],
            "blocked",
            "`{state}`: tests_first=required and no tests_change recorded, yet a red gate \
             (gate.green = {}) admits implementation.edit: the implementation is edited before \
             any test exists",
            claims(&decision)["gate.green"]
        );
    }
}

/// `gate_on=after_handover`: no test written, the seed handed over with its existing tests
/// passing, the gate red there.
#[test]
fn a_red_gate_at_a_handed_over_seed_does_not_admit_the_edit() {
    let compiled = compiled();
    let fixture = load("adv1-a-red-gate-at-a-handed-over-seed", &compiled);
    let handed = decide(&fixture, &compiled, "seed-green-and-handed-over");
    assert_eq!(actions(&handed)["gate.run"], "admissible");
    assert_eq!(actions(&handed)["implementation.edit"], "blocked");
    let red = decide(&fixture, &compiled, "seed-gate-red");
    assert_eq!(
        actions(&red)["implementation.edit"],
        "blocked",
        "tests_first=required, gate_on=after_handover, no tests_change recorded: a handover at \
         the seed and a red gate there admit implementation.edit before any test exists"
    );
}

/// A gate failure about h2 arriving while the implementation is at h3 says nothing at h3.
#[test]
fn a_late_red_gate_about_another_revision_does_not_admit_the_edit() {
    let compiled = compiled();
    let fixture = load("adv1-a-late-red-gate-about-another-revision", &compiled);
    let late = decide(&fixture, &compiled, "late-gate-red-about-h2");
    assert_eq!(claims(&late)["gate.green"], "unknown");
    assert!(excluded_only_as(
        &late,
        "gate.green",
        ExclusionReason::RevisionMismatch
    ));
    assert_eq!(actions(&late)["implementation.edit"], "blocked");
    assert_eq!(actions(&late)["change.handover"], "admissible");
}
