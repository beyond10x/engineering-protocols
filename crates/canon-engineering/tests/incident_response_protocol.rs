//! Acceptance for `story:incident-response-protocol`: `incident.response/1` is a Canon
//! `protocol/1` document at `protocols/incident-response/1.yaml`, validated, compiled and
//! evaluated by the same Canon kernel as every engineering protocol, through the fixture harness.
//!
//! The fixture `inc-492` (`fixtures/incident-response/`) is transcribed from
//! `docs/examples/incident-response.md`: the operational incident leaves emergency mode on
//! restoration evidence while its cause is still UNKNOWN. A state may set the case snapshot's
//! artifact revisions: the rollback produces a new revision of the service, and the observations
//! of the old revision no longer apply. The release the rollback does not move is then observed
//! healthy, which says nothing about the service (`story:incident-response-subject-binding`).

mod support;

use std::collections::BTreeSet;

use b10x_canon::ir::Ir;
use b10x_canon::model::{ActionId, ClaimId, Decision, ExclusionReason, Predicate, Truth};
use canon_engineering::vocabulary::{self, Category};
use support::Fixture;

/// The fixture, as a path relative to the repository root.
const INC_492: &str = "fixtures/incident-response/inc-492.fixture.yaml";

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

/// The status Canon's decision gives obligation `id`: `open` or `discharged`.
fn obligation(decision: &Decision, id: &str) -> String {
    let section = decision
        .obligations
        .as_ref()
        .expect("the decision has an obligations section");
    let entries = section.as_array().expect("obligations is an array");
    let entry = entries
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("the decision has no obligation `{id}`: {section}"));
    entry["status"]
        .as_str()
        .expect("an obligation's status is text")
        .to_owned()
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

#[test]
fn inc_492_leaves_emergency_while_cause_unknown() {
    // 1. The protocol validates through Canon and compiles to a `canon-ir/1` document.
    let path = support::protocol_path("incident-response", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));
    let reread = b10x_canon::eval::read_ir(compiled.canon_ir())
        .unwrap_or_else(|refusal| panic!("Canon reads the compiled protocol back: {refusal}"));
    assert_eq!(&reread, compiled.ir());
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "incident.response");

    // Every name the protocol declares is a term of the vocabulary, in its category.
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
        Category::ObligationId,
        ir.obligations.keys().map(|id| id.as_str()).collect(),
    );
    declared(
        Category::ActionId,
        ir.actions.keys().map(|id| id.as_str()).collect(),
    );
    let names = |ids: Vec<&str>| ids.into_iter().map(str::to_owned).collect::<Vec<_>>();
    // The service and the release are declared. Evidence about the release is admissible, and a
    // match bound to the service reads none of it (Canon `EvidenceMatch::subject`); step 5 checks
    // the bindings.
    assert_eq!(
        names(ir.artifacts.keys().map(|id| id.as_str()).collect()),
        ["release", "service"]
    );
    assert_eq!(
        names(ir.claims.keys().map(|id| id.as_str()).collect()),
        ["cause.identified", "impact.bounded", "service.healthy"]
    );
    assert_eq!(
        names(ir.obligations.keys().map(|id| id.as_str()).collect()),
        ["investigate_cause", "restore_service"]
    );
    assert_eq!(
        names(ir.actions.keys().map(|id| id.as_str()).collect()),
        [
            "emergency.leave",
            "logs.search",
            "metrics.inspect",
            "release.inspect",
            "release.rollback",
            "traffic.shift",
        ]
    );

    // The read actions are declared as reads, the authority-gated ones as writes; leaving
    // emergency mode declares no effect class.
    let effect = |action: &str| {
        ir.actions[&ActionId::new(action)]
            .effect
            .as_ref()
            .map(|effect| effect.as_str().to_owned())
    };
    for read in ["metrics.inspect", "logs.search", "release.inspect"] {
        assert_eq!(effect(read).as_deref(), Some("read"), "{read}");
    }
    for gated in ["traffic.shift", "release.rollback"] {
        assert_eq!(effect(gated).as_deref(), Some("write"), "{gated}");
    }
    assert_eq!(effect("emergency.leave"), None);

    // Restoration and investigation progress independently: leaving emergency mode rests on
    // restoration claims and never on `cause.identified`, nor on the cause analysis that
    // establishes it, whether tested as a claim or matched as evidence, directly or through the
    // claims the precondition tests.
    let leave = &ir.actions[&ActionId::new("emergency.leave")];
    let (claims, kinds) = reached(ir, &leave.precondition);
    assert_eq!(claims, ["impact.bounded", "service.healthy"]);
    assert_eq!(kinds, ["impact_assessment", "operational_observation"]);
    assert!(leave.requires.is_empty());

    let fixture = Fixture::load(INC_492).unwrap_or_else(|error| panic!("{INC_492}: {error}"));
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
            "cause-identified",
            "rollback-approved",
            "rolled-back",
            "release-observed",
            "service-restored",
            "cause-identified-after-restore",
        ]
    );
    let decide = |state: &str| {
        fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}` evaluates: {refusal}"))
    };

    // 2. Impact bounded, the service unhealthy, no cause analysis and no authority decision.
    let initial = decide("initial");
    assert_eq!(claim(&initial, "impact.bounded"), Truth::True);
    assert_eq!(claim(&initial, "service.healthy"), Truth::False);
    assert_eq!(claim(&initial, "cause.identified"), Truth::Unknown);
    assert_eq!(obligation(&initial, "restore_service"), "open");
    for read in ["metrics.inspect", "logs.search", "release.inspect"] {
        assert_eq!(action(&initial, read), "admissible", "{read}");
    }
    for gated in ["traffic.shift", "release.rollback"] {
        assert_eq!(action(&initial, gated), "approval-required", "{gated}");
    }
    assert_eq!(action(&initial, "emergency.leave"), "blocked");

    // The cause is identified while the service is still unhealthy: the investigation's progress
    // does not leave emergency mode or discharge the restoration.
    let cause = decide("cause-identified");
    assert_eq!(claim(&cause, "cause.identified"), Truth::True);
    assert_eq!(claim(&cause, "service.healthy"), Truth::False);
    assert_eq!(obligation(&cause, "restore_service"), "open");
    assert_eq!(action(&cause, "emergency.leave"), "blocked");

    // 3. The rollback is approved; the service is still unhealthy and emergency mode holds.
    let approved = decide("rollback-approved");
    assert_eq!(action(&approved, "release.rollback"), "admissible");
    assert_eq!(claim(&approved, "service.healthy"), Truth::False);
    assert_eq!(action(&approved, "emergency.leave"), "blocked");

    // The rollback produces a new revision of the service, of which nothing is observed yet. The
    // records of the old revision no longer apply, so the service's health is UNKNOWN, not FALSE
    // and not TRUE: the restoration stays open and emergency mode holds.
    let rolled_back = decide("rolled-back");
    assert_eq!(claim(&rolled_back, "service.healthy"), Truth::Unknown);
    assert_eq!(obligation(&rolled_back, "restore_service"), "open");
    assert_eq!(action(&rolled_back, "emergency.leave"), "blocked");
    let old_revision = [
        ("cause.identified", "cause-1"),
        ("impact.bounded", "impact-1"),
        ("service.healthy", "health-1"),
    ];
    for (claim, evidence) in old_revision {
        assert_eq!(
            excluded(&rolled_back, claim),
            [(evidence.to_owned(), ExclusionReason::RevisionMismatch)],
            "{claim}"
        );
    }

    // The release r42, which the rollback did not move, is observed healthy and its impact
    // assessed bounded, both at its current revision. Those records are about the release, not
    // the service: they move no claim about the service, so the restoration stays open and
    // emergency mode holds. They are not excluded either; they are simply not the service's.
    let observed = decide("release-observed");
    for about_service in ["impact.bounded", "service.healthy"] {
        assert_eq!(
            claim(&observed, about_service),
            claim(&rolled_back, about_service),
            "a record about the release moved `{about_service}`"
        );
    }
    assert_eq!(obligation(&observed, "restore_service"), "open");
    assert_eq!(action(&observed, "emergency.leave"), "blocked");
    for (claim, evidence) in old_revision {
        assert_eq!(
            excluded(&observed, claim),
            [(evidence.to_owned(), ExclusionReason::RevisionMismatch)],
            "{claim}"
        );
    }

    // 4. A healthy observation of the new revision arrives, with the impact assessed again on it
    //    and no cause analysis of it. The records of the old revision stay excluded.
    let restored = decide("service-restored");
    assert_eq!(claim(&restored, "service.healthy"), Truth::True);
    assert_eq!(claim(&restored, "impact.bounded"), Truth::True);
    assert_eq!(claim(&restored, "cause.identified"), Truth::Unknown);
    assert_eq!(obligation(&restored, "restore_service"), "discharged");
    assert_eq!(action(&restored, "emergency.leave"), "admissible");
    for (claim, evidence) in old_revision {
        assert_eq!(
            excluded(&restored, claim),
            [(evidence.to_owned(), ExclusionReason::RevisionMismatch)],
            "{claim}"
        );
    }

    // 5. Every evidence match names the artifact its kind is about: an impact assessment and an
    //    operational observation are about the service. A cause analysis names no subject: the
    //    design gives the kind none, and `release.inspect` produces one from a release.
    let mut bindings = BTreeSet::new();
    let mut collect = |predicate: &Predicate| {
        predicate.visit(&mut |node| {
            if let Predicate::Evidence(matching) = node {
                bindings.insert((
                    matching.kind.as_str().to_owned(),
                    matching.subject.as_ref().map(|id| id.as_str().to_owned()),
                ));
            }
        });
    };
    ir.claims
        .values()
        .for_each(|claim| collect(&claim.true_when));
    ir.obligations
        .values()
        .for_each(|obligation| collect(&obligation.discharged_when));
    ir.actions
        .values()
        .for_each(|action| collect(&action.precondition));
    ir.outcomes
        .values()
        .filter_map(|outcome| outcome.requires.predicate())
        .for_each(&mut collect);
    let service = Some("service".to_owned());
    assert_eq!(
        bindings.into_iter().collect::<Vec<_>>(),
        [
            ("cause_analysis".to_owned(), None),
            ("impact_assessment".to_owned(), service.clone()),
            ("operational_observation".to_owned(), service),
        ]
    );

    // The fixture's own expectations are the ones above.
    assert_eq!(fixture.check(&compiled), Ok(()));
}

/// The ids of the obligations Canon's decision lists, in the order it lists them.
fn obligation_ids(decision: &Decision) -> Vec<String> {
    decision
        .obligations
        .as_ref()
        .expect("the decision has an obligations section")
        .as_array()
        .expect("obligations is an array")
        .iter()
        .map(|entry| {
            entry["id"]
                .as_str()
                .expect("an obligation's id is text")
                .to_owned()
        })
        .collect()
}

/// Acceptance for `story:incident-investigation-obligation`: the investigation of the cause is an
/// obligation of its own, `investigate_cause`, discharged when `cause.identified` is TRUE. It
/// progresses independently of `restore_service`, and stays open after emergency mode may be left,
/// until a cause analysis of a current revision of the service or the release identifies the cause.
#[test]
fn inc_492_investigation_stays_open_after_emergency_leave() {
    let compiled = support::compile_protocol(&support::protocol_path("incident-response", 1))
        .unwrap_or_else(|error| panic!("{error}"));
    let ir = compiled.ir();

    // The obligation is a vocabulary term in the obligation category, and the protocol declares
    // it with its own discharge condition: `cause.identified` is TRUE.
    let term = vocabulary::lookup("investigate_cause").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(term.category, Category::ObligationId);
    let investigate = ir
        .obligations
        .iter()
        .find(|(id, _)| id.as_str() == "investigate_cause")
        .map(|(_, obligation)| obligation)
        .expect("the protocol declares `investigate_cause`");
    assert_eq!(
        reached(ir, &investigate.discharged_when),
        (
            vec!["cause.identified".to_owned()],
            vec!["cause_analysis".to_owned()]
        )
    );

    // Leaving emergency mode still never rests on the cause.
    let (claims, kinds) = reached(
        ir,
        &ir.actions[&ActionId::new("emergency.leave")].precondition,
    );
    assert_eq!(claims, ["impact.bounded", "service.healthy"]);
    assert_eq!(kinds, ["impact_assessment", "operational_observation"]);

    let fixture = Fixture::load(INC_492).unwrap_or_else(|error| panic!("{INC_492}: {error}"));
    let decide = |state: &str| {
        let decision = fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}` evaluates: {refusal}"));
        // Every state's decision lists exactly the two obligations.
        let mut ids = obligation_ids(&decision);
        ids.sort();
        assert_eq!(ids, ["investigate_cause", "restore_service"], "{state}");
        decision
    };

    // 1. Nothing is known of the cause: both obligations are open.
    let initial = decide("initial");
    assert_eq!(claim(&initial, "cause.identified"), Truth::Unknown);
    assert_eq!(obligation(&initial, "investigate_cause"), "open");
    assert_eq!(obligation(&initial, "restore_service"), "open");

    // 2. A cause analysis of revision s1 discharges the investigation and nothing else: the
    //    restoration stays open and emergency mode holds.
    let cause = decide("cause-identified");
    assert_eq!(claim(&cause, "cause.identified"), Truth::True);
    assert_eq!(obligation(&cause, "investigate_cause"), "discharged");
    assert_eq!(obligation(&cause, "restore_service"), "open");
    assert_eq!(action(&cause, "emergency.leave"), "blocked");

    // 3. The rollback moves the service to s2. The analysis of s1 no longer applies, so the cause
    //    is UNKNOWN again and the investigation reopens.
    let rolled_back = decide("rolled-back");
    assert_eq!(claim(&rolled_back, "cause.identified"), Truth::Unknown);
    assert_eq!(
        excluded(&rolled_back, "cause.identified"),
        [("cause-1".to_owned(), ExclusionReason::RevisionMismatch)]
    );
    assert_eq!(obligation(&rolled_back, "investigate_cause"), "open");

    // 4. The service is restored on s2: the restoration is discharged and emergency mode may be
    //    left, while the investigation stays open.
    let restored = decide("service-restored");
    assert_eq!(claim(&restored, "cause.identified"), Truth::Unknown);
    assert_eq!(obligation(&restored, "restore_service"), "discharged");
    assert_eq!(action(&restored, "emergency.leave"), "admissible");
    assert_eq!(obligation(&restored, "investigate_cause"), "open");

    // 5. A cause analysis of the service at s2 arrives after the restoration: the investigation is
    //    discharged, and the restoration and the admissible exit are unchanged.
    let after = decide("cause-identified-after-restore");
    assert_eq!(claim(&after, "cause.identified"), Truth::True);
    assert_eq!(
        excluded(&after, "cause.identified"),
        [("cause-1".to_owned(), ExclusionReason::RevisionMismatch)]
    );
    assert_eq!(obligation(&after, "investigate_cause"), "discharged");
    assert_eq!(obligation(&after, "restore_service"), "discharged");
    assert_eq!(action(&after, "emergency.leave"), "admissible");

    // The fixture's own expectations agree.
    assert_eq!(fixture.check(&compiled), Ok(()));
}

/// The fixture's text with `from` replaced by `to` once; `from` must occur in it.
fn inc_492_with(from: &str, to: &str) -> String {
    let text = std::fs::read_to_string(support::repo_root().join(INC_492)).expect("inc-492 reads");
    assert!(text.contains(from), "inc-492 contains `{from}`");
    text.replacen(from, to, 1)
}

/// The harness compares the obligations and actions a state expects with Canon's decision, both
/// ways, and passes the authority decisions to Canon, which refuses one it cannot read.
#[test]
fn inc_492_harness_reports_obligation_and_action_differences() {
    let compiled = support::compile_protocol(&support::protocol_path("incident-response", 1))
        .unwrap_or_else(|error| panic!("{error}"));
    for (from, to, expected) in [
        (
            "        restore_service: discharged\n",
            "        restore_service: open\n",
            vec![
                "state `service-restored`: obligation `restore_service` is discharged, expected open",
            ],
        ),
        (
            "        release.rollback: admissible\n",
            "        release.rollback: approval-required\n",
            vec![
                "state `rollback-approved`: action `release.rollback` is admissible, expected \
                 approval-required",
            ],
        ),
        (
            "        traffic.shift: approval-required\n",
            "",
            vec!["state `initial`: action `traffic.shift` is approval-required, expected nothing"],
        ),
        (
            "        emergency.leave: blocked\n",
            "        emergency.leave: blocked\n        emergency.enter: blocked\n",
            vec!["state `initial`: action `emergency.enter` is not declared, expected blocked"],
        ),
        (
            "decision: granted",
            "decision: maybe",
            vec![
                "state `rollback-approved`: evaluation refused: malformed-input: ",
                "state `rolled-back`: evaluation refused: malformed-input: ",
                "state `release-observed`: evaluation refused: malformed-input: ",
                "state `service-restored`: evaluation refused: malformed-input: ",
                "state `cause-identified-after-restore`: evaluation refused: malformed-input: ",
            ],
        ),
    ] {
        let fixture =
            Fixture::from_yaml(&inc_492_with(from, to)).unwrap_or_else(|error| panic!("{error}"));
        let found = fixture
            .check(&compiled)
            .expect_err(&format!("a fixture with `{to}` differs"));
        assert_eq!(found.len(), expected.len(), "{found:?}");
        for (line, prefix) in found.iter().zip(&expected) {
            assert!(line.starts_with(prefix), "`{line}`, expected `{prefix}`");
        }
    }
}

/// `set_revisions` and `add_authority` are refused, not dropped, when they cannot be applied.
#[test]
fn inc_492_harness_refuses_state_inputs_it_cannot_apply() {
    for (from, to, refusal) in [
        (
            "      service: s2\n",
            "      incident: s2\n",
            "state `rolled-back`: sets the revision of artifact `incident`, which the case \
             does not list",
        ),
        (
            "      service: s2\n",
            "      service: 2\n",
            "state `rolled-back`: case: malformed-input: ",
        ),
        (
            "      service: s2\n",
            "      service: s2\n      service: s3\n",
            "artifact `service` is set more than once",
        ),
        (
            "    set_revisions:\n      service: s2\n",
            "    set_revisions:\n",
            "a key is written with no value",
        ),
        (
            "    add_authority: []\n",
            "    add_authority:\n",
            "a key is written with no value",
        ),
        (
            "        restore_service: open\n",
            "        restore_service: open\n        restore_service: open\n",
            "`restore_service` is expected more than once",
        ),
    ] {
        let error = match Fixture::from_yaml(&inc_492_with(from, to)) {
            Ok(_) => panic!("a fixture with `{to}` loads; expected `{refusal}`"),
            Err(error) => error.to_string(),
        };
        assert!(
            error.contains(refusal),
            "a fixture with `{to}` is refused as `{error}`, expected `{refusal}`"
        );
    }
}
