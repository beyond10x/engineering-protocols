//! Adversary pass on story:story-delivery-2-red-gate-needs-red-tests: under `tests_first=required`
//! a red gate or a red continuous integration admits `implementation.edit` only when the story's
//! tests were red after they were written. The rule reads that through Canon 0.8.0's `since`
//! form: a `test_result` of `fail` about the implementation, at the revision of the latest
//! `tests_change` and observed after it, excluded records included.
//!
//! Two cases recorded where Canon 0.7.0's `after`, the rule's first reading, stopped short: it
//! ordered records by when they were observed, not by revision, and anchored on the first
//! `tests_change`. Both histories admitted the edit before that rule too; under `since`
//! (story:story-delivery-2-red-after-the-latest-write) both are blocked.
//!
//! These cases drive the rule from histories built inline (a fixture document per case, evaluated
//! state by state and not checked against expectations), so they add nothing to the fixture
//! directories that `story_delivery_2_protocol.rs` lists.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each built-in protocol's test uses part of it
mod builtin_fixtures;

use builtin_fixtures::{actions, claims, decide};
use support::{Compiled, Fixture};

fn compiled() -> Compiled {
    builtin_fixtures::compile_major("story-delivery", 2)
}

/// One evidence record: observed at `at` (minutes after 09:00 on 2026-10-27), of `kind`, with
/// `result` when given, about `subject` at `revision`.
fn record(
    id: &str,
    at: u32,
    kind: &str,
    result: Option<&str>,
    subject: &str,
    revision: &str,
) -> String {
    let result = result.map(|r| format!(" result: {r},")).unwrap_or_default();
    format!(
        "      - observed_at: \"2026-10-27T{:02}:{:02}:00Z\"\n        record: {{format: canon-evidence/1, id: {id}, kind: {kind},{result} subject: {subject}, subject_revision: {revision}}}\n",
        9 + at / 60,
        at % 60
    )
}

/// A fixture over `story.delivery/2` with the case inputs given and one state per entry of
/// `states`: its id, the implementation revision it sets (if any) and the records it adds.
fn fixture(
    tests_first: &str,
    gate_on: &str,
    states: &[(&str, Option<&str>, Vec<String>)],
) -> Fixture {
    let mut text = format!(
        "format: els-fixture/1\nid: adv2-inline\nprotocol: protocols/story-delivery/2.yaml\nat: \"2026-10-27T12:00:00Z\"\ncase:\n  format: canon-case/1\n  id: SD-ADV2\n  protocol: story.delivery\n  revision: c1\n  artifacts:\n    intent: {{revision: i1}}\n    plan: {{revision: p1}}\n    implementation: {{revision: h1}}\n  inputs:\n    tests_first: {tests_first}\n    gate_on: {gate_on}\nstates:\n"
    );
    for (id, revision, records) in states {
        text.push_str(&format!("  - id: {id}\n"));
        if let Some(revision) = revision {
            text.push_str(&format!(
                "    set_revisions:\n      implementation: {revision}\n"
            ));
        }
        if records.is_empty() {
            text.push_str("    add_evidence: []\n");
        } else {
            text.push_str("    add_evidence:\n");
            for record in records {
                text.push_str(record);
            }
        }
        text.push_str("    expect:\n      claims: {}\n");
    }
    Fixture::from_yaml(&text)
        .unwrap_or_else(|error| panic!("the inline fixture loads: {error}\n{text}"))
}

/// SD-210's history carried on: the tests written at h2, a late failing run about h1 (the seed,
/// which carries none of the story's tests, "a late result of a run started before the write"),
/// then the tests passing at h2 on their first run and the gate red there. The tests were never
/// red after they were written. Canon 0.7.0's `after` read the late h1 record (observed after
/// the tests_change) as the red run and the red gate admitted the edit; `since` reads only
/// failures at h2, the revision of the latest write, so the edit is blocked
/// (story:story-delivery-2-red-after-the-latest-write).
#[test]
fn a_late_fail_about_the_seed_no_longer_admits_the_edit_on_a_red_gate() {
    late_body(&compiled());
}

fn late_body(compiled: &Compiled) {
    let fixture = fixture(
        "required",
        "each_revision",
        &[
            (
                "tests-written",
                Some("h2"),
                vec![record(
                    "tests-written-h2",
                    0,
                    "tests_change",
                    None,
                    "implementation",
                    "h2",
                )],
            ),
            (
                "a-seed-run-fails-late",
                None,
                vec![record(
                    "tests-h1-red",
                    60,
                    "test_result",
                    Some("fail"),
                    "implementation",
                    "h1",
                )],
            ),
            (
                "first-run-at-h2-passes-gate-red",
                None,
                vec![
                    record(
                        "tests-h2-pass",
                        70,
                        "test_result",
                        Some("pass"),
                        "implementation",
                        "h2",
                    ),
                    record(
                        "gate-h2-fail",
                        80,
                        "gate_result",
                        Some("fail"),
                        "implementation",
                        "h2",
                    ),
                ],
            ),
        ],
    );
    let late = decide(&fixture, compiled, "a-seed-run-fails-late");
    assert_eq!(actions(&late)["implementation.edit"], "blocked");
    let red = decide(&fixture, compiled, "first-run-at-h2-passes-gate-red");
    assert_eq!(claims(&red)["tests.pass"], "true");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(
        actions(&red)["implementation.edit"],
        "blocked",
        "the late failure is about h1, not h2, the revision of the latest tests_change, so it is \
         not a red run of the story's tests (story:story-delivery-2-red-after-the-latest-write)"
    );
}

/// The tests written and red at h2, the implementation edited to h3 where they pass; then more of
/// the story's tests written at h4 that pass on their first run (never red) and the gate red at
/// h4, for example on the new tests' formatting. Canon 0.7.0's `after` anchored on the first
/// tests_change, so the red run of the earlier tests counted for the tests written later and the
/// red gate admitted the edit; `since` anchors on the latest tests_change, at h4, where no run
/// failed, so the edit is blocked (story:story-delivery-2-red-after-the-latest-write).
#[test]
fn tests_written_again_and_never_red_are_no_longer_admitted_by_a_red_gate() {
    again_body(&compiled());
}

fn again_body(compiled: &Compiled) {
    let fixture = fixture(
        "required",
        "each_revision",
        &[
            (
                "tests-red",
                Some("h2"),
                vec![
                    record(
                        "tests-written-h2",
                        0,
                        "tests_change",
                        None,
                        "implementation",
                        "h2",
                    ),
                    record(
                        "tests-h2-fail",
                        10,
                        "test_result",
                        Some("fail"),
                        "implementation",
                        "h2",
                    ),
                ],
            ),
            (
                "implemented-green",
                Some("h3"),
                vec![record(
                    "tests-h3-pass",
                    20,
                    "test_result",
                    Some("pass"),
                    "implementation",
                    "h3",
                )],
            ),
            (
                "more-tests-first-run-passes",
                Some("h4"),
                vec![
                    record(
                        "tests-written-h4",
                        30,
                        "tests_change",
                        None,
                        "implementation",
                        "h4",
                    ),
                    record(
                        "tests-h4-pass",
                        40,
                        "test_result",
                        Some("pass"),
                        "implementation",
                        "h4",
                    ),
                ],
            ),
            (
                "gate-red-on-the-new-tests",
                None,
                vec![record(
                    "gate-h4-fail",
                    50,
                    "gate_result",
                    Some("fail"),
                    "implementation",
                    "h4",
                )],
            ),
        ],
    );
    let passed = decide(&fixture, compiled, "more-tests-first-run-passes");
    assert_eq!(actions(&passed)["implementation.edit"], "blocked");
    let red = decide(&fixture, compiled, "gate-red-on-the-new-tests");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(
        actions(&red)["implementation.edit"],
        "blocked",
        "the tests written at h4 were never red; the h2 failure belongs to the earlier write \
         (story:story-delivery-2-red-after-the-latest-write)"
    );
}

/// A failing run before any test was written (at the seed) is not a red run of the story's
/// tests: tests written at h2 and passing at once, then the gate red, does not admit the edit.
#[test]
fn a_fail_before_the_tests_were_written_does_not_make_them_red() {
    let compiled = compiled();
    let fixture = fixture(
        "required",
        "each_revision",
        &[
            (
                "seed-red",
                None,
                vec![record(
                    "tests-h1-red",
                    0,
                    "test_result",
                    Some("fail"),
                    "implementation",
                    "h1",
                )],
            ),
            (
                "tests-written-pass-gate-red",
                Some("h2"),
                vec![
                    record(
                        "tests-written-h2",
                        10,
                        "tests_change",
                        None,
                        "implementation",
                        "h2",
                    ),
                    record(
                        "tests-h2-pass",
                        20,
                        "test_result",
                        Some("pass"),
                        "implementation",
                        "h2",
                    ),
                    record(
                        "gate-h2-fail",
                        30,
                        "gate_result",
                        Some("fail"),
                        "implementation",
                        "h2",
                    ),
                ],
            ),
        ],
    );
    let red = decide(&fixture, &compiled, "tests-written-pass-gate-red");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(actions(&red)["implementation.edit"], "blocked");
}

/// A failing test_result about another artifact (the plan) after the tests were written is not a
/// red run of the story's tests.
#[test]
fn a_fail_about_another_subject_does_not_make_the_tests_red() {
    let compiled = compiled();
    let fixture = another_subject_red();
    let red = decide(&fixture, &compiled, "tests-written-plan-red-pass-gate-red");
    assert_eq!(claims(&red)["tests.pass"], "true");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(actions(&red)["implementation.edit"], "blocked");
}

fn another_subject_red() -> Fixture {
    fixture(
        "required",
        "each_revision",
        &[(
            "tests-written-plan-red-pass-gate-red",
            Some("h2"),
            vec![
                record(
                    "tests-written-h2",
                    0,
                    "tests_change",
                    None,
                    "implementation",
                    "h2",
                ),
                record("plan-red", 10, "test_result", Some("fail"), "plan", "p1"),
                record(
                    "tests-h2-pass",
                    20,
                    "test_result",
                    Some("pass"),
                    "implementation",
                    "h2",
                ),
                record(
                    "gate-h2-fail",
                    30,
                    "gate_result",
                    Some("fail"),
                    "implementation",
                    "h2",
                ),
            ],
        )],
    )
}

/// Tests written about the implementation at h2 and passing at once, then a later
/// `tests_change` about the plan with a failing run there, then the gate red at h2.
fn another_subject_written_and_red() -> Fixture {
    fixture(
        "required",
        "each_revision",
        &[(
            "plan-written-and-red-gate-red",
            Some("h2"),
            vec![
                record(
                    "tests-written-h2",
                    0,
                    "tests_change",
                    None,
                    "implementation",
                    "h2",
                ),
                record(
                    "tests-h2-pass",
                    10,
                    "test_result",
                    Some("pass"),
                    "implementation",
                    "h2",
                ),
                record("plan-written", 20, "tests_change", None, "plan", "p1"),
                record("plan-red", 30, "test_result", Some("fail"), "plan", "p1"),
                record(
                    "gate-h2-fail",
                    40,
                    "gate_result",
                    Some("fail"),
                    "implementation",
                    "h2",
                ),
            ],
        )],
    )
}

/// A write and a red run about another artifact after the implementation's tests were written
/// do not make those tests red: the latest write `since` reads is the implementation's.
#[test]
fn a_write_and_fail_about_another_subject_do_not_make_the_tests_red() {
    let red = decide(
        &another_subject_written_and_red(),
        &compiled(),
        "plan-written-and-red-gate-red",
    );
    assert_eq!(claims(&red)["tests.pass"], "true");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(actions(&red)["implementation.edit"], "blocked");
}

/// The mutant without `subject: implementation` in the `since` form anchors on the latest
/// `tests_change` of any artifact, the plan's, and reads the plan's red run after it, so it admits
/// the edit over `a_write_and_fail_about_another_subject_do_not_make_the_tests_red`'s history;
/// that case kills it. (`since` reads a failure only at its anchor's subject, so the history of
/// `a_fail_about_another_subject_does_not_make_the_tests_red` no longer does.)
#[test]
fn the_other_subject_write_kills_the_subject_less_since_mutant() {
    let path = support::repo_root().join("protocols/story-delivery/2.yaml");
    let text = std::fs::read_to_string(&path).expect("the protocol reads");
    let form = "                  result: fail\n                subject: implementation\n";
    assert_eq!(
        text.matches(form).count(),
        1,
        "the since form is written once"
    );
    let mutant =
        support::compile_protocol_text(&text.replace(form, "                  result: fail\n"))
            .unwrap_or_else(|error| panic!("the mutant compiles: {error}"));
    let red = decide(
        &another_subject_written_and_red(),
        &mutant,
        "plan-written-and-red-gate-red",
    );
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
    let red = decide(
        &another_subject_red(),
        &mutant,
        "tests-written-plan-red-pass-gate-red",
    );
    assert_eq!(actions(&red)["implementation.edit"], "blocked");
}

/// A failing run in the same second as the tests_change, both written in whole seconds, is not
/// ordered after it (`since` is unknown), and an unknown precondition does not admit the edit.
#[test]
fn a_fail_at_the_instant_of_the_write_does_not_admit_the_edit() {
    let compiled = compiled();
    let fixture = fixture(
        "required",
        "each_revision",
        &[
            (
                "tests-written-and-red-at-one-instant",
                Some("h2"),
                vec![
                    record(
                        "tests-written-h2",
                        0,
                        "tests_change",
                        None,
                        "implementation",
                        "h2",
                    ),
                    record(
                        "tests-h2-fail",
                        0,
                        "test_result",
                        Some("fail"),
                        "implementation",
                        "h2",
                    ),
                ],
            ),
            (
                "pass-and-gate-red",
                Some("h3"),
                vec![
                    record(
                        "tests-h3-pass",
                        20,
                        "test_result",
                        Some("pass"),
                        "implementation",
                        "h3",
                    ),
                    record(
                        "gate-h3-fail",
                        30,
                        "gate_result",
                        Some("fail"),
                        "implementation",
                        "h3",
                    ),
                ],
            ),
        ],
    );
    let red = decide(&fixture, &compiled, "pass-and-gate-red");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(actions(&red)["implementation.edit"], "blocked");
}

/// The red run comes after a second tests_change, at its revision: the order is read from the
/// latest write, so the red run counts, and the edit is admitted.
#[test]
fn a_red_run_after_a_second_write_admits_the_edit_on_a_red_gate() {
    let compiled = compiled();
    let fixture = fixture(
        "required",
        "each_revision",
        &[
            (
                "written-twice-then-red",
                Some("h3"),
                vec![
                    record(
                        "tests-written-h2",
                        0,
                        "tests_change",
                        None,
                        "implementation",
                        "h2",
                    ),
                    record(
                        "tests-h2-pass",
                        5,
                        "test_result",
                        Some("pass"),
                        "implementation",
                        "h2",
                    ),
                    record(
                        "tests-written-h3",
                        10,
                        "tests_change",
                        None,
                        "implementation",
                        "h3",
                    ),
                    record(
                        "tests-h3-fail",
                        20,
                        "test_result",
                        Some("fail"),
                        "implementation",
                        "h3",
                    ),
                ],
            ),
            (
                "green-gate-red",
                Some("h4"),
                vec![
                    record(
                        "tests-h4-pass",
                        30,
                        "test_result",
                        Some("pass"),
                        "implementation",
                        "h4",
                    ),
                    record(
                        "gate-h4-fail",
                        40,
                        "gate_result",
                        Some("fail"),
                        "implementation",
                        "h4",
                    ),
                ],
            ),
        ],
    );
    let red_tests = decide(&fixture, &compiled, "written-twice-then-red");
    assert_eq!(actions(&red_tests)["implementation.edit"], "admissible");
    let red = decide(&fixture, &compiled, "green-gate-red");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
}

/// Under `tests_first=optional` the never-red history changes nothing: the edit is admissible
/// before any test, after a first-run pass, and with the gate red.
#[test]
fn tests_optional_admits_the_edit_in_the_never_red_history() {
    let compiled = compiled();
    let fixture = fixture(
        "optional",
        "each_revision",
        &[
            ("seed", None, vec![]),
            (
                "first-run-passes",
                Some("h2"),
                vec![
                    record(
                        "tests-written-h2",
                        0,
                        "tests_change",
                        None,
                        "implementation",
                        "h2",
                    ),
                    record(
                        "tests-h2-pass",
                        10,
                        "test_result",
                        Some("pass"),
                        "implementation",
                        "h2",
                    ),
                ],
            ),
            (
                "gate-red",
                None,
                vec![record(
                    "gate-h2-fail",
                    20,
                    "gate_result",
                    Some("fail"),
                    "implementation",
                    "h2",
                )],
            ),
        ],
    );
    for state in ["seed", "first-run-passes", "gate-red"] {
        let decision = decide(&fixture, &compiled, state);
        assert_eq!(
            actions(&decision)["implementation.edit"],
            "admissible",
            "`{state}`"
        );
    }
}

/// One evidence record observed at 09:00:`second`.`millis` on 2026-10-27, written at millisecond
/// precision, of `kind`, with `result` when given, about the implementation at `revision`.
fn record_ms(
    id: &str,
    second: u32,
    millis: u32,
    kind: &str,
    result: Option<&str>,
    revision: &str,
) -> String {
    let result = result.map(|r| format!(" result: {r},")).unwrap_or_default();
    format!(
        "      - observed_at: \"2026-10-27T09:00:{second:02}.{millis:03}Z\"\n        record: {{format: canon-evidence/1, id: {id}, kind: {kind},{result} subject: implementation, subject_revision: {revision}}}\n"
    )
}

/// The tests written at `write_at` and run red at `fail_at` milliseconds into one second, every
/// record at millisecond precision; then the implementation edited to h3, where the tests pass,
/// and the gate red there.
fn one_second(write_at: u32, fail_at: u32) -> Fixture {
    fixture(
        "required",
        "each_revision",
        &[
            (
                "written-and-run-in-one-second",
                Some("h2"),
                vec![
                    record_ms("tests-written-h2", 0, write_at, "tests_change", None, "h2"),
                    record_ms(
                        "tests-h2-fail",
                        0,
                        fail_at,
                        "test_result",
                        Some("fail"),
                        "h2",
                    ),
                ],
            ),
            (
                "pass-and-gate-red",
                Some("h3"),
                vec![
                    record_ms("tests-h3-pass", 20, 0, "test_result", Some("pass"), "h3"),
                    record_ms("gate-h3-fail", 30, 0, "gate_result", Some("fail"), "h3"),
                ],
            ),
        ],
    )
}

/// takt's self-runs recorded the tests_change and the first red run within one second in 8 of 16
/// cases. Written at millisecond precision, a failure 500 ms after the write in the same second
/// is ordered after it, and the red gate admits the edit; a failure 500 ms before the write in
/// that second is not, and the edit is blocked; at one millisecond the two are unordered, and
/// the edit is blocked.
#[test]
fn a_fail_later_in_the_same_second_admits_the_edit_at_millisecond_precision() {
    let compiled = compiled();
    for (write_at, fail_at, expected) in [
        (250, 750, "admissible"),
        (750, 250, "blocked"),
        (500, 500, "blocked"),
    ] {
        let fixture = one_second(write_at, fail_at);
        let red = decide(&fixture, &compiled, "pass-and-gate-red");
        assert_eq!(claims(&red)["gate.green"], "false");
        assert_eq!(
            actions(&red)["implementation.edit"],
            expected,
            "write {write_at} ms, fail {fail_at} ms"
        );
    }
}
