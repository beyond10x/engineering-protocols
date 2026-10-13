//! Adversary pass 2 on `story:incident-response-subject-binding`: the documents around
//! `incident.response/1` read as specifications of what ships — the worked example in
//! `docs/examples/incident-response.md`, the vocabulary the protocol's names come from, and the
//! landing page's status rows in `website/product.json`.
//!
//! Every case reads its documents at run time; no file in the repository is written.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::{BTreeMap, BTreeSet};

use b10x_canon::ir::Ir;
use b10x_canon::model::{ArtifactId, Decision, Predicate, Truth};
use canon_engineering::vocabulary::{self, Category};
use serde_yaml_ng::Value;
use support::Fixture;

const INC_492: &str = "fixtures/incident-response/inc-492.fixture.yaml";
const EXAMPLE: &str = "docs/examples/incident-response.md";

fn read(path: &str) -> String {
    std::fs::read_to_string(support::repo_root().join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn truth(word: &str) -> Truth {
    match word {
        "TRUE" => Truth::True,
        "FALSE" => Truth::False,
        "UNKNOWN" => Truth::Unknown,
        other => panic!("`{other}` is not a truth value"),
    }
}

/// One ```text block of the example: the claim values it states, the obligations it lists as
/// open, and the actions it lists with the status their marker names.
#[derive(Default)]
struct Block {
    claims: BTreeMap<String, Truth>,
    obligations: Vec<String>,
    actions: BTreeMap<String, String>,
}

fn example_blocks() -> Vec<Block> {
    let text = read(EXAMPLE);
    let mut blocks = Vec::new();
    let mut current: Option<Block> = None;
    let mut section = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            match current.take() {
                Some(block) => blocks.push(block),
                None if trimmed == "```text" => current = Some(Block::default()),
                None => {}
            }
            section.clear();
            continue;
        }
        let Some(block) = current.as_mut() else {
            continue;
        };
        if trimmed.is_empty() {
            continue;
        }
        if let Some(header) = trimmed.strip_suffix(':') {
            section = header.to_owned();
            continue;
        }
        if let Some((id, value)) = trimmed.split_once(" = ") {
            block
                .claims
                .insert(id.trim().to_owned(), truth(value.trim()));
        } else if section.ends_with("obligation") {
            block.obligations.push(trimmed.to_owned());
        } else if section == "Actions" {
            let (id, status) = match trimmed.split_once(" [approval]") {
                Some((id, "")) => (id, "approval-required"),
                Some(_) => panic!("{EXAMPLE}: `{trimmed}` carries text after its marker"),
                None => (trimmed, "admissible"),
            };
            block.actions.insert(id.to_owned(), status.to_owned());
        }
    }
    assert!(current.is_none(), "{EXAMPLE}: unterminated text block");
    blocks
}

fn claims(decision: &Decision) -> BTreeMap<String, Truth> {
    decision
        .claims
        .iter()
        .map(|(id, entry)| (id.as_str().to_owned(), entry.value))
        .collect()
}

fn obligation(decision: &Decision, id: &str) -> String {
    decision
        .obligations
        .as_ref()
        .and_then(|section| section.as_array())
        .and_then(|entries| entries.iter().find(|entry| entry["id"] == id))
        .and_then(|entry| entry["status"].as_str())
        .unwrap_or_else(|| panic!("no obligation `{id}`"))
        .to_owned()
}

fn actions(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .actions
        .as_ref()
        .and_then(|section| section.as_object())
        .expect("an actions section")
        .iter()
        .map(|(id, entry)| {
            (
                id.clone(),
                entry["status"].as_str().expect("a status").to_owned(),
            )
        })
        .collect()
}

/// The unit's own contract, driven from the document: `inc-492` is "transcribed from
/// `docs/examples/incident-response.md`" (`incident_response_protocol.rs:5`). Canon's decision of
/// the fixture's first state is the example's first block (every claim; the urgent obligation
/// open; the listed actions with their status and every unlisted one blocked), and its decision
/// after the rollback and fresh health evidence is the second block, with emergency mode left
/// while the investigation stays open.
#[test]
fn the_incident_example_is_what_canon_decides_for_inc_492() {
    let blocks = example_blocks();
    assert_eq!(blocks.len(), 2, "{EXAMPLE} holds two text blocks");
    let fixture = Fixture::load(INC_492).unwrap_or_else(|error| panic!("{error}"));
    let compiled = fixture.compile().unwrap_or_else(|error| panic!("{error}"));
    let states = fixture.states();
    let first = states.first().expect("a first state").to_string();
    // The second block is the state after the rollback and fresh health evidence, before any
    // cause analysis of the new revision: `service-restored`, not the fixture's last state.
    let last = "service-restored".to_owned();
    assert!(states.contains(&last.as_str()), "inc-492 has `{last}`");
    let decide = |state: &str| {
        fixture
            .evaluate(&compiled, state)
            .unwrap_or_else(|refusal| panic!("state `{state}`: {refusal}"))
    };

    let before = decide(&first);
    assert_eq!(
        claims(&before),
        blocks[0].claims,
        "{EXAMPLE} block 1 against `{first}`"
    );
    assert_eq!(blocks[0].obligations, ["restore_service"]);
    assert_eq!(obligation(&before, "restore_service"), "open");
    let mut expected = blocks[0].actions.clone();
    for id in actions(&before).keys() {
        expected
            .entry(id.clone())
            .or_insert_with(|| "blocked".to_owned());
    }
    assert_eq!(
        actions(&before),
        expected,
        "{EXAMPLE} block 1 against `{first}`"
    );

    let after = decide(&last);
    assert_eq!(
        claims(&after),
        blocks[1].claims,
        "{EXAMPLE} block 2 against `{last}`"
    );
    assert_eq!(actions(&after)["emergency.leave"], "admissible");
    assert_eq!(obligation(&after, "restore_service"), "discharged");
    assert_eq!(obligation(&after, "investigate_cause"), "open");
}

/// Every artifact `incident.response/1` and `inc-492` name — the protocol's declarations, the
/// subject of every evidence match, the case's artifacts and every record's subject — is an
/// artifact kind of the engineering vocabulary, and the protocol describes each declared artifact
/// with the vocabulary's meaning. Every record's kind is an evidence kind the protocol declares.
#[test]
fn every_artifact_the_incident_protocol_and_inc_492_name_is_a_vocabulary_artifact_kind() {
    let fixture = Fixture::load(INC_492).unwrap_or_else(|error| panic!("{error}"));
    let compiled = fixture.compile().unwrap_or_else(|error| panic!("{error}"));
    let ir: &Ir = compiled.ir();

    let mut named: BTreeMap<String, &'static str> = BTreeMap::new();
    for id in ir.artifacts.keys() {
        named.insert(id.as_str().to_owned(), "declared");
    }
    let mut subjects = BTreeSet::new();
    let mut collect = |predicate: &Predicate| {
        predicate.visit(&mut |node| {
            if let Predicate::Evidence(matching) = node
                && let Some(subject) = &matching.subject
            {
                subjects.insert(subject.as_str().to_owned());
            }
        });
    };
    ir.claims
        .values()
        .for_each(|claim| collect(&claim.true_when));
    ir.obligations
        .values()
        .for_each(|obligation| collect(&obligation.discharged_when));
    ir.actions
        .values()
        .for_each(|action| collect(&action.precondition));
    assert_eq!(
        subjects.iter().collect::<Vec<_>>(),
        ["release", "service"],
        "the subjects the protocol's matches name"
    );
    for subject in subjects {
        named
            .entry(subject)
            .or_insert("an evidence match's subject");
    }
    let document: Value = serde_yaml_ng::from_str(&read(INC_492)).expect("the fixture is YAML");
    for artifact in document["case"]["artifacts"]
        .as_mapping()
        .expect("the case's artifacts")
        .keys()
    {
        named
            .entry(artifact.as_str().expect("an artifact id").to_owned())
            .or_insert("the case");
    }
    let last = *fixture.states().last().expect("a state");
    for record in fixture.evidence(last) {
        named
            .entry(record.subject.as_str().to_owned())
            .or_insert("a record's subject");
        assert!(
            ir.evidence_kinds.contains_key(&record.kind),
            "record `{}` is of kind `{}`, which the protocol does not declare",
            record.id.as_str(),
            record.kind.as_str()
        );
    }
    assert_eq!(
        named.keys().collect::<Vec<_>>(),
        ["release", "service"],
        "{named:?}"
    );

    for (id, source) in &named {
        let term =
            vocabulary::lookup(id).unwrap_or_else(|error| panic!("{source} `{id}`: {error}"));
        assert_eq!(term.category, Category::ArtifactKind, "{source} `{id}`");
        let described = ir.artifacts[&ArtifactId::new(id.as_str())]
            .description
            .as_deref();
        assert_eq!(
            described,
            Some(term.meaning),
            "the protocol describes artifact `{id}` otherwise than the vocabulary"
        );
    }
}

/// `website/product.json` and the status file its status section names, against what ships: every
/// protocol in `protocols/` has one status row, labelled `<id>/<revision>`, marked shipped and
/// linking the page `canon-engineering-docs` generates for it; every protocol row names a protocol
/// that ships; and a row saying every evidence match is bound to its artifact names a protocol in
/// which every match names a subject.
#[test]
fn the_landing_page_status_rows_are_the_shipped_protocols() {
    let product: Value =
        serde_yaml_ng::from_str(&read("website/product.json")).expect("JSON reads as YAML");
    // A status section names its `b10x-status/1` file (relative to `website/`) or lists its rows.
    let files: Vec<Value> = product["sections"]
        .as_sequence()
        .expect("sections")
        .iter()
        .filter(|section| section["kind"] == "status")
        .map(|section| match section["items"].as_str() {
            Some(path) => {
                let status: Value = serde_yaml_ng::from_str(&read(&format!("website/{path}")))
                    .expect("JSON reads as YAML");
                assert_eq!(status["format"], "b10x-status/1", "website/{path}");
                status["items"].clone()
            }
            None => section["items"].clone(),
        })
        .collect();
    assert!(!files.is_empty(), "the landing page has a status section");
    let rows: Vec<&Value> = files
        .iter()
        .flat_map(|items| items.as_sequence().expect("items").iter())
        .collect();
    let is_protocol_label = |label: &str| {
        label
            .rsplit_once('/')
            .is_some_and(|(id, major)| id.contains('.') && major.parse::<u64>().is_ok())
    };

    let mut shipped = BTreeMap::new();
    let directory = support::repo_root().join("protocols");
    let mut entries: Vec<_> = std::fs::read_dir(&directory)
        .expect("protocols/")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.is_dir())
        .collect();
    entries.sort();
    for dir in entries {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        for file in std::fs::read_dir(&dir).expect("a protocol directory") {
            let file = file.expect("an entry").path();
            let Some(major) = file
                .file_stem()
                .and_then(|stem| stem.to_str())
                .filter(|_| file.extension().is_some_and(|ext| ext == "yaml"))
                .map(str::to_owned)
            else {
                continue;
            };
            let compiled = support::compile_protocol(&format!("protocols/{name}/{major}.yaml"))
                .unwrap_or_else(|error| panic!("{error}"));
            let ir = compiled.ir().clone();
            let label = format!("{}/{}", ir.protocol.id.as_str(), ir.protocol.revision);
            shipped.insert(label, (format!("/docs/protocols/{name}/{major}"), ir));
        }
    }
    assert!(!shipped.is_empty(), "protocols/ ships a protocol");

    let mut wrong = Vec::new();
    for (label, (href, ir)) in &shipped {
        let matching: Vec<&&Value> = rows.iter().filter(|row| row["label"] == **label).collect();
        let [row] = matching.as_slice() else {
            wrong.push(format!(
                "{} status rows are labelled `{label}`",
                matching.len()
            ));
            continue;
        };
        if row["status"] != "shipped" {
            wrong.push(format!("`{label}` ships, the row says {:?}", row["status"]));
        }
        if row["href"].as_str() != Some(href.as_str()) {
            wrong.push(format!(
                "`{label}` row links {:?}, its generated page is {href}",
                row["href"]
            ));
        }
        let detail = row["detail"].as_str().unwrap_or_default();
        if detail.contains("every evidence match bound to its artifact") {
            let mut unbound = BTreeSet::new();
            let mut collect = |owner: String, predicate: &Predicate| {
                predicate.visit(&mut |node| {
                    if let Predicate::Evidence(matching) = node
                        && matching.subject.is_none()
                    {
                        unbound.insert(format!("{owner}: {}", matching.kind.as_str()));
                    }
                });
            };
            for (id, claim) in &ir.claims {
                collect(format!("claim {}", id.as_str()), &claim.true_when);
            }
            for (id, obligation) in &ir.obligations {
                collect(
                    format!("obligation {}", id.as_str()),
                    &obligation.discharged_when,
                );
            }
            for (id, action) in &ir.actions {
                collect(format!("action {}", id.as_str()), &action.precondition);
            }
            if !unbound.is_empty() {
                wrong.push(format!(
                    "`{label}` row says every evidence match is bound, unbound: {unbound:?}"
                ));
            }
        }
    }
    for row in &rows {
        let label = row["label"].as_str().expect("a label");
        if is_protocol_label(label) && !shipped.contains_key(label) {
            wrong.push(format!(
                "row `{label}` names no protocol in protocols/ (status {:?})",
                row["status"]
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Wrong-reason guard on `adversary2_incident_harness::an_artifact_the_case_omits`, which accepts
/// any `missing-artifact` refusal: inc-492 without the release is refused, at every state, for
/// the release and for nothing else.
#[test]
fn a_case_without_the_release_is_refused_for_the_release_at_every_state() {
    let text = read(INC_492);
    let line = "    release: {revision: r42}\n";
    assert!(text.contains(line), "inc-492's case lists the release");
    let fixture =
        Fixture::from_yaml(&text.replacen(line, "", 1)).unwrap_or_else(|error| panic!("{error}"));
    let compiled = fixture.compile().unwrap_or_else(|error| panic!("{error}"));
    let states: Vec<String> = fixture.states().into_iter().map(str::to_owned).collect();
    assert_eq!(states.len(), 7);
    for state in states {
        let refusal = fixture
            .evaluate(&compiled, &state)
            .expect_err("a case without the release is refused");
        assert_eq!(refusal.code(), "missing-artifact", "{state}: {refusal}");
        assert_eq!(
            refusal.to_string(),
            "case `INC-492` does not give the current revision of artifact `release`",
            "{state}"
        );
    }
}
