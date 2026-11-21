//! The protocol catalog as one release asset: `engineering-protocols-catalog-<version>.tar.gz` and
//! its `.sha256` in `sha256sum` format. The archive is everything under `protocols/` plus the
//! schema file, at repository-relative paths and with no top-level directory, so every path in the
//! manifest resolves from the directory the archive is unpacked into. The manifest decides it, by
//! refusal rather than by walk: the files come from `protocols/manifest.yaml` plus the manifest
//! itself, and a manifest the gate (`crates/canon-engineering/tests/catalog_manifest.rs`) refuses
//! for its paths is refused here too, before anything is written. Ids, revisions and `canon`
//! values need Canon and stay with the gate.
//!
//! Reproducible: entries in byte order of their paths, each a regular file with mode 0644, owner
//! and group 0 with empty names and modification time 0, no directory entries, a gzip header with
//! no time and no name, and a fixed compression level.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use flate2::{Compression, GzBuilder};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::generate::builtin_name;

/// The manifest, relative to the repository root. It does not list itself.
const MANIFEST: &str = "protocols/manifest.yaml";
/// The only manifest format this command reads.
const FORMAT: &str = "b10x-protocol-catalog/1";
/// The one catalog file that is not a protocol.
const VOCABULARY: &str = "protocols/vocabulary.yaml";
/// The manifest's JSON Schema, which the manifest's `schema` must name.
const SCHEMA: &str = "schema/b10x.protocol-catalog.v1.schema.json";
/// The workspace manifest the version in the archive's file name is read from.
const WORKSPACE: &str = "Cargo.toml";
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
    #[allow(dead_code)] // named by the schema; the gate checks it against Canon
    id: String,
    #[allow(dead_code)] // named by the schema; the gate checks it against Canon
    revision: u64,
    sha256: String,
    #[allow(dead_code)] // named by the schema; the gate checks it against Canon
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

/// `protocols/<name>/<major>.yaml`, compared segment by segment as the manifest check does.
fn is_protocol_path(path: &str) -> bool {
    let Some(rest) = path.strip_prefix("protocols/") else {
        return false;
    };
    let Some((name, file)) = rest.split_once('/') else {
        return false;
    };
    builtin_name::is_name(name)
        && file
            .strip_suffix(".yaml")
            .and_then(builtin_name::major)
            .is_some()
}

/// The bytes of a listed file, which must exist.
fn read_listed(root: &Path, path: &str) -> Result<Vec<u8>> {
    let file = root.join(path);
    if !file.is_file() {
        bail!(
            "{MANIFEST} lists {path}, which does not exist ({})",
            file.display()
        );
    }
    fs::read(&file).with_context(|| format!("reading {path}"))
}

/// Every file below `root/protocols` but the manifest, relative to `root` and `/`-separated.
fn catalog_files(root: &Path) -> Result<BTreeSet<String>> {
    let mut found = BTreeSet::new();
    let mut pending = vec![(root.join("protocols"), String::from("protocols"))];
    while let Some((dir, relative)) = pending.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("reading {relative}"))? {
            let entry = entry.with_context(|| format!("reading {relative}"))?;
            let path = format!("{relative}/{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                pending.push((entry.path(), path));
            } else if path != MANIFEST {
                found.insert(path);
            }
        }
    }
    Ok(found)
}

/// `[workspace.package] version` of `<root>/Cargo.toml`, which must be usable in a file name.
fn version(root: &Path) -> Result<String> {
    let text = fs::read_to_string(root.join(WORKSPACE))
        .with_context(|| format!("reading {WORKSPACE} for the workspace version"))?;
    let table: toml::Table =
        toml::from_str(&text).with_context(|| format!("parsing {WORKSPACE}"))?;
    let version = table
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .with_context(|| format!("{WORKSPACE} has no [workspace.package] version"))?;
    if version.is_empty()
        || !version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
    {
        bail!("{WORKSPACE} has workspace version {version:?}, not usable in a file name");
    }
    Ok(version.to_owned())
}

/// Every refusal of the manifest's paths the gate makes, each naming the path it is about.
fn path_problems(root: &Path, catalog: &Catalog) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    if catalog.format != FORMAT {
        problems.push(format!(
            "{MANIFEST} has format {:?}, not {FORMAT}",
            catalog.format
        ));
    }
    if catalog.schema != SCHEMA {
        problems.push(format!(
            "{MANIFEST} names schema {}, not {SCHEMA}",
            catalog.schema
        ));
    }
    if catalog.vocabulary.path != VOCABULARY {
        problems.push(format!(
            "{MANIFEST} lists the vocabulary at {}, not {VOCABULARY}",
            catalog.vocabulary.path
        ));
    }
    let mut listed = BTreeSet::new();
    for path in std::iter::once(&catalog.vocabulary.path)
        .chain(catalog.protocols.iter().map(|protocol| &protocol.path))
    {
        if !listed.insert(path.as_str()) {
            problems.push(format!("{MANIFEST} lists {path} more than once"));
        }
    }
    for protocol in &catalog.protocols {
        if !is_protocol_path(&protocol.path) {
            problems.push(format!(
                "{MANIFEST} lists {}, which is not protocols/<name>/<major>.yaml",
                protocol.path
            ));
        }
    }
    for path in catalog_files(root)? {
        if !listed.contains(path.as_str()) {
            problems.push(format!(
                "{path} is under protocols/ but {MANIFEST} does not list it"
            ));
        }
    }
    if let Err(error) = version(root) {
        problems.push(format!("{error:#}"));
    }
    Ok(problems)
}

/// The archive of the catalog `root` holds: the version for its file name, its gzip bytes and the
/// number of files in it.
fn archive(root: &Path) -> Result<(String, Vec<u8>, usize)> {
    let text = String::from_utf8(read_listed(root, MANIFEST)?)
        .with_context(|| format!("{MANIFEST} is not UTF-8"))?;
    let catalog: Catalog =
        serde_yaml_ng::from_str(&text).with_context(|| format!("parsing {MANIFEST}"))?;
    let problems = path_problems(root, &catalog)?;
    if !problems.is_empty() {
        bail!("{}", problems.join("\n"));
    }
    let version = version(root)?;

    let mut digests =
        BTreeMap::from([(catalog.vocabulary.path.as_str(), &catalog.vocabulary.sha256)]);
    for protocol in &catalog.protocols {
        digests.insert(protocol.path.as_str(), &protocol.sha256);
    }
    let mut files: BTreeSet<&str> = digests.keys().copied().collect();
    files.extend([MANIFEST, SCHEMA]);

    let gzip = GzBuilder::new()
        .operating_system(GZIP_OS_UNIX)
        .write(Vec::new(), Compression::new(LEVEL));
    let mut tar = tar::Builder::new(gzip);
    for path in &files {
        let bytes = read_listed(root, path)?;
        if let Some(sha256) = digests.get(path)
            && hex(&bytes) != **sha256
        {
            bail!("{path} does not have the sha256 {MANIFEST} lists for it");
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
        version,
        gzip.finish().context("finishing the gzip stream")?,
        files.len(),
    ))
}

/// Writes `engineering-protocols-catalog-<version>.tar.gz` and its `.sha256` into `out`, built from
/// the catalog `root` holds, `<version>` being its `Cargo.toml`'s `[workspace.package] version`.
/// Nothing is written when the manifest's paths are refused, a listed file is missing or a listed
/// file differs from its listed digest.
pub fn write(root: &Path, out: &Path) -> Result<Written> {
    let (version, bytes, files) = archive(root)?;
    let name = format!("engineering-protocols-catalog-{version}.tar.gz");
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
