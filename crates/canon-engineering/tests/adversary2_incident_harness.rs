//! Adversary pass 2 on `story:incident-response-protocol`: the harness inputs pass 1 did not
//! probe — an observation time written inside the evidence record (which Canon at 8fc260a now
//! reads), a later denial of a capability already granted, the order of `add_authority` entries,
//! an artifact the protocol declares and the case omits, and whether any `inc-492` expectation
//! depends on the fixture's observation times.
//!
//! Every fixture here is `inc-492` with one textual change, loaded with `Fixture::from_yaml`; no
//! file in the repository is written.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use support::Fixture;

const INC_492: &str = "fixtures/incident-response/inc-492.fixture.yaml";

/// The fixture's text with `from` replaced by `to` once; `from` must occur in it.
fn inc_492_with(from: &str, to: &str) -> String {
    let text = std::fs::read_to_string(support::repo_root().join(INC_492)).expect("inc-492 reads");
    assert!(text.contains(from), "inc-492 contains `{from}`");
    text.replacen(from, to, 1)
}

fn compiled() -> support::Compiled {
    support::compile_protocol(&support::protocol_path("incident-response", 1))
        .unwrap_or_else(|error| panic!("{error}"))
}

/// `support/mod.rs` module docs: "It refuses, when loading, a fixture holding evidence observed
/// after its instant". Since the Canon bump to 8fc260a, `canon-evidence/1` reads an optional
/// `observed_at` on the record itself (at 33540b1 the record refused the key). The harness checks
/// only the outer `observed_at` of the state entry, so a record observed a day after the
/// evaluation instant loads.
#[test]
fn a_record_observed_after_the_instant_is_refused() {
    let text = inc_492_with(
        "          id: health-2\n",
        "          id: health-2\n          observed_at: \"2026-10-05T00:00:00Z\"\n",
    );
    match Fixture::from_yaml(&text) {
        Ok(fixture) => {
            let carried: Vec<String> = fixture
                .evidence("service-restored")
                .iter()
                .filter_map(|record| {
                    record
                        .observed_at
                        .as_ref()
                        .map(|at| format!("{} observed_at {}", record.id.as_str(), at.as_str()))
                })
                .collect();
            panic!(
                "a fixture whose record health-2 is observed at 2026-10-05T00:00:00Z, after its \
                 instant {}, loads; the records carry {carried:?} to Canon",
                fixture.at()
            );
        }
        Err(error) => assert!(
            error.to_string().contains("after the evaluation instant"),
            "refused, but as `{error}`"
        ),
    }
}

/// `support/mod.rs` module docs: "Each record reaches Canon with its entry's `observed_at` as the
/// record's own", which Canon's order forms read. Until `story:evidence-order-red-first` the
/// harness passed no observation time at all; this case pinned that and now pins its reverse.
/// Every record carries its entry's instant, the one that writes its own (here the same instant as
/// the state entry, so the fixture loads) as well as those that do not.
#[test]
fn the_harness_passes_each_entrys_observation_time_to_canon() {
    let text = inc_492_with(
        "          id: health-2\n",
        "          id: health-2\n          observed_at: \"2026-10-04T11:40:00Z\"\n",
    );
    let fixture = Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}"));
    let carried: Vec<String> = fixture
        .evidence("service-restored")
        .iter()
        .filter_map(|record| {
            record
                .observed_at
                .as_ref()
                .map(|at| format!("{} observed_at {}", record.id.as_str(), at.as_str()))
        })
        .collect();
    assert_eq!(
        carried,
        [
            "health-1 observed_at 2026-10-04T11:32:00Z",
            "impact-1 observed_at 2026-10-04T11:33:00Z",
            "cause-1 observed_at 2026-10-04T11:34:00Z",
            "release-health-1 observed_at 2026-10-04T11:35:00Z",
            "release-impact-1 observed_at 2026-10-04T11:36:00Z",
            "health-2 observed_at 2026-10-04T11:40:00Z",
            "impact-2 observed_at 2026-10-04T11:45:00Z",
        ],
        "the evidence the harness passes to Canon is not observed at its entries' instants"
    );
}

/// Probe, green: a denial of a capability an earlier state granted is not a revocation. The
/// decisions accumulate into one `canon-authority/1` list, Canon refuses it as deciding the
/// capability twice, and the refusal surfaces only from `check`, for that state and every later
/// one; the fixture loads.
#[test]
fn a_later_denial_of_a_granted_capability_is_refused_by_canon_at_check() {
    let text = inc_492_with(
        "  - id: rolled-back\n    set_revisions:\n      service: s2\n    add_authority: []\n",
        "  - id: rolled-back\n    set_revisions:\n      service: s2\n    add_authority:\n      - {capability: release.rollback, decision: denied}\n",
    );
    let fixture = Fixture::from_yaml(&text).expect("the fixture loads");
    let differences = fixture.check(&compiled()).expect_err("check differs");
    assert_eq!(
        differences,
        [
            "state `rolled-back`: evaluation refused: duplicate-identifier: `--authority` decides \
             capability `release.rollback` more than once",
            "state `release-observed`: evaluation refused: duplicate-identifier: `--authority` \
             decides capability `release.rollback` more than once",
            "state `service-restored`: evaluation refused: duplicate-identifier: `--authority` \
             decides capability `release.rollback` more than once",
            "state `cause-identified-after-restore`: evaluation refused: duplicate-identifier: \
             `--authority` decides capability `release.rollback` more than once",
        ]
    );
}

/// Probe, green: the order of `add_authority` entries does not change a decision.
#[test]
fn the_order_of_authority_entries_does_not_matter() {
    let compiled = compiled();
    let grant = "    add_authority:\n      - {capability: release.rollback, decision: granted}\n";
    let forward = inc_492_with(
        grant,
        "    add_authority:\n      - {capability: release.rollback, decision: granted}\n      - {capability: traffic.shift, decision: denied}\n",
    );
    let backward = inc_492_with(
        grant,
        "    add_authority:\n      - {capability: traffic.shift, decision: denied}\n      - {capability: release.rollback, decision: granted}\n",
    );
    let forward = Fixture::from_yaml(&forward).expect("loads");
    let backward = Fixture::from_yaml(&backward).expect("loads");
    for state in forward.states() {
        let one = forward.evaluate(&compiled, state).expect("evaluates");
        let other = backward.evaluate(&compiled, state).expect("evaluates");
        assert_eq!(one, other, "state `{state}`");
    }
    let decision = forward
        .evaluate(&compiled, "rollback-approved")
        .expect("evaluates");
    assert_eq!(
        decision.actions.as_ref().expect("actions")["traffic.shift"]["status"],
        "blocked"
    );
}

/// Probe, green: an artifact the protocol declares and the case omits loads, and Canon refuses
/// every state at `check` (`missing-artifact`); `set_revisions` on it is refused at load.
#[test]
fn an_artifact_the_case_omits() {
    // `incident.response/1` declares `release` again (story:incident-response-subject-binding),
    // so the probe runs on the shipped protocol: inc-492's case without the release, at each of
    // its seven states.
    let without = inc_492_with("    release: {revision: r42}\n", "");
    let fixture = Fixture::from_yaml(&without).expect("loads");
    let differences = fixture.check(&compiled()).expect_err("differs");
    assert_eq!(differences.len(), 7, "{differences:?}");
    assert!(
        differences
            .iter()
            .all(|line| line.contains("evaluation refused: missing-artifact")),
        "{differences:?}"
    );
    let set = without.replacen("      service: s2\n", "      release: r41\n", 1);
    let error = Fixture::from_yaml(&set).expect_err("refused").to_string();
    assert!(
        error.contains("sets the revision of artifact `release`, which the case does not list"),
        "{error}"
    );
}

/// Probe, green: no expectation of `inc-492` depends on the order of its observation times. The
/// healthy observation of s2 written as observed before the unhealthy one of s1, and every
/// observation at the evaluation instant itself, leave every state as expected. Since
/// `story:stale-evidence-fixtures` an operational observation applies for 30 minutes and the
/// harness passes the instant to Canon, so every time here stays within the 30 minutes before the
/// fixture's instant; the healthy observation of s2 written 2 hours earlier has expired, and
/// leaves the service's health UNKNOWN, not FALSE, in the states that rest on it.
#[test]
fn inc_492_expectations_do_not_depend_on_observation_times() {
    let compiled = compiled();
    let earlier = inc_492_with(
        "observed_at: \"2026-10-04T11:40:00Z\"",
        "observed_at: \"2026-10-04T11:31:00Z\"",
    );
    let fixture = Fixture::from_yaml(&earlier).expect("loads");
    assert_eq!(fixture.check(&compiled), Ok(()));

    let expired = inc_492_with(
        "observed_at: \"2026-10-04T11:40:00Z\"",
        "observed_at: \"2026-10-04T10:00:00Z\"",
    );
    let fixture = Fixture::from_yaml(&expired).expect("loads");
    let mut differences = Vec::new();
    for state in ["service-restored", "cause-identified-after-restore"] {
        differences.extend([
            format!("state `{state}`: claim `service.healthy` is unknown, expected true"),
            format!("state `{state}`: obligation `restore_service` is open, expected discharged"),
            format!("state `{state}`: action `emergency.leave` is blocked, expected admissible"),
        ]);
    }
    assert_eq!(fixture.check(&compiled), Err(differences));

    let text = std::fs::read_to_string(support::repo_root().join(INC_492)).expect("reads");
    let mut all_at_instant = String::new();
    for line in text.lines() {
        match line.split_once("observed_at: ") {
            Some((indent, _)) => {
                all_at_instant.push_str(indent);
                all_at_instant.push_str("observed_at: \"2026-10-04T12:00:00Z\"");
            }
            None => all_at_instant.push_str(line),
        }
        all_at_instant.push('\n');
    }
    let fixture = Fixture::from_yaml(&all_at_instant).expect("loads");
    assert_eq!(fixture.check(&compiled), Ok(()));
}
