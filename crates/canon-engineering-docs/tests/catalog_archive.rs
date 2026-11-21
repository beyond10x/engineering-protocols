//! `story:catalog-release-archive`: `catalog-archive --out <dir>` writes the protocol catalog as one
//! reproducible `engineering-protocols-catalog-<version>.tar.gz` and its `.sha256` in `sha256sum`
//! format. The archive holds what `protocols/manifest.yaml` lists, plus the manifest, and nothing
//! else.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use flate2::read::GzDecoder;
use serde_yaml_ng::Value;
use sha2::{Digest, Sha256};

const ARCHIVE: &str = concat!(
    "engineering-protocols-catalog-",
    env!("CARGO_PKG_VERSION"),
    ".tar.gz"
);

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A fresh, empty directory for one case.
fn fresh(case: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("catalog-archive-{}-{case}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir scratch");
    dir
}

fn catalog_archive(root: &Path, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon-engineering-docs"))
        .args([
            "catalog-archive",
            "--root",
            root.to_str().expect("UTF-8 path"),
            "--out",
            out.to_str().expect("UTF-8 path"),
        ])
        .output()
        .expect("run canon-engineering-docs catalog-archive")
}

/// Runs the command on the real tree into a fresh directory and returns the directory.
fn build(case: &str) -> PathBuf {
    let out = fresh(case);
    let run = catalog_archive(&repository(), &out);
    assert!(
        run.status.success(),
        "catalog-archive failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn manifest() -> Value {
    let text = fs::read_to_string(repository().join("protocols/manifest.yaml"))
        .expect("read protocols/manifest.yaml");
    serde_yaml_ng::from_str(&text).expect("parse protocols/manifest.yaml")
}

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("manifest has no string {key}"))
}

/// Every path the manifest lists: vocabulary, protocols and schema, with its own path.
fn listed_paths() -> BTreeSet<String> {
    let manifest = manifest();
    let mut paths = BTreeSet::new();
    paths.insert(string(&manifest["vocabulary"], "path").to_string());
    paths.insert(string(&manifest, "schema").to_string());
    paths.insert("protocols/manifest.yaml".to_string());
    for protocol in manifest["protocols"].as_sequence().expect("protocols list") {
        paths.insert(string(protocol, "path").to_string());
    }
    paths
}

struct Member {
    path: String,
    kind: tar::EntryType,
    mode: u32,
    mtime: u64,
    uid: u64,
    gid: u64,
    user: Option<String>,
    group: Option<String>,
}

fn members(archive: &[u8]) -> Vec<Member> {
    let mut tar = tar::Archive::new(GzDecoder::new(archive));
    tar.entries()
        .expect("read entries")
        .map(|entry| {
            let entry = entry.expect("entry");
            let header = entry.header();
            Member {
                path: entry.path().expect("path").to_string_lossy().into_owned(),
                kind: header.entry_type(),
                mode: header.mode().expect("mode"),
                mtime: header.mtime().expect("mtime"),
                uid: header.uid().expect("uid"),
                gid: header.gid().expect("gid"),
                user: header.username().expect("user name").map(str::to_string),
                group: header.groupname().expect("group name").map(str::to_string),
            }
        })
        .collect()
}

#[test]
fn two_runs_write_identical_archives_and_digests() {
    let first = build("twice-a");
    let second = build("twice-b");
    assert_eq!(
        fs::read(first.join(ARCHIVE)).expect("first archive"),
        fs::read(second.join(ARCHIVE)).expect("second archive"),
        "two runs on one tree must write byte-identical archives"
    );
    let digest = format!("{ARCHIVE}.sha256");
    assert_eq!(
        fs::read(first.join(&digest)).expect("first digest"),
        fs::read(second.join(&digest)).expect("second digest")
    );
    let names: BTreeSet<String> = fs::read_dir(&first)
        .expect("list out")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("UTF-8")
        })
        .collect();
    assert_eq!(names, BTreeSet::from([ARCHIVE.to_string(), digest]));
}

#[test]
fn digest_file_is_one_sha256sum_line_for_the_archive() {
    let out = build("digest");
    let archive = fs::read(out.join(ARCHIVE)).expect("archive");
    let line = fs::read_to_string(out.join(format!("{ARCHIVE}.sha256"))).expect("digest file");
    assert_eq!(line, format!("{}  {ARCHIVE}\n", sha256_hex(&archive)));
}

#[test]
fn archive_holds_exactly_what_the_manifest_lists_and_each_file_has_its_digest() {
    let out = build("content");
    let archive = fs::read(out.join(ARCHIVE)).expect("archive");
    let unpacked = fresh("content-unpacked");
    tar::Archive::new(GzDecoder::new(&archive[..]))
        .unpack(&unpacked)
        .expect("unpack");

    let mut found = BTreeSet::new();
    let mut stack = vec![unpacked.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                found.insert(
                    path.strip_prefix(&unpacked)
                        .expect("below root")
                        .to_str()
                        .expect("UTF-8")
                        .to_string(),
                );
            }
        }
    }
    assert_eq!(found, listed_paths());

    let inside: Value = serde_yaml_ng::from_str(
        &fs::read_to_string(unpacked.join("protocols/manifest.yaml")).expect("manifest inside"),
    )
    .expect("parse manifest inside");
    let mut listed = vec![(
        string(&inside["vocabulary"], "path").to_string(),
        string(&inside["vocabulary"], "sha256").to_string(),
    )];
    for protocol in inside["protocols"].as_sequence().expect("protocols list") {
        listed.push((
            string(protocol, "path").to_string(),
            string(protocol, "sha256").to_string(),
        ));
    }
    assert_eq!(listed.len(), 11);
    for (path, sha256) in listed {
        let bytes = fs::read(unpacked.join(&path)).expect("listed file in archive");
        assert_eq!(sha256_hex(&bytes), sha256, "{path}");
    }

    for same in ["protocols/manifest.yaml", string(&manifest(), "schema")] {
        assert_eq!(
            fs::read(unpacked.join(same)).expect("inside"),
            fs::read(repository().join(same)).expect("in tree"),
            "{same} must be byte-identical to the tree's"
        );
    }
}

#[test]
fn entries_are_plain_files_in_byte_order_with_no_ownership_or_time() {
    let out = build("reproducible");
    let archive = fs::read(out.join(ARCHIVE)).expect("archive");

    assert_eq!(&archive[..3], &[0x1f, 0x8b, 8], "gzip magic and deflate");
    assert_eq!(
        archive[3], 0,
        "gzip flags: no file name, no comment, no extra"
    );
    assert_eq!(&archive[4..8], &[0; 4], "gzip header mtime");

    let members = members(&archive);
    assert_eq!(members.len(), listed_paths().len());
    for member in &members {
        let path = &member.path;
        assert!(member.kind.is_file(), "{path} is not a regular file");
        assert_eq!(member.mode, 0o644, "{path} mode");
        assert_eq!(member.mtime, 0, "{path} mtime");
        assert_eq!((member.uid, member.gid), (0, 0), "{path} owner");
        assert_eq!(member.user.as_deref().unwrap_or(""), "", "{path} user name");
        assert_eq!(
            member.group.as_deref().unwrap_or(""),
            "",
            "{path} group name"
        );
    }
    let paths: Vec<&[u8]> = members.iter().map(|m| m.path.as_bytes()).collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "entries in byte order of their paths");
}

#[test]
fn a_listed_file_that_does_not_exist_fails_naming_its_path() {
    let (root, out) = scratch_root("missing");
    fs::remove_file(root.join("protocols/software-change/1.yaml")).expect("delete listed file");
    refused_naming(
        &catalog_archive(&root, &out),
        &out,
        "protocols/software-change/1.yaml",
    );
}

/// A copy of the inputs the command reads, `Cargo.toml`, `protocols/` and `schema/`, and an empty
/// `--out` directory beside it.
fn scratch_root(case: &str) -> (PathBuf, PathBuf) {
    let root = fresh(&format!("{case}-root"));
    let out = fresh(&format!("{case}-out"));
    fs::copy(repository().join("Cargo.toml"), root.join("Cargo.toml")).expect("copy Cargo.toml");
    for dir in ["protocols", "schema"] {
        copy_tree(&repository().join(dir), &root.join(dir));
    }
    (root, out)
}

fn edit_manifest(root: &Path, edit: impl FnOnce(&mut Value)) {
    let path = root.join("protocols/manifest.yaml");
    let mut value: Value =
        serde_yaml_ng::from_str(&fs::read_to_string(&path).expect("read manifest"))
            .expect("parse manifest");
    edit(&mut value);
    fs::write(&path, serde_yaml_ng::to_string(&value).expect("write yaml")).expect("write");
}

/// The run failed, its error names `path` and `--out` is still empty.
fn refused_naming(run: &Output, out: &Path, path: &str) {
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "must be refused; stderr: {stderr}");
    assert!(stderr.contains(path), "stderr must name {path}: {stderr}");
    assert_eq!(
        fs::read_dir(out).expect("list out").count(),
        0,
        "a refused run leaves no archive or digest"
    );
}

#[test]
fn a_vocabulary_listed_elsewhere_is_refused_naming_its_path() {
    let (root, out) = scratch_root("vocabulary");
    fs::rename(
        root.join("protocols/vocabulary.yaml"),
        root.join("protocols/words.yaml"),
    )
    .expect("rename the vocabulary");
    edit_manifest(&root, |manifest| {
        manifest["vocabulary"]["path"] = "protocols/words.yaml".into();
    });
    refused_naming(&catalog_archive(&root, &out), &out, "protocols/words.yaml");
}

#[test]
fn a_protocol_path_not_of_the_form_is_refused_naming_its_path() {
    let (root, out) = scratch_root("form");
    fs::rename(
        root.join("protocols/software-change/1.yaml"),
        root.join("protocols/software-change/01.yaml"),
    )
    .expect("rename listed file");
    edit_manifest(&root, |manifest| {
        for protocol in manifest["protocols"].as_sequence_mut().expect("list") {
            if protocol["path"].as_str() == Some("protocols/software-change/1.yaml") {
                protocol["path"] = "protocols/software-change/01.yaml".into();
            }
        }
    });
    refused_naming(
        &catalog_archive(&root, &out),
        &out,
        "protocols/software-change/01.yaml",
    );
}

#[test]
fn a_path_listed_twice_is_refused_naming_its_path() {
    let (root, out) = scratch_root("twice");
    edit_manifest(&root, |manifest| {
        let protocols = manifest["protocols"].as_sequence_mut().expect("list");
        let first = protocols[0].clone();
        protocols.push(first);
    });
    refused_naming(
        &catalog_archive(&root, &out),
        &out,
        "protocols/eval-gated-change/1.yaml",
    );
}

#[test]
fn a_file_under_protocols_the_manifest_does_not_list_is_refused_naming_its_path() {
    let (root, out) = scratch_root("unlisted");
    fs::create_dir_all(root.join("protocols/notes")).expect("mkdir");
    fs::write(root.join("protocols/notes/draft.txt"), "not listed\n").expect("write");
    refused_naming(
        &catalog_archive(&root, &out),
        &out,
        "protocols/notes/draft.txt",
    );
}

#[test]
fn the_file_name_carries_the_version_of_the_root_not_of_the_binary() {
    let (root, out) = scratch_root("version");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace.package]\nversion = \"9.9.9\"\n",
    )
    .expect("write Cargo.toml");
    let run = catalog_archive(&root, &out);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let name = "engineering-protocols-catalog-9.9.9.tar.gz";
    let archive = fs::read(out.join(name)).expect("archive named for the root's version");
    assert_eq!(
        fs::read_to_string(out.join(format!("{name}.sha256"))).expect("digest file"),
        format!("{}  {name}\n", sha256_hex(&archive))
    );
    assert_eq!(fs::read_dir(&out).expect("list out").count(), 2);
}

#[test]
fn a_root_without_a_workspace_version_is_refused_naming_cargo_toml() {
    let (root, out) = scratch_root("no-version");
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").expect("write Cargo.toml");
    refused_naming(&catalog_archive(&root, &out), &out, "Cargo.toml");

    fs::remove_file(root.join("Cargo.toml")).expect("delete Cargo.toml");
    refused_naming(&catalog_archive(&root, &out), &out, "Cargo.toml");
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
