//! Acceptance for `story:vocabulary-yaml-source`: the engineering vocabulary is data.
//!
//! `protocols/vocabulary.yaml` is the source of the vocabulary, and `vocabulary.rs` is its typed
//! reader. Item 4 of the story (the `story:els-vocabulary` suites pass unedited) is decided by
//! running `tests/vocabulary.rs` and `tests/adversary_vocabulary.rs`, not here.

use std::path::{Path, PathBuf};

use canon_engineering::vocabulary::{self, Category, Marking, Vocabulary};

use Category::{ActionId as Action, ArtifactKind as Artifact, ClaimId as Claim};
use Category::{EvidenceKind as Evidence, ObligationId as Obligation, OutcomeId as Outcome};
use Marking::{Core, SoftwareChange};

/// The 35 terms of `crates/els/src/vocabulary.rs` at `13d180f`, in its order, but
/// `repository.edit`, which `story:software-change-profiles` split into `tests.write` and
/// `implementation.edit`:
/// (id, category, marking, meaning).
const EXPECTED: &[(&str, Category, Marking, &str)] = &[
    (
        "intent",
        Artifact,
        Core,
        "what the change or response is meant to achieve",
    ),
    (
        "system_specification",
        Artifact,
        Core,
        "the specified behaviour of the system concerned",
    ),
    (
        "plan",
        Artifact,
        Core,
        "the intended steps toward the intent",
    ),
    (
        "release",
        Artifact,
        Core,
        "a versioned, deployable unit of the system",
    ),
    (
        "deployment",
        Artifact,
        Core,
        "a release running in an environment",
    ),
    (
        "service",
        Artifact,
        Core,
        "the running service a case concerns",
    ),
    (
        "implementation",
        Artifact,
        SoftwareChange,
        "a revision of the code that realizes the plan",
    ),
    (
        "operational_observation",
        Evidence,
        Core,
        "an observation of the running system",
    ),
    (
        "objective_observation",
        Evidence,
        Core,
        "an observation of whether the intent's objective is met",
    ),
    (
        "impact_assessment",
        Evidence,
        Core,
        "an assessment of who and what an incident affects",
    ),
    (
        "cause_analysis",
        Evidence,
        Core,
        "an analysis of why an incident happened",
    ),
    (
        "test_result",
        Evidence,
        SoftwareChange,
        "the result of running tests against an implementation revision",
    ),
    (
        "code_review",
        Evidence,
        SoftwareChange,
        "a review of an implementation revision",
    ),
    (
        "build_provenance",
        Evidence,
        SoftwareChange,
        "the record of how a release was built from an implementation",
    ),
    (
        "release.proven",
        Claim,
        Core,
        "the release comes from a verified implementation and carries current build provenance",
    ),
    (
        "deployment.healthy",
        Claim,
        Core,
        "the deployment is running healthily",
    ),
    (
        "objective.realized",
        Claim,
        Core,
        "the intent's objective is met",
    ),
    (
        "impact.bounded",
        Claim,
        Core,
        "the incident's impact is known and contained",
    ),
    (
        "service.healthy",
        Claim,
        Core,
        "the service is running healthily",
    ),
    (
        "cause.identified",
        Claim,
        Core,
        "the incident's cause is known",
    ),
    (
        "tests.pass",
        Claim,
        SoftwareChange,
        "the tests pass for the current implementation revision",
    ),
    (
        "implementation.reviewed",
        Claim,
        SoftwareChange,
        "the current implementation revision is reviewed",
    ),
    (
        "implementation.verified",
        Claim,
        SoftwareChange,
        "the current implementation revision is verified",
    ),
    ("metrics.inspect", Action, Core, "read the system's metrics"),
    ("logs.search", Action, Core, "search the system's logs"),
    (
        "release.inspect",
        Action,
        Core,
        "read what a release contains",
    ),
    (
        "release.rollback",
        Action,
        Core,
        "return to a previous release",
    ),
    (
        "traffic.shift",
        Action,
        Core,
        "move traffic between deployments",
    ),
    ("emergency.leave", Action, Core, "leave emergency mode"),
    (
        "repository.inspect",
        Action,
        SoftwareChange,
        "read the repository",
    ),
    (
        "repository.merge",
        Action,
        SoftwareChange,
        "merge a revision into the repository's mainline",
    ),
    (
        "tests.run",
        Action,
        SoftwareChange,
        "run the tests against an implementation revision",
    ),
    (
        "restore_service",
        Obligation,
        Core,
        "bring the affected service back to health",
    ),
    (
        "accepted",
        Outcome,
        Core,
        "the case closed with its result accepted",
    ),
];

/// The term item 3 appends to the YAML text, and nowhere else.
const APPENDED: &str = "  - id: example_term
    category: evidence_kind
    marking: core
    meaning: a term added only to the YAML
";

type Row = (String, Category, Marking, String);

#[test]
fn vocabulary_yaml_is_the_source() {
    assert_eq!(EXPECTED.len(), 34, "the expected list has the 34 terms");
    let expected: Vec<Row> = EXPECTED
        .iter()
        .map(|(id, c, m, meaning)| (id.to_string(), *c, *m, meaning.to_string()))
        .collect();
    let text = read_to_string(&vocabulary_yaml());

    // 1. The file's first 34 entries are the 34 terms, in order, read without the crate's
    //    reader. Later stories append their own terms after them and assert those themselves.
    let from_file = rows_in_document(&text);
    assert!(
        from_file.len() >= 34,
        "protocols/vocabulary.yaml holds {} entries",
        from_file.len()
    );
    assert_eq!(
        from_file[..34],
        expected[..],
        "protocols/vocabulary.yaml does not open with the 34 terms of vocabulary.rs at 13d180f"
    );

    // 2. The built-in API returns exactly those entries in file order and finds each by id.
    let built_in: Vec<Row> = vocabulary::terms()
        .iter()
        .map(|t| {
            (
                t.id.to_string(),
                t.category,
                t.marking,
                t.meaning.to_string(),
            )
        })
        .collect();
    assert_eq!(built_in, from_file, "vocabulary::terms() is not the file");
    for (id, category, marking, meaning) in &from_file {
        let term = vocabulary::lookup(id).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(term.id, id.as_str());
        assert_eq!(term.category, *category, "`{id}`");
        assert_eq!(term.marking, *marking, "`{id}`");
        assert_eq!(term.meaning, meaning.as_str(), "`{id}`");
    }

    // 3. The typed reader reads the file as the built-in API does, and a term appended only to
    //    the YAML text appears through it.
    let read = Vocabulary::from_yaml(&text).expect("protocols/vocabulary.yaml reads");
    let read_rows: Vec<Row> = read
        .terms()
        .iter()
        .map(|t| {
            (
                t.id.to_string(),
                t.category,
                t.marking,
                t.meaning.to_string(),
            )
        })
        .collect();
    assert_eq!(
        read_rows, built_in,
        "the reader and vocabulary::terms() differ"
    );
    assert!(
        text.ends_with('\n'),
        "protocols/vocabulary.yaml ends without a newline"
    );
    let extended_text = format!("{text}{APPENDED}");
    assert_eq!(
        rows_in_document(&extended_text).len(),
        from_file.len() + 1,
        "the appended term does not land in the terms list"
    );
    let extended = Vocabulary::from_yaml(&extended_text).expect("the extended text reads");
    assert_eq!(extended.terms().len(), from_file.len() + 1);
    let added = extended
        .lookup("example_term")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(added.id, "example_term");
    assert_eq!(added.category, Category::EvidenceKind);
    assert_eq!(added.marking, Marking::Core);
    assert_eq!(added.meaning, "a term added only to the YAML");
    assert_eq!(extended.lookup("intent").map(|t| t.id), Ok("intent"));
    let refusal = vocabulary::lookup("example_term")
        .map(|t| t.id)
        .expect_err("the built-in vocabulary has no `example_term`");
    assert_eq!(refusal.term, "example_term");

    // The story's outcome: vocabulary.rs holds no term of its own.
    let source = read_to_string(&manifest_dir().join("src/vocabulary.rs"));
    for (id, ..) in &from_file {
        assert!(
            !source.contains(&format!("\"{id}\"")),
            "src/vocabulary.rs spells the term `{id}` itself"
        );
    }
}

/// The Rust API produces the data as well as reading it (Atlas ADR 0077 point 3): what it writes
/// reads back to the same terms, and what it writes is checked like any other document.
#[test]
fn vocabulary_round_trips_through_yaml() {
    let built_in = Vocabulary::from_yaml(&read_to_string(&vocabulary_yaml()))
        .expect("protocols/vocabulary.yaml reads");
    let written = built_in.to_yaml();
    let read_back = Vocabulary::from_yaml(&written)
        .unwrap_or_else(|e| panic!("the written vocabulary does not read back: {e}\n{written}"));
    assert_eq!(read_back.terms(), vocabulary::terms());
    assert_eq!(read_back, built_in);

    let empty_meaning =
        Vocabulary::default().with_term("example_term", Category::EvidenceKind, Marking::Core, "");
    let refusal = Vocabulary::from_yaml(&empty_meaning.to_yaml())
        .expect_err("a term built in Rust without a meaning is refused when read back");
    assert!(
        refusal.to_string().contains("example_term"),
        "refusal `{refusal}` does not name the term"
    );

    let one = Vocabulary::default().with_term(
        "example_term",
        Category::EvidenceKind,
        Marking::Core,
        "a term built in Rust",
    );
    let one_back = Vocabulary::from_yaml(&one.to_yaml()).expect("a well-formed term reads back");
    assert_eq!(one_back, one);
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is unset: run this test through cargo"),
    )
}

fn vocabulary_yaml() -> PathBuf {
    manifest_dir().join("../../protocols/vocabulary.yaml")
}

fn read_to_string(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// The document's `terms` as rows, read as plain YAML with the spelling the file uses.
fn rows_in_document(text: &str) -> Vec<Row> {
    let doc: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).expect("the text is YAML");
    assert_eq!(
        doc.get("format").and_then(|f| f.as_str()),
        Some("vocabulary/1"),
        "the document's format"
    );
    let terms = doc
        .get("terms")
        .and_then(|t| t.as_sequence())
        .expect("the document has a `terms` list");
    terms
        .iter()
        .map(|entry| {
            let field = |name: &str| {
                entry
                    .get(name)
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| panic!("an entry has no string `{name}`: {entry:?}"))
                    .to_owned()
            };
            let category = match field("category").as_str() {
                "artifact_kind" => Category::ArtifactKind,
                "evidence_kind" => Category::EvidenceKind,
                "claim_id" => Category::ClaimId,
                "action_id" => Category::ActionId,
                "obligation_id" => Category::ObligationId,
                "outcome_id" => Category::OutcomeId,
                "decision_name" => Category::DecisionName,
                "case_input" => Category::CaseInput,
                "input_value" => Category::InputValue,
                "independence_dimension" => Category::IndependenceDimension,
                other => panic!("unknown category `{other}`"),
            };
            let marking = match field("marking").as_str() {
                "core" => Marking::Core,
                "software.change" => Marking::SoftwareChange,
                "self.improvement" => Marking::SelfImprovement,
                "loop.improvement" => Marking::LoopImprovement,
                other => panic!("unknown marking `{other}`"),
            };
            (field("id"), category, marking, field("meaning"))
        })
        .collect()
}
