//! Adversary pass 1 on `story:software-change-protocol`.
//!
//! Two kinds of case:
//!
//! - cases run against the shipped `protocols/software-change/1.yaml` that assert what the story,
//!   design § 9 and `docs/examples/software-change.md` say, and that the shipped protocol does not
//!   do;
//! - protocol mutants that are wrong by the same sources and still pass every check the acceptance
//!   test `chg_1842_merge_waits_for_current_revision_tests_and_authority` makes. Each mutant is
//!   compiled from a copy of the protocol text in memory; nothing under `protocols/` or
//!   `fixtures/` is written.
//!
//! The acceptance test's checks are replayed here with Canon directly, because the harness's
//! `Compiled` is built only from a file in the repository.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::{BTreeMap, BTreeSet};

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir::Ir;
use b10x_canon::model::{
    ActionId, ClaimId, Decision, EvidenceRecord, ExclusionReason, Predicate, Truth,
};
use serde_yaml_ng::Value;

const PROTOCOL: &str = "protocols/software-change/1.yaml";
const CHG_1842: &str = "fixtures/software-change/chg-1842.fixture.yaml";

fn read(path: &str) -> String {
    std::fs::read_to_string(support::repo_root().join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn compile(text: &str) -> Ir {
    let protocol = b10x_canon::model::parse(text).unwrap_or_else(|error| panic!("{error}"));
    b10x_canon::validate::validate(&protocol).unwrap_or_else(|problems| panic!("{problems:?}"));
    b10x_canon::ir::compile(&protocol).unwrap_or_else(|problems| panic!("{problems:?}"))
}

fn shipped() -> Ir {
    compile(&read(PROTOCOL))
}

/// The protocol with `from` replaced by `to` once, compiled by Canon.
fn mutant(from: &str, to: &str) -> Ir {
    let text = read(PROTOCOL);
    assert!(text.contains(from), "the protocol contains `{from}`");
    compile(&text.replacen(from, to, 1))
}

/// A `canon-case/1` snapshot of CHG-1842 with implementation revision `implementation`, with the
/// case inputs `chg-1842.fixture.yaml` gives.
fn case(implementation: &str) -> Value {
    serde_yaml_ng::from_str(&format!(
        "{{format: canon-case/1, id: CHG-1842, protocol: software.change, \
         inputs: {{risk: standard, change_kind: refactor, affects_security_boundary: false}}, \
         artifacts: \
         {{intent: {{revision: i1}}, system_specification: {{revision: s1}}, \
         plan: {{revision: p1}}, implementation: {{revision: {implementation}}}, \
         release: {{revision: v0}}, deployment: {{revision: d0}}}}}}"
    ))
    .unwrap()
}

fn record(id: &str, kind: &str, result: &str, subject: &str, revision: &str) -> EvidenceRecord {
    let value: Value = serde_yaml_ng::from_str(&format!(
        "{{format: canon-evidence/1, id: {id}, kind: {kind}, result: {result}, \
         subject: {subject}, subject_revision: {revision}}}"
    ))
    .unwrap();
    eval::evidence_from_value(&value).unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn decide(ir: &Ir, case: &Value, evidence: &[EvidenceRecord], authority: Option<&str>) -> Decision {
    let case = eval::case_from_value(case).unwrap_or_else(|refusal| panic!("{refusal}"));
    let supplied = Supplied {
        authority,
        ..Supplied::default()
    };
    eval::evaluate_with(ir, &case, evidence, supplied).unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn claim(decision: &Decision, id: &str) -> Truth {
    decision.claims.get(&ClaimId::new(id)).unwrap().value
}

fn action(decision: &Decision, id: &str) -> String {
    decision.actions.as_ref().unwrap()[id]["status"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn outcome(decision: &Decision, id: &str) -> String {
    decision.outcomes.as_ref().unwrap()[id]["status"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn text(value: &Value) -> String {
    match value {
        Value::Bool(flag) => flag.to_string(),
        Value::String(text) => text.clone(),
        other => panic!("unexpected expectation {other:?}"),
    }
}

fn expected(map: &Value) -> BTreeMap<String, String> {
    map.as_mapping()
        .map(|entries| {
            entries
                .iter()
                .map(|(key, value)| (text(key), text(value)))
                .collect()
        })
        .unwrap_or_default()
}

/// The claims and evidence kinds `predicate` reaches, as `software_change_protocol.rs:46-64`.
fn reached(ir: &Ir, predicate: &Predicate) -> (Vec<String>, Vec<String>) {
    let mut claims = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    let mut pending = vec![predicate];
    while let Some(next) = pending.pop() {
        next.visit(&mut |node| match node {
            Predicate::Claim(test) => {
                if claims.insert(test.claim.as_str().to_owned()) {
                    pending.push(&ir.claims[&test.claim].true_when);
                }
            }
            Predicate::Evidence(matching) => {
                kinds.insert(matching.kind.as_str().to_owned());
            }
            Predicate::First(_) | Predicate::Before(_) => {
                for (kind, _) in node.order_reads() {
                    kinds.insert(kind.as_str().to_owned());
                }
            }
            Predicate::All(_) | Predicate::Any(_) | Predicate::Not(_) | Predicate::Input(_) => {}
        });
    }
    (claims.into_iter().collect(), kinds.into_iter().collect())
}

const OPEN_ACTIONS: [&str; 3] = ["repository.inspect", "tests.write", "tests.run"];

/// Every check `chg_1842_merge_waits_for_current_revision_tests_and_authority` makes on a
/// compiled protocol (`software_change_protocol.rs:135-271`, the name lists aside, which no
/// mutant here changes), replayed against `ir`: one line per check that fails.
fn acceptance_differences(ir: &Ir) -> Vec<String> {
    let mut differences = Vec::new();
    let mut check = |ok: bool, what: &str| {
        if !ok {
            differences.push(what.to_owned());
        }
    };

    let verified = &ir.claims[&ClaimId::new("implementation.verified")].true_when;
    let verified_ok = matches!(verified, Predicate::All(conjuncts)
        if conjuncts.len() == 2
            && conjuncts.iter().filter(|conjunct| matches!(conjunct, Predicate::Evidence(_))).count()
                == 1
            && conjuncts.iter().any(|conjunct| matches!(conjunct, Predicate::Evidence(matching)
                if matching.kind.as_str() == "test_result"
                    && matching.result.as_deref() == Some("pass"))));
    check(verified_ok, "implementation.verified shape");

    let merge = &ir.actions[&ActionId::new("repository.merge")];
    let capabilities: Vec<&str> = merge.requires.iter().map(|id| id.as_str()).collect();
    check(capabilities == ["repository.merge"], "merge capabilities");
    let (claims, kinds) = reached(ir, &merge.precondition);
    check(
        claims == ["implementation.verified", "security.reviewed"],
        "merge reached claims",
    );
    check(
        kinds == ["security_review", "test_result"],
        "merge reached kinds",
    );
    for open in OPEN_ACTIONS {
        let declared = &ir.actions[&ActionId::new(open)];
        check(declared.requires.is_empty(), open);
        check(declared.precondition == Predicate::All(Vec::new()), open);
    }
    let produces: Vec<&str> = ir.actions[&ActionId::new("tests.run")]
        .may_produce
        .iter()
        .map(|id| id.as_str())
        .collect();
    check(
        produces == ["regression_test_result", "test_result"],
        "tests.run produces",
    );

    let fixture = support::Fixture::load(CHG_1842).unwrap_or_else(|error| panic!("{error}"));
    let document: Value = serde_yaml_ng::from_str(&read(CHG_1842)).unwrap();
    let case = document["case"].clone();
    let mut authority: Option<Vec<Value>> = None;
    for state in document["states"].as_sequence().unwrap() {
        let id = state["id"].as_str().unwrap();
        assert!(
            state.get("set_revisions").is_none(),
            "chg-1842 sets no revision"
        );
        if let Some(decisions) = state.get("add_authority").and_then(Value::as_sequence)
            && (!decisions.is_empty() || authority.is_some())
        {
            authority
                .get_or_insert_with(Vec::new)
                .extend(decisions.iter().cloned());
        }
        let authority_text = authority
            .as_ref()
            .map(|decisions| serde_yaml_ng::to_string(decisions).unwrap());
        let decision = decide(ir, &case, &fixture.evidence(id), authority_text.as_deref());

        let claims: BTreeMap<String, String> = decision
            .claims
            .iter()
            .map(|(claim, entry)| (claim.as_str().to_owned(), entry.value.to_string()))
            .collect();
        if claims != expected(&state["expect"]["claims"]) {
            differences.push(format!("state `{id}`: claims {claims:?}"));
        }
        let actions: BTreeMap<String, String> = decision
            .actions
            .as_ref()
            .and_then(|section| section.as_object())
            .map(|entries| {
                entries
                    .iter()
                    .map(|(id, entry)| (id.clone(), entry["status"].as_str().unwrap().to_owned()))
                    .collect()
            })
            .unwrap_or_default();
        if actions != expected(&state["expect"]["actions"]) {
            differences.push(format!("state `{id}`: actions {actions:?}"));
        }
        for reaching in ["tests.pass", "implementation.verified", "release.proven"] {
            let excluded: Vec<(String, ExclusionReason)> = decision
                .claims
                .get(&ClaimId::new(reaching))
                .unwrap()
                .excluded_evidence
                .iter()
                .map(|exclusion| (exclusion.evidence.as_str().to_owned(), exclusion.reason))
                .collect();
            if excluded != [("tests-r1".to_owned(), ExclusionReason::RevisionMismatch)] {
                differences.push(format!("state `{id}`: {reaching} excluded {excluded:?}"));
            }
        }
        if (id == "initial" || id == "merge-approved")
            && outcome(&decision, "accepted") != "blocked"
        {
            differences.push(format!("state `{id}`: accepted not blocked"));
        }
    }
    differences
}

/// The replay is faithful: the shipped protocol passes it.
#[test]
fn the_replayed_acceptance_passes_the_shipped_protocol() {
    assert_eq!(acceptance_differences(&shipped()), Vec::<String>::new());
}

/// Story summary: "revision-bound implementation evidence"; story Outcome: "`test_result` (bound
/// to the implementation revision)"; the protocol's own `tests.pass` description: "the tests pass
/// for the current implementation revision". A passing `test_result` whose subject is the
/// release (current revision v0), not the implementation, is not a test of R2. Revision binding
/// keeps it, because it is bound to its own subject's current revision, and nothing in the
/// protocol ties `test_result` to `implementation`. Expected: `tests.pass` and
/// `implementation.verified` stay UNKNOWN and merging stays blocked, as in state `initial`.
///
/// Tried for every artifact the change does not move (each at its current revision in the case),
/// with the merge grant of state `merge-approved` given.
#[test]
fn a_test_result_about_another_artifact_does_not_verify_the_implementation() {
    let ir = shipped();
    let granted = "- {capability: repository.merge, decision: granted}\n";
    let mut verified_by = Vec::new();
    for (subject, revision) in [
        ("intent", "i1"),
        ("system_specification", "s1"),
        ("plan", "p1"),
        ("release", "v0"),
        ("deployment", "d0"),
    ] {
        let evidence = [
            record("tests-r1", "test_result", "pass", "implementation", "R1"),
            record("tests-other", "test_result", "pass", subject, revision),
        ];
        let decision = decide(&ir, &case("R2"), &evidence, Some(granted));
        let found = (
            claim(&decision, "tests.pass"),
            claim(&decision, "implementation.verified"),
            action(&decision, "repository.merge"),
        );
        if found != (Truth::Unknown, Truth::Unknown, "blocked".to_owned()) {
            verified_by.push(format!("{subject}@{revision}: {found:?}"));
        }
    }
    assert_eq!(
        verified_by,
        Vec::<String>::new(),
        "implementation R2 was never tested; a passing test_result about each of these artifacts \
         made tests.pass and implementation.verified TRUE and repository.merge admissible"
    );
}

/// The artifact each evidence kind is about, from design § 9 (`evidence:` section, `subject:`):
/// `test_result` and `code_review` are about the implementation, `build_provenance` about the
/// release, `operational_observation` about the deployment. § 9 declares no `objective_observation`
/// evidence kind; its subject is `intent` because § 9's `intent` artifact is the only one that
/// carries the objective (`artifacts:` section, `required: [objective, …]`). § 9's
/// `completion.rolled_back` gives `rollback_result` no subject; it is about the release, the
/// artifact `release.rollback` returns to a previous revision (story:software-change-negative-outcomes).
/// `security_review` is about the implementation revision it reviews
/// (story:security-independence-rules).
const ABOUT: [(&str, &str); 7] = [
    ("test_result", "implementation"),
    ("code_review", "implementation"),
    ("build_provenance", "release"),
    ("operational_observation", "deployment"),
    ("objective_observation", "intent"),
    ("rollback_result", "release"),
    ("security_review", "implementation"),
];

/// Each declared artifact's revision in the CHG-1842 snapshot `case("R2")`.
const CURRENT: [(&str, &str); 6] = [
    ("intent", "i1"),
    ("system_specification", "s1"),
    ("plan", "p1"),
    ("implementation", "R2"),
    ("release", "v0"),
    ("deployment", "d0"),
];

fn about(kind: &str) -> &'static str {
    ABOUT
        .iter()
        .find(|(declared, _)| *declared == kind)
        .unwrap_or_else(|| panic!("`ABOUT` names no subject for `{kind}`"))
        .1
}

/// Where [`about`]'s answer for `kind` comes from.
fn basis(kind: &str) -> &'static str {
    match kind {
        "objective_observation" => "the intent artifact's required objective field, design § 9",
        "rollback_result" => "the artifact release.rollback returns to a previous revision",
        "security_review" => "the implementation revision it reviews",
        _ => "the evidence kind's subject, design § 9",
    }
}

fn current(artifact: &str) -> &'static str {
    CURRENT
        .iter()
        .find(|(declared, _)| *declared == artifact)
        .unwrap_or_else(|| panic!("the case has no artifact `{artifact}`"))
        .1
}

/// Every evidence match the compiled protocol holds, as `(where, kind, result, subject)`, in claims,
/// obligations, actions and outcomes.
fn evidence_matches(ir: &Ir) -> Vec<(String, String, Option<String>, Option<String>)> {
    let mut found = Vec::new();
    let mut collect = |at: String, predicate: &Predicate| {
        predicate.visit(&mut |node| {
            if let Predicate::Evidence(matching) = node {
                found.push((
                    at.clone(),
                    matching.kind.as_str().to_owned(),
                    matching.result.clone(),
                    matching.subject.as_ref().map(|id| id.as_str().to_owned()),
                ));
            }
        });
    };
    for (id, claim) in &ir.claims {
        collect(format!("claim {id}"), &claim.true_when);
    }
    for (id, obligation) in &ir.obligations {
        collect(format!("obligation {id}"), &obligation.discharged_when);
    }
    for (id, action) in &ir.actions {
        collect(format!("action {id}"), &action.precondition);
    }
    for (id, outcome) in &ir.outcomes {
        if let Some(requires) = outcome.requires.predicate() {
            collect(format!("outcome {id}"), requires);
        }
    }
    found
}

/// F1, the class: every evidence match in `software.change/1` names, as its subject, the artifact
/// its kind is about ([`ABOUT`]): for four kinds the subject § 9's `evidence:` section declares,
/// for `objective_observation` the `intent` artifact, whose required `objective` field it observes
/// (§ 9 declares no such evidence kind). A match without a subject reads a record of its kind about
/// any declared artifact (Canon 8d1599e `EvidenceMatch::subject`).
#[test]
fn every_evidence_match_names_the_artifact_its_kind_is_about() {
    let ir = shipped();
    let matches = evidence_matches(&ir);
    assert!(!matches.is_empty(), "the protocol matches evidence");
    let unbound: Vec<String> = matches
        .iter()
        .filter(|(_, kind, _, subject)| subject.as_deref() != Some(about(kind)))
        .map(|(at, kind, _, subject)| {
            format!(
                "{at}: {kind} about {subject:?}, its kind is about {} ({})",
                about(kind),
                basis(kind)
            )
        })
        .collect();
    assert_eq!(unbound, Vec::<String>::new());
}

/// F1, the class, by evaluation: for every claim's evidence match, a record of the matched kind and
/// result about any other declared artifact, at that artifact's current revision, leaves the claim
/// where it was. The other kinds are supplied about their own artifacts with the result their
/// matches need, so a claim that also needs them (`release.proven` needs
/// `implementation.verified`) is decided by the record under test alone.
#[test]
fn evidence_about_another_artifact_moves_no_claim() {
    let ir = shipped();
    let matches = evidence_matches(&ir);
    let positive = |kind: &str| -> String {
        matches
            .iter()
            .find(|(_, declared, _, _)| declared == kind)
            .and_then(|(_, _, result, _)| result.clone())
            .unwrap_or_else(|| "present".to_owned())
    };
    let mut moved = Vec::new();
    for (at, kind, result, _) in &matches {
        let Some(claim_id) = at.strip_prefix("claim ") else {
            continue;
        };
        let baseline: Vec<EvidenceRecord> = ABOUT
            .iter()
            .filter(|(other, _)| other != kind)
            .map(|(other, subject)| {
                record(
                    &format!("{other}-own"),
                    other,
                    &positive(other),
                    subject,
                    current(subject),
                )
            })
            .collect();
        let before = claim(&decide(&ir, &case("R2"), &baseline, None), claim_id);
        for (artifact, revision) in CURRENT {
            if artifact == about(kind) {
                continue;
            }
            let mut evidence = baseline.clone();
            let shown = result.clone().unwrap_or_else(|| "present".to_owned());
            evidence.push(record("elsewhere", kind, &shown, artifact, revision));
            let after = claim(&decide(&ir, &case("R2"), &evidence, None), claim_id);
            if after != before {
                moved.push(format!(
                    "{claim_id}: {kind} {shown} about {artifact}@{revision} moved it {before:?} -> {after:?}"
                ));
            }
        }
    }
    assert_eq!(moved, Vec::<String>::new());
}

/// Design § 9: `accepted` is the terminal state reached only from `observing`, which is reached
/// only through `candidate` (`implementation.verified`) and `released` (`release.proven`). Canon
/// has no states, so the outcome's `requires` is the only place that path can be kept. The
/// shipped `accepted` requires `objective.realized` and `deployment.healthy` and nothing about
/// the implementation, so a change never tested at its current revision, never merged, released
/// or deployed is legitimately accepted as soon as the existing deployment is healthy and the
/// objective is observed met.
#[test]
fn accepted_is_blocked_while_the_implementation_is_unverified() {
    let ir = shipped();
    let evidence = [
        record("tests-r1", "test_result", "pass", "implementation", "R1"),
        record(
            "health-d0",
            "operational_observation",
            "healthy",
            "deployment",
            "d0",
        ),
        record(
            "objective-i1",
            "objective_observation",
            "satisfied",
            "intent",
            "i1",
        ),
    ];
    let decision = decide(&ir, &case("R2"), &evidence, None);
    assert_eq!(claim(&decision, "implementation.verified"), Truth::Unknown);
    assert_eq!(action(&decision, "repository.merge"), "blocked");
    assert_eq!(
        outcome(&decision, "accepted"),
        "blocked",
        "the change is accepted while implementation R2 is unverified and unmerged"
    );
}

const ACCEPTED_ALL: &str = "    requires:
      all:
        - claim: objective.realized";
const ACCEPTED_ANY: &str = "    requires:
      any:
        - claim: objective.realized";

/// "Accepted legitimate too early": `accepted` on objective realized OR deployment healthy. No
/// fixture state makes one of the two claims TRUE without the other (both are UNKNOWN in every
/// state), and the acceptance asserts only `blocked`, so the mutant passes.
#[test]
fn acceptance_rejects_accepted_on_any_instead_of_all() {
    let wrong = mutant(ACCEPTED_ALL, ACCEPTED_ANY);

    let healthy_only = [record(
        "health-d0",
        "operational_observation",
        "healthy",
        "deployment",
        "d0",
    )];
    assert_eq!(
        outcome(
            &decide(&shipped(), &case("R2"), &healthy_only, None),
            "accepted"
        ),
        "blocked"
    );
    assert_eq!(
        outcome(
            &decide(&wrong, &case("R2"), &healthy_only, None),
            "accepted"
        ),
        "legitimate"
    );

    let differences = acceptance_differences(&wrong);
    assert!(
        !differences.is_empty(),
        "a protocol whose `accepted` needs only one of objective.realized and deployment.healthy \
         passes the acceptance"
    );
}

const TESTS_PASS: &str = "  tests.pass:
    description: the tests pass for the current implementation revision
    true_when:
      evidence:
        kind: test_result
        result: pass
";
const TESTS_PASS_ANY_RESULT: &str = "  tests.pass:
    description: the tests pass for the current implementation revision
    true_when:
      evidence:
        kind: test_result
";

/// `tests.pass` true on any `test_result`, a failing one included. Every test result in
/// `chg-1842` passes, and the acceptance pins the result of `implementation.verified`'s conjunct
/// (`software_change_protocol.rs:193`) but not that of `tests.pass`, so the mutant passes.
#[test]
fn acceptance_rejects_tests_pass_on_a_failing_test_result() {
    let wrong = mutant(TESTS_PASS, TESTS_PASS_ANY_RESULT);

    let failing_r2 = [record(
        "tests-r2",
        "test_result",
        "fail",
        "implementation",
        "R2",
    )];
    assert_eq!(
        claim(
            &decide(&shipped(), &case("R2"), &failing_r2, None),
            "tests.pass"
        ),
        Truth::False
    );
    assert_eq!(
        claim(
            &decide(&wrong, &case("R2"), &failing_r2, None),
            "tests.pass"
        ),
        Truth::True
    );

    let differences = acceptance_differences(&wrong);
    assert!(
        !differences.is_empty(),
        "a protocol whose tests.pass is TRUE on a failing test result passes the acceptance"
    );
}

/// The same for the three result-qualified claims no fixture state ever gives evidence for:
/// dropping `result:` from any of them passes the acceptance.
#[test]
fn acceptance_rejects_claims_that_ignore_their_evidence_result() {
    let cases = [
        (
            "        kind: code_review\n        result: approved\n",
            "code_review",
        ),
        (
            "        kind: operational_observation\n        result: healthy\n",
            "operational_observation",
        ),
        (
            "        kind: objective_observation\n        result: satisfied\n",
            "objective_observation",
        ),
    ];
    let mut passed = Vec::new();
    for (from, kind) in cases {
        let wrong = mutant(from, &format!("        kind: {kind}\n"));
        if acceptance_differences(&wrong).is_empty() {
            passed.push(kind);
        }
    }
    assert_eq!(
        passed,
        Vec::<&str>::new(),
        "claims on these kinds accept any result and pass the acceptance"
    );
}

// removed: the replay compares no effect class; the acceptance asserts effects (software_change_protocol.rs:248, mutant shown red)

/// Probes the shipped protocol's merge gate along the lines the fixture does not walk. Green:
/// what could not be broken.
#[test]
fn merge_needs_current_tests_and_its_own_grant() {
    let ir = shipped();
    let r1 = record("tests-r1", "test_result", "pass", "implementation", "R1");
    let r2 = record("tests-r2", "test_result", "pass", "implementation", "R2");
    let r2_fail = record(
        "tests-r2-fail",
        "test_result",
        "fail",
        "implementation",
        "R2",
    );
    let granted = "- {capability: repository.merge, decision: granted}\n";

    // Authority granted, tests only for R1: blocked.
    let stale = decide(&ir, &case("R2"), std::slice::from_ref(&r1), Some(granted));
    assert_eq!(action(&stale, "repository.merge"), "blocked");
    // Tests for R2, merge denied: blocked.
    let denied = "- {capability: repository.merge, decision: denied}\n";
    let refused = decide(&ir, &case("R2"), &[r1.clone(), r2.clone()], Some(denied));
    assert_eq!(action(&refused, "repository.merge"), "blocked");
    // Tests for R2, a grant for another capability only: approval-required.
    let other = "- {capability: repository.write, decision: granted}\n";
    let elsewhere = decide(&ir, &case("R2"), &[r1.clone(), r2.clone()], Some(other));
    assert_eq!(action(&elsewhere, "repository.merge"), "approval-required");
    // Tests for R2 both pass and fail: UNKNOWN, blocked even with a grant.
    let split = decide(&ir, &case("R2"), &[r2.clone(), r2_fail], Some(granted));
    assert_eq!(claim(&split, "implementation.verified"), Truth::Unknown);
    assert_eq!(action(&split, "repository.merge"), "blocked");
    // After the grant, the implementation moves to R3: the R2 result is excluded, blocked.
    let moved = decide(&ir, &case("R3"), &[r1, r2], Some(granted));
    assert_eq!(claim(&moved, "tests.pass"), Truth::Unknown);
    assert_eq!(action(&moved, "repository.merge"), "blocked");
}
