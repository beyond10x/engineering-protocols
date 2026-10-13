//! Adversary cases for `story:els-vocabulary`: what the acceptance test does not pin.

use std::collections::BTreeSet;
use std::path::Path;

use canon_engineering::vocabulary::{self, Category, Marking};

/// The story's *Term set declared here* table, written independently of the unit's own copy,
/// without `repository.edit`, which `story:software-change-profiles` split.
const STORY_TABLE: &[(&str, Category, Marking)] = &[
    ("intent", Category::ArtifactKind, Marking::Core),
    (
        "system_specification",
        Category::ArtifactKind,
        Marking::Core,
    ),
    ("plan", Category::ArtifactKind, Marking::Core),
    ("release", Category::ArtifactKind, Marking::Core),
    ("deployment", Category::ArtifactKind, Marking::Core),
    ("service", Category::ArtifactKind, Marking::Core),
    (
        "implementation",
        Category::ArtifactKind,
        Marking::SoftwareChange,
    ),
    (
        "operational_observation",
        Category::EvidenceKind,
        Marking::Core,
    ),
    (
        "objective_observation",
        Category::EvidenceKind,
        Marking::Core,
    ),
    ("impact_assessment", Category::EvidenceKind, Marking::Core),
    ("cause_analysis", Category::EvidenceKind, Marking::Core),
    (
        "test_result",
        Category::EvidenceKind,
        Marking::SoftwareChange,
    ),
    (
        "code_review",
        Category::EvidenceKind,
        Marking::SoftwareChange,
    ),
    (
        "build_provenance",
        Category::EvidenceKind,
        Marking::SoftwareChange,
    ),
    ("release.proven", Category::ClaimId, Marking::Core),
    ("deployment.healthy", Category::ClaimId, Marking::Core),
    ("objective.realized", Category::ClaimId, Marking::Core),
    ("impact.bounded", Category::ClaimId, Marking::Core),
    ("service.healthy", Category::ClaimId, Marking::Core),
    ("cause.identified", Category::ClaimId, Marking::Core),
    ("tests.pass", Category::ClaimId, Marking::SoftwareChange),
    (
        "implementation.reviewed",
        Category::ClaimId,
        Marking::SoftwareChange,
    ),
    (
        "implementation.verified",
        Category::ClaimId,
        Marking::SoftwareChange,
    ),
    ("metrics.inspect", Category::ActionId, Marking::Core),
    ("logs.search", Category::ActionId, Marking::Core),
    ("release.inspect", Category::ActionId, Marking::Core),
    ("release.rollback", Category::ActionId, Marking::Core),
    ("traffic.shift", Category::ActionId, Marking::Core),
    ("emergency.leave", Category::ActionId, Marking::Core),
    (
        "repository.inspect",
        Category::ActionId,
        Marking::SoftwareChange,
    ),
    (
        "repository.merge",
        Category::ActionId,
        Marking::SoftwareChange,
    ),
    ("tests.run", Category::ActionId, Marking::SoftwareChange),
    ("restore_service", Category::ObligationId, Marking::Core),
    ("accepted", Category::OutcomeId, Marking::Core),
];

/// The vocabulary opens with exactly the story's table, in its order; later stories append their
/// own terms after it (`story:vocabulary-yaml-source`) and assert those themselves.
#[test]
fn adversary_vocabulary_is_exactly_the_story_table() {
    let terms = vocabulary::terms();
    assert!(terms.len() >= 34, "{} terms", terms.len());
    let first = &terms[..34];
    assert_eq!(
        first
            .iter()
            .map(|t| (t.id, t.category, t.marking))
            .collect::<Vec<_>>(),
        STORY_TABLE.to_vec(),
        "the first 34 terms are not the story table in its order"
    );
    let declared: BTreeSet<(&str, String, String)> = first
        .iter()
        .map(|t| (t.id, format!("{:?}", t.category), t.marking.to_string()))
        .collect();
    let table: BTreeSet<(&str, String, String)> = STORY_TABLE
        .iter()
        .map(|(id, c, m)| (*id, format!("{c:?}"), m.to_string()))
        .collect();
    let extra: Vec<_> = declared.difference(&table).collect();
    let missing: Vec<_> = table.difference(&declared).collect();
    assert!(
        extra.is_empty() && missing.is_empty(),
        "extra {extra:?}, missing {missing:?}"
    );
}

/// Lookup is by exact string. Kills prefix, substring, case-folding and trimming mutants of
/// `lookup`, each of which the unit's suite leaves green.
#[test]
fn adversary_lookup_refuses_partial_and_variant_spellings() {
    for id in [
        "",
        "tests",
        "release.",
        "implementation.",
        "repository",
        "Service.Healthy",
        "SERVICE.HEALTHY",
        " service.healthy",
        "service.healthy ",
        "service.healthy.extra",
        "healthy",
        "accept",
    ] {
        let refusal = vocabulary::lookup(id)
            .map(|t| t.id)
            .expect_err(&format!("`{id}` must be refused"));
        assert_eq!(refusal.term, id);
        assert!(refusal.to_string().contains(&format!("`{id}`")));
    }
    // The exact spelling still resolves to itself.
    assert_eq!(vocabulary::lookup("release").unwrap().id, "release");
    assert_eq!(vocabulary::lookup("tests.pass").unwrap().id, "tests.pass");
}

/// Expectation 3 read without the unit's header/value rule: every dotted id anywhere in an
/// example's text blocks resolves to exactly one claim or action entry. An id the unit's
/// extraction rule skips (a line under another header, a status outside its closed list)
/// is still checked here.
#[test]
fn adversary_every_dotted_id_in_example_text_blocks_resolves() {
    for example in ["software-change.md", "incident-response.md"] {
        let ids = dotted_ids(example);
        assert!(!ids.is_empty(), "{example}: no dotted id found");
        for id in &ids {
            let n = vocabulary::terms().iter().filter(|t| t.id == id).count();
            assert_eq!(n, 1, "{example}: `{id}` names {n} entries");
            let term = vocabulary::lookup(id).unwrap();
            assert!(
                matches!(term.category, Category::ClaimId | Category::ActionId),
                "{example}: `{id}` is {:?}",
                term.category
            );
        }
    }
}

fn dotted_ids(file: &str) -> BTreeSet<String> {
    let path = Path::new(
        &std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is unset: run this test through cargo"),
    )
    .join("../../docs/examples")
    .join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    let mut out = BTreeSet::new();
    let mut in_block = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if !in_block {
            continue;
        }
        for tok in t.split(|c: char| !(c.is_ascii_lowercase() || c == '_' || c == '.')) {
            let tok = tok.trim_matches('.');
            if let Some((a, b)) = tok.split_once('.')
                && !a.is_empty()
                && !b.is_empty()
            {
                out.insert(tok.to_owned());
            }
        }
    }
    out
}
