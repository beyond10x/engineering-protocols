//! Adversarial review of `story:evidence-order-red-first`, at `cdcffee`: `software.change/1`'s
//! red-first rule read from evidence order (Canon 0.3.0 `first` and `before`), driven with
//! histories the story's three fixtures do not hold.
//!
//! The red-first rule of the test-driven principle (Atlas ADR 0084; AEP
//! `principles/development/test-driven.yaml`) needs two things of a bugfix: the first result of its
//! regression test failed, and that run came before the first change of the implementation.
//! `regression.shown_first` reads only the second, so a regression test that passed first and ran
//! before the edit discharges `red_first.required`.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::eval;
use b10x_canon::model::{Decision, Json};
use support::Fixture;

/// A `software.change/1` bugfix fixture at standard risk and implementation revision R1, with the
/// given states (YAML, indented as list items under `states:`).
fn bugfix(id: &str, states: &str) -> Fixture {
    let text = format!(
        "format: els-fixture/1
id: {id}
protocol: protocols/software-change/1.yaml
at: \"2026-10-13T12:00:00Z\"
case:
  format: canon-case/1
  id: CHG-ADV3
  protocol: software.change
  inputs:
    risk: standard
    change_kind: bugfix
    affects_security_boundary: false
  artifacts:
    intent: {{revision: i1}}
    system_specification: {{revision: s1}}
    plan: {{revision: p1}}
    implementation: {{revision: R1}}
    release: {{revision: v0}}
    deployment: {{revision: d0}}
states:
{states}"
    );
    Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{id} loads: {error}"))
}

/// One evidence entry, as a state's `add_evidence` item.
fn entry(at: &str, id: &str, kind: &str, result: Option<&str>, subject: &str, rev: &str) -> String {
    let result = result.map_or(String::new(), |result| format!(", result: {result}"));
    format!(
        "      - observed_at: \"{at}\"
        record: {{format: canon-evidence/1, id: {id}, kind: {kind}{result}, subject: {subject}, subject_revision: {rev}}}
"
    )
}

/// One state with no expectations of its own beyond the claims being reported.
fn state(id: &str, set_implementation: Option<&str>, evidence: &[String]) -> String {
    let set = set_implementation.map_or(String::new(), |rev| {
        format!("    set_revisions:\n      implementation: {rev}\n")
    });
    let evidence = if evidence.is_empty() {
        "    add_evidence: []\n".to_owned()
    } else {
        format!("    add_evidence:\n{}", evidence.concat())
    };
    format!("  - id: {id}\n{set}{evidence}    expect:\n      claims: {{}}\n")
}

struct Seen {
    claims: BTreeMap<String, String>,
    obligations: BTreeMap<String, String>,
    actions: BTreeMap<String, String>,
}

fn statuses(section: &Option<Json>) -> BTreeMap<String, String> {
    match section {
        Some(Json::Array(entries)) => entries
            .iter()
            .map(|entry| {
                (
                    entry["id"].as_str().unwrap_or_default().to_owned(),
                    entry["status"].as_str().unwrap_or_default().to_owned(),
                )
            })
            .collect(),
        Some(Json::Object(entries)) => entries
            .iter()
            .map(|(id, entry)| {
                (
                    id.clone(),
                    entry["status"].as_str().unwrap_or_default().to_owned(),
                )
            })
            .collect(),
        _ => BTreeMap::new(),
    }
}

fn seen(decision: &Decision) -> Seen {
    Seen {
        claims: decision
            .claims
            .iter()
            .map(|(id, entry)| (id.as_str().to_owned(), entry.value.to_string()))
            .collect(),
        obligations: statuses(&decision.obligations),
        actions: statuses(&decision.actions),
    }
}

fn decide(fixture: &Fixture, state: &str) -> Seen {
    let compiled = fixture.compile().expect("software.change/1 compiles");
    let decision = fixture
        .evaluate(&compiled, state)
        .unwrap_or_else(|refusal| panic!("state `{state}` refused: {refusal}"));
    seen(&decision)
}

const T08: &str = "2026-10-13T08:00:00Z";
const T09: &str = "2026-10-13T09:00:00Z";
const T10: &str = "2026-10-13T10:00:00Z";
const T11: &str = "2026-10-13T11:00:00Z";

/// ADR 0084: red first means the first regression test result FAILED and came before the first
/// implementation change. Here the regression test passes first on R1 (it reproduces nothing), a
/// failing suite run on R1 admits the edit by the Decision 1 path, and the edit moves the
/// implementation to R2. The regression test was never red before the edit, so
/// `red_first.required` must stay open. It is discharged: `regression.shown_first` is only
/// `before{regression_test_result, implementation_change}` and never reads the result.
#[test]
fn a_regression_test_that_passed_first_does_not_discharge_red_first() {
    let fixture = bugfix(
        "adv3-green-first-edit",
        &[
            state(
                "regression-green",
                None,
                &[entry(
                    T09,
                    "reg-r1-pass",
                    "regression_test_result",
                    Some("pass"),
                    "implementation",
                    "R1",
                )],
            ),
            state(
                "suite-red",
                None,
                &[entry(
                    T10,
                    "suite-r1-fail",
                    "test_result",
                    Some("fail"),
                    "implementation",
                    "R1",
                )],
            ),
            state(
                "edited",
                Some("R2"),
                &[entry(
                    T11,
                    "change-r2",
                    "implementation_change",
                    None,
                    "implementation",
                    "R2",
                )],
            ),
        ]
        .concat(),
    );
    // The edit is reachable: the protocol admits it before the change is recorded.
    let before_edit = decide(&fixture, "suite-red");
    assert_eq!(before_edit.actions["implementation.edit"], "admissible");
    let edited = decide(&fixture, "edited");
    assert_eq!(
        edited.obligations["red_first.required"], "open",
        "red_first.required is discharged by a regression test whose first result was a pass \
         (regression.shown_first = {}): the order form reads that the test ran before the edit, \
         not that it ran red",
        edited.claims["regression.shown_first"]
    );
}

/// The same hole with the regression test passing first and failing later, both before the edit:
/// `first` is FALSE (the fixture `red-first-green-first` pins that), yet the obligation that is
/// named for red-first is discharged.
#[test]
fn green_then_red_before_the_edit_does_not_discharge_red_first() {
    let fixture = bugfix(
        "adv3-green-red-edit",
        &[
            state(
                "runs",
                None,
                &[
                    entry(
                        T08,
                        "reg-r1-pass",
                        "regression_test_result",
                        Some("pass"),
                        "implementation",
                        "R1",
                    ),
                    entry(
                        T09,
                        "reg-r1-fail",
                        "regression_test_result",
                        Some("fail"),
                        "implementation",
                        "R1",
                    ),
                    entry(
                        T10,
                        "suite-r1-fail",
                        "test_result",
                        Some("fail"),
                        "implementation",
                        "R1",
                    ),
                ],
            ),
            state(
                "edited",
                Some("R2"),
                &[entry(
                    T11,
                    "change-r2",
                    "implementation_change",
                    None,
                    "implementation",
                    "R2",
                )],
            ),
        ]
        .concat(),
    );
    // The edit is reachable: the failing suite run on R1 admits it.
    let runs = decide(&fixture, "runs");
    assert_eq!(runs.actions["implementation.edit"], "admissible");
    let edited = decide(&fixture, "edited");
    assert_eq!(
        edited.obligations["red_first.required"], "open",
        "red_first.required is discharged although the first regression test result was a pass"
    );
}

/// Ties at the earliest instant: a red and a green regression run at one instant leave `first`
/// UNKNOWN and the edit blocked; a red run and the first change at one instant leave
/// `regression.shown_first` UNKNOWN and the obligation open.
#[test]
fn ties_at_the_earliest_instant_settle_nothing() {
    let fixture = bugfix(
        "adv3-ties",
        &[
            state(
                "red-and-green",
                None,
                &[
                    entry(
                        T09,
                        "reg-a-fail",
                        "regression_test_result",
                        Some("fail"),
                        "implementation",
                        "R1",
                    ),
                    entry(
                        T09,
                        "reg-b-pass",
                        "regression_test_result",
                        Some("pass"),
                        "implementation",
                        "R1",
                    ),
                ],
            ),
            state(
                "changed-at-once",
                Some("R2"),
                &[entry(
                    T09,
                    "change-r2",
                    "implementation_change",
                    None,
                    "implementation",
                    "R2",
                )],
            ),
        ]
        .concat(),
    );
    let tied = decide(&fixture, "red-and-green");
    assert_eq!(tied.claims["regression.reproduced"], "unknown");
    assert_eq!(tied.actions["implementation.edit"], "blocked");
    let changed = decide(&fixture, "changed-at-once");
    assert_eq!(changed.claims["regression.shown_first"], "unknown");
    assert_eq!(changed.obligations["red_first.required"], "open");
}

/// Records about another subject: a red regression run about the release reproduces nothing, and
/// a release-subject `implementation_change` is not the implementation's first change. These also
/// kill the mutants that drop `subject: implementation` from either order form, which no fixture
/// of the story does (see `adversary3_red_first_mutants.rs`).
#[test]
fn order_forms_read_only_the_implementation() {
    let other = bugfix(
        "adv3-other-subject",
        &state(
            "release-red",
            None,
            &[entry(
                T09,
                "reg-release-fail",
                "regression_test_result",
                Some("fail"),
                "release",
                "v0",
            )],
        ),
    );
    let seen = decide(&other, "release-red");
    assert_eq!(seen.claims["regression.reproduced"], "unknown");
    assert_eq!(seen.actions["implementation.edit"], "blocked");

    let mixed = bugfix(
        "adv3-mixed-subjects",
        &[
            state(
                "release-green-first",
                None,
                &[
                    entry(
                        T08,
                        "reg-release-pass",
                        "regression_test_result",
                        Some("pass"),
                        "release",
                        "v0",
                    ),
                    entry(
                        T08,
                        "change-release",
                        "implementation_change",
                        None,
                        "release",
                        "v0",
                    ),
                    entry(
                        T09,
                        "reg-r1-fail",
                        "regression_test_result",
                        Some("fail"),
                        "implementation",
                        "R1",
                    ),
                ],
            ),
            state(
                "edited",
                Some("R2"),
                &[entry(
                    T10,
                    "change-r2",
                    "implementation_change",
                    None,
                    "implementation",
                    "R2",
                )],
            ),
        ]
        .concat(),
    );
    let seen = decide(&mixed, "release-green-first");
    assert_eq!(seen.claims["regression.reproduced"], "true");
    let edited = decide(&mixed, "edited");
    assert_eq!(edited.claims["regression.shown_first"], "true");
    assert_eq!(edited.obligations["red_first.required"], "discharged");
}

/// Canon's stated limit (`story:revision-ordering` there): a red regression run about a revision
/// the implementation never had counts like any other. Pinned as it stands, not as a finding.
#[test]
fn a_red_run_about_a_revision_never_held_counts_canons_stated_limit() {
    let fixture = bugfix(
        "adv3-foreign-revision",
        &state(
            "foreign-red",
            None,
            &[entry(
                T09,
                "reg-rx-fail",
                "regression_test_result",
                Some("fail"),
                "implementation",
                "RX-never-held",
            )],
        ),
    );
    let seen = decide(&fixture, "foreign-red");
    assert_eq!(seen.claims["regression.reproduced"], "true");
    assert_eq!(seen.actions["implementation.edit"], "admissible");
}

/// A record without `observed_at` has no place in the order: a caller other than the fixture
/// harness (which always sets one) that omits it leaves `first` and `before` UNKNOWN, so the
/// edit stays blocked and the obligation open.
#[test]
fn a_record_without_an_observation_time_settles_no_order() {
    let fixture = bugfix(
        "adv3-unplaced",
        &[state(
            "red",
            None,
            &[
                entry(
                    T09,
                    "reg-r1-fail",
                    "regression_test_result",
                    Some("fail"),
                    "implementation",
                    "R1",
                ),
                entry(
                    T10,
                    "reg-r1-fail-unplaced",
                    "regression_test_result",
                    Some("fail"),
                    "implementation",
                    "R1",
                ),
                entry(
                    T11,
                    "change-unplaced",
                    "implementation_change",
                    None,
                    "implementation",
                    "R1",
                ),
            ],
        )]
        .concat(),
    );
    let compiled = fixture.compile().expect("compiles");
    let mut evidence = fixture.evidence("red");
    for record in &mut evidence {
        if record.id.as_str().ends_with("unplaced") {
            record.observed_at = None;
        }
    }
    let decision = eval::evaluate_with(
        compiled.ir(),
        fixture.case(),
        &evidence,
        eval::Supplied::default(),
    )
    .expect("evaluates");
    let seen = seen(&decision);
    assert_eq!(seen.claims["regression.reproduced"], "unknown");
    assert_eq!(seen.actions["implementation.edit"], "blocked");
    assert_eq!(seen.claims["regression.shown_first"], "unknown");
    assert_eq!(seen.obligations["red_first.required"], "open");
}
