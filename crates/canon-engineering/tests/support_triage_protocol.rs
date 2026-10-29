//! Acceptance for `support.triage/1`: a Canon `protocol/1` document at
//! `protocols/support-triage/1.yaml`, validated, compiled, checked over its whole state space and
//! evaluated by the same Canon kernel as every protocol here, through the fixture harness.
//!
//! The protocol is tool-agnostic: its actions name what is done to a ticket, never the tracker that
//! holds it; which connection serves an action is a runtime binding outside the protocol (decision
//! record 0083). Three fixtures under `fixtures/support-triage/` carry its scenarios:
//!
//! - `tri-48213`: a ticket read at its current revision, its requester identified, a proposed
//!   classification reviewed and approved, then routed under authority: `triaged`. A read of an
//!   earlier revision says nothing, and when the requester writes again everything bound to the
//!   ticket is UNKNOWN until it is read again.
//! - `tri-48214`: a breached service-level target opens the escalation obligation; an escalation
//!   requested under authority discharges it: `escalated`.
//! - `tri-48215`: a rejected classification and an unidentified requester: `needs_human`, and
//!   routing stays blocked.
//!
//! The ticket read expires after 15 minutes (`max_age`). The harness passes each state's instant
//! to Canon, and each read of a ticket's current revision in the fixtures is fresh at its state's
//! instant; the expiry itself is evaluated here directly with Canon's evaluator.

mod support;

use std::collections::BTreeSet;

use b10x_canon::ir::Ir;
use b10x_canon::model::{
    ActionId, ClaimId, Decision, ExclusionReason, Instant, OutcomeId, Predicate, Truth,
};
use support::Fixture;

const TRI_48213: &str = "fixtures/support-triage/tri-48213.fixture.yaml";
const TRI_48214: &str = "fixtures/support-triage/tri-48214.fixture.yaml";
const TRI_48215: &str = "fixtures/support-triage/tri-48215.fixture.yaml";

const OUTCOMES: [&str; 3] = ["escalated", "needs_human", "triaged"];

fn claim(decision: &Decision, claim: &str) -> Truth {
    decision
        .claims
        .get(&ClaimId::new(claim))
        .unwrap_or_else(|| panic!("the decision has no claim `{claim}`"))
        .value
}

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

fn obligation(decision: &Decision, id: &str) -> String {
    let section = decision
        .obligations
        .as_ref()
        .expect("the decision has an obligations section");
    section
        .as_array()
        .expect("obligations is an array")
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("the decision has no obligation `{id}`: {section}"))["status"]
        .as_str()
        .expect("an obligation's status is text")
        .to_owned()
}

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

/// The outcomes legitimate in `decision`, in identifier order.
fn legitimate(decision: &Decision) -> Vec<&'static str> {
    OUTCOMES
        .into_iter()
        .filter(|id| outcome(decision, id) == "legitimate")
        .collect()
}

/// Every claim and evidence kind `predicate` rests on, through any number of claim tests.
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
            Predicate::First(_) | Predicate::Before(_) | Predicate::After(_) => {
                for (kind, _) in node.order_reads() {
                    kinds.insert(kind.as_str().to_owned());
                }
            }
            Predicate::All(_) | Predicate::Any(_) | Predicate::Not(_) | Predicate::Input(_) => {}
        });
    }
    (claims.into_iter().collect(), kinds.into_iter().collect())
}

fn keys<'a, K: 'a + AsRef<str>>(ids: impl Iterator<Item = &'a K>) -> Vec<String> {
    ids.map(|id| id.as_ref().to_owned()).collect()
}

fn load(path: &str) -> Fixture {
    let fixture = Fixture::load(path).unwrap_or_else(|error| panic!("{path}: {error}"));
    assert_eq!(fixture.protocol_path(), "protocols/support-triage/1.yaml");
    fixture
}

#[test]
fn support_triage_declares_tool_agnostic_actions_in_step_order() {
    let path = support::protocol_path("support-triage", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));
    let reread = b10x_canon::eval::read_ir(compiled.canon_ir())
        .unwrap_or_else(|refusal| panic!("Canon reads the compiled protocol back: {refusal}"));
    assert_eq!(&reread, compiled.ir());
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "support.triage");
    assert_eq!(ir.protocol.revision, 1);

    assert_eq!(keys(ir.artifacts.keys()), ["requester", "ticket"]);
    assert_eq!(
        keys(ir.evidence_kinds.keys()),
        [
            "classification",
            "classification_review",
            "escalation_record",
            "requester_profile",
            "routing_record",
            "ticket_snapshot",
        ]
    );
    assert_eq!(
        keys(ir.claims.keys()),
        [
            "classification.proposed",
            "classification.reviewed",
            "escalation.requested",
            "requester.known",
            "sla.breached",
            "ticket.classified",
            "ticket.current",
            "ticket.routed",
            "ticket.triaged",
        ]
    );
    assert_eq!(keys(ir.obligations.keys()), ["escalate_sla_breach"]);
    assert_eq!(
        keys(ir.actions.keys()),
        [
            "classification.review",
            "escalation.request",
            "reply.draft",
            "requester.lookup",
            "ticket.classify",
            "ticket.read",
            "ticket.route",
        ]
    );
    assert_eq!(keys(ir.outcomes.keys()), OUTCOMES);

    // Reads need no authority; the review and every write need a capability of their own name.
    let act = |id: &str| &ir.actions[&ActionId::new(id)];
    let effect = |id: &str| act(id).effect.as_ref().map(|e| e.as_str().to_owned());
    let capabilities = |id: &str| -> Vec<String> {
        act(id)
            .requires
            .iter()
            .map(|c| c.as_str().to_owned())
            .collect()
    };
    for read in ["ticket.read", "requester.lookup"] {
        assert_eq!(effect(read).as_deref(), Some("read"), "{read}");
        assert!(capabilities(read).is_empty(), "{read}");
    }
    assert_eq!(effect("ticket.classify").as_deref(), Some("none"));
    assert!(capabilities("ticket.classify").is_empty());
    assert_eq!(effect("classification.review").as_deref(), Some("none"));
    for governed in [
        "classification.review",
        "ticket.route",
        "reply.draft",
        "escalation.request",
    ] {
        assert_eq!(capabilities(governed), [governed], "{governed}");
    }
    for write in ["ticket.route", "reply.draft", "escalation.request"] {
        assert_eq!(effect(write).as_deref(), Some("write"), "{write}");
    }

    // Step order (decision record 0084, by precondition): read, then classify, then review, then
    // route. Routing rests on the read, the classification, its review and the requester.
    let (claims, kinds) = reached(ir, &act("ticket.route").precondition);
    assert_eq!(
        claims,
        [
            "classification.proposed",
            "classification.reviewed",
            "requester.known",
            "ticket.classified",
            "ticket.current",
        ]
    );
    assert_eq!(
        kinds,
        [
            "classification",
            "classification_review",
            "requester_profile",
            "ticket_snapshot",
        ]
    );
    assert_eq!(
        reached(ir, &act("classification.review").precondition).0,
        ["classification.proposed"]
    );
    assert_eq!(
        reached(ir, &act("ticket.classify").precondition).0,
        ["ticket.current"]
    );
    assert_eq!(act("ticket.read").precondition, Predicate::All(Vec::new()));

    // Every evidence match is bound to the artifact it is about.
    for (id, declared) in &ir.claims {
        declared.true_when.visit(&mut |node| {
            if let Predicate::Evidence(matching) = node {
                assert!(
                    matching.subject.is_some(),
                    "claim `{id}` reads unbound evidence"
                );
            }
        });
    }

    // `triaged` rests on governed evidence only an action under authority produces.
    let triaged = ir.outcomes[&OutcomeId::new("triaged")]
        .requires
        .predicate()
        .expect("triaged requires a predicate");
    let (_, kinds) = reached(ir, triaged);
    assert!(kinds.contains(&"routing_record".to_owned()), "{kinds:?}");
    assert!(
        kinds.contains(&"classification_review".to_owned()),
        "{kinds:?}"
    );
}

#[test]
fn canon_check_finds_no_unreachable_outcome_and_no_authority_bypass() {
    let path = support::protocol_path("support-triage", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));
    let report = b10x_canon::check::check(compiled.ir(), None)
        .unwrap_or_else(|refusal| panic!("canon check runs: {refusal}"));
    assert!(report.is_clean(), "canon check found:\n{report}");
    assert!(report.states <= b10x_canon::check::STATE_BOUND);
}

#[test]
fn tri_48213_is_triaged_on_current_evidence_under_authority() {
    let fixture = load(TRI_48213);
    assert_eq!(fixture.at(), "2026-10-05T09:30:00Z");
    let compiled = fixture.compile().expect("the protocol compiles");
    assert_eq!(fixture.check(&compiled), Ok(()));
    assert_eq!(
        fixture.states(),
        [
            "initial",
            "ticket-read",
            "requester-identified",
            "classification-proposed",
            "review-granted",
            "classification-approved",
            "route-granted",
            "routed",
            "requester-wrote-again",
        ]
    );
    let decide = |state: &str| {
        fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}` evaluates: {refusal}"))
    };

    // The read of revision c2 says nothing about c3: it was breached, the current read is not.
    let read = decide("ticket-read");
    assert_eq!(claim(&read, "sla.breached"), Truth::False);
    assert_eq!(
        excluded(&read, "sla.breached"),
        [("snapshot-c2".to_owned(), ExclusionReason::RevisionMismatch)]
    );
    assert_eq!(obligation(&read, "escalate_sla_breach"), "discharged");

    // Classify comes before route: a proposed, unreviewed classification does not open routing.
    let proposed = decide("classification-proposed");
    assert_eq!(action(&proposed, "ticket.route"), "blocked");
    assert_eq!(
        action(&proposed, "classification.review"),
        "approval-required"
    );

    // Reviewed and approved, routing and the reply draft wait for authority only.
    let approved = decide("classification-approved");
    assert_eq!(claim(&approved, "ticket.classified"), Truth::True);
    assert_eq!(action(&approved, "ticket.route"), "approval-required");
    assert_eq!(action(&approved, "reply.draft"), "approval-required");
    assert!(legitimate(&approved).is_empty());

    let routed = decide("routed");
    assert_eq!(claim(&routed, "ticket.triaged"), Truth::True);
    assert_eq!(legitimate(&routed), ["triaged"]);

    // The requester writes again: the ticket moves to c4 and nothing read of c3 counts.
    let moved = decide("requester-wrote-again");
    assert_eq!(claim(&moved, "ticket.triaged"), Truth::Unknown);
    assert_eq!(obligation(&moved, "escalate_sla_breach"), "open");
    assert_eq!(action(&moved, "ticket.route"), "blocked");
    assert!(legitimate(&moved).is_empty());
}

#[test]
fn tri_48213_triage_lapses_when_the_ticket_read_is_older_than_15_minutes() {
    let fixture = load(TRI_48213);
    let compiled = fixture.compile().expect("the protocol compiles");
    let observed = [
        ("snapshot-c2", "2026-10-05T09:00:00Z"),
        ("snapshot-c3", "2026-10-05T09:20:00Z"),
        ("requester-u1", "2026-10-05T09:21:00Z"),
        ("classification-c3", "2026-10-05T09:22:00Z"),
        ("review-c3", "2026-10-05T09:24:00Z"),
        ("route-c3", "2026-10-05T09:26:00Z"),
    ];
    let evidence: Vec<_> = fixture
        .evidence("routed")
        .into_iter()
        .map(|mut record| {
            let (_, at) = observed
                .iter()
                .find(|(id, _)| *id == record.id.as_str())
                .unwrap_or_else(|| panic!("no observation time for `{}`", record.id.as_str()));
            record.observed_at = Some(Instant::new(*at));
            record
        })
        .collect();
    let authority = "- {capability: classification.review, decision: granted}\n\
                     - {capability: ticket.route, decision: granted}\n";
    let at = |instant: &str| {
        b10x_canon::eval::evaluate_with(
            compiled.ir(),
            fixture.case(),
            &evidence,
            b10x_canon::eval::Supplied {
                authority: Some(authority),
                at: Some(instant),
                decisions: None,
            },
        )
        .unwrap_or_else(|refusal| panic!("evaluates at {instant}: {refusal}"))
    };

    // Exactly 15 minutes old, the read still applies.
    let fresh = at("2026-10-05T09:35:00Z");
    assert_eq!(claim(&fresh, "ticket.current"), Truth::True);
    assert_eq!(legitimate(&fresh), ["triaged"]);

    // Older, it is expired: whether the target is breached is unknown again, routing is blocked
    // and `triaged` is no longer legitimate until the ticket is read again.
    let stale = at("2026-10-05T09:36:00Z");
    assert_eq!(claim(&stale, "ticket.current"), Truth::Unknown);
    assert_eq!(claim(&stale, "sla.breached"), Truth::Unknown);
    assert!(
        excluded(&stale, "ticket.current")
            .contains(&("snapshot-c3".to_owned(), ExclusionReason::Expired)),
        "{:?}",
        excluded(&stale, "ticket.current")
    );
    assert_eq!(claim(&stale, "ticket.triaged"), Truth::Unknown);
    assert_eq!(obligation(&stale, "escalate_sla_breach"), "open");
    assert_eq!(action(&stale, "ticket.route"), "blocked");
    assert!(legitimate(&stale).is_empty());
}

#[test]
fn tri_48214_a_breach_is_escalated_under_authority() {
    let fixture = load(TRI_48214);
    let compiled = fixture.compile().expect("the protocol compiles");
    assert_eq!(fixture.check(&compiled), Ok(()));
    let decide = |state: &str| {
        fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}` evaluates: {refusal}"))
    };

    let breach = decide("breach-read");
    assert_eq!(obligation(&breach, "escalate_sla_breach"), "open");
    assert_eq!(action(&breach, "escalation.request"), "approval-required");
    assert!(legitimate(&breach).is_empty());

    let escalated = decide("escalated");
    assert_eq!(obligation(&escalated, "escalate_sla_breach"), "discharged");
    assert_eq!(legitimate(&escalated), ["escalated"]);
}

#[test]
fn tri_48215_a_rejected_classification_or_an_unknown_requester_needs_a_human() {
    let fixture = load(TRI_48215);
    let compiled = fixture.compile().expect("the protocol compiles");
    assert_eq!(fixture.check(&compiled), Ok(()));
    let decide = |state: &str| {
        fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}` evaluates: {refusal}"))
    };

    assert!(legitimate(&decide("classification-proposed")).is_empty());

    // Routing authority is granted throughout; the rejected review still blocks routing.
    let rejected = decide("review-rejected");
    assert_eq!(claim(&rejected, "classification.reviewed"), Truth::False);
    assert_eq!(action(&rejected, "ticket.route"), "blocked");
    assert_eq!(legitimate(&rejected), ["needs_human"]);

    let unidentified = decide("requester-unidentified");
    assert_eq!(claim(&unidentified, "requester.known"), Truth::False);
    assert_eq!(legitimate(&unidentified), ["needs_human"]);
}
