//! Adversarial cases for `git.changed` (story `a-gate-sees-the-change-and-may-not-apply`).
use assertion_providers::{Change, Collected, ProviderContext, ProviderError, collect};
use serde_json::json;
use std::{collections::BTreeMap, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
        .args(["-c", "commit.gpgsign=false", "-c", "core.quotePath=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn commit(root: &Path, message: &str) -> String {
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "--allow-empty", "-m", message]);
    git(root, &["rev-parse", "HEAD"])
}

fn init() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q", "-b", "main"]);
    root
}

fn changed(root: &Path, base: &str, subject: &str, glob: &str) -> Result<Collected, ProviderError> {
    let mut context = ProviderContext::new(root);
    context.change = Change {
        base: Some(base.into()),
        subject: Some(subject.into()),
        ..Change::default()
    };
    collect(
        "git.changed",
        &BTreeMap::from([("glob".into(), json!(glob))]),
        &context,
    )
}

/// Every other path operation (`fs.file.exists("CHANGELOG.md")`) is relative to the context root.
/// With the root in a subdirectory of the repository, `git.changed("CHANGELOG.md")` must mean the
/// same file, not the repository's top-level one.
#[test]
fn a_root_in_a_subdirectory_matches_paths_relative_to_that_root() {
    let repo = init();
    let sub = repo.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("README.md"), "r").unwrap();
    std::fs::write(repo.path().join("CHANGELOG.md"), "top").unwrap();
    let base = commit(repo.path(), "one");
    std::fs::write(sub.join("CHANGELOG.md"), "sub entry").unwrap();
    let subject = commit(repo.path(), "two");
    assert_eq!(
        changed(&sub, &base, &subject, "CHANGELOG.md").unwrap(),
        Collected::Known(json!(true)),
        "the root's own CHANGELOG.md changed, and the gate is rooted at sub/"
    );
}

/// A repository holding one path that is not UTF-8 is environmental, not a malformed request: it
/// must not turn every `git.changed` into a hard error that aborts the whole gates run.
#[test]
fn a_non_utf8_path_in_the_change_does_not_abort_collection() {
    use std::os::unix::ffi::OsStrExt;
    let repo = init();
    std::fs::write(repo.path().join("README.md"), "r").unwrap();
    let base = commit(repo.path(), "one");
    let name = std::ffi::OsStr::from_bytes(b"caf\xe9.txt");
    std::fs::write(repo.path().join(name), "latin-1 name").unwrap();
    std::fs::write(repo.path().join("CHANGELOG.md"), "entry").unwrap();
    let subject = commit(repo.path(), "two");
    let result = changed(repo.path(), &base, &subject, "CHANGELOG.md");
    assert!(
        matches!(
            result,
            Ok(Collected::Known(_)) | Ok(Collected::Unavailable(_))
        ),
        "expected an observation or unknown, got {result:?}"
    );
}

/// The outcome the story serves is "this change must touch the changelog". When the base is the
/// target branch's tip and it moved on after the subject forked, a direct base..subject diff
/// attributes the target branch's own CHANGELOG edit to the change, so the gate passes for a change
/// that never touched the changelog.
#[test]
fn a_base_that_is_not_an_ancestor_does_not_credit_the_change_with_the_bases_edits() {
    let repo = init();
    std::fs::write(repo.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
    std::fs::write(repo.path().join("README.md"), "r\n").unwrap();
    commit(repo.path(), "fork point");
    git(repo.path(), &["checkout", "-q", "-b", "feature"]);
    std::fs::write(repo.path().join("README.md"), "r, edited\n").unwrap();
    let subject = commit(repo.path(), "feature: no release note");
    git(repo.path(), &["checkout", "-q", "main"]);
    std::fs::write(repo.path().join("CHANGELOG.md"), "# Changelog\n- other\n").unwrap();
    let base = commit(repo.path(), "main: someone else's release note");
    assert_eq!(
        changed(repo.path(), &base, &subject, "CHANGELOG.md").unwrap(),
        Collected::Known(json!(false)),
        "the subject's change does not touch CHANGELOG.md"
    );
}

/// A glob that no git path can ever match is accepted and silently false. In `applies_when` that
/// makes the gate `not_applicable` for every change (fail-open). `./x` and `dir/` are common
/// spellings; they must either match what they name or be refused.
#[test]
fn a_glob_that_can_never_match_a_git_path_is_refused_or_matches() {
    let repo = init();
    std::fs::create_dir(repo.path().join("docs")).unwrap();
    std::fs::write(repo.path().join("README.md"), "r").unwrap();
    let base = commit(repo.path(), "one");
    std::fs::write(repo.path().join("CHANGELOG.md"), "entry").unwrap();
    std::fs::write(repo.path().join("docs/guide.md"), "guide").unwrap();
    let subject = commit(repo.path(), "two");
    for glob in ["./CHANGELOG.md", "docs/"] {
        let result = changed(repo.path(), &base, &subject, glob);
        assert!(
            matches!(
                result,
                Err(_) | Ok(Collected::Known(serde_json::Value::Bool(true)))
            ),
            "{glob:?}: expected a refusal or true, got {result:?}"
        );
    }
}

/// Renames, deletions, spaces and newlines in names, and base == subject.
#[test]
fn renames_deletions_odd_names_and_an_empty_range() {
    let repo = init();
    std::fs::write(repo.path().join("CHANGELOG.md"), "c").unwrap();
    std::fs::write(repo.path().join("gone.txt"), "g").unwrap();
    let base = commit(repo.path(), "one");
    git(repo.path(), &["mv", "CHANGELOG.md", "HISTORY.md"]);
    std::fs::remove_file(repo.path().join("gone.txt")).unwrap();
    std::fs::write(repo.path().join("with space\nand newline.md"), "x").unwrap();
    let subject = commit(repo.path(), "two");
    for (glob, expected) in [
        ("CHANGELOG.md", true),
        ("HISTORY.md", true),
        ("gone.txt", true),
        ("with space\nand newline.md", true),
        ("with space", false),
        ("and newline.md", false),
    ] {
        assert_eq!(
            changed(repo.path(), &base, &subject, glob).unwrap(),
            Collected::Known(json!(expected)),
            "{glob:?}"
        );
    }
    assert_eq!(
        changed(repo.path(), &subject, &subject, "**").unwrap(),
        Collected::Known(json!(false))
    );
}

/// A shallow clone whose base is beyond the boundary is unknown, never false.
#[test]
fn a_shallow_clone_missing_the_base_is_unavailable() {
    let repo = init();
    std::fs::write(repo.path().join("CHANGELOG.md"), "1").unwrap();
    let base = commit(repo.path(), "one");
    std::fs::write(repo.path().join("CHANGELOG.md"), "2").unwrap();
    commit(repo.path(), "two");
    std::fs::write(repo.path().join("CHANGELOG.md"), "3").unwrap();
    let subject = commit(repo.path(), "three");
    let shallow = tempfile::tempdir().unwrap();
    let url = format!("file://{}", repo.path().display());
    git(shallow.path(), &["clone", "-q", "--depth", "1", &url, "c"]);
    let clone = shallow.path().join("c");
    assert!(matches!(
        changed(&clone, &base, &subject, "CHANGELOG.md").unwrap(),
        Collected::Unavailable(_)
    ));
}
