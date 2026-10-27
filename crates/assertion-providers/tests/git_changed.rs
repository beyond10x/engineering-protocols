use assertion_providers::{Change, Collected, ProviderContext, collect};
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

fn repository() -> (tempfile::TempDir, String, String) {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    std::fs::create_dir(root.path().join("docs")).unwrap();
    std::fs::write(root.path().join("a.txt"), "a").unwrap();
    git(root.path(), &["add", "-A"]);
    git(root.path(), &["commit", "-q", "-m", "one"]);
    let base = git(root.path(), &["rev-parse", "HEAD"]);
    std::fs::write(root.path().join("docs/guide.md"), "guide").unwrap();
    git(root.path(), &["add", "-A"]);
    git(root.path(), &["commit", "-q", "-m", "two"]);
    let subject = git(root.path(), &["rev-parse", "HEAD"]);
    (root, base, subject)
}

fn changed(
    root: &Path,
    base: Option<&str>,
    subject: Option<&str>,
    glob: &str,
) -> Result<Collected, assertion_providers::ProviderError> {
    let mut context = ProviderContext::new(root);
    context.change = Change {
        base: base.map(str::to_owned),
        subject: subject.map(str::to_owned),
        ..Change::default()
    };
    collect(
        "git.changed",
        &BTreeMap::from([("glob".into(), json!(glob))]),
        &context,
    )
}

#[test]
fn paths_between_base_and_subject_are_matched_by_glob() {
    let (root, base, subject) = repository();
    let (b, s) = (Some(base.as_str()), Some(subject.as_str()));
    assert_eq!(
        changed(root.path(), b, s, "docs/**").unwrap(),
        Collected::Known(json!(true))
    );
    assert_eq!(
        changed(root.path(), b, s, "docs/*.md").unwrap(),
        Collected::Known(json!(true))
    );
    assert_eq!(
        changed(root.path(), b, s, "a.txt").unwrap(),
        Collected::Known(json!(false))
    );
    assert_eq!(
        changed(root.path(), s, b, "docs/guide.md").unwrap(),
        Collected::Known(json!(true)),
        "the diff is symmetric: it names paths that differ"
    );
}

#[test]
fn a_missing_revision_or_repository_is_unavailable_not_false() {
    let (root, base, _) = repository();
    assert!(matches!(
        changed(root.path(), Some(&base), Some("no-such-ref"), "a.txt").unwrap(),
        Collected::Unavailable(_)
    ));
    let elsewhere = tempfile::tempdir().unwrap();
    assert!(matches!(
        changed(elsewhere.path(), Some(&base), Some(&base), "a.txt").unwrap(),
        Collected::Unavailable(_)
    ));
    assert!(matches!(
        changed(root.path(), None, Some(&base), "a.txt").unwrap(),
        Collected::Unavailable(_)
    ));
}

#[test]
fn a_revision_that_looks_like_an_option_is_refused_before_git_runs() {
    let (root, base, _) = repository();
    let marker = root.path().join("pwned");
    for bad in [
        format!("--output={}", marker.display()),
        "-h".into(),
        format!("{base}..{base}"),
        "HEAD; touch pwned".into(),
    ] {
        assert!(
            changed(root.path(), Some(&base), Some(&bad), "a.txt").is_err(),
            "{bad}"
        );
        assert!(
            changed(root.path(), Some(&bad), Some(&base), "a.txt").is_err(),
            "{bad}"
        );
    }
    assert!(!marker.exists());
}

#[test]
fn the_repository_configuration_cannot_run_a_program() {
    let (root, base, subject) = repository();
    let marker = root.path().join("pwned");
    let program = format!("touch {}", marker.display());
    git(root.path(), &["config", "core.fsmonitor", &program]);
    git(root.path(), &["config", "core.pager", &program]);
    git(root.path(), &["config", "diff.external", &program]);
    assert_eq!(
        changed(root.path(), Some(&base), Some(&subject), "docs/**").unwrap(),
        Collected::Known(json!(true))
    );
    assert!(!marker.exists());
}

#[test]
fn story_operations_read_only_the_context() {
    let mut context = ProviderContext::new(".");
    let tag = |t: &str| BTreeMap::from([("tag".to_owned(), json!(t))]);
    assert!(matches!(
        collect("story.has_tag", &tag("x"), &context).unwrap(),
        Collected::Unavailable(_)
    ));
    assert!(matches!(
        collect("story.id", &BTreeMap::new(), &context).unwrap(),
        Collected::Unavailable(_)
    ));
    context.change.story_id = Some("s-1".into());
    context.change.story_tags = Some(vec!["consumer-visible".into()]);
    assert_eq!(
        collect("story.has_tag", &tag("consumer-visible"), &context).unwrap(),
        Collected::Known(json!(true))
    );
    assert_eq!(
        collect("story.has_tag", &tag("other"), &context).unwrap(),
        Collected::Known(json!(false))
    );
    assert_eq!(
        collect("story.id", &BTreeMap::new(), &context).unwrap(),
        Collected::Known(json!("s-1"))
    );
}
