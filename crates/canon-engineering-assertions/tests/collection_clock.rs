#[cfg(unix)]
#[test]
fn live_run_uses_completion_time_for_freshness() {
    use assertion_providers::CommandBinding;
    use canon_engineering_assertions::{
        Project,
        cli::{Gates, Input, gates},
    };
    use std::{collections::BTreeMap, fs};
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("gates.yaml"),
        r#"format: engineering-gates/1
language: canon-expr/1
gates:
  - id: slow
    text: A slow observation must still be fresh when collection completes.
    assertions: ['process.succeeded("slow")']
"#,
    )
    .unwrap();
    let mut project = Project {
        freshness_seconds: 1,
        ..Project::default()
    };
    project.commands.insert(
        "slow".into(),
        CommandBinding {
            program: None,
            executable: "/usr/bin/sleep".into(),
            args: vec!["2".into()],
            env: BTreeMap::new(),
        },
    );
    fs::write(
        root.path().join("project.yaml"),
        serde_yaml_ng::to_string(&project).unwrap(),
    )
    .unwrap();
    let (_, exit) = gates(Gates::Run {
        input: Input {
            file: "gates.yaml".into(),
            root: root.path().into(),
            project: Some("project.yaml".into()),
            ..Input::default()
        },
        out: ".engineering/assertions/evidence.json".into(),
        now: None,
    })
    .unwrap();
    assert_eq!(exit, 3, "completion after expiration must be UNKNOWN");
}
