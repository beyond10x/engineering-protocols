//! Every derived input of the site: one page and one `b10x-protocol-graph/1` document per protocol,
//! the protocols index and the vocabulary, written below `website/` and checked there for drift.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail, ensure};
use b10x_canon::ir::{Ir, compile};
use b10x_canon::model::{one_line, parse};
use canon_engineering::vocabulary::{Category, Marking, Vocabulary};

use crate::graph;
use crate::markdown::{HEADER, code, text, yaml_string};
use crate::protocol::{self, Source};
use crate::status;

/// The Docusaurus site, relative to the repository root. Generated files live in its `docs/protocols/`,
/// `docs/vocabulary.md` and `data/protocol-graphs/`.
pub const SITE: &str = "website";

/// The generated graph documents, relative to the site. Every file here is generated.
const GRAPHS: &str = "data/protocol-graphs";
/// The example protocols, relative to the repository root: previews the site renders as graphs in
/// its concept pages. They are not shipped protocols.
pub const EXAMPLES: &str = "website/examples/protocols";

/// The generated example graph documents, relative to the site. Every file here is generated.
const EXAMPLE_GRAPHS: &str = "data/example-graphs";

/// What generation reads from the repository.
pub struct Inputs {
    pub vocabulary: String,
    pub protocols: Vec<Source>,
    /// Preview protocols under [`EXAMPLES`]; each yields a graph document and nothing else.
    pub examples: Vec<Source>,
}

/// The protocol name rule `crates/canon-engineering/build.rs` and the `canon-engineering` binary
/// apply, from the same file.
#[allow(dead_code)] // the generator uses the name rule only
#[path = "../../canon-engineering/src/builtin_name.rs"]
mod builtin_name;

use builtin_name::is_name;

/// Every `<relative>/<name>/<major>.yaml` under `root`, sorted by name and major.
fn sources(root: &Path, relative: &str) -> Result<Vec<Source>> {
    let dir = root.join(relative);
    let mut found = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|name| anyhow!("protocol directory {name:?} is not UTF-8"))?;
        ensure!(
            is_name(&name),
            "protocol directory `{}` must be lowercase letters, digits and hyphens",
            one_line(&name)
        );
        for file in fs::read_dir(entry.path())? {
            let file = file?;
            let file_name = file
                .file_name()
                .into_string()
                .map_err(|name| anyhow!("protocol file {name:?} is not UTF-8"))?;
            let Some(stem) = file_name.strip_suffix(".yaml") else {
                continue;
            };
            let path = format!("{relative}/{name}/{file_name}");
            let major = stem
                .parse::<u64>()
                .ok()
                .filter(|major| *major > 0 && major.to_string() == stem)
                .with_context(|| {
                    format!("{}: a protocol file is named <major>.yaml", one_line(&path))
                })?;
            let text =
                fs::read_to_string(file.path()).with_context(|| format!("reading {path}"))?;
            found.push(Source {
                name: name.clone(),
                major,
                path,
                text,
            });
        }
    }
    found.sort_by(|a, b| (&a.name, a.major).cmp(&(&b.name, b.major)));
    Ok(found)
}

impl Inputs {
    /// Reads `protocols/vocabulary.yaml`, every `protocols/<name>/<major>.yaml` and, when the
    /// directory exists, every example under [`EXAMPLES`].
    pub fn read(root: &Path) -> Result<Self> {
        let vocabulary_path = root.join("protocols/vocabulary.yaml");
        let vocabulary = fs::read_to_string(&vocabulary_path)
            .with_context(|| format!("reading {}", vocabulary_path.display()))?;
        let protocols = sources(root, "protocols")?;
        let examples = if root.join(EXAMPLES).is_dir() {
            sources(root, EXAMPLES)?
        } else {
            Vec::new()
        };
        Ok(Self {
            vocabulary,
            protocols,
            examples,
        })
    }
}

fn category_title(category: Category) -> &'static str {
    match category {
        Category::ArtifactKind => "Artifact kinds",
        Category::EvidenceKind => "Evidence kinds",
        Category::ClaimId => "Claims",
        Category::ActionId => "Actions",
        Category::ObligationId => "Obligations",
        Category::OutcomeId => "Outcomes",
        Category::DecisionName => "Decisions",
        Category::CaseInput => "Case inputs",
        Category::InputValue => "Input values",
        Category::IndependenceDimension => "Independence dimensions",
    }
}

fn vocabulary_page(vocabulary: &Vocabulary) -> String {
    let terms = vocabulary.terms();
    let mut out = format!(
        "---\n{HEADER}\ntitle: \"Engineering vocabulary\"\nsidebar_position: 3\ndescription: \"The names the engineering protocols use, each with its category, marking and meaning.\"\ncustom_edit_url: null\n---\n\nThe {} names the engineering protocols use, each declared once in [`protocols/vocabulary.yaml`](https://github.com/beyond10x/engineering-protocols/blob/main/protocols/vocabulary.yaml) with its category, its marking and its meaning. The vocabulary holds names and what they mean; what a claim, a piece of evidence or an obligation is, and how one is decided, belongs to Canon.\n\n- **`core`**: not specific to Git, pull requests or code; usable by every engineering protocol.\n- **`software.change`**: only makes sense for Git, pull requests or code.\n",
        terms.len()
    );
    for category in Category::ALL {
        let _ = write!(
            out,
            "\n## {}\n\nCategory {}.\n\n",
            category_title(category),
            code(category.as_str())
        );
        let in_category: Vec<_> = terms
            .iter()
            .filter(|term| term.category == category)
            .collect();
        if in_category.is_empty() {
            out.push_str("*No term in this category.*\n");
            continue;
        }
        out.push_str("| Term | Marking | Meaning |\n|---|---|---|\n");
        for term in in_category {
            let marking = match term.marking {
                Marking::Core => "core",
                Marking::SoftwareChange => "software.change",
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                code(term.id),
                code(marking),
                text(term.meaning)
            );
        }
    }
    out
}

struct Shipped {
    file: String,
    title: String,
    revision: u64,
    description: Option<String>,
    counts: String,
}

fn index_page(shipped: &[Shipped]) -> String {
    let mut out = format!(
        "---\n{HEADER}\ntitle: \"Protocols\"\ndescription: \"The shipped engineering protocols, each rendered from its compiled Canon protocol/1 document.\"\ncustom_edit_url: null\n---\n\nEvery shipped engineering protocol is a Canon `protocol/1` document at `protocols/<name>/<major>.yaml`. Each page here is generated from the document's compiled form, `canon-ir/1`, so it shows what Canon understood.\n\n"
    );
    if shipped.is_empty() {
        out.push_str("**No protocol is shipped yet.** This page lists each protocol as soon as its document lands in `protocols/`. The engineering [vocabulary](../vocabulary.md) ships today.\n");
    } else {
        out.push_str("| Protocol | Revision | Description | Declares |\n|---|---|---|---|\n");
        for protocol in shipped {
            let _ = writeln!(
                out,
                "| [{}](./{}) | {} | {} | {} |",
                code(&protocol.title),
                protocol
                    .file
                    .strip_prefix("protocols/")
                    .unwrap_or(&protocol.file),
                protocol.revision,
                protocol
                    .description
                    .as_deref()
                    .map(|d| text(d.trim()))
                    .unwrap_or_default(),
                protocol.counts
            );
        }
    }
    out
}

/// Parses and compiles one protocol document with Canon; any problem names the document.
fn compiled(source: &Source) -> Result<Ir> {
    let document = parse(&source.text).map_err(|error| anyhow!("{}: {error}", source.path))?;
    compile(&document).map_err(|problems| {
        let problems: Vec<String> = problems.iter().map(ToString::to_string).collect();
        anyhow!("{}: {}", source.path, problems.join("; "))
    })
}

/// Every generated file, by path below `website/`.
pub fn render(inputs: &Inputs) -> Result<BTreeMap<String, String>> {
    let vocabulary = Vocabulary::from_yaml(&inputs.vocabulary)
        .map_err(|error| anyhow!("protocols/vocabulary.yaml: {error}"))?;
    let mut files = BTreeMap::new();
    files.insert(
        "docs/reference/assertions.md".into(),
        canon_engineering_assertions::documentation::reference()?,
    );
    let mut shipped = Vec::new();
    let mut statuses = Vec::new();
    for source in &inputs.protocols {
        let ir = compiled(source)?;
        files.insert(
            format!("docs/{}", source.file()),
            protocol::page(&ir, source),
        );
        let graph = graph::document(&ir, &source.path);
        files.insert(
            source.graph_file(),
            serde_json::to_string_pretty(&graph)? + "\n",
        );
        files
            .entry(format!("docs/protocols/{}/_category_.yml", source.name))
            .or_insert_with(|| {
                format!(
                    "{HEADER}\nlabel: {}\n",
                    yaml_string(ir.protocol.id.as_str())
                )
            });
        shipped.push(Shipped {
            file: source.file(),
            title: format!("{}/{}", ir.protocol.id, source.major),
            revision: ir.protocol.revision,
            description: ir.protocol.description.clone(),
            counts: format!(
                "{} claims, {} actions, {} obligations, {} outcomes",
                ir.claims.len(),
                ir.actions.len(),
                ir.obligations.len(),
                ir.outcomes.len()
            ),
        });
        statuses.push((source, ir));
    }
    let protocols: Vec<_> = statuses.iter().map(|(source, ir)| (*source, ir)).collect();
    files.insert(
        status::STATUS_FILE.to_owned(),
        serde_json::to_string_pretty(&status::document(&vocabulary, &protocols))? + "\n",
    );
    files.insert(status::STATUS_PAGE.to_owned(), status::page());
    files.insert("docs/protocols/index.md".to_owned(), index_page(&shipped));
    files.insert(
        "docs/protocols/_category_.yml".to_owned(),
        format!("{HEADER}\nlabel: Protocols\nposition: 2\n"),
    );
    files.insert(
        "docs/vocabulary.md".to_owned(),
        vocabulary_page(&vocabulary),
    );
    for example in &inputs.examples {
        let graph = graph::document(&compiled(example)?, &example.path);
        files.insert(
            format!("{EXAMPLE_GRAPHS}/{}-{}.json", example.name, example.major),
            serde_json::to_string_pretty(&graph)? + "\n",
        );
    }
    Ok(files)
}

/// Whether a file at `path` (relative to the site) is one this generator writes: every file under
/// the two graph directories, the status document, and any other file headed with [`HEADER`].
fn is_generated(path: &str, content: &str) -> bool {
    path == status::STATUS_FILE
        || path.starts_with(&format!("{GRAPHS}/"))
        || path.starts_with(&format!("{EXAMPLE_GRAPHS}/"))
        || content.lines().take(2).any(|line| line == HEADER)
}

/// Writes `files` into the site and removes generated files that are no longer produced; with
/// `check`, changes nothing and fails when the tree differs. Returns what differed.
pub fn apply(site: &Path, files: &BTreeMap<String, String>, check: bool) -> Result<Vec<String>> {
    let mut drift = Vec::new();
    for (path, content) in files {
        let target = site.join(path);
        match fs::read_to_string(&target) {
            Ok(existing) if existing == *content => continue,
            Ok(_) => drift.push(format!("{path} differs from a fresh render")),
            Err(_) => drift.push(format!("{path} is missing")),
        }
        if !check {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, content).with_context(|| format!("writing {}", target.display()))?;
        }
    }
    let roots = ["docs/protocols", GRAPHS, EXAMPLE_GRAPHS];
    let mut pending: Vec<String> = roots.iter().map(|root| (*root).to_owned()).collect();
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(site.join(&dir)) else {
            continue;
        };
        for entry in entries {
            let entry = entry?;
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            let path = format!("{dir}/{name}");
            if entry.file_type()?.is_dir() {
                pending.push(path);
                continue;
            }
            if files.contains_key(&path) {
                continue;
            }
            if fs::read_to_string(entry.path()).is_ok_and(|content| is_generated(&path, &content)) {
                drift.push(format!("{path} is generated but no longer produced"));
                if !check {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        if !check && !roots.contains(&dir.as_str()) {
            let _ = fs::remove_dir(site.join(&dir));
        }
    }
    if check && !drift.is_empty() {
        bail!(
            "generated documentation is stale; run `canon-engineering-docs generate`:\n  {}",
            drift.join("\n  ")
        );
    }
    Ok(drift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const VOCABULARY: &str = include_str!("../../../protocols/vocabulary.yaml");
    /// Canon's `fixtures/investigation/protocol.yaml` at canon `32ae0d0`, copied byte for byte.
    /// It is a test input only and is never shipped under `protocols/`.
    const INVESTIGATION: &str = include_str!("../tests/fixtures/investigation.yaml");

    fn root() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    fn with(protocols: Vec<Source>) -> Result<BTreeMap<String, String>> {
        render(&Inputs {
            vocabulary: VOCABULARY.to_owned(),
            protocols,
            examples: Vec::new(),
        })
    }

    fn source(name: &str, text: &str) -> Source {
        Source {
            name: name.to_owned(),
            major: 1,
            path: format!("protocols/{name}/1.yaml"),
            text: text.to_owned(),
        }
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "canon-engineering-docs-test-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    #[test]
    fn the_investigation_fixture_page_shows_every_claim_action_and_outcome() {
        let files = with(vec![source("investigation", INVESTIGATION)]).expect("renders");
        let page = &files["docs/protocols/investigation/1.mdx"];
        let graph: Value =
            serde_json::from_str(&files["data/protocol-graphs/investigation-1.json"])
                .expect("json");
        let ir = compile(&parse(INVESTIGATION).expect("parses")).expect("compiles");
        let groups: [(&str, Vec<String>); 4] = [
            ("claim", ir.claims.keys().map(ToString::to_string).collect()),
            (
                "action",
                ir.actions.keys().map(ToString::to_string).collect(),
            ),
            (
                "outcome",
                ir.outcomes.keys().map(ToString::to_string).collect(),
            ),
            (
                "evidence",
                ir.evidence_kinds.keys().map(ToString::to_string).collect(),
            ),
        ];
        assert_eq!(groups.each_ref().map(|(_, ids)| ids.len()), [1, 2, 1, 2]);
        for (kind, ids) in &groups {
            assert_eq!(
                &graph::names(&graph, kind),
                ids,
                "{kind} nodes in the graph"
            );
            for id in ids {
                assert!(
                    page.contains(&format!("\n| `{id}` |")),
                    "{id} has no table row"
                );
            }
        }
        assert!(page.starts_with(&format!(
            "---\n{HEADER}\nid: \"1\"\ntitle: \"investigation/1\"\n"
        )));
        assert!(page.contains("slug: /protocols/investigation/1\n"));
        assert!(
            page.contains("import graph from '@site/data/protocol-graphs/investigation-1.json';\n")
        );
        assert!(page.contains("<ProtocolGraph data={graph} />"));
        assert!(page.contains("all of (evidence `falsification_attempt` with result `survived`; evidence `supporting_observation`)"));
        assert_eq!(graph["edges"].as_array().map(Vec::len), Some(5));
        assert_eq!(
            graph["protocol"]["source"],
            "protocols/investigation/1.yaml"
        );
        assert!(page.contains("*This protocol declares no obligations.*"));
        let index = &files["docs/protocols/index.md"];
        assert!(index.contains("[`investigation/1`](./investigation/1.mdx)"));
        assert!(!index.contains("No protocol is shipped yet"));
    }

    #[test]
    fn with_no_protocol_the_index_says_none_is_shipped() {
        let files = with(Vec::new()).expect("renders");
        assert!(files["docs/protocols/index.md"].contains("**No protocol is shipped yet.**"));
        assert_eq!(
            files.keys().map(String::as_str).collect::<Vec<_>>(),
            [
                "data/status.json",
                "docs/protocols/_category_.yml",
                "docs/protocols/index.md",
                "docs/reference/assertions.md",
                "docs/status.mdx",
                "docs/vocabulary.md"
            ]
        );
        assert!(
            files
                .iter()
                .all(|(path, content)| is_generated(path, content))
        );
    }

    #[test]
    fn an_example_yields_only_a_graph_document_and_drift_in_it_is_caught() {
        let mut example = source("investigation", INVESTIGATION);
        example.path = format!("{EXAMPLES}/investigation/1.yaml");
        let files = render(&Inputs {
            vocabulary: VOCABULARY.to_owned(),
            protocols: Vec::new(),
            examples: vec![example],
        })
        .expect("renders");
        let graph: Value =
            serde_json::from_str(&files["data/example-graphs/investigation-1.json"]).expect("json");
        assert_eq!(
            graph["protocol"]["source"],
            "website/examples/protocols/investigation/1.yaml"
        );
        assert!(
            !files
                .keys()
                .any(|path| path.starts_with("docs/protocols/investigation"))
        );
        assert!(files["docs/protocols/index.md"].contains("**No protocol is shipped yet.**"));

        let site = scratch("examples");
        apply(&site, &files, false).expect("writes");
        let without = with(Vec::new()).expect("renders");
        let error = apply(&site, &without, true).expect_err("stale").to_string();
        assert!(
            error.contains(
                "data/example-graphs/investigation-1.json is generated but no longer produced"
            ),
            "{error}"
        );
        let _ = fs::remove_dir_all(&site);
    }

    #[test]
    fn the_status_document_has_one_shipped_row_per_protocol() {
        let files = with(vec![source("investigation", INVESTIGATION)]).expect("renders");
        let status: Value = serde_json::from_str(&files["data/status.json"]).expect("json");
        assert_eq!(status["format"], "b10x-status/1");
        let items = status["items"].as_array().expect("items");
        let protocols: Vec<&Value> = items
            .iter()
            .filter(|item| item["area"] == "Protocols")
            .collect();
        assert_eq!(protocols.len(), 1);
        assert_eq!(protocols[0]["label"], "investigation/1");
        assert_eq!(protocols[0]["status"], "shipped");
        assert_eq!(protocols[0]["href"], "/docs/protocols/investigation/1");
        assert_eq!(
            protocols[0]["detail"],
            "Canon protocol/1 data declaring 1 claim, 2 evidence kinds, 2 actions and 1 outcome."
        );
        let terms = Vocabulary::from_yaml(VOCABULARY)
            .expect("vocabulary")
            .terms()
            .len();
        let vocabulary = items
            .iter()
            .find(|item| item["label"] == "Engineering vocabulary")
            .expect("vocabulary row");
        assert!(
            vocabulary["detail"]
                .as_str()
                .is_some_and(|detail| detail.starts_with(&format!("{terms} terms,"))),
            "{vocabulary}"
        );
        let mut labels: Vec<&str> = items
            .iter()
            .filter_map(|item| item["label"].as_str())
            .collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), count, "labels are unique");

        let without = with(Vec::new()).expect("renders");
        let status: Value = serde_json::from_str(&without["data/status.json"]).expect("json");
        assert!(
            status["items"]
                .as_array()
                .expect("items")
                .iter()
                .all(|item| item["area"] != "Protocols")
        );
    }

    #[test]
    fn the_vocabulary_page_lists_every_term() {
        let files = with(Vec::new()).expect("renders");
        let page = &files["docs/vocabulary.md"];
        let vocabulary = Vocabulary::from_yaml(VOCABULARY).expect("vocabulary");
        assert_eq!(vocabulary.terms().len(), 93);
        assert!(
            page.contains("\n## Independence dimensions\n\nCategory `independence_dimension`.")
        );
        for term in vocabulary.terms() {
            assert!(
                page.contains(&format!("\n| `{}` |", term.id)),
                "{}",
                term.id
            );
        }
    }

    #[test]
    fn an_invalid_protocol_is_refused_with_its_path() {
        let broken = INVESTIGATION.replace("evidence: supporting_observation", "evidence: hearsay");
        let error = with(vec![source("investigation", &broken)])
            .expect_err("refused")
            .to_string();
        assert!(
            error.starts_with("protocols/investigation/1.yaml:"),
            "{error}"
        );
        assert!(error.contains("hearsay"), "{error}");
    }

    #[test]
    fn document_text_cannot_inject_markup() {
        let hostile = INVESTIGATION.replace(
            "description: Try to refute the explanation.",
            "description: \"<script>x</script> {danger} | [link](/elsewhere)\"",
        );
        let files = with(vec![source("investigation", &hostile)]).expect("escaped, not refused");
        let page = &files["docs/protocols/investigation/1.mdx"];
        assert!(
            page.contains("\\<script\\>x\\</script\\> \\{danger\\} \\| \\[link\\](/elsewhere)")
        );
    }

    #[test]
    fn the_committed_site_inputs_are_fresh_renders() {
        let files = render(&Inputs::read(root()).expect("reads protocols/")).expect("renders");
        apply(&root().join(SITE), &files, true).expect("committed files match a fresh render");
    }

    #[test]
    fn check_fails_on_a_hand_edit_or_a_stale_file_and_generate_repairs_both() {
        let site = scratch("drift");
        let files = with(vec![source("investigation", INVESTIGATION)]).expect("renders");
        apply(&site, &files, false).expect("writes");
        assert!(apply(&site, &files, true).expect("fresh").is_empty());

        let page = site.join("docs/protocols/investigation/1.mdx");
        let edited = fs::read_to_string(&page).expect("page") + "\nhand edit\n";
        fs::write(&page, edited).expect("edit");
        let error = apply(&site, &files, true).expect_err("drift").to_string();
        assert!(
            error.contains("docs/protocols/investigation/1.mdx differs"),
            "{error}"
        );

        let without = with(Vec::new()).expect("renders");
        let error = apply(&site, &without, true).expect_err("stale").to_string();
        for stale in [
            "docs/protocols/investigation/1.mdx",
            "data/protocol-graphs/investigation-1.json",
        ] {
            assert!(
                error.contains(&format!("{stale} is generated but no longer produced")),
                "{error}"
            );
        }
        apply(&site, &without, false).expect("repairs");
        assert!(!page.exists());
        assert!(
            !site
                .join("data/protocol-graphs/investigation-1.json")
                .exists()
        );
        assert!(apply(&site, &without, true).expect("fresh").is_empty());
        let _ = fs::remove_dir_all(&site);
    }
}
