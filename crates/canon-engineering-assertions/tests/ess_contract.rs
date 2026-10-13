use std::{fs, path::Path, process::Command};

// Document/Gate are generated. Normalized project/runtime envelopes retain ergonomic native
// PathBuf, IpAddr and the checked Canon Plan; this guard compares every native field/type.
#[test]
fn normalized_runtime_models_match_the_ess_field_types() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let yaml: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&fs::read_to_string(root.join("ess/domains/gates.yaml")).unwrap())
            .unwrap();
    let mut actual = std::collections::BTreeMap::new();
    for path in [
        "crates/canon-engineering-assertions/src/lib.rs",
        "crates/assertion-providers/src/lib.rs",
    ] {
        let parsed = syn::parse_file(&fs::read_to_string(root.join(path)).unwrap()).unwrap();
        for item in parsed.items {
            if let syn::Item::Struct(item) = item {
                let name = item.ident.to_string();
                if [
                    "Project",
                    "Bundle",
                    "CommandBinding",
                    "ExternalProvider",
                    "NetworkBinding",
                ]
                .contains(&name.as_str())
                {
                    let fields = item
                        .fields
                        .iter()
                        .map(|f| (f.ident.as_ref().unwrap().to_string(), wire_type(&f.ty)))
                        .collect::<std::collections::BTreeMap<_, _>>();
                    actual.insert(format!("engineering.gates.{name}"), fields);
                }
            }
        }
    }
    assert_eq!(actual.len(), 5, "every native envelope must be compared");
    let mut expected = std::collections::BTreeMap::new();
    for ty in yaml["types"].as_sequence().unwrap() {
        let name = ty["name"].as_str().unwrap();
        if actual.contains_key(name) {
            expected.insert(
                name.to_owned(),
                ty["fields"]
                    .as_sequence()
                    .unwrap()
                    .iter()
                    .map(|f| {
                        (
                            f["name"].as_str().unwrap().to_owned(),
                            f["type"].as_str().unwrap().replace(' ', ""),
                        )
                    })
                    .collect::<std::collections::BTreeMap<_, _>>(),
            );
        }
    }
    assert_eq!(actual, expected);
}
fn wire_type(ty: &syn::Type) -> String {
    let syn::Type::Path(path) = ty else {
        panic!("unmapped native type")
    };
    let part = path.path.segments.last().unwrap();
    let name = part.ident.to_string();
    let args = match &part.arguments {
        syn::PathArguments::AngleBracketed(args) => args
            .args
            .iter()
            .map(|arg| match arg {
                syn::GenericArgument::Type(ty) => wire_type(ty),
                _ => panic!("unmapped type argument"),
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };
    match name.as_str() {
        "String" | "PathBuf" | "IpAddr" => "String".into(),
        "u64" | "i64" | "usize" => "Integer".into(),
        "Vec" => format!("List<{}>", args.join(",")),
        "BTreeMap" => format!("Map<{}>", args.join(",")),
        "Value" => "engineering.expr.Value".into(),
        "Plan" => "engineering.expr.PlanDocument".into(),
        "Observation" => "engineering.expr.Observation".into(),
        _ => format!("engineering.gates.{name}"),
    }
}

#[test]
fn gate_spec_compiles_synthesizes_and_generated_contract_has_no_drift() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = tempfile::tempdir().unwrap();
    for (name, args) in [
        ("validate", vec!["specify", "validate", "--strict-requires"]),
        ("compile", vec!["specify", "compile"]),
        ("synthesize", vec!["verify", "conform", "synthesize"]),
    ] {
        let mut command = Command::new("ess");
        command
            .current_dir(&root)
            .args(args)
            .args(["--path", "ess"]);
        if name != "validate" {
            command.arg("--out").arg(output.path().join(name));
        }
        let result = command
            .output()
            .expect("ESS 0.59.0 is required for the repository gate");
        assert!(
            result.status.success(),
            "{name}: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        if name == "synthesize" {
            assert!(
                String::from_utf8_lossy(&result.stdout).contains("0 refusal(s)"),
                "{}",
                String::from_utf8_lossy(&result.stdout)
            );
        }
    }
    assert_eq!(
        fs::read_to_string(root.join("ess/domains/expr.yaml")).unwrap(),
        canon_engineering_assertions::documentation::expression_model()
    );
    let generated = output.path().join("generated");
    let result = Command::new("ess")
        .current_dir(&root)
        .args([
            "generate",
            "types",
            "--path",
            "ess",
            "--root",
            "engineering.gates.Document",
            "--target",
            "rust",
            "--package",
            "b10x-engineering-gates-contract",
            "--names",
            "short",
            "--strict-requires",
            "--out",
        ])
        .arg(&generated)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for file in [
        "Cargo.toml",
        "types.rs",
        "types-report.json",
        "source.schema.json",
    ] {
        assert_eq!(
            fs::read(generated.join(file)).unwrap(),
            fs::read(
                root.join("crates/canon-engineering-assertions/generated")
                    .join(file)
            )
            .unwrap(),
            "generated {file} drift"
        );
    }
}

#[test]
fn provider_contract_validates_and_generated_types_have_no_drift() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = tempfile::tempdir().unwrap();
    for (name, args) in [
        ("validate", vec!["specify", "validate", "--strict-requires"]),
        ("compile", vec!["specify", "compile"]),
        ("synthesize", vec!["verify", "conform", "synthesize"]),
    ] {
        let mut command = Command::new("ess");
        command
            .current_dir(&root)
            .args(args)
            .args(["--path", "ess/providers"]);
        if name != "validate" {
            command.arg("--out").arg(output.path().join(name));
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        if name == "synthesize" {
            assert!(String::from_utf8_lossy(&result.stdout).contains("0 refusal(s)"));
        }
    }
    let generated = output.path().join("generated");
    let result = Command::new("ess")
        .current_dir(&root)
        .args([
            "generate",
            "types",
            "--path",
            "ess/providers",
            "--all-types",
            "--target",
            "rust",
            "--package",
            "b10x-assertion-provider-contract",
            "--names",
            "short",
            "--strict-requires",
            "--out",
        ])
        .arg(&generated)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for file in [
        "Cargo.toml",
        "types.rs",
        "types-report.json",
        "source.schema.json",
    ] {
        assert_eq!(
            fs::read(generated.join(file)).unwrap(),
            fs::read(root.join("crates/assertion-providers/generated").join(file)).unwrap(),
            "provider generated {file} drift"
        );
    }
}
