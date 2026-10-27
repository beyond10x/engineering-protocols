use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

#[test]
fn documented_workflow_uses_default_paths() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join(".engineering")).unwrap();
    fs::write(
        root.path().join(".engineering/gates.yaml"),
        include_str!("../../../fixtures/assertions/gates.yaml"),
    )
    .unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname = \"example\"\n",
    )
    .unwrap();
    fs::write(root.path().join("README.md"), "Apache-2.0").unwrap();
    assert_eq!(
        good(&cli(root.path(), &["gates", "validate"]))["valid"],
        true
    );
    good(&cli(root.path(), &["gates", "explain"]));
    let run = cli(root.path(), &["gates", "run"]);
    good(&run);
    let replay = cli(
        root.path(),
        &[
            "gates",
            "evaluate",
            "--evidence",
            ".engineering/assertions/evidence.json",
        ],
    );
    good(&replay);
    assert_eq!(run.stdout, replay.stdout);
}
fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon-engineering"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}
fn gates(root: &Path, expressions: &[&str]) {
    fs::write(root.join("gates.yaml"),serde_json::to_vec(&json!({"format":"engineering-gates/1","language":"canon-expr/1","gates":[{"id":"ready","text":"Ready to finish","assertions":expressions}]})).unwrap()).unwrap();
}
fn good(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn retained_evidence_replays_without_reading_sources_and_expires() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("README.md"), "hello").unwrap();
    gates(
        root.path(),
        &[
            "file_exists(\"README.md\")",
            "text.contains(\"README.md\", \"hello\")",
        ],
    );
    let first = cli(
        root.path(),
        &["gates", "run", "--file", "gates.yaml", "--now", "1000"],
    );
    assert_eq!(good(&first)["truth"], "true");
    fs::remove_file(root.path().join("README.md")).unwrap();
    fs::remove_file(root.path().join("gates.yaml")).unwrap();
    let replay = cli(
        root.path(),
        &[
            "gates",
            "evaluate",
            "--evidence",
            ".engineering/assertions/evidence.json",
        ],
    );
    good(&replay);
    assert_eq!(first.stdout, replay.stdout);
    let expired = cli(
        root.path(),
        &[
            "gates",
            "evaluate",
            "--evidence",
            ".engineering/assertions/evidence.json",
            "--now",
            "1301",
        ],
    );
    assert_eq!(expired.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&expired.stdout).unwrap()["truth"],
        "unknown"
    );
    let changed = cli(
        root.path(),
        &[
            "gates",
            "evaluate",
            "--evidence",
            ".engineering/assertions/evidence.json",
            "--source-identity",
            "different",
        ],
    );
    assert_eq!(changed.status.code(), Some(3));
}
#[test]
fn recipes_deduplicate_and_empty_or_invalid_assertions_never_certify() {
    let root = tempfile::tempdir().unwrap();
    gates(
        root.path(),
        &["file_exists(\"README.md\") and fs.file.exists(\"README.md\")"],
    );
    assert_eq!(
        good(&cli(
            root.path(),
            &["gates", "validate", "--file", "gates.yaml"]
        ))["requests"],
        1
    );
    for expression in [
        "fs.file.exists(42)",
        "fs.file.exists(\"x\").imaginary",
        "true or undefined()",
        "1 < 2 < 3",
    ] {
        gates(root.path(), &[expression]);
        let output = cli(root.path(), &["gates", "validate", "--file", "gates.yaml"]);
        assert_eq!(output.status.code(), Some(2), "{expression}");
    }
    gates(root.path(), &[]);
    assert_eq!(
        cli(root.path(), &["gates", "run", "--file", "gates.yaml"])
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn known_negative_is_false_and_bad_evidence_is_a_hard_error() {
    let root = tempfile::tempdir().unwrap();
    gates(root.path(), &["fs.file.exists(\"missing\")"]);
    let output = cli(
        root.path(),
        &["gates", "run", "--file", "gates.yaml", "--now", "10"],
    );
    assert_eq!(output.status.code(), Some(1));
    let path = root.path().join(".engineering/assertions/evidence.json");
    let mut bundle: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    bundle["observations"][0]["outcome"] =
        json!({"kind":"known","value":{"kind":"string","value":"false"}});
    fs::write(path, serde_json::to_vec(&bundle).unwrap()).unwrap();
    assert_eq!(
        cli(
            root.path(),
            &[
                "gates",
                "evaluate",
                "--evidence",
                ".engineering/assertions/evidence.json"
            ]
        )
        .status
        .code(),
        Some(2)
    );
}
#[test]
fn gate_source_cannot_be_replaced_by_evidence_output() {
    let root = tempfile::tempdir().unwrap();
    gates(root.path(), &["true"]);
    let before = fs::read(root.path().join("gates.yaml")).unwrap();
    assert_eq!(
        cli(
            root.path(),
            &[
                "gates",
                "run",
                "--file",
                "gates.yaml",
                "--out",
                "./gates.yaml"
            ]
        )
        .status
        .code(),
        Some(2)
    );
    assert_eq!(before, fs::read(root.path().join("gates.yaml")).unwrap());
}

/// A fixture builds a repository from `repository.commits` (each commit writes its `files`), then
/// runs the gates document for every case with the commits named by index as base and subject. The
/// result, each gate's result and the exit code must match `expect`, and replaying the retained
/// evidence must print the same report.
fn run_fixture(text: &str) {
    let fixture: Value = serde_yaml_ng::from_str(text).unwrap();
    assert_eq!(fixture["format"], "engineering-gates-fixture/1");
    let root = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| -> String {
        let output = Command::new("git")
            .current_dir(root.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    git(&["init", "-q"]);
    let mut oids = Vec::new();
    for commit in fixture["repository"]["commits"].as_array().unwrap() {
        for (path, content) in commit["files"].as_object().unwrap() {
            fs::write(root.path().join(path), content.as_str().unwrap()).unwrap();
        }
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "fixture commit"]);
        oids.push(git(&["rev-parse", "HEAD"]));
    }
    fs::write(
        root.path().join("gates.yaml"),
        serde_json::to_vec(&fixture["gates"]).unwrap(),
    )
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let mut args = vec![
            "gates".to_owned(),
            "run".into(),
            "--file".into(),
            "gates.yaml".into(),
            "--now".into(),
            "1000".into(),
        ];
        for role in ["base", "subject"] {
            if let Some(index) = case[role].as_u64() {
                args.extend([format!("--{role}"), oids[index as usize].clone()]);
            }
        }
        if let Some(story) = case.get("story") {
            args.extend(["--story-id".into(), story["id"].as_str().unwrap().into()]);
            for tag in story["tags"].as_array().unwrap() {
                args.extend(["--story-tag".into(), tag.as_str().unwrap().into()]);
            }
        }
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let run = cli(root.path(), &args);
        let report: Value = serde_json::from_slice(&run.stdout)
            .unwrap_or_else(|e| panic!("{name}: {e}: {}", String::from_utf8_lossy(&run.stderr)));
        let expect = &case["expect"];
        assert_eq!(report["result"], expect["result"], "{name}: result");
        for (id, result) in expect["gates"].as_object().unwrap() {
            let gate = report["gates"]
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["id"] == *id)
                .unwrap_or_else(|| panic!("{name}: no gate {id}"));
            assert_eq!(gate["result"], *result, "{name}: gate {id}");
        }
        let code = match expect["result"].as_str().unwrap() {
            "pass" => 0,
            "fail" => 1,
            "unknown" => 3,
            "not_applicable" => 4,
            other => panic!("{name}: unknown expected result {other}"),
        };
        assert_eq!(run.status.code(), Some(code), "{name}: exit code");
        let replay = cli(
            root.path(),
            &[
                "gates",
                "evaluate",
                "--evidence",
                ".engineering/assertions/evidence.json",
            ],
        );
        assert_eq!(run.stdout, replay.stdout, "{name}: replay");
    }
}

#[test]
fn git_changed_fixture() {
    run_fixture(include_str!(
        "../../../fixtures/assertions/git-changed.fixture.yaml"
    ));
}

#[test]
fn applies_when_fixture() {
    run_fixture(include_str!(
        "../../../fixtures/assertions/applies-when.fixture.yaml"
    ));
}
