//! Adversary pass 2 on `story:assertion-catalog-path`: after the move, the scratch checkout that
//! `adversary2_protocol_registry` builds carries the assertion catalog only through its copy of
//! `crates/`. This holds that test to its claim: a copy of the workspace without
//! `crates/canon-engineering-assertions/catalog.yaml` does not build, and the failure names the
//! file, so a green `build_embeds_only_...` really depends on the file being copied.

use std::path::{Path, PathBuf};
use std::process::Command;

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap_or_else(|error| panic!("{}: {error}", to.display()));
    for entry in
        std::fs::read_dir(from).unwrap_or_else(|error| panic!("{}: {error}", from.display()))
    {
        let entry = entry.expect("entry");
        let kind = entry.file_type().expect("file type");
        let dest = to.join(entry.file_name());
        if kind.is_dir() {
            if entry.file_name() != "target" {
                copy_tree(&entry.path(), &dest);
            }
        } else if kind.is_file() {
            std::fs::copy(entry.path(), &dest)
                .unwrap_or_else(|error| panic!("{}: {error}", dest.display()));
        }
    }
}

#[test]
fn a_checkout_without_the_assertion_catalog_does_not_build_and_names_the_file() {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let root = Path::new(&manifest).join("../..");
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("adversary3-catalog-path");
    let _ = std::fs::remove_dir_all(&scratch);
    let checkout = scratch.join("checkout");
    std::fs::create_dir_all(&checkout).expect("checkout dir");
    for file in ["Cargo.toml", "Cargo.lock"] {
        std::fs::copy(root.join(file), checkout.join(file)).expect("manifest copies");
    }
    copy_tree(&root.join("crates"), &checkout.join("crates"));
    let protocols = checkout.join("protocols");
    std::fs::create_dir_all(&protocols).expect("protocols dir");
    std::fs::copy(
        root.join("protocols/vocabulary.yaml"),
        protocols.join("vocabulary.yaml"),
    )
    .expect("vocabulary copies");
    let catalog = checkout.join("crates/canon-engineering-assertions/catalog.yaml");
    assert!(catalog.is_file(), "the crates/ copy carries the catalog");
    std::fs::remove_file(&catalog).expect("remove the copied catalog");

    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let out = Command::new(cargo)
        .current_dir(&checkout)
        .args([
            "build",
            "--offline",
            "--locked",
            "-q",
            "-p",
            "b10x-canon-engineering",
            "--bin",
            "canon-engineering",
        ])
        .env("CARGO_TARGET_DIR", scratch.join("target"))
        .output()
        .expect("cargo runs");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(!out.status.success(), "a checkout without catalog.yaml builds");
    assert!(
        stderr.contains("catalog.yaml"),
        "the failure names catalog.yaml: {stderr}"
    );
}
