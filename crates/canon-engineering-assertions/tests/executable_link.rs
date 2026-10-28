#![cfg(unix)]

#[path = "support/argv0.rs"]
mod argv0;

use assertion_providers::CommandBinding;
use canon_engineering_assertions::{
    Admission, GateResult, Project, collect, evaluate_gates, prepare_with,
};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn fixture(root: &Path, link: &Path) {
    let expression = format!("process.run(\"tool\").stdout == \"{}\"", link.display());
    fs::write(
        root.join("gates.yaml"),
        serde_json::to_string(&serde_json::json!({
            "format": "engineering-gates/1",
            "language": "canon-expr/1",
            "gates": [{"id": "g", "text": "the link runs under its own name", "assertions": [expression]}],
        }))
        .unwrap(),
    )
    .unwrap();
    let project = Project {
        commands: BTreeMap::from([(
            "tool".into(),
            CommandBinding {
                executable: link.to_owned(),
                program: None,
                argv0: None,
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

fn prepare(root: &Path) -> anyhow::Result<canon_engineering_assertions::Prepared> {
    prepare_with(
        root,
        Path::new("gates.yaml"),
        Some(Path::new("project.yaml")),
        None,
        &Admission::default(),
    )
}

#[test]
fn an_executable_link_runs_under_its_own_name() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let multiplexer = tools.join("multiplexer");
    argv0::argv0_printer(&multiplexer);
    let link = tools.join("tool");
    symlink(&multiplexer, &link).unwrap();
    fixture(root.path(), &link);

    let bundle = collect(prepare(root.path()).unwrap(), 1000).unwrap();
    let observed = serde_json::to_string(&bundle.observations).unwrap();
    assert!(
        observed.contains(&format!(
            "\"stdout\":{{\"kind\":\"string\",\"value\":\"{}\"}}",
            link.display()
        )),
        "the command ran under another name than the link it was admitted under: {observed}"
    );
    assert_eq!(
        evaluate_gates(&bundle, 1000, None, None).unwrap().result,
        GateResult::Pass,
        "{observed}"
    );

    let prepared = prepare(root.path()).unwrap();
    let other = tools.join("other");
    script(&other, "printf '%s' other");
    fs::remove_file(&link).unwrap();
    symlink(&other, &link).unwrap();
    let error = collect(prepared, 1000)
        .err()
        .expect("a link pointed at another program after admission is refused");
    assert!(
        error.to_string().contains("changed after planning"),
        "{error}"
    );

    fs::remove_file(&link).unwrap();
    symlink(&multiplexer, &link).unwrap();
    let prepared = prepare(root.path()).unwrap();
    script(&multiplexer, "printf '%s' changed");
    let error = collect(prepared, 1000)
        .err()
        .expect("a link whose target changed after admission is refused");
    assert!(
        error.to_string().contains("changed after planning"),
        "{error}"
    );
}
