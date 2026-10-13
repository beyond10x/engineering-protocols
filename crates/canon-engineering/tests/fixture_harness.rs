//! Acceptance for `story:fixture-harness`: one harness, `tests/support/mod.rs`, loads a fixture,
//! validates and compiles its protocol through Canon and evaluates the fixture's states with
//! Canon's evaluator, proven on the `smoke` fixture in `fixtures/smoke/`.
//!
//! The negative cases below decide what the harness refuses or reports instead of evaluating
//! something other than what the fixture says.

mod support;

use b10x_canon::model::{ClaimId, Decision, Truth};
use canon_engineering::vocabulary::{self, Category, Marking};
use support::Fixture;

/// The smoke fixture, as a path relative to the repository root.
const SMOKE: &str = "fixtures/smoke/smoke.fixture.yaml";

/// The value Canon's decision gives `claim`.
fn value(decision: &Decision, claim: &str) -> Truth {
    decision
        .claims
        .get(&ClaimId::new(claim))
        .unwrap_or_else(|| panic!("the decision has no claim `{claim}`"))
        .value
}

/// The smoke fixture's text, read at run time.
fn smoke_text() -> String {
    std::fs::read_to_string(support::repo_root().join(SMOKE)).expect("the smoke fixture reads")
}

/// The smoke fixture's text with `from` replaced by `to`, which must occur in it.
fn smoke_with(from: &str, to: &str) -> String {
    let text = smoke_text();
    assert!(text.contains(from), "the smoke fixture contains `{from}`");
    text.replacen(from, to, 1)
}

#[test]
fn harness_compiles_and_evaluates_smoke_fixture() {
    // The smoke protocol's claim and evidence kind are core terms of the vocabulary.
    let claim = vocabulary::lookup("service.healthy").expect("service.healthy is a term");
    assert_eq!(
        (claim.category, claim.marking),
        (Category::ClaimId, Marking::Core)
    );
    let kind = vocabulary::lookup("operational_observation").expect("a term");
    assert_eq!(
        (kind.category, kind.marking),
        (Category::EvidenceKind, Marking::Core)
    );

    let fixture = Fixture::load(SMOKE).unwrap_or_else(|error| panic!("{SMOKE}: {error}"));
    assert_eq!(fixture.protocol_path(), "fixtures/smoke/protocol.yaml");
    assert_eq!(fixture.at(), "2026-10-04T00:00:00Z");

    // 1. The smoke protocol validates through Canon and compiles to a `canon-ir/1` document.
    let compiled = fixture
        .compile()
        .unwrap_or_else(|error| panic!("the smoke protocol compiles: {error}"));
    let reread = b10x_canon::eval::read_ir(compiled.canon_ir())
        .unwrap_or_else(|refusal| panic!("Canon reads the compiled protocol back: {refusal}"));
    assert_eq!(&reread, compiled.ir());

    // 2. The initial state holds no evidence: `service.healthy` is UNKNOWN, not FALSE.
    assert!(fixture.evidence("initial").is_empty());
    let initial = fixture
        .evaluate(&compiled, "initial")
        .unwrap_or_else(|refusal| panic!("state `initial` evaluates: {refusal}"));
    assert_eq!(value(&initial, "service.healthy"), Truth::Unknown);

    // 3. One operational observation of a healthy service, observed before T0, added to it:
    //    `service.healthy` is TRUE.
    let observed = fixture.evidence("healthy-observed");
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].kind.as_str(), "operational_observation");
    assert_eq!(observed[0].result.as_deref(), Some("healthy"));
    let healthy = fixture
        .evaluate(&compiled, "healthy-observed")
        .unwrap_or_else(|refusal| panic!("state `healthy-observed` evaluates: {refusal}"));
    assert_eq!(value(&healthy, "service.healthy"), Truth::True);

    // The fixture's own expectations are the ones above.
    assert_eq!(fixture.check(&compiled), Ok(()));
}

#[test]
fn harness_reports_a_state_whose_decision_differs_from_its_expectation() {
    let text = smoke_with("service.healthy: unknown", "service.healthy: false");
    let fixture = Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}"));
    let compiled = fixture.compile().unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        fixture.check(&compiled),
        Err(vec![
            "state `initial`: claim `service.healthy` is unknown, expected false".to_owned()
        ])
    );
}

#[test]
fn harness_reports_a_claim_the_expectation_does_not_list() {
    let text = smoke_with(
        "        service.healthy: unknown\n",
        "        service.unlisted: unknown\n",
    );
    let fixture = Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}"));
    let compiled = fixture.compile().unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        fixture.check(&compiled),
        Err(vec![
            "state `initial`: claim `service.healthy` is unknown, expected nothing".to_owned(),
            "state `initial`: claim `service.unlisted` is not declared, expected unknown"
                .to_owned(),
        ])
    );
}

#[test]
fn harness_refuses_a_fixture_it_cannot_evaluate_as_written() {
    for (from, to, refusal) in [
        (
            "format: els-fixture/1",
            "format: els-fixture/2",
            "format is `els-fixture/2`, expected `els-fixture/1`",
        ),
        (
            "at: \"2026-10-04T00:00:00Z\"",
            "at: \"2026-10-04\"",
            "evaluation instant `2026-10-04` is not written YYYY-MM-DDTHH:MM:SSZ",
        ),
        (
            "observed_at: \"2026-10-03T23:00:00Z\"",
            "observed_at: \"2026-10-04T00:00:01Z\"",
            "state `healthy-observed`: evidence `observation-1` is observed at \
             `2026-10-04T00:00:01Z`, after the evaluation instant `2026-10-04T00:00:00Z`",
        ),
        (
            "observed_at: \"2026-10-03T23:00:00Z\"",
            "observed_at: \"yesterday\"",
            "state `healthy-observed`: evidence `observation-1` observation instant `yesterday` \
             is not written YYYY-MM-DDTHH:MM:SSZ",
        ),
        (
            "protocol: fixtures/smoke/protocol.yaml",
            "protocol: ../smoke/protocol.yaml",
            "protocol path `../smoke/protocol.yaml` is absolute or leaves the repository",
        ),
        (
            "protocol: fixtures/smoke/protocol.yaml",
            "protocol: /etc/protocol.yaml",
            "protocol path `/etc/protocol.yaml` is absolute or leaves the repository",
        ),
        (
            "  - id: healthy-observed",
            "  - id: initial",
            "state `initial` is given more than once",
        ),
        (
            "states:\n",
            "authority: []\nstates:\n",
            "unknown field `authority`",
        ),
    ] {
        let text = smoke_with(from, to);
        let error = match Fixture::from_yaml(&text) {
            Ok(_) => panic!("a fixture with `{to}` loads; expected `{refusal}`"),
            Err(error) => error.to_string(),
        };
        assert!(
            error.contains(refusal),
            "a fixture with `{to}` is refused as `{error}`, expected `{refusal}`"
        );
    }
}

/// The smoke fixture with state `healthy-observed` given its own instant `at`.
fn smoke_observed_at(at: &str) -> String {
    smoke_with(
        "  - id: healthy-observed\n",
        &format!("  - id: healthy-observed\n    at: \"{at}\"\n"),
    )
}

#[test]
fn harness_evaluates_each_state_at_its_own_instant() {
    // A state without `at` is evaluated at the fixture's instant, a state with one at its own,
    // and evidence observed after the fixture's instant but not after its state's loads.
    let text = smoke_observed_at("2026-10-04T00:30:00Z").replacen(
        "observed_at: \"2026-10-03T23:00:00Z\"",
        "observed_at: \"2026-10-04T00:30:00Z\"",
        1,
    );
    let fixture = Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(fixture.at(), "2026-10-04T00:00:00Z");
    assert_eq!(fixture.instant("initial"), "2026-10-04T00:00:00Z");
    assert_eq!(fixture.instant("healthy-observed"), "2026-10-04T00:30:00Z");
    let compiled = fixture.compile().unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(fixture.check(&compiled), Ok(()));

    // A state after one that gives its own instant inherits it.
    let inc = std::fs::read_to_string(
        support::repo_root().join("fixtures/incident-response/stale-health.fixture.yaml"),
    )
    .expect("stale-health reads");
    let fixture = Fixture::from_yaml(&inc).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(fixture.instant("fresh"), "2026-10-13T11:29:00Z");
    assert_eq!(fixture.instant("stale"), "2026-10-13T11:31:00Z");

    // The instant reaches Canon: `stale-health` without its stale state's own instant evaluates
    // that state at the fixture's, where the observation of the service has not yet expired, and
    // differs from the state's expectations in exactly what the observation decides.
    let own = "    at: \"2026-10-13T11:31:00Z\"\n";
    assert_eq!(
        inc.matches(own).count(),
        1,
        "stale-health gives one state at"
    );
    let inherited =
        Fixture::from_yaml(&inc.replacen(own, "", 1)).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(inherited.instant("stale"), "2026-10-13T11:29:00Z");
    let compiled = inherited
        .compile()
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        inherited.check(&compiled),
        Err(vec![
            "state `stale`: claim `service.healthy` is true, expected unknown".to_owned(),
            "state `stale`: obligation `restore_service` is discharged, expected open".to_owned(),
            "state `stale`: action `emergency.leave` is admissible, expected blocked".to_owned(),
        ])
    );
}

#[test]
fn harness_refuses_a_state_instant_it_cannot_evaluate_at() {
    let two_instants = |initial: &str, observed: &str| {
        smoke_observed_at(observed).replacen(
            "  - id: initial\n",
            &format!("  - id: initial\n    at: \"{initial}\"\n"),
            1,
        )
    };
    for (text, refusal) in [
        (
            smoke_observed_at("2026-10-04"),
            "state `healthy-observed`: evaluation instant `2026-10-04` is not written \
             YYYY-MM-DDTHH:MM:SSZ"
                .to_owned(),
        ),
        (
            smoke_observed_at("2026-02-30T00:00:00Z"),
            "state `healthy-observed`: evaluation instant `2026-02-30T00:00:00Z` is not written \
             YYYY-MM-DDTHH:MM:SSZ"
                .to_owned(),
        ),
        (
            two_instants("2026-10-03T23:59:59Z", "2026-10-04T00:10:00Z"),
            "state `initial`: evaluation instant `2026-10-03T23:59:59Z` is before the fixture's \
             evaluation instant, `2026-10-04T00:00:00Z`"
                .to_owned(),
        ),
        (
            two_instants("2026-10-04T00:10:00Z", "2026-10-04T00:05:00Z"),
            "state `healthy-observed`: evaluation instant `2026-10-04T00:05:00Z` is before the \
             evaluation instant of the state before it, `2026-10-04T00:10:00Z`"
                .to_owned(),
        ),
        (
            // The state before it gives no instant of its own: it is the fixture's.
            smoke_observed_at("2026-10-03T23:30:00Z"),
            "state `healthy-observed`: evaluation instant `2026-10-03T23:30:00Z` is before the \
             evaluation instant of the state before it, `2026-10-04T00:00:00Z`"
                .to_owned(),
        ),
        (
            smoke_observed_at("2026-10-04T00:30:00Z").replacen(
                "observed_at: \"2026-10-03T23:00:00Z\"",
                "observed_at: \"2026-10-04T00:31:00Z\"",
                1,
            ),
            "state `healthy-observed`: evidence `observation-1` is observed at \
             `2026-10-04T00:31:00Z`, after the evaluation instant `2026-10-04T00:30:00Z`"
                .to_owned(),
        ),
        (
            smoke_observed_at("2026-10-04T00:30:00Z")
                .replacen(
                    "observed_at: \"2026-10-03T23:00:00Z\"",
                    "observed_at: \"2026-10-04T00:30:00Z\"",
                    1,
                )
                .replacen(
                    "          id: observation-1\n",
                    "          id: observation-1\n          observed_at: \"2026-10-04T00:31:00Z\"\n",
                    1,
                ),
            "state `healthy-observed`: evidence `observation-1` record is observed at \
             `2026-10-04T00:31:00Z`, after the evaluation instant `2026-10-04T00:30:00Z`"
                .to_owned(),
        ),
        (
            smoke_with(
                "  - id: healthy-observed\n",
                "  - id: healthy-observed\n    at:\n",
            ),
            "a key is written with no value".to_owned(),
        ),
    ] {
        let error = match Fixture::from_yaml(&text) {
            Ok(_) => panic!("a fixture is loaded; expected `{refusal}`:\n{text}"),
            Err(error) => error.to_string(),
        };
        assert!(
            error.contains(&refusal),
            "a fixture is refused as `{error}`, expected `{refusal}`:\n{text}"
        );
    }
}

#[test]
fn harness_refuses_a_fixture_without_states() {
    let text = smoke_text();
    let (head, _) = text
        .split_once("\nstates:\n")
        .expect("the smoke fixture lists states");
    let text = format!("{head}\nstates: []\n");
    let error = Fixture::from_yaml(&text)
        .map(|_| ())
        .expect_err("no states is refused");
    assert_eq!(error.to_string(), "the fixture lists no states");
}
