//! Parked case of adversary pass 1 on story:story-delivery-2-edit-after-a-red-gate, red on the
//! release that ships the red-gate rule and kept out of the build until
//! story:story-delivery-2-red-gate-needs-red-tests decides it: the order forms cannot yet say
//! that the tests were red at an earlier revision. To run it, move the function back into
//! `tests/story_delivery_2_red_gate_adv1.rs`; its fixture is
//! `fixtures/story-delivery-2/red-gate-adv1/adv1-a-red-gate-before-the-tests-were-red.fixture.yaml`.

/// Tests written and passing on their first run (never red), then the gate red on their
/// formatting: `a-first-run-pass-is-not-red` refuses the edit, and the red gate reopens it.
#[test]
fn a_red_gate_does_not_admit_the_edit_when_the_tests_were_never_red() {
    let compiled = compiled();
    let fixture = load("adv1-a-red-gate-before-the-tests-were-red", &compiled);
    let passed = decide(&fixture, &compiled, "tests-written-first-run-passes");
    assert_eq!(actions(&passed)["implementation.edit"], "blocked");
    let red = decide(&fixture, &compiled, "gate-red-on-the-tests-formatting");
    assert_eq!(claims(&red)["tests.shown_red"], "false");
    assert_eq!(
        actions(&red)["implementation.edit"],
        "blocked",
        "the tests were written and passed on their first run, so they were never red; a gate \
         failing on the new tests' formatting admits implementation.edit anyway"
    );
}
