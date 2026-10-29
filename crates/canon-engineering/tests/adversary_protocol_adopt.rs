//! Adversarial review of `story:protocol-adopt-protocol` at `a3f8688`: `protocol.adopt/1`.
//!
//! The behavioural cases build `protocol.adopt/1` fixtures in memory and evaluate them through the
//! harness, so every state reaches Canon at its own instant exactly as a fixture file's does. The
//! mutant case writes mutated copies of the protocol under the tree's `target/` (never the protocol
//! itself), compiles each and runs every fixture under `fixtures/protocol-adopt/` against it: a
//! mutant no fixture tells apart is a rule no fixture exercises (AGENTS.md: "Every protocol rule
//! lands with a fixture under `fixtures/` that exercises it").
//!
//! The invariants attacked are the story's and Atlas ADR 0086 § Decision 4-5: `adopted` only with
//! a diff report within the floor, a passing check report and an approval under the capability
//! `protocol.adopt`, each about the current revision of `generated_protocol`; `protocol.approve`
//! only once the revision is within the floor and its checks pass.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, ExclusionReason, Json};
use support::{Compiled, Fixture, compile_protocol, repo_root};

const PROTOCOL: &str = "protocols/protocol-adopt/1.yaml";

/// One evidence entry of a fixture state about `generated_protocol`, written as the fixture format
/// writes it.
fn entry(observed_at: &str, id: &str, kind: &str, result: &str, revision: &str) -> String {
    format!(
        "      - observed_at: \"{observed_at}\"\n        record: {{format: canon-evidence/1, \
         id: {id}, kind: {kind}, result: {result}, subject: generated_protocol, \
         subject_revision: {revision}}}\n"
    )
}

/// One state: its id, the extra keys it gives (already indented), and its evidence entries.
fn state(id: &str, extra: &str, evidence: &[String]) -> String {
    let evidence = if evidence.is_empty() {
        "    add_evidence: []\n".to_owned()
    } else {
        format!("    add_evidence:\n{}", evidence.concat())
    };
    format!("  - id: {id}\n{extra}{evidence}    expect: {{claims: {{}}}}\n")
}

const GRANT: &str = "    add_authority:\n      - {capability: protocol.adopt, decision: granted}\n";
const DENY: &str = "    add_authority:\n      - {capability: protocol.adopt, decision: denied}\n";

/// A `protocol.adopt/1` fixture evaluated at 12:00, the generated protocol at r1, with `states`.
fn adopt(states: &[String]) -> Fixture {
    let text = format!(
        "format: els-fixture/1\nid: adversary-protocol-adopt\nprotocol: {PROTOCOL}\n\
         at: \"2026-10-29T12:00:00Z\"\ncase:\n  format: canon-case/1\n  id: ADOPT-9001\n  \
         protocol: protocol.adopt\n  artifacts:\n    generated_protocol: {{revision: r1}}\n\
         states:\n{}",
        states.concat()
    );
    Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn compiled() -> Compiled {
    compile_protocol(PROTOCOL).unwrap_or_else(|error| panic!("{PROTOCOL}: {error}"))
}

fn decide(fixture: &Fixture, compiled: &Compiled, state: &str) -> Decision {
    fixture
        .evaluate(compiled, state)
        .unwrap_or_else(|refusal| panic!("state `{state}` refused: {}: {refusal}", refusal.code()))
}

fn claim(decision: &Decision, id: &str) -> String {
    decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .map_or("undeclared".to_owned(), |(_, entry)| {
            entry.value.to_string()
        })
}

fn excluded(decision: &Decision, id: &str) -> Vec<(String, ExclusionReason)> {
    decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .map(|(_, entry)| {
            entry
                .excluded_evidence
                .iter()
                .map(|exclusion| (exclusion.evidence.as_str().to_owned(), exclusion.reason))
                .collect()
        })
        .unwrap_or_default()
}

fn statuses(section: &Option<Json>) -> BTreeMap<String, String> {
    match section {
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

fn action(decision: &Decision, id: &str) -> String {
    statuses(&decision.actions)
        .get(id)
        .cloned()
        .unwrap_or_else(|| "undeclared".to_owned())
}

fn adopted(decision: &Decision) -> String {
    statuses(&decision.outcomes)
        .get("adopted")
        .cloned()
        .unwrap_or_else(|| "undeclared".to_owned())
}

/// A diff report within the floor and a passing check report, both about `revision`, observed at
/// `diff_at` and `check_at`.
fn reports(revision: &str, diff_at: &str, check_at: &str) -> Vec<String> {
    vec![
        entry(
            diff_at,
            &format!("diff-{revision}"),
            "diff_report",
            "within_floor",
            revision,
        ),
        entry(
            check_at,
            &format!("check-{revision}"),
            "check_report",
            "passed",
            revision,
        ),
    ]
}

// ---------------------------------------------------------------------------------------------
// The approval record
// ---------------------------------------------------------------------------------------------

/// `protocol.approve` is admissible "only once it is within the floor and its checks pass", so an
/// approval recorded before either report exists was given while approving was blocked. The
/// outcome reads `adoption.approved` with no order against the reports, so such a record adopts.
#[test]
fn an_approval_recorded_before_either_report_adopts_nothing() {
    let fixture = adopt(&[state(
        "approved-first",
        GRANT,
        &[
            entry(
                "2026-10-29T10:00:00Z",
                "approval-r1",
                "adoption_approval",
                "approved",
                "r1",
            ),
            reports("r1", "2026-10-29T11:00:00Z", "2026-10-29T11:05:00Z").concat(),
        ],
    )]);
    let decision = decide(&fixture, &compiled(), "approved-first");
    assert_eq!(claim(&decision, "floor.respected"), "true");
    assert_eq!(claim(&decision, "checks.pass"), "true");
    assert_eq!(
        adopted(&decision),
        "blocked",
        "an approval observed at 10:00, before the diff report (11:00) and the check report \
         (11:05) that protocol.approve's precondition needs, adopts the revision"
    );
}

/// ADR 0086 § Decision 4: "`adopted` requires the capability `protocol.adopt`". An
/// `adoption_approval` record that names no producer the case lists, beside the two reports,
/// adopts the revision with no authority decision at all and with `protocol.adopt` denied.
#[test]
fn an_approval_record_adopts_without_the_grant() {
    let mut evidence = reports("r1", "2026-10-29T11:00:00Z", "2026-10-29T11:05:00Z");
    evidence.push(entry(
        "2026-10-29T11:30:00Z",
        "approval-r1",
        "adoption_approval",
        "approved",
        "r1",
    ));
    let fixture = adopt(&[
        state("no-authority", "", &evidence),
        state("denied", DENY, &[]),
    ]);
    let compiled = compiled();
    let none = decide(&fixture, &compiled, "no-authority");
    let denied = decide(&fixture, &compiled, "denied");
    assert_eq!(action(&none, "protocol.approve"), "approval-required");
    assert_eq!(action(&denied, "protocol.approve"), "blocked");
    assert_eq!(
        (adopted(&none), adopted(&denied)),
        ("blocked".to_owned(), "blocked".to_owned()),
        "an approval record nobody vouches for adopts with protocol.adopt undecided (first) \
         and denied (second)"
    );
}

/// A rejected approval about the current revision is `false`, not `unknown`, and adopts nothing.
#[test]
fn a_rejected_approval_is_false_and_adopts_nothing() {
    let mut evidence = reports("r1", "2026-10-29T11:00:00Z", "2026-10-29T11:05:00Z");
    evidence.push(entry(
        "2026-10-29T11:30:00Z",
        "approval-r1",
        "adoption_approval",
        "rejected",
        "r1",
    ));
    let fixture = adopt(&[state("rejected", GRANT, &evidence)]);
    let decision = decide(&fixture, &compiled(), "rejected");
    assert_eq!(claim(&decision, "adoption.approved"), "false");
    assert_eq!(action(&decision, "protocol.approve"), "admissible");
    assert_eq!(adopted(&decision), "blocked");
}

/// An approval of r1 does not carry over to r2, even with fresh reports about r2 that pass: r2
/// needs its own approval (ADR 0086 § Decision 5).
#[test]
fn an_approval_of_r1_does_not_adopt_r2() {
    let mut first = reports("r1", "2026-10-29T10:00:00Z", "2026-10-29T10:05:00Z");
    first.push(entry(
        "2026-10-29T10:30:00Z",
        "approval-r1",
        "adoption_approval",
        "approved",
        "r1",
    ));
    let fixture = adopt(&[
        state("adopted-r1", GRANT, &first),
        state(
            "reports-r2",
            "    set_revisions:\n      generated_protocol: r2\n",
            &reports("r2", "2026-10-29T11:00:00Z", "2026-10-29T11:05:00Z"),
        ),
    ]);
    let compiled = compiled();
    assert_eq!(
        adopted(&decide(&fixture, &compiled, "adopted-r1")),
        "legitimate"
    );
    let moved = decide(&fixture, &compiled, "reports-r2");
    assert_eq!(claim(&moved, "floor.respected"), "true");
    assert_eq!(claim(&moved, "checks.pass"), "true");
    assert_eq!(claim(&moved, "adoption.approved"), "unknown");
    assert_eq!(
        excluded(&moved, "adoption.approved"),
        [("approval-r1".to_owned(), ExclusionReason::RevisionMismatch)]
    );
    assert_eq!(action(&moved, "protocol.approve"), "admissible");
    assert_eq!(adopted(&moved), "blocked");
}

// ---------------------------------------------------------------------------------------------
// The reports
// ---------------------------------------------------------------------------------------------

/// A failed check report about the current revision is `false`, blocks approving, and adopts
/// nothing even beside the grant and an approval record.
#[test]
fn a_failed_check_is_false_and_blocks_approval_and_adoption() {
    let fixture = adopt(&[state(
        "check-failed",
        GRANT,
        &[
            entry(
                "2026-10-29T11:00:00Z",
                "diff-r1",
                "diff_report",
                "within_floor",
                "r1",
            ),
            entry(
                "2026-10-29T11:05:00Z",
                "check-r1",
                "check_report",
                "failed",
                "r1",
            ),
            entry(
                "2026-10-29T11:30:00Z",
                "approval-r1",
                "adoption_approval",
                "approved",
                "r1",
            ),
        ],
    )]);
    let decision = decide(&fixture, &compiled(), "check-failed");
    assert_eq!(claim(&decision, "floor.respected"), "true");
    assert_eq!(claim(&decision, "checks.pass"), "false");
    assert_eq!(action(&decision, "protocol.approve"), "blocked");
    assert_eq!(adopted(&decision), "blocked");
}

/// Two reports about one revision that disagree decide nothing: `unknown`, not `true`, and the
/// revision is not adopted. The same for two approval records that disagree.
#[test]
fn conflicting_records_on_one_revision_adopt_nothing() {
    let fixture = adopt(&[state(
        "floor-conflict",
        GRANT,
        &[
            reports("r1", "2026-10-29T11:00:00Z", "2026-10-29T11:05:00Z").concat(),
            entry(
                "2026-10-29T11:10:00Z",
                "diff-r1-again",
                "diff_report",
                "outside_floor",
                "r1",
            ),
            entry(
                "2026-10-29T11:30:00Z",
                "approval-r1",
                "adoption_approval",
                "approved",
                "r1",
            ),
            entry(
                "2026-10-29T11:31:00Z",
                "approval-r1-again",
                "adoption_approval",
                "rejected",
                "r1",
            ),
        ],
    )]);
    let decision = decide(&fixture, &compiled(), "floor-conflict");
    assert_eq!(claim(&decision, "floor.respected"), "unknown");
    assert_eq!(claim(&decision, "adoption.approved"), "unknown");
    assert_eq!(action(&decision, "protocol.approve"), "blocked");
    assert_eq!(adopted(&decision), "blocked");
}

/// A report about an artifact the protocol does not declare is refused, never read as about the
/// generated protocol.
#[test]
fn a_report_about_another_artifact_is_refused() {
    let fixture = adopt(&[state(
        "other-subject",
        GRANT,
        &[
            "      - observed_at: \"2026-10-29T11:00:00Z\"\n        record: {format: \
             canon-evidence/1, id: diff-other, kind: diff_report, result: within_floor, \
             subject: floor_protocol, subject_revision: r1}\n"
                .to_owned(),
        ],
    )]);
    let refusal = fixture
        .evaluate(&compiled(), "other-subject")
        .expect_err("a report about an undeclared artifact is refused");
    assert_eq!(refusal.code(), "undeclared-artifact");
}

// ---------------------------------------------------------------------------------------------
// Mutants
// ---------------------------------------------------------------------------------------------

/// The fixtures under `fixtures/protocol-adopt/` that fail against the protocol with `from`
/// replaced by `to` once; panics when `from` is not in the protocol exactly once.
fn killed_by(name: &str, from: &str, to: &str) -> Vec<String> {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(PROTOCOL)).expect("protocol reads");
    assert_eq!(text.matches(from).count(), 1, "mutant `{name}`: {from:?}");
    let mutant = text.replacen(from, to, 1);
    let directory = "target/adversary-protocol-adopt-mutants";
    std::fs::create_dir_all(root.join(directory)).expect("mutant directory");
    let path = format!("{directory}/{name}.yaml");
    std::fs::write(root.join(&path), mutant).expect("mutant writes");
    let compiled = compile_protocol(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
    let mut files: Vec<String> = std::fs::read_dir(root.join("fixtures/protocol-adopt"))
        .expect("fixtures")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("utf-8")
        })
        .filter(|file| file.ends_with(".fixture.yaml"))
        .collect();
    files.sort();
    files
        .into_iter()
        .filter(|file| {
            let fixture = Fixture::load(&format!("fixtures/protocol-adopt/{file}"))
                .unwrap_or_else(|error| panic!("{file}: {error}"));
            fixture.check(&compiled).is_err()
        })
        .collect()
}

/// Each rule of `protocol.adopt/1` is killed by some fixture under `fixtures/protocol-adopt/`.
#[test]
fn protocol_adopt_mutants_are_killed_by_a_fixture() {
    let mutants: [(&str, &str, &str); 8] = [
        (
            "pa-outcome-drop-floor",
            "        - claim: floor.respected\n        - claim: checks.pass\n        \
             - claim: adoption.approved\n",
            "        - claim: checks.pass\n        - claim: adoption.approved\n",
        ),
        (
            "pa-outcome-drop-checks",
            "        - claim: checks.pass\n        - claim: adoption.approved\n",
            "        - claim: adoption.approved\n",
        ),
        (
            "pa-outcome-drop-approval",
            "        - claim: checks.pass\n        - claim: adoption.approved\n",
            "        - claim: checks.pass\n",
        ),
        (
            "pa-precondition-drop-floor",
            "    precondition:\n      all:\n        - claim: floor.respected\n",
            "    precondition:\n      all:\n",
        ),
        (
            "pa-precondition-drop-checks",
            "        - claim: checks.pass\n    requires:\n",
            "    requires:\n",
        ),
        (
            "pa-drop-capability",
            "    requires:\n      - capability: protocol.adopt\n",
            "",
        ),
        (
            "pa-check-any-result",
            "        kind: check_report\n        result: passed\n",
            "        kind: check_report\n",
        ),
        (
            "pa-approval-any-result",
            "        kind: adoption_approval\n        result: approved\n",
            "        kind: adoption_approval\n",
        ),
    ];
    let mut survivors = Vec::new();
    for (name, from, to) in &mutants {
        if killed_by(name, from, to).is_empty() {
            survivors.push(*name);
        }
    }
    assert!(
        survivors.is_empty(),
        "mutants no protocol-adopt fixture kills: {survivors:?}"
    );
}
