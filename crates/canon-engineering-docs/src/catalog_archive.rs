//! The protocol catalog as one release asset: `engineering-protocols-catalog-<version>.tar.gz` and
//! its `.sha256` in `sha256sum` format. The archive holds what `protocols/manifest.yaml` lists
//! (vocabulary, protocols, schema) and the manifest itself, at their repository-relative paths and
//! with no top-level directory, so every path in the manifest resolves from the directory the
//! archive is unpacked into.
//!
//! Reproducible: entries in byte order of their paths, each a regular file with mode 0644, owner
//! and group 0 with empty names and modification time 0, no directory entries, a gzip header with
//! no time and no name, and a fixed compression level.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path};

use anyhow::{Context, Result, bail, ensure};
use flate2::{Compression, GzBuilder};
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// The manifest, relative to the repository root.
const MANIFEST: &str = "protocols/manifest.yaml";
/// The only manifest format this command reads.
const FORMAT: &str = "b10x-protocol-catalog/1";
/// The gzip operating system byte, fixed so the header is the same on every platform: Unix.
const GZIP_OS_UNIX: u8 = 3;
/// The deflate level, fixed so the bytes do not depend on a default.
const LEVEL: u32 = 9;

/// `protocols/manifest.yaml`, the fields of `schema/b10x.protocol-catalog.v1.schema.json`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    format: String,
    schema: String,
    vocabulary: Listed,
    protocols: Vec<Protocol>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Listed {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Protocol {
    path: String,
    #[allow(dead_code)] // named by the schema; the archive does not read it
    id: String,
    #[allow(dead_code)] // named by the schema; the archive does not read it
    revision: u64,
    sha256: String,
    #[allow(dead_code)] // named by the schema; the archive does not read it
    canon: String,
}

/// What a run wrote.
pub struct Written {
    pub archive: String,
    pub sha256: String,
    pub files: usize,
}

/// The lowercase hexadecimal SHA-256 of `bytes`.
fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// A listed path stays inside the repository: relative, with only plain components.
fn ensure_inside(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty()
            && Path::new(path)
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
        "{MANIFEST} lists {path:?}, which is not a path inside the repository"
    );
    Ok(())
}

/// The bytes of a listed file, which must exist.
fn read_listed(root: &Path, path: &str) -> Result<Vec<u8>> {
    ensure_inside(path)?;
    let file = root.join(path);
    if !file.is_file() {
        bail!(
            "{MANIFEST} lists {path}, which does not exist ({})",
            file.display()
        );
    }
    fs::read(&file).with_context(|| format!("reading {path}"))
}

/// The archive of the catalog `root` holds, as its gzip bytes and the number of files in it.
fn archive(root: &Path) -> Result<(Vec<u8>, usize)> {
    let text = String::from_utf8(read_listed(root, MANIFEST)?)
        .with_context(|| format!("{MANIFEST} is not UTF-8"))?;
    let catalog: Catalog =
        serde_yaml_ng::from_str(&text).with_context(|| format!("parsing {MANIFEST}"))?;
    ensure!(
        catalog.format == FORMAT,
        "{MANIFEST} has format {:?}, not {FORMAT}",
        catalog.format
    );

    let mut files: BTreeSet<&str> = BTreeSet::from([MANIFEST, catalog.schema.as_str()]);
    let mut digests = vec![(&catalog.vocabulary.path, &catalog.vocabulary.sha256)];
    for protocol in &catalog.protocols {
        digests.push((&protocol.path, &protocol.sha256));
    }
    for (path, _) in &digests {
        files.insert(path);
    }

    let gzip = GzBuilder::new()
        .operating_system(GZIP_OS_UNIX)
        .write(Vec::new(), Compression::new(LEVEL));
    let mut tar = tar::Builder::new(gzip);
    for path in &files {
        let bytes = read_listed(root, path)?;
        for (listed, sha256) in &digests {
            if listed.as_str() == *path {
                ensure!(
                    hex(&bytes) == **sha256,
                    "{path} does not have the sha256 {MANIFEST} lists for it"
                );
            }
        }
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(bytes.len() as u64);
        tar.append_data(&mut header, path, &bytes[..])
            .with_context(|| format!("adding {path}"))?;
    }
    let gzip = tar.into_inner().context("finishing the tar stream")?;
    Ok((
        gzip.finish().context("finishing the gzip stream")?,
        files.len(),
    ))
}

/// Writes `engineering-protocols-catalog-<version>.tar.gz` and its `.sha256` into `out`, built from
/// the catalog `root` holds. Nothing is written when a listed file is missing or differs from its
/// listed digest.
pub fn write(root: &Path, out: &Path) -> Result<Written> {
    let (bytes, files) = archive(root)?;
    let name = format!(
        "engineering-protocols-catalog-{}.tar.gz",
        env!("CARGO_PKG_VERSION")
    );
    let sha256 = hex(&bytes);
    fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    fs::write(out.join(&name), &bytes).with_context(|| format!("writing {name}"))?;
    fs::write(
        out.join(format!("{name}.sha256")),
        format!("{sha256}  {name}\n"),
    )
    .with_context(|| format!("writing {name}.sha256"))?;
    Ok(Written {
        archive: name,
        sha256,
        files,
    })
}
