#![cfg(unix)]

#[path = "support/argv0.rs"]
mod argv0;

use assertion_providers::{CommandBinding, ExternalProvider};
use canon_engineering_assertions::{
    Admission, GateResult, Prepared, Project, collect, digest, evaluate_gates, prepare_with,
};
use canon_expr::{Catalog, Function, Implementation, Provider, Type};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};

fn binding(executable: &Path) -> CommandBinding {
    CommandBinding {
        executable: executable.to_owned(),
        program: None,
        argv0: None,
        args: vec![],
        env: BTreeMap::new(),
    }
}

fn gates(root: &Path, expression: &str) {
    fs::write(
        root.join("gates.yaml"),
        serde_json::to_string(&serde_json::json!({
            "format": "engineering-gates/1",
            "language": "canon-expr/1",
            "gates": [{"id": "g", "text": "the admitted command runs as admitted", "assertions": [expression]}],
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_project(root: &Path, commands: &[(&str, &Path)]) {
    let project = Project {
        commands: commands
            .iter()
            .map(|(name, path)| ((*name).to_owned(), binding(path)))
            .collect(),
        ..Project::default()
    };
    fs::write(
        root.join("project.yaml"),
        serde_yaml_ng::to_string(&project).unwrap(),
    )
    .unwrap();
}

fn prepare(root: &Path) -> anyhow::Result<Prepared> {
    prepare_with(
        root,
        Path::new("gates.yaml"),
        Some(Path::new("project.yaml")),
        None,
        &Admission::default(),
    )
}

fn result(prepared: Prepared) -> GateResult {
    let bundle = collect(prepared, 1000).unwrap();
    evaluate_gates(&bundle, 1000, None, None).unwrap().result
}

/// Two rustup-style proxies (`cargo`, `rustfmt`) linked to one multiplexer. Swapping which
/// command names which link changes what each command runs as, so the gate result changes, but
/// the project serializes the shared target for both commands and the link map is project-wide,
/// so the context identity does not move.
#[test]
fn two_commands_that_swap_their_links_keep_one_context_identity() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let multiplexer = tools.join("multiplexer");
    argv0::argv0_printer(&multiplexer);
    let cargo = tools.join("cargo");
    let rustfmt = tools.join("rustfmt");
    symlink(&multiplexer, &cargo).unwrap();
    symlink(&multiplexer, &rustfmt).unwrap();
    gates(
        root.path(),
        &format!("process.run(\"test\").stdout == \"{}\"", cargo.display()),
    );

    write_project(root.path(), &[("test", &cargo), ("fmt", &rustfmt)]);
    let first = prepare(root.path()).unwrap();
    let first_identity = first.bindings.context_identity.clone();
    assert_eq!(result(first), GateResult::Pass);

    write_project(root.path(), &[("test", &rustfmt), ("fmt", &cargo)]);
    let second = prepare(root.path()).unwrap();
    let second_identity = second.bindings.context_identity.clone();
    assert_eq!(result(second), GateResult::Fail);

    assert_ne!(
        first_identity, second_identity,
        "story:gate-executable-identity-bound-to-spawn: `test` ran as cargo in one project and as \
         rustfmt in the other, and both carry one context identity: the link map is keyed by link, \
         not by command, so which command runs under which name is not in the identity"
    );
}

/// One command names the link, the other its target. Moving the link to the other command
/// changes which command runs under the link's name; the identity is the same.
#[test]
fn a_link_moved_to_the_command_that_named_its_target_keeps_the_context_identity() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let multiplexer = tools.join("rustup");
    argv0::argv0_printer(&multiplexer);
    let cargo = tools.join("cargo");
    symlink(&multiplexer, &cargo).unwrap();
    gates(
        root.path(),
        &format!("process.run(\"test\").stdout == \"{}\"", cargo.display()),
    );

    write_project(
        root.path(),
        &[("test", &cargo), ("toolchain", &multiplexer)],
    );
    let first = prepare(root.path()).unwrap();
    let first_identity = first.bindings.context_identity.clone();
    assert_eq!(result(first), GateResult::Pass);

    write_project(
        root.path(),
        &[("test", &multiplexer), ("toolchain", &cargo)],
    );
    let second = prepare(root.path()).unwrap();
    let second_identity = second.bindings.context_identity.clone();
    assert_eq!(result(second), GateResult::Fail);

    assert_ne!(
        first_identity, second_identity,
        "story:gate-executable-identity-bound-to-spawn: `test` ran as cargo, then as rustup, under \
         one context identity"
    );
}

/// Kills the mutant that drops `executable_links` from the identity: a link and its target named
/// directly serialize the same project and the same locks, so only the link map separates them.
#[test]
fn a_link_and_its_target_named_directly_have_different_context_identities() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let multiplexer = tools.join("rustup");
    argv0::argv0_printer(&multiplexer);
    let cargo = tools.join("cargo");
    symlink(&multiplexer, &cargo).unwrap();
    gates(root.path(), "process.succeeded(\"test\")");

    write_project(root.path(), &[("test", &cargo)]);
    let through_link = prepare(root.path()).unwrap().bindings.context_identity;
    write_project(root.path(), &[("test", &multiplexer)]);
    let direct = prepare(root.path()).unwrap().bindings.context_identity;
    assert_ne!(
        through_link, direct,
        "story:gate-executable-identity-bound-to-spawn: a link and its target share an identity"
    );
}

/// `argv0` is set by admission only. A project that writes it must be refused, for a command and
/// for an external provider's command.
#[test]
fn a_project_that_writes_argv0_is_refused() {
    let root = tempfile::tempdir().unwrap();
    gates(root.path(), "process.succeeded(\"tool\")");
    fs::write(
        root.path().join("project.yaml"),
        "format: engineering-assertions-project/1\n\
         commands:\n  tool:\n    executable: /usr/bin/true\n    argv0: /usr/bin/false\n",
    )
    .unwrap();
    let error = prepare(root.path()).err().expect(
        "story:gate-executable-identity-bound-to-spawn: a command's argv0 was read from YAML",
    );
    assert!(format!("{error:#}").contains("argv0"), "{error:#}");

    fs::write(
        root.path().join("project.yaml"),
        "format: engineering-assertions-project/1\n\
         providers:\n  acme.ready:\n    identity: x\n    sha256: x\n    command:\n      \
         executable: /usr/bin/true\n      argv0: /usr/bin/false\n",
    )
    .unwrap();
    let error = prepare(root.path()).err().expect(
        "story:gate-executable-identity-bound-to-spawn: a provider command's argv0 was read from YAML",
    );
    assert!(format!("{error:#}").contains("argv0"), "{error:#}");
}

/// The 0.11.1 form of `an_executable_link_runs_under_its_own_name`: a shell script behind a link
/// printed the link as `$0`. The script now runs from its resolved path, so the interpreter is
/// handed the target and a script that dispatches on `$0` no longer sees the name it was
/// admitted under. The commit rewrote that test to a compiled printer instead of stating this.
#[test]
fn a_script_reached_through_a_link_sees_the_link_as_its_name() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let multiplexer = tools.join("multiplexer");
    fs::write(&multiplexer, "#!/bin/sh\nprintf '%s' \"$0\"\n").unwrap();
    fs::set_permissions(&multiplexer, fs::Permissions::from_mode(0o755)).unwrap();
    let link = tools.join("tool");
    symlink(&multiplexer, &link).unwrap();
    gates(
        root.path(),
        &format!("process.run(\"tool\").stdout == \"{}\"", link.display()),
    );
    write_project(root.path(), &[("tool", &link)]);
    let bundle = collect(prepare(root.path()).unwrap(), 1000).unwrap();
    let observed = serde_json::to_string(&bundle.observations).unwrap();
    assert!(
        observed.contains(&format!(
            "\"stdout\":{{\"kind\":\"string\",\"value\":\"{}\"}}",
            link.display()
        )),
        "story:gate-executable-identity-bound-to-spawn: a script admitted through the link {} ran \
         under its target's name (0.11.1 ran it as the link): {observed}",
        link.display()
    );
}

const PROVIDER: &str = r##"use std::io::{Read, Write};
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let name = std::env::args_os().next().unwrap_or_default();
    let expected = std::env::var_os("EXPECT").unwrap_or_default();
    let value = if name == expected { "true" } else { "false" };
    let response = format!(
        "{{\"format\":\"engineering-provider-response/1\",\"status\":\"known\",\"value_json\":\"{value}\"}}"
    );
    std::io::stdout().write_all(response.as_bytes()).unwrap();
}
"##;

fn provider_binary(dir: &Path) -> PathBuf {
    let source = dir.join("provider.rs");
    fs::write(&source, PROVIDER).unwrap();
    let binary = dir.join("provider-real");
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    assert!(
        Command::new(rustc)
            .args(["--edition=2021", "--crate-name", "argv0_provider", "-o"])
            .arg(&binary)
            .arg(&source)
            .status()
            .unwrap()
            .success()
    );
    binary
}

fn provider_fixture(root: &Path, link: &Path, sha: &str) {
    let catalog = Catalog {
        format: "canon-catalog/1".into(),
        functions: BTreeMap::from([(
            "acme.named".into(),
            Function {
                parameters: vec![],
                returns: Type::Bool,
                implementation: Implementation::Observation(Provider {
                    provider: "example.argv0/1".into(),
                    operation: "acme.named".into(),
                    digest: sha.to_owned(),
                }),
            },
        )]),
    };
    fs::write(
        root.join("custom.yaml"),
        serde_yaml_ng::to_string(&catalog).unwrap(),
    )
    .unwrap();
    let mut command = binding(link);
    command
        .env
        .insert("EXPECT".into(), link.to_string_lossy().into_owned());
    let mut project = Project::default();
    project.catalogs.push("custom.yaml".into());
    project.providers.insert(
        "acme.named".into(),
        ExternalProvider {
            identity: "example.argv0/1".into(),
            sha256: sha.to_owned(),
            command,
        },
    );
    fs::write(
        root.join("project.yaml"),
        serde_yaml_ng::to_string(&project).unwrap(),
    )
    .unwrap();
    gates(root, "acme.named()");
}

/// An external provider admitted through a link: its `sha256` is the target's digest, it runs
/// under the link's name, and a retargeted link refuses collection.
#[test]
fn an_external_provider_through_a_link_runs_under_the_links_name() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let real = provider_binary(&tools);
    let sha = digest(&fs::read(&real).unwrap());
    let link = tools.join("provider");
    symlink(&real, &link).unwrap();
    provider_fixture(root.path(), &link, &sha);
    assert_eq!(
        result(prepare(root.path()).unwrap()),
        GateResult::Pass,
        "story:gate-executable-identity-bound-to-spawn: the provider did not run as {}",
        link.display()
    );

    let prepared = prepare(root.path()).unwrap();
    let copy = tools.join("provider-copy");
    fs::copy(&real, &copy).unwrap();
    fs::remove_file(&link).unwrap();
    symlink(&copy, &link).unwrap();
    let error = collect(prepared, 1000).err().expect(
        "story:gate-executable-identity-bound-to-spawn: a retargeted provider link was collected",
    );
    assert!(
        error.to_string().contains("changed after planning"),
        "{error}"
    );
}
