use canon_engineering_assertions::{
    Admission, Bundle, Change, GateResult, GatesReport, collect, evaluate_gates, prepare_with,
};
use canon_expr::Truth;
use serde_json::json;
use std::{fs, path::Path};

const NOW: i64 = 1000;

fn write_gates(root: &Path, gates: serde_json::Value) {
    fs::write(
        root.join("gates.yaml"),
        serde_json::to_vec(&json!({
            "format": "engineering-gates/1",
            "language": "canon-expr/1",
            "gates": gates,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn story(tags: &[&str]) -> Admission {
    Admission {
        change: Change {
            story_id: Some("story-1".into()),
            story_tags: Some(tags.iter().map(|t| (*t).to_owned()).collect()),
            ..Change::default()
        },
    }
}

fn run(root: &Path, admission: &Admission) -> (Bundle, GatesReport) {
    let prepared = prepare_with(root, Path::new("gates.yaml"), None, None, admission).unwrap();
    let bundle = collect(prepared, NOW).unwrap();
    let report = evaluate_gates(&bundle, NOW, None, None).unwrap();
    (bundle, report)
}

fn result_of(report: &GatesReport, id: &str) -> GateResult {
    report.gates.iter().find(|g| g.id == id).unwrap().result
}

fn consumer_visible_gate() -> serde_json::Value {
    json!({
        "id": "changelog",
        "text": "A consumer-visible change records a release note",
        "applies_when": "story.has_tag(\"consumer-visible\")",
        "assertions": ["fs.file.exists(\"CHANGELOG.md\")"],
    })
}

#[test]
fn not_applicable_is_never_pass() {
    let root = tempfile::tempdir().unwrap();
    write_gates(root.path(), json!([consumer_visible_gate()]));
    let (_, report) = run(root.path(), &story(&["internal"]));
    assert_eq!(result_of(&report, "changelog"), GateResult::NotApplicable);
    assert_ne!(report.result, GateResult::Pass);
    assert_eq!(report.result, GateResult::NotApplicable);
    assert_ne!(report.truth, Truth::True);
    let wire = serde_json::to_value(&report).unwrap();
    assert_eq!(wire["gates"][0]["result"], "not_applicable");
    assert_eq!(wire["result"], "not_applicable");
}

#[test]
fn an_applicable_gate_evaluates_its_assertions() {
    let root = tempfile::tempdir().unwrap();
    write_gates(root.path(), json!([consumer_visible_gate()]));
    let (_, report) = run(root.path(), &story(&["consumer-visible"]));
    assert_eq!(result_of(&report, "changelog"), GateResult::Fail);
    assert_eq!(report.result, GateResult::Fail);
    fs::write(root.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
    let (_, report) = run(root.path(), &story(&["consumer-visible"]));
    assert_eq!(result_of(&report, "changelog"), GateResult::Pass);
    assert_eq!(report.result, GateResult::Pass);
    assert_eq!(report.truth, Truth::True);
}

#[test]
fn a_gate_without_applies_when_always_applies() {
    let root = tempfile::tempdir().unwrap();
    write_gates(
        root.path(),
        json!([{"id":"plain","text":"Always","assertions":["true"]}]),
    );
    let (_, report) = run(root.path(), &Admission::default());
    assert_eq!(result_of(&report, "plain"), GateResult::Pass);
    assert_eq!(report.truth, Truth::True);
}

#[test]
fn applies_when_without_story_context_is_unknown_never_not_applicable() {
    let root = tempfile::tempdir().unwrap();
    write_gates(root.path(), json!([consumer_visible_gate()]));
    let (_, report) = run(root.path(), &Admission::default());
    assert_eq!(result_of(&report, "changelog"), GateResult::Unknown);
    assert_eq!(report.result, GateResult::Unknown);
    assert_eq!(report.truth, Truth::Unknown);
}

#[test]
fn a_failing_gate_outranks_not_applicable_and_unknown_outranks_pass() {
    let root = tempfile::tempdir().unwrap();
    write_gates(
        root.path(),
        json!([
            consumer_visible_gate(),
            {"id":"broken","text":"Fails","assertions":["false"]},
            {"id":"fine","text":"Passes","assertions":["true"]},
        ]),
    );
    let (_, report) = run(root.path(), &story(&["internal"]));
    assert_eq!(result_of(&report, "changelog"), GateResult::NotApplicable);
    assert_eq!(result_of(&report, "broken"), GateResult::Fail);
    assert_eq!(result_of(&report, "fine"), GateResult::Pass);
    assert_eq!(report.result, GateResult::Fail);
    write_gates(
        root.path(),
        json!([
            consumer_visible_gate(),
            {"id":"fine","text":"Passes","assertions":["true"]},
        ]),
    );
    let (_, report) = run(root.path(), &story(&["internal"]));
    assert_eq!(report.result, GateResult::Pass);
}

#[test]
fn a_not_applicable_gates_failing_assertions_do_not_fail_the_run() {
    let root = tempfile::tempdir().unwrap();
    write_gates(
        root.path(),
        json!([{
            "id":"scoped","text":"Scoped",
            "applies_when":"story.has_tag(\"consumer-visible\")",
            "assertions":["false"],
        }]),
    );
    let (_, report) = run(root.path(), &story(&[]));
    assert_eq!(report.result, GateResult::NotApplicable);
}

#[test]
fn replay_of_retained_evidence_keeps_not_applicable_and_the_change_enters_the_identity() {
    let root = tempfile::tempdir().unwrap();
    write_gates(root.path(), json!([consumer_visible_gate()]));
    let (bundle, first) = run(root.path(), &story(&["internal"]));
    let replayed: Bundle = serde_json::from_slice(&serde_json::to_vec(&bundle).unwrap()).unwrap();
    let report = evaluate_gates(&replayed, NOW, None, None).unwrap();
    assert_eq!(report.gates, first.gates);
    let (other, _) = run(root.path(), &story(&["consumer-visible"]));
    assert_ne!(other.context_identity, bundle.context_identity);
    let mismatch = evaluate_gates(&bundle, NOW, None, Some(&other.context_identity)).unwrap();
    assert_eq!(mismatch.result, GateResult::Unknown);
}

#[test]
fn an_empty_applies_when_is_refused() {
    let root = tempfile::tempdir().unwrap();
    write_gates(
        root.path(),
        json!([{"id":"x","text":"x","applies_when":"  ","assertions":["true"]}]),
    );
    assert!(
        prepare_with(
            root.path(),
            Path::new("gates.yaml"),
            None,
            None,
            &Admission::default()
        )
        .is_err()
    );
}
