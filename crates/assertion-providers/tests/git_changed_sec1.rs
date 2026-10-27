//! Security conformance: repository configuration must not make `git.changed` start a program.
use assertion_providers::{Change, ProviderContext, collect};
use serde_json::json;
use std::{collections::BTreeMap, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

/// A repository whose `.git/config` declares a promisor remote and whose subject tree object is
/// missing: `diff-tree` then lazily fetches, and the fetch starts `remote.origin.uploadpack`
/// through a shell. Nothing in the committed tree is involved; only repository configuration.
#[test]
fn a_promisor_remote_in_the_repository_configuration_cannot_run_a_program() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let marker = outside.path().join("ran");
    let r = root.path();
    git(r, &["init", "-q"]);
    std::fs::write(r.join("a.txt"), "a").unwrap();
    git(r, &["add", "-A"]);
    git(r, &["commit", "-q", "-m", "one"]);
    let base = git(r, &["rev-parse", "HEAD"]);
    std::fs::create_dir(r.join("docs")).unwrap();
    std::fs::write(r.join("docs/guide.md"), "guide").unwrap();
    git(r, &["add", "-A"]);
    git(r, &["commit", "-q", "-m", "two"]);
    let subject = git(r, &["rev-parse", "HEAD"]);
    let tree = git(r, &["rev-parse", "HEAD:docs"]);
    std::fs::remove_file(r.join(".git/objects").join(&tree[..2]).join(&tree[2..])).unwrap();
    git(r, &["config", "core.repositoryformatversion", "1"]);
    git(r, &["config", "extensions.partialClone", "origin"]);
    git(r, &["config", "remote.origin.promisor", "true"]);
    let url = outside.path().join("nowhere");
    git(r, &["config", "remote.origin.url", url.to_str().unwrap()]);
    let program = format!("/usr/bin/touch {}; exit 1 #", marker.display());
    git(r, &["config", "remote.origin.uploadpack", &program]);

    let mut context = ProviderContext::new(r);
    context.change = Change {
        base: Some(base),
        subject: Some(subject),
        ..Change::default()
    };
    let result = collect(
        "git.changed",
        &BTreeMap::from([("glob".into(), json!("docs/**"))]),
        &context,
    );
    assert!(
        !marker.exists(),
        "repository configuration ran a program during git.changed (result: {result:?})"
    );
}
