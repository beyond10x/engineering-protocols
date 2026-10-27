//! Adversarial cases for gate results and the change in the evidence identity (story
//! `a-gate-sees-the-change-and-may-not-apply`).
use canon_engineering_assertions::{
    Admission, Bundle, Change, GateResult, GatesReport,
    cli::{Gates, Input, gates},
    collect, evaluate, evaluate_gates, prepare_with,
};
use canon_expr::Truth;
use serde_json::json;
use std::{fs, path::Path, process::Command};

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
        ..Admission::default()
    }
}

fn run(root: &Path, admission: &Admission) -> (Bundle, GatesReport) {
    let prepared = prepare_with(root, Path::new("gates.yaml"), None, None, admission).unwrap();
    let bundle = collect(prepared, NOW).unwrap();
    let report = evaluate_gates(&bundle, NOW, None, None).unwrap();
    (bundle, report)
}

fn scoped(id: &str, assertion: &str) -> serde_json::Value {
    json!({
        "id": id, "text": id,
        "applies_when": "story.has_tag(\"consumer-visible\")",
        "assertions": [assertion],
    })
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

/// `evaluate` is the library entry point the consumer calls today. A gate that does not apply is
/// not a failure, so its report must not say `false` for a run whose only gate is not applicable
/// and whose assertions hold.
#[test]
fn evaluate_does_not_report_a_not_applicable_gate_as_false() {
    let root = tempfile::tempdir().unwrap();
    write_gates(root.path(), json!([scoped("scoped", "true")]));
    let (bundle, gated) = run(root.path(), &story(&["internal"]));
    assert_eq!(gated.result, GateResult::NotApplicable);
    let plain = evaluate(&bundle, NOW, None, None).unwrap();
    assert_ne!(
        plain.truth,
        Truth::False,
        "evaluate() reports a not-applicable gate as a failure"
    );
}

/// The Admission doc comment: evidence collected for one change cannot be replayed as evidence for
/// another. A base named by a ref that moved is another change, with another `git.changed` result.
#[test]
fn a_base_named_by_a_ref_that_moved_is_another_context() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q", "-b", "work"]);
    fs::write(root.path().join("README.md"), "r\n").unwrap();
    git(root.path(), &["add", "-A"]);
    git(root.path(), &["commit", "-q", "-m", "one"]);
    git(root.path(), &["branch", "target"]);
    fs::write(root.path().join("CHANGELOG.md"), "entry\n").unwrap();
    git(root.path(), &["add", "-A"]);
    git(root.path(), &["commit", "-q", "-m", "two"]);
    write_gates(
        root.path(),
        json!([{"id":"note","text":"note","assertions":["git.changed(\"CHANGELOG.md\")"]}]),
    );
    let admission = Admission {
        change: Change {
            base: Some("target".into()),
            subject: Some("HEAD".into()),
            ..Change::default()
        },
        ..Admission::default()
    };
    let (first, first_report) = run(root.path(), &admission);
    assert_eq!(first_report.result, GateResult::Pass);
    git(root.path(), &["branch", "-f", "target", "HEAD"]);
    let (second, second_report) = run(root.path(), &admission);
    assert_eq!(second_report.result, GateResult::Fail);
    assert!(
        first.context_identity != second.context_identity
            || first.source_identity != second.source_identity,
        "the first run's pass replays under the second run's identities"
    );
}

/// The CLI help: "--story-tag ... giving none leaves `story.has_tag` unknown".
#[test]
fn cli_story_id_without_a_story_tag_leaves_has_tag_unknown_as_documented() {
    let root = tempfile::tempdir().unwrap();
    write_gates(root.path(), json!([scoped("scoped", "true")]));
    let input = Input {
        file: "gates.yaml".into(),
        root: root.path().into(),
        story_id: Some("story-1".into()),
        ..Input::default()
    };
    let (out, _) = gates(Gates::Run {
        input,
        out: ".engineering/assertions/evidence.json".into(),
        now: Some(NOW),
    })
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        report["gates"][0]["result"], "unknown",
        "giving no --story-tag must leave story.has_tag unknown"
    );
}

/// Aggregate combinations: all not applicable, not applicable with unknown, applicable with an
/// unknown assertion, and an applies_when that is unknown over failing assertions.
#[test]
fn aggregate_combinations() {
    let root = tempfile::tempdir().unwrap();
    write_gates(
        root.path(),
        json!([scoped("a", "true"), scoped("b", "false")]),
    );
    let (_, report) = run(root.path(), &story(&[]));
    assert_eq!(report.result, GateResult::NotApplicable);
    assert_eq!(report.truth, Truth::Unknown);
    write_gates(
        root.path(),
        json!([
            scoped("a", "true"),
            {"id":"u","text":"u","assertions":["story.id() == \"story-1\""]},
        ]),
    );
    let (_, report) = run(root.path(), &Admission::default());
    assert_eq!(report.result, GateResult::Unknown);
    let (_, report) = run(root.path(), &story(&[]));
    assert_eq!(report.result, GateResult::Pass);
    write_gates(
        root.path(),
        json!([{
            "id":"g","text":"g",
            "applies_when":"story.has_tag(\"consumer-visible\")",
            "assertions":["true", "git.changed(\"x\")"],
        }]),
    );
    let (_, report) = run(root.path(), &story(&["consumer-visible"]));
    assert_eq!(report.gates[0].result, GateResult::Unknown);
    assert_eq!(report.result, GateResult::Unknown);
    write_gates(root.path(), json!([scoped("f", "false")]));
    let (_, report) = run(root.path(), &Admission::default());
    assert_eq!(report.gates[0].result, GateResult::Unknown);
}

/// An `applies_when` that is not a boolean is refused at preparation.
#[test]
fn a_non_boolean_applies_when_is_refused() {
    let root = tempfile::tempdir().unwrap();
    write_gates(
        root.path(),
        json!([{"id":"x","text":"x","applies_when":"story.id()","assertions":["true"]}]),
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
