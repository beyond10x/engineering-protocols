//! Acceptance for `story:catalog-manifest`: `protocols/manifest.yaml`, a `b10x-protocol-catalog/1`
//! document whose schema is `schema/b10x.protocol-catalog.v1.schema.json`, lists every file of the
//! catalog with the sha256 of its bytes, and each protocol file with the id and revision Canon
//! parses from it and the lowest Canon release that reads it.
//!
//! [`check`] is the whole check, over the repository root, the manifest text and the `Cargo.lock`
//! text. It runs once on the repository tree, which must give no problem, and once per mutation of
//! a scratch copy, which must give exactly the problem that mutation makes. The `canon` value is
//! declared by hand and checked in one direction only: a Canon pinned in `Cargo.lock` lower than
//! it is a problem, an understated value is not detected.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[allow(dead_code)] // the check uses the name and major rules, not `reference`
#[path = "../src/builtin_name.rs"]
mod builtin_name;

/// The manifest, relative to the repository root. It does not list itself.
const MANIFEST: &str = "protocols/manifest.yaml";
/// The manifest's `format`.
const FORMAT: &str = "b10x-protocol-catalog/1";
/// The one catalog file that is not a protocol.
const VOCABULARY: &str = "protocols/vocabulary.yaml";
/// The manifest's JSON Schema, relative to the repository root.
const SCHEMA: &str = "schema/b10x.protocol-catalog.v1.schema.json";
/// The package in `Cargo.lock` whose release parses the protocol files.
const CANON_PACKAGE: &str = "b10x-canon";

/// A `b10x-protocol-catalog/1` document. These types, with `deny_unknown_fields`, are the checked
/// model; the schema file describes the same fields
/// (`the_schema_names_exactly_the_fields_the_types_accept`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    vocabulary: VocabularyEntry,
    protocols: Vec<ProtocolEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VocabularyEntry {
    path: String,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProtocolEntry {
    path: String,
    id: String,
    revision: u64,
    sha256: String,
    canon: String,
}

/// What [`check`] finds wrong, each naming the path it is about.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// The manifest is not a document the types accept, such as one with a field the schema does
    /// not know; the message is serde's.
    Manifest(String),
    /// The manifest's `format` is not [`FORMAT`].
    Format(String),
    /// `Cargo.lock` cannot be read, pins no [`CANON_PACKAGE`], or pins one whose version is not
    /// `MAJOR.MINOR.PATCH`.
    Lock(String),
    /// A file under `protocols/` the manifest does not list.
    Unlisted(String),
    /// A listed path names no file.
    Missing(String),
    /// A listed path that exists but cannot be read as a file.
    Unreadable { path: String, error: String },
    /// A path listed more than once.
    Duplicate(String),
    /// A listed protocol path that is not `protocols/<name>/<major>.yaml` under the rule `build.rs`
    /// embeds built-ins by, such as a path with `..` or the manifest itself.
    OutsideCatalog(String),
    /// The vocabulary entry names a file other than [`VOCABULARY`].
    Vocabulary(String),
    /// A listed sha256 that is not the digest of the file's bytes.
    Digest {
        path: String,
        listed: String,
        actual: String,
    },
    /// Canon does not parse a listed protocol file.
    Unparsed { path: String, error: String },
    /// A listed id that is not the one Canon parses from the file.
    Id {
        path: String,
        listed: String,
        parsed: String,
    },
    /// A listed revision that is not the one Canon parses from the file.
    Revision {
        path: String,
        listed: u64,
        parsed: u64,
    },
    /// A listed `canon` that is not `MAJOR.MINOR.PATCH`.
    CanonValue { path: String, value: String },
    /// The Canon pinned in `Cargo.lock` is lower than a listed `canon`.
    CanonPin {
        path: String,
        canon: String,
        pinned: String,
    },
}

/// Every problem of `manifest` against the files under `root` and the Canon pinned in `lock`,
/// sorted; none when the manifest holds.
fn check(root: &Path, manifest: &str, lock: &str) -> Vec<Problem> {
    let manifest: Manifest = match serde_yaml_ng::from_str(manifest) {
        Ok(manifest) => manifest,
        Err(error) => return vec![Problem::Manifest(error.to_string())],
    };
    let mut problems = Vec::new();
    if manifest.format != FORMAT {
        problems.push(Problem::Format(manifest.format.clone()));
    }
    let pinned = match pinned_canon(lock) {
        Ok(pinned) => Some(pinned),
        Err(error) => {
            problems.push(Problem::Lock(error));
            None
        }
    };

    let mut listed = BTreeSet::new();
    for path in std::iter::once(&manifest.vocabulary.path)
        .chain(manifest.protocols.iter().map(|entry| &entry.path))
    {
        if !listed.insert(path.as_str()) {
            problems.push(Problem::Duplicate(path.clone()));
        }
    }

    if manifest.vocabulary.path == VOCABULARY {
        read_listed(
            root,
            &manifest.vocabulary.path,
            &manifest.vocabulary.sha256,
            &mut problems,
        );
    } else {
        problems.push(Problem::Vocabulary(manifest.vocabulary.path.clone()));
    }

    for entry in &manifest.protocols {
        match release(&entry.canon) {
            None => problems.push(Problem::CanonValue {
                path: entry.path.clone(),
                value: entry.canon.clone(),
            }),
            Some(canon) => {
                if let Some((pinned_text, pinned)) = &pinned
                    && *pinned < canon
                {
                    problems.push(Problem::CanonPin {
                        path: entry.path.clone(),
                        canon: entry.canon.clone(),
                        pinned: pinned_text.clone(),
                    });
                }
            }
        }
        if !is_protocol_path(&entry.path) {
            problems.push(Problem::OutsideCatalog(entry.path.clone()));
            continue;
        }
        let Some(bytes) = read_listed(root, &entry.path, &entry.sha256, &mut problems) else {
            continue;
        };
        let parsed = std::str::from_utf8(&bytes)
            .map_err(|error| error.to_string())
            .and_then(|text| b10x_canon::model::parse(text).map_err(|error| error.to_string()));
        match parsed {
            Err(error) => problems.push(Problem::Unparsed {
                path: entry.path.clone(),
                error,
            }),
            Ok(protocol) => {
                let parsed_id = protocol.protocol.id.as_str();
                if entry.id != parsed_id {
                    problems.push(Problem::Id {
                        path: entry.path.clone(),
                        listed: entry.id.clone(),
                        parsed: parsed_id.to_owned(),
                    });
                }
                if entry.revision != protocol.protocol.revision {
                    problems.push(Problem::Revision {
                        path: entry.path.clone(),
                        listed: entry.revision,
                        parsed: protocol.protocol.revision,
                    });
                }
            }
        }
    }

    for path in catalog_files(root, &mut problems) {
        if !listed.contains(path.as_str()) {
            problems.push(Problem::Unlisted(path));
        }
    }

    problems.sort();
    problems
}

/// The bytes of a listed file, recording a problem for a missing or unreadable file and for a
/// digest other than `sha256`.
fn read_listed(
    root: &Path,
    path: &str,
    sha256: &str,
    problems: &mut Vec<Problem>,
) -> Option<Vec<u8>> {
    let bytes = match fs::read(root.join(path)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            problems.push(Problem::Missing(path.to_owned()));
            return None;
        }
        Err(error) => {
            problems.push(Problem::Unreadable {
                path: path.to_owned(),
                error: error.to_string(),
            });
            return None;
        }
    };
    let actual = digest(&bytes);
    if actual != sha256 {
        problems.push(Problem::Digest {
            path: path.to_owned(),
            listed: sha256.to_owned(),
            actual,
        });
    }
    Some(bytes)
}

/// `protocols/<name>/<major>.yaml`, under the rule `build.rs` embeds built-ins by.
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

/// Every file below `root/protocols` but the manifest, relative to `root` and `/`-separated.
fn catalog_files(root: &Path, problems: &mut Vec<Problem>) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut pending = vec![(root.join("protocols"), String::from("protocols"))];
    while let Some((dir, relative)) = pending.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => {
                problems.push(Problem::Unreadable {
                    path: relative,
                    error: error.to_string(),
                });
                continue;
            }
        };
        for entry in entries {
            let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = format!("{relative}/{name}");
            if entry.path().is_dir() {
                pending.push((entry.path(), path));
            } else if path != MANIFEST {
                found.insert(path);
            }
        }
    }
    found
}

/// Lowercase hex of the sha256 of `bytes`.
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// `MAJOR.MINOR.PATCH`, each a decimal without a sign or leading zeros.
fn release(text: &str) -> Option<(u64, u64, u64)> {
    let part = |part: &str| {
        let number: u64 = part.parse().ok()?;
        (number.to_string() == part).then_some(number)
    };
    let mut parts = text.split('.');
    let release = (
        part(parts.next()?)?,
        part(parts.next()?)?,
        part(parts.next()?)?,
    );
    parts.next().is_none().then_some(release)
}

/// The lowest version of [`CANON_PACKAGE`] that `Cargo.lock` pins, as written and as a release.
fn pinned_canon(lock: &str) -> Result<(String, (u64, u64, u64)), String> {
    #[derive(Deserialize)]
    struct Lock {
        #[serde(default)]
        package: Vec<Package>,
    }
    #[derive(Deserialize)]
    struct Package {
        name: String,
        version: String,
    }
    let lock: Lock = toml::from_str(lock).map_err(|error| error.to_string())?;
    let mut pinned = Vec::new();
    for package in lock.package {
        if package.name != CANON_PACKAGE {
            continue;
        }
        let release = release(&package.version).ok_or_else(|| {
            format!(
                "{CANON_PACKAGE} {} is not MAJOR.MINOR.PATCH",
                package.version
            )
        })?;
        pinned.push((release, package.version));
    }
    pinned.sort();
    let (release, version) = pinned
        .into_iter()
        .next()
        .ok_or_else(|| format!("Cargo.lock pins no {CANON_PACKAGE}"))?;
    Ok((version, release))
}

/// The repository root, read at run time.
fn repository_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    Path::new(&manifest).join("../..")
}

fn read(root: &Path, path: &str) -> String {
    let path = root.join(path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// A copy of the repository's `protocols/` in a temporary directory under the target directory,
/// with the repository's manifest and `Cargo.lock` text, for one mutation.
struct Scratch {
    dir: tempfile::TempDir,
    manifest: String,
    lock: String,
}

impl Scratch {
    fn new() -> Self {
        let root = repository_root();
        let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))
            .unwrap_or_else(|error| panic!("scratch directory: {error}"));
        copy_dir(&root.join("protocols"), &dir.path().join("protocols"));
        Self {
            dir,
            manifest: read(&root, MANIFEST),
            lock: read(&root, "Cargo.lock"),
        }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root().join(relative)
    }

    fn check(&self) -> Vec<Problem> {
        check(self.root(), &self.manifest, &self.lock)
    }

    fn parsed_manifest(&self) -> Manifest {
        serde_yaml_ng::from_str(&self.manifest)
            .unwrap_or_else(|error| panic!("{MANIFEST}: {error}"))
    }

    /// Rewrites the scratch manifest through the typed model.
    fn edit_manifest(&mut self, edit: impl FnOnce(&mut Manifest)) {
        let mut manifest = self.parsed_manifest();
        edit(&mut manifest);
        self.manifest = serde_yaml_ng::to_string(&manifest).expect("serialising the manifest");
    }

    /// Rewrites the scratch manifest as untyped YAML, for fields the types do not have.
    fn edit_manifest_yaml(&mut self, edit: impl FnOnce(&mut serde_yaml_ng::Value)) {
        let mut value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&self.manifest)
            .unwrap_or_else(|error| panic!("{MANIFEST}: {error}"));
        edit(&mut value);
        self.manifest = serde_yaml_ng::to_string(&value).expect("serialising the manifest");
    }

    /// The listed entry of the protocol file at `path`.
    fn entry(&self, path: &str) -> ProtocolEntry {
        self.parsed_manifest()
            .protocols
            .into_iter()
            .find(|entry| entry.path == path)
            .unwrap_or_else(|| panic!("{MANIFEST} lists no {path}"))
    }
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|error| panic!("{}: {error}", to.display()));
    let entries = fs::read_dir(from).unwrap_or_else(|error| panic!("{}: {error}", from.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", from.display()));
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|error| panic!("{}: {error}", entry.path().display()));
        }
    }
}

fn entry_mut<'a>(manifest: &'a mut Manifest, path: &str) -> &'a mut ProtocolEntry {
    manifest
        .protocols
        .iter_mut()
        .find(|entry| entry.path == path)
        .unwrap_or_else(|| panic!("{MANIFEST} lists no {path}"))
}

const SOFTWARE_CHANGE: &str = "protocols/software-change/1.yaml";
const STORY_DELIVERY_2: &str = "protocols/story-delivery/2.yaml";

#[test]
fn the_manifest_matches_the_catalog() {
    let root = repository_root();
    assert_eq!(
        check(&root, &read(&root, MANIFEST), &read(&root, "Cargo.lock")),
        Vec::<Problem>::new()
    );
}

#[test]
fn the_manifest_lists_the_vocabulary_and_every_protocol_file() {
    let root = repository_root();
    let manifest: Manifest = serde_yaml_ng::from_str(&read(&root, MANIFEST))
        .unwrap_or_else(|error| panic!("{MANIFEST}: {error}"));
    let listed: BTreeSet<String> = std::iter::once(manifest.vocabulary.path)
        .chain(manifest.protocols.into_iter().map(|entry| entry.path))
        .collect();
    let mut problems = Vec::new();
    assert_eq!(listed, catalog_files(&root, &mut problems));
    assert_eq!(problems, Vec::<Problem>::new());
    assert!(listed.contains(VOCABULARY));
    assert!(!listed.contains(MANIFEST));
}

#[test]
fn an_unmutated_scratch_copy_has_no_problem() {
    assert_eq!(Scratch::new().check(), Vec::<Problem>::new());
}

#[test]
fn a_file_under_protocols_missing_from_the_manifest_is_a_problem() {
    let scratch = Scratch::new();
    let added = "protocols/support-triage/2.yaml";
    fs::copy(
        scratch.path("protocols/support-triage/1.yaml"),
        scratch.path(added),
    )
    .expect("copying a protocol file");
    assert_eq!(scratch.check(), vec![Problem::Unlisted(added.to_owned())]);

    let mut scratch = Scratch::new();
    scratch.edit_manifest(|manifest| {
        manifest
            .protocols
            .retain(|entry| entry.path != SOFTWARE_CHANGE);
    });
    assert_eq!(
        scratch.check(),
        vec![Problem::Unlisted(SOFTWARE_CHANGE.to_owned())]
    );
}

#[test]
fn a_listed_file_that_does_not_exist_is_a_problem() {
    let scratch = Scratch::new();
    let removed = "protocols/ml-experiment/1.yaml";
    fs::remove_file(scratch.path(removed)).expect("removing a protocol file");
    assert_eq!(scratch.check(), vec![Problem::Missing(removed.to_owned())]);

    let scratch = Scratch::new();
    fs::remove_file(scratch.path(VOCABULARY)).expect("removing the vocabulary");
    assert_eq!(
        scratch.check(),
        vec![Problem::Missing(VOCABULARY.to_owned())]
    );
}

#[test]
fn a_listed_sha256_that_differs_from_the_bytes_is_a_problem() {
    for path in ["protocols/incident-response/1.yaml", VOCABULARY] {
        let scratch = Scratch::new();
        let listed = if path == VOCABULARY {
            scratch.parsed_manifest().vocabulary.sha256
        } else {
            scratch.entry(path).sha256
        };
        let mut bytes = fs::read(scratch.path(path)).expect("reading a catalog file");
        bytes.extend_from_slice(b"\n");
        fs::write(scratch.path(path), &bytes).expect("writing a catalog file");
        assert_eq!(
            scratch.check(),
            vec![Problem::Digest {
                path: path.to_owned(),
                listed,
                actual: digest(&bytes),
            }],
            "{path}"
        );
    }
}

#[test]
fn a_listed_id_that_differs_from_what_canon_parses_is_a_problem() {
    let mut scratch = Scratch::new();
    scratch.edit_manifest(|manifest| {
        entry_mut(manifest, SOFTWARE_CHANGE).id = "software.changes".to_owned();
    });
    assert_eq!(
        scratch.check(),
        vec![Problem::Id {
            path: SOFTWARE_CHANGE.to_owned(),
            listed: "software.changes".to_owned(),
            parsed: "software.change".to_owned(),
        }]
    );
}

#[test]
fn a_listed_revision_that_differs_from_what_canon_parses_is_a_problem() {
    let mut scratch = Scratch::new();
    let parsed = scratch.entry(STORY_DELIVERY_2).revision;
    scratch.edit_manifest(|manifest| {
        entry_mut(manifest, STORY_DELIVERY_2).revision = parsed + 1;
    });
    assert_eq!(
        scratch.check(),
        vec![Problem::Revision {
            path: STORY_DELIVERY_2.to_owned(),
            listed: parsed + 1,
            parsed,
        }]
    );
}

#[test]
fn a_pinned_canon_lower_than_a_listed_canon_is_a_problem() {
    let mut scratch = Scratch::new();
    let (pinned, (major, minor, _)) = pinned_canon(&scratch.lock).expect("Cargo.lock pins Canon");
    let above = format!("{major}.{}.0", minor + 1);
    scratch.edit_manifest(|manifest| {
        entry_mut(manifest, SOFTWARE_CHANGE).canon = above.clone();
    });
    assert_eq!(
        scratch.check(),
        vec![Problem::CanonPin {
            path: SOFTWARE_CHANGE.to_owned(),
            canon: above,
            pinned,
        }]
    );

    let mut scratch = Scratch::new();
    let marker = format!("name = \"{CANON_PACKAGE}\"\nversion = \"");
    let start = scratch
        .lock
        .find(&marker)
        .unwrap_or_else(|| panic!("Cargo.lock has no {CANON_PACKAGE} package"))
        + marker.len();
    let end = start + scratch.lock[start..].find('"').expect("a quoted version");
    scratch.lock.replace_range(start..end, "0.1.0");
    let expected: Vec<Problem> = scratch
        .parsed_manifest()
        .protocols
        .into_iter()
        .filter(|entry| release(&entry.canon).is_some_and(|canon| canon > (0, 1, 0)))
        .map(|entry| Problem::CanonPin {
            path: entry.path,
            canon: entry.canon,
            pinned: "0.1.0".to_owned(),
        })
        .collect();
    assert!(
        expected.iter().any(
            |problem| matches!(problem, Problem::CanonPin { path, .. } if path == STORY_DELIVERY_2)
        ),
        "{STORY_DELIVERY_2} needs a Canon above 0.1.0"
    );
    let mut expected = expected;
    expected.sort();
    assert_eq!(scratch.check(), expected);
}

#[test]
fn an_understated_canon_is_not_detected() {
    let mut scratch = Scratch::new();
    scratch.edit_manifest(|manifest| {
        entry_mut(manifest, STORY_DELIVERY_2).canon = "0.1.0".to_owned();
    });
    assert_eq!(scratch.check(), Vec::<Problem>::new());
}

#[test]
fn a_field_the_schema_does_not_know_is_a_problem() {
    let unknown = |edit: fn(&mut serde_yaml_ng::Value)| {
        let mut scratch = Scratch::new();
        scratch.edit_manifest_yaml(edit);
        scratch.check()
    };
    let problems = [
        unknown(|value| {
            value["released"] = "2026-11-21".into();
        }),
        unknown(|value| {
            value["vocabulary"]["id"] = "vocabulary".into();
        }),
        unknown(|value| {
            value["protocols"][0]["status"] = "shipped".into();
        }),
    ];
    for (problems, field) in problems.iter().zip(["released", "id", "status"]) {
        assert!(
            matches!(
                problems.as_slice(),
                [Problem::Manifest(message)] if message.contains(&format!("unknown field `{field}`"))
            ),
            "{field}: {problems:?}"
        );
    }
}

#[test]
fn a_manifest_of_another_format_is_a_problem() {
    let mut scratch = Scratch::new();
    scratch.edit_manifest(|manifest| manifest.format = "b10x-protocol-catalog/2".to_owned());
    assert_eq!(
        scratch.check(),
        vec![Problem::Format("b10x-protocol-catalog/2".to_owned())]
    );
}

#[test]
fn a_path_outside_the_catalog_or_listed_twice_is_a_problem() {
    let mut scratch = Scratch::new();
    scratch.edit_manifest(|manifest| {
        let mut listed = entry_mut(manifest, SOFTWARE_CHANGE).clone();
        listed.path = MANIFEST.to_owned();
        manifest.protocols.push(listed);
    });
    assert_eq!(
        scratch.check(),
        vec![Problem::OutsideCatalog(MANIFEST.to_owned())]
    );

    let mut scratch = Scratch::new();
    scratch.edit_manifest(|manifest| {
        let mut listed = entry_mut(manifest, SOFTWARE_CHANGE).clone();
        listed.path = "protocols/../protocols/software-change/1.yaml".to_owned();
        manifest.protocols.push(listed);
    });
    assert_eq!(
        scratch.check(),
        vec![Problem::OutsideCatalog(
            "protocols/../protocols/software-change/1.yaml".to_owned()
        )]
    );

    let mut scratch = Scratch::new();
    scratch.edit_manifest(|manifest| {
        let listed = entry_mut(manifest, SOFTWARE_CHANGE).clone();
        manifest.protocols.push(listed);
    });
    assert_eq!(
        scratch.check(),
        vec![Problem::Duplicate(SOFTWARE_CHANGE.to_owned())]
    );
}

/// The schema file describes exactly the fields the Rust types accept: at each level its
/// `properties` are the keys the types serialise, every one is `required`, no other property is
/// allowed, and every property says what it means.
#[test]
fn the_schema_names_exactly_the_fields_the_types_accept() {
    let root = repository_root();
    let schema: serde_json::Value = serde_json::from_str(&read(&root, SCHEMA))
        .unwrap_or_else(|error| panic!("{SCHEMA}: {error}"));
    let sample = serde_json::to_value(Manifest {
        format: FORMAT.to_owned(),
        vocabulary: VocabularyEntry {
            path: VOCABULARY.to_owned(),
            sha256: digest(b""),
        },
        protocols: vec![ProtocolEntry {
            path: SOFTWARE_CHANGE.to_owned(),
            id: "software.change".to_owned(),
            revision: 1,
            sha256: digest(b""),
            canon: "0.1.0".to_owned(),
        }],
    })
    .expect("serialising a manifest");

    let levels = [
        ("/", &schema, &sample),
        (
            "/properties/vocabulary",
            &schema["properties"]["vocabulary"],
            &sample["vocabulary"],
        ),
        (
            "/properties/protocols/items",
            &schema["properties"]["protocols"]["items"],
            &sample["protocols"][0],
        ),
    ];
    for (pointer, schema, sample) in levels {
        let accepted: BTreeSet<&str> = sample
            .as_object()
            .unwrap_or_else(|| panic!("{pointer}: the sample is an object"))
            .keys()
            .map(String::as_str)
            .collect();
        let properties = schema["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{SCHEMA}{pointer} has no properties"));
        let described: BTreeSet<&str> = properties.keys().map(String::as_str).collect();
        assert_eq!(described, accepted, "{SCHEMA}{pointer}: properties");
        let required: BTreeSet<&str> = schema["required"]
            .as_array()
            .unwrap_or_else(|| panic!("{SCHEMA}{pointer} has no required"))
            .iter()
            .map(|name| name.as_str().expect("a required name is a string"))
            .collect();
        assert_eq!(required, accepted, "{SCHEMA}{pointer}: required");
        assert_eq!(
            schema["additionalProperties"],
            serde_json::Value::Bool(false),
            "{SCHEMA}{pointer}: additionalProperties"
        );
        for (name, property) in properties {
            assert!(
                property["description"]
                    .as_str()
                    .is_some_and(|text| !text.trim().is_empty()),
                "{SCHEMA}{pointer}/properties/{name} has no description"
            );
        }
    }
    assert_eq!(schema["properties"]["format"]["const"], FORMAT);
}
