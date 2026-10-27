//! Security conformance: a search_path entry that names the tree by another spelling is refused.
use assertion_providers::{CommandBinding, ProviderError, admit_command};
use std::{collections::BTreeMap, fs, path::PathBuf};

fn program(name: &str) -> CommandBinding {
    CommandBinding {
        executable: PathBuf::new(),
        program: Some(name.into()),
        args: vec![],
        env: BTreeMap::new(),
    }
}

fn refused(result: Result<impl std::fmt::Debug, ProviderError>) -> String {
    match result {
        Err(ProviderError::Invalid(message)) => message,
        other => panic!("expected an admission refusal, got {other:?}"),
    }
}

#[test]
fn a_dot_dot_entry_that_lands_in_the_tree_is_refused() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("repo");
    let sibling = parent.path().join("tools");
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::create_dir_all(&sibling).unwrap();
    fs::copy("/usr/bin/true", root.join("bin/tool")).unwrap();
    let entry = sibling.join("..").join("repo").join("bin");
    let mut binding = program("tool");
    let message = refused(admit_command(&mut binding, &root, &[entry], &[]));
    assert!(message.contains("tree"), "{message}");
}

#[test]
fn a_proc_self_cwd_entry_that_lands_in_the_tree_is_refused() {
    let root = std::env::current_dir().unwrap();
    let entry = PathBuf::from("/proc/self/cwd/tests");
    let mut binding = program("conformance.rs");
    let message = refused(admit_command(&mut binding, &root, &[entry], &[]));
    assert!(message.contains("tree"), "{message}");
}
