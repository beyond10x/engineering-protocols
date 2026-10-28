#![cfg(unix)]

use assertion_providers::CommandBinding;
use canon_engineering_assertions::{
    Admission, GateResult, Prepared, Project, collect, evaluate_gates, prepare_with,
};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn fixture(root: &Path, executable: &Path, expression: &str) {
    fs::write(
        root.join("gates.yaml"),
        serde_json::to_string(&serde_json::json!({
            "format": "engineering-gates/1",
            "language": "canon-expr/1",
            "gates": [{"id": "g", "text": "the admitted program runs", "assertions": [expression]}],
        }))
        .unwrap(),
    )
    .unwrap();
    let project = Project {
        commands: BTreeMap::from([(
            "tool".into(),
            CommandBinding {
                executable: executable.to_owned(),
                program: None,
                args: vec![],
                env: BTreeMap::new(),
            },
        )]),
        ..Project::default()
    };
    fs::write(
        root.join("project.yaml"),
        serde_yaml_ng::to_string(&project).unwrap(),
    )
    .unwrap();
}

fn argv0_fixture(root: &Path, executable: &Path, expected: &Path) {
    fixture(
        root,
        executable,
        &format!("process.run(\"tool\").stdout == \"{}\"", expected.display()),
    );
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

fn passes(root: &Path) -> GateResult {
    let bundle = collect(prepare(root).unwrap(), 1000).unwrap();
    evaluate_gates(&bundle, 1000, None, None).unwrap().result
}

fn point(link: &Path, target: &Path) {
    let staged = link.with_extension("staged");
    let _ = fs::remove_file(&staged);
    symlink(target, &staged).unwrap();
    fs::rename(&staged, link).unwrap();
}

/// The link is writable by the user the gate runs as; both targets are not. Admission digests
/// what the link reaches, `verify_executables` re-digests it before and after the request, and
/// the spawn resolves the link a third time. A swap that is undone between those three reads
/// runs a program whose bytes admission never digested, and the bundle certifies its result.
#[test]
#[ignore = "story:gate-executable-identity-bound-to-spawn: the spawn re-resolves the link admission digested"]
fn a_link_swapped_between_its_checks_runs_a_program_admission_never_digested() {
    let honest = PathBuf::from("/usr/bin/false");
    let forged = PathBuf::from("/usr/bin/true");
    assert!(honest.is_file() && forged.is_file());
    let root = tempfile::tempdir().unwrap();
    let links = tempfile::tempdir().unwrap();
    let link = links.path().canonicalize().unwrap().join("check");
    point(&link, &honest);
    fixture(root.path(), &link, "process.succeeded(\"tool\")");

    let paused = Arc::new(AtomicBool::new(true));
    let parked = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));
    let flipper = {
        let (paused, parked, stop, link) =
            (paused.clone(), parked.clone(), stop.clone(), link.clone());
        let (honest, forged) = (honest.clone(), forged.clone());
        thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                if paused.load(Ordering::SeqCst) {
                    point(&link, &honest);
                    parked.store(true, Ordering::SeqCst);
                    while paused.load(Ordering::SeqCst) && !stop.load(Ordering::SeqCst) {
                        thread::yield_now();
                    }
                    parked.store(false, Ordering::SeqCst);
                    continue;
                }
                point(&link, &forged);
                point(&link, &honest);
            }
            point(&link, &honest);
        })
    };

    let started = Instant::now();
    let mut forged_pass = None;
    let mut attempts = 0;
    while started.elapsed() < Duration::from_secs(60) && attempts < 3000 {
        attempts += 1;
        paused.store(true, Ordering::SeqCst);
        while !parked.load(Ordering::SeqCst) {
            thread::yield_now();
        }
        let prepared = prepare(root.path()).unwrap();
        paused.store(false, Ordering::SeqCst);
        if let Ok(bundle) = collect(prepared, 1000)
            && evaluate_gates(&bundle, 1000, None, None).unwrap().result == GateResult::Pass
        {
            forged_pass = Some(attempts);
            break;
        }
    }
    stop.store(true, Ordering::SeqCst);
    flipper.join().unwrap();
    assert_eq!(
        forged_pass,
        None,
        "story:gate-executable-identity-bound-to-spawn: collection admitted {} (digest locked \
         while the link reached it) and certified a pass \
         that only {} can produce, after {attempts} attempts",
        honest.display(),
        forged.display()
    );
}

#[test]
fn a_link_chain_runs_under_the_first_links_name() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let target = tools.join("multiplexer");
    script(&target, "printf '%s' \"$0\"");
    let middle = tools.join("middle");
    symlink(&target, &middle).unwrap();
    let first = tools.join("first");
    symlink(&middle, &first).unwrap();
    argv0_fixture(root.path(), &first, &first);
    assert_eq!(passes(root.path()), GateResult::Pass);
}

#[test]
fn a_relative_link_in_the_tree_runs_under_its_tree_path() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let target = tools.path().canonicalize().unwrap().join("multiplexer");
    script(&target, "printf '%s' \"$0\"");
    let real_root = root.path().canonicalize().unwrap();
    fs::create_dir(real_root.join("bin")).unwrap();
    symlink(&target, real_root.join("bin/tool")).unwrap();
    argv0_fixture(
        root.path(),
        Path::new("bin/tool"),
        &real_root.join("bin/tool"),
    );
    assert_eq!(passes(root.path()), GateResult::Pass);
}

#[test]
fn a_relative_executable_with_a_parent_component_runs_as_written() {
    let root = tempfile::tempdir().unwrap();
    let real_root = root.path().canonicalize().unwrap();
    fs::create_dir(real_root.join("sub")).unwrap();
    fs::create_dir(real_root.join("bin")).unwrap();
    script(&real_root.join("bin/tool"), "printf '%s' \"$0\"");
    argv0_fixture(
        root.path(),
        Path::new("sub/../bin/tool"),
        &real_root.join("sub/../bin/tool"),
    );
    assert_eq!(passes(root.path()), GateResult::Pass);
}

#[test]
fn a_dangling_link_is_refused_at_admission() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let link = tools.join("tool");
    symlink(tools.join("missing"), &link).unwrap();
    fixture(root.path(), &link, "process.succeeded(\"tool\")");
    let error = prepare(root.path())
        .err()
        .expect("a dangling link is admitted");
    assert!(
        format!("{error:#}").contains("resolve admitted executable"),
        "{error:#}"
    );
}

#[test]
fn a_link_to_a_directory_is_refused_at_admission() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    fs::create_dir(tools.join("dir")).unwrap();
    let link = tools.join("tool");
    symlink(tools.join("dir"), &link).unwrap();
    fixture(root.path(), &link, "process.succeeded(\"tool\")");
    let error = prepare(root.path())
        .err()
        .expect("a link to a directory is admitted");
    assert!(format!("{error:#}").contains("not a file"), "{error:#}");
}

#[test]
fn a_link_retargeted_to_a_directory_after_admission_refuses_collection() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let tools = tools.path().canonicalize().unwrap();
    let target = tools.join("tool-real");
    script(&target, "exit 0");
    fs::create_dir(tools.join("dir")).unwrap();
    let link = tools.join("tool");
    point(&link, &target);
    fixture(root.path(), &link, "process.succeeded(\"tool\")");
    let prepared = prepare(root.path()).unwrap();
    point(&link, &tools.join("dir"));
    assert!(
        collect(prepared, 1000).is_err(),
        "a link retargeted to a directory after admission was collected"
    );
}
