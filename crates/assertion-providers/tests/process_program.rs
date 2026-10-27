use assertion_providers::{
    Collected, CommandBinding, ProviderContext, ProviderError, admit_command, collect,
    executable_digest,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// A directory outside `root` holding an executable named `tool` that prints its first argument.
fn tools() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let tool = dir.path().join("tool");
    fs::copy("/usr/bin/printf", &tool).unwrap();
    (dir, tool)
}

fn program(name: &str) -> CommandBinding {
    CommandBinding {
        executable: PathBuf::new(),
        program: Some(name.into()),
        args: vec!["ran".into()],
        env: BTreeMap::new(),
    }
}

fn run(root: &Path, binding: CommandBinding) -> Collected {
    let mut context = ProviderContext::new(root);
    context.commands.insert("c".into(), binding);
    collect(
        "process.run",
        &BTreeMap::from([("binding".into(), json!("c"))]),
        &context,
    )
    .unwrap()
}

fn refusal(result: Result<impl std::fmt::Debug, ProviderError>) -> String {
    match result {
        Err(ProviderError::Invalid(message)) => message,
        other => panic!("expected an admission refusal, got {other:?}"),
    }
}

#[test]
fn a_project_search_path_resolves_the_program_and_records_its_digest() {
    let root = tempfile::tempdir().unwrap();
    let (dir, tool) = tools();
    let mut binding = program("tool");
    let resolved = admit_command(&mut binding, root.path(), &[dir.path().into()], &[])
        .unwrap()
        .unwrap();
    let dir_path = dir.path().canonicalize().unwrap();
    assert_eq!(resolved.program, "tool");
    assert_eq!(resolved.path, dir_path.join("tool"));
    assert_eq!(resolved.sha256, executable_digest(&tool).unwrap());
    assert_eq!(binding.executable, resolved.path);
    let Collected::Known(out) = run(root.path(), binding) else {
        panic!("the resolved program runs")
    };
    assert_eq!(out["stdout"], "ran");
}

#[test]
fn a_runtime_default_search_path_is_used() {
    let root = tempfile::tempdir().unwrap();
    let (dir, _) = tools();
    let mut binding = program("tool");
    let resolved = admit_command(&mut binding, root.path(), &[], &[dir.path().into()])
        .unwrap()
        .unwrap();
    assert_eq!(
        resolved.path,
        dir.path().canonicalize().unwrap().join("tool")
    );
    let Collected::Known(out) = run(root.path(), binding) else {
        panic!("the resolved program runs")
    };
    assert_eq!(out["stdout"], "ran");
}

#[test]
fn the_project_search_path_wins_over_the_runtime_default() {
    let root = tempfile::tempdir().unwrap();
    let (project_dir, _) = tools();
    let (default_dir, _) = tools();
    let mut binding = program("tool");
    let resolved = admit_command(
        &mut binding,
        root.path(),
        &[project_dir.path().into()],
        &[default_dir.path().into()],
    )
    .unwrap()
    .unwrap();
    assert!(
        resolved
            .path
            .starts_with(project_dir.path().canonicalize().unwrap())
    );
}

#[test]
fn no_search_path_keeps_the_refusal() {
    let root = tempfile::tempdir().unwrap();
    let mut binding = program("tool");
    let message = refusal(admit_command(&mut binding, root.path(), &[], &[]));
    assert!(message.contains("tool"), "{message}");
    let unresolved = {
        let mut context = ProviderContext::new(root.path());
        context.commands.insert("c".into(), program("tool"));
        collect(
            "process.run",
            &BTreeMap::from([("binding".into(), json!("c"))]),
            &context,
        )
    };
    match unresolved {
        Err(ProviderError::Invalid(message)) => assert!(message.contains("tool"), "{message}"),
        other => panic!("an unresolved program must not run: {other:?}"),
    }
}

#[test]
fn a_program_inside_the_tree_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let inside = root.path().join("bin");
    fs::create_dir(&inside).unwrap();
    fs::copy("/usr/bin/printf", inside.join("tool")).unwrap();
    let mut binding = program("tool");
    let message = refusal(admit_command(
        &mut binding,
        root.path(),
        std::slice::from_ref(&inside),
        &[],
    ));
    assert!(message.contains(&inside.display().to_string()), "{message}");
    let mut binding = program("tool");
    let message = refusal(admit_command(
        &mut binding,
        root.path(),
        &[],
        std::slice::from_ref(&inside),
    ));
    assert!(message.contains(&inside.display().to_string()), "{message}");
}

#[test]
fn an_entry_that_is_inside_the_tree_is_refused_even_after_a_good_one() {
    let root = tempfile::tempdir().unwrap();
    let (dir, _) = tools();
    let inside = root.path().join("bin");
    fs::create_dir(&inside).unwrap();
    let mut binding = program("tool");
    let message = refusal(admit_command(
        &mut binding,
        root.path(),
        &[dir.path().into(), inside.clone()],
        &[],
    ));
    assert!(message.contains(&inside.display().to_string()), "{message}");
}

#[test]
fn a_symlink_into_the_tree_is_refused() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let inside = root.path().join("bin");
    fs::create_dir(&inside).unwrap();
    fs::copy("/usr/bin/printf", inside.join("real")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    symlink(inside.join("real"), dir.path().join("tool")).unwrap();
    let mut binding = program("tool");
    let message = refusal(admit_command(
        &mut binding,
        root.path(),
        &[dir.path().into()],
        &[],
    ));
    assert!(message.contains("tool"), "{message}");
    assert!(message.contains("tree"), "{message}");

    let linked = tempfile::tempdir().unwrap();
    let directory_link = linked.path().join("bin");
    symlink(&inside, &directory_link).unwrap();
    let mut binding = program("tool");
    let message = refusal(admit_command(
        &mut binding,
        root.path(),
        std::slice::from_ref(&directory_link),
        &[],
    ));
    assert!(
        message.contains(&directory_link.display().to_string()),
        "{message}"
    );
}

#[test]
fn a_symlink_to_a_program_outside_the_tree_is_followed() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let (real, tool) = tools();
    let dir = tempfile::tempdir().unwrap();
    symlink(&tool, dir.path().join("alias")).unwrap();
    let mut binding = program("alias");
    let resolved = admit_command(&mut binding, root.path(), &[dir.path().into()], &[])
        .unwrap()
        .unwrap();
    assert_eq!(
        resolved.path,
        dir.path().canonicalize().unwrap().join("alias")
    );
    assert_eq!(resolved.sha256, executable_digest(&tool).unwrap());
    drop(real);
}

#[test]
fn a_relative_search_path_entry_is_refused() {
    let root = tempfile::tempdir().unwrap();
    for entry in ["bin", "./bin", "../bin", ""] {
        let mut binding = program("tool");
        let message = refusal(admit_command(
            &mut binding,
            root.path(),
            &[entry.into()],
            &[],
        ));
        assert!(message.contains("absolute"), "{entry}: {message}");
    }
}

#[test]
fn a_missing_program_is_named() {
    let root = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut binding = program("tool");
    let message = refusal(admit_command(
        &mut binding,
        root.path(),
        &[dir.path().into()],
        &[],
    ));
    assert!(message.contains("tool"), "{message}");
    assert!(
        message.contains(&dir.path().display().to_string()),
        "{message}"
    );
}

#[test]
fn a_program_name_is_a_bare_name() {
    let root = tempfile::tempdir().unwrap();
    let (dir, _) = tools();
    for name in ["", ".", "..", "a/b", "../tool", "/usr/bin/printf", "to\0ol"] {
        let mut binding = program(name);
        refusal(admit_command(
            &mut binding,
            root.path(),
            &[dir.path().into()],
            &[],
        ));
    }
}

#[test]
fn a_file_without_the_execute_bit_is_not_a_match() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let (dir, tool) = tools();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o644)).unwrap();
    let mut binding = program("tool");
    let message = refusal(admit_command(
        &mut binding,
        root.path(),
        &[dir.path().into()],
        &[],
    ));
    assert!(message.contains("tool"), "{message}");
}

#[test]
fn the_first_search_path_entry_with_the_program_wins() {
    let root = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    let (first, _) = tools();
    let (second, _) = tools();
    let mut binding = program("tool");
    let resolved = admit_command(
        &mut binding,
        root.path(),
        &[
            empty.path().into(),
            first.path().into(),
            second.path().into(),
        ],
        &[],
    )
    .unwrap()
    .unwrap();
    assert!(
        resolved
            .path
            .starts_with(first.path().canonicalize().unwrap())
    );
}

#[test]
fn a_command_cannot_name_both_a_program_and_an_executable() {
    let root = tempfile::tempdir().unwrap();
    let (dir, tool) = tools();
    let mut binding = program("tool");
    binding.executable = tool;
    refusal(admit_command(
        &mut binding,
        root.path(),
        &[dir.path().into()],
        &[],
    ));
}

#[test]
fn executable_still_works() {
    let root = tempfile::tempdir().unwrap();
    let mut binding = CommandBinding {
        executable: "/usr/bin/printf".into(),
        program: None,
        args: vec!["absolute".into()],
        env: BTreeMap::new(),
    };
    assert!(
        admit_command(&mut binding, root.path(), &[], &[])
            .unwrap()
            .is_none()
    );
    assert_eq!(binding.executable, PathBuf::from("/usr/bin/printf"));
    let Collected::Known(out) = run(root.path(), binding) else {
        panic!("an absolute executable runs")
    };
    assert_eq!(out["stdout"], "absolute");
    let relative = CommandBinding {
        executable: "printf".into(),
        program: None,
        args: vec![],
        env: BTreeMap::new(),
    };
    let mut context = ProviderContext::new(root.path());
    context.commands.insert("c".into(), relative);
    assert!(
        collect(
            "process.run",
            &BTreeMap::from([("binding".into(), json!("c"))]),
            &context
        )
        .is_err(),
        "a relative executable is still refused"
    );
}

#[test]
fn the_caller_path_is_never_used() {
    let root = tempfile::tempdir().unwrap();
    let mut binding = program("printf");
    refusal(admit_command(&mut binding, root.path(), &[], &[]));
}
