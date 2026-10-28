//! A program that prints its own `argv[0]`. A shell script cannot be one: the kernel hands the
//! interpreter the path it executed, so `$0` names the file, never the name it was started under.

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

const SOURCE: &str = "fn main() { use std::io::Write; use std::os::unix::ffi::OsStrExt; \
    let name = std::env::args_os().next().unwrap_or_default(); \
    std::io::stdout().write_all(name.as_bytes()).unwrap(); }";

fn built() -> &'static Path {
    static BUILT: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &BUILT
        .get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let source = dir.path().join("argv0.rs");
            fs::write(&source, SOURCE).unwrap();
            let binary = dir.path().join("argv0");
            let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
            let status = Command::new(rustc)
                .arg("--edition=2021")
                .arg("-o")
                .arg(&binary)
                .arg(&source)
                .status()
                .expect("rustc builds the argv[0] printer");
            assert!(
                status.success(),
                "rustc failed to build the argv[0] printer"
            );
            (dir, binary)
        })
        .1
}

/// Write the printer to `path`, executable.
pub fn argv0_printer(path: &Path) {
    fs::copy(built(), path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}
