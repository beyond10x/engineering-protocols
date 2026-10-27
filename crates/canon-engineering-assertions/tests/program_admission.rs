use assertion_providers::CommandBinding;
use canon_engineering_assertions::{
    Admission, Bundle, Project, collect, digest, evaluate_gates, prepare_with,
};
use std::{collections::BTreeMap, fs, path::Path};

fn gates(root: &Path) {
    fs::write(
        root.join("gates.yaml"),
        r#"{"format":"engineering-gates/1","language":"canon-expr/1","gates":[{"id":"g","text":"t","assertions":["process.succeeded(\"tool\")"]}]}"#,
    )
    .unwrap();
}

fn project(root: &Path, search_path: Vec<std::path::PathBuf>) {
    let project = Project {
        search_path,
        commands: BTreeMap::from([(
            "tool".into(),
            CommandBinding {
                executable: Default::default(),
                program: Some("tool".into()),
                args: vec![],
                env: BTreeMap::new(),
            },
        )]),
        ..Project::default()
    };
    fs::write(
        root.join("project.yaml"),
        serde_yaml_ng::to_string(&project).unwrap(),
    )
    .unwrap();
}

fn prepare(
    root: &Path,
    admission: &Admission,
) -> anyhow::Result<canon_engineering_assertions::Prepared> {
    prepare_with(
        root,
        Path::new("gates.yaml"),
        Some(Path::new("project.yaml")),
        None,
        admission,
    )
}

#[test]
fn the_runtime_default_search_path_resolves_and_the_evidence_records_it() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    fs::copy("/usr/bin/true", tools.path().join("tool")).unwrap();
    gates(root.path());
    project(root.path(), vec![]);
    let admission = Admission {
        search_path: vec![tools.path().into()],
        ..Admission::default()
    };
    let bundle = collect(prepare(root.path(), &admission).unwrap(), 1000).unwrap();
    assert_eq!(bundle.programs.len(), 1);
    let program = &bundle.programs[0];
    assert_eq!(program.binding, "command:tool");
    assert_eq!(program.program, "tool");
    assert_eq!(
        program.path,
        tools.path().canonicalize().unwrap().join("tool")
    );
    assert_eq!(
        program.sha256,
        digest(&fs::read(tools.path().join("tool")).unwrap())
    );
    assert_eq!(
        evaluate_gates(&bundle, 1000, None, None).unwrap().result,
        canon_engineering_assertions::GateResult::Pass
    );
    let wire = serde_json::to_value(&bundle).unwrap();
    assert_eq!(wire["programs"][0]["sha256"], program.sha256);
    let mut older = wire.clone();
    older.as_object_mut().unwrap().remove("programs");
    let replayed: Bundle = serde_json::from_value(older).unwrap();
    assert!(
        replayed.programs.is_empty(),
        "evidence before programs still replays"
    );
}

#[test]
fn a_changed_program_cannot_reuse_the_admission() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    fs::copy("/usr/bin/true", tools.path().join("tool")).unwrap();
    gates(root.path());
    project(root.path(), vec![tools.path().into()]);
    let prepared = prepare(root.path(), &Admission::default()).unwrap();
    fs::copy("/usr/bin/false", tools.path().join("tool")).unwrap();
    let error = collect(prepared, 1000)
        .err()
        .expect("the swapped program is refused");
    assert!(
        error.to_string().contains("changed after planning"),
        "{error}"
    );
}

#[test]
fn no_search_path_and_no_runtime_default_refuses_the_program_at_admission() {
    let root = tempfile::tempdir().unwrap();
    gates(root.path());
    project(root.path(), vec![]);
    let error = prepare(root.path(), &Admission::default())
        .err()
        .expect("refused");
    assert!(error.to_string().contains("tool"), "{error}");
}

#[test]
fn a_search_path_inside_the_tree_is_refused_at_admission() {
    let root = tempfile::tempdir().unwrap();
    let inside = root.path().canonicalize().unwrap().join("bin");
    fs::create_dir(&inside).unwrap();
    fs::copy("/usr/bin/true", inside.join("tool")).unwrap();
    gates(root.path());
    project(root.path(), vec![inside.clone()]);
    let error = prepare(root.path(), &Admission::default())
        .err()
        .expect("refused");
    assert!(
        error.to_string().contains(&inside.display().to_string()),
        "{error}"
    );
}
