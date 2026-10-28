use assertion_providers::{CommandBinding, ExternalProvider};
use canon_engineering_assertions::{
    Document, EssPresence, Gate, Project, collect, digest, evaluate, prepare,
};
use canon_expr::{Catalog, Function, Implementation, Provider, Truth, Type};
use std::{collections::BTreeMap, fs, process::Command};

#[test]
fn registered_rust_provider_extends_namespace_and_replay_does_not_execute_it() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".engineering/assertions")).unwrap();
    let source = root.path().join("provider.rs");
    fs::write(&source,r##"use std::io::Read;
fn main(){let mut input=String::new();std::io::stdin().read_to_string(&mut input).unwrap();
assert!(input.contains("engineering-provider-request/1"));
std::fs::write(".engineering/assertions/provider-ran", "yes").unwrap();
println!(r#"{{"format":"engineering-provider-response/1","status":"known","value_json":"true"}}"#);}"##).unwrap();
    let executable = root.path().join("provider");
    assert!(
        Command::new("rustc")
            .args(["--edition=2024", "--crate-name", "test_provider"])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let sha = digest(&fs::read(&executable).unwrap());
    let function = Function {
        parameters: vec![],
        returns: Type::Bool,
        implementation: Implementation::Observation(Provider {
            provider: "example.rust/1".into(),
            operation: "acme.ready".into(),
            digest: sha.clone(),
        }),
    };
    let catalog = Catalog {
        format: "canon-catalog/1".into(),
        functions: BTreeMap::from([("acme.ready".into(), function)]),
    };
    fs::write(
        root.path().join("custom.yaml"),
        serde_yaml_ng::to_string(&catalog).unwrap(),
    )
    .unwrap();
    let mut project = Project::default();
    project.catalogs.push("custom.yaml".into());
    project.providers.insert(
        "acme.ready".into(),
        ExternalProvider {
            identity: "example.rust/1".into(),
            sha256: sha,
            command: CommandBinding {
                program: None,
                argv0: None,
                executable: "provider".into(),
                args: vec![],
                env: BTreeMap::new(),
            },
        },
    );
    fs::write(
        root.path().join("project.yaml"),
        serde_yaml_ng::to_string(&project).unwrap(),
    )
    .unwrap();
    let document = Document {
        format: "engineering-gates/1".into(),
        language: "canon-expr/1".into(),
        gates: vec![Box::new(Gate {
            id: "external".into(),
            text: "The registered provider confirms readiness".into(),
            applies_when: EssPresence::Absent,
            assertions: vec!["acme.ready()".into()],
        })],
    };
    fs::write(
        root.path().join("gates.yaml"),
        serde_yaml_ng::to_string(&document).unwrap(),
    )
    .unwrap();
    let prepared = prepare(
        root.path(),
        "gates.yaml".as_ref(),
        Some("project.yaml".as_ref()),
        None,
    )
    .unwrap();
    assert!(
        !root
            .path()
            .join(".engineering/assertions/provider-ran")
            .exists()
    );
    let bundle = collect(prepared, 100).unwrap();
    let report = evaluate(&bundle, 100, None, None).unwrap();
    assert_eq!(report.truth, Truth::True);
    let marker = root.path().join(".engineering/assertions/provider-ran");
    assert!(marker.exists());
    fs::remove_file(&marker).unwrap();
    let encoded = serde_json::to_vec(&bundle).unwrap();
    let restored = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&evaluate(&restored, 100, None, None).unwrap()).unwrap()
    );
    assert!(!marker.exists());
}

#[test]
fn inventory_assertion_detects_removed_tests_and_generated_extra_files() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("expected")).unwrap();
    fs::create_dir(root.path().join("actual")).unwrap();
    fs::write(root.path().join("expected/a"), "same").unwrap();
    fs::write(root.path().join("actual/a"), "same").unwrap();
    let document = Document {
        format: "engineering-gates/1".into(),
        language: "canon-expr/1".into(),
        gates: vec![Box::new(Gate {
            id: "complete".into(),
            text: "Both named tests and the generated tree match".into(),
            applies_when: EssPresence::Absent,
            assertions: vec![
                "tests.passed(\"tests.json\")".into(),
                "tests.report(\"tests.json\").inventory == [\"a\", \"b\"]".into(),
                "generated.matches(\"expected\", \"actual\")".into(),
            ],
        })],
    };
    fs::write(
        root.path().join("gates.yaml"),
        serde_yaml_ng::to_string(&document).unwrap(),
    )
    .unwrap();
    let report = |inventory: &[&str]| serde_json::json!({"format":"engineering-tests/1","inventory":inventory,"passed":inventory.len(),"failed":0,"skipped":0});
    fs::write(
        root.path().join("tests.json"),
        serde_json::to_vec(&report(&["a", "b"])).unwrap(),
    )
    .unwrap();
    let run = || {
        evaluate(
            &collect(
                prepare(root.path(), "gates.yaml".as_ref(), None, None).unwrap(),
                100,
            )
            .unwrap(),
            100,
            None,
            None,
        )
        .unwrap()
    };
    assert_eq!(run().truth, Truth::True);
    fs::write(
        root.path().join("tests.json"),
        serde_json::to_vec(&report(&["a"])).unwrap(),
    )
    .unwrap();
    assert_eq!(run().truth, Truth::False);
    fs::write(
        root.path().join("tests.json"),
        serde_json::to_vec(&report(&["a", "b"])).unwrap(),
    )
    .unwrap();
    fs::write(root.path().join("actual/extra"), "extra").unwrap();
    assert_eq!(run().truth, Truth::False);
}
