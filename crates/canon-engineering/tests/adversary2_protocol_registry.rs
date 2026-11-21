//! Adversary pass 2 on `story:protocol-registry` (wave 2026-10-04-w13), at 99d62db plus the
//! uncommitted phase 2 and the pass-1 fixes.
//!
//! `src/builtin_name.rs` says its rule "is the one `canon-engineering-docs` applies to the same
//! tree", and `build.rs` applies it to decide which files it embeds. These cases hold both uses of
//! the rule to that statement, and pin the library's refusal variants, which the acceptance checks
//! only through `Display`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use canon_engineering::registry::{self, Error};

/// The repository root, read at run time.
fn repository_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    Path::new(&manifest).join("../..")
}

fn canon_engineering(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon-engineering"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("canon-engineering {}: {error}", args.join(" ")))
}

/// The library's refusals are matchable by variant, carry what was asked for, and say which of
/// the two it was. The acceptance checks only that `Display` contains `name@major`, which both
/// variants print, so a swap of `UnknownName` and `UnknownMajor` would pass it.
#[test]
fn get_refuses_with_the_variant_that_names_what_is_missing() {
    match registry::get("no-such-protocol", 1) {
        Err(Error::UnknownName { name, major }) => {
            assert_eq!((name.as_str(), major), ("no-such-protocol", 1));
        }
        other => panic!("get(\"no-such-protocol\", 1) is UnknownName: {other:?}"),
    }
    match registry::get("software-change", 99) {
        Err(Error::UnknownMajor { name, major }) => {
            assert_eq!((name.as_str(), major), ("software-change", 99));
        }
        other => panic!("get(\"software-change\", 99) is UnknownMajor: {other:?}"),
    }
    assert_eq!(
        registry::get("no-such-protocol", 1)
            .expect_err("refused")
            .to_string(),
        "no-such-protocol@1: no built-in protocol is named `no-such-protocol`"
    );
    assert_eq!(
        registry::get("software-change", 99)
            .expect_err("refused")
            .to_string(),
        "software-change@99: built-in `software-change` has no major version 99"
    );

    // `get` parses and validates on every call; the same call twice gives the same built-in.
    for (name, major) in registry::list() {
        let first = registry::get(name, major).expect("a listed built-in is served");
        let second = registry::get(name, major).expect("a listed built-in is served again");
        assert_eq!((first.name, first.major), (name, major));
        assert_eq!(first.yaml, second.yaml);
        assert_eq!(first.model, second.model);
    }
}

/// `src/builtin_name.rs` states that its rule is the one `canon-engineering-docs` applies, and
/// `canon-engineering-docs` (`crates/canon-engineering-docs/src/generate.rs`, `is_name`) requires a
/// name to start with a lowercase letter. So a name starting with a digit or a hyphen is malformed,
/// a usage error (exit 2) like every other malformed argument, not a well-formed name that happens
/// to be unknown (exit 1).
#[test]
fn show_refuses_a_name_els_docs_refuses_as_malformed() {
    for argument in ["1@1", "2fa@1", "-x@1", "--@1", "-@1"] {
        let out = canon_engineering(&["protocols", "show", "--", argument]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "canon-engineering protocols show -- {argument:?}: canon-engineering-docs refuses the name, so it is malformed: {out:?}"
        );
        assert!(out.stdout.is_empty(), "{out:?}");
    }
}

/// Removes a scratch directory when dropped, so a failing step leaves no build behind.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

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

/// Builds the copy's `canon-engineering` binary into `target`.
fn build(checkout: &Path, target: &Path) -> Output {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    Command::new(cargo)
        .current_dir(checkout)
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
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("cargo runs")
}

fn write_protocol(protocols: &Path, relative: &str, text: &str) {
    let path = protocols.join(relative);
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("protocol dir");
    std::fs::write(&path, text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

/// `build.rs` decides what is embedded with `src/builtin_name.rs`; nothing else in the suite gives
/// it a tree outside the rule, so replacing either check in `build.rs` with a looser one stays
/// green. This builds a copy of the workspace against such trees, with its own target dir:
///
/// 1. only `protocols/vocabulary.yaml`: it builds, `list` prints nothing, `show` refuses (exit 1);
/// 2. `software-change/01.yaml`: the build fails naming the file;
/// 3. `Software/1.yaml`: the build fails naming the directory;
/// 4. `2fa/1.yaml`: `canon-engineering-docs` refuses this directory (its name starts with a digit),
///    so the build, which claims the same rule, fails naming it too.
#[test]
fn build_embeds_only_what_the_rule_and_canon_engineering_docs_accept() {
    let root = repository_root();
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("adversary2-registry-build");
    let _ = std::fs::remove_dir_all(&scratch);
    let scratch = Scratch(scratch);
    let checkout = scratch.0.join("checkout");
    let target = scratch.0.join("target");
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
    let software_change = std::fs::read_to_string(root.join("protocols/software-change/1.yaml"))
        .expect("software-change reads");

    // 1. No protocol directory at all.
    let out = build(&checkout, &target);
    assert!(
        out.status.success(),
        "step 1: a tree with no protocol builds: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let binary = target.join("debug/canon-engineering");
    let list = Command::new(&binary)
        .args(["protocols", "list"])
        .output()
        .expect("canon-engineering runs");
    assert_eq!(list.status.code(), Some(0), "step 1: {list:?}");
    assert!(list.stdout.is_empty(), "step 1: {list:?}");
    let show = Command::new(&binary)
        .args(["protocols", "show", "software-change@1"])
        .output()
        .expect("canon-engineering runs");
    assert_eq!(show.status.code(), Some(1), "step 1: {show:?}");
    assert!(
        String::from_utf8_lossy(&show.stderr).contains("software-change@1"),
        "step 1: {show:?}"
    );

    // 2. A major with a leading zero.
    write_protocol(&protocols, "software-change/01.yaml", &software_change);
    let out = build(&checkout, &target);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success()
            && stderr.contains("protocols/software-change/01.yaml: a protocol file is named"),
        "step 2: software-change/01.yaml fails the build: {stderr}"
    );
    std::fs::remove_dir_all(protocols.join("software-change")).expect("step 2 cleans up");

    // 3. An uppercase directory name.
    write_protocol(&protocols, "Software/1.yaml", &software_change);
    let out = build(&checkout, &target);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success() && stderr.contains("protocol directory `Software`"),
        "step 3: protocols/Software/ fails the build: {stderr}"
    );
    std::fs::remove_dir_all(protocols.join("Software")).expect("step 3 cleans up");

    // 4. A directory name canon-engineering-docs refuses.
    write_protocol(&protocols, "2fa/1.yaml", &software_change);
    let out = build(&checkout, &target);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success() && stderr.contains("protocol directory `2fa`"),
        "step 4: canon-engineering-docs refuses protocols/2fa/, so the build that claims its rule fails too: \
         exit {:?}, {stderr}",
        out.status.code()
    );
}
