//! Adversary pass 1 on `story:catalog-release-archive`: cases `catalog_archive.rs` does not state.
//! The pins (digest, format, a missing `--out`, another checkout of one tree) each kill a mutant the
//! unit's own five tests leave alive; the two red cases drive the command with a manifest that
//! `crates/canon-engineering/tests/catalog_manifest.rs` refuses and the command accepts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime};

use flate2::read::GzDecoder;

const ARCHIVE: &str = concat!(
    "engineering-protocols-catalog-",
    env!("CARGO_PKG_VERSION"),
    ".tar.gz"
);

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fresh(case: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-catalog-archive-{}-{case}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir scratch");
    dir
}

fn run(cwd: Option<&Path>, root: &Path, out: &Path) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_canon-engineering-docs"));
    command.args([
        "catalog-archive",
        "--root",
        root.to_str().expect("UTF-8 path"),
        "--out",
        out.to_str().expect("UTF-8 path"),
    ]);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command.output().expect("run canon-engineering-docs")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("mkdir");
    for entry in fs::read_dir(from).expect("read dir") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy");
        }
    }
}

/// A copy of `protocols/` and `schema/` under a fresh directory.
fn scratch_root(case: &str) -> PathBuf {
    let root = fresh(case);
    fs::copy(repository().join("Cargo.toml"), root.join("Cargo.toml")).expect("copy Cargo.toml");
    for dir in ["protocols", "schema"] {
        copy_tree(&repository().join(dir), &root.join(dir));
    }
    root
}

fn edit_manifest(root: &Path, edit: impl FnOnce(&mut serde_yaml_ng::Value)) {
    let path = root.join("protocols/manifest.yaml");
    let mut value: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&fs::read_to_string(&path).expect("read manifest"))
            .expect("parse manifest");
    edit(&mut value);
    fs::write(&path, serde_yaml_ng::to_string(&value).expect("write yaml")).expect("write");
}

fn member_paths(archive: &[u8]) -> Vec<String> {
    tar::Archive::new(GzDecoder::new(archive))
        .entries()
        .expect("entries")
        .map(|entry| {
            entry
                .expect("entry")
                .path()
                .expect("path")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn refused_naming(run: &Output, out: &Path, path: &str) {
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "must be refused; stderr: {stderr}");
    assert!(stderr.contains(path), "stderr must name {path}: {stderr}");
    let left: Vec<_> = fs::read_dir(out)
        .expect("list out")
        .map(|entry| entry.expect("entry").file_name())
        .collect();
    assert!(left.is_empty(), "a refused run leaves nothing: {left:?}");
}

#[test]
fn a_listed_file_whose_bytes_differ_from_its_digest_is_refused_and_writes_nothing() {
    let root = scratch_root("digest-root");
    let out = fresh("digest-out");
    let file = root.join("protocols/incident-response/1.yaml");
    let mut bytes = fs::read(&file).expect("read");
    bytes.push(b'\n');
    fs::write(&file, bytes).expect("write");
    refused_naming(
        &run(None, &root, &out),
        &out,
        "protocols/incident-response/1.yaml",
    );
}

#[test]
fn a_manifest_of_another_format_is_refused_and_writes_nothing() {
    let root = scratch_root("format-root");
    let out = fresh("format-out");
    edit_manifest(&root, |manifest| {
        manifest["format"] = "b10x-protocol-catalog/2".into();
    });
    refused_naming(&run(None, &root, &out), &out, "b10x-protocol-catalog/2");
}

#[test]
fn an_out_directory_that_does_not_exist_is_created() {
    let out = fresh("nested").join("a/b");
    let done = run(None, &repository(), &out);
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    assert!(out.join(ARCHIVE).is_file());
    assert!(out.join(format!("{ARCHIVE}.sha256")).is_file());
}

/// Another checkout of one tree: other file modes and mtimes, a relative `--root` and `--out` from
/// another working directory. The bytes are those of the archive built from the real tree.
#[test]
fn another_checkout_of_the_same_tree_gives_the_same_bytes() {
    use std::os::unix::fs::PermissionsExt;

    let reference = fresh("checkout-reference");
    let done = run(None, &repository(), &reference);
    assert!(done.status.success());

    let base = fresh("checkout-base");
    let root = base.join("checkout");
    fs::create_dir_all(&root).expect("mkdir checkout");
    fs::copy(repository().join("Cargo.toml"), root.join("Cargo.toml")).expect("copy Cargo.toml");
    for dir in ["protocols", "schema"] {
        copy_tree(&repository().join(dir), &root.join(dir));
    }
    let when = SystemTime::UNIX_EPOCH + Duration::from_secs(1_234_567_890);
    let mut pending = vec![root.clone()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("chmod");
                fs::File::options()
                    .write(true)
                    .open(&path)
                    .expect("open")
                    .set_modified(when)
                    .expect("set mtime");
            }
        }
    }
    let done = run(Some(&base), Path::new("checkout"), Path::new("out"));
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    assert_eq!(
        fs::read(base.join("out").join(ARCHIVE)).expect("copy archive"),
        fs::read(reference.join(ARCHIVE)).expect("reference archive"),
        "another checkout of one tree must give the same archive bytes"
    );
}

/// `protocols/./software-change/1.yaml` passes `ensure_inside` (`Path::components` drops the `.`),
/// is sorted by its spelling and stored by the tar crate as `protocols/software-change/1.yaml`. The
/// manifest check refuses the path (`OutsideCatalog`). Either the command refuses it too, or the
/// archive it writes keeps its own claims: members unique and in byte order of their paths.
#[test]
fn a_path_spelled_with_a_dot_component_is_refused_or_keeps_entry_order() {
    let root = scratch_root("dot-root");
    let out = fresh("dot-out");
    edit_manifest(&root, |manifest| {
        for protocol in manifest["protocols"].as_sequence_mut().expect("list") {
            if protocol["path"].as_str() == Some("protocols/software-change/1.yaml") {
                protocol["path"] = "protocols/./software-change/1.yaml".into();
            }
        }
    });
    let done = run(None, &root, &out);
    if !done.status.success() {
        return;
    }
    let paths = member_paths(&fs::read(out.join(ARCHIVE)).expect("archive"));
    let mut sorted = paths.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        paths, sorted,
        "the command accepted a non-plain path and wrote entries out of byte order"
    );
}

/// The manifest's `schema` is the one listed file with no digest. The command packages whatever
/// repository file it names; the manifest check and the schema's `const` refuse any value but
/// `schema/b10x.protocol-catalog.v1.schema.json`.
#[test]
fn a_schema_other_than_the_one_the_format_names_is_refused() {
    let root = scratch_root("schema-root");
    let out = fresh("schema-out");
    fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("write");
    edit_manifest(&root, |manifest| {
        manifest["schema"] = "Cargo.toml".into();
    });
    let done = run(None, &root, &out);
    let members = if done.status.success() {
        member_paths(&fs::read(out.join(ARCHIVE)).expect("archive"))
    } else {
        Vec::new()
    };
    assert!(
        !done.status.success(),
        "a manifest naming schema Cargo.toml was packaged: {members:?}"
    );
    assert!(String::from_utf8_lossy(&done.stderr).contains("Cargo.toml"));
}
