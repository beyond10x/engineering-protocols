//! Adversary pass on story:story-delivery-2-red-after-the-latest-write: the `since` form reads
//! failures only at the revision of the latest `tests_change`, so a failure at a later revision
//! that still holds the same tests (no write in between) is not read, and when the write and its
//! first red run share an instant at the evaluation's resolution (whole seconds, as a recorder
//! that writes no fraction gives), no failure counts. The two cases pin that narrowing and that
//! sub-second instants resolve it.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

#[allow(dead_code)] // each built-in protocol's test uses part of it
mod builtin_fixtures;

use builtin_fixtures::{actions, claims, decide};
use support::Fixture;

/// One evidence record about the implementation at `revision`, observed at `at` minutes after
/// 09:00 on 2026-10-27, written in whole seconds as takt writes them.
fn record(id: &str, at: u32, kind: &str, result: Option<&str>, revision: &str) -> String {
    let result = result.map(|r| format!(" result: {r},")).unwrap_or_default();
    format!(
        "      - observed_at: \"2026-10-27T{:02}:{:02}:00Z\"\n        record: {{format: canon-evidence/1, id: {id}, kind: {kind},{result} subject: implementation, subject_revision: {revision}}}\n",
        9 + at / 60,
        at % 60
    )
}

fn fixture(states: &[(&str, &str, Vec<String>)]) -> Fixture {
    let mut text = String::from(
        "format: els-fixture/1\nid: adv3-inline\nprotocol: protocols/story-delivery/2.yaml\nat: \"2026-10-27T12:00:00Z\"\ncase:\n  format: canon-case/1\n  id: SD-ADV3\n  protocol: story.delivery\n  revision: c1\n  artifacts:\n    intent: {revision: i1}\n    plan: {revision: p1}\n    implementation: {revision: h1}\n  inputs:\n    tests_first: required\n    gate_on: each_revision\nstates:\n",
    );
    for (id, revision, records) in states {
        text.push_str(&format!(
            "  - id: {id}\n    set_revisions:\n      implementation: {revision}\n    add_evidence:\n"
        ));
        for record in records {
            text.push_str(record);
        }
        text.push_str("    expect:\n      claims: {}\n");
    }
    Fixture::from_yaml(&text)
        .unwrap_or_else(|error| panic!("the inline fixture loads: {error}\n{text}"))
}

/// The same history with every instant written at `fraction` (empty for whole seconds): the tests
/// written at h2 and run red there 500 ms later; the implementation edited to h3, where the same
/// tests, with no write in between, fail again a minute later; edited to h4, where they pass, and
/// the gate red there, for example on formatting.
fn edited_after_a_red_run(fraction: &str) -> Fixture {
    let at = |id: &str, minute: u32, millis: u32, kind: &str, result: Option<&str>, rev: &str| {
        let mut line = record(id, minute, kind, result, rev);
        if !fraction.is_empty() {
            let second = format!(":{minute:02}:00Z", minute = minute % 60);
            let stamped = format!(":{:02}:00.{millis:03}Z", minute % 60);
            line = line.replacen(&second, &stamped, 1);
        }
        line
    };
    fixture(&[
        (
            "tests-written-and-red",
            "h2",
            vec![
                at("tests-written-h2", 0, 0, "tests_change", None, "h2"),
                at("tests-h2-fail", 0, 500, "test_result", Some("fail"), "h2"),
            ],
        ),
        (
            "edited-still-red",
            "h3",
            vec![at("tests-h3-fail", 1, 0, "test_result", Some("fail"), "h3")],
        ),
        (
            "edited-green-gate-red",
            "h4",
            vec![
                at("tests-h4-pass", 2, 0, "test_result", Some("pass"), "h4"),
                at("gate-h4-fail", 3, 0, "gate_result", Some("fail"), "h4"),
            ],
        ),
    ])
}

/// The rule is narrower than "a red run at any revision holding the tests": `since` reads
/// failures only at the revision of the latest `tests_change`. Written in whole seconds, the write
/// and its red run at h2 share one second and are unordered, so the later red run at h3 does not
/// admit the edit on a red gate (Canon 0.7.0's `after` did). The decision for
/// story:story-delivery-2-red-after-the-latest-write is to keep the narrower rule, which reads
/// only runs of the tests as last written, and to resolve the tie by sub-second instants
/// (`the_same_history_at_millisecond_precision_admits_the_edit`).
#[test]
fn a_red_run_at_a_later_revision_does_not_stand_in_for_one_at_the_write() {
    let compiled = builtin_fixtures::compile_major("story-delivery", 2);
    let red = decide(
        &edited_after_a_red_run(""),
        &compiled,
        "edited-green-gate-red",
    );
    assert_eq!(claims(&red)["tests.pass"], "true");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(
        actions(&red)["implementation.edit"],
        "blocked",
        "the h2 failure shares the write's second, and the h3 failure is not at the write's \
         revision (story:story-delivery-2-red-after-the-latest-write, Delivered)"
    );
}

/// Written in milliseconds, the h2 failure is ordered after the write, and the red gate admits
/// the edit, so a recorder that writes sub-second instants never meets the tie above.
#[test]
fn the_same_history_at_millisecond_precision_admits_the_edit() {
    let compiled = builtin_fixtures::compile_major("story-delivery", 2);
    let red = decide(
        &edited_after_a_red_run("ms"),
        &compiled,
        "edited-green-gate-red",
    );
    assert_eq!(claims(&red)["tests.pass"], "true");
    assert_eq!(claims(&red)["gate.green"], "false");
    assert_eq!(actions(&red)["implementation.edit"], "admissible");
}
