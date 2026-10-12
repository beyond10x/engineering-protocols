//! Acceptance for `story:els-vocabulary`: the first-domain engineering terms.

use std::path::Path;

use canon_engineering::vocabulary::{self, Category, Marking};

/// The story's *Term set declared here* table, copied: (term, category, marked `software.change`),
/// without `repository.edit`, which `story:software-change-profiles` split into `tests.write` and
/// `implementation.edit`.
const TABLE: &[(&str, Category, bool)] = &[
    // artifact kinds
    ("intent", Category::ArtifactKind, false),
    ("system_specification", Category::ArtifactKind, false),
    ("plan", Category::ArtifactKind, false),
    ("release", Category::ArtifactKind, false),
    ("deployment", Category::ArtifactKind, false),
    ("service", Category::ArtifactKind, false),
    ("implementation", Category::ArtifactKind, true),
    // evidence kinds
    ("operational_observation", Category::EvidenceKind, false),
    ("objective_observation", Category::EvidenceKind, false),
    ("impact_assessment", Category::EvidenceKind, false),
    ("cause_analysis", Category::EvidenceKind, false),
    ("test_result", Category::EvidenceKind, true),
    ("code_review", Category::EvidenceKind, true),
    ("build_provenance", Category::EvidenceKind, true),
    // claim ids
    ("release.proven", Category::ClaimId, false),
    ("deployment.healthy", Category::ClaimId, false),
    ("objective.realized", Category::ClaimId, false),
    ("impact.bounded", Category::ClaimId, false),
    ("service.healthy", Category::ClaimId, false),
    ("cause.identified", Category::ClaimId, false),
    ("tests.pass", Category::ClaimId, true),
    ("implementation.reviewed", Category::ClaimId, true),
    ("implementation.verified", Category::ClaimId, true),
    // action ids
    ("metrics.inspect", Category::ActionId, false),
    ("logs.search", Category::ActionId, false),
    ("release.inspect", Category::ActionId, false),
    ("release.rollback", Category::ActionId, false),
    ("traffic.shift", Category::ActionId, false),
    ("emergency.leave", Category::ActionId, false),
    ("repository.inspect", Category::ActionId, true),
    ("repository.merge", Category::ActionId, true),
    ("tests.run", Category::ActionId, true),
    // obligation ids
    ("restore_service", Category::ObligationId, false),
    // outcome ids
    ("accepted", Category::OutcomeId, false),
];

#[test]
fn vocabulary_declares_first_domain_terms() {
    assert_eq!(
        TABLE.len(),
        34,
        "the copied table has the story's 35 terms but one"
    );
    assert_eq!(
        TABLE.iter().filter(|(_, _, sc)| *sc).count(),
        10,
        "the copied table marks 10 terms software.change"
    );

    // 1. Each table term resolves to exactly one entry, in the table's category.
    for (id, category, _) in TABLE {
        assert_eq!(
            entries_named(id),
            1,
            "`{id}` must name exactly one vocabulary entry"
        );
        let term = vocabulary::lookup(id).unwrap_or_else(|e| panic!("`{id}`: {e}"));
        assert_eq!(term.id, *id);
        assert_eq!(term.category, *category, "`{id}` is in the wrong category");
    }

    // 2. The software.change column carries that marking; the rest are core.
    for (id, _, software_change) in TABLE {
        let term = vocabulary::lookup(id).unwrap();
        let expected = if *software_change {
            Marking::SoftwareChange
        } else {
            Marking::Core
        };
        assert_eq!(term.marking, expected, "`{id}` carries the wrong marking");
    }
    assert_eq!(Marking::SoftwareChange.as_str(), "software.change");
    assert_eq!(Marking::Core.as_str(), "core");

    // 3. Every claim id and action id in the examples' text blocks resolves to one entry.
    for example in ["software-change.md", "incident-response.md"] {
        let refs = example_ids(example);
        assert!(
            refs.iter().any(|(_, c)| *c == Category::ClaimId),
            "{example}: no claim id read out of its text blocks"
        );
        assert!(
            refs.iter().any(|(_, c)| *c == Category::ActionId),
            "{example}: no action id read out of its text blocks"
        );
        for (id, category) in &refs {
            assert_eq!(
                entries_named(id),
                1,
                "{example}: `{id}` must name exactly one vocabulary entry"
            );
            let term = vocabulary::lookup(id).unwrap_or_else(|e| panic!("{example}: {e}"));
            assert_eq!(
                term.category, *category,
                "{example}: `{id}` is used as {category:?} but declared as {:?}",
                term.category
            );
        }
    }

    // 4. The old incident spelling and an undeclared term are refused by name.
    for id in ["service_healthy", "repository.force_push"] {
        let refusal = vocabulary::lookup(id).expect_err(id);
        assert_eq!(entries_named(id), 0);
        assert!(
            refusal.to_string().contains(id),
            "refusal `{refusal}` does not name `{id}`"
        );
    }
}

#[test]
fn vocabulary_spells_ids_by_category_and_maps_to_canon_identifiers() {
    for term in vocabulary::terms() {
        assert_eq!(entries_named(term.id), 1, "`{}` is declared twice", term.id);
        // An obligation id is either spelling (`restore_service`, `tests.required`); every other
        // category is one of the two.
        if term.category == Category::ObligationId {
            assert!(
                is_token(term.id) || is_dotted(term.id),
                "`{}` is not spelled as an obligation id",
                term.id
            );
        } else {
            let dotted = matches!(term.category, Category::ClaimId | Category::ActionId);
            assert_eq!(
                is_dotted(term.id),
                dotted,
                "`{}` is not spelled as a {:?}",
                term.id,
                term.category
            );
            if !dotted {
                assert!(
                    is_token(term.id),
                    "`{}` is not one snake_case token",
                    term.id
                );
            }
        }
        assert_eq!(
            term.claim_id().map(|c| c.as_str().to_owned()),
            (term.category == Category::ClaimId).then(|| term.id.to_owned())
        );
        assert_eq!(
            term.action_id().map(|a| a.as_str().to_owned()),
            (term.category == Category::ActionId).then(|| term.id.to_owned())
        );
        assert!(!term.meaning.is_empty(), "`{}` has no meaning", term.id);
    }
}

fn entries_named(id: &str) -> usize {
    vocabulary::terms().iter().filter(|t| t.id == id).count()
}

fn is_token(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('_')
        && !s.ends_with('_')
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

fn is_dotted(s: &str) -> bool {
    matches!(s.split_once('.'), Some((a, b)) if is_token(a) && is_token(b))
}

/// Reads claim and action ids out of an example's ```text blocks.
///
/// A line names a claim when it sits under a `Claims:` header or assigns a truth value
/// (`TRUE`, `FALSE`, `UNKNOWN`); it names an action when it sits under an `Actions:` header
/// or assigns an action status (`admissible`, `approval_required`, `blocked`). The id is the
/// line's leading token. Other lines (evidence prose, obligations, case headers) are skipped.
fn example_ids(file: &str) -> Vec<(String, Category)> {
    let path = Path::new(
        &std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is unset: run this test through cargo"),
    )
    .join("../../docs/examples")
    .join(file);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));

    let mut ids = Vec::new();
    let mut in_block = false;
    let mut section: Option<&str> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_block = !in_block && trimmed == "```text";
            section = None;
            continue;
        }
        if !in_block {
            continue;
        }
        if trimmed.is_empty() {
            section = None;
            continue;
        }
        if let Some(header) = trimmed.strip_suffix(':') {
            section = Some(header);
            continue;
        }
        let token: String = trimmed
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '=' && *c != '[')
            .collect();
        let value = trimmed.split_once('=').map(|(_, v)| v.trim());
        let category = match (section, value) {
            (Some("Claims"), _) | (_, Some("TRUE" | "FALSE" | "UNKNOWN")) => Category::ClaimId,
            (Some("Actions"), _) | (_, Some("admissible" | "approval_required" | "blocked")) => {
                Category::ActionId
            }
            _ => continue,
        };
        ids.push((token, category));
    }
    assert!(!in_block, "{file}: unterminated code block");
    ids
}
