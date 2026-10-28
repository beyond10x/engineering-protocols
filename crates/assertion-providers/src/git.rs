//! Change-aware observations: which paths differ between the context's `base` and `subject`
//! revisions, and the story the runtime says the change belongs to. Git runs with a fixed argv,
//! no shell, a cleared environment and revisions that were resolved to object ids first.
use crate::{
    Arguments, Change, Collected, CommandBinding, ProviderContext, ProviderError, arg, known,
    process,
};
use std::{collections::BTreeMap, path::PathBuf};

/// Absolute locations tried in order. Neither the caller's `PATH` nor the working tree is searched.
const GIT_LOCATIONS: &[&str] = &[
    "/usr/bin/git",
    "/usr/local/bin/git",
    "/bin/git",
    "/opt/homebrew/bin/git",
];
/// Repository configuration is untrusted input: `core.fsmonitor` is a program git runs even for
/// `diff-tree`, so every invocation overrides what a repository could use to start one.
const HARDENING: &[&str] = &[
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.hooksPath=/dev/null",
    "-c",
    "core.pager=cat",
    "-c",
    "diff.external=",
    "-c",
    "core.sshCommand=false",
    "-c",
    "protocol.allow=never",
    "-c",
    "protocol.ext.allow=never",
];
const MAX_REVISION: usize = 256;
const MAX_GLOB: usize = 1024;

pub(super) fn collect(
    operation: &str,
    args: &Arguments,
    context: &ProviderContext,
) -> Result<Collected, ProviderError> {
    match operation {
        "git.changed" => changed(arg(args, "glob")?, context),
        "story.id" => Ok(match &context.change.story_id {
            Some(id) => Collected::Known(id.clone().into()),
            None => Collected::Unavailable("the context carries no story id".into()),
        }),
        "story.has_tag" => {
            let tag = arg(args, "tag")?;
            Ok(match &context.change.story_tags {
                Some(tags) => Collected::Known(tags.iter().any(|t| t == tag).into()),
                None => Collected::Unavailable("the context carries no story tags".into()),
            })
        }
        _ => Err(ProviderError::Invalid(format!(
            "unregistered operation: {operation}"
        ))),
    }
}

fn changed(glob: &str, context: &ProviderContext) -> Result<Collected, ProviderError> {
    validate_glob(glob)?;
    let Some(base) = &context.change.base else {
        return Ok(Collected::Unavailable(
            "the context carries no base revision".into(),
        ));
    };
    let Some(subject) = &context.change.subject else {
        return Ok(Collected::Unavailable(
            "the context carries no subject revision".into(),
        ));
    };
    validate_revision("base", base)?;
    validate_revision("subject", subject)?;
    let Some(git) = find_git() else {
        return Ok(Collected::Unavailable(
            "git was not found at a fixed system location".into(),
        ));
    };
    let base = match resolve(&git, base, context)? {
        Ok(oid) => oid,
        Err(reason) => return Ok(Collected::Unavailable(reason)),
    };
    let subject = match resolve(&git, subject, context)? {
        Ok(oid) => oid,
        Err(reason) => return Ok(Collected::Unavailable(reason)),
    };
    // What this change touched is measured from where it forked, so edits the base gained since do
    // not count. A subject already inside the base has no fork of its own: the two trees are compared.
    let fork = match run(&git, &["merge-base", &base, &subject], context)? {
        Ok(stdout) => stdout.trim().to_owned(),
        Err(reason) => return Ok(Collected::Unavailable(reason)),
    };
    if !is_object_id(&fork) {
        return Ok(Collected::Unavailable(
            "base and subject have no common ancestor".into(),
        ));
    }
    let left = if fork == subject { &base } else { &fork };
    let output = match run_bytes(
        &git,
        &[
            "diff-tree",
            "-r",
            "--name-only",
            "--no-renames",
            "--relative",
            "-z",
            left,
            &subject,
            "--",
        ],
        context,
    )? {
        Ok(output) => output,
        Err(reason) => return Ok(Collected::Unavailable(reason)),
    };
    known(
        output
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .any(|path| path_matches(glob, path)),
    )
}

/// A path that is not UTF-8 can only be matched by a pattern that does not name the replacement
/// character, so it is compared in its lossy form and never aborts the run.
fn path_matches(glob: &str, path: &[u8]) -> bool {
    match std::str::from_utf8(path) {
        Ok(path) => glob_matches(glob, path),
        Err(_) => !glob.contains('\u{fffd}') && glob_matches(glob, &String::from_utf8_lossy(path)),
    }
}

fn is_object_id(text: &str) -> bool {
    matches!(text.len(), 40 | 64) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

fn find_git() -> Option<PathBuf> {
    GIT_LOCATIONS
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

/// The change with `base` and `subject` replaced by the commit ids they name now, so that what is
/// identified and what is collected is one commit. A revision that cannot be resolved here (no git,
/// no repository, a shallow clone without it) is left as typed: collection reports it unavailable.
/// A revision that is malformed is refused.
pub fn resolve_change(root: &std::path::Path, change: &Change) -> Result<Change, ProviderError> {
    let mut resolved = change.clone();
    let context = ProviderContext::new(root);
    for (role, slot) in [
        ("base", &mut resolved.base),
        ("subject", &mut resolved.subject),
    ] {
        let Some(revision) = slot.as_deref() else {
            continue;
        };
        validate_revision(role, revision)?;
        let Some(git) = find_git() else {
            continue;
        };
        match resolve(&git, revision, &context) {
            Ok(Ok(oid)) => *slot = Some(oid),
            Ok(Err(_)) | Err(ProviderError::Io(..)) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(resolved)
}

/// `Ok(Err(reason))` is environmental (the repository or revision is not there); an `Err` is a
/// malformed request.
fn resolve(
    git: &std::path::Path,
    revision: &str,
    context: &ProviderContext,
) -> Result<Result<String, String>, ProviderError> {
    let spec = format!("{revision}^{{commit}}");
    let output = run(
        git,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            "--end-of-options",
            &spec,
        ],
        context,
    )?;
    Ok(output.and_then(|stdout| {
        let oid = stdout.trim();
        if is_object_id(oid) {
            Ok(oid.to_owned())
        } else {
            Err(format!("revision {revision} did not resolve to a commit"))
        }
    }))
}

fn run(
    git: &std::path::Path,
    argv: &[&str],
    context: &ProviderContext,
) -> Result<Result<String, String>, ProviderError> {
    match run_bytes(git, argv, context)? {
        Ok(stdout) => String::from_utf8(stdout)
            .map(Ok)
            .map_err(|e| ProviderError::Invalid(format!("git output is not UTF-8: {e}"))),
        Err(reason) => Ok(Err(reason)),
    }
}

fn run_bytes(
    git: &std::path::Path,
    argv: &[&str],
    context: &ProviderContext,
) -> Result<Result<Vec<u8>, String>, ProviderError> {
    let binding = CommandBinding {
        program: None,
        argv0: None,
        executable: git.to_owned(),
        args: HARDENING
            .iter()
            .chain(argv)
            .map(|a| (*a).to_owned())
            .collect(),
        env: BTreeMap::from([
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_NO_LAZY_FETCH".into(), "1".into()),
            ("GIT_PROTOCOL_FROM_USER".into(), "0".into()),
            ("GIT_OPTIONAL_LOCKS".into(), "0".into()),
            ("GIT_TERMINAL_PROMPT".into(), "0".into()),
            ("LC_ALL".into(), "C".into()),
        ]),
    };
    let Some(out) = process::run(&binding, &[], context)? else {
        return Ok(Err("git timed out or exceeded the output limit".into()));
    };
    if out.code != Some(0) {
        let reason = String::from_utf8_lossy(&out.stderr);
        let reason = reason.lines().next().unwrap_or("").trim();
        return Ok(Err(format!(
            "git {} failed with exit {:?}{}{}",
            argv[0],
            out.code,
            if reason.is_empty() { "" } else { ": " },
            reason.chars().take(200).collect::<String>()
        )));
    }
    Ok(Ok(out.stdout))
}

/// A revision is a ref-like name or object id. Anything that git could read as an option, a range,
/// a pathspec or a revision expression with side effects is refused before git is started.
pub(crate) fn validate_revision(role: &str, revision: &str) -> Result<(), ProviderError> {
    let ok = !revision.is_empty()
        && revision.len() <= MAX_REVISION
        && !revision.starts_with('-')
        && !revision.contains("..")
        && revision
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._/~^-@".contains(&b));
    if ok {
        Ok(())
    } else {
        Err(ProviderError::Invalid(format!(
            "{role} is not a valid revision: {revision:?}"
        )))
    }
}

/// A glob is matched against paths git reports: relative to the root, no empty, `.` or `..` segment.
/// One that could never match (`./x`, `dir/`, `/x`, `a//b`) is refused rather than left false.
pub fn validate_glob(glob: &str) -> Result<(), ProviderError> {
    let ok = !glob.is_empty()
        && glob.len() <= MAX_GLOB
        && !glob.contains('\0')
        && !glob
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."));
    if ok {
        Ok(())
    } else {
        Err(ProviderError::Invalid(format!(
            "glob must be a non-empty relative pattern with no empty, `.` or `..` segment: {glob:?}"
        )))
    }
}

/// `*` and `?` stay inside a path segment; a `**` segment matches zero or more whole segments.
pub(crate) fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = pattern.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    segments(&pattern, &path)
}

fn segments(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => (0..=path.len()).any(|skip| segments(rest, &path[skip..])),
        Some((first, rest)) => path
            .split_first()
            .is_some_and(|(head, tail)| wildcard(first, head) && segments(rest, tail)),
    }
}

fn wildcard(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let (mut p, mut t) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == text[t]) && pattern[p] != '*' {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some((p, t));
            p += 1;
        } else if let Some((sp, st)) = star {
            p = sp + 1;
            t = st + 1;
            star = Some((sp, st + 1));
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == '*' {
        p += 1;
    }
    p == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs_match_by_segment() {
        assert!(glob_matches("CHANGELOG.md", "CHANGELOG.md"));
        assert!(!glob_matches("CHANGELOG.md", "docs/CHANGELOG.md"));
        assert!(glob_matches("docs/*.md", "docs/a.md"));
        assert!(!glob_matches("docs/*.md", "docs/sub/a.md"));
        assert!(glob_matches("docs/**", "docs/sub/a.md"));
        assert!(glob_matches("**/*.rs", "a.rs"));
        assert!(glob_matches("**/*.rs", "crates/x/src/a.rs"));
        assert!(!glob_matches("**/*.rs", "crates/x/src/a.rs.orig"));
        assert!(glob_matches("a?c", "abc"));
        assert!(!glob_matches("a?c", "a/c"));
        assert!(glob_matches("*", "anything"));
        assert!(!glob_matches("*", "a/b"));
        assert!(glob_matches("src/**/mod.rs", "src/mod.rs"));
        assert!(glob_matches("src/**/mod.rs", "src/a/b/mod.rs"));
    }

    #[test]
    fn revisions_that_git_could_read_as_options_or_ranges_are_refused() {
        for good in [
            "HEAD",
            "main",
            "origin/main",
            "HEAD~2",
            "v1.0.0",
            "abc123",
            &"a".repeat(40),
        ] {
            assert!(validate_revision("base", good).is_ok(), "{good}");
        }
        for bad in [
            "",
            "-h",
            "--output=/tmp/x",
            "a..b",
            "a b",
            "a;b",
            "$(x)",
            "a\nb",
            "HEAD@{1}",
            ":/x",
            "a\0b",
            &"a".repeat(257),
        ] {
            assert!(validate_revision("base", bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn globs_that_escape_the_tree_are_refused() {
        for bad in [
            "",
            "/etc/passwd",
            "../x",
            "a/../b",
            "a\0b",
            "./CHANGELOG.md",
            "docs/",
            "a//b",
            "a/./b",
            ".",
        ] {
            assert!(validate_glob(bad).is_err(), "{bad:?}");
        }
        assert!(validate_glob("docs/**/*.md").is_ok());
    }
}
