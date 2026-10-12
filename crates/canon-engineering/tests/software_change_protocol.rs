//! Acceptance for `story:software-change-protocol`: `software.change/1` is a Canon `protocol/1`
//! document at `protocols/software-change/1.yaml`, validated, compiled and evaluated by the same
//! Canon kernel as every engineering protocol, through the fixture harness.
//!
//! The fixture `chg-1842` (`fixtures/software-change/`) is transcribed from
//! `docs/examples/software-change.md`: a passing test result for implementation revision R1 says
//! nothing about the current revision R2, so the tests' passing is UNKNOWN, not FALSE, and merging
//! waits for a passing test result bound to R2 and then for an authority decision.

mod support;

use std::collections::BTreeSet;

use b10x_canon::ir::Ir;
use b10x_canon::model::{
    ActionId, ClaimId, Decision, ExclusionReason, OutcomeId, Predicate, Truth,
};
use canon_engineering::vocabulary::{self, Category};
use support::Fixture;

/// The fixture, as a path relative to the repository root.
const CHG_1842: &str = "fixtures/software-change/chg-1842.fixture.yaml";

/// The value Canon's decision gives `claim`.
fn claim(decision: &Decision, claim: &str) -> Truth {
    decision
        .claims
        .get(&ClaimId::new(claim))
        .unwrap_or_else(|| panic!("the decision has no claim `{claim}`"))
        .value
}

/// The evidence Canon's decision lists as excluded from `claim`, and why, in evidence-id order.
fn excluded(decision: &Decision, claim: &str) -> Vec<(String, ExclusionReason)> {
    decision
        .claims
        .get(&ClaimId::new(claim))
        .unwrap_or_else(|| panic!("the decision has no claim `{claim}`"))
        .excluded_evidence
        .iter()
        .map(|exclusion| (exclusion.evidence.as_str().to_owned(), exclusion.reason))
        .collect()
}

/// Every claim and every evidence kind `predicate` rests on, each sorted: the claims it tests and
/// the kinds it matches, and, through any number of claim tests, those of each tested claim's
/// `true_when`.
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
            Predicate::All(_) | Predicate::Any(_) | Predicate::Not(_) | Predicate::Input(_) => {}
        });
    }
    (claims.into_iter().collect(), kinds.into_iter().collect())
}

/// The status Canon's decision gives action `id`: `admissible`, `approval-required` or `blocked`.
fn action(decision: &Decision, id: &str) -> String {
    let section = decision
        .actions
        .as_ref()
        .expect("the decision has an actions section");
    section[id]["status"]
        .as_str()
        .unwrap_or_else(|| panic!("the decision has no action `{id}`: {section}"))
        .to_owned()
}

/// The status Canon's decision gives outcome `id`: `legitimate` or `blocked`.
fn outcome(decision: &Decision, id: &str) -> String {
    let section = decision
        .outcomes
        .as_ref()
        .expect("the decision has an outcomes section");
    section[id]["status"]
        .as_str()
        .unwrap_or_else(|| panic!("the decision has no outcome `{id}`: {section}"))
        .to_owned()
}

/// The claim reasons Canon's decision gives a blocked outcome `id`, each `<claim>=<value>`.
fn outcome_reasons(decision: &Decision, id: &str) -> Vec<String> {
    let section = decision
        .outcomes
        .as_ref()
        .expect("the decision has an outcomes section");
    section[id]["reasons"]
        .as_array()
        .unwrap_or_else(|| panic!("outcome `{id}` gives no reasons: {section}"))
        .iter()
        .map(|reason| {
            let claim = reason["claim"]
                .as_str()
                .unwrap_or_else(|| panic!("a reason that is not a claim: {reason}"));
            let value = reason["value"].as_str().expect("a claim reason's value");
            format!("{claim}={value}")
        })
        .collect()
}

/// The actions that need neither a claim nor authority.
const OPEN_ACTIONS: [&str; 3] = ["repository.inspect", "repository.edit", "tests.run"];

#[test]
fn chg_1842_merge_waits_for_current_revision_tests_and_authority() {
    // 1. The protocol validates through Canon and compiles to a `canon-ir/1` document.
    let path = support::protocol_path("software-change", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));
    let reread = b10x_canon::eval::read_ir(compiled.canon_ir())
        .unwrap_or_else(|refusal| panic!("Canon reads the compiled protocol back: {refusal}"));
    assert_eq!(&reread, compiled.ir());
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "software.change");
    assert_eq!(ir.protocol.revision, 1);

    // Every name the protocol declares is a term of the vocabulary, in its category, the names of
    // the explicit decisions its outcomes require included.
    let declared = |category: Category, names: Vec<&str>| {
        for name in names {
            let term = vocabulary::lookup(name).unwrap_or_else(|error| panic!("{error}"));
            assert_eq!(term.category, category, "`{name}`");
        }
    };
    declared(
        Category::ArtifactKind,
        ir.artifacts.keys().map(|id| id.as_str()).collect(),
    );
    declared(
        Category::EvidenceKind,
        ir.evidence_kinds.keys().map(|id| id.as_str()).collect(),
    );
    declared(
        Category::ClaimId,
        ir.claims.keys().map(|id| id.as_str()).collect(),
    );
    declared(
        Category::ActionId,
        ir.actions.keys().map(|id| id.as_str()).collect(),
    );
    declared(
        Category::OutcomeId,
        ir.outcomes.keys().map(|id| id.as_str()).collect(),
    );
    declared(
        Category::DecisionName,
        ir.outcomes
            .values()
            .filter_map(|outcome| outcome.requires.decision())
            .map(|name| name.as_str())
            .collect(),
    );
    let names = |ids: Vec<&str>| ids.into_iter().map(str::to_owned).collect::<Vec<_>>();
    assert_eq!(
        names(ir.artifacts.keys().map(|id| id.as_str()).collect()),
        [
            "deployment",
            "implementation",
            "intent",
            "plan",
            "release",
            "system_specification",
        ]
    );
    assert_eq!(
        names(ir.evidence_kinds.keys().map(|id| id.as_str()).collect()),
        [
            "build_provenance",
            "code_review",
            "objective_observation",
            "operational_observation",
            "rollback_result",
            "test_result",
        ]
    );
    assert_eq!(
        names(ir.claims.keys().map(|id| id.as_str()).collect()),
        [
            "deployment.healthy",
            "implementation.reviewed",
            "implementation.verified",
            "objective.realized",
            "release.proven",
            "tests.pass",
        ]
    );
    assert!(ir.obligations.is_empty());
    assert_eq!(
        names(ir.actions.keys().map(|id| id.as_str()).collect()),
        [
            "release.rollback",
            "repository.edit",
            "repository.inspect",
            "repository.merge",
            "tests.run",
        ]
    );
    assert_eq!(
        names(ir.outcomes.keys().map(|id| id.as_str()).collect()),
        [
            "abandoned",
            "accepted",
            "declined",
            "rolled_back",
            "superseded",
        ]
    );

    // `implementation.verified` is a conjunction whose one conjunct, for now, is a passing test
    // result; later stories each add one.
    let verified = &ir.claims[&ClaimId::new("implementation.verified")].true_when;
    let Predicate::All(conjuncts) = verified else {
        panic!("implementation.verified is a conjunction: {verified:?}");
    };
    assert_eq!(conjuncts.len(), 1, "{conjuncts:?}");
    let Predicate::Evidence(matching) = &conjuncts[0] else {
        panic!("its conjunct is an evidence match: {conjuncts:?}");
    };
    assert_eq!(matching.kind.as_str(), "test_result");
    assert_eq!(matching.result.as_deref(), Some("pass"));

    // Merging needs authority, and it rests on the verified implementation and through it on
    // test results. Rolling back a release needs authority and nothing else, and may produce a
    // rollback result. The other actions need nothing.
    let merge = &ir.actions[&ActionId::new("repository.merge")];
    let capabilities: Vec<&str> = merge.requires.iter().map(|id| id.as_str()).collect();
    assert_eq!(capabilities, ["repository.merge"]);
    let (claims, kinds) = reached(ir, &merge.precondition);
    assert_eq!(claims, ["implementation.verified"]);
    assert_eq!(kinds, ["test_result"]);
    let rollback = &ir.actions[&ActionId::new("release.rollback")];
    let capabilities: Vec<&str> = rollback.requires.iter().map(|id| id.as_str()).collect();
    assert_eq!(capabilities, ["release.rollback"]);
    assert_eq!(rollback.precondition, Predicate::All(Vec::new()));
    let produces: Vec<&str> = rollback.may_produce.iter().map(|id| id.as_str()).collect();
    assert_eq!(produces, ["rollback_result"]);
    for open in OPEN_ACTIONS {
        let declared = &ir.actions[&ActionId::new(open)];
        assert!(declared.requires.is_empty(), "{open}");
        assert_eq!(declared.precondition, Predicate::All(Vec::new()), "{open}");
    }
    let produces: Vec<&str> = ir.actions[&ActionId::new("tests.run")]
        .may_produce
        .iter()
        .map(|id| id.as_str())
        .collect();
    assert_eq!(produces, ["test_result"]);

    // Reading the repository and running the tests are reads; editing, merging and rolling back
    // are writes.
    let effect = |action: &str| {
        ir.actions[&ActionId::new(action)]
            .effect
            .as_ref()
            .map(|effect| effect.as_str().to_owned())
    };
    for read in ["repository.inspect", "tests.run"] {
        assert_eq!(effect(read).as_deref(), Some("read"), "{read}");
    }
    for write in ["repository.edit", "repository.merge", "release.rollback"] {
        assert_eq!(effect(write).as_deref(), Some("write"), "{write}");
    }

    // `accepted` needs every one of a realized objective, a healthy deployment and a proven
    // release (design § 9 reaches `accepted` only through `candidate` and `released`).
    let accepted = ir.outcomes[&OutcomeId::new("accepted")]
        .requires
        .predicate()
        .expect("accepted requires a predicate, not an explicit decision");
    let Predicate::All(required) = accepted else {
        panic!("accepted requires a conjunction: {accepted:?}");
    };
    let mut tested: Vec<(&str, Truth)> = required
        .iter()
        .map(|member| match member {
            Predicate::Claim(test) => (test.claim.as_str(), test.is),
            other => panic!("accepted requires claim tests only: {other:?}"),
        })
        .collect();
    tested.sort();
    assert_eq!(
        tested,
        [
            ("deployment.healthy", Truth::True),
            ("objective.realized", Truth::True),
            ("release.proven", Truth::True),
        ]
    );
    let (claims, kinds) = reached(ir, accepted);
    assert_eq!(
        claims,
        [
            "deployment.healthy",
            "implementation.verified",
            "objective.realized",
            "release.proven",
        ]
    );
    assert_eq!(
        kinds,
        [
            "build_provenance",
            "objective_observation",
            "operational_observation",
            "test_result",
        ]
    );

    let fixture = Fixture::load(CHG_1842).unwrap_or_else(|error| panic!("{CHG_1842}: {error}"));
    assert_eq!(fixture.protocol_path(), path);
    assert_eq!(fixture.at(), "2026-10-04T12:00:00Z");
    let compiled_again = fixture
        .compile()
        .unwrap_or_else(|error| panic!("the fixture's protocol compiles: {error}"));
    assert_eq!(compiled_again.canon_ir(), compiled.canon_ir());
    assert_eq!(
        fixture.states(),
        [
            "initial",
            "tests-pass-r2",
            "merge-approved",
            "review-rejected",
            "objective-unmet",
            "deployment-unhealthy",
            "tests-fail-r2",
        ]
    );
    let decide = |state: &str| {
        fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}` evaluates: {refusal}"))
    };
    // The R1 test result is excluded from every claim that reaches test results, in every state:
    // the case's implementation revision is R2.
    let r1_excluded = |decision: &Decision| {
        for reaching in ["tests.pass", "implementation.verified", "release.proven"] {
            assert_eq!(
                excluded(decision, reaching),
                [("tests-r1".to_owned(), ExclusionReason::RevisionMismatch)],
                "{reaching}"
            );
        }
    };

    // 2. Implementation revision R2, one passing test result bound to R1, no authority decision.
    //    The running deployment is observed healthy; that alone does not accept the change.
    let initial = decide("initial");
    assert_eq!(claim(&initial, "tests.pass"), Truth::Unknown);
    assert_eq!(claim(&initial, "implementation.verified"), Truth::Unknown);
    r1_excluded(&initial);
    for open in OPEN_ACTIONS {
        assert_eq!(action(&initial, open), "admissible", "{open}");
    }
    assert_eq!(action(&initial, "repository.merge"), "blocked");
    assert_eq!(claim(&initial, "deployment.healthy"), Truth::True);
    assert_eq!(claim(&initial, "objective.realized"), Truth::Unknown);
    assert_eq!(outcome(&initial, "accepted"), "blocked");
    assert_eq!(
        outcome_reasons(&initial, "accepted"),
        ["objective.realized=unknown", "release.proven=unknown"]
    );

    // 3. A passing test result bound to R2 arrives; merging now waits only for authority.
    let tested = decide("tests-pass-r2");
    assert_eq!(claim(&tested, "tests.pass"), Truth::True);
    assert_eq!(claim(&tested, "implementation.verified"), Truth::True);
    r1_excluded(&tested);
    for open in OPEN_ACTIONS {
        assert_eq!(action(&tested, open), "admissible", "{open}");
    }
    assert_eq!(action(&tested, "repository.merge"), "approval-required");

    // 4. An authority decision approves merging.
    let approved = decide("merge-approved");
    assert_eq!(claim(&approved, "implementation.verified"), Truth::True);
    assert_eq!(action(&approved, "repository.merge"), "admissible");
    assert_eq!(outcome(&approved, "accepted"), "blocked");

    // Each result-qualified claim is decided by its evidence's result, not by its presence: a
    // rejecting review, an unmet objective, an unhealthy observation beside a healthy one and a
    // failing test result beside a passing one never make their claim TRUE.
    let rejected = decide("review-rejected");
    assert_eq!(claim(&rejected, "implementation.reviewed"), Truth::False);
    let unmet = decide("objective-unmet");
    assert_eq!(claim(&unmet, "objective.realized"), Truth::False);
    assert_eq!(claim(&unmet, "deployment.healthy"), Truth::True);
    assert_eq!(outcome(&unmet, "accepted"), "blocked");
    let unhealthy = decide("deployment-unhealthy");
    assert_eq!(claim(&unhealthy, "deployment.healthy"), Truth::Unknown);
    let failing = decide("tests-fail-r2");
    assert_eq!(claim(&failing, "tests.pass"), Truth::Unknown);
    assert_eq!(claim(&failing, "implementation.verified"), Truth::Unknown);
    r1_excluded(&failing);
    // The grant stands, but the current revision is no longer verified.
    assert_eq!(action(&failing, "repository.merge"), "blocked");
    for state in [&rejected, &unmet, &unhealthy, &failing] {
        assert_eq!(outcome(state, "accepted"), "blocked");
    }

    // The fixture's own expectations are the ones above.
    assert_eq!(fixture.check(&compiled), Ok(()));
}
