//! Adversarial cases for program resolution (story `gate-commands-name-a-program-portably`).
use assertion_providers::{
    CommandBinding, ProviderError, admit_command, executable_digest, resolve_program,
};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

fn program(name: &str) -> CommandBinding {
    CommandBinding {
        executable: PathBuf::new(),
        program: Some(name.into()),
        args: vec![],
        env: BTreeMap::new(),
    }
}

fn tool_in(dir: &Path, source: &str) -> PathBuf {
    let tool = dir.join("tool");
    fs::copy(source, &tool).unwrap();
    tool
}

#[test]
fn the_first_entry_holding_an_executable_file_wins() {
    let root = tempfile::tempdir().unwrap();
    let (plain, folder, first, second) = (
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
    );
    let not_exec = plain.path().join("tool");
    fs::write(&not_exec, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&not_exec, fs::Permissions::from_mode(0o644)).unwrap();
    fs::create_dir(folder.path().join("tool")).unwrap();
    let chosen = tool_in(first.path(), "/usr/bin/true");
    tool_in(second.path(), "/usr/bin/false");
    let missing = second.path().join("no-such-dir");
    let search: Vec<PathBuf> = vec![
        missing,
        plain.path().into(),
        folder.path().into(),
        first.path().into(),
        second.path().into(),
    ];
    let resolved = resolve_program(root.path(), "tool", &search).unwrap();
    assert_eq!(
        resolved.path,
        first.path().canonicalize().unwrap().join("tool")
    );
    assert_eq!(resolved.sha256, executable_digest(&chosen).unwrap());
}

#[test]
fn a_project_search_path_replaces_the_runtime_default_without_fallback() {
    let root = tempfile::tempdir().unwrap();
    let (empty, default) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    tool_in(default.path(), "/usr/bin/true");
    let mut binding = program("tool");
    let result = admit_command(
        &mut binding,
        root.path(),
        &[empty.path().into()],
        &[default.path().into()],
    );
    assert!(
        matches!(&result, Err(ProviderError::Invalid(m)) if m.contains("tool")),
        "{result:?}"
    );
    assert!(binding.executable.as_os_str().is_empty());
}

/// A symlinked program records the path it was found at and the digest of the bytes it runs.
#[test]
fn a_symlinked_program_records_the_digest_of_its_target() {
    let root = tempfile::tempdir().unwrap();
    let (dir, real) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let target = tool_in(real.path(), "/usr/bin/true");
    std::os::unix::fs::symlink(&target, dir.path().join("tool")).unwrap();
    let resolved = resolve_program(root.path(), "tool", &[dir.path().into()]).unwrap();
    assert_eq!(resolved.sha256, executable_digest(&target).unwrap());
    assert!(resolved.path.is_absolute());
}

/// Acceptance 4: a `search_path` entry inside the repository root is refused at admission, naming
/// it. Spelled through `..` from a sibling, and not yet existing, the entry is inside the root and
/// is silently skipped instead.
#[test]
fn an_entry_inside_the_tree_spelled_through_a_sibling_is_refused() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("repo");
    let sibling = parent.path().join("sibling");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&sibling).unwrap();
    let outside = tempfile::tempdir().unwrap();
    tool_in(outside.path(), "/usr/bin/true");
    let inside = sibling.join("..").join("repo").join("bin");
    let result = resolve_program(&root, "tool", &[inside.clone(), outside.path().into()]);
    assert!(
        matches!(&result, Err(ProviderError::Invalid(m)) if m.contains("bin")),
        "an entry inside the tree must be refused, got {result:?}"
    );
}

#[test]
fn names_that_are_not_bare_are_refused() {
    let root = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    tool_in(dir.path(), "/usr/bin/true");
    for bad in ["", ".", "..", "a/tool", "/usr/bin/true", "to\0ol"] {
        assert!(
            resolve_program(root.path(), bad, &[dir.path().into()]).is_err(),
            "{bad:?}"
        );
    }
    let relative = resolve_program(root.path(), "tool", &[PathBuf::from("relative/dir")]);
    assert!(relative.is_err());
}
