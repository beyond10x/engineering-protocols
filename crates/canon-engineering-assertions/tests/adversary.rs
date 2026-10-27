use assertion_providers::CommandBinding;
use canon_engineering_assertions::cli::{Gates, Input, gates};
use canon_engineering_assertions::{Document, EssPresence, Gate, Project, prepare};
use std::{collections::BTreeMap, fs};

fn fixture(root: &std::path::Path, assertions: Vec<String>, project: Project) {
    fs::create_dir_all(root.join(".engineering")).unwrap();
    let document = Document {
        format: "engineering-gates/1".into(),
        language: "canon-expr/1".into(),
        gates: vec![Box::new(Gate {
            id: "checked".into(),
            text: "All observations are checked before acceptance".into(),
            applies_when: EssPresence::Absent,
            assertions,
        })],
    };
    fs::write(
        root.join("gates.yaml"),
        serde_yaml_ng::to_string(&document).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("project.yaml"),
        serde_yaml_ng::to_string(&project).unwrap(),
    )
    .unwrap();
}

#[cfg(unix)]
#[test]
fn evidence_output_cannot_follow_symlink_created_during_collection() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let mut project = Project::default();
    project.commands.insert(
        "make-link".into(),
        CommandBinding {
            executable: "/usr/bin/ln".into(),
            args: vec![
                "-s".into(),
                outside.path().to_str().unwrap().into(),
                ".engineering/assertions".into(),
            ],
            env: BTreeMap::new(),
        },
    );
    fixture(
        root.path(),
        vec!["process.run(\"make-link\").stdout == \"\"".into()],
        project,
    );
    let result = gates(Gates::Run {
        input: Input {
            file: "gates.yaml".into(),
            root: root.path().into(),
            project: Some("project.yaml".into()),
            ..Input::default()
        },
        out: ".engineering/assertions/evidence.json".into(),
        now: Some(100),
    });
    assert!(
        result.is_err(),
        "an observation replaced the evidence directory with a symlink, but publication followed it"
    );
    assert!(!outside.path().join("evidence.json").exists());
}

#[test]
fn all_command_operations_are_preflighted_before_collection() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        vec!["process.succeeded(\"unregistered\")".into()],
        Project::default(),
    );
    assert!(
        prepare(
            root.path(),
            "gates.yaml".as_ref(),
            Some("project.yaml".as_ref()),
            None
        )
        .is_err(),
        "process.succeeded must require admitted command binding before collecting any requests"
    );
}

#[cfg(unix)]
#[test]
fn changed_admitted_command_executable_cannot_reuse_prepared_identity() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let executable = tools.path().join("tool");
    fs::copy("/usr/bin/true", &executable).unwrap();
    let mut project = Project::default();
    project.commands.insert(
        "tool".into(),
        CommandBinding {
            executable: executable.clone(),
            args: vec![],
            env: BTreeMap::new(),
        },
    );
    fixture(
        root.path(),
        vec!["not process.succeeded(\"tool\")".into()],
        project,
    );
    let prepared = prepare(
        root.path(),
        "gates.yaml".as_ref(),
        Some("project.yaml".as_ref()),
        None,
    )
    .unwrap();
    fs::copy("/usr/bin/false", &executable).unwrap();
    assert!(
        canon_engineering_assertions::collect(prepared, 100).is_err(),
        "changed command implementation collected evidence carrying the old context digest"
    );
}

#[test]
fn tcp_host_admission_is_preflighted_before_any_collection() {
    let root = tempfile::tempdir().unwrap();
    let mut project = Project::default();
    project.networks.insert(
        "known-network".into(),
        assertion_providers::NetworkBinding::default(),
    );
    fixture(
        root.path(),
        vec!["net.tcp.reachable(\"unregistered-host\", 443, \"known-network\")".into()],
        project,
    );
    assert!(
        prepare(
            root.path(),
            "gates.yaml".as_ref(),
            Some("project.yaml".as_ref()),
            None
        )
        .is_err(),
        "a registered network name does not admit an unregistered host"
    );
}

#[test]
fn explicit_generated_inputs_include_nested_build_named_directories() {
    let root = tempfile::tempdir().unwrap();
    for directory in ["expected", "actual"] {
        fs::create_dir_all(root.path().join(directory).join("target")).unwrap();
        fs::write(root.path().join(directory).join("target/output"), "before").unwrap();
    }
    fixture(
        root.path(),
        vec!["generated.matches(\"expected\", \"actual\")".into()],
        Project::default(),
    );
    let prepared = prepare(
        root.path(),
        "gates.yaml".as_ref(),
        Some("project.yaml".as_ref()),
        None,
    )
    .unwrap();
    for directory in ["expected", "actual"] {
        fs::write(root.path().join(directory).join("target/output"), "after").unwrap();
    }
    assert!(
        canon_engineering_assertions::collect(prepared, 100).is_err(),
        "explicit generated inputs changed but snapshot silently excluded nested target/ bytes"
    );
}
