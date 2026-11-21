//! Adversary pass 2 on `story:catalog-release-archive`, against the correction 8745e6f. Each case
//! holds one refusal alone: the root keeps a valid `Cargo.toml`, so the only reason a run can be
//! refused is the one the case builds.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fresh(case: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary2-catalog-archive-{}-{case}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir scratch");
    dir
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

/// `Cargo.toml`, `protocols/` and `schema/` of the real tree, and an empty `--out` beside them.
fn scratch(case: &str) -> (PathBuf, PathBuf) {
    let root = fresh(&format!("{case}-root"));
    let out = fresh(&format!("{case}-out"));
    fs::copy(repository().join("Cargo.toml"), root.join("Cargo.toml")).expect("copy Cargo.toml");
    for dir in ["protocols", "schema"] {
        copy_tree(&repository().join(dir), &root.join(dir));
    }
    (root, out)
}

fn run(root: &Path, out: &Path) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_canon-engineering-docs"))
        .args([
            "catalog-archive",
            "--root",
            root.to_str().expect("UTF-8 path"),
            "--out",
            out.to_str().expect("UTF-8 path"),
        ])
        .output()
        .expect("run canon-engineering-docs")
}

fn edit_manifest(root: &Path, edit: impl FnOnce(&mut serde_yaml_ng::Value)) {
    let path = root.join("protocols/manifest.yaml");
    let mut value: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&fs::read_to_string(&path).expect("read manifest"))
            .expect("parse manifest");
    edit(&mut value);
    fs::write(&path, serde_yaml_ng::to_string(&value).expect("write yaml")).expect("write");
}

fn refused_naming(run: &Output, out: &Path, needle: &str) {
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        !run.status.success(),
        "must be refused ({needle}); stdout: {}",
        String::from_utf8_lossy(&run.stdout)
    );
    assert!(
        stderr.contains(needle),
        "stderr must name {needle}: {stderr}"
    );
    let left: Vec<_> = fs::read_dir(out)
        .expect("list out")
        .map(|entry| entry.expect("entry").file_name())
        .collect();
    assert!(left.is_empty(), "a refused run leaves nothing: {left:?}");
}

/// Moves `protocols/software-change/1.yaml` to `protocols/<dir>/<file>` and lists it there.
fn relist_software_change(root: &Path, dir: &str, file: &str) -> String {
    let to = format!("protocols/{dir}/{file}");
    fs::create_dir_all(root.join("protocols").join(dir)).expect("mkdir");
    fs::rename(
        root.join("protocols/software-change/1.yaml"),
        root.join(&to),
    )
    .expect("move");
    let _ = fs::remove_dir(root.join("protocols/software-change"));
    let listed = to.clone();
    edit_manifest(root, |manifest| {
        for protocol in manifest["protocols"].as_sequence_mut().expect("list") {
            if protocol["path"].as_str() == Some("protocols/software-change/1.yaml") {
                protocol["path"] = listed.into();
                return;
            }
        }
        panic!("software-change is listed");
    });
    to
}

/// Pass 1's schema case writes a `Cargo.toml` without a version, so since 8745e6f the version
/// refusal alone makes it pass. Here `Cargo.toml` is valid and the schema file exists.
#[test]
fn a_schema_other_than_the_fixed_one_is_refused_on_its_own() {
    let (root, out) = scratch("schema");
    fs::write(root.join("schema/other.schema.json"), "{}\n").expect("write");
    edit_manifest(&root, |manifest| {
        manifest["schema"] = "schema/other.schema.json".into();
    });
    refused_naming(&run(&root, &out), &out, "schema/other.schema.json");
}

/// Names the manifest check refuses under `builtin_name::is_name`: uppercase, underscore, a
/// leading digit, a leading hyphen.
#[test]
fn a_protocol_directory_that_is_not_a_name_is_refused() {
    for (case, name) in [
        ("upper", "Software-change"),
        ("underscore", "software_change"),
        ("digit", "1software"),
        ("hyphen", "-software"),
    ] {
        let (root, out) = scratch(&format!("name-{case}"));
        let path = relist_software_change(&root, name, "1.yaml");
        refused_naming(&run(&root, &out), &out, &path);
    }
}

/// Majors the manifest check refuses under `builtin_name::major`, and a nested directory and a
/// file that is not `.yaml`.
#[test]
fn a_protocol_file_that_is_not_a_major_yaml_is_refused() {
    for (case, dir, file) in [
        ("zero", "software-change", "0.yaml"),
        ("plus", "software-change", "+1.yaml"),
        ("yml", "software-change", "1.yml"),
        ("nested", "software-change/v1", "1.yaml"),
    ] {
        let (root, out) = scratch(&format!("major-{case}"));
        let path = relist_software_change(&root, dir, file);
        refused_naming(&run(&root, &out), &out, &path);
    }
}

#[test]
fn a_version_that_cannot_go_in_a_file_name_is_refused() {
    for (case, version) in [
        ("slash", "0.16.0/x"),
        ("space", "0.16.0 x"),
        ("newline", "0.16.0\\nx"),
        ("empty", ""),
    ] {
        let (root, out) = scratch(&format!("version-{case}"));
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace.package]\nversion = \"{version}\"\n"),
        )
        .expect("write Cargo.toml");
        refused_naming(&run(&root, &out), &out, "Cargo.toml");
    }
}

#[test]
fn a_pre_release_version_with_build_metadata_is_named_verbatim() {
    let (root, out) = scratch("prerelease");
    fs::write(
        root.join("Cargo.toml"),
        "workspace.package.version = \"0.17.0-rc.1+b5\"\n",
    )
    .expect("write Cargo.toml");
    let done = run(&root, &out);
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    let name = "engineering-protocols-catalog-0.17.0-rc.1+b5.tar.gz";
    assert!(out.join(name).is_file());
    let line = fs::read_to_string(out.join(format!("{name}.sha256"))).expect("digest");
    assert!(line.ends_with(&format!("  {name}\n")), "{line}");
}

/// A `[package]`-only `Cargo.toml` has no workspace version.
#[test]
fn a_package_only_cargo_toml_is_refused() {
    let (root, out) = scratch("package-only");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"1.0.0\"\n",
    )
    .expect("write Cargo.toml");
    refused_naming(&run(&root, &out), &out, "Cargo.toml");
}

/// The manifest check's walk counts a second manifest in a subdirectory, a dotfile and an editor
/// backup as unlisted; so does the command.
#[test]
fn stray_files_under_protocols_are_refused() {
    for (case, path) in [
        ("submanifest", "protocols/software-change/manifest.yaml"),
        ("dotfile", "protocols/.keep"),
        ("backup", "protocols/software-change/1.yaml~"),
        ("readme", "protocols/README.md"),
    ] {
        let (root, out) = scratch(&format!("stray-{case}"));
        fs::write(root.join(path), "x\n").expect("write");
        refused_naming(&run(&root, &out), &out, path);
    }
}
