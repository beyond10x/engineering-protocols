//! Adversary pass 2 on `story:fixture-harness`: cases that kill mutants of the harness the
//! acceptance suite and pass 1 leave alive. Each names the mutant it kills.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::path::PathBuf;

use serde_yaml_ng::Value;
use support::{CompileError, Fixture};

const SMOKE: &str = "fixtures/smoke/smoke.fixture.yaml";
const AT: &str = "at: \"2026-10-04T00:00:00Z\"";
const OBSERVED_AT: &str = "observed_at: \"2026-10-03T23:00:00Z\"";

fn smoke_text() -> String {
    std::fs::read_to_string(support::repo_root().join(SMOKE)).expect("the smoke fixture reads")
}

fn smoke_with(from: &str, to: &str) -> String {
    let text = smoke_text();
    assert!(text.contains(from), "the smoke fixture contains `{from}`");
    text.replacen(from, to, 1)
}

/// The smoke fixture with evaluation instant `at` and its one observation made at `observed`.
fn smoke_at(at: &str, observed: &str) -> String {
    smoke_with(AT, &format!("at: \"{at}\"")).replacen(
        OBSERVED_AT,
        &format!("observed_at: \"{observed}\""),
        1,
    )
}

/// Leap years by the Gregorian rule: every fourth year, except centuries, except every fourth
/// century. Pass 1 tried only 2026 (not divisible by 4).
/// Kills: `leap = year % 4 == 0` (1900, 2100 load), dropping `|| year % 400 == 0` (2000 refused),
/// `2 if leap => 28` and `leap = false` (2000, 2024 refused).
#[test]
fn adversary2_harness_leap_years_follow_the_gregorian_rule() {
    for at in [
        "2000-02-29T00:00:00Z",
        "2024-02-29T00:00:00Z",
        "2400-02-29T23:59:59Z",
    ] {
        let fixture = Fixture::from_yaml(&smoke_at(at, at))
            .unwrap_or_else(|e| panic!("`{at}` names a day that exists: {e}"));
        assert_eq!(fixture.at(), at);
    }
    for at in [
        "1900-02-29T00:00:00Z",
        "2100-02-29T00:00:00Z",
        "2023-02-29T00:00:00Z",
    ] {
        let error = Fixture::from_yaml(&smoke_at(at, "1800-01-01T00:00:00Z"))
            .map(|_| ())
            .expect_err(&format!("`{at}` names no day"));
        assert_eq!(
            error.to_string(),
            format!("evaluation instant `{at}` is not written YYYY-MM-DDTHH:MM:SSZ")
        );
    }
    // The observation instant goes through the same check.
    let error = Fixture::from_yaml(&smoke_at("2101-01-01T00:00:00Z", "2100-02-29T00:00:00Z"))
        .map(|_| ())
        .expect_err("`2100-02-29` names no day");
    assert_eq!(
        error.to_string(),
        "state `healthy-observed`: evidence `observation-1` observation instant \
         `2100-02-29T00:00:00Z` is not written YYYY-MM-DDTHH:MM:SSZ or \
         YYYY-MM-DDTHH:MM:SS.fZ (one to nine fraction digits)"
    );
}

/// A claim Canon evaluates TRUE where the fixture expects UNKNOWN (or FALSE) is a difference. The
/// suite only ever compares equal values, or UNKNOWN found against FALSE expected, so a comparison
/// that is not equality passes it.
/// Kills: `if found == expected` -> `if found <= expected` in `Fixture::check`.
#[test]
fn adversary2_harness_check_reports_true_where_unknown_or_false_is_expected() {
    let compiled =
        support::compile_protocol("fixtures/smoke/protocol.yaml").unwrap_or_else(|e| panic!("{e}"));
    for expected in ["unknown", "false"] {
        let text = smoke_with(
            "        service.healthy: true\n",
            &format!("        service.healthy: {expected}\n"),
        );
        let fixture = Fixture::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            fixture.check(&compiled),
            Err(vec![format!(
                "state `healthy-observed`: claim `service.healthy` is true, expected {expected}"
            )])
        );
    }
}

/// The fixture as YAML, so a test can hand Canon's readers the exact value the harness gives them.
fn yaml(text: &str) -> Value {
    serde_yaml_ng::from_str(text).expect("the fixture is YAML")
}

/// A case Canon refuses is refused with Canon's code and message, unchanged.
/// Kills: dropping `refusal.code()` or `{refusal}` from the case refusal in `Fixture::from_yaml`.
#[test]
fn adversary2_harness_case_refusal_is_canons_code_and_message() {
    let text = smoke_with("service: {revision: r1}", "service: {revision: 1}");
    let canon = b10x_canon::eval::case_from_value(&yaml(&text)["case"])
        .expect_err("Canon refuses `revision: 1`");
    assert_eq!(canon.code(), "malformed-input");
    let error = Fixture::from_yaml(&text)
        .map(|_| ())
        .expect_err("the harness refuses it too");
    assert_eq!(error.to_string(), format!("case: malformed-input: {canon}"));
}

/// An evidence record Canon refuses is refused with its state, its position, and Canon's code and
/// message, unchanged.
/// Kills: dropping the code, the message or the `index + 1` from the evidence refusal.
#[test]
fn adversary2_harness_evidence_refusal_is_canons_code_and_message() {
    let text = smoke_with("subject_revision: r1", "subject_revision: 1");
    let record = &yaml(&text)["states"][1]["add_evidence"][0]["record"];
    let canon =
        b10x_canon::eval::evidence_from_value(record).expect_err("Canon refuses the record");
    assert_eq!(canon.code(), "malformed-input");
    let error = Fixture::from_yaml(&text)
        .map(|_| ())
        .expect_err("the harness refuses it too");
    assert_eq!(
        error.to_string(),
        format!("state `healthy-observed`: evidence 1: malformed-input: {canon}")
    );
}

/// Removes the directory this test made inside the repository's ignored `target/`.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A protocol Canon's validator rejects is `CompileError::Invalid` holding Canon's problems,
/// unchanged, and displayed as them. No test compiled an invalid protocol before.
/// Kills: `map_err(CompileError::Invalid)` replaced by an empty or rewritten problem list.
#[test]
fn adversary2_harness_compile_passes_canon_problems_through() {
    let protocol = support::repo_root().join("fixtures/smoke/protocol.yaml");
    let text = std::fs::read_to_string(protocol).expect("the smoke protocol reads");
    let from = "evidence_kinds:\n  operational_observation:";
    assert!(
        text.contains(from),
        "the smoke protocol declares its evidence kind"
    );
    let text = text.replacen(from, "evidence_kinds:\n  other_observation:", 1);
    let parsed = b10x_canon::model::parse(&text).expect("still protocol/1");
    let canon = b10x_canon::validate::validate(&parsed).expect_err("an undeclared kind");
    assert!(!canon.is_empty());

    let name = format!("adversary2-harness-invalid-{}", std::process::id());
    let dir = Scratch(support::repo_root().join("target").join(&name));
    std::fs::create_dir_all(&dir.0).expect("scratch directory under the ignored target/");
    std::fs::write(dir.0.join("protocol.yaml"), &text).expect("write the invalid protocol");
    match support::compile_protocol(&format!("target/{name}/protocol.yaml")) {
        Err(CompileError::Invalid(problems)) => {
            assert_eq!(problems, canon);
            let lines: Vec<String> = canon.iter().map(ToString::to_string).collect();
            assert_eq!(
                CompileError::Invalid(problems).to_string(),
                format!("the protocol is invalid: {}", lines.join("; "))
            );
        }
        other => panic!("expected Canon's problems, got {other:?}"),
    }
}

/// An observation instant may carry a fraction of one to nine digits, as Canon 0.8.0 reads
/// `observed_at`; any other fraction is refused with the form named, and a fraction is counted
/// against the evaluation instant in whole seconds, as Canon counts it: an observation in the
/// evaluation instant's own second is not after it, one in the next second is.
#[test]
fn harness_observation_instants_take_a_fraction_counted_in_whole_seconds() {
    for observed in [
        "2026-10-03T23:00:00.5Z",
        "2026-10-03T23:00:00.123456789Z",
        "2026-10-04T00:00:00.999Z",
    ] {
        Fixture::from_yaml(&smoke_at("2026-10-04T00:00:00Z", observed))
            .unwrap_or_else(|error| panic!("`{observed}` loads: {error}"));
    }
    for observed in [
        "2026-10-03T23:00:00.Z",
        "2026-10-03T23:00:00.1234567890Z",
        "2026-10-03T23:00:00.5",
        "2026-10-03T23:00:00,5Z",
        "2026-10-03T23:00:00.5z",
        "2026-10-03T23:00:00.-5Z",
        "2026-10-03T23:00:00.５Z",
        "2026-10-03T23:00:60.5Z",
        "2026-10-03T23:00:00Z.5",
    ] {
        let error = Fixture::from_yaml(&smoke_at("2026-10-04T00:00:00Z", observed))
            .map(|_| ())
            .expect_err(&format!("`{observed}` is refused"));
        assert_eq!(
            error.to_string(),
            format!(
                "state `healthy-observed`: evidence `observation-1` observation instant \
                 `{observed}` is not written YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS.fZ \
                 (one to nine fraction digits)"
            )
        );
    }
    let error = Fixture::from_yaml(&smoke_at("2026-10-04T00:00:00Z", "2026-10-04T00:00:01.0Z"))
        .map(|_| ())
        .expect_err("the next second is after the evaluation instant");
    assert_eq!(
        error.to_string(),
        "state `healthy-observed`: evidence `observation-1` is observed at \
         `2026-10-04T00:00:01.0Z`, after the evaluation instant `2026-10-04T00:00:00Z`"
    );
}
