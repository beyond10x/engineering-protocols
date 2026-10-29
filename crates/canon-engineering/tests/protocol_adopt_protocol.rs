//! Acceptance for `story:protocol-adopt-protocol` (Atlas ADR 0086 § Decision 2-5): adopting a
//! generated protocol is a case of its own. `protocol.adopt/1` is Canon `protocol/1` data at
//! `protocols/protocol-adopt/1.yaml`, a built-in outside engineering whose terms stay out of
//! `protocols/vocabulary.yaml`, as `support.triage/1`'s do.
//!
//! Its outcome `adopted` is earned only by one revision of the generated protocol with a
//! `canon diff` report within the organization floor, a passing `canon check` report and an
//! approval recorded by `protocol.approve`, which requires the capability `protocol.adopt`; each
//! record is bound to the revision under adoption. Evaluation reads the two reports as evidence and
//! runs neither tool.
//!
//! Fixtures under `fixtures/protocol-adopt/`:
//!
//! - `adoption`: no evidence (1), both reports (2), the grant and the approval (3), then the
//!   generated protocol moves to a new revision (4).
//! - `adoption-outside-floor`: a diff report outside the floor beside everything else (5).
//! - `adoption-check-failed`, `adoption-rejected`, `adoption-unvouched` and
//!   `adoption-approved-first`: a failed check, a rejected approval, an approval by a principal the
//!   case does not list as independent (or by nobody named), and an approval observed before the
//!   first diff report; none adopts. Canon outcomes read claims, not authority decisions, so the
//!   approval counts only from a principal the case lists on `different_principal`, and only after
//!   the first diff report.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use b10x_canon::model::{ClaimId, Decision, ExclusionReason, Truth};
use canon_engineering::registry;
use support::Fixture;

const ADOPTION: &str = "fixtures/protocol-adopt/adoption.fixture.yaml";
const OUTSIDE_FLOOR: &str = "fixtures/protocol-adopt/adoption-outside-floor.fixture.yaml";

/// Every claim `protocol.adopt/1` declares, in identifier order.
const CLAIMS: [&str; 3] = ["adoption.approved", "checks.pass", "floor.respected"];

fn load(path: &str) -> Fixture {
    let fixture = Fixture::load(path).unwrap_or_else(|error| panic!("{path}: {error}"));
    assert_eq!(fixture.protocol_path(), "protocols/protocol-adopt/1.yaml");
    fixture
}

fn claim(decision: &Decision, id: &str) -> Truth {
    decision
        .claims
        .get(&ClaimId::new(id))
        .unwrap_or_else(|| panic!("the decision has no claim `{id}`"))
        .value
}

fn excluded(decision: &Decision, id: &str) -> Vec<(String, ExclusionReason)> {
    decision
        .claims
        .get(&ClaimId::new(id))
        .unwrap_or_else(|| panic!("the decision has no claim `{id}`"))
        .excluded_evidence
        .iter()
        .map(|exclusion| (exclusion.evidence.as_str().to_owned(), exclusion.reason))
        .collect()
}

fn status(decision: &Decision, section: &str, id: &str) -> String {
    let found = match section {
        "actions" => decision.actions.as_ref(),
        "outcomes" => decision.outcomes.as_ref(),
        other => panic!("no section `{other}`"),
    }
    .unwrap_or_else(|| panic!("the decision has a {section} section"));
    found[id]["status"]
        .as_str()
        .unwrap_or_else(|| panic!("the decision has no {section} entry `{id}`: {found}"))
        .to_owned()
}

/// Whether evaluation reports the outcome `adopted`.
fn adopted(decision: &Decision) -> bool {
    status(decision, "outcomes", "adopted") == "legitimate"
}

#[test]
fn adoption_needs_floor_check_and_authority() {
    let fixture = load(ADOPTION);
    let compiled = fixture
        .compile()
        .unwrap_or_else(|error| panic!("protocols/protocol-adopt/1.yaml compiles: {error}"));
    let ir = compiled.ir();
    assert_eq!(ir.protocol.id.as_str(), "protocol.adopt");
    assert_eq!(ir.protocol.revision, 1);
    let claims: Vec<&str> = ir.claims.keys().map(|id| id.as_str()).collect();
    assert_eq!(claims, CLAIMS);
    let approve = &ir.actions[&b10x_canon::model::ActionId::new("protocol.approve")];
    let capabilities: Vec<&str> = approve
        .requires
        .iter()
        .map(|requirement| requirement.as_str())
        .collect();
    assert_eq!(
        capabilities,
        ["protocol.adopt"],
        "protocol.approve's authority"
    );
    assert_eq!(fixture.check(&compiled), Ok(()));
    assert_eq!(
        fixture.states(),
        ["opened", "reports-in", "approved", "revision-moved"]
    );
    let evaluate = |state: &str| {
        fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}`: {refusal}"))
    };

    // 1. No evidence, no authority decision: both report claims UNKNOWN, no outcome.
    let opened = evaluate("opened");
    assert_eq!(claim(&opened, "floor.respected"), Truth::Unknown, "1.");
    assert_eq!(claim(&opened, "checks.pass"), Truth::Unknown, "1.");
    assert!(!adopted(&opened), "1. no outcome");

    // 2. A diff report within the floor and a passing check report, both bound to r1: approving
    //    needs authority, and there is still no outcome.
    let reports = evaluate("reports-in");
    assert_eq!(claim(&reports, "floor.respected"), Truth::True, "2.");
    assert_eq!(claim(&reports, "checks.pass"), Truth::True, "2.");
    assert_eq!(
        status(&reports, "actions", "protocol.approve"),
        "approval-required",
        "2."
    );
    assert!(!adopted(&reports), "2. no outcome");

    // 3. protocol.adopt granted and protocol.approve's approval recorded: adopted.
    let approved = evaluate("approved");
    assert_eq!(
        status(&approved, "actions", "protocol.approve"),
        "admissible",
        "3."
    );
    assert_eq!(claim(&approved, "adoption.approved"), Truth::True, "3.");
    assert!(adopted(&approved), "3. adopted");

    // 4. The generated protocol at r2, every record still about r1: UNKNOWN, not FALSE, each
    //    record excluded as revision_mismatch, and no outcome.
    let moved = evaluate("revision-moved");
    for (id, record) in [
        ("floor.respected", "diff-r1"),
        ("checks.pass", "check-r1"),
        ("adoption.approved", "approval-r1"),
    ] {
        assert_eq!(claim(&moved, id), Truth::Unknown, "4. {id}");
        assert_eq!(
            excluded(&moved, id),
            [(record.to_owned(), ExclusionReason::RevisionMismatch)],
            "4. {id}"
        );
    }
    assert!(!adopted(&moved), "4. no outcome");

    // 5. A diff report outside the floor, a passing check report, the grant and even an approval
    //    record: floor.respected FALSE and no outcome.
    let outside = load(OUTSIDE_FLOOR);
    let compiled_outside = outside.compile().expect("the protocol compiles");
    assert_eq!(outside.check(&compiled_outside), Ok(()));
    let outside_floor = outside
        .evaluate(&compiled_outside, "outside-floor")
        .unwrap_or_else(|refusal| panic!("state `outside-floor`: {refusal}"));
    assert_eq!(claim(&outside_floor, "floor.respected"), Truth::False, "5.");
    assert_eq!(claim(&outside_floor, "checks.pass"), Truth::True, "5.");
    assert!(!adopted(&outside_floor), "5. no outcome");

    // 6. canon check finds no unreachable outcome and no authority bypass.
    let report = b10x_canon::check::check(ir, None)
        .unwrap_or_else(|refusal| panic!("6. canon check runs: {refusal}"));
    assert!(report.is_clean(), "6. canon check found:\n{report}");
    assert!(report.states <= b10x_canon::check::STATE_BOUND, "6.");

    // 7. The registry lists protocol-adopt@1 among the built-ins, as released.
    assert!(
        registry::list().contains(&("protocol-adopt", 1)),
        "7. registry::list(): {:?}",
        registry::list()
    );
    let builtin = registry::get("protocol-adopt", 1)
        .unwrap_or_else(|error| panic!("7. registry::get(protocol-adopt, 1): {error}"));
    let on_disk =
        std::fs::read_to_string(support::repo_root().join("protocols/protocol-adopt/1.yaml"))
            .unwrap_or_else(|error| panic!("7. protocols/protocol-adopt/1.yaml: {error}"));
    assert_eq!(
        builtin.yaml, on_disk,
        "7. the built-in is the file as released"
    );
}

/// Every fixture under `fixtures/protocol-adopt/` holds against the protocol: the failed check, the
/// rejected approval, the approval by a principal the case does not list (or by nobody named), and
/// the approval observed before the reports each adopt nothing.
#[test]
fn every_protocol_adopt_fixture_holds() {
    let directory = support::repo_root().join("fixtures/protocol-adopt");
    let mut files: Vec<String> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            entry
                .expect("a directory entry")
                .file_name()
                .into_string()
                .expect("a UTF-8 file name")
        })
        .filter(|file| file.ends_with(".fixture.yaml"))
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "adoption-approved-first.fixture.yaml",
            "adoption-check-failed.fixture.yaml",
            "adoption-outside-floor.fixture.yaml",
            "adoption-rejected.fixture.yaml",
            "adoption-unvouched.fixture.yaml",
            "adoption.fixture.yaml",
        ]
    );
    for file in files {
        let fixture = load(&format!("fixtures/protocol-adopt/{file}"));
        let compiled = fixture.compile().expect("the protocol compiles");
        assert_eq!(fixture.check(&compiled), Ok(()), "{file}");
        for state in fixture.states() {
            let decision = fixture
                .evaluate(&compiled, state)
                .unwrap_or_else(|refusal| panic!("{file} `{state}`: {refusal}"));
            let expected = file == "adoption.fixture.yaml" && state == "approved";
            assert_eq!(adopted(&decision), expected, "{file} `{state}`");
        }
    }
}

/// The protocol names no Git, pull request or code (the story's pressure test), so a generated
/// variant of any protocol is adopted by the same case.
#[test]
fn protocol_adopt_names_no_code_host_term() {
    let text =
        std::fs::read_to_string(support::repo_root().join("protocols/protocol-adopt/1.yaml"))
            .expect("protocols/protocol-adopt/1.yaml exists");
    let lowered = text.to_lowercase();
    for term in [
        "git",
        "pull request",
        "pull_request",
        "commit",
        "merge",
        "branch",
    ] {
        assert!(
            !lowered.contains(term),
            "protocols/protocol-adopt/1.yaml names `{term}`"
        );
    }
}
